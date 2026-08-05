//! # oya-shared-pdp-kernel
//!
//! Embedded-PDP port kernel for FD-001 (story G004, ADR-0536 D-2).
//!
//! ## Posture
//! ADR-0536 D-2: the PDP is embedded in-process in every service — an
//! authorization decision never takes a network hop — and a central policy
//! store compiles, signs, and pushes content-addressed policy bundles to
//! every PDP. Precedent: Cedar / Amazon Verified Permissions (embedded,
//! formally verified evaluator + central policy store); Google Zanzibar
//! (zookie freshness tokens; isolation is structural, not conventional).
//!
//! This crate is the vendor-neutral PORT: the [`PolicyDecisionPoint`] trait
//! over the locked PDP contract family in
//! `oya-shared-platform-contracts-kernel::pdp`, plus the value types every
//! engine adapter consumes — [`PolicyBundle`] (version-bearing policy bundle
//! as pushed by the policy store), [`EntitySlice`] (the PIP entity slice a
//! PEP assembles per request), [`DecisionCache`] keyed on
//! `(request-fingerprint, policy-version)` per the G004 acceptance shape,
//! and [`DecisionAuditRecord`] (audit record per decision — every decision,
//! allow or deny, cached or evaluated, is attributable).
//!
//! Ports-for-owned-stack review ("would this trait change at W5 cutover?"):
//! no — Cedar is the TERMINAL engine decision per ADR-0536 D-2 (formally
//! verified upstream crate), and this port models the destination decision
//! surface (PARC request in, attributable decision + audit record out),
//! not any transient engine detail.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;

use serde::{Deserialize, Serialize};

use oya_shared_platform_contracts_kernel::ContractViolation;
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, Obligation, PolicyVersion,
};

/// One entity in the per-request PIP slice: its typed uid, attribute map
/// (deterministic order), and parent edges (group membership, tenant
/// containment).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRecord {
    pub uid: EntityRef, // data_class: TENANT_SCOPED
    /// Attribute map exposed to ABAC conditions (deterministic order).
    pub attributes: BTreeMap<String, serde_json::Value>, // data_class: TENANT_SCOPED
    /// Parent entity edges (e.g. Principal -> Group, Group -> Tenant).
    pub parents: Vec<EntityRef>, // data_class: TENANT_SCOPED
}

/// The entity slice a PEP assembles for one authorization request. The PDP
/// evaluates against EXACTLY this slice — it never reaches out to a PIP at
/// decision time (embedded-PDP doctrine: no network hop on the request path).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySlice {
    pub entities: Vec<EntityRecord>, // data_class: TENANT_SCOPED
}

