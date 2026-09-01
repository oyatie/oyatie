//! The policy enforcement point: this process decides by the checked-in
//! Cedar seed, evaluated by the platform's own engine.
//!
//! Two choices are load-bearing. First, the seed is compiled into the
//! binary with `include_str!` and strict-validated at load, so a policy set
//! that does not compile is a boot refusal rather than a surface that
//! silently permits nothing. Second, the process never re-states the
//! seed's conditions in Rust: a hand-written grant table would make the
//! served posture something other than the validated one, and the
//! checked-in policy would become decoration.
//!
//! The slug bridge is the bundle's own `action_map`. The platform's
//! authorization contract slug-checks request actions, so an engine action
//! id like `InvokeAction` can never be a request action; the map is where
//! contract slugs meet engine ids, and an unmapped slug fails closed.

use std::collections::BTreeMap;
use std::sync::Arc;

use policy_pdp_cedar::CedarPdp;
use policy_pdp_kernel::{PdpRuntimeConfig, PdpRuntimeGuard, PolicyBundle, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::PolicyVersion;
use shared_ulid_id_kernel::SeededIdGenerator;

/// The surfaces this process authorizes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    /// Submitting an Action — the write path.
    Invoke,
    /// Reading the ontology — the surface the shell's module card gates on.
    Use,
}

impl Surface {
    /// The contract slug this surface presents to the PDP.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Invoke => "foundry.ontology.invoke",
            Self::Use => "foundry.ontology.use",
        }
    }

    /// The autonomy tier the surface asserts. Reading carries none — that
    /// is a property of acting, not of looking — and the read action's
    /// context does not declare the attribute.
    pub const fn autonomy_tier(self) -> Option<i64> {
        match self {
            Self::Invoke => Some(1),
            Self::Use => None,
        }
    }
}

const SCHEMA_SRC: &str = include_str!("../../../cedar/foundry.cedarschema");
const POLICIES_SRC: &str = include_str!("../../../cedar/foundry-policies.cedar");

/// The operations console is the only surface the seed grants.
pub const OPS_CONSOLE: &str = "ops-console";

/// Why the enforcement point refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PepError {
    /// The bundle failed to compile or strict-validate; nothing is loaded.
    BundleRejected { detail: String },
    /// The PDP answered Deny, or refused to answer. Both are refusals: the
    /// port's own contract says every error is fail-closed.
    Denied,
}

impl std::fmt::Display for PepError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BundleRejected { detail } => {
                write!(formatter, "the policy bundle was rejected: {detail}")
            }
            Self::Denied => write!(formatter, "the policy decision point refused"),
        }
    }
}

/// Build the bundle this process serves from. The action map is the whole
/// slug bridge: `invoke` reaches the existing engine action, `use` is
/// identity because the seed declares it under its contract name.
fn bundle(version: &str) -> Result<PolicyBundle, PepError> {
    Ok(PolicyBundle {
        version: PolicyVersion::new(version).map_err(|error| PepError::BundleRejected {
            detail: format!("{error:?}"),
        })?,
        schema_src: SCHEMA_SRC.to_owned(),
        policies_src: POLICIES_SRC.to_owned(),
        tenant_policies: BTreeMap::new(),
        templates: Vec::new(),
        template_links: Vec::new(),
        action_map: BTreeMap::from([
            (
                Surface::Invoke.slug().to_owned(),
                r#"Action::"InvokeAction""#.to_owned(),
            ),
            (
                Surface::Use.slug().to_owned(),
                r#"Action::"foundry.ontology.use""#.to_owned(),
            ),
        ]),
    })
}

/// Compile and strict-validate the seed, then serve from it behind the
/// runtime guard (deadline, circuit, decision metrics, unwind capture).
pub fn load_guarded(version: &str) -> Result<PdpRuntimeGuard, PepError> {
    let pdp = CedarPdp::load(
        &bundle(version)?,
        Arc::new(SeededIdGenerator::default()),
        DECISION_CACHE_CAPACITY,
    )
    .map_err(|error| PepError::BundleRejected {
        detail: format!("{error:?}"),
    })?;
    Ok(PdpRuntimeGuard::new(
        Arc::new(pdp) as Arc<dyn PolicyDecisionPoint>,
        PdpRuntimeConfig::new(DECISION_DEADLINE, CIRCUIT_OPEN_AFTER_FAILURES),
    ))
}

const DECISION_CACHE_CAPACITY: usize = 256;
/// An in-process decision is microseconds of work; a deadline this wide
/// only ever fires on something pathological, which is exactly when a
/// fail-closed refusal is the right answer.
const DECISION_DEADLINE: std::time::Duration = std::time::Duration::from_millis(250);
const CIRCUIT_OPEN_AFTER_FAILURES: u32 = 5;
