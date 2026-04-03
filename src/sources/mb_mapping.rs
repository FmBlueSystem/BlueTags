use crate::models::SourceResult;

pub struct MbGenre {
    pub mb_genre: String,
    pub mb_subgenre: Option<String>,
    pub confidence: f32,
}

pub fn lookup(_source_tag: &str) -> Option<MbGenre> {
    // Card 03
    None
}

pub fn bootstrap(_pool: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> anyhow::Result<()> {
    // Card 03
    Ok(())
}

pub fn to_source_result(genre: MbGenre) -> SourceResult {
    use crate::models::SourceName;
    SourceResult {
        source: SourceName::MbMapping,
        year: None,
        genre: Some(genre.mb_genre),
        subgenre: genre.mb_subgenre,
        confidence: genre.confidence,
    }
}
