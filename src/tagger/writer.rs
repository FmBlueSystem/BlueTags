use crate::models::{TagWriteStatus, VoteResult};
use anyhow::Result;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{ItemKey, ItemValue, TagItem};
use std::path::Path;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Escribe el comentario "FAKE" en el tag del archivo, sin tocar año/género.
pub fn mark_as_fake(path: &Path, dry_run: bool) -> TagWriteStatus {
    if dry_run {
        println!("[FAKE] {}", path.display());
        return TagWriteStatus::DryRun;
    }
    match try_mark_fake(path) {
        Ok(_) => TagWriteStatus::Written,
        Err(e) => TagWriteStatus::Error(e.to_string()),
    }
}

fn try_mark_fake(path: &Path) -> Result<()> {
    let mut tagged_file = Probe::open(path)?.guess_file_type()?.read()?;
    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| anyhow::anyhow!("Sin tag primario en {}", path.display()))?;
    // Campo dedicado para no pisar COMMENT que tiene metadata DJ compleja
    tag.insert_unchecked(TagItem::new(
        ItemKey::Unknown("FAKE".to_string()),
        ItemValue::Text("true".to_string()),
    ));
    tagged_file.save_to_path(path, lofty::config::WriteOptions::default())?;
    Ok(())
}

pub fn write_tags(path: &Path, vote: &VoteResult, dry_run: bool) -> TagWriteStatus {
    if vote.needs_review {
        return TagWriteStatus::NeedsReview;
    }

    if dry_run {
        print_dry_run(path, vote);
        return TagWriteStatus::DryRun;
    }

    match try_write(path, vote) {
        Ok(_) => TagWriteStatus::Written,
        Err(e) => TagWriteStatus::Error(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn try_write(path: &Path, vote: &VoteResult) -> Result<()> {
    let mut tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

    // Determinar tipo de tag antes del borrow mutable
    let is_vorbis = matches!(
        tagged_file.primary_tag().map(|t| t.tag_type()),
        Some(lofty::tag::TagType::VorbisComments)
    );

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| anyhow::anyhow!("Sin tag primario en {}", path.display()))?;

    // Año original → YEAR (Vorbis) o Year (ID3)
    // Nunca sobreescribimos DATE en Vorbis (es el año de la compilación DMC)
    if let Some((year, _)) = &vote.year {
        if is_vorbis {
            // Limpiar cualquier YEAR previo.
            // lofty lee "YEAR" de Vorbis y lo mapea a ItemKey::Year (no Unknown),
            // por eso hay que remover AMBOS para evitar triplicación.
            tag.retain(|i| match i.key() {
                ItemKey::Year => false,
                ItemKey::Unknown(k) => !k.eq_ignore_ascii_case("year"),
                _ => true,
            });
            tag.insert_unchecked(TagItem::new(
                ItemKey::Unknown("YEAR".to_string()),
                ItemValue::Text(year.to_string()),
            ));
        } else {
            tag.insert(TagItem::new(ItemKey::Year, ItemValue::Text(year.to_string())));
        }
    }

    // Género (known key — insert normal)
    if let Some((genre, _)) = &vote.genre {
        tag.insert(TagItem::new(ItemKey::Genre, ItemValue::Text(genre.clone())));
    }

    // Sub-género — Unknown key requiere insert_unchecked
    if let Some((sub, _)) = &vote.subgenre {
        tag.insert_unchecked(TagItem::new(
            ItemKey::Unknown("SUBGENRE".to_string()),
            ItemValue::Text(sub.clone()),
        ));
    }

    // Década — campo `decade` en Vorbis, GROUPING en ID3
    // NUNCA pisamos GROUPING en Vorbis (contiene el nivel de energía del DJ)
    if let Some(decade) = &vote.decade {
        let key = if is_vorbis {
            ItemKey::Unknown("decade".to_string())
        } else {
            ItemKey::Unknown("GROUPING".to_string())
        };
        tag.insert_unchecked(TagItem::new(key, ItemValue::Text(decade.clone())));
    }

    tagged_file.save_to_path(path, lofty::config::WriteOptions::default())?;
    Ok(())
}

fn print_dry_run(path: &Path, vote: &VoteResult) {
    let year_str = vote
        .year
        .as_ref()
        .map(|(y, s)| format!("{y} ({:.0}%)", s * 100.0))
        .unwrap_or_else(|| "-".to_string());
    let genre_str = vote
        .genre
        .as_ref()
        .map(|(g, s)| format!("{g} ({:.0}%)", s * 100.0))
        .unwrap_or_else(|| "-".to_string());
    let sub_str = vote
        .subgenre
        .as_ref()
        .map(|(s, _)| s.clone())
        .unwrap_or_else(|| "-".to_string());

    println!(
        "[DRY-RUN] {}\n  year={} | genre={} | subgenre={} | decade={}",
        path.display(),
        year_str,
        genre_str,
        sub_str,
        vote.decade.as_deref().unwrap_or("-"),
    );
}
