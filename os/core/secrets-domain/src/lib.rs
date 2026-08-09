//! # talos-secrets
//!
//! Manages Talos cluster secrets and PKI: the root CA bundles (Kubernetes,
//! etcd, OS/API, aggregator), service-account keys, bootstrap/join tokens, and
//! the controllers that derive leaf certificates and rotate them. Mirrors
//! Talos's `internal/app/machined/pkg/controllers/secrets/*` and the
//! `SecretsBundle` model in `pkg/machinery/config/generate/secrets`.
//!
//! ## Layout
//!
//! * [`bundle`] — the [`SecretsBundle`]: four CAs, service-account key, cluster
//!   identity, and tokens, plus the PKI primitives ([`CertificateAuthority`],
//!   [`Certificate`], [`KeyPair`], [`Validity`]).
//! * [`certsans`] — the SAN accumulator used to build server-cert SAN sets.
//! * [`rotation`] — the renewal policy and per-cert lifecycle state machine.
//! * [`kubernetes`], [`etcd`], [`api`], [`trustd`] — the per-domain certificate
//!   controllers, each deriving and rotating its leaves from the bundle.
//!
//! This crate also surfaces the **secret status resources** ([`SecretStatus`])
//! that Talos publishes as it generates certs, and a [`BundlePersistence`]
//! boundary for loading/storing the bundle on disk.
//!
//! # This crate is a MODEL, and production cannot link it
//!
//! Nothing here performs cryptography. It is modeled end to end, and the model
//! is not weak crypto, it is *no* crypto:
//!
//! * `KeyPair::from_seed` sets the private key to the seed **verbatim**, and
//!   the public key to a reversible byte transform of it, so the public half
//!   recovers the private half.
//! * `InMemorySigner` "signs" with an 8-byte keyed FNV hash, and it is the only
//!   signer `CertificateAuthority` has — `bootstrap` builds one from whatever
//!   key pair it is handed, so even real key material yields a forgeable MAC
//!   where a signature should be, and `verify` accepts it.
//! * `SecretsBundle::generate` derives all four CAs, the service-account key,
//!   the cluster secret and both tokens from one seed string, and
//!   `FsBundleStore::save` writes that seed to disk in **plaintext**, so every
//!   key in the cluster is regenerable from one line of one file.
//!
//! Unlike `os-trustd-domain`, there is no real backend beside the model to
//! prefer: gating the modeled constructors and leaving the rest would gate
//! everything reachable from them, which is this crate. So the gate sits at the
//! crate root. Off-feature the crate is **empty**, and any production reference
//! to any of it is E0432 (unresolved import) rather than a doc comment nobody
//! reads. Real PKI for the OS port is a separate piece of work; until it lands,
//! the build graph says so out loud instead of shipping a model that looks like
//! an implementation.
//!
//! Measured on this tree: the production `os-secrets-domain` library has five
//! reverse dependencies and **no binary** among them, and neither consuming
//! library uses it outside `#[cfg(test)]`, so this gate removes no production
//! behaviour — it removes the ability to acquire some.
#![cfg(any(test, feature = "modeled-crypto"))]

pub mod api;
pub mod bundle;
pub mod certsans;
pub mod etcd;
pub mod kubernetes;
pub mod kubernetes_projection;
pub mod rotation;
pub mod trustd;

pub use bundle::{
    CA_TTL_SECS, CaKind, CertUsage, Certificate, CertificateAuthority, ClusterIdentity,
    InMemorySigner, KeyPair, ModelPemSecretMaterialEncoder, ModelSecretMaterialEncoder,
    SecretMaterialEncoder, SecretsBundle, Signer, Subject, Token, Validity,
};
pub use certsans::{CertSans, San};
pub use kubernetes_projection::{
    KUBERNETES_SECRET_PROJECTION_NAMES, KubernetesSecretEntry, kubernetes_secret_entries,
    kubernetes_secret_entries_with_encoder,
};
pub use rotation::{CertState, RenewalPolicy};

