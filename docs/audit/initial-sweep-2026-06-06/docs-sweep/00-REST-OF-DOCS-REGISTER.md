# 00 — REST-OF-DOCS FINDINGS REGISTER (WF2 synthesis)

**Role:** WF2 SYNTHESIS lead merge of the mechanical footprint (`10-stale-term-footprint.md`) + all 8 lane artifacts (`20-*.md`) into ONE rest-of-docs register for the amendment phase.
**Date:** 2026-06-06.
**Corpus:** `/Users/jasonlee/Developer/source/docs/` (the live oyatie docs tree) — NOT `linux/` (which holds only audit artifacts + the kernel/stack pilot). The `linux-nonadr` lane is the sole exception: it reviews `linux/docs/{context,migration,research}/`.
**Out of scope:** `decisions/ADR-*` (SSOT — amended in the WF1 ADR lane, not here). Term counts below are ADR-excluded unless noted.
**Canon authority:** the 12 RULED-CANON items in the WF2 brief. Every finding cites the canon item it violates.

**Verification posture (founder rule — verify at every step):** every lane digest was checked against its full artifact body before merge. The digests are faithful to the artifacts (carve-outs, false-positive notes, and coverage caveats all present in both). No phantom findings introduced; where a lane reports an ESTIMATE (not a census) it is carried forward as an estimate, not promoted to fact.

**Reachability legend:** DECISION→ADR · INSTRUCTION→session-context-bundle · GENERATED-REFERENCE→built-from-specs/ADRs · ORPHAN→not-needed/archive.

---

## 1. CANON-CONTRADICTIONS (ranked, cross-lane merged)

Ranked by **blast radius × severity**. Each row: the canon item violated, where it lives, the contradicting claim, and the fix. "Lanes" lists which sweep(s) independently surfaced it (cross-lane corroboration raises confidence).

### CC-1 — "Foundry" brand still LIVE and load-bearing, doing BOTH split-target jobs at once — canon #2
**Most pervasive contradiction in the entire corpus; surfaced by every lane.**
- **Canon:** "foundry" RETIRED → **cloud-intelligence** (AI/agent substrate) OR **governance** (fitness/policy lane), per context. GLOSSARY already records the retirement (`GLOSSARY.md:1034`, ADR-0335) — so the docs contradict their own glossary.
- **Where (lead sites):** top-level — `DESIGN.md §1/§3` (Foundry = Axis 4), `PRD.md` North Star, `MASTERPLAN.md:60,104`, `PRD-CANONICAL §9.5`, `DOC-CATALOG` + 26 supervisor rows. Standards — 50/103 files; `data-class.md:98,104` (7-axis *pillar*), `axis-foundry` team-owner in 19 files, `autonomy-ceiling.md` built on `oya-foundry-runtime-*::invoke`, `specs/deep-dive-oyatie-sst-consolidation.md:90-91` ("Foundry | Axis 4: AI agent runtime + control plane"). Governance — 151 files; live `teams/axis-foundry/CHARTER.md`, `playbook-foundry.md`, 75 `oya-foundry-*` crate refs. Products — `products/foundry/PRD.md:35,70` (substrate) AND `prds/foundry.md:20-24` (Proof-Ladder/fitness lane). Runbooks — 38 files. Architecture — ADR-0023 "Foundry sandbox", ADR-0099 "Foundry Supervisor". Journeys/personas — 325 raw (mostly mechanical/FP, see CC carve-outs). linux-nonadr — `oya-foundry-vcs-*` in CI-design.
- **The sharp problem (not just a rename):** the ONE retired brand names BOTH successor lanes simultaneously. `teams/axis-foundry/CHARTER.md:7-43` owns *both* "Agent Runtime (Foundry — Axis 3)" → cloud-intelligence AND "Foundry (Axis 4)" fitness/gates/CI → governance. Same in `products/foundry/PRD.md` (`:70` substrate vs `:75` engineering-platform/fitness). A single global swap would mis-route the governance sense.
- **Fix:** sense-routed split rename (see §2 for the routing rule). Reject mixed `oya-foundry-*` + `oya-intelligence-*` for the same seam in a fitness lane.

### CC-2 — Cedar framed AS the policy ENGINE; owned PARC absent everywhere — canon #6
**Second most pervasive; PARC appears 0× across the entire docs corpus.**
- **Canon:** Cedar = the policy **CONTRACT** / fragment language; the **owned PARC** is the engine (PDP).
- **Where:** top-level — `GLOSSARY.md:894` ("Cedar is Oyatie's… policy **engine**"), `PRD-CANONICAL.md:311,793`, `DESIGN.md:535`. Architecture — 10 standalone sites: `unified-ecosystem-thesis:104,111,272,1938`, `training-cost-doctrine:115,221,432,1240`, `wave-3-g-executive-briefing:177,1212`, plus diagrams `cedar-policy-evaluation-flow.md:41` (Cedar as "policy evaluator") + `capability-tier-projection-flow.md:36,64`. Governance — pervasive engine framing; `platform-tenancy-identity/CHARTER.md:8` ("Cedar… single source of truth for RBAC/ABAC decisions"). Runbooks — `cedar-fragment-emergency-rollback.md` (`cedar-cli authorize` as runtime evaluator) + `bootstrap-ci-compromise.md:74-240`. Journeys — `j167/story.md:34,74` ("The Cedar engine evaluates the policy").
- **Footprint nuance:** mechanical grep is **170 raw → 24 genuine** (`10-`); the rest are `Workflow Engine` / `cedar-policy-engine` IaC-path / `Cedar policy-engine logs` (audit subsystem) FALSE POSITIVES. Standards (`cedar-policy-discipline.md`), products (`tenancy/ontology/workplace`), and `component-boundaries.md` already use Cedar correctly as the contract — those are the template; the doctrine/thesis/diagram docs lag them. ADR-0191/0183 already use the correct "Cedar PDP" decomposition.
- **Fix:** every "Cedar engine" assertion splits into Cedar-the-contract + PARC-the-engine; diagram evaluator node → PARC. Name PARC explicitly when authoring. Do NOT mass-rewrite the FP `workflow-engine`/`policy-engine-logs` hits.

