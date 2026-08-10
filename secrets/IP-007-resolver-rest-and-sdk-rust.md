---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-007-resolver-rest-and-sdk-rust
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [contract-test, sdk-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: resolver-rest + SDK (Rust)

## Intent

Ship two crates:
1. `-rest` — admin REST endpoints per `contracts/openapi/cloud-secrets.yaml` (NOT the hot path; admin only).
2. `-sdk` — the canonical Rust client SDK; the primary integration surface.

## ChangeSet boundary

Two new crates. SDK is the public surface; REST is admin only.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-secret-reference-resolver-rest/Cargo.toml` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-rest/src/lib.rs` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-rest/src/handlers/*.rs` | create — one handler per OpenAPI path |
| `…/oya-cloud-secrets-secret-reference-resolver-sdk/Cargo.toml` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-sdk/src/lib.rs` | create — `SecretClient` |
| `…/src/secret_wrapper.rs` | create — `Secret<T>` newtype with Zeroize + redacted Debug |
| `…/src/with_secret.rs` | create — scoped callback API |
| `secrets/catalog/oya-cloud-secrets-secret-reference-resolver-rest.yaml` | create |
| `secrets/catalog/oya-cloud-secrets-secret-reference-resolver-sdk.yaml` | create |

## Code Shape

```rust
// SDK public API
pub struct SecretClient { /* … */ }

impl SecretClient {
    pub fn builder() -> SecretClientBuilder { /* … */ }

    pub async fn resolve(&self, reference: &SecretReference) -> Result<Secret<Vec<u8>>, ResolveError>;
    pub async fn with_secret<F, T>(&self, reference: &SecretReference, f: F) -> Result<T, ResolveError>
    where F: FnOnce(&[u8]) -> Result<T, ResolveError>;
    pub async fn subscribe_revocations(&self) -> Result<RevocationStream, RevocationError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-rest
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-sdk
cargo run -p oya-dev-cli -- gate validate sdk-contract-conformance --sdk-lang rust
```

## Test Plan

- Contract tests against OpenAPI: every handler honours schema.
- SDK smoke: end-to-end resolve via SDK → sandbox OpenBao.
- `Secret<T>` Debug returns `[REDACTED]`.
- `with_secret` callback zeroises after return.

## Halt Conditions

- SDK exposes raw value type — BLOCKER.

## Next IP

`IP-008-sdk-ts-python-bindings.md`

## Wave 15-IP-substance A-G

### A. Problem
The PRD makes the SDK the primary integration surface, but admin REST and Rust SDK behavior must be identical or product teams will create local secret-resolution shortcuts.

### B. Approach
Expose admin/health/config REST routes for operators and a Rust SDK for product µservices. Runtime resolution stays SDK-side with `Secret<T>` wrappers, redacted debug output, zeroization, and corpus-backed parser compatibility.

### C. Deliverables
- `oya-cloud-secrets-secret-reference-resolver-rest` and `oya-cloud-secrets-secret-reference-resolver-sdk`.
- REST contract alignment with `contracts/openapi/cloud-secrets.yaml`.
- SDK API matching `sdk-plan.md` and `reference-implementations/static-and-dynamic-secret-flow-rust-sdk.md`.
- SLO hooks for secret resolution and audit completeness.
- Catalog files for rest and sdk crates.

### D. Ordered Implementation Steps
1. Generate/validate REST handlers from OpenAPI for admin-only surfaces.
2. Implement Rust `SecretReference` parsing by calling the domain crate.
3. Implement `CloudSecretsClient::resolve` over the usecase/API contract.
4. Wrap values in `Secret<T>` with redacted `Debug` and callback-only access.
5. Add zeroize-on-drop behavior and cache TTL ceiling enforcement.
6. Add SDK smoke tests against sandbox OpenBao.
7. Run SDK contract conformance across REST and Rust client fixtures.

### E. Acceptance
- `cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-rest`.
- `cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-sdk`.
- `cargo run -p oya-dev-cli -- gate validate sdk-contract-conformance --sdk-lang rust`.
- `Secret<T>` never exposes raw values through `Debug`, logs, panics, or telemetry.

### F. Evidence
Evidence anchors are `PRD.md` FR-01/FR-02, `manifest.json`, `contracts/openapi/cloud-secrets.yaml`, `sdk-plan.md`, `reference-implementations/static-and-dynamic-secret-flow-rust-sdk.md`, and `slos/secret-resolve-latency.openslo.yaml`.

### G. Counterpart Comparison
AWS, Google, Azure, Vault, and Akeyless all ship SDKs. The parity matrix marks Oyatie's required difference: the SDK enforces `Secret<T>`, TTL ceilings, no-log, and LEAN-A11 compatibility instead of treating secrecy as caller discipline.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant where Rust SDK examples run in CI and consume workflow-provided secret handles. The SDK comparison itself remains anchored on vendor SDK behavior, not CI secret storage as the primary truth.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `secrets/contracts/openapi/cloud-secrets.yaml`, `secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `secrets/contracts/proto/cloud-secrets.proto`, `secrets/IP-007-resolver-rest-and-sdk-rust.md`.

## DR posture (per ADR-0343)

- Target source: `secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `secrets/runbooks/hsm-key-rotation.md`, `secrets/runbooks/openbao-restart.md`, `secrets/manifest.json`, `secrets/IP-007-resolver-rest-and-sdk-rust.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `secrets/IP-007-resolver-rest-and-sdk-rust.md`; matched trigger term(s): `sandbox`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
