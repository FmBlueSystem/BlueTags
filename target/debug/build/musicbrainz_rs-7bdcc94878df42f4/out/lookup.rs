/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_area_45f07934_675a_46d6_a577_6f8637a411b1_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/area/45f07934-675a-46d6-a577-6f8637a411b1.json");
    let first_deserialized: musicbrainz_rs::entity::area::Area = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::area::Area = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_artist_5b11f4ce_a62d_471e_81fc_a69a8278c7da_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/artist/5b11f4ce-a62d-471e-81fc-a69a8278c7da.json");
    let first_deserialized: musicbrainz_rs::entity::artist::Artist = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::artist::Artist = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_artist_db92a151_1ac2_438b_bc43_b82e149ddd50_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/artist/db92a151-1ac2-438b-bc43-b82e149ddd50.json");
    let first_deserialized: musicbrainz_rs::entity::artist::Artist = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::artist::Artist = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_event_fe39727a_3d21_4066_9345_3970cbd6cca4_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/event/fe39727a-3d21-4066-9345-3970cbd6cca4.json");
    let first_deserialized: musicbrainz_rs::entity::event::Event = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::event::Event = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_genre_f66d7266_eb3d_4ef3_b4d8_b7cd992f918b_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/genre/f66d7266-eb3d-4ef3-b4d8-b7cd992f918b.json");
    let first_deserialized: musicbrainz_rs::entity::genre::Genre = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::genre::Genre = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_instrument_dd430e7f_36ba_49a5_825b_80a525e69190_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/instrument/dd430e7f-36ba-49a5-825b-80a525e69190.json");
    let first_deserialized: musicbrainz_rs::entity::instrument::Instrument = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::instrument::Instrument = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_label_46f0f4cd_8aab_4b33_b698_f459faf64190_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/label/46f0f4cd-8aab-4b33-b698-f459faf64190.json");
    let first_deserialized: musicbrainz_rs::entity::label::Label = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::label::Label = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_place_478558f9_a951_4067_ad91_e83f6ba63e74_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/place/478558f9-a951-4067-ad91-e83f6ba63e74.json");
    let first_deserialized: musicbrainz_rs::entity::place::Place = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::place::Place = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_recording_b9ad642e_b012_41c7_b72a_42cf4911f9ff_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/recording/b9ad642e-b012-41c7-b72a-42cf4911f9ff.json");
    let first_deserialized: musicbrainz_rs::entity::recording::Recording = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::recording::Recording = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_59211ea4_ffd2_4ad9_9a4e_941d3148024a_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release/59211ea4-ffd2-4ad9-9a4e-941d3148024a.json");
    let first_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_942b9e7b_3a43_4f22_b200_ab69ee302d16_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release/942b9e7b-3a43-4f22-b200-ab69ee302d16.json");
    let first_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_987f3e2d_22a6_4a4f_b840_c80c26b8b91a_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release/987f3e2d-22a6-4a4f-b840-c80c26b8b91a.json");
    let first_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_b1dc9838_adf3_43f2_93f9_802b46e5fe59_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release/b1dc9838-adf3-43f2-93f9-802b46e5fe59.json");
    let first_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_c9d52105_5c20_3216_bc1b_e54918f8f688_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release/c9d52105-5c20-3216-bc1b-e54918f8f688.json");
    let first_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release::Release = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_release_group_c9fdb94c_4975_4ed6_a96f_ef6d80bb7738_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/release-group/c9fdb94c-4975-4ed6-a96f-ef6d80bb7738.json");
    let first_deserialized: musicbrainz_rs::entity::release_group::ReleaseGroup = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::release_group::ReleaseGroup = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_series_c5dd4f78_4160_458a_9edc_f87c5ebbc700_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/series/c5dd4f78-4160-458a-9edc-f87c5ebbc700.json");
    let first_deserialized: musicbrainz_rs::entity::series::Series = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::series::Series = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_url_46d8f693_52e4_4d03_936f_7ca8459019a7_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/url/46d8f693-52e4-4d03-936f-7ca8459019a7.json");
    let first_deserialized: musicbrainz_rs::entity::url::Url = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::url::Url = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_work_b1df2cf3_69a9_3bc0_be44_f71e79b27a22_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/work/b1df2cf3-69a9-3bc0-be44-f71e79b27a22.json");
    let first_deserialized: musicbrainz_rs::entity::work::Work = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::work::Work = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

/// Check that the original JSON retrieved from the MusicBrainz API can be deserialized, then
/// serialized and deserialized again without information loss.
#[cfg(not(feature = "legacy_serialize"))]
#[test]
#[allow(non_snake_case)]
fn test_work_c1b0e8a2_2461_4d48_9a89_f4e6d624d342_json() {
    let data = include_str!("/Users/freddymolina/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/musicbrainz_rs-0.9.1/tests/serde/data/lookup/work/c1b0e8a2-2461-4d48-9a89-f4e6d624d342.json");
    let first_deserialized: musicbrainz_rs::entity::work::Work = serde_json::from_str(&data).expect("first deserialization failed");

    let serialized = serde_json::to_string(&first_deserialized).expect("serialization failed");
    let second_deserialized: musicbrainz_rs::entity::work::Work = serde_json::from_str(&serialized).expect("second deserialization failed");

    assert_eq!(first_deserialized, second_deserialized);
}