### CC-3 — Authority inversion: masterplan/decision-principles treated as hand-authored apex, ADRs framed as downstream — canon #1
- **Canon:** masterplan is GENERATED from ADRs; **ADRs = SSOT**.
- **Where:** `DOC-CATALOG.md:70` catalogs `MASTERPLAN.md` as hand-authored council authority (`agent_authoring_allowed: NO`) even though `MASTERPLAN.md:30,36` itself correctly says "compatibility projection… not the implementation authority" — the catalog contradicts the doc header. Standards — `INDEX.md:24-25` authority-chain places `/specs/decision-principles.json + /specs/forbidden-operations.json` at the apex and **omits ADRs entirely**; 21 files carry it in frontmatter; "Implements MASTERPLAN Directive N" cited as authority across image-discipline/dependency-policy/agent-instructions/claude-code-harness. Governance — `../../../../templates/checklists/escalation-checklist.md:34` + `agents/CROSS-REFERENCE-INDEX.md:31` codify MASTERPLAN as "Tier 0; amend-only-by-Founder" with **81 "Directive N"** binding citations.
- **Only-correct exemplars:** `automation/adr-index-pipeline.md:29` ("ADR pack… is the source of truth… manual edits rejected"); `ci-lanes.md:97` `oya-governance-masterplan-drift` gate. The generation machinery exists in the gate catalog but the prose standards still cite the old hand-authored MASTERPLAN.
- **Fix:** reclassify MASTERPLAN.md → GENERATED-REFERENCE in DOC-CATALOG; re-frame `authority_chain`/`canonical_authority` to ADRs=SSOT; resolve every "Directive N" cite to its originating ADR; extend the adr-index-pipeline generated model to the masterplan.

### CC-4 — CI: Jenkins / ArgoCD framed as canonical endpoint; oya-ci absent — canon #4
- **Canon:** unified **oya-ci** (Run+graph; Prow+Tekton+Argo) = endpoint; Jenkins/Argo OPERATIVE-until-cutover then retire (build-first-cutover-later); NOT the canonical endpoint.
- **Where (Jenkins-as-endpoint):** `MASTERPLAN.md:106` ("Promote only through… Jenkins required checks"), `PRD-CANONICAL.md:330,333,749,802` (Jenkins LTS in the agent-safe lifecycle); standards `agentic-dev-team-optimization.md:23` + `brief-template.md:53,314` (Jenkins as live coordination CI / mandatory brief section). `oya-ci` appears in **zero** top-level docs.
- **Where (ArgoCD-as-canonical):** standards `gitops-iac-cluster-tier-boundaries.md:11,87` — **"oyatie does NOT build replacements for ArgoCD / OpenTofu / Cluster API"** + ArgoCD declared canonical Tier-A owner (the cleanest single contradiction with build-first-cutover-later). Architecture echoes ADR-0160/0171/0240 (Flagger/ArgoCD as canonical controller).
- **Compliant exemplars (do NOT re-flag):** Argo Rollouts behind `oya-platform-rollout-controller-kernel` adapter (governance lane); the oya-ci **endpoint** spec `research/bespoke-ci-design/40-PRODUCT-SPEC.md` (linux-nonadr). Jenkins/Forgejo = **0 hits** in architecture and governance lanes.
- **Fix:** re-frame Jenkins/Argo as operative-until-cutover bridges under the oya-ci endpoint; delete/invert the "does NOT build replacements" sentence; flag ADR-0160/0171/0240/0349 to the ADR lane.

