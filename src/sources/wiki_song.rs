use crate::models::{SourceName, SourceResult};
use crate::rate_limit::Limiter;
use anyhow::Result;
use serde::Deserialize;

const WIKI_SEARCH: &str = "https://en.wikipedia.org/w/api.php";
const WIKI_SUMMARY: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";
const USER_AGENT: &str = "music-tagger/1.0 (fmolinam@gmail.com)";

// Géneros a detectar en el extract del artículo, ordenados por especificidad
const GENRE_PATTERNS: &[(&str, &str)] = &[
    ("new jack swing", "new jack swing"),
    ("freestyle", "freestyle"),
    ("synth-pop", "synth-pop"),
    ("synthpop", "synth-pop"),
    ("new wave", "new wave"),
    ("post-punk", "post-punk"),
    ("hip hop", "hip hop"),
    ("hip-hop", "hip hop"),
    ("electro funk", "electro"),
    ("latin pop", "latin pop"),
    ("dance-pop", "dance-pop"),
    ("contemporary r&b", "soul"),
    ("rhythm and blues", "soul"),
    ("r&b", "soul"),
    ("funk", "funk"),
    ("soul", "soul"),
    ("disco", "disco"),
    ("house", "house"),
    ("pop rock", "rock"),
    ("rock", "rock"),
    ("pop", "pop"),
    ("jazz", "jazz"),
    ("reggae", "reggae"),
    ("blues", "blues"),
    ("electronic", "electronic"),
];

// Charts que indican género
const CHART_GENRE: &[(&str, &str)] = &[
    ("hot black singles", "soul"),
    ("r&b", "soul"),
    ("hot r&b", "soul"),
    ("dance chart", "disco"),
    ("hot dance", "disco"),
    ("dance club", "disco"),
    ("hot 100", "pop"),
    ("billboard hot 100", "pop"),
];

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SearchResponse {
    query: Option<SearchQuery>,
}

#[derive(Deserialize)]
struct SearchQuery {
    search: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    title: String,
}

#[derive(Deserialize)]
struct WikiSummary {
    extract: Option<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn analyze(
    artist: &str,
    title: &str,
    client: &reqwest::Client,
    limiter: &Limiter,
) -> Option<SourceResult> {
    limiter.until_ready().await;

    let result = lookup_song(artist, title, client).await.ok()?;
    result
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

async fn lookup_song(
    artist: &str,
    title: &str,
    client: &reqwest::Client,
) -> Result<Option<SourceResult>> {
    // 1. Buscar el artículo de la canción en Wikipedia
    let search_query = format!("\"{}\" \"{}\"", artist, title);
    let resp = client
        .get(WIKI_SEARCH)
        .header("User-Agent", USER_AGENT)
        .query(&[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", &search_query),
            ("srlimit", "3"),
            ("format", "json"),
        ])
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    let results = resp.query.map(|q| q.search).unwrap_or_default();
    if results.is_empty() {
        return Ok(None);
    }

    // 2. Tomar el mejor resultado con esta prioridad:
    //    a) Artículo de la canción con artista: "Title (Artist song)"
    //    b) Artículo que contiene el título de la canción (not album/discography)
    //    c) Primer resultado
    let artist_lower = artist.to_lowercase();
    let title_lower = title.to_lowercase();
    let best_title = results
        .iter()
        // Prioridad 1: artículo con nombre de canción Y (song/single)
        .find(|r| {
            let t = r.title.to_lowercase();
            t.contains(&title_lower) && (t.contains("song") || t.contains("single"))
        })
        // Prioridad 2: artículo cuyo título coincide con el título de la canción
        .or_else(|| results.iter().find(|r| {
            let t = r.title.to_lowercase();
            t.contains(&title_lower) && !t.contains("album") && !t.contains("discography")
        }))
        // Prioridad 3: primer resultado que no sea álbum/discografía
        .or_else(|| results.iter().find(|r| {
            let t = r.title.to_lowercase();
            !t.contains("album") && !t.contains("discography") && !t.contains("remix")
        }))
        .or_else(|| results.first())
        .map(|r| &r.title);

    let article_title = match best_title {
        Some(t) => t,
        None => return Ok(None),
    };

    // 3. Fetch el summary del artículo
    let slug = article_title.replace(' ', "_");
    let summary_resp = client
        .get(format!("{WIKI_SUMMARY}/{}", urlencoding(&slug)))
        .header("User-Agent", USER_AGENT)
        .send()
        .await;

    let summary_resp = match summary_resp {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(None),
    };

    let summary = summary_resp.json::<WikiSummary>().await?;
    let extract = match summary.extract {
        Some(e) if !e.is_empty() => e,
        _ => return Ok(None),
    };

    let lower = extract.to_lowercase();

    // 4. Extraer género del texto — primero por chart mentions, luego por keywords
    let mut genre = None;
    let mut subgenre = None;

    // Charts son alta señal de género
    for (pattern, chart_genre) in CHART_GENRE {
        if lower.contains(pattern) {
            if genre.is_none() {
                genre = Some(chart_genre.to_string());
            } else if subgenre.is_none() && Some(chart_genre.to_string()) != genre {
                subgenre = Some(chart_genre.to_string());
            }
        }
    }

    // Keywords de género en el texto del artículo
    for (pattern, mapped_genre) in GENRE_PATTERNS {
        if lower.contains(pattern) {
            if genre.is_none() {
                genre = Some(mapped_genre.to_string());
            } else if subgenre.is_none() && Some(mapped_genre.to_string()) != genre {
                subgenre = Some(mapped_genre.to_string());
            }
        }
    }

    // 5. Extraer año si lo menciona
    let year = extract_year(&lower);

    let has_data = genre.is_some() || year.is_some();
    if !has_data {
        return Ok(None);
    }

    Ok(Some(SourceResult {
        source: SourceName::WikiSong,
        year,
        genre,
        subgenre,
        confidence: 0.85,
        mbid: None,
    }))
}

fn extract_year(text: &str) -> Option<u32> {
    // Buscar patrones como "released in 1986", "released on July 28, 1986"
    let re = regex::Regex::new(r"released[^.]*?(\d{4})").ok()?;
    re.captures(text)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .filter(|y| *y >= 1950 && *y <= 2030)
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            c if c.is_alphanumeric() || c == '_' || c == '-' || c == '(' || c == ')' => {
                c.to_string()
            }
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
