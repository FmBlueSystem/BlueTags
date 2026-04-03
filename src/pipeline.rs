use crate::cache::CachePool;
use crate::config::Config;
use crate::models::{SourceResult, TagWriteStatus, TrackMetadata};
use crate::rate_limit::RateLimiters;
use crate::sources::{acoustid, discogs, essentia, lastfm, mb_mapping, musicbrainz, wikipedia};
use crate::tagger::{voter, writer};
use anyhow::Result;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::{Path, PathBuf};

pub struct ProcessResult {
    pub path: PathBuf,
    pub status: TagWriteStatus,
    pub vote: Option<crate::models::VoteResult>,
}

// ---------------------------------------------------------------------------
// Entry point — llamado por cada thread de rayon
// ---------------------------------------------------------------------------

pub fn process_track(
    path: &Path,
    config: &Config,
    cache: &CachePool,
    limiters: &RateLimiters,
    essentia_clf: Option<&essentia::EssentiaClassifier>,
    dry_run: bool,
    force: bool,
    skip_existing: bool,
) -> ProcessResult {
    // Crear un runtime tokio por thread rayon (Card 14)
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    let status = rt.block_on(async {
        process_async(
            path,
            config,
            cache,
            limiters,
            essentia_clf,
            dry_run,
            force,
            skip_existing,
        )
        .await
    });

    match status {
        Ok((status, vote)) => ProcessResult {
            path: path.to_path_buf(),
            status,
            vote: Some(vote),
        },
        Err(e) => {
            eprintln!("[pipeline] error en {}: {e}", path.display());
            ProcessResult {
                path: path.to_path_buf(),
                status: TagWriteStatus::Error(e.to_string()),
                vote: None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Async pipeline
// ---------------------------------------------------------------------------

async fn process_async(
    path: &Path,
    config: &Config,
    cache: &CachePool,
    limiters: &RateLimiters,
    essentia_clf: Option<&essentia::EssentiaClassifier>,
    dry_run: bool,
    force: bool,
    skip_existing: bool,
) -> Result<(TagWriteStatus, crate::models::VoteResult)> {
    // 1. Leer metadata existente con lofty
    let track = read_metadata(path)?;

    // 2. Skip si ya tiene datos y --skip-existing
    if skip_existing && has_existing_tags(&track) && !force {
        return Ok((TagWriteStatus::Skipped, dummy_vote()));
    }

    // 3. Construir cliente HTTP compartido para esta task
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // 4. Fuentes que no requieren API key
    let mut results: Vec<SourceResult> = Vec::new();

    // MB Mapping (local, sin red)
    if let Some(genre) = track.existing_genre.as_deref() {
        if let Some(mapped) = mb_mapping::lookup(&cache.pool, genre) {
            results.push(mb_mapping::to_source_result(mapped));
        }
    }

    // 5. Fuentes concurrentes (todas se ejecutan en paralelo dentro de tokio)
    let (mb_res, discogs_res, lastfm_res) = tokio::join!(
        musicbrainz::analyze(&track, &client, &limiters.musicbrainz),
        discogs::analyze(&track, &client, &limiters.discogs, &config.discogs_token),
        lastfm::analyze(&track, &client, &limiters.lastfm, &config.lastfm_api_key),
    );

    if let Some(r) = mb_res { results.push(r); }
    if let Some(r) = discogs_res { results.push(r); }
    if let Some(r) = lastfm_res { results.push(r); }

    // AcoustID (opcional)
    if !config.acoustid_app_key.is_empty() {
        if let Some(r) = acoustid::analyze(&track, &client, &limiters.acoustid, &config.acoustid_app_key).await {
            results.push(r);
        }
    }

    // Essentia (opcional)
    if let Some(clf) = essentia_clf {
        if let Some(r) = clf.analyze(path) {
            results.push(r);
        }
    }

    // 6. Wikipedia (validar el género ganador preliminar)
    if let Some(top_genre) = results.iter().find_map(|r| r.genre.as_deref()) {
        if let Some(validation) = wikipedia::validate_genre(
            top_genre,
            &client,
            &limiters.wikipedia,
            cache,
        ).await {
            if validation.confirmed {
                results.push(SourceResult {
                    source: crate::models::SourceName::Wikipedia,
                    year: None,
                    genre: Some(top_genre.to_string()),
                    subgenre: validation.parent_genre,
                    confidence: 0.70,
                });
            }
        }
    }

    // 7. Voting
    let vote = voter::vote(results, config.confidence_threshold);

    // 8. Escribir tags
    let status = writer::write_tags(path, &vote, dry_run);

    Ok((status, vote))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_metadata(path: &Path) -> Result<TrackMetadata> {
    let tagged = Probe::open(path)?.guess_file_type()?.read()?;
    let tag = tagged.primary_tag();

    Ok(TrackMetadata {
        path: path.to_path_buf(),
        artist: tag.and_then(|t| t.artist().map(|s| s.to_string())),
        title: tag.and_then(|t| t.title().map(|s| s.to_string())),
        album: tag.and_then(|t| t.album().map(|s| s.to_string())),
        existing_year: tag.and_then(|t| t.year()),
        existing_genre: tag.and_then(|t| t.genre().map(|s| s.to_string())),
        fingerprint: None,
        mbid: None,
    })
}

fn has_existing_tags(track: &TrackMetadata) -> bool {
    track.existing_year.is_some() && track.existing_genre.is_some()
}

fn dummy_vote() -> crate::models::VoteResult {
    crate::models::VoteResult {
        year: None,
        decade: None,
        genre: None,
        subgenre: None,
        needs_review: false,
        sources_used: vec![],
    }
}

// ---------------------------------------------------------------------------
// Scan de archivos de audio en un directorio
// ---------------------------------------------------------------------------

pub fn scan_audio_files(path: &Path) -> Vec<PathBuf> {
    const AUDIO_EXTS: &[&str] = &["flac", "mp3", "aiff", "aif", "wav"];

    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                files.extend(scan_audio_files(&p));
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if AUDIO_EXTS.contains(&ext.to_lowercase().as_str()) {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    files
}
