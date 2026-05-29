//! Similarity-search value objects for the vector substrate (P10-vector).
//!
//! `DistanceMetric` and `SearchHit` are backported from the P10 impl-plan
//! (oya-vector-similarity-kernel/src/types.rs) into this existing live crate
//! per the merge-into-existing-crates execution variant
//! (F-M02B-PLAN-LIVE-CRATE-RECONCILIATION; delta 1).

#![forbid(unsafe_code)]

/// Distance metric used for ANN similarity search.
///
/// Matches the `DistanceMetric` spec in the P10 impl-plan
/// (`oya-vector-similarity-kernel/src/types.rs`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DistanceMetric {
    /// Cosine similarity (1 − cos θ); lower score = closer.
    Cosine,
    /// Dot-product / inner-product; higher score = closer.
    DotProduct,
    /// Euclidean (L2) distance; lower score = closer.
    L2,
}

impl DistanceMetric {
    /// Returns `true` for metrics where a lower score indicates closer vectors.
    pub fn lower_is_closer(self) -> bool {
        matches!(self, Self::Cosine | Self::L2)
    }
}

/// A single hit returned by an ANN similarity search.
///
/// Matches the `SearchHit` spec in the P10 impl-plan
/// (`oya-vector-similarity-kernel/src/types.rs`).
/// IDs are `String` to remain dep-free at the domain layer;
/// both `embedding_id` and `object_id` must be well-formed UUID strings.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    /// Stable identifier of the stored embedding row (UUID string).
    pub embedding_id: String, // data_class: INTERNAL_ONLY
    /// Ontology object the embedding belongs to (UUID string).
    pub object_id: String, // data_class: INTERNAL_ONLY
    /// Object-type discriminant (e.g. `"entity"`, `"person"`).
    pub object_type: String, // data_class: INTERNAL_ONLY
    /// Model that produced the embedding.
    pub model_id: String, // data_class: INTERNAL_ONLY
    /// Computed distance score (interpretation depends on [`DistanceMetric`]).
    pub distance: f32, // data_class: INTERNAL_ONLY
    /// Arbitrary JSON metadata stored alongside the embedding.
    ///
    /// Uses `serde_json::Value` so that numeric, boolean, null, and nested
    /// JSON values from `jsonb` columns are representable without loss.
    pub metadata: serde_json::Value, // data_class: INTERNAL_ONLY
}

impl SearchHit {
    /// Construct a new `SearchHit`, rejecting any empty field and any
    /// `embedding_id` / `object_id` that is not a well-formed UUID
    /// (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
    pub fn new(
        embedding_id: impl Into<String>,
        object_id: impl Into<String>,
        object_type: impl Into<String>,
        model_id: impl Into<String>,
        distance: f32,
        metadata: serde_json::Value,
    ) -> Result<Self, SearchHitError> {
        let embedding_id = embedding_id.into();
        let object_id = object_id.into();
        let object_type = object_type.into();
        let model_id = model_id.into();
        if embedding_id.trim().is_empty() {
            return Err(SearchHitError::EmptyEmbeddingId);
        }
        if !is_uuid(&embedding_id) {
            return Err(SearchHitError::InvalidEmbeddingIdFormat);
        }
        if object_id.trim().is_empty() {
            return Err(SearchHitError::EmptyObjectId);
        }
        if !is_uuid(&object_id) {
            return Err(SearchHitError::InvalidObjectIdFormat);
        }
        if object_type.trim().is_empty() {
            return Err(SearchHitError::EmptyObjectType);
        }
        if model_id.trim().is_empty() {
            return Err(SearchHitError::EmptyModelId);
        }
        Ok(Self {
            embedding_id,
            object_id,
            object_type,
            model_id,
            distance,
            metadata,
        })
    }
}

