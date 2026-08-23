//! Pure SVID-delivery operator reconcile kernel (G002 slice-1b-iii-c; ADR-0561).
//!
//! This crate owns the desired/observed state shapes and the pure reconcile
//! decision function for the in-cluster operator that PRODUCES the PDP's
//! `cloud-iam-pdp-svid` Secret (the single missing producer that closes
//! FRIC-1781490000 and unblocks G004). It intentionally has NO kube-rs,
//! k8s-openapi, async runtime, system-clock, or crypto dependency — issuance and
//! Secret projection live in the transient adapter, the reconcile loop lives in
//! the facade app, and the clock is always injected via [`Clock`].
//!
//! ## The decision
//!
//! The operator drives a single platform SVID for the PDP server identity at
//! `spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`. The reconcile function is
//! a pure transform from (observed Secret state, desired SVID spec, now) to one
//! of three [`Action`]s:
//!
//! - [`Action::Issue`] — no Secret is present yet (the cold-start producer path);
//! - [`Action::Rotate`] — a Secret is present but its leaf is at/within the
//!   rotation window of its expiry (or already expired), so a fresh SVID must be
//!   minted before callers fail-closed at the PDP;
//! - [`Action::Noop`] — a Secret is present and its leaf is comfortably fresh.
//!
//! The kernel never mints, reads, or writes anything: it only decides. The
//! transient adapter executes the chosen action by driving the trustd issuer and
//! projecting the `kubernetes.io/tls` Secret.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests`/integration tests may use unwrap/expect/panic under cfg(test) only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// An injected wall-clock source (epoch seconds). The kernel never reads the
/// system clock directly so reconcile remains a pure, deterministic transform.
pub trait Clock {
    /// The current time as unix epoch seconds.
    fn now_epoch_seconds(&self) -> u64;
}

/// The desired SVID-delivery state for the PDP: the SPIFFE id to embed, the
/// requested certificate lifetime, the rotation window before expiry at which a
/// rotation is forced, and the target Secret coordinates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesiredState {
    /// The platform SVID URI the issued leaf must carry as its single URI SAN
    /// (`spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`).
    pub spiffe_id: String, // data_class: PUBLIC
    /// Requested certificate lifetime in seconds.
    pub ttl_secs: u64, // data_class: PUBLIC
    /// How long before leaf expiry a rotation is forced. A leaf whose remaining
    /// lifetime is at or below this window is rotated proactively so callers
    /// never race the PDP's fail-closed boot against an expired SVID.
    pub rotation_window_secs: u64, // data_class: PUBLIC
    /// The Kubernetes Secret name the projected `kubernetes.io/tls` material is
    /// written to. The consumer contract fixes this to `cloud-iam-pdp-svid`.
    pub secret_name: String, // data_class: PUBLIC
    /// The namespace the Secret is projected into (the iam namespace).
    pub secret_namespace: String, // data_class: PUBLIC
}

/// The observed state of the delivered Secret as read from the cluster.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedState {
    /// The currently-delivered SVID Secret, if one is present in the cluster.
    /// `None` is the cold-start (never-produced) case.
    pub secret: Option<ObservedSvidSecret>, // data_class: PUBLIC
}

impl ObservedState {
    /// An observation in which no Secret is present (cold start).
    #[must_use]
    pub fn absent() -> Self {
        Self { secret: None }
    }

    /// An observation carrying a present Secret whose leaf expires at
    /// `leaf_not_after_epoch_seconds`.
    #[must_use]
    pub fn present(leaf_not_after_epoch_seconds: u64) -> Self {
        Self {
            secret: Some(ObservedSvidSecret {
                leaf_not_after_epoch_seconds,
            }),
        }
    }
}

/// The load-bearing fields of a delivered SVID Secret the reconcile decision
/// consults: the issued leaf's expiry instant. The kernel reasons only about the
/// expiry; the opaque cert material is the adapter's concern.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedSvidSecret {
    /// The `notAfter` of the delivered leaf, as unix epoch seconds.
    pub leaf_not_after_epoch_seconds: u64, // data_class: PUBLIC
}

/// The reconcile decision: the single action the adapter must take to converge
/// the observed Secret state onto the desired SVID-delivery state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// No Secret is present — mint a fresh SVID and create the Secret.
    Issue {
        /// The desired SVID-delivery spec to materialise.
        desired: DesiredState, // data_class: PUBLIC
        /// The issuance instant (epoch seconds).
        requested_at_epoch_seconds: u64, // data_class: PUBLIC
    },
    /// A Secret is present but its leaf is at/within the rotation window (or
    /// already expired) — mint a fresh SVID and update the Secret in place.
    Rotate {
        /// The desired SVID-delivery spec to re-materialise.
        desired: DesiredState, // data_class: PUBLIC
        /// The expiry of the leaf being rotated out (epoch seconds).
        observed_leaf_not_after_epoch_seconds: u64, // data_class: PUBLIC
        /// The issuance instant (epoch seconds).
        requested_at_epoch_seconds: u64, // data_class: PUBLIC
    },
    /// A Secret is present and its leaf is comfortably fresh — do nothing.
    Noop, // data_class: PUBLIC
}

/// Decide the single converging action for the observed Secret state against the
/// desired SVID-delivery spec, as of `clock`.
///
/// The decision is total and deterministic:
/// - no Secret present ⇒ [`Action::Issue`];
/// - Secret present and remaining leaf lifetime ≤ `rotation_window_secs`
///   (including an already-expired leaf) ⇒ [`Action::Rotate`];
/// - otherwise ⇒ [`Action::Noop`].
///
/// Remaining lifetime is computed with a saturating subtraction so a leaf whose
/// `notAfter` is already in the past yields zero remaining lifetime (≤ any
/// window ⇒ rotate), never an underflow.
pub fn reconcile<C: Clock>(observed: &ObservedState, desired: &DesiredState, clock: &C) -> Action {
    let now = clock.now_epoch_seconds();
    match &observed.secret {
        None => Action::Issue {
            desired: desired.clone(),
            requested_at_epoch_seconds: now,
        },
        Some(secret) => {
            let remaining = secret.leaf_not_after_epoch_seconds.saturating_sub(now);
            if remaining <= desired.rotation_window_secs {
                Action::Rotate {
                    desired: desired.clone(),
                    observed_leaf_not_after_epoch_seconds: secret.leaf_not_after_epoch_seconds,
                    requested_at_epoch_seconds: now,
                }
            } else {
                Action::Noop
            }
        }
    }
}