use std::collections::BTreeMap;
use os_kernel::error::{Error, Result};
use os_kernel::os::FileSystem;

// ---------------------------------------------------------------------------
// Secret status resources
// ---------------------------------------------------------------------------

/// Which secret-bearing subsystem a status resource describes. Talos publishes
/// one status resource per controller so the rest of machined can wait on PKI
/// readiness before starting the corresponding service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretKind {
    /// The root secrets bundle (`secrets/Root`).
    Root,
    /// Talos API (`apid`) secrets.
    Api,
    /// `trustd` secrets.
    Trustd,
    /// Kubernetes control-plane secrets.
    Kubernetes,
    /// etcd secrets.
    Etcd,
}

impl SecretKind {
    /// The COSI-style resource id Talos uses for this status.
    pub fn resource_id(self) -> &'static str {
        match self {
            SecretKind::Root => "secrets/root",
            SecretKind::Api => "secrets/api",
            SecretKind::Trustd => "secrets/trustd",
            SecretKind::Kubernetes => "secrets/kubernetes",
            SecretKind::Etcd => "secrets/etcd",
        }
    }
}

/// Whether the secrets for a subsystem are ready (all required certs present
/// and valid) or still pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretReadiness {
    /// One or more required certs are missing/expired/stale.
    Pending,
    /// All required certs are present and valid.
    Ready,
}

impl SecretReadiness {
    /// Whether this is the ready state.
    pub fn is_ready(self) -> bool {
        matches!(self, SecretReadiness::Ready)
    }

    /// Lowercase string form.
    pub fn as_str(self) -> &'static str {
        match self {
            SecretReadiness::Pending => "pending",
            SecretReadiness::Ready => "ready",
        }
    }
}

/// A status resource published by a secrets controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretStatus {
    /// Which subsystem.
    pub kind: SecretKind,
    /// Readiness.
    pub readiness: SecretReadiness,
    /// A monotonically increasing version bumped on every change, mirroring
    /// COSI resource metadata versioning.
    pub version: u64,
    /// Optional human-readable detail (e.g. why pending).
    pub detail: Option<String>,
}

impl SecretStatus {
    /// A fresh pending status at version 1.
    pub fn pending(kind: SecretKind) -> Self {
        SecretStatus {
            kind,
            readiness: SecretReadiness::Pending,
            version: 1,
            detail: None,
        }
    }

    /// Transition to ready, bumping the version if it changed.
    pub fn mark_ready(&mut self) {
        if self.readiness != SecretReadiness::Ready {
            self.readiness = SecretReadiness::Ready;
            self.detail = None;
            self.version += 1;
        }
    }

    /// Transition to pending with a reason, bumping the version if it changed.
    pub fn mark_pending(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        let changed =
            self.readiness != SecretReadiness::Pending || self.detail.as_deref() != Some(&reason);
        if changed {
            self.readiness = SecretReadiness::Pending;
            self.detail = Some(reason);
            self.version += 1;
        }
    }
}

/// An in-memory registry of the secret status resources, keyed by kind. Stands
/// in for the COSI resource store the controllers write into.
#[derive(Debug, Clone, Default)]
pub struct SecretStatusRegistry {
    statuses: BTreeMap<SecretKind, SecretStatus>,
}

impl SecretStatusRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        SecretStatusRegistry {
            statuses: BTreeMap::new(),
        }
    }

    /// Set/update the readiness for a kind, creating the status if absent.
    pub fn set(&mut self, kind: SecretKind, ready: bool, detail: Option<&str>) {
        let entry = self
            .statuses
            .entry(kind)
            .or_insert_with(|| SecretStatus::pending(kind));
        if ready {
            entry.mark_ready();
        } else {
            entry.mark_pending(detail.unwrap_or("not ready"));
        }
    }

    /// Read a status.
    pub fn get(&self, kind: SecretKind) -> Option<&SecretStatus> {
        self.statuses.get(&kind)
    }

    /// Whether a kind is ready.
    pub fn is_ready(&self, kind: SecretKind) -> bool {
        self.statuses
            .get(&kind)
            .map(|s| s.readiness.is_ready())
            .unwrap_or(false)
    }

    /// Whether all tracked kinds are ready.
    pub fn all_ready(&self) -> bool {
        !self.statuses.is_empty() && self.statuses.values().all(|s| s.readiness.is_ready())
    }
}