impl EntitySlice {
    /// Surface-all invariant check: every uid is well-formed and no uid
    /// appears twice (a duplicate would make attribute resolution ambiguous).
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        let mut seen: Vec<&EntityRef> = Vec::new();
        for record in &self.entities {
            if record.uid.entity_type.is_empty() || record.uid.entity_id.is_empty() {
                out.push(ContractViolation::MissingValue {
                    field: "entity_slice.entities.uid",
                });
            }
            if seen.contains(&&record.uid) {
                out.push(ContractViolation::BrokenReference {
                    field: "entity_slice.entities",
                    detail: format!(
                        "duplicate entity uid {}::{}",
                        record.uid.entity_type, record.uid.entity_id
                    ),
                });
            }
            seen.push(&record.uid);
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

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
    /// Named templates for PBAC instantiations.
    pub templates: Vec<TemplateSrc>, // data_class: INTERNAL_ONLY
    /// PBAC template instantiations compiled into this bundle.
    pub template_links: Vec<TemplateLink>, // data_class: TENANT_SCOPED
    /// Contract-action-slug -> engine-action-uid map, compiled by the policy
    /// store (contract actions are slug-form per the locked PDP contract;
    /// engine action ids are namespaced uids). Unknown slugs fail closed.
    pub action_map: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

/// Why the PDP refused to decide. Every variant is fail-closed: a PEP MUST
/// treat any error as deny.
#[derive(Debug, Clone, PartialEq)]
pub enum PdpError {
    /// The request violates the locked PDP contract.
    InvalidRequest(Vec<ContractViolation>),
    /// The caller pinned a zookie freshness floor the loaded bundle does not
    /// satisfy (equality-only comparison per the contract): the PDP refuses
    /// rather than answer against stale policy.
    StalePolicyVersion {
        required: PolicyVersion,
        loaded: PolicyVersion,
    },
    /// The bundle failed parse/strict-validation/link and was NOT loaded.
    BundleRejected { detail: String },
    /// The request's action slug has no engine mapping in the loaded bundle.
    UnknownAction { action: String },
    /// Engine-level evaluation failure (malformed entity slice, etc.).
    Evaluation { detail: String },
    /// A decision id could not be minted; the decision is not emitted
    /// because it would be unattributable in the audit chain.
    DecisionIdUnavailable { detail: String },
    /// The PDP reached a decision but could not durably append the signed
    /// audit-chain event. Callers must fail closed rather than use an
    /// unaudited authorization outcome.
    AuditChainEmission { detail: String },
}

impl fmt::Display for PdpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(violations) => {
                write!(f, "invalid authorization request: ")?;
                for (i, v) in violations.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{v}")?;
                }
                Ok(())
            }
            Self::StalePolicyVersion { required, loaded } => write!(
                f,
                "policy bundle too stale: caller pinned {} but loaded version is {}",
                required.as_str(),
                loaded.as_str()
            ),
            Self::BundleRejected { detail } => write!(f, "policy bundle rejected: {detail}"),
            Self::UnknownAction { action } => {
                write!(
                    f,
                    "action {action:?} has no engine mapping in the loaded bundle"
                )
            }
            Self::Evaluation { detail } => write!(f, "evaluation failed: {detail}"),
            Self::DecisionIdUnavailable { detail } => {
                write!(f, "decision id unavailable: {detail}")
            }
            Self::AuditChainEmission { detail } => {
                write!(f, "audit-chain emission failed: {detail}")
            }
        }
    }
}

impl std::error::Error for PdpError {}

/// Audit record per decision (G004 acceptance): every decision — allow or
/// deny, cached or freshly evaluated — produces one attributable record
/// keyed by `decision_id` (the audit-chain correlation key from the locked
/// contract).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionAuditRecord {
    pub decision_id: String,                 // data_class: INTERNAL_ONLY
    pub request_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: TENANT_SCOPED
    pub principal: EntityRef,                // data_class: TENANT_SCOPED
    pub action: String,                      // data_class: INTERNAL_ONLY
    pub resource: EntityRef,                 // data_class: TENANT_SCOPED
    pub decision: Decision,                  // data_class: INTERNAL_ONLY
    pub policy_version: PolicyVersion,       // data_class: INTERNAL_ONLY
    pub determining_policy_ids: Vec<String>, // data_class: INTERNAL_ONLY
    /// Whether the decision content was served from the decision cache.
    pub cache_hit: bool, // data_class: INTERNAL_ONLY
}

/// One authorization outcome: the contract response plus its audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdpOutcome {
    pub response: AuthorizationResponse,
    pub audit: DecisionAuditRecord,
    /// Whether the decision content was served from the decision cache.
    pub cache_hit: bool,
}

/// The embedded-PDP port. Implementations evaluate in-process against the
/// loaded [`PolicyBundle`] — never over the network — with deny-by-default
/// and forbid-overrides-permit semantics (the locked contract restates the
/// engine semantics; adapters must satisfy them).
pub trait PolicyDecisionPoint: Send + Sync {
    /// Decide one PARC request against the supplied entity slice. Every
    /// error is fail-closed: the PEP MUST treat it as deny.
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError>;

    /// The version token of the currently loaded bundle.
    fn loaded_policy_version(&self) -> PolicyVersion;
}

