use crate::models::{SourceName, SourceResult, VoteResult};
use crate::tagger::decade::to_decade;
use std::collections::HashMap;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Normalización de sinónimos — MetaBrainz CSVs + fallback manual
// ---------------------------------------------------------------------------

/// Tabla de normalización cargada una sola vez desde los CSVs de MetaBrainz
/// (metabrainz/genre-matching) + extensiones manuales para 80s.
static GENRE_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

const DISCOGS_CSV: &str = include_str!("../../data/discogs_genre_mapping.csv");
const LASTFM_CSV: &str = include_str!("../../data/lastfm_genre_mapping.csv");

fn build_genre_map() -> HashMap<String, String> {
    let mut map = HashMap::new();

    // --- Cargar CSVs de MetaBrainz ---
    // Formato Discogs: genre,subgenre,mb_tag1,mb_tag2
    // Formato Last.fm: genre,subgenre,mb_tag1,mb_tag2,
    for csv in [DISCOGS_CSV, LASTFM_CSV] {
        for line in csv.lines().skip(1) {
            let cols: Vec<&str> = line.split(',').map(|s| s.trim().trim_matches('"')).collect();
            if cols.len() >= 3 {
                let source_sub = cols[1].to_lowercase();
                let mb_tag = cols[2].to_lowercase();
                if !source_sub.is_empty() && !mb_tag.is_empty() {
                    map.insert(source_sub, mb_tag);
                }
            }
        }
    }

    // --- Extensiones manuales para 80s (no cubiertas por los CSVs) ---
    let manual = [
        // Familia Funk/Soul → preservar identidad
        ("funk / soul", "r&b"), ("funk/soul", "r&b"),
        ("contemporary r&b", "r&b"), ("rnb/swing", "r&b"),
        ("rhythm & blues", "r&b"), ("rhythm and blues", "r&b"),
        ("rnb", "r&b"), ("r&b", "r&b"),
        ("p.funk", "funk"), ("bayou funk", "funk"),
        ("neo soul", "soul"),
        ("euro disco", "disco"), ("italo-disco", "disco"),
        ("hi-nrg", "disco"), ("hi nrg", "disco"),

        // Electronic → preservar subgéneros
        ("synthpop", "synth-pop"), ("electropop", "synth-pop"),
        ("no wave", "new wave"),
        ("acid house", "house"),
        ("electro funk", "electro"),
        ("eurodance", "dance-pop"),

        // Hip Hop variantes
        ("hip-hop", "hip hop"), ("hiphop", "hip hop"),
        ("pop rap", "hip hop"), ("gangsta", "hip hop"),

        // Rock variantes
        ("classic rock", "rock"), ("pop rock", "rock"),
        ("hard rock", "rock"), ("arena rock", "rock"),
        ("rock & roll", "rock"), ("indie rock", "alternative rock"),

        // Pop variantes
        ("classic pop", "pop"), ("adult contemporary", "pop"),
        ("latin pop", "latin"), ("latin", "latin"),

        // Jazz variantes
        ("modern jazz", "jazz"),

        // Otros
        ("dub", "reggae"), ("ska", "reggae"),
        ("punk rock", "punk"),
    ];

    for (source, canonical) in manual {
        map.entry(source.to_string()).or_insert_with(|| canonical.to_string());
    }

    map
}

fn normalize_genre(genre: &str) -> String {
    let map = GENRE_MAP.get_or_init(build_genre_map);
    map.get(genre).cloned().unwrap_or_else(|| genre.to_string())
}

// ---------------------------------------------------------------------------
// Pesos por fuente — diferenciados por campo
// ---------------------------------------------------------------------------

// Peso para AÑO: Discogs es una base de datos de releases → muy preciso.
// MusicBrainz y AcoustID también confiables. Last.fm no provee año.
fn weight_track(source: &SourceName) -> f32 {
    match source {
        SourceName::Discogs     => 3.0, // base de datos de releases, máxima precisión
        SourceName::MusicBrainz => 2.5, // editorial curado, confiable con fix de earliest release
        SourceName::AcoustId    => 2.0, // fingerprint → MB, confiable
        SourceName::LastFm      => 0.0, // no provee año confiable
        SourceName::Essentia    => 1.0,
        SourceName::ExistingTag => 1.0,
        SourceName::MbMapping | SourceName::Wikipedia | SourceName::WikiSong
        | SourceName::AcousticBrainz => 0.0,
    }
}

