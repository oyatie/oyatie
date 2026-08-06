# Spec: DMARC Identifier-Alignment Verdict

**Task slug:** mail-domain-dmarc-alignment-verdict  
**Vertical:** mail  
**Crate:** oya-mail-domain  
**Module:** crates/oya-mail-domain/src/governance.rs  
**RFC authority:** RFC 7489 §3 (DMARC), RFC 7208 (SPF alignment), RFC 6376 (DKIM alignment)  

---

## Objective

The existing `DmarcVerdict::new` derives `DmarcAction` from raw SPF/DKIM pass booleans. RFC 7489 §3 requires that the authenticated identifier (the domain in the SPF `Return-Path` or DKIM `d=` tag) be *aligned* with the RFC 5322 `From:` domain before a pass counts toward DMARC. A forwarded message may carry a raw SPF pass for a relay domain that is not aligned with `From:`, producing a false Accept under the current logic.

This spec defines the alignment-aware extension:

1. **ST1** — Add `DmarcVerdict::new_aligned` taking explicit `spf_aligned`/`dkim_aligned` inputs; rewrite `DmarcVerdict::new` as a thin back-compat wrapper.
2. **ST2** — Surface `report_only: Classified<bool>` so `p=none` non-aligned messages are `Accept` but explicitly flagged for monitoring.

---

## Vertical and Module Layout

```
crates/oya-mail-domain/
  src/
    lib.rs                         # pub mod governance; pub use governance::*;
    governance.rs                  # ONLY file modified by this task
    sending_domain_authentication.rs
    thread_state.rs
```

Flat clean-arch: all domain logic lives as top-level items or `impl` blocks inside `governance.rs`. No new modules, no new crates.

---

## Data Model Changes

### DmarcVerdict (extended)

```rust
pub struct DmarcVerdict {
    pub domain_ref:   Classified<String>,     // DataClass::InternalOnly
    pub action:       Classified<DmarcAction>,// DataClass::InternalOnly
    pub report_only:  Classified<bool>,       // DataClass::InternalOnly  (NEW ST2)
    pub evidence_ref: Classified<String>,     // DataClass::Audit         (unchanged)
}
```

`report_only = true` iff `policy == DmarcPolicy::None && !(spf_aligned || dkim_aligned)`.

### New constructor (ST1)

```rust
impl DmarcVerdict {
    /// RFC 7489-compliant: derives action from identifier-aligned pass results.
    pub fn new_aligned(
        domain_ref:   String,
        spf_aligned:  bool,
        dkim_aligned: bool,
        policy:       DmarcPolicy,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError>;

    /// Back-compat wrapper: treats raw pass as aligned (pre-RFC-7489 callers).
    pub fn new(
        domain_ref:   String,
        spf:          bool,
        dkim:         bool,
        policy:       DmarcPolicy,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError>;
}
```

`DmarcVerdict::new(d, spf, dkim, policy, ev)` calls `new_aligned(d, spf, dkim, policy, ev)` unchanged — existing callers observe no behavioral difference for messages where raw == aligned.

---

## Action Derivation Table

| `spf_aligned || dkim_aligned` | `policy`               | `action`    | `report_only` |
|-------------------------------|------------------------|-------------|---------------|
| `true`                        | any                    | Accept      | false         |
| `false`                       | `DmarcPolicy::None`    | Accept      | **true**      |
| `false`                       | `DmarcPolicy::Quarantine` | Quarantine | false        |
| `false`                       | `DmarcPolicy::Reject`  | Reject      | false         |

---

## Data Classification

| Field        | Class              | Rationale |
|--------------|--------------------|-----------|
| `domain_ref` | `InternalOnly`     | Operational routing identifier |
| `action`     | `InternalOnly`     | Enforcement decision |
| `report_only`| `InternalOnly`     | Monitoring flag, operational |
| `evidence_ref`| `DataClass::Audit`| Audit trail per existing pattern |

`report_only` is `InternalOnly` (not `Audit`) because it is a monitoring-mode flag, not itself an audit record. The `evidence_ref` carries the audit reference.

---

## Testing Strategy

All tests live in the `#[cfg(test)] mod tests` block at the bottom of `governance.rs`.

### ST1 tests

| Test name | Scenario | Expected |
|-----------|----------|----------|
| `dmarc_aligned_false_reject_policy_rejects` | `spf_aligned=false, dkim_aligned=false, policy=Reject` | `DmarcAction::Reject` |
| `dmarc_aligned_false_quarantine_policy_quarantines` | `spf_aligned=false, dkim_aligned=false, policy=Quarantine` | `DmarcAction::Quarantine` |
| `dmarc_aligned_spf_only_accepts` | `spf_aligned=true, dkim_aligned=false, policy=Reject` | `DmarcAction::Accept` |
| `dmarc_fail_quarantine_and_logged` (existing) | unchanged | `DmarcAction::Quarantine` (must still pass) |

### ST2 tests

| Test name | Scenario | Expected |
|-----------|----------|----------|
| `dmarc_none_policy_non_aligned_is_report_only` | `p=none, spf_aligned=false, dkim_aligned=false` | `Accept`, `report_only=true` |
| `dmarc_aligned_pass_not_report_only` | `p=none, spf_aligned=true` | `Accept`, `report_only=false` |
| `dmarc_reject_not_report_only` | `p=reject, aligned=false` | `Reject`, `report_only=false` |
| `dmarc_evidence_ref_is_audit_class` | any valid verdict | `evidence_ref.data_class == DataClass::Audit` |

---

## Boundaries and Constraints

- **No new crate.** This task extends one module in one existing crate.
- **No root `Cargo.toml` edit.** `oya-mail-domain` depends only on `oya-data-boundary-kernel`.
- **No new dependency.** All logic is pure domain computation.
- **Backward compatibility.** `DmarcVerdict::new` signature is preserved; callers do not break.
- **No adapter/REST/gRPC layer.** This is pure domain logic; no HTTP/proto changes in scope.

---

## OpenAPI / Proto Note

`DmarcVerdict` is a domain aggregate, not yet exposed via a REST or gRPC surface in this vertical. When an inbound SMTP adapter is added, it will map to an internal API response. No OpenAPI 3.2.0 or proto3 schema change is required by this task.

---

## References

- RFC 7489 §3.1 — Identifier Alignment
- RFC 7208 §2.6 — SPF Result Codes
- RFC 6376 §3.5 — DKIM `d=` tag
- `crates/oya-mail-domain/src/governance.rs` — current implementation
- `docs/adr-archive/ADR-0130-deprecate-knowledge-graph-registry-file-migrate-to-ontology.md` — SLO gate (no SLO change this task)