/// Canonical fingerprint of the decision-relevant request surface: tenant,
/// principal, action, resource, ABAC context, and the entity slice. The
/// correlation fields (`request_id`) and the freshness floor
/// (`min_policy_version`) are EXCLUDED — they never change the decision.
///
/// The fingerprint is the full canonical JSON string, not a digest: cache
/// correctness must not depend on hash-collision odds, and the bounded cache
/// caps memory. Entity records are sorted by uid so PEP assembly order
/// cannot split cache entries.
#[must_use]
pub fn request_fingerprint(request: &AuthorizationRequest, entities: &EntitySlice) -> String {
    let mut records: Vec<&EntityRecord> = entities.entities.iter().collect();
    records.sort_by(|a, b| {
        (&a.uid.entity_type, &a.uid.entity_id).cmp(&(&b.uid.entity_type, &b.uid.entity_id))
    });
    let entity_payload: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            let mut parents: Vec<(&String, &String)> = r
                .parents
                .iter()
                .map(|p| (&p.entity_type, &p.entity_id))
                .collect();
            parents.sort();
            serde_json::json!({
                "uid": [r.uid.entity_type, r.uid.entity_id],
                "attributes": r.attributes,
                "parents": parents,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "tenant_id": request.tenant_id,
        "principal": [request.principal.entity_type, request.principal.entity_id],
        "action": request.action,
        "resource": [request.resource.entity_type, request.resource.entity_id],
        "context": request.context,
        "entities": entity_payload,
    });
    payload.to_string()
}

/// The decision content a cache may replay. Correlation fields (decision id,
/// request id) are NEVER cached: every replayed decision is re-minted with a
/// fresh decision id so the audit chain stays one-record-per-decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedDecision {
    pub decision: Decision,
    pub determining_policy_ids: Vec<String>,
    pub obligations: Vec<Obligation>,
}

/// Cache key per the G004 acceptance shape: `(request-hash, policy-version)`.
/// Keying on the bundle version makes revocation structural: a bundle swap
/// changes the version, every prior entry becomes unreachable, and the
/// sub-60s revocation SLO reduces to bundle-propagation latency.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecisionCacheKey {
    pub request_fingerprint: String,
    pub policy_version: String,
}

/// Bounded in-process decision cache (insertion-order eviction). Embedded
/// PDPs are per-process; the cache never crosses a service boundary.
#[derive(Debug)]
pub struct DecisionCache {
    capacity: usize,
    map: HashMap<DecisionCacheKey, CachedDecision>,
    order: VecDeque<DecisionCacheKey>,
}

impl DecisionCache {
    /// A cache holding at most `capacity` decisions. A zero capacity
    /// disables caching (every lookup misses).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn get(&self, key: &DecisionCacheKey) -> Option<&CachedDecision> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: DecisionCacheKey, value: CachedDecision) {
        if self.capacity == 0 {
            return;
        }
        if self.map.insert(key.clone(), value).is_none() {
            self.order.push_back(key);
        }
        while self.map.len() > self.capacity {
            let Some(evicted) = self.order.pop_front() else {
                break;
            };
            self.map.remove(&evicted);
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
        EntityRef {
            entity_type: entity_type.to_owned(),
            entity_id: entity_id.to_owned(),
        }
    }

    fn request() -> AuthorizationRequest {
        AuthorizationRequest {
            request_id: "req-1".to_owned(),
            tenant_id: "acme".to_owned(),
            principal: entity_ref("OyaPlatform::Principal", "alice"),
            action: "resource.read".to_owned(),
            resource: entity_ref("OyaPlatform::TenantResource", "doc-1"),
            context: BTreeMap::new(),
            min_policy_version: None,
        }
    }

    fn slice() -> EntitySlice {
        EntitySlice {
            entities: vec![
                EntityRecord {
                    uid: entity_ref("OyaPlatform::Principal", "alice"),
                    attributes: BTreeMap::from([(
                        "tenant_id".to_owned(),
                        serde_json::json!("acme"),
                    )]),
                    parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
                },
                EntityRecord {
                    uid: entity_ref("OyaPlatform::Group", "tenant-admins"),
                    attributes: BTreeMap::new(),
                    parents: vec![],
                },
            ],
        }
    }

