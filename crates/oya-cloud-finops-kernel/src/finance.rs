//! Finance primitives for P11-finance-library (merge-variant: added to existing crate).
//!
//! Provides `Currency` (ISO 4217 subset, KRW-first), `JournalEntryStatus`, and
//! `LedgerError` — the smallest net-new types not already present in any live crate.
//! Zero external dependencies: std only.
//!
//! Variant decision: F-M02B-PLAN-LIVE-CRATE-RECONCILIATION mandates merge into
//! existing crates rather than scaffolding new ones (user-directive-option-2,
//! 2026-05-17).

/// ISO 4217 currency code subset.
///
/// KRW is listed first as the primary platform currency (Korean Won).
/// Compile-time exhaustive enum — no `Other(String)` escape hatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum Currency {
    /// Korean Won — primary platform currency.
    KRW,
    /// US Dollar.
    USD,
    /// Euro.
    EUR,
    /// Japanese Yen.
    JPY,
    /// British Pound Sterling.
    GBP,
    /// Chinese Yuan Renminbi.
    CNY,
    /// Australian Dollar.
    AUD,
    /// Canadian Dollar.
    CAD,
    /// Swiss Franc.
    CHF,
    /// Hong Kong Dollar.
    HKD,
    /// Singapore Dollar.
    SGD,
    /// Swedish Krona.
    SEK,
    /// Norwegian Krone.
    NOK,
    /// Danish Krone.
    DKK,
    /// New Zealand Dollar.
    NZD,
    /// Indian Rupee.
    INR,
    /// Brazilian Real.
    BRL,
    /// South African Rand.
    ZAR,
    /// Mexican Peso.
    MXN,
    /// UAE Dirham.
    AED,
    /// Saudi Riyal.
    SAR,
}

impl Currency {
    /// Returns the ISO 4217 alphabetic code as a `&str`.
    pub fn code(self) -> &'static str {
        match self {
            Self::KRW => "KRW",
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::JPY => "JPY",
            Self::GBP => "GBP",
            Self::CNY => "CNY",
            Self::AUD => "AUD",
            Self::CAD => "CAD",
            Self::CHF => "CHF",
            Self::HKD => "HKD",
            Self::SGD => "SGD",
            Self::SEK => "SEK",
            Self::NOK => "NOK",
            Self::DKK => "DKK",
            Self::NZD => "NZD",
            Self::INR => "INR",
            Self::BRL => "BRL",
            Self::ZAR => "ZAR",
            Self::MXN => "MXN",
            Self::AED => "AED",
            Self::SAR => "SAR",
        }
    }

    /// Returns the ISO 4217 numeric code.
    pub fn numeric(self) -> u16 {
        match self {
            Self::KRW => 410,
            Self::USD => 840,
            Self::EUR => 978,
            Self::JPY => 392,
            Self::GBP => 826,
            Self::CNY => 156,
            Self::AUD => 36,
            Self::CAD => 124,
            Self::CHF => 756,
            Self::HKD => 344,
            Self::SGD => 702,
            Self::SEK => 752,
            Self::NOK => 578,
            Self::DKK => 208,
            Self::NZD => 554,
            Self::INR => 356,
            Self::BRL => 986,
            Self::ZAR => 710,
            Self::MXN => 484,
            Self::AED => 784,
            Self::SAR => 682,
        }
    }

    /// Returns the number of minor unit decimal places per ISO 4217.
    /// KRW and JPY have 0 decimal places; most others have 2.
    pub fn decimal_places(self) -> u8 {
        match self {
            Self::KRW | Self::JPY => 0,
            _ => 2,
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

// ─── JournalEntryStatus ───────────────────────────────────────────────────────

/// Lifecycle status of a double-entry journal entry.
///
/// State machine:
/// ```text
/// Draft → Pending → Posted
///   │                  │
///   └──────────────→ Voided
/// ```
/// `Voided` is a terminal state; `Posted` entries cannot transition back.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum JournalEntryStatus {
    /// Entry is being constructed; invariants not yet validated.
    Draft,
    /// Entry is validated and awaiting approval/posting.
    Pending,
    /// Entry has been permanently recorded in the ledger.
    Posted,
    /// Entry has been voided (creates a reversing entry in practice).
    Voided,
}

impl JournalEntryStatus {
    /// Returns the canonical string label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Posted => "posted",
            Self::Voided => "voided",
        }
    }

    /// Returns `true` if this is a terminal state (no further transitions allowed).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Posted | Self::Voided)
    }

    /// Returns valid successor states from this state.
    pub fn allowed_transitions(self) -> &'static [JournalEntryStatus] {
        match self {
            Self::Draft => &[Self::Pending, Self::Voided],
            Self::Pending => &[Self::Posted, Self::Voided],
            Self::Posted => &[],
            Self::Voided => &[],
        }
    }
}

