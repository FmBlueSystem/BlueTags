use crate::models::{SourceName, SourceResult, TrackMetadata};
use crate::rate_limit::Limiter;
use anyhow::Result;
use serde::Deserialize;

const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const USER_AGENT: &str = "music-tagger/1.0 (fmolinam@gmail.com)";

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MbSearchResponse {
    recordings: Vec<MbRecording>,
}

#[derive(Deserialize, Clone)]
struct MbRecording {
    id: Option<String>,
    score: Option<u32>,
    title: Option<String>,
    #[serde(rename = "first-release-date")]
    first_release_date: Option<String>,
    genres: Option<Vec<MbGenre>>,
    tags: Option<Vec<MbTag>>,
}

/// Detecta si una grabación es un remix/edit/rework por el título.
fn is_remix_recording(rec: &MbRecording) -> bool {
    let title = rec.title.as_deref().unwrap_or("").to_lowercase();
    let remix_patterns = ["remix)", "mix)", "rework)", "edit)", "bootleg)", "version)", "re-edit)"];
    remix_patterns.iter().any(|p| title.contains(p))
}

#[derive(Deserialize, Clone)]
struct MbGenre {
    name: String,
    count: Option<u32>,
}

#[derive(Deserialize, Clone)]
struct MbTag {
    name: String,
    count: Option<u32>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Retorna (SourceResult, Option<MBID>) — el MBID se usa para lookup en AcousticBrainz.
pub async fn analyze(
    track: &TrackMetadata,
    client: &reqwest::Client,
    limiter: &Limiter,
) -> (Option<SourceResult>, Option<String>) {
    let (artist, title) = match (track.artist.as_deref(), track.title.as_deref()) {
        (Some(a), Some(t)) => (a, t),
        _ => return (None, None),
    };

    limiter.until_ready().await;

    match search_recording(client, artist, title).await {
        Ok(Some(rec)) => {
            let mbid = rec.id.clone();
            (build_result(rec), mbid)
        }
        Ok(None) => (None, None),
        Err(e) => {
            eprintln!("[musicbrainz] error: {e}");
            (None, None)
        }
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Extrae el artista primario: "Michael Jackson & Siedah Garrett" → "Michael Jackson"
/// Handles: " & ", " feat. ", " ft. ", " featuring ", " vs. ", " vs "
fn primary_artist(artist: &str) -> &str {
    for sep in [" & ", " feat. ", " ft. ", " featuring ", " vs. ", " vs "] {
        if let Some(idx) = artist.find(sep) {
            return artist[..idx].trim();
        }
    }
    artist
}

async fn search_recording(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Result<Option<MbRecording>> {
    // Intento 1: artista completo
    let candidates = mb_search(client, artist, title).await?;
    if !candidates.is_empty() {
        return Ok(pick_best(candidates));
    }

    // Intento 2: solo artista primario (antes de & / feat.)
    let primary = primary_artist(artist);
    if primary != artist {
        let candidates2 = mb_search(client, primary, title).await?;
        if !candidates2.is_empty() {
            return Ok(pick_best(candidates2));
        }
    }

    Ok(None)
}

async fn mb_search(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Result<Vec<MbRecording>> {
    let query = format!(
        "recording:\"{}\" AND artist:\"{}\"",
        escape_lucene(title),
        escape_lucene(artist)
    );

    let resp = client
        .get(format!("{MB_BASE}/recording"))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .query(&[("query", &query), ("limit", &"5".to_string()), ("fmt", &"json".to_string())])
        .send()
        .await?
        .error_for_status()?
        .json::<MbSearchResponse>()
        .await?;

    // Score >= 70, priorizar recordings con genre/tags
    let candidates: Vec<_> = resp
        .recordings
        .into_iter()
        .filter(|r| r.score.unwrap_or(0) >= 70)
        .collect();

    Ok(candidates)
}

fn pick_best(candidates: Vec<MbRecording>) -> Option<MbRecording> {
    let has_genre_data = |r: &MbRecording| -> bool {
        r.genres.as_ref().map_or(false, |g| !g.is_empty())
            || r.tags.as_ref().map_or(false, |t| !t.is_empty())
    };

    let parse_year = |r: &MbRecording| -> u32 {
        r.first_release_date
            .as_deref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse::<u32>().ok())
            .unwrap_or(u32::MAX)
    };

    // Separar originales de remixes — preferir originales
    let (originals, remixes): (Vec<_>, Vec<_>) = candidates.iter()
        .partition(|r| !is_remix_recording(r));

    let pool = if !originals.is_empty() { &originals } else { &remixes };

    pool.iter()
        .filter(|r| has_genre_data(r))
        .min_by_key(|r| parse_year(r))
        .or_else(|| pool.iter().min_by_key(|r| parse_year(r)))
        .cloned()
        .cloned()
}

fn build_result(rec: MbRecording) -> Option<SourceResult> {
    let year = rec
        .first_release_date
        .as_deref()
        .and_then(|d| d.split('-').next())
        .and_then(|y| y.parse::<u32>().ok());

    // Preferir géneros (editorial) sobre tags (crowdsource)
    let genre = rec
        .genres
        .as_ref()
        .and_then(|g| g.iter().max_by_key(|x| x.count.unwrap_or(0)))
        .map(|g| title_case_genre(&g.name))
        .or_else(|| {
            rec.tags
                .as_ref()
                .and_then(|t| t.iter().max_by_key(|x| x.count.unwrap_or(0)))
                .map(|t| title_case_genre(&t.name))
        });

    let subgenre = rec
        .genres
        .as_ref()
        .and_then(|g| {
            if g.len() > 1 {
                g.iter()
                    .filter(|x| x.count.unwrap_or(0) > 0)
                    .nth(1)
                    .map(|g| title_case_genre(&g.name))
            } else {
                None
            }
        });

    let has_data = year.is_some() || genre.is_some();
    if !has_data {
        return None;
    }

    // Confianza dinámica: score del search × dominancia del top tag
    // Si el top genre/tag tiene count=10 y el total es 15, dominance=0.67
    let (tag_dominance, total_genre_votes) = {
        let all_counts: Vec<u32> = rec.genres.as_ref()
            .map(|g| g.iter().filter_map(|x| x.count).collect())
            .or_else(|| rec.tags.as_ref().map(|t| t.iter().filter_map(|x| x.count).collect()))
            .unwrap_or_default();
        let total: u32 = all_counts.iter().sum();
        let top = all_counts.into_iter().max().unwrap_or(0);
        let dominance = if total > 0 { (top as f32 / total as f32).clamp(0.5, 1.0) } else { 0.7 };
        (dominance, total)
    };
    let search_score = rec.score.unwrap_or(70) as f32 / 100.0;

    // Penalización por baja evidencia de género en MB.
    // MB frecuentemente etiqueta recordings con el género del ARTISTA (no del track).
    // Si hay muy pocos votos (<= 3), la confianza de género se reduce para que
    // otras fuentes track-específicas (WikiSong, AcousticBrainz, Discogs) puedan ganar.
    let genre_confidence_penalty = if genre.is_some() && total_genre_votes <= 3 {
        0.70 // reduce a 70% cuando hay poca evidencia track-específica
    } else {
        1.0
    };

    Some(SourceResult {
        source: SourceName::MusicBrainz,
        year,
        genre,
        subgenre,
        confidence: search_score * tag_dominance * genre_confidence_penalty,
        mbid: None,
    })
}

fn escape_lucene(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace(':', "\\:")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Normaliza un género/subgénero a Title Case.
/// "synth-pop" → "Synth-Pop", "dance-pop" → "Dance-Pop", "r&b" → "R&B"
pub fn title_case_genre(s: &str) -> String {
    if s.is_empty() { return s.to_string(); }
    // Casos conocidos que no siguen la regla genérica
    let known: &[(&str, &str)] = &[
        ("r&b", "R&B"),
        ("hi-nrg", "Hi-NRG"),
        ("hi nrg", "Hi-NRG"),
        ("dj mix", "DJ Mix"),
        ("electro-funk", "Electro Funk"),
        ("electro funk", "Electro Funk"),
        ("hip-house", "Hip-House"),
        ("hip house", "Hip-House"),
        ("old-school hip-hop", "Old-School Hip-Hop"),
        ("old school hip-hop", "Old-School Hip-Hop"),
        ("contemporary r&b", "Contemporary R&B"),
        ("synth-pop", "Synth-Pop"),
        ("synth pop", "Synth-Pop"),
        ("dance-pop", "Dance-Pop"),
        ("reggae-pop", "Reggae-Pop"),
        ("sophisti-pop", "Sophisti-Pop"),
        ("post-disco", "Post-Disco"),
    ];
    let lower = s.to_lowercase();
    if let Some(&(_, canonical)) = known.iter().find(|(k, _)| *k == lower.as_str()) {
        return canonical.to_string();
    }
    // Genérico: capitalizar primera letra de cada palabra separada por espacio o guión
    s.split_whitespace()
        .map(|word| {
            let mut result = String::new();
            let mut first_in_segment = true;
            for (i, c) in word.char_indices() {
                if c == '-' {
                    result.push(c);
                    first_in_segment = true;
                } else if first_in_segment {
                    result.extend(c.to_uppercase());
                    first_in_segment = false;
                } else {
                    result.push(c);
                }
                let _ = i;
            }
            result
        })
        .collect::<Vec<_>>()
        .join(" ")
}
