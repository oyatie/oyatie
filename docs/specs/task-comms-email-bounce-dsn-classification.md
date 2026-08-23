# Spec: comms-email-bounce-dsn-classification

**Vertical:** mail  
**Crate:** `shared-email-comms-kernel`  
**ADRs:** ADR-0201 (email substrate), ADR-0145 (audit chain events), ADR-0173 (vendor lock-in avoidance)  
**Stage:** SPEC → IMPL → VERIFY

---

## Objective

The email comms kernel currently exposes a single flat
`DeliveryEventKind::Bounced` variant with no category signal. This leaves the
suppression seam unable to distinguish a permanent mailbox-does-not-exist
failure (RFC 3463 `5.1.1`) from a temporary quota-full transient (RFC 3463
`4.2.2`). This task adds:

1. A typed `BounceCategory` enum (Hard / Soft / Transient) and two pure
   classifier functions mapping RFC 3463 enhanced status codes and RFC 5321
   SMTP reply codes into it.
2. A pure `bounce_suppression_decision` function that, given a `BounceCategory`
   plus an accumulated soft-bounce count, returns a `BounceSuppressionOutcome`
   (Suppress / Retry / NoAction) and wires `Suppress` into the existing
   `RecipientSuppressed` error path.

No webhook parsing, no I/O, no async, no persistence — pure kernel decision
logic. The existing `DeliveryEventKind::Bounced` wire vocabulary and its
`Display` output (`"bounced"`) are left entirely unchanged.

---

## Vertical context

`shared-email-comms-kernel` is the port-in-kernel (ADR-0056) shared by
every email-sending µservice (Identity, Tenancy, Workflow Studio, Billing,
Audit, Foundry). It carries zero runtime dependencies so any layer can import
it without pulling in provider SDKs. The existing `enforce_deliverability_invariants`
function already enforces DKIM / SPF / DMARC / rate-ceiling / suppression at
pre-flight; the bounce classifier and suppression decision extend that
suppression surface post-delivery.

RFC references:
- **RFC 3463** — Enhanced Mail System Status Codes (`X.Y.Z` triples where X
  is the status class: 2 = success, 4 = transient failure, 5 = permanent
  failure).
- **RFC 5321** — Simple Mail Transfer Protocol (3-digit SMTP reply codes; 4xx
  = transient negative completion, 5xx = permanent negative completion).

---

## Module layout (flat clean-arch inside `src/`)

```
crates/shared-email-comms-kernel/
  Cargo.toml          ← unchanged; [dependencies] stays empty
  src/
    lib.rs            ← single flat file; new types + functions appended here
                        (no sub-modules; crate is still small enough to stay flat)
```

New items are appended inside `src/lib.rs` in declaration order:

1. `BounceCategory` enum + `Display`
2. `classify_bounce_enhanced(code: &str) -> Option<BounceCategory>`
3. `classify_bounce_smtp(code: u16) -> Option<BounceCategory>`
4. `BounceSuppressionOutcome` enum + `Display`
5. `SOFT_BOUNCE_SUPPRESS_THRESHOLD: u32` constant
6. `bounce_suppression_decision(category, prior_soft_bounce_count) -> BounceSuppressionOutcome`
7. New `#[cfg(test)]` cases appended to the existing `mod tests` block

---

## Contracts

### BounceCategory (ST1)

```rust
/// Bounce severity category derived from RFC 3463 enhanced status codes
/// or RFC 5321 SMTP reply codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BounceCategory {
    /// Permanent failure — recipient address is definitively unreachable.
    /// Drives immediate suppression. RFC 3463 class 5.x.x / SMTP 5xx.
    Hard,
    /// Repeated temporary failure indicating a persistent mailbox problem
    /// (e.g. over-quota, policy reject on the recipient side). Drives
    /// suppression after SOFT_BOUNCE_SUPPRESS_THRESHOLD occurrences.
    Soft,
    /// Short-lived transient failure — connection refused, greylisting,
    /// DNS momentarily unavailable. No action; eligible for retry by the
    /// calling µservice. RFC 3463 class 4.x.x (non-soft sub-classes) /
    /// SMTP 4xx (non-soft codes).
    Transient,
}
```

#### `classify_bounce_enhanced`

Parses an RFC 3463 enhanced status code string of the form `"X.Y.Z"` where X,
Y, Z are unsigned integers.

Classification rules:

| Status class | Sub-class condition | `BounceCategory` |
|---|---|---|
| `5` | any | `Hard` |
| `4` | subject sub-class `2`, detail `1` (`4.2.1`) | `Soft` |
| `4` | any other | `Transient` |
| other / malformed | — | `None` |

Rationale for `4.2.1` as `Soft`: RFC 3463 §3.3 assigns `X.2.1` to
"mailbox disabled, not accepting messages" — a persistent per-mailbox policy
condition rather than a transient infrastructure issue.

```rust
pub fn classify_bounce_enhanced(code: &str) -> Option<BounceCategory>
```

Returns `None` if `code` is not a well-formed `"X.Y.Z"` triple.

#### `classify_bounce_smtp`

Maps RFC 5321 3-digit SMTP reply codes.

