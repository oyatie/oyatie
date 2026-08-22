# Standard — WASM runtime canonical (Wasmtime)

> ADR anchor: `docs/decisions/ADR-0709-general-live-apex.md`.
> Code anchor: `crates/shared-wasm-runtime-kernel/`.
> Gate anchor: `crates/check-wasm-runtime-discipline/`.
> Authored 2026-05-18.

## When to use WASM in oyatie

WASM is the canonical extensibility surface for **deterministic,
short-lived, capability-limited** workloads where the host needs:

- Sub-process trust boundary (untrusted code can't read the
  host's memory, files, or network without explicit imports).
- Deterministic CPU / memory ceilings the scheduler can enforce.
- Hot-loadable bytecode (no rebuild of the host binary).

Three sanctioned WASM call sites exist today (each = one sandbox
class in `shared-wasm-runtime-kernel`):

1. **Envoy north-south filter** (`envoy-filter`) — WAF, regulatory
   response shaping, bespoke authz / observability filters.
   Highest QPS, tiny fuel + memory ceilings.
2. **Workflow Studio user-supplied node** (`workflow-studio-node`)
   — expressions, mappers, transformers authored in the visual
   editor.
3. **Foundry tool sandbox** (`foundry-tool`) — LLM-driven tool
   execution where the tool source is not first-party.

## When NOT to use WASM

| Workload | Use instead |
| --- | --- |
| Long-running stateful workload | Container (per ADR-0147 ladder) |
| Untrusted code needing full POSIX | gVisor / Firecracker (ADR-0147) |
| First-party Rust code | Just compile it into the host binary |
| Background batch with network egress + persistent state | Sidecar pod |
| Browser-only DOM logic | Native client stack (ADR-0185) |

If the workload doesn't fit one of the three sanctioned classes,
the answer is "don't add a fourth class without an ADR".

## Sandbox class invariants

| Class | Fuel ceiling | Memory ceiling | Wall-clock | Allowed imports |
| --- | --- | --- | --- | --- |
| `envoy-filter` | 1,000,000 | 16 MiB | 50 ms | `oya:envoy/*` only |
| `workflow-studio-node` | 50,000,000 | 128 MiB | 5 s | `oya:workflow/*` only |
| `foundry-tool` | 500,000,000 | 512 MiB | 60 s | `oya:foundry/*` only |

**No ambient authority.** No sandbox class includes
`wasi:filesystem`, `wasi:sockets`, or `wasi:cli/environ`. Every
host call resolves through an explicit imported capability.

## Capability tokens

Every invocation carries a `CapabilityToken { tenant_id,
sandbox_class, call_site, token }`. The kernel rejects
invocations where the token's sandbox class disagrees with the
caller's, where any field is empty, or where the token does not
verify against the tenant's rotation cycle.

This is the Cloudflare Workers / Fastly Compute@Edge security
posture; any reviewer from those teams should recognize it.

## Discipline

- Only the canonical adapter sub-crate
  (`shared-wasm-runtime-kernel-adapter-wasmtime`) may depend
  on `wasmtime`.
- No µservice may depend on `wasmer`, `wasmedge`, or `wasmtime`.
- `check-wasm-runtime-discipline` is the advisory gate; it
  flips to BLOCKER at T+60d.

## References

- ADR-0200 — WASM runtime canonical (Wasmtime).
- ADR-0147 — container sandboxing runtime ladder.
- ADR-0182 — north-south vs east-west separation.
- ADR-0185 — Workflow Studio client stack.
- BytecodeAlliance / Wasmtime — upstream project.
- Fastly Compute@Edge — operational reference deployment.
