# Plan: comms-kernel-rate-ceiling-idempotency-preflight

**Lane:** mail  
**Crate:** `oya-shared-email-comms-kernel`  
**Branch:** `feat/task-comms-kernel-rate-ceiling-idempotency-preflight-2026-05-28`

---

## Objective

Enforce the two send-time invariants promised by ADR-0149 (idempotency) and
ADR-0201 (uniform per-tenant rate ceilings) that are currently declared on
`EmailCommsError` but never produced anywhere in the kernel. Both checks are
pure, dep-free extensions to `enforce_deliverability_invariants` (or a sibling
it delegates to).

---

## Subtasks

### ST1 — Per-tenant per-minute rate ceiling

**What:** Extend `enforce_deliverability_invariants` to accept a caller-supplied
`recent_send_count: u32` and `rate_ceiling: u32`. When
`recent_send_count >= rate_ceiling` the function must emit
`EmailCommsError::RateCeilingExceeded { tenant, per_minute }` where
`per_minute == rate_ceiling`.

**Signature change (additive):**

```rust
pub fn enforce_deliverability_invariants(
    binding: &DeliverabilityBinding,
    message: &OutboundMessage,
    suppressed: &[EmailAddress],
    warm_up_complete: bool,
    recent_send_count: u32,   // new
    rate_ceiling: u32,        // new (0 = no ceiling)
) -> Result<(), EmailCommsError>
```

A `rate_ceiling` of `0` means uncapped (callers that do not have a ceiling pass
`0`; existing adapter call-sites updated accordingly).

**Acceptance:**
- `cargo check -p oya-shared-email-comms-kernel --all-targets` clean.
- New unit test: send at ceiling → `RateCeilingExceeded` with correct tenant +
  `per_minute`; send at ceiling-1 → `Ok`.
- All existing `preflight_*` tests still pass (call sites updated to pass
  `0, 0`).

---

### ST2 — Idempotency-key conflict detection

**What:** Add a `prior_fingerprints: &std::collections::HashMap<String, u64>`
parameter to `enforce_deliverability_invariants` (or an extracted
`enforce_idempotency` helper called by it). The kernel derives a stable
`u64` fingerprint over `(from, to[0], subject, html_body)` using a
dependency-free FNV-1a fold. Rules:

| Prior record for key? | Fingerprints match? | Outcome |
|---|---|---|
| No | — | `Ok(())` — fresh key |
| Yes | Yes | `Ok(())` — identical re-send collapsed |
| Yes | No | `Err(IdempotencyConflict { key })` |

**Acceptance:**
- `cargo nextest run -p oya-shared-email-comms-kernel` passes.
- Three new unit tests covering the three cases above.
- `[dependencies]` in `Cargo.toml` stays empty.

---

## Acceptance (combined)

1. `cargo check -p oya-shared-email-comms-kernel --all-targets` → zero errors,
   zero warnings (beyond pre-existing allowed lints).
2. `cargo nextest run -p oya-shared-email-comms-kernel` → all tests green.
3. `Cargo.toml [dependencies]` remains empty.
4. No root `Cargo.toml` touched.
5. No other crate touched.

---

## Boundaries

- This task modifies **only** `crates/oya-shared-email-comms-kernel/src/lib.rs`.
- No new files, no new crates, no new workspace members.
- The four adapter shells (`SesEmailComms`, `PostalEmailComms`,
  `MailgunEmailComms`, `SmtpEmailComms`) are updated only to thread the new
  parameters through to `enforce_deliverability_invariants`; their behavior
  outside preflight is unchanged.
