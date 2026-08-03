# Plan: cloud-kms-api-version-negotiation

Vertical: **cloud** | Crate: **oya-cloud-kms-api**
Branch: `feat/task-cloud-kms-api-version-negotiation-2026-05-28`

## Objective

Expose a public `negotiate_cloud_kms_api_version` function that resolves an
optional raw `Oyatie-Version` header value to either an accepted version string
or a typed `CloudKmsApiError`, and add the crate's first `#[cfg(test)]` module
covering the four required acceptance cases.

---

## Subtasks

### [kms-1] Public `negotiate_cloud_kms_api_version` function

**What**: Add `pub fn negotiate_cloud_kms_api_version(header: Option<&str>) -> Result<&'static str, CloudKmsApiError>` to `src/lib.rs`.

**Logic**:
- `None` → `Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION)`
- `Some(v)` where `v.trim().is_empty()` → `Err(CloudKmsApiError::MissingPublicApiVersion)`
- `Some(v)` where `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS.contains(&v)` → `Ok(v)` as `&'static str` (the matched slice element)
- anything else → `Err(CloudKmsApiError::UnsupportedPublicApiVersion { oyatie_version: v.to_string() })`

**Reuse**: delegates to / mirrors the existing private `validate_public_api_version` logic; the version list and error types are reused directly from the existing constants and enum — no duplication.

**Return type**: `Result<&'static str, CloudKmsApiError>` — the `Ok` arm returns a reference into `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` (which is `&[&'static str]`), so no allocation on the success path.

**Acceptance**:
- `cargo check -p oya-cloud-kms-api --all-targets` passes
- Function uses `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` / `CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION` directly (no duplicated version list)
- Return type reuses `CloudKmsApiError` / `CloudKmsApiErrorCode` with no new variants

---

### [kms-2] Success-path metadata: resolved `api_version` field

**What**: Add `api_version: &'static str` field to `CloudKmsApiResponseMetadata`; update `CloudKmsCryptoSuccessResponse::ok` to thread `api_version` through; populate it from `boundary.oyatie_version` in `authorize_cloud_kms_encrypt_from_api` / `authorize_cloud_kms_decrypt_from_api` (both functions already hold `request.boundary.oyatie_version`).

**Constraints**:
- Signatures of `authorize_cloud_kms_encrypt_from_api` and `authorize_cloud_kms_decrypt_from_api` are **unchanged** (same parameters, same return type)
- The `api_version` value is observable via `response.metadata.api_version`
- No new public function; additive field on existing struct

**Migration note**: the existing integration test file `tests/cloud_kms_api.rs` constructs `CloudKmsCryptoSuccessResponse` only via `.ok(...)`, so updating `ok`'s signature is the only change required for test compatibility.

**Acceptance**:
- No signature change to the two `authorize_*` functions
- `cargo check -p oya-cloud-kms-api --all-targets` passes
- `response.metadata.api_version` is populated and matches `boundary.oyatie_version`

---

### [kms-3] `#[cfg(test)]` module with ≥ 4 cases

**What**: Add `#[cfg(test)] mod version_negotiation_tests` at the bottom of `src/lib.rs` with the following test cases:

| # | Name | Input | Expected |
|---|------|-------|----------|
| 1 | `absent_header_returns_default_version` | `None` | `Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION)` |
| 2 | `each_supported_version_is_echoed` | each element of `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` | `Ok(version)` |
| 3 | `unknown_version_string_returns_typed_error` | `Some("1999-01-01")` | `Err(UnsupportedPublicApiVersion { .. })` with status code 400 |
| 4 | `empty_header_value_returns_missing_error` | `Some("")` and `Some("  ")` | `Err(MissingPublicApiVersion)` with status code 400 |

**Acceptance**:
- `cargo nextest run -p oya-cloud-kms-api` is green with ≥ 4 new test cases
- `cargo check -p oya-cloud-kms-api --all-targets` passes

---

## Acceptance gate (combined)

```
cargo check -p oya-cloud-kms-api --all-targets   # zero errors
cargo nextest run -p oya-cloud-kms-api           # green, >=4 new unit tests
```

## Boundaries

- **Only** `crates/oya-cloud-kms-api/src/lib.rs` is modified in production code
- No changes to `oya-cloud-kms-domain`, `oya-cloud-region-domain`, or `data-boundary-kernel`
- No new crate added; root `Cargo.toml` untouched
- No async runtime introduced; pure synchronous boundary logic
