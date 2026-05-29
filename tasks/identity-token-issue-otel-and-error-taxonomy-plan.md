# Plan: identity-token-issue-otel-and-error-taxonomy

## Objective

Add `src/observability.rs` to `oya-identity-usecase` exposing a stable OTel event/attribute
taxonomy for the `identity.token.issue` and `identity.token.rotate` app functions. Pure
mapping over the existing error and status types — issuance logic unchanged.

## Acceptance Criteria

1. `src/observability.rs` declares:
   - `SURFACE` constant (`"identity.token.issue"`) for the telemetry surface name
   - `OutcomeLabel` enum with `success` / `failure` string forms
   - `IdentityTokenIssueEvent` value type with stable fields:
     - `surface: &'static str` — always `"identity.token.issue"`
     - `outcome: OutcomeLabel` — success or failure
     - `error_code: Option<&'static str>` — `None` on success; maps to the stable error code string
     - `purpose: Option<&'static str>` — the PascalCase purpose label from the request, `None` when
       validation fails before the purpose can be extracted
     - `tenant_id_hash: u64` — low-cardinality FNV-1a hash of the tenant_id, never raw value
     - `data_class: &'static str` — always `"AUDIT"` (OperationalDataClass::Audit label)
   - `identity_token_issue_event_for_success` constructor
   - `identity_token_issue_event_for_error` constructor
   - `identity_token_rotate_event_for_success` constructor
   - `identity_token_rotate_event_for_error` constructor
2. Every `IdentityTokenIssueApiError` variant maps to a stable, non-empty `error_code` string.
3. Tests assert all error variants map to non-empty codes; success paths have `error_code = None`;
   `tenant_id_hash` is stable and low-cardinality (same input → same output).
4. `cargo nextest run -p oya-identity-usecase` passes green.
5. `cargo check -p oya-identity-usecase --all-targets` passes with zero errors.
6. Only `oya-identity-usecase` crate files, plus two lane docs, appear in the diff.

## Edge Cases

- Error variants with inner data (e.g. `TenantMismatch { .. }`) must still map to a stable
  constant code string — no inner data leaks into the telemetry event.
- `Identity(IdentityError)` wrapper delegates to `identity_error_code()` already present in
  `lib.rs`; the observability module must not duplicate that logic.
- `tenant_id_hash` must never be the raw tenant_id string — only a FNV-1a hash.
- No new crate dependencies (no `tracing`, no `opentelemetry` crate deps) are added to
  `Cargo.toml`; this is pure data mapping.
- The module is `pub mod observability` in `lib.rs`; no binary or runtime adapter code.

## Subtasks

1. [x] Write `tasks/identity-token-issue-otel-and-error-taxonomy-plan.md` (this file)
2. [ ] Write `docs/specs/task-identity-token-issue-otel-and-error-taxonomy.md`
3. [ ] Create `crates/oya-identity-usecase/src/observability.rs` with types + constructors
4. [ ] Add `pub mod observability;` to `src/lib.rs`
5. [ ] Add tests for all error variant → error_code mappings, success paths, hash stability
6. [ ] Run `cargo check -p oya-identity-usecase --all-targets` and confirm zero errors
7. [ ] Run `cargo nextest run -p oya-identity-usecase` and confirm green
8. [ ] Commit only the allowed paths
