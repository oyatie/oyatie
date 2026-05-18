# Runbook — Envoy WASM filter rollback

> ADR anchor: ADR-0200, ADR-0182.
> Severity: SEV-1 if filter regression blocks production
> traffic; SEV-2 otherwise.

## Trigger

- New filter rolls out and request error rate spikes.
- Hot-reload fails xDS reconciliation.

## Procedure

1. Page on-call.
2. Revert the filter to prior generation:
   `oya-cli governance wasm-filter rollback --id {filter_id}`.
3. Verify Envoy applies the rollback within 5 s.
4. Audit-chain entry `governance.wasm.rollback` emitted.
5. Investigate root cause; engage filter author.

## References

- ADR-0200, ADR-0182.
- IP-WASMTIME-001.
