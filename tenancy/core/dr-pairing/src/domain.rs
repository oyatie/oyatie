//! Pure DR-pairing decision logic: candidate scoring, placement
//! invariants, the failover state machine, version arithmetic, and the
//! deterministic idempotency key.
//!
//! Every function here is a total function of its arguments. No clock, no
//! randomness, no I/O — the same inputs always yield the same decision, so
//! a promotion that a test reproduces is the promotion production made.

use crate::kernel::{
    DrCellCandidate, DrPairEventKind, DrPairingError, PairState, PromotionDecision, reason,
};

/// FNV-1a-64 offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a-64 prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Score floor every candidate starts from, before penalties and bonuses.
const SCORE_BASE: u32 = 1_000;
/// Score added when the candidate sits in a different fault domain from the
/// home cell — the whole point of a DR cell.
const SCORE_FAULT_DOMAIN_BONUS: u32 = 500;
/// Score removed per percentage point of load.
const SCORE_LOAD_WEIGHT: u32 = 4;

/// FNV-1a over raw bytes.
///
/// Deliberately not a cryptographic hash: this value is an idempotency key
/// for a sink's dedupe table, never a security claim. See the crate `Gaps`
/// paragraph.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// The deterministic idempotency key for one transition.
///
/// Derived from (tenant, event label, resulting version) so that a retry
/// after an audit-sink failure produces the same key and the sink can
/// recognise the duplicate. Length-prefixed field framing keeps
/// `("ab", "c")` from colliding with `("a", "bc")`.
#[must_use]
pub fn derive_idempotency_key(tenant_id: &str, event_label: &str, to_version: u32) -> String {
    let mut material = String::new();
    for field in [tenant_id, event_label] {
        material.push_str(&field.len().to_string());
        material.push(':');
        material.push_str(field);
        material.push('|');
    }
    material.push_str(&to_version.to_string());
    format!("drp-{:016x}", fnv1a64(material.as_bytes()))
}

/// The next pair version.
///
/// # Errors
/// [`DrPairingError::PairVersionExhausted`] when `current` is `u32::MAX`.
/// The controller refuses the transition rather than wrapping to 0, which
/// would make every subsequent stale write look current.
pub fn next_version(current: u32) -> Result<u32, DrPairingError> {
    current
        .checked_add(1)
        .ok_or(DrPairingError::PairVersionExhausted)
}

/// Score one candidate for DR placement. Higher is better.
///
/// Only load and fault-domain separation are modelled here; the candidate's
/// health and jurisdiction are hard filters applied by [`select_dr_cell`],
/// not soft scores, because neither is tradeable.
#[must_use]
pub fn score_candidate(home_fault_domain: &str, candidate: &DrCellCandidate) -> u32 {
    let load_penalty = u32::from(candidate.load_percent).saturating_mul(SCORE_LOAD_WEIGHT);
    let base = SCORE_BASE.saturating_sub(load_penalty);
    if candidate.fault_domain == home_fault_domain {
        base
    } else {
        base.saturating_add(SCORE_FAULT_DOMAIN_BONUS)
    }
}

/// Whether a candidate may host the DR side of a pair at all.
///
/// The three filters are absolute: same jurisdiction (a residency control),
/// not the home cell (a DR cell that is the home cell is not a DR cell),
/// and healthy (an unhealthy cell is not a recovery target).
#[must_use]
pub fn is_eligible_candidate(
    home_cell: &str,
    jurisdiction: &str,
    candidate: &DrCellCandidate,
) -> bool {
    candidate.healthy
        && candidate.jurisdiction == jurisdiction
        && candidate.cell_id != home_cell
        && candidate.validate().is_ok()
}

