use rayon::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use std::path::{Path, PathBuf};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(serde::Serialize, Debug, Clone)]
pub struct AudioFeatures {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocal_pct: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn analyze_batch(paths: &[PathBuf]) -> Vec<AudioFeatures> {
    paths.par_iter().map(|p| analyze(p)).collect()
}

fn analyze(path: &Path) -> AudioFeatures {
    let file_str = path.to_string_lossy().to_string();
    match analyze_inner(path) {
        Ok(feat) => feat,
        Err(e) => AudioFeatures {
            file: file_str,
            vocal_pct: None,
            brightness: None,
            error: Some(e.to_string()),
        },
    }
}

fn analyze_inner(path: &Path) -> anyhow::Result<AudioFeatures> {
    let file_str = path.to_string_lossy().to_string();

    let src = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
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
        .ok_or_else(|| anyhow::anyhow!("no audio track found"))?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2);
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs().make(
        &track.codec_params,
        &DecoderOptions::default(),
    )?;

    // Decode all packets, collect interleaved f32 samples
    let mut interleaved: Vec<f32> = Vec::new();

    loop {
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
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buf.copy_interleaved_ref(decoded);
        interleaved.extend_from_slice(buf.samples());
    }

    // Mix interleaved multi-channel to mono
    let num_channels = channels.max(1);
    let num_frames = interleaved.len() / num_channels;
    let mut samples: Vec<f32> = Vec::with_capacity(num_frames);
    for frame in 0..num_frames {
        let mut sum = 0.0f32;
        for ch in 0..num_channels {
            sum += interleaved[frame * num_channels + ch];
        }
        samples.push(sum / num_channels as f32);
    }

    const FFT_SIZE: usize = 65536;

    if samples.len() < FFT_SIZE {
        return Err(anyhow::anyhow!(
            "not enough samples: {} < {}",
            samples.len(),
            FFT_SIZE
        ));
    }

    // Take central FFT_SIZE samples
    let start = (samples.len() - FFT_SIZE) / 2;
    let window_samples = &samples[start..start + FFT_SIZE];

    // Apply Hann window and build complex input
    let mut windowed: Vec<Complex<f32>> = window_samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let hann = 0.5
                * (1.0
                    - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos());
            Complex::new(s * hann, 0.0)
        })
        .collect();

    // FFT
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    fft.process(&mut windowed);

    // Only use positive frequencies (first half)
    let half = FFT_SIZE / 2;
    let spectrum = &windowed[..half];

    let hz_per_bin = sample_rate as f32 / FFT_SIZE as f32;
    let vocal_lo = (300.0 / hz_per_bin) as usize;
    let vocal_hi = ((3400.0 / hz_per_bin) as usize).min(half - 1);
    let bright_lo = ((4000.0 / hz_per_bin) as usize).min(half - 1);

    let total_power: f32 = spectrum.iter().map(|c| c.norm_sqr()).sum();

    if total_power < 1e-10 {
        return Err(anyhow::anyhow!("silent or near-silent file"));
    }

    let vocal_power: f32 = spectrum[vocal_lo..=vocal_hi]
        .iter()
        .map(|c| c.norm_sqr())
        .sum();
    let bright_power: f32 = spectrum[bright_lo..].iter().map(|c| c.norm_sqr()).sum();

    Ok(AudioFeatures {
        file: file_str,
        vocal_pct: Some((vocal_power / total_power).sqrt()),
        brightness: Some((bright_power / total_power).sqrt()),
        error: None,
    })
}