/// Returns `true` iff `s` matches the canonical UUID format
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (case-insensitive hex).
fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    let dashes: [usize; 4] = [8, 13, 18, 23];
    for &pos in &dashes {
        if b[pos] != b'-' {
            return false;
        }
    }
    for (i, &byte) in b.iter().enumerate() {
        if dashes.contains(&i) {
            continue;
        }
        if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

/// Errors produced when constructing a [`SearchHit`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchHitError {
    EmptyEmbeddingId,
    /// `embedding_id` is not a well-formed UUID.
    InvalidEmbeddingIdFormat,
    EmptyObjectId,
    /// `object_id` is not a well-formed UUID.
    InvalidObjectIdFormat,
    EmptyObjectType,
    EmptyModelId,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const EMB_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";
    const OBJ_UUID: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

    fn empty_meta() -> serde_json::Value {
        serde_json::Value::Object(serde_json::Map::new())
    }

    #[test]
    fn distance_metric_lower_is_closer_cosine() {
        assert!(DistanceMetric::Cosine.lower_is_closer());
    }

    #[test]
    fn distance_metric_lower_is_closer_l2() {
        assert!(DistanceMetric::L2.lower_is_closer());
    }

    #[test]
    fn distance_metric_dot_product_higher_is_closer() {
        assert!(!DistanceMetric::DotProduct.lower_is_closer());
    }

    #[test]
    fn distance_metric_ord_ordering() {
        assert!(DistanceMetric::Cosine < DistanceMetric::DotProduct);
        assert!(DistanceMetric::DotProduct < DistanceMetric::L2);
    }

    #[test]
    fn search_hit_new_valid() {
        let hit = SearchHit::new(
            EMB_UUID,
            OBJ_UUID,
            "entity",
            "text-embedding-3-small",
            0.12,
            empty_meta(),
        )
        .expect("valid hit");
        assert_eq!(hit.embedding_id, EMB_UUID);
        assert_eq!(hit.object_id, OBJ_UUID);
        assert_eq!(hit.object_type, "entity");
        assert_eq!(hit.model_id, "text-embedding-3-small");
        assert!((hit.distance - 0.12_f32).abs() < f32::EPSILON);
        assert_eq!(
            hit.metadata,
            serde_json::Value::Object(serde_json::Map::new())
        );
    }

    #[test]
    fn search_hit_carries_metadata() {
        let meta = serde_json::json!({"source": "crawler", "score": 42, "active": true});
        let hit = SearchHit::new(
            EMB_UUID,
            OBJ_UUID,
            "entity",
            "text-embedding-3-small",
            0.5,
            meta.clone(),
        )
        .expect("valid hit with metadata");
        assert_eq!(hit.metadata, meta);
    }

    #[test]
    fn search_hit_rejects_empty_embedding_id() {
        let err = SearchHit::new("  ", OBJ_UUID, "entity", "model-x", 0.0, empty_meta())
            .expect_err("empty embedding_id rejected");
        assert_eq!(err, SearchHitError::EmptyEmbeddingId);
    }

    // P1: synthetic violation — non-UUID embedding_id must be rejected
    #[test]
    fn search_hit_rejects_non_uuid_embedding_id() {
        let err = SearchHit::new(
            "not-a-uuid",
            OBJ_UUID,
            "entity",
            "model-x",
            0.0,
            empty_meta(),
        )
        .expect_err("non-UUID embedding_id rejected");
        assert_eq!(err, SearchHitError::InvalidEmbeddingIdFormat);
    }

    // P1: synthetic violation — non-UUID object_id must be rejected
    #[test]
    fn search_hit_rejects_non_uuid_object_id() {
        let err = SearchHit::new(
            EMB_UUID,
            "not-a-uuid",
            "entity",
            "model-x",
            0.0,
            empty_meta(),
        )
        .expect_err("non-UUID object_id rejected");
        assert_eq!(err, SearchHitError::InvalidObjectIdFormat);
    }

    #[test]
    fn search_hit_rejects_empty_object_type() {
        let err = SearchHit::new(EMB_UUID, OBJ_UUID, "  ", "model-x", 0.0, empty_meta())
            .expect_err("empty object_type rejected");
        assert_eq!(err, SearchHitError::EmptyObjectType);
    }

    #[test]
    fn search_hit_rejects_empty_model_id() {
        let err = SearchHit::new(EMB_UUID, OBJ_UUID, "entity", "", 0.0, empty_meta())
            .expect_err("empty model_id rejected");
        assert_eq!(err, SearchHitError::EmptyModelId);
    }

    #[test]
    fn distance_metric_clone_copy_consistent() {
        let m = DistanceMetric::Cosine;
        let m2 = m;
        assert_eq!(m, m2);
    }

    #[test]
    fn is_uuid_rejects_short_string() {
        assert!(!is_uuid("550e8400-e29b-41d4-a716"));
    }

    #[test]
    fn is_uuid_rejects_wrong_dash_position() {
        // Same length but dash in wrong place
        assert!(!is_uuid("550e8400xe29b-41d4-a716-446655440000"));
    }
}
