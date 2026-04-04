use anyhow::Result;

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
    /// Carga todas las keys — falla si alguna obligatoria falta.
    /// Usar para comandos `tag` y `retry`.
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            discogs_token: std::env::var("DISCOGS_TOKEN").unwrap_or_default(),
            lastfm_api_key: std::env::var("LASTFM_API_KEY").unwrap_or_default(),
            acoustid_app_key: std::env::var("ACOUSTID_APP_KEY").unwrap_or_default(),
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

    /// Valida que las keys de API estén presentes — llamar antes de `tag`/`retry`.
    pub fn validate_api_keys(&self) -> anyhow::Result<()> {
        if self.discogs_token.is_empty() {
            anyhow::bail!("DISCOGS_TOKEN no definido en .env");
        }
        if self.lastfm_api_key.is_empty() {
            anyhow::bail!("LASTFM_API_KEY no definido en .env");
        }
        if self.acoustid_app_key.is_empty() {
            anyhow::bail!("ACOUSTID_APP_KEY no definido en .env");
        }
        Ok(())
    }
}
