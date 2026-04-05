/// Post-voting artist correction using MusicBrainz
pub async fn correct_artist(artist: &str, client: &reqwest::Client) -> Option<String> {
    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:{}&limit=1&fmt=json",
        urlencoding(artist)
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "music-tagger/1.0 (fmolinam@gmail.com)")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;

    let artists = json.get("artists")?.as_array()?;
    if artists.is_empty() {
        return None;
    }

    let first = &artists[0];
    let score: u64 = first
        .get("score")
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .or_else(|| first.get("score").and_then(|s| s.as_u64()))
        .unwrap_or(0);

    let corrected_name = first.get("name")?.as_str()?;

    // Rate limit: 1s between calls
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    if score >= 90 && corrected_name != artist {
        Some(corrected_name.to_string())
    } else {
        None
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => '+'.to_string(),
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => {
                c.to_string()
            }
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

/// Map genre to DJ internal system (11 categories)
pub fn map_genre(genre: &str) -> Option<&'static str> {
    let g = genre.to_lowercase();
    let g = g.trim();

    if matches!(g, "electronic" | "house" | "techno" | "trance" | "edm") {
        return Some("House & Electronic");
    }
    if matches!(g, "dance" | "pop") {
        return Some("Pop & Dance");
    }
    if matches!(g, "rock" | "metal" | "punk") {
        return Some("Rock");
    }
    if matches!(g, "hip hop" | "hip-hop" | "r&b" | "rap" | "trap") {
        return Some("Hip-Hop & R&B");
    }
    if matches!(g, "soul" | "funk" | "disco") {
        return Some("Disco, Funk & Soul");
    }
    if matches!(g, "jazz" | "blues") {
        return Some("Jazz");
    }
    if matches!(g, "reggae" | "dancehall" | "ska") {
        return Some("Reggae & Dancehall");
    }
    if matches!(g, "latin" | "salsa" | "reggaeton") {
        return Some("World & Latin");
    }
    if g == "country" {
        return Some("Country");
    }
    if g == "classical" {
        return Some("Classical");
    }
    if matches!(g, "new wave" | "synth") {
        return Some("Synth Pop & New Wave");
    }

    None
}
