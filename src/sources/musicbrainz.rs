use crate::models::{SourceName, SourceResult, TrackMetadata};
use crate::rate_limit::Limiter;
use anyhow::Result;
use serde::Deserialize;

const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const USER_AGENT: &str = "music-tagger/1.0 (music-tagger@example.com)";

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MbSearchResponse {
    recordings: Vec<MbRecording>,
}

#[derive(Deserialize)]
struct MbRecording {
    score: Option<u32>,
    #[serde(rename = "first-release-date")]
    first_release_date: Option<String>,
    genres: Option<Vec<MbGenre>>,
    tags: Option<Vec<MbTag>>,
}

#[derive(Deserialize)]
struct MbGenre {
    name: String,
    count: Option<u32>,
}

#[derive(Deserialize)]
struct MbTag {
    name: String,
    count: Option<u32>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn analyze(
    track: &TrackMetadata,
    client: &reqwest::Client,
    limiter: &Limiter,
) -> Option<SourceResult> {
    let artist = track.artist.as_deref()?;
    let title = track.title.as_deref()?;

    limiter.until_ready().await;

    let result = search_recording(client, artist, title).await;

    match result {
        Ok(Some(rec)) => build_result(rec),
        Ok(None) => None,
        Err(e) => {
            eprintln!("[musicbrainz] error: {e}");
            None
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

    // Tomar el resultado con mayor score (MB devuelve sorted por score)
    let best = resp
        .recordings
        .into_iter()
        .filter(|r| r.score.unwrap_or(0) >= 70)
        .next();

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

    Some(SourceResult {
        source: SourceName::MusicBrainz,
        year,
        genre,
        subgenre,
        confidence: 0.85,
    })
}

fn escape_lucene(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace(':', "\\:")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
