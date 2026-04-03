use crate::cache::DbPool;
use crate::models::{SourceName, SourceResult};
use anyhow::Result;
use rusqlite::params;

// Bundleado en el binario al compilar — sin dependencia de red
const MAPPING_CSV: &str = include_str!("../../data/mb_genre_mapping.csv");
const VALID_GENRES_CSV: &str = include_str!("../../data/mb_genres_valid.csv");

#[derive(Debug, Clone)]
pub struct MbGenre {
    pub mb_genre: String,
    pub mb_subgenre: Option<String>,
    pub confidence: f32,
}

// ---------------------------------------------------------------------------
// Bootstrap — ejecutar UNA sola vez al iniciar si la tabla está vacía
// ---------------------------------------------------------------------------

pub fn bootstrap(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM mb_genre_mapping",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        return Ok(()); // ya poblada
    }

    // Parsear mb_genre_mapping.csv (source_tag, mb_genre, mb_subgenre)
    let mut lines = MAPPING_CSV.lines();
    lines.next(); // skip header

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 2 {
            continue;
        }
        let source_tag = cols[0].trim();
        let mb_genre = cols[1].trim();
        let mb_subgenre = cols.get(2).map(|s| s.trim()).filter(|s| !s.is_empty());

        conn.execute(
            "INSERT OR IGNORE INTO mb_genre_mapping (source_tag, mb_genre, mb_subgenre, confidence)
             VALUES (?1, ?2, ?3, 1.0)",
            params![source_tag, mb_genre, mb_subgenre],
        )?;
    }

    tracing_log(&format!(
        "mb_mapping: bootstrap completado ({} mappings cargados)",
        count_mapping(&conn)
    ));

    Ok(())
}

// ---------------------------------------------------------------------------
// Lookup — buscar un tag de fuente externa → MB género canónico
// ---------------------------------------------------------------------------

pub fn lookup(pool: &DbPool, source_tag: &str) -> Option<MbGenre> {
    let normalized = normalize(source_tag);
    let conn = pool.get().ok()?;

    let result = conn.query_row(
        "SELECT mb_genre, mb_subgenre, confidence FROM mb_genre_mapping WHERE source_tag = ?1",
        params![normalized],
        |row| {
            Ok(MbGenre {
                mb_genre: row.get(0)?,
                mb_subgenre: row.get(1)?,
                confidence: row.get::<_, f64>(2)? as f32,
            })
        },
    );

    result.ok()
}

// ---------------------------------------------------------------------------
// is_valid_mb_genre — verificar si un string es un género canónico MB
// Usa el CSV bundleado de 812 géneros válidos
// ---------------------------------------------------------------------------

pub fn is_valid_mb_genre(genre: &str) -> bool {
    let normalized = normalize(genre);
    VALID_GENRES_CSV
        .lines()
        .skip(1) // skip header
        .any(|line| line.trim() == normalized)
}

// ---------------------------------------------------------------------------
// to_source_result
// ---------------------------------------------------------------------------

pub fn to_source_result(genre: MbGenre) -> SourceResult {
    SourceResult {
        source: SourceName::MbMapping,
        year: None,
        genre: Some(genre.mb_genre),
        subgenre: genre.mb_subgenre,
        confidence: genre.confidence,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

fn count_mapping(conn: &rusqlite::Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM mb_genre_mapping", [], |r| r.get(0))
        .unwrap_or(0)
}

fn tracing_log(msg: &str) {
    eprintln!("[mb_mapping] {msg}");
}
