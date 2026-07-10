---
id: ADR-0615
title: "Capability boundary rulings — resolving ADR-0562's flagged_boundaries (the substrate/product split + the 14 app-vs-capability dispositions)"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-07-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0562]
depends_on: [ADR-0562, ADR-0280, ADR-0536]
related: [ADR-0245, ADR-0512, ADR-0139, ADR-0532, ADR-0533, ADR-0555]
related_specs:
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
milestone: W0
---

# ADR-0615: Capability boundary rulings — resolving ADR-0562's flagged_boundaries

## Status

**Proposed — 2026-07-10** (ratified under the founder's 2026-07-08 autonomous-drive delegation and
an explicit 2026-07-10 ruling on the fourteen boundary questions, with "the long-term-correct,
maintainable, hyperscaler pattern" as the bar; door: one-way — the same one-way class as ADR-0562,
because a surface ruled `app/` vs a capability `facade/` is a placement commitment the membership
lint then enforces). Lifecycle status stays **Proposed** until the formal Accept rides
cross-artifact propagation (the ADR-Accepted-must-propagate gate): this ADR records the founder's
decisions and amends ADR-0562's `flagged_boundaries_for_leader` from open questions to resolved
dispositions; it moves no crates.

## Context

ADR-0562 ratified the capability-first source-tree shape and the closed capability registry, but
deliberately left a set of **flagged boundaries for the leader** — surfaces where the
capability-vs-`app/` line, or a registry↔DAG reconciliation, required a founder ruling rather than a
mechanical placement. `specs/capability-registry.json` carries these as
`flagged_boundaries_for_leader` and as `boundary_note` "FLAGGED for leader … pending OWNERS
confirmation" fields on `storage`, `console`, `compliance`, and `comms`. Several other surfaces were
coarse-absorbed into a substrate capability **by name** (e.g. `oya/diagnostics` under
`observability`, `oya/imaging` under `storage`) pending confirmation that the name matched the
system's actual nature.

The founder ruled all fourteen on 2026-07-10. This ADR is the durable record and the authority every
Batch-5 reorg move-plan implements.

## Decision

### §1 The governing rule — the substrate/product split

The disposition of every flagged surface is decided by **ADR-0562 §3 rule #5 and §6**, applied
verbatim:

