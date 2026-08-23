//! Transient kube-rs adapter for the SVID-delivery operator (G002 slice-1b-iii-c;
//! ADR-0561, ADR-0506, ADR-0510).
//!
//! This crate executes the pure kernel's [`Action`] decision: it mints a real
//! X.509-SVID for the PDP's platform identity by driving the unchanged trustd
//! issuer ([`TrustdSvidIssuer::issue`] over the `EcdsaP256Signer` CA), and
//! projects the result as a `kubernetes.io/tls` Secret named EXACTLY
//! `cloud-iam-pdp-svid`, carrying `tls.crt` / `tls.key` / `ca.crt` in the
//! byte-for-byte shape the PDP's `MtlsContext::from_path` consumer parses.
//!
//! ## Clean-arch boundary
//!
//! - issuance is the existing `WorkloadIdentityIssuer` / `TrustdSvidIssuer` port
//!   over `os-trustd-domain` — NOT reimplemented here;
//! - the reconcile DECISION is the pure kernel (`reconcile`), NOT here;
//! - kube-rs + k8s-openapi are the ADR-0510 transient boundary, isolated to this
//!   crate (the owned cloud-k8s port is the cutover destination).
//!
//! ## Crypto root (scope caveat, ADR-0561 D4/D5)
//!
//! The operator roots SVID issuance on the trustd in-memory `EcdsaP256` CA via
//! the unchanged `SigningBackend` seam. The cloud-kms per-cell sealing-root swap
//! stays DEFERRED behind that seam — no kernel/port change is needed to land it
//! later. This adapter closes the CERT-DELIVERY dimension only.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::StreamExt;
use k8s_openapi::ByteString;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    Client,
    api::{Api, Patch, PatchParams},
    runtime::{Controller, controller::Action as ControllerAction, watcher},
};
use serde_json::json;
use tracing::{error, info, warn};

use base64::Engine as _;

use iam_identity_workload_svid_kernel::SpiffeId;
use iam_identity_workload_svid_operator_kernel::{
    Action, Clock, DesiredState, ObservedState, ObservedSvidSecret, reconcile,
};

use os_trustd_domain::JoinToken;
use os_trustd_domain::ca::CertificateAuthority;
use os_trustd_domain::der;
use os_trustd_domain::service::SecurityService;
use os_trustd_domain::signer::EcdsaP256Signer;
use os_trustd_domain::x509::KeyPair;

/// ADR-0510 boundary marker constant: the owned destination is cloud-k8s.
pub const ADR_0510_TRANSIENT_KUBE_ADAPTER: &str =
    "ADR-0510 transient kube-rs adapter; owned destination is cloud-k8s";

/// The `kubernetes.io/tls` Secret type string (the consumer mounts this shape).
pub const TLS_SECRET_TYPE: &str = "kubernetes.io/tls";
/// The Secret data key carrying the PDP server leaf chain (PEM).
pub const TLS_CRT_KEY: &str = "tls.crt";
/// The Secret data key carrying the PDP server private key (PKCS#8 PEM).
pub const TLS_KEY_KEY: &str = "tls.key";
/// The Secret data key carrying the trust-anchor CA cert(s) (PEM).
pub const CA_CRT_KEY: &str = "ca.crt";

// =====================================================================
// SVID secret material: the byte-for-byte consumer contract output
// =====================================================================

/// The three PEM members of the projected `kubernetes.io/tls` Secret, in the
/// EXACT shape `MtlsContext::from_path` consumes: `tls.crt` (server leaf chain
/// PEM), `tls.key` (PKCS#8 PEM), `ca.crt` (CA cert PEM). Producing this struct is
/// the closure proof — it round-trips through the PDP consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvidSecretMaterial {
    /// `tls.crt` — the PDP server leaf certificate chain, PEM-encoded.
    pub tls_crt_pem: String,
    /// `tls.key` — the PDP server private key (PKCS#8), PEM-encoded.
    pub tls_key_pem: String,
    /// `ca.crt` — the trust-anchor CA certificate(s), PEM-encoded.
    pub ca_crt_pem: String,
    /// The issued leaf's `notAfter` (epoch seconds) — fed back into the observed
    /// state so the next reconcile can decide rotation against it.
    pub leaf_not_after_epoch_seconds: u64,
}

