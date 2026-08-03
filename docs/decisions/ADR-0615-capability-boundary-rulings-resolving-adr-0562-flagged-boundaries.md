---
id: ADR-0615
title: "Capability boundary rulings — resolving ADR-0562's flagged_boundaries (the substrate/product split + the 14 app-vs-capability dispositions)"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-07-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: [ADR-0635]
amends: [ADR-0562]
depends_on: [ADR-0562, ADR-0280]
related: [ADR-0245, ADR-0512, ADR-0139, ADR-0532, ADR-0533, ADR-0536, ADR-0555]
related_specs:
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
milestone: W0
---

# ADR-0615: Capability boundary rulings — resolving ADR-0562's flagged_boundaries

## Status

**Accepted — 2026-07-10** (ratified under the founder's 2026-07-08 autonomous-drive delegation and
an explicit 2026-07-10 ruling on the fourteen boundary questions, with "the long-term-correct,
maintainable, hyperscaler pattern" as the bar; door: one-way — the same one-way class as ADR-0562,
because a surface ruled `app/` vs a capability `facade/` is a placement commitment the membership
lint then enforces). This ADR is **Accepted in the same atomic batch as ADR-0562's Accept**, riding
cross-artifact propagation in that one commit (bound in the masterplan `planning_authority.bound_adrs`
+ the sequencing provenance): it records the founder's decisions and amends ADR-0562's
`flagged_boundaries_for_leader` from open questions to resolved dispositions; it moves no crates.

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