impl std::fmt::Display for JournalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ─── LedgerError ─────────────────────────────────────────────────────────────

/// Domain errors for finance/ledger operations in FinOps context.
///
/// Distinct from `FinopsError` (cost-report/recommendation domain) — this
/// covers double-entry accounting primitives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerError {
    /// Debit total ≠ credit total in a journal entry.
    JournalUnbalanced {
        /// Total debit amount in minor units (e.g. cents).
        debits_minor: i128,
        /// Total credit amount in minor units.
        credits_minor: i128,
    },
    /// Arithmetic attempted across incompatible currencies.
    CrossCurrencyArithmetic {
        /// Left-hand currency.
        lhs: Currency,
        /// Right-hand currency.
        rhs: Currency,
    },
    /// Journal entry has fewer than two lines.
    InsufficientLines,
    /// A journal line amount is negative (sign is encoded by debit/credit side).
    NegativeAmount,
    /// Attempted state transition is not permitted by the status machine.
    InvalidStatusTransition {
        from: JournalEntryStatus,
        to: JournalEntryStatus,
    },
    /// Entry already exists in the ledger (duplicate id).
    DuplicateEntryId(String),
}

impl LedgerError {
    /// Human-readable error message.
    pub fn message(&self) -> String {
        match self {
            Self::JournalUnbalanced {
                debits_minor,
                credits_minor,
            } => {
                format!("journal entry unbalanced: debits {debits_minor} ≠ credits {credits_minor}")
            }
            Self::CrossCurrencyArithmetic { lhs, rhs } => {
                format!("cross-currency arithmetic: {lhs} and {rhs} are incompatible")
            }
            Self::InsufficientLines => "journal entry must have at least two lines".to_owned(),
            Self::NegativeAmount => "journal line amount must be non-negative".to_owned(),
            Self::InvalidStatusTransition { from, to } => {
                format!("invalid journal status transition: {from} → {to}")
            }
            Self::DuplicateEntryId(id) => {
                format!("journal entry id already exists in ledger: {id}")
            }
        }
    }
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

/// Validates a proposed `JournalEntryStatus` transition.
///
/// Returns `Ok(())` if `to` is in `from.allowed_transitions()`,
/// otherwise `Err(LedgerError::InvalidStatusTransition { from, to })`.
pub fn validate_status_transition(
    from: JournalEntryStatus,
    to: JournalEntryStatus,
) -> Result<(), LedgerError> {
    if from.allowed_transitions().contains(&to) {
        Ok(())
    } else {
        Err(LedgerError::InvalidStatusTransition { from, to })
    }
}

#[cfg(test)]
mod tests {
    #![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
    use super::*;

    // ─── Currency ─────────────────────────────────────────────────────────────

    #[test]
    fn currency_codes_are_distinct() {
        use std::collections::HashSet;
        let all = [
            Currency::KRW,
            Currency::USD,
            Currency::EUR,
            Currency::JPY,
            Currency::GBP,
            Currency::CNY,
            Currency::AUD,
            Currency::CAD,
            Currency::CHF,
            Currency::HKD,
            Currency::SGD,
            Currency::SEK,
            Currency::NOK,
            Currency::DKK,
            Currency::NZD,
            Currency::INR,
            Currency::BRL,
            Currency::ZAR,
            Currency::MXN,
            Currency::AED,
            Currency::SAR,
        ];
        let codes: HashSet<_> = all.iter().map(|c| c.code()).collect();
        assert_eq!(
            codes.len(),
            all.len(),
            "all currency codes must be distinct"
        );
    }