/// Why minting the SVID Secret material failed. Every variant is fail-closed: a
/// failed mint never yields a partial Secret.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MintError {
    /// The desired SPIFFE id did not parse as a cell-rooted SVID URI.
    InvalidSpiffeId(String),
    /// CA bootstrap / signer generation failed.
    CaSetup(String),
    /// The trustd issuer refused issuance (policy, expired CA, real-DER mint).
    Issuance(String),
    /// Serializing the issued CA certificate to DER failed.
    CaEncode(String),
}

impl fmt::Display for MintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpiffeId(detail) => write!(f, "invalid desired SPIFFE id: {detail}"),
            Self::CaSetup(detail) => write!(f, "issuing CA setup failed: {detail}"),
            Self::Issuance(detail) => write!(f, "SVID issuance failed: {detail}"),
            Self::CaEncode(detail) => write!(f, "CA certificate DER encode failed: {detail}"),
        }
    }
}

impl std::error::Error for MintError {}

/// PEM-encode raw DER under `label` (64-char base64 lines), matching the
/// kubernetes.io/tls on-mount shape `MtlsContext::from_path` parses.
fn pem(label: &str, der_bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der_bytes);
    let mut body = String::new();
    let mut start = 0;
    while start < b64.len() {
        let end = (start + 64).min(b64.len());
        body.push_str(&b64[start..end]);
        body.push('\n');
        start = end;
    }
    format!("-----BEGIN {label}-----\n{body}-----END {label}-----\n")
}

/// The issuance backend the operator drives to mint a PDP server SVID. The
/// production impl roots on a trustd `EcdsaP256` CA (the unchanged `SigningBackend`
/// seam; the cloud-kms swap is the deferred ADR-0561 D4/D5 follow-up). A test or a
/// future owned-stack impl can substitute a different backend without touching the
/// kernel or the kube-rs runtime.
pub trait SvidIssuanceBackend {
    /// Mint the PDP server SVID for `desired` as of `now`, returning the three
    /// PEM members the consumer Secret carries plus the issued leaf's expiry.
    ///
    /// # Errors
    /// [`MintError`] on an invalid SPIFFE id, CA setup failure, issuance refusal,
    /// or CA encode failure. Fail-closed: never returns partial material.
    fn mint(&mut self, desired: &DesiredState, now: u64) -> Result<SvidSecretMaterial, MintError>;
}

/// The production issuance backend: a self-rooted trustd `EcdsaP256` CA whose
/// `SecurityService` issues the PDP server SVID via the unchanged
/// [`TrustdSvidIssuer`]-equivalent flow. The CA persists across mints so a rotated
/// leaf chains to the SAME trust anchor the already-delivered `ca.crt` carries.
///
/// [`TrustdSvidIssuer`]: iam_identity_workload_svid_trustd::TrustdSvidIssuer
pub struct TrustdEcdsaIssuanceBackend {
    service: SecurityService<EcdsaP256Signer>,
    ca_signer: EcdsaP256Signer,
    join_token: String,
}

impl TrustdEcdsaIssuanceBackend {
    /// Bootstrap a self-rooted trustd CA + service for SVID issuance, valid from
    /// `ca_not_before` for `ca_ttl_secs`. The join token gates issuance exactly as
    /// node issuance is gated.
    ///
    /// # Errors
    /// [`MintError::CaSetup`] if signer generation, CA bootstrap, or join-token
    /// construction fails.
    pub fn bootstrap(
        ca_common_name: &str,
        join_token: &str,
        ca_not_before: u64,
        ca_ttl_secs: u64,
    ) -> Result<Self, MintError> {
        let ca_signer =
            EcdsaP256Signer::generate().map_err(|e| MintError::CaSetup(e.to_string()))?;
        let ca_key = KeyPair::new(ca_signer.private_key_der(), ca_signer.public_key_spki_der());
        let token = JoinToken::new(join_token).map_err(|e| MintError::CaSetup(e.to_string()))?;
        let ca = CertificateAuthority::bootstrap(
            ca_common_name,
            ca_key,
            ca_signer.clone(),
            ca_not_before,
            ca_ttl_secs,
        )
        .map_err(|e| MintError::CaSetup(e.to_string()))?;
        Ok(Self {
            service: SecurityService::new(token, ca),
            ca_signer,
            join_token: join_token.to_owned(),
        })
    }
}