// ---------------------------------------------------------------------------
// Bundle persistence
// ---------------------------------------------------------------------------

/// The on-disk path Talos persists the secrets bundle to (`STATE` partition).
pub const BUNDLE_PATH: &str = "/system/state/secrets.yaml";

/// Persistence boundary for the secrets bundle. Talos serializes the bundle to
/// `STATE` so it survives reboots. We model save/load as a small trait over the
/// [`FileSystem`] boundary; serialization uses a stable, dependency-free
/// key=value text format keyed by the deterministic generation seed.
pub trait BundlePersistence {
    /// Persist a bundle's generation parameters so it can be reproduced.
    fn save(&mut self, seed: &str, created_at: u64) -> Result<()>;

    /// Load and regenerate a bundle, or `Ok(None)` if none is stored.
    fn load(&self) -> Result<Option<SecretsBundle>>;

    /// Whether a bundle has been persisted.
    fn exists(&self) -> bool;
}

/// A [`BundlePersistence`] implementation backed by a [`FileSystem`].
///
/// We persist the generation seed and creation time rather than raw key
/// material, which keeps the round-trip deterministic with the in-memory key
/// model while still exercising the load/store path the real controller uses.
pub struct FsBundleStore<F: FileSystem> {
    fs: F,
    path: String,
}

impl<F: FileSystem> FsBundleStore<F> {
    /// Construct over a filesystem, using the default [`BUNDLE_PATH`].
    pub fn new(fs: F) -> Self {
        FsBundleStore {
            fs,
            path: BUNDLE_PATH.to_string(),
        }
    }

    /// Construct with a custom path.
    pub fn with_path(fs: F, path: impl Into<String>) -> Self {
        FsBundleStore {
            fs,
            path: path.into(),
        }
    }

    /// Borrow the underlying filesystem.
    pub fn fs(&self) -> &F {
        &self.fs
    }
}

impl<F: FileSystem> BundlePersistence for FsBundleStore<F> {
    fn save(&mut self, seed: &str, created_at: u64) -> Result<()> {
        if seed.contains('\n') || seed.is_empty() {
            return Err(Error::invalid("invalid bundle seed"));
        }
        let body = format!("seed={seed}\ncreated_at={created_at}\n");
        self.fs.write(&self.path, body.as_bytes())
    }

