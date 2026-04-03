/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_annotation_entity_bdb24cb5_404b_4f60_bba4_7b730325ae47_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/annotation/entity_bdb24cb5-404b-4f60-bba4-7b730325ae47.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::annotation::Annotation> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::annotation::Annotation> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_area_Ile_de_France_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/area/Ile-de-France.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::area::Area> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::area::Area> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_artist_artist_fred_AND_type_group_AND_country_US_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/artist/artist_fred_AND_type_group_AND_country_US.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::artist::Artist> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::artist::Artist> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_cdstub_title_Doo_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/cdstub/title_Doo.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::cdstub::CDStub> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::cdstub::CDStub> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_event_unique_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/event/unique.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::event::Event> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::event::Event> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_instrument_Nose_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/instrument/Nose.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::instrument::Instrument> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::instrument::Instrument> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_label_Devils_Records_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/label/Devils_Records.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::label::Label> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::label::Label> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_place_chipping_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/place/chipping.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::place::Place> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::place::Place> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_recording_isrc_GBAHT1600302_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/recording/isrc_GBAHT1600302.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::recording::Recording> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::recording::Recording> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_recording_we_will_rock_you_AND_arid_0383dadf_2a4e_4d10_a46a_e9e041da8eb3_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/recording/we_will_rock_you_AND_arid_0383dadf-2a4e-4d10-a46a-e9e041da8eb3.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::recording::Recording> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::recording::Recording> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_drivers_license_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/release/drivers_license.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_release_Schneider_AND_Shake_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/release/release_Schneider_AND_Shake.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release::Release> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_group_release_Tenance_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/release-group/release_Tenance.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release_group::ReleaseGroup> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::release_group::ReleaseGroup> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_series_Studio_Brussel_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/series/Studio_Brussel.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::series::Series> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::series::Series> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_tag_shoegaze_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/tag/shoegaze.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::tag::Tag> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::tag::Tag> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_url_Hello_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/url/Hello.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::url::Url> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::url::Url> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_work_work_Frozen_AND_arid_4c006444_ccbf_425e_b3e7_03a98bab5997_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/search/work/work_Frozen_AND_arid_4c006444-ccbf-425e-b3e7-03a98bab5997.json");
    let first_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::work::Work> = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::search::SearchResult<musicbrainz_rs::entity::work::Work> = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