impl TrustdEcdsaIssuanceBackend {
    /// Issue a real X.509-SVID for `spiffe_uri` (named `workload_name`) from this
    /// operator's CA, returning the leaf DER, the subject's PKCS#8 private key, and
    /// the issued leaf's `notAfter`. Drives the unchanged trustd issuance flow
    /// (join-token gated CSR → `handle_certificate` → real ASN.1 leaf DER), exactly
    /// as `iam_identity_workload_svid_trustd::TrustdSvidIssuer::issue` does.
    ///
    /// # Errors
    /// [`MintError`] on issuance refusal or real-DER mint failure.
    fn issue_workload_svid(
        &mut self,
        workload_name: &str,
        spiffe_uri: &str,
        ttl_secs: u64,
        now: u64,
    ) -> Result<(Vec<u8>, Vec<u8>, u64), MintError> {
        let subject_signer =
            EcdsaP256Signer::generate().map_err(|e| MintError::CaSetup(e.to_string()))?;
        let requester_key = KeyPair::new(
            subject_signer.private_key_der(),
            subject_signer.public_key_spki_der(),
        );
        let csr = os_trustd_domain::ca::CertificateSigningRequest::for_workload(
            workload_name,
            spiffe_uri,
            &requester_key,
            ttl_secs,
        );
        let cert_request = os_trustd_domain::service::CertificateRequest {
            join_token: self.join_token.clone(),
            csr,
        };
        let response = self
            .service
            .handle_certificate(&cert_request, &requester_key, now)
            .map_err(|e| MintError::Issuance(e.to_string()))?;
        let leaf_cert = response.identity.certificate;
        let leaf_not_after = leaf_cert.validity.not_after;
        let leaf_der = der::encode_leaf_der(
            &leaf_cert,
            &subject_signer,
            self.service.ca_certificate(),
            &self.ca_signer,
        )
        .map_err(|e| MintError::Issuance(e.to_string()))?;
        Ok((leaf_der, subject_signer.private_key_der(), leaf_not_after))
    }

    /// Issue a CALLER X.509-SVID (leaf DER + PKCS#8 key) for `spiffe_uri` from this
    /// operator's CA — the leaf chains to the SAME trust anchor the operator's
    /// delivered `ca.crt` carries. This is the closure surface the PDP keystone
    /// drives: a caller minted here presents to a PDP booted from the operator's
    /// Secret and is bound to its SVID-derived tenant.
    ///
    /// # Errors
    /// [`MintError`] on an invalid SPIFFE id or issuance failure.
    pub fn issue_caller_svid(
        &mut self,
        spiffe_uri: &str,
        ttl_secs: u64,
        now: u64,
    ) -> Result<(Vec<u8>, Vec<u8>), MintError> {
        // Validate it is a cell-rooted SVID URI before minting.
        SpiffeId::parse(spiffe_uri).map_err(|e| MintError::InvalidSpiffeId(e.to_string()))?;
        let (leaf_der, key_der, _not_after) =
            self.issue_workload_svid("caller", spiffe_uri, ttl_secs, now)?;
        Ok((leaf_der, key_der))
    }

    /// The operator CA's certificate DER (`ca.crt` material) — the real SPKI a
    /// consumer anchors caller SVIDs against.
    ///
    /// # Errors
    /// [`MintError::CaEncode`] if the CA certificate cannot be DER-encoded.
    pub fn ca_der(&self) -> Result<Vec<u8>, MintError> {
        der::encode_ca_der(self.service.ca_certificate(), &self.ca_signer)
            .map_err(|e| MintError::CaEncode(e.to_string()))
    }
}

impl SvidIssuanceBackend for TrustdEcdsaIssuanceBackend {
    fn mint(&mut self, desired: &DesiredState, now: u64) -> Result<SvidSecretMaterial, MintError> {
        // The desired SPIFFE id must be a cell-rooted SVID URI.
        let spiffe_id = SpiffeId::parse(&desired.spiffe_id)
            .map_err(|e| MintError::InvalidSpiffeId(e.to_string()))?;

        // The PDP server SVID (its private half lands in tls.key); chains to this
        // operator's CA so the co-emitted ca.crt anchors it.
        let (leaf_der, key_der, leaf_not_after) =
            self.issue_workload_svid("cloud-iam-pdp", spiffe_id.as_uri(), desired.ttl_secs, now)?;

        // The CA certificate DER → ca.crt (real SPKI the consumer anchors on).
        let ca_der = self.ca_der()?;

        Ok(SvidSecretMaterial {
            tls_crt_pem: pem("CERTIFICATE", &leaf_der),
            tls_key_pem: pem("PRIVATE KEY", &key_der),
            ca_crt_pem: pem("CERTIFICATE", &ca_der),
            leaf_not_after_epoch_seconds: leaf_not_after,
        })
    }
}

