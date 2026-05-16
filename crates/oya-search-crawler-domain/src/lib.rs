//! Search crawler domain: target descriptors and fetch outcomes.
//!
//! Per M03-P05-IP-001: feeds inverted/vector indexes via the parser pipeline.
//! Pure types — no external runtime; consumers wire concrete fetchers in
//! adapter crates per ADR-0015 dep-direction.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrawlScheme {
    Https,
    File,
    InternalFeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrawlPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrawlTarget {
    pub target_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub scheme: CrawlScheme,               // data_class: INTERNAL_ONLY
    pub canonical_url: String,             // data_class: INTERNAL_ONLY
    pub priority: CrawlPriority,           // data_class: INTERNAL_ONLY
    pub depth_budget: u16,                 // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrawlError {
    EmptyTargetId,
    EmptyTenant,
    EmptyUrl,
    DepthBudgetZero,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrawlTargetCreate {
    pub target_id: String,
    pub tenant_id: String,
    pub scheme: CrawlScheme,
    pub canonical_url: String,
    pub priority: CrawlPriority,
    pub depth_budget: u16,
    pub headers: BTreeMap<String, String>,
}

impl CrawlTarget {
    pub fn new(input: CrawlTargetCreate) -> Result<Self, CrawlError> {
        if input.target_id.trim().is_empty() {
            return Err(CrawlError::EmptyTargetId);
        }
        if input.tenant_id.trim().is_empty() {
            return Err(CrawlError::EmptyTenant);
        }
        if input.canonical_url.trim().is_empty() {
            return Err(CrawlError::EmptyUrl);
        }
        if input.depth_budget == 0 {
            return Err(CrawlError::DepthBudgetZero);
        }
        Ok(Self {
            target_id: input.target_id,
            tenant_id: input.tenant_id,
            scheme: input.scheme,
            canonical_url: input.canonical_url,
            priority: input.priority,
            depth_budget: input.depth_budget,
            headers: input.headers,
        })
    }

    pub fn is_internal_feed(&self) -> bool {
        matches!(self.scheme, CrawlScheme::InternalFeed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> CrawlTargetCreate {
        CrawlTargetCreate {
            target_id: "tgt_001".to_string(),
            tenant_id: "ten_kr".to_string(),
            scheme: CrawlScheme::Https,
            canonical_url: "https://kr.example.com/post/1".to_string(),
            priority: CrawlPriority::Normal,
            depth_budget: 3,
            headers: BTreeMap::new(),
        }
    }

    #[test]
    fn builds_valid_target() {
        let target = CrawlTarget::new(base_input()).expect("valid target");
        assert_eq!(target.target_id, "tgt_001");
        assert_eq!(target.depth_budget, 3);
        assert!(!target.is_internal_feed());
    }

    #[test]
    fn rejects_empty_tenant() {
        let input = CrawlTargetCreate {
            tenant_id: "".to_string(),
            ..base_input()
        };
        assert_eq!(CrawlTarget::new(input), Err(CrawlError::EmptyTenant));
    }

    #[test]
    fn rejects_zero_depth_budget() {
        let input = CrawlTargetCreate {
            depth_budget: 0,
            ..base_input()
        };
        assert_eq!(CrawlTarget::new(input), Err(CrawlError::DepthBudgetZero));
    }

    #[test]
    fn detects_internal_feed_scheme() {
        let input = CrawlTargetCreate {
            scheme: CrawlScheme::InternalFeed,
            ..base_input()
        };
        let target = CrawlTarget::new(input).expect("valid");
        assert!(target.is_internal_feed());
    }
}
