use crate::{
    engines::{self, EngineManager},
    formats,
    models::*,
    probing,
    process::{self, Cancel},
};
use std::{
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
};
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
/// Compression is a real, category-aware plan, not just a label in the UI.
pub fn compression_options(
    input: &InputFile,
    options: &ConversionOptions,
) -> Result<ConversionOptions, String> {
    let mut o = options.clone();
    if o.mode == "convert" {
        return Ok(o);
    }
    if !["smart", "target", "lossless", "custom"].contains(&o.mode.as_str()) {
        return Err("Unknown compression mode.".into());
    }
    if o.target_bytes.is_some_and(|n| n >= input.size)
        && ["target", "custom"].contains(&o.mode.as_str())
    {
        return Err("Para comprimir, escolha um tamanho alvo menor que o original.".into());
    }
    if o.mode == "custom" {
        return Ok(o);
    }
    let lossless = o.mode == "lossless";
    if lossless && input.category == "video" {
        return Err("A compressão de vídeo usa perda de qualidade. Escolha uma intensidade ou use Converter.".into());
    }
    o.lossless = lossless;
    o.format = match input.category.as_str() {
        "image" if input.format == "svg" => "zip",
        "image" if input.format == "gif" && lossless => "zip",
        "image" if input.format == "gif" => "gif",
        "image" if input.bit_depth.unwrap_or(8) > 8 => "png",
        "image" => "webp",
        "audio" if lossless => "flac",
        "audio" => "mp3",
        "video" => "mp4",
        "pdf" => "pdf",
        "archive" => "7z",
        _ => "zip",
    }
    .into();
    o.codec = "auto".into();
    o.encoder = options.encoder.clone();
    o.archive_action = if input.category == "archive" {
        "repack"
    } else {
        "create"
    }
    .into();
    o.archive_level = 9;
    o.merge_paths.clear();
    if o.mode != "target" {
        o.target_bytes = None;
    }
    if o.mode == "target" && (o.target_bytes == Some(0) || o.target_bytes.is_none()) {
        return Err("Informe um tamanho maior que zero.".into());
    }
    if o.mode == "target" && !(["jpg", "webp", "avif", "mp3", "mp4"].contains(&o.format.as_str())) {
        return Err("Tamanho desejado disponível para imagens de 8 bits, áudio e vídeo. Use compressão automática para este arquivo.".into());
    }
    let (quality, crf, bitrate) = match o.preset.as_str() {
        "high" | "maximum" => (85, 23, 192),
        "small" | "verySmall" => (50, 32, 96),
        _ => (72, 27, 128),
    };
    o.quality = quality;
    o.crf = crf;
    o.bitrate = bitrate;
    o.audio_bitrate = bitrate.min(128);
    // Compression budgets come from the source, not resolution-based CRF guesses.
    // The same normalized plan is used by the estimator and the two-pass encoder.
    if input.category == "video" && ["smart", "target"].contains(&o.mode.as_str()) {
        let duration = input
            .duration
            .filter(|d| d.is_finite() && *d > 0.)
            .ok_or("Duração indisponível")?;
        if o.mode == "smart" {
            let ratio = match o.preset.as_str() {
                "high" | "maximum" => 0.85,
                "small" | "verySmall" => 0.45,
                _ => 0.65,
            };
            o.target_bytes = Some((input.size as f64 * ratio) as u64);
        }
        let total_kbps = o.target_bytes.unwrap() as f64 * 8. * 0.97 / duration / 1000.;
        if input.media.audio_streams > 0 {
            // Reserve most of a low-bitrate video's budget for its picture.
            let per_track = (total_kbps * 0.25 / input.media.audio_streams as f64) as u32;
            o.audio_bitrate = o.audio_bitrate.min(per_track.max(32));
        }
        engines::target_bitrate(o.target_bytes.unwrap(), duration,
            o.audio_bitrate.saturating_mul(input.media.audio_streams))
            .map_err(|_| "Este vídeo já tem uma taxa muito baixa para esse tamanho. Escolha Melhor qualidade ou um alvo maior, abaixo do original.".to_string())?;
        o.encoder = "software".into();
    }
    if input.category == "audio" && o.mode == "smart" {
        if let Some(duration) = input.duration.filter(|d| d.is_finite() && *d > 0.) {
            let ceiling =
                ((input.size.saturating_sub(16384) as f64 * 0.85 * 8.) / duration / 1000.) as u32;
            o.bitrate = [
                32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
            ]
            .into_iter()
            .filter(|r| *r <= bitrate && *r <= ceiling)
            .max()
            .unwrap_or(32);
        }
    }
    if lossless {
        o.width = None;
        o.height = None;
        o.percent = None;
        o.sample_rate = None;
        o.channels = None;
        o.bit_depth = None;
        o.normalize = false;
        o.keep_alpha = true;
    }
    Ok(o)
}
pub fn validate_options(o: &ConversionOptions) -> Result<(), String> {
    if o.suffix.len() > 64
        || o.suffix
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err("Sufixo de arquivo inválido.".into());
    }
    if !["auto", "h264", "h265", "av1", "prores"].contains(&o.codec.as_str())
        || !["auto", "software", "h264_nvenc", "h264_qsv", "h264_amf"].contains(&o.encoder.as_str())
        || o.bit_depth.is_some_and(|v| ![16, 24].contains(&v))
    {
        return Err("Unsupported codec, encoder or bit depth.".into());
    }
    if o.quality > 100
        || o.quality == 0
        || o.crf > 51
        || o.archive_level > 9
        || o.dpi < 36
        || o.dpi > 1200
        || o.threads > 32
        || o.bitrate > 10000
        || o.bitrate < 16
        || o.audio_bitrate > 10000
        || o.audio_bitrate < 16
        || o.width.is_some_and(|v| v == 0 || v > 32768)
        || o.height.is_some_and(|v| v == 0 || v > 32768)
        || o.percent.is_some_and(|v| v == 0 || v > 1000)
        || o.fps.is_some_and(|v| v == 0 || v > 240)
        || o.channels.is_some_and(|v| v == 0 || v > 8)
        || o.sample_rate.is_some_and(|v| !(8000..=384000).contains(&v))
    {
        return Err("Invalid conversion settings.".into());
    }
    if o.background.len() != 7
        || !o.background.starts_with('#')
        || !o.background[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err("Invalid background color".into());
    }
    if o.lossless
        && (o.normalize
            || o.width.is_some()
            || o.height.is_some()
            || o.percent.is_some()
            || o.sample_rate.is_some()
            || o.channels.is_some()
            || o.bit_depth.is_some()
            || !o.keep_alpha)
    {
        return Err(
            "Lossless mode cannot resize, normalize, discard alpha or change audio samples.".into(),
        );
    }
    Ok(())
}
pub fn execute(
    input: &InputFile,
    o: &ConversionOptions,
    eng: &EngineManager,
    temp_root: &Path,
    cancel: &Cancel,
    mut progress: impl FnMut(Option<f64>, Option<String>),
) -> Result<(PathBuf, u64), String> {
    let normalized = compression_options(input, o)?;
    let o = &normalized;
    validate_options(o)?;
    let cap = formats::validate(&input.format, &o.format, o.lossless)?;
    cancel.check()?;
    if o.lossless && o.format == "webp" && input.bit_depth.unwrap_or(8) > 8 {
        return Err(
            "WebP supports 8-bit samples. Use PNG or TIFF to preserve this image losslessly."
                .into(),
        );
    }
    let source = PathBuf::from(&input.path);
    let output_dir = if o.output_dir.is_empty() {
        source
            .parent()
            .ok_or("Invalid source folder")?
            .to_path_buf()
    } else {
        PathBuf::from(&o.output_dir)
    };
    if !output_dir.is_dir() {
        return Err("Output folder does not exist.".into());
    }
    fs::create_dir_all(temp_root).map_err(err)?;
    let need = input
        .size
        .saturating_mul(3)
        .saturating_add(32 * 1024 * 1024);
    if fs2::available_space(&output_dir).map_err(err)? < need
        || fs2::available_space(temp_root).map_err(err)? < need
    {
        return Err("Not enough disk space for conversion and temporary files.".into());
    }
    let temp = tempfile::Builder::new()
        .prefix("forma-job-")
        .tempdir_in(temp_root)
        .map_err(err)?;
    let temp_lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(temp.path().join(".forma-job.lock"))
        .map_err(err)?;
    fs2::FileExt::lock_exclusive(&temp_lock).map_err(err)?;
    // Neutral filename prevents ImageMagick pseudo-protocol / bracket interpretation.
    let staged = temp.path().join(format!("source.{}", input.format));
    if fs::hard_link(&source, &staged).is_err() {
        fs::copy(&source, &staged).map_err(err)?;
    }
    let directory = (input.format == "pdf" && ["png", "jpg"].contains(&o.format.as_str()))
        || (cap.category == "archive" && o.archive_action == "extract");
    let dest = temp.path().join(if directory {
        "pages".into()
    } else {
        format!("result.{}", o.format)
    });
    if directory {
        fs::create_dir(&dest).map_err(err)?;
    }
    if cap.category == "archive" {
        archive(&staged, &dest, o, eng, temp.path(), cancel, input)?;
    } else {
        let engine = if cap.engine == "pdf" && o.format == "pdf" {
            "qpdf"
        } else {
            &cap.engine
        };
        eng.verify(engine)?;
        let mut run_once = |opt: &ConversionOptions| -> Result<(), String> {
            let mut p = engines::build(input, opt, &staged, &dest, temp.path())?;
            if !opt.merge_paths.is_empty() {
                if input.category != "image" || opt.format != "pdf" {
                    return Err("Merge is available only for images to PDF.".into());
                }
                let at = p
                    .args
                    .iter()
                    .position(|s| s == &staged.to_string_lossy())
                    .ok_or("Invalid merge plan")?
                    + 1;
                let mut sources = Vec::new();
                for (i, path) in opt.merge_paths.iter().enumerate() {
                    let f = probing::inspect(Path::new(path), eng)?;
                    if f.category != "image" {
                        return Err("Only images can be merged into PDF".into());
                    }
                    let staged_extra = temp.path().join(format!("merge-{i}.{}", f.format));
                    fs::copy(path, &staged_extra).map_err(err)?;
                    sources.push(staged_extra.to_string_lossy().into_owned());
                }
                p.args.splice(at..at, sources);
            }
            fs::create_dir_all(temp.path().join("office-out")).map_err(err)?;
            if p.engine == "libreoffice" {
                let user = temp.path().join("profile/user");
                fs::create_dir_all(&user).map_err(err)?;
                fs::write(user.join("registrymodifications.xcu"),r#"<?xml version="1.0"?><oor:items xmlns:oor="http://openoffice.org/2001/registry"><item oor:path="/org.openoffice.Office.Common/Security/Scripting"><prop oor:name="MacroSecurityLevel" oor:op="fuse"><value>3</value></prop></item><item oor:path="/org.openoffice.Office.Common/Load"><prop oor:name="UpdateDocMode" oor:op="fuse"><value>0</value></prop></item></oor:items>"#).map_err(err)?;
            }
            let executable = eng.executable(&p.engine)?;
            // Two-pass video sizing allocates overhead/audio before encoding.
            if opt.target_bytes.is_some()
                && input.category == "video"
                && ["mp4", "mov", "mkv", "avi", "webm"].contains(&opt.format.as_str())
                && opt.codec != "prores"
            {
                let mut first = p.args.clone();
                first.pop();
                let log = temp.path().join("pass").to_string_lossy().to_string();
                first.extend([
                    "-pass".into(),
                    "1".into(),
                    "-passlogfile".into(),
                    log.clone(),
                    "-f".into(),
                    "null".into(),
                    "NUL".into(),
                ]);
                process::run(
                    &executable,
                    &first,
                    temp.path(),
                    cancel,
                    input.duration,
                    |v, s| progress(v.map(|n| n * 0.45), s),
                )?;
                let mut second = p.args.clone();
                let final_arg = second.pop().unwrap();
                second.extend([
                    "-pass".into(),
                    "2".into(),
                    "-passlogfile".into(),
                    log,
                    final_arg,
                ]);
                process::run(
                    &executable,
                    &second,
                    temp.path(),
                    cancel,
                    input.duration,
                    |v, s| progress(v.map(|n| 45. + n * 0.55), s),
                )?;
            } else {
                let result = process::run_limited(
                    &executable,
                    &p.args,
                    temp.path(),
                    cancel,
                    input.duration,
                    if p.engine == "libreoffice" { 300 } else { 86400 },
                    &mut progress,
                );
                if p.engine == "libreoffice" {
                    result.map_err(|_| "The document could not be converted within five minutes. It may be damaged, unsupported, or too complex. Try opening and saving it in its original application.".to_string())?;
                } else if result.is_err() && opt.encoder != "software" && cancel.check().is_ok() {
                    let mut fallback = opt.clone();
                    fallback.encoder = "software".into();
                    let plan = engines::build(input, &fallback, &staged, &dest, temp.path())?;
                    process::run(
                        &executable,
                        &plan.args,
                        temp.path(),
                        cancel,
                        input.duration,
                        &mut progress,
                    )?;
                } else {
                    result?;
                }
            }
            if p.engine == "libreoffice" {
                let result = temp
                    .path()
                    .join("office-out")
                    .join(format!("source.{}", opt.format));
                if !result.exists() {
                    return Err("LibreOffice did not produce the requested format.".into());
                }
                fs::rename(result, &dest).map_err(err)?;
            }
            Ok(())
        };
        if let Some(target) = o
            .target_bytes
            .filter(|_| input.category == "image" && !o.lossless)
        {
            if !["jpg", "webp", "avif"].contains(&o.format.as_str()) {
                return Err(
                    "Target size requires JPG, WEBP or AVIF, or an explicit lossless attempt."
                        .into(),
                );
            }
            let mut lo = 1u8;
            let mut hi = 100u8;
            let mut best = None;
            let best_path = temp.path().join("best");
            let mut smallest = u64::MAX;
            while lo <= hi {
                cancel.check()?;
                let mid = lo + (hi - lo) / 2;
                let mut options = o.clone();
                options.quality = mid;
                run_once(&options)?;
                let size = fs::metadata(&dest).map_err(err)?.len();
                smallest = smallest.min(size);
                if size <= target {
                    fs::copy(&dest, &best_path).map_err(err)?;
                    best = Some(mid);
                    lo = mid + 1
                } else {
                    if mid == 1 {
                        break;
                    }
                    hi = mid - 1
                }
            }
            if best.is_none() {
                return Err(format!("The target cannot be reached at this resolution. Smallest measured output: {smallest} bytes."));
            }
            fs::copy(best_path, &dest).map_err(err)?;
        } else {
            run_once(o)?
        }
    }
    cancel.check()?;
    validate_output(&dest, &o.format, directory, eng, cancel)?;
    if !directory
        && cap.category != "archive"
        && ["audio", "video", "image", "pdf"].contains(&input.category.as_str())
    {
        let measured = probing::inspect(&dest, eng)?;
        if let (Some(before), Some(after)) = (input.duration, measured.duration) {
            if (before - after).abs() > 0.75f64.max(before * 0.01) {
                return Err("Output validation failed: duration changed unexpectedly.".into());
            }
        }
        if input.format == "pdf" && o.format == "pdf" && input.media.pages != measured.media.pages {
            return Err("Output validation failed: PDF page count changed.".into());
        }
        if input.category == "image"
            && ["gif", "webp", "tiff"].contains(&o.format.as_str())
            && input.media.frames.unwrap_or(1) != measured.media.frames.unwrap_or(1)
        {
            return Err("Output validation failed: animation frame count changed.".into());
        }
        if input.category == "video"
            && measured.category == "video"
            && input.media.audio_streams != measured.media.audio_streams
        {
            return Err("Output validation failed: audio stream count changed.".into());
        }
    }
    if o.lossless && input.category == "audio" && cap.category != "archive" {
        let a = pcm_hash(&staged, eng, temp.path(), cancel)?;
        let b = pcm_hash(&dest, eng, temp.path(), cancel)?;
        if a != b {
            return Err("Lossless validation failed: decoded PCM differs.".into());
        }
    }
    if o.lossless && input.category == "image" && cap.category != "archive" {
        let exe = eng.executable("imagemagick")?;
        let args = vec![
            "compare".into(),
            "-metric".into(),
            "AE".into(),
            staged.to_string_lossy().into_owned(),
            dest.to_string_lossy().into_owned(),
            "null:".into(),
        ];
        process::run(&exe, &args, temp.path(), cancel, None, |_, _| {})
            .map_err(|_| "Lossless validation failed: decoded pixels differ.")?;
    }
    let size = size_of(&dest);
    if let Some(target) = o.target_bytes {
        if (o.lossless || ["wav", "flac", "aiff"].contains(&o.format.as_str())) && size > target {
            return Err(format!("The requested lossless target size cannot be achieved. Measured output: {size} bytes. Switch to Smart compression to allow quality loss."));
        }
    }
    // An attempted compression that grew the file is not a successful reduction.
    // Keep the original and discard the temporary candidate without publishing it.
    if o.mode != "convert" && !directory && size >= input.size {
        return Ok((source, input.size));
    }
    let stem = format!(
        "{}{}",
        source.file_stem().unwrap_or_default().to_string_lossy(),
        o.suffix
    );
    let path = publish(
        &dest,
        &output_dir,
        &stem,
        if directory { "" } else { &o.format },
        cancel,
    )?;
    Ok((path, size))
}
fn pcm_hash(p: &Path, eng: &EngineManager, cwd: &Path, cancel: &Cancel) -> Result<String, String> {
    process::run(
        &eng.executable("ffmpeg")?,
        &[
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe,data,crypto",
            "-i",
            &p.to_string_lossy(),
            "-map",
            "0:a:0",
            "-c:a",
            "pcm_s32le",
            "-f",
            "hash",
            "-hash",
            "sha256",
            "-",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
        cwd,
        cancel,
        None,
        |_, _| {},
    )
}
pub fn size_of(p: &Path) -> u64 {
    if p.is_file() {
        fs::metadata(p).map(|m| m.len()).unwrap_or(0)
    } else {
        walkdir::WalkDir::new(p)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum()
    }
}
fn validate_output(
    p: &Path,
    format: &str,
    directory: bool,
    eng: &EngineManager,
    cancel: &Cancel,
) -> Result<(), String> {
    if size_of(p) == 0 {
        return Err("Converter produced an empty result.".into());
    }
    if directory {
        for entry in walkdir::WalkDir::new(p) {
            let e = entry.map_err(err)?;
            if e.file_type().is_symlink() {
                return Err("Links in output are not permitted.".into());
            }
            if e.file_type().is_file() && ["png", "jpg"].contains(&format) {
                let result = probing::inspect(e.path(), eng)?;
                if result.format != format {
                    return Err("Invalid PDF page output".into());
                }
            }
        }
        return Ok(());
    }
    if format == "pdf" {
        process::run(
            &eng.executable("qpdf")?,
            &["--check".into(), p.to_string_lossy().into()],
            &eng.root,
            cancel,
            None,
            |_, _| {},
        )?;
    } else if ["zip", "7z", "tar", "tar.gz", "gz", "bz2", "xz"].contains(&format) { /* archive adapter already tests with password */
    } else {
        let found = probing::inspect(p, eng)?;
        let same = found.format == format
            || ["jpg", "jpeg"].contains(&format) && found.format == "jpg"
            || ["mp4", "mov", "m4a"].contains(&format)
                && ["mp4", "mov", "m4a"].contains(&found.format.as_str())
            || ["ogg", "opus"].contains(&format)
                && ["ogg", "opus"].contains(&found.format.as_str())
            || ["tif", "tiff"].contains(&format) && found.format == "tiff";
        if !same {
            return Err(format!(
                "Output validation failed: expected {format}, detected {}",
                found.format
            ));
        }
    }
    Ok(())
}
pub fn publish(
    src: &Path,
    dir: &Path,
    stem: &str,
    ext: &str,
    cancel: &Cancel,
) -> Result<PathBuf, String> {
    if src.is_file() {
        return crate::output::publish_file(src, dir, stem, ext, cancel);
    }
    for n in 0..10000 {
        cancel.check()?;
        let name = format!(
            "{}{}{}",
            stem,
            if n == 0 {
                String::new()
            } else {
                format!(" ({n})")
            },
            if ext.is_empty() {
                String::new()
            } else {
                format!(".{ext}")
            }
        );
        let dest = dir.join(name);
        if src.is_dir() {
            match fs::create_dir(&dest) {
                Ok(()) => {
                    let result = (|| {
                        for entry in walkdir::WalkDir::new(src).min_depth(1) {
                            cancel.check()?;
                            let e = entry.map_err(err)?;
                            let p = dest.join(e.path().strip_prefix(src).unwrap());
                            if e.file_type().is_dir() {
                                fs::create_dir_all(p).map_err(err)?
                            } else {
                                fs::copy(e.path(), p).map_err(err)?;
                            }
                        }
                        Ok::<(), String>(())
                    })();
                    if let Err(e) = result {
                        let _ = fs::remove_dir_all(&dest);
                        return Err(e);
                    }
                    return Ok(dest);
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(err(e)),
            }
        } else {
            match OpenOptions::new().write(true).create_new(true).open(&dest) {
                Ok(mut out) => {
                    let result = (|| -> Result<(), String> {
                        use std::io::{Read, Write};
                        let mut f = fs::File::open(src).map_err(err)?;
                        let mut buffer = [0; 65536];
                        loop {
                            cancel.check()?;
                            let n = f.read(&mut buffer).map_err(err)?;
                            if n == 0 {
                                break;
                            }
                            out.write_all(&buffer[..n]).map_err(err)?;
                        }
                        out.sync_all().map_err(err)
                    })();
                    if let Err(e) = result {
                        drop(out);
                        let _ = fs::remove_file(&dest);
                        return Err(err(e));
                    }
                    return Ok(dest);
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(err(e)),
            }
        }
    }
    Err("Too many output filename collisions".into())
}
fn archive(
    src: &Path,
    dest: &Path,
    o: &ConversionOptions,
    eng: &EngineManager,
    temp: &Path,
    cancel: &Cancel,
    input: &InputFile,
) -> Result<(), String> {
    eng.verify("7zip")?;
    let exe = eng.executable("7zip")?;
    let mut password = Vec::new();
    if !o.password.is_empty() {
        password.push(format!("-p{}", o.password))
    }
    let invoke = |mut args: Vec<String>, cwd: &Path| -> Result<String, String> {
        let at = args.iter().position(|s| s == "--").unwrap_or(args.len());
        args.splice(at..at, password.clone());
        process::run(&exe, &args, cwd, cancel, None, |_, _| {})
    };
    let unpack = temp.join("unpack");
    fs::create_dir(&unpack).map_err(err)?;
    if input.category != "archive" || o.archive_action == "create" {
        fs::copy(src, unpack.join(&input.name)).map_err(err)?;
        for path in &o.merge_paths {
            let p = Path::new(path);
            let name = p.file_name().ok_or("Invalid filename")?;
            let destination = unpack.join(name);
            if destination.exists() {
                return Err("Archive inputs have duplicate names. Rename them first.".into());
            }
            fs::copy(p, destination).map_err(err)?;
        }
    } else {
        let listing = invoke(
            vec![
                "l".into(),
                "-slt".into(),
                "-ba".into(),
                src.to_string_lossy().into(),
            ],
            temp,
        )?;
        check_archive_listing(&listing, fs2::available_space(temp).map_err(err)?)?;
        invoke(
            vec![
                "x".into(),
                "-y".into(),
                format!("-o{}", unpack.display()),
                src.to_string_lossy().into(),
            ],
            temp,
        )?;
        if input.format == "tar.gz" {
            let entries = fs::read_dir(&unpack)
                .map_err(err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(err)?;
            if entries.len() != 1 {
                return Err("Invalid TAR.GZ container".into());
            }
            let tar = temp.join("inner.tar");
            fs::rename(entries[0].path(), &tar).map_err(err)?;
            let list = invoke(
                vec![
                    "l".into(),
                    "-slt".into(),
                    "-ba".into(),
                    tar.to_string_lossy().into(),
                ],
                temp,
            )?;
            check_archive_listing(&list, fs2::available_space(temp).map_err(err)?)?;
            invoke(
                vec![
                    "x".into(),
                    "-y".into(),
                    format!("-o{}", unpack.display()),
                    tar.to_string_lossy().into(),
                ],
                temp,
            )?;
        }
    }
    for entry in walkdir::WalkDir::new(&unpack) {
        let e = entry.map_err(err)?;
        if e.file_type().is_symlink() {
            return Err("Archive contains unsafe symbolic links.".into());
        }
    }
    if o.archive_action == "extract" {
        for e in fs::read_dir(&unpack).map_err(err)? {
            let e = e.map_err(err)?;
            fs::rename(e.path(), dest.join(e.file_name())).map_err(err)?;
        }
        return Ok(());
    }
    let mut files = vec!["*".to_string()];
    let mut cwd = unpack.clone();
    let mut kind = o.format.as_str();
    if o.format == "tar.gz" {
        let tar = temp.join("content.tar");
        invoke(
            vec![
                "a".into(),
                "-ttar".into(),
                tar.to_string_lossy().into(),
                "--".into(),
                "*".into(),
            ],
            &unpack,
        )?;
        files = vec![tar.to_string_lossy().into()];
        kind = "gzip";
        cwd = temp.to_path_buf();
    } else if ["gz", "bz2", "xz"].contains(&kind) {
        let entries = fs::read_dir(&unpack)
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        if entries.len() != 1 || !entries[0].path().is_file() {
            return Err("GZ/BZ2/XZ supports one file. Use TAR.GZ for multiple files.".into());
        }
        files = vec![entries[0].path().to_string_lossy().into()];
        kind = match kind {
            "gz" => "gzip",
            "bz2" => "bzip2",
            _ => "xz",
        };
    }
    if !o.password.is_empty() && !(["zip", "7z"].contains(&kind)) {
        return Err("Passwords are supported only for ZIP and 7Z output.".into());
    }
    let mut args = vec![
        "a".into(),
        format!("-t{kind}"),
        format!("-mx={}", o.archive_level),
        dest.to_string_lossy().into(),
        "--".into(),
    ];
    args.extend(files);
    invoke(args, &cwd)?;
    invoke(vec!["t".into(), dest.to_string_lossy().into()], temp)?;
    Ok(())
}
pub fn check_archive_listing(s: &str, available: u64) -> Result<(), String> {
    if s.contains("[OUTPUT_TRUNCATED]") {
        return Err("Archive listing is too large to validate safely.".into());
    }
    let mut total = 0u64;
    for line in s.lines() {
        if let Some(p) = line.strip_prefix("Path = ") {
            if p.starts_with(['/', '\\'])
                || p.contains(':')
                || p.split(['/', '\\']).any(|s| s == "..")
            {
                return Err("Unsafe archive path rejected.".into());
            }
        }
        if line
            .strip_prefix("Symbolic Link =")
            .or_else(|| line.strip_prefix("Hard Link ="))
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err("Archive links are not supported.".into());
        }
        if let Some(n) = line
            .strip_prefix("Size = ")
            .and_then(|v| v.parse::<u64>().ok())
        {
            total = total.saturating_add(n)
        }
    }
    if total > available / 2 {
        return Err("Not enough disk space to safely extract this archive.".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_codec_is_not_silently_replaced() {
        let options = ConversionOptions {
            codec: "not-a-codec".into(),
            ..Default::default()
        };
        assert!(validate_options(&options).is_err())
    }
    #[test]
    fn traversal_rejected() {
        for s in ["../x", "C:\\x", "/etc/x", "a/../../x"] {
            assert!(check_archive_listing(&format!("Path = {s}"), u64::MAX).is_err())
        }
    }
    #[test]
    fn no_overwrite() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("a.txt");
        fs::write(&src, "original").unwrap();
        let out = publish(&src, d.path(), "a", "txt", &Cancel::default()).unwrap();
        assert_eq!(out.file_name().unwrap(), "a (1).txt");
        assert_eq!(fs::read_to_string(src).unwrap(), "original")
    }
}