/// Build the `kubernetes.io/tls` Secret JSON for `material` under `name`/`namespace`.
/// The three PEM members are base64-encoded into `.data` exactly as the Kubernetes
/// API stores them (the consumer's projected mount decodes them back to PEM files).
#[must_use]
pub fn secret_manifest(
    name: &str,
    namespace: &str,
    material: &SvidSecretMaterial,
) -> serde_json::Value {
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
    json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "type": TLS_SECRET_TYPE,
        "metadata": {
            "name": name,
            "namespace": namespace,
            "labels": {
                "app.kubernetes.io/managed-by": "iam-svid-operator",
                "app.kubernetes.io/part-of": "oyatie-microservices",
            },
        },
        "data": {
            TLS_CRT_KEY: b64(&material.tls_crt_pem),
            TLS_KEY_KEY: b64(&material.tls_key_pem),
            CA_CRT_KEY: b64(&material.ca_crt_pem),
        },
    })
}

// =====================================================================
// Observed-state projection from a delivered Secret
// =====================================================================

/// Why a delivered Secret could not be projected into kernel observed state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectError {
    /// The Secret was missing the `tls.crt` member or it was not valid base64.
    MissingOrUndecodableLeaf(String),
    /// The `tls.crt` PEM did not parse as a single X.509 leaf certificate.
    MalformedLeaf(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOrUndecodableLeaf(detail) => {
                write!(f, "delivered Secret tls.crt missing/undecodable: {detail}")
            }
            Self::MalformedLeaf(detail) => write!(f, "delivered Secret leaf malformed: {detail}"),
        }
    }
}

impl std::error::Error for ProjectError {}

/// Project a delivered Secret's `tls.crt` PEM into the kernel's
/// [`ObservedSvidSecret`] by parsing the leaf's `notAfter`. `tls_crt_pem` is the
/// decoded PEM string (the controller decodes the Secret `.data` member first).
///
/// # Errors
/// [`ProjectError`] when the leaf PEM is missing or malformed (treated as
/// fail-closed: the controller re-issues rather than trusting a broken Secret).
pub fn observed_secret_from_leaf_pem(
    tls_crt_pem: &str,
) -> Result<ObservedSvidSecret, ProjectError> {
    use x509_parser::pem::Pem;
    use x509_parser::prelude::FromDer;

    let mut der: Option<Vec<u8>> = None;
    for block in Pem::iter_from_buffer(tls_crt_pem.as_bytes()) {
        let pem = block.map_err(|e| ProjectError::MalformedLeaf(format!("PEM parse: {e}")))?;
        if pem.label == "CERTIFICATE" {
            der = Some(pem.contents);
            break;
        }
    }
    let der = der.ok_or_else(|| {
        ProjectError::MissingOrUndecodableLeaf("no CERTIFICATE PEM block".to_owned())
    })?;
    let (_rest, cert) = x509_parser::certificate::X509Certificate::from_der(&der)
        .map_err(|e| ProjectError::MalformedLeaf(format!("DER parse: {e}")))?;
    let not_after = u64::try_from(cert.validity().not_after.timestamp()).unwrap_or(0);
    Ok(ObservedSvidSecret {
        leaf_not_after_epoch_seconds: not_after,
    })
}

// =====================================================================
// Reconcile cycle: observe → decide (pure kernel) → actuate
// =====================================================================

/// The outcome of one reconcile cycle (for logging / requeue decisions).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcileReport {
    /// The action the pure kernel decided.
    pub action: Action,
    /// Whether the action mutated cluster state (Issue/Rotate) vs Noop.
    pub mutated: bool,
}

