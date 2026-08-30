//! Revision builders for link and action type definitions.
//!
//! `new()` constructs at revision 1; an evolution candidate states its
//! target revision through `with_revision`. The evolution law lives in
//! [`OntologyEngine::evolve_link_type`](crate::OntologyEngine::evolve_link_type)
//! and
//! [`OntologyEngine::evolve_action_type`](crate::OntologyEngine::evolve_action_type):
//! strict monotonicity, semantic fields frozen, and (for actions) existing
//! parameters frozen with new parameters admitted only as optional.

use crate::definitions::{ActionTypeDefinition, LinkTypeDefinition};

impl LinkTypeDefinition {
    /// State the candidate revision for an evolution. Returns `self` for
    /// chaining.
    pub fn with_revision(mut self, revision: u32) -> Self {
        self.revision = revision;
        self
    }
}

impl ActionTypeDefinition {
    /// State the candidate revision for an evolution. Returns `self` for
    /// chaining.
    pub fn with_revision(mut self, revision: u32) -> Self {
        self.revision = revision;
        self
    }
}
