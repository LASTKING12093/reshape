use crate::{
    models::*,
    planning::{remux_possible, Estimate},
};
pub static PREVIEW_WORKER: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub fn refine(
    f: &InputFile,
    o: &ConversionOptions,
    eng: &crate::engines::EngineManager,
    root: &std::path::Path,
) -> Result<Estimate, String> {
    let _lock = PREVIEW_WORKER
        .lock()
        .map_err(|_| "Preview worker unavailable")?;
    if f.category != "image"
        || f.size > 40 * 1024 * 1024
        || f.media.frames.unwrap_or(1) > 1
        || !["jpg", "webp", "avif", "png"].contains(&o.format.as_str())
        || o.target_bytes.is_some()
    {
        return Err("No refinement for this plan".into());
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let tmp = tempfile::Builder::new()
        .prefix("reshape-estimate-")
        .tempdir_in(root)
        .map_err(|e| e.to_string())?;
    let src = tmp.path().join(format!("source.{}", f.format));
    if std::fs::hard_link(&f.path, &src).is_err() {
        std::fs::copy(&f.path, &src).map_err(|e| e.to_string())?;
    }
    let dest = tmp.path().join(format!("result.{}", o.format));
    let plan = crate::engines::build(f, o, &src, &dest, tmp.path())?;
    crate::process::run_limited(
        &eng.executable(&plan.engine)?,
        &plan.args,
        tmp.path(),
        &crate::process::Cancel::default(),
        None,
        12,
        |_, _| {},
    )?;
    let size = std::fs::metadata(dest).map_err(|e| e.to_string())?.len();
    Ok(retained_estimate(
        f,
        o,
        Estimate {
            low: (size as f64 * 0.98) as u64,
            high: (size as f64 * 1.02) as u64,
            method: "measured".into(),
        },
    ))
}
pub fn estimate(f: &InputFile, o: &ConversionOptions) -> Estimate {
    retained_estimate(f, o, estimate_candidate(f, o))
}
// Execution discards any larger compression candidate. Predict that actual
// retained result, including measured images, without promising a fake saving.
fn retained_estimate(f: &InputFile, o: &ConversionOptions, mut e: Estimate) -> Estimate {
    if o.mode != "convert" {
        if e.low >= f.size {
            return Estimate {
                low: f.size,
                high: f.size,
                method: "original".into(),
            };
        }
        e.high = e.high.min(f.size);
    }
    e
}
fn estimate_candidate(f: &InputFile, o: &ConversionOptions) -> Estimate {
    let range = |value: f64, spread: f64, method: &str| Estimate {
        low: (value * (1. - spread)).max(1.) as u64,
        high: (value * (1. + spread)).max(1.) as u64,
        method: method.into(),
    };
    if remux_possible(f, o) {
        return range(f.size as f64, 0.03, "remux");
    }
    if let Some(target) = o.target_bytes {
        return range(target as f64, 0.08, "target");
    }
    if let Some(duration) = f.duration.filter(|v| *v > 0.) {
        if ["mp3", "aac", "m4a", "ogg", "opus", "wma", "ac3"].contains(&o.format.as_str()) {
            let vbr = o.format == "mp3" && o.mode == "convert" && o.preset != "custom";
            let rate = if vbr {
                if o.quality >= 85 {
                    245.
                } else if o.quality >= 65 {
                    175.
                } else {
                    100.
                }
            } else {
                o.bitrate as f64
            };
            return range(
                duration * rate * 125. + 16384.,
                if vbr { 0.3 } else { 0.08 },
                if vbr { "vbr" } else { "bitrate" },
            );
        }
        if ["wav", "aiff"].contains(&o.format.as_str()) {
            let rate = o.sample_rate.unwrap_or_else(|| {
                f.sample_rate
                    .as_deref()
                    .unwrap_or("44100")
                    .parse()
                    .unwrap_or(44100)
            });
            let depth = o
                .bit_depth
                .map(u32::from)
                .unwrap_or(f.bit_depth.unwrap_or(16))
                .max(16);
            return range(
                duration
                    * rate as f64
                    * o.channels.map(u32::from).unwrap_or(f.channels.unwrap_or(2)) as f64
                    * depth as f64
                    / 8.
                    + 4096.,
                0.01,
                "pcm",
            );
        }
        if ["mp4", "mov", "mkv", "webm", "avi"].contains(&o.format.as_str()) {
            let pixels = o.width.or(f.width).unwrap_or(1920) as f64
                * o.height.or(f.height).unwrap_or(1080) as f64;
            let bitrate = pixels
                * o.fps.map(f64::from).or(f.media.fps).unwrap_or(30.)
                * 0.085
                * 2f64.powf((23. - o.crf as f64) / 6.);
            return range(
                duration
                    * (bitrate + o.audio_bitrate as f64 * 1000. * f.media.audio_streams as f64)
                    / 8.,
                0.55,
                "heuristic",
            );
        }
    }
    let ratio = if o.lossless {
        0.9
    } else if f.category == "image" {
        match o.preset.as_str() {
            "high" => 0.7,
            "small" => 0.3,
            _ => 0.5,
        }
    } else if f.format == "pdf" {
        0.75
    } else {
        0.85
    };
    range(f.size as f64 * ratio, 0.6, "heuristic")
}
#[cfg(test)]
mod tests {
    use super::*;
    fn file() -> InputFile {
        serde_json::from_value(serde_json::json!({"path":"a.flac","name":"a.flac","format":"flac","category":"audio","size":1400000,"duration":12,"width":null,"height":null,"sampleRate":"44100","channels":2,"bitDepth":16})).unwrap()
    }
    #[test]
    fn cbr_tracks_duration() {
        let f = file();
        let o = ConversionOptions {
            format: "mp3".into(),
            mode: "smart".into(),
            bitrate: 128,
            ..Default::default()
        };
        let e = estimate(&f, &o);
        assert!(e.low < 208384 && e.high > 208384);
        assert!(e.high < f.size);
    }
    #[test]
    fn pcm_can_grow() {
        let f = file();
        let e = estimate(
            &f,
            &ConversionOptions {
                format: "wav".into(),
                ..Default::default()
            },
        );
        assert!(e.low > f.size);
    }
    #[test]
    fn old_history_migrates() {
        assert_eq!(file().media.audio_streams, 0);
    }
    #[test]
    fn compression_predicts_retained_original_not_larger_candidate() {
        let f = file();
        let o = ConversionOptions {
            mode: "custom".into(),
            format: "wav".into(),
            ..Default::default()
        };
        let e = estimate(&f, &o);
        assert_eq!((e.low, e.high), (f.size, f.size));
        assert_eq!(e.method, "original");
    }
    #[test]
    fn reported_low_bitrate_video_has_source_relative_budgets() {
        let mut f = file();
        f.category = "video".into();
        f.format = "mp4".into();
        f.size = 8_074_035;
        f.duration = Some(82.);
        f.width = Some(1062);
        f.height = Some(720);
        f.media.fps = Some(30.);
        f.media.audio_streams = 1;
        let mut last = f.size;
        for preset in ["high", "balanced", "small"] {
            let p = crate::planning::plan(
                &f,
                &ConversionOptions {
                    mode: "smart".into(),
                    preset: preset.into(),
                    ..Default::default()
                },
                &[],
            )
            .unwrap();
            assert!(p.estimate.high < last);
            last = p.estimate.low;
            let again = crate::conversion::compression_options(&f, &p.options).unwrap();
            assert_eq!(p.options.target_bytes, again.target_bytes);
            assert_eq!(p.options.audio_bitrate, again.audio_bitrate);
        }
        assert!(crate::planning::plan(
            &f,
            &ConversionOptions {
                mode: "target".into(),
                target_bytes: Some(f.size + 1),
                ..Default::default()
            },
            &[]
        )
        .is_err());
    }
}
