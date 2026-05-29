# Runbook — Wasmtime runtime failover

> ADR anchor: ADR-0200.
> Severity: SEV-2 (regional adapter degraded); SEV-1 if
> cluster-wide.

## Trigger

- Wasmtime adapter pod fleet < quorum.
- Fuel-exhaustion rate spike across many tenants.
- Crash loop on a specific bytecode.

## Procedure

1. Page on-call.
2. Restart the wasmtime-sandbox sidecar fleet per
   `microservices/intelligence/iac/helm/wasmtime-sandbox/`.
3. If specific bytecode is implicated, quarantine the tool
   per `wasm-tool-quarantine.md`.
4. Audit-chain entry `foundry.wasm.failover` emitted.

## References

- ADR-0200.
- IP-WASMTIME-001.