The founder ruled all fourteen on 2026-07-10. This ADR is the durable **record of those decisions**,
**Accepted** in the same atomic batch as ADR-0562 (see Status, above); once it lands, the Batch-5
reorg move-plans implement the dispositions it records.

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
| 8 | observability diagnostics | `oya/diagnostics` (clinical lab: HL7v2/pathology, HIPAA) → **`app/health-diagnostics`**. The registry auto-absorbed it under `observability` **by name-collision** (clinical "diagnostics" ≠ system diagnostics). CLASSIFICATION resolved here; the `observability.absorbs_current_dirs` entry (0-crate) is REMOVED in the observability Batch-5 move-plan, retained until then to avoid orphaning. | §3#5 (corrects a name-driven placeholder) |
| 9 | intelligence detection | `oya/detection` (fraud/abuse) → **intelligence sub-module** (facade). A different OWNERS team alone does not trigger a split; ADR-0562 §7 requires an OWNERS boundary **and** a clean port seam **and** an ADR amendment. Stays coarse. | registry `intelligence` absorb; §7 |
| 10 | marketplace dev-cli | `dev-cli` (generic gate binary, 122 src, zero marketplace domain logic) → **`ci/`** and is **de-CLI'd** (CLI-retirement: it becomes a gate app/API, not a CLI surface). | §3 rule #6 + ci charter (deployable gate code; rule #3's "CI engines → build/" covers build machinery, and build/ owns no crates) |
| 11 | marketplace vs billing | escrow-reserve / revenue-share-accrue / clawback / payout-settle / tax-form → **billing** (the settlement/metering capability). marketplace keeps discovery/listing/plugin-lifecycle. The SKU/pricing sell-catalog is a **`build/`-generated VIEW**, never a marketplace crate. | registry billing+marketplace charters; §5 |
| 12 | console shell vs app | `console` = the **web-shell SUBSTRATE** (core/ports/facade: shell, token broker, design system, nav). Ops-dashboard leaves composing 2+ capabilities (incident-command, cluster-health, on-call-handoff, …) → **`app/ops-console/<vertical>`**. Single-capability views stay that capability's facade. (The finops-portal SERVICE stays a billing surface per Q11 + the billing absorb; an ops-console finops VIEW merely composes billing+observability.) | §1 (app = web shell); §3#5/§6 |
| 13 | governance | **Do NOT fold `oya/governance` into `compliance`.** It holds zero crates (IaC + SLOs + runbooks + scorecards); its SLOs are **enforcement/CI-pipeline** SLOs (gate-validate-latency, per-lane-runtime-budget, aspirational-enforcement-correctness), not regulatory. It **decomposes** per §3: authority/policy-as-data/conformance → **`governance/`** meta-dir (owns no *capability* crates — the governance-engine crates `libs/oya-check-*`, `libs/oya-governance-*`, `tools/oya-governance-*`, `governance/corpus/*` are already globbed here per §4); gate SLOs → **`ci/observability/slos/`**; the envoy-wasm-filter SLO → **`gateway/observability/slos/`**; `autosharding-events` → **`cell/observability/slos/`**; IaC → **`iac/`** (or the operated service's capability); runbooks → their operated capability. `compliance/` owns only genuine regulatory-evidence runtime (in `oya/compliance`). | §3 rule #2; the actual `oya/governance` content |
| 14 | compliance vs audit | **Keep separate.** `audit` = the always-on tamper-evident Merkle log substrate; `compliance` = the regulatory-evidence engine that *consumes* audit through a clean port seam. A collapse would couple two distinct concerns and needs its own ADR amendment (§7). | registry `compliance` boundary_note; §7 |

**Net effect:** the only real crate move is dev-cli → `ci` (Q10). The drive/recordings landed
`storage/facade` domain crates are **confirmed in place** (Q3) — a future consumer product composing
them → `app/`, but the domain crates stay storage. The two clinical surfaces routed to
`app/healthcare` (emergency, imaging) plus the separate clinical-lab surface routed to
`app/health-diagnostics` (diagnostics, §6/Q8 — NOT an `app/healthcare` context) hold **zero crates
today** — parking them removes their name-driven substrate absorb and *reduces* registry drift, not
a code move. Twelve of fourteen need zero or scaffold-only relocation.

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

### §5 The policy extraction — `iam` → `policy`, the 24th capability (registry v1.1.0)

**Ruling (founder, 2026-07-10).** The Cedar-backed **PBAC+ReBAC decision plane** is EXTRACTED from
the coarse `iam` collapse into its own standalone capability **`policy`**, reversing ADR-0562 §2's
`iam/`-absorbs-`policy` grouping. This is an ADR-0562 **§7 split** (coarse-splits-only-by-ADR-
amendment) ridden by this ADR amendment, and it satisfies §7's twin preconditions:

- **OWNERS boundary** — the Cedar PDP / policy plane is `axis-policy-engine`'s, distinct from
  `axis-identity`'s IdP.
- **Clean port seam** — `policy` **consumes** `iam` identity (the verified principal / token) and is
  **consumed by ~all** capabilities' PEPs (the gateway edge + every protected service) for authz
  decisions; the seam is the PDP request/response boundary, not a shared module.

**Face model (ADR-0280 §D-13.D, which already marks `policy` "Standalone ✓"):** `policy` splits into
a **G face** — authoring / signing / distribution of policy + ReBAC tuples — and a **C0 face** — the
per-cell runtime PDP + last-known-good versioned tenant-policy / ReBAC snapshot store. It is NOT a
singleton global PDP; a stale snapshot **denies or routes to the authoritative shard, never silently
authorizes** (the static-stability invariant, §D-13.E).

**iam keeps** identity / credentials / passkeys, product-shared identity, consent(-graph),
tenant-RBAC (the role store/assignments), and workload-identity consumption — and produces the
principal that `policy/` evaluates. **`policy` gets** `oya/policy` (the sole moved path; the Cedar
PDP crates physically under `iam/**` stay iam-mapped to avoid a double-map) and the pre-existing
**`policy-engine` DAG node** (ADR-0280 §D-13; the node already exists with its `→identity`,
`→tenancy`, `→cell` edges and its five `*→policy-engine` consumer edges — no DAG edit needed). After
extraction the registry holds **24 capabilities mapping 1:1 to the 24 DAG nodes** (iam previously
owned two nodes, `identity` + `policy-engine`).

**Precedent.** The hyperscalers keep identity and the authorization decision plane as **distinct
planes**: AWS **IAM ↔ Verified Permissions** (Cedar, store-per-tenant); Google **Cloud IAM ↔
Zanzibar** (logically-global, physically-distributed-with-local-replicas). Homing the PDP inside the
IdP is the coarse convenience the closed registry now corrects.

The registry records this as: a new `policy` capability entry (`dag_node: "policy-engine"`,
`absorbs_current_dirs: ["oya/policy"]`), `oya/policy` + `policy-engine` removed from `iam`, iam's
`dag_nodes` reduced to the single `dag_node: "identity"`, iam's charter/boundary_note de-policied,
and `schema_version` bumped to `1.1.0`.

### §6 The `app/healthcare` product shape (filling the silence)

ADR-0562 named healthcare surfaces as `app/` verticals but did not fix the product boundary; the
founder ruled it 2026-07-10:

- **ONE `app/healthcare` product**, with bounded contexts **emr, pharmacy, patient-monitoring,
  healthcare-integration** (HL7/FHIR ingest), plus the two clinical surfaces ruled here — **emergency**
  (Q2, ED clinical: door-to-balloon/needle, MCI, trauma-alert) and **imaging** (Q4, PACS). It composes
  2+ capabilities (comms + storage + compliance + iam + workflow) for a tenant, so it is `app/`, not a
  substrate facade.
- **`app/health-diagnostics` is a SEPARATE product** (Q8: `oya/diagnostics`, clinical lab —
  HL7v2/pathology, HIPAA). It is deliberately NOT an `app/healthcare` context: the registry auto-
  absorbed it under `observability` by name-collision ("diagnostics" ≠ system diagnostics), and the
  clinical-lab domain is a distinct product bounded context.
- **`social` is a SEPARATE `app/` product**, not a healthcare context (it appears independently in
  the `app_products` roster).

Both healthcare-adjacent destinations (`app/healthcare` and the separate `app/health-diagnostics`)
hold **zero crates today** — the ruling fixes the destination and removes the name-driven substrate
absorbs; the crates land when the products are actually built.

## Consequences

- The capability axis of the closed registry is now fully ruled: no `boundary_note` remains in the
  "FLAGGED for leader … pending OWNERS" state.
- Batch-5 reorg move-plans can be authored against a decided target: each flagged capability's move
  knows whether a surface stays `facade/` or relocates to `app/`.
- `app/healthcare` (emr, pharmacy, patient-monitoring, healthcare-integration, emergency, imaging),
  the SEPARATE `app/health-diagnostics` product (Q8, clinical lab — NOT an `app/healthcare` context),
  and the `app/` consumer surfaces (drive, recordings) are decided destinations; they are created
  when those products are actually built — today they are 0-crate scaffolds, so the ruling removes
  phantom substrate absorbs rather than moving code. (§6 fixes the `app/healthcare` product shape;
  diagnostics is `app/health-diagnostics` per Q8, not `app/healthcare`.)
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
