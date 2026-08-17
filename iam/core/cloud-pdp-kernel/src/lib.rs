//! # oya-cloud-iam-pdp-kernel
//!
//! Pure ports of the cloud-iam policy-decision-point SERVICE (ADR-0559,
//! G004 slice 1).
//!
//! ## Posture
//! ADR-0536 D-2 makes Cedar the single policy language: every service embeds
//! the formally-verified `cedar-policy` engine behind the shared
//! [`oya_shared_pdp_kernel::PolicyDecisionPoint`] port, and a central policy
//! store (cloud-iam, the three-plane IdP substrate) compiles, signs, and
//! pushes content-addressed policy bundles. This crate is the SERVICE-side
//! kernel for that central decision point: the seams the runnable
//! cloud-iam PDP composes — it deliberately does NOT re-model decision
//! evaluation (that port and its Cedar adapter live in
//! `libs/oya-shared-pdp-kernel` / `iam/adapters/pdp-cedar`;
//! reuse, never fork — ADR-0243 two-decision-algorithms prohibition).
//!
//! Ports here (cutover litmus per the ports-for-owned-stack doctrine —
//! "would this trait change at W5 cutover?" — answered per port):
//!
//! - [`PolicyBundleStore`]: where serving bundles come from. Slice 1 backs it
//!   with a file/ConfigMap adapter; the destination is the policy-bundle
//!   CRD + operator distribution fabric (ADR-0559 follow-up slices). The
//!   trait models the destination surface (load a complete, version-bearing
//!   [`PolicyBundle`]); distribution transports change behind it, the trait
//!   does not.
//! - [`DecisionAuditSink`]: one attributable record per decision (G004
//!   acceptance), the same emission seam shape as oya-identity's `AuditSink`
//!   — emission never fails the decision path, and the audit-chain bridge
//!   lands behind this same port.
//! - [`PdpConfig`]: twelve-factor configuration as a pure function of a
//!   key->value map (the oya-identity `from_lookup` precedent), so the
//!   Deployment manifest stays the single configuration surface and tests
//!   never mutate process environment.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

use oya_shared_pdp_kernel::{DecisionAuditRecord, PolicyBundle};

// =====================================================================
// Policy-bundle store port
// =====================================================================

/// Why a policy bundle could not be produced. Every variant is fail-closed:
/// the service REFUSES TO BOOT on any load error (the oya-identity boot
/// precedent — a serving process is a correctly-configured process), and a
/// failed reload never replaces a serving bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleStoreError {
    /// The backing store could not be reached/read at all.
    Unavailable { detail: String },
    /// The store yielded bytes that do not form a valid [`PolicyBundle`]
    /// document (parse failure, unknown fields, invariant violations).
    Malformed { detail: String },
    /// The signed-bundle envelope did not verify against the trusted
    /// public-key set: missing/empty signatures, no signature from a trusted
    /// key, or a signature that does not validate the stored inner bytes. The
    /// inner bundle is NEVER parsed past this gate (fail-closed: an
    /// unverifiable bundle can carry forged policy and must not load).
    SignatureRejected { detail: String },
}

impl fmt::Display for BundleStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { detail } => write!(f, "policy-bundle store unavailable: {detail}"),
            Self::Malformed { detail } => write!(f, "policy bundle malformed: {detail}"),
            Self::SignatureRejected { detail } => {
                write!(f, "policy bundle signature rejected: {detail}")
            }
        }
    }
}

impl std::error::Error for BundleStoreError {}

/// The policy-store backend port: produce the complete bundle the PDP should
/// serve. The bundle CARRIES its version token (content address upstream).
/// Signature verification before load is a STORE-SIDE obligation at this
/// boundary (ADR-0536 D-2): the file-store adapter verifies a signed-bundle
/// envelope against a trusted public-key set and refuses
/// ([`BundleStoreError::SignatureRejected`]) before the inner bundle is ever
/// parsed — the port shape is unchanged (still yields a verified
/// [`PolicyBundle`]), so the CRD/operator store swaps the adapter behind it at
/// W5 cutover without changing this contract.
pub trait PolicyBundleStore: Send + Sync {
    /// Load the current bundle. Any error is fail-closed: boot refusal at
    /// start-up, keep-serving-the-old-bundle on reload.
    fn load(&self) -> Result<PolicyBundle, BundleStoreError>;

