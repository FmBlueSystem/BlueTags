/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_by_label_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/browse/release/by_label.json");
    let first_deserialized: musicbrainz_rs::entity::BrowseResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::BrowseResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

