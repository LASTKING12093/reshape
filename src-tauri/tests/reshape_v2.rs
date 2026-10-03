use forma_lib::{
    conversion,
    engines::EngineManager,
    estimation,
    models::*,
    planning,
    presets::{self, Preset},
    previews, probing,
    process::Cancel,
    queue::Queue,
};
use std::{fs, path::PathBuf};
fn engines() -> EngineManager {
    EngineManager::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries"))
}
fn file(name: &str) -> InputFile {
    probing::inspect(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-fixtures")
            .join(name),
        &engines(),
    )
    .unwrap()
}
#[test]
fn hdr_is_blocked_before_transcoding() {
    let mut f = file("video.mp4");
    f.media.hdr = true;
    let o = ConversionOptions {
        mode: "smart".into(),
        ..Default::default()
    };
    assert!(planning::plan(&f, &o, &[])
        .unwrap_err_string()
        .contains("HDR"));
}
trait PlanError {
    fn unwrap_err_string(self) -> String;
}
impl PlanError for Result<planning::Plan, String> {
    fn unwrap_err_string(self) -> String {
        match self {
            Ok(_) => panic!("expected rejection"),
            Err(e) => e,
        }
    }
}
#[test]
fn remux_preserves_video_codec_and_duration() {
    let f = file("video.mp4");
    let temp = tempfile::tempdir().unwrap();
    let o = ConversionOptions {
        format: "mkv".into(),
        remux: true,
        subtitles: true,
        output_dir: temp.path().to_string_lossy().into(),
        ..Default::default()
    };
    let p = planning::plan(&f, &o, &[]).unwrap();
    assert!(planning::remux_possible(&f, &p.options));
    let (path, _) = conversion::execute(
        &f,
        &p.options,
        &engines(),
        temp.path(),
        &Cancel::default(),
        |_, _| {},
    )
    .unwrap();
    let out = probing::inspect(&path, &engines()).unwrap();
    assert_eq!(f.media.codec, out.media.codec);
    assert!((f.duration.unwrap() - out.duration.unwrap()).abs() < 0.2);
}
#[test]
fn target_budget_includes_every_audio_stream() {
    let mut f = file("video.mp4");
    f.duration = Some(60.);
    f.media.audio_streams = 3;
    let o = ConversionOptions {
        format: "mp4".into(),
        target_bytes: Some(2_000_000),
        audio_bitrate: 128,
        ..Default::default()
    };
    assert!(planning::plan(&f, &o, &[]).is_err());
}
#[test]
fn real_image_preview_and_measured_estimate() {
    let f = file("photo.jpg");
    let temp = tempfile::tempdir().unwrap();
    let preview = previews::render(&f, 0., &engines(), temp.path()).unwrap();
    assert!(preview.starts_with("data:image/png;base64,iVBOR"));
    let o = ConversionOptions {
        format: "webp".into(),
        mode: "custom".into(),
        ..Default::default()
    };
    let measured = estimation::refine(&f, &o, &engines(), temp.path()).unwrap();
    assert_eq!(measured.method, "measured");
    assert!(measured.low > 0);
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
#[test]
fn transparent_probe_and_animation_guard() {
    let mut f = file("imagem ação 日本.png");
    f.media.frames = Some(2);
    assert!(planning::plan(
        &f,
        &ConversionOptions {
            format: "jpg".into(),
            ..Default::default()
        },
        &[]
    )
    .is_err());
}
#[test]
fn invalid_suffix_and_secret_free_presets() {
    let o = ConversionOptions {
        suffix: "../oops".into(),
        ..Default::default()
    };
    assert!(conversion::validate_options(&o).is_err());
    let p = Preset {
        id: "a".into(),
        name: " Music ".into(),
        category: "audio".into(),
        favorite: true,
        options: ConversionOptions {
            password: "secret".into(),
            output_dir: "private".into(),
            merge_paths: vec!["private".into()],
            width: Some(800),
            ..Default::default()
        },
    };
    let p = presets::sanitize(vec![p]).unwrap();
    assert!(p[0].options.password.is_empty());
    assert!(p[0].options.output_dir.is_empty());
    assert!(p[0].options.merge_paths.is_empty());
    assert!(p[0].options.width.is_none());
    assert_eq!(p[0].name, "Music");
}
#[test]
fn queue_recovers_paused_and_reorders() {
    let temp = tempfile::tempdir().unwrap();
    let q = Queue::new(engines().root, temp.path().into());
    q.control("pause", None).unwrap();
    let f = file("photo.jpg");
    let a = q.add(f.clone(), ConversionOptions::default()).unwrap();
    let b = q.add(f, ConversionOptions::default()).unwrap();
    q.reorder(vec![b.clone(), a.clone()]).unwrap();
    assert!(q.reorder(vec![a.clone(), a.clone()]).is_err());
    let recovered = Queue::new(engines().root, temp.path().into());
    assert!(recovered.snapshot().paused);
    assert_eq!(recovered.snapshot().jobs[0].id, b);
    recovered.control("cancelAll", None).unwrap();
    assert!(recovered
        .snapshot()
        .jobs
        .iter()
        .all(|j| j.status == "cancelled"));
}
#[test]
fn gpu_failure_falls_back_to_cpu() {
    let f = file("video.mp4");
    let tmp = tempfile::tempdir().unwrap();
    let o = ConversionOptions {
        format: "mp4".into(),
        encoder: "h264_nvenc".into(),
        output_dir: tmp.path().to_string_lossy().into(),
        ..Default::default()
    };
    let (path, size) = conversion::execute(
        &f,
        &o,
        &engines(),
        tmp.path(),
        &Cancel::default(),
        |_, _| {},
    )
    .unwrap();
    assert!(size > 0);
    assert_eq!(
        probing::inspect(&path, &engines())
            .unwrap()
            .media
            .codec
            .as_deref(),
        Some("h264")
    );
}
#[test]
fn atomic_publish_does_not_replace_original() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    fs::write(&source, b"new content").unwrap();
    fs::write(tmp.path().join("output.txt"), b"original").unwrap();
    let out =
        forma_lib::output::publish_file(&source, tmp.path(), "output", "txt", &Cancel::default())
            .unwrap();
    assert_eq!(
        fs::read(tmp.path().join("output.txt")).unwrap(),
        b"original"
    );
    assert_eq!(out.file_name().unwrap(), "output (1).txt");
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 3);
}