### CC-5 — Forgejo treated as canonical board/VCS/webhook source; GitHub demoted — canon #3
- **Canon:** Forge = GitHub NOW → bespoke VCS later; **Forgejo DROPPED (mirror at most)**.
- **Where:** runbooks — 4 substantive "Active" docs (`forgejo-agent-board-workflow.md`, `-board-verification-checklist.md`, `-board-webhook-projection.md`, `-claim-ref-cas.md`) make Forgejo the canonical board+VCS and treat GitHub as the thing to avoid (`forgejo-agent-board-workflow.md:143-159`: "Forgejo PR against `dev`," "do not use GitHub PR/merge flows," "add the self-hosted Forgejo remote"); bound to live `Forgejo 11.0.14` + ADR-0377 "conditional authority." Specs — `slice-ci-webhook-replay-guard{,-research}.md` make `X-Forgejo-Delivery` the **primary** idempotency key, co-equal with GitHub. linux-nonadr — `bespoke-ci-design/00-current-shape.md` Forgejo×31 ("Forgejo-native… until cutover"). Footprint — 21 files (ADR-excluded).
- **Nuance worth keeping:** the Jenkins-as-bridge framing inside `forgejo-agent-board-workflow.md:145` is the *right shape* (build-first-cutover-later) — it's just pointed at Forgejo-native CI instead of oya-ci. Governance lane is git-server-agnostic (canon-OK; `gitea` only as one enumerated adapter).
- **Fix:** reframe GitHub-NOW board/VCS (Forgejo mirror-at-most, or retire the spike runbooks if the board experiment is dead); re-scope the replay-guard to `X-GitHub-Delivery` primary; flag ADR-0377 to the ADR lane.

### CC-6 — Data tier: Postgres/Citus/Milvus/ClickHouse/Kafka hard-coded as canonical stack, not transitional bridges; no Pulsar — canon #5
- **Canon:** OWN the whole tier (endpoint); Postgres/Citus/Milvus/ClickHouse = TRANSITIONAL bridges; Redis→Valkey, Kafka→Pulsar.
- **Where:** top-level — `PRD-CANONICAL §8.4` (Citus/Milvus/ClickHouse + "Kafka in KRaft mode", **no Pulsar**), `DESIGN.md:316,517`. Architecture — `product-graph` raw Kafka; **3-way eventing contradiction** (see CC-9). Governance — `templates/prd-template.md:250-251`, `platform-eventing-og/CHARTER.md` (Kafka topic contracts as canonical backbone); **Valkey/Pulsar = 0**. Products — `products/foundry/PRD.md:118` (Kafka), `:182` (Redis), `cloud/PRD.md:73` (Postgres/Citus/pgvector/Redis/Kafka/ClickHouse managed-service list). Footprint — Kafka 44 (only 3 co-mention Pulsar), Redis 49 (19 co-mention Valkey), Postgres 84 (all bridge-consistent).
- **Asymmetry:** **Redis→Valkey is the bright spot** (`dependency-policy.md`, `lts-versions-verified.md`, ADR-0336; PRD-CANONICAL bans Redis; tenancy/ontology PRDs use Valkey). **Kafka→Pulsar is materially less complete** — raw Kafka persists as endpoint in many event/asyncapi standards with no Pulsar framing.
- **Nuance:** `cloud/PRD.md:73` is a managed-service catalog SOLD to tenants — offering Redis/Kafka-compatible managed services can be legit; the fix is wording ("Valkey/Pulsar (Redis/Kafka-compatible)") not deletion.
- **Fix:** add transitional-bridge→owned-endpoint framing wherever a vendor is named as destination; complete the Kafka→Pulsar sweep; fix `_TEMPLATE.md`/`prd-template.md` so new docs inherit Valkey/Pulsar.

### CC-7 — Isolation: framekernel-host endpoint missing; native/secure-by-default framing instead of assume-breach-microVM-default — canon #7
- **Canon:** framekernel-host = COMMITTED endpoint; assume-breach **microVM DEFAULT** (NOT native-default / secure-by-default-native).
- **Where:** top-level — `PRD-CANONICAL.md:319` scopes Kata+Cloud-Hypervisor to *untrusted tenant code only* (native-default everything else); no framekernel/assume-breach. Architecture — microVM-default present and compliant, but framekernel-host endpoint absent (ADR-0254/0248); ADR-0023 still "Foundry sandbox." linux-nonadr — `conformance-gates.md:64` "microVM-per-pod as the **secure-by-default** boundary" + `cloud-native-stack.md:72` "secure-by-default defaults" (the exact retired phrasing). Standards soft-flag — default is `runc` with microVM as *upgrade* (ADR-0147), arguably weaker than microVM-DEFAULT.
- **Compliant (do NOT flag as native-default):** Cloud-Hypervisor/Kata/Firecracker microVM-default is canon-aligned; the only gap is the missing framekernel-host endpoint. `native-default`/`secure-by-default-native` exact phrasing = **0 hits corpus-wide** (the contradiction is missing-endpoint + the two `secure-by-default` softenings in linux-nonadr).
- **Fix:** add framekernel-host as the committed endpoint to the isolation ADRs/framing; rename the two `secure-by-default` phrasings to assume-breach vocabulary; ADR cross-check the runc-default-vs-microVM-default tension.

