#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "--smoke-test") {
        let Some(path) = args.get(2) else {
            std::process::exit(2)
        };
        if let Err(e) = forma_lib::smoke::run(std::path::Path::new(path)) {
            let _ = std::fs::write(std::path::Path::new(path).join("smoke-error.txt"), e);
            std::process::exit(1)
        }
        return;
    }
    forma_lib::run();
}
