use crate::models::{SourceName, SourceResult, TrackMetadata};
use crate::rate_limit::Limiter;
use anyhow::Result;
use serde::Deserialize;

const LASTFM_BASE: &str = "https://ws.audioscrobbler.com/2.0";

// Tags de ruido a filtrar — no son géneros
const NOISE_TAGS: &[&str] = &[
    "seen live", "beautiful", "favorite", "favourite", "amazing", "awesome",
    "cool", "love", "my favorite", "great", "best", "good", "classic",
    "masterpiece", "perfect", "brilliant", "excellent", "fantastic", "catchy",
    "chill", "relax", "relaxing", "workout", "running", "driving", "party",
    "sad", "happy", "melancholic", "epic", "underrated", "overrated",
    "all time favorite", "all-time favorite", "to listen", "must listen",
];

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TrackInfoResponse {
    track: Option<TrackInfo>,
}

#[derive(Deserialize)]
struct TrackInfo {
    toptags: Option<TopTags>,
}

#[derive(Deserialize)]
struct TopTags {
    tag: Vec<Tag>,
}

#[derive(Deserialize)]
struct Tag {
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
    api_key: &str,
) -> Option<SourceResult> {
    let artist = track.artist.as_deref()?;
    let title = track.title.as_deref()?;

    limiter.until_ready().await;

    let tags = fetch_tags(client, artist, title, api_key).await.ok()??;

    build_result(tags)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

async fn fetch_tags(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
    api_key: &str,
) -> Result<Option<Vec<Tag>>> {
    let resp = client
        .get(LASTFM_BASE)
        .query(&[
            ("method", "track.getInfo"),
            ("artist", artist),
            ("track", title),
            ("api_key", api_key),
            ("format", "json"),
            ("autocorrect", "1"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<TrackInfoResponse>()
        .await?;

    Ok(resp
        .track
        .and_then(|t| t.toptags)
        .map(|tt| tt.tag))
}

fn build_result(tags: Vec<Tag>) -> Option<SourceResult> {
    // Filtrar ruido y quedarnos con los tags ordenados por count
    let genre_tags: Vec<&Tag> = tags
        .iter()
        .filter(|t| !is_noise(&t.name))
        .collect();

    let genre = genre_tags.first().map(|t| t.name.to_lowercase());
    let subgenre = genre_tags.get(1).map(|t| t.name.to_lowercase());

    genre.as_ref()?;

    // Confianza dinámica: dominancia del top tag sobre el total
    let total_count: u32 = genre_tags.iter().filter_map(|t| t.count).sum();
    let top_count: u32 = genre_tags.first().and_then(|t| t.count).unwrap_or(0);
    let confidence = if total_count > 0 {
        (top_count as f32 / total_count as f32).clamp(0.3, 0.95)
    } else {
        0.5
    };

    Some(SourceResult {
        source: SourceName::LastFm,
        year: None,
        genre,
        subgenre,
        confidence,
        mbid: None,
    })
}

fn is_noise(tag: &str) -> bool {
    let lower = tag.to_lowercase();
    NOISE_TAGS.iter().any(|n| *n == lower.as_str())
}