### CC-8 — Identity: oya-identity not owned-endpoint; Zitadel framed as canonical/authoritative — canon #6
- **Canon:** **oya-identity** owned (endpoint); Zitadel = BRIDGE.
- **Where:** top-level — raw OIDC/SAML primitives, no owned `oya-identity`, no Zitadel-bridge framing (`PRD-CANONICAL §6.2`, `DESIGN.md:543`, `GLOSSARY.md:900`); `oya-identity` = 0 top-level hits. Architecture — `adr-cross-reference-graph:251` echoes ADR-0187 "Canonical OIDC IdP: Zitadel primary"; `keystone-bundle-intern-walkthrough.md:119,132,223,229,231` makes Zitadel the runtime authoritative identity ("Zitadel issues JWT").
- **Compliant exemplars:** standards `identity-vendor-isolation.md` treats Zitadel as a confined bridge with explicit Phase-2 swap to owned `oya-identity-server` (the model to copy); journeys `IP-008 Zitadel adapter` (SCIM bridge). Zitadel-as-bridge is correct in 2 lanes — only top-level + architecture treat it as canonical.
- **Fix:** name oya-identity as the owned endpoint; reframe "authoritative identity = Zitadel" → "= oya-identity (Zitadel bridge during transition)"; flag ADR-0187 title to the ADR lane.

### CC-9 — Eventing tier is internally 3-way contradictory (Pulsar vs Redpanda+NATS vs Kafka) — canon #5 + internal-contradiction
- **Canon:** Kafka→Pulsar bridge toward owned endpoint. ADR-0005 = Pulsar backbone (aligned).
- **Where (real internal contradiction, not just stale term):** `keystone-bundle-idea-refine-deep-dive.md:297` asserts **Redpanda + NATS**; `:527-2772` repeatedly **NATS JetStream** ("Postgres CDC → Debezium → NATS JetStream"); `product-graph` still raw **Kafka**; while `adr-cross-reference-graph:1924` correctly echoes ADR-0005 **Pulsar**. Three mutually inconsistent backbones in one cluster.
- **Fix:** resolve to one canon line — Pulsar is the transitional bridge; either NATS is a scoped sub-component for sub-1s jobs (state it explicitly) or it is stale and dropped. Reconcile against ADR-0005.

### CC-10 — KR-first vs global-canonical: PRD.md internally contradicts itself — canon #10
- **Canon:** GLOBAL-CANONICAL core + localization packs; KR = FIRST pack to market (NOT KR-core).
- **Where:** `PRD.md:36,56,249` ("Korea-as-launch-locale is the test bed"; "Korea launches first") vs `PRD.md:91`/§12 + `DESIGN.md:591` ("earlier 'Korea-as-launch-locale' framing is **retired**" → global-canonical + KR-as-one-pack). PRD.md carries both framings unreconciled. Canon sides with the latter.
- **Compliant exemplars:** the 4 live `products/*/PRD.md` use canon-correct `W-*` gate-defined waves and `tenant_class`; `prds/INDEX.md:63` enforces the flat catalog.
- **Fix:** delete the KR-as-launch framing from PRD.md North Star/decision-log; keep global-canonical core + KR-as-first-pack.

### CC-11 — tenant-tier vocabulary (should be tenant-CLASS) — canon #9
- **Canon:** tenant-**CLASS** (not tenant-tier/tier-system). Namespaced `*_tier` (autonomy_tier/eu_ai_act_risk_tier/dr_tier/storage_tier) are CANONICAL — do NOT touch.
- **Where:** footprint 138 files; **129 are one identical persona boilerplate line** `tenant-tier-bound` (mechanical, see §2). Genuine content sites: standards `throttling-tiers.md`/`finops`/`per-tenant-quotas`/`tenant-lifecycle.md` (Free/Pro/Enterprise as tenant-*tier* axis) + spec `task-intel-autonomy-ceiling-tenant-tier-policy.md`; governance `stable-cohort-spec.md` ×2; top-level `DESIGN.md:78` "tenant-tier", `PRD.md:159` "per-tier consent."
- **Compliant exemplars:** all 4 product PRDs use `tenant_class` correctly; governance `dpia-template`/`foundry-capability-publishing.md:46` use tenant-class. Vocabulary is inconsistent within the corpus.
- **Carve-outs:** 2 retirement-doc FPs (`tier-system-retired-replaced-by-tenant-class`); `cell-tier` (runbooks, borderline namespace question — decide if sanctioned); authz edge/origin "tier" + IaC Tier-A/B/C tooling layers are NOT tenant tiers (fine).
- **Fix:** mechanical → tenant-class (template-level for the 129 personas); content review for the ~12 real plan-axis sites; glossary partition of the overloaded word "tier."

### CC-12 — M0-M3 / MVP wave-vocab still pervasive (retired → gate-defined waves) — canon #9
- **Canon:** M0-M3/MVP wave-vocab RETIRED → gate-defined waves. `glossary.json` already records `"old":"MVP / Milestone (M0..M3)"`.
- **Where:** footprint M0-M3 62 real (77 raw, 15 FP) + MVP 35. Products/prds — highest density: `prds/INDEX.md` milestone column (`M02b-substrate-ready`, `M03-first-paying-tenant`), every prd frontmatter `milestone_first_ship: M0x`, `prds/foundry.md:26` "M01–M12 milestones", `localization-packs/INDEX.md` + `kr.md` lead-milestones. linux-nonadr — `M1/M2` kernel-local labels (`testing-strategy.md`, `conformance-gates.md:36`).
- **Compliant exemplars (the migration template):** the 4 products PRDs + runbooks stubs use `W-Foundation`/`W-*` gate-defined waves; governance `glossary-vocabulary.md` enforces the retirement; `tactical-m3-launch` retirement is documented.
- **Heavy FP carve-out:** MacBook Air M3, m3.material.io, Gate M1 sales funnel, M2 meta-review pass labels, `M1-KB-F4` synthesis-fix IDs, Shamir M-of-N thresholds, ServiceNow cutover milestones (j179), runbook mitigation-step M1/M2/M3 labels. Per-file judgement required.
- **Fix:** replace genuine M0x with gate-defined wave IDs; reconcile the two coexisting vocabularies (products W-* vs prds M0x); treat customer-migration/device/pass-label M-tokens as FP.

