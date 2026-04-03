mod config;
mod models;
mod sources;
mod tagger;

use anyhow::Result;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let _config = config::Config::from_env()?;

    println!("music-tagger — iniciando");

    Ok(())
}
