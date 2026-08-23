# Spec: oidc-issuer-verification-key-grace-selection

## Objective

Add a pure, deterministic verification-key selector for the rotation grace overlap
window to the `identity-oidc-issuer-kernel` crate. The function complements the
existing `current_signing_key` (Active-only signer selector) by covering the RP-side
verify-only overlap the `lib.rs:345` doc describes but does not implement.

## Crate Boundary

**Single crate**: `crates/identity-oidc-issuer-kernel`. No new workspace members.
No new dependencies. Zero I/O. Caller-supplied clock.

## Contracts

- **Input**: `keys: &[SigningKey]`, `kid: &str`, `now_epoch_seconds: i64`,
  `grace: VerificationGrace`
- **Output**: `Option<&SigningKey>`
- **Pure / deterministic**: same inputs → same output, always.
- **No panics**: ADR-0083 Tier 3 (all paths return `Option` or `Result`, never panic).

## New Public Surface

### Constants

```rust
/// Hard ceiling on the verification grace period (24 hours).
pub const VERIFICATION_GRACE_SECONDS: i64 = 86_400;
```

### Newtype

```rust
pub struct VerificationGrace(i64);

impl VerificationGrace {
    /// Construct, validating `0 <= value <= VERIFICATION_GRACE_SECONDS`.
    pub fn new(seconds: i64) -> Result<Self, IssuerError>;
    pub fn seconds(self) -> i64;
}
```

### Errors (added to `IssuerError`)

```rust
/// Grace period was negative.
NegativeGracePeriod,
/// Grace period exceeded `VERIFICATION_GRACE_SECONDS`.
GracePeriodTooLong { requested_seconds: i64, ceiling_seconds: i64 },
```

### Function

```rust
/// Select a signing key for RP-side signature verification, honouring the
/// rotation grace overlap window.
pub fn select_verification_key<'a>(
    keys: &'a [SigningKey],
    kid: &str,
    now_epoch_seconds: i64,
    grace: VerificationGrace,
) -> Option<&'a SigningKey>;
```

**Selection semantics** (evaluated in order):

1. Find a key with `key.kid() == kid`. If none → `None`.
2. If state is `Active` → `Some(key)` (unconditional).
3. If state is `RotatedOut`:
   - If `activated_at_epoch_seconds` is `None` → `None`.
   - Else if `now_epoch_seconds - activated_at <= grace.seconds()` → `Some(key)`.
   - Else → `None`.
4. Otherwise (`NotYetActive`, `Retired`) → `None`.

**Note on `RotatedOut` age**: The key was rotated out after a period of being `Active`;
`activated_at_epoch_seconds` records when it entered `Active`, which is the canonical
reference point for computing "how old is this key's signing material". Grace counts
from that activation time: tokens signed at or just after activation are the oldest
outstanding tokens that still need to be verified.

## Mod Layout (flat-clean-arch, ADR-0509)

All new code lives in `src/lib.rs` — no new modules needed (the kernel is a single
flat file per the zero-dep / single-crate pattern already established).

## Testing Strategy

### Acceptance tests (both inline `cfg(test)` and `tests/oidc_issuer_kernel.rs`)

| Test name | Scenario | Expected |
|---|---|---|
| `verification_key_active_accept` | Active key, any grace | `Some` |
| `verification_key_rotated_within_grace` | RotatedOut, now - activated ≤ grace | `Some` |
| `verification_key_rotated_past_grace` | RotatedOut, now - activated > grace | `None` |
| `verification_key_rotated_no_activation_record` | RotatedOut, activated_at=None | `None` |
| `verification_key_retired_reject` | Retired key | `None` |
| `verification_key_not_yet_active_reject` | NotYetActive key | `None` |
| `verification_key_unknown_kid` | kid not present | `None` |
| `verification_grace_ceiling_bound` | grace > ceiling | `Err(GracePeriodTooLong)` |
| `verification_grace_negative` | grace < 0 | `Err(NegativeGracePeriod)` |
| `verification_grace_zero` | grace = 0, now == activated_at | `Some` |
| `verification_grace_zero_past` | grace = 0, now > activated_at | `None` |
| `verification_grace_at_ceiling` | grace = VERIFICATION_GRACE_SECONDS | `Ok` |

## Observability / SLO

This is a pure kernel with no I/O: no OTel spans, no metrics, no SLO targets required
(the crate emits no observable signals by design). Adapters wrapping this function may
add OTel spans at the call site.

## Security Notes

- `RotatedOut` keys with no activation record are **rejected** (conservative default):
  unknown activation time means we cannot bound the age of tokens they signed.
- Grace ceiling of 86400 s (24 h) prevents a misconfigured deployment from treating a
  retired key as trusted indefinitely.
- `NotYetActive` and `Retired` keys are hard-rejected regardless of grace: only the
  `Active`/`RotatedOut` states participate in verification.
- Per RFC 8725 BCP §3.1, symmetric HS-family keys are rejected at issuance (existing
  `Algorithm::parse` guard); this function operates on `SigningKey` values that already
  passed that gate.
