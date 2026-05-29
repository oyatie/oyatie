//! Audit-chain query kernel: repository, export, and engagement ports.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full surface in IP-012.
#![allow(dead_code)]

/// Repository port for tenant-scoped audit queries.
pub trait AuditQueryRepository {
    type Query;
    type Page;
    type Error;
    fn query(&self, q: &Self::Query) -> Result<Self::Page, Self::Error>;
}

/// Export bundle builder port.
pub trait ExportBuilder {
    type Query;
    type Bundle;
    type Error;
    fn build(&self, q: &Self::Query) -> Result<Self::Bundle, Self::Error>;
}

/// Auditor engagement resolver port (Cedar-gated).
pub trait AuditorEngagementResolver {
    type Engagement;
    type Error;
    fn resolve(&self, engagement_id: &str) -> Result<Self::Engagement, Self::Error>;
}
