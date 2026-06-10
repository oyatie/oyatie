//! # oya-shared-pdp-adapter-cedar
//!
//! Embedded cedar-policy PDP adapter for FD-001 (story G004, ADR-0536 D-2).
//!
//! ## Posture
//! Implements the [`PolicyDecisionPoint`] port from `oya-shared-pdp-kernel`
//! over the upstream, formally-verified `cedar-policy` engine (arXiv
//! 2403.04651): default-deny, forbid-overrides-permit, order-independent.
//! Evaluation is strictly in-process — an authorization decision never takes
//! a network hop (ADR-0536 D-2; precedent: Cedar / Amazon Verified
//! Permissions embedded evaluator).
//!
//! Cedar is the TERMINAL engine decision, not a transitional impl: ADR-0536
//! D-2 retires the hand-rolled `oya-policy-cedar-*` evaluator in favor of
//! this crate (two decision algorithms must never coexist, ADR-0243).
//!
//! ## Behavior
//! - Bundles are parsed, template-linked, and STRICT-validated before load;
//!   a rejected bundle never replaces a serving one (fail closed).
//! - Zookie freshness: a caller-pinned `min_policy_version` that does not
//!   match the loaded bundle version is a refusal, never a stale answer.
//! - Decision cache keyed `(request-fingerprint, policy-version)`: a bundle
//!   swap changes the version and structurally invalidates every prior
//!   entry, so revocation latency reduces to bundle propagation
//!   (sub-60s revocation SLO, G004).
//! - One audit record per decision — allow or deny, cached or evaluated —
//!   with a freshly minted decision id every time.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use cedar_policy::{
    Authorizer, Context, Decision as CedarDecision, Entities, Entity, EntityId, EntityTypeName,
    EntityUid, PolicyId, PolicySet, Request, RestrictedExpression, Schema, SlotId, Template,
    ValidationMode, Validator,
};

use oya_shared_pdp_kernel::{
    CachedDecision, DecisionAuditRecord, DecisionCache, DecisionCacheKey, EntityRecord,
    EntitySlice, PdpError, PdpOutcome, PolicyBundle, PolicyDecisionPoint, request_fingerprint,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, Obligation, PolicyVersion,
};
use oya_shared_ulid_id_kernel::IdGenerator;

/// Annotation key whose value names the obligation a permit carries.
/// PEPs MUST enforce obligations or fail closed (locked PDP contract).
const OBLIGATION_ANNOTATION: &str = "obligation";

struct LoadedBundle {
    version: PolicyVersion,
    schema: Schema,
    policy_set: PolicySet,
    action_map: BTreeMap<String, String>,
}

/// The embedded Cedar PDP. One instance per process; the policy-store
/// delivery fabric swaps bundles in place via [`CedarPdp::swap_bundle`].
pub struct CedarPdp {
    state: RwLock<LoadedBundle>,
    cache: Mutex<DecisionCache>,
    id_gen: Arc<dyn IdGenerator>,
    authorizer: Authorizer,
}

impl CedarPdp {
    /// Compile and strict-validate `bundle`, then serve from it.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when any part of the bundle fails to
    /// parse, link, or strict-validate — nothing is loaded in that case.
    pub fn load(
        bundle: &PolicyBundle,
        id_gen: Arc<dyn IdGenerator>,
        cache_capacity: usize,
    ) -> Result<Self, PdpError> {
        let state = compile(bundle)?;
        Ok(Self {
            state: RwLock::new(state),
            cache: Mutex::new(DecisionCache::new(cache_capacity)),
            id_gen,
            authorizer: Authorizer::new(),
        })
    }

    /// Atomically replace the serving bundle (the revocation path). The new
    /// bundle is fully compiled and strict-validated BEFORE the swap; on any
    /// error the current bundle keeps serving (fail closed, static
    /// stability). Prior cache entries become unreachable because the cache
    /// key carries the bundle version.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when the new bundle fails to compile;
    /// [`PdpError::Evaluation`] when the state lock is poisoned.
    pub fn swap_bundle(&self, bundle: &PolicyBundle) -> Result<(), PdpError> {
        let next = compile(bundle)?;
        let mut state = self.state.write().map_err(|_| PdpError::Evaluation {
            detail: "policy state lock poisoned".to_owned(),
        })?;
        *state = next;
        Ok(())
    }
}

