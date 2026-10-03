//! Publish a complete file with create-new semantics; never expose a partial final file.
use crate::process::Cancel;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub fn publish_file(
    src: &Path,
    dir: &Path,
    stem: &str,
    ext: &str,
    cancel: &Cancel,
) -> Result<PathBuf, String> {
    // Same-volume NTFS publication is a single, atomic directory-entry operation.
    for n in 0..10000 {
        cancel.check()?;
        let suffix = if n == 0 {
            String::new()
        } else {
            format!(" ({n})")
        };
        let dest = dir.join(format!(
            "{stem}{suffix}{}",
            if ext.is_empty() {
                String::new()
            } else {
                format!(".{ext}")
            }
        ));
        match fs::hard_link(src, &dest) {
            Ok(()) => return Ok(dest),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => break, // Cross-volume or filesystem without hard links: staged stream copy.
        }
    }
    let mut temp = tempfile::Builder::new()
        .prefix(".reshape-output-")
        .tempfile_in(dir)
        .map_err(|e| e.to_string())?;
    let mut input = fs::File::open(src).map_err(|e| e.to_string())?;
    let mut buffer = [0; 65536];
    loop {
        cancel.check()?;
        let n = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        temp.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
    }
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    for n in 0..10000 {
        cancel.check()?;
        let suffix = if n == 0 {
            String::new()
        } else {
            format!(" ({n})")
        };
        let dest = dir.join(format!(
            "{stem}{suffix}{}",
            if ext.is_empty() {
                String::new()
            } else {
                format!(".{ext}")
            }
        ));
        match temp.persist_noclobber(&dest) {
            Ok(_) => return Ok(dest),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => temp = e.file,
            Err(e) => return Err(e.error.to_string()),
        }
    }
    Err("Too many output filename collisions".into())
}