### CC-13 — linux pilot mission docs sell the OLD "port the kernel" North-Star — canon #11
- **Canon:** maximal vertical scope, modernization NOT a port, own-endpoint cloud-OS.
- **Where:** `roadmap.md:4,60` ("safe, idiomatic Rust expression of the Linux kernel" / "entire Linux kernel in Rust via full replacement") + `source-parity-context.md:4` ("source-parity… migration of Linux kernel code") frame the whole program as a Linux port with zero mention of the hyperscaler mission, framekernel, Capsule model, or any ADR — contradicting the crystallized mission in `cloud-native-stack.md:3-9`.
- **Disposition:** these are genuine Phase-0 historical artifacts (the C2Rust leaf-port really happened in `port/`); re-scope explicitly as "Phase-0 port provenance / `port/` supplier history" demoted under the live mission, NOT presented as the goal.
- **Compliant exemplars:** `component-boundaries.md` + `engineering-conventions.md` (§12 design-to-owned-ideal) are fully canon-aligned references.

---

## 2. MECHANICAL STALE-TERM SWEEP PLAN

Per-term rename rules with counts and per-context routing. **Golden rule:** fix the GENERATOR/TEMPLATE, not the output files, wherever the term arrives via boilerplate (personas, journeys, PRD template, runbook stubs) — O(1) instead of O(N) and prevents drift.

### foundry → intelligence | governance (SPLIT, sense-routed) — 831 files (ADR-excluded)
> **CENSUS CORRECTION (2026-06-06, A.0-1).** This register's original mechanical figures (731 non-ADR files; 105-file Palantir-Foundry carve-out) were a raw `grep` floor and are STALE. The census-of-record is the SSOT verification correction at `../synthesis/decision-record-oyatie-canon.md:110` ("total non-ADR = 831 (not 731), Palantir-Foundry carve-out = 43 files (not 105)" — spot-checked vs real files; 831 carries a sampled journeys/personas tail above the grep floor, 43 is the curated Palantir-the-product subset). The corrected figures (831 / 43) are now used throughout this section. No other figure on this page is changed.
- **NOT a uniform swap.** Route per token family:
  - `oya-foundry-*` / `axis-foundry` / `ai foundry` / agent-runtime / provider / capability / RAG / model / sandbox / supervisor sense → **cloud-intelligence** (~274 files near intelligence/agent/provider/capability/model/adapter).
  - `foundry-fitness` / `council-foundry` / `governance-foundry` / `amendment-foundry` / Proof-Ladder / fitness-lane / CI-gate sense → **governance** (~135 files; 29 carry explicit governance tokens).
- **HARD CARVE-OUTS (must NOT be swept):**
  - **Palantir Foundry** (43 files, census-of-record per A.0-1; third-party product name — formerly mis-counted as 105).
  - **`Marlboro-Forge` / `forgery`** (journeys) — fictional company / security-event, not VCS-forge.
  - **`foundry-fitness-to-governance-transition-2026-05-21.md` + `transition-classification.json`** — the retirement RECORD itself (canon-compliant; covers only the governance sense).
- **Operational caution:** verify binary/package/service names against `microservices/` before swapping doc commands — `foundry/supervisor/lifecycle.md` ships binary `oya-foundry-supervisor` but builds package `oya-intelligence-supervisor-app` (a live name-lag bug, not just stale text). The `oya-foundry-brand-residue-kernel` enforcer itself carries the retired brand and its checklists (`pre-push.md:30`, `pre-merge.md:32`) police only `Oyatie`, never `foundry` — wire `foundry` into the residue token list.
- **Half-migration state:** governance side ~73% landed at crate level (`oya-governance-*` 204 refs vs `oya-foundry-*` 75); intelligence side barely started (`oya-intelligence-*` 6 refs). Complete in one pass; fitness lane must reject mixed prefixes per seam.

### tenant-tier / tier-system → tenant-class — 138 files
- **129 are one identical persona boilerplate line** (`tenant-tier-bound` at `:161`) → fix the persona GENERATOR/template, not 129 files. Remaining ~9 are genuine plan-axis content (standards/governance/top-level) → per-site swap.
- **Do NOT touch** namespaced `*_tier` identifiers (autonomy_tier/eu_ai_act_risk_tier/dr_tier/storage_tier). **Carve out** 2 retirement-doc FPs. **Decide** `cell-tier` (borderline namespace).

