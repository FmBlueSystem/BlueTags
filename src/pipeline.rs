use crate::cache::CachePool;
use crate::config::Config;
use crate::models::{SourceResult, TagWriteStatus, TrackMetadata};
use crate::rate_limit::RateLimiters;
use crate::sources::{acousticbrainz, acoustid, discogs, essentia, lastfm, mb_mapping, musicbrainz, wiki_song, wikipedia};
use crate::tagger::{correction, voter, writer};
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
    ab_db: Option<&acousticbrainz::AcousticBrainzDb>,
    dry_run: bool,
    force: bool,
    skip_existing: bool,
    correct_artist: bool,
    map_genre: bool,
    force_decade: Option<&str>,
) -> ProcessResult {
    if is_in_fakes_folder(path) {
        let status = writer::mark_as_fake(path, dry_run);
        return ProcessResult { path: path.to_path_buf(), status, vote: None };
    }

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
            ab_db,
            dry_run,
            force,
            skip_existing,
            correct_artist,
            map_genre,
            force_decade,
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
    ab_db: Option<&acousticbrainz::AcousticBrainzDb>,
    dry_run: bool,
    force: bool,
    skip_existing: bool,
    correct_artist: bool,
    map_genre: bool,
    force_decade: Option<&str>,
) -> Result<(TagWriteStatus, crate::models::VoteResult)> {
    // 1. Leer metadata existente con lofty
    let mut track = read_metadata(path)?;

    // 2. Si es remix, limpiar título para buscar el tema ORIGINAL
    //    "Like a Virgin (Maik Schafer Pleasure and Pain Remix)" → "Like a Virgin"
    if let Some(title) = &track.title {
        if is_remix_title(title) {
            track.title = Some(clean_remix_title(title));
        }
    }

    // 3. Skip si ya tiene datos y --skip-existing
    if skip_existing && has_existing_tags(&track) && !force {
        return Ok((TagWriteStatus::Skipped, dummy_vote()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut results: Vec<SourceResult> = Vec::new();

    // 4. Fuentes concurrentes
    let (mb_res, discogs_res, lastfm_res) = tokio::join!(
        musicbrainz::analyze(&track, &client, &limiters.musicbrainz),
        discogs::analyze(&track, &client, &limiters.discogs, &config.discogs_token),
        lastfm::analyze(&track, &client, &limiters.lastfm, &config.lastfm_api_key),
    );

    let (mb_source, mb_mbid) = mb_res;
    if let Some(r) = mb_source { results.push(r); }
    if let Some(r) = discogs_res { results.push(r); }
    if let Some(r) = lastfm_res { results.push(r); }

    // 5. AcoustID (opcional) — corre ANTES de AcousticBrainz para proveer MBID via fingerprint
    let acoustid_mbid = if !config.acoustid_app_key.is_empty() {
        if let Some(r) = acoustid::analyze(&track, &client, &limiters.acoustid, &config.acoustid_app_key).await {
            let mbid = r.mbid.clone();
            results.push(r);
            mbid
        } else {
            None
        }
    } else {
        None
    };

    // 5b. AcousticBrainz — lookup local por MBID (AcoustID tiene prioridad sobre MB)
    let resolved_mbid = acoustid_mbid.or(mb_mbid);
    if let (Some(db), Some(ref mbid)) = (ab_db, &resolved_mbid) {
        if let Some(r) = db.lookup(mbid) {
            results.push(r);
        }
    }

    // Essentia (opcional)
    if let Some(clf) = essentia_clf {
        if let Some(r) = clf.analyze(path) {
            results.push(r);
        }
    }

    // 6. WikiSong — buscar artículo de la canción en Wikipedia
    if let (Some(artist), Some(title)) = (track.artist.as_deref(), track.title.as_deref()) {
        if let Some(r) = wiki_song::analyze(artist, title, &client, &limiters.wikipedia).await {
            results.push(r);
        }
    }

    // 7. Wikipedia (validar el género con más peso, no el primero en la lista)
    if let Some(top_genre) = voter::quick_genre_leader(&results) {
        let top_genre = top_genre.as_str();
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
                    mbid: None,
                });
            }
        }
    }

    // 7. Voting
    let vote = voter::vote(results, config.confidence_threshold, force_decade);

    // 8. Escribir tags
    let status = writer::write_tags(path, &vote, dry_run);

    // 9. Post-write corrections (optional, only when flags are set)
    if !dry_run {
        if correct_artist {
            if let Some(ref artist) = track.artist {
                if let Some(corrected) = correction::correct_artist(artist, &client).await {
                    let _ = apply_artist_correction(path, corrected);
                }
            }
        }
        if map_genre {
            let _ = apply_genre_mapping(path);
        }
    }

    Ok((status, vote))
}

fn apply_artist_correction(path: &Path, corrected: String) -> anyhow::Result<()> {
    let mut tagged_file = Probe::open(path)?.guess_file_type()?.read()?;
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.set_artist(corrected);
        tagged_file.save_to_path(path, lofty::config::WriteOptions::default())?;
    }
    Ok(())
}

