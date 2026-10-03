//! Repeatable installed-build smoke test. Never runs during normal application startup.
use crate::{
    conversion,
    engines::EngineManager,
    models::ConversionOptions,
    probing,
    process::{self, Cancel},
};
use std::{fs, path::Path};
pub fn run(destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let root = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("No executable directory")?
        .join("binaries");
    let eng = EngineManager::new(root);
    let health = eng.health();
    let hardware = eng.hardware();
    let mut results = Vec::new();
    let image = destination.join("smoke ação 日本.png");
    process::capture(
        &eng.executable("imagemagick")?,
        &[
            "-size",
            "64x64",
            "gradient:#7660ce-#eeeeee",
            "-depth",
            "8",
            &image.to_string_lossy(),
        ],
        destination,
    )?;
    let wav = destination.join("smoke.wav");
    process::capture(
        &eng.executable("ffmpeg")?,
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:a",
            "pcm_s24le",
            &wav.to_string_lossy(),
        ],
        destination,
    )?;
    let md = destination.join("smoke.md");
    fs::write(&md, "# Reshape\n\nOffline conversion smoke test.\n").map_err(|e| e.to_string())?;
    let docx = destination.join("smoke.docx");
    process::capture(
        &eng.executable("pandoc")?,
        &[&md.to_string_lossy(), "-o", &docx.to_string_lossy()],
        destination,
    )?;
    for (source, format, lossless) in [
        (&image, "jpg", false),
        (&image, "webp", true),
        (&wav, "flac", true),
        (&wav, "mp3", false),
        (&md, "html", false),
        (&docx, "pdf", false),
        (&md, "7z", true),
    ] {
        let result = probing::inspect(source, &eng).and_then(|input| {
            conversion::execute(
                &input,
                &ConversionOptions {
                    format: format.into(),
                    lossless,
                    output_dir: destination.to_string_lossy().into(),
                    ..Default::default()
                },
                &eng,
                destination,
                &Cancel::default(),
                |_, _| {},
            )
        });
        results.push(serde_json::json!({"input":source.file_name().unwrap().to_string_lossy(),"outputFormat":format,"success":result.is_ok(),"error":result.as_ref().err(),"output":result.as_ref().ok().map(|(p,n)|serde_json::json!({"path":p,"bytes":n}))}));
    }
    // Exercise the user-reported FLAC compression route in the actual installed binary.
    let flac = destination.join("compression-1.4mb.flac");
    process::capture(
        &eng.executable("ffmpeg")?,
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=color=pink:duration=12:sample_rate=44100:seed=42",
            "-c:a",
            "flac",
            "-sample_fmt",
            "s32",
            &flac.to_string_lossy(),
        ],
        destination,
    )?;
    let input = probing::inspect(&flac, &eng)?;
    let mut sizes = Vec::new();
    for preset in ["high", "balanced", "small"] {
        let result = conversion::execute(
            &input,
            &ConversionOptions {
                mode: "smart".into(),
                preset: preset.into(),
                format: "flac".into(),
                lossless: true,
                output_dir: destination.to_string_lossy().into(),
                ..Default::default()
            },
            &eng,
            destination,
            &Cancel::default(),
            |_, _| {},
        );
        let reduced = result.as_ref().is_ok_and(|(path, size)| {
            *size < input.size && path.extension().is_some_and(|s| s == "mp3")
        });
        if let Ok((_, size)) = &result {
            sizes.push(*size);
        }
        results.push(serde_json::json!({"input":"compression-1.4mb.flac","preset":preset,"inputBytes":input.size,"outputBytes":result.as_ref().ok().map(|(_,s)|s),"success":reduced,"error":result.as_ref().err()}));
    }
    let tiny = destination.join("tiny-original.txt");
    fs::write(&tiny, "a").map_err(|e| e.to_string())?;
    let tiny_input = probing::inspect(&tiny, &eng)?;
    let no_gain = conversion::execute(
        &tiny_input,
        &ConversionOptions {
            mode: "smart".into(),
            output_dir: destination.to_string_lossy().into(),
            ..Default::default()
        },
        &eng,
        destination,
        &Cancel::default(),
        |_, _| {},
    );
    results.push(serde_json::json!({"input":"tiny-original.txt","expected":"unchanged","success":no_gain.as_ref().is_ok_and(|(p,s)| p == Path::new(&tiny_input.path) && *s == 1),"error":no_gain.as_ref().err()}));
    let strengths_work = sizes.len() == 3 && sizes[0] > sizes[1] && sizes[1] > sizes[2];
    // Regression: an already low-bitrate 720p MP4 must get smaller budgets,
    // not a resolution-only estimate several times larger than its source.
    let video = destination.join("low-bitrate-720p.mp4");
    process::capture(
        &eng.executable("ffmpeg")?,
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=1062x720:rate=24:duration=8",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=8",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-b:v",
            "640k",
            "-c:a",
            "aac",
            "-b:a",
            "96k",
            "-shortest",
            &video.to_string_lossy(),
        ],
        destination,
    )?;
    let video_input = probing::inspect(&video, &eng)?;
    let source_before = fs::read(&video).map_err(|e| e.to_string())?;
    let mut previous_size = video_input.size;
    for preset in ["high", "balanced", "small"] {
        let plan = crate::planning::plan(
            &video_input,
            &ConversionOptions {
                mode: "smart".into(),
                preset: preset.into(),
                output_dir: destination.to_string_lossy().into(),
                ..Default::default()
            },
            &[],
        )?;
        let result = conversion::execute(
            &video_input,
            &plan.options,
            &eng,
            destination,
            &Cancel::default(),
            |_, _| {},
        );
        let valid = result.as_ref().is_ok_and(|(p, n)| {
            *p != video
                && *n < previous_size
                && *n >= plan.estimate.low
                && *n <= plan.estimate.high
                && probing::inspect(p, &eng).is_ok_and(|f| {
                    f.width == video_input.width
                        && f.height == video_input.height
                        && f.media.audio_streams == video_input.media.audio_streams
                        && (f.duration.unwrap_or(0.) - video_input.duration.unwrap_or(0.)).abs()
                            < 0.2
                })
        });
        if let Ok((_, n)) = result.as_ref() {
            previous_size = *n;
        }
        results.push(serde_json::json!({"test":"low bitrate video compression", "preset":preset,
            "inputBytes":video_input.size,"outputBytes":result.as_ref().ok().map(|(_,s)|s),
            "estimate":plan.estimate,"success":valid && fs::read(&video).ok().as_ref() == Some(&source_before),"error":result.as_ref().err()}));
    }
    let image_input = probing::inspect(&image, &eng)?;
    let preview = crate::previews::render(&image_input, 0., &eng, destination);
    results.push(serde_json::json!({"test":"actual image preview","success":preview.as_ref().is_ok_and(|s|s.starts_with("data:image/png;base64,iVBOR")),"error":preview.as_ref().err()}));
    let measured = crate::estimation::refine(
        &image_input,
        &ConversionOptions::default(),
        &eng,
        destination,
    );
    results.push(serde_json::json!({"test":"measured image estimate","success":measured.as_ref().is_ok_and(|e|e.low>0 && e.high>=e.low),"error":measured.as_ref().err()}));
    let zero = destination.join("invalid-empty.png");
    fs::write(&zero, []).map_err(|e| e.to_string())?;
    results.push(serde_json::json!({"test":"empty input rejected","success":probing::inspect(&zero,&eng).is_err()}));
    let q = crate::queue::Queue::new(eng.root.clone(), destination.join("queue-state"));
    q.control("pause", None)?;
    let opts = ConversionOptions {
        format: "jpg".into(),
        output_dir: destination.to_string_lossy().into(),
        ..Default::default()
    };
    let first = q.add(image_input.clone(), opts.clone())?;
    let second = q.add(image_input, opts)?;
    q.reorder(vec![second, first])?;
    q.control("resume", None)?;
    let started = std::time::Instant::now();
    while q
        .snapshot()
        .jobs
        .iter()
        .any(|j| ["pending", "processing"].contains(&j.status.as_str()))
        && started.elapsed().as_secs() < 90
    {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let snap = q.snapshot();
    results.push(serde_json::json!({"test":"batch reorder resume and name conflicts","success":snap.jobs.len()==2 && snap.jobs.iter().all(|j|j.status=="completed") && snap.jobs[0].output!=snap.jobs[1].output}));
    let success = results.iter().all(|r| r["success"] == true)
        && health.iter().all(|e| e.ready)
        && strengths_work;
    let report =
        serde_json::json!({"success":success,"engines":health,"hardware":hardware,"tests":results});
    fs::write(
        destination.join("smoke-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    if success {
        Ok(())
    } else {
        Err("Smoke test failed; see smoke-report.json".into())
    }
}
