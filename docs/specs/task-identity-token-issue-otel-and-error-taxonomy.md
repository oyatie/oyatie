# Spec: identity-token-issue-otel-and-error-taxonomy

## Objective

Introduce `src/observability.rs` in `identity-usecase` that wraps the
`issue_identity_token_from_app` and `rotate_identity_token_from_app` call sites with a stable
OTel-compatible event/attribute taxonomy. This module is pure data mapping — no issuance logic
changes, no new crate dependencies.

## Crate Boundary

Only `identity-usecase` (`crates/identity-usecase/`) is modified. No root `Cargo.toml`
edits. No new workspace members.

## Mod Layout (flat-clean-arch / ADR-0509)

```
crates/identity-usecase/src/
  lib.rs              — existing, adds `pub mod observability;`
  observability.rs    — NEW: stable taxonomy types and constructors
tests/
  identity_token_issue.rs   — existing tests (unchanged)
  observability.rs           — NEW: tests for taxonomy correctness
```

## Contracts

- Surface name: `identity.token.issue` (matches `IDENTITY_TOKEN_ISSUE_SURFACE` in `lib.rs`)
- OTel attribute taxonomy (low-cardinality stable strings):
  | Attribute       | Type              | Notes                                          |
  |-----------------|-------------------|------------------------------------------------|
  | `surface`       | `&'static str`    | Always `"identity.token.issue"`                |
  | `outcome`       | `OutcomeLabel`    | `"success"` or `"failure"`                    |
  | `error_code`    | `Option<&'static str>` | None on success; stable SCREAMING_SNAKE code |
  | `purpose`       | `Option<&'static str>` | PascalCase label; None if not yet validated |
  | `tenant_id_hash`| `u64`             | FNV-1a hash of tenant_id, never raw value      |
  | `data_class`    | `&'static str`    | Always `"AUDIT"` (OperationalDataClass::Audit) |

- SLO: feeds `http_requests_total{service="identity",status=~"2.."}` via outcome label;
  existing `microservices/identity/slos/availability.openslo.yaml` is not modified.
- AUDIT operational class: `data_class = "AUDIT"` marks every event as operational audit data
  per the `OperationalDataClass::Audit` label in `data-boundary-kernel`.

## Observability Module API

```rust
pub const SURFACE: &str = "identity.token.issue";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeLabel { Success, Failure }

impl OutcomeLabel {
    pub const fn as_str(self) -> &'static str { ... }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueEvent {
    pub surface: &'static str,           // always SURFACE
    pub outcome: OutcomeLabel,
    pub error_code: Option<&'static str>,
    pub purpose: Option<&'static str>,
    pub tenant_id_hash: u64,
    pub data_class: &'static str,        // always "AUDIT"
}

pub fn identity_token_issue_event_for_success(
    request: &IdentityTokenIssueApiRequest,
) -> IdentityTokenIssueEvent;

pub fn identity_token_issue_event_for_error(
    request: &IdentityTokenIssueApiRequest,
    error: &IdentityTokenIssueApiError,
) -> IdentityTokenIssueEvent;

pub fn identity_token_rotate_event_for_success(
    request: &IdentityTokenRotationRequest,
) -> IdentityTokenIssueEvent;

pub fn identity_token_rotate_event_for_error(
    request: &IdentityTokenRotationRequest,
    error: &IdentityTokenIssueApiError,
) -> IdentityTokenIssueEvent;
```

## Testing Strategy

Integration test file `tests/observability.rs`:

1. **All error variants → stable non-empty code**: iterate every `IdentityTokenIssueApiError`
   variant and assert `event.error_code` is `Some` with a non-empty string.
2. **Success path**: `error_code = None`, `outcome = Success`.
3. **Purpose extraction**: valid purpose → `Some("CapabilityInvocation")`; invalid purpose
   (validation fails early) → `None`.
4. **tenant_id_hash stability**: same tenant_id always produces the same hash.
5. **tenant_id_hash is not the raw value**: hash is a `u64`, never the tenant_id string.
6. **Rotate surface**: rotation success/error events use the same `SURFACE` constant.

## SLO / OpenSLO

The existing `microservices/identity/slos/availability.openslo.yaml` references
`http_requests_total{service="identity",status=~"2.."}`. The `outcome` label from this
taxonomy is the domain-layer evidence that a runtime adapter will eventually project onto
`http_requests_total`. No SLO file changes in this slice.

## Security / Data Boundary

- `tenant_id_hash` is a FNV-1a 64-bit hash — no raw tenant ID is stored in the event.
- `error_code` and `purpose` are always `&'static str` labels — no dynamic user input leaks.
- `data_class = "AUDIT"` per `OperationalDataClass::Audit` designation.
- No PII or INTERNAL_ONLY data in any telemetry field.
