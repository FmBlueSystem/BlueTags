pub struct GenreValidation {
    pub parent_genre: Option<String>,
    pub origin_decade: Option<String>,
    pub confirmed: bool,
}

pub async fn validate_genre(_genre: &str) -> Option<GenreValidation> {
    // Card 08
    None
}
