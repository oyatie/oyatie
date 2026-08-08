# forms DR failover runbook

## Purpose

Recover the forms service inside the declared ADR-0343 DR envelope without changing tenant version pins or weakening Cedar admission.

## Recovery order

1. Freeze new mutating ingress for forms at api-gateway while read-only routes stay available when data freshness permits.
2. Promote PostgreSQL WAL-G state and object-storage versions for the affected cell, then restore Valkey cache/session state from the replicated substrate where declared.
3. hold submission acknowledgements, replay response outbox records, reseal audit-chain evidence, and restore export queues.
4. Re-run Cedar policy checks, tenant-version routing checks, and service health probes before reopening writes.
5. Record drill or incident evidence in audit-chain and attach the evidence id to the next manifest update.

## Stop condition

Writes reopen only after manifest-declared RPO/RTO targets are met, public surface versions still route to the pinned tenant default, and no ADR-0340/ADR-0338 placement violation is present.
