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
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use cedar_policy::{
    Authorizer, Context, Decision as CedarDecision, Entities, Entity, EntityId, EntityTypeName,
    EntityUid, PolicyId, PolicySet, Request, RestrictedExpression, Schema, SlotId, Template,
    ValidationMode, Validator,
};

use oya_audit_chain_domain::{
    AuditAppendInput, AuditChain, AuditChainError, AuditEvent, Ed25519SigningKey,
    Ed25519VerificationKeySet, Plane,
};
use oya_audit_chain_file_adapter::{FileAuditLedger, FileAuditLedgerError};
use oya_data_boundary_kernel::{DataClass, Purpose};

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

/// Audit-chain surface emitted for every durable PDP decision record.
pub const PDP_DECISION_AUDIT_SURFACE: &str = "authorization.pdp.decision";

/// Durable signed audit-chain appender for PDP decisions. It owns the narrow
/// adapter seam between the shared in-process Cedar PDP and the audit-chain
/// file ledger: every append is Ed25519-signed, hash-chained, and persisted
/// before the authorization outcome is returned to the caller.
pub struct PdpDecisionAuditChainLogger {
    ledger: FileAuditLedger,
    signer: Ed25519SigningKey,
    trusted_keys: Ed25519VerificationKeySet,
    chain: Mutex<AuditChain>,
}

impl fmt::Debug for PdpDecisionAuditChainLogger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PdpDecisionAuditChainLogger")
            .field("ledger", &self.ledger)
            .field("signer", &self.signer)
            .finish_non_exhaustive()
    }
}

impl PdpDecisionAuditChainLogger {
    /// Load the existing ledger and prepare to append signed PDP decision events.
    ///
    /// # Errors
    /// [`PdpAuditChainError::Load`] when the persisted ledger cannot be read or
    /// replayed as per-tenant audit-chain shards.
    pub fn new(
        ledger: FileAuditLedger,
        signer: Ed25519SigningKey,
        trusted_keys: Ed25519VerificationKeySet,
    ) -> Result<Self, PdpAuditChainError> {
        verify_signer_is_trusted(&signer, &trusted_keys)?;
        let chain = load_trusted_multi_tenant_shards(&ledger, &trusted_keys)?;
        Ok(Self {
            ledger,
            signer,
            trusted_keys,
            chain: Mutex::new(chain),
        })
    }

    /// Append, sign, and durably persist one PDP decision audit-chain event.
    ///
    /// # Errors
    /// Returns an error if the in-memory chain lock is poisoned, the audit-chain
    /// kernel rejects the append, or the file ledger cannot persist the new tip.
    pub fn record(&self, audit: &DecisionAuditRecord) -> Result<AuditEvent, PdpAuditChainError> {
        let mut current = self
            .chain
            .lock()
            .map_err(|_| PdpAuditChainError::LockPoisoned)?;
        let persisted = load_trusted_multi_tenant_shards(&self.ledger, &self.trusted_keys)?;
        if persisted != *current {
            return Err(PdpAuditChainError::Persist(
                FileAuditLedgerError::ChainDiverged,
            ));
        }
        let mut next = current.clone();
        let event = next
            .append_signed(decision_audit_input(audit), &self.signer)
            .map_err(PdpAuditChainError::Append)?
            .clone();
        self.ledger
            .append_chain(&next)
            .map_err(PdpAuditChainError::Persist)?;
        load_trusted_multi_tenant_shards(&self.ledger, &self.trusted_keys)?;
        *current = next;
        Ok(event)
    }
}

fn verify_signer_is_trusted(
    signer: &Ed25519SigningKey,
    trusted_keys: &Ed25519VerificationKeySet,
) -> Result<(), PdpAuditChainError> {
    let trusted_key = trusted_keys
        .trusted_key_for(signer.key_id())
        .map_err(PdpAuditChainError::UntrustedSigner)?;
    let signer_key = signer.verification_key();
    if &signer_key != trusted_key {
        return Err(PdpAuditChainError::UntrustedSigner(
            AuditChainError::Ed25519SignatureKeyMismatch {
                key_id: signer.key_id().to_owned(),
            },
        ));
    }
    Ok(())
}