| Code range | `BounceCategory` |
|---|---|
| 550–559 | `Hard` |
| 500–549, 560–599 | `Hard` |
| 452 | `Soft` (over-quota permanent-ish) |
| 400–499 (excluding 452) | `Transient` |
| other | `None` |

```rust
pub fn classify_bounce_smtp(code: u16) -> Option<BounceCategory>
```

Returns `None` for codes outside the 4xx–5xx range.

---

### BounceSuppressionOutcome + decision (ST2)

```rust
/// Outcome of the bounce suppression decision function.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BounceSuppressionOutcome {
    /// Caller must add the recipient to the suppression list.
    /// Maps to EmailCommsError::RecipientSuppressed at the adapter layer.
    Suppress,
    /// Eligible for retry. Caller should increment soft-bounce count and
    /// attempt re-delivery after a back-off interval.
    Retry,
    /// No suppression or retry action required (transient; infrastructure
    /// retry is sufficient).
    NoAction,
}

/// Number of accumulated soft bounces at which a recipient is promoted to
/// suppression.
pub const SOFT_BOUNCE_SUPPRESS_THRESHOLD: u32 = 3;

/// Pure decision: given the bounce category and the number of soft bounces
/// already recorded for this recipient, return the suppression outcome.
///
/// Decision table:
///
/// | category  | prior_soft_bounce_count          | outcome   |
/// |-----------|----------------------------------|-----------|
/// | Hard      | any                              | Suppress  |
/// | Soft      | >= SOFT_BOUNCE_SUPPRESS_THRESHOLD | Suppress  |
/// | Soft      | < SOFT_BOUNCE_SUPPRESS_THRESHOLD  | Retry     |
/// | Transient | any                              | NoAction  |
pub fn bounce_suppression_decision(
    category: BounceCategory,
    prior_soft_bounce_count: u32,
) -> BounceSuppressionOutcome
```

`Suppress` outcomes map to `EmailCommsError::RecipientSuppressed` at the
adapter layer; this kernel function is the pure decision; the adapter is
responsible for constructing and returning the error variant.

---

## Testing strategy

All tests live in the `#[cfg(test)] mod tests` block inside `src/lib.rs`.
New test cases are appended to the existing block.

### ST1: classify_bounce_enhanced

| Test | Input | Expected |
|---|---|---|
| `classify_enhanced_5xx_is_hard` | `"5.1.1"`, `"5.0.0"`, `"5.7.1"` | `Some(Hard)` |
| `classify_enhanced_4xx_is_transient` | `"4.2.2"`, `"4.4.7"` | `Some(Transient)` |
| `classify_enhanced_421_soft` | `"4.2.1"` | `Some(Soft)` |
| `classify_enhanced_malformed_is_none` | `"5.1"`, `"abc"`, `""` | `None` |
| `delivery_event_kind_bounced_display_unchanged` | `DeliveryEventKind::Bounced` | `"bounced"` |

### ST1: classify_bounce_smtp

| Test | Input | Expected |
|---|---|---|
| `classify_smtp_5xx_is_hard` | `550`, `521`, `554` | `Some(Hard)` |
| `classify_smtp_452_is_soft` | `452` | `Some(Soft)` |
| `classify_smtp_4xx_is_transient` | `421`, `450`, `451` | `Some(Transient)` |
| `classify_smtp_out_of_range_is_none` | `200`, `350`, `600` | `None` |

### ST2: bounce_suppression_decision

| Test | Category | prior_soft | Expected |
|---|---|---|---|
| `hard_bounce_suppresses` | `Hard` | 0 | `Suppress` |
| `single_soft_bounce_retries` | `Soft` | 0 | `Retry` |
| `soft_at_threshold_suppresses` | `Soft` | `SOFT_BOUNCE_SUPPRESS_THRESHOLD` | `Suppress` |
| `transient_no_action` | `Transient` | 0 | `NoAction` |

### Existing tests

All existing tests in `mod tests` must continue to pass without modification.
The `enforce_deliverability_invariants` signature and call sites are not changed
by this task.

---

## Boundaries

- Modify: `crates/shared-email-comms-kernel/src/lib.rs` only.
- Do not touch: root `Cargo.toml`, any other crate, any other file.
- Do not add any entry to `Cargo.toml [dependencies]`.
- Do not alter `DeliveryEventKind` variants or their `Display` output.
- Do not alter `enforce_deliverability_invariants` signature or behavior.
- Do not introduce async, I/O, or persistence of any kind.
- Do not parse raw MIME/DSN payloads — that is adapter/webhook territory.

---

## Definition of done

1. `cargo check -p shared-email-comms-kernel --all-targets` → clean.
2. `cargo nextest run -p shared-email-comms-kernel` → all tests pass.
3. `Cargo.toml [dependencies]` is empty.
4. `DeliveryEventKind::Bounced.to_string()` still returns `"bounced"`.
5. `BounceCategory`, `classify_bounce_enhanced`, `classify_bounce_smtp`,
   `BounceSuppressionOutcome`, `SOFT_BOUNCE_SUPPRESS_THRESHOLD`, and
   `bounce_suppression_decision` are all `pub` and reachable from callers.
