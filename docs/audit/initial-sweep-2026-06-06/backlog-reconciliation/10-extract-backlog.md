# 10 — EXTRACT: Source-side Platform-Readiness Backlog

**Source (verbatim):** `/Users/jasonlee/Developer/source/.omx/backlog/platform-readiness-backlog.md` (690 lines, READ IN FULL).
**Title (L1):** "Oyatie Platform Readiness — Durable Backlog". Compiled during 2026-06-01 deep-interview grounding sweep. Severity legend (L5): P0 (blocks credible delivery) · P1 (major, pre-GA) · P2 (hygiene). Status legend (L7): OPEN · IN-PROGRESS (none closed — fresh backlog).
**This is a planning artifact, not execution-readiness evidence** (L457, L580). `.omx` SoT discipline: planning inputs until promoted to ADR/spec/masterplan + cross-artifact agreement (Founder Refinement #5, L248).

READ-ONLY extract. All citations are file path + line. No source file was edited.

---

## (a) PILLARS A–Q (one-line intent + key decision)

- **A. ARCHITECTURE GAPS** (vs hyperscaler best practice) — L16. Sub-items:
  - **A1 [P0]** No unified module-integration manifest (L18). Decision: author `module-integration-manifest.schema.json` unifying 6 surfaces; fold ADR-0145/0255 + capability-tier-schema. (Architect: P0 but place AFTER D1 resolved, HR/Payroll first consumer — L345.)
  - **A2 [P0/verify]** Workflow–ontology runtime co-location unconfirmed (L31). Decision: make co-location explicit in an ADR. → **A2 RESOLVED** (L167): Oyatie DELIBERATELY chose FEDERATED PEER SUBSTRATES (ontology=Postgres+Citus/Kafka-outbox; workflow=own-Postgres+Valkey/Kafka-subscriber durable; intelligence=stateless dispatch). ADR-0059 "microservices never call each other directly." Tension vs external F2 co-location best-practice.
  - **A3 [P0/verify]** Effective dating not confirmed first-class in ontology (L41). Decision: confirm/instate effective dating as first-class ontology primitive before HR/Payroll. (Architect: VERIFIED ABSENT from 1797-line kernel → P0 FIRST Phase-2 hard gate, L344.)
  - **A4 [P1]** "Power of One" upgrade-safety posture not explicit (L49). Decision: state config-not-customization as invariant + enforce expand/contract migrations in CI. (Architect: Phase-0 platform invariant, L347.)
  - **A5 [P1]** Studio sequenced too early vs industry (L55). Decision: gate studio GA behind ≥2 reference modules + governance + git-metadata.
  - **New A2 gaps** (L175): A2a [P0] missing EntityMutated/Created/Deleted event schema; A2b [P0] missing gRPC .proto for workflow→AgentAuthored AI-step; A2c [P1] no e2e Intelligence→Workflow→Ontology re-ground doc; A2d [P1] tool-call error-handling protocol unspecified; A2e [P1] multi-turn session-store schema+TTL prose-only.

- **B. CONSISTENCY / STATUS-INTEGRITY GAPS** — L62.
  - **B1 [P0]** Status-label drift ("REAL" vs scaffolded) (L64). Decision: single canonical status enum in machine-readable registry + CI lane. (FD-001 SC-01/SC-09 stop-condition.)
  - **B2 [P1]** Superseded-ADR residue (frontend ADR-0393 vs 0372; foundry naming per ADR-0335) (L76). Decision: sweep+retire, rename foundry-* crates, superseded-reference lint.
  - **B3 [P2]** HR/Payroll legacy placement under oya/ not microservices/ (L85). Decision: migrate exclusively (never concurrent).
  - **B/D EXPANDED audit** (agent a3844dd6; 346 ADRs/28 specs/87 dirs; false-green CRITICAL) — L138: P0 items B-P0-1 duplicate ADR-0377; B-P0-2 observability conflict ADR-0042 vs 0383 (Grafana LGTM canonical); B-P0-3 frontend ADR-0393 Leptos still Proposed overturning Accepted 0372; B-P0-4 foundry retirement incomplete despite ADR-0335. P1: B-P1-1 status vocab incoherent; B-P1-2 spec↔code mismatch (6 specs w/o dirs, 50+ dirs w/o specs); B-P1-3 PRD-md vs JSON-spec divergence.

- **C. REPO SHAPE / SPRAWL / ORGANIZATION** — L91 (initially "pending"). Populated by **REPO SPRAWL agent afe14b41** (L204): C-P0-1 .claude/worktrees 198 orphaned/176G (LATER CORRECTED L377: NOT in VCS, local-disk only); C-P0-2 ADR-0131 microservices/ layout 0% implemented; C-P0-3 5 services duplicated oya/+services/; C-P0-4 385 crates underscore-name vs kebab-dir (BNF v4.1 ADR-0056); C-P1 nesting/no-spec/723 workspace members. Root cause: every item = a missing enforced gate (pillar G).

- **D. SCM / CI / DELIVERY** — L96. Initial: Forgejo canonical SCM; Jenkins LTS + ArgoCD (ADR-0349); CI-go-live is the unlock. **CORRECTED (L196):** destination = `oya-ci` bespoke-Rust Forgejo-native K8s-native (kube-rs) Prow reimplementation on Talos (ADR-0513 Accepted+founder-locked); ADR-0514 target spec (D1-D6); Jenkins=transitory bridge (ADR-0380) deleted at Phase-1 cutover; ADR-0511 (Argo Workflows orchestrator) overtaken → mark superseded_by ADR-0513.

- **E. WORKFLOW ENGINE + STUDIO DIRECTION** (hardened DR wurvn7hai) — L111. Confirmed 3-0: E1 engine paradigm = DURABLE EXECUTION (event-sourced replay, Temporal model); E2 shared Rust sdk-core + per-language SDKs via C bindings = proven BUILD pattern; E3 two studio primitives = DETERMINISTIC SKILL node + AGENTIC step (SAP Joule). Tension: most-de-risked engine=workflows-as-CODE vs Oyatie citizen-dev VISUAL studio → resolution: ONE engine, durable-execution underneath, serving BOTH (studio compiles to same durable runtime).

- **F. ONTOLOGY+WORKFLOW+AI SYNERGY — BEST-PRACTICE PATTERN** (hardened DR wmazj0c1j) — L124. Confirmed 3-0 (Palantir Foundry/AIP + MS Dataverse/Copilot): F1 AI grounds via typed scoped tools (anti-hallucination); F2 AI reads AND WRITES same shared object model via same typed actions, CO-LOCATED runtime; F3 deterministic workflows exposed to AI as callable tools; F4 AI can author/modify ontology itself (human-supervised); F5 governance = permission-bounded least-privilege grounding. Verdict: Oyatie has every CONCEPT; decisive unknown = A2; synergy research makes co-location the clear best practice.

- **G. [P0 PILLAR] ENFORCEABLE CI/CD + AUTOMATION FOR SSOT + ANTI-SPRAWL** (user req 2026-06-01) — L153. Requirement: CI/CD that BLOCKS merge, prevents sprawl, guarantees SSOT. Each audit gap → an enforced gate. Sub-sections: **G BINDING SHAPE CONSTRAINT** (L185) buck2-native classes NOT new `oya gate` CLI lanes; **G HYGIENE / ACCOUNTED-GC** (L302) sinker++ reaps worktrees/branches/artifacts/containers/processes; **G FINAL — current state ~21% there** (L272) 8/28 lanes blocking, all native-toolchain via Jenkins bridge. (Architect D3: split G-INTEGRITY (Phase-0, no buck2) vs G-STRUCTURE (buck2-gated), L341.)

- **H. [DECISION] WASM PRELOAD / STREAMING-COMPILE DURING DEAD-TIME** — L287. Decision: Leptos/Rust-WASM shell MUST preload+streaming-compile during auth round-trip + idle. **H RESEARCH RESULT** (agent ac4b0f52, L351): implementation-ready (modulepreload, compileStreaming, Content-Type application/wasm hard-req, Brotli-11, 103 Early Hints, Leptos islands ~24KB vs ~274KB; budgets LCP≤2.5s/INP≤200ms/TTI<3.5s/WASM≤1MB brotli; 4 failure modes). Becomes ADR frontend-wasm-preload-strategy + perf gate.

- **I. [DECISION] RENDERING = SSR + SELECTIVE HYDRATION (Leptos islands)** — L316. Decision: SSR + selective/islands hydration, NOT full-page; refines ADR-0393 to Leptos `islands` mode; SSR streaming; render_envelope BFF contract; hydration/island-count/per-island WASM budgets. (Note: there is a SECOND "§I" at L683, see (e).)

- **J. [RESEARCH+DECISION AREA] UI/UX BEST PRACTICE** — L329. Anchors ADR-0061/0317, FD-001 design system, Leptos islands, Korea-first, WCAG 2.2 AAA. **§J RESEARCH LANDED** (agent aa0a12ba, L417): 3-tier DTCG tokens; one persistent shell chrome; Cmd+K palette; data-dense tables; AI copilot side-panel + autonomy tiers→Cedar; WCAG 2.2 AA floor; INP≤200ms/LCP≤2.5s; i18n Korean (W3C KLREQ); design-system governance + CI gates. Gates: token-SSOT + axe + perf-budget + design-conformance + bundle-size.

- **K. [GOAL + GAP ANALYSIS] HONEST AWS/GOOGLE-CONTENDER BAR** — L366. Goal: spec corpus robust enough that IF all implemented = serious honest Google/AWS contender. **§K CONTENDER-GAP REGISTER** (agent a62f4c38, L430): spec coverage EXTENSIVE; gap = EVIDENCE + ACTUATION + few absent caps. Top blockers: measured SLO, live provisioning, measured DR drill (~6-12mo); absent CDN/Vector-DB/HSM/CSA-STAR/billing-alerts; healthcare measured-HIPAA/BAA, SMART-on-FHIR absent, NCPDP e-Rx under-spec. **§K EXTERNAL-BAR** (DR wng2z0nw8, L440): cert table-stakes (SOC/ISO/PCI/CSA-STAR/FedRAMP/CMMC/IL/FIPS/HITRUST); compliance PER-SERVICE not platform-wide; ONC §170.315(g)(10)/Inferno gate; honest-claim gate.

- **L. [DECISION] MULTI-PLATFORM NATIVE CLIENT STACK** — L470. Web=Leptos; Linux=Tauri+Leptos; Windows=WinUI3; Apple=Swift/SwiftUI; Android=Kotlin/Compose. SHARED RUST CORE + native UI per platform (1Password pattern); UniFFI + C-ABI + WASM bindings; multi-target DTCG tokens. **§L RESULT** (agent a43cc9b1, L558): bindings detail (UniFFI/csbindgen/wasm-bindgen), Automerge sync, Buck2 11 target triples, no-cfg-in-core gate.

- **M. [REQUIREMENT] EXTREME PARALLEL DEVELOPMENT — SAFE REALIZATION + COORDINATION** — L477. First-class deliverable. Safety model: per-lane worktree isolation+GC, affected-only gating, merge-queue projected/speculative state (ADR-0111), one-service-per-lane, exclusive serialization for LSC, RBE+CAS. **§M RESULT** (agent a239c26a, L567): 5 pillars (projected-state queue / affected-targets / hermetic RBE+CAS / per-lane worktree+GC / exclusive lock for LSC) + 8 rules; already ADR'd 0111/0124/0360/0366.

- **N. [REQUIREMENT] VERIFICATION/RESILIENCE BAR** — L486. Four enforced evidence-emitting disciplines: EXACT CAPACITY (USE method), CHAOS (Chaos Mesh/Litmus on Talos), MUTATION (cargo-mutants score gate), STRESS+LOAD (k6/Gatling). **§N RESULT** (agent a12ebf53, L572): capacity-model.json/chaos-evidence.json/mutation tiers/k6 6 profiles → production-readiness-evidence.json → SLO-gated promotion (ADR-0130/0139).

- **O. [REQUIREMENT] COMPREHENSIVE TESTING TAXONOMY — ALL ENFORCED** — L498. Per-service TEST-EVIDENCE BUNDLE consumed by promotion gate (ADR-0139). Test types each mapped to tool→stage→gate: unit, integration, system, e2e+e2e-UX, contract, regression+visual, performance, security (SAST/DAST/fuzz/Cedar), usability, compatibility matrix, UAT, white-box+mutation, black-box, automated (default), exploratory. No GA/honest-claim without bundle green.

- **P. [REQUIREMENT] CROSS-ARTIFACT SSOT AGREEMENT** — L521. Every decision propagated into ADR + SSOT spec + masterplan.json + roadmap, ALL must AGREE, AUTO-ENFORCED (cross-artifact-agreement gate, part of G-integrity). Generated (masterplan.generated.json/board-sync/decisions.json/ADR-INDEX) validated == regenerated-from-source. Connects ADR-0365/0512.

- **Q. [REQUIREMENT] PURE-RUST TOOLING — no shell/python accumulation** — L547. All tooling/gates/CI glue = Rust (oya-dev-cli + buck2-native oya-governance-* + oya-ci kube-rs). ALLOWLIST: (1) single bootstrap shell; (2) buck2 vendored host-only prelude python. Active offenders: scripts/tests/cloud_*_check.py (7 python gate scripts) + scripts/*.sh + infra/ci/buck2-affected-gate.sh → port to Rust. Language-discipline gate BLOCKS new .sh/.py outside allowlist.

---

## (b) DECISION REGISTER items 1–21 (verbatim claim + ADRs to author/amend)

Register intro (L527): "each needs ADR + SSOT + masterplan + roadmap entry, gate-enforced".

1. (L528) "D1 trinity 'co-located consistency domain, federated execution' + EntityMutated schema (A2a) + workflow->AI gRPC contract (A2b)" — **[new ADR]**.
2. (L529) "Effective-dating as first-class ontology-kernel temporal type (A3)" — **[new ADR, verified absent]**.
3. (L530) "Enforcement system: Prow-shaped cloud-ci/oya-ci buck2-native gate taxonomy + generated SSOT registries + hygiene/GC pass (§G); retire `oya` CLI from CI authority" — **[extend ADR-0513 + amend ADR-0363]**.
4. (L531) "Pure-split oya/cloud canonical structure + ADR-0131 path reconcile + verified removal of legacy microservices/" — **[amend ADR-0131 + ADR-0512]**.
5. (L532) "Packs: classify canonical shared/versioned pack roots vs service-shaped sprawl; preserve ADR-0010/0064 roots, move service code to oya/cloud/libs" — **[amend/confirm ADR-0064 + ADR-0010]**.
6. (L533) "Multi-platform native client: shared Rust core + native UI per platform (§L)" — **[new ADR]**.
7. (L534) "Parallel-dev coordination model (§M)" — **[extend ADR-0111 merge-queue + parallel-swarm; ADR-0513 cloud-ci/oya-ci Tide is the merge-queue home, not a deferred option]**.
8. (L535) "Verification/resilience bar (§N) + comprehensive enforced testing taxonomy (§O)" — **[extend ADR-0139 + ADR-0346]**.
9. (L536) "Frontend: ADR-0393 Leptos (source status Accepted; generated-index and migration evidence still required) + SSR/islands (§I) + WASM-preload (§H) + UI/UX design-system + multi-target tokens (§J)" — **[new/amend ADRs]**.
10. (L537) "Honest-claim contender gate + per-service compliance-in-scope + HIPAA framing fix + ONC (g)(10)/Inferno (§K)" — **[new ADR]**.
11. (L538) "Cross-artifact SSOT agreement gate (§P)" — **[extend ADR-0365]**.
12. (L539) "FIXES (consistency): renumber duplicate ADR-0377; mark ADR-0511 superseded_by ADR-0513; complete foundry->intelligence eradication; 3-axis status enum; regenerate ADR-INDEX/decisions.json from source".
13. (L540) "MASTERPLAN + ROADMAP: add the platform-readiness program (Phases 0-3) + sequencing + pillars A-P as masterplan entries; roadmap reflects; all agree".
14. (L554) "Pure-Rust tooling discipline + language-discipline gate (§Q)" — **[new ADR or amend ADR-0513/§G enforcement]**; allowlist spec (bootstrap + buck2-prelude-host-python); port scripts/tests/*.py + *.sh → Rust gate crates.
15. (L556) "False-green closure + D1 reality-check (idea-refine 2026-06-02, founder-decided; detail: .omx/plans/phase0-additions-false-green-d1.md)" — **[amend ADR-0513 enforcement + ADR-0365 cross-artifact; new AC-0.13/AC-0.10b in PRD; new T0.13/T0.10b in test-spec]**. Evidence: 222/298 integration tests not in buck2 targets + d4_body_limit.rs cargo-RED/buck2-GREEN orphan → cargo-green != buck2-green (bidirectional divergence). ADDITIONS ADD-1/2/3.
16. (L585) "CI-ADR-SPRAWL CONSOLIDATION (hyperscaler-gap-analysis): ADR-0349/0359/0361/0408/0511/0513/0514 are 7+ overlapping CI/CD ADRs" → consolidate into ONE canonical CI ADR (destination = ADR-0513; rest superseded/amended-into-it). §P cross-artifact pass; Phase-0/1 ADR-hygiene target.
17. (L586) "DOGFOOD-NEED SEQUENCING": sequence Phase-B/C actuation by what an actual oya/ vertical CONSUMES (K8s/storage/DB/observability/LLM-gateway/IAM), NOT big-4 catalog completeness; contract-FREEZE VM/networking-dataplane/marketplace until a tenant needs them.
18. (L587) "MERGE-CONFLICT ELIMINATION — 'impossible once a PR is opened' (founder 2026-06-02; detail: .omx/plans/merge-conflict-elimination.md)" — **[extend ADR-0111 projected-state queue + ADR-0124 clustering + ADR-0513 tide; ties AC-0.13 generated-not-committed and new AC-0.15/P0.9]**. Single highest lever: stop committing generated files. Phase-0 fold-in P0.9; Phase-1 Tide W1-W6.
19. (L647) "BESPOKE CLOUD TOOLCHAIN SERVICES + PIPELINE ISOLATION (founder 2026-06-02)": SCM/CI/CD are Oyatie Cloud developer-platform services for internal AND external tenants, not internal-only. Services: cloud-scm, cloud-ci (Prow-shaped Rust), cloud-cd (Rust release ledger/reconciler). Forgejo/Jenkins/ArgoCD/Argo-Rollouts = bridge adapters with deletion criteria. Master-plan home `/specs/bespoke-cloud-toolchain-services.json` + `masterplan.json#bespoke_cloud_toolchain_services` under P-TOOLCHAIN. Isolation mandatory: tenant=`oyatie-internal` for dogfood; every tenant pipeline isolated across identities/secrets/runner-pools/workspaces/caches/artifacts/logs/ledgers/deploy-targets/callbacks/audit. Connects ADR-0111/0124/0130/0139/0346/0510/0513.
20. (L661) "AUTOMATION RATCHET (founder 2026-06-02)": anything enforceable/automatable must be enforced/automated. Phase 0 publishes `specs/phase0-automation-matrix.json` classifying every rule as blocking-now / advisory-until-P0.0 / controller-owned-Phase-1 / genuinely-human-judgment. Manual exceptions require owner+target-gate+blocking-fixture+retirement-phase+evidence-path. New `oya` CLI commands FORBIDDEN.
21. (L670) "CLAIM CEILING / HYPERSCALER PRODUCTION-READINESS CLAIM CONTRACT (founder 2026-06-02)": no false/empty/aspirational promises. Claims (`mechanically enforced`,`production-ready`,`hyperscaler-grade`,`secure`,`isolated`,`tenant-facing`,`retired`,`done`,`parity`,`full`,`complete`,`automatic`) regulated by `/specs/hyperscaler-production-readiness-claim-contract.json` + Phase-0 `specs/phase0-claim-evidence-map.json` gate. Tiered language (target_non_claim / spec_ready / mechanically_enforced / production_ready / hyperscaler_grade) + evidence domains. Program-wide "hyperscaler-grade" = target/non-claim until per-service/per-plane evidence meets contract.

---

## (c) LOCKED PROGRAM PARAMETERS + CONSENSUS DECISIONS + STRUCTURAL RULINGS

### LOCKED PROGRAM PARAMETERS (founder consensus, 2026-06-01) — L459
- SCOPE: full 4-phase program.
- RUNWAY: 24+ months / not runway-constrained → foundation-first viable (kernel/OS ambition horizon).
- SUBSTRATE COMPLETION BAR: deployed + measured SLO + DOGFOODED BY A VERTICAL (Phase-2 substrate co-developed with thin payroll-close vertical as conformance harness).
- ROLLBACK TRIGGER: buck2-builds-first-party stall = HARD CHECKPOINT (re-plan enforcement; #1 critical-path risk = the safety valve).
- ENFORCEMENT: **buck2-native only** (no throwaway presubmit); buck2-builds-first-party = accepted gating prerequisite.
- D1: "co-located consistency domain, federated execution" (one ontology write-path; workflow=federated durable-execution calling ontology typed-actions in-txn w/ consistency token; intelligence=federated stateless). Acceptance = read-your-writes conformance test on payroll close.
- TRIPLE SIGN-OFF: founder + architect + critic → proceed to /ralplan (L466).

### CONSENSUS DECISIONS (founder, 2026-06-01) — L399
- SCOPE: FULL 4-PHASE PROGRAM (founder overrode thin-vertical-first consensus pick); mitigation = co-develop thin payroll-close vertical AS substrate conformance harness.
- ENFORCEMENT: BUCK2-NATIVE ONLY (founder overrode 4-presubmit-now pick); buck2-builds-first-party = #1 critical-path risk with checkpoint.
- D1: founder asked for recommended shape.

### D1 RECOMMENDATION: "CO-LOCATED CONSISTENCY DOMAIN, FEDERATED EXECUTION" — L404
- Ontology = SINGLE transactional system-of-record (typed entities + effective-dating + typed ACTIONS + audit); ONE write path = ontology typed-actions.
- Workflow engine = federated durable-execution service (Temporal-shaped, §E) for ORCHESTRATION; domain mutations execute as ontology typed-actions INSIDE ontology transaction + return consistency token; workflow's own DB = orchestration state ONLY.
- Intelligence = fully federated/stateless; grounds on same ontology.
- async Kafka outbox REMOVED from critical consistency path → SAP anti-pattern eliminated by construction.
- ACCEPTANCE: conformance test — HR effective-dated change at T0 observed by payroll-close at T0, NO stale read.

### STRUCTURAL RULING: PURE SPLIT — oya/ + cloud/ ONLY — L214
- CANONICAL TOP-LEVEL = exactly two service trees: `oya/` = PRODUCTS (UI shell, HR/Payroll + business modules, workflow-engine/studio, ontology, intelligence, comms, marketplace; runs as a tenant, k8s-agnostic); `cloud/` = PLATFORM (iam/kms/secrets/compute/k8s/network/storage/data/billing/finops/cell/observability/cloud-intelligence; served-through substrate).
- Everything else holding services = SPRAWL to eliminate (`services/` incl 5 duplicates analytics/app-shell-frontend/ci-webhook-gateway/policy/treasury → consolidate; `platforms/`; stray trees). Remaining non-service top-level: docs/ specs/ libs/ tools/ scripts/ contracts/ .git agent-state(gitignored).
- Enforcement (L227): a service directory may exist ONLY under oya/ or cloud/; no duplicate service across trees; oya vs cloud validated against product-vs-platform boundary + dogfood-purity (no oya→cloud internal dep); worktrees/agent-state never in VCS.
- ADR-0131 flat colocation survives but nests under {oya,cloud}/<ms>/; amend ADR-0131 + ADR-0512; legacy microservices/ removed only after verified-empty.

### STRUCTURAL RULINGS (founder 2026-06-01) — L412
- TOOLCHAIN/SCM/CI/CD ARE PART OF "THE THING": in-tree, versioned with code, first-class (buck2 + oya-ci Prow + Forgejo→bespoke-VCS); dogfood: platform builds+ships itself with itself.
- PACKS + REGIONAL-PACKS: MERGE + colocate, reconcile with one-version rule + ADR-0010. Interim: cross-cutting regulation = SINGLE shared versioned pack modules DECLARE; module-specific overlays live IN module. (Resolved by §STRUCTURAL RESEARCH L449: ADR-0064 + ADR-0010 trichotomy — canonical base / shared pack lib / per-module overlay; cross-pack imports FORBIDDEN.)

### Other founder rulings / principles
- **CONFIRMATIONS (founder, L236):** CI = BESPOKE PROW IN RUST (ADR-0513 operative; ADR-0511 superseded_by 0513; Tide/merge-queue belongs here, not deferred; `oya` CLI retired from CI authority). FRONTEND: ALL frontend = LEPTOS (ADR-0393 canonical; promote Proposed→Accepted; ADR-0372 SolidJS→Superseded; fix active references; do NOT rewrite historical 0372).
- **FOUNDER REFINEMENT (2026-06-02, L243):** #1 `oya verify`/`oya gate` legacy local wrappers only, merge/exit authority = Prow cloud-ci/oya-ci; #2 merge queue not deferred, owned by cloud-ci/oya-ci Tide; #3 pure split confirmed, amend ADR-0131/0512; #4 do not blindly collapse valid packs/regional-packs (ADR-0010/0064), service-shaped pack sprawl → {oya,cloud}/libs; #5 `.omx` SoT discipline — plans/backlog are planning inputs until promoted into ADR/spec/masterplan + cross-artifact agreement.
- **REPO-SURFACE PRINCIPLE (founder, L250):** adopt hyperscaler monorepo patterns (one-version rule, OWNERS/CODEOWNERS, BUILD-visibility/PACKAGE fences, presubmit-vs-postsubmit, LSC tooling, generated-not-hand-maintained registries, trunk-based+merge-queue); bespoke ONLY where named differentiator.
- **CORRECTIONS (founder L196 §D; critic FACTUAL CORRECTIONS L376):** "176G committed to VCS" FALSE (local-disk only, .gitignore:38 excludes; re-scope C-P0-1 to local GC); "REAL:0 / almost entirely scaffolded" FALSE (measured LOC: intelligence 95,964 / workflow-engine 43,447 / cloud-intelligence 12,603 / cloud-iam 6,973 / cloud-kms 4,832); RETIRE word "REAL"; verified true: 723 workspace members, oya=87 dirs, EntityMutated ABSENT, duplicate ADR-0377 real.

---

## (d) EXECUTION_APPROVAL status

- **EXECUTION_APPROVAL = PENDING (not executed).** (L593, #1 approval-state: "base plan architect+critic APPROVE; additions delta-validated 2026-06-02; 3rd-review fixed; EXECUTION_APPROVAL = PENDING (not executed)".)
- L607: "EXECUTION_APPROVAL PENDING — recommend ONE consolidated re-verify before execution (the artifact changed materially this pass)."
- Plan-not-started caveats: register #18 (L587) "the 13-PR drain/minimal-tide prototype evidence below is prior operational evidence used as design input, NOT Phase-0 execution approval and NOT evidence that this plan started."
- Triple sign-off recorded (L466) → "proceed to /ralplan"; but post-approval additions (AC-0.13/AC-0.10b register #15) had architect+critic delta-validation PENDING (L584), then "all closed; plan internally consistent" (L607) with approval still PENDING.
- "READY FOR /ralplan" markers at L580 / L466 / L283; "GROUNDING + CONSENSUS PHASE COMPLETE" L456; "ALL RESEARCH COMPLETE" L579.

---

## (e) §I AGENT-EXECUTION-CONTROLLER — CAPTURE-ONLY / decision-pending (2026-06-05) — L683

NOTE: this is a SECOND section labeled "## I." appended at end (the first "## I." at L316 = SSR+selective-hydration). This one = "AGENT-EXECUTION CONTROLLER — pod-runner + work-item + evidence-bundle".

- **Provenance:** extracted from now-superseded harness distillation `/Users/jasonlee/Developer/oya-code/docs/source-distillation-cloud-intelligence.md`; full idea doc `docs/ideas/agent-execution-controller.md`. Everything ELSE in that distillation = thinner/partly-stale restatement of artifacts already in source (cloud-intelligence PRD+IP-001, oya/intelligence/*, ADR-0255/0263/0296/0392/0408) — NOT worth porting. This is the one concept source does not already own.
- **The idea:** agent EXECUTION as ephemeral policy-gated unit of work — run agent CLI (Claude Code/Codex/Gemini) as a K8s Job, isolated+audited, handed back as sealed evidence. Three contracts: `work-item.v1`+`controller-receipt.v1` lifecycle (queued→claimable→claimed→pod_scheduled→running→{completed|failed|cancelled}, Cedar `WorkItemClaim` grant per claim); `pod-schedule-plan.v1` (K8s Job API, Talos machine API, mTLS gRPC, non-root, trace context); `evidence-bundle.v1` (run identity, Cedar verdicts, status, **reproduction command**, transcript refs — never a status string). Thin-runner discipline; provider-native CLI lifecycle preserved (stream-json/hooks/subagents/thought-signatures), not flattened to chat-completions.
- **Boundary (load-bearing):** agent EXECUTION, distinct from cloud/cloud-intelligence (inference egress/token gateway) and oya/intelligence (substrate/supervisor/guardrails). If pursued = NEW flat single-concern service under oya/ or cloud/ per pure split — NOT folded into cloud-intelligence (ADR-0131/0132).
- **DECISION GATE (do before any code):** ADR-0116 (retire external agent-coordination tooling) + ADR-0363 (retire agentic VCS foundry) deliberately killed the adjacent layer — this "missed area" may be missing on purpose. Founder must rule: revive-as-narrower-net-new vs decline. If declined → record decline in idea doc, let oya-code harness repo lapse. If accepted → audit that repo's crates/ + examples/work-items/ for salvageable code before re-authoring, promote to ADR per .omx SoT discipline.
- **Why it matters:** without an owned execution contract, parallel-agent work has no canonical work-item/evidence shape — teams invent per-lane status files (anti-pattern). Connects §M, §N/§O.
- **Evidence:** distillation cross-checked vs cloud-intelligence README/PRD/IP-001 + oya/intelligence/ tree + ADR-0255; grep found NO existing delivery-fabric/swarm-controller/pod-runner home in docs/decisions, specs/, cloud/, oya/.

---

## Cross-cutting notes captured (not pillars but load-bearing)

- **ARCHITECT VERDICT** (agent a87e4fce, code-verified, L337): D1 unsound-as-proposed (federated = enforcement-by-trust); REFRAME by consistency-domain. If federation retained, price all 4 mechanisms (outbox+EntityMutated proto / consistency-token read-your-writes barrier / saga+idempotency / proven latency bound). D2 add substrate conformance harness mandatory. D3 split G-integrity (Phase-0) vs G-structure (buck2). D4 serialize structural migrations exclusive. D5 status enum = 3 orthogonal axes (decision/maturity/constraint). A3 P0 first Phase-2 gate. 8 ralplan PRECONDITIONS imposed.
- **CRITIC VERDICT** (a2df58531, L381): REVISE — do not proceed to ralplan as-is; C1/C2/C3 block; over-rotating on single unverified Workday co-location = bigger near-term risk; invert sequencing to thin vertical first; minimum enforcement = 4 absent gates as cheap presubmit. (Founder later OVERRODE both inversions: full 4-phase + buck2-native only, L400-401.)
- **RECONCILED ARCHITECT+CRITIC POSITION** (L390): 5 points (mechanical inventory replaces STANDING; D1 decided by thin-vertical experiment; 4 gates presubmit now; invert sequencing; need founder runway/completion-bar/scope).
- **RECONCILIATION 2026-06-02** (L582): verified prior-session changes clean (G004 HEAD 9819a57bb feat/oya-ci-tide, PR#64→Forgejo dev, 247 worktrees); SSOT-drift closed (AC-0.13/0.10b reconciled into candidate PRD/test-spec).
- **THIRD-REVIEWER PASS 2026-06-02** (L591): 8 blockers all VALID + FIXED; EXECUTION_APPROVAL PENDING. KNOWN DRIFT: docs/AGENTS.md (Jenkins-sole ADR-0349/0359/0361) vs ADR-0511/0513 (oya-ci) → register #16 must update operating contract.
- **FOLLOW-UP / ENFORCEMENT REBASELINE** (L609, L632): root inventory closed-world — oya(87)/cloud(25)/services(5)/platforms(0)/packs(9)/regional-packs(5)/libs(168); legacy microservices absent/removal-candidate. Live branch-protection requires cargo-fmt/check/clippy/nextest/deny + oya-verify; no full oya-ci-gate context; reviewer-approval = process requirement until oya-pr-review producer (501) live.
- **HYPERSCALER-GAP central finding** (L588): massive contract substrate, ~zero actuation; #1 gap = RECONCILER/ACTUATION runtime (the `Unimplemented` seam), Phase-1/2 keystone; cloud-intelligence = only real-I/O plane.
