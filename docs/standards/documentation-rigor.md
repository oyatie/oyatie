---
purpose: "Canonical intern-buildability bar for every doc class. Layered on top of doc-style.md (Diátaxis quadrants, RFC-2119, frontmatter) — this standard adds the depth/density/cross-reference requirements that let an intern with programming skill but zero prior architecture knowledge build the described system from documentation alone."
doc_status: published
---

---
doc_class: Standard
shape: Reference
length_cap: 600
authority_tier: 2
status: Accepted
date: 2026-05-20
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json + docs/standards/doc-style.md
planned_enforcement_ref: oya-governance-doc-rigor
companion_docs:
  - docs/standards/doc-style.md
  - docs/STANDARDS-AND-TEMPLATES.md
  - docs/templates/adr-template-v2.md
  - docs/templates/runbook-template-v2.md
  - docs/products/_TEMPLATE.md
  - docs/templates/design-doc-template.md
related_adrs:
  - ADR-0053
  - ADR-0063
  - ADR-0212
  - ADR-0242
  - ADR-0255
related_memories:
  - autonomous-implementation-artifacts
  - quality-performance-scalability-bar
  - doc-coverage-enforced
---

# Documentation Rigor — The Intern-Buildability Bar

## Doctrinal authority

Layered on top of [`doc-style.md`](doc-style.md) (style, Diátaxis quadrants, RFC-2119 normative language) and the [`STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) catalog. Where this standard sets a *higher* bar than doc-style.md or a template, this standard wins for the listed doc classes.

## Applicability — existing and new docs alike

This standard applies **retroactively to every canonical doc in the corpus**, not just docs authored after 2026-05-20. Every existing ADR, PRD, spec, standard, runbook, onboarding guide, user-stories file, architecture walkthrough, and migration playbook MUST be audited against §1.1 (hyperscaler-grade sub-test), §1.2 (engineering-rigor dimensions matrix), §2 (doc-class rigor matrix), and §3.1 (six-hops graph-traversability invariant) and upgraded where any (M)-cell fails. The CI lane `oya-governance-doc-rigor` reads every doc under `docs/`, `microservices/*/`, `packs/*/`, `specs/`, and `crates/*/docs/` — not just newly-touched ones. Lane is **advisory until 2026-07-15** to give the corpus-wide upgrade pass time to land; **BLOCKER from 2026-07-16**. No doc gets a grandfather clause.

The upgrade order is:
1. Hub docs first: `docs/README.md`, `docs/AGENTS.md`, `docs/DOC-CATALOG.md`, `docs/STANDARDS-AND-TEMPLATES.md`, `docs/GLOSSARY.md`. Without rigor on hubs, downstream traversability fails.
2. Keystone ADRs (any ADR in the 2026-05-20 bundle).
3. Every µservice's PRD, README, runbook index, and contracts.
4. Every standards doc under `docs/standards/`.
5. Every runbook under `docs/runbooks/`.
6. Every user-stories doc under `docs/user-stories/`.
7. Every spec under `/specs/` and `microservices/*/contracts/`.
8. Onboarding under `docs/onboarding/`.
9. Architecture walkthroughs under `docs/architecture/`.
10. Migration playbooks under `docs/migrations/`.

Pre-existing docs that pre-date the doctrinal substrate (e.g., 2024-2025 vintage) are not exempt — they upgrade or they retire per [`feedback_markdown_retirement_policy`](../specs/markdown-retirement-policy.json). "Legacy" is not a status; canonical or retired.

### Completeness invariants — everything accounted for

The corpus is complete when ALL of the following hold:

1. **Every µservice has the full doc set — PR-143 baseline floor, ~100+ artifacts.** PR #143 (2026-05-17 flat-layout substrate + 17 µservice audit-grade packs, 1,515 artifacts) is the **minimal baseline**. Every µservice MUST ship ≥70 artifacts mirroring the observability template (the canonical exemplar at `microservices/observability/`, currently 132 artifacts = 188% of baseline). The roster:

   **Mandatory artifacts per µservice (the PR-143 ~70-artifact roster):**
   - **Strategic docs (4):** `PRD.md`, `PHASE-NN-*.md` (≥1 phase doc), `threat-model.md`, `dpia.md`.
   - **Architecture & ops docs (10):** `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md`, `capacity-model.md`, `cost-budget.md`, `failure-modes.md`, `multi-region.md`, `incident-response.md`, `backfill-replay.md`, `compliance.md`.
   - **Product positioning (2):** `competitor-parity-matrix.md`, `sdk-plan.md`.
   - **Policy / Cedar (≥6):** `policy/*.cedar` (≥4 Cedar v4.2 LTS fragments: default-deny + defence-in-depth FORBID), `policy/data-residency.md`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`.
   - **Runbooks (≥6):** `runbooks/*.md` — each meets §2 runbook rigor (Trigger/Pre-checks/Procedure/Verification/Rollback/Post-incident/References).
   - **Contracts (≥3):** `contracts/openapi-v1.yaml` (OpenAPI 3.2.0), `contracts/asyncapi-v1.yaml` (AsyncAPI 3.1.0), `contracts/<bc>-v1.proto` (proto3) for gRPC surfaces; plus convention docs e.g. `contracts/metric-naming-convention.md`.
   - **Capabilities (≥3):** `capabilities/*.yaml` per the v2 capability-record schema.
   - **Dashboards (≥3):** `dashboards/*.json` (Grafana) + `dashboards/*.md` (cross-reference docs).
   - **SLOs (≥4):** `slos/*.openslo.yaml` (per ADR-0130 agentic SLO-gated promotion).
   - **Implementation plans (≥15):** `IP-NNN-*.md` covering the build sequence; each IP is a single-PR-sized atomic deliverable.
   - **Catalog records (≥11):** `catalog/*.yaml` — one per crate per BC×layer in the µservice (the µservice-registry-diff side per ADR-0246's catalog discipline).
   - **IaC (≥8):** `iac/*.{tf,yaml,jsonnet}` covering the µservice's K8s manifests, Helm values, Terraform module, OpenBao policy, network policy, secret bindings.
   - **Manifest (1):** `manifest.json` declaring µservice tier (substrate vs product), audience type, dependency graph, layer enum conformance (per ADR-0105 13-layer), BC roster, ownership, SLO tier.
   - **Audit findings (1):** `AUDIT-FINDINGS-<date>.json` rolled per audit pass, retained as evidence.
   - **Scorecards (1):** `scorecards/overrides.json` for the standard scorecard family.

   **Above-and-beyond signal:** µservices that exceed the baseline (PR-143 observability ships 132 artifacts at 188%) are the operating bar. New µservices SHOULD target ≥100 artifacts. The ~70 number is the floor; ≥100 is the operating bar; ≥130 is the rigor we ship.

   Missing baseline files trigger `oya-governance-microservice-doc-set` BLOCKER (advisory until 2026-07-15, BLOCKER thereafter). Aggregate corpus completeness reported by `oya-governance-doc-completeness` daily.

   **Worked exemplar:** `microservices/observability/` — read this directory tree before drafting any new µservice. The shape MUST match.

   **Corpus snapshot 2026-05-20 (the retroactive gap to close):** The PR-143 ~70-floor and the ≥100 operating bar apply to every existing µservice, not only to new ones. The 2026-05-20 audit of `microservices/*/` shows the following gaps:

   | Tier | Count | µservices |
   |---|---:|---|
   | **Below floor (<70 artifacts)** — full doc-set buildout required | 6 | `payments` (1), `api-gateway` (16), `feature-flags` (16), `intelligence` (17), `connector` (18), `ops-dashboard-control-center` (36) |
   | **Borderline (70–99 artifacts)** — gap-fill to operating bar required | 8 | `compliance` (73), `tenancy` (79), `comms-email` (81), `finops-portal` (85), `ontology` (88), `mail` (94), `notes` (99), `social` (99) |
   | **At/above operating bar (≥100)** — §1.1/§1.2/§3.1 rigor audit required (no new artifacts mandated, but existing ones must clear the bar) | 32 | `shorts` (100) through `foundry` (561), all 32 µservices currently in this band |
   | **Above-and-beyond exemplar (≥130)** | 8 | `observability` (146), `cloud-iac` (150), `governance` (172), `workflow-studio` (198), `foundry` (561), plus 3 more crossing the threshold |

   Total upgrade workload: **14 µservices need artifact-count remediation** + **32 µservices need rigor audit against §1.1/§1.2/§3.1**. The `oya-governance-doc-set-completeness` lane reports the per-µservice gap daily and gates the platform's promotion-to-GA milestones per ADR-0250.

   Upgrade ordering: below-floor µservices first (they cannot serve production traffic until they meet ADR-0212 buildability doctrine), then borderline, then audit pass on the 32 at-bar µservices. The ordering is *not* alphabetical — substrate µservices (cell, tenancy, policy-engine, cloud-secrets, foundry) are remediated before product µservices (community, marketplace, etc.) because substrate gaps propagate.

   No grandfather clause. No "this µservice was scaffolded before the bar landed" exception. Canonical or retired.

   ### Artifact-count tier ≠ ADR-adherence pass

   Two orthogonal axes determine whether a µservice meets the bar:

   - **Axis A — artifact count:** Below floor / borderline / at-bar / above-and-beyond. Measures *quantity* of the doc set.
   - **Axis B — ADR-adherence:** The 27-row §3.2.1 matrix verifying conformance to the keystone bundle 2026-05-20 (ADR-0242..0258 + 0263 + 0272-0292 + 0293-0296 + amendments). Measures *quality of conformance*.

   **A µservice can be above-and-beyond on Axis A (130+ artifacts) and still REVISE on Axis B.** A µservice scaffolded before 2026-05-20 may have rich documentation but lack the per-row answers in §3.2.1 (e.g., it may not declare `provider_credential_mode`, may not cite ADR-0246 amendment library-first dispatch, may use pre-amendment ontology read-path semantics, may emit audit events outside the ADR-0263 registry, may hard-code `oyatie` strings against ADR-0284).

   Per the 2026-05-20 corpus snapshot, the 8 above-and-beyond exemplars (observability, cloud-iac, governance, workflow-studio, foundry, plus 3 more) **MUST be ADR-adherence-audited just as rigorously as the below-floor µservices**. The audit may surface a long ADR-adherence gap list even when artifact count is 5× the floor. Resolution: edit-in-place to add the missing declarations + amend `ARCHITECTURE.md` / `compliance.md` / `manifest.json` / `migrations/` to match the 27-row matrix.

   The full upgrade workload across the 46 µservices is therefore:

   - **6 below-floor µservices:** Axis A buildout (artifact set) **+** Axis B ADR-adherence wiring.
   - **8 borderline µservices:** Axis A gap-fill **+** Axis B ADR-adherence wiring.
   - **24 at-bar µservices (100-129):** Axis B ADR-adherence audit + remediation (Axis A already passes).
   - **8 above-and-beyond exemplars (≥130):** Axis B ADR-adherence audit + remediation (Axis A already passes; **no grandfather clause for ADR-adherence**).

   All 46 µservices get an Axis B audit. The exemplars are not exempt; their artifact-count head start does not exempt them from the keystone bundle 2026-05-20's authority. Equally rigorous, no shortcuts.
2. **Every primitive has a binding ADR.** No primitive (Postgres table, Cedar entity type, Rust trait, Kubernetes CRD, OpenAPI route, AsyncAPI channel, principal slug, audit event class) exists in code or spec without a binding ADR cited in source comments / spec `_meta.binding_adr` / Cedar fragment metadata.
3. **Every primitive has a spec.** Every Postgres table is in a JSON Schema + DDL pair; every Cedar fragment has a `cedar-fragment-schema.json`-compliant manifest; every API has OpenAPI 3.2.0; every channel has AsyncAPI 3.1.0; every event has an Avro/JSON Schema event-schema in `event-schema-versioning-canonical.md`'s registry.
4. **Every operational primitive has a runbook.** If a primitive can fail, drift, or need rotation, it has a runbook at `docs/runbooks/<slug>.md` meeting §2 runbook rigor. Cross-referenced from the binding ADR's §F.
5. **Every public contract has a versioning + deprecation policy.** OpenAPI/AsyncAPI/event/ABI/CLI contracts declare their SemVer policy + deprecation cadence + sunset rules per ADR-0258.
6. **Every term used is defined.** Every term appearing in ≥2 canonical docs has a glossary entry at `docs/GLOSSARY.md` with: definition, first-introduced ADR, hyperscaler analog, related terms.
7. **Every cross-reference resolves.** No `[broken link]`. No unresolved placeholder markers in canonical doc bodies. CI lane `oya-governance-doc-link-resolves` enforces.
8. **Every cell-tier / compliance-pack / sovereign-cloud variant is documented.** For each µservice and each primitive: which packs activate it, which cell tiers it deploys to, which sovereign-cloud overlays apply.
9. **Every retired doc is retired explicitly.** Files removed from canonical scope get a tombstone entry in `docs/retired/` per the markdown-retirement-policy. No silent deletions.
10. **No orphan files.** Every file under `docs/`, `microservices/`, `packs/`, `specs/`, `crates/*/docs/` is reachable via the §3.1 graph traversal OR is explicitly tombstoned. CI lane `oya-governance-doc-orphan-detection` enforces.

The completeness invariants apply to the corpus as a whole, not per-PR. The `oya-governance-doc-completeness` aggregate lane reports the corpus-wide gap count daily and gates the platform's promotion-to-GA milestones (per ADR-0250 build-ahead-of-certification).

## 1. The intern-buildability test

Every canonical doc MUST pass this thought-experiment:

> Given **only this doc + the docs it references**, could a programming-capable intern with **zero prior oyatie architecture knowledge** build the described system / execute the described procedure / configure the described surface correctly on the first attempt, and would the result be **hyperscaler-grade** (i.e., indistinguishable in rigor from what AWS / Google Cloud / Stripe / Palantir / Cloudflare would ship)?

A doc that requires the reader to "ask someone" or "see also Slack" or "the real source-of-truth is the code" **fails the bar** and MUST be upgraded before its enforcement lane promotes to BLOCKER.

The test is not "could a senior engineer figure it out." It is "could an intern build it from cold." The cost of an extra paragraph is bounded; the cost of tribal knowledge is unbounded.

### 1.1 The hyperscaler-grade rigor sub-test

Beyond intern-buildability, the produced system MUST meet the rigor a hyperscaler would ship. Concretely, every canonical doc that describes a primitive, surface, or procedure MUST exhibit ALL of:

1. **Named precedent.** At least one explicit "this is the X pattern from Y" citation (e.g., "this is the AWS S3 bucket-replication pattern", "this is the Stripe platform-facilitator pattern", "this is the Palantir Foundry ontology projection pattern"). No invented architectures without a named precedent.
2. **Failure-mode tree.** Every primitive enumerates ≥3 failure modes (network partition, byzantine actor, regional outage, key compromise, etc.) and the system's behavior in each — not just the happy path.
3. **Capacity math.** Every capacity claim is backed by a derivation (Little's Law, binomial probability, percentile arithmetic, queue-theory steady-state). No "should be enough" hand-waves.
4. **Observability hooks.** Every primitive declares its emitted audit events, traces, metrics, and logs — not "we'll add observability later".
5. **Rollback path.** Every change to running state has an explicit rollback procedure — not "restore from backup".
6. **Multi-region awareness.** Every globally-scoped primitive declares its behavior across regions, including the failure mode when a region is unreachable.
7. **Sovereign-cell awareness.** Every primitive that touches PII / payments / regulated data declares its behavior under KR-CSAP, EU-sovereign, CN-PIPL, IL5/6, FedRAMP-High packs — not "compliance is somebody else's problem".
8. **Versioning + deprecation.** Every public contract (API, schema, event, ABI) declares its versioning model and deprecation cadence.

A doc that hits intern-buildability but misses any of items 1-8 is **APPROVE-WITH-FINDINGS at best, REVISE at worst**. Hyperscaler-grade is the floor, not the ceiling.

### 1.2 Engineering-rigor dimensions

Every canonical doc that describes a system, primitive, or surface MUST address all six engineering-rigor dimensions below where applicable. The applicability column lists which doc classes the dimension is mandatory for (M), recommended for (R), or N/A (-).

| Dimension | What it means | Applies to | Acceptance signals |
|---|---|---|---|
| **Maintainability** | A future maintainer (intern or veteran) can change this without breaking adjacent surfaces. | ADR (M), PRD (M), Spec (M), Standard (M), Runbook (R) | Explicit module boundaries; versioning policy; deprecation cadence; "what is hard-coded vs configurable"; per-config-flag rationale; reverse dependencies enumerated. |
| **Observability** | The system can be debugged, monitored, and audited from production telemetry alone. | ADR (M), PRD (M), Spec (M), Standard (M), Runbook (M) | Every primitive declares emitted metrics (name, type, dimensions, cardinality budget), traces (span shape, parent-child rules), logs (level, schema, retention class), audit events (per ADR-0263 emission contract). SLO floor named. Dashboards named or stub-linked. |
| **Scalability** | The primitive serves 10× and 100× the current load without architectural change. | ADR (M), PRD (M), Spec (R), Runbook (R) | Capacity math (Little's Law / queue theory / binomial); explicit bottleneck identification + the shard / partition / cache strategy that resolves it; horizontal-scale-out path; explicit "the system goes red when X exceeds Y" thresholds. |
| **Performance** | Latency, throughput, jitter, tail behavior are quantified and budgeted. | ADR (M), PRD (M), Spec (R), Runbook (R) | P50/P95/P99 targets with error bars; modeling note OR benchmark commit; per-region budget split; tail-latency mitigation (hedging, fan-out-and-take-first, circuit-breakers); cold-start budget. No bare percentile claims. |
| **Optimization** | The cost-performance frontier is examined; lazy/eager/cache trade-offs are named. | ADR (M), PRD (R), Spec (R), Runbook (-) | Explicit per-call cost model (CPU-µs, RAM-MB, IOPS, $/M-requests); explicit "we picked lazy because X" or "we picked eager because Y"; cache-invalidation policy if any; cold-vs-warm path latency separation; profiling-evidence link if applicable. |
| **Code quality** | The implementation has tests, error handling, types, lints; the doc names the quality bar. | ADR (M), PRD (M), Spec (M), Standard (M) | Required test classes (unit / property / fuzz / load / e2e); coverage floor (≥85% line, ≥75% branch); lint passes named (`oya-check-*`); type-strictness level (Rust deny(warnings), TS strict, etc.); SemVer + ABI policy. |

A canonical doc that fails any **(M)** cell for its doc class is **REVISE**. The §4 audit rolls up the dimension grades into the overall verdict.

This six-dimension matrix MUST be present in every ADR's §C or §E (consequences / implementation footprint), every PRD's §E (non-functional requirements), every Spec's `_meta` block, and every Standard's body. Runbooks satisfy Observability + Maintainability via their §F Post-incident + §G References sections.

## 2. Doc-class rigor matrix

Each doc class MUST meet the minimum content density, section count, and cross-reference shape below. Numbers are floors, not ceilings — exceed where the topic warrants.

| Doc class | Min lines | Required sections | Density signals | Forbidden |
|---|---:|---|---|---|
| ADR (decision) | 1500 | A Context / B Decision / C Consequences / D Detailed mechanics (D-1..D-N) / E Implementation footprint (file paths, crates, schemas) / F Migration / G References | ≥2 hyperscaler precedent citations; ≥1 anti-pattern call-out per decision; concrete DDL/Cedar grammar/Rust trait shape when introducing primitives; cross-references both inbound (who cites this ADR) and outbound (which ADRs this ADR cites) | "placeholder marker" / "see code" / "left as exercise" / aspirational latency without measurement |
| Amendment ADR | 1000 | A Context (what's being amended and why) / B Decision delta / C Consequences delta / D Detailed mechanics delta / E Migration / F References | ≥1 hyperscaler precedent per delta; cross-reference the amended ADR by section; explicit "what does NOT change" paragraph | Re-stating the base ADR; vague "clarification" prose |
| PRD (product requirements) | 1500 | A Problem / B Target users (B2C personas + B2B personas) / C User stories (≥40 stories spanning all surfaces) / D Functional requirements / E Non-functional requirements / F UX flows / G Success metrics / H Compliance impact / I Open questions / J Out-of-scope | ≥40 user stories; ≥6 UX flow diagrams (ASCII or links); explicit personas with goals + frustrations; metric thresholds (P50/P95/P99 latency, conversion %, retention %); compliance-pack mapping | Single-persona PRDs; metric-free "delight the user" prose; out-of-scope section that lists nothing |
| Spec (machine-readable, JSON Schema) | 600 | `$schema`, `$id`, `title`, `version`, `_meta` (purpose + industry_citations + related_adrs + status + enforcement_status), `type`, `required`, `properties` (every field with `description` ≥1 sentence + ≥1 example), `oneOf`/`allOf`/`if-then` constraints where applicable | Every property has `description` + `examples`; cross-references the binding ADR via `_meta.binding_adr`; passes `python -c "json.load(open(...))"` validation; passes `ajv validate` if a meta-schema exists | Bare `type: object` with no `properties`; properties without descriptions; missing `_meta` block |
| Runbook | 250 | A Trigger conditions / B Pre-checks / C Procedure (numbered, ≥10 steps) / D Verification / E Rollback / F Post-incident / G References | Every step has: command/API surface OR Cedar permit OR OpenBao path; timing budget (≤Ns); audit-stream tag emitted; explicit "if this step fails" branch; cross-reference to ≥2 related runbooks | "Notify on-call" without channel + escalation list; rollback section that says "restore from backup"; bare prose with no commands |
| Standard | 250 | Doctrinal authority paragraph / numbered sections / examples / forbidden-patterns table / CI lane name / cross-reference to companion docs and ADRs | ≥1 RFC-2119 MUST / SHOULD / MAY per section; explicit "what good looks like" example block; explicit "anti-pattern" block | "Best practices" lists without specificity; standards that don't name an enforcement lane |
| Onboarding | 1000 | A Day 0 (laptop bring-up) / B Day 1 (first commit) / C Week 1 (first slice owned) / D Month 1 (first incident shadow) / E Glossary / F Common pitfalls / G Escalation channels | Every step ends in a verifiable artifact (a passing test, a merged PR, an answered question); every link works; every command shown is paste-runnable | "Talk to your onboarding buddy"; commands without expected output; assumed-knowledge gaps |
| User stories | 2000 | A Personas (≥6) / B Surface-by-surface stories (each story: As X / I want Y / so that Z + Acceptance criteria + UX sketch link) / C Edge cases per surface / D Anti-stories (what we explicitly refuse to do) | ≥120 stories total across B2C and B2B; ≥3 anti-stories per surface; explicit accessibility AC per story; explicit i18n AC per story | Stories without acceptance criteria; persona-free stories; surface gaps |
| Architecture deep-dive / walkthrough | 1500 | A Entry point (cold-start question) / B Layer-by-layer trace / C Concrete example end-to-end / D Common confusions / E Where to read next | Step-by-step trace of one real flow with file paths + line numbers; one diagram per layer; explicit hyperscaler analog ("this is like AWS X for Y reason") | Bullet-point feature lists masquerading as architecture; diagrams without sequence/context |
| Migration playbook | 500 | A Why / B Before-state / C After-state / D Step-by-step (with rollback per step) / E Validation / F Sunset / G Risk register | Every step has rollback; every state delta is observable; explicit version-bump policy; explicit ADR + lane references | Big-bang migrations with no per-step rollback; migrations without sunset date |

## 3. Cross-reference shape

Documents form a connected graph. Standalone docs are anti-patterns.

**Every canonical doc MUST:**
1. Cite the binding ADR(s) in frontmatter (`related_adrs:` list).
2. Cite the companion docs in frontmatter (`companion_docs:` list) — at least the catalog entry that lists this doc.
3. Cite ≥1 inbound source — a doc that points *to* this doc, so the graph remains traversable from any node.
4. Cite ≥1 hyperscaler / industry precedent if the doc introduces a non-trivial primitive.
5. Cite the enforcement lane that gates its quality (`planned_enforcement_ref:` or `enforced_by:`).

A doc with zero outgoing references SHOULD be treated as a leaf — confirm it is actually a leaf (e.g., a glossary terminator), or add references.

### 3.1 Graph-traversability invariant

From **any** canonical entry point (`docs/README.md`, any ADR, any PRD, any standards doc), an intern MUST be able to reach every primitive used in the implementation by following cross-references in ≤6 hops. This is the **six-hops invariant**.

- Entry points include: `docs/README.md`, `docs/AGENTS.md`, `docs/DOC-CATALOG.md`, `docs/STANDARDS-AND-TEMPLATES.md`, the keystone-bundle synthesis doc, every µservice's PRD.md, every ADR.
- Reachability is BFS-traversal over markdown links + frontmatter `related_*` + `companion_docs` lists.
- A primitive is "reached" when an intern lands on the doc that *introduces* or *normatively specifies* it — not merely a doc that mentions it.
- Verified by a deterministic graph-walker (`tools/doc-graph-walker/`); CI lane `oya-governance-doc-graph-6hops` (advisory until 2026-07-15, BLOCKER thereafter).

If an intern reads ADR-0244 (tenant model) and wants to understand provider-BYOK, they MUST be able to reach ADR-0255 §D-4 in ≤2 hops via `provider_credential_mode` cross-reference. If they read a runbook and need the Cedar permit for the operation, they MUST reach the relevant ADR-0243 §D-N permit definition in ≤3 hops. If they read a PRD and need the compliance-pack manifest schema, they MUST reach `specs/compliance-pack-schema.json` in ≤3 hops.

The graph is the documentation product. Each canonical doc is a node. Each cross-reference is an edge. The graph MUST be:
- **Connected** — no isolated subgraphs.
- **Strongly-connected on hubs** — `docs/README.md` reachable from every node; every node reachable from `docs/README.md`.
- **Bidirectional** — for every "A cites B" there is either a direct or a cataloged "B is cited by A" reference (via `inbound_citations:` frontmatter list or via the catalog's reverse-index).
- **Acyclic on supersession** — supersedes/superseded-by chains are DAGs; no cycles.

## 3.2 ADR-adherence + consistency invariants

Every µservice (existing and new) MUST adhere to the keystone bundle 2026-05-20 ADRs + the four F5-CRITICAL fix ADRs + the F4 library-first amendments. Adherence is verified per the following matrix; each row is a per-µservice question with a documented answer in the µservice's `ARCHITECTURE.md` or `compliance.md`.

### 3.2.1 Per-µservice ADR-adherence checklist

For every µservice, the following questions MUST have explicit answers (not "see code"):

| # | ADR | Question | Answer location |
|---:|---|---|---|
| 1 | ADR-0242 (oyatie-is-a-tenant) | What principals under `oyatie.*` does this µservice operate as? What tenant-scoped principals call it? | `ARCHITECTURE.md §principals` |
| 2 | ADR-0243 (Cedar universal gate) | Which Cedar fragments gate this µservice's actions? Where is the default-deny baseline? | `policy/*.cedar` + `ARCHITECTURE.md §cedar-gates` |
| 3 | ADR-0244 (tenant scoping) | Which tables / events / rows carry `tenant_id`? What `audience_type` does this µservice serve? What `provider_credential_mode` does it honor? | `ARCHITECTURE.md §tenant-scoping` + `migrations/` |
| 4 | ADR-0245 (substrate vs product) | Substrate or product? If substrate, which products consume it? If product, which substrates it depends on? | `manifest.json:tier` + `ARCHITECTURE.md §substrate-product-binding` |
| 5 | ADR-0246 + amendment (policy-engine library-first) | Does it use the caller-side `oya-shared-policy-eval` library? What's its `policy_evaluation_mode`? | `ARCHITECTURE.md §policy-evaluation` |
| 6 | ADR-0247 (self-modification doctrine) | Does this µservice produce or consume self-modification artifacts? If so, what's its meta-trust-root attestation path? | `compliance.md §self-modification` |
| 7 | ADR-0248 (cellular architecture) | Which cell tier (0/1/2/3) does it deploy to? What's the per-cell shard width? Which cells does it span? | `multi-region.md` + `manifest.json:cell_eligibility` |
| 8 | ADR-0249 (multi-category marketplace) | Does it expose marketplace surfaces? If so, which categories? | `competitor-parity-matrix.md` + `ARCHITECTURE.md §marketplace` |
| 9 | ADR-0250 (build-ahead-of-certification) | What certification levels does it ship-ready-for on day-one? | `compliance.md §day-one-cert-readiness` |
| 10 | ADR-0251 + CN-PIPL-2021 pack | Which compliance packs activate it? Which pack overlays are required? Including CN-PIPL-2021 when relevant. | `compliance.md §pack-overlay-roster` |
| 11 | ADR-0252 (HLC + TrueTime) | Which time-coordination tier? HLC default, TrueTime for which BCs? | `ARCHITECTURE.md §time-coordination` |
| 12 | ADR-0253 (HTTP/3 + QUIC default; fallback chain HTTP/3 → HTTP/2 → HTTP/1.1; strict TLS; ECH; PQC) | Confirms HTTP/3 default across REST + AsyncAPI surfaces. Negotiation order MUST be HTTP/3 > HTTP/2 > HTTP/1.1 — first acceptable wins; never skip a tier; HTTP/1.0 forbidden. TLS MUST be strict (TLS 1.3 floor, full chain validation, HSTS `max-age≥63072000; includeSubDomains; preload`, certificate-transparency required, OCSP stapling, no MITM-bypass headers); no `insecure_skip_verify` anywhere; no `tls.MinVersion < 1.3`; no self-signed certs in any tier except offline-rooted-CA ceremony per ADR-0295. **ECH (Encrypted Client Hello, RFC 9460 + draft-ietf-tls-esni-22) enabled wherever the platform terminates TLS** — publish HTTPS RR with `ech=` config in DNS via the per-tenant DKIM/SPF/DMARC ADR-0273 toolchain; serve ECH config-id on every Tier-0/1/2/3 cell ingress; rotate ECH keys per `docs/runbooks/cedar-fragment-emergency-rollback.md` cadence (≥90d default). ECH-disabled clients fall through to standard TLS 1.3 without breakage (graceful degradation). **PQC (post-quantum hybrid) enabled wherever the client+server pair both negotiate it** — KEM hybrid `X25519MLKEM768` (per draft-kwiatkowski-tls-ecdhe-mlkem-02 + IANA codepoint `0x11ec`) preferred; signature hybrid `ed25519+ml_dsa_65` for new certificate chains issued by oyatie-rooted CAs (sigstore + Fulcio supply-chain doctrine); non-PQ clients fall through to classical curves (X25519 / P-256) without breakage. Both ECH and PQC follow the "where possible" rule — advertise + negotiate where supported; degrade silently where not; never refuse a session because peer lacks PQC/ECH (refusing would break adoption-curve). Per-µservice document: which Alt-Svc / `h3` advertisement does it serve, what does its h3→h2 fallback look like under QUIC-blocked networks, what's its TLS profile (cipher suites, curve preferences, AEAD-only), whether ECH is advertised on this surface, whether PQC hybrid is offered in the ClientHello / ServerHello. | `contracts/openapi-v1.yaml` + `ARCHITECTURE.md §transport` + `iac/<env>-ingress.yaml` + `iac/<env>-ech-config.yaml` + `iac/<env>-pqc-cert.yaml` |
| 13 | ADR-0254 (deployment model + Cloud Hypervisor) | K8s + Cloud Hypervisor + Kata pods. Which µservice components are Wasm vs container vs VM? | `iac/` + `ARCHITECTURE.md §deployment-shape` |
| 14 | ADR-0255 + amendment (intelligence two-layer) | Does it call Intelligence? If so, library-first or network-opt-in? What audience tag is set per call? | `ARCHITECTURE.md §intelligence-dispatch` |
| 15 | ADR-0257 + amendment (ontology read-path) | Does it read the Ontology? Library-first or network-only? What's its `ontology_read_mode`? `freshness_floor`? | `ARCHITECTURE.md §ontology-read-path` |
| 16 | ADR-0258 (API versioning) | Which SemVer policy + deprecation cadence applies? | `contracts/*.yaml:info.version` + `CHANGELOG.md` |
| 17 | ADR-0263 (observability emission contract) | Which audit-event-classes does it emit? Cardinality budget per metric? Trace span shape? | `dashboards/*.json` + `ARCHITECTURE.md §observability` |
| 18 | ADR-0272 (cookie consent per-purpose) | If user-facing: per-purpose consent surface? | `compliance.md §consent` |
| 19 | ADR-0273 (per-tenant DKIM/SPF/DMARC) | If mail-emitting: per-tenant deliverability? | `compliance.md §email-deliverability` |
| 20 | ADR-0276 (backup portability GDPR Art. 20) | What's the per-tenant backup-export format? | `backfill-replay.md §portability` |
| 21 | ADR-0280 (substrate-of-substrate dependency) | What's its substrate-dependency DAG position? | `manifest.json:substrate_dependencies` |
| 22 | ADR-0284 (platform-owner name indirection) | Has it migrated off any hard-coded `oyatie` strings? | grep audit + `compliance.md §platform-owner-indirection` |
| 23 | ADR-0292 (minor user doctrine) | If consumer-facing: COPPA <13 refusal, KOSA 14-17 tier, EU age-verification | `compliance.md §minor-protection` |
| 24 | ADR-0293 (meta-trust-root) | If Foundry-touching: meta-trust-root attestation path | `compliance.md §meta-trust-attestation` |
| 25 | ADR-0294 (Cedar fragment soak) | If publishing Cedar fragments: ≥60s soak window respected | `policy/*.cedar` headers + `ARCHITECTURE.md §fragment-publish` |
| 26 | ADR-0295 (bootstrap CI SPIFFE + kill-switch) | If bootstrap-tier-1: SPIFFE attestation + kill-switch wiring | `compliance.md §bootstrap-trust-chain` |
| 27 | ADR-0296 (library-first credential sidecar) | If holding any provider credential: sidecar isolation OR ≤60s OpenBao TTL | `ARCHITECTURE.md §credential-isolation` |
| 28 | ADR-0297 (abuse-defence: anti-bot + anti-spoof + anti-scrape) | If internet-facing: which anti-bot, anti-spoof, anti-scrape controls are wired? See §3.2.3 below for the mandatory taxonomy. | `ARCHITECTURE.md §abuse-defence` + `iac/<env>-edge-waf.yaml` + `policy/abuse-defence.cedar` |

A µservice that answers fewer than 28 of these is REVISE. The audit lane `oya-governance-adr-adherence-matrix` reads every µservice's `ARCHITECTURE.md + compliance.md` and reports per-row pass/fail (advisory until 2026-07-15, BLOCKER thereafter).

### 3.2.2 Cross-µservice consistency invariants

The corpus is consistent when:

1. **Field naming is consistent.** Every `tenant_id`, `principal_id`, `audit_event_class`, `byok_enabled`, `provider_credential_mode`, `home_cell`, `dr_cell`, `jurisdiction_code`, `compliance_packs[]`, `audience_type`, `lifecycle_state` field is the same shape across every µservice's schema + Cedar + OpenAPI surface.
2. **Audit-event-class taxonomy is consistent.** Every µservice's emitted event classes are listed in the central registry per ADR-0263 §D-N; no µservice-private event classes outside the registry.
3. **OpenAPI is uniformly 3.2.0; AsyncAPI is uniformly 3.1.0; proto3 only.** Per `specs/api-contract-ssot-canonical.json` (canonical contract-version source: OpenAPI 3.2.0 + proto3 projections, AsyncAPI 3.1.0 event contracts; the retired `tools/hooks/_canonical-primitives.md` cheat sheet is disposed). Any contract file at a different version is REVISE.
4. **OpenBao SecretReference path shape is consistent.** `${openbao:secret/<tenant_id>/<scope>/<name>}` across every µservice.
5. **Cell-tier-conformance is consistent.** Every µservice declaring `cell_eligibility` uses the same enum values from ADR-0248 §D-1 (Tier 0 / Tier 1 / Tier 2 / Tier 3).
6. **Compliance-pack-id values are consistent.** Every µservice's `compliance.md §pack-overlay-roster` cites pack-ids from the central registry; no ad-hoc pack-ids.
7. **Layer-enum values are consistent.** Every crate's layer slug matches ADR-0105's 13-layer canonical set; no per-ADR forks (closed by A1-BL-1 fix).
8. **Naming-justification tables are present.** Every ADR and every µservice's `manifest.json` carries the naming-justifications block per `feedback_naming_justification`.
9. **Six-hops graph traversal works on every entry point.** Per §3.1.
10. **BYOK terminology is consistent.** Every doc that mentions BYOK disambiguates provider-BYOK vs encryption-BYOK (per the 2026-05-20 scope split).

CI lane `oya-governance-cross-consistency` enforces invariants 1-10 daily.

### 3.2.3 Abuse-defence baseline — anti-bot, anti-spoof, anti-scrape

Every internet-facing µservice MUST wire the following defence-in-depth controls. The bar is: a determined adversary with hyperscaler-grade resources (bot farms, reflection / amplification, leaked credentials, residential-proxy networks, AI-driven CAPTCHA solvers) cannot succeed via volumetric, credential-stuffing, or scraping attacks. Controls are documented in `ARCHITECTURE.md §abuse-defence`, encoded in `iac/<env>-edge-waf.yaml`, and gated by `policy/abuse-defence.cedar`.

#### Anti-bot controls (MUST)

| # | Control | Tier | Notes |
|---:|---|---|---|
| 1 | **Edge rate-limiting (per-IP, per-fingerprint, per-tenant, per-route)** | Tier-0 edge | Token-bucket + sliding-window; burst caps per route class (auth, write, read, admin) |
| 2 | **Behavioural fingerprinting** | Edge | TLS JA4 / JA4+ / HTTP/2-3 frame-pattern fingerprint; passive — never alone gates a request |
| 3 | **Bot-management with ML scoring** | Edge | Cloudflare Bot Management / Akamai Bot Manager / in-house equivalent at parity; score forwarded to µservice as header `X-Oya-Bot-Score` for downstream policy |
| 4 | **CAPTCHA-on-suspicion** | Edge | hCaptcha + Turnstile + Cloudflare Challenge; presented only when bot-score crosses threshold; **never on default path** (accessibility floor) |
| 5 | **Device attestation** | Edge / app | App Attest (iOS), Play Integrity (Android), WebAuthn Origin-binding (web); for native + signed-in surfaces |
| 6 | **Stolen-credential check** | Auth path | HIBP API / oyatie's internal credential-stuffing-detector; pause sign-in if password appears in dump corpus |
| 7 | **Per-action quota gates** | µservice Cedar | Cedar-evaluated quota gate per action class; bot-score + quota composed; quota persists across IP rotation via tenant-id binding |
| 8 | **Honeypot routes + canary payloads** | µservice | Routes that no legitimate client should hit; canary payloads (fake API keys, fake user-ids) seeded into surface to detect scrapers that ingest them |

#### Anti-spoof controls (MUST)

| # | Control | Layer | Notes |
|---:|---|---|---|
| 1 | **Email anti-spoof** | Domain | DKIM + SPF + DMARC (p=reject post-rollout) + ARC + BIMI; per-tenant via ADR-0273 |
| 2 | **Domain anti-spoof / cert pinning** | TLS | Strict TLS 1.3 (per row 12); CT-required; HSTS preload; for native apps, `expect-ct` + cert pinning on production cert; DNS-over-HTTPS for resolver path |
| 3 | **Identity anti-spoof** | Auth | Step-up auth classes per `docs/standards/step-up-auth-classes.md`; WebAuthn passkeys preferred; phishing-resistant MFA for high-risk operations |
| 4 | **Session anti-spoof** | Auth | HMAC-signed session tokens with audience binding; SameSite=Strict cookies; rotating session-id on privilege escalation; bound to TLS exporter (token-binding RFC 8473 where supported) |
| 5 | **Payload anti-spoof** | API | Signed payloads for webhooks (per ADR-0273-style HMAC); for native + machine clients, mTLS or signed JWT bound to client identity |
| 6 | **Audit-trail anti-spoof** | Audit | Per ADR-0263: every emitted audit event is signed by the per-µservice signing key in the sidecar (per ADR-0296); audit chain is Merkle-sealed per ADR-0028 audit-chain doctrine; no in-µservice forgery possible |
| 7 | **Webhook anti-spoof** | Inbound | HMAC signature verification on every inbound webhook; replay-window ≤5min; idempotency-key required |
| 8 | **Caller anti-spoof (workload identity)** | mTLS | SPIFFE workload identity per ADR-0295; every µservice-to-µservice call carries SVID; Cedar gate verifies caller identity before action |

#### Anti-scrape controls (MUST)

| # | Control | Layer | Notes |
|---:|---|---|---|
| 1 | **Rate-limit per-tenant + per-fingerprint** | Edge | Aggressive low caps on unauthenticated read endpoints; higher caps for authenticated tenants per their tier |
| 2 | **Pattern-anomaly detection** | Edge / µservice | Detect breadth-first crawl signatures (sequential IDs, alphabetical pagination), high-page-depth fan-out, parallel-tab signatures |
| 3 | **robots.txt + Sitemaps + crawl-delay** | Edge | Authoritative robots.txt per-tenant + per-locale; reject misbehaving crawlers by user-agent + behavioural pattern |
| 4 | **Paid-API tier for legitimate scrapers** | API gateway | Offer a paid API surface for legitimate bulk consumers (search engines, data-aggregators); ToS-of-service crawlers gated by accepted terms |
| 5 | **Content fingerprinting / per-user watermarking** | µservice | High-value content carries per-user invisible watermark (zero-width chars, image steganography, audio-watermark) so leaked corpora identify the source |
| 6 | **Adaptive challenge on scrape-pattern** | Edge | Bot-score + scrape-pattern + tenant-policy → adaptive challenge (CAPTCHA, JS-execution proof-of-work, throttle-then-degrade) |
| 7 | **Dynamic content rewriting** | µservice | CSS class names randomised per session; structural HTML mutated; semantic API surface remains stable, scraping surface does not |
| 8 | **Legal-channel registration** | Out-of-band | Public Bug Bounty surface + abuse-report email + DMCA agent + GDPR Article 14 right-to-object surface; bots that ignore robots.txt + accept ToS get DMCA + civil action |

#### UX floor — defence-in-depth MUST NOT sacrifice UX

The bar: a legitimate user with no bot signals MUST experience the same smooth, frictionless flow they would experience with no abuse-defence wired. Friction is imposed *only when bot-score crosses threshold* — never as a default-path tax.

| UX invariant | Requirement | Anti-pattern |
|---|---|---|
| **Default path is friction-free** | Bot-score below threshold → request flows with zero added latency, zero UI changes, zero re-auth challenges. ML scoring is passive and asynchronous. | "CAPTCHA on every login" / "JS proof-of-work on every page load" |
| **Latency budget unchanged** | Edge bot-mgmt + behavioural fingerprinting + JA4 add ≤2ms p99 to the default-path latency. Hard CI gate on this. | "Sync to Cloudflare API on every request" / "Synchronous HIBP check on every login" |
| **Accessibility floor (WCAG 2.2 AA)** | Every challenge (CAPTCHA, JS PoW, device-attest) has accessible alternative: audio CAPTCHA, keyboard-only navigation, screen-reader-friendly challenge text, no time pressure for assistive-tech users. Cross-ref `docs/standards/a11y-canonical.md` + `docs/standards/wcag-2-2-aa-checklist.md`. | Image-only CAPTCHA with no audio fallback / Time-pressured challenge / Mouse-required challenge |
| **Cognitive load floor** | Challenge presented MUST be solvable in ≤10 seconds by a legitimate user. Multi-step challenges only for confirmed-high-score bot-suspicion. | 10-tile reCAPTCHA on every page / "Find all the buses" challenges with adversarial images |
| **Session continuity** | Successful challenge resolution preserves session state — user does NOT lose form data, cart contents, scroll position, or get logged out. Challenge UX is a *modal overlay*, not a navigation. | Challenge resolves → redirected to login → form data lost |
| **Tenant-tier-adaptive sensitivity** | Higher-tier authenticated tenants get smoother experience (lower bot-mgmt sensitivity, fewer challenges). Anonymous + sandbox-tier tenants get strictest baseline. | "Everyone same sensitivity" → smooth for adversaries; harsh for paying users |
| **Locale-aware challenge** | CAPTCHA + challenge UI follows tenant locale (per ADR-0064 canonical-base + localization); challenge text translates; image challenges use locale-appropriate imagery. | English-only challenge for KR/JP/CN tenants |
| **Mobile UX parity** | Native iOS + Android challenge UX uses platform-idiomatic surfaces (App Attest + Play Integrity); no web-CAPTCHA shoved into mobile webviews. | "Image-grid CAPTCHA on iPhone" |
| **Error recovery is graceful** | Failed challenge → clear remediation path; never a dead end. Recovery options: try a different challenge type, switch device, retry with cooldown, contact support. | "You failed too many times. Bye." |
| **Friendly-crawler-partner allow-list bypass** | Search engines + accredited researchers + a11y-tooling get explicit allow-list bypass via `audience_type = FRIENDLY_CRAWLER_PARTNER`; never see a challenge. | Google search bot blocked from indexing |
| **Performance during scrape-pattern detection** | Adaptive challenge MUST throttle bots without slowing legitimate concurrent traffic to the same surface. Per-fingerprint isolation, not per-route. | Bot scraping `/products/*` → all `/products/*` slows for everyone |
| **Transparent telemetry to tenants** | Tenant-admin dashboards show the abuse-defence outcomes affecting their own tenant: false-positive rate, friction events, blocked-bot count. Per ADR-0263 audit-event surface. | Black-box abuse defence with no tenant visibility |

UX-friction CI lane: `oya-governance-abuse-defence-ux-floor`. Verifies that the default-path latency budget holds, that every challenge in `policy/abuse-defence.cedar` has documented a11y alternatives, and that no µservice's abuse-defence config sets default-path friction.

The bar is: **security AND UX, not security OR UX**. Hyperscaler precedent: Stripe Radar (passive scoring; visible only on confirmed-suspicion), Cloudflare Turnstile (replaces visible CAPTCHA with invisible challenge for ~95% of legitimate traffic), Apple App Attest (silent device attestation), WebAuthn passkeys (phishing-resistant AND smoother than passwords).

#### LIFE-SAFETY HARD RULE — emergency-services NEVER see a challenge

**A challenge presented to an emergency worker is a defect — potentially a fatal one.** Abuse-defence MUST be silently bypassed for:

| Class | Examples | Bypass mechanism |
|---|---|---|
| **First responders** | EMS paramedics, firefighters, law enforcement on duty, search-and-rescue, hazmat | `audience_type = EMERGENCY_SERVICES`; jurisdiction-registered principal |
| **9-1-1 / NENA i3 / E-CAD dispatchers** | 9-1-1 PSAP operators, dispatchers, RapidSOS integrations | NENA-i3-registered ESInet edge identity; carrier-verified SIP-TRUNK origin |
| **Healthcare acute-care surfaces** | ER triage, ICU monitoring, code-blue alerts, telemedicine urgent-care, EHR break-glass | `audience_type = EMERGENCY_SERVICES` + HIPAA-eligible cell + break-glass Cedar permit |
| **Crisis hotlines** | 988 Suicide & Crisis Lifeline (US), Samaritans (UK), 1393 (KR), 7700 (JP), domestic-violence hotlines, child-safety reporting, sexual-assault crisis lines | `audience_type = EMERGENCY_SERVICES`; crisis-line federated identity |
| **Public-safety mass-notification** | Wireless Emergency Alerts (US WEA), EU-Alert, KR Public Safety Cell Broadcast, JP J-ALERT | Carrier-pinned origin + cosign-signed payload; never challenged |
| **Disaster-response surfaces** | Red Cross / FEMA / OCHA field-tools, search-and-rescue ICS surfaces, refugee-registration | `audience_type = EMERGENCY_SERVICES` + disaster-mode tenant pack |
| **Eldercare / fall-detect / medical-alert** | Life Alert, Apple Watch fall-detect, Galaxy SOS, ADT health monitoring | Per-device-attested origin + EMS escalation path |
| **Child safety reports** | NCMEC CyberTipline submissions, NSPCC reports, KR-ChildSafe, GDPR/COPPA-required minor-protection reports | NCMEC-registered origin; mandatory-reporter cert |

**Bypass mechanics (MUST):**

1. **No challenge under any circumstance.** Not even on bot-score saturation. Not even on credential-stuffing-score saturation. Not even on rate-limit overrun. Emergency-services traffic gets through.
2. **Bypass at the edge, not at the µservice.** The Cedar policy `policy/abuse-defence.cedar` evaluates `audience_type == "EMERGENCY_SERVICES"` BEFORE any score gate; the bypass is L3/L4-cheap, not L7-expensive.
3. **Bypass does NOT skip audit.** Every emergency-services request is still tenant-scoped + tenant_id-stamped + audit-emitted per ADR-0263. The audit-event-class is `AbuseDefenceEmergencyServiceBypass` (added to the ADR-0263 registry follow-up). Bypass + audit = accountability without friction.
4. **Bypass identity is cryptographically attested.** `audience_type = EMERGENCY_SERVICES` is set ONLY when the principal is:
   - A NENA-i3-registered ESInet edge identity (for 9-1-1 dispatch)
   - A carrier-verified SIP-TRUNK origin (for emergency-call routing)
   - A jurisdiction-registered emergency-services tenant (per ADR-0244 reserved-namespace registration)
   - A NCMEC-registered submitter, FEMA-registered responder, NCMEC CyberTipline origin, or equivalent jurisdictional emergency-services registry
   - A federated identity from a known crisis-line operator (988 Lifeline / Samaritans / KR-1393 / etc.)
5. **Forgery defense is rotation + revocation, not friction.** A forged `EMERGENCY_SERVICES` claim must be detectable via the attestation chain (SPIFFE workload identity per ADR-0295 + cosign-signed federated trust + per-jurisdiction registry sync). On suspected forgery: revoke that single attestation in real-time at the trust root; do NOT challenge other legitimate emergency-services traffic.
6. **Rate-limit floor is elevated, not zero.** Emergency-services tenants get a much higher rate-limit floor (default 10× consumer tier); a truly broken / compromised emergency-services tenant cannot DoS the platform, but normal emergency traffic surges (mass-casualty incident, natural disaster) are handled without friction. Use circuit-breaker per ADR-0254 — open the circuit on per-tenant pathology, never on tenant audience-type.
7. **Crisis-hotline bots can legitimately operate.** Crisis-line text-bot triage (988 chat, Samaritans chat, Crisis Text Line) routes legitimate automated intake — bot-score must NOT block them. They're identified by `audience_type = EMERGENCY_SERVICES + bot_class = CRISIS_TRIAGE` (a sub-class) and pass without challenge.
8. **Mobile UX parity.** Native-app emergency surfaces (iOS Emergency SOS, Android Emergency SOS, KakaoTalk emergency-call integration, LINE 1-1-9 integration) attest via App Attest + Play Integrity; the abuse-defence stack treats those as elevated-trust origins.
9. **Geographic graceful degradation.** When ECH / PQC / advanced-TLS-1.3 features cause incompatibility with a legacy emergency-services dispatch system, the platform negotiates DOWN to whatever the dispatch system supports — NEVER refuses the session. (Cross-ref documentation-rigor.md §3.2.1 row 12 graceful degradation principle.)
10. **Test in chaos exercises.** Quarterly chaos-test: simulate a mass-casualty incident (10× normal 9-1-1 volume) and verify zero challenge events on emergency-services traffic. CI lane `oya-governance-emergency-services-chaos-test` enforces.

**Forbidden patterns (MUST NEVER):**

- "Show a CAPTCHA on the 988 chat surface to prevent abuse" → NEVER. Crisis-line chat is a life-safety surface.
- "Rate-limit the EMS dispatch API to 100 req/s" → No. EMS dispatch traffic surges during mass-casualty events; elevated floor + tenant-isolation circuit-breaker, never rate-limit at the surface-level.
- "Require step-up auth for ER break-glass" → No. Break-glass workflow per ADR-0247 has its own post-hoc audit-and-justify pattern; pre-action friction is forbidden.
- "Quietly degrade the 9-1-1 caller's location accuracy under load" → No. RFC 8147 (location-routing requirements for emergency calls) requires authoritative location; degrade other features first.
- "Show a TLS warning for an emergency-services client that doesn't support PQ" → No. Negotiate down silently per row 12 graceful degradation.

**Why this is a hard rule:** A 911 caller blocked by CAPTCHA is a life lost. A paramedic delayed by step-up auth at a code-blue is a life lost. A crisis-line caller routed through a JS PoW challenge during a suicidal episode is a life lost. The cost of false-negative bot-detection on the emergency path is bounded (single incident, audited, recoverable); the cost of friction on the emergency path is unbounded (irreversible loss of life). Make the trade-off explicit and pick the right side.

**Regulatory anchors:**

- US: FCC 47 CFR §9 (E9-1-1 / NG9-1-1), NENA i3 Standard for NG9-1-1 (NENA-STA-010.3), HHS 988 Suicide & Crisis Lifeline rules, NCMEC CyberTipline (18 USC §2258A mandatory reporter)
- EU: EECC (European Electronic Communications Code) Article 109 Emergency Communications, eIDAS Article 24, EU-Alert (Article 110)
- KR: 통신비밀보호법 (Communications Secrets Protection Act), 위치정보의 보호 및 이용 등에 관한 법률 (Location Information Act), 119 Emergency Service operational mandate
- JP: 119 Emergency Service, J-ALERT mandate
- AU/NZ: Triple Zero (000), Emergency+ app integration
- UK: 999 / 112, BT EHA (Emergency Handling Agreement)

**New ADR required:** `docs/decisions/ADR-0709-general-live-apex.md` is added to the Wave-3 backlog. It codifies the bypass mechanics + the `audience_type = EMERGENCY_SERVICES` extension to ADR-0244 + the trust-attestation chain + the new CI lanes + the regulatory anchors above.

**Cross-references that need updating when ADR-0298 lands:**
- ADR-0244 `audience_type` ENUM: add `EMERGENCY_SERVICES` value (joins `FRIENDLY_CRAWLER_PARTNER`, `MINOR_TARGETED`, `INTERNAL_DEV_TOOLS` from ADR-0297 follow-ups)
- ADR-0297 §D-7: add `EMERGENCY_SERVICES` audience-type tuning row (zero-friction baseline + elevated rate-limit + crisis-triage-bot allow-list)
- ADR-0263 audit-event registry: add `AbuseDefenceEmergencyServiceBypass` + `EmergencyServiceForgeryDetected` + `EmergencyServiceRateLimitElevation` classes
- ADR-0292 minor doctrine: cross-link crisis-line bypass for minors (988 + Crisis Text Line + KR-1393 may accept under-13 with no parental-consent requirement)
- §3.2.1 row 28 + §3.2.3 above: cross-link the emergency-services bypass as the highest-priority hardrule

### 3.2.5 Critical-path edge-case coverage matrix

Emergency-services is the most consequential critical path, but it is **not the only one**. Every µservice MUST enumerate the critical paths it serves and document edge-case handling for each. Across the corpus, the critical paths are:

| # | Critical path | Why it's critical | Special handling (MUST) | Safety/security/policy invariant |
|---:|---|---|---|---|
| 1 | **Emergency services** | Life-safety | Per §3.2.3 above; ADR-0298 | Bypass at edge; audit retained; cryptographic attestation; forgery-revocation not friction |
| 2 | **Account recovery / lockout** | Locked-out legitimate users; phishing-resistance must not become user-resistance | Multi-factor recovery (passkey backup + recovery code + delegated trusted contact); cool-down + step-up; never permanent lockout without ombudsman path | Phishing-resistant ≠ user-hostile; recovery is auditable + tenant-policy-bounded |
| 3 | **Financial fraud dispute + chargeback** | Wrongful charge or stolen-card; victim must be heard fast | Pre-charged-back fast-track path; per-PSP chargeback API integration; per-pack (PCI / KR-FSS) reg-mandated timing (15-day acknowledge, 60-day resolve) | Per-pack mandatory timing honored; audit retained for QSA pull; tenant cannot stonewall consumer |
| 4 | **Elder financial abuse** | Cognitively-impaired users targeted by scammers | Cooling-off period on large transfers; trusted-contact alerts; per-tenant config + per-pack (FINRA Rule 4512) | Trusted-contact opt-in; alert is informational not blocking; respects autonomy |
| 5 | **Healthcare urgent care + EHR break-glass** | Acute medical scenario; break-glass must not require pre-action approval | Per ADR-0247 break-glass pattern: post-hoc audit-and-justify; HIPAA-eligible cell; PHI access logged + reason-coded | Break-glass audit MUST be cryptographically sealed + non-repudiable per ADR-0028 |
| 6 | **Whistleblower + ethics report** | SOX 806, EU Whistleblower Directive (2019/1937), KR Anti-Corruption Act, US Dodd-Frank §922 | Anonymous-submission surface; never tied to caller identity by µservice; per-pack overlay for jurisdiction; chain-of-custody to ombudsman | Anonymity preserved E2E; tenant admin CANNOT see submitter; metadata-minimized + sealed-sender |
| 7 | **Press freedom / journalist source** | First-Amendment / Article-19 / KR-Press-Freedom protections | Per-tenant SecureDrop-class option; Tor-friendly ingress; no IP-log + no IP-correlation; per-pack overlay (e.g., publisher tenant + tenant-pack `publisher-source-protection`) | No metadata retained beyond minimum-required; per-jurisdiction reporter-privilege honored |
| 8 | **Domestic violence / abuse survivor** | Stalkerware risk; controlling abuser may share device | Silent shelter mode (incognito session); hide-from-shared-device option; per-tenant escape-plan + alert-when-checked-by-other-party; no SMS-MFA fallback (predictable) | Survivor has unilateral power to lock out abuser even if abuser holds shared credentials; audit visible only to survivor |
| 9 | **Child safety + mandatory reporting** | 18 USC §2258A (US) + KR-Children-Protection Act + EU-CSAM-Reg + UK-Online-Safety-Act | NCMEC CyberTipline integration; mandatory-reporter cert routing; minor self-report path (no parental consent required for safety report); no parental-control surface can suppress | Minor's safety report MUST reach mandatory reporter regardless of parental-control config; audit trail preserves chain-of-custody |
| 10 | **Deceased-user account** | Beneficiary access; legal-rep access; per-jurisdiction inheritance law | Legacy-contact (Apple-class) + legal-rep court-order ingress + per-jurisdiction inheritance overlay; explicit per-tenant pre-mortem wish honored | DSAR-cascade per ADR-0276 honors deceased-user wish; legal-rep access requires court order; tenant cannot unilaterally lock out heirs |
| 11 | **Custody / shared-account dispute** | Separated parents accessing child-shared accounts; legal custody status | Per-jurisdiction family-court order integration; per-account cooling-off on custody-disputed accounts; child-best-interest standard | No parent can unilaterally lock the other out without court order; child's surface preserved |
| 12 | **Disability accommodations (beyond a11y floor)** | Assistive-tech-only users, voice-control-only, single-switch users, dementia, post-trauma | Per-tenant accessibility-profile honored; alternative auth methods (voice biometric, single-switch, longer time budgets); a11y-floor + tenant-extension | WCAG 2.2 AAA target on accommodation paths; per-tenant override of friction-defaults |
| 13 | **Non-native-language user** | Translation breaking auth or critical workflows | Locale fallback chain (preferred → English → translate-on-fly); critical surfaces (auth, financial, medical) NEVER auto-translated without consent | Translation is opt-in for sensitive surfaces; per-locale UX parity (no second-class locales) |
| 14 | **Low-bandwidth / disaster-zone / offline-first** | Rural + post-disaster + satellite-internet users; intermittent connectivity | Offline-first sync (CRDT per `oya-collab-crdt-portability-kernel`); progressive enhancement; degraded mode preserves core workflow | Offline-mode audit retained + reconciled; no data loss on resync; per-tenant offline-quota |
| 15 | **Banking / financial inclusion** | Unbanked, gig workers, remittance recipients, undocumented | Cash-out paths (Toss / KakaoPay / WeChat / mobile money); no SSN-required for low-tier consumer; per-pack jurisdiction overlay | Per-pack KYB/KYC tier respects regulator floor (not above-and-beyond by default); financial-inclusion path doesn't strand under-banked |
| 16 | **Activist / dissident in authoritarian jurisdiction** | High-risk users in countries with state-level surveillance | Tor-friendly ingress; metadata-minimization mode; per-tenant `audience_type = HIGH_RISK_USER` overlay; no cross-border data export unless tenant-opted | E2EE preserved; metadata minimal; per-pack `pack-cn-pipl` + per-tenant override permitted within regulator floor |
| 17 | **Service outage during regulator-deadline** | NIS2 72-hour notification, GDPR 72h, KR-PIPA 72h | Degraded-mode preserves regulator-required action; per-pack breach-notification-workflow continues via DR-pair cell | Regulator deadlines honored even during outages; audit retained; per-pack workflow has graceful-degraded path |
| 18 | **Audit / regulator / law-enforcement access** | Court orders, FOIA, GDPR Art. 15 DSAR, KR-PIPA Art. 35 audit, FedRAMP continuous-monitoring | Read-only audit-surface per-pack overlay; per-jurisdiction warrant gating; per-tenant transparency report | Regulator gets audit-grade evidence; tenant gets transparency report; per-pack warrant-canary surface |
| 19 | **Tenant break-glass / dead-account recovery** | Locked-out tenant-admin; dead-key Shamir reconstitution; transfer-of-ownership | Per-pack ombudsman path (oyatie council-security 2-member quorum); Shamir reconstitution per ADR-0247; per-tenant pre-mortem wishes | Break-glass audit-trail cryptographically sealed; multi-jurisdiction Shamir threshold honored; no unilateral platform action against tenant |
| 20 | **Cognitive-impairment / intoxication / post-trauma** | User in altered state making consequential decisions | Slow-down on high-value decisions; trusted-contact alerts; cool-down on rapid sequential mutations; per-tenant config + a11y opt-in | Respects autonomy; never blocks; informational nudges only; per-jurisdiction guardianship-law overlay |
| 21 | **Pseudonymous + privacy-by-default users** | Users who legitimately need pseudonymity (gay rights activists, hate-target groups, victims) | Per-tenant pseudonymity-preservation; KYB tier separated from public identity; cross-reference protection | Pseudonymity-class principal scope; audit-trail accessible only to authorized tier; per-jurisdiction legal-name-required compliance |
| 22 | **Mass-casualty incident / disaster-zone surge** | 10× normal traffic; tenant-isolation must hold while platform absorbs surge | Elevated emergency-services rate-limit floor; per-cell DR-pair failover; per-pack disaster-mode | Cell-isolation per ADR-0248 preserved; per-tenant SLO degrades gracefully; emergency-services NEVER throttled |
| 23 | **Cross-jurisdiction conflict** | EU GDPR conflicts with US CLOUD Act; KR-PIPA conflicts with US subpoena; CN-PIPL conflicts with EU GDPR | Per-pack data-residency hard-stop; per-tenant jurisdictional preference + per-pack regulator floor | Higher-restriction pack wins; cross-border transfer requires multi-pack alignment; transparency-report cites all applicable packs |
| 24 | **Account-hijack victim recovery** | Stolen credentials, SIM-swap, OAuth-token-theft | Phishing-resistant passkey + hardware-key recovery; per-tenant SIM-swap detection (telco signal); 72h cool-down on high-value mutations post-recovery | Recovery is auditable + non-repudiable; legitimate user gets back fast; hijacker cannot piggy-back; per-pack timing honored |
| 25 | **Mistaken-action / unintended-mutation recovery** | User accidentally deletes / sends / pays | Undo-window (15s default; per-tenant configurable); per-surface "are you sure" only for irreversible high-value mutations; per-pack mandatory cool-down (e.g., HIPAA delete cool-down) | Undo is fast (≤15s); confirmation is rare (only irreversible); cool-down per pack required when regulator-mandated |
| 26 | **Concurrent-session conflict (e.g., abuse, custody)** | Two parties holding same credentials but conflicting interests | Per-session conflict-detection (geo, device, behavioural); per-jurisdiction trusted-contact alert; ombudsman path | Both parties informed; neither unilaterally locked out without due process; per-pack regulator notification |
| 27 | **Bug-bounty + responsible-disclosure submitter** | Researcher reports vuln; must not be blocked by abuse-defence | Per-tenant `audience_type = SECURITY_RESEARCHER` allow-list; `/.well-known/security.txt` per RFC 9116 | Bug bounty submitter not challenged on submission path; safe-harbor per platform policy |
| 28 | **Bot / agent acting on behalf of human (LLM agent, IFTTT, n8n workflow)** | Legitimate automation; bot-mgmt may flag false-positive | Per-tenant `delegated_agent_token` model; tenant-attested delegation chain; bot-mgmt sees attestation and allows | Delegated agent inherits tenant scope; cross-tenant delegation blocked; audit chains delegated principal back to authorizing human |
| 29 | **High-net-worth tenant + transaction limits** | Legitimate large transactions blocked by per-tenant generic limits | Per-tenant explicit transaction-tier (KYB-verified high-net-worth); per-pack regulator (e.g., FINRA, KR-FSS) tier | Tier set at KYB time + auditable; never exceeded without re-KYB; regulator-floor honored |
| 30 | **Service degradation during regional outage** | Single-region cell out; tenants in that cell need continuity | DR-pair failover per ADR-0241; cross-region replication where pack allows; per-pack data-residency hard-stop respected | Failover preserves audit + per-pack data-residency; per-tenant SLO tier defines recovery time |

#### Edge-case coverage rule (per µservice)

Every µservice's `compliance.md §critical-path-edge-cases` MUST list the rows above that apply to its surface, with:
- The µservice's specific handling per row
- Cross-reference to the binding ADR + runbook + Cedar policy
- The CI lane that verifies the handling

CI lane: `oya-governance-critical-path-coverage` reads `compliance.md §critical-path-edge-cases` per µservice and verifies the applicable rows are addressed.

#### The safety-security-policy invariant

Every edge case MUST simultaneously honor:

1. **Safety** — no human harm; life-safety > everything; cognitive autonomy respected; child-best-interest paramount.
2. **Security** — no creating a bypass that adversaries can exploit; attestation + audit + revocation, not friction; defense-in-depth preserved.
3. **Policy adherence** — per-pack regulator floor honored; per-tenant policy bounded by pack; cross-jurisdiction conflict resolved by higher-restriction-wins.

A handling that achieves one but sacrifices another is REVISE. A handling that finds the design that achieves all three is the bar.

**Forbidden anti-patterns:**
- "We'll just add a CAPTCHA on the recovery path" → No. Account recovery is critical-path #2; locked-out user is the safety concern.
- "Whistleblower submissions go through normal auth" → No. Submission must be anonymous; binding to caller identity is the safety violation.
- "Domestic-violence shelter mode shares audit trail with tenant admin" → No. Survivor controls their audit visibility; tenant admin cannot stalk.
- "Crisis-hotline minors require parental consent" → No. Child safety > parental control on mandatory-reporting paths.
- "Disaster-zone mass-traffic gets generic rate-limit" → No. Emergency-services elevated floor + cell isolation; never throttle 911.

#### New ADRs required (added to Wave-3-D backlog)

- `ADR-0299-account-recovery-resilience.md` (covers row 2 + 24 — account recovery + hijack-recovery)
- `ADR-0300-whistleblower-press-freedom-anonymity.md` (covers rows 6, 7, 16, 21)
- `ADR-0301-survivor-safety-domestic-abuse-mode.md` (covers row 8)
- `ADR-0302-deceased-user-inheritance-doctrine.md` (covers row 10)
- `ADR-0303-cognitive-impairment-decision-resilience.md` (covers rows 4, 20)
- `ADR-0304-cross-jurisdiction-conflict-resolution.md` (covers row 23)
- `ADR-0305-delegated-agent-authority-chain.md` (covers row 28)
- `ADR-0306-disaster-mode-cell-resilience.md` (covers rows 14, 22, 30)

These join ADR-0297 (abuse-defence) and ADR-0298 (emergency-services) as the **critical-path doctrine cluster** for the keystone bundle 2026-05-20.

### 3.2.6 DRMP — Detection + Risk + Mitigation + Prevention baseline

§3.2.3 (abuse-defence) + §3.2.4 (hyperscaler defense-in-depth) + §3.2.5 (critical-path edge cases) layer in here as the **Detection → Risk → Mitigation → Prevention (DRMP) lifecycle**. Every threat / fraud / abuse / policy-violation surface MUST be addressed across all four phases — not just detected, not just prevented, but the full loop.

**The DRMP lifecycle (MUST cover all four phases for every threat class):**

```
                  ┌─────────────────────────────────────┐
                  │                                     │
                  ▼                                     │
   ┌───────────────────────┐                     ┌─────────────────┐
   │  PREVENTION (proactive)│                    │  MITIGATION      │
   │  -- pre-action friction│                    │  -- contain      │
   │  -- choke points       │                    │  -- notify       │
   │  -- nudges + education │                    │  -- remediate    │
   │  -- defense-in-depth   │                    │  -- recover      │
   └───────────────────────┘                     └─────────────────┘
              │                                            ▲
              ▼                                            │
   ┌───────────────────────┐                     ┌─────────────────┐
   │  DETECTION             │ ─── signal ───────▶ │  RISK SCORING   │
   │  -- streaming + batch  │                     │  -- composite    │
   │  -- rules + ML         │                     │  -- explainable  │
   │  -- pattern + anomaly  │                     │  -- per-jurisd.  │
   └───────────────────────┘                     └─────────────────┘
              ▲                                            │
              │                                            │
              └────── feedback (analyst label + outcome) ──┘
```

Detection without mitigation = signal noise. Mitigation without prevention = whack-a-mole. Prevention without detection = trust in static defenses. Detection without risk = unprioritized alert flood. All four phases required, all four cross-referenced.

#### Subsections layout

- §3.2.6.A — Detection categories (8 families)
- §3.2.6.B — Risk scoring (composite + fairness)
- §3.2.6.C — Mitigation taxonomy (containment + notification + remediation + recovery)
- §3.2.6.D — Prevention taxonomy (pre-action friction + choke points + nudges + defense-in-depth)
- §3.2.6.E — ML model lifecycle (per EU AI Act + NIST AI RMF + ISO/IEC 42001)
- §3.2.6.F — Observability (per ADR-0263)
- §3.2.6.G — UX-floor + policy-adherence invariants
- §3.2.6.H — Per-µservice obligation
- §3.2.6.I — CI lanes + new ADRs

#### 3.2.6.A — Detection categories (read this as previously written)

Every µservice handling transactions, identity, content, or audit-trail-relevant action MUST contribute to + consume from the detection substrate. The detection substrate itself is a substrate µservice (`microservices/detection/` — flagged for Wave-3-D buildout).

#### Detection categories — 8 families

| # | Family | Targets | Substrate components | Per-µservice obligation |
|---:|---|---|---|---|
| 1 | **Payment fraud** | Card-not-present, stolen-card, friendly fraud, refund fraud, chargeback fraud, BIN attacks, velocity abuse | Per-PSP risk score (Stripe Radar, Adyen RevenueProtect, Toss riskOps), in-house composite scorer, graph-based fraud-ring detection | Payments µservice MUST emit risk-score on every charge + dispute; emit `PaymentRiskScoreEmitted` event |
| 2 | **Account-takeover (ATO)** | Credential stuffing, SIM-swap, OAuth-token theft, session-hijack, phishing-landing | Behavioural fingerprint drift, geo-impossibility, device-change-after-auth, password-reset-velocity | Identity µservice MUST emit ATO signal on suspicious sign-in; cross-ref §3.2.5 row 24 |
| 3 | **Synthetic identity** | Manufactured KYC, deepfake selfie, AI-generated docs, mule-account farming | Document liveness check + face-match + KYB graph correlation + Bureau-data cross-check | Identity + KYB µservices MUST emit synthetic-identity signal at onboarding |
| 4 | **AML + sanctions** | Money laundering structuring, sanctions evasion (OFAC/EU/UN/KR-MOFA), terrorist financing, PEP screening | Transaction-graph analysis, sanctions-list match, PEP enrichment, suspicious-activity threshold | Payments µservice MUST emit AML signal on threshold-crossing; per-pack regulator-floor; Cross-ref §3.2.4 Domain 9 sanctions lists |
| 5 | **Content abuse** | CSAM, terrorism, hate speech, non-consensual intimate imagery (NCII), copyright infringement, misinformation | NCMEC PhotoDNA + GIFCT hash-matching + ML classifier + human-review queue + per-pack DSA Article 16+17 + EU-CSAM-Reg | Content-emitting µservices (messenger, mail, community, social, marketplace) MUST emit content-classification signal; cross-ref §3.2.5 row 9 |
| 6 | **Fake reviews / engagement manipulation** | Review-bombing, paid-review fraud, fake follows, click-farm activity, view-count manipulation | Graph community-detection, behavioural pattern detection, temporal clustering | Marketplace + community + social MUST emit engagement-manipulation signal |
| 7 | **Insider risk** | Tenant-admin exfiltration, employee data access pattern anomaly, JIT-access abuse | UEBA per §3.2.4 Domain 8 + per-employee baseline + sensitive-resource access patterns | Every µservice's audit-stream feeds the UEBA substrate per ADR-0263 |
| 8 | **Policy violation** | Cedar permit forge, audit-row tamper attempt, cross-tenant access attempt, sidecar credential exfil, sanctions-bypass attempt | Cedar evaluation anomaly + audit-chain tamper detection + cross-tenant flow detection + sidecar exit anomaly | All policy-evaluating µservices emit `PolicyViolationDetected` signals |

#### 3.2.6.C — Mitigation taxonomy (containment + notification + remediation + recovery)

When a detection signal fires + risk-score crosses threshold, mitigation MUST execute the response loop. Mitigation is graded by **time-to-effect** and **proportionality** (don't lock out a tenant for a single bad transaction; do isolate a compromised cell on confirmed breach).

| Phase | Action class | Time-to-effect target | Per-pack timing floor | Hyperscaler precedent |
|---|---|---|---|---|
| **Containment** | Per-action: freeze transaction, lock account, quarantine content, isolate compromised principal, kill malicious session, sandbox suspect process, revoke leaked credential, blackhole malicious IP at edge | ≤5s for high-confidence; ≤60s for triage-confirmed | KR-FSS: ≤24h for financial fraud freeze | Stripe Radar (live freeze), Cloudflare Magic Transit (BGP blackhole), AWS Account Lockdown |
| **Containment escalation** | Per-cell: isolate compromised cell from cross-cell mesh; per-tenant: enable shelter mode (cross-ref §3.2.5 row 8); per-region: regional failover | ≤5min cell isolation; ≤15min regional failover per ADR-0241 | NIS2: ≤24h impact assessment | AWS GuardDuty + Detective auto-isolate, Google Chronicle automated response |
| **Notification — affected party** | Notify the user/tenant/principal whose entity is affected; include explanation per GDPR Art. 22 + EU AI Act Art. 86 + Reg B adverse-action notice | ≤15min for account-affecting; ≤1h for transaction-level | GDPR 72h breach notification; CCPA + KR-PIPA notice cadence | Stripe consumer-facing fraud notification, Apple Account Security notifications |
| **Notification — regulator** | Per-pack regulator workflow (`breach_notification_workflow_id` per ADR-0251); per ADR-0251 §nis2_three_stage_cadence (24h/72h/1mo) | Pack-defined; cross-ref ADR-0251 | GDPR 72h, KR-PIPA 72h, HIPAA 60d, NIS2 24h/72h/1mo, NY DFS Cybersecurity Reg 23-NYCRR-500 72h | Vanta breach-notification automation |
| **Notification — law enforcement** | Where mandatory: NCMEC CyberTipline (CSAM per 18 USC §2258A), GIFCT (terrorism), FinCEN SAR (AML threshold), state child-abuse mandatory reporter | Per-statute deadlines | Mandatory; cannot be opted out | NCMEC PhotoDNA matching workflow, Stripe AML SAR filing |
| **Remediation — reverse** | Refund transaction; restore content; recover account; unlock principal; rollback policy change; restore from backup | ≤transactional latency; ≤15min for content restore | KR-FSS chargeback 60d; UK Faster Payments reversal; per-pack mandatory restoration of fundamental-rights affected service per EU AI Act Art. 27 | Stripe refund, AWS service restoration runbook |
| **Remediation — quarantine** | Move suspect entity to quarantine namespace; per-tenant isolation; review queue routing | ≤30min routing to investigator | EU DSA Article 17 statement-of-reasons | YouTube quarantine, Twitter/X account-suspension-with-appeal |
| **Recovery — reconciliation** | Audit-trail reconciliation; restore signed-state; verify invariants; post-action verification report | ≤24h reconciliation per SEV1+; ≤1wk for SEV2 | Per ADR-0028 audit-chain seal + ADR-0276 backup-portability | AWS automated post-incident reconciliation, Stripe ledger-rebuild |
| **Recovery — communication** | Update affected party with resolution + remediation; per-pack transparency report | ≤72h resolution communication; quarterly transparency report | GDPR + DSA + state-AG cadences | Stripe consumer-facing resolution emails, Apple Transparency Reports |
| **Recovery — appeal mechanism** | Per GDPR Art. 22 + EU AI Act Art. 86 + state-level: user appeals adverse action; routes to human reviewer; per-tenant SLA | Pack-defined; ≤30d default for substantive review | EU AI Act Art. 86 right-to-meaningful-explanation; NY AEDT 2023; ECOA Reg B | Meta Oversight Board, OpenAI appeal mechanism |

**Mitigation invariants (MUST):**

1. **Proportionality** — mitigation severity proportional to risk score. Single-event medium-confidence → step-up auth or cool-down; high-confidence pattern → freeze; breach-confirmed → isolate + regulator notify.
2. **Reversibility-by-default for low-confidence** — first-action mitigation MUST be reversible (cool-down, soft-block, request additional verification); irreversible mitigations (account-deletion, public content removal, transaction reversal-with-fees) only after escalation + human-reviewed.
3. **Auditable + non-repudiable** — every mitigation action emits audit event per ADR-0263 with: tenant + principal + entity + signal + risk-score + action-class + reviewer-identity + appeal-mechanism-link. Cryptographically sealed per ADR-0028.
4. **No silent mitigation** — affected party MUST be notified (with the §3.2.5 row 8 DV-survivor exception where notifying the abuser would harm the survivor; in that case shelter mode applies).
5. **Critical-path exemption preserved** — emergency-services (§3.2.5 row 1) NEVER mitigation-blocked; audit-and-investigate, never block.
6. **Per-pack regulator floor honored** — mitigation cadence respects each active pack's regulator-mandated timing (no faster, no slower than the pack permits where the regulator has a floor).
7. **Cross-jurisdictional conflict resolution** — when packs disagree on mitigation cadence, higher-restriction floor wins (cross-ref §3.2.5 row 23).
8. **Appeal mechanism mandatory for adverse action** — per GDPR Art. 22 + EU AI Act Art. 86 + ECOA Reg B + NY AEDT 2023: every adverse-action mitigation surfaces appeal route to user with human-reviewer SLA.

#### 3.2.6.D — Prevention taxonomy (proactive + architectural + behavioural)

Prevention is upstream of detection. If a class of harm is prevented architecturally, detection becomes a confirmation signal not a primary defense. Every detection family (§3.2.6.A) has paired prevention controls.

| Family | Pre-action prevention | Architectural prevention | Behavioural prevention | Education + nudge |
|---|---|---|---|---|
| **Payment fraud** | Per-tenant transaction-tier (KYB-verified); cool-down on first-N-days high-value; 3DS2 mandatory for high-risk; per-PSP velocity caps | Tokenization (no PAN at rest); per-cell PCI-DSS isolation; mandatory signed-payload webhooks; idempotency-key required | Risk-tier-adaptive per-user limits; trusted-contact opt-in for elder users; per-pack KR-FSS time-window restrictions | "Are you sure" on high-value irreversible mutations (cross-ref §3.2.5 row 25); per-tenant security tips |
| **Account-takeover (ATO)** | Phishing-resistant WebAuthn passkeys preferred; SMS-OTP disallowed for high-risk; SIM-swap detect via telco signal; HIBP credential check at sign-in | Per-tenant identity-broker isolation; per-session HMAC + audience binding + TLS exporter | Behavioural-baseline-based step-up auth; geo-impossibility soft-block; device-trust scoring | Phishing-training surfaces; per-tenant security-newsletter; in-product passkey-onboarding nudge |
| **Synthetic identity** | Mandatory liveness check on document upload; face-match to passkey-enrolled device; KYB graph correlation pre-onboard | Per-pack KYC tier mandatory; per-jurisdiction identity-provider integration (Real-Name-Verification in KR/JP) | Behavioural-baseline drift detection over the onboarding flow itself | Education: how the platform verifies; what doesn't work (deepfake docs flagged) |
| **AML + sanctions** | Pre-charge sanctions screen (OFAC/EU/UN/KR-MOFA); per-tenant transaction-tier caps; pre-payout dwell period | Per-pack data-residency hard-stop; per-jurisdiction FinCEN/KR-FIU/JP-FIU reporting wiring | Per-tenant SAR threshold tuning (within regulator floor); per-pack high-risk-corridor flagging | Tenant-admin onboarding includes AML obligation summary |
| **Content abuse (CSAM, terrorism, NCII, copyright, misinformation)** | NCMEC PhotoDNA pre-upload check + GIFCT hash match + per-pack DSA Art 16+17 + EU-CSAM-Reg | Per-tenant content-policy Cedar gate; per-cell content-classifier; per-region content-removal-timing | Per-tenant content-classifier sensitivity (within pack floor); minor-protection per ADR-0292 | Per-tenant content-policy clarity; per-tenant moderator training; tenant-public transparency report |
| **Fake reviews / engagement manipulation** | Per-tenant verified-purchase requirement for reviews; per-account write-velocity caps; per-IP review-volume caps | Per-tenant trust-graph (verified relationship signals); per-pack disclosure-required (FTC Endorsement Guides) | Behavioural-baseline anomaly detection (review burst clusters); per-graph community-detection | Tenant policy on disclosure + sponsored-content rules |
| **Insider risk** | JIT-access via PAM (CyberArk/Teleport/Boundary); pre-action MFA; sensitive-resource read-only-by-default | Per-cell employee-access scope; per-tenant cross-tenant access forbidden by Cedar default-deny | Per-employee behavioural baseline; UEBA scoring; pre-departure access review | Quarterly insider-threat training; per-role onboarding |
| **Policy violation** | Cedar default-deny baseline; per-action permit required; signed audit-row per ADR-0263; sidecar credential isolation per ADR-0296 | Per-cell network-policy default-deny (Cilium); per-tenant cross-tenant L3-reachability forbidden | Cedar permit anomaly detection; audit-chain tamper detection | Tenant-admin Cedar fragment authoring training |

**Prevention layer taxonomy (defense-in-depth):**

| Layer | Control class | Examples |
|---|---|---|
| **L0 — Edge** | Tier-0 cell controls per ADR-0248 | DDoS scrubbing, WAF, rate-limit, bot-mgmt, geo-blocks |
| **L1 — Network** | Service mesh + workload identity | Cilium ambient eBPF, SPIFFE/SPIRE per ADR-0295, mTLS, default-deny network-policy |
| **L2 — Auth** | Identity + session controls | WebAuthn passkeys, SIM-swap detect, HIBP check, session HMAC + audience binding |
| **L3 — Policy (Cedar)** | Per-action authorization | Cedar v4.2 LTS fragments, default-deny, per-tenant audience-type, per-pack overlay |
| **L4 — Application** | Per-µservice business logic guards | Velocity caps, cool-downs, per-tenant transaction-tier, idempotency-key |
| **L5 — Data** | Per-data-class controls | Tokenization, per-tenant DEK + KEK, per-pack residency, DLP egress scan |
| **L6 — Observability** | Real-time + batch detection per §3.2.6.A | Audit-event emission per ADR-0263, anomaly score, drift detection |
| **L7 — Investigation** | Case-management + escalation | Triage queue, evidence correlation, link analysis, ombudsman escalation |
| **L8 — User** | Education + nudge | Phishing training, "are you sure", security-newsletter, in-product hints |
| **L9 — Organizational** | Governance, process, training | Pentest, bug bounty, red team, tabletop, ethics board, ombudsman |

A single threat class is addressed by controls at *multiple layers*, not one. A payment-fraud incident is contained-not-only at L0 (Stripe Radar score) + L3 (Cedar permit) + L4 (per-tenant tier) + L5 (tokenized PAN) + L6 (anomaly) + L7 (investigation). Layered defense; no single-point-of-failure.

**Prevention invariants (MUST):**

1. **No single point of failure** — every threat class addressed at ≥3 of the L0-L9 layers above. Documented per-µservice in `compliance.md §prevention-layers`.
2. **Friction at the right layer** — friction lives at the layer that minimizes UX impact. CAPTCHA at L0 (only when bot-score crosses threshold) NOT L8 (every page load).
3. **Architectural > behavioural where possible** — tokenization (architectural) > "warn user about phishing" (behavioural). Architectural prevents the class; behavioural mitigates individual incidents.
4. **Critical-path exemption** — emergency-services + healthcare-acute-care + crisis-line MUST NOT be prevention-blocked at any layer (per §3.2.5 row 1). Prevention applies to non-critical paths.
5. **User education is real but bounded** — phishing training reduces but doesn't eliminate the class; architectural controls (passkeys) eliminate the class. Education is a complement not a substitute.
6. **Per-pack regulator floor preserved** — prevention controls cannot operate above the regulator floor where the regulator has explicit mandate (e.g., KR-FSS mandates certain financial fraud cooling-off periods — the platform cannot eliminate them via slick UX).

#### Pattern-detection substrate primitives (MUST exist)

1. **Streaming detection pipeline** — Apache Flink or equivalent (Beam / Kafka Streams / Materialize) consumes audit events per ADR-0263; per-detection-family rules + ML models score in flight; signals emitted to investigation queue.
2. **Batch detection pipeline** — Apache Spark or equivalent (Polars + ClickHouse + Trino) runs scheduled jobs over the audit-event lake (ClickHouse cold tier per `cloud-iac/clickhouse-cluster-iac.yaml`) for retrospective detection.
3. **Feature store** — Vertex AI Feature Store / Feast / Tecton hosts the per-entity (user, tenant, transaction, content) feature vectors used by ML scorers. Per-tenant feature isolation enforced.
4. **Rules engine** — Sigma-rule-class declarative rule language; per-rule lifecycle (Proposed → Soaking → Active → Sunset) mirrors Cedar fragment lifecycle per ADR-0294 (soak window applies to detection rules too).
5. **Composite scorer** — Combines per-family signals into a unified per-entity risk score; explainable (per EU AI Act high-risk Article 13 transparency obligation); LIME/SHAP-style feature importance available on appeal.
6. **Graph store + community detection** — Apache AGE (Postgres+graph) or Neo4j; runs link analysis for fraud-ring detection, mule-account graphs, synthetic-identity clusters, click-farm topology.
7. **Investigation case-management** — Per-case workflow: signal → triage → investigation → escalation/dismissal → feedback → model retrain. Integrates with Cedar (only authorized investigators see PII per case); cross-ref `microservices/ops-dashboard-control-center/` panels.
8. **Sandbox + replay** — Detection model + rules can be replayed against historical audit-stream for back-testing before promotion to Active.

#### ML model lifecycle (per EU AI Act + NIST AI RMF + ISO/IEC 42001:2023)

| Stage | Requirement | Hyperscaler precedent |
|---|---|---|
| **Training** | Per-tenant data residency honored; cross-tenant training requires explicit pack-permitted consent | Vertex AI Training, AWS SageMaker, Azure ML |
| **Validation** | Bias audit per `docs/standards/fintech-compliance.md` + fair-lending laws (ECOA, KR-Financial-Consumer-Protection-Act); validation against held-out fairness slices | IBM AI Fairness 360, Microsoft Fairlearn, Google What-If Tool |
| **A/B testing** | Champion-challenger pattern; shadow-mode then canary then full; rollback per ADR-0294 anomaly-rollback | Statsig, Optimizely, LaunchDarkly Experimentation |
| **Drift detection** | Feature drift + label drift + concept drift detection daily; alert on threshold-crossing | Arize AI, Fiddler, WhyLabs, Evidently AI |
| **Fairness re-audit** | Quarterly fairness re-audit per protected class (per jurisdiction: race/gender/age in US; KR has specific protected classes; EU has Article 21 Charter) | Same as Validation row + scheduled |
| **Model versioning** | Per-model SemVer per ADR-0258; model card per Google Model Card template; per-version reproducibility | MLflow, Weights & Biases, Hugging Face Hub |
| **Rollback** | Per-pack regulator timing (e.g., EU AI Act post-market monitoring 24h on serious incident) | Rollback runbook per pack |
| **Appeal mechanism** | Per GDPR Article 22 + EU AI Act Article 86 + state-level (e.g., NY AEDT Bias Audit Law) right-to-meaningful-explanation; appeal routes to human reviewer with case-management substrate | OpenAI right-to-appeal pattern, Meta independent oversight board |

#### Detection-fairness invariants (per EU AI Act high-risk + civil-rights laws)

1. **No proxy discrimination** — features that proxy protected classes (zip code → race, name → ethnicity, language preference → national origin) flagged + explainable. Fair-lending review mandatory for any payments / credit / housing / employment detection.
2. **Per-class TPR/FPR equity** — true-positive rate + false-positive rate per protected class within ±2pp baseline. Wider gaps require explicit ADR justification + regulator notification (under EU AI Act).
3. **Disparate impact testing** — 4/5ths rule (Federal Uniform Guidelines on Employee Selection Procedures); equivalent under EU + KR + JP regulator floors.
4. **Explainability floor** — every adverse-action signal (denied transaction, locked account, content removed) carries human-readable explanation per ECOA Reg B (US), GDPR Art. 13/22, EU AI Act Art. 13.
5. **Per-jurisdiction model variants** — global model + per-pack overlay (e.g., KR-FSS forbids certain features per `Financial Consumer Protection Act` Art. 30; EU forbids social scoring per AI Act Art. 5). Overlay enforced at evaluation time.

#### Detection observability (per ADR-0263)

Detection substrate MUST emit:
- **`DetectionSignalEmitted`** — every detection signal with severity + family + tenant + entity + score + features-contributing-most
- **`DetectionRulePromoted`** / **`DetectionRuleSunset`** — rule lifecycle events
- **`DetectionModelDeployed`** / **`DetectionModelRolledBack`** — model lifecycle
- **`DetectionAppealFiled`** / **`DetectionAppealAdjudicated`** — appeal-mechanism audit per GDPR Art. 22
- **`DetectionFairnessReportEmitted`** — quarterly fairness-audit emission
- **`DetectionDriftAlertTriggered`** — drift detection signals

#### UX-floor (cross-ref §3.2.3) AND policy-adherence invariant

Detection MUST NOT:
- Block legitimate user actions on the default path (per §3.2.3 UX-floor)
- Skip the §3.2.5 critical-path exemptions — emergency services bypass detection-flag → still get through (audit + investigate, never block)
- Operate without appeal mechanism (GDPR Art. 22 + EU AI Act Art. 86)
- Use a global model that ignores per-pack jurisdictional constraints (cross-ref §3.2.5 row 23)
- Treat false-positive cost as zero — false-positive on financial fraud locks legitimate users out; on content abuse, silences legitimate speech; on insider risk, accuses innocent employees

#### Per-µservice obligation

Every µservice's `compliance.md §detection-substrate-binding` MUST list:
- Which of the 8 detection families it contributes signals to
- Which audit-event-classes it emits per ADR-0263
- Which features it computes for the feature store
- Which detection signals it consumes + how it acts on them (e.g., "payments consumes ATO signal from identity and applies cool-down on high-value transaction")
- Per-tenant per-pack overlay (e.g., HIPAA-pack tenants → PHI features never enter feature store)
- Appeal mechanism wiring for adverse actions taken on this µservice's surface

#### CI lanes

- `oya-governance-detection-substrate-emission` — verifies every µservice emits the events declared
- `oya-governance-detection-fairness-audit` — quarterly fairness audit cadence enforced
- `oya-governance-detection-appeal-coverage` — every adverse-action surface has appeal mechanism
- `oya-governance-detection-explainability` — every detection score has feature-importance available
- Aggregate: `oya-governance-detection-baseline`

#### New ADRs required (added to Wave-3-D backlog, joining 0298-0306)

- `ADR-0307-detection-substrate-streaming-batch.md` (covers families 1-8 + streaming+batch primitives)
- `ADR-0308-ml-model-lifecycle-ai-act-compliance.md` (covers ML lifecycle table above)
- `ADR-0309-detection-fairness-audit-civil-rights.md` (covers fairness invariants + per-pack overlay)
- `ADR-0310-investigation-case-management.md` (covers triage → investigation → escalation → feedback workflow + Cedar-gated PII access)

This brings the **critical-path + detection doctrine cluster** to 14 new ADRs (0297-0310) gated for Wave-3-D authoring.

#### Per-µservice ADR-adherence matrix extension (rows 49-52)

| 49 | ADR-0307 (detection substrate) | Which of 8 detection families does this µservice contribute to? Which events emitted per ADR-0263? | `compliance.md §detection-substrate-binding` |
| 50 | ADR-0308 (ML lifecycle) | If µservice trains/serves ML: model lifecycle per Validation/Drift/Fairness/Versioning/Appeal rows | `compliance.md §ml-model-lifecycle` |
| 51 | ADR-0309 (fairness) | Bias audit pass + per-class TPR/FPR equity + explainability floor | `compliance.md §detection-fairness-audit` |
| 52 | ADR-0310 (investigation) | Case-management integration + appeal-mechanism wiring | `compliance.md §investigation-binding` |

§3.2.1 ADR-adherence matrix is now **52 rows** (28 keystone-bundle + 20 hyperscaler defense + 4 detection-substrate). A µservice answering fewer than 52 rows is REVISE.

#### What goes in `policy/abuse-defence.cedar`

```cedar
// Cedar v4.2 fragment — abuse-defence baseline
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

forbid (principal, action, resource) when {
    principal.request_rate_per_minute > resource.rate_limit_per_minute
};

forbid (principal == ?, action in [Action::Read, Action::Scrape], resource) when {
    principal.fingerprint_age_seconds < 30
    && action.depth > 50
};
```

#### CI lanes

- `oya-governance-anti-bot-coverage` — every internet-facing µservice declares its anti-bot controls and CI verifies the 8-row anti-bot table is filled.
- `oya-governance-anti-spoof-coverage` — same for anti-spoof.
- `oya-governance-anti-scrape-coverage` — same for anti-scrape.
- Aggregate lane `oya-governance-abuse-defence` rolls up all three.

#### Canonical authority — ADR-0297

`docs/decisions/ADR-0700-ci-admission-live-apex.md` (3,112 lines, landed 2026-05-20) is the canonical authority codifying the three taxonomies above + the Cedar fragment shape + the CI lanes + per-cell-tier variants (Tier-0 edge cells carry richer bot-mgmt than Tier-3 data cells, which are not internet-facing) + per-tenant audience-type tuning + 8 compliance interactions (GDPR / CCPA / COPPA / KOSA / DSA / EU-AI-Act / regional packs / PCI-HIPAA) + 18 audit-event classes added to the ADR-0263 emission registry.

### 3.2.4 Hyperscaler defense-in-depth — the full security baseline

Beyond abuse-defence (§3.2.3), every internet-facing µservice and every substrate µservice MUST honor the broader hyperscaler defense-in-depth catalogue. Below is the **20-domain hyperscaler security baseline** — each domain has its own table; each table lists the mandatory controls + the hyperscaler precedent + the CI lane + the doc surface where it lives.

#### Domain 1 — DDoS protection (volumetric + protocol + application)

| Layer | Control | Hyperscaler precedent | CI lane |
|---|---|---|---|
| L3/L4 volumetric | Anycast + SYN-cookies + BGP-flowspec + scrubbing | AWS Shield Advanced, Cloudflare Magic Transit, Google Cloud Armor | `oya-governance-ddos-l3l4` |
| L4 protocol | TCP-state tracking + QUIC connection-limit + slow-loris timeout | Cloudflare, Akamai Prolexic | `oya-governance-ddos-protocol` |
| L7 application | Adaptive rate-limiting + JS challenge + bot-mgmt composition | Cloudflare, AWS WAF, Imperva | `oya-governance-ddos-l7` |
| Egress | Per-µservice outbound rate-limit + DLP scan | AWS WAF, Cilium Cluster Mesh | `oya-governance-egress-rate-limit` |

#### Domain 2 — WAF (OWASP Top 10 + API security)

Every internet-facing µservice MUST sit behind a WAF that filters: SQL injection, XSS, CSRF, RFI/LFI, command injection, XXE, insecure deserialization, server-side request forgery (SSRF), prototype pollution, path traversal, host-header injection, HTTP request smuggling. Per OWASP Top 10 (2021 + planned 2025 revisions) + OWASP API Top 10 (2023).

Precedents: Cloudflare WAF Managed Rules, AWS WAF Managed Rules (`AWSManagedRulesCommonRuleSet`), Azure WAF (Application Gateway), ModSecurity OWASP CRS 4.x.

Per-µservice docs: `policy/waf-rules.md` declaring the active rule sets + custom rules + exception list.

#### Domain 3 — Secrets scanning + leak prevention

| Layer | Control | Precedent | CI lane |
|---|---|---|---|
| Pre-commit | Pre-commit hook scans staged diff for secrets | GitGuardian pre-commit, gitleaks, talisman | `oya-governance-precommit-secrets` |
| Repo | Scheduled repo scan; rotate-on-detect; quarantine | GitHub Secret Scanning, TruffleHog, GitGuardian | `oya-governance-repo-secret-scan` |
| Runtime | eBPF-based detection of in-memory secrets at egress | Cloudflare R2 secret-detection, AWS Macie | `oya-governance-runtime-secret-detect` |
| Image | Container image scan for embedded secrets | Snyk, Anchore, Trivy | `oya-governance-image-secret-scan` |
| Public surface | Per-tenant key-canary tokens to detect leaks | Stripe canary keys, GitHub canary tokens | `oya-governance-canary-token-deploy` |

#### Domain 4 — SAST + DAST + IAST + SCA

| Class | Control | Precedent | CI lane |
|---|---|---|---|
| SAST | Static analysis on every PR: rust-clippy + cargo-audit + Semgrep + CodeQL | Snyk Code, Semgrep Pro, GitHub Advanced Security, Checkmarx | `oya-governance-sast` |
| DAST | Dynamic scan against staging: ZAP + Burp Suite + Pentest-Tools | OWASP ZAP, PortSwigger Burp Enterprise | `oya-governance-dast-staging` |
| IAST | Instrumented test run captures coverage + vulns at runtime | Contrast Security, Synopsys Seeker | `oya-governance-iast` |
| SCA | Dependency vuln scan + SBOM | owned SCA gate, SBOM, advisory feed, Anchore-compatible evidence | `oya-governance-sca` |
| Fuzzing | Continuous fuzzing of public APIs + parsers | OSS-Fuzz, AFL++, libFuzzer, Honggfuzz | `oya-governance-fuzz` |
| SBOM | SBOM in CycloneDX + SPDX; attached to every release; signed via cosign + Rekor | Sigstore SBOM, FOSSA, Anchore Enterprise | `oya-governance-sbom-coverage` |

#### Domain 5 — Container + supply-chain hardening

| Control | Requirement | Precedent |
|---|---|---|
| Image base | Distroless OR minimal (Alpine, scratch); FROM scratch where Rust-static; per ADR-0254 | Google Distroless, Chainguard Images, RedHat UBI Micro |
| Image vuln scan | Trivy + Grype + Snyk in CI; severity-gated (CVSS ≥7 = block) | Snyk Container, Anchore Enterprise, Aqua Trivy |
| Image signing | Sigstore cosign + Rekor + Fulcio; verified at admission via Kyverno or Sigstore Policy Controller | AWS Signer, Notary v2, Sigstore |
| SLSA level | SLSA L3+ build provenance for every image; hermetic builds; isolated builders | Google SLSA, GitHub Actions OIDC + Sigstore |
| Vulnerability SLA | CVSS 9.0+: ≤24h patch; CVSS 7.0+: ≤7d; CVSS 4.0+: ≤30d | AWS Shield response SLA, Google VRP |
| Build provenance | in-toto attestations; reproducible builds where possible | Sigstore, in-toto, Reproducible Builds |
| Dependency pinning | Cargo.lock + npm package-lock + Pipfile.lock pinned + signed; no floating versions | owned lockfile freshness + dependency automation policy |

#### Domain 6 — Network segmentation + zero-trust

| Layer | Control | Precedent |
|---|---|---|
| Pod-to-pod | Cilium ambient eBPF NetworkPolicy + L7 authz | Cilium, Istio Ambient, Tetragon |
| Service-to-service | mTLS via SPIFFE + SPIRE per ADR-0295 | Google BeyondProd, Istio mTLS, AWS App Mesh |
| Cell-to-cell | Per-cell network isolation; explicit allow-list for cross-cell | AWS Cell-Based Architecture, Google Spanner zones |
| Tenant-to-tenant | No L3 reachability between tenants in shared cells; verified by chaos-test | AWS account boundary, Google project boundary |
| North-south | Edge → service mesh ingress only via mTLS-verified gateway | Cloudflare Tunnels, AWS API Gateway → VPC Lambda |
| East-west | Default-deny baseline; every flow explicitly allowed in NetworkPolicy | Google BeyondCorp, Zscaler ZIA |

Per-µservice doc: `iac/network-policy.yaml` declares allow-list; verified by `oya-governance-network-policy-coverage`.

#### Domain 7 — DLP (Data Loss Prevention)

| Surface | Control | Precedent |
|---|---|---|
| Egress | Outbound payload scan for PII + PHI + PCI + secrets; per-data-class egress policy | Google DLP API, Symantec DLP, Forcepoint DLP, AWS Macie |
| Email | Per-tenant outbound mail scan + policy enforcement | Proofpoint, Mimecast, Google Workspace DLP |
| File uploads | Per-file content classification at upload | AWS Macie, Microsoft Purview |
| Clipboard | Native-app clipboard scoping; no cross-tenant copy-paste | Microsoft Intune, VMware Workspace ONE |
| Print + screenshot | Watermark + audit-trail for sensitive content | Microsoft Purview Information Protection |

Per-µservice doc: `policy/dlp-egress.cedar`; CI lane `oya-governance-dlp-egress-coverage`.

#### Domain 8 — UEBA + insider threat + just-in-time access

| Control | Requirement | Precedent |
|---|---|---|
| Behavioural baselines | Per-employee + per-service-account baselines; anomaly-score on every action | Microsoft Defender for Identity, Google Chronicle, Splunk UEBA |
| Just-in-time access | PAM — admin access requires ticket + approval + time-bound (≤4h default) | CyberArk, HashiCorp Boundary, Teleport, AWS Session Manager |
| Session recording | Break-glass sessions recorded; chain-of-custody preserved | CyberArk PSM, AWS Session Manager logging |
| Access reviews | Quarterly per-employee scope review; automatic revocation of unused entitlements | Microsoft Entra ID Governance, SailPoint, Okta Identity Governance |
| Background checks | Pre-employment + role-change | Sterling, Checkr, HireRight |
| Step-up auth on sensitive ops | Per `docs/standards/step-up-auth-classes.md`; WebAuthn passkey + hardware token for highest tier | Cross-ref ADR + step-up-auth-classes standard |

Per-µservice doc: `compliance.md §insider-threat-controls`; CI lane `oya-governance-jit-access-coverage`.

#### Domain 9 — Threat intelligence integration

| Feed | Use | Precedent |
|---|---|---|
| IP reputation (Spamhaus, FireHOL, AbuseIPDB) | Block known malicious IPs at edge | Cloudflare IP Reputation, AWS Network Firewall |
| Tor exit nodes | Tag traffic; per-tenant policy may block or challenge | Cloudflare Tor BL, Akamai |
| Known-bot IPs | Compose with bot-mgmt score | Cloudflare Bot Management, DataDome |
| CVE feed (NVD + GitHub Advisory + OSV) | Auto-rebuild on critical CVE in deps; rotate on critical CVE in TLS lib | Snyk, Sigstore CVE-tracker |
| Stolen-credential corpora (HIBP + in-house) | Block known-leaked passwords at sign-in | HIBP API, Microsoft compromised-credentials |
| Sanctions lists (OFAC, EU, UN, KR-MOFA) | KYB-time + transaction-time check | Refinitiv World-Check, Dow Jones RiskCenter |

Per-µservice doc: `compliance.md §threat-intelligence-feeds`.

#### Domain 10 — Incident response + forensics

| Capability | Requirement | Precedent |
|---|---|---|
| Detection | SIEM + EDR on every cluster; per-µservice anomaly alerting | Splunk SOAR, Microsoft Sentinel, Elastic Security, Crowdstrike Falcon |
| Severity classification | SEV1 / SEV2 / SEV3 / SEV4 per `docs/standards/incident-severity.md` | Cross-ref existing standard |
| Forensic snapshot | On SEV1+: snapshot affected cell to immutable read-only archive | AWS Detective, Google Chronicle |
| Chain of custody | Cryptographically sealed evidence chain per ADR-0028 audit-chain | NIST SP 800-86 |
| Tabletop exercises | Quarterly tabletop per scenario class | NIST CSF, MITRE ATT&CK exercises |
| Post-mortem | Blameless within 5 business days; cross-ref `docs/standards/postmortem-template.md` | Google SRE Workbook |
| Communication | Per-tenant + per-regulator notification; per ADR-0251 breach-notification-workflow | GDPR Art. 33 + 34, CCPA, KR-PIPA Art. 34, NIS2 Art. 23 |

Per-µservice doc: `incident-response.md` (already in roster); CI lane `oya-governance-incident-response-coverage`.

#### Domain 11 — Vulnerability management

| Control | SLA | Precedent |
|---|---|---|
| CVSS 9.0+ (Critical) | ≤24h patch + verify | AWS critical-patch SLA, Microsoft Patch Tuesday emergency |
| CVSS 7.0+ (High) | ≤7d | Google Project Zero policy |
| CVSS 4.0+ (Medium) | ≤30d | Standard vendor cadence |
| Patch deployment | Canary-then-blue-green per ADR-0254 | AWS deployment patterns |
| Patch rollback | Per `docs/runbooks/cve-critical-patch.md` (existing) | Cross-ref existing runbook |

#### Domain 12 — Penetration testing + red teaming + bug bounty

| Cadence | Activity | Precedent |
|---|---|---|
| Quarterly | External pentest by accredited firm | Trail of Bits, NCC Group, Bishop Fox, IOActive |
| Annual | Red team exercise (assumed-breach scenario) | Mandiant Red Team, Crowdstrike Red Team |
| Continuous | Public bug bounty | HackerOne, Bugcrowd, Intigriti |
| Continuous | Responsible disclosure surface (`/security.txt` RFC 9116) | Standard |

Per-µservice doc: `compliance.md §pentest-and-bounty-cadence`.

#### Domain 13 — Zero-knowledge + confidential computing + E2EE

| Surface | Control | Precedent |
|---|---|---|
| Messenger | MLS RFC 9420 E2EE per `docs/standards/messenger-e2e-encryption-mls.md` | Cross-ref |
| Sensitive workloads | Confidential computing: AWS Nitro Enclaves, Intel SGX, AMD SEV-SNP, GCP Confidential VM | Build-ahead-of-certification per ADR-0250 |
| Sealed-sender / metadata-minimization | Apply where regulator/threat-model demands it | Signal sealed-sender, Apple Private Relay |
| Client-side encryption | Where browser+native both can support it | Tutanota, ProtonMail, Apple iCloud Advanced Data Protection |

#### Domain 14 — Data classification + tagging + lineage

| Layer | Control | Precedent |
|---|---|---|
| Ingest | Auto-classify at API boundary; PII / PHI / PCI / Sovereign tagging | AWS Macie, Microsoft Purview, BigID, Immuta |
| Storage | Per-data-class encryption + retention + DSAR-handling | Cross-ref ADR-0276 + `data-class.md` standard |
| Lineage | Track data lineage from ingest through every transformation to egress | DataHub, Alation, OpenLineage |
| Tagging | Per-row tenant_id + data_class tags; verified by `oya-check-tenant-cost-labels-coverage` | Cross-ref existing CI lane |

#### Domain 15 — Backup + recovery + business continuity

Per existing `docs/standards/backup-canonical.md` + `dr-business-continuity.md` standards. Hyperscaler precedents: AWS Backup, AWS DRS (Disaster Recovery), Azure Site Recovery, GCP Backup-DR.

- 3-2-1 rule (3 copies, 2 media, 1 off-site)
- Immutable backups (S3 Object Lock with Compliance mode)
- Tested restore quarterly per `docs/runbooks/dr-business-continuity.md`
- RPO/RTO per cell tier (per ADR-0241)

CI lane: `oya-governance-backup-restore-tested`.

#### Domain 16 — Cryptographic key + rotation discipline

| Class | Rotation | Precedent |
|---|---|---|
| Data-encryption keys (DEK) | ≤90d default; ≤30d for KR-FSS / PCI | AWS KMS auto-rotation, GCP Cloud KMS |
| Key-encryption keys (KEK) | ≤365d | AWS CloudHSM rotation |
| Root keys (HSM-rooted) | ≤24mo via Shamir ceremony per ADR-0247 + ADR-0293 | Cross-ref |
| TLS server certs | ≤90d (Let's Encrypt cadence) | Cross-ref ADR-0253 |
| Crypto-agility | PQ hybrid migration path per ADR-0253 amendment | Hyperscaler reference: AWS PQC roadmap |

#### Domain 17 — Tenant isolation guarantees

| Surface | Control | Precedent |
|---|---|---|
| Compute | Per-tenant Cloud Hypervisor + Kata pod per ADR-0254 | AWS Firecracker (Lambda), Google gVisor (alternative explicitly NOT chosen) |
| Network | Per-tenant Cilium NetworkPolicy + L7 authz | Cilium, Tetragon |
| Storage | Per-tenant Postgres schema or Citus shard; per-tenant KMS DEK | Stripe per-account encryption |
| Audit | Per-tenant audit-stream per ADR-0263 | Stripe audit log, AWS CloudTrail org-mode |
| Chaos-test | Quarterly cross-tenant reachability test (assumed-breach in one tenant; verify no L3 reachability to another) | Netflix Chaos Monkey, AWS Fault Injection Simulator |

CI lane: `oya-governance-tenant-isolation-chaos-test`.

#### Domain 18 — Physical + facility security

Per SOC 2 Type 2 + ISO 27001:2022 facility controls. Hyperscaler precedents: AWS data center compliance, Google data center security, Azure data center security.

- Data center perimeter (cameras, badges, mantrap)
- Hardware tamper-detection (intrusion seals on offline HSM safes per ADR-0247)
- Disposal procedures (NIST 800-88 cryptographic erase + physical destruction)
- Visitor logs + escort policy

Per-µservice doc: `compliance.md §facility-controls` (mostly inherited from cell substrate).

#### Domain 19 — Supply chain + third-party risk

| Control | Requirement | Precedent |
|---|---|---|
| Vendor risk assessment | Pre-onboarding for any third-party data processor | BitSight, SecurityScorecard, Coalition |
| DPA (Data Processing Agreement) | Signed before any cross-border data transfer | GDPR Art. 28, Standard Contractual Clauses |
| Sub-processor list | Public + updated; per-tenant opt-out per pack | Stripe sub-processor list, AWS sub-processor list |
| Vendor audit cadence | Annual SOC 2 review for high-risk vendors | Vanta, Drata, OneTrust |

#### Domain 20 — Crypto-agility + post-quantum readiness

Per ADR-0253 amendment + future ADRs. Migration plan: hybrid (X25519 + ML-KEM-768) NOW; full PQ when peer-support common (2027+ target). Crypto-library choices: `aws-lc-rs`, `openssl-3-pqc`, `BoringSSL-pqc` — all maintained-PQ-track libraries.

Per-µservice doc: `compliance.md §crypto-agility-plan`; CI lane `oya-governance-crypto-agility`.

---

#### Aggregate CI lane

`oya-governance-hyperscaler-defense-baseline` is the aggregate that reads §3.2.3 (abuse-defence 24 controls + UX-floor) + §3.2.4 (20 domains above). Per-µservice answer count: ≥150 control rows across all 20 domains + abuse-defence. Reports daily corpus gap.

#### Per-µservice ADR-adherence matrix extension

§3.2.1 row 28 is the abuse-defence row. The hyperscaler-defense baseline §3.2.4 adds **rows 29-48** to the µservice ADR-adherence matrix — one row per domain:

| 29 | Domain 1 (DDoS) | L3/L4 + L4 + L7 + egress controls per §3.2.4 D1 | `ARCHITECTURE.md §ddos-defense` + `iac/<env>-edge-ddos.yaml` |
| 30 | Domain 2 (WAF) | OWASP CRS rule set + custom rules | `policy/waf-rules.md` |
| 31 | Domain 3 (Secrets) | Pre-commit + repo + runtime + image + canary | CI lane gates |
| 32 | Domain 4 (SAST/DAST/IAST/SCA + Fuzz + SBOM) | All 6 sub-controls wired | CI lane gates |
| 33 | Domain 5 (Container + supply chain) | Distroless + signed + SLSA L3+ + vuln SLA | `iac/` + CI lanes |
| 34 | Domain 6 (Network seg + zero-trust) | Cilium + SPIFFE + cell-isolation + default-deny | `iac/network-policy.yaml` |
| 35 | Domain 7 (DLP) | Egress + email + uploads + clipboard + print | `policy/dlp-egress.cedar` |
| 36 | Domain 8 (UEBA + JIT) | Behavioural + JIT + session-record + access-review | `compliance.md §insider-threat-controls` |
| 37 | Domain 9 (Threat intel) | IP rep + Tor + bot lists + CVE + HIBP + sanctions | `compliance.md §threat-intelligence-feeds` |
| 38 | Domain 10 (IR + forensics) | SIEM + EDR + forensic snapshot + chain-of-custody | `incident-response.md` |
| 39 | Domain 11 (Vuln mgmt) | CVE patch SLA + canary + rollback | `compliance.md §vuln-mgmt-sla` |
| 40 | Domain 12 (Pentest + bounty) | Quarterly pentest + annual red team + bug bounty + `/security.txt` | `compliance.md §pentest-and-bounty-cadence` |
| 41 | Domain 13 (E2EE + confidential compute) | MLS + Nitro/SGX/SEV-SNP where applicable | `ARCHITECTURE.md §e2ee-confidential` |
| 42 | Domain 14 (Data class + lineage) | Auto-classify + tag + lineage | `compliance.md §data-classification` |
| 43 | Domain 15 (Backup + DR) | 3-2-1 + immutable + tested restore + RPO/RTO | `backfill-replay.md` + `dr-business-continuity.md` |
| 44 | Domain 16 (Key rotation) | DEK/KEK/Root/TLS rotation cadences + PQ migration | `compliance.md §key-rotation-cadence` |
| 45 | Domain 17 (Tenant isolation) | Cloud Hypervisor + Cilium + Postgres schema + audit-stream + chaos-test | `ARCHITECTURE.md §tenant-isolation` |
| 46 | Domain 18 (Facility) | SOC 2 + tamper-detect + disposal + visitor-log | `compliance.md §facility-controls` (inherited from cell) |
| 47 | Domain 19 (Supply chain) | VRA + DPA + sub-processor list + annual audit | `compliance.md §supply-chain-risk` |
| 48 | Domain 20 (Crypto-agility + PQ) | PQ-hybrid library choice + migration plan | `compliance.md §crypto-agility-plan` |

A µservice answering fewer than 48 rows of §3.2.1 is REVISE. The §3.2 ADR-adherence matrix now has 48 rows total (28 keystone-bundle + 20 hyperscaler defense-in-depth).

## 4. The "intern-buildable" cross-section audit

For any doc graph rooted at a hero PRD or hero ADR, the audit answers:

- **Cold-start coverage.** Can the intern reach every primitive used in the implementation by following references from the root? (BFS coverage check.)
- **Term coverage.** Is every term used in the doc defined in the doc, the linked glossary, or a reachable doc?
- **Command coverage.** Is every CLI/API mentioned reachable in `docs/cli/`, `docs/api/`, or a contract under `contracts/`?
- **Failure-mode coverage.** Does each doc enumerate failure modes — what happens if step N fails, what the audit trail looks like, what the rollback is?
- **Compliance coverage.** Does each doc that touches PII, payments, or self-modification map to its compliance pack(s) per ADR-0251?
- **Numeric coverage.** Does each performance / capacity claim cite the benchmark or modeling note?

Audit output: pass / pass-with-findings / revise / blocker. CI lane: `oya-governance-doc-rigor` (advisory until 2026-07-15, BLOCKER thereafter).

## 5. What good looks like — exemplars

| Doc class | Exemplar | Why |
|---|---|---|
| ADR | `docs/decisions/ADR-0702-identity-authz-live-apex.md` (2125 lines) | Full DDL + Cedar entity types + hyperscaler precedents + migration plan + naming-justification table |
| PRD | `microservices/messenger/PRD.md` (1718 lines) | ≥140 stories across B2C + B2B; ≥6 UX flows; explicit personas; compliance mapping |
| Spec | `specs/tenant-model.json` (post-2026-05-20 edits) | Every property has description + examples + binding ADR; `_meta` block populated; passes JSON Schema validation |
| Runbook | `docs/runbooks/breach-notification-council-escalation.md` | §A–§G complete; every step has command + audit tag; explicit escalation chain |
| Standard | `docs/standards/doc-style.md` | Diátaxis quadrants + RFC-2119 + frontmatter shape + enforcement lane + companion-doc references |
| Onboarding | `docs/onboarding/intern-day-one.md` (1130 lines) | 10-step day-one; every step ends in a verifiable artifact; pitfalls section |
| User stories | `docs/user-stories/b2c-consumer-surfaces.md` (2314 lines) + `b2b-work-surfaces.md` (3317 lines) | 273 stories combined; per-surface AC; anti-stories |
| Architecture walkthrough | `docs/architecture/keystone-bundle-intern-walkthrough.md` (1733 lines) | 73-step Alice→Bob MLS-E2E DM flow with file paths + line numbers |
| Migration playbook | `docs/standards/migration-playbook.md` | Per-step rollback; observable state delta; explicit sunset |

## 6. What bad looks like — anti-patterns

| Anti-pattern | Why it fails the intern bar | Fix |
|---|---|---|
| "placeholder marker" / "see code" / "left as exercise" | Intern has no path forward | Inline the primitive OR cross-reference a doc that does |
| Bullet lists that mention concepts without defining them | Intern doesn't know what they don't know | Add a §0 Glossary anchor or link the glossary entry |
| Aspirational latency without measurement | Intern can't tell if their build is conformant | Cite benchmark commit or modeling note |
| Aspirational performance numbers without evidence | Intern (and SRE) cannot tell if the target is achievable or already violated; SLOs become unenforceable decoration | Cite the benchmark commit (preferred) OR a modeling note in `docs/performance-budgets/` that decomposes the budget per stage, states assumptions, and includes a sensitivity analysis; use format `[P5..P95 error bars] (evidence: modeling note docs/performance-budgets/<slug>.md)` inline in the ADR performance table |
| Bare prose runbook (no commands) | Intern can't execute | Every step has the exact command + expected output |
| Standalone doc (no references in either direction) | Intern can't navigate from this node | Add inbound + outbound references |
| "Talk to your onboarding buddy" | Tribal knowledge | Write it down |
| Vague compliance phrases ("GDPR-compliant") | Intern can't audit | Map to specific pack + section + Cedar fragment |
| Diagram with no text | Intern can't infer sequence or context | Caption with prose; cite the source of truth |
| `oneOf` / `if-then` JSON Schema with no `description` | Intern can't tell which branch applies | Each branch gets a `title` + `description` |

## 7. Upgrade pass procedure

When a doc fails the intern-buildable bar, the upgrade pass:

1. Open the doc next to its binding ADR.
2. Run the §4 audit checklist; mark each row pass / fail.
3. Apply fixes in this order: cross-references first, then density (add examples / DDL / commands), then cross-section coverage (failure modes, compliance, numerics).
4. Re-run the audit; iterate until pass.
5. Cross-reference the upgrade PR in the doc's `change_log` (or §G References / Change log).
6. The doc's enforcement lane promotes from advisory to BLOCKER only after the upgrade pass green.

## 8. Enforcement

CI lane: `oya-governance-doc-rigor`. Status:

- **advisory** until 2026-07-15 to give the upgrade pass time to land across the corpus.
- **BLOCKER** from 2026-07-16. PRs that introduce or modify a canonical doc that fails the bar cannot merge.

The lane reads `docs/standards/documentation-rigor.md` (this doc) + the doc-class checklist matrix to grade each touched file. The lane MUST be deterministic — same input → same verdict.

## 9. Out of scope

- This standard does not redefine voice, tone, or style — that is `doc-style.md`'s scope.
- This standard does not catalog which docs exist — that is `DOC-CATALOG.md` and `STANDARDS-AND-TEMPLATES.md`.
- This standard does not enumerate every doc that must exist for every µservice — that is `ADR-0063` doc-coverage doctrine.
- This standard does not grade code comments — that is `code-review.md` + `code-style.md`.

## 10. References

- `docs/standards/doc-style.md` — voice, tone, Diátaxis, frontmatter shape
- `docs/STANDARDS-AND-TEMPLATES.md` — canonical doc catalog
- `docs/DOC-CATALOG.md` — what doc-class lives where
- `docs/AGENTS.md` — operating contract
- ADR-0053 — doc-style canonical
- ADR-0063 — doc-coverage doctrine (per-µservice)
- ADR-0212 — buildability doctrine (100+ artifacts per µservice)
- ADR-0242 — oyatie-is-a-tenant (drives the cross-reference shape)
- ADR-0255 — Intelligence two-layer (drives the per-doc-class examples above)
- `feedback_autonomous_implementation_artifacts` (memory) — long-term goal: intern-buildable + machine-buildable
- `feedback_quality_performance_scalability_bar` (memory) — Stripe / Palantir / Linear bar
- `feedback_doc_coverage_enforced` (memory) — every µservice ships the full doc set

## 11. Change log

- 2026-05-20: Initial publication. Layered on top of doc-style.md. Authored as part of the keystone-bundle 2026-05-20 promotion gate work — closes the gap between existing style guidance and the intern-buildable bar required by `feedback_autonomous_implementation_artifacts`.