> A deployable surface composing **2+ capabilities** for a tenant → `app/<product>/`.
> A **single-capability** sold surface **is a `facade/`** of that capability (§6: "a single-capability
> app is a *mis-placed facade*"; a capability facade never lives in `app/`).

This is the AWS/Google substrate-vs-product split: the communication *substrate* (SES/Chime), the
object-store *substrate* (S3), the data-plane *substrate* (BigQuery) are capabilities we run and
sell as infrastructure; the end-user *products* that wire 2+ of them for a tenant are separate
composition surfaces. Homing a tenant product *under* a substrate capability's `facade/` is precisely
the junk-drawer sprawl the §6 membership lint exists to forbid. **The substrate/product split is
confirmed as the governing rule.**

### §2 The fourteen dispositions

| # | Flagged boundary | Ruling | Basis |
|---|---|---|---|
| 1 | comms substrate vs product (messenger/meet/mail) | **comms = the multi-channel comms SUBSTRATE** (messenger-stream, mail-mailbox, meet-media, notifications, contact-center engines + their sold API facades — the Twilio/SES/Chime analog). Confirmed **as-landed**. A future end-user collaboration *product* wiring comms+iam+storage+billing → `app/`; none exists today, so no relocation. (comms/core also holds the calendar + connect-address-book engines; the calendar/connect PRODUCTS — `oya/calendar`, `oya/connect` in `app_products` — are `app/`, while their comms engines stay comms.) | §3#5/§6; the landed crates are substrate engines, not end-user product apps |
| 2 | comms emergency | `oya/emergency` (ED clinical: door-to-balloon/needle, MCI, trauma-alert) → **`app/healthcare`**. The contact-center emergency-caller-bypass stays a `comms/facade` feature. | §3#5 (clinical product composing comms+workflow+iam) |
| 3 | storage drive/recordings | **Confirm `storage/facade/{drive,recordings}` as-landed** — the landed `storage-drive-domain`/`storage-recordings-domain` crates (ADR-0562 §10.9) are the storage substrate's product faces (single-capability, like the Q1 comms engines + Q5 data facades). A future consumer Drive/Recordings *product* wiring storage+iam+comms+billing → `app/` when built; the domain crates stay storage. `storage/facade` also = the S3-analog CAS sold as infra. | §3#6 + tie-breaker (lowest DAG node = storage); §10.9 |
| 4 | imaging (PACS) | `oya/imaging` → **`app/healthcare`** (PACS composes storage+compliance+iam+comms). Already named an `app/` vertical in `app_products_note`. | §3#5 |
| 5 | data product surfaces | `analytics-app`, `warehouse-tenant-olap-service`, `ontology-scorecards-resolver` → **confirmed `data/facade`** (single-capability data-plane product faces, Snowflake/BI analog). | §6; landed correctly under `data/facade` |
| 6 | iam consent | consent = **iam sub-module** (registry §2). The DAG `consent-graph` node is a dependency-ordering forward-declaration, not a separate capability. | registry `iam` boundary_note; ADR-0280 §D-1 |
| 7 | iam identity duplication | `oya/identity` (cloud IdP) → **iam/core**; `oya/oya-identity` (product-shared, consumes core) → **iam/facade**. | registry `iam` boundary_note; §4 face rule |
| 8 | observability diagnostics | `oya/diagnostics` (clinical lab: HL7v2/pathology, HIPAA) → **`app/health-diagnostics`**. The registry auto-absorbed it under `observability` **by name-collision** (clinical "diagnostics" ≠ system diagnostics); the absorb is corrected. | §3#5 (corrects a name-driven placeholder) |
| 9 | intelligence detection | `oya/detection` (fraud/abuse) → **intelligence sub-module** (facade). A different OWNERS team alone does not trigger a split; ADR-0562 §7 requires an OWNERS boundary **and** a clean port seam **and** an ADR amendment. Stays coarse. | registry `intelligence` absorb; §7 |
| 10 | marketplace dev-cli | `dev-cli` (generic gate binary, 122 src, zero marketplace domain logic) → **`ci/`** and is **de-CLI'd** (CLI-retirement: it becomes a gate app/API, not a CLI surface). | §3 rule #6 + ci charter (deployable gate code; rule #3's "CI engines → build/" covers build machinery, and build/ owns no crates) |
| 11 | marketplace vs billing | escrow-reserve / revenue-share-accrue / clawback / payout-settle / tax-form → **billing** (the settlement/metering capability). marketplace keeps discovery/listing/plugin-lifecycle. The SKU/pricing sell-catalog is a **`build/`-generated VIEW**, never a marketplace crate. | registry billing+marketplace charters; §5 |
| 12 | console shell vs app | `console` = the **web-shell SUBSTRATE** (core/ports/facade: shell, token broker, design system, nav). Ops-dashboard leaves composing 2+ capabilities (incident-command, cluster-health, on-call-handoff, …) → **`app/ops-console/<vertical>`**. Single-capability views stay that capability's facade. (The finops-portal SERVICE stays a billing surface per Q11 + the billing absorb; an ops-console finops VIEW merely composes billing+observability.) | §1 (app = web shell); §3#5/§6 |
| 13 | governance | **Do NOT fold `oya/governance` into `compliance`.** It holds zero crates (IaC + SLOs + runbooks + scorecards); its SLOs are **enforcement/CI-pipeline** SLOs (gate-validate-latency, per-lane-runtime-budget, aspirational-enforcement-correctness), not regulatory. It **decomposes** per §3: authority/policy-as-data/conformance → **`governance/`** meta-dir (owns no *capability* crates — the governance-engine crates `libs/oya-check-*`, `libs/oya-governance-*`, `tools/oya-governance-*`, `governance/corpus/*` are already globbed here per §4); gate SLOs → **`ci/observability/slos/`**; the envoy-wasm-filter SLO → **`gateway/observability/slos/`**; `autosharding-events` → **`cell/observability/slos/`**; IaC → **`iac/`** (or the operated service's capability); runbooks → their operated capability. `compliance/` owns only genuine regulatory-evidence runtime (in `oya/compliance`). | §3 rule #2; the actual `oya/governance` content |
| 14 | compliance vs audit | **Keep separate.** `audit` = the always-on tamper-evident Merkle log substrate; `compliance` = the regulatory-evidence engine that *consumes* audit through a clean port seam. A collapse would couple two distinct concerns and needs its own ADR amendment (§7). | registry `compliance` boundary_note; §7 |

**Net effect:** the only real crate move is dev-cli → `ci` (Q10). The drive/recordings landed
`storage/facade` domain crates are **confirmed in place** (Q3) — a future consumer product composing
them → `app/`, but the domain crates stay storage. The three healthcare surfaces (emergency,
imaging, diagnostics) hold **zero crates today** — parking them to `app/healthcare` removes their
name-driven substrate absorb and *reduces* registry drift, not a code move. Twelve of fourteen need
zero or scaffold-only relocation.

### §3 Registry corrections + execution deferral

This ADR amends ADR-0562's `flagged_boundaries_for_leader`: all four questions are **resolved** as
above and the registry records them (`resolved_by: ADR-0615`), and the `storage`/`console`/
`compliance`/`comms` `boundary_note` fields are updated from "FLAGGED … pending OWNERS" to the
ruling.

The **structural** relocations — moving `oya/{emergency,imaging,diagnostics,drive,recordings}` from
their substrate `absorbs_current_dirs` to `app_products.current_dirs`, decomposing `oya/governance`,
and homing dev-cli under `ci/` — are executed **atomically inside each capability's Batch-5
move-plan** (`specs/reorg/<capability>-move-plan.json`), not in this ratification PR. Doing the
absorb-removal here without the corresponding move would orphan the still-present directories against
the membership lint. The ADR is the authority; the moves carry the structure.

### §4 The `governance/` clarification (durable)

`governance/` is a **meta directory off the runtime ladder** (`owns_crates: false` for *capability*
crates), the home of ADRs/specs/policy-as-data/the dep-lint authority/the capability registry/the
masterplan — and the governance-engine crates already globbed to it (`libs/oya-check-*`,
`libs/oya-governance-*`, `tools/oya-governance-*`, `governance/corpus/*`). It is **not** a runtime
`compliance` sub-tree. `compliance` is the regulatory-evidence runtime capability. The two never
merge; the `oya/governance` directory's non-crate residue decomposes to its true homes per §2 Q13.

## Consequences

- The capability axis of the closed registry is now fully ruled: no `boundary_note` remains in the
  "FLAGGED for leader … pending OWNERS" state.
- Batch-5 reorg move-plans can be authored against a decided target: each flagged capability's move
  knows whether a surface stays `facade/` or relocates to `app/`.
- `app/healthcare` (emergency, imaging, diagnostics) and the `app/` consumer surfaces (drive,
  recordings) are decided destinations; they are created when those products are actually built —
  today they are 0-crate scaffolds, so the ruling removes phantom substrate absorbs rather than
  moving code.
- The substrate/product split is now a citable rule (ADR-0615 §1) for every future placement
  question, backing the ADR-0562 §6 membership lint.

## Alternatives considered

- **Home consumer/clinical products under the substrate capability's `facade/`** (keep imaging under
  `storage/facade`, diagnostics under `observability`). Rejected: it violates §6 (a 2+-capability
  tenant product is not a single-capability facade) and reintroduces the junk-drawer the closed
  registry exists to prevent.
- **Fold `oya/governance` wholesale into `compliance`** (the registry's coarse absorb). Rejected:
  its content is enforcement/CI-pipeline + policy-as-data, not regulatory evidence; folding it would
  couple the CI gate's latency SLO to a compliance capability permanently (drift).
- **Collapse `compliance` into `audit`.** Rejected: the clean port seam (always-on log substrate vs
  evidence engine) is worth keeping; a collapse can be revisited by a dedicated amendment if the
  coupling proves dominant.

## Precedent

Google's capability-rooted source tree (`//base`, `//net`, `//storage`) with products composed
above; AWS's substrate services (S3, SES, Chime SDK, Connect) distinct from end-user applications;
Meta's product/domain-rooted top with shared infra below. All root by WHAT a system is; none home a
tenant product inside a substrate's sold surface.
