# Plan: oidc-issuer-verification-key-grace-selection

## Objective

Add a pure, deterministic verification-key selector for the rotation grace overlap
window to `oya-identity-oidc-issuer-kernel`.

## Context

After key rotation, RPs (relying parties) hold tokens signed by the old key. The
issuer's JWKS already publishes `RotatedOut` keys (via `is_published`), but there is
no kernel function that accepts a caller-supplied clock and answers "should this key be
trusted for verify-only use right now?". This slice adds that function.

References: `lib.rs:345` doc (RotatedOut published so "relying parties can finish
verifying outstanding tokens"), spec slug `oidc-issuer-verification-key-grace-selection`.

## Edge Cases & Acceptance Criteria

| Scenario | Expected result |
|---|---|
| kid matches `Active` key | `Some(&key)` unconditionally |
| kid matches `RotatedOut`, `now - activated_at <= grace` | `Some(&key)` |
| kid matches `RotatedOut`, `now - activated_at > grace` | `None` |
| kid matches `RotatedOut`, `activated_at_epoch_seconds` is `None` | `None` (no activation record → cannot compute age) |
| kid matches `Retired` | `None` always |
| kid matches `NotYetActive` | `None` always |
| kid not found in slice | `None` |
| `VerificationGrace` construction with negative seconds | `Err(IssuerError::NegativeGracePeriod)` |
| `VerificationGrace` construction over `VERIFICATION_GRACE_SECONDS` ceiling | `Err(IssuerError::GracePeriodTooLong)` |
| `VerificationGrace` construction at exactly the ceiling | `Ok(...)` |
| `VerificationGrace(0)` — zero grace | `RotatedOut` key accepted only when `now == activated_at` |

## K8s / Cloud-native Notes

This is a pure kernel slice — no network, no I/O. Callers (adapter crates, handlers)
supply the clock. Safe to call from any async/sync context.

## Subtasks (ordered)

1. [x] Write `tasks/oidc-issuer-verification-key-grace-selection-plan.md` (this file)
2. [ ] Write `docs/specs/task-oidc-issuer-verification-key-grace-selection.md`
3. [ ] Add `IssuerError::NegativeGracePeriod` and `IssuerError::GracePeriodTooLong` variants + Display arms
4. [ ] Add `VERIFICATION_GRACE_SECONDS: i64` const
5. [ ] Add `VerificationGrace(i64)` newtype with `new()` constructor
6. [ ] Add `select_verification_key()` function
7. [ ] Add unit tests (inline `cfg(test)` mod) covering all acceptance criteria
8. [ ] Add integration tests (tests/oidc_issuer_kernel.rs) covering same criteria
9. [ ] `cargo check -p oya-identity-oidc-issuer-kernel --all-targets` → zero errors
10. [ ] `cargo nextest run -p oya-identity-oidc-issuer-kernel` → all green
11. [ ] Self-review: correctness, security, architecture, performance
12. [ ] Simplify: guard clauses, dead-code, naming

## Acceptance Evidence Required

- `cargo nextest run -p oya-identity-oidc-issuer-kernel` passes with the new tests
  (active-accept, rotated-within-grace-accept, rotated-past-grace-reject,
   retired-reject, not-yet-active-reject, unknown-kid-None, grace-ceiling-bound)
- No change to any existing public fn behaviour
- Zero new external dependencies
