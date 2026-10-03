use crate::{formats, models::*, process};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone)]
pub struct EngineManager {
    pub root: PathBuf,
}
impl EngineManager {
    pub fn hardware(&self) -> Vec<String> {
        let Ok(exe) = self.executable("ffmpeg") else {
            return vec![];
        };
        ["h264_nvenc", "h264_qsv", "h264_amf"]
            .into_iter()
            .filter(|codec| {
                process::capture(
                    &exe,
                    &[
                        "-v",
                        "error",
                        "-f",
                        "lavfi",
                        "-i",
                        "color=size=128x128:rate=1",
                        "-frames:v",
                        "1",
                        "-c:v",
                        codec,
                        "-f",
                        "null",
                        "NUL",
                    ],
                    &self.root,
                )
                .is_ok()
            })
            .map(str::to_string)
            .collect()
    }
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn executable(&self, id: &str) -> Result<PathBuf, String> {
        let rel = match id {
            "ffmpeg" => "ffmpeg/ffmpeg.exe",
            "ffprobe" => "ffmpeg/ffprobe.exe",
            "imagemagick" => "imagemagick/magick.exe",
            "7zip" => "7zip/7z.exe",
            "pandoc" => "pandoc/pandoc.exe",
            "libreoffice" => "libreoffice/program/soffice.com",
            "qpdf" => "qpdf/bin/qpdf.exe",
            "pdf" => "poppler/Library/bin/pdftoppm.exe",
            "pdfinfo" => "poppler/Library/bin/pdfinfo.exe",
            _ => return Err("Unknown engine".into()),
        };
        let p = self.root.join(rel);
        if !p.is_file() {
            return Err(format!(
                "Bundled engine {id} is missing. Reinstall Reshape."
            ));
        }
        Ok(p)
    }
    pub fn health(&self) -> Vec<EngineCapability> {
        [
            "ffmpeg",
            "ffprobe",
            "imagemagick",
            "7zip",
            "pandoc",
            "libreoffice",
            "qpdf",
            "pdf",
        ]
        .iter()
        .map(|id| {
            let result = self.executable(id).and_then(|p| {
                let args = match *id {
                    "ffmpeg" | "ffprobe" => vec!["-version"],
                    "imagemagick" => vec!["-version"],
                    "7zip" => vec!["i"],
                    "pdf" => vec!["-h"],
                    _ => vec!["--version"],
                };
                process::capture(&p, &args, &self.root)
            });
            let integrity = self.verify(id).is_ok();
            EngineCapability {
                id: id.to_string(),
                ready: result.is_ok() && integrity,
                version: result
                    .map(|s| {
                        s.lines()
                            .find(|s| !s.trim().is_empty())
                            .unwrap_or("Ready")
                            .chars()
                            .take(120)
                            .collect()
                    })
                    .unwrap_or_else(|e| e),
                integrity,
            }
        })
        .collect()
    }
    pub fn verify(&self, id: &str) -> Result<(), String> {
        let path = self.executable(id)?;
        let file = self.root.join("manifest.json");
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(file).map_err(|_| "Engine integrity manifest missing")?,
        )
        .map_err(|e| e.to_string())?;
        let rel = path
            .strip_prefix(&self.root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let expected = manifest[&rel]
            .as_str()
            .ok_or("Engine not in integrity manifest")?;
        let mut input = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hash = Sha256::new();
        let mut b = [0; 65536];
        loop {
            let n = input.read(&mut b).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hash.update(&b[..n]);
        }
        if hex::encode(hash.finalize()) != expected {
            return Err("Engine integrity check failed. Reinstall Reshape.".into());
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct CommandPlan {
    pub engine: String,
    pub args: Vec<String>,
}
fn add(v: &mut Vec<String>, a: &[&str]) {
    v.extend(a.iter().map(|s| s.to_string()))
}
pub fn build(
    input: &InputFile,
    o: &ConversionOptions,
    source: &Path,
    dest: &Path,
    temp: &Path,
) -> Result<CommandPlan, String> {
    let cap = formats::validate(&input.format, &o.format, o.lossless)?;
    if input.category == "video"
        && (((["webm", "avi"].contains(&o.format.as_str())) && o.codec != "auto")
            || (o.codec == "prores" && o.format != "mov"))
    {
        return Err("The selected codec is incompatible with this container. Use Automatic or select another output format.".into());
    }
    let src = source.to_string_lossy();
    let dst = dest.to_string_lossy();
    let mut a = Vec::new();
    let mut engine = cap.engine;
    match engine.as_str() {
        "imagemagick" => {
            add(
                &mut a,
                &[
                    "-limit", "memory", "256MiB", "-limit", "map", "512MiB", "-limit", "disk",
                    "4GiB", &src,
                ],
            );
            if !o.lossless {
                if let Some(p) = o.percent {
                    add(&mut a, &["-resize", &format!("{}%", p)])
                } else if o.width.is_some() || o.height.is_some() {
                    let geo = format!(
                        "{}x{}{}",
                        o.width.map(|n| n.to_string()).unwrap_or_default(),
                        o.height.map(|n| n.to_string()).unwrap_or_default(),
                        if o.keep_aspect { "" } else { "!" }
                    );
                    add(&mut a, &["-resize", &geo]);
                }
            }
            if !o.keep_metadata {
                add(&mut a, &["+profile", "!icc,*"]);
            }
            if o.format == "jpg" || !o.keep_alpha {
                add(
                    &mut a,
                    &[
                        "-background",
                        &o.background,
                        "-alpha",
                        "remove",
                        "-alpha",
                        "off",
                    ],
                )
            }
            add(
                &mut a,
                &[
                    "-units",
                    "PixelsPerInch",
                    "-density",
                    &o.dpi.to_string(),
                    "-quality",
                    &o.quality.to_string(),
                ],
            );
            if o.lossless && o.format == "webp" {
                add(&mut a, &["-define", "webp:lossless=true"])
            }
            if o.format == "ico" {
                add(&mut a, &["-resize", "256x256>"])
            }
            add(&mut a, &[&dst]);
        }
        "ffmpeg" => {
            add(
                &mut a,
                &[
                    "-hide_banner",
                    "-nostdin",
                    "-y",
                    "-progress",
                    "pipe:1",
                    "-nostats",
                    "-protocol_whitelist",
                    "file,pipe,data,crypto",
                    "-i",
                    &src,
                    "-map_metadata",
                    if o.keep_metadata { "0" } else { "-1" },
                    "-map_chapters",
                    if o.chapters { "0" } else { "-1" },
                ],
            );
            let video = ["mp4", "mov", "mkv", "avi", "webm"].contains(&o.format.as_str());
            if video && crate::planning::remux_possible(input, o) {
                add(&mut a, &["-map", "0:v?", "-map", "0:a?", "-c", "copy"]);
                if o.subtitles {
                    add(&mut a, &["-map", "0:s?"]);
                }
                if o.format == "mkv" {
                    add(&mut a, &["-map", "0:t?"]);
                }
                add(&mut a, &[&dst]);
                return Ok(CommandPlan { engine, args: a });
            }
            if video {
                if input.media.hdr {
                    return Err("HDR: use MKV stream copy to preserve dynamic range.".into());
                }
                add(&mut a, &["-map", "0:v:0", "-map", "0:a?"]);
                let codec = match (o.format.as_str(), o.codec.as_str()) {
                    ("webm", _) => "libvpx-vp9",
                    (_, "h265") => "libx265",
                    (_, "av1") => "libaom-av1",
                    ("mov", "prores") => "prores_ks",
                    ("avi", _) => "mpeg4",
                    _ => "libx264",
                };
                add(&mut a, &["-c:v", codec]);
                if o.target_bytes.is_some() {
                    let duration = input
                        .duration
                        .filter(|d| *d > 0.)
                        .ok_or("Duration is unavailable for target size")?;
                    let kbps = target_bitrate(
                        o.target_bytes.unwrap(),
                        duration,
                        o.audio_bitrate.saturating_mul(input.media.audio_streams),
                    )?;
                    add(&mut a, &["-b:v", &format!("{kbps}k")]);
                } else if codec == "mpeg4" {
                    add(&mut a, &["-q:v", "4"])
                } else if codec == "prores_ks" {
                    add(&mut a, &["-profile:v", "3"])
                } else {
                    add(&mut a, &["-crf", &o.crf.to_string()]);
                    if codec == "libvpx-vp9" {
                        add(&mut a, &["-b:v", "0"])
                    }
                }
                if codec == "libx264" || codec == "libx265" {
                    add(&mut a, &["-preset", "medium"])
                }
                if codec == "libaom-av1" {
                    add(&mut a, &["-cpu-used", "6"])
                }
                let mut filters = Vec::new();
                if o.width.is_some() || o.height.is_some() {
                    let w = o.width.map(|v| v.to_string()).unwrap_or("-2".into());
                    let h = o.height.map(|v| v.to_string()).unwrap_or("-2".into());
                    filters.push(if o.keep_aspect&&o.width.is_some()&&o.height.is_some(){format!("scale={w}:{h}:force_original_aspect_ratio=decrease:force_divisible_by=2")}else{format!("scale={w}:{h}")});
                } else {
                    filters.push("scale=trunc(iw/2)*2:trunc(ih/2)*2".into())
                }
                add(
                    &mut a,
                    &[
                        "-vf",
                        &filters.join(","),
                        "-pix_fmt",
                        if codec == "prores_ks" {
                            "yuv422p10le"
                        } else {
                            "yuv420p"
                        },
                    ],
                );
                if o.fps.is_none() {
                    add(&mut a, &["-fps_mode", "passthrough"]);
                }
                if let Some(f) = o.fps {
                    add(&mut a, &["-r", &f.to_string()])
                }
                add(
                    &mut a,
                    &[
                        "-c:a",
                        if o.format == "webm" { "libopus" } else { "aac" },
                        "-b:a",
                        &format!("{}k", o.audio_bitrate),
                    ],
                );
                if o.subtitles {
                    if o.format == "mkv" {
                        add(&mut a, &["-map", "0:s?", "-c:s", "copy"])
                    } else if o.format == "mp4" || o.format == "mov" {
                        add(&mut a, &["-map", "0:s?", "-c:s", "mov_text"])
                    } else {
                        return Err(
                            "Subtitle preservation is supported for MKV/MP4/MOV only.".into()
                        );
                    }
                }
                if ["mp4", "mov"].contains(&o.format.as_str()) {
                    add(&mut a, &["-movflags", "+faststart"])
                }
            } else {
                add(&mut a, &["-map", "0:a:0"]);
                if o.keep_artwork
                    && o.keep_metadata
                    && input.category == "audio"
                    && ["mp3", "flac", "m4a"].contains(&o.format.as_str())
                {
                    add(
                        &mut a,
                        &[
                            "-map",
                            "0:v:disp:attached_pic?",
                            "-c:v",
                            "copy",
                            "-disposition:v",
                            "attached_pic",
                        ],
                    );
                } else {
                    add(&mut a, &["-vn"]);
                }
                let codec = match o.format.as_str() {
                    "flac" => "flac",
                    "wav" | "aiff" => {
                        if o.format == "wav" {
                            if o.bit_depth == Some(24) || input.bit_depth.unwrap_or(16) > 16 {
                                "pcm_s24le"
                            } else {
                                "pcm_s16le"
                            }
                        } else if o.bit_depth == Some(24) || input.bit_depth.unwrap_or(16) > 16 {
                            "pcm_s24be"
                        } else {
                            "pcm_s16be"
                        }
                    }
                    "mp3" => "libmp3lame",
                    "aac" | "m4a" => "aac",
                    "ogg" => "libvorbis",
                    "opus" => "libopus",
                    "wma" => "wmav2",
                    "ac3" => "ac3",
                    _ => return Err("Unsupported audio codec".into()),
                };
                add(&mut a, &["-c:a", codec]);
                if codec == "flac" {
                    add(&mut a, &["-compression_level", "8"]);
                }
                if !["wav", "aiff", "flac"].contains(&o.format.as_str()) {
                    if o.format == "mp3"
                        && o.target_bytes.is_none()
                        && o.preset != "custom"
                        && o.mode == "convert"
                    {
                        add(
                            &mut a,
                            &[
                                "-q:a",
                                if o.quality >= 85 {
                                    "0"
                                } else if o.quality >= 65 {
                                    "3"
                                } else {
                                    "7"
                                },
                            ],
                        );
                    } else {
                        let rate = if let Some(bytes) = o.target_bytes {
                            target_bitrate(bytes, input.duration.ok_or("Duration unavailable")?, 0)?
                        } else {
                            o.bitrate
                        };
                        add(&mut a, &["-b:a", &format!("{rate}k")]);
                    }
                }
                if let Some(rate) = o.sample_rate {
                    add(&mut a, &["-ar", &rate.to_string()])
                }
                if let Some(ch) = o.channels {
                    add(&mut a, &["-ac", &ch.to_string()])
                }
                if o.normalize {
                    add(&mut a, &["-af", "loudnorm=I=-16:TP=-1.5:LRA=11"])
                }
            }
            add(&mut a, &["-threads", &o.threads.max(1).to_string(), &dst]);
        }
        "pandoc" => {
            let from = match input.format.as_str() {
                "md" => "markdown",
                "txt" => "markdown",
                s => s,
            };
            let to = match o.format.as_str() {
                "md" => "markdown",
                "txt" => "plain",
                s => s,
            };
            add(
                &mut a,
                &[
                    "--sandbox",
                    "--standalone",
                    "--from",
                    from,
                    "--to",
                    to,
                    &src,
                    "--output",
                    &dst,
                ],
            );
        }
        "libreoffice" => {
            let profile = url::Url::from_directory_path(temp.join("profile"))
                .map_err(|_| "Invalid profile path")?;
            add(
                &mut a,
                &[
                    &format!("-env:UserInstallation={profile}"),
                    "--headless",
                    "--norestore",
                    "--nodefault",
                    "--nolockcheck",
                    "--convert-to",
                    &o.format,
                    "--outdir",
                    &temp.join("office-out").to_string_lossy(),
                    &src,
                ],
            );
        }
        "pdf" => {
            if o.format == "pdf" {
                engine = "qpdf".into();
                if !o.lossless {
                    add(
                        &mut a,
                        &[
                            "--optimize-images",
                            &format!("--jpeg-quality={}", o.quality),
                        ],
                    )
                }
                add(
                    &mut a,
                    &[
                        "--object-streams=generate",
                        "--stream-data=compress",
                        "--recompress-flate",
                        "--compression-level=9",
                        &src,
                        &dst,
                    ],
                );
            } else {
                add(
                    &mut a,
                    &[
                        if o.format == "png" { "-png" } else { "-jpeg" },
                        "-r",
                        &o.dpi.to_string(),
                        &src,
                        &dest.join("page").to_string_lossy(),
                    ],
                );
            }
        }
        _ => return Err("Archive requires multi-step adapter".into()),
    }
    if engine == "ffmpeg"
        && o.encoder != "software"
        && o.encoder != "auto"
        && o.target_bytes.is_none()
        && ["mp4", "mov", "mkv"].contains(&o.format.as_str())
    {
        if !["h264_nvenc", "h264_qsv", "h264_amf"].contains(&o.encoder.as_str()) {
            return Err("Unsupported hardware encoder".into());
        }
        if o.codec != "auto" && o.codec != "h264" {
            return Err(
                "Hardware mode in this build supports H.264. Select software for other codecs."
                    .into(),
            );
        }
        let mut i = 0;
        while i < a.len() {
            if a[i] == "-c:v" {
                a[i + 1] = o.encoder.clone();
                i += 2
            } else if a[i] == "-crf" || a[i] == "-preset" {
                a.drain(i..i + 2);
            } else {
                i += 1
            }
        }
        let last = a.pop().unwrap();
        match o.encoder.as_str() {
            "h264_nvenc" => add(
                &mut a,
                &["-rc", "vbr", "-cq", &o.crf.to_string(), "-b:v", "0"],
            ),
            "h264_qsv" => add(&mut a, &["-global_quality", &o.crf.to_string()]),
            "h264_amf" => add(
                &mut a,
                &[
                    "-rc",
                    "cqp",
                    "-qp_i",
                    &o.crf.to_string(),
                    "-qp_p",
                    &o.crf.to_string(),
                ],
            ),
            _ => {}
        }
        a.push(last);
    }
    Ok(CommandPlan { engine, args: a })
}
pub fn target_bitrate(bytes: u64, duration: f64, audio: u32) -> Result<u32, String> {
    if duration <= 0. {
        return Err("Invalid duration".into());
    }
    let kbps = (bytes as f64 * 8. * 0.97 / duration / 1000.) - audio as f64;
    if kbps < 16. {
        return Err("Target is too small for the selected duration and audio bitrate.".into());
    }
    Ok(kbps.floor() as u32)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_accounts_for_audio() {
        assert_eq!(target_bitrate(100_000_000, 60., 128).unwrap(), 12805)
    }
    #[test]
    fn tiny_target_fails() {
        assert!(target_bitrate(100, 60., 128).is_err())
    }
}