fn load_trusted_multi_tenant_shards(
    ledger: &FileAuditLedger,
    trusted_keys: &Ed25519VerificationKeySet,
) -> Result<AuditChain, PdpAuditChainError> {
    let chain = ledger
        .load_multi_tenant_shards()
        .map_err(PdpAuditChainError::Load)?;
    chain
        .verify_signed_with_keys(trusted_keys)
        .map_err(PdpAuditChainError::TrustedSignatureReplay)?;
    Ok(chain)
}

/// Error surface for the audit-chain adapter seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PdpAuditChainError {
    Load(FileAuditLedgerError),
    TrustedSignatureReplay(AuditChainError),
    UntrustedSigner(AuditChainError),
    Append(AuditChainError),
    Persist(FileAuditLedgerError),
    LockPoisoned,
}

impl fmt::Display for PdpAuditChainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "load failed: {error:?}"),
            Self::TrustedSignatureReplay(error) => {
                write!(formatter, "trusted signature replay failed: {error:?}")
            }
            Self::UntrustedSigner(error) => {
                write!(formatter, "audit-chain signer is not trusted: {error:?}")
            }
            Self::Append(error) => write!(formatter, "append failed: {error:?}"),
            Self::Persist(error) => write!(formatter, "persist failed: {error:?}"),
            Self::LockPoisoned => write!(formatter, "audit-chain logger lock poisoned"),
        }
    }
}

impl std::error::Error for PdpAuditChainError {}

/// Cedar PDP wrapper that fail-closes unless each decision is durably signed
/// into the audit-chain ledger.
pub struct AuditChainCedarPdp {
    inner: CedarPdp,
    audit_logger: PdpDecisionAuditChainLogger,
}

impl AuditChainCedarPdp {
    /// Compile and strict-validate `bundle`, then serve from it with mandatory
    /// durable signed audit-chain emission.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when the bundle is invalid.
    pub fn load(
        bundle: &PolicyBundle,
        id_gen: Arc<dyn IdGenerator>,
        cache_capacity: usize,
        audit_logger: PdpDecisionAuditChainLogger,
    ) -> Result<Self, PdpError> {
        Ok(Self {
            inner: CedarPdp::load(bundle, id_gen, cache_capacity)?,
            audit_logger,
        })
    }

    /// Atomically replace the serving bundle while preserving the audit logger.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when the new bundle fails to compile;
    /// [`PdpError::Evaluation`] when the state lock is poisoned.
    pub fn swap_bundle(&self, bundle: &PolicyBundle) -> Result<(), PdpError> {
        self.inner.swap_bundle(bundle)
    }
}

impl PolicyDecisionPoint for AuditChainCedarPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        let outcome = self.inner.authorize(request, entities)?;
        self.audit_logger
            .record(&outcome.audit)
            .map_err(|error| PdpError::AuditChainEmission {
                detail: error.to_string(),
            })?;
        Ok(outcome)
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

fn decision_audit_input(audit: &DecisionAuditRecord) -> AuditAppendInput {
    AuditAppendInput {
        tenant_id: audit.tenant_id.clone(),
        surface: PDP_DECISION_AUDIT_SURFACE.to_owned(),
        plane: Plane::Control,
        purpose: Purpose::CoreService,
        data_classes: vec![DataClass::InternalOnly, DataClass::Audit],
        decision: decision_lineage_payload(audit),
    }
}

fn decision_lineage_payload(audit: &DecisionAuditRecord) -> String {
    serde_json::json!({
        "decision_id": &audit.decision_id,
        "request_id": &audit.request_id,
        "tenant_id": &audit.tenant_id,
        "principal": {
            "entity_type": &audit.principal.entity_type,
            "entity_id": &audit.principal.entity_id,
        },
        "resource": {
            "entity_type": &audit.resource.entity_type,
            "entity_id": &audit.resource.entity_id,
        },
        "action": &audit.action,
        "decision": decision_label(audit.decision),
        "policy_version": audit.policy_version.as_str(),
        "determining_policy_ids": &audit.determining_policy_ids,
        "cache_hit": audit.cache_hit,
    })
    .to_string()
}

fn decision_label(decision: Decision) -> &'static str {
    match decision {
        Decision::Allow => "allow",
        Decision::Deny => "deny",
    }
}

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
            restricted_expression(
                &format!("entity {} attr {key}", record.uid.entity_id),
                value,
            )?,
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
        let cedar_entities =
            Entities::from_entities(cedar_entities, Some(&state.schema)).map_err(|e| {
                PdpError::Evaluation {
                    detail: format!("entity slice rejected by schema: {e}"),
                }
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