fn compile(bundle: &PolicyBundle) -> Result<LoadedBundle, PdpError> {
    let (schema, _warnings) =
        Schema::from_cedarschema_str(&bundle.schema_src).map_err(|e| PdpError::BundleRejected {
            detail: format!("schema rejected: {e}"),
        })?;
    let parsed =
        PolicySet::from_str(&bundle.policies_src).map_err(|e| PdpError::BundleRejected {
            detail: format!("static policies rejected: {e}"),
        })?;
    // The parser assigns positional ids (policy0, policy1, ...); re-key each
    // policy by its @id annotation so determining-policy ids in decisions and
    // audit records are the STABLE authored ids, not source positions. A
    // duplicate @id is a bundle defect and fails closed.
    let mut policy_set = PolicySet::new();
    for policy in parsed.policies() {
        let policy = match policy.annotation("id") {
            Some(id) => policy.new_id(PolicyId::new(id)),
            None => policy.clone(),
        };
        let policy_id = policy.id().clone();
        policy_set
            .add(policy)
            .map_err(|e| PdpError::BundleRejected {
                detail: format!("static policy {policy_id} rejected: {e}"),
            })?;
    }
    for template in &bundle.templates {
        let parsed = Template::parse(
            Some(PolicyId::new(&template.template_id)),
            template.src.as_str(),
        )
        .map_err(|e| PdpError::BundleRejected {
            detail: format!("template {} rejected: {e}", template.template_id),
        })?;
        policy_set
            .add_template(parsed)
            .map_err(|e| PdpError::BundleRejected {
                detail: format!("template {} rejected: {e}", template.template_id),
            })?;
    }
    for link in &bundle.template_links {
        let mut values = HashMap::new();
        values.insert(SlotId::principal(), entity_uid(&link.principal)?);
        values.insert(SlotId::resource(), entity_uid(&link.resource)?);
        policy_set
            .link(
                PolicyId::new(&link.template_id),
                PolicyId::new(&link.link_id),
                values,
            )
            .map_err(|e| PdpError::BundleRejected {
                detail: format!("template link {} rejected: {e}", link.link_id),
            })?;
    }
    let validation = Validator::new(schema.clone()).validate(&policy_set, ValidationMode::Strict);
    if !validation.validation_passed() {
        let errors: Vec<String> = validation
            .validation_errors()
            .map(|e| e.to_string())
            .collect();
        return Err(PdpError::BundleRejected {
            detail: format!("strict validation failed: {}", errors.join("; ")),
        });
    }
    Ok(LoadedBundle {
        version: bundle.version.clone(),
        schema,
        policy_set,
        action_map: bundle.action_map.clone(),
    })
}

fn entity_uid(entity_ref: &EntityRef) -> Result<EntityUid, PdpError> {
    let type_name =
        EntityTypeName::from_str(&entity_ref.entity_type).map_err(|e| PdpError::Evaluation {
            detail: format!("entity type {:?} rejected: {e}", entity_ref.entity_type),
        })?;
    let id = match EntityId::from_str(&entity_ref.entity_id) {
        Ok(id) => id,
        // EntityId parsing is infallible (FromStr<Err = Infallible>).
        Err(infallible) => match infallible {},
    };
    Ok(EntityUid::from_type_name_and_id(type_name, id))
}

/// ABAC values cross the port as JSON; the schema seed models string, bool,
/// and long attributes, so exactly those are mapped. Anything else fails
/// closed rather than silently coercing.
fn restricted_expression(
    field: &str,
    value: &serde_json::Value,
) -> Result<RestrictedExpression, PdpError> {
    match value {
        serde_json::Value::String(s) => Ok(RestrictedExpression::new_string(s.clone())),
        serde_json::Value::Bool(b) => Ok(RestrictedExpression::new_bool(*b)),
        serde_json::Value::Number(n) => {
            n.as_i64()
                .map(RestrictedExpression::new_long)
                .ok_or_else(|| PdpError::Evaluation {
                    detail: format!("{field}: non-integer numbers are not mappable to Cedar"),
                })
        }
        _ => Err(PdpError::Evaluation {
            detail: format!("{field}: only string/bool/long values are mappable to Cedar"),
        }),
    }
}

fn cedar_entity(record: &EntityRecord) -> Result<Entity, PdpError> {
    let uid = entity_uid(&record.uid)?;
    let mut attrs = HashMap::new();
    for (key, value) in &record.attributes {
        attrs.insert(
            key.clone(),
            restricted_expression(&format!("entity {} attr {key}", record.uid.entity_id), value)?,
        );
    }
    let mut parents = HashSet::new();
    for parent in &record.parents {
        parents.insert(entity_uid(parent)?);
    }
    Entity::new(uid, attrs, parents).map_err(|e| PdpError::Evaluation {
        detail: format!("entity {} rejected: {e}", record.uid.entity_id),
    })
}

