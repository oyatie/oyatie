---
doc_class: RetirementPlan
template_id: TPL-RETIREMENT-PLAN
microservice: connector (umbrella — retiring)
status: Retiring
declaration_date: 2026-05-17
removal_target: when all 8 sub-µservices reach HG-<MS> green at p99 SLO sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/mail.json, /specs/microservices/messenger.json, /specs/microservices/calendar.json]
owner_team: council-architecture
date: 2026-05-17
doc_status: published
---

# RETIREMENT-PLAN — umbrella µservice

> Even though `microservices/connector/` currently has no production-bound
> contents (the 8 sub-µservices already ship under their own folders per
> ADR-0135), this RETIREMENT-PLAN.md exists at `microservices/connector/`
> to declare the formal retirement criteria, sub-µservice progress
> tracking, and verification checklist for the umbrella concept.
> When all 8 retirement triggers fire, `microservices/connector/` (including
> this file) is deleted in the Phase 6 ChangeSet of ADR-0134.

## Status

**Retiring as of 2026-05-17. Retirement completes when the trigger below
fires for all 8 sub-µservices.**

## The 8 sub-µservices

Per ADR-0135, the super-app is dissolved into exactly 8 first-class
flat µservices:

| µservice | Folder | PRD | Pack-fill status (2026-05-17) | HG-<MS> gate | Phase status |
|---|---|---|---|---|---|
| `mail` | `microservices/mail/` | shipped | **84 files** populated | HG-MAIL registered | Phase 1 active |
| `messenger` | `microservices/messenger/` | shipped | **96 files** populated | HG-MESSENGER registered | Phase 1 active |
| `calendar` | `microservices/calendar/` | shipped | **103 files** populated (PRD + threat-model + dpia + compliance + capacity-model + cost-budget + multi-region + failure-modes + incident-response + IP-001..IP-015) | HG-CALENDAR registered | Phase 1 active |
| `community` | `microservices/community/` | shipped | **126 files** populated (PRD + IP-001..IP-015) | HG-COMMUNITY registered | Phase 1 active |
| `social` | `microservices/social/` | shipped | **96 files** populated | HG-SOCIAL registered | Phase 1 active |
| `shorts` | `microservices/shorts/` | shipped | **97 files** populated | HG-SHORTS registered | Phase 1 active |
| `network` | `microservices/network/` | shipped | **100 files** populated | HG-NETWORK registered | Phase 1 active |
| `community` (anonymity-mode) | `microservices/community/` | folded 2026-05-21 | `microservices/anonymous/` deleted; 106 artifacts extracted into community/ as anonymity posting-mode capability tier | HG-ANONYMOUS superseded by community gate | See `community/IP-N-anonymous-fold-extraction.md` |

> **Pack-fill numbers are point-in-time as of 2026-05-17.** Authoritative
> current count: `find microservices/<ms>/ -type f | wc -l` per µservice.

## Trigger — "umbrella retires when…"

The umbrella µservice (this folder, this plan, and any sibling
artifacts in `microservices/connector/`) retires when **all 8** of the
following conditions hold simultaneously:

1. **HG-MAIL** accepts at p99 SLOs sustained 30d.
2. **HG-MESSENGER** accepts at p99 SLOs sustained 30d.
3. **HG-CALENDAR** accepts at p99 SLOs sustained 30d.
4. **HG-COMMUNITY** accepts at p99 SLOs sustained 30d.
5. **HG-SOCIAL** accepts at p99 SLOs sustained 30d.
6. **HG-SHORTS** accepts at p99 SLOs sustained 30d.
7. **HG-NETWORK** accepts at p99 SLOs sustained 30d.
8. **HG-ANONYMOUS** accepts at p99 SLOs sustained 30d.

When the trigger fires, the Phase 6 ChangeSet from ADR-0134 executes:

1. Delete `microservices/connector/` (this folder, this file, any siblings).
2. Remove `connect-umbrella-retirement-readiness` CI lane from
   `.github/branch-protection.yaml`.
