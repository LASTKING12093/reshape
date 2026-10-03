use crate::{engines::EngineManager, formats, models::InputFile, process};
use serde_json::Value;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};
pub fn inspect(path: &Path, engines: &EngineManager) -> Result<InputFile, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let m = fs::metadata(&path).map_err(|e| e.to_string())?;
    if !m.is_file() || m.len() == 0 {
        return Err("The source file is empty or invalid.".into());
    }
    let mut f = File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = [0; 8192];
    let n = f.read(&mut buf).map_err(|e| e.to_string())?;
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mut format = infer::get(&buf[..n])
        .map(|v| v.extension().to_string())
        .unwrap_or(ext.clone());
    if ["docx", "xlsx", "pptx", "odt", "ods", "odp"].contains(&ext.as_str())
        && !buf[..n].starts_with(b"PK\x03\x04")
    {
        return Err("This document is damaged or its extension does not match its contents.".into());
    }
    if format == "jpeg" {
        format = "jpg".into()
    }
    if format == "tif" {
        format = "tiff".into()
    }
    // Both containers use EBML; magic-only detection can label Matroska as WebM.
    if format == "webm" && buf[..n].windows(8).any(|v| v == b"matroska") {
        format = "mkv".into();
    }
    if format == "zip" {
        let mut z = zip::ZipArchive::new(File::open(&path).map_err(|e| e.to_string())?)
            .map_err(|_| "The archive is corrupted")?;
        let names = z.file_names().map(str::to_string).collect::<Vec<_>>();
        format = if names.iter().any(|s| s == "word/document.xml") {
            "docx"
        } else if names.iter().any(|s| s == "xl/workbook.xml") {
            "xlsx"
        } else if names.iter().any(|s| s == "ppt/presentation.xml") {
            "pptx"
        } else if let Ok(mut entry) = z.by_name("mimetype") {
            let mut s = String::new();
            let _ = entry.read_to_string(&mut s);
            if s.contains("spreadsheet") {
                "ods"
            } else if s.contains("presentation") {
                "odp"
            } else {
                "odt"
            }
        } else {
            "zip"
        }
        .into();
    }
    if format == "gz" && name.to_lowercase().ends_with(".tar.gz") {
        format = "tar.gz".into()
    }
    if format == "doc" && ["xls", "ppt", "doc"].contains(&ext.as_str()) {
        format = ext.clone()
    }
    if format == "mp4" && ["m4a", "m4v", "mov"].contains(&ext.as_str()) {
        format = ext.clone()
    }
    if formats::capability(&format).is_err() {
        format = "binary".into()
    }
    let cap = formats::capability(&format)?;
    let mut input = InputFile {
        path: path.to_string_lossy().to_string(),
        name,
        format,
        category: cap.category,
        size: m.len(),
        duration: None,
        width: None,
        height: None,
        sample_rate: None,
        channels: None,
        bit_depth: None,
        media: Default::default(),
    };
    if input.category == "audio" || input.category == "video" {
        let data = process::capture(
            &engines.executable("ffprobe")?,
            &[
                "-v",
                "error",
                "-protocol_whitelist",
                "file,pipe,data,crypto",
                "-show_format",
                "-show_streams",
                "-of",
                "json",
                &input.path,
            ],
            &engines.root,
        )?;
        let v: Value = serde_json::from_str(&data).map_err(|_| "Media probe failed")?;
        input.duration = v["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse().ok());
        input.media.bitrate = v["format"]["bit_rate"]
            .as_str()
            .and_then(|s| s.parse().ok());
        let streams = v["streams"].as_array().ok_or("No media streams")?;
        if streams.is_empty() {
            return Err("The source file appears to be corrupted.".into());
        }
        for s in streams {
            if s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1 {
                input.width = s["width"].as_u64().map(|v| v as u32);
                input.height = s["height"].as_u64().map(|v| v as u32);
                input.media.codec = s["codec_name"].as_str().map(str::to_string);
                input.media.hdr = matches!(
                    s["color_transfer"].as_str(),
                    Some("smpte2084" | "arib-std-b67")
                );
                input.media.fps = s["avg_frame_rate"]
                    .as_str()
                    .and_then(|v| v.split_once('/'))
                    .and_then(|(a, b)| Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?))
                    .filter(|v| v.is_finite());
            }
            if s["codec_type"] == "subtitle" {
                input
                    .media
                    .subtitle_codecs
                    .push(s["codec_name"].as_str().unwrap_or("unknown").into());
            }
            if s["codec_type"] == "audio" {
                input.media.audio_streams += 1;
                input.media.audio_codec = s["codec_name"].as_str().map(str::to_string);
                input.sample_rate = s["sample_rate"].as_str().map(str::to_string);
                input.channels = s["channels"].as_u64().map(|v| v as u32);
                input.bit_depth = s["bits_per_raw_sample"]
                    .as_str()
                    .and_then(|v| v.parse().ok())
                    .filter(|v| *v > 0)
                    .or(s["bits_per_sample"]
                        .as_u64()
                        .map(|v| v as u32)
                        .filter(|v| *v > 0));
            }
        }
    } else if input.category == "image" {
        let s = process::capture(
            &engines.executable("imagemagick")?,
            &[
                "identify",
                "-ping",
                "-format",
                "%w %h %z %[channels] %n\\n",
                &input.path,
            ],
            &engines.root,
        )?;
        let mut wh = s.split_whitespace();
        input.width = wh.next().and_then(|v| v.parse().ok());
        input.height = wh.next().and_then(|v| v.parse().ok());
        input.bit_depth = wh.next().and_then(|v| v.parse().ok());
        input.media.alpha = wh.next().is_some_and(|s| s.ends_with('a'));
        input.media.frames = wh.next().and_then(|v| v.parse().ok());
        if input.width.is_none() {
            return Err("Invalid image".into());
        }
    } else if input.format == "pdf" {
        let text = process::capture(
            &engines.executable("pdfinfo")?,
            &[&input.path],
            &engines.root,
        )?;
        input.media.pages = text
            .lines()
            .find_map(|l| l.strip_prefix("Pages:").and_then(|n| n.trim().parse().ok()));
    }
    Ok(input)
}
