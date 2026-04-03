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

    // Paso 1: search → release id (1 req)
    limiter.until_ready().await;
    let release_id = search_release(client, artist, title, token).await.ok()??;

    // Paso 2: fetch release → genres + styles + year (1 req)
    limiter.until_ready().await;
    let release = fetch_release(client, release_id, token).await.ok()??;

    build_result(release)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

async fn search_release(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
    token: &str,
) -> Result<Option<u64>> {
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

    Ok(resp.results.into_iter().next().map(|r| r.id))
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

fn build_result(release: ReleaseResponse) -> Option<SourceResult> {
    let genre = release
        .genres
        .as_ref()
        .and_then(|g| g.first())
        .map(|g| g.to_lowercase());

    let subgenre = release
        .styles
        .as_ref()
        .and_then(|s| s.first())
        .map(|s| s.to_lowercase());

    let has_data = release.year.is_some() || genre.is_some();
    if !has_data {
        return None;
    }

    Some(SourceResult {
        source: SourceName::Discogs,
        year: release.year,
        genre,
        subgenre,
        confidence: 0.90,
    })
}
