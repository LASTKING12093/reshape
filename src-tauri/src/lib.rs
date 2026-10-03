mod commands;
pub mod conversion;
pub mod engines;
pub mod errors;
pub mod estimation;
pub mod formats;
pub mod models;
pub mod output;
pub mod planning;
pub mod presets;
pub mod previews;
pub mod probing;
pub mod process;
pub mod queue;
pub mod smoke;
use tauri::Manager;
pub fn run() {
    #[cfg(windows)]
    unsafe {
        let id: Vec<u16> = "com.reshape.desktop"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                // Supply the current mark to both window sizes, independent of Explorer's
                // cached executable icon and legacy Forma taskbar grouping.
                window.set_icon(tauri::image::Image::new_owned(
                    include_bytes!("../icons/window-icon.rgba").to_vec(),
                    256,
                    256,
                ))?;
            }
            let root = if cfg!(debug_assertions) {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries")
            } else {
                app.path().resource_dir()?.join("binaries")
            };
            let state = queue::Queue::new(root, app.path().app_data_dir()?);
            app.manage(state);
            // Explicit QA-only launch arguments alter this window, never Windows display settings.
            let args: Vec<String> = std::env::args().collect();
            if let Some(i) = args.iter().position(|s| s == "--qa-scale") {
                if let Some(scale) = args
                    .get(i + 1)
                    .and_then(|s| s.parse::<f64>().ok())
                    .filter(|s| [1., 1.25, 1.5, 2.].contains(s))
                {
                    if let Some(window) = app.get_webview_window("main") {
                        window.set_zoom(scale)?;
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::open_licenses,
            commands::startup_paths,
            commands::inspect,
            commands::enqueue,
            commands::control,
            commands::save_settings,
            commands::check_engines,
            commands::open_result,
            commands::clear_history,
            commands::clear_temporary,
            commands::transformation_plan,
            commands::refine_estimate,
            commands::preview,
            commands::reorder,
            commands::save_presets
        ])
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar Reshape");
}
