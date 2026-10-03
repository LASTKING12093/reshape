use crate::{conversion, engines::EngineManager, formats, models::*, process::Cancel};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
pub struct Inner {
    pub jobs: Vec<ConversionJob>,
    pub history: Vec<ConversionJob>,
    pub settings: Settings,
    pub paused: bool,
    pub running: HashMap<String, Cancel>,
    pub engines: Vec<EngineCapability>,
    pub hardware: Vec<String>,
    pub presets: Vec<crate::presets::Preset>,
}
#[derive(Clone)]
pub struct Queue {
    pub inner: Arc<Mutex<Inner>>,
    pub manager: EngineManager,
    pub data: PathBuf,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub presets: Vec<crate::presets::Preset>,
    pub jobs: Vec<ConversionJob>,
    pub history: Vec<ConversionJob>,
    pub settings: Settings,
    pub paused: bool,
    pub engines: Vec<EngineCapability>,
    pub registry: Vec<formats::FormatCapability>,
    pub hardware: Vec<String>,
}
impl Queue {
    pub fn new(root: PathBuf, data: PathBuf) -> Self {
        let _ = fs::create_dir_all(&data);
        let settings = fs::read(data.join("settings.json"))
            .ok()
            .and_then(|s| serde_json::from_slice(&s).ok())
            .unwrap_or_default();
        let history = fs::read(data.join("history.json"))
            .ok()
            .and_then(|s| serde_json::from_slice(&s).ok())
            .unwrap_or_default();
        let presets = fs::read(data.join("presets.json"))
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();
        let mut jobs: Vec<ConversionJob> = fs::read(data.join("pending.json"))
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();
        for j in &mut jobs {
            j.status = "pending".into();
            j.progress = None;
        }
        let recovered = !jobs.is_empty();
        let q = Self {
            inner: Arc::new(Mutex::new(Inner {
                jobs,
                presets,
                history,
                settings,
                paused: recovered,
                running: HashMap::new(),
                engines: vec![],
                hardware: vec![],
            })),
            manager: EngineManager::new(root),
            data,
        };
        let clone = q.clone();
        thread::spawn(move || {
            let engines = clone.manager.health();
            clone.inner.lock().unwrap().engines = engines;
            let hardware = clone.manager.hardware();
            clone.inner.lock().unwrap().hardware = hardware;
        });
        q
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.inner.lock().unwrap();
        Snapshot {
            presets: s.presets.clone(),
            jobs: s.jobs.clone(),
            history: s.history.clone(),
            settings: s.settings.clone(),
            paused: s.paused,
            engines: s.engines.clone(),
            registry: formats::registry(),
            hardware: s.hardware.clone(),
        }
    }
    pub fn add(&self, input: InputFile, options: ConversionOptions) -> Result<String, String> {
        let options =
            crate::planning::plan(&input, &options, &self.inner.lock().unwrap().hardware)?.options;
        let id = uuid::Uuid::new_v4().to_string();
        self.inner.lock().unwrap().jobs.push(ConversionJob {
            id: id.clone(),
            input,
            options,
            status: "pending".into(),
            progress: None,
            speed: None,
            elapsed: 0.,
            remaining: None,
            output: None,
            output_size: None,
            error: None,
            diagnostic: None,
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        self.persist();
        self.schedule();
        Ok(id)
    }
    pub fn schedule(&self) {
        let mut s = self.inner.lock().unwrap();
        if s.paused {
            return;
        }
        let cpu = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        let limit = s.settings.concurrency.clamp(1, 4).min(cpu.max(1));
        while s.running.len() < limit {
            let video_active = s.jobs.iter().any(|j| {
                ["processing", "cancelling"].contains(&j.status.as_str())
                    && j.input.category == "video"
            });
            let Some(index) = s.jobs.iter().position(|j| {
                j.status == "pending" && !(video_active && j.input.category == "video")
            }) else {
                break;
            };
            let cancel = Cancel::default();
            s.jobs[index].status = "processing".into();
            let job = s.jobs[index].clone();
            s.running.insert(job.id.clone(), cancel.clone());
            let me = self.clone();
            let temp = if s.settings.temp_dir.is_empty() {
                self.data.join("temp")
            } else {
                PathBuf::from(&s.settings.temp_dir)
            };
            thread::spawn(move || {
                let start = Instant::now();
                let result = conversion::execute(
                    &job.input,
                    &job.options,
                    &me.manager,
                    &temp,
                    &cancel,
                    |pct, speed| {
                        let mut state = me.inner.lock().unwrap();
                        if let Some(j) = state.jobs.iter_mut().find(|j| j.id == job.id) {
                            j.progress = pct;
                            j.speed = speed;
                            j.elapsed = start.elapsed().as_secs_f64();
                            j.remaining =
                                pct.filter(|v| *v > 1.).map(|v| j.elapsed * (100. - v) / v);
                        }
                    },
                );
                {
                    let mut state = me.inner.lock().unwrap();
                    let history_enabled = state.settings.history_enabled;
                    let mut completed = None;
                    if let Some(j) = state.jobs.iter_mut().find(|j| j.id == job.id) {
                        j.elapsed = start.elapsed().as_secs_f64();
                        j.remaining = None;
                        match result {
                            Ok((path, size)) => {
                                j.status = if job.options.mode != "convert"
                                    && path == PathBuf::from(&job.input.path)
                                {
                                    "unchanged"
                                } else {
                                    "completed"
                                }
                                .into();
                                j.progress = Some(100.);
                                j.output = Some(path.to_string_lossy().into());
                                j.output_size = Some(size)
                            }
                            Err(message) => {
                                j.status = if cancel.check().is_err() {
                                    "cancelled"
                                } else {
                                    "failed"
                                }
                                .into();
                                let safe = message
                                    .replace(&job.input.path, "[input]")
                                    .replace(&job.options.password_if_set(), "[redacted]")
                                    .replace(&me.data.to_string_lossy().to_string(), "[app-data]");
                                j.error = Some(crate::errors::friendly(&safe));
                                j.diagnostic = Some(safe)
                            }
                        }
                        let mut h = j.clone();
                        h.options.password.clear();
                        // Diagnostic events intentionally omit names, paths, arguments and passwords.
                        {
                            use std::io::Write;
                            let logs = me.data.join("logs");
                            let _ = fs::create_dir_all(&logs);
                            let path = logs.join("events.jsonl");
                            if fs::metadata(&path).is_ok_and(|m| m.len() > 1_048_576) {
                                let _ = fs::rename(&path, logs.join("previous.jsonl"));
                            }
                            if let Ok(mut file) =
                                fs::OpenOptions::new().create(true).append(true).open(path)
                            {
                                let event = serde_json::json!({"id":j.id,"status":j.status,"inputFormat":j.input.format,"outputFormat":j.options.format,"seconds":j.elapsed,"inputBytes":j.input.size,"outputBytes":j.output_size});
                                let _ = writeln!(file, "{event}");
                            }
                        }
                        completed = Some(h)
                    }
                    state.running.remove(&job.id);
                    if history_enabled {
                        if let Some(h) = completed {
                            state.history.insert(0, h);
                            state.history.truncate(1000);
                        }
                    }
                }
                me.persist();
                me.schedule();
            });
        }
    }
    pub fn control(&self, action: &str, id: Option<&str>) -> Result<(), String> {
        let mut s = self.inner.lock().unwrap();
        match action {
            "pause" => s.paused = true,
            "resume" => s.paused = false,
            "cancelAll" => {
                for c in s.running.values() {
                    c.stop();
                }
                for j in &mut s.jobs {
                    if j.status == "pending" {
                        j.status = "cancelled".into();
                    } else if j.status == "processing" {
                        j.status = "cancelling".into();
                    }
                }
            }
            "retryFailed" => {
                for j in &mut s.jobs {
                    if j.status == "failed" {
                        j.status = "pending".into();
                        j.progress = None;
                        j.error = None;
                        j.diagnostic = None;
                        j.elapsed = 0.;
                        j.output = None;
                        j.output_size = None;
                    }
                }
            }
            "cancel" => {
                let id = id.ok_or("Missing job")?;
                if let Some(c) = s.running.get(id) {
                    c.stop();
                    if let Some(j) = s.jobs.iter_mut().find(|j| j.id == id) {
                        j.status = "cancelling".into();
                    }
                } else if let Some(j) = s
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id && j.status == "pending")
                {
                    j.status = "cancelled".into()
                }
            }
            "retry" => {
                let j = s
                    .jobs
                    .iter_mut()
                    .find(|j| Some(j.id.as_str()) == id)
                    .ok_or("Job not found")?;
                if ["processing", "pending", "cancelling"].contains(&j.status.as_str()) {
                    return Err("Job is still active".into());
                }
                j.status = "pending".into();
                j.error = None;
                j.diagnostic = None;
                j.progress = None;
                j.elapsed = 0.;
                j.output = None;
                j.output_size = None;
                j.remaining = None;
            }
            "remove" => s.jobs.retain(|j| {
                Some(j.id.as_str()) != id
                    || ["processing", "cancelling"].contains(&j.status.as_str())
            }),
            "clear" => s
                .jobs
                .retain(|j| ["processing", "pending", "cancelling"].contains(&j.status.as_str())),
            _ => return Err("Unknown queue action".into()),
        }
        drop(s);
        self.persist();
        self.schedule();
        Ok(())
    }
    pub fn persist(&self) {
        let s = self.inner.lock().unwrap();
        let mut pending: Vec<_> = s
            .jobs
            .iter()
            .filter(|j| ["pending", "processing", "cancelling"].contains(&j.status.as_str()))
            .cloned()
            .collect();
        for j in &mut pending {
            j.options.password.clear();
        }
        for (name, value) in [
            ("settings.json", serde_json::to_vec_pretty(&s.settings)),
            ("history.json", serde_json::to_vec_pretty(&s.history)),
            ("pending.json", serde_json::to_vec_pretty(&pending)),
            ("presets.json", serde_json::to_vec_pretty(&s.presets)),
        ] {
            if let Ok(bytes) = value {
                let tmp = self.data.join(format!("{name}.tmp"));
                if fs::write(&tmp, bytes).is_ok() {
                    let _ = fs::rename(tmp, self.data.join(name));
                }
            }
        }
    }
    pub fn reorder(&self, ids: Vec<String>) -> Result<(), String> {
        let mut s = self.inner.lock().unwrap();
        let pending: Vec<_> = s
            .jobs
            .iter()
            .filter(|j| j.status == "pending")
            .cloned()
            .collect();
        if ids.len() != pending.len()
            || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
            || ids.iter().any(|id| !pending.iter().any(|j| &j.id == id))
        {
            return Err("A fila mudou. Tente novamente.".into());
        }
        let mut ordered = ids
            .iter()
            .map(|id| pending.iter().find(|j| &j.id == id).unwrap().clone());
        for j in &mut s.jobs {
            if j.status == "pending" {
                *j = ordered.next().unwrap();
            }
        }
        drop(s);
        self.persist();
        Ok(())
    }
}
impl ConversionOptions {
    fn password_if_set(&self) -> String {
        if self.password.is_empty() {
            "\0UNUSED\0".into()
        } else {
            self.password.clone()
        }
    }
}