    #[test]
    fn fingerprint_ignores_correlation_and_freshness_fields() {
        let base = request_fingerprint(&request(), &slice());
        let mut r = request();
        r.request_id = "req-2".to_owned();
        r.min_policy_version = Some(PolicyVersion::new("psv-9").unwrap());
        assert_eq!(request_fingerprint(&r, &slice()), base);
    }

    #[test]
    fn fingerprint_is_entity_order_independent() {
        let base = request_fingerprint(&request(), &slice());
        let mut reversed = slice();
        reversed.entities.reverse();
        assert_eq!(request_fingerprint(&request(), &reversed), base);
    }

    #[test]
    fn fingerprint_tracks_decision_relevant_changes() {
        let base = request_fingerprint(&request(), &slice());
        let mut r = request();
        r.action = "resource.write".to_owned();
        assert_ne!(request_fingerprint(&r, &slice()), base);

        let mut attr_changed = slice();
        attr_changed.entities[0]
            .attributes
            .insert("step_up_class".to_owned(), serde_json::json!("a"));
        assert_ne!(request_fingerprint(&request(), &attr_changed), base);
    }

    #[test]
    fn entity_slice_rejects_duplicate_uids() {
        let mut s = slice();
        let dup = s.entities[0].clone();
        s.entities.push(dup);
        let violations = s.validate().unwrap_err();
        assert!(matches!(
            violations.as_slice(),
            [ContractViolation::BrokenReference { .. }]
        ));
    }

    #[test]
    fn cache_is_bounded_and_evicts_in_insertion_order() {
        let mut cache = DecisionCache::new(2);
        let value = CachedDecision {
            decision: Decision::Deny,
            determining_policy_ids: vec![],
            obligations: vec![],
        };
        for i in 0..3 {
            cache.insert(
                DecisionCacheKey {
                    request_fingerprint: format!("fp-{i}"),
                    policy_version: "psv-1".to_owned(),
                },
                value.clone(),
            );
        }
        assert_eq!(cache.len(), 2);
        assert!(
            cache
                .get(&DecisionCacheKey {
                    request_fingerprint: "fp-0".to_owned(),
                    policy_version: "psv-1".to_owned(),
                })
                .is_none(),
            "oldest entry must be evicted first"
        );
    }

    #[test]
    fn cache_key_separates_policy_versions() {
        let mut cache = DecisionCache::new(8);
        cache.insert(
            DecisionCacheKey {
                request_fingerprint: "fp".to_owned(),
                policy_version: "psv-1".to_owned(),
            },
            CachedDecision {
                decision: Decision::Allow,
                determining_policy_ids: vec!["rbac-tenant-admin-group".to_owned()],
                obligations: vec![],
            },
        );
        assert!(
            cache
                .get(&DecisionCacheKey {
                    request_fingerprint: "fp".to_owned(),
                    policy_version: "psv-2".to_owned(),
                })
                .is_none(),
            "a bundle swap must make prior entries unreachable"
        );
    }

    #[test]
    fn zero_capacity_disables_caching() {
        let mut cache = DecisionCache::new(0);
        cache.insert(
            DecisionCacheKey {
                request_fingerprint: "fp".to_owned(),
                policy_version: "psv-1".to_owned(),
            },
            CachedDecision {
                decision: Decision::Deny,
                determining_policy_ids: vec![],
                obligations: vec![],
            },
        );
        assert!(cache.is_empty());
    }

    #[test]
    fn pdp_error_messages_are_legible() {
        let e = PdpError::StalePolicyVersion {
            required: PolicyVersion::new("psv-2").unwrap(),
            loaded: PolicyVersion::new("psv-1").unwrap(),
        };
        assert_eq!(
            e.to_string(),
            "policy bundle too stale: caller pinned psv-2 but loaded version is psv-1"
        );
    }
}
