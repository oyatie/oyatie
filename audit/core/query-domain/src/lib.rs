//! Audit-chain query domain: pure validation + pagination.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full residency,
//! auditor-engagement, and pagination rules in IP-012.
#![allow(dead_code)]

pub use audit_query_api::AuditQuery;

/// Pure validation error for an audit query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryDomainError {
    EmptyTenantId,
    WindowTooLarge,
    InvalidCursor,
}
