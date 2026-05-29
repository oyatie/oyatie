# Plan: pooling-openbao-secret-fetch

## Objective

Implement live OpenBao KV secret-fetch behind the `CredentialHandleIssuerPort`
(SecretResolution port) in `oya-intelligence-credential-resolver-adapter`,
replacing the metadata-only sidecar adapter with a real HTTP adapter that
fetches credential material from OpenBao KV-v2 and returns short-lived
in-memory `CredentialHandle`s.

## Edge Cases & Acceptance Criteria

| # | Edge case | Expected outcome |
|---|-----------|-----------------|
| 1 | Happy-path: KV GET returns `{"data":{"data":{"api_key":"sk-..."}}}` | CredentialHandle issued with generation=1, TTL ≤ 60s |
| 2 | Happy-path: KV GET returns `oauth_access_token` field | CredentialHandle issued (same as api_key path) |
| 3 | OpenBao returns HTTP 401 | `CredentialHandleIssueFailure` with reason `openbao:token-expired` |
| 4 | OpenBao returns HTTP 403 | Failure with reason `openbao:forbidden` |
| 5 | OpenBao returns HTTP 404 | Failure with reason `openbao:secret-not-found` |
| 6 | OpenBao returns HTTP 503 | Failure with reason `openbao:vault-sealed` |
| 7 | JSON decode fails (malformed body) | Failure with reason `openbao:decode-error` |
| 8 | Transport error (connection refused) | Failure with reason `openbao:transport-error` |
| 9 | `BAO_TOKEN` env var missing/empty | `OpenBaoKvAdapterConfig` build-time error |
| 10 | Credential material is empty string after fetch | Failure — reject empty material |
| 11 | Secret material never appears in `Debug` / failure `reason` / `evidence_ref` | Sanitization holds |
| 12 | Seat path is derived from `SecretReference::canonical_ref()` (openbao://secret/<path>) | Path mapping correct |

## Cloud-Native / Contract Implications

- No SDK: direct HTTP to `/v1/secret/data/<seat-path>` per OpenBao KV-v2 wire spec
- `BAO_TOKEN` sourced from env (`std::env::var`) at construction
- hyper-client (`hyper` + `hyper-util` + `http-body-util`) per ADR-0090 preference — no reqwest
- For tests: in-process mock HTTP server on `127.0.0.1:0` using `hyper` server + `tokio`
- `CredentialHandle` never embeds raw key material — only opaque refs
- Debug/Display impls on adapter redact the vault token

## Subtasks (ordered)

1. **SPEC** — write `docs/specs/task-pooling-openbao-secret-fetch.md`
2. **CARGO** — update `Cargo.toml` in `oya-intelligence-credential-resolver-adapter`:
   - add `hyper`, `hyper-util`, `http-body-util`, `bytes`, `serde`, `serde_json`, `tokio`, `tracing` as workspace deps
   - add `[dev-dependencies]`: `tokio` (full), `serde_json`
3. **SRC** — add `src/openbao_kv.rs` mod:
   - `OpenBaoKvAdapterConfig` (base_url, seat_path, vault_token redacted)
   - `OpenBaoKvAdapter` implementing `CredentialHandleIssuerPort`
   - KV GET helper over bare hyper, JSON parse `KvReadResponse`
   - Error → `CredentialHandleIssueFailure` mapping (sanitized, no raw material)
4. **LIB** — expose `OpenBaoKvAdapter` and `OpenBaoKvAdapterConfig` from `lib.rs`
5. **TESTS RED** — write `src/openbao_kv.rs` `#[cfg(test)]` + integration test:
   - in-process mock server (hyper server on `127.0.0.1:0`)
   - JSON parse unit tests
   - error mapping unit tests
6. **BUILD GREEN** — implement until `cargo check` + `cargo nextest` pass
7. **REVIEW** — multi-axis self-review (correctness / security / arch)
8. **SIMPLIFY** — clean up dead code, guard clauses, naming

## Acceptance Evidence

- `cargo check -p oya-intelligence-credential-resolver-adapter --all-targets` → zero errors
- `cargo nextest run -p oya-intelligence-credential-resolver-adapter` → all tests pass
- No raw secret material in Debug output (validated by test assertions)
- `git diff --stat origin/dev` touches only allowed paths
