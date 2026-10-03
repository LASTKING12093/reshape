use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct Cancel(pub Arc<AtomicBool>);
impl Cancel {
    pub fn stop(&self) {
        self.0.store(true, Ordering::SeqCst)
    }
    pub fn check(&self) -> Result<(), String> {
        if self.0.load(Ordering::SeqCst) {
            Err("Conversion was cancelled.".into())
        } else {
            Ok(())
        }
    }
}
#[cfg(windows)]
struct Job(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
#[cfg(windows)]
fn protect(child: &std::process::Child) -> Result<Job, String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::*;
    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            return Err("Cannot create process isolation.".into());
        }
        let job = Job(handle);
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            std::mem::size_of_val(&info) as u32,
        ) == 0
            || AssignProcessToJobObject(handle, child.as_raw_handle() as _) == 0
        {
            return Err("Cannot isolate converter process.".into());
        }
        Ok(job)
    }
}
pub fn run(
    exe: &Path,
    args: &[String],
    cwd: &Path,
    cancel: &Cancel,
    duration: Option<f64>,
    progress: impl FnMut(Option<f64>, Option<String>),
) -> Result<String, String> {
    run_limited(exe, args, cwd, cancel, duration, 86400, progress)
}
pub fn run_limited(
    exe: &Path,
    args: &[String],
    cwd: &Path,
    cancel: &Cancel,
    duration: Option<f64>,
    timeout: u64,
    mut progress: impl FnMut(Option<f64>, Option<String>),
) -> Result<String, String> {
    cancel.check()?;
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if exe.file_name().is_some_and(|s| s == "magick.exe") {
        if let Some(home) = exe.parent() {
            cmd.env("MAGICK_HOME", home)
                .env("MAGICK_CONFIGURE_PATH", home)
                .env("MAGICK_TEMPORARY_PATH", cwd);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000004);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Engine could not start: {e}"))?;
    #[cfg(windows)]
    let job = match protect(&child) {
        Ok(j) => j,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    // Start suspended: children cannot escape the Job Object before assignment.
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        #[link(name = "ntdll")]
        extern "system" {
            fn NtResumeProcess(process: windows_sys::Win32::Foundation::HANDLE) -> i32;
        }
        if unsafe { NtResumeProcess(child.as_raw_handle() as _) } < 0 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Could not resume converter process.".into());
        }
    }
    let out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let stdout = thread::spawn(move || {
        let mut collected = String::new();
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if line.starts_with("out_time_us=") || line.starts_with("speed=") {
                let _ = tx.send(line.clone());
            }
            if collected.len() < 262144 {
                collected.push_str(&line);
                collected.push('\n')
            } else if !collected.ends_with("[OUTPUT_TRUNCATED]") {
                collected.push_str("[OUTPUT_TRUNCATED]");
            }
        }
        collected
    });
    let stderr = thread::spawn(move || {
        let mut r = BufReader::new(err);
        let mut buf = [0; 4096];
        let mut tail = Vec::new();
        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }
            tail.extend_from_slice(&buf[..n]);
            if tail.len() > 32768 {
                tail.drain(..tail.len() - 32768);
            }
        }
        String::from_utf8_lossy(&tail).into_owned()
    });
    let start = Instant::now();
    let mut pct = None;
    let mut speed = None;
    let status = loop {
        for line in rx.try_iter() {
            if let Some(n) = line
                .strip_prefix("out_time_us=")
                .and_then(|v| v.parse::<f64>().ok())
            {
                pct = duration
                    .filter(|d| *d > 0.)
                    .map(|d| (n / 1_000_000. / d * 100.).clamp(0., 99.5))
            }
            if let Some(v) = line.strip_prefix("speed=") {
                speed = Some(v.to_string())
            }
        }
        progress(pct, speed.clone());
        if cancel.check().is_err() || start.elapsed() > Duration::from_secs(timeout) {
            #[cfg(windows)]
            drop(job);
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout.join();
            let _ = stderr.join();
            return Err("Conversion was cancelled.".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(e.to_string()),
        }
    };
    #[cfg(windows)]
    drop(job);
    let output = stdout.join().unwrap_or_default();
    let errors = stderr.join().unwrap_or_default();
    if !status.success() {
        let safe_args = args
            .iter()
            .map(|a| {
                if a.starts_with("-p")
                    && !a.starts_with("-progress")
                    && !a.starts_with("-preset")
                    && !a.starts_with("-pix_fmt")
                    && !a.starts_with("-pass")
                {
                    "[redacted]".to_string()
                } else if a.contains('\\') || a.contains('/') || a.starts_with("-env:") {
                    "[path]".to_string()
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>();
        return Err(format!(
            "Engine {} failed (exit {}). Command arguments: {:?}. {}",
            exe.file_name().unwrap_or_default().to_string_lossy(),
            status.code().unwrap_or(-1),
            safe_args,
            errors
        ));
    }
    Ok(output)
}
pub fn capture(exe: &Path, args: &[&str], cwd: &Path) -> Result<String, String> {
    run(
        exe,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        cwd,
        &Cancel::default(),
        None,
        |_, _| {},
    )
}
