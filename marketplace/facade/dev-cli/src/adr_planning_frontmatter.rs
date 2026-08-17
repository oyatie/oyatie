//! Retirement-marked CLI adapter imports for the controller-owned planning projection core.
//!
//! Parsing and projection authority lives in `ci-planning-projection`; the development CLI keeps
//! these crate-local names only for existing validation and compatibility surfaces.

pub(crate) use ci_generated_artifact_freshness::read_planning_impact_adrs;
pub(crate) use ci_planning_projection::{
    PlanningAdr, PlanningDeliverable, frontmatter_list, frontmatter_scalar, read_frontmatter,
};
