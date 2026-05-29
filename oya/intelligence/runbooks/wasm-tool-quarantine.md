# Runbook — WASM tool quarantine

> ADR anchor: ADR-0200.
> Severity: SEV-1 (suspected compromised tool) or SEV-2
> (degraded behavior).

## Trigger

- A WASM tool exhibits suspicious behavior (excessive fuel,
  excessive memory, or repeated capability-token verification
  failures).
- External report of compromise.

## Procedure

1. Disable the tool in the Foundry registry:
   `oya-cli foundry tool disable --id {tool_id}`.
2. Capture the tool's bytecode + WIT contract for forensic
   review.
3. Revoke the per-(tenant, tool) capability token via
   IP-WASMTIME-002 path.
4. Audit-chain entry `foundry.tool.quarantined` emitted.
5. Investigate root cause; engage tool author.
6. Re-enable after fix + verification.

## References

- ADR-0200.
- IP-WASMTIME-001, IP-WASMTIME-002.
