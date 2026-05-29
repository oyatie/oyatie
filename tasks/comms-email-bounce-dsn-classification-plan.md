# Plan: comms-email-bounce-dsn-classification

**Lane:** mail  
**Crate:** `oya-shared-email-comms-kernel`  
**Branch:** `feat/task-comms-email-bounce-dsn-classification-2026-05-28`

---

## Objective

Extend the email comms kernel with deterministic bounce classification and a
pure suppression/retry decision. The existing `DeliveryEventKind::Bounced`
variant carries no category signal; this task adds a typed `BounceCategory`
enum, a pure classifier mapping RFC 3463 enhanced status codes and RFC 5321
SMTP reply codes into Hard / Soft / Transient buckets, and a
`bounce_suppression_decision` function that decides whether a recipient is
promoted to the existing `RecipientSuppressed` suppression seam or eligible
for retry. No I/O, no persistence, no webhook parsing — pure kernel logic.

---

## Subtasks

### ST1 — bounce-category-enum

**What:** Introduce `BounceCategory { Hard, Soft, Transient }` and a pure
`classify_bounce` function that maps:

- RFC 3463 enhanced status code strings (`"5.x.x"` → `Hard`, `"4.x.x"` →
  `Transient`, with soft-bounce sub-class discrimination)
- RFC 5321 SMTP numeric codes (5xx → `Hard`, 4xx → `Transient`/`Soft`)

into a `BounceCategory`. The existing `DeliveryEventKind::Bounced` wire
vocabulary and its `Display` output (`"bounced"`) are not altered.

**New public surface (pure functions + enum, no I/O):**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BounceCategory {
    Hard,
    Soft,
    Transient,
}

/// Classify a bounce from an RFC 3463 enhanced status code string
/// (e.g. "5.1.1", "4.2.2").  Returns None if the input is not a
/// well-formed enhanced status code.
pub fn classify_bounce_enhanced(code: &str) -> Option<BounceCategory>;

/// Classify a bounce from an RFC 5321 SMTP numeric reply code.
/// Returns None for codes outside the 4xx/5xx range.
pub fn classify_bounce_smtp(code: u16) -> Option<BounceCategory>;
```

**Acceptance:**
- `cargo check -p oya-shared-email-comms-kernel --all-targets` clean.
- Unit tests assert:
  - `"5.1.1"` → `Hard`, `"5.0.0"` → `Hard`, `"5.7.1"` → `Hard`
  - `"4.2.2"` → `Transient`, `"4.4.7"` → `Transient`
  - `"4.2.1"` → `Soft` (mailbox temporarily unavailable sub-class)
  - SMTP `550` → `Hard`, `421` → `Transient`, `452` → `Soft`
  - `DeliveryEventKind::Bounced.to_string()` → `"bounced"` (unchanged)

---

### ST2 — suppression-decision

**What:** Add a pure `bounce_suppression_decision` function that, given a
`BounceCategory` and `prior_soft_bounce_count: u32`, returns a
`BounceSuppressionOutcome`. Wire `Suppress` into the existing
`RecipientSuppressed` suppression seam.

**New public surface:**

```rust
/// Outcome of evaluating whether to suppress or retry after a bounce.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BounceSuppressionOutcome {
    /// Add recipient to the suppression list immediately.
    Suppress,
    /// Eligible for retry; record the soft-bounce count and try again.
    Retry,
    /// No action needed (e.g. transient, below threshold).
    NoAction,
}

/// Soft-bounce count at which repeated soft bounces are promoted to
/// suppression.
pub const SOFT_BOUNCE_SUPPRESS_THRESHOLD: u32 = 3;

/// Pure decision: given the bounce category and how many soft bounces this
/// recipient has already accumulated, decide whether to suppress, retry, or
/// take no action.
///
/// Rules:
/// - Hard → Suppress (always)
/// - Soft, prior_soft >= SOFT_BOUNCE_SUPPRESS_THRESHOLD → Suppress
/// - Soft, prior_soft <  SOFT_BOUNCE_SUPPRESS_THRESHOLD → Retry
/// - Transient → NoAction
pub fn bounce_suppression_decision(
    category: BounceCategory,
    prior_soft_bounce_count: u32,
) -> BounceSuppressionOutcome;
```

**Acceptance:**
- `cargo nextest run -p oya-shared-email-comms-kernel` green.
- Tests cover:
  - `Hard` → `Suppress`
  - `Soft`, `prior=0` → `Retry`
  - `Soft`, `prior=SOFT_BOUNCE_SUPPRESS_THRESHOLD` → `Suppress`
  - `Transient` → `NoAction`
  - `enforce_deliverability_invariants` existing tests unchanged.

---

## Acceptance (combined)

1. `cargo check -p oya-shared-email-comms-kernel --all-targets` → zero errors,
   zero warnings (beyond pre-existing allowed lints).
2. `cargo nextest run -p oya-shared-email-comms-kernel` → all tests green.
3. `Cargo.toml [dependencies]` remains empty.
4. `DeliveryEventKind::Bounced.to_string()` still returns `"bounced"`.
5. No root `Cargo.toml` touched.
6. No other crate touched.

---

## Boundaries

- Modify: `crates/oya-shared-email-comms-kernel/src/lib.rs` only.
- Do not touch: root `Cargo.toml`, any other crate, any other file.
- Do not add any dependency to `Cargo.toml [dependencies]`.
- Do not alter `DeliveryEventKind` variants or their `Display` output.
- Do not add I/O, async, or persistence concerns.
