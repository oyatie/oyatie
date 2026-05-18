# IP-WASMTIME-004 — Component Model + WIT onboarding for foundry tools

> ADR anchor: ADR-0200.
> Owner: `oya-foundry`.
> Estimate: 4 days.

## Goal

Onboard new Foundry tools via WIT (WebAssembly Interface
Types) definitions that match the `oya:foundry/*` import
allowlist. Tool authors write WIT, compile to component-model
bytecode, register in the tool registry.

## Why this IP

WASI Preview 2 Component Model is the ABI per ADR-0200. WIT
is how upstream Wasmtime expresses the interface contract.
Without WIT onboarding, tool authors guess at the ABI and
end up with imports that fail at instantiation.

## Tasks

### 1. WIT canonical surface

- `oya:foundry/argv_read` — read tool input.
- `oya:foundry/stdout_write` — write tool output.
- `oya:foundry/log` — structured logging.
- `oya:foundry/capability_token_check` — verify the per-call
  capability token.

### 2. Tool packaging

- Build script template emits component-model bytecode.
- Registry validates bytecode against the WIT contract.

### 3. Tests

- Sample tool written in Rust + `wit-bindgen` + compiled to
  component-model.
- Sample tool in TinyGo (per Fastly community precedent).
- Sample tool in C via clang/wasi-libc.

### 4. Acceptance criteria

- A Rust + TinyGo + C tool all run end-to-end through the
  Foundry sandbox.
- Imports outside the WIT contract fail at instantiation.

## References

- ADR-0200.
- BytecodeAlliance Component Model spec.
- WIT canonical reference (upstream).
