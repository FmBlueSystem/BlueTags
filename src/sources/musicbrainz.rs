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
    #[serde(rename = "first-release-date")]
    first_release_date: Option<String>,
    genres: Option<Vec<MbGenre>>,
    tags: Option<Vec<MbTag>>,
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

async fn search_recording(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Result<Option<MbRecording>> {
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

    // Entre los resultados con score >= 70:
    // 1. Priorizar recordings que tengan genre/tags (datos útiles)
    // 2. Entre esos, elegir el de fecha más antigua (evitar re-releases)
    let candidates: Vec<_> = resp
        .recordings
        .into_iter()
        .filter(|r| r.score.unwrap_or(0) >= 70)
        .collect();

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

    // Primero intentar candidatos con datos de género, luego el resto
    let best = candidates
        .iter()
        .filter(|r| has_genre_data(r))
        .min_by_key(|r| parse_year(r))
        .or_else(|| candidates.iter().min_by_key(|r| parse_year(r)))
        .cloned();

    Ok(best)
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
        .map(|g| g.name.clone())
        .or_else(|| {
            rec.tags
                .as_ref()
                .and_then(|t| t.iter().max_by_key(|x| x.count.unwrap_or(0)))
                .map(|t| t.name.clone())
        });

    let subgenre = rec
        .genres
        .as_ref()
        .and_then(|g| {
            if g.len() > 1 {
                g.iter()
                    .filter(|x| x.count.unwrap_or(0) > 0)
                    .nth(1)
                    .map(|g| g.name.clone())
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
    let tag_dominance = {
        let all_counts: Vec<u32> = rec.genres.as_ref()
            .map(|g| g.iter().filter_map(|x| x.count).collect())
            .or_else(|| rec.tags.as_ref().map(|t| t.iter().filter_map(|x| x.count).collect()))
            .unwrap_or_default();
        let total: u32 = all_counts.iter().sum();
        let top = all_counts.into_iter().max().unwrap_or(0);
        if total > 0 { (top as f32 / total as f32).clamp(0.5, 1.0) } else { 0.7 }
    };
    let search_score = rec.score.unwrap_or(70) as f32 / 100.0;

    Some(SourceResult {
        source: SourceName::MusicBrainz,
        year,
        genre,
        subgenre,
        confidence: search_score * tag_dominance,
    })
}

fn escape_lucene(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace(':', "\\:")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
