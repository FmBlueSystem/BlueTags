use crate::models::{SourceResult, TrackMetadata};
use std::path::Path;

pub struct EssentiaClassifier;

impl EssentiaClassifier {
    pub fn load(_model_path: &Path) -> anyhow::Result<Self> {
        // Card 09
        Ok(EssentiaClassifier)
    }

    pub fn analyze(&self, _track: &TrackMetadata) -> Option<SourceResult> {
        // Card 09
        None
    }
}
