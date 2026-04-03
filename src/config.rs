use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub discogs_token: String,
    pub lastfm_api_key: String,
    pub acoustid_app_key: String,
    pub confidence_threshold: f32,
    pub jobs: usize,
    pub essentia_model_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            discogs_token: std::env::var("DISCOGS_TOKEN")
                .context("DISCOGS_TOKEN no definido en .env")?,
            lastfm_api_key: std::env::var("LASTFM_API_KEY")
                .context("LASTFM_API_KEY no definido en .env")?,
            acoustid_app_key: std::env::var("ACOUSTID_APP_KEY")
                .context("ACOUSTID_APP_KEY no definido en .env")?,
            confidence_threshold: std::env::var("MUSIC_TAGGER_CONFIDENCE")
                .unwrap_or_else(|_| "0.65".to_string())
                .parse()
                .unwrap_or(0.65),
            jobs: std::env::var("MUSIC_TAGGER_JOBS")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .unwrap_or(8),
            essentia_model_path: std::env::var("ESSENTIA_MODEL_PATH").ok(),
        })
    }
}
