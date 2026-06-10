//! Design-system components mandated by story G007, implemented per
//! `specs/design-system/*.json` (Accepted machine-readable specs):
//!
//! - [`tenant_context_switcher`] — DS-TENANT_CONTEXT_SWITCHER
//! - [`policy_disclosure_banner`] — DS-POLICY_DISCLOSURE_BANNER
//! - [`audit_evidence_timeline`] — DS-AUDIT_EVIDENCE_TIMELINE
//! - [`ops_deployment_status_panel`] — DS-OPS_DEPLOYMENT_STATUS_PANEL
//!
//! Every component is WCAG 2.2 AA-shaped (native buttons for keyboard
//! reachability, landmark roles, `aria-live` status surfaces) and encodes its
//! spec `security_invariants` in the type system where possible (closed
//! enums, type-state gates, constructors that refuse invalid states) so a
//! violation is unrepresentable rather than merely linted.

pub mod audit_evidence_timeline;
pub mod ops_deployment_status_panel;
pub mod policy_disclosure_banner;
pub mod tenant_context_switcher;
