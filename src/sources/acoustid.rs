use crate::models::{SourceName, SourceResult, TrackMetadata};
use crate::rate_limit::Limiter;
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusty_chromaprint::{Configuration, Fingerprinter};
use serde::Deserialize;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const ACOUSTID_BASE: &str = "https://api.acoustid.org/v2";
const MAX_DECODE_SECS: usize = 120; // 2 minutos para fingerprint

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct AcoustIdResponse {
    status: String,
    results: Option<Vec<AcoustIdResult>>,
}

#[derive(Deserialize)]
struct AcoustIdResult {
    score: f64,
    recordings: Option<Vec<AcoustIdRecording>>,
}

#[derive(Deserialize)]
struct AcoustIdRecording {
    id: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn analyze(
    track: &TrackMetadata,
    client: &reqwest::Client,
    limiter: &Limiter,
    api_key: &str,
) -> Option<SourceResult> {
    let (fingerprint, duration) = compute_fingerprint(&track.path).ok()?;

    limiter.until_ready().await;

    let mbid = lookup_acoustid(client, &fingerprint, duration, api_key)
        .await
        .ok()??;

    Some(SourceResult {
        source: SourceName::AcoustId,
        year: None,
        genre: None,
        subgenre: None,
        confidence: 0.90,
        mbid: Some(mbid),
    })
}

// ---------------------------------------------------------------------------
// Fingerprinting con symphonia + rusty-chromaprint
// ---------------------------------------------------------------------------

fn compute_fingerprint(path: &Path) -> Result<(String, u32)> {
    let (samples, sample_rate, channels) = decode_audio(path)?;
    let duration = samples.len() as u32 / (sample_rate * channels);

    let config = Configuration::preset_test1();
    let mut printer = Fingerprinter::new(&config);
    printer.start(sample_rate, channels)?;

    for chunk in samples.chunks(4096) {
        printer.consume(chunk);
    }
    printer.finish();

    let fp_data = printer.fingerprint();

    // Convertir Vec<u32> → bytes → base64url
    let bytes: Vec<u8> = fp_data
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let encoded = URL_SAFE_NO_PAD.encode(&bytes);

    Ok((encoded, duration))
}

fn decode_audio(path: &Path) -> Result<(Vec<i16>, u32, u32)> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| anyhow::anyhow!("Sin pista de audio"))?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u32)
        .unwrap_or(2);
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let max_samples = (sample_rate * channels) as usize * MAX_DECODE_SECS;
    let mut samples: Vec<i16> = Vec::with_capacity(max_samples.min(1024 * 1024));

    loop {
        if samples.len() >= max_samples {
            break;
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let mut buf = SampleBuffer::<i16>::new(decoded.capacity() as u64, *decoded.spec());
        buf.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buf.samples());
    }

    Ok((samples, sample_rate, channels))
}

// ---------------------------------------------------------------------------
// AcoustID API lookup
// ---------------------------------------------------------------------------

async fn lookup_acoustid(
    client: &reqwest::Client,
    fingerprint: &str,
    duration: u32,
    api_key: &str,
) -> Result<Option<String>> {
    let params = [
        ("client", api_key),
        ("fingerprint", fingerprint),
        ("duration", &duration.to_string()),
        ("meta", "recordings"),
    ];

    let resp = client
        .post(format!("{ACOUSTID_BASE}/lookup"))
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json::<AcoustIdResponse>()
        .await?;

    if resp.status != "ok" {
        return Ok(None);
    }

    let mbid = resp
        .results
        .and_then(|results| {
            results
                .into_iter()
                .filter(|r| r.score > 0.8)
                .next()
        })
        .and_then(|r| r.recordings)
        .and_then(|recs| recs.into_iter().next())
        .map(|rec| rec.id);

    Ok(mbid)
}
