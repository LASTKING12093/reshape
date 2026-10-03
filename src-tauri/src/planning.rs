//! A single, local planning boundary shared by estimates and execution.
use crate::{conversion, engines, formats, models::*};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub low: u64,
    pub high: u64,
    pub method: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub options: ConversionOptions,
    pub estimate: Estimate,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

pub fn remux_possible(f: &InputFile, o: &ConversionOptions) -> bool {
    f.category == "video"
        && o.mode == "convert"
        && o.remux
        && o.width.is_none()
        && o.height.is_none()
        && o.fps.is_none()
        && !o.normalize
        && o.target_bytes.is_none()
        && (o.format == "mkv"
            || (["mp4", "mov"].contains(&o.format.as_str())
                && matches!(
                    f.media.codec.as_deref(),
                    Some("h264" | "hevc" | "av1" | "mpeg4")
                )
                && matches!(
                    f.media.audio_codec.as_deref(),
                    None | Some("aac" | "mp3" | "alac" | "ac3")
                )
                && f.media.subtitle_codecs.iter().all(|s| s == "mov_text")))
}
pub fn plan(f: &InputFile, o: &ConversionOptions, hardware: &[String]) -> Result<Plan, String> {
    let mut o = conversion::compression_options(f, o)?;
    conversion::validate_options(&o)?;
    let cap = formats::validate(&f.format, &o.format, o.lossless)?;
    let mut warnings = Vec::new();
    if f.category == "video"
        && !["mp4", "mov", "mkv", "avi", "webm", "zip", "7z"].contains(&o.format.as_str())
    {
        warnings.push("Somente a primeira faixa de áudio será extraída; vídeo e demais faixas não entram no resultado.".into());
    }
    if o.format == "mp3" && o.channels.map(u32::from).or(f.channels).unwrap_or(2) > 2 {
        return Err("MP3 aceita até dois canais. Em Avançado, escolha 2 canais ou use FLAC para preservar o áudio multicanal.".into());
    }
    if o.format == "mp3" && !(32..=320).contains(&o.bitrate) {
        return Err("MP3: escolha bitrate de 32 a 320 kbps.".into());
    }
    if o.encoder == "auto" {
        o.encoder = if ["auto", "h264"].contains(&o.codec.as_str()) && o.target_bytes.is_none() {
            hardware.first().cloned().unwrap_or("software".into())
        } else {
            "software".into()
        };
    }
    if o.remux && !remux_possible(f, &o) {
        return Err(
            "Remux incompatível. Use MKV sem redimensionar ou desative a cópia de fluxos.".into(),
        );
    }
    if f.category == "video"
        && ["mp4", "mov", "mkv", "avi", "webm"].contains(&o.format.as_str())
        && !remux_possible(f, &o)
    {
        if f.media.hdr {
            return Err("Este vídeo é HDR. Use Converter → MKV → Copiar fluxos para preservar HDR. A compressão HDR ainda não está disponível.".into());
        }
        if !f.media.subtitle_codecs.is_empty() && !o.subtitles {
            warnings
                .push("As legendas serão removidas. Ative Preservar legendas em Avançado.".into());
        }
        if o.subtitles
            && o.format != "mkv"
            && f.media
                .subtitle_codecs
                .iter()
                .any(|s| !["mov_text", "subrip", "ass", "webvtt", "text"].contains(&s.as_str()))
        {
            return Err("Legendas gráficas exigem saída MKV para preservação.".into());
        }
        warnings.push("Compressão com perdas. O arquivo original é preservado.".into());
    }
    if f.media.frames.unwrap_or(1) > 1
        && cap.engine == "imagemagick"
        && !["gif", "webp", "tiff", "pdf"].contains(&o.format.as_str())
    {
        return Err("Imagem animada ou multipágina: escolha GIF, WebP, TIFF ou PDF para preservar todos os quadros.".into());
    }
    if f.media.alpha && o.format == "jpg" {
        warnings.push("JPEG remove transparência usando a cor de fundo escolhida.".into());
    }
    if f.category == "audio" && !["flac", "wav", "aiff"].contains(&o.format.as_str()) {
        warnings
            .push("Áudio com perdas. Volume permanece igual, salvo normalização explícita.".into());
    }
    if f.format == "pdf" && o.format == "pdf" {
        warnings.push("Otimiza estrutura e imagens, preservando texto e vetores. PDFs já otimizados podem não diminuir.".into());
    }
    if o.target_bytes.is_some() && ["video", "audio"].contains(&f.category.as_str()) {
        let video = ["mp4", "mov", "mkv", "avi", "webm"].contains(&o.format.as_str());
        let rate = engines::target_bitrate(
            o.target_bytes.unwrap(),
            f.duration.ok_or("Duração indisponível")?,
            if video {
                o.audio_bitrate.saturating_mul(f.media.audio_streams)
            } else {
                0
            },
        )?;
        if !video && o.format == "mp3" && !(32..=320).contains(&rate) {
            return Err("Tamanho alvo fora da faixa MP3 (32–320 kbps). Ajuste o tamanho ou escolha outro formato.".into());
        }
        if o.mode != "smart" {
            warnings.push("Tamanho alvo aproximado; inclui reserva de 3% para o contêiner.".into());
        }
    }
    // Validate container/codec combinations before a job is submitted.
    if cap.engine != "7zip" {
        engines::build(
            f,
            &o,
            std::path::Path::new("source"),
            std::path::Path::new("result"),
            std::path::Path::new("."),
        )?;
    }
    let estimate = crate::estimation::estimate(f, &o);
    if estimate.low >= f.size && o.mode != "convert" {
        warnings.push(
            "A saída provavelmente não será menor. Se isso ocorrer, o original será mantido."
                .into(),
        );
    }
    let preferred: &[&str] = match f.category.as_str() {
        "image" => &["webp", "avif", "jpg", "png"],
        "audio" => &["flac", "mp3", "m4a", "opus"],
        "video" => &["mp4", "mkv", "webm"],
        _ => &["pdf", "docx", "zip", "7z"],
    };
    let available = formats::capability(&f.format)?;
    let recommendations = preferred
        .iter()
        .filter(|s| available.outputs.iter().any(|o| o == **s))
        .map(|s| s.to_string())
        .collect();
    Ok(Plan {
        options: o,
        estimate,
        warnings,
        recommendations,
    })
}
