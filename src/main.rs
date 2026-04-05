mod cache;
mod cli;
mod config;
mod models;
mod pipeline;
mod rate_limit;
mod sources;
mod tagger;

use crate::sources::essentia::EssentiaClassifier;
use clap::Parser;
use colored::Colorize;
use lofty::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = cli::Cli::parse();
    let config = config::Config::from_env()?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.jobs)
        .build_global()?;

    // Inicializar cache SQLite
    let cache = Arc::new(cache::build_pool("music-tagger.db")?);

    // Bootstrap MB Genre Mapping
    sources::mb_mapping::bootstrap(&cache.pool)?;

    // Inicializar rate limiters (compartidos entre todos los threads via Arc)
    let limiters = Arc::new(rate_limit::RateLimiters::new());

    // Cargar modelo Essentia (opcional)
    let essentia_clf: Option<Arc<EssentiaClassifier>> = if cli.no_essentia {
        None
    } else {
        config.essentia_model_path.as_ref().and_then(|p| {
            EssentiaClassifier::load(Path::new(p)).map(Arc::new)
        })
    };

    // Cargar AcousticBrainz DB (opcional, ~1.3M recordings)
    let ab_db_path = Path::new("data/acousticbrainz_genres.db");
    let ab_db: Option<Arc<sources::acousticbrainz::AcousticBrainzDb>> =
        sources::acousticbrainz::AcousticBrainzDb::open(ab_db_path).map(Arc::new);

    match cli.command {
        cli::Commands::Audit { path } => {
            run_audit(&path, &config, &cache, &limiters, essentia_clf.as_deref())?;
        }
        cli::Commands::Tag { path, dry_run, write, force, skip_existing, correct_artist, map_genre, force_decade } => {
            config.validate_api_keys()?;
            // Validar formato de force_decade si se provee
            if let Some(ref fd) = force_decade {
                let valid = fd.ends_with('s') && fd.len() == 5
                    && fd[..4].parse::<u32>().is_ok();
                if !valid {
                    eprintln!("error: --force-decade debe tener formato \"NNNNs\" (ej: \"1980s\")");
                    std::process::exit(1);
                }
            }
            let actual_dry_run = !write || dry_run;
            run_tag(
                &path,
                &config,
                &cache,
                &limiters,
                essentia_clf.as_deref(),
                ab_db.as_deref(),
                actual_dry_run,
                force,
                skip_existing,
                correct_artist,
                map_genre,
                force_decade.as_deref(),
            )?;
        }
        cli::Commands::Retry { path, write } => {
            config.validate_api_keys()?;
            run_tag(&path, &config, &cache, &limiters, essentia_clf.as_deref(), ab_db.as_deref(), !write, false, false, false, false, None)?;
        }
        cli::Commands::AudioFeatures { path, json } => {
            use music_tagger::audio_features;
            let files = music_tagger::pipeline::scan_audio_files(&path);
            let results = audio_features::analyze_batch(&files);
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for r in &results {
                    if let Some(e) = &r.error {
                        eprintln!("[ERROR] {}: {}", r.file, e);
                    } else {
                        println!("{} - vocal:{:.2} bright:{:.2}",
                            r.file,
                            r.vocal_pct.unwrap_or(0.0),
                            r.brightness.unwrap_or(0.0));
                    }
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

fn run_audit(
    path: &std::path::Path,
    _config: &config::Config,
    _cache: &Arc<cache::CachePool>,
    _limiters: &Arc<rate_limit::RateLimiters>,
    _essentia: Option<&EssentiaClassifier>,
) -> anyhow::Result<()> {
    let all_files = pipeline::scan_audio_files(path);
    let files: Vec<_> = all_files.iter().filter(|f| !pipeline::is_remix(f)).cloned().collect();
    let remixes_skipped = all_files.len() - files.len();
    println!("Auditando {} archivos en {} ({} remixes ignorados)", files.len(), path.display(), remixes_skipped);

    let mut missing_year = 0;
    let mut missing_genre = 0;

    for file in &files {
        let tagged = lofty::probe::Probe::open(file)?.guess_file_type()?.read()?;
        let tag = tagged.primary_tag();

        let has_year = tag.and_then(|t| t.year()).is_some();
        let has_genre = tag.and_then(|t| t.genre()).is_some();

        if !has_year || !has_genre {
            println!(
                "  {} {}",
                if !has_year && !has_genre { "[MISSING year+genre]".red() }
                else if !has_year { "[MISSING year]".yellow() }
                else { "[MISSING genre]".yellow() },
                file.display()
            );
        }

        if !has_year { missing_year += 1; }
        if !has_genre { missing_genre += 1; }
    }

    println!("\nTotal: {} archivos | sin año: {} | sin género: {}", files.len(), missing_year, missing_genre);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tag (rayon + tokio bridge — Card 14)
// ---------------------------------------------------------------------------

fn run_tag(
    path: &std::path::Path,
    config: &config::Config,
    cache: &Arc<cache::CachePool>,
    limiters: &Arc<rate_limit::RateLimiters>,
    essentia: Option<&EssentiaClassifier>,
    ab_db: Option<&sources::acousticbrainz::AcousticBrainzDb>,
    dry_run: bool,
    force: bool,
    skip_existing: bool,
    correct_artist: bool,
    map_genre: bool,
    force_decade: Option<&str>,
) -> anyhow::Result<()> {
    let files = pipeline::scan_audio_files(path);
    let remix_count = files.iter().filter(|f| pipeline::is_remix(f)).count();
    println!("Procesando {} archivos ({} remixes incluidos)...", files.len(), remix_count);

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40}] {pos}/{len} {wide_msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    // rayon::par_iter — cada thread crea su propio tokio runtime (Card 14)
    let results: Vec<pipeline::ProcessResult> = files
        .par_iter()
        .map(|file| {
            pb.set_message(file.file_name().unwrap_or_default().to_string_lossy().to_string());
            let result = pipeline::process_track(
                file,
                config,
                cache,
                limiters,
                essentia,
                ab_db,
                dry_run,
                force,
                skip_existing,
                correct_artist,
                map_genre,
                force_decade,
            );
            pb.inc(1);
            result
        })
        .collect();

    pb.finish_with_message("Listo");

    // Resumen
    let written = results.iter().filter(|r| matches!(r.status, models::TagWriteStatus::Written)).count();
    let dry    = results.iter().filter(|r| matches!(r.status, models::TagWriteStatus::DryRun)).count();
    let review = results.iter().filter(|r| matches!(r.status, models::TagWriteStatus::NeedsReview)).count();
    let errors = results.iter().filter(|r| matches!(r.status, models::TagWriteStatus::Error(_))).count();

    // Listar archivos que necesitan revisión
    for r in results.iter().filter(|r| matches!(r.status, models::TagWriteStatus::NeedsReview)) {
        println!("[NEEDS REVIEW] {}", r.path.display());
    }

    println!("\n{}", "─".repeat(60));
    println!("  Written:      {}", written.to_string().green());
    println!("  Dry-run:      {}", dry.to_string().cyan());
    println!("  Needs review: {}", review.to_string().yellow());
    println!("  Errors:       {}", errors.to_string().red());
    println!("  Total:        {}", files.len());

    Ok(())
}
