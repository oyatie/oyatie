---
doc_class: BackfillReplayPlan
template_id: TPL-BACKFILL-REPLAY
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + ops-sre-reliability + council-privacy
related_adrs: [ADR-0255, ADR-0263]
doc_status: published
---

# Backfill + Replay Plan — intelligence µservice

## Purpose

Define how the substrate handles conversation backfill (tenant migrating in / re-importing
historical dispatches), audit-row replay (forensic reconstruction for compliance), and the
boundaries around each. The intelligence µservice does NOT persist conversation state (caller-side
RAG); the substrate's primary replayable state is the audit-tap stream.

## What is replayable

| Asset | Replayable? | Source | Latency to replay |
|---|---|---|---|
| Audit-tap records | yes | audit-chain seal stream | within minutes |
| Per-call cost record | yes (via finops projection from audit-tap) | finops µservice | minutes |
| Provider routing decisions | yes (embedded in audit-tap record) | audit-chain | minutes |
| Refusal decisions | yes (embedded in audit-tap record) | audit-chain | minutes |
| Eval scores | yes (per-call eval emitted) | audit-chain + eval-worker | minutes |
| Prompt content (text/image/audio/video) | partial — hash + classification class only stored; raw content NEVER stored beyond stream | n/a | n/a |
| Output content | partial — same | n/a | n/a |
| Streaming chunks | not replayable | provider stream transient | n/a |

## Conversation backfill (tenant import)

When a tenant migrates from another platform and wants to import conversation history, the
substrate offers a `dispatch.backfill` endpoint with these constraints:

1. Backfill items are NOT re-dispatched to providers (no replay-cost).
2. Backfill items become audit-tap records (sealed) with provenance label `backfill`.
3. Backfill never re-runs refusal classifier (the original platform's refusal stands; oyatie
   audit-tap records `external-refusal-stance`).
4. Backfill rate-limited per tenant; daily cap 100k items default (configurable).
5. PHI / PII in backfill content is treated identically to live dispatch — minimisation +
   classification + pack-residency apply.

## Audit-row replay (forensic reconstruction)

For compliance investigations (e.g., DSR; regulator request; tenant audit), the substrate offers
audit-row replay:

```text
Forensic query → audit-chain stream → filter by (tenant_id, time-range, audience, modality, ...)
   ↓
Emit replay manifest (signed by SPIFFE forensic role)
   ↓
Replay manifest signed-export to requesting party (tenant ops / regulator / auditor)
```

Replay is read-only; no side effects. Cedar `auditor-scope.cedar` gates replay access to
time-boxed engagements only.

## Replay tooling

- `cargo run -p oya-dev-cli -- intelligence replay --tenant <id> --from <ts> --to <ts>` — operator-side
  replay manifest export; requires JIT elevation per OpenBao policy.
- `oya-intelligence-eval-worker` periodically replays a sampled subset for canonicalen-set continuity;
  read-only; no provider re-dispatch.

## DSR (Data Subject Request) integration

Right-to-erasure / right-to-access flows leverage replay:

- **Right-to-access (GDPR Art. 15 / KR PIPA Art. 35)**: tenant raises DSR; replay manifest
  generated for the user's audit records; pseudonymised export.
- **Right-to-erasure (GDPR Art. 17 / PIPA Art. 36)**: pseudonymise the user-id hash within the
  audit records (replace with `erased:<dsr-id>`); content fields (already not stored) are not
  affected; audit-chain seal continues to verify the modified record's structural integrity.
- DSR processing SLA: 30 days (GDPR) / 30 days (PIPA) / 15 days (LGPD) — substrate honours the
  strictest applicable per pack.

## DSR limitations

- Audit-tap content was never raw prompt/output; only hash + classification + meta. Hash is one-way
  so DSR-mask substitutes a deterministic erased-marker.
- Records older than retention window may already be deleted before DSR processed; documented in
  DPIA R-08-equivalent.

## Replay verification

- `cargo run -p oya-dev-cli -- gate validate audit-replay-integrity --microservice intelligence` — exit 0.
- Quarterly drill: forensic replay of last quarter's dispatch volume for spot-check tenants.

## References

- ADR-0255, ADR-0263.
- `microservices/intelligence/policy/auditor-scope.cedar`.
- `microservices/intelligence/dpia.md`.
- `microservices/intelligence/runbooks/audit-row-forgery-detected.md`.
- Bominal ADR-0028 audit-chain.
