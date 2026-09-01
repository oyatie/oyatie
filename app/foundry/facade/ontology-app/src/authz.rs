//! From a PDP verdict to the kernel's `ActionPolicyDecision` — the one
//! conversion the write path depends on.
//!
//! The decision this produces is minted ONLY from a real Allow: its
//! `decision_id` is the PDP's own, so the log entry the writer appends is
//! attributable to the authorization that permitted it. Nothing here
//! constructs a decision on any other path, which is what keeps
//! "authorized" from becoming a thing this process can assert about
//! itself.

use std::collections::BTreeMap;

use data_ontology_kernel::{ActionPolicyDecision, AutonomyTier};
use policy_pdp_kernel::{EntityRecord, EntitySlice, PdpRuntimeGuard, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};

use crate::pdp::{OPS_CONSOLE, PepError, Surface};

/// Who is calling, as established by the credential — never by a header.
///
/// `roles` is part of that establishment, not a convenience: if this
/// process synthesized role membership for whoever asked, the seed's
/// `principal is Principal in Role::"foundry-operator"` clause would be
/// vacuous and the permit would cover the world. A caller presenting no
/// role is denied by absence of any permit that reaches it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Caller {
    pub tenant_id: String,    // data_class: TENANT_SCOPED
    pub principal_id: String, // data_class: TENANT_SCOPED
    pub roles: Vec<String>,   // data_class: TENANT_SCOPED
}

/// The enforcement point: one guarded PDP and the entity shape the seed
/// expects.
pub struct PolicyEnforcementPoint {
    guard: PdpRuntimeGuard,
}

impl std::fmt::Debug for PolicyEnforcementPoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PolicyEnforcementPoint")
            .field("policy_version", &self.loaded_policy_version())
            .finish()
    }
}

impl PolicyEnforcementPoint {
    /// Compile, strict-validate and serve the checked-in seed.
    pub fn load(version: &str) -> Result<Self, PepError> {
        Ok(Self {
            guard: crate::pdp::load_guarded(version)?,
        })
    }

    /// The bundle version currently serving — the operator's handle for
    /// telling which posture produced a refusal.
    pub fn loaded_policy_version(&self) -> PolicyVersion {
        self.guard.loaded_policy_version()
    }

    /// Decide one request, and on Allow mint the kernel decision the write
    /// path requires. Every error is a refusal: the port's contract states
    /// that a PEP must treat any PDP error as deny.
    pub fn decide(
        &self,
        caller: &Caller,
        surface: Surface,
        object_ref: &str,
    ) -> Result<ActionPolicyDecision, PepError> {
        let outcome = self
            .guard
            .authorize(
                &request(caller, surface, object_ref),
                &entities(caller, object_ref),
            )
            .map_err(|_| PepError::Denied)?;
        if outcome.response.decision != Decision::Allow {
            return Err(PepError::Denied);
        }
        Ok(ActionPolicyDecision {
            decision_id: outcome.response.decision_id,
            tenant_id: caller.tenant_id.clone(),
            principal_id: caller.principal_id.clone(),
            allowed_surfaces: vec![OPS_CONSOLE.to_owned()],
            autonomy_tier: AutonomyTier::T1Assist,
        })
    }
}

fn principal_ref(caller: &Caller) -> EntityRef {
    EntityRef {
        entity_type: "Principal".to_owned(),
        entity_id: caller.principal_id.clone(),
    }
}

fn object_ref_of(object_ref: &str) -> EntityRef {
    EntityRef {
        entity_type: "OntologyObject".to_owned(),
        entity_id: object_ref.to_owned(),
    }
}

fn request(caller: &Caller, surface: Surface, object_ref: &str) -> AuthorizationRequest {
    let mut context = BTreeMap::from([(
        "surface".to_owned(),
        serde_json::Value::String(OPS_CONSOLE.to_owned()),
    )]);
    // A JSON integer here must reach Cedar as a `Long`, or the seed's
    // `context.autonomy_tier <= 1` can never hold and the permit never
    // fires. The adapter maps integers to `Long` and refuses non-integer
    // numbers rather than coercing them.
    if let Some(tier) = surface.autonomy_tier() {
        context.insert(
            "autonomy_tier".to_owned(),
            serde_json::Value::Number(tier.into()),
        );
    }
    AuthorizationRequest {
        request_id: format!("{}:{object_ref}", surface.slug()),
        tenant_id: caller.tenant_id.clone(),
        principal: principal_ref(caller),
        action: surface.slug().to_owned(),
        resource: object_ref_of(object_ref),
        context,
        min_policy_version: None,
    }
}

/// The entity slice the seed's conditions read. The principal's tenant is
/// the CALLER's tenant — established by the credential — while the
/// object's tenant is the one it is addressed under, so a cross-tenant
/// attempt presents two different tenants and the structural forbid fires.
fn entities(caller: &Caller, object_ref: &str) -> EntitySlice {
    let role_refs: Vec<EntityRef> = caller
        .roles
        .iter()
        .map(|role| EntityRef {
            entity_type: "Role".to_owned(),
            entity_id: role.clone(),
        })
        .collect();
    let mut records = vec![
        EntityRecord {
            uid: principal_ref(caller),
            attributes: BTreeMap::from([(
                "tenant".to_owned(),
                serde_json::Value::String(caller.tenant_id.clone()),
            )]),
            parents: role_refs.clone(),
        },
        EntityRecord {
            uid: object_ref_of(object_ref),
            attributes: BTreeMap::from([(
                "tenant".to_owned(),
                serde_json::Value::String(TENANT_OF_RECORD.to_owned()),
            )]),
            parents: Vec::new(),
        },
    ];
    records.extend(role_refs.into_iter().map(|uid| EntityRecord {
        uid,
        attributes: BTreeMap::new(),
        parents: Vec::new(),
    }));
    EntitySlice { entities: records }
}

/// The tenant every seeded object belongs to in this lane. The read and
/// write surfaces bind the object's real tenant once the request handlers
/// land; until then a single owning tenant keeps the cross-tenant refusal
/// honest rather than vacuous.
const TENANT_OF_RECORD: &str = "ten_acme";
