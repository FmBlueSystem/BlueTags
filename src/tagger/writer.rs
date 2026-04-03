use crate::models::{TagWriteStatus, VoteResult};
use anyhow::Result;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{ItemKey, ItemValue, TagItem};
use std::path::Path;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

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

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| anyhow::anyhow!("Sin tag primario en {}", path.display()))?;

    // Año
    if let Some((year, _)) = &vote.year {
        tag.insert(TagItem::new(
            ItemKey::Year,
            ItemValue::Text(year.to_string()),
        ));
    }

    // Género
    if let Some((genre, _)) = &vote.genre {
        tag.insert(TagItem::new(
            ItemKey::Genre,
            ItemValue::Text(genre.clone()),
        ));
    }

    // Sub-género (campo custom)
    if let Some((sub, _)) = &vote.subgenre {
        tag.insert(TagItem::new(
            ItemKey::Unknown("SUBGENRE".to_string()),
            ItemValue::Text(sub.clone()),
        ));
    }

    // Década (GROUPING / TGRP)
    if let Some(decade) = &vote.decade {
        tag.insert(TagItem::new(
            ItemKey::Unknown("GROUPING".to_string()),
            ItemValue::Text(decade.clone()),
        ));
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
