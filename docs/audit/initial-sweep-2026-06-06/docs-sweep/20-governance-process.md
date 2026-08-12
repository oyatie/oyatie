# Rest-of-Docs Review — Lane: governance-process (20)

**Reviewer lane:** governance-process
**Source corpus reviewed:** `/Users/jasonlee/Developer/source/docs/` (canonical `feat/oya-ci-tide` checkout — the live oyatie repo, NOT a `/private/tmp` transient worktree)
**Groupings + file counts (all confirmed against task spec):**
governance-lanes (65), checklists (31), templates (29), release (28), teams (40), advanced-cicd (39), automation (19), agents (11), onboarding (12). **Total: 274 docs.**
**Date:** 2026-06-06

> Method note: term counts were disambiguated. The headline "argo" hits were **almost entirely `c-argo`/`cargo` false positives** (cargo-vet, cargo nextest, cargo run); real **Argo = Argo Rollouts** only. The headline "foundry/jenkins/forgejo" line-counts are **~100% foundry** — `jenkins` and `forgejo` have **0 file hits** anywhere in this slice (clean on those). So the dominant — effectively sole — canon contradiction in this lane is the **retired `foundry` brand**.

---

## TL;DR — Top canon contradictions (lead)

1. **[CANON #2 — foundry brand RETIRED] is the systemic violation of this entire lane.** `foundry` survives in **151 files / ~640 line-hits**, including a first-class team/axis (`teams/axis-foundry/CHARTER.md`), a progressive-delivery playbook (`playbook-foundry.md`), 75 distinct `oya-foundry-*` crate refs, and `axis-foundry` cited as the `Owner:` of essentially every advanced-cicd/release spec. Worse: the brand is doing **double duty across BOTH canon replacement names at once** — see #2 below.
2. **[CANON #2] `teams/axis-foundry/CHARTER.md` collapses the two things canon explicitly splits.** Its Mission owns *both* "Agent Runtime (Foundry — Axis 3)" (the AI/agent substrate → canon **cloud-intelligence**) *and* "Foundry (Axis 4)" = fitness functions / gates / CI lanes / scorecards / catalog (→ canon **governance**). One retired brand now names both successor lanes. `teams/axis-foundry/CHARTER.md:7-11,13-43`.
3. **[CANON #1 — masterplan is GENERATED from ADRs, NOT hand-authored authority] is contradicted by the doc-governance model.** MASTERPLAN.md is classified **authority-class, human-only, council-edited** (`../../../../templates/checklists/escalation-checklist.md:34` "authority-class docs are human-only by design"; `agents/CROSS-REFERENCE-INDEX.md:31` "Tier 0; amend-only-by-Founder"), and **81 "Directive N" citations** across the slice treat masterplan Directives as the binding root authority (e.g. `release/.../progressive-delivery-strategy.md:25` "per Directive 4"; `release/branch-pipeline/agent-roles-spec.md:81` "per Directive 2"). Canon says the masterplan is a *generated view of the ADRs*; here it is the apex hand-authored source. (Note: exactly one doc gets the generated-from-ADR model right — `automation/adr-index-pipeline.md:29` — but it scopes it to ADR-INDEX, not the masterplan.)
4. **[CANON #9 — tenant-CLASS, not tenant-tier] residual.** `tenant-tier` survives in 2 files: `release/progressive-delivery/stable-cohort-spec.md:57` and its advanced-cicd duplicate `advanced-cicd/progressive-delivery/stable-cohort-spec.md:58` ("Foundry capabilities have a **tenant-tier** ceiling"). Elsewhere `tenant-class` IS used correctly (dpia-template §7, foundry-capability-publishing.md:46, axis-workspace CHARTER), so this is a stale residue, not a systemic miss.
5. **[CANON #6 — Cedar = CONTRACT, owned PARC = engine] framing drift.** Cedar is pervasively framed as the **engine** ("Cedar policy + runtime gate", `cedar_policy_ref`, "T2→T3 uplift Cedar policy"), and **PARC appears 0 times** in the entire slice. No doc positions Cedar as a contract over an owned PARC engine. `platform-tenancy-identity/CHARTER.md:8` even calls "Cedar policy ... the single source of truth for RBAC/ABAC decisions" — engine framing.
6. **[CANON #5 — data tier is OWN-endpoint; Postgres/Citus/ClickHouse/Redis/Kafka are TRANSITIONAL bridges] framing drift.** These are framed as canonical infra, not bridges: `templates/prd-template.md:250-251` ("Postgres + Citus for tenant-bound state", "ClickHouse replicas"); `teams/platform-eventing-og/CHARTER.md` owns "Kafka topic contracts" + `oya-platform-eventing-adapter-kafka` as the canonical eventing backbone. **Valkey / Pulsar appear 0 times** — the Redis→Valkey / Kafka→Pulsar ratchet is unrepresented.

---

## Canon items that are CLEAN or COMPLIANT in this lane (do not re-flag)

- **[CANON #3 — Forge: GitHub NOW → bespoke later; Forgejo DROPPED]** — COMPLIANT. Docs are explicitly git-server-agnostic: `release/branch-pipeline/branch-protection-rules.md:11-12,28` ("Git-server-agnostic; same shape on GitHub / GitLab / …", schema-as-source-of-truth + per-provider adapters); `../../../../templates/checklists/inventory-update-checklist.md:63` abstracts "VCS/forge tools". **Forgejo = 0 hits.** `gitea` appears only as one of four enumerated branch-protection *adapters* (alongside github/gitlab/bitbucket) — a mirror-at-most posture, canon-OK. *Only nit:* the adapters are named `oya-foundry-branch-protection-adapter-*` (foundry brand, see #1).
- **[CANON #4 — unified oya-ci; Jenkins/Argo OPERATIVE-until-cutover, not endpoint]** — **Jenkins = 0, Argo-CD = 0** in this slice. **Argo Rollouts** (the only real "Argo") is correctly framed as a *secondary sanctioned controller* behind a provider-agnostic `oya-platform-rollout-controller-kernel` adapter (`release/progressive-delivery/progressive-delivery-strategy.md:48-50`, `canary-rail-spec.md:26-27`) — own-endpoint/vendor-bridge pattern, canon-COMPLIANT. The CI authority itself, however, sits under foundry branding (`oya-foundry-ci-runner-*`, see #1), and `oya-ci`/`Prow`/`Tekton` are **absent** here (they live in the dedicated ci/ tree, out of this slice).
- **[CANON #9 — M0-M3/MVP wave-vocab RETIRED → gate-defined waves]** — Largely **handled correctly**. `governance-lanes/glossary-vocabulary.md` is a *fitness lane that enforces* the retirement; `teams/README.md:202` + `teams/tactical-first-vertical-pilot/CHARTER.md:99-105` document the `tactical-m3-launch` retirement and the `Foundation → Substrate → Axis-Preview …` wave language; `../../../../templates/checklists/wave-gate.md` uses gate-defined waves (W-Foundation). Residual M-tokens are **false positives** (runbook-template-v2 uses M1/M2/M3 as *mitigation-step labels*; escalation-checklist.md:46 / implementation-plan-template.md:41 reference "MVP-shaped scope"/"no MVP placeholders" as *anti-patterns*). Not a violation.
- **[CANON #9 — "tier" namespaced]** — `autonomy_tier` / `AutonomyTier` / T1-T4 usage is **canon-SANCTIONED** and must NOT be flagged. (Borderline: `persona-tier` in `automation/glossary-pipeline.md:39` and `ADR-0007` filename is not in the sanctioned namespace list — mild stale-vocab, low priority.)
- **[CANON #7 — framekernel-host / assume-breach microVM]** — **0 hits** in this slice (isolation lives in the platform/architecture tree). Not covered here; no contradiction to flag, just out of scope.

---

## The half-migration — quantified (the most actionable single finding)

The foundry→{governance, cloud-intelligence} rename **landed at the crate level for the governance side, but nowhere else, and barely started for the intelligence side:**

| Successor lane | distinct crate refs in slice | status |
|---|---|---|
| `oya-governance-*` (fitness/policy) | **204** | rename mostly landed at crate level |
| `oya-foundry-*` (still retired brand) | **75** | rename NOT landed |
| `oya-intelligence-*` (AI substrate) | **6** | rename barely started |

Symptoms of the incoherent in-flight state:
- **Self-referential irony:** the brand-hygiene enforcer itself carries the retired brand — `governance-lanes/brand-residue.md:11` + `INDEX.md:26`: kernel `oya-foundry-brand-residue-kernel` (EXISTING) runs via binary `tools/oya-governance-brand-residue`. The thing meant to catch brand residue *is* brand residue.
- **Brand-check checklists guard the wrong brand:** `../../../../templates/checklists/pre-push.md:30` and `pre-merge.md:32` only police the `Oyatie` string (per ADR-0017); **neither mentions `foundry`** — so the retired brand is explicitly *outside* enforced brand hygiene. The rename was never operationalized as policy.
- **Mixed crate prefixes in the SAME charter:** `teams/axis-foundry/CHARTER.md` lists `oya-foundry-capability-kernel` *and* `oya-governance-*` (fitness crates) *and* future `oya-intelligence-api` / `oya-foundation-app` side by side — three brand generations in one ownership table.
- **`contracts/openapi/foundry/` + `oya-intelligence-rag-api`** coexist (`onboarding/ai-platform-engineer-month-one.md:108-113,131`) — contract path on old brand, crate on new brand.

---

## Findings grouped by doc / doc-group

### governance-lanes/ (65) — foundry: 18 files / 60 lines
- **`brand-residue.md` / `INDEX.md:26`** — CANON #2: brand-hygiene lane keyed to `oya-foundry-brand-residue-kernel`. *Refinement:* rename kernel to `oya-governance-brand-residue-kernel` and **add `foundry` to the residue token list** so the lane catches its own brand.
- **`foundry-corpus-citation.md`** — CANON #2 + #1: lane name + concept "foundry corpus" treats foundry as permanent canonical; "enforces: MASTERPLAN P3.5 — foundry corpus cross-cite" cites masterplan as a numbered-principle authority (canon #1 drift). *Reachability:* GENERATED-REFERENCE (lane spec) but its enforced-clause anchor is a masterplan principle, which under canon should derive from an ADR.
- **Bulk of the 65 lane-specs** are well-formed GENERATED-REFERENCE (`oya-governance-*-kernel` + `cargo run -p` invocation + severity + budget). Internally consistent, low slop. Residual foundry appears mainly in kernel names and "Owner: axis-foundry".
- Reachability: lane-specs = GENERATED-REFERENCE (pointed at by `INDEX.md` + AGENTS fitness-lane list). Healthy.

### checklists/ (31) — foundry: 17 files / 25 lines
- **`escalation-checklist.md:34`** — CANON #1: codifies MASTERPLAN/PRD/CONSTITUTION as "authority-class … human-only by design" — the anti-generated stance. **Lead contradiction #3 source.**
- **`escalation-checklist.md:48` / `done-definition-checklist.md:77` / `pr-review-checklist.md:92` / `pre-flight-checklist.md:66`** — CANON #6 framing: Cedar treated as the gate engine for T2+ uplift (no PARC).
- **`foundry-capability-publishing.md`** — CANON #2 (filename + content). Note line 46 *correctly* uses "Tenant-class override" (canon #9 good) — so vocab is inconsistent within the same lane family.
- Checklists are otherwise high-signal, low-slop, INSTRUCTION-class (session-context bundles for agents). Reachable.

### templates/ (29) — foundry: 11 files / 30 lines
- **`capability-record-template-v2.yaml:59`** `cedar_policy_ref: policies/foundry/<id>.cedar` — CANON #2 path + #6 Cedar-as-engine.
- **`prd-template.md:250-251`** — CANON #5: Postgres+Citus / ClickHouse as canonical (no bridge framing). **`prd-template.md:85`** references "ADR-0140 (retired per ADR-0145)" — stale ADR reference left inline (refinement: drop or update).
- **`runbook-template-v2.md:107`** `oya foundry capability tier-set` — CANON #2 in an example CLI; the M1/M2/M3 here are mitigation-step labels (NOT wave vocab — do not flag).
- Templates are TEMPLATE/GENERATED-REFERENCE class, reachable via `STANDARDS-AND-TEMPLATES.md` / `templates/INDEX.md`.

### release/ (28) — foundry: 19 files / 67 lines
- **`progressive-delivery/stable-cohort-spec.md:57`** — CANON #9 `tenant-tier` (lead #4) + CANON #2 "Foundry capabilities".
- **`progressive-delivery/playbook-foundry.md`**, **`branch-pipeline/foundry-pipeline-mirror.md`** — CANON #2 in filenames (brand baked into the doc tree, not just content).
- **`branch-pipeline/branch-protection-rules.md`** — CANON #3 COMPLIANT (git-server-agnostic) but uses `oya-foundry-branch-protection-adapter-github` (#2).
- Argo Rollouts usages here are all canon-COMPLIANT (see clean-list).
- **Duplication note:** `release/` and `advanced-cicd/` share near-identical `progressive-delivery/` and `branch-pipeline/` subtrees (same specs, differing only in relative ADR link depth `../../` vs `../../../`). This is a structural redundancy / divergence risk — see AI-slop section.

### teams/ (40) — foundry: 24 files / 115 lines
- **`axis-foundry/CHARTER.md`** — the keystone CANON #2 violation (lead #1/#2). 46 foundry hits in one file; owns both successor lanes.
- **`teams/README.md`** — 16 foundry hits; lists `axis-foundry` as a live axis. Correctly documents the `tactical-m3-launch` retirement (canon #9 good).
- Every vertical/platform/ops CHARTER carries ≥1 `axis-foundry` cross-axis dependency row — so the retired brand is load-bearing across the whole team graph. Renaming axis-foundry → {governance + cloud-intelligence} ripples into ~all 40 charters.
- Reachability: charters = INSTRUCTION/DECISION-adjacent, reachable via `teams/README.md`. Healthy structure, wrong brand.

### advanced-cicd/ (39) — foundry: 34 files / 130 lines
- Highest foundry file-density. Mirrors `release/` (duplicate progressive-delivery + branch-pipeline trees).
- **`branch-pipeline/ADR-0055-branch-pipeline.md`** lives under advanced-cicd rather than `decisions/` — an ADR copy outside the ADR tree (reachability/SSOT smell; the ADR pack should be the only home per `automation/adr-index-pipeline.md:29`).
- `oya-foundry-ci-runner-adapter-{github-actions,gitlab-ci,buildkite,circleci}` — CI-agnostic adapter set (canon-OK pattern) under retired brand (#2).

### automation/ (19) — foundry: 19 files / 107 lines
- **`adr-index-pipeline.md:29`** — the **one canon-#1-CORRECT doc**: "ADR pack … is the source of truth … ADR-INDEX.md is a derived view … manual edits are rejected." *Refinement:* extend this exact generated-from-ADR model to MASTERPLAN to resolve lead #3.
- **`openapi-pipeline.md` / `architecture-map-kernel-spec.md` / `service-map-spec.md` / `audit-chain-map-spec.md`** — solid generated-artifact pipelines; foundry residue in kernel names only.
- Reachability: GENERATED-REFERENCE pipelines. Healthy.

### agents/ (11) — foundry: 3 files / 6 lines
- Lowest foundry density (cleanest grouping). **`CROSS-REFERENCE-INDEX.md:31`** is the canon-#1 evidence (MASTERPLAN "Tier 0; amend-only-by-Founder").
- Otherwise agent-protocol docs (cheat-sheet, decision-tree, completion-protocol) are INSTRUCTION-class and reachable via `agents/INDEX.md`.

### onboarding/ (12) — foundry: 6 files / 217 lines
- **`intern-month-one.md` (138 foundry hits) — AI-SLOP, highest-severity slop in the lane.** The file repeats a near-identical templated block dozens of times, varying only the ADR number: *"Apply `foundry` doctrine to doctrine replay and first reviewed contribution with ADR-XXXX as the decision anchor … discovered with `rg -n "foundry" docs specs crates | sed -n '1,20p'` … cite GLOSSARY.md, ADR-XXXX, and `foundry` term if present."* This is mechanical fabricated-precision filler (canon #2 brand × AI-slop). Recommend regenerate from a single parameterized template, drop the foundry-grep ritual.
- **`ai-platform-engineer-month-one.md` (59 hits)** — long enumerated file lists mixing `contracts/openapi/foundry/*` (old brand) with `oya-intelligence-rag-api` (new brand) — the half-migration made visible to new hires.
- Reachability: onboarding = INSTRUCTION-class, reachable. But the intern-month-one slop pile is a stale/garbage candidate for the >48h stale-file audit (Task #14).

---

## AI-slop register (this lane)

1. **`onboarding/intern-month-one.md`** — dozens of duplicated templated day-blocks with only the ADR id swapped + a ritual `rg foundry` instruction. Fabricated precision + repetition. **Highest-severity.**
2. **`release/` ↔ `advanced-cicd/` duplicate subtrees** — `progressive-delivery/` and `branch-pipeline/` exist in both, near-byte-identical except link depth. Divergence risk (edit one, forget the other) + double the foundry/tenant-tier residue surface. Recommend single source + symlink/include, or pick one home.
3. **`templates/prd-template.md:85`** — inline "ADR-0140 (retired per ADR-0145)" stale cross-ref carried in a template (every PRD spawned from it inherits the dead reference).
4. Low-grade: `persona-tier` vocab drift (`automation/glossary-pipeline.md:39`) vs the namespaced-`tier` rule.

---

## Reachability summary
- **GENERATED-REFERENCE** (healthy): governance-lanes/* lane-specs, automation/* pipelines, templates/*, agents/* protocol docs.
- **INSTRUCTION → session-context-bundle** (healthy): checklists/*, onboarding/* (modulo slop), teams/* charters.
- **DECISION → ADR smell:** `advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md` — an ADR living outside `decisions/`; should be reachable only via the ADR pack (SSOT), not duplicated in a docs subtree.
- **ORPHAN/not-needed candidates:** the `release/` vs `advanced-cicd/` duplicated subtrees (one should be canonical, the other archived or replaced by an include).

---

## Counts at a glance
- foundry: **151 files**, ~640 line-hits (governance-lanes 18, checklists 17, templates 11, release 19, teams 24, advanced-cicd 34, automation 19, agents 3, onboarding 6).
- jenkins: **0** · forgejo: **0** · Argo-CD: **0** · gitea: 1 (adapter enum, OK) · Argo Rollouts: present, canon-compliant.
- tenant-tier: **2 files** (stable-cohort-spec ×2). tenant-class: used correctly elsewhere.
- PARC: **0** · Valkey: **0** · Pulsar: **0** · cloud-intelligence (term): **0** · oya-intelligence-* crates: 6.
- "Directive N" masterplan-authority citations: **81**.
- oya-governance-* crate refs: **204** vs oya-foundry-* : **75** (governance rename ~73% landed at crate level).
