//! Search ranking domain — scored hits over `QueryPlan` matches.
//!
//! Per M03-P05-IP-001/IP-002. Combines inverted + vector scores; concrete
//! BM25/HNSW math lives in adapter crates.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::cmp::Ordering;

use oya_search_query_domain::QueryPlan;

#[derive(Clone, Debug, PartialEq)]
pub struct RankedHit {
    pub document_id: String, // data_class: INTERNAL_ONLY
    pub shard_id: String,    // data_class: INTERNAL_ONLY
    pub bm25_score: f32,     // data_class: INTERNAL_ONLY
    pub vector_score: f32,   // data_class: INTERNAL_ONLY
    pub combined_score: f32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub enum RankError {
    EmptyDocumentId,
    NaNScore,
    BelowMinScore,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedHitCreate {
    pub document_id: String,
    pub shard_id: String,
    pub bm25_score: f32,
    pub vector_score: f32,
}

impl RankedHit {
    pub fn new(plan: &QueryPlan, input: RankedHitCreate) -> Result<Self, RankError> {
        if input.document_id.trim().is_empty() {
            return Err(RankError::EmptyDocumentId);
        }
        if !input.bm25_score.is_finite() || !input.vector_score.is_finite() {
            return Err(RankError::NaNScore);
        }
        let combined = combine(plan, input.bm25_score, input.vector_score);
        if combined < plan.min_score {
            return Err(RankError::BelowMinScore);
        }
        Ok(Self {
            document_id: input.document_id,
            shard_id: input.shard_id,
            bm25_score: input.bm25_score,
            vector_score: input.vector_score,
            combined_score: combined,
        })
    }

    pub fn cmp_descending(left: &Self, right: &Self) -> Ordering {
        right
            .combined_score
            .partial_cmp(&left.combined_score)
            .unwrap_or(Ordering::Equal)
    }
}

fn combine(plan: &QueryPlan, bm25: f32, vec_score: f32) -> f32 {
    use oya_search_query_domain::QueryMode::*;
    match plan.mode {
        InvertedOnly => bm25,
        VectorOnly => vec_score,
        Hybrid => 0.5 * bm25 + 0.5 * vec_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_search_query_domain::{QueryLocale, QueryMode, QueryPlanCreate};

    fn plan(mode: QueryMode, min_score: f32) -> QueryPlan {
        QueryPlan::new(QueryPlanCreate {
            query_id: "q".to_string(),
            tenant_id: "ten_alpha".to_string(),
            raw_query: "hi".to_string(),
            mode,
            locale: QueryLocale::Generic,
            limit: 5,
            min_score,
            shard_keys: vec!["s1".to_string()],
        })
        .expect("plan")
    }

    #[test]
    fn hybrid_combines_scores_evenly() {
        let p = plan(QueryMode::Hybrid, 0.0);
        let hit = RankedHit::new(
            &p,
            RankedHitCreate {
                document_id: "d1".to_string(),
                shard_id: "s1".to_string(),
                bm25_score: 0.8,
                vector_score: 0.4,
            },
        )
        .expect("hit");
        assert!((hit.combined_score - 0.6).abs() < 1e-6);
    }

    #[test]
    fn rejects_below_min_score() {
        let p = plan(QueryMode::Hybrid, 0.9);
        let err = RankedHit::new(
            &p,
            RankedHitCreate {
                document_id: "d1".to_string(),
                shard_id: "s1".to_string(),
                bm25_score: 0.1,
                vector_score: 0.1,
            },
        )
        .expect_err("below threshold");
        assert_eq!(err, RankError::BelowMinScore);
    }

    #[test]
    fn rejects_empty_document_id() {
        let p = plan(QueryMode::InvertedOnly, 0.0);
        let err = RankedHit::new(
            &p,
            RankedHitCreate {
                document_id: "".to_string(),
                shard_id: "s1".to_string(),
                bm25_score: 1.0,
                vector_score: 0.0,
            },
        )
        .expect_err("doc id required");
        assert_eq!(err, RankError::EmptyDocumentId);
    }

    #[test]
    fn cmp_descending_orders_hits_highest_first() {
        let p = plan(QueryMode::VectorOnly, 0.0);
        let make = |id: &str, vs: f32| -> RankedHit {
            RankedHit::new(
                &p,
                RankedHitCreate {
                    document_id: id.to_string(),
                    shard_id: "s1".to_string(),
                    bm25_score: 0.0,
                    vector_score: vs,
                },
            )
            .expect("hit")
        };
        let mut hits = [make("low", 0.1), make("high", 0.9), make("mid", 0.5)];
        hits.sort_by(RankedHit::cmp_descending);
        assert_eq!(hits[0].document_id, "high");
        assert_eq!(hits[2].document_id, "low");
    }
}