3. Strip the "Connect" umbrella node from `docs/architecture/product-graph.md`
   + `docs/architecture/product-graph.html`; keep the 8 children as
   first-class nodes.
4. Delete `/specs/microservices/*.json` legacy pointers (only after their
   `replacement_path` targets at `/specs/microservices/<ms>/*.json` are promoted).
5. Emit a final registry/placeholder-debt/adr-follow-ups.yaml#connect-umbrella-retirement-marker marker ADR (separately
   numbered at retirement time; not pre-authored here).

## Per-sub-µservice retirement preconditions

Each sub-µservice has its own internal retirement sub-trigger that must
fire before its HG-<MS> can be claimed at p99 sustained 30d:

### `mail`

- All 15 IPs in `microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md`
  merged.
- ADR-0134 Phase 5 (legacy `oya-mail-*` removal) complete.
- `microservices/mail/{deprecation-notice.md,migration-from-connect.md}`
  deleted (per their own Phase 5 self-deletion).

### `messenger`

- All 15 IPs in `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md`
  merged.
- ADR-0134 Phase 5 (legacy `oya-messenger-*` removal) complete.
- `microservices/messenger/{deprecation-notice.md,migration-from-connect.md}`
  deleted.

### `calendar`

- All 15 IPs in `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md`
  merged.
- ADR-0134 Phase 5 (legacy `oya-calendar-*` removal) complete.
- `microservices/calendar/{deprecation-notice.md,migration-from-connect.md}`
  authored + merged then deleted in Phase 5.

> **Calendar deprecation-notice + migration-from-connect.md are not authored
> in this batch** (the prompt covers mail + messenger only). The calendar
> axis must author the equivalent pair in a successor-IP ChangeSet before
> Phase 2 adapter soak begins for calendar; if `microservices/calendar/
> {deprecation-notice.md,migration-from-connect.md}` are absent at the
> moment the calendar axis tries to claim HG-CALENDAR, the
> `oya-governance-deprecation-notice-presence` lane will refuse.

### `community`

- All 15 IPs in `microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md`
  merged.
- No legacy `oya-connector-community-*` crates exist (verified via `find
  crates -maxdepth 1 -type d -name 'oya-connector-community-*' | wc -l`
  → 0); community has no Strangler migration to run, only stand-up.

### `social`, `shorts`, `network`, `anonymous`

- Net-new µservices. PRD must be authored. Phase-01 plan must be authored.
  15 IPs must be authored + merged. HG-<MS> must be registered. OpenSLO
  must be authored. No legacy `oya-connector-{social,shorts,network,anonymous}
  -*` crates exist (verified via `find crates -maxdepth 1 -type d -name
  'oya-connector-{social,shorts,network,anonymous}-*' | wc -l` → 0); these
  µservices have no Strangler migration to run, only stand-up.

## Cross-µservice composition during the transition

During the retirement window (2026-05-17 through trigger-fire):

- The 3 sub-µservices with legacy crates (mail, messenger, calendar)
  execute ADR-0134's 6-phase Strangler in parallel; each maintains its own
  adapter + canary timeline; nothing serialises across the three.
- The 5 net-new µservices (community, social, shorts, network, anonymous)
  stand up independently; each has its own phase-01 plan + IPs.
