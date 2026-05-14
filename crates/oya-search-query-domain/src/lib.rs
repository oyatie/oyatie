//! Search query planner (domain types).
//!
//! Per M03-P05-IP-001/IP-002: builds the query plan consumed by both
//! inverted-index and vector-index lookups. Pure types; concrete planners
//! choose physical operators downstream.

#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueryMode {
    InvertedOnly,
    VectorOnly,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueryLocale {
    Kr,
    Jp,
    En,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryPlan {
    pub query_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub raw_query: String,       // data_class: INTERNAL_ONLY
    pub mode: QueryMode,         // data_class: INTERNAL_ONLY
    pub locale: QueryLocale,     // data_class: INTERNAL_ONLY
    pub limit: u16,              // data_class: INTERNAL_ONLY
    pub min_score: f32,          // data_class: INTERNAL_ONLY
    pub shard_keys: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryPlanError {
    EmptyQueryId,
    EmptyTenant,
    EmptyQueryText,
    LimitZero,
    NegativeOrNanMinScore,
    NoShardKeys,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryPlanCreate {
    pub query_id: String,
    pub tenant_id: String,
    pub raw_query: String,
    pub mode: QueryMode,
    pub locale: QueryLocale,
    pub limit: u16,
    pub min_score: f32,
    pub shard_keys: Vec<String>,
}

impl QueryPlan {
    pub fn new(input: QueryPlanCreate) -> Result<Self, QueryPlanError> {
        if input.query_id.trim().is_empty() {
            return Err(QueryPlanError::EmptyQueryId);
        }
        if input.tenant_id.trim().is_empty() {
            return Err(QueryPlanError::EmptyTenant);
        }
        if input.raw_query.trim().is_empty() {
            return Err(QueryPlanError::EmptyQueryText);
        }
        if input.limit == 0 {
            return Err(QueryPlanError::LimitZero);
        }
        if !input.min_score.is_finite() || input.min_score < 0.0 {
            return Err(QueryPlanError::NegativeOrNanMinScore);
        }
        if input.shard_keys.is_empty() {
            return Err(QueryPlanError::NoShardKeys);
        }
        Ok(Self {
            query_id: input.query_id,
            tenant_id: input.tenant_id,
            raw_query: input.raw_query,
            mode: input.mode,
            locale: input.locale,
            limit: input.limit,
            min_score: input.min_score,
            shard_keys: input.shard_keys,
        })
    }

    pub fn requires_vector(&self) -> bool {
        matches!(self.mode, QueryMode::VectorOnly | QueryMode::Hybrid)
    }

    pub fn requires_inverted(&self) -> bool {
        matches!(self.mode, QueryMode::InvertedOnly | QueryMode::Hybrid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> QueryPlanCreate {
        QueryPlanCreate {
            query_id: "q_001".to_string(),
            tenant_id: "ten_kr".to_string(),
            raw_query: "안녕".to_string(),
            mode: QueryMode::Hybrid,
            locale: QueryLocale::Kr,
            limit: 10,
            min_score: 0.0,
            shard_keys: vec!["shard_kr_001".to_string()],
        }
    }

    #[test]
    fn builds_hybrid_plan() {
        let plan = QueryPlan::new(base()).expect("plan");
        assert!(plan.requires_vector() && plan.requires_inverted());
        assert_eq!(plan.limit, 10);
    }

    #[test]
    fn rejects_zero_limit() {
        let err = QueryPlan::new(QueryPlanCreate { limit: 0, ..base() }).expect_err("limit zero");
        assert_eq!(err, QueryPlanError::LimitZero);
    }

    #[test]
    fn rejects_negative_min_score() {
        let err = QueryPlan::new(QueryPlanCreate {
            min_score: -0.1,
            ..base()
        })
        .expect_err("negative score");
        assert_eq!(err, QueryPlanError::NegativeOrNanMinScore);
    }

    #[test]
    fn requires_at_least_one_shard_key() {
        let err = QueryPlan::new(QueryPlanCreate {
            shard_keys: vec![],
            ..base()
        })
        .expect_err("no shards");
        assert_eq!(err, QueryPlanError::NoShardKeys);
    }

    #[test]
    fn inverted_only_excludes_vector() {
        let plan = QueryPlan::new(QueryPlanCreate {
            mode: QueryMode::InvertedOnly,
            ..base()
        })
        .expect("plan");
        assert!(plan.requires_inverted());
        assert!(!plan.requires_vector());
    }
}