// Peso para GÉNERO: precisión inversamente proporcional a la granularidad de la taxonomía.
fn weight_genre(source: &SourceName) -> f32 {
    match source {
        SourceName::AcousticBrainz => 4.0, // multi-fuente pre-reconciliada, 1.3M recordings
        SourceName::WikiSong    => 3.5, // artículo específico de la canción
        SourceName::MusicBrainz => 3.0, // editorial, peer-reviewed, taxonomía granular
        SourceName::LastFm      => 2.5, // crowdsourced por oyentes, muy granular
        SourceName::Essentia    => 2.0, // análisis de audio, sin sesgo editorial
        SourceName::Wikipedia   => 1.5, // género del artista (no del track), menor precisión
        SourceName::Discogs     => 1.0, // taxonomía muy gruesa, poco fiable para 80s pop/soul
        SourceName::MbMapping | SourceName::AcoustId | SourceName::ExistingTag => 0.0,
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

    // Peso máximo posible por fuentes que respondieron (para normalización híbrida)
    let mut max_year_weight: f32 = 0.0;
    let mut max_genre_weight: f32 = 0.0;

    // Contar fuentes primarias por campo (excluir Wikipedia = validación, no fuente)
    let mut year_source_count: u32 = 0;
    let mut genre_source_count: u32 = 0;

    for result in &results {
        let tw = weight_track(&result.source);
        let gw = weight_genre(&result.source);
        let is_primary = result.source != SourceName::Wikipedia;

        sources_used.push(result.source.clone());

        // Votar año
        if let Some(year) = result.year {
            if tw > 0.0 {
                *year_scores.entry(year).or_default() += tw * result.confidence;
                max_year_weight += tw;
                if is_primary { year_source_count += 1; }
            }
        }

        // Votar género (con normalización de sinónimos)
        if let Some(ref genre) = result.genre {
            let lower = genre.to_lowercase();
            if !lower.starts_with("mbid:") && gw > 0.0 {
                let normalized = normalize_genre(&lower);
                *genre_scores.entry(normalized).or_default() += gw * result.confidence;
                max_genre_weight += gw;
                if is_primary { genre_source_count += 1; }
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

    // ---------------------------------------------------------------------------
    // Parent-genre coalescence: si un subgénero y su padre compiten,
    // fusionar los votos del subgénero al padre (el sub queda como subgénero).
    coalesce_parent_genres(&mut genre_scores);

    // Normalización por pluralidad + source count penalty
    // ---------------------------------------------------------------------------
    // plurality = winner / total_votes (dominio entre candidatos)
    // source_factor = (n+1)/(n+2) (penalización suave por pocas fuentes)
    // final = plurality × source_factor

    let total_year_score: f32 = year_scores.values().sum();
    let total_genre_score: f32 = genre_scores.values().sum();

    let year_winner = best_entry(&year_scores);
    let year_result = year_winner.map(|(val, score)| {
        let plurality = if total_year_score > 0.0 { score / total_year_score } else { 0.0 };
        let confidence = plurality * source_factor(year_source_count);
        (*val, confidence)
    });

    let genre_winner = best_entry(&genre_scores);
    let genre_result = genre_winner.map(|(val, score)| {
        let plurality = if total_genre_score > 0.0 { score / total_genre_score } else { 0.0 };
        let confidence = plurality * source_factor(genre_source_count);
        (val.clone(), confidence)
    });

    let subgenre_winner = best_entry(&subgenre_scores);
    let total_subgenre_score: f32 = subgenre_scores.values().sum();
    let subgenre_result = subgenre_winner.map(|(val, score)| {
        let plurality = if total_subgenre_score > 0.0 { score / total_subgenre_score } else { 0.0 };
        (val.clone(), plurality)
    });

    // needs_review: género es obligatorio.
    // Absolute floor: si el top raw score ≥ 2.5, la fuente es suficientemente fuerte
    // para pasar aunque haya desacuerdo (ej. WikiSong peso 3.5 × conf 0.85 = 2.975).
    let genre_ok = genre_result.as_ref().map(|(_, s)| *s >= confidence_threshold).unwrap_or(false);
    let genre_strong = genre_scores.values().copied().fold(0.0f32, f32::max) >= 2.5;
    let needs_review = !(genre_ok || genre_strong)
        || (genre_ok && genre_source_count < 2 && year_result.is_none() && !genre_strong);

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

/// Si un subgénero y su padre compiten en los votos, fusionar el sub al padre.
/// Ej: "synth-pop"(1.0) + "pop"(2.3) → "pop"(3.3), "synth-pop" se mueve a subgenre.
fn coalesce_parent_genres(scores: &mut HashMap<String, f32>) {
    const PARENT_MAP: &[(&str, &str)] = &[
        ("synth-pop", "pop"),
        ("dance-pop", "pop"),
        ("electro", "electronic"),
        ("house", "electronic"),
        ("freestyle", "electronic"),
        ("breakbeat", "electronic"),
        ("new wave", "rock"),
        ("post-punk", "rock"),
        ("alternative rock", "rock"),
        ("punk", "rock"),
        ("new jack swing", "r&b"),
        ("funk", "r&b"),
        ("soul", "r&b"),
        ("disco", "r&b"),
        ("gospel", "soul"),
    ];

    for &(child, parent) in PARENT_MAP {
        let child_score = *scores.get(child).unwrap_or(&0.0);
        let parent_score = *scores.get(parent).unwrap_or(&0.0);
        // Solo fusionar si AMBOS existen como candidatos
        if child_score > 0.0 && parent_score > 0.0 {
            *scores.entry(parent.to_string()).or_default() += child_score;
            scores.remove(child);
        }
    }
}

/// Penalización suave por pocas fuentes: factor(n) = (n+1)/(n+2)
/// n=1 → 0.67, n=2 → 0.75, n=3 → 0.80, n=4 → 0.83
fn source_factor(n: u32) -> f32 {
    (n as f32 + 1.0) / (n as f32 + 2.0)
}

/// Pre-voto rápido: retorna el género con mayor peso acumulado.
/// Usado por el pipeline para decidir qué género validar con Wikipedia.
pub fn quick_genre_leader(results: &[SourceResult]) -> Option<String> {
    let mut scores: HashMap<String, f32> = HashMap::new();
    for r in results {
        if let Some(ref genre) = r.genre {
            let g = genre.to_lowercase();
            if !g.starts_with("mbid:") {
                let w = weight_genre(&r.source);
                if w > 0.0 {
                    *scores.entry(g).or_default() += w * r.confidence;
                }
            }
        }
    }
    best_entry(&scores).map(|(g, _)| g.clone())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn best_entry<K: Eq + std::hash::Hash>(map: &HashMap<K, f32>) -> Option<(&K, f32)> {
    map.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(k, v)| (k, *v))
}
