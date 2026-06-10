# FULL SCOPE OF WORK — Oyatie consolidation → enforced-canon → platform program

> **Purpose:** the EXHAUSTIVE "whole mountain" — every track, phase, and item from the CURRENT state to the end-state, so a fresh-context session sees the entire program (not just the next step). READ-ONLY synthesis; no source file mutated.
> **Date:** 2026-06-06. **Mode:** deduplicated, sequenced work-breakdown.
> **SSOT inputs (read in full):** `synthesis/decision-record-oyatie-canon.md` · `UNIFIED-EXECUTION-PLAN.md` · `source/.omx/backlog/platform-readiness-backlog.md` (690 lines) · `source/.omc/plans/monorepo-consolidation-migration.md` · `justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md` · `docs-sweep/00-REST-OF-DOCS-REGISTER.md` (CC-1..CC-13).
> **Status tags:** `{DONE | IN-PROGRESS | NEXT | DEFERRED}`. 🚪 = door:one-way + founder sign-off point. 🔑 = credential/tooling dependency.

---

## 0. CURRENT STATE (done this session)

- **STEP-0 clean base committed** (e77f16eb2, signed) — precondition to ALL mutation. **DONE.**
- **ADR-0515 ratified** (869e48ca4, signed) — the ONE unified CI/CD canon (D-CICD): consolidates 0349/0359/0361/0408/0511/0513/0514 into one ADR; oya-ci/oya-cd bespoke-Rust Prow+Tekton+Argo patterns, build-first-cutover-later. **DONE.**
- Both commits on source branch `feat/oya-ci-tide`, pushed to `github-mirror`, **NO dev PR** (firewall-first per D-SEQUENCE). **DONE.**
- **Enforcement is a verified façade:** 0 gates block a merge today — the sole required context `oya-ci-required` has NO live producer (both branch-protection files self-disclaim it). This IS the mechanism of the drift. (FE-1, audit §4.)
- **Sequence = FIREWALL-FIRST** (D-SEQUENCE): make enforcement REAL first, then fix the canon THROUGH it.
- **Pre-lanes status (consolidation):** 0.4 ✓ · 0.5 ✓ (G2/G4 ratified) · 0.6 DEFERRED-to-consolidation-time · 0.7 ✓. Pre-Wave-0 gates COMPLETE (§A.6 provenance GREEN, A.0-1 census 831/43, A.0-2 design-set FROZEN 🚪).

**The critical-path spine (one line):** STEP-0 ✓ → ADR-0515 ✓ → **FE-1 producer go-live** 🚪🔑 → accounting-registry (Gate-2) → 4 keystone gates → A-lane amendments (each gate-verified) → doc-reorg + accounting closure → 6-repo migration (M-lanes) → platform build-out (4-phase program) → deferred ratchet campaigns.

---

## TRACK 0 — PHASE-0 FIREWALL (make enforcement REAL before touching canon)

> *Nothing downstream is "enforced" until FE-1 is real. Independent of the build-graph migration; ships in the G-INTEGRITY track (specs+filesystem, no buck2-build-graph dep).*

