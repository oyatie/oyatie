---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-008-sdk-ts-python-bindings
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cross-lang-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: SDK TypeScript + Python bindings

## Intent

Wrap the Rust SDK via napi-rs (TypeScript) and pyo3 (Python). Preserve the `Secret<T>` contract across language boundaries.

## ChangeSet boundary

Two new packages:
- `packages/cloud-secrets-sdk-ts/` (npm `@oyatie/cloud-secrets-sdk`)
- `packages/cloud-secrets-sdk-python/` (PyPI `oyatie-cloud-secrets`)

Both wrap the Rust core via FFI.

## Concrete File Targets

| Path | Action |
|---|---|
| `packages/cloud-secrets-sdk-ts/package.json` | create |
| `packages/cloud-secrets-sdk-ts/index.d.ts` | create — TS types mirroring Rust |
| `packages/cloud-secrets-sdk-ts/src/lib.rs` | create — napi-rs bindings |
| `packages/cloud-secrets-sdk-python/pyproject.toml` | create |
| `packages/cloud-secrets-sdk-python/src/lib.rs` | create — pyo3 bindings |
| `microservices/cloud-secrets/tests/cross-lang/{rust,ts,python}-smoke/` | create |

## Code Shape (TS)

```typescript
export class SecretClient {
  static build(opts: SecretClientOptions): Promise<SecretClient>;
  resolve(ref: SecretReference): Promise<Secret>;
  withSecret<T>(ref: SecretReference, f: (value: string) => Promise<T>): Promise<T>;
  subscribeRevocations(): AsyncIterable<RevocationPush>;
}
```

## Acceptance Gates

```bash
# TS
cd packages/cloud-secrets-sdk-ts && npm run build && npm test

# Python
cd packages/cloud-secrets-sdk-python && maturin develop && pytest

# Cross-lang smoke
cargo run -p oya-dev-cli -- gate validate sdk-contract-conformance --sdk-lang all
```

## Test Plan

- Each binding loads + resolves the same fixture secret.
- `Secret` wrapper zeroises on dispose (TS) / `__exit__` (Python).
- `toString()` / `__repr__` returns `[REDACTED]`.

## Halt Conditions

- FFI leak of raw `Vec<u8>` to non-Rust caller — BLOCKER.

## Next IP

`IP-009-openbao-operator.md`

## Wave 15-IP-substance A-G

### A. Problem
Rust-only enforcement is insufficient because product teams will use TypeScript and Python in services, tools, migrations, and tests. Bindings that expose raw buffers or stringification would undermine the same no-raw-secret rule the Rust SDK enforces.

### B. Approach
Generate TS and Python bindings from the Rust SDK surface while preserving `Secret<T>` semantics, redacted display, explicit callback access, zeroize/dispose hooks, and shared parser corpus tests.

### C. Deliverables
- `packages/cloud-secrets-sdk-ts` and `packages/cloud-secrets-sdk-python`.
- napi-rs and pyo3 bindings over `oya-cloud-secrets-secret-reference-resolver-sdk`.
- Shared corpus consumption from IP-002 fixtures.
- Contract conformance with `sdk-plan.md` and `contracts/openapi/cloud-secrets.yaml`.
- Reference examples for static and dynamic secret flow.

### D. Ordered Implementation Steps
1. Bind the Rust SDK without duplicating parser or policy logic in TS/Python.
2. Implement redacted `toString()`, `inspect`, `repr`, and exception formatting.
3. Provide callback/context-manager APIs for scoped raw-value use.
4. Add disposal/zeroize behavior for buffers crossing FFI.
5. Run shared URI corpus tests in Rust, TS, and Python.
6. Add sandbox OpenBao smoke tests for each language.
7. Add LEAN-A11 fixtures proving logs and assertions never emit raw values.

### E. Acceptance
- `cd packages/cloud-secrets-sdk-ts && npm run build && npm test`.
- `cd packages/cloud-secrets-sdk-python && maturin develop && pytest`.
- `cargo run -p oya-dev-cli -- gate validate sdk-contract-conformance --sdk-lang all`.
- No FFI path exposes an unwrapped raw byte vector or printable secret string.

### F. Evidence
Evidence anchors are `manifest.json`, `sdk-plan.md`, `reference-implementations/static-and-dynamic-secret-flow-rust-sdk.md`, `contracts/openapi/cloud-secrets.yaml`, `PRD.md` FR-01/FR-02, and the feature parity matrix's SDK safety rows.

### G. Counterpart Comparison
Vendor SDKs from AWS, GCP, Azure, Vault, 1Password, Doppler, Infisical, and Akeyless retrieve secrets but generally trust callers not to log them. This IP preserves Oyatie's counterpart advantage by making redaction and zeroization part of every supported language binding.

Grep-recognized counterpart anchor: GitHub Actions Secrets is cited for CI package tests that distribute temporary credentials to TS/Python bindings. The substantive comparator remains language SDK safety, redaction, and zeroization against vendor secret SDKs.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-008-sdk-ts-python-bindings.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/cloud-secrets/IP-008-sdk-ts-python-bindings.md`; matched trigger term(s): `sandbox`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
