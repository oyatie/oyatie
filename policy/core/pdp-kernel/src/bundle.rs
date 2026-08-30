//! Policy bundles as pushed by the policy-store control plane.

use crate::*;

/// A named policy template as compiled into a bundle by the policy store.
/// The id is explicit (templates are linked by id, never by source position).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSrc {
    pub template_id: String, // data_class: INTERNAL_ONLY
    pub src: String,         // data_class: INTERNAL_ONLY
}

/// A PBAC template instantiation (policy-as-data): the policy store links a
/// template per grant instead of authoring ad-hoc policies. Precedent:
/// Amazon Verified Permissions policy templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateLink {
    /// Id of the template being instantiated (e.g. `pbac-resource-read-grant`).
    pub template_id: String, // data_class: INTERNAL_ONLY
    /// Unique id of this instantiation; appears in determining-policy ids.
    pub link_id: String, // data_class: INTERNAL_ONLY
    pub principal: EntityRef, // data_class: TENANT_SCOPED
    pub resource: EntityRef,  // data_class: TENANT_SCOPED
}

/// A policy bundle as pushed by the policy-store control plane. The bundle
/// CARRIES its version token: content-addressing and signing are the policy
/// store's responsibility (it compiles, signs, and pushes content-addressed
/// bundles per ADR-0536 D-2); the embedded PDP treats the token as opaque
/// and echoes it on every decision (zookie semantics).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBundle {
    /// Opaque policy-store version token (content address upstream).
    pub version: PolicyVersion, // data_class: INTERNAL_ONLY
    /// Cedar-schema source for the entity/action model.
    pub schema_src: String, // data_class: INTERNAL_ONLY
    /// Static policy set source (structural forbid + RBAC/ABAC policies).
    pub policies_src: String, // data_class: INTERNAL_ONLY
    /// Per-tenant policy overlays: `tenant_id` -> tenant-scoped Cedar policy
    /// source. The compiled overlay applies ONLY to decisions for its owning
    /// tenant (the SVID-bound `tenant_id`); it is NEVER visible to another
    /// tenant's decisions (selection is keyed by the request's own tenant_id).
    /// Cross-tenant isolation for any overlay permit is enforced at RUNTIME by
    /// the global `structural-tenant-isolation` forbid over the schema-required
    /// `tenant_id` attribute (forbid-overrides-permit; arXiv 2403.04651) — that
    /// forbid, not any load-time check, is the formally-verified isolation
    /// boundary. Security-critical global gates (e.g. step-up on restricted
    /// reads) are likewise encoded as forbids so an overlay permit cannot
    /// bypass a deny-by-omission gate. Defaults empty (`#[serde(default)]`),
    /// so a flat bundle with no overlays still parses — backward compatible.
    #[serde(default)]
    pub tenant_policies: BTreeMap<String, String>, // data_class: TENANT_SCOPED
    /// Named templates for PBAC instantiations.
    pub templates: Vec<TemplateSrc>, // data_class: INTERNAL_ONLY
    /// PBAC template instantiations compiled into this bundle.
    pub template_links: Vec<TemplateLink>, // data_class: TENANT_SCOPED
    /// Contract-action-slug -> engine-action-uid map, compiled by the policy
    /// store (contract actions are slug-form per the locked PDP contract;
    /// engine action ids are namespaced uids). Unknown slugs fail closed.
    pub action_map: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}
