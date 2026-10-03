use forma_lib::{
    conversion,
    engines::EngineManager,
    models::ConversionOptions,
    probing,
    process::{self, Cancel},
};
use std::{fs, path::PathBuf};
fn engine() -> EngineManager {
    EngineManager::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries"))
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../test-fixtures")
        .join(name)
}
fn convert(source: &str, format: &str, lossless: bool) {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture(source), &eng).unwrap();
    let o = ConversionOptions {
        format: format.into(),
        lossless,
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let out =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    assert!(out.0.exists());
    assert!(out.1 > 0);
    assert_eq!(
        fs::read_dir(root.path()).unwrap().count(),
        1,
        "temporary files leaked"
    );
}
#[test]
fn png_jpg() {
    convert("imagem ação 日本.png", "jpg", false)
}

#[test]
fn compression_flac_14mb_reduces_and_strength_changes_size() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("compress-1.4mb.flac"), &eng).unwrap();
    assert!((1_300_000..1_600_000).contains(&input.size));
    let original = fs::read(&input.path).unwrap();
    let mut sizes = Vec::new();
    for preset in ["high", "balanced", "small"] {
        // Reproduce the old UI: smart compression with stale FLAC/lossless options.
        let options = ConversionOptions {
            format: "flac".into(),
            lossless: true,
            mode: "smart".into(),
            preset: preset.into(),
            output_dir: root.path().to_string_lossy().into(),
            ..Default::default()
        };
        let (path, size) = conversion::execute(
            &input,
            &options,
            &eng,
            root.path(),
            &Cancel::default(),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(path.extension().unwrap(), "mp3");
        assert!(
            size < input.size / 3,
            "Compression should make a substantial difference on this fixture"
        );
        sizes.push(size);
    }
    assert!(sizes[0] > sizes[1] && sizes[1] > sizes[2]);
    assert_eq!(fs::read(&input.path).unwrap(), original);
}

#[test]
fn compression_no_gain_keeps_original_without_copy() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("tiny.txt"), &eng).unwrap();
    let options = ConversionOptions {
        mode: "smart".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (path, size) = conversion::execute(
        &input,
        &options,
        &eng,
        root.path(),
        &Cancel::default(),
        |_, _| {},
    )
    .unwrap();
    assert_eq!(path, PathBuf::from(&input.path));
    assert_eq!(size, input.size);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn compression_lossless_flac_preserves_pcm() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("sound.flac"), &eng).unwrap();
    let options = ConversionOptions {
        mode: "lossless".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (path, size) = conversion::execute(
        &input,
        &options,
        &eng,
        root.path(),
        &Cancel::default(),
        |_, _| {},
    )
    .unwrap();
    assert!(size <= input.size);
    assert_eq!(path.extension().unwrap(), "flac");
}

#[test]
fn compression_invalid_target_has_actionable_error() {
    let eng = engine();
    let input = probing::inspect(&fixture("sound.flac"), &eng).unwrap();
    let options = ConversionOptions {
        mode: "target".into(),
        target_bytes: Some(0),
        ..Default::default()
    };
    assert!(conversion::compression_options(&input, &options)
        .unwrap_err()
        .contains("maior que zero"));
}

