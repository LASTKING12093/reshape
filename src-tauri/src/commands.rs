use crate::{
    models::*,
    probing,
    queue::{Queue, Snapshot},
};
use std::path::PathBuf;
use tauri::State;
#[tauri::command]
pub fn open_licenses(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let legal = app.path().resource_dir().map_err(|e| e.to_string())?.join("legal");
    if !legal.is_dir() { return Err("License folder is missing. Reinstall Reshape.".into()); }
    let windows = std::env::var_os("WINDIR").ok_or("Windows directory unavailable")?;
    std::process::Command::new(PathBuf::from(windows).join("explorer.exe"))
        .arg(legal).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn startup_paths() -> Vec<String> {
    std::env::args()
        .skip_while(|s| s != "--open")
        .skip(1)
        .filter(|s| PathBuf::from(s).exists())
        .take(1000)
        .collect()
}
#[tauri::command]
pub fn transformation_plan(
    input: InputFile,
    options: ConversionOptions,
    state: State<Queue>,
) -> Result<crate::planning::Plan, String> {
    crate::planning::plan(&input, &options, &state.inner.lock().unwrap().hardware)
}
#[tauri::command]
pub async fn refine_estimate(
    input: InputFile,
    options: ConversionOptions,
    state: State<'_, Queue>,
) -> Result<crate::planning::Estimate, String> {
    let engine = state.manager.clone();
    let root = state.data.join("temp");
    tauri::async_runtime::spawn_blocking(move || {
        crate::estimation::refine(&input, &options, &engine, &root)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn preview(
    input: InputFile,
    position: f64,
    state: State<'_, Queue>,
) -> Result<String, String> {
    let engine = state.manager.clone();
    let root = state.data.join("temp");
    tauri::async_runtime::spawn_blocking(move || {
        crate::previews::render(&input, position, &engine, &root)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn reorder(ids: Vec<String>, state: State<Queue>) -> Result<(), String> {
    state.reorder(ids)
}
#[tauri::command]
pub fn save_presets(
    presets: Vec<crate::presets::Preset>,
    state: State<Queue>,
) -> Result<(), String> {
    state.inner.lock().unwrap().presets = crate::presets::sanitize(presets)?;
    state.persist();
    Ok(())
}
#[tauri::command]
pub fn snapshot(state: State<Queue>) -> Snapshot {
    state.snapshot()
}
#[derive(serde::Serialize)]
pub struct Inspection {
    files: Vec<InputFile>,
    errors: Vec<String>,
    truncated: bool,
}
#[tauri::command]
pub async fn inspect(
    paths: Vec<String>,
    recursive: bool,
    state: State<'_, Queue>,
) -> Result<Inspection, String> {
    let engines = state.manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut files = Vec::new();
        let mut errors = Vec::new();
        let mut candidates = Vec::new();
        let mut truncated = false;
        for path in paths {
            let p = PathBuf::from(&path);
            if p.is_dir() {
                for e in walkdir::WalkDir::new(&p)
                    .follow_links(false)
                    .max_depth(if recursive { usize::MAX } else { 1 })
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    candidates.push(e.path().to_owned());
                    if candidates.len() >= 1000 {
                        truncated = true;
                        break;
                    }
                }
            } else {
                candidates.push(p)
            }
            if candidates.len() >= 1000 {
                break;
            }
        }
        candidates.sort();
        candidates.dedup();
        for path in candidates {
            match probing::inspect(&path, &engines) {
                Ok(f) => files.push(f),
                Err(e) => errors.push(format!(
                    "{}: {e}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )),
            }
        }
        Inspection {
            files,
            errors,
            truncated,
        }
    })
    .await
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn enqueue(
    input: InputFile,
    options: ConversionOptions,
    state: State<'_, Queue>,
) -> Result<String, String> {
    let q = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let fresh = probing::inspect(&PathBuf::from(input.path), &q.manager)?;
        q.add(fresh, options)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn control(action: String, id: Option<String>, state: State<Queue>) -> Result<(), String> {
    state.control(&action, id.as_deref())
}
#[tauri::command]
pub fn save_settings(settings: Settings, state: State<Queue>) -> Result<(), String> {
    if !["system", "dark", "light"].contains(&settings.theme.as_str())
        || !["pt", "en"].contains(&settings.language.as_str())
    {
        return Err("Invalid settings".into());
    }
    let mut settings = settings;
    settings.concurrency = settings.concurrency.clamp(1, 4);
    state.inner.lock().unwrap().settings = settings;
    state.persist();
    state.schedule();
    Ok(())
}
#[tauri::command]
pub async fn check_engines(state: State<'_, Queue>) -> Result<Vec<EngineCapability>, String> {
    let q = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let engines = q.manager.health();
        q.inner.lock().unwrap().engines = engines.clone();
        engines
    })
    .await
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn clear_history(state: State<Queue>) {
    state.inner.lock().unwrap().history.clear();
    state.persist()
}
#[tauri::command]
pub fn clear_temporary(state: State<Queue>) -> Result<usize, String> {
    let s = state.inner.lock().unwrap();
    if !s.running.is_empty() {
        return Err("Wait for active tasks before clearing temporary files.".into());
    }
    let root = if s.settings.temp_dir.is_empty() {
        state.data.join("temp")
    } else {
        PathBuf::from(&s.settings.temp_dir)
    };
    let mut count = 0;
    if !root.exists() {
        return Ok(0);
    }
    for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        if !e.file_type().map_err(|e| e.to_string())?.is_dir()
            || !e.file_name().to_string_lossy().starts_with("forma-job-")
        {
            continue;
        }
        let marker = e.path().join(".forma-job.lock");
        if let Ok(file) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(marker)
        {
            if fs2::FileExt::try_lock_exclusive(&file).is_ok() {
                drop(file);
                std::fs::remove_dir_all(e.path()).map_err(|e| e.to_string())?;
                count += 1
            }
        }
    }
    Ok(count)
}
#[tauri::command]
pub fn open_result(id: String, folder: bool, state: State<Queue>) -> Result<(), String> {
    let s = state.inner.lock().unwrap();
    let j = s
        .jobs
        .iter()
        .chain(s.history.iter())
        .find(|j| j.id == id)
        .ok_or("Result not found")?;
    let p = PathBuf::from(j.output.as_ref().ok_or("No output")?);
    if !p.exists() {
        return Err("The output was moved or deleted.".into());
    }
    let path = if folder && p.is_file() {
        p.parent().unwrap().to_path_buf()
    } else {
        p
    };
    // Open folders in Explorer; files via their registered Windows association.
    if path.is_dir() {
        let system = std::env::var_os("WINDIR").ok_or("Windows directory unavailable")?;
        std::process::Command::new(PathBuf::from(system).join("explorer.exe"))
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
            let result = unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    verb.as_ptr(),
                    wide.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1,
                )
            };
            if result as isize <= 32 {
                return Err("Windows could not open this file.".into());
            }
        }
    }
    Ok(())
}