impl CedarPdp {
    fn evaluate(
        &self,
        state: &LoadedBundle,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<CachedDecision, PdpError> {
        let action_uid_src =
            state
                .action_map
                .get(&request.action)
                .ok_or_else(|| PdpError::UnknownAction {
                    action: request.action.clone(),
                })?;
        let action = EntityUid::from_str(action_uid_src).map_err(|e| PdpError::BundleRejected {
            detail: format!("action map entry {action_uid_src:?} rejected: {e}"),
        })?;
        let mut context_pairs = Vec::new();
        for (key, value) in &request.context {
            context_pairs.push((
                key.clone(),
                restricted_expression(&format!("context {key}"), value)?,
            ));
        }
        let context = Context::from_pairs(context_pairs).map_err(|e| PdpError::Evaluation {
            detail: format!("context rejected: {e}"),
        })?;
        let cedar_request = Request::new(
            entity_uid(&request.principal)?,
            action,
            entity_uid(&request.resource)?,
            context,
            Some(&state.schema),
        )
        .map_err(|e| PdpError::Evaluation {
            detail: format!("request rejected by schema: {e}"),
        })?;
        let mut cedar_entities = Vec::new();
        for record in &entities.entities {
            cedar_entities.push(cedar_entity(record)?);
        }
        let cedar_entities = Entities::from_entities(cedar_entities, Some(&state.schema))
            .map_err(|e| PdpError::Evaluation {
                detail: format!("entity slice rejected by schema: {e}"),
            })?;
        let response =
            self.authorizer
                .is_authorized(&cedar_request, &state.policy_set, &cedar_entities);
        let decision = match response.decision() {
            CedarDecision::Allow => Decision::Allow,
            CedarDecision::Deny => Decision::Deny,
        };
        let mut determining_policy_ids: Vec<String> = response
            .diagnostics()
            .reason()
            .map(ToString::to_string)
            .collect();
        determining_policy_ids.sort();
        let mut obligations = Vec::new();
        if decision.is_allow() {
            for policy_id in response.diagnostics().reason() {
                let annotation = state
                    .policy_set
                    .policy(policy_id)
                    .and_then(|p| p.annotation(OBLIGATION_ANNOTATION));
                if let Some(obligation_id) = annotation {
                    obligations.push(Obligation {
                        obligation_id: obligation_id.to_owned(),
                        parameters: BTreeMap::new(),
                    });
                }
            }
            obligations.sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
        }
        Ok(CachedDecision {
            decision,
            determining_policy_ids,
            obligations,
        })
    }

    fn outcome(
        &self,
        request: &AuthorizationRequest,
        version: &PolicyVersion,
        content: &CachedDecision,
        cache_hit: bool,
    ) -> Result<PdpOutcome, PdpError> {
        let decision_id = self
            .id_gen
            .new_ulid()
            .map_err(|e| PdpError::DecisionIdUnavailable {
                detail: e.to_string(),
            })?
            .as_str()
            .to_lowercase();
        let response = AuthorizationResponse {
            decision_id: decision_id.clone(),
            request_id: request.request_id.clone(),
            decision: content.decision,
            policy_version: version.clone(),
            determining_policy_ids: content.determining_policy_ids.clone(),
            obligations: content.obligations.clone(),
        };
        response
            .validate()
            .map_err(|violations| PdpError::Evaluation {
                detail: format!("decision violates the PDP contract: {violations:?}"),
            })?;
        let audit = DecisionAuditRecord {
            decision_id,
            request_id: request.request_id.clone(),
            tenant_id: request.tenant_id.clone(),
            principal: request.principal.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
            decision: content.decision,
            policy_version: version.clone(),
            determining_policy_ids: content.determining_policy_ids.clone(),
            cache_hit,
        };
        Ok(PdpOutcome {
            response,
            audit,
            cache_hit,
        })
    }
}

impl PolicyDecisionPoint for CedarPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        request.validate().map_err(PdpError::InvalidRequest)?;
        entities.validate().map_err(PdpError::InvalidRequest)?;
        let state = self.state.read().map_err(|_| PdpError::Evaluation {
            detail: "policy state lock poisoned".to_owned(),
        })?;
        if let Some(required) = &request.min_policy_version {
            // Zookie semantics: equality is the only comparison consumers
            // may rely on (the contract makes ordering store-owned).
            if required != &state.version {
                return Err(PdpError::StalePolicyVersion {
                    required: required.clone(),
                    loaded: state.version.clone(),
                });
            }
        }
        let key = DecisionCacheKey {
            request_fingerprint: request_fingerprint(request, entities),
            policy_version: state.version.as_str().to_owned(),
        };
        let cached = {
            let cache = self.cache.lock().map_err(|_| PdpError::Evaluation {
                detail: "decision cache lock poisoned".to_owned(),
            })?;
            cache.get(&key).cloned()
        };
        if let Some(content) = cached {
            return self.outcome(request, &state.version, &content, true);
        }
        let content = self.evaluate(&state, request, entities)?;
        {
            let mut cache = self.cache.lock().map_err(|_| PdpError::Evaluation {
                detail: "decision cache lock poisoned".to_owned(),
            })?;
            cache.insert(key, content.clone());
        }
        self.outcome(request, &state.version, &content, false)
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        match self.state.read() {
            Ok(state) => state.version.clone(),
            // A poisoned lock still names the version it held; PolicyVersion
            // is immutable after load so the clone below cannot observe a
            // torn write.
            Err(poisoned) => poisoned.into_inner().version.clone(),
        }
    }
}