/// Why a reconcile cycle failed closed.
#[derive(Debug)]
pub enum ReconcileError {
    /// Minting the SVID material failed.
    Mint(MintError),
    /// Projecting the kube-rs object into observed state failed.
    Project(ProjectError),
    /// A kube-rs API call failed.
    Kube(String),
    /// The issuance-backend lock was poisoned.
    BackendLockPoisoned,
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mint(e) => write!(f, "{e}"),
            Self::Project(e) => write!(f, "{e}"),
            Self::Kube(m) => write!(f, "kube api error: {m}"),
            Self::BackendLockPoisoned => write!(f, "issuance backend lock poisoned"),
        }
    }
}

impl std::error::Error for ReconcileError {}

/// Run one pure-then-actuated reconcile cycle against an in-memory observed state
/// and an issuance backend, returning the projected Secret material when the
/// kernel chose to Issue/Rotate. This is the in-process core the kube-rs runtime
/// and the facade tests both drive (no live K8s required).
///
/// # Errors
/// [`ReconcileError::Mint`] when issuance fails (fail-closed: no Secret is
/// produced on a mint failure).
pub fn run_reconcile_once<B, C>(
    observed: &ObservedState,
    desired: &DesiredState,
    backend: &mut B,
    clock: &C,
) -> Result<(ReconcileReport, Option<SvidSecretMaterial>), ReconcileError>
where
    B: SvidIssuanceBackend,
    C: Clock,
{
    let action = reconcile(observed, desired, clock);
    match &action {
        Action::Noop => Ok((
            ReconcileReport {
                action,
                mutated: false,
            },
            None,
        )),
        Action::Issue {
            requested_at_epoch_seconds,
            ..
        }
        | Action::Rotate {
            requested_at_epoch_seconds,
            ..
        } => {
            let material = backend
                .mint(desired, *requested_at_epoch_seconds)
                .map_err(ReconcileError::Mint)?;
            Ok((
                ReconcileReport {
                    action,
                    mutated: true,
                },
                Some(material),
            ))
        }
    }
}

// =====================================================================
// kube-rs runtime (ADR-0510 transient boundary)
// =====================================================================

/// Exponential backoff for fail-closed requeue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExponentialBackoff {
    /// The base requeue interval in seconds.
    pub base_seconds: u64,
    /// The cap on the backoff interval in seconds.
    pub max_seconds: u64,
}

impl ExponentialBackoff {
    /// The backoff delay for `attempt` (saturating, capped at `max_seconds`).
    #[must_use]
    pub fn delay_seconds(&self, attempt: u32) -> u64 {
        let multiplier = 1_u64.checked_shl(attempt).unwrap_or(u64::MAX);
        self.base_seconds
            .saturating_mul(multiplier)
            .min(self.max_seconds)
    }
}

struct SvidReconcileContext<B, C> {
    secret_api: Api<Secret>,
    desired: DesiredState,
    backend: Mutex<B>,
    clock: C,
    backoff: ExponentialBackoff,
    backoff_attempt: Mutex<u32>,
}

impl<B, C> SvidReconcileContext<B, C>
where
    B: SvidIssuanceBackend + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    async fn observe(&self) -> Result<ObservedState, ReconcileError> {
        match self.secret_api.get_opt(&self.desired.secret_name).await {
            Ok(None) => Ok(ObservedState::absent()),
            Ok(Some(secret)) => {
                let pem = secret
                    .data
                    .as_ref()
                    .and_then(|d| d.get(TLS_CRT_KEY))
                    .map(|ByteString(bytes)| String::from_utf8_lossy(bytes).into_owned());
                match pem {
                    Some(pem) => {
                        let observed =
                            observed_secret_from_leaf_pem(&pem).map_err(ReconcileError::Project)?;
                        Ok(ObservedState {
                            secret: Some(observed),
                        })
                    }
                    // A Secret with no tls.crt is treated as absent → re-issue.
                    None => Ok(ObservedState::absent()),
                }
            }
            Err(e) => Err(ReconcileError::Kube(e.to_string())),
        }
    }

    async fn apply(&self, material: &SvidSecretMaterial) -> Result<(), ReconcileError> {
        let manifest = secret_manifest(
            &self.desired.secret_name,
            &self.desired.secret_namespace,
            material,
        );
        let params = PatchParams::apply("iam-svid-operator").force();
        self.secret_api
            .patch(&self.desired.secret_name, &params, &Patch::Apply(&manifest))
            .await
            .map_err(|e| ReconcileError::Kube(e.to_string()))?;
        Ok(())
    }

    fn next_backoff_delay_seconds(&self) -> u64 {
        match self.backoff_attempt.lock() {
            Ok(mut attempt) => {
                let delay = self.backoff.delay_seconds(*attempt);
                *attempt = attempt.saturating_add(1);
                delay
            }
            Err(_) => self.backoff.max_seconds,
        }
    }

    fn reset_backoff(&self) {
        if let Ok(mut attempt) = self.backoff_attempt.lock() {
            *attempt = 0;
        }
    }
}

