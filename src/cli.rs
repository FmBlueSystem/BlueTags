use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "music-tagger",
    about = "Auto-tag music libraries with year, decade, genre and sub-genre",
    version = "0.1.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Umbral de confianza mínimo para escribir tags (0.0-1.0)
    #[arg(long, default_value = "0.65", global = true)]
    pub confidence: f32,

    /// Threads paralelos (default: núm. de CPUs)
    #[arg(long, default_value = "8", global = true)]
    pub jobs: usize,

    /// Deshabilitar fuente Essentia/ONNX
    #[arg(long, global = true)]
    pub no_essentia: bool,

    /// Deshabilitar fuente AcoustID
    #[arg(long, global = true)]
    pub no_acoustid: bool,

    /// Guardar log de decisiones en archivo JSON
    #[arg(long, global = true)]
    pub log: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Reportar campos faltantes sin escribir nada
    Audit {
        /// Directorio o archivo a auditar
        path: PathBuf,
    },

    /// Etiquetar archivos de audio
    Tag {
        /// Directorio o archivo a procesar
        path: PathBuf,

        /// Mostrar qué haría sin escribir (default)
        #[arg(long)]
        dry_run: bool,

        /// Escribir tags reales en los archivos
        #[arg(long)]
        write: bool,

        /// Forzar re-tag aunque ya tenga valores
        #[arg(long)]
        force: bool,

        /// Saltar archivos que ya tengan género y año
        #[arg(long)]
        skip_existing: bool,

        /// Corregir nombre de artista via MusicBrainz (score>=90)
        #[arg(long)]
        correct_artist: bool,

        /// Mapear GENRE al sistema DJ interno (11 categorías)
        #[arg(long)]
        map_genre: bool,

        /// Forzar década en todos los archivos del batch (ej: "1980s")
        #[arg(long)]
        force_decade: Option<String>,
    },

    /// Reprocesar solo archivos marcados como NEEDS_REVIEW
    Retry {
        /// Directorio a reprocesar
        path: PathBuf,

        /// Escribir tags reales en los archivos
        #[arg(long)]
        write: bool,
    },

    /// Analizar características de audio (vocal%, brightness) — output JSON
    AudioFeatures {
        /// Directorio con archivos FLAC a analizar
        path: PathBuf,

        /// Output JSON a stdout
        #[arg(long)]
        json: bool,
    },
}