    /// Human-legible description of the backing source (for boot logs and
    /// refusal diagnostics). Never used for decisions.
    fn describe(&self) -> String;
}

// =====================================================================
// Decision-audit emission port
// =====================================================================

/// Decision-audit emission port. One [`DecisionAuditRecord`] is emitted per
/// decision — allow or deny, cached or evaluated (G004: every decision is
/// attributable). Implementations append immutably; emission MUST NOT fail
/// the decision path — a sink error is swallowed after best effort, never
/// surfaced as an allow or a refusal.
pub trait DecisionAuditSink: Send + Sync {
    /// Append one sealed record.
    fn record(&self, record: &DecisionAuditRecord);
}

/// In-memory [`DecisionAuditSink`] backed by a mutex-guarded append-only
/// vector — the reference sink for tests and single-node bring-up (the
/// oya-identity `InMemoryAuditSink` shape).
#[derive(Debug, Default)]
pub struct InMemoryDecisionAuditSink {
    records: Mutex<Vec<DecisionAuditRecord>>, // data_class: AUDIT
}

impl InMemoryDecisionAuditSink {
    /// Build an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the recorded log (clone). Order is emission order.
    #[must_use]
    pub fn records(&self) -> Vec<DecisionAuditRecord> {
        match self.records.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Number of records emitted so far.
    #[must_use]
    pub fn len(&self) -> usize {
        match self.records.lock() {
            Ok(guard) => guard.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Whether no records have been emitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl DecisionAuditSink for InMemoryDecisionAuditSink {
    fn record(&self, record: &DecisionAuditRecord) {
        // A poisoned lock must not panic the decision path (ADR-0083 Tier 3):
        // recover the guard and append regardless.
        match self.records.lock() {
            Ok(mut guard) => guard.push(record.clone()),
            Err(poisoned) => poisoned.into_inner().push(record.clone()),
        }
    }
}

// =====================================================================
// Service configuration
// =====================================================================

/// `OYA_CLOUD_IAM_PDP_BUNDLE_PATH` — path to the policy-bundle JSON document
/// (required; ConfigMap-mounted in K8s for slice 1).
pub const ENV_BUNDLE_PATH: &str = "OYA_CLOUD_IAM_PDP_BUNDLE_PATH";
/// `OYA_CLOUD_IAM_PDP_REST_ADDR` — REST bind address (default `0.0.0.0:8080`).
pub const ENV_REST_ADDR: &str = "OYA_CLOUD_IAM_PDP_REST_ADDR";
/// `OYA_CLOUD_IAM_PDP_GRPC_ADDR` — gRPC bind address (default `0.0.0.0:8081`).
pub const ENV_GRPC_ADDR: &str = "OYA_CLOUD_IAM_PDP_GRPC_ADDR";
/// `OYA_CLOUD_IAM_PDP_DECISION_CACHE_CAPACITY` — bounded decision-cache size
/// (default `65536`; `0` disables caching). Cache keys carry the bundle
/// version, so revocation stays structural regardless of this knob.
pub const ENV_DECISION_CACHE_CAPACITY: &str = "OYA_CLOUD_IAM_PDP_DECISION_CACHE_CAPACITY";
/// `OYA_CLOUD_IAM_PDP_MTLS_CERT_DIR` — directory of the delivered mTLS cert mount
/// (the kubernetes.io/tls Secret projection: `tls.crt`/`tls.key`/`ca.crt`; default
/// `/etc/oya-cloud-iam-pdp/tls`). The production boot builds an `MtlsContext` from
/// this mount and refuses to boot (never plain TCP) if the material is
/// absent/empty/malformed (G002 slice-1b-iii; ADR-0561).
pub const ENV_MTLS_CERT_DIR: &str = "OYA_CLOUD_IAM_PDP_MTLS_CERT_DIR";
/// `OYA_CLOUD_IAM_PDP_BUNDLE_TRUST_DIR` — directory of trusted policy-bundle
/// SIGNING public keys (Ed25519, hex per file; ConfigMap projection). The
/// file-store adapter loads every key in this directory into the trusted set
/// and verifies the signed-bundle envelope against it BEFORE parsing the inner
/// bundle (G004 bundle-signing slice). This is REQUIRED with no default and
/// fail-closed: an absent/empty value is a BOOT REFUSAL ([`ConfigError::Missing`]).
/// A PDP that cannot prove which keys to trust must not serve a policy decision
/// (mirrors the mTLS trust-root fail-closed precedent). The directory itself
/// being absent/empty-of-keys is a load-time boot refusal in the adapter.
pub const ENV_BUNDLE_TRUST_DIR: &str = "OYA_CLOUD_IAM_PDP_BUNDLE_TRUST_DIR";

const DEFAULT_REST_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_GRPC_ADDR: &str = "0.0.0.0:8081";
const DEFAULT_DECISION_CACHE_CAPACITY: usize = 65_536;
const DEFAULT_MTLS_CERT_DIR: &str = "/etc/oya-cloud-iam-pdp/tls";

/// A configuration defect found while resolving [`PdpConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A required variable is unset/empty.
    Missing { variable: &'static str },
    /// A variable is set but not parseable as its type.
    Invalid {
        variable: &'static str,
        detail: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { variable } => {
                write!(f, "missing required environment variable {variable}")
            }
            Self::Invalid { variable, detail } => {
                write!(f, "environment variable {variable} invalid: {detail}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Service configuration resolved from the environment (twelve-factor;
/// K8s-native — the Deployment manifest is the single configuration surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdpConfig {
    /// Path to the policy-bundle JSON document (ConfigMap mount in slice 1).
    pub bundle_path: String,
    /// Directory of trusted policy-bundle signing public keys (Ed25519, hex).
    /// REQUIRED (no default) and fail-closed: the file-store adapter verifies
    /// the signed-bundle envelope against these keys before parsing the inner
    /// bundle (G004 bundle-signing slice). An absent/empty value is a boot
    /// refusal — a PDP that cannot prove which keys to trust must not serve.
    pub bundle_trust_dir: String,
    /// REST (axum) bind address.
    pub rest_addr: String,
    /// gRPC (tonic) bind address.
    pub grpc_addr: String,
    /// Bounded decision-cache capacity (`0` disables caching).
    pub decision_cache_capacity: usize,
    /// Directory of the delivered mTLS cert mount (kubernetes.io/tls Secret
    /// projection). The production boot builds an `MtlsContext` from this mount
    /// and fail-closes if it is absent/empty/malformed.
    pub mtls_cert_dir: String,
}

impl PdpConfig {
    /// Resolve the configuration from a key->value map (pure; the
    /// oya-identity `from_lookup` precedent). Empty values count as unset.
    ///
    /// # Errors
    /// [`ConfigError`] when a required variable is unset or unparseable.
    pub fn from_lookup(vars: &BTreeMap<String, String>) -> Result<Self, ConfigError> {
        let get = |key: &str| {
            vars.get(key)
                .map(String::as_str)
                .filter(|v| !v.trim().is_empty())
        };
        let bundle_path = get(ENV_BUNDLE_PATH)
            .ok_or(ConfigError::Missing {
                variable: ENV_BUNDLE_PATH,
            })?
            .to_owned();
        // Required, no default, fail-closed: an absent/empty trust anchor is a
        // boot refusal (the mTLS trust-root precedent — a process that cannot
        // prove which keys to trust must never serve a decision).
        let bundle_trust_dir = get(ENV_BUNDLE_TRUST_DIR)
            .ok_or(ConfigError::Missing {
                variable: ENV_BUNDLE_TRUST_DIR,
            })?
            .to_owned();
        let rest_addr = get(ENV_REST_ADDR).unwrap_or(DEFAULT_REST_ADDR).to_owned();
        let grpc_addr = get(ENV_GRPC_ADDR).unwrap_or(DEFAULT_GRPC_ADDR).to_owned();
        let decision_cache_capacity = match get(ENV_DECISION_CACHE_CAPACITY) {
            None => DEFAULT_DECISION_CACHE_CAPACITY,
            Some(raw) => raw.parse::<usize>().map_err(|e| ConfigError::Invalid {
                variable: ENV_DECISION_CACHE_CAPACITY,
                detail: e.to_string(),
            })?,
        };
        let mtls_cert_dir = get(ENV_MTLS_CERT_DIR)
            .unwrap_or(DEFAULT_MTLS_CERT_DIR)
            .to_owned();
        Ok(Self {
            bundle_path,
            bundle_trust_dir,
            rest_addr,
            grpc_addr,
            decision_cache_capacity,
            mtls_cert_dir,
        })
    }

    /// Resolve the configuration from process environment variables.
    ///
    /// # Errors
    /// [`ConfigError`] when a required variable is unset or unparseable.
    pub fn from_env() -> Result<Self, ConfigError> {
        let vars: BTreeMap<String, String> = std::env::vars().collect();
        Self::from_lookup(&vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use oya_shared_platform_contracts_kernel::pdp::{Decision, EntityRef, PolicyVersion};

    fn full_vars() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                ENV_BUNDLE_PATH.to_owned(),
                "/etc/pdp/bundle.json".to_owned(),
            ),
            (ENV_BUNDLE_TRUST_DIR.to_owned(), "/etc/pdp/trust".to_owned()),
            (ENV_REST_ADDR.to_owned(), "127.0.0.1:9090".to_owned()),
            (ENV_GRPC_ADDR.to_owned(), "127.0.0.1:9091".to_owned()),
            (ENV_DECISION_CACHE_CAPACITY.to_owned(), "128".to_owned()),
            (ENV_MTLS_CERT_DIR.to_owned(), "/var/run/pdp/svid".to_owned()),
        ])
    }

    /// The minimal required set: both fail-closed required variables present.
    fn required_vars() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                ENV_BUNDLE_PATH.to_owned(),
                "/etc/pdp/bundle.json".to_owned(),
            ),
            (ENV_BUNDLE_TRUST_DIR.to_owned(), "/etc/pdp/trust".to_owned()),
        ])
    }

    #[test]
    fn config_resolves_with_all_variables() {
        let config = PdpConfig::from_lookup(&full_vars()).unwrap();
        assert_eq!(
            config,
            PdpConfig {
                bundle_path: "/etc/pdp/bundle.json".to_owned(),
                bundle_trust_dir: "/etc/pdp/trust".to_owned(),
                rest_addr: "127.0.0.1:9090".to_owned(),
                grpc_addr: "127.0.0.1:9091".to_owned(),
                decision_cache_capacity: 128,
                mtls_cert_dir: "/var/run/pdp/svid".to_owned(),
            }
        );
    }

    #[test]
    fn config_defaults_addresses_and_cache_capacity() {
        let config = PdpConfig::from_lookup(&required_vars()).unwrap();
        assert_eq!(config.rest_addr, DEFAULT_REST_ADDR);
        assert_eq!(config.grpc_addr, DEFAULT_GRPC_ADDR);
        assert_eq!(
            config.decision_cache_capacity,
            DEFAULT_DECISION_CACHE_CAPACITY
        );
    }

    #[test]
    fn missing_bundle_trust_dir_is_refused() {
        // The trust anchor is fail-closed required: bundle_path present but no
        // trust dir => boot refusal (a PDP that cannot prove which keys to
        // trust must never serve a decision).
        let vars = BTreeMap::from([(
            ENV_BUNDLE_PATH.to_owned(),
            "/etc/pdp/bundle.json".to_owned(),
        )]);
        let err = PdpConfig::from_lookup(&vars).unwrap_err();
        assert_eq!(
            err,
            ConfigError::Missing {
                variable: ENV_BUNDLE_TRUST_DIR
            }
        );
        assert!(err.to_string().contains(ENV_BUNDLE_TRUST_DIR));
    }

    #[test]
    fn empty_bundle_trust_dir_counts_as_unset() {
        let mut vars = required_vars();
        vars.insert(ENV_BUNDLE_TRUST_DIR.to_owned(), "  ".to_owned());
        assert!(matches!(
            PdpConfig::from_lookup(&vars),
            Err(ConfigError::Missing {
                variable: ENV_BUNDLE_TRUST_DIR
            })
        ));
    }

    #[test]
    fn mtls_cert_dir_resolves_present_and_defaults() {
        // Present: the env value flows through verbatim.
        let mut vars = required_vars();
        vars.insert(
            ENV_MTLS_CERT_DIR.to_owned(),
            "/custom/svid/mount".to_owned(),
        );
        let config = PdpConfig::from_lookup(&vars).unwrap();
        assert_eq!(config.mtls_cert_dir, "/custom/svid/mount");

        // Absent: the twelve-factor default is the canonical mount path.
        let defaulted = PdpConfig::from_lookup(&required_vars()).unwrap();
        assert_eq!(defaulted.mtls_cert_dir, DEFAULT_MTLS_CERT_DIR);
        assert_eq!(defaulted.mtls_cert_dir, "/etc/oya-cloud-iam-pdp/tls");
    }

    #[test]
    fn missing_bundle_path_is_refused() {
        let err = PdpConfig::from_lookup(&BTreeMap::new()).unwrap_err();
        assert_eq!(
            err,
            ConfigError::Missing {
                variable: ENV_BUNDLE_PATH
            }
        );
        assert!(err.to_string().contains(ENV_BUNDLE_PATH));
    }

    #[test]
    fn empty_bundle_path_counts_as_unset() {
        let vars = BTreeMap::from([(ENV_BUNDLE_PATH.to_owned(), "  ".to_owned())]);
        assert!(matches!(
            PdpConfig::from_lookup(&vars),
            Err(ConfigError::Missing { .. })
        ));
    }

    #[test]
    fn non_numeric_cache_capacity_is_refused() {
        let mut vars = full_vars();
        vars.insert(ENV_DECISION_CACHE_CAPACITY.to_owned(), "lots".to_owned());
        let err = PdpConfig::from_lookup(&vars).unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { .. }));
        assert!(err.to_string().contains(ENV_DECISION_CACHE_CAPACITY));
    }

    #[test]
    fn bundle_store_errors_are_legible() {
        let unavailable = BundleStoreError::Unavailable {
            detail: "no such file".to_owned(),
        };
        assert_eq!(
            unavailable.to_string(),
            "policy-bundle store unavailable: no such file"
        );
        let malformed = BundleStoreError::Malformed {
            detail: "unknown field `policies`".to_owned(),
        };
        assert_eq!(
            malformed.to_string(),
            "policy bundle malformed: unknown field `policies`"
        );
        let rejected = BundleStoreError::SignatureRejected {
            detail: "no trusted key signed the bundle".to_owned(),
        };
        assert_eq!(
            rejected.to_string(),
            "policy bundle signature rejected: no trusted key signed the bundle"
        );
    }

    fn audit_record(decision_id: &str) -> DecisionAuditRecord {
        DecisionAuditRecord {
            decision_id: decision_id.to_owned(),
            request_id: "req-1".to_owned(),
            tenant_id: "acme".to_owned(),
            principal: EntityRef {
                entity_type: "OyaPlatform::Principal".to_owned(),
                entity_id: "alice".to_owned(),
            },
            action: "resource.read".to_owned(),
            resource: EntityRef {
                entity_type: "OyaPlatform::TenantResource".to_owned(),
                entity_id: "doc-1".to_owned(),
            },
            decision: Decision::Deny,
            policy_version: PolicyVersion::new("psv-1").unwrap(),
            determining_policy_ids: vec![],
            cache_hit: false,
        }
    }

    #[test]
    fn in_memory_sink_appends_in_emission_order() {
        let sink = InMemoryDecisionAuditSink::new();
        assert!(sink.is_empty());
        sink.record(&audit_record("dec-1"));
        sink.record(&audit_record("dec-2"));
        let records = sink.records();
        assert_eq!(sink.len(), 2);
        assert_eq!(records[0].decision_id, "dec-1");
        assert_eq!(records[1].decision_id, "dec-2");
    }
}
