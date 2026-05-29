---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: messenger
deprecated_artifact: oya-messenger-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-MESSENGER accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/messenger.json]
owner_team: axis-messenger
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-messenger-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-messenger-*` crate family under `microservices/messenger/src/crates/`
per ADR-0131. See **`microservices/messenger/migration-from-connect.md`**
for the full import-path map (37 crate mappings), Hyrum's-Law-bound surface
callouts, configuration delta, runbook continuity table, and step-by-step
migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-MESSENGER
accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #2).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter soak +
Phase 3 canary), the indicative advisory removal date is **2026-11-17**,
gated on the SLO trigger.

## Reason

1. **ADR-0132 — no-grouping forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer.
2. **ADR-0139 — agentic SLO-gated promotion.** Messenger needs independent
   SLO targets per surface (persistent-connection-count, message-stream
   throughput, presence-replication lag, websocket-frame p99, huddle SFU
   MTTR).
3. **ADR-0131 — per-µservice flat layout.** Messenger's 10 runbooks,
   threat-model, DPIA, multi-region plan, sdk-plan need to live under one
   folder.
4. **MLS RFC 9420 E2E group messaging** is a net-new clean-replacement-
   boundary capability that did not exist in the legacy surface; the new
   µservice ships it, the legacy did not.

## Migration Guide pointer

→ **`microservices/messenger/migration-from-connect.md`**

Includes: 1:1 import-path map (37 mappings); MLS E2E net-new-boundary
note; concrete `use` and `Cargo.toml` rewrites; configuration delta;
runbook continuity table (6 preserved + 4 net-new); Hyrum's-Law surface
callouts (close-code semantics, message ordering, presence cadence,
mention fan-out latency, search ranking tie-break, frame-size cap);
5-step migration recipe; 6-phase Strangler timeline; verification
checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-messenger-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-messenger-domain` | split per BC → `oya-messenger-{channel-store,message-stream,presence,file-attachment,thread-tree,mention-router,read-receipt-tracker,rest-api-surface,websocket-frame-protocol,search-and-cedar-filter,huddles-livekit-signaling}-domain` |

Plus all `oya-messenger-{kernel,usecase,api,adapter*,rest,worker,
sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-messenger-*` crates ship in parallel | 1 | No (additive) | — |
| New `oya-messenger-e2e-mls-*` MLS RFC 9420 crates | 1 | No (net-new; no legacy counterpart) | — |
| `oya-messenger-migration-adapter` shim authored | 2 | No (preserves legacy surface) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-messenger-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connector/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (loud + immediate + CI-detectable).
- **ADR-0134** (migration policy).
- **Version bump.** Per semver on each consumer's `Cargo.toml`.
- **Sunset schedule.** 6-month advisory window from 2026-05-17.
- **Owning-axis migration ChangeSets.** axis-messenger ships migration
  ChangeSets for every internal consumer per the Churn Rule before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases —
  HG-MESSENGER gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4
  commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5
  commands.
- [ ] No references to the deprecated system remain — `rg
  "oya_connect_messenger" --type rust` produces zero hits outside
  historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- `microservices/messenger/migration-from-connect.md` — full migration guide.
- `microservices/messenger/PRD.md` — target-state product definition.
- `microservices/messenger/runbooks/*.md` — 10 runbooks.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- RFC 9420 — MLS (Messaging Layer Security) protocol.