/// Pick the best DR cell for a home cell from a candidate list.
///
/// Deterministic: candidates are compared by score, and ties break on the
/// lexicographically smallest `cell_id` so that two controllers reading the
/// same catalog choose the same cell.
///
/// # Errors
/// [`DrPairingError::NoEligibleDrCell`] when no candidate passes
/// [`is_eligible_candidate`].
pub fn select_dr_cell<'a>(
    home_cell: &str,
    home_fault_domain: &str,
    jurisdiction: &str,
    candidates: &'a [DrCellCandidate],
) -> Result<&'a DrCellCandidate, DrPairingError> {
    let mut best: Option<(&'a DrCellCandidate, u32)> = None;
    for candidate in candidates {
        if !is_eligible_candidate(home_cell, jurisdiction, candidate) {
            continue;
        }
        let score = score_candidate(home_fault_domain, candidate);
        let better = match best {
            None => true,
            Some((incumbent, incumbent_score)) => {
                score > incumbent_score
                    || (score == incumbent_score && candidate.cell_id < incumbent.cell_id)
            }
        };
        if better {
            best = Some((candidate, score));
        }
    }
    best.map(|(candidate, _)| candidate)
        .ok_or(DrPairingError::NoEligibleDrCell)
}

/// The blocking reason a pair's lifecycle state implies for promotion, or
/// `None` when the state permits promotion.
///
/// This is the state half of the failover machine: only `HomeActive`
/// promotes. `Planned` has never been accepted for failover use, and
/// `DrActive` is already promoted — promoting it again is the split-brain
/// move this controller exists to refuse.
#[must_use]
pub const fn promotion_block_for_state(state: PairState) -> Option<u16> {
    match state {
        PairState::HomeActive => None,
        PairState::Planned => Some(reason::PAIR_NOT_ACTIVATED),
        PairState::DrActive => Some(reason::ALREADY_PROMOTED),
    }
}

/// Whether `from -> to` is a legal edge of the failover state machine.
///
/// Legal edges: `Planned -> HomeActive` (activation), `HomeActive ->
/// DrActive` (promotion), `DrActive -> HomeActive` (restoration), and
/// `HomeActive -> Planned` (re-planning: the DR side is re-cabled to an
/// un-exercised cell, which withdraws the failover capability until the
/// pair is activated again). Every other pair, including every self-edge,
/// is illegal — notably `DrActive -> Planned`, because a pair is not
/// re-cabled while it is serving from its DR cell.
///
/// This table is the single authority: every state-changing path in the
/// crate consults it, including assignment, which is why re-planning is a
/// named edge here rather than an unmodelled write.
#[must_use]
pub const fn is_legal_transition(from: PairState, to: PairState) -> bool {
    matches!(
        (from, to),
        (PairState::Planned, PairState::HomeActive)
            | (PairState::HomeActive, PairState::DrActive)
            | (PairState::DrActive, PairState::HomeActive)
            | (PairState::HomeActive, PairState::Planned)
    )
}

/// Whether a stored pair in `state` may be overwritten in place by an
/// assignment, which always writes [`PairState::Planned`].
///
/// Only a pair that is already `Planned` qualifies: rewriting it changes
/// which cell is planned, not whether the tenant can fail over. Overwriting
/// an activated pair would silently demote it — the tenant would keep
/// serving while its failover capability disappeared — so that case is
/// routed through the explicit re-planning edge instead.
#[must_use]
pub const fn permits_in_place_reassignment(state: PairState) -> bool {
    matches!(state, PairState::Planned)
}

