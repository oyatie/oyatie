---
doc_class: BackfillReplay
title: "Backfill + Replay procedures"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Backfill + Replay procedures


## Backfill scenarios

### Catalog index rebuild
- Trigger: index corruption suspected; new search field added.
- Procedure: blue-green index swap; `REINDEX CONCURRENTLY`; verify p95 before traffic cutover.

### Installation projection rebuild
- Trigger: ontology projection lag > 24h.
- Procedure: stop projection worker; truncate ontology table; replay from Postgres event log.

### Audit-chain seal backfill
- Trigger: audit-chain outage detected after the fact.
- Procedure: read buffered events from local outbox; replay in chronological order; verify chain integrity.

## Replay invariants

- Replay must be deterministic: same events → same final state.
- Replay must be idempotent: re-running replay produces same result.
- Replay must be auditable: every replay run logged to evidence ledger.

## Replay tooling

```bash
cargo run -p oya-dev-cli -- replay --microservice <ms> --from-offset <ulid> --to-offset <ulid>
```