### M0-M3 / MVP → gate-defined waves — 62 real (M0-M3) + 35 (MVP)
- CONTENT-CHANGE, per-file (NOT a token swap — the framing is stale). Adopt the products `W-*` gate-defined wave names as the target vocabulary.
- **Carve out 15+ FP families** (devices, URLs, sales-funnel gates, review-pass labels, synthesis-fix IDs, Shamir thresholds, ServiceNow cutover milestones, runbook mitigation-step labels). Some hits are the retirement record itself (glossary.json) — do-not-amend.

### Redis → Valkey — 49 files
- 19/49 already co-mention Valkey (near-mechanical residue) → finish framing. 30/49 raw Redis → CONTENT-CHANGE (add Redis→Valkey transitional-bridge framing). Identifier/env-var tokens (`LIVEKIT_REDIS_ADDR`, `oya-…-redis`) are mechanical. Substring FPs: `redistribute`/`yellow_redistribute` (journeys). Fix `_TEMPLATE.md:159,166` + `prd-template.md:250` so new docs inherit Valkey.

### Kafka → Pulsar — 44 files
- **Less complete than Redis→Valkey.** Only 3/44 co-mention Pulsar; 41 are raw Kafka needing transitional-bridge framing. CONTENT-CHANGE for the framing; `adapter-kafka`/`oya-platform-eventing-adapter-kafka` tokens are mechanical. Fix `_TEMPLATE.md:66` + `prd-template.md:251`. **Also reconcile the 3-way eventing contradiction (CC-9)** — Pulsar is the canon line, Redpanda+NATS must be scoped or dropped.

### Jenkins — 35 files — CONTENT-CHANGE (not a rename)
- Reframe to oya-ci endpoint; Jenkins = operative-until-cutover bridge (build-first-cutover-later), NOT canonical. **Carve out** journeys FP (fictional "Tom Jenkins", j130 — 5 files, 100% FP). Architecture/governance Jenkins = 0 (clean). The Jenkins-as-bridge framing inside the Forgejo runbook is canon-shaped — keep it, just repoint at oya-ci.

### Forgejo — 21 files — CONTENT-CHANGE (not a rename)
- Forgejo DROPPED → GitHub now / bespoke VCS later; mirror-at-most. Re-scope the 4 runbooks + the 2 webhook-replay specs + the CI-design current-shape doc to GitHub-primary. Flag ADR-0377 to the ADR lane. Architecture/governance Forgejo = 0 (clean).

### Cedar engine → Cedar (contract) + PARC (engine) — 170 raw → 24 genuine
- CONTENT-CHANGE on the 24 genuine "Cedar AS engine" assertions only. **Do NOT mass-rewrite** the 146 FP (`Workflow Engine`, `cedar-policy-engine` IaC paths, `Cedar policy-engine logs` audit subsystem, `Cedar denial event` signals). Introduce PARC as the owned PDP where the engine is named.

### Mechanical-sweep totals (ADR-excluded file counts)

| Term | Files | Genuine after carve-out | Fix class |
|---|---|---|---|
| foundry | **831** | sense-routed (~274 intelligence / ~135 governance; 43 Palantir carve-out) | SPLIT mechanical |
| tenant-tier \| tier-system | **138** | ~9 content + 129 template-line | mechanical + FP carve |
| M0-M3 (bare) | **77** | **62 real** | content + heavy FP |
| MVP | **35** | content | content-change |
| Redis | **49** | 30 raw + 19 partial | mostly content |
| Kafka | **44** | 41 raw (3 co-Pulsar) | mostly content |
| Jenkins | **35** | content (−5 journey FP) | content-change |
| Forgejo | **21** | content | content-change |
| Cedar.*engine | **170 raw** | **24 genuine** | content (subset) |
| native-default / secure-by-default-native | **0** | n/a (2 soft `secure-by-default` in linux-nonadr) | n/a |
| eliminate-Postgres | **0** | n/a (Postgres 84, all bridge-consistent) | n/a |

---

## 3. AI-SLOP / STALE / PLAIN-WRONG (delete / rewrite candidates)

