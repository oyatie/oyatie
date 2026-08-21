//! DR-pairing controller — IP-019 (`tenancy/IP-019-dr-pairing-controller.md`).
//!
//! Assigns every tenant a same-jurisdiction home/DR cell pair, decides
//! whether that pair may be promoted, drives failover and failback as an
//! explicit state machine, and narrates every transition to an audit sink.
//!
//! The controlling idea is that a DR promotion is an isolation decision
//! before it is an availability decision. A tenant carries an immutable
//! jurisdiction; promoting it into a cell outside that jurisdiction would
//! open a residency incident while closing an availability one. So the
//! same-jurisdiction rule is a hard filter at assignment, re-checked at
//! promotion, and re-checked again at failback — never a preference and
//! never a score.
//!
//! # Shape
//!
//! IP-019 sketches a four-crate layout (`kernel`/`domain`/`usecase`/
//! `adapter`). The tenancy capability is capped at twelve crates and the
//! workspace lockfile is frozen, so the same separation lands here as a
//! module tree inside one crate:
//!
//! - [`kernel`] — the vocabulary: [`DrPair`], [`PairState`],
//!   [`PromotionDecision`], the [`reason`] code enumeration, the five
//!   ports, [`DrPairAuditEvent`], and [`DrPairingError`].
//! - [`domain`] — pure decisions: candidate scoring and selection, the
//!   legal-transition table, version arithmetic, the idempotency key.
//! - [`usecase`] — [`DrPairingController`]: assignment, re-planning,
//!   assessment, promotion, restoration, each committed under
//!   compare-and-swap, plus the re-narration path that closes an audit gap
//!   left by a sink outage.
//! - [`inmemory`] — an adapter for every port, with fault injection.
//!
//! # Guarantees
//!
//! - **Residency.** No [`DrPairingController`] write path records a pair
//!   whose cells sit in different jurisdictions: the catalog must place
//!   both inside the declared jurisdiction and the residency policy must
//!   permit the placement, or the write is refused with
//!   [`DrPairingError::JurisdictionMismatch`]. The controller is where that
//!   control lives, and it is the application's write path. The
//!   [`DrPairRepository`] port underneath it is NOT: `record` is how a
//!   compare-and-swap lands its bytes and how a migration backfills a
//!   store, it cannot see the cell catalog, and a caller that reaches past
//!   the controller to call it directly writes a row no control agreed to.
//!   [`evaluate_promotion`] will not catch such a row either — it reads
//!   only the store and the probe, as its own docs say. Residency is a
//!   property of the controller, not of the port.
//! - **A DR cell is not the home cell.** `home_cell == dr_cell` is refused
//!   at assignment and, if such a row is found in the store, reported as
//!   [`reason::DEGENERATE_PAIR`] rather than promoted.
//! - **No split brain.** Every transition carries the version it believes
//!   it is changing and commits through
//!   [`DrPairRepository::compare_and_swap`]. A stale write is refused with
//!   [`DrPairingError::StalePairVersion`], never applied. The guard rests
//!   on a monotonic version chain, so [`DrPairRepository::record`] must
//!   refuse a write that does not advance the version —
//!   [`inmemory::InMemoryDrPairRepository`] does, with
//!   [`DrPairingError::NonMonotonicPairVersion`].
//! - **Nothing commits unnarrated and stays that way.** A transition
//!   commits before it can be narrated, so an audit-sink outage can leave a
//!   durable state change without its event. That is reported as
//!   [`DrPairingError::NarrationPending`] — never as
//!   [`DrPairingError::StalePairVersion`], which would read as somebody
//!   else's write — and it carries the committed version, which
//!   [`DrPairingController::renarrate`] turns back into the identical
//!   event. Identical, because every field is derived: the same
//!   idempotency key, so a deduping sink accepts it exactly once however
//!   many times the caller retries.
//! - **No event claims a signal it did not check.** An audit event's
//!   `decision` comes from [`domain::decision_for_event`], and only a
//!   promotion — the one transition that runs the assessment — records
//!   `Eligible`.
//! - **Fail closed, distinguishably.** A probe that cannot answer yields
//!   [`DrPairingError::SloProbeFailed`], not `Eligible` and not an
//!   unexplained block. Every block carries a documented code from
//!   [`reason`], and [`reason::text`] turns that code back into the
//!   sentence an operator needs at 3am.
//! - **Determinism.** Nothing in this crate reads a clock or draws
//!   randomness. Instants arrive as `at_millis` parameters and idempotency
//!   keys are derived from the transition, so a replay is byte-identical.
//!
//! # Gaps
//!
//! Deliberately deferred, and why:
//!
//! - **No durable adapter.** IP-019 persists pairs in Postgres and reads
//!   SLO signals from the tenancy dashboards. This crate ships the ports
//!   and an in-memory implementation only; adding `sqlx`, a Valkey client,
//!   or an HTTP client would change the workspace lockfile, which this lane
//!   may not do. [`inmemory`] is the whole adapter surface today.
//! - **No `cell-assignment` dependency.** IP-019 sources cell composition
//!   from the sibling `cell-assignment` bounded context. A path dependency
//!   on a sibling crate also rewrites the lockfile, so the fleet read model
//!   is declared locally as [`DrCellCatalog`] / [`DrCellCandidate`]. That
//!   is a real coupling left unlinked: when the two crates are wired, the
//!   adapter that satisfies [`DrCellCatalog`] from `cell-assignment` is the
//!   piece that closes it, and the two models must be reconciled then.
//! - **No Cedar evaluation.** IP-019 evaluates
//!   `tenancy/policy/data-residency.cedar` before any pair assignment or
//!   promotion. Cedar cannot be linked here for the same reason, so the
//!   policy is a [`ResidencyPolicy`] port. The jurisdiction-equality half
//!   of the rule IS enforced in-crate against the catalog; the
//!   policy-authored half is delegated to whatever implements the port.
//! - **No cryptographic hash.** IP-019 reaches for a content hash for the
//!   idempotency key. `blake3` is an external crate, so
//!   [`domain::derive_idempotency_key`] uses an inline FNV-1a-64. That is
//!   adequate for a sink dedupe table and is NOT a tamper-evidence claim;
//!   the audit-chain seal in IP-019 needs a real hash.
//! - **Synchronous ports.** The ports are sync traits, not `async`, because
//!   `tokio` is likewise out of reach. The decision logic is CPU-only, so
//!   the async boundary belongs in the adapters that eventually wrap these
//!   traits, not in the controller.
//! - **No TrueTime/HLC ordering.** IP-019 §D.5 orders promotions with the
//!   ADR-0252 clock abstractions. Here `at_millis` is caller-supplied and
//!   used only for the audit record; ordering rests entirely on
//!   `pair_version` compare-and-swap. That is sound for a single pair and
//!   is NOT a cross-tenant global ordering claim.
//! - **Errors are context-free codes.** [`DrPairingError`] is a `Copy`
//!   enumeration, so a refusal names WHAT failed but not WHICH cell: an
//!   assignment refused with [`DrPairingError::JurisdictionMismatch`] does
//!   not say whether the home cell or the DR cell drifted, and a refused
//!   assignment writes no audit event, so nothing in the trail records the
//!   attempt. Carrying the cell id would make the error allocate; the
//!   honest fix is a structured refusal event, which is deferred with the
//!   rest of the durable adapter work.
//! - **In-memory only, and not a store.** [`inmemory`] holds everything in
//!   process memory: a restart loses every pair and every buffered event,
//!   [`inmemory::RecordingEventSink`] has no cap of its own (drain it), and
//!   its fault-injection switches take `&self` and are not feature-gated.
//!   Those are drill affordances; see the module docs.
//! - **No quorum guard.** IP-019 §D.4 requires operator approval for a
//!   planned exercise and burn-rate-breach-plus-quorum for an automatic
//!   promotion. This crate models the state gate ([`PairState::Planned`]
//!   never promotes) and the health gate, but the approval and quorum
//!   inputs are not represented — a caller supplies them today.
//! - **RPO/RTO are declarations.** `rpo_seconds` and `rto_seconds` are
//!   carried on the pair and never checked against a measurement. Per
//!   IP-019 §E they remain capability targets until drill evidence lands.
//!
//! ADR-0083 Tier-3: production code here carries no `unwrap`/`expect`/`panic`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use kernel::{
    DrCellCandidate, DrCellCatalog, DrPair, DrPairAuditEvent, DrPairEventKind, DrPairEventSink,
    DrPairRepository, DrPairingError, DrSloProbe, PairState, PromotionDecision, ResidencyPolicy,
    is_tight_identifier, reason,
};
pub use usecase::{DrPairingController, FailoverCommand, PairAssignment, evaluate_promotion};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inmemory::{InMemoryDrPairRepository, StaticDrSloProbe};

    fn pair(state: PairState) -> DrPair {
        DrPair {
            tenant_id: "ten_alpha".to_owned(),
            home_cell: "cell-eu-1".to_owned(),
            dr_cell: "cell-eu-2".to_owned(),
            jurisdiction: "eu".to_owned(),
            pair_version: 3,
            state,
            rpo_seconds: 30,
            rto_seconds: 300,
        }
    }

    #[test]
    fn every_blocking_reason_code_is_documented_and_unique() {
        let mut codes: Vec<u16> = reason::CATALOG.iter().map(|(code, _)| *code).collect();
        let count = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), count, "reason codes must be unique");
        assert!(
            !codes.contains(&reason::UNSPECIFIED),
            "0 is the reserved sentinel and is never a blocking reason"
        );
        for (code, text) in reason::CATALOG {
            assert_eq!(reason::text(*code), Some(*text));
        }
        assert_eq!(reason::text(reason::UNSPECIFIED), None);
        assert_eq!(reason::text(9_999), None);
    }

    #[test]
    fn the_stub_decision_is_no_longer_well_formed() {
        // The scaffold returned `Blocked { reason_code: 0 }` unconditionally.
        // 0 is now an explicit sentinel, so that value is rejected on sight.
        assert!(
            !PromotionDecision::Blocked {
                reason_code: reason::UNSPECIFIED
            }
            .is_well_formed()
        );
        assert!(PromotionDecision::Eligible.is_well_formed());
        assert!(
            PromotionDecision::Blocked {
                reason_code: reason::DR_REPLICA_UNHEALTHY
            }
            .is_well_formed()
        );
    }

    #[test]
    fn a_blocked_decision_renders_its_reason_for_an_operator() {
        let rendered = PromotionDecision::Blocked {
            reason_code: reason::JURISDICTION_DRIFT,
        }
        .to_string();
        assert_eq!(
            rendered,
            "blocked[5]: the pair's cells no longer share the recorded jurisdiction"
        );
    }

    #[test]
    fn evaluate_promotion_blocks_a_missing_pair_rather_than_erroring() {
        let repo = InMemoryDrPairRepository::new();
        let probe = StaticDrSloProbe::new().with_healthy("cell-eu-2");
        let decision =
            evaluate_promotion(&repo, &probe, "ten_ghost").expect("a missing pair is not an error");
        assert_eq!(
            decision,
            PromotionDecision::Blocked {
                reason_code: reason::PAIR_NOT_FOUND
            }
        );
    }

    #[test]
    fn evaluate_promotion_admits_a_healthy_activated_pair() {
        let repo = InMemoryDrPairRepository::new();
        repo.record(&pair(PairState::HomeActive))
            .expect("the fixture pair is valid");
        let probe = StaticDrSloProbe::new().with_healthy("cell-eu-2");
        assert_eq!(
            evaluate_promotion(&repo, &probe, "ten_alpha"),
            Ok(PromotionDecision::Eligible)
        );
    }

    #[test]
    fn evaluate_promotion_refuses_a_planned_pair() {
        let repo = InMemoryDrPairRepository::new();
        repo.record(&pair(PairState::Planned))
            .expect("the fixture pair is valid");
        let probe = StaticDrSloProbe::new().with_healthy("cell-eu-2");
        assert_eq!(
            evaluate_promotion(&repo, &probe, "ten_alpha"),
            Ok(PromotionDecision::Blocked {
                reason_code: reason::PAIR_NOT_ACTIVATED
            })
        );
    }

    #[test]
    fn evaluate_promotion_surfaces_a_probe_failure_as_an_error() {
        let repo = InMemoryDrPairRepository::new();
        repo.record(&pair(PairState::HomeActive))
            .expect("the fixture pair is valid");
        let probe = StaticDrSloProbe::new().with_probe_failure("cell-eu-2");
        assert_eq!(
            evaluate_promotion(&repo, &probe, "ten_alpha"),
            Err(DrPairingError::SloProbeFailed)
        );
    }

    #[test]
    fn a_repository_refuses_to_record_a_pair_whose_dr_cell_is_its_home_cell() {
        let repo = InMemoryDrPairRepository::new();
        let mut degenerate = pair(PairState::HomeActive);
        degenerate.dr_cell.clone_from(&degenerate.home_cell);
        assert_eq!(
            repo.record(&degenerate),
            Err(DrPairingError::HomeCellIsDrCell)
        );
        assert_eq!(repo.is_empty(), Ok(true));
    }

    #[test]
    fn serving_cell_follows_the_lifecycle_state() {
        assert_eq!(pair(PairState::Planned).serving_cell(), "cell-eu-1");
        assert_eq!(pair(PairState::HomeActive).serving_cell(), "cell-eu-1");
        assert_eq!(pair(PairState::DrActive).serving_cell(), "cell-eu-2");
    }

    #[test]
    fn a_pair_with_a_padded_identifier_is_not_a_valid_pair() {
        assert!(is_tight_identifier("ten_alpha"));
        assert!(!is_tight_identifier("ten_alpha "));
        assert!(!is_tight_identifier(" "));
        assert!(!is_tight_identifier(""));

        let mut padded = pair(PairState::HomeActive);
        padded.tenant_id = "ten_alpha ".to_owned();
        assert_eq!(padded.validate(), Err(DrPairingError::InvalidTenantId));

        let mut padded_cell = pair(PairState::HomeActive);
        padded_cell.dr_cell = " cell-eu-2".to_owned();
        assert_eq!(padded_cell.validate(), Err(DrPairingError::InvalidCellId));

        let mut padded_juris = pair(PairState::HomeActive);
        padded_juris.jurisdiction = "eu\t".to_owned();
        assert_eq!(
            padded_juris.validate(),
            Err(DrPairingError::InvalidJurisdiction)
        );
    }

    #[test]
    fn version_zero_is_not_a_stored_version() {
        let mut unversioned = pair(PairState::HomeActive);
        unversioned.pair_version = 0;
        assert_eq!(
            unversioned.validate(),
            Err(DrPairingError::InvalidPairVersion)
        );
        // A row like that reads as degenerate rather than promotable.
        let repo = InMemoryDrPairRepository::new();
        repo.seed_unchecked(&unversioned)
            .expect("a legacy row can be seeded");
        let probe = StaticDrSloProbe::new().with_healthy("cell-eu-2");
        assert_eq!(
            evaluate_promotion(&repo, &probe, "ten_alpha"),
            Ok(PromotionDecision::Blocked {
                reason_code: reason::DEGENERATE_PAIR
            })
        );
    }

    #[test]
    fn errors_render_a_distinct_operator_sentence() {
        assert_eq!(
            DrPairingError::SloProbeFailed.to_string(),
            "the DR SLO probe could not answer"
        );
        assert_eq!(
            DrPairingError::JurisdictionMismatch.to_string(),
            "home and DR cells are in different jurisdictions"
        );
        assert_eq!(
            DrPairingError::PromotionBlocked {
                reason_code: reason::ALREADY_PROMOTED
            }
            .to_string(),
            "promotion blocked[3]: the tenant is already serving from its DR cell"
        );
        assert_eq!(
            DrPairingError::NarrationPending {
                committed_version: 7
            }
            .to_string(),
            "the transition committed at version 7 but its audit event was not accepted; re-narrate it"
        );
        assert_eq!(
            DrPairingError::NonMonotonicPairVersion.to_string(),
            "the write would not advance the stored pair version"
        );
    }
}
