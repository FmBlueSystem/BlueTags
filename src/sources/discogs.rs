use crate::models::{SourceName, SourceResult, TrackMetadata};
use crate::rate_limit::Limiter;
use anyhow::Result;
use serde::Deserialize;

const DISCOGS_BASE: &str = "https://api.discogs.com";

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    id: u64,
    genre: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ReleaseResponse {
    genres: Option<Vec<String>>,
    styles: Option<Vec<String>>,
    year: Option<u32>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn analyze(
    track: &TrackMetadata,
    client: &reqwest::Client,
    limiter: &Limiter,
    token: &str,
) -> Option<SourceResult> {
    let artist = track.artist.as_deref()?;
    let title = track.title.as_deref()?;

    // Paso 1: search → release id + confianza del genre voting (1 req)
    limiter.until_ready().await;
    let (release_id, search_confidence) = search_release(client, artist, title, token).await.ok()??;

    // Paso 2: fetch release → genres + styles + year (1 req)
    limiter.until_ready().await;
    let release = fetch_release(client, release_id, token).await.ok()??;

    build_result(release, search_confidence)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

async fn search_release(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
    token: &str,
) -> Result<Option<(u64, f32)>> {
    let resp = client
        .get(format!("{DISCOGS_BASE}/database/search"))
        .header("Authorization", format!("Discogs token={token}"))
        .header("User-Agent", "music-tagger/1.0")
        .query(&[
            ("artist", artist),
            ("track", title),
            ("type", "release"),
            ("per_page", "5"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<SearchResponse>()
        .await?;

    // Votar el género más frecuente entre los resultados y tomar el primer
    // release que tenga ese género. Evita que un pressing minoritario (ej. Electronic)
    // gane sobre el género mayoritario (ej. Funk / Soul).
    let results = resp.results;
    let mut genre_votes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for r in &results {
        for g in r.genre.iter().flatten() {
            *genre_votes.entry(g.to_lowercase()).or_default() += 1;
        }
    }
    let top_genre = genre_votes
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(g, _)| g);

    // Confianza dinámica: ratio de acuerdo × factor por cantidad de resultados
    // 4/5 dicen "funk/soul" → 0.80. Solo 1 resultado → penalizado.
    let result_count = results.len();
    let (best_id, confidence) = if let Some(ref tg) = top_genre {
        let votes_for_winner = results
            .iter()
            .filter(|r| r.genre.iter().flatten().any(|g| g.to_lowercase() == *tg))
            .count();
        let agreement = votes_for_winner as f32 / result_count.max(1) as f32;
        let count_factor = (result_count as f32 / 3.0).min(1.0);
        let id = results
            .iter()
            .find(|r| r.genre.iter().flatten().any(|g| g.to_lowercase() == *tg))
            .map(|r| r.id);
        (id, agreement * count_factor)
    } else {
        let id = results.into_iter().next().map(|r| r.id);
        (id, 0.5) // sin géneros en el search, confianza baja
    };

    Ok(best_id.map(|id| (id, confidence)))
}

async fn fetch_release(
    client: &reqwest::Client,
    release_id: u64,
    token: &str,
) -> Result<Option<ReleaseResponse>> {
    let resp = client
        .get(format!("{DISCOGS_BASE}/releases/{release_id}"))
        .header("Authorization", format!("Discogs token={token}"))
        .header("User-Agent", "music-tagger/1.0")
        .send()
        .await?
        .error_for_status()?
        .json::<ReleaseResponse>()
        .await?;

    Ok(Some(resp))
}

fn build_result(release: ReleaseResponse, search_confidence: f32) -> Option<SourceResult> {
    let raw_genre = release
        .genres
        .as_ref()
        .and_then(|g| g.first())
        .map(|g| g.to_lowercase());

    let first_style = release
        .styles
        .as_ref()
        .and_then(|s| s.first())
        .map(|s| s.to_lowercase());

    let second_style = release
        .styles
        .as_ref()
        .and_then(|s| s.get(1))
        .map(|s| s.to_lowercase());

    // Cuando Discogs usa géneros gruesos (Electronic, Rock, Pop, Funk / Soul),
    // el primer style es más preciso y mejor candidato para género.
    let genre = match raw_genre.as_deref() {
        Some("electronic" | "rock" | "pop" | "funk / soul") => {
            first_style.clone().or(raw_genre)
        }
        _ => raw_genre,
    };

    // Subgénero: segundo style si promovimos el primero, sino el primero
    let subgenre = if genre == first_style { second_style } else { first_style };

    let has_data = release.year.is_some() || genre.is_some();
    if !has_data {
        return None;
    }

    Some(SourceResult {
        source: SourceName::Discogs,
        year: release.year,
        genre,
        subgenre,
        confidence: search_confidence,
        mbid: None,
    })
}
