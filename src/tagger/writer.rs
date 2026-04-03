use crate::models::{TagWriteStatus, VoteResult};
use std::path::Path;

pub fn write_tags(_path: &Path, _vote: &VoteResult, dry_run: bool) -> TagWriteStatus {
    // Card 11
    if dry_run {
        TagWriteStatus::DryRun
    } else {
        TagWriteStatus::Skipped
    }
}
