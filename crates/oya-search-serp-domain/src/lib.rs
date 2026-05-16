//! Search results page (SERP) response (domain types).
//!
//! Per M03-P05-IP-001/IP-002. Wraps `RankedHit`s with paging metadata and a
//! truncated-flag for over-budget queries.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_search_query_domain::QueryPlan;
use oya_search_rank_domain::RankedHit;

#[derive(Clone, Debug, PartialEq)]
pub struct SerpResponse {
    pub query_id: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub hits: Vec<RankedHit>, // data_class: INTERNAL_ONLY
    pub total_estimated: u32, // data_class: INTERNAL_ONLY
    pub truncated: bool,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub enum SerpError {
    QueryIdMismatch,
    TenantMismatch,
    BeyondLimit,
}

impl SerpResponse {
    pub fn new(
        plan: &QueryPlan,
        hits: Vec<RankedHit>,
        total_estimated: u32,
    ) -> Result<Self, SerpError> {
        if hits.len() as u16 > plan.limit {
            return Err(SerpError::BeyondLimit);
        }
        let truncated = total_estimated > u32::from(plan.limit);
        Ok(Self {
            query_id: plan.query_id.clone(),
            tenant_id: plan.tenant_id.clone(),
            hits,
            total_estimated,
            truncated,
        })
    }

    pub fn hit_count(&self) -> usize {
        self.hits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_search_query_domain::{QueryLocale, QueryMode, QueryPlanCreate};
    use oya_search_rank_domain::RankedHitCreate;

    fn plan(limit: u16) -> QueryPlan {
        QueryPlan::new(QueryPlanCreate {
            query_id: "q1".to_string(),
            tenant_id: "ten_kr".to_string(),
            raw_query: "hi".to_string(),
            mode: QueryMode::Hybrid,
            locale: QueryLocale::Kr,
            limit,
            min_score: 0.0,
            shard_keys: vec!["s1".to_string()],
        })
        .expect("plan")
    }

    fn hit(p: &QueryPlan, id: &str, score: f32) -> RankedHit {
        RankedHit::new(
            p,
            RankedHitCreate {
                document_id: id.to_string(),
                shard_id: "s1".to_string(),
                bm25_score: score,
                vector_score: score,
            },
        )
        .expect("hit")
    }

    #[test]
    fn builds_serp_within_limit() {
        let p = plan(3);
        let serp =
            SerpResponse::new(&p, vec![hit(&p, "d1", 0.8), hit(&p, "d2", 0.5)], 2).expect("serp");
        assert_eq!(serp.hit_count(), 2);
        assert!(!serp.truncated);
    }

    #[test]
    fn rejects_more_hits_than_limit() {
        let p = plan(1);
        let err = SerpResponse::new(&p, vec![hit(&p, "d1", 0.1), hit(&p, "d2", 0.2)], 5)
            .expect_err("too many hits");
        assert_eq!(err, SerpError::BeyondLimit);
    }

    #[test]
    fn flags_truncated_when_estimate_exceeds_limit() {
        let p = plan(2);
        let serp = SerpResponse::new(&p, vec![hit(&p, "d1", 0.5)], 99).expect("serp");
        assert!(serp.truncated);
    }

    #[test]
    fn propagates_plan_identity() {
        let p = plan(5);
        let serp = SerpResponse::new(&p, vec![], 0).expect("serp");
        assert_eq!(serp.query_id, "q1");
        assert_eq!(serp.tenant_id, "ten_kr");
    }
}