    #[test]
    fn currency_numeric_codes_are_nonzero() {
        let all = [
            Currency::KRW,
            Currency::USD,
            Currency::EUR,
            Currency::JPY,
            Currency::GBP,
            Currency::CNY,
            Currency::AUD,
            Currency::CAD,
            Currency::CHF,
            Currency::HKD,
            Currency::SGD,
            Currency::SEK,
            Currency::NOK,
            Currency::DKK,
            Currency::NZD,
            Currency::INR,
            Currency::BRL,
            Currency::ZAR,
            Currency::MXN,
            Currency::AED,
            Currency::SAR,
        ];
        for c in all {
            assert!(c.numeric() > 0, "{c} must have nonzero numeric code");
        }
    }

    #[test]
    fn krw_jpy_have_zero_decimal_places() {
        assert_eq!(Currency::KRW.decimal_places(), 0);
        assert_eq!(Currency::JPY.decimal_places(), 0);
    }

    #[test]
    fn usd_eur_have_two_decimal_places() {
        assert_eq!(Currency::USD.decimal_places(), 2);
        assert_eq!(Currency::EUR.decimal_places(), 2);
    }

    #[test]
    fn currency_display_matches_code() {
        assert_eq!(format!("{}", Currency::KRW), "KRW");
        assert_eq!(format!("{}", Currency::USD), "USD");
    }

    // ─── JournalEntryStatus ──────────────────────────────────────────────────

    #[test]
    fn draft_can_transition_to_pending_or_voided() {
        assert!(
            validate_status_transition(JournalEntryStatus::Draft, JournalEntryStatus::Pending)
                .is_ok()
        );
        assert!(
            validate_status_transition(JournalEntryStatus::Draft, JournalEntryStatus::Voided)
                .is_ok()
        );
    }

    #[test]
    fn draft_cannot_transition_to_posted_directly() {
        assert!(matches!(
            validate_status_transition(JournalEntryStatus::Draft, JournalEntryStatus::Posted),
            Err(LedgerError::InvalidStatusTransition { .. })
        ));
    }

    #[test]
    fn pending_can_transition_to_posted_or_voided() {
        assert!(
            validate_status_transition(JournalEntryStatus::Pending, JournalEntryStatus::Posted)
                .is_ok()
        );
        assert!(
            validate_status_transition(JournalEntryStatus::Pending, JournalEntryStatus::Voided)
                .is_ok()
        );
    }

    #[test]
    fn posted_is_terminal() {
        assert!(JournalEntryStatus::Posted.is_terminal());
        assert!(matches!(
            validate_status_transition(JournalEntryStatus::Posted, JournalEntryStatus::Draft),
            Err(LedgerError::InvalidStatusTransition { .. })
        ));
    }

    #[test]
    fn voided_is_terminal() {
        assert!(JournalEntryStatus::Voided.is_terminal());
        assert!(matches!(
            validate_status_transition(JournalEntryStatus::Voided, JournalEntryStatus::Draft),
            Err(LedgerError::InvalidStatusTransition { .. })
        ));
    }

    #[test]
    fn status_labels_are_distinct() {
        use std::collections::HashSet;
        let labels: HashSet<_> = [
            JournalEntryStatus::Draft,
            JournalEntryStatus::Pending,
            JournalEntryStatus::Posted,
            JournalEntryStatus::Voided,
        ]
        .iter()
        .map(|s| s.label())
        .collect();
        assert_eq!(labels.len(), 4);
    }

    // ─── LedgerError ─────────────────────────────────────────────────────────

