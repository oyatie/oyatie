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
