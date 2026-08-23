# Spec: comms-kernel-rate-ceiling-idempotency-preflight

**Vertical:** mail  
**Crate:** `shared-email-comms-kernel`  
**ADRs:** ADR-0149 (idempotency), ADR-0201 (uniform per-tenant rate ceilings)  
**Stage:** SPEC → IMPL → VERIFY

---

## Objective

Two `EmailCommsError` variants — `RateCeilingExceeded` and
`IdempotencyConflict` — are declared but never produced. The kernel's
`enforce_deliverability_invariants` function therefore fails to uphold the
two core send-time invariants documented in ADR-0149 and ADR-0201. This task
adds the two missing checks as pure, dep-free extensions to that function so
that all four adapters (SES, Postal, Mailgun, SMTP) uniformly enforce both
invariants at pre-flight.

---

## Vertical context

`shared-email-comms-kernel` is the port-in-kernel (ADR-0056) shared by
every email-sending µservice. It deliberately carries zero runtime
dependencies so that any layer can import it without pulling in provider SDKs.
The kernel exposes the `EmailComms` trait; real provider SDK wiring lives in
µservice-side adapter feature flags.

---

## Module layout (flat clean-arch inside `src/`)

```
src/
  lib.rs          ← single file; all types, trait, invariant fn, adapter shells,
                    and #[cfg(test)] block live here (no sub-modules needed at
                    this crate size)
```

The two new checks are added **inside `enforce_deliverability_invariants`**
(rate ceiling) and via an inline helper `message_fingerprint` (idempotency).
No new files or modules are introduced.

---

## Contracts

### Rate-ceiling check (ST1)

The function signature gains two new trailing parameters:

```rust
pub fn enforce_deliverability_invariants(
    binding: &DeliverabilityBinding,
    message: &OutboundMessage,
    suppressed: &[EmailAddress],
    warm_up_complete: bool,
    recent_send_count: u32,   // sends already recorded in the current minute window
    rate_ceiling: u32,        // 0 = uncapped
) -> Result<(), EmailCommsError>
```

Rejection condition: `rate_ceiling > 0 && recent_send_count >= rate_ceiling`

Emitted error: `EmailCommsError::RateCeilingExceeded { tenant: binding.tenant.clone(), per_minute: rate_ceiling }`

The check is placed **after** the existing DMARC/SPF/DKIM checks and **before**
the suppression loop, so all structural-config failures are surfaced before
volume-limit failures.

### Idempotency check (ST2)

The function signature gains one more trailing parameter:

```rust
pub fn enforce_deliverability_invariants(
    binding: &DeliverabilityBinding,
    message: &OutboundMessage,
    suppressed: &[EmailAddress],
    warm_up_complete: bool,
    recent_send_count: u32,
    rate_ceiling: u32,
    prior_fingerprints: &std::collections::HashMap<String, u64>,
) -> Result<(), EmailCommsError>
```

The fingerprint function uses a dep-free FNV-1a fold over `from + to_addrs +
subject + html_body` bytes:

```rust
fn message_fingerprint(message: &OutboundMessage) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64  = 1099511628211;
    let mut h = FNV_OFFSET;
    for byte in message.from.as_str().bytes()
        .chain(message.to.iter().flat_map(|a| a.as_str().bytes()))
        .chain(message.subject.bytes())
        .chain(message.html_body.bytes())
    {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}
```

Decision table:

| `prior_fingerprints.get(&key)` | `fp == prior` | Result |
|---|---|---|
| `None` | — | `Ok(())` |
| `Some(prior)` | `true` | `Ok(())` — collapse |
| `Some(prior)` | `false` | `Err(IdempotencyConflict { key })` |

Callers that do not track idempotency pass `&HashMap::new()`.

---

## Adapter update (call-site threading)

All four adapter shells call `enforce_deliverability_invariants` from
`preflight`. Each is updated to pass `0, 0, &Default::default()` for the new
parameters, preserving existing behaviour (uncapped, no prior fingerprints).
The `EmailComms::preflight` trait signature is **not changed** — the new
parameters are internal to the invariant function; callers that need to enforce
limits construct the values before calling `preflight`.

---

## Testing strategy

All tests live in the `#[cfg(test)] mod tests` block in `src/lib.rs`.

### ST1 tests (rate ceiling)

| Test name | Input | Expected |
|---|---|---|
| `rate_ceiling_at_limit_rejected` | `recent=10, ceiling=10` | `RateCeilingExceeded { per_minute: 10 }` |
| `rate_ceiling_below_limit_accepted` | `recent=9, ceiling=10` | `Ok(())` |
| `rate_ceiling_zero_means_uncapped` | `recent=999, ceiling=0` | `Ok(())` |

### ST2 tests (idempotency)

| Test name | Prior map | Message | Expected |
|---|---|---|---|
| `idempotency_fresh_key_accepted` | `{}` | any | `Ok(())` |
| `idempotency_same_key_identical_message_collapsed` | `{key -> fp}` | same message | `Ok(())` |
| `idempotency_same_key_different_message_rejected` | `{key -> fp}` | mutated subject | `IdempotencyConflict { key }` |

### Existing tests

All existing `preflight_*` tests are updated to pass the new parameters as
`0, 0, &Default::default()` — no behavioural change.

---

## Boundaries

- Modify: `crates/shared-email-comms-kernel/src/lib.rs` only.
- Do not touch: root `Cargo.toml`, any other crate, any other file.
- Do not add any dependency to `Cargo.toml [dependencies]`.

---

## Definition of done

1. `cargo check -p shared-email-comms-kernel --all-targets` → clean.
2. `cargo nextest run -p shared-email-comms-kernel` → all tests pass.
3. `Cargo.toml [dependencies]` is empty.
4. `RateCeilingExceeded` and `IdempotencyConflict` variants are reachable from
   production code paths (no longer dead variants).
