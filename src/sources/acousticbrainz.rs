use crate::models::{SourceName, SourceResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Lookup de géneros en el dataset AcousticBrainz (SQLite local).
/// 1.3M recordings con géneros de Discogs + Last.fm pre-reconciliados.
pub struct AcousticBrainzDb {
    conn: Mutex<Connection>,
}

impl AcousticBrainzDb {
    pub fn open(db_path: &Path) -> Option<Self> {
        if !db_path.exists() {
            eprintln!(
                "[acousticbrainz] DB no encontrada en {}. Fuente deshabilitada.",
                db_path.display()
            );
            return None;
        }
        let conn = Connection::open(db_path).ok()?;
        eprintln!("[acousticbrainz] DB cargada: {}", db_path.display());
        Some(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Busca géneros para un MBID. Retorna el género más frecuente entre las fuentes.
    pub fn lookup(&self, mbid: &str) -> Option<SourceResult> {
        let conn = self.conn.lock().ok()?;
        let mut stmt = conn
            .prepare("SELECT genre, subgenre, source FROM genres WHERE mbid = ?1")
            .ok()?;

        let rows: Vec<(String, Option<String>, String)> = stmt
            .query_map([mbid], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .ok()?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return None;
        }

        // Votar el género más frecuente entre Discogs + Last.fm
        let mut genre_votes: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut subgenre_votes: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for (genre, subgenre, _source) in &rows {
            *genre_votes.entry(genre.to_lowercase()).or_default() += 1;
            if let Some(sub) = subgenre {
                *subgenre_votes.entry(sub.to_lowercase()).or_default() += 1;
            }
        }

        let top_genre = genre_votes
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(g, _)| g.clone());

        let top_subgenre = subgenre_votes
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(s, _)| s.clone());

        let genre = top_genre?;

        // Confianza: ratio de fuentes que coinciden en el top género
        let total_sources = rows.len() as f32;
        let agreeing = *genre_votes.get(&genre).unwrap_or(&0) as f32;
        let confidence = (agreeing / total_sources).clamp(0.5, 0.95);

        Some(SourceResult {
            source: SourceName::AcousticBrainz,
            year: None, // El dataset no incluye año
            genre: Some(genre),
            subgenre: top_subgenre,
            confidence,
            mbid: None,
        })
    }
}
