use crate::cache::{CachedGenre, CachePool};
use crate::rate_limit::Limiter;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;

const WIKI_BASE: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";
const USER_AGENT: &str = "music-tagger/1.0 (music-tagger@example.com)";

// Compilar regex una sola vez
static RE_SUBGENRE: OnceLock<Regex> = OnceLock::new();
static RE_DECADE: OnceLock<Regex> = OnceLock::new();

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct WikiSummary {
    extract: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenreValidation {
    pub parent_genre: Option<String>,
    pub origin_decade: Option<String>,
    pub confirmed: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn validate_genre(
    genre: &str,
    client: &reqwest::Client,
    limiter: &Limiter,
    cache: &CachePool,
) -> Option<GenreValidation> {
    let slug = to_slug(genre);

    // Cache hit
    if let Ok(Some(cached)) = cache.get_genre(&slug) {
        return Some(GenreValidation {
            parent_genre: cached.parent_genre,
            origin_decade: cached.origin_decade,
            confirmed: cached.confirmed,
        });
    }

    limiter.until_ready().await;

    let result = fetch_and_parse(client, &slug, genre).await.ok()?;

    // Guardar en cache
    if let Some(ref v) = result {
        let _ = cache.set_genre(
            &slug,
            &CachedGenre {
                parent_genre: v.parent_genre.clone(),
                origin_decade: v.origin_decade.clone(),
                confirmed: v.confirmed,
                mb_tag: None,
            },
        );
    }

    result
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

async fn fetch_and_parse(
    client: &reqwest::Client,
    slug: &str,
    genre: &str,
) -> Result<Option<GenreValidation>> {
    let url = format!("{WIKI_BASE}/{}", urlencoding(slug));

    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await;

    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(None),
    };

    let summary = resp.json::<WikiSummary>().await?;
    let extract = match summary.extract {
        Some(e) if !e.is_empty() => e,
        _ => return Ok(None),
    };

    let re_sub = RE_SUBGENRE.get_or_init(|| {
        Regex::new(r"(?i)subgenre of ([A-Za-z][A-Za-z &\-]+)").unwrap()
    });
    let re_dec = RE_DECADE.get_or_init(|| {
        Regex::new(r"(?i)originated in the (\d{4}s|\d{4})").unwrap()
    });

    let parent_genre = re_sub
        .captures(&extract)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_lowercase());

    let origin_decade = re_dec
        .captures(&extract)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    // "confirmed" = el nombre del género aparece en el texto del artículo
    let confirmed = extract.to_lowercase().contains(&genre.to_lowercase());

    Ok(Some(GenreValidation {
        parent_genre,
        origin_decade,
        confirmed,
    }))
}

fn to_slug(genre: &str) -> String {
    genre.trim().to_lowercase().replace(' ', "_")
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            c if c.is_alphanumeric() || c == '_' || c == '-' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
