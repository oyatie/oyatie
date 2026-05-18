---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: mail
deprecated_artifact: oya-connect-mail-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-MAIL accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/mail.json]
owner_team: axis-mail
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-mail-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and
> Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-mail-*` crate family under `microservices/mail/src/crates/` per ADR-0131.
See **`microservices/mail/migration-from-connect.md`** for the full
import-path map, Hyrum's-Law-bound surface callouts, configuration delta,
and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** The concrete removal target is HG-MAIL
accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #1).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter soak +
Phase 3 canary), the indicative advisory removal date is **2026-11-17**,
gated on the SLO trigger.

## Reason

The legacy `oya-connect-mail-*` family was authored before the following
ADRs crystallised; each ADR makes the legacy shape non-conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a brand-layer
   concept and must not appear in crate names.
2. **ADR-0130 — agentic SLO-gated promotion.** Mail needs independent SLO
   targets per surface (mailbox-fill, inbound DKIM/SPF/DMARC verify latency,
   outbound queue depth, IMAP fetch p99, eDiscovery export turnaround); a
   `connect-*` umbrella SLO cannot serve them.
3. **ADR-0131 — per-µservice flat layout.** Mail's IaC, runbooks, threat-
   model, DPIA, compliance, capacity-model and cost-budget all need to live
   under one folder (`microservices/mail/`), not scattered across a Connect
   suite directory.
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR-FSS 5y retention +
   전자문서법), pack-eu (GDPR Art. 17), pack-us (HIPAA-mail), etc., need to
   live at per-µservice overlay granularity.

## Migration Guide pointer

→ **`microservices/mail/migration-from-connect.md`**

Includes: 1:1 import-path map (40 crate mappings); concrete `use` and
`Cargo.toml` rewrites; configuration delta table; dual-context isolation
invariant preservation; Hyrum's-Law surface callouts; 5-step migration
recipe; 6-phase Strangler timeline; verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-mail-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-mail-domain` | split per BC → `oya-mail-{mailbox-store,inbound-smtp,outbound-smtp,imap-frontend,search-index,retention-policy,legal-hold,dual-context-isolation}-domain` |

Per ADR-0134, additional `oya-connect-mail-{kernel,usecase,api,adapter*,rest,
worker,sdk,app}-*` crates that may be scaffolded during Phase 2 adapter
authoring will also fall under this deprecation notice and follow the same
mapping.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-mail-*` crates ship in parallel | 1 | No (additive) | — |
| `oya-connect-mail-migration-adapter` shim authored | 2 | No (preserves legacy surface) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-mail-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No (the folder never had production-bound contents in oyatie) | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate +
  CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped per
  semver when its legacy imports are removed (treating the
  `oya-connect-mail-*` re-export as the public contract).
- **Sunset schedule.** 6-month advisory window from this notice; concrete
  date 2026-11-17 contingent on the HG-MAIL SLO trigger.
- **Owning-axis migration ChangeSets.** axis-mail ships migration ChangeSets
  for every known internal consumer per the Churn Rule before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases —
  HG-MAIL gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4 commands
  (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration are fully removed —
  verified by Phase 5 commands.
- [ ] No references to the deprecated system remain — `rg "oya_connect_mail"
  --type rust` produces zero hits outside historical surfaces.
- [ ] Deprecation notices are removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- `microservices/mail/migration-from-connect.md` — full migration guide.
- `microservices/mail/PRD.md` — target-state product definition.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
