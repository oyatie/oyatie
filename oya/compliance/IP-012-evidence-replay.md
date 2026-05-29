---
microservice: compliance
ip: IP-012
title: Evidence replay (re-emit historical evidence into freshly-restored audit chain)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0180, ADR-0209]
---

# IP-012 — Evidence replay

## Purpose

DR scenario: SeaweedFS hot bucket restored from backup; audit chain hot tier rebuilt. Need to replay historical evidence into the restored substrate without breaking seal chain (per ADR-0145).

## Acceptance criteria

1. `oya-compliance-replay` job consumes cold-tier evidence + cosign re-seal chain.
2. Validates chain continuity (each artifact's `prev_seal_hex` matches predecessor).
3. Replay outputs to a fresh ledger with new seal hex; chain links via `replay_from_seal_hex`.
4. Auditor portal shows replay banner: "Replayed evidence from <date>; original seal chain at <archive>."
5. ≥ 5 integration tests: replay-happy-path + chain-break-detected + cross-tenant-replay-isolation + replay-banner-renders + cold-fetch-fail-graceful.

## Replay flow

```
[DR event: hot tier lost]
  → restore cold tier from off-site backup
  → run replay job:
      - read cold artifacts in chronological order
      - verify prev_seal_hex chain
      - emit new artifacts with new seal hex
      - new seal hex includes replay_from_seal_hex (links to original)
  → auditor portal flags as "replayed"
```

## Risk + mitigation

- **Risk:** replay misses an event from the cold tier corruption. **Mitigation:** cold tier has 3-way replication; verify checksum on read.
- **Risk:** auditors challenge the integrity of replayed evidence. **Mitigation:** replay banner + original-seal-hex visible per artifact; cold-tier off-site backup verifiable independently.

## Acceptance evidence

`evidence/ip-012-evidence-replay-acceptance.json`.

## Cross-references

- ADR-0145 — substrate.
- ADR-0180 — DR portfolio policy.
- ADR-0209 — substrate authority.
- IP-006 — SeaweedFS storage.