/// The promotion posture an audit event of `kind` records.
///
/// Derived from the kind alone, for two reasons. It is honest: only a
/// promotion runs the assessment that can establish `Eligible`, so every
/// other transition records the block that was true when it committed —
/// `PAIR_NOT_ACTIVATED` for anything that leaves the pair `Planned` or
/// takes it out of `Planned`, and `ALREADY_PROMOTED` for a restoration,
/// whose subject was serving from DR at the instant it acted. And it is
/// reproducible: because no probe is involved,
/// [`crate::DrPairingController::renarrate`] can rebuild a lost event with
/// the identical `decision`, and therefore the identical idempotency key.
#[must_use]
pub const fn decision_for_event(kind: DrPairEventKind) -> PromotionDecision {
    match kind {
        DrPairEventKind::PairAssigned
        | DrPairEventKind::PairReplanned
        | DrPairEventKind::PairActivated => PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED,
        },
        DrPairEventKind::Promoted => PromotionDecision::Eligible,
        DrPairEventKind::Restored => PromotionDecision::Blocked {
            reason_code: reason::ALREADY_PROMOTED,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(cell_id: &str, fault_domain: &str, load_percent: u8) -> DrCellCandidate {
        DrCellCandidate {
            cell_id: cell_id.to_owned(),
            jurisdiction: "eu".to_owned(),
            fault_domain: fault_domain.to_owned(),
            healthy: true,
            load_percent,
        }
    }

    #[test]
    fn select_prefers_a_different_fault_domain_over_lower_load() {
        // Same-domain candidate is idle; other-domain candidate is busy.
        // Fault-domain separation is worth more than 50 points of load.
        let candidates = vec![
            candidate("cell-same", "dc-1", 0),
            candidate("cell-other", "dc-2", 50),
        ];
        let chosen = select_dr_cell("cell-home", "dc-1", "eu", &candidates)
            .expect("one candidate is eligible");
        assert_eq!(chosen.cell_id, "cell-other");
    }

    #[test]
    fn select_breaks_ties_on_lowest_cell_id() {
        let candidates = vec![
            candidate("cell-zulu", "dc-2", 10),
            candidate("cell-alpha", "dc-2", 10),
        ];
        let chosen =
            select_dr_cell("cell-home", "dc-1", "eu", &candidates).expect("tie is resolvable");
        assert_eq!(chosen.cell_id, "cell-alpha");
    }

    #[test]
    fn select_refuses_the_home_cell_as_its_own_dr_cell() {
        let candidates = vec![candidate("cell-home", "dc-1", 0)];
        let error = select_dr_cell("cell-home", "dc-1", "eu", &candidates)
            .expect_err("the home cell is never its own DR cell");
        assert_eq!(error, DrPairingError::NoEligibleDrCell);
    }

    #[test]
    fn select_refuses_a_cross_jurisdiction_candidate() {
        let mut other = candidate("cell-us", "dc-9", 0);
        other.jurisdiction = "us".to_owned();
        let error = select_dr_cell("cell-home", "dc-1", "eu", &[other])
            .expect_err("residency is a hard filter");
        assert_eq!(error, DrPairingError::NoEligibleDrCell);
    }

    #[test]
    fn select_refuses_an_unhealthy_candidate() {
        let mut sick = candidate("cell-sick", "dc-2", 0);
        sick.healthy = false;
        let error = select_dr_cell("cell-home", "dc-1", "eu", &[sick])
            .expect_err("health is a hard filter");
        assert_eq!(error, DrPairingError::NoEligibleDrCell);
    }

    #[test]
    fn next_version_refuses_to_wrap() {
        assert_eq!(next_version(7), Ok(8));
        assert_eq!(
            next_version(u32::MAX),
            Err(DrPairingError::PairVersionExhausted)
        );
    }

    #[test]
    fn idempotency_key_is_deterministic_and_field_framed() {
        let a = derive_idempotency_key("ten_a", "promoted", 4);
        assert_eq!(a, derive_idempotency_key("ten_a", "promoted", 4));
        assert_ne!(a, derive_idempotency_key("ten_a", "promoted", 5));
        // Framing keeps ("ab","c") from colliding with ("a","bc").
        assert_ne!(
            derive_idempotency_key("ab", "c", 1),
            derive_idempotency_key("a", "bc", 1)
        );
    }

    #[test]
    fn only_four_transitions_are_legal() {
        let states = [
            PairState::Planned,
            PairState::HomeActive,
            PairState::DrActive,
        ];
        let legal: Vec<(PairState, PairState)> = states
            .iter()
            .flat_map(|from| states.iter().map(move |to| (*from, *to)))
            .filter(|(from, to)| is_legal_transition(*from, *to))
            .collect();
        assert_eq!(
            legal,
            vec![
                (PairState::Planned, PairState::HomeActive),
                (PairState::HomeActive, PairState::Planned),
                (PairState::HomeActive, PairState::DrActive),
                (PairState::DrActive, PairState::HomeActive),
            ]
        );
        // A pair serving from DR is never re-cabled back to a plan.
        assert!(!is_legal_transition(
            PairState::DrActive,
            PairState::Planned
        ));
    }

    #[test]
    fn only_a_planned_pair_is_reassigned_in_place() {
        assert!(permits_in_place_reassignment(PairState::Planned));
        assert!(!permits_in_place_reassignment(PairState::HomeActive));
        assert!(!permits_in_place_reassignment(PairState::DrActive));
    }

    #[test]
    fn only_a_promotion_event_records_an_eligible_decision() {
        let kinds = [
            DrPairEventKind::PairAssigned,
            DrPairEventKind::PairReplanned,
            DrPairEventKind::PairActivated,
            DrPairEventKind::Promoted,
            DrPairEventKind::Restored,
        ];
        for kind in kinds {
            let decision = decision_for_event(kind);
            assert!(decision.is_well_formed(), "{} is explainable", kind.label());
            assert_eq!(
                decision == PromotionDecision::Eligible,
                kind == DrPairEventKind::Promoted,
                "only a promotion establishes eligibility"
            );
        }
        assert_eq!(
            decision_for_event(DrPairEventKind::Restored),
            PromotionDecision::Blocked {
                reason_code: reason::ALREADY_PROMOTED
            }
        );
        assert_eq!(
            decision_for_event(DrPairEventKind::PairActivated),
            PromotionDecision::Blocked {
                reason_code: reason::PAIR_NOT_ACTIVATED
            }
        );
    }

    #[test]
    fn every_event_kind_names_one_legal_edge() {
        let kinds = [
            DrPairEventKind::PairAssigned,
            DrPairEventKind::PairReplanned,
            DrPairEventKind::PairActivated,
            DrPairEventKind::Promoted,
            DrPairEventKind::Restored,
        ];
        for kind in kinds {
            let (from, to) = kind.edge();
            if kind == DrPairEventKind::PairAssigned {
                // The one self-edge: an assignment rewrites a plan.
                assert_eq!((from, to), (PairState::Planned, PairState::Planned));
            } else {
                assert!(
                    is_legal_transition(from, to),
                    "{} records an edge the state machine refuses",
                    kind.label()
                );
            }
        }
        // The edge is what lets a lost event be rebuilt from the stored
        // state, so no two kinds may share one `to_state` ambiguously.
        assert_ne!(
            DrPairEventKind::PairAssigned.edge().0,
            DrPairEventKind::PairReplanned.edge().0
        );
    }

    #[test]
    fn state_machine_blocks_promotion_from_planned_and_dr_active() {
        assert_eq!(promotion_block_for_state(PairState::HomeActive), None);
        assert_eq!(
            promotion_block_for_state(PairState::Planned),
            Some(reason::PAIR_NOT_ACTIVATED)
        );
        assert_eq!(
            promotion_block_for_state(PairState::DrActive),
            Some(reason::ALREADY_PROMOTED)
        );
    }

    #[test]
    fn a_fully_loaded_valid_candidate_still_scores_above_zero() {
        // 100 is the highest load `validate` accepts: 1000 - 100*4 = 600,
        // same fault domain, no bonus. This is plain subtraction, not
        // saturation — see the next test for the saturating branch.
        let hot = candidate("cell-hot", "dc-1", 100);
        assert_eq!(score_candidate("dc-1", &hot), 600);
        assert_eq!(score_candidate("dc-2", &hot), 1100);
    }

    #[test]
    fn load_penalty_saturates_at_zero_rather_than_underflowing() {
        // `score_candidate` is a total function over any candidate value,
        // including one the catalog should never have served. At 250% load
        // the penalty is 1000 and at 255% it exceeds the base; both floor
        // at 0 instead of wrapping to a near-`u32::MAX` score, which would
        // make the worst candidate in the fleet the winner.
        let mut absurd = candidate("cell-absurd", "dc-1", 250);
        assert_eq!(score_candidate("dc-1", &absurd), 0);
        absurd.load_percent = 255;
        assert_eq!(score_candidate("dc-1", &absurd), 0);
        // The bonus still applies on top of the floor, and the candidate is
        // rejected outright by the eligibility filter regardless.
        assert_eq!(score_candidate("dc-2", &absurd), 500);
        assert!(!is_eligible_candidate("cell-home", "eu", &absurd));
    }
}
