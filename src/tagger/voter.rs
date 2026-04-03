use crate::models::{SourceName, SourceResult, VoteResult};
use crate::tagger::decade::to_decade;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Pesos por fuente — diferenciados por campo
// ---------------------------------------------------------------------------

fn weight_track(source: &SourceName) -> f32 {
    match source {
        SourceName::Discogs => 3.0,
        SourceName::MusicBrainz => 2.0,
        SourceName::LastFm => 2.0,
        SourceName::AcoustId => 2.0,
        SourceName::Essentia => 1.0,
        SourceName::ExistingTag => 1.0,
        SourceName::MbMapping | SourceName::Wikipedia => 0.0,
    }
}

fn weight_genre(source: &SourceName) -> f32 {
    match source {
        SourceName::MbMapping => 3.0,
        SourceName::Discogs => 2.0,
        SourceName::MusicBrainz => 2.0,
        SourceName::Wikipedia => 2.0,
        SourceName::LastFm => 1.0,
        SourceName::Essentia => 1.0,
        SourceName::AcoustId | SourceName::ExistingTag => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Voting
// ---------------------------------------------------------------------------

pub fn vote(results: Vec<SourceResult>, confidence_threshold: f32) -> VoteResult {
    let mut year_scores: HashMap<u32, f32> = HashMap::new();
    let mut genre_scores: HashMap<String, f32> = HashMap::new();
    let mut subgenre_scores: HashMap<String, f32> = HashMap::new();
    let mut sources_used: Vec<SourceName> = Vec::new();

    // Acumular peso máximo posible por fuentes que realmente respondieron
    let mut max_year_weight: f32 = 0.0;
    let mut max_genre_weight: f32 = 0.0;

    for result in &results {
        let tw = weight_track(&result.source);
        let gw = weight_genre(&result.source);

        sources_used.push(result.source.clone());

        // Votar año
        if let Some(year) = result.year {
            if tw > 0.0 {
                *year_scores.entry(year).or_default() += tw * result.confidence;
                max_year_weight += tw;
            }
        }

        // Votar género
        if let Some(ref genre) = result.genre {
            let normalized = genre.to_lowercase();
            // Ignorar valores internos tipo "mbid:xxx" de AcoustID
            if !normalized.starts_with("mbid:") && gw > 0.0 {
                *genre_scores.entry(normalized).or_default() += gw * result.confidence;
                max_genre_weight += gw;
            }
        }

        // Votar subgénero
        if let Some(ref sub) = result.subgenre {
            let normalized = sub.to_lowercase();
            if gw > 0.0 {
                *subgenre_scores.entry(normalized).or_default() += gw * result.confidence;
            }
        }
    }

    // Ganador de año — normalizar por peso real de sources que votaron
    let year_winner = best_entry(&year_scores);
    let year_result = year_winner.map(|(val, score)| {
        let normalized = if max_year_weight > 0.0 { score / max_year_weight } else { 0.0 };
        (*val, normalized)
    });

    // Ganador de género
    let genre_winner = best_entry(&genre_scores);
    let genre_result = genre_winner.map(|(val, score)| {
        let normalized = if max_genre_weight > 0.0 { score / max_genre_weight } else { 0.0 };
        (val.clone(), normalized)
    });

    // Ganador de subgénero
    let subgenre_winner = best_entry(&subgenre_scores);
    let subgenre_result = subgenre_winner.map(|(val, score)| {
        let normalized = if max_genre_weight > 0.0 { score / max_genre_weight } else { 0.0 };
        (val.clone(), normalized)
    });

    // needs_review si ningún campo supera el umbral
    let year_ok = year_result.as_ref().map(|(_, s)| *s >= confidence_threshold).unwrap_or(false);
    let genre_ok = genre_result.as_ref().map(|(_, s)| *s >= confidence_threshold).unwrap_or(false);
    let needs_review = !year_ok && !genre_ok;

    // Decade desde el año ganador
    let decade = year_result.as_ref().map(|(year, _)| to_decade(*year));

    VoteResult {
        year: year_result,
        decade,
        genre: genre_result,
        subgenre: subgenre_result,
        needs_review,
        sources_used,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn best_entry<K: Eq + std::hash::Hash>(map: &HashMap<K, f32>) -> Option<(&K, f32)> {
    map.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(k, v)| (k, *v))
}
