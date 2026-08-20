//! Foundation-bypass ledger kernel.
//!
//! Pure, in-memory append-only ledger (no I/O) for recording foundation-bypass
//! events with expiry tracking. Backed by `Vec<BypassEntry>`; persistent stores
//! live in adapter crates per Clean Architecture.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BypassEntry {
    pub id: String,                 // data_class: PUBLIC
    pub gate: String,               // data_class: PUBLIC
    pub agent: String,              // data_class: INTERNAL_ONLY
    pub rationale: String,          // data_class: INTERNAL_ONLY
    pub adr_citation: String,       // data_class: PUBLIC
    pub created_epoch_seconds: u64, // data_class: PUBLIC
    pub expires_epoch_seconds: u64, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerError {
    DuplicateId(String),
    EmptyGate,
    EmptyRationale,
    EmptyAdrCitation,
    ExpiryNotAfterCreation,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BypassLedger {
    entries: Vec<BypassEntry>,
}

impl BypassLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a bypass entry. Returns `Err` if validation fails.
    pub fn record(&mut self, entry: BypassEntry) -> Result<(), LedgerError> {
        if entry.gate.trim().is_empty() {
            return Err(LedgerError::EmptyGate);
        }
        if entry.rationale.trim().is_empty() {
            return Err(LedgerError::EmptyRationale);
        }
        if entry.adr_citation.trim().is_empty() {
            return Err(LedgerError::EmptyAdrCitation);
        }
        if entry.expires_epoch_seconds <= entry.created_epoch_seconds {
            return Err(LedgerError::ExpiryNotAfterCreation);
        }
        if self.entries.iter().any(|existing| existing.id == entry.id) {
            return Err(LedgerError::DuplicateId(entry.id.clone()));
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Return all stored entries in append order.
    pub fn replay(&self) -> Vec<BypassEntry> {
        self.entries.clone()
    }

    /// Return entries whose expiry is `<= now`.
    pub fn expired(&self, now_epoch_seconds: u64) -> Vec<BypassEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.expires_epoch_seconds <= now_epoch_seconds)
            .cloned()
            .collect()
    }

    /// Number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, created: u64, expires: u64) -> BypassEntry {
        BypassEntry {
            id: id.into(),
            gate: "claim-ceiling".into(),
            agent: "claude-m02-p03".into(),
            rationale: "scaffold per ADR-0054".into(),
            adr_citation: "ADR-0054".into(),
            created_epoch_seconds: created,
            expires_epoch_seconds: expires,
        }
    }

    #[test]
    fn records_and_replays_in_order() {
        let mut ledger = BypassLedger::new();
        ledger.record(entry("a", 100, 200)).unwrap();
        ledger.record(entry("b", 110, 210)).unwrap();
        let replayed = ledger.replay();
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].id, "a");
        assert_eq!(replayed[1].id, "b");
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut ledger = BypassLedger::new();
        ledger.record(entry("a", 100, 200)).unwrap();
        assert_eq!(
            ledger.record(entry("a", 150, 250)),
            Err(LedgerError::DuplicateId("a".into()))
        );
    }

    #[test]
    fn rejects_expiry_not_after_creation() {
        let mut ledger = BypassLedger::new();
        assert_eq!(
            ledger.record(entry("a", 200, 100)),
            Err(LedgerError::ExpiryNotAfterCreation)
        );
    }

    #[test]
    fn expired_returns_only_past_due_entries() {
        let mut ledger = BypassLedger::new();
        ledger.record(entry("a", 100, 200)).unwrap();
        ledger.record(entry("b", 100, 500)).unwrap();
        let expired = ledger.expired(300);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, "a");
    }

    #[test]
    fn rejects_empty_required_fields() {
        let mut ledger = BypassLedger::new();
        let mut e = entry("a", 100, 200);
        e.gate = String::new();
        assert_eq!(ledger.record(e), Err(LedgerError::EmptyGate));
    }
}
