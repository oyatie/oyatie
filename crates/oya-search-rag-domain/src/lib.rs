//! Search RAG endpoint domain — request/answer envelope under Data Use Boundary.
//!
//! Per M03-P05-IP-003. `RagAnswer` carries citations back to `SerpResponse`
//! hits; `enforce_data_boundary` rejects answers whose citation set escapes
//! the requesting tenant.

#![forbid(unsafe_code)]

use oya_search_query_domain::{QueryLocale, QueryPlan};
use oya_search_serp_domain::SerpResponse;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RagCapability {
    FoundryReadOnly,
    FoundryWriteBack,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagQuery {
    pub rag_query_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub user_prompt: String,       // data_class: INTERNAL_ONLY
    pub locale: QueryLocale,       // data_class: INTERNAL_ONLY
    pub capability: RagCapability, // data_class: INTERNAL_ONLY
    pub max_citations: u16,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagCitation {
    pub document_id: String, // data_class: INTERNAL_ONLY
    pub shard_id: String,    // data_class: INTERNAL_ONLY
    pub combined_score: f32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagAnswer {
    pub rag_query_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub answer_text: String,         // data_class: INTERNAL_ONLY
    pub citations: Vec<RagCitation>, // data_class: INTERNAL_ONLY
    pub capability: RagCapability,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub enum RagError {
    EmptyRagQueryId,
    EmptyPrompt,
    MaxCitationsZero,
    TenantBoundaryViolation,
    CitationCountExceeded,
}

impl RagQuery {
    pub fn new(
        rag_query_id: String,
        tenant_id: String,
        user_prompt: String,
        locale: QueryLocale,
        capability: RagCapability,
        max_citations: u16,
    ) -> Result<Self, RagError> {
        if rag_query_id.trim().is_empty() {
            return Err(RagError::EmptyRagQueryId);
        }
        if user_prompt.trim().is_empty() {
            return Err(RagError::EmptyPrompt);
        }
        if max_citations == 0 {
            return Err(RagError::MaxCitationsZero);
        }
        Ok(Self {
            rag_query_id,
            tenant_id,
            user_prompt,
            locale,
            capability,
            max_citations,
        })
    }
}

/// Per IP-003: enforce that every citation belongs to the requesting tenant
/// and respects `max_citations`. This is the data-boundary guard wired by
/// the upstream `oya-search-rag-app::enforce_data_boundary` symbol.
pub fn enforce_data_boundary(
    query: &RagQuery,
    plan: &QueryPlan,
    serp: &SerpResponse,
    answer_text: String,
) -> Result<RagAnswer, RagError> {
    if plan.tenant_id != query.tenant_id || serp.tenant_id != query.tenant_id {
        return Err(RagError::TenantBoundaryViolation);
    }
    if serp.hits.len() as u16 > query.max_citations {
        return Err(RagError::CitationCountExceeded);
    }
    let citations = serp
        .hits
        .iter()
        .map(|hit| RagCitation {
            document_id: hit.document_id.clone(),
            shard_id: hit.shard_id.clone(),
            combined_score: hit.combined_score,
        })
        .collect();
    Ok(RagAnswer {
        rag_query_id: query.rag_query_id.clone(),
        tenant_id: query.tenant_id.clone(),
        answer_text,
        citations,
        capability: query.capability,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_search_query_domain::{QueryMode, QueryPlanCreate};
    use oya_search_rank_domain::{RankedHit, RankedHitCreate};

    fn plan(tenant: &str) -> QueryPlan {
        QueryPlan::new(QueryPlanCreate {
            query_id: "q".to_string(),
            tenant_id: tenant.to_string(),
            raw_query: "hi".to_string(),
            mode: QueryMode::Hybrid,
            locale: QueryLocale::Kr,
            limit: 5,
            min_score: 0.0,
            shard_keys: vec!["s1".to_string()],
        })
        .expect("plan")
    }

    fn serp(p: &QueryPlan, hit_ids: &[&str]) -> SerpResponse {
        let hits = hit_ids
            .iter()
            .map(|id| {
                RankedHit::new(
                    p,
                    RankedHitCreate {
                        document_id: (*id).to_string(),
                        shard_id: "s1".to_string(),
                        bm25_score: 0.5,
                        vector_score: 0.5,
                    },
                )
                .expect("hit")
            })
            .collect();
        SerpResponse::new(p, hits, hit_ids.len() as u32).expect("serp")
    }

    fn query(tenant: &str, max: u16) -> RagQuery {
        RagQuery::new(
            "rq_1".to_string(),
            tenant.to_string(),
            "what is X?".to_string(),
            QueryLocale::Kr,
            RagCapability::FoundryReadOnly,
            max,
        )
        .expect("query")
    }

    #[test]
    fn returns_answer_with_citations() {
        let p = plan("ten_kr");
        let s = serp(&p, &["d1", "d2"]);
        let q = query("ten_kr", 5);
        let ans = enforce_data_boundary(&q, &p, &s, "answer".to_string()).expect("answer");
        assert_eq!(ans.citations.len(), 2);
        assert_eq!(ans.tenant_id, "ten_kr");
    }

    #[test]
    fn rejects_cross_tenant_serp() {
        let p = plan("ten_jp");
        let s = serp(&p, &["d1"]);
        let q = query("ten_kr", 5);
        let err = enforce_data_boundary(&q, &p, &s, "answer".to_string())
            .expect_err("cross-tenant rejected");
        assert_eq!(err, RagError::TenantBoundaryViolation);
    }

    #[test]
    fn rejects_too_many_citations() {
        let p = plan("ten_kr");
        let s = serp(&p, &["d1", "d2", "d3"]);
        let q = query("ten_kr", 2);
        let err = enforce_data_boundary(&q, &p, &s, "answer".to_string())
            .expect_err("citation cap enforced");
        assert_eq!(err, RagError::CitationCountExceeded);
    }

    #[test]
    fn rejects_empty_prompt() {
        let err = RagQuery::new(
            "rq".to_string(),
            "ten_kr".to_string(),
            "   ".to_string(),
            QueryLocale::Kr,
            RagCapability::FoundryReadOnly,
            5,
        )
        .expect_err("empty prompt");
        assert_eq!(err, RagError::EmptyPrompt);
    }
}
