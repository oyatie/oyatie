# sheets DR failover runbook

## Purpose

Recover the sheets service inside the declared ADR-0343 DR envelope without changing tenant version pins or weakening Cedar admission.

## Recovery order

1. Freeze new mutating ingress for sheets at api-gateway while read-only routes stay available when data freshness permits.
2. Promote PostgreSQL WAL-G state and object-storage versions for the affected cell, then restore Valkey cache/session state from the replicated substrate where declared.
3. pause heavy recalc jobs, replay workbook mutation logs, and reopen collaboration once formula snapshots are consistent.
4. Re-run Cedar policy checks, tenant-version routing checks, and service health probes before reopening writes.
5. Record drill or incident evidence in audit-chain and attach the evidence id to the next manifest update.

## Stop condition

Writes reopen only after manifest-declared RPO/RTO targets are met, public surface versions still route to the pinned tenant default, and no ADR-0340/ADR-0338 placement violation is present.