| Doc | Problem | Action |
|---|---|---|
| `onboarding/intern-month-one.md` | 138 foundry hits = dozens of duplicated templated day-blocks varying only the ADR id + a ritual `rg foundry` step; fabricated-precision filler. **Highest-severity slop in the corpus.** | Regenerate from one parameterized template; drop the foundry-grep ritual. |
| `erp-coverage/PRD.md` | ~150× verbatim-repeated acceptance sentence (one per SAP module); inflates to 2,514 lines, zero per-module info. | Collapse to one cross-module acceptance contract + a delta table; deslop. |
| `DESIGN.md §3.0.5.5` payback table (`:211-223`) | Fabricated precision ("~2 weeks ⇒ thousands of $/month", "1-2 reviewers worth of leverage"). | Rewrite/remove; cite basis or delete. |
| `PRD.md §4.1` success metrics (`:145,:147`) | Fabricated-precision targets with circular "Why this number" rationales ("Below 3 = not a product"). | Rewrite rationales or drop the numbers. |
| `unified-ecosystem-thesis-2026-05-21.md` | Self-confessed padding (`revision_history`: "v1: 7,369 lines, 700 Thesis-clause repetitions") + fabricated `line_floor:2500` (mandating min length induces padding) + unsourced $ figures (fenced as internal-only). | Verify v2 collapse removed the loops; drop `line_floor`; keep $ figures fenced-internal only. |
| `day-in-the-life-coherent-ecosystem-2026-05-21.md` (1.05 MB) / `enterprise-software-coverage-matrix-2026-05-21.md` (2.05 MB) | Size/padding + reachability liability (862 / 598 Cedar mentions; 180 foundry). | Size/slop review; not read in full. |
| 136 runbook stubs (66% of lane) | Empty `TODO — fill at W-Foundation` skeletons; "207 runbooks" is a false coverage signal (only 71 real). Several safety-critical-and-empty (`foundry-robotics-safe-stop.md`, `industrial-ot-write-emergency-stop.md`, `healthcare-break-glass.md`) → canon #12 safety gates have no runnable procedure. | Stop counting stubs as runbooks; wave-bind; author the safety-critical ones FIRST. |
| `cloud-native-stack.md:91` | "Redis **+40%** vs Linux" fabricated/inconsistent vs the doc-family's own Asterinas **1.31× (+31%)** (`testing-strategy.md:2,169,545,553`). | Fix → "+31% (1.31×)". |
| `testing-strategy.md` | AI-slop process-narration ("I now have the precise repo layout…", `:7`) in an otherwise sound methodology doc. | Strip process-narration before canon. |
| `release/` ↔ `advanced-cicd/` duplicate subtrees | Near-byte-identical `progressive-delivery/` + `branch-pipeline/` trees; divergence risk + double the residue surface. | Single source + include, or pick one home. |
| `anti-patterns.md` (2914L), `cedar-policy-authoring.md` (806), `layer-enum-adr-0105.md` (834), `naming-convention-bnf-v4.md` (825) | Blow the declared 250-line cap (`INDEX.md:92`). | Length-cap review (anti-patterns is user-mandated; others trim). |
| `hyperscaler-best-practices.md` | Verbatim TL;DR/conclusion duplication; research-dump. | Mark GENERATED-REFERENCE / archive, don't maintain as living standard. |
| `products/README.md` | "7-axis + 14-vertical" taxonomy with ~16 DEAD PRD links (none exist) + contradicts the flat-catalog ruling (`prds/INDEX.md:63`). | Regenerate from the live tree; drop axis/arm framing. |

---

## 4. REFINEMENT OPPORTUNITIES

