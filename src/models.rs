use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub existing_year: Option<u32>,
    pub existing_genre: Option<String>,
    pub fingerprint: Option<String>,
    pub mbid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceResult {
    pub source: SourceName,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub subgenre: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceName {
    Discogs,
    MusicBrainz,
    LastFm,
    AcoustId,
    Wikipedia,
    Essentia,
    MbMapping,
    ExistingTag,
}

impl std::fmt::Display for SourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceName::Discogs => write!(f, "Discogs"),
            SourceName::MusicBrainz => write!(f, "MusicBrainz"),
            SourceName::LastFm => write!(f, "Last.fm"),
            SourceName::AcoustId => write!(f, "AcoustID"),
            SourceName::Wikipedia => write!(f, "Wikipedia"),
            SourceName::Essentia => write!(f, "Essentia"),
            SourceName::MbMapping => write!(f, "MB Mapping"),
            SourceName::ExistingTag => write!(f, "ExistingTag"),
        }
    }
}

#[derive(Debug)]
pub struct VoteResult {
    pub year: Option<(u32, f32)>,
    pub decade: Option<String>,
    pub genre: Option<(String, f32)>,
    pub subgenre: Option<(String, f32)>,
    pub needs_review: bool,
    pub sources_used: Vec<SourceName>,
}

#[derive(Debug)]
pub enum TagWriteStatus {
    Written,
    DryRun,
    NeedsReview,
    Skipped,
    Error(String),
}
