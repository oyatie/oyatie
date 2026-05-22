# IP-WASMTIME-001 — Foundry tool sandbox: Wasmtime integration

> ADR anchor: ADR-0200, ADR-0136, ADR-0147.
> Owner: `oya-foundry`.
> Estimate: 6 days.

## Goal

Integrate `oya-shared-wasm-runtime-kernel` into the Foundry
tool-execution path so untrusted-yet-deterministic tool
implementations run inside the `foundry-tool` sandbox class
(ADR-0200) with fuel + memory + wall-clock ceilings.

## Why this IP

Foundry executes LLM-driven tool calls. First-party tools are
trusted Rust code. Third-party / agent-supplied tools must
execute inside a sandbox so a compromised tool cannot escape
to the host filesystem, network, or other tenants' data. WASM
+ Wasmtime is the canonical extensibility surface per
ADR-0200.

## Pre-conditions

- `crates/oya-shared-wasm-runtime-kernel` lands.
- `crates/oya-check-wasm-runtime-discipline` lands.
- ADR-0200 ratified.

## Tasks

### 1. Add the runtime adapter crate

- `crates/oya-shared-wasm-runtime-kernel-adapter-wasmtime` —
  the parent-wired adapter that lives behind the kernel trait.
- Foundry depends only on the kernel; the adapter is wired by
  the µservice's composition root.

### 2. Tool registration

- Foundry tool registry adds a `sandbox_class = FoundryTool`
  field for every WASM-shaped tool entry.
- The registry validates that the tool's bytecode is WASI
  Preview 2 component-model and that all imports are within
  the `oya:foundry/*` allowlist.

### 3. Invocation path

- Replace the existing tool-execution sub-process with a
  kernel `invoke(SandboxClass::FoundryTool, &invocation)`
  call.
- Per-invocation capability token bound to (tenant, sandbox
  class, call-site).
- Invocation results flow through the existing
  audit-chain + observability paths unchanged.

### 4. Resource accounting

- Per-tenant Wasmtime fuel consumption counted toward the
  tenant's FinOps budget per ADR-0174.

### 5. Tests

- Unit tests for the registry validation.
- Integration tests with a sample WASM tool (an "echo" tool
  bytecode) that exercises the full path.
- Negative tests: a tool that tries to import `wasi:filesystem`
  is rejected at instantiation.
- Negative tests: a tool that exceeds fuel ceiling is killed
  cleanly.

## Failure modes

- Wasmtime adapter unavailable: Foundry rejects new WASM tool
  invocations; first-party Rust tools still work.
- Capability-token verification failure: invocation rejected;
  audit-chain entry.

## Acceptance criteria

- `cargo test -p oya-foundry-tool-runner` green.
- An "echo" WASM tool runs end-to-end with the full sandbox
  posture.
- Per-tenant fuel budget reflects WASM tool execution.

## Rollback

Parent feature-flags WASM-tool execution off; existing first-
party Rust tools unaffected.

## References

- ADR-0200, ADR-0136, ADR-0147.
- `crates/oya-shared-wasm-runtime-kernel`.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