- **Glossary additions:** add PARC (owned engine), framekernel-host, assume-breach-microVM, oya-ci, oya-identity, Pulsar, tenant-class as canonical terms; mark Cedar-as-engine / Jenkins-as-endpoint / foundry / tenant-tier as retired tokens in the GLOSSARY *top* tables (not just the bottom narrative). Partition the overloaded word "tier" (tenant-class vs namespaced-tier vs authority-tier vs edge/origin-tier vs IaC Tier-A/B/C).
- **Wire the brand-residue lane:** add `foundry` to the residue token list; rename `oya-foundry-brand-residue-kernel` → `oya-governance-*`; the brand-check checklists must police `foundry`, not only `Oyatie`.
- **Reconcile dual vocabularies:** products `W-*` waves vs prds `M0x`; products `tenant_class` vs standards `tenant-tier` — converge on the canon-correct product-PRD pattern as template.
- **Generated-from-source:** `products/README.md` and `MASTERPLAN.md` should be GENERATED (mirrors the canon #1 principle); extend `adr-index-pipeline.md`'s model to the masterplan.
- **Templates are the leverage point:** fix `products/_TEMPLATE.md`, `templates/prd-template.md`, `templates/runbook-template.md`, the persona day-summary template, and the journey handshake-component generator — every downstream doc inherits the fix.
- **Use the best-aligned docs as reconciliation templates:** `PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` (Valkey/Kyverno/OpenBao/Iceberg/SPIFFE correct), `identity-vendor-isolation.md` (Zitadel-bridge done right), `component-boundaries.md` + `engineering-conventions.md` (§12 design-to-owned-ideal), the 4 products PRDs (W-* + tenant_class), the `task-*` spec slices (tight, single-crate, ADR-cited — the healthiest part of the corpus).
- **Name PARC explicitly when authoring** any policy-evaluation doc, so the Cedar=contract / PARC=engine split is legible (PARC = 0 hits everywhere today).
- **Author the canon #12 safety-gate set** (HITL / no-actuation / biometric-off / no-lethal) — currently an absence/gap in product docs, with no runnable runbook backing.

---

## 5. REACHABILITY CLASSIFICATION SUMMARY

**DECISION → ADR** (fixes require ADR amendment): all CC-1..CC-13 dispositions. Flag to the ADR lane: ADR-0335 (foundry), ADR-0025 (Foundry-consolidation), ADR-0187 (Zitadel-primary), ADR-0160/0171/0240/0349 (Argo/Jenkins), ADR-0377 (Forgejo board), the authority-chain ADR, ADR-0005 (Pulsar vs Redpanda+NATS), ADR-0254/0248 (framekernel endpoint).

**GENERATED-REFERENCE** (regenerate post-amendment, don't hand-edit): MASTERPLAN.md (currently mislabeled as authority — CC-3), DESIGN/PRD/COMPETITIVE-GAP/GLOSSARY, the architecture `*-line-audit`/`*-cross-reference`/`product-graph` indexes (term-hits are ADR-echoes, not assertions), journeys + personas (acceptance-narrative layer — RE-GENERATE via templates), the 4 products PRDs + per-µservice prds, INDEX files, standards INDEX/lts-versions/hyperscaler-best-practices/ci-lanes, automation pipelines, governance-lane specs, linux research syntheses.

**INSTRUCTION → session-context-bundle:** DOC-CATALOG, MISTAKES-LEDGER, checklists, onboarding, teams charters, `brief-template`/`agent-instructions-discipline`/`claude-code-harness`, `_TEMPLATE.md`, `cloud-native-stack.md`/`engineering-conventions.md`, the migration-batch/slice pilot prompts (retire-at-integration).

**ORPHAN → not-needed / archive candidates:**
- `CONTRADICTION-LEDGER.md` — stale 2026-05-09 draft with superseded foundry resolutions → ARCHIVE-or-regenerate.
- `corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md` + all 05-20 wave duplicates where a 05-21 exists → archive (superseded-by-construction).
- `specs/deep-dive-oyatie-sst-consolidation.md` + `deep-dive-trace-*` — pinned to the retired 2026-05-09 Builder-OS→Foundry reframing → archive once CC-1/CC-3 land.
- `products/README.md` → needs-regeneration (broken-link aspirational catalog).
- `products/product-docs-w1-2026-05-20-checkpoint.md` → one-shot wave checkpoint, archive.
- `products/foundry/supervisor/**` (21 files) → unaudited bominal port; ORPHAN unless reachable from the post-CC-1 cloud-intelligence PRD; BENCHMARKS files = fabrication risk.
- 4 `forgejo-*` runbooks + `kafka-topic-provisioning.md` stub → retire/mirror-only if the board spike is dead.
- `advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md` — an ADR living outside `decisions/` (SSOT smell) → should exist only in the ADR pack.
- `roadmap.md` / `source-parity-context.md` / `phase2-context.md` / `phase3-context.md` (rkernel) — superseded Phase-0 port-mission content → re-scope as port-provenance, demote under live mission.

---

## 6. COVERAGE HONESTY (no silent truncation)

**Verified, not asserted:** all 8 lane artifacts + the footprint were read in full and cross-checked against their returned digests before this merge; the digests are faithful (carve-outs, FP notes, and coverage caveats present in both).

Per-lane depth (deep-read = full/block read · scanned = grep+sampled · counted = file-level grep only):

- **footprint (10-):** counted-only by design (ADR-excluded `grep -rliI`), with big-five mechanical-vs-content sampling. Honest that brief's quoted numbers (foundry 830 etc.) are full-corpus INCLUDING ADRs; post-exclusion is smaller. Both reported.
- **toplevel-canonical:** 8 deep-read + 1 skim (GLOSSARY 1149L). Highest-signal docs; complete.
- **architecture:** 45 files; **9 deep-read, 36 scanned**. Honest: the large `*-line-audit`/`cross-reference` docs are GENERATED indexes (term-hits = ADR-echoes), verified by line-anchored sampling, NOT fully read. 1MB+2MB docs flagged-not-read (size).
- **standards-specs:** load-bearing standards deep-read; ~100 `task-*` spec slices sampled. Complete on the contradiction set; specs are the healthiest, lightly sampled.
- **products-localization:** products README/template/checkpoint + localization INDEX/kr deep-read; foundry PRD deep-read; cloud deep head+grep body; erp-coverage + workplace (~2,500L each) grep-targeted, NOT line-by-line; 21-file supervisor cluster counted+name-scanned only (flagged ORPHAN). Honest that "34/14" product/pack counts are aspirational — only 4 PRDs + 1 pack physically exist.
- **governance-process:** 274 docs, counts matched task spec exactly. Term disambiguation honest (argo hits = `cargo`/`c-argo` FP; foundry/jenkins/forgejo line = ~100% foundry). Half-migration quantified (204/75/6 crate refs).
- **runbooks:** 207 files; 71 substantive deep/sampled, 136 empty stubs identified. Honest that the M0-M3 hits are `M1-KB-F4`/Shamir FP (confirmed in lane-6 follow-up).
- **journeys-personas:** 1044 docs **counted exhaustively at file level**; ~10 deep/contextual reads. **Explicitly NOT a full read** — the genuine-contradiction residue (C-1 Cedar j167, C-2 Redis ~3 files) is **confirmed-present but its exact file-count is a SAMPLED ESTIMATE, not a census**. Confidence HIGH on mechanical/FP verdict, MEDIUM on genuine-residue size. This is the one lane carrying an explicit estimate-not-census caveat.
- **linux-nonadr:** judgment (not mechanical) read of `linux/docs/{context,migration,research}/`; per-doc dispositions given. `rust-engineering-guardrails.md` (45KB) grep-scanned not deep-read (no canon surface found).

**Net:** every lane reports what it deep-read vs scanned vs counted; no lane silently claims full coverage it didn't achieve. The single carried-forward uncertainty is the exact size of the journeys/personas genuine-contradiction residue (estimated "a handful", not censused).
