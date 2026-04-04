use music_tagger::tagger::decade::to_decade;
use music_tagger::tagger::voter::vote;
use music_tagger::models::{SourceName, SourceResult};

// ---------------------------------------------------------------------------
// decade::to_decade
// ---------------------------------------------------------------------------

#[test]
fn test_decade_1990s() {
    assert_eq!(to_decade(1997), "1990s");
}

#[test]
fn test_decade_2000s() {
    assert_eq!(to_decade(2003), "2000s");
}

#[test]
fn test_decade_1980s() {
    assert_eq!(to_decade(1985), "1980s");
}

// ---------------------------------------------------------------------------
// voter::vote
// ---------------------------------------------------------------------------

fn make_result(source: SourceName, year: Option<u32>, genre: Option<&str>, confidence: f32) -> SourceResult {
    SourceResult {
        source,
        year,
        genre: genre.map(str::to_string),
        subgenre: None,
        confidence,
    }
}

#[test]
fn test_vote_clear_winner() {
    let results = vec![
        make_result(SourceName::Discogs, Some(1997), Some("electronic"), 0.90),
        make_result(SourceName::MusicBrainz, Some(1997), Some("electronic"), 0.85),
        make_result(SourceName::LastFm, None, Some("electronic"), 0.70),
    ];

    let vote_result = vote(results, 0.65);

    assert_eq!(vote_result.year.as_ref().map(|(y, _)| *y), Some(1997));
    assert_eq!(vote_result.genre.as_ref().map(|(g, _)| g.as_str()), Some("electronic"));
    assert!(!vote_result.needs_review);
    assert_eq!(vote_result.decade.as_deref(), Some("1990s"));
}

#[test]
fn test_vote_needs_review_when_no_data() {
    let results = vec![];
    let vote_result = vote(results, 0.65);
    assert!(vote_result.needs_review);
    assert!(vote_result.year.is_none());
    assert!(vote_result.genre.is_none());
}

#[test]
fn test_vote_conflict_weights() {
    let results = vec![
        make_result(SourceName::Discogs, Some(1995), Some("rock"), 0.90),
        make_result(SourceName::LastFm, Some(1994), Some("pop"), 0.70),
    ];

    let vote_result = vote(results, 0.65);
    // Discogs peso 3.0 para año > Last.fm 0.0 → Discogs gana año
    assert_eq!(vote_result.year.as_ref().map(|(y, _)| *y), Some(1995));
    // Last.fm peso 2.5 para género > Discogs 1.0 → Last.fm gana género
    assert_eq!(vote_result.genre.as_ref().map(|(g, _)| g.as_str()), Some("pop"));
}

#[test]
fn test_vote_genre_normalization() {
    let results = vec![
        make_result(SourceName::Discogs, None, Some("Electronic"), 0.90),
        make_result(SourceName::LastFm, None, Some("electronic"), 0.70),
    ];

    let vote_result = vote(results, 0.65);
    // Ambos votan "electronic" (normalizado), debe ganar
    assert_eq!(vote_result.genre.as_ref().map(|(g, _)| g.as_str()), Some("electronic"));
}