- Cross-µservice handoffs continue via Workflow events + Ontology
  reads/writes per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`.
  Examples:
  - `mail.MessageReceived` → calendar reads for invitation parsing.
  - `messenger.HuddleRecordingFinalised` → mail emits an email summary.
  - `community.PostFlagged` → moderation pipeline emits per Foundry
    guardrails.

No direct `use oya_<other_ms>_*` imports cross µservice boundaries during
or after retirement.

## Operational gates

| CI lane | Status | Promotion behaviour |
|---|---|---|
| `oya-governance-no-grouping` (ADR-0132) | BLOCKER on dev | Already enforces no-new-suite forward-policy |
| `oya-governance-per-microservice-layout` (ADR-0131) | BLOCKER on dev | Already enforces per-µservice flat layout |
| `oya-governance-connector-legacy-symbol-zero-usage` (ADR-0134 §Phase 4) | REPORT-ONLY → BLOCKER | Blocks Phase 5 removal until usage = 0 |
| `oya-governance-connector-retirement-readiness` (ADR-0134 §Phase 6 + this plan) | REPORT-ONLY → BLOCKER | Blocks Phase 6 retirement until all 8 HG-<MS> green at p99 sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

This umbrella retirement closes the entire deprecation cycle. The skill's
checklist applies at umbrella scope:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  All 8 sub-µservices HG-<MS> green at p99 SLO sustained 30d. Verified by:
  ```bash
  buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity-claims
  # expect: HG-MAIL, HG-MESSENGER, HG-CALENDAR, HG-COMMUNITY, HG-SOCIAL,
  #         HG-SHORTS, HG-NETWORK, HG-ANONYMOUS all green
  ```
- [ ] **Migration guides exist with concrete steps and examples.**
  ```bash
  test -f microservices/mail/migration-from-connect.md
  test -f microservices/messenger/migration-from-connect.md
  test -f microservices/calendar/migration-from-connect.md   # axis-calendar must author before HG-CALENDAR claim
  # community, social, shorts, network, anonymous: no migration-from-connect.md
  # needed (no legacy `oya-connector-<ms>-*` crates existed); confirm via:
  for ms in community social shorts network anonymous; do
    find crates -maxdepth 1 -type d -name "oya-connector-$ms-*" | wc -l   # expect 0 each
  done
  ```
- [ ] **All active consumers have been migrated.**
  ```bash
  buck2 build //:quality-lane-registry-authority-check # lane=connect-legacy-symbol-zero-usage
  # expect: exit 0; lane BLOCKER green
  ```
- [ ] **Old code, tests, documentation, configuration are fully removed.**
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connector-*"   | grep -E "(mail|messenger|calendar|community|social|shorts|network|anonymous)"   | wc -l   # expect 0
  ls /specs/microservices/ 2>/dev/null | wc -l   # expect 0 (folder deleted)
  ```
- [ ] **No references to the deprecated system remain in the codebase**
  (excluding historical ADR / RETIRED.md / git-log surfaces):
  ```bash
  rg "oya_connector_(mail|messenger|calendar|community|social|shorts|network|anonymous)" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/baseline/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose).**
  ```bash
  for ms in mail messenger calendar; do
    test ! -f microservices/$ms/deprecation-notice.md      # expect file absent
    test ! -f microservices/$ms/migration-from-connect.md  # expect file absent
  done
  test ! -d microservices/connector/   # this folder absent → umbrella fully retired
  ```

## Sub-µservice progress dashboard

Updated by axis-mail / axis-messenger / axis-calendar / axis-community /
axis-social / axis-shorts / axis-network / axis-anonymous on each
significant phase transition. The dashboard is canonically rendered by
`cargo run -p oya-dev-cli -- gate report connect-umbrella-retirement-readiness`;
the table below is a human-readable snapshot.

| Sub-µservice | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 | HG-<MS> ≥ 30d |
|---|---|---|---|---|---|---|
| mail        | ACTIVE | pending | pending | pending | pending | pending |
| messenger   | ACTIVE | pending | pending | pending | pending | pending |
| calendar    | ACTIVE | pending | pending | pending | pending | pending |
| community   | n/a (no legacy) | n/a | n/a | n/a | n/a | pending |
| social      | not stood up | n/a | n/a | n/a | n/a | pending |
| shorts      | not stood up | n/a | n/a | n/a | n/a | pending |
| network     | not stood up | n/a | n/a | n/a | n/a | pending |
| anonymous   | not stood up | n/a | n/a | n/a | n/a | pending |

## References

- ADR-0135: super-app expansion into 8 flat µservices (target topology).
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: dissolution Strangler migration (operational policy).
- `microservices/mail/migration-from-connect.md`.
- `microservices/messenger/migration-from-connect.md`.
- (axis-calendar to author) `microservices/calendar/migration-from-connect.md`.
- `feedback_no_silent_regression.md`.
- `feedback_workflow_objectgraph_adapter_layer.md`.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern, Verification.
