use crate::models::{SourceResult, VoteResult};

pub fn vote(_results: Vec<SourceResult>, _confidence_threshold: f32) -> VoteResult {
    // Card 10
    VoteResult {
        year: None,
        decade: None,
        genre: None,
        subgenre: None,
        needs_review: true,
        sources_used: vec![],
    }
}