    #[test]
    fn ledger_error_messages_are_non_empty() {
        let errors = [
            LedgerError::JournalUnbalanced {
                debits_minor: 1000,
                credits_minor: 900,
            },
            LedgerError::CrossCurrencyArithmetic {
                lhs: Currency::USD,
                rhs: Currency::KRW,
            },
            LedgerError::InsufficientLines,
            LedgerError::NegativeAmount,
            LedgerError::InvalidStatusTransition {
                from: JournalEntryStatus::Posted,
                to: JournalEntryStatus::Draft,
            },
            LedgerError::DuplicateEntryId("je-001".to_owned()),
        ];
        for e in &errors {
            assert!(
                !e.message().is_empty(),
                "error message must not be empty: {e:?}"
            );
        }
    }

    #[test]
    fn cross_currency_error_contains_both_codes() {
        let e = LedgerError::CrossCurrencyArithmetic {
            lhs: Currency::USD,
            rhs: Currency::KRW,
        };
        let msg = e.message();
        assert!(msg.contains("USD"), "message must mention lhs: {msg}");
        assert!(msg.contains("KRW"), "message must mention rhs: {msg}");
    }

    #[test]
    fn unbalanced_error_contains_amounts() {
        let e = LedgerError::JournalUnbalanced {
            debits_minor: 1500,
            credits_minor: 1000,
        };
        let msg = e.message();
        assert!(msg.contains("1500"), "message must mention debits: {msg}");
        assert!(msg.contains("1000"), "message must mention credits: {msg}");
    }

    // ─── Audit-chain append-only contract ────────────────────────────────────

    /// Regression guard for P1 Codex review PRRT_kwDOSbSl2s6CnoW2:
    /// `evidence/audit-chain.jsonl` is declared append-only; deleting historical
    /// records breaks provenance/replay guarantees.  This test asserts that the
    /// corrective_restore event is present (written when 12 deleted records were
    /// re-appended) and that a representative sample of the originally-deleted
    /// records are also present, catching any future deletion of those rows.
    #[test]
    fn audit_chain_is_append_only_and_corrective_restore_present() {
        // Locate the workspace root relative to CARGO_MANIFEST_DIR so this test
        // works from any working directory during `cargo nextest run`.
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // CARGO_MANIFEST_DIR is crates/oya-cloud-finops-kernel; workspace root is ../../
        let workspace_root = manifest_dir
            .parent()
            .expect("crates/")
            .parent()
            .expect("workspace root");
        let audit_chain = workspace_root.join("evidence").join("audit-chain.jsonl");

        let content = std::fs::read_to_string(&audit_chain)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", audit_chain.display()));

        // Every non-empty line must be valid JSON (basic JSONL well-formedness).
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            assert!(
                trimmed.starts_with('{') && trimmed.ends_with('}'),
                "audit-chain.jsonl line {} is not a JSON object: {}",
                i + 1,
                &trimmed[..trimmed.len().min(120)]
            );
        }

        // The corrective_restore event must be present — its existence proves
        // the 12 records deleted by the m02b-p11-finance-retry branch were
        // re-appended instead of left absent.
        assert!(
            content.contains("\"event_type\": \"corrective_restore\""),
            "audit-chain.jsonl must contain a corrective_restore event \
             (P1 fix for PRRT_kwDOSbSl2s6CnoW2)"
        );
        assert!(
            content.contains("P1-FIX-PR76-AUDIT-CHAIN-RESTORE"),
            "corrective_restore event must reference P1-FIX-PR76-AUDIT-CHAIN-RESTORE"
        );

        // Representative originally-deleted records must still be present.
        let required_fragments = [
            "\"M-CC-P06-IP-002-PHASE-2\"",
            "\"M-CC-P06-IP-002-FINAL\"",
            "\"ip_status_flip\"",
            "\"ADR-0092\"",
            "\"seam_audit_baseline\"",
            "\"doctrine_created\"",
            "\"doc_antipattern_audit\"",
            "DIRECTIVE-CONSENSUS-DEBATE-SUBAGENT-LENS-2026-05-14",
            "MULTISPECTRUM-V2.3-A-FAMILY-2026-05-15",
        ];
        for fragment in required_fragments {
            assert!(
                content.contains(fragment),
                "audit-chain.jsonl must contain originally-deleted record \
                 matching {fragment:?} — append-only contract violated"
            );
        }
    }
}