fn apply_genre_mapping(path: &Path) -> anyhow::Result<()> {
    let current_genre = {
        let tagged_file = Probe::open(path)?.guess_file_type()?.read()?;
        tagged_file
            .primary_tag()
            .and_then(|t| t.genre().map(|g| g.to_string()))
    };
    if let Some(genre_str) = current_genre {
        if let Some(mapped) = correction::map_genre(&genre_str) {
            let mut tagged_file2 = Probe::open(path)?.guess_file_type()?.read()?;
            if let Some(tag) = tagged_file2.primary_tag_mut() {
                tag.set_genre(mapped.to_string());
                tagged_file2.save_to_path(path, lofty::config::WriteOptions::default())?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_metadata(path: &Path) -> Result<TrackMetadata> {
    let tagged = Probe::open(path)?.guess_file_type()?.read()?;
    let tag = tagged.primary_tag();

    let mut artist = tag.and_then(|t| t.artist().map(|s| s.to_string()));
    let mut title  = tag.and_then(|t| t.title().map(|s| s.to_string()));

    // Si los tags embebidos no tienen artist/title, parsear desde el nombre de archivo
    // Formato esperado: "Artist - Title.flac" o "Artist - Title (extra).flac"
    if artist.is_none() || title.is_none() {
        if let Some((a, t)) = parse_filename(path) {
            if artist.is_none() { artist = Some(a); }
            if title.is_none()  { title  = Some(t); }
        }
    }

    Ok(TrackMetadata {
        path: path.to_path_buf(),
        artist,
        title,
        album: tag.and_then(|t| t.album().map(|s| s.to_string())),
        existing_year: tag.and_then(|t| t.year()),
        existing_genre: tag.and_then(|t| t.genre().map(|s| s.to_string())),
        fingerprint: None,
        mbid: None,
    })
}

/// Extrae artist y title desde el nombre de archivo con formato "Artist - Title".
fn parse_filename(path: &Path) -> Option<(String, String)> {
    let stem = path.file_stem()?.to_str()?;
    let (artist_part, title_part) = stem.split_once(" - ")?;
    let artist = artist_part.trim().to_string();
    // Quitar info extra entre paréntesis al final del título
    let title = title_part
        .trim()
        .split(" (")
        .next()
        .unwrap_or(title_part.trim())
        .to_string();
    if artist.is_empty() || title.is_empty() {
        return None;
    }
    Some((artist, title))
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

// ---------------------------------------------------------------------------
// Detección de remixes / edits — retorna true si el archivo NO es original
// ---------------------------------------------------------------------------

/// Retorna true si el TITLE embebido indica un remix (nombre entre paréntesis al final,
/// o cualquiera de los patrones de is_remix pero aplicado al tag en lugar del filename).
pub fn is_remix_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    // Mismos patrones que is_remix
    if REMIX_PATTERNS.iter().any(|p| lower.contains(p)) {
        return true;
    }
    // Patrón adicional: título termina con "(Nombre Apellido)" — crédito de remixer
    // Detecta "Song Title (Peter Slaghuis)", "Song (Ben Liebrand)", etc.
    // Heurística: último paréntesis contiene exactamente 2-4 palabras (nombre propio)
    // y no es información de tiempo/parte como "(Part 1)" o "(Live)"
    if let Some(open) = lower.rfind(" (") {
        let inner = lower[open + 2..].trim_end_matches(')').trim();
        let words: Vec<&str> = inner.split_whitespace().collect();
        let looks_like_name = words.len() >= 1
            && words.len() <= 4
            && words.iter().all(|w| w.chars().all(|c| c.is_alphabetic() || c == '\'' || c == '-'));
        // Excluir descriptores comunes que no son nombres de remixer
        const NOT_NAMES: &[&str] = &[
            "part", "vol", "live", "remastered", "acoustic", "demo",
            "explicit", "album", "original", "single", "version", "feat",
            "edit", "reprise", "interlude", "intro", "outro",
        ];
        let not_part = !NOT_NAMES.iter().any(|n| inner.starts_with(n));
        if looks_like_name && not_part {
            return true;
        }
    }
    false
}

/// Limpia el título de remix para buscar el tema original.
/// "Like a Virgin (Maik Schafer Pleasure and Pain Remix)" → "Like a Virgin"
/// "Careless Whisper (DMC Remix)" → "Careless Whisper"
pub fn clean_remix_title(title: &str) -> String {
    // Quitar todo desde el primer paréntesis que contiene un patrón de remix
    let lower = title.to_lowercase();
    if let Some(idx) = lower.find(" (") {
        let after = &lower[idx..];
        if REMIX_PATTERNS.iter().any(|p| after.contains(p))
            || after.trim_start_matches(" (").split_whitespace().count() <= 4
        {
            return title[..idx].trim().to_string();
        }
    }
    // Fallback: quitar último paréntesis
    title.split(" (").next().unwrap_or(title).trim().to_string()
}

/// Retorna true si el archivo está dentro de alguna carpeta llamada `_fakes`.
pub fn is_in_fakes_folder(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str().to_str().map(|s| s.eq_ignore_ascii_case("_fakes")).unwrap_or(false)
    })
}

const REMIX_PATTERNS: &[&str] = &[
    "remix",
    "remixed",
    "re-edit",
    "re-mix",
    "re-situated",
    "re-version",
    "club mix",
    "extended mix",
    "dub mix",
    "radio mix",
    "radio edit",
    "12\" mix",
    "7\" mix",
    "dance mix",
    "party mix",
    "instrumental",
    "acappella",
    "a cappella",
    "mashup",
    "mash-up",
    "bootleg",
    "medley",
    "megamix",
    "mega mix",
    "nu disco mix",
    "reconstruction",
    "rework",
];

pub fn is_remix(path: &Path) -> bool {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    REMIX_PATTERNS.iter().any(|p| name.contains(p))
}
