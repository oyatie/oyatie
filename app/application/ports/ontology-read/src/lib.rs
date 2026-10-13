//! Native read results for the Application Ontology explorer.
//! Credentials stay in the server transport; a tenant path is never a grant.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::future::Future;

mod labels;
mod links;
mod models;
mod query;
mod schema;
pub use labels::EntityTypeLabel;
pub use links::{LinkCardinality, LinkSchema};
pub use models::{
    AuditDisposition, AuditRow, EntityType, HistoryRow, ObjectDetail, ObjectPage, PinnedObject,
    Property, UpcastState,
};
pub use query::{DEFAULT_PAGE_LIMIT, FilterValue, MAX_PAGE_LIMIT, ObjectListQuery, PropertyFilter};
pub use schema::{
    EntitySchema, PropertyDefinition, PropertyTier, StructFieldDeclaration, ValueTypeDeclaration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyReadError {
    Unauthenticated,
    Forbidden,
    InvalidQuery,
    AbsentObject,
    RevisionConflict,
    FilterConflict,
    SourceUnavailable,
}

impl std::fmt::Display for OntologyReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unauthenticated => "a current user session is required",
            Self::Forbidden => "ontology access refused",
            Self::InvalidQuery => "invalid ontology read query",
            Self::AbsentObject => "the durable projection holds no object at this reference",
            Self::RevisionConflict => "the requested revision was never accepted",
            Self::FilterConflict => "stored property values have another kind",
            Self::SourceUnavailable => "the ontology read source is unavailable",
        })
    }
}

impl std::error::Error for OntologyReadError {}

/// The server adapter revalidates the user's credential and tenant on each read.
/// No browser principal, role list or sample fallback authorizes a live result.
pub trait OntologyReadSource: Send + Sync {
    /// Native human names; consumers join only matching schema IDs and revisions.
    fn type_labels(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<Vec<EntityTypeLabel>, OntologyReadError>> + Send;

    /// Current registered relationships; separate reads do not form an atomic snapshot.
    fn link_types(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<Vec<LinkSchema>, OntologyReadError>> + Send;

    /// Current accepted declarations, separate from the compatible type-name read.
    fn schemas(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<Vec<EntitySchema>, OntologyReadError>> + Send;

    fn types(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<Vec<EntityType>, OntologyReadError>> + Send;

    fn objects(
        &self,
        tenant: &str,
        query: &ObjectListQuery,
    ) -> impl Future<Output = Result<ObjectPage, OntologyReadError>> + Send;

    fn object(
        &self,
        tenant: &str,
        object_ref: &str,
        revision: u32,
    ) -> impl Future<Output = Result<ObjectDetail, OntologyReadError>> + Send;

    /// History is the native action log, with each row's own schema revision.
    fn history(
        &self,
        tenant: &str,
        object_ref: &str,
    ) -> impl Future<Output = Result<Vec<HistoryRow>, OntologyReadError>> + Send;

    fn audit(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<Vec<AuditRow>, OntologyReadError>> + Send;
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod schema_tests;

#[cfg(test)]
mod link_tests;
