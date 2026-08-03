# Spec: Cloud KMS API Version Negotiation

| Field | Value |
|-------|-------|
| Vertical | cloud |
| Task slug | `cloud-kms-api-version-negotiation` |
| Crate | `oya-cloud-kms-api` |
| Branch | `feat/task-cloud-kms-api-version-negotiation-2026-05-28` |
| ADR authority | ADR-0509 (flat single-crate-per-service), ADR-0131 (per-microservice flat layout) |
| Status | SPEC |

---

## Objective

Extend `oya-cloud-kms-api/src/lib.rs` with a public API-version negotiation
function that is callable by any adapter layer (REST handler, gRPC interceptor)
before a full request is dispatched. The function resolves an optional raw
`Oyatie-Version` header value to either:

- the accepted `&'static str` version (zero-allocation on the success path), or
- a typed `CloudKmsApiError` mapped through the existing error taxonomy.

Additionally, surface the resolved version on the success-path response metadata
(`CloudKmsApiResponseMetadata`) so callers can confirm which API version served
the request without altering the existing `authorize_*` function signatures.

Finally, add the crate's first `#[cfg(test)]` unit-test module covering the four
protocol-specified cases (absent / supported / unsupported / malformed).

---

## Existing surface

The crate already defines:

```rust
pub const CLOUD_KMS_PUBLIC_API_VERSION_HEADER: &str = "Oyatie-Version";
pub const CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION: &str = "2026-05-21";
pub const CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS: &[&str] =
    &["2026-05-21", "2026-02-21", "2025-11-21"];

pub enum CloudKmsApiError {
    MissingPublicApiVersion,
    UnsupportedPublicApiVersion { oyatie_version: String },
    // … other variants
}

pub enum CloudKmsApiErrorCode {
    PublicApiVersionMissing,
    PublicApiVersionUnsupported,
    // …
}

pub struct CloudKmsApiResponseMetadata {
    pub request_id: String,
}
```

The private `validate_public_api_version(&str) -> Result<(), CloudKmsApiError>`
already encodes the version membership check; the new public function unifies
and exposes this as a standalone boundary primitive.

---

## New public API

### `negotiate_cloud_kms_api_version`

```rust
/// Resolve an optional raw `Oyatie-Version` header to the accepted API version.
///
/// | Header | Result |
/// |--------|--------|
/// | absent (`None`) | `Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION)` |
/// | empty / whitespace-only | `Err(MissingPublicApiVersion)` → 400 |
/// | member of `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` | `Ok(version)` |
/// | any other value | `Err(UnsupportedPublicApiVersion { oyatie_version })` → 400 |
pub fn negotiate_cloud_kms_api_version(
    header: Option<&str>,
) -> Result<&'static str, CloudKmsApiError>
```

Return type is `&'static str` — `Ok` arm returns the pointer directly out of
`CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS`, which is `&[&'static str]`, so no
heap allocation occurs on the hot path.

### `CloudKmsApiResponseMetadata` (additive field)

```rust
pub struct CloudKmsApiResponseMetadata {
    pub request_id: String,   // existing
    pub api_version: String,  // NEW: resolved Oyatie-Version for this response
}
```

`CloudKmsCryptoSuccessResponse::ok` signature gains `api_version: impl Into<String>`;
the two `authorize_*` functions populate it from `boundary.oyatie_version` with
no signature change to those public functions.

---

## Mod layout (flat clean-arch, ADR-0509)

All logic lives inside `src/lib.rs` as the single-file flat-crate pattern already
established by this crate. No new modules or files are introduced. The
`#[cfg(test)]` block is appended at the bottom of `src/lib.rs`.

```
crates/oya-cloud-kms-api/
  src/
    lib.rs          ← negotiate_cloud_kms_api_version + updated metadata struct + tests
  tests/
    cloud_kms_api.rs  ← existing integration tests (unchanged except field update)
  Cargo.toml          ← unchanged
```

---

## OpenAPI 3.2.0 contract fragment

The version negotiation function enforces the following header contract,
consistent with the existing KMS REST surface:

```yaml
# Applicable to: POST /v1/keys/{key_id}/encrypt  and  POST /v1/keys/{key_id}/decrypt
parameters:
  - name: Oyatie-Version
    in: header
    required: false
    schema:
      type: string
      pattern: '^\d{4}-\d{2}-\d{2}$'
      example: "2026-05-21"
    description: >
      Requested Cloud KMS public API version.
      Absent → defaults to 2026-05-21.
      Unsupported or malformed → HTTP 400 CLOUD_KMS_PUBLIC_API_VERSION_UNSUPPORTED
      or CLOUD_KMS_PUBLIC_API_VERSION_MISSING.
responses:
  '400':
    description: Version header malformed or unsupported
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/CloudKmsApiErrorResponse'
        examples:
          missing:
            value:
              error:
                code: CLOUD_KMS_PUBLIC_API_VERSION_MISSING
                message: "Oyatie-Version header is required"
          unsupported:
            value:
              error:
                code: CLOUD_KMS_PUBLIC_API_VERSION_UNSUPPORTED
                message: "Oyatie-Version header must be a supported YYYY-MM-DD public API version"
```

---

## proto3 contract fragment

```protobuf
// CloudKmsApiResponseMetadata — negotiation extension
message CloudKmsApiResponseMetadata {
  string request_id  = 1;
  string api_version = 2;  // resolved Oyatie-Version; e.g. "2026-05-21"
}
```

---

## Testing strategy

| Layer | Location | Coverage |
|-------|----------|----------|
| Unit (`#[cfg(test)]`) | `src/lib.rs` | absent header → default; each supported version echoed; unknown string → typed 400 error; empty/whitespace → typed 400 error |
| Integration (`tests/`) | `tests/cloud_kms_api.rs` | existing tests continue to pass; `metadata.api_version` is observable on happy-path responses |

Minimum: **4 new unit tests** in the `#[cfg(test)]` module.

Test style follows the crate's established pattern (no `unwrap` in unit tests;
direct equality assertions via `assert_eq!`).

---

## Boundaries

- **In scope**: `crates/oya-cloud-kms-api/src/lib.rs` only
- **Out of scope**: `oya-cloud-kms-domain`, `oya-cloud-region-domain`, `data-boundary-kernel`; no new crates; no root `Cargo.toml` edits; no async runtime; no new error variants (reuse existing `MissingPublicApiVersion` / `UnsupportedPublicApiVersion`)
- **Signature freeze**: `authorize_cloud_kms_encrypt_from_api` and `authorize_cloud_kms_decrypt_from_api` parameter lists are unchanged

---

## Acceptance criteria

| # | Criterion | Command |
|---|-----------|---------|
| 1 | Crate compiles clean with all targets | `cargo check -p oya-cloud-kms-api --all-targets` |
| 2 | ≥ 4 new unit tests pass | `cargo nextest run -p oya-cloud-kms-api` |
| 3 | `negotiate_cloud_kms_api_version(None)` returns default version | unit test |
| 4 | Each entry in `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` echoed | unit test |
| 5 | Unknown version → `UnsupportedPublicApiVersion` with status 400 | unit test |
| 6 | Empty / whitespace header → `MissingPublicApiVersion` with status 400 | unit test |
| 7 | `authorize_*` signatures unchanged | static check via `cargo check` |
| 8 | `metadata.api_version` populated on success path | integration test assertion |