#[test]
fn compression_queue_reports_no_reduction() {
    let root = tempfile::tempdir().unwrap();
    let q = forma_lib::queue::Queue::new(engine().root, root.path().join("state"));
    let input = probing::inspect(&fixture("tiny.txt"), &q.manager).unwrap();
    let options = ConversionOptions {
        mode: "smart".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let id = q.add(input.clone(), options).unwrap();
    let start = std::time::Instant::now();
    loop {
        let snapshot = q.snapshot();
        let job = snapshot.jobs.iter().find(|j| j.id == id).unwrap();
        if !["pending", "processing"].contains(&job.status.as_str()) {
            assert_eq!(job.status, "unchanged", "{:?}", job.error);
            assert_eq!(job.output.as_deref(), Some(input.path.as_str()));
            break;
        }
        assert!(start.elapsed().as_secs() < 30);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
#[test]
fn jpg_png() {
    convert("photo.jpg", "png", true)
}
#[test]
fn png_webp() {
    convert("imagem ação 日本.png", "webp", true)
}
#[test]
fn reject_16bit_webp_lossless() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("16bit.png"), &eng).unwrap();
    assert_eq!(input.bit_depth, Some(16));
    let o = ConversionOptions {
        format: "webp".into(),
        lossless: true,
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    assert!(
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {})
            .unwrap_err()
            .contains("8-bit")
    );
}
#[test]
fn merge_images_pdf() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("photo.jpg"), &eng).unwrap();
    let o = ConversionOptions {
        format: "pdf".into(),
        merge_paths: vec![fixture("imagem ação 日本.png").to_string_lossy().into()],
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (pdf, _) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    let info = process::capture(
        &eng.executable("pdfinfo").unwrap(),
        &[&pdf.to_string_lossy()],
        root.path(),
    )
    .unwrap();
    assert!(info
        .lines()
        .any(|s| s.starts_with("Pages:") && s.trim().ends_with('2')));
}
#[test]
fn create_encrypted_archive_and_extract() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("photo.jpg"), &eng).unwrap();
    let o = ConversionOptions {
        format: "7z".into(),
        archive_action: "create".into(),
        password: "test espaços !@ 日本".into(),
        merge_paths: vec![fixture("tiny.txt").to_string_lossy().into()],
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (archive, _) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    let input = probing::inspect(&archive, &eng).unwrap();
    let o = ConversionOptions {
        archive_action: "extract".into(),
        merge_paths: vec![],
        ..o
    };
    let (folder, _) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    assert_eq!(
        fs::read(folder.join("photo.jpg")).unwrap(),
        fs::read(fixture("photo.jpg")).unwrap()
    );
    assert_eq!(fs::read_to_string(folder.join("tiny.txt")).unwrap(), "a");
}
#[test]
fn tar_gzip_repack() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("archive.zip"), &eng).unwrap();
    let o = ConversionOptions {
        format: "tar.gz".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (tar, _) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    let input = probing::inspect(&tar, &eng).unwrap();
    let opt = ConversionOptions {
        format: "7z".into(),
        archive_action: "extract".into(),
        ..o
    };
    let (folder, _) = conversion::execute(
        &input,
        &opt,
        &eng,
        root.path(),
        &Cancel::default(),
        |_, _| {},
    )
    .unwrap();
    assert!(folder.join("document.md").exists());
}
#[test]
fn video_two_pass() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("video.mp4"), &eng).unwrap();
    let o = ConversionOptions {
        format: "mp4".into(),
        target_bytes: Some(100_000),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (_, size) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    assert!(size > 30_000 && size < 160_000);
}
#[test]
fn scheduler_pause_cancel_resume_history() {
    use forma_lib::queue::Queue;
    let root = tempfile::tempdir().unwrap();
    let q = Queue::new(engine().root, root.path().join("state"));
    q.control("pause", None).unwrap();
    let input = probing::inspect(&fixture("photo.jpg"), &q.manager).unwrap();
    let opts = ConversionOptions {
        format: "png".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let id = q.add(input.clone(), opts.clone()).unwrap();
    assert_eq!(q.snapshot().jobs[0].status, "pending");
    q.control("cancel", Some(&id)).unwrap();
    assert_eq!(q.snapshot().jobs[0].status, "cancelled");
    let id2 = q.add(input, opts).unwrap();
    q.control("resume", None).unwrap();
    let start = std::time::Instant::now();
    loop {
        let state = q.snapshot();
        let job = state.jobs.iter().find(|j| j.id == id2).unwrap();
        if job.status == "completed" {
            assert!(state.history.iter().any(|h| h.id == id2));
            break;
        }
        assert_ne!(job.status, "failed", "{:?}", job.error);
        assert!(start.elapsed().as_secs() < 30);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
#[test]
fn wav_flac() {
    convert("som ação 日本.wav", "flac", true)
}
#[test]
fn wav_mp3() {
    convert("som ação 日本.wav", "mp3", false)
}
#[test]
fn flac_wav() {
    convert("sound.flac", "wav", true)
}
#[test]
fn mp4_webm() {
    convert("video.mp4", "webm", false)
}
#[test]
fn incompatible_codec_rejected() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("video.mp4"), &eng).unwrap();
    let o = ConversionOptions {
        format: "webm".into(),
        codec: "h265".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    assert!(
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {})
            .unwrap_err()
            .contains("incompatible")
    );
}
#[test]
fn mp4_mp3() {
    convert("video.mp4", "mp3", false)
}
#[test]
fn docx_pdf() {
    convert("document.docx", "pdf", false)
}
#[test]
fn md_html() {
    convert("document.md", "html", false)
}
#[test]
fn zip_7z() {
    convert("archive.zip", "7z", true)
}
#[test]
fn invalid_file() {
    assert!(probing::inspect(&fixture("invalid.png"), &engine()).is_err())
}
#[test]
fn misnamed_png() {
    assert_eq!(
        probing::inspect(&fixture("misnamed.dat"), &engine())
            .unwrap()
            .format,
        "png"
    )
}
#[test]
fn tiny_text() {
    convert("tiny.txt", "html", false)
}
#[test]
fn image_pdf() {
    convert("imagem ação 日本.png", "pdf", false)
}
#[test]
fn reject_incompatible() {
    let eng = engine();
    let input = probing::inspect(&fixture("photo.jpg"), &eng).unwrap();
    assert!(conversion::execute(
        &input,
        &ConversionOptions {
            format: "mp3".into(),
            ..Default::default()
        },
        &eng,
        &std::env::temp_dir(),
        &Cancel::default(),
        |_, _| {}
    )
    .is_err())
}
#[test]
fn cancel_real_process() {
    let eng = engine();
    let c = Cancel::default();
    let copy = c.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        copy.stop()
    });
    let start = std::time::Instant::now();
    let result = process::run(
        &eng.executable("ffmpeg").unwrap(),
        &[
            "-re",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=640x480:rate=30",
            "-t",
            "120",
            "-f",
            "null",
            "NUL",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
        &eng.root,
        &c,
        None,
        |_, _| {},
    );
    assert!(result.unwrap_err().contains("cancelled"));
    assert!(start.elapsed().as_secs() < 10)
}
#[test]
fn image_target() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("photo.jpg"), &eng).unwrap();
    let o = ConversionOptions {
        format: "jpg".into(),
        target_bytes: Some(900),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (_, size) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    assert!(size <= 900)
}
#[test]
fn impossible_lossless() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("sound.flac"), &eng).unwrap();
    let o = ConversionOptions {
        format: "flac".into(),
        lossless: true,
        target_bytes: Some(1),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    assert!(
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {})
            .unwrap_err()
            .contains("cannot be achieved")
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0)
}
#[test]
fn pdf_render_and_optimize() {
    let eng = engine();
    let root = tempfile::tempdir().unwrap();
    let input = probing::inspect(&fixture("document.docx"), &eng).unwrap();
    let o = ConversionOptions {
        format: "pdf".into(),
        output_dir: root.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (pdf, _) =
        conversion::execute(&input, &o, &eng, root.path(), &Cancel::default(), |_, _| {}).unwrap();
    let input = probing::inspect(&pdf, &eng).unwrap();
    for format in ["pdf", "png", "jpg"] {
        let opt = ConversionOptions {
            format: format.into(),
            output_dir: root.path().to_string_lossy().into(),
            ..Default::default()
        };
        let (out, size) = conversion::execute(
            &input,
            &opt,
            &eng,
            root.path(),
            &Cancel::default(),
            |_, _| {},
        )
        .unwrap();
        assert!(size > 0);
        assert!(out.exists());
    }
}
