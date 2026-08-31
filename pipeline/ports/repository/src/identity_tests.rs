use crate::{DigestBuilder, ObjectAlgorithm, ObjectId, RepositoryId};

#[test]
fn object_ids_preserve_algorithm_and_canonicalize_hex() {
    let sha1 = ObjectId::from_hex("A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0").unwrap();
    let sha256 = ObjectId::from_hex(&"b1".repeat(32)).unwrap();

    assert_eq!(sha1.algorithm(), ObjectAlgorithm::Sha1);
    assert_eq!(sha1.to_hex(), "a0".repeat(20));
    assert_eq!(sha256.algorithm(), ObjectAlgorithm::Sha256);
    assert_eq!(sha256.to_hex(), "b1".repeat(32));
}

#[test]
fn semantic_ids_refuse_whitespace_and_ambiguity() {
    assert!(RepositoryId::new("oyatie/oyatie").is_ok());
    assert!(RepositoryId::new("mutable identity").is_err());
    assert!(RepositoryId::new("").is_err());
}

#[test]
fn digest_framing_distinguishes_field_boundaries() {
    let mut left = DigestBuilder::new(b"test-v1");
    left.push_bytes(b"ab");
    left.push_bytes(b"c");
    let mut right = DigestBuilder::new(b"test-v1");
    right.push_bytes(b"a");
    right.push_bytes(b"bc");

    assert_ne!(left.finish(), right.finish());
}