| # | Item | Status | Deps / blocking edges | 🚪 / 🔑 |
|---|---|---|---|---|
| T0.1 | ADR-0515 unified CI/CD canon authored+ratified (the producer spec) | **DONE** | STEP-0 | 🚪 founder-signed |
| T0.2 | **FE-1 APEX — stand up + PROVE the `oya-ci-required` producer** (first real piece of oya-ci); post on a real SHA; apply the ruleset; snapshot live required-checks | **NEXT** | T0.1; producer = `oya-ci-controller-kernel:471` (wired only to Helm/ArgoCD scaffold today) | 🚪 FE-1 producer go-live = authority for ALL CI-enforcement claims · 🔑 GitHub admin (branch-protection ruleset apply), push creds |
| T0.3 | **Gate-2 TOTAL-ACCOUNTING** (`cloud-ci-total-accounting`) — owns `accounting-registry.generated.json`; `git ls-files × OWNERS × ADR-justification × masterplan-reachability`; BLOCKS unaccounted/unowned/unjustified/unreachable/no-ttl/registry-drift; 7 RED fixtures; report-then-git-mv-archive (never rm), 2nd-verifier-gated | **NEXT** | T0.2 | 🚪 self-test must flag live 780 `oya-foundry-*` + 57 unwired `oya-governance-*` |
| T0.4 | **Gate-1 CROSS-ARTIFACT-AGREEMENT** (`cloud-ci-cross-artifact-agreement`; amends ADR-0365) — decision→{ADR,spec,masterplan,roadmap} crosswalk; BLOCKS orphan/unpropagated/status-disagreement/generated-drift/dual-collision/supersession-half-edge; 7 RED fixtures freezing LIVE exhibits (axes_count 6≠7, dup-0377, 0511↔0513 half-edge) | **NEXT** | T0.3 (predicate over registry) | the gate whose absence let the two consensus bodies drift |
| T0.5 | **Gate-3 STALENESS REAPER** (`cloud-ci-staleness-reaper`; §G sinker++, linux Task-#14 >48h class) — Gate-2 registry + generated `ttl-policy` per resource-class + git-log last-touch; report-then-archive; protected classes never reaped; 7 RED fixtures | **NEXT** | T0.3 | covers the ai-slop `_partial`/`_verify` scratch-doc archive class |
| T0.6 | **Gate-4 AUTOMATION-RATCHET** (`cloud-ci-automation-ratchet`; register #20) — seed `phase0-automation-matrix.json` + 4 RED/GREEN fixtures ALREADY ON DISK; BLOCKS enforceable-marked-human-judgment/advisory-claiming-enforced/oya-cli-authority/incomplete-exception/no-retirement/ratchet-regression; **polices Gates 1-3** (no gate self-certifies) | **NEXT** | T0.3, T0.4, T0.5 (Gate-4 over all) | reuse 4 on-disk fixtures + 2 net-new; flags 57 governance crates + diataxis/prd-axis as advisory-claiming-enforced |
| T0.7 | **FE-2/FE-3 single SoT for required contexts** — generate all four config artifacts from one SoT; assert equality via `protection-context-match --live-required-contexts`; wired RED proof (goes RED today, forcing FE-1) | **NEXT** | T0.2 | three disagreeing lists today (branch-protection vs groovy-16 vs reported-17 vs Jenkinsfile) |
| T0.8 | **Wire orphaned/dark real checkers into required roster** — `predictable-naming-kernel` (I1b, catches 74 non-enum dirs + 137 catalog roles), real bounded-contexts checker replacing LEAN-A2 cedar copy-paste WRONG-CHECK (I4c), foundation-bypass lane (FE-8 expired bypass), `brand-residue` (FE-4); promote tenant-boundary oya→cloud report-only→fail-closed (I4d) | **NEXT** | T0.2 | |
| T0.9 | Resolve `byp_adr_0349` foundation-bypass record (CI-cluster Jenkins debt) + renew/close FE-8 expired `byp_adr_0346` mirror bypass | **NEXT** | T0.8 | |

**Track-0 dependency spine:** FE-1 producer → Gate-2 (registry) → Gates 1/3/4 (predicates over it). One registry, four predicates, no four parallel scanners.

---

## TRACK 1 — PHASE-1 AMENDMENTS (gate-verified, single-owner A-lanes; amend-in-place + additive, NO renumber per D13-AMENDED)

> *Each amendment is verified by a Track-0 gate, not prose. Mutate source ONLY after founder-go + WIP-commit-first. Never two sweeps over the ~346-id space.*

### A-CI (merges S2 + CI-cluster net + register #16) — **partly DONE via ADR-0515**
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| Consolidate 0349/0359/0361/0408/0511/0513/0514 → ONE ADR-0515 | **DONE** | T0.1 | 🚪 D-CICD ruled |
| Finish 6→1: 0349/0361 DROP (never-ratified Jenkins debt), 0359 tombstone, **0511→Superseded-by-0513/0515** (+reciprocal supersedes edge), 0408/0514 AMEND-in-place (adopted Buck2 substrate, separate bounded-context ADR `depends_on:[0408]`) | **NEXT** | A-INTEGRITY gates; Gate-1 acyclic-supersession proof | |
| Strike Tekton from "four faces" (zero Accepted support) | **NEXT** | above | |
| `byp_adr_0349` bypass-record resolution; **Jenkins-gate-path DELETE at Phase-1 cutover** (ControllerDispatcher cutover, ADR-0514 D4); 89 Jenkinsfiles + jenkins-adapter stay OPERATIVE as explicitly-UNRATIFIED de-facto bridge until oya-ci built+proven | **NEXT / DEFERRED** (cutover) | FE-1 live + oya-ci proven | 🚪 build-first-cutover-later |
| **Relocate the bespoke-ci product-spec** (`docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md`) into its canonical home | **NEXT** | doc-reorg topology | |

### A-FOUNDRY (merges S1 + FE-4 + CC-1) — sense-routed per-file rename (NEVER blind swap)
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| **Fix ADR-0363's false "eradicated" claim** (`:35`); drop the word; enumerate residue (11 canonical dirs incl. live `docs/teams/axis-foundry` + ~4110 file-level mentions) | **NEXT** | Gate-1/brand-residue | 🚪 |
| **4-WAY sense-routed rename** (CURRENT reality, NOT cloud-intelligence re-home): platform→`oya-intelligence-*` (~274 files) · fitness→`oya-governance-*` (~135 files) · agentic-VCS→RETIRED (git+Forgejo+oya-ci) · foundry-fitness→`oya-governance` | **NEXT** | template-first; coordinates with each M-lane codename pass | |
| **HARD carve-outs (must NOT sweep):** Palantir-Foundry (43 files, census-of-record) · Marlboro-Forge/forgery (journeys) · the foundry→governance retirement RECORD itself | **NEXT** | rename pass | |
| Cedar `oyatie.foundry.*` principal namespace amendment (ADR-0247); fix name-lag bug (`oya-foundry-supervisor` binary builds `oya-intelligence-supervisor-app` package) | **NEXT** | rename pass | |
| Wire `foundry` into `brand-residue` token list; rename `oya-foundry-brand-residue-kernel`→`oya-governance-*`; checklists must police `foundry` not only `Oyatie` | **NEXT** | T0.8 | |
| Half-migration state: governance ~73% landed (204 vs 75 refs), intelligence barely started (6 refs) — complete in one pass; fitness lane rejects mixed prefixes per seam | **NEXT** | | |

### A-INTEGRITY (merges S3+S4+S5+S6 + FE-7 + D11) — all become RED fixtures inside Gate-1
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| **Renumber dup-0377** (forge-board variant → free block >0514; kafka-to-pulsar keeps 0377); rebuild index | **NEXT** | Gate-1 | 🚪 batch |
| **Phantom-0150 Cedar real-id** — assign Cedar-engine a real id in free block (assigned at Wave-1 L1.0 MAP); repoint ALL `-cedar-policy-engine.md` citations (≥10 ADRs: 0243/0246/0476/0457/0429/0443/0428) + 0083 mis-cites; leave 0150-cursor-pagination untouched | **NEXT** | Gate-1 | |
| **KCMVP/KISA restore** from `*-shippable-tier` corruption — **6 files STILL PENDING** (incl. GLOSSARY + ADR-0043 + ADR-0002) | **NEXT** | | |
| ALL dangling supersedes/amends edges: 0476→0421 (fix mis-number), 0057→0055-v3, 0069→0088, 0377→0397/0436, Cedar-as-0083 | **NEXT** | | |
| Remove ADR-0335 `:771` stray status line ("completed-locally" not enum); fix 0006 self-referential "Ontology→Ontology" tautology; fix 0045 "Citus=AGPL" factual error | **NEXT** | | |
| **3-axis status enum** — decision-state (Proposed/Accepted/Superseded) + maturity-state (contracts-only/domain-landed/runnable/deployed-with-SLO) + constraint-state (advisory/binding); generated + CI-validated | **NEXT** | Gate-1 | |
| **Regenerate ADR-INDEX/decisions.json** from source (via `tools/oya-adr-index-regenerator-app`, NEVER hand-edit) — fixes stale 0372-Accepted drift | **NEXT** | enum + renumbers | |
| **GENERATE `axes_count`** from axis enum (S6/FE-7; 6→7, do NOT hand-fix one number) | **NEXT** | Gate-1 | |

### A-STRUCT (merges S7 + FE-9 + D-PURESPLIT) — pure-split eradication
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| **Eradicate everything except `oya/` + `cloud/`** — `services/` (6 dirs incl 5 duplicates analytics/app-shell-frontend/ci-webhook-gateway/policy/treasury), `platforms/` (1), flat-crates split-brain (`crates/` vs `oya/.../crates/`) | **NEXT** | service-tree-purity lane | 🚪 D-PURESPLIT door:one-way — no "complete" claim until green |
| **Amend ADR-0131 (`microservices/` ×15) + ADR-0512 (×6)** → `{oya,cloud}/<service>/`; state two-tree-only rule | **NEXT** | Gate-1 | |
| Add BLOCKING `service-tree-purity` lane (RED fixture); migrate/archive the 6+1; collapse crates/ split-brain | **NEXT** | T0.3 | |

### A-TASTE (S8 + D-DOCTRINE Linus-taste)
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| ADR-0368 over-abstraction ("maximum safe concurrency at all times" idle=defect) → reframe "capacity-bounded, M0-gated parallelism" (D8) | **NEXT** | | 🚪 founder taste call |
| ADR-0109 "Both patterns canonical" self-flagged carve-out → collapse to ONE parameterized lifecycle-kernel shape | **NEXT** | | |

### A-IDENTITY (CC-8 + D5 + D6 + D16)
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| **Cedar→contract+PARC ~24 genuine sites** (CC-2; PARC=0 hits corpus-wide today; do NOT mass-rewrite 146 FP `workflow-engine`/`policy-engine-logs`) | **NEXT** | A-INTEGRITY (real Cedar id) | 🚪 D6 |
| identity = oya-identity owned endpoint + Zitadel-bridge framing (CC-8); 0476/0187 supersession edges (0187→superseded-as-endpoint) | **NEXT** | | 🚪 D5 |
| Crypto libs (aws-lc-rs/webauthn-rs/opensk, 0506/0507/0508) reused behind ports | **NEXT** | | |
| Autonomy ceiling 0007↔0022 reconcile — runtime-enforced hard Cedar gate, owned by `governance` (D16) | **NEXT** | | 🚪 D16 |

### A-VOCAB / D12 (vocabulary namespacing) — namespacing, not supersession
| Item | Status | Deps |
|---|---|---|
| `tier` → `autonomy_tier` (T1–T4) / `eu_ai_act_risk_tier` (0–4) / `dr_tier` / `storage_tier` / `tenant_class` (CC-11; retired tenant-tier 0329) | **NEXT** | A-INTEGRITY |
| tenant-tier→tenant-class (138 files; 129 = ONE persona template line → fix GENERATOR; ~9 real plan-axis sites per-site; carve `*_tier` namespaced + 2 retirement FPs + decide `cell-tier`) | **NEXT** | template fix |
| ADR-0163 "environment tiers" → "environment stages" | **NEXT** | |

### Proposed-ledger / D14 (~132/145 Proposed → resolve; ZERO unaccounted)
| Item | Status | Deps | 🚪 |
|---|---|---|---|
| RATIFY ≈122 + 1 RENUMBER-then-ratify + 1 KEEP-as-Proposed-by-design | **NEXT** | Gate-1 | 🚪 D14 batch door:one-way |
| **DROP 3:** 0325 (prices retired primitive) · 0316 (superseded by 0329) · 0349 (Jenkins-half) | **NEXT** | | |
| **AMEND-MANDATORY 0352** | **NEXT** | | |
| 0347 Proposed→Accept (foundry rename); ratify D-LAYER grounding (0242/0245/0247/0244/0009) | **NEXT** | | |

### CC-1..CC-13 doc fixes (WF2 register; A6) — branch-locality: 0421/0457/0429/0443/0428/0488 re-resolve at merge
| CC | Item | Status |
|---|---|---|
| CC-1 | foundry per-file rename (→ A-FOUNDRY) | **NEXT** |
| CC-2 | Cedar→contract+PARC (→ A-IDENTITY) | **NEXT** |
| CC-3 | masterplan-authority inversion (DOC-CATALOG/standards-INDEX omit ADRs as apex; reclassify MASTERPLAN→GENERATED-REFERENCE; resolve 81 "Directive N" cites to originating ADRs) | **NEXT** |
| CC-4 | Jenkins/ArgoCD-as-canonical→oya-ci bridges; delete "does NOT build replacements for ArgoCD/OpenTofu" sentence; flag ADR-0160/0171/0240/0349 | **NEXT** |
| CC-5 | Forgejo→GitHub-now (4 runbooks + 2 webhook-replay specs → `X-GitHub-Delivery` primary; mirror-at-most); flag ADR-0377 | **NEXT** |
| CC-6 | data-tier bridges (Postgres/Citus/Milvus/ClickHouse transitional) + Kafka→Pulsar (D-EVENT); fix `_TEMPLATE.md`/`prd-template.md` so new docs inherit Valkey/Pulsar | **NEXT** |
| CC-7 | isolation framekernel-host endpoint + assume-breach-microVM-default; rename 2 `secure-by-default` phrasings | **NEXT** |
| CC-8 | identity oya-identity+Zitadel-bridge (→ A-IDENTITY) | **NEXT** |
| CC-9 | eventing 3-way contradiction (Pulsar vs Redpanda+NATS vs Kafka) → resolve to Pulsar canon; reconcile ADR-0005 | **NEXT** |
| CC-10 | KR-first framing-clarity only (DOWNGRADED — consistent w/ D-KR; delete KR-as-launch from PRD.md North Star, keep global-canonical+KR-first-pack) | **NEXT** |
| CC-11 | tenant-tier→tenant-class (→ A-VOCAB) | **NEXT** |
| CC-12 | M0-M3/MVP → gate-defined waves (62 real M0-M3 + 35 MVP; heavy FP carve-out: MacBook M3, Shamir, ServiceNow milestones) | **NEXT** |
| CC-13 | linux pilot North-Star re-scope (roadmap.md/source-parity-context.md "port the kernel" → demote as Phase-0 port-provenance under live hyperscaler mission) | **NEXT** |

### AI-SLOP / deslop targets (docs-sweep §3) — fold into Phase-2 doc-reorg
- `onboarding/intern-month-one.md` (138 foundry hits, templated day-blocks — highest-severity slop); `erp-coverage/PRD.md` (~150× repeated sentence, 2514 lines); DESIGN.md payback table fabricated precision; PRD.md success-metric circular rationales; `unified-ecosystem-thesis` padding + `line_floor:2500`; 1MB/2MB day-in-the-life + coverage-matrix docs; **136 runbook stubs** (66% empty — incl. safety-critical-and-empty `foundry-robotics-safe-stop`/`industrial-ot-write-emergency-stop`/`healthcare-break-glass`); fix `cloud-native-stack.md` "+40%"→"+31%"; strip `testing-strategy.md` process-narration; `release/`↔`advanced-cicd/` duplicate subtrees; over-cap docs (anti-patterns 2914L etc.); `products/README.md` 16 dead PRD links. **Status: NEXT (Phase-2).**

---

## TRACK 2 — PHASE-2 DOC-REORG + TOTAL ACCOUNTING (after gates + amendments land)

| # | Item | Status | Deps | 🚪 |
|---|---|---|---|---|
| T2.1 | **Diátaxis 44→6 homes** on `source/docs/` (2888 files): tutorials/ how-to/ reference/(generated) explanation/ _project/(generated) decisions/(immutable, only special case); foundry/ sense-route before filing; raw/wiki/site/harness/ci/agents → triage/archive | **NEXT** | A-FOUNDRY (sense-route) | 🚪 D-DOCORG |
| T2.2 | **Unified per-doc record** `doc-records.generated.json` — {id,path,doc_class,axis,tier,owner_team,reachability,generated,ttl,validation_check,dependent_docs}; one shape for all docs incl. ADRs (no special cases); supersedes split CatalogRow | **NEXT** | T2.1 | |
| T2.3 | Doc-reorg GATES advisory→BLOCKING (each RED-fixture-proven): promote `diataxis-doc-class`; author `prd-axis-coverage` lane+kernel; replace broken `doc-catalog` (reads non-existent `docs/CATALOG.md`) with `oya-governance-doc-record`; fix the path bug | **NEXT** | T0.3, T2.2 | |
| T2.4 | DUP merges: DUP-1 `docs/specs/`(110)⟷`tasks/`(110); DUP-2 `templates/`⟷`docs/templates/` (-v2 split-brain); DUP-3 pack 4-homes; secondary `docs/products`⟷`specs/products`, `contracts/*.proto`⟷`specs/proto` | **NEXT** | T2.1 | |
| T2.5 | **OWNERS files tree-wide** (closes O-1: ZERO OWNERS today, ~4% doc frontmatter, near-0% code — largest D-DOCTRINE gap) | **NEXT** | T0.3 | |
| T2.6 | **Reachability closure** (closes O-2): declare Cargo.toml/code-tree manifest a masterplan companion registry (`oya/`·`cloud/`·`libs/` = 0 masterplan hits today; reachable only via Cargo members) | **NEXT** | T0.3 | |
| T2.7 | **Generated accounting-registry over the WHOLE tree** — all 27 source/ top-level entries; archive orphan husks (`crates/`/`services/`/`test-results/`/`memory/`); add TTL to bulk markdown corpora (runbooks 172/personas 133/user-journeys 190); burn down 289-row legacy-data_class allowance w/ TTL | **NEXT** | T0.3 | |
| T2.8 | **Stale-file >48h audit** (campaign task #14) — the ai-slop pileup, after amendments land | **DEFERRED** (after amendments) | T0.5 (reaper) | |
| T2.9 | Glossary additions (PARC/framekernel-host/assume-breach-microVM/oya-ci/oya-identity/Pulsar/tenant-class as canonical; mark retired tokens in top tables; partition overloaded "tier") | **NEXT** | | |

---

## TRACK 3 — NET-NEW ADR AUTHORING (platform-readiness program; register 1-21 / pillars E-Q; additive >0514 free block)

> *The backlog 4-phase platform-readiness program merged into ONE canon (D-SCOPE-UNIFY). Each needs ADR + SSOT spec + masterplan + roadmap, cross-artifact-gate-enforced (§P).*

### Trinity / consistency keystone
| # | Item (register) | Status | Deps | 🚪 |
|---|---|---|---|---|
| T3.1 | **D1 meta-ADR** — "co-located consistency domain, federated execution" (ontology=single transactional SoR write-path; workflow=federated durable-execution calling ontology typed-actions in-txn + consistency token; intelligence=federated stateless); Kafka outbox OUT of critical consistency path; acceptance = passing payroll-close read-your-writes conformance test; resolve D1/D01/D15 id-naming clash | **NEXT** | Gate-1; A1 manifest after | 🚪 D-D1-TOPOLOGY (defining architectural call, irreversible-in-practice) |
| T3.2 | **Effective-dating as first-class ontology-kernel temporal type** (A3; VERIFIED ABSENT from 1797-line kernel — real net-new kernel build); first Phase-2 hard gate | **NEXT** | T3.1 | |
| T3.3 | **EntityMutated/Created/Deleted proto** (A2a; verified ABSENT) feeding workflow saga compensation | **NEXT** | T3.1 | |
| T3.4 | **workflow→AI-step gRPC contract** (A2b; verified ABSENT) for AgentAuthored invocation | **NEXT** | T3.1 | |
| T3.5 | **module-integration-manifest** (A1) `module-integration-manifest.schema.json` unifying 6 surfaces (entities/actions/events/projections/templates/UX/tools/Cedar/compliance); HR/Payroll first consumer; place AFTER D1 resolved | **NEXT** | T3.1 | |
| T3.6 | consistency-token/version-vector read-your-writes barrier (the mechanism synthesis missed — outbox+saga alone insufficient); saga compensation + idempotency keys; proven latency bound | **NEXT** | T3.1 | |

### Enforcement / governance pillars
| # | Item | Status | Deps |
|---|---|---|---|
| T3.7 | Enforcement pillar **G** ADR (Prow-shaped buck2-native gate taxonomy + generated SSOT registries + hygiene/GC; split G-INTEGRITY (Phase-0, no buck2) vs G-STRUCTURE (buck2-gated)) | **IN-PROGRESS** (Track-0 builds it) | T0.* |
| T3.8 | **§P cross-artifact-agreement** ADR (extend ADR-0365) | **IN-PROGRESS** (Gate-1) | T0.4 |
| T3.9 | **#20 automation-ratchet** ADR | **IN-PROGRESS** (Gate-4) | T0.6 |
| T3.10 | **#21 claim-ceiling** ADR + `hyperscaler-production-readiness-claim-contract.json` + `phase0-claim-evidence-map.json` (tiered language: target_non_claim/spec_ready/mechanically_enforced/production_ready/hyperscaler_grade); resolve FE-10 naming collision (rename ADR-0054 depth-ratchet → `agent-claim-depth-ceiling`) | **NEXT** | T0.6 |
| T3.11 | **§Q pure-Rust-tooling** ADR + language-discipline gate (BLOCK new .sh/.py outside allowlist: single bootstrap shell + buck2 host-prelude python); port 7 `scripts/tests/cloud_*_check.py` + `buck2-affected-gate.sh` → Rust gate crates (deletion-tagged) | **NEXT** | T0.6 |

### Frontend
| # | Item | Status | Deps |
|---|---|---|---|
| T3.12 | Frontend ADR-0393 Leptos canonical (source Accepted; regenerate indexes; migrate `oya/app-shell-frontend` SolidJS→Leptos = real impl; flip ADR-0513/0515 deck ref; superseded-reference lint) | **NEXT / IN-PROGRESS** (source status set) | A-INTEGRITY index-regen |
| T3.13 | **§I SSR + selective hydration** (Leptos islands mode ~24KB vs 274KB; SSR streaming; render_envelope BFF) ADR | **NEXT** | T3.12 |
| T3.14 | **§H WASM-preload/streaming-compile** ADR (modulepreload + preload as=fetch wasm; compileStreaming in auth window; IndexedDB cache; brotli/103-Early-Hints; budgets LCP≤2.5s/INP≤200ms/TTI<3.5s/WASM≤1MB) + frontend perf-budget gate (4 failure modes) | **NEXT** | T3.12 |
| T3.15 | **§J UI/UX design-system** ADR (3-tier DTCG tokens generated-SSOT; unified shell/role=space; Cmd+K palette; data-dense tables; copilot side-panel UX + EU AI Act disclosure; WCAG 2.2 AA/AAA + axe gate; i18n/Korean KLREQ) | **NEXT** | T3.12 |
| T3.16 | **§L multi-platform native clients** ADR (shared Rust core + native UI; UniFFI Swift/Kotlin + C-ABI WinUI3 + WASM web/Tauri; multi-target tokens; no-cfg-in-core gate; Automerge/SQLite sync; Buck2 11-triple matrix) | **NEXT** | T3.12, T3.15 |

### Verification / testing / honest-claim
| # | Item | Status | Deps |
|---|---|---|---|
| T3.17 | **§N verification/resilience bar** ADR (capacity USE-method + chaos Chaos-Mesh + cargo-mutants mutation-score + k6 load; per-service `production-readiness-evidence.json`; SLO-gated promotion ADR-0130/0139) | **NEXT** | T0.6 |
| T3.18 | **§O testing taxonomy** ADR (unit/integration/system/e2e/contract/regression/perf/security/usability/compat/UAT/whitebox/blackbox/automated/exploratory all enforced; per-service test-evidence bundle; presubmit-affected vs postsubmit-full) | **NEXT** | T3.17 |
| T3.19 | **§K honest-claim contender** ADR + per-service compliance-in-scope field + HIPAA framing fix (BAA+NIST/HITRUST not "HIPAA cert") + ONC §170.315(g)(10)/Inferno gate; contender-gap register (CDN/Vector-DB/HSM/CSA-STAR/SMART-on-FHIR/NCPDP absent) | **NEXT** | T3.10, T3.5 |

### Toolchain / parallel-dev
| # | Item | Status | Deps |
|---|---|---|---|
| T3.20 | **#19 bespoke cloud toolchain services** ADR (cloud-scm/cloud-ci/cloud-cd as tenant-facing dogfood products; tenant=`oyatie-internal`; mandatory per-tenant pipeline isolation across identities/secrets/runners/caches/artifacts/logs/ledgers); `specs/bespoke-cloud-toolchain-services.json` under P-TOOLCHAIN | **NEXT** | T0.1 |
| T3.21 | **§M parallel-dev coordination** ADR (extend 0111/0124/0360/0366: projected-state merge queue, affected-targets presubmit, hermetic RBE+CAS, per-lane worktree GC w/ UUID branches, exclusive-lock for LSC); **#18 merge-conflict elimination** (stop committing generated files = highest lever) | **NEXT** | T3.20 |
| T3.22 | **#15 false-green closure / D1 reality-check** — native generator emits buck2 rust_test targets (0 uncovered .rs, AC-0.13 buck2-green==cargo-green); cargo authoritative-incumbent deletion-tagged; executable RED in-memory read-your-writes test (222/298 integration tests escape CI today) | **NEXT** | T3.21, T3.1 |
| T3.23 | **#17 dogfood-need sequencing** + **#5 packs classification** (ADR-0064/0010 shared-pack-lib + per-module overlay, NOT blanket merge; corpus.lock anti-drift) | **NEXT** | |
| T3.24 | **#13 MASTERPLAN + ROADMAP** — add the platform-readiness program (Phases 0-3) + pillars A-Q as masterplan entries; roadmap reflects; all agree (gate-enforced) | **NEXT** | Gate-1 |

### Bominal-restored ADRs (D-KR / D-SEQ / D-SAFETY / D-RECOVER)
| # | Item | Status | Deps | 🚪 |
|---|---|---|---|---|
| T3.25 | **Unified safety-gate invariant ADR** (D-SAFETY, governance-owned): HITL clinical-Dx · no-closed-loop-actuation OT · biometric-default-off + 5-stage escalation · no-autonomous-lethal defense; hooked into D16 runtime Cedar gate | **NEXT** | T3.10 | 🚪 safety/liability |
| T3.26 | **KR EmploymentClassification enum** as KR-localization-pack data model (D-KR: 정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원); KR-first-to-market sequencing posture on global-canonical core; restore .Trash KR HR/payroll 8-pack | **NEXT** | T3.23 | 🚪 D-KR |
| T3.27 | **Infra-sovereignty ratchet** ADR (D-SEQ: ordered list IaC→gateway/KMS→mail/cache/stream→DB/storage + per-substrate M0 evidence-gate, NO calendar dates); uniform vertical milestone shape (research→M0→operating-contract→EPIC) | **NEXT** | | 🚪 D-SEQ anchors roadmap |
| T3.28 | **Data-engine-endpoint ADR** (D-INTEL/D4: own-the-whole-tier endpoint; vendored bridges + linux engine ADRs 0515+ point up) | **NEXT** | | 🚪 D4 |
| T3.29 | **TWO meta-ADRs**: domain-cohesion gate (D15: closed 16-domain enum + `domain` field on ADR-0364 template + gate on ADR-0365 lifecycle) + masterplan-generated-wiring (D1) | **NEXT** | Gate-1 | 🚪 |
| T3.30 | D-RECOVER folds: healthcare released-view record-boundary→ADR-0332 (🚪); medical→emr mapping; Connect per-context DEK matrix; M3 first-paid-customer target + First-Proof-Slice "first buildable+sellable slice"; CCTV 5-stage + biometric-off; person-pillar HARD exclusion-zone (통신비밀보호법); PG-rail + 전자금융업 license-ladder; **DROP Bominal Train**; keep Bominal Law + Bominal Finance as named-deferred | **NEXT** | T3.25, T3.26 | mixed |
| T3.31 | **D-AEC-DECLINE** — record the agent-execution-controller decline in `docs/ideas/agent-execution-controller.md`; let `oya-code` harness lapse; ADR-0116/0363 stand unamended | **NEXT** | | 🚪 declined |
| T3.32 | Amend cloud-intelligence service-local docs to cite ADR-0389/0390 (exist centrally) | **NEXT** | | |
| T3.33 | Net-new ADRs for the 4 keystone gates (>0514, under amended ADR-0365) + accounting-ledger schema + service-tree-purity invariant + generated layer-enum SSOT (I1a: 3 disagreeing copies today) | **IN-PROGRESS** (Track-0) | T0.* | |

---

## TRACK 4 — THE 6-REPO MIGRATION (M-lanes L1-L11; the 982-commit dev-merge, gate-verified POST-firewall)

> *Strict-serial single-driver ralph loop; STD-first/no_std-last; one squash-PR per landing zone; BUILD-TO-BOTH-GATES; `github-mirror` remote (NEVER origin/Forgejo). ~10 lanes (L8 dropped).*

### Pre-lanes (gating, once) + GATE-BEFORE-START
| # | Item | Status | 🚪 / 🔑 |
|---|---|---|---|
| 0.4 | authority-snapshot + signing pre-provision | **DONE** | 🔑 signing key (DONE), push creds |
| 0.5 | truthing + source/merge-surface manifests; G2 tools/ exception + G4 (drop L8, cloud-k8s docs-only, codex new sibling, 95 k8s/44 ctrd split, codename ratify) | **DONE** | 🚪 G2/G4 ratified |
| 0.6 | no_std build-capability + whole-graph-inertness spike (excluded-state incl. exclude-key edit, full 12-entry kernel exclude) | **DEFERRED to consolidation-time** | 🚪 founder sign-off (needs real merged root) |
| 0.7 | governance-file bootstrap | **DONE** | |
| GBS | **GATE-BEFORE-START** — kernel workflow DONE + independent kernel re-verify (check-tcb/diff-oracle/both-build/assert-talos) | **NEXT** | 🚪 founder go |
| G0 | authority-flip HALT (the DOMINANT risk: `github-lane-unlocker-required`→`oya-ci-required` in flight, ADR-0513); loop step-0 re-diffs every iteration | **ongoing guard** | 🚪 HALT on flip |
| G1 | GitHub push credentials for `github-mirror` | **NEXT** | 🔑 GitHub admin |

### M-lanes (STD-first → no_std last) — each carries codename→`oya-*` rename + the ~12 §6 conformance gates
| Lane | Item | Status | Source path |
|---|---|---|---|
| L1 | **office** → `oya/office` CREATE (rename `oyaoffice-*`→`oya-office-*`) | **NEXT** | `~/Developer/office/crates` (13 crates) |
| L2 | **oyago** → `oya/transpiler-go-to-rust` CREATE | **NEXT** | `~/Developer/oyago/crates` (×177 brand) |
| L3 | **oyapy** → `oya/transpiler-python-to-rust` CREATE | **NEXT** | `~/Developer/oyapy/crates` (×181 brand) |
| L4 | **claude SDK** → `cloud/.../oya-cloud-intelligence-anthropic-claude-adapter` CREATE (relicense MIT→Apache-2.0) | **NEXT** | `~/Developer/claude` (`claude-agent-sdk`) |
| L5 | **codex SDK** → MERGE/NEW sibling into `cloud/.../oya-cloud-intelligence-codex-adapter` | **NEXT** | `~/Developer/codex/sdk/rust` |
| L6 | **k8s** → MERGE 95 crates into `oya-cloud-k8s-*` under managed-k8s-control-plane-host + 4 `managed-k8s-*` | **NEXT** | `linux/stack/kubernetes/crates` (95 of 139) |
| L7 | **containerd** → `cloud/cloud-container-runtime` CREATE (44 `ctrd_*` of 139) | **NEXT** | same dir, 44 subset |
| L8 | **cloud-data/db-engine** | **DROPPED** (no source — owned DB engine is a future D4 build) | n/a |
| L9 | **node-os** → `cloud/cloud-node-os` CREATE (rename `talos-*`×45→`oya-cloud-node-os-*`, STD 1.96.0) | **NEXT** | `linux/stack/operating-system` (51 crates) |
| L10 | **docs** → `docs/{context,research}` + 13 pilot ADRs renumbered into LIVE free block; retire pilot scaffold | **NEXT** | linux pilot docs |
| L11 | **framekernel** (no_std, LAST) → `cloud/cloud-kernel` CREATE, workspace-EXCLUDED, nightly-2026-02-28, build-std | **NEXT** | `linux/stack/kernel` |

### §6 CONFORMANCE GATES (D-CONFORM; additive to WIP loop step-7 + per-lane verify; every M+A lane must pass)
| # | Gate | Status |
|---|---|---|
| 1 | BNF layer-suffix ENUM (closed enum; reject -core/-runtime/-port/-api-contracts/-gateway/-web/snake_case) | **NEXT** |
| 2 | Hexagonal layer-import-matrix (LEAN-A2; the biggest reshape — oyago/oyapy/claude/codex monoliths SPLIT) | **NEXT** |
| 3 | Microservice slot2 registration (flat catalog) | **NEXT** |
| 4 | Manifest hygiene (resolver-2, version.workspace, publish=false, license=Apache-2.0, [lints] workspace, [lib] doctest=false, rust-version pin) | **NEXT** |
| 5 | Dependency-rationale no-orphan | **NEXT** |
| 6 | Vendor A/B/C registry (fix misplaced office deny.toml) | **NEXT** |
| 7 | Per-service colocation + buildability-bar ADR-shape | **NEXT** |
| 8 | rebrand-arrow/retired-terms scan (M0-M3, tier-system, "Foundry" live) | **NEXT** |
| 9 | `data_class` on every new kernel-struct field (in WIP loop — keep) | **NEXT** |

---

## TRACK 5 — PRODUCT / PLATFORM BUILD-OUT (locked 4-phase program; 24+ month runway, not runway-constrained)

> *D-SCOPE-UNIFY: full 4-phase program. Substrate completion bar = deployed + measured SLO + DOGFOODED BY A VERTICAL. Builds parallelize (D8/D-LANES), oya-ci-prioritized crown-jewel, M0-gated + capacity-bounded.*

| # | Item | Status | Deps |
|---|---|---|---|
| T5.1 | **oya-ci crown-jewel** full build (D8 priority day-0; unblocks the dogfood loop) — substrate first → grow the brain on its data; phased ratchet w/ measured promotion gates (CAS-hit>60% etc. set at impl) | **IN-PROGRESS** (FE-1 = first piece) | Track-0 |
| T5.2 | **Ontology/Workflow/Intelligence trinity** build (co-located consistency domain, federated execution; effective-dating kernel; typed actions; durable-execution workflow engine — Temporal-shaped Rust sdk-core + per-lang SDKs) | **NEXT** | T3.1-T3.6 |
| T5.3 | **Thin payroll-close conformance vertical** (MANDATORY substrate harness — substrate-without-consumer = false-green); read-your-writes staleness-bound test; ≥1 paid KR group ~3000 employees (M3 first-paid) | **NEXT** | T5.2, T3.26 |
| T5.4 | **~25 cloud platform services** (cloud-compute/k8s/data/iam/kms/secrets/network/storage/billing/finops/cell/observability/cloud-intelligence...) — dogfood products consumed by oya/ + external tenants; LIVE PROVISIONING actuation (the `Unimplemented` reconciler seam = #1 gap), measured SLO + DR-drill evidence | **NEXT** | T5.1, dogfood-need sequencing #17 |
| T5.5 | **Verticals** — B2B (0321/0315), healthcare (0332), consumer (0220), defense (NET-NEW: classified/air-gapped/ITAR/IL-levels/cross-domain), power-grid/critical-infra OT (NET-NEW: NERC-CIP/IEC-62443/SCADA), marketplace (0249), "+ many more" from bominal far-future; each via research→M0→operating-contract→EPIC | **NEXT** | T3.27, T5.3 |
| T5.6 | **AI two-layer** — cloud-intelligence (Bedrock-analog framework+runtime, cloud-owned) + oya-intelligence (per-tenant servicing); 23-dim GA-parity + 4 beat-bars (Automated-Reasoning guardrails · per-session microVM isolation + long-running agents framekernel-native · durable execution · eval-as-proof); dual consumption (metered API + subscription/seat); no parity claim w/o measured evidence | **NEXT** | T5.4 |
| T5.7 | **Honest-claim closure** — Big-4 SOC2-T2+HIPAA dual-audit; per-service compliance evidence; Inferno (g)(10); measured-SLO/live-provisioning/DR-drills (6-12mo cloud, 3-6mo enterprise/healthcare) | **NEXT** | T5.4, T3.19 |
| T5.8 | **Multi-platform client stack** build (Web/Linux-Tauri/WinUI3/Apple-SwiftUI/Android-Compose over shared Rust core) | **NEXT** | T3.16 |
| T5.9 | **Verification/resilience evidence production** (capacity/chaos/mutation/load harnesses Phase-1; evidence Phase-2/3) | **NEXT** | T3.17, T3.18 |

---

## TRACK 6 — DEFERRED RATCHET CAMPAIGNS (later, gated; after migration settles)

| # | Item | Status | Why deferred |
|---|---|---|---|
| T6.1 | **Full ADR-0000+ re-foundation** (renumber/consolidate whole corpus into clean generative series; consolidates-provenance) | **DEFERRED** (D13-AMENDED) | WIP migration depends on STABLE ADR ids; full renumber invalidates every live citation during the authority-flip |
| T6.2 | **AI-engine re-home** `oya/intelligence` (96k LOC, 128 crates) → `cloud/cloud-intelligence` (Bedrock-engine relocation; overrides ADR-0389 port-not-relocation lean) | **DEFERRED** (D-INTEL) | large migration, ratchet-sequenced; Wave-0 rename targets CURRENT home not destination (would drift docs ahead of code) |
| T6.3 | **Governance build-out** spec-shell (0 .rs, 6 .cedar today) → live Rust crates (governance defines gates, oya-ci runs them — authority/runner split) | **DEFERRED** (D-INTEL) | promote from spec-stage; multi-quarter |
| T6.4 | **AI-substrate maturity program** — 23-dim GA-parity + 4 beat-bars, ADR-0123-gated; benchmark at `docs/research/ai-substrate-maturity-2026-06-06/` | **DEFERRED** (D-INTEL) | multi-quarter |
| T6.5 | **Owed-depth authoring per vertical** at each vertical's M0 gate (NERC/MFDS/IEC-62443/SGP4-orbital/capital-markets-license-ladder/ISA-95/SLAM-ROS2-VDA5050/낙찰하한율); **vertical-coverage map #18** (which verticals, compliance regime, in-source vs net-new) | **DEFERRED** (D-DEPTH, two-way) | authored when vertical sequenced, never silently lost; lives in #18 map + masterplan |
| T6.6 | **Bominal restorations** (D-RECOVER cleanup beyond Track-3 folds) — lineage Theme-4, first-customer Theme-6, MED/LOW Theme-8 residuals; reconciliation workflow `wguxmaorw` bominal(past)-vs-oyatie(present) diff → founder interview agenda | **DEFERRED** (mostly two-way) | interview-driven, not blind recovery |
| T6.7 | Stale-file >48h audit (= T2.8) | **DEFERRED** | after amendments land |

---

## DEPENDENCY / CRITICAL-PATH SPINE (the irreversible backbone)

```
STEP-0 ✓ ──► ADR-0515 ✓ ──► 🚪🔑 FE-1 producer go-live ──► Gate-2 (accounting-registry)
                                                              ├─► Gate-1 (cross-artifact)
                                                              ├─► Gate-3 (staleness reaper)
                                                              └─► Gate-4 (automation-ratchet, polices 1-3)
   ──► FE-2/3 single-SoT + wire dark checkers (T0.7/T0.8)
   ──► PHASE-1 A-lanes (each GATE-VERIFIED, never two sweeps over 346 ids):
         A-CI · A-FOUNDRY · A-INTEGRITY · A-STRUCT · A-TASTE · A-IDENTITY · A-VOCAB · Proposed-ledger · CC-1..13
   ──► PHASE-2 doc-reorg (44→6) + OWNERS/reachability closure + tree-wide accounting + net-new ADRs
   ──► 🚪 GATE-BEFORE-START (kernel re-verify) ──► 6-repo MIGRATION L1..L11 (gate-verified, §6 conformance)
   ──► PLATFORM BUILD-OUT (oya-ci → trinity → payroll vertical → ~25 cloud services → verticals → AI two-layer)
   ──► DEFERRED ratchets (ADR-0000+ refound · AI re-home · governance build-out · maturity · #18 · bominal)

Every "complete/eradicated/enforced/parity/done" claim stays #21-claim-ceiling-BLOCKED until its gate is GREEN.
```

**🚪 founder sign-off / door:one-way points:** FE-1 producer go-live · ADR-0515 (done) · A-CI 0513/0515 canon · A-STRUCT D-PURESPLIT completeness · A-TASTE framing · every source mutation (founder-go + WIP-commit-first) · G2 tools-exception (done) · G4 drop-L8 et al (done) · A.0-2 design-set frozen (done) · D1 topology · D-SAFETY · D-KR · D-SEQ · D4 data-engine · D14 Proposed batch · GATE-BEFORE-START · authority-flip G0 HALT.

**🔑 credential / tooling dependencies:** GitHub admin (branch-protection ruleset apply for FE-1; push creds for github-mirror = G1) · SSH commit-signing key (done, G3) · the `oya-adr-index-regenerator-app` + `oya-doc-staleness-inventory-app` (gate-load-bearing tools/ standing exception) · the `oya-gen` / generator tooling (note: per audit corrections the REAL generator is `scripts/ci/generate-first-party-buck.rs`, members-based — the earlier `oya gen propagate --check` / `cargo metadata --no-deps:129` citations were FABRICATED/stale; #20 ratchet specifically retires `oya gen`-bound `verified_by`) · nightly-2026-02-28 kernel toolchain (L11) · the 4 on-disk Gate-4 seed fixtures + `phase0-automation-matrix.json`.

---

## END-STATE / NORTH STAR (one paragraph)

Oyatie is **one monorepo, one canon, one masterplan generated from ADRs (the SSOT), maintained BY ENFORCEMENT** — a pure two-tree split (`oya/` products · `cloud/` platform) where every file, doc, and folder is accounted-for (owner + justification + masterplan-reachability + TTL) by a single generated accounting registry policed by four born-blocking keystone gates, so drift, staleness, contradiction, and false-green are structurally impossible and no "complete/hyperscaler-grade/parity" claim survives without measured RED/GREEN evidence. On that firewalled base, the **oya-ci/cloud-ci/cloud-cd bespoke-Rust toolchain** dogfoods itself building and shipping the **ontology/workflow/intelligence trinity** (co-located consistency domain, federated execution, effective-dated, zero SAP integration-tax — proven by a passing payroll-close read-your-writes test) plus the **~25-service cloud platform** (sold as IaaS/PaaS) and the **maximal vertical scope** (B2B, healthcare, consumer, defense, power-grid, and more — sequenced not cut, each through research→M0→operating-contract→EPIC with a governance-owned cross-vertical safety-gate), all served by a **two-layer AI substrate** (cloud-owned Bedrock-analog framework + per-tenant servicing) at GA-parity-plus-four-owned-differentiators — an honest, evidence-backed, Linus-taste, hyperscaler-grade Rust cloud that PROVES the cloud by running its own products on it as tenant `oyatie-internal`, with the clean ADR-0000+ re-foundation and the AI-engine re-home completing the ratchet once the migration settles.

---

## COVERAGE HONESTY (what I could NOT fully enumerate — no silent caps)

- **decision-record-oyatie-canon.md** — read IN FULL (203 lines, all ~32 D-rulings enumerated).
- **UNIFIED-EXECUTION-PLAN.md** — read IN FULL (116 lines).
- **00-JUSTIFY-ACCOUNT-ROBUSTNESS.md** — read IN FULL (201 lines; the 31864-token whole-file Read failed on cap, but the file is 201 lines and was fully returned via the line-ranged read).
- **00-REST-OF-DOCS-REGISTER.md** — read IN FULL (232 lines; all CC-1..CC-13 + mechanical-sweep table + slop list + reachability + coverage honesty).
- **platform-readiness-backlog.md** — read IN FULL across three passes (lines 1-306, 307-506, 507-690 = all 690 lines; pillars A-Q, DECISION REGISTER 1-21, locked params, §I agent-execution-controller capture).
- **monorepo-consolidation-migration.md** — read IN FULL (197 lines; revision ledger, 11 lanes, ralph loop, G0-G4 gates).
- **NOT read (referenced but out of scope of the six SSOT docs):** the per-lane `10-*.md` justify-account artifacts (synthesized in the 00- master, not re-derived); `01-ADR-DISPOSITION-TABLE.md` (~294 of 345 ids carry UNCHANGED verdicts there — body-level charter pass not done on them); the bespoke-CI `40-PRODUCT-SPEC.md`; AMENDMENT-PLAN.md (superseded by UNIFIED-EXECUTION-PLAN); the `.omx/plans/` sub-plans (phase0-additions-false-green-d1.md, merge-conflict-elimination.md, structural-lock spec) — their decisions ARE folded into register #15/#18 above but the raw plan files were not opened.
- **Sampled-not-censused per the source audits (carried forward as estimates):** foundry residue honest figure 11 dirs + ~4110 mentions (raw 2180/3771/4714 worktree-inflated); 831/43 foundry/Palantir file split (sampled journeys/personas tail); journeys/personas genuine-contradiction residue ("a handful", MEDIUM confidence); 57-59 unwired oya-governance-* crates (lower bound 22); enforcement reality inferred from source+fixtures+self-disclaimers, NOT a live farm/GitHub-API run.
- **Proposed-count discrepancy noted:** decision-record says "~122 ratify / 132 Proposed"; justify-audit corpus status says "99 Proposed (of 348/345)"; prompt says "~145 Proposed" — the three figures disagree (snapshot-timing + branch-locality); the D14 batch resolves whatever the live count is at execution, zero unaccounted.
```
