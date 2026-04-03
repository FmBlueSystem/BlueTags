mod cache;
mod config;
mod models;
mod sources;
mod tagger;

use anyhow::Result;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let _config = config::Config::from_env()?;

    // Inicializar cache SQLite
    let cache = cache::build_pool("music-tagger.db")?;

    // Bootstrap MB Genre Mapping (no-op si ya está poblado)
    sources::mb_mapping::bootstrap(&cache.pool)?;

    println!("music-tagger — iniciando");

    Ok(())
}
