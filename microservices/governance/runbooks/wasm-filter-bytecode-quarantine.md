# Runbook — WASM filter bytecode quarantine

> ADR anchor: ADR-0200.
> Severity: SEV-1.

## Trigger

- External report of compromise in a registered filter.
- Capability-token verification failures spiking on a filter.

## Procedure

1. Page substrate authority.
2. Disable the filter in the registry.
3. Capture bytecode + WIT contract for forensic review.
4. Revoke the filter's capability token.
5. Audit-chain entry emitted.
6. Re-enable after fix + verification.

## References

- ADR-0200.
- IP-WASMTIME-001.
