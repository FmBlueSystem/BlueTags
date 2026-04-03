use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

pub type DbPool = Pool<SqliteConnectionManager>;

const TTL_TRACK_SECS: i64 = 90 * 24 * 60 * 60;   // 90 días
const TTL_GENRE_SECS: i64 = 180 * 24 * 60 * 60;  // 180 días

pub struct CachePool {
    pub pool: DbPool,
}

// ---------------------------------------------------------------------------
// Tipos cacheados
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CachedTrack {
    pub mbid: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub subgenre: Option<String>,
    pub confidence: f32,
    pub sources_used: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CachedGenre {
    pub parent_genre: Option<String>,
    pub origin_decade: Option<String>,
    pub confirmed: bool,
    pub mb_tag: Option<String>,
}

// ---------------------------------------------------------------------------
// Inicialización
// ---------------------------------------------------------------------------

pub fn build_pool(db_path: &str) -> Result<CachePool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(manager)?;

    let conn = pool.get()?;

    // WAL mode — mejor concurrencia con múltiples readers
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS track_cache (
            fingerprint     TEXT PRIMARY KEY,
            mbid            TEXT,
            year            INTEGER,
            genre           TEXT,
            subgenre        TEXT,
            confidence      REAL NOT NULL DEFAULT 0.0,
            sources_used    TEXT NOT NULL DEFAULT '[]',
            cached_at       INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS genre_cache (
            genre_slug      TEXT PRIMARY KEY,
            parent_genre    TEXT,
            origin_decade   TEXT,
            confirmed       INTEGER NOT NULL DEFAULT 0,
            mb_tag          TEXT,
            cached_at       INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mb_genre_mapping (
            source_tag      TEXT PRIMARY KEY,
            mb_genre        TEXT NOT NULL,
            mb_subgenre     TEXT,
            confidence      REAL NOT NULL DEFAULT 1.0
        );
    ")?;

    Ok(CachePool { pool })
}

// ---------------------------------------------------------------------------
// Track cache
// ---------------------------------------------------------------------------

impl CachePool {
    pub fn get_track(&self, fingerprint: &str) -> Result<Option<CachedTrack>> {
        let conn = self.pool.get()?;
        let now = unix_now();

        let result = conn.query_row(
            "SELECT mbid, year, genre, subgenre, confidence, sources_used, cached_at
             FROM track_cache WHERE fingerprint = ?1",
            params![fingerprint],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        );

        match result {
            Ok((mbid, year, genre, subgenre, confidence, sources_json, cached_at)) => {
                if now - cached_at > TTL_TRACK_SECS {
                    return Ok(None); // expirado
                }
                let sources_used: Vec<String> =
                    serde_json::from_str(&sources_json).unwrap_or_default();
                Ok(Some(CachedTrack {
                    mbid,
                    year: year.map(|y| y as u32),
                    genre,
                    subgenre,
                    confidence: confidence as f32,
                    sources_used,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_track(&self, fingerprint: &str, track: &CachedTrack) -> Result<()> {
        let conn = self.pool.get()?;
        let sources_json = serde_json::to_string(&track.sources_used)?;

        conn.execute(
            "INSERT OR REPLACE INTO track_cache
             (fingerprint, mbid, year, genre, subgenre, confidence, sources_used, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                fingerprint,
                track.mbid,
                track.year.map(|y| y as i64),
                track.genre,
                track.subgenre,
                track.confidence as f64,
                sources_json,
                unix_now(),
            ],
        )?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Genre cache
    // ---------------------------------------------------------------------------

    pub fn get_genre(&self, genre_slug: &str) -> Result<Option<CachedGenre>> {
        let conn = self.pool.get()?;
        let now = unix_now();

        let result = conn.query_row(
            "SELECT parent_genre, origin_decade, confirmed, mb_tag, cached_at
             FROM genre_cache WHERE genre_slug = ?1",
            params![genre_slug],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        );

        match result {
            Ok((parent_genre, origin_decade, confirmed, mb_tag, cached_at)) => {
                if now - cached_at > TTL_GENRE_SECS {
                    return Ok(None);
                }
                Ok(Some(CachedGenre {
                    parent_genre,
                    origin_decade,
                    confirmed: confirmed != 0,
                    mb_tag,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_genre(&self, genre_slug: &str, genre: &CachedGenre) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO genre_cache
             (genre_slug, parent_genre, origin_decade, confirmed, mb_tag, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                genre_slug,
                genre.parent_genre,
                genre.origin_decade,
                genre.confirmed as i64,
                genre.mb_tag,
                unix_now(),
            ],
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