/// The kube-rs operator runtime: watches the target Secret and converges it onto
/// the desired SVID-delivery spec each reconcile cycle.
pub struct KubeSvidOperatorRuntime<B, C> {
    context: Arc<SvidReconcileContext<B, C>>,
}

impl<B, C> KubeSvidOperatorRuntime<B, C>
where
    B: SvidIssuanceBackend + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    /// Wire the runtime to a kube client, the desired spec, the issuance backend,
    /// the clock, and the requeue backoff.
    #[must_use]
    pub fn new(
        client: Client,
        desired: DesiredState,
        backend: B,
        clock: C,
        backoff: ExponentialBackoff,
    ) -> Self {
        let secret_api: Api<Secret> = Api::namespaced(client, &desired.secret_namespace);
        Self {
            context: Arc::new(SvidReconcileContext {
                secret_api,
                desired,
                backend: Mutex::new(backend),
                clock,
                backoff,
                backoff_attempt: Mutex::new(0),
            }),
        }
    }

    /// Run the controller until shutdown signal. Reconcile is fail-closed: a mint
    /// or API failure requeues with exponential backoff and never produces a
    /// partial Secret.
    pub async fn run(self) {
        let controller = Controller::new(
            self.context.secret_api.clone(),
            watcher::Config::default(),
        )
        .shutdown_on_signal()
        .run(
            reconcile_secret::<B, C>,
            error_policy::<B, C>,
            self.context.clone(),
        )
        .for_each(|result| async move {
            match result {
                Ok((object_ref, action)) => {
                    info!(object = ?object_ref, requeue = ?action, "iam svid-operator reconcile complete");
                }
                Err(error) => {
                    error!(error = %error, "iam svid-operator reconcile stream error");
                }
            }
        });

        info!(
            namespace = %self.context.desired.secret_namespace,
            secret = %self.context.desired.secret_name,
            "starting iam svid-delivery kube-rs operator"
        );
        controller.await;
    }
}

async fn reconcile_secret<B, C>(
    _object: Arc<Secret>,
    context: Arc<SvidReconcileContext<B, C>>,
) -> Result<ControllerAction, ReconcileError>
where
    B: SvidIssuanceBackend + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    let started = Instant::now();
    let observed = context.observe().await?;
    let (report, material) = {
        let mut backend = context
            .backend
            .lock()
            .map_err(|_| ReconcileError::BackendLockPoisoned)?;
        run_reconcile_once(&observed, &context.desired, &mut *backend, &context.clock)?
    };
    if let Some(material) = material {
        context.apply(&material).await?;
    }
    context.reset_backoff();
    emit_reconcile_event(&report, started.elapsed());
    Ok(ControllerAction::requeue(Duration::from_secs(
        context.backoff.base_seconds,
    )))
}

fn error_policy<B, C>(
    _object: Arc<Secret>,
    error: &ReconcileError,
    context: Arc<SvidReconcileContext<B, C>>,
) -> ControllerAction
where
    B: SvidIssuanceBackend + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    warn!(error = %error, "iam svid-operator reconcile failed closed");
    ControllerAction::requeue(Duration::from_secs(context.next_backoff_delay_seconds()))
}

fn emit_reconcile_event(report: &ReconcileReport, elapsed: Duration) {
    let action = match &report.action {
        Action::Issue { .. } => "issue",
        Action::Rotate { .. } => "rotate",
        Action::Noop => "noop",
    };
    info!(
        event_name = "iam_svid_operator_reconcile",
        action = action,
        mutated = report.mutated,
        metric_name = "iam_svid_operator_reconcile_convergence_seconds",
        convergence_seconds = elapsed.as_secs_f64(),
        "iam svid-operator reconcile cycle"
    );
}