    fn load(&self) -> Result<Option<SecretsBundle>> {
        if !self.fs.exists(&self.path) {
            return Ok(None);
        }
        let text = self.fs.read_to_string(&self.path)?;
        let mut seed: Option<String> = None;
        let mut created_at: Option<u64> = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (k, v) = line
                .split_once('=')
                .ok_or_else(|| Error::parse("malformed bundle line"))?;
            match k {
                "seed" => seed = Some(v.to_string()),
                "created_at" => {
                    created_at = Some(v.parse().map_err(|_| Error::parse("bad created_at"))?)
                }
                _ => return Err(Error::parse(format!("unknown bundle key '{k}'"))),
            }
        }
        let seed = seed.ok_or_else(|| Error::parse("bundle missing seed"))?;
        let created_at = created_at.ok_or_else(|| Error::parse("bundle missing created_at"))?;
        Ok(Some(SecretsBundle::generate(&seed, created_at)?))
    }

    fn exists(&self) -> bool {
        self.fs.exists(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::os::MemoryFs;

    /// The crate-root gate must stay on a line of its own in `src/lib.rs`.
    ///
    /// The barrier itself is the `cfg`, not this test — no test can watch the
    /// gate bite from inside a build where the gate is open, which every test
    /// build is. What this test buys is that the gate cannot leave *quietly*:
    /// deleting it, editing it, or commenting it out all turn this red.
    ///
    /// Matching at the *start* of a line rather than anywhere in the file is
    /// deliberate. A `contains` check passes on a gate that has been commented
    /// out — a `//`-prefixed line still contains the string — which is the
    /// known hole in the sibling guards in `os-trustd-domain` and
    /// `os-cluster-mgmt-domain`. Prefix-matching a whole line closes it while
    /// still allowing a trailing comment on the gate itself. It also stops the
    /// test satisfying itself: `GATE` below is written with escaped quotes, so
    /// its own source line does not begin with the value it holds.
    ///
    /// Proven to fire: commenting the gate out and running this test gives
    ///
    /// ```text
    /// src/lib.rs must carry the crate-root modeled-crypto gate on a line of its own
    /// ```
    #[test]
    fn crate_root_gate_is_present() {
        const GATE: &str = "#![cfg(any(test, feature = \"modeled-crypto\"))]";
        assert!(
            include_str!("lib.rs")
                .lines()
                .any(|l| l.trim_start().starts_with(GATE)),
            "src/lib.rs must carry the crate-root modeled-crypto gate on a line of its own"
        );
    }

    /// No production build target may turn the model on.
    ///
    /// This is the ONE route to the model that the modeled target's restricted
    /// `visibility` cannot express. Visibility answers "who may depend on the
    /// modeled target" — mechanically, as a buck2 analysis error. It says
    /// nothing about the *production* target growing
    /// `features = ["modeled-crypto"]`, which would hand the model to every
    /// consumer of `os-secrets-domain` without touching the modeled target at
    /// all. So this test is not redundant with visibility; it is its complement,
    /// and it is deliberately the smallest form that covers that one route.
    ///
    /// Exactly one occurrence is asserted rather than parsed per target, which
    /// also makes it anti-vacuous for free: a rename or reshape that loses the
    /// modeled target drops the count to 0 and turns this red, so it cannot
    /// silently stop checking anything.
    ///
    /// Proven to fire, by mutation rather than by argument. Adding
    /// `features = ["modeled-crypto"]` to the production `rust_library` gives
    ///
    /// ```text
    /// assertion `left == right` failed: exactly one BUCK target may enable
    /// modeled-crypto (`os-secrets-domain-modeled`)
    ///   left: 2
    ///  right: 1
    /// ```
    #[test]
    fn no_production_buck_target_enables_the_model() {
        // Comments are stripped first so prose about the feature cannot be read
        // as a target enabling it.
        let buck: String = include_str!("../BUCK")
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(
            buck.matches("modeled-crypto").count(),
            1,
            "exactly one BUCK target may enable modeled-crypto \
             (`os-secrets-domain-modeled`); a second occurrence means another \
             target grew the feature, and zero means this guard stopped \
             checking anything"
        );
    }

    /// The feature must never become a DEFAULT feature.
    ///
    /// Non-defaultness is the whole load-bearing property: the crate-root
    /// `cfg` only strips the crate while the feature is off, and a `default`
    /// entry turns it on for every `cargo build` in the workspace. Before this
    /// test that property was carried by a comment in `Cargo.toml` and nothing
    /// else — the sibling guards read `src/lib.rs` and `BUCK`, and neither
    /// reads the manifest. buck2 stays green through such a change because
    /// buck2 features come from the target attribute and never consult
    /// `Cargo.toml` at all, so buck2 alone cannot notice it.
    ///
    /// Proven to fire, by mutation: adding `default = ["modeled-crypto"]` to
    /// the `[features]` section gives
    ///
    /// ```text
    /// Cargo.toml [features] must not declare `default`: modeled-crypto is
    /// non-default and that is the whole barrier
    /// ```
    #[test]
    fn cargo_manifest_declares_no_default_feature() {
        let manifest = include_str!("../Cargo.toml");

        let features: Vec<&str> = manifest
            .lines()
            .skip_while(|l| l.trim() != "[features]")
            .skip(1)
            .take_while(|l| !l.trim_start().starts_with('['))
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        assert_eq!(
            features,
            ["modeled-crypto = []"],
            "Cargo.toml [features] must not declare `default`: modeled-crypto \
             is non-default and that is the whole barrier"
        );
    }

    #[test]
    fn secret_status_version_bumps_on_change() {
        let mut s = SecretStatus::pending(SecretKind::Etcd);
        assert_eq!(s.version, 1);
        assert!(!s.readiness.is_ready());
        s.mark_ready();
        assert_eq!(s.version, 2);
        assert!(s.readiness.is_ready());
        // Idempotent: no further bump.
        s.mark_ready();
        assert_eq!(s.version, 2);
        s.mark_pending("etcd ca rotated");
        assert_eq!(s.version, 3);
        assert_eq!(s.detail.as_deref(), Some("etcd ca rotated"));
        assert_eq!(s.readiness.as_str(), "pending");
    }

    #[test]
    fn registry_tracks_all_ready() {
        let mut reg = SecretStatusRegistry::new();
        reg.set(SecretKind::Root, true, None);
        reg.set(SecretKind::Kubernetes, false, Some("issuing"));
        assert!(reg.is_ready(SecretKind::Root));
        assert!(!reg.all_ready());
        reg.set(SecretKind::Kubernetes, true, None);
        assert!(reg.all_ready());
        assert_eq!(SecretKind::Etcd.resource_id(), "secrets/etcd");
        // Root went pending(v1) -> ready(v2).
        assert_eq!(reg.get(SecretKind::Root).unwrap().version, 2);
    }

    #[test]
    fn empty_registry_is_not_all_ready() {
        let reg = SecretStatusRegistry::new();
        assert!(!reg.all_ready());
    }

    #[test]
    fn bundle_persistence_round_trips() {
        let mut store = FsBundleStore::new(MemoryFs::new());
        assert!(!store.exists());
        assert!(store.load().unwrap().is_none());

        store.save("my-cluster", 1000).unwrap();
        assert!(store.exists());

        let loaded = store.load().unwrap().unwrap();
        loaded.validate().unwrap();
        // Deterministic regeneration: same CA public key as a fresh generate.
        let fresh = SecretsBundle::generate("my-cluster", 1000).unwrap();
        assert_eq!(
            loaded.ca(CaKind::Os).keypair().public_der(),
            fresh.ca(CaKind::Os).keypair().public_der()
        );
    }

    #[test]
    fn bundle_persistence_rejects_bad_input() {
        let mut store = FsBundleStore::new(MemoryFs::new());
        assert!(store.save("", 1).is_err());
        assert!(store.save("bad\nseed", 1).is_err());
    }

    #[test]
    fn load_rejects_malformed_file() {
        let mut fs = MemoryFs::new();
        fs.write(BUNDLE_PATH, b"garbage-without-equals\n").unwrap();
        let store = FsBundleStore::new(fs);
        assert!(store.load().is_err());
    }

    #[test]
    fn end_to_end_root_ready_drives_status() {
        // Generate a bundle, publish root readiness, then run the k8s controller
        // and mark kubernetes ready — exercising the cross-module flow.
        let mut bundle = SecretsBundle::generate("e2e", 1000).unwrap();
        let mut reg = SecretStatusRegistry::new();
        reg.set(SecretKind::Root, bundle.validate().is_ok(), None);
        assert!(reg.is_ready(SecretKind::Root));

        let mut sans = CertSans::new();
        sans.append("10.0.0.1").unwrap();
        let mut k8s = kubernetes::KubernetesController::new(sans, "cluster.local").unwrap();
        let issued = k8s.reconcile(&mut bundle, 1000).unwrap();
        let ready = issued.len() == kubernetes::K8sCert::all().len();
        reg.set(SecretKind::Kubernetes, ready, None);
        assert!(reg.is_ready(SecretKind::Kubernetes));
    }
}
