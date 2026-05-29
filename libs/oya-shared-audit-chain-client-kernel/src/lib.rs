//! Audit-chain client kernel — per-µservice trait surface for ADR-0145
//! Invariant 1 (decentralized audit emission).
//!
//! # ADR-0145 Invariant 1
//!
//! Every state-changing inter-µservice call MUST emit an audit-chain
//! seal AT THE CALLING service. Each µservice integrates this trait
//! and emits via its own audit-client; no central mediator.
//!
//! # Skeleton scope
//!
//! This crate currently ships the TRAIT SURFACE only. The
//! production impl is tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-client-impl`.
//!
//! Methods returning concrete behavior `unimplemented!()` for the
//! skeleton; trait-impl tests against a `NoopAuditChainClient` are
//! provided so dependents can compile + smoke against the trait.
//!
//! # Naming justification
//!
//! `oya-shared-audit-chain-client-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:audit-chain-client>-<layer:kernel>`. The
//! `shared` axis is the canonical Oyatie identifier for cross-µservice
//! substrate (see feedback_glossary_shared_not_platform memory).
//!
//! # References
//!
//! - ADR-0145 — inter-microservice communication reform.
//! - ADR-0056 — port-in-kernel; this crate is layer=kernel.
//! - ADR-0083 — audit-chain canonical seal format.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// Logical identifier of the calling µservice (e.g. "network", "tasks").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallingMicroservice(pub String); // data_class: INTERNAL_ONLY

/// Logical identifier of the called µservice for the seal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalledMicroservice(pub String); // data_class: INTERNAL_ONLY

/// Canonical seal event kind. Closed enum; expand only via ADR.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SealEventKind {
    /// State-changing capability invocation (write path).
    StateChange,
    /// Authorization decision recorded for audit replay.
    AuthorizationDecision,
    /// Cross-µservice ack of a previously-emitted state change.
    Acknowledgement,
}

/// One seal emission request. The kernel does NOT canonicalize the
/// payload here; that lives in the production impl tracked by the
/// follow-up record. The skeleton accepts opaque payload bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealEmission {
    pub from: CallingMicroservice,  // data_class: INTERNAL_ONLY
    pub to: CalledMicroservice,     // data_class: INTERNAL_ONLY
    pub capability_id: String,      // data_class: INTERNAL_ONLY
    pub event_kind: SealEventKind,  // data_class: INTERNAL_ONLY
    pub trace_id: String,           // data_class: INTERNAL_ONLY
    pub payload_digest_hex: String, // data_class: INTERNAL_ONLY
}

/// Failure surface for the kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditChainClientError {
    /// The skeleton's deliberately-unimplemented path was invoked. The
    /// production impl will replace this with concrete error variants.
    SkeletonNotYetImplemented(&'static str),
    /// A required field on the seal emission was empty.
    EmptyField(&'static str),
}

impl fmt::Display for AuditChainClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditChainClientError::SkeletonNotYetImplemented(method) => write!(
                f,
                "oya-shared-audit-chain-client-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-client-impl)"
            ),
            AuditChainClientError::EmptyField(name) => {
                write!(
                    f,
                    "oya-shared-audit-chain-client-kernel: required field {name:?} is empty"
                )
            }
        }
    }
}

impl std::error::Error for AuditChainClientError {}

/// The trait every µservice integrates to emit a seal at the calling
/// site of any state-changing inter-µservice call.
pub trait AuditChainClient: Send + Sync {
    /// Emit a single seal. Production impl writes to the audit-chain
    /// µservice via its canonical gRPC surface; the skeleton method
    /// returns `SkeletonNotYetImplemented` so dependents can wire the
    /// trait without coupling to the missing impl.
    ///
    /// # Errors
    /// - `EmptyField` when any required field on `emission` is empty.
    /// - `SkeletonNotYetImplemented` for the default skeleton impl.
    fn emit_seal(&self, emission: &SealEmission) -> Result<(), AuditChainClientError>;
}

/// No-op skeleton client used as the default integration target until
/// the production impl lands. Validates the emission shape and returns
/// `Ok(())` on a well-formed request — this lets call-site integration
/// tests cover field validation without bringing up audit-chain.
#[derive(Clone, Debug, Default)]
pub struct NoopAuditChainClient;

impl AuditChainClient for NoopAuditChainClient {
    fn emit_seal(&self, emission: &SealEmission) -> Result<(), AuditChainClientError> {
        if emission.from.0.is_empty() {
            return Err(AuditChainClientError::EmptyField("from"));
        }
        if emission.to.0.is_empty() {
            return Err(AuditChainClientError::EmptyField("to"));
        }
        if emission.capability_id.is_empty() {
            return Err(AuditChainClientError::EmptyField("capability_id"));
        }
        if emission.trace_id.is_empty() {
            return Err(AuditChainClientError::EmptyField("trace_id"));
        }
        Ok(())
    }
}

/// Skeleton "production-equivalent" placeholder that explicitly returns
/// `SkeletonNotYetImplemented` so dependents that want compile-time
/// proof of unfinished impl can opt in.
#[derive(Clone, Debug, Default)]
pub struct SkeletonAuditChainClient;

impl AuditChainClient for SkeletonAuditChainClient {
    fn emit_seal(&self, _emission: &SealEmission) -> Result<(), AuditChainClientError> {
        Err(AuditChainClientError::SkeletonNotYetImplemented(
            "emit_seal",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_formed_emission() -> SealEmission {
        SealEmission {
            from: CallingMicroservice("network".into()),
            to: CalledMicroservice("audit-chain".into()),
            capability_id: "jobs-handoff.publish".into(),
            event_kind: SealEventKind::StateChange,
            trace_id: "01HMZ1234567890".into(),
            payload_digest_hex: "sha256:abc".into(),
        }
    }

    #[test]
    fn noop_client_accepts_well_formed_emission() {
        let client = NoopAuditChainClient;
        assert!(client.emit_seal(&well_formed_emission()).is_ok());
    }

    #[test]
    fn noop_client_rejects_empty_required_fields() {
        let client = NoopAuditChainClient;
        let mut e = well_formed_emission();
        e.from = CallingMicroservice(String::new());
        assert_eq!(
            client.emit_seal(&e),
            Err(AuditChainClientError::EmptyField("from"))
        );
    }

    #[test]
    fn skeleton_client_returns_not_yet_implemented() {
        let client = SkeletonAuditChainClient;
        let err = client
            .emit_seal(&well_formed_emission())
            .expect_err("skeleton must return not-yet-implemented");
        assert_eq!(
            err,
            AuditChainClientError::SkeletonNotYetImplemented("emit_seal")
        );
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = AuditChainClientError::SkeletonNotYetImplemented("emit_seal");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0145-audit-client-impl"));
    }
}
