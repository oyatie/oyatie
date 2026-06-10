# PERSONAL ADR VERIFICATION — each Accepted ADR vs the conversation-authority

> Founder mandate (D-AUTHORITY-CONVERSATION): verify MYSELF (no rubber-stamping), each Accepted ADR vs the decision-record rulings; nothing missed; verify nothing dropped is worth preserving. Read personally from `source/docs/decisions/`. `[NEW]` = a contradiction the prior agent audit under-weighted/missed. Tooling caveat: term-scan "argo" matches "C**argo**" (false positives — discounted).

## Batch 1 — ADR-0001…0056 (foundation), verified
| ADR | status | verdict | conversation-authority check (cited) |
|---|---|---|---|
| 0001 cohesion-thesis | accepted | **AMEND** | KEEP the thesis (one product / flat catalog / 6 substrates — aligns pure-split + the consolidated substrates). FIX: "**single agent runtime**" substrate + "Foundry" (:65,:80-87) → **cloud-intelligence** domain (D-LLM-DOMAIN); foundry = forbidden vocab. |
| 0006 ontology | accepted | **AMEND** | Ontology kernel sound. MISSED (confirmed): no **effective-dating / consistency-token** — the D-D1-TOPOLOGY keystone gap; add as first-class kernel type. |
| 0008 data-use-boundary | accepted | **KEEP** (minor) | Exemplary privacy contract; no canon conflict. minor: "Foundry" (:147) → intelligence. |
| 0011 cross-µsvc-contract-registry | accepted | **AMEND** | The registry IS the cross-artifact-agreement substrate (aligns the keystone gate). FIX: its CI lane → **CaC pipeline** (D-GOVERNANCE-CENTRAL, not CLI); foundry (:10,:49,:60,:125) → intelligence. |
| 0017 brand-naming/repo-layout | accepted | **AMEND** | GitHub slug `jason931225/oyatie` = correct but frame **GitHub-interim → Sapling-bespoke** (D2/D-FORGE), not permanent; foundry (:39,:54) → intelligence. (argo hits = "Cargo", false.) |
| 0018 glossary canon | accepted | **AMEND** | The forbidden-token CI lane = the brand-residue gate (now carries forgejo/foundry). FIX: **delete the glossary "Foundry" row** (:45); gate → CaC pipeline. |
| 0028 cloud-µsvc (3-phase compute) | accepted | **AMEND** | [hyperscaler] **SPOF confirmed:** Phase-1 primary OCI-Seoul + secondary OCI-Chuncheon = **same provider** failover (+ AWS fail-open); cross-provider DR needed. cloud-as-flat-peer vs D-LAYER. |
| 0029 connect 12-app suite | accepted | **AMEND** | [hyperscaler] 12 Workspace apps as ONE milestone, no M0 gate → gate per-app. Foundry (:25-51) → intelligence. Reconcile to the now-separate oya/ app services (mail/calendar/docs/…). |
| 0030 search (5-stage) | accepted | **AMEND** | [hyperscaler] full search engine as one µservice, no M0 gate. "Search↔**Foundry** bridge" → Search↔**intelligence** bridge (:26-134). |
| 0031 ads+analytics | accepted | **AMEND** | [hyperscaler] **SPOF confirmed:** "**singleton** tenant-ads-gate" = fleet-wide single point; shard/per-cell it. |
| 0034 per-µsvc data-class overrides | accepted | **KEEP** (minor) | Exemplary runtime hard-deny (tenant Cedar can't raise — aligns D6/D16). minor: reconcile KR-minor-age vs 0008. |
| 0055 object-graph→ontology | accepted | **KEEP** (minor) | Clean rename. minor: the LLM ref (:59) read as cloud-intelligence-consistent (D-LLM-DOMAIN). |
| 0056 rust-clean-arch BNF v4.1 | Accepted | **AMEND [NEW]** | **The BNF grammar enshrines `oya "-" "check" "-" rule-name` — i.e. `oya-check-*` CLI check crates as canonical.** This CONTRADICTS D-CLOUD-NATIVE (not-CLI) + D-GOVERNANCE-CENTRAL (checks = CaC pipeline gates, not `oya-check-*` CLI). The prior audit's 0056-AMEND (flat-crate grammar) **missed this**. FIX: the check production → CaC gate-crate convention run by the pipeline. foundry (:100) → intelligence. |

**Batch-1 dropped-worth-preserving:** none dropped in this batch (all KEEP/AMEND; substance preserved).
## Batch 6+7 — ADR-0184…0393, verified (rest of the <400 high-risk band; bucketed by class)
- **CLI-governance + Jenkins + oya-vcs cluster → cloud-native CaC/CaaS pipeline (founder "we don't keep cli"; D-CICD-AUTHORITY GitHub-Actions-live):** the ENTIRE CI/governance/agentic set — 0222, 0223, 0234, 0235, 0237, 0238, 0240, 0241, 0258, 0331, 0332, 0350, 0351, 0362, 0363, 0364, 0365, 0366, 0367, 0368, 0369, 0370, 0371, 0374, 0375, 0376, 0378, 0380, 0388, 0390, 0391 — **AMEND** (gate/agentic LOGIC preserved; CLI+Jenkins+oya-vcs+forgejo delivery eradicated → pipeline). The agentic-pipeline north-stars (0366/0367/0368/0369) keep their intent, lose the CLI/Jenkins substrate.
- **`[NEW]` 0223 oya-git drop-in → SUPERSEDE/heavy-AMEND:** the `oya git` wrapper is retired (0363) + cloud-native-not-CLI (audit said KEEP-forge-agnostic).
- **0187 canonical-oidc-zitadel-PRIMARY → SUPERSEDE:** Zitadel = bridge, oya-identity owned (D5/0476); downstream 0188/0189/0190/0191 reframe Zitadel→bridge.
- **0202 gitops-iac ArgoCD-as-canonical-engine → AMEND (CRITICAL own-vs-adopt, C-15):** ArgoCD = CD bridge reuse-behind-port (D10/D-CICD), not the owned engine.
- **0383 obs-stack-reconciliation holds BOTH Mimir AND VictoriaMetrics → AMEND:** the standing obs contradiction is internal; the code wires VictoriaMetrics — resolve to one (D-DOCTRINE robust-not-false).
- **0373 llm-gateway-production-design → confirm cloud-intelligence domain (D-LLM-DOMAIN);** 0389/0390 cloud-intelligence = correct home (KEEP+fold engine re-home).
- **Stack ratchets:** Kafka→Pulsar (0192/0193/0195/0350); Flagger→Argo-Rollouts (already 0160/0165/0180/0181); ArgoCD=CD-bridge-OK where not framed-as-owned-engine.
- **foundry/forgejo vocab:** pervasive across the band → eradicate (forbidden vocab). LLM (0200/0329/0330/0331/0335/0373/0389/0390)→cloud-intelligence.
- **KEEP/clean (verified):** 0185, 0197, 0204, 0206, 0207, 0208, 0210, 0379, 0393 (+ 0194/0198/0199/0205/0209 minor: foundry/obs-name only).
- **dropped-worth-preserving:** 0333/0334 (cell/shorts retired) + the connect-dissolution (0237) — content folded into successors (cell-rebalancer/social/the separate connect apps); nothing lost.

**<400 BAND COMPLETE (0001-0393 all verified personally).** Net: the low band's drift is dominated by ONE class — the **CLI/Jenkins/forgejo/oya-vcs governance+CI substrate** — plus foundry vocab, the stack ratchets (Zitadel/Flagger/Kafka/ArgoCD-engine), LLM-home, and the foundational grammar/enum (0056/0092/0105). The gate/agentic/governance LOGIC is sound and PRESERVED; only the retired delivery substrate is replaced. Confirms founder heuristics (lower #=higher staleness; <400=very-high-risk).

## Batch 8 — ADR ≥400 band, verified personally (the lower-risk band; founder: lower#=higher staleness, so this band is lighter — but I read+cited each hit)
Scope: 0408 + 0476–0515 (the only ≥400 ADRs on disk; the 0405–0415 range is empty except 0408 — see integrity note). Evidence gathered read-only; **every verdict below is mine, citation-checked against `source/docs/decisions/`.**

- **0408 buck2-driven-ci-cd** (Proposed) → **AMEND + resolve-Proposed.** The Buck2-as-engine decision survives (ADR-0392). DEAD: ":59 complementary to ADR-0359 (Jenkins replaces GitHub Actions)" + ":47/:69 `oya verify` CI mirror / thin governance-verify orchestrator." Jenkins = eradicate (D-CICD-AUTHORITY GitHub-Actions-live); `oya verify` as CI authority directly contradicts **0515:90** ("no `oya` CLI is merge authority; oya verify/oya gate are local migration evidence only") → re-home to the pipeline.
- **0476 oya-identity** (Accepted) → **AMEND.** Core (bespoke human identity, owned, Zitadel-bridge) = KEEP, aligns D5. FIX: `:70 "oya-vcs (ADR-0409): GitHub human browser auth"` — **forbidden vocab + PHANTOM ref** (see integrity note) → `cloud-scm`, drop the (ADR-0409) edge.
- **0481 oya-flags** (Accepted) → **AMEND.** Core (bespoke feature-flag server superseding flagd) = KEEP. FIX: `:25 verified_by "oya gate validate flag-schema"` + `:33 verified_by "oya gate validate honest-claims"` = live CLI-governance → re-home `verified_by` to the CaC/CaaS pipeline (oya-gate SUPERSEDED, logic preserved); `:23 "oya-vcs (ADR-0409) GitOps repo"` + `:24 Buck2 gate "oya-gate-validate-flag-schema"` → cloud-scm + pipeline-gate naming; drop phantom 0409; `related:[…0509]` is a stale edge (0509 is superseded — see below).
- **0482 bespoke-substrate-roadmap** (Accepted) → **AMEND.** Roadmap intent KEEP. FIX: `:54 roadmap row "oya-vcs (ADR-0409) | GitHub (ADR-0363) | parallel-run…"` → restate as `cloud-scm` destination, drop phantom 0409, eradicate forbidden vocab in the row.
- **0509 hyperscaler-service-decomposition** (status says Accepted — **actually SUPERSEDED**) → **INTEGRITY FIX (A-INTEGRITY).** `[NEW]` ADR-0512 frontmatter declares `supersedes:[ADR-0357, ADR-0509]` but 0509 has `status: Accepted` + `superseded_by: []` (asymmetric/dangling edge). Set 0509 `status: Superseded`, `superseded_by: [ADR-0512]`. This auto-resolves the apparent contradiction at `0509:115 "crates/oya-storage-port acceptable"` vs **0512:53 flat top-level `crates/` forbidden** — 0509 is simply superseded, not live.
- **0510 scm-bespoke-destination/cutover-trigger** (Proposed) → **AMEND + resolve-Proposed.** The decision (GitHub = transitory host → bespoke SCM destination behind a numeric cutover trigger) is **doctrine-aligned and worth keeping** — this is the `cloud-scm` destination ADR. FIX: eradicate the historical vocab strings it quotes — `:33/:117 "oya vcs / oya git", "git + Jenkins + GitHub"` — restate as cloud-scm + GitHub-Actions/oya-ci, no jenkins/oya-vcs/oya-git tokens.
- **0511 ci-orchestration** (Superseded by 0515) → **VOCAB-ERADICATE body.** Forgejo/Jenkins/`oya gate`/`oya verify` saturate the body (:19–:116). The supersession is correct; per the forbidden-vocab override, scrub forgejo/jenkins/oya-vcs even from this superseded body.
- **0512 canonical-monorepo-pattern** (Accepted) → **KEEP.** Correctly eradicates a dead `foundry` crate (:41,:57) and forbids flat `crates/` (:53) — doctrine-aligned (pure-split). Action: ensure the 0509 back-edge is added (above).
- **0513 oya-ci bespoke-Rust-Prow** (Superseded by 0515) + **0514 build-ci-cd target-arch** (Superseded by 0515) → **VOCAB-ERADICATE bodies.** Forgejo/Jenkins/`oya-dev-cli` throughout; supersession correct; scrub forbidden vocab from the superseded bodies.
- **0515 oya-ci-cd unified** (Accepted) → **KEEP.** Correct framing: `:90 "no oya CLI is merge authority (oya verify/oya gate are local migration evidence only)"`; the `oya-cli/oya-gate/oya-verify` strings at `:130` are **RED test-case names that assert those are NOT authority** (a ban-fixture, like the brand-residue gate naming the forbidden words) — legitimate, KEEP.

**`[NEW]` INTEGRITY DEFECTS I found personally (not in the agent term-scan — they live in the citation graph):**
1. **PHANTOM ADR-0409.** `oya-vcs` is cited "(ADR-0409)" by 3 *Accepted* ADRs (0476, 0481×2, 0482×2) but **no ADR-0409 file exists** (0405–0415 empty except 0408). Dangling ref to a now-forbidden concept → feeds A-INTEGRITY (no-dangling-ref invariant) + the oya-vcs eradication (→ cloud-scm / ADR-0510 destination).
2. **ADR-0509 asymmetric supersession** (above) — 0512→0509 edge present one-way only.

## Dropped / ARCHIVE items — "is anything dropped worth preserving?" (founder mandate), verified
- **0057 cutover-rename-v4** (Accepted): SOURCE of the `oya-check-*` LEAN crates (`:50 oya-shared-*-check-cli`) + dropped `oya-governance-fitness-*` + "fitness"→"check". The check-family it births is now **SUPERSEDE** (CaC pipeline gates, not `oya-check-*` CLI); rename mechanics are historical/done. **Nothing lost** — the check LOGIC re-homes to the pipeline.
- **0097 / 0101 / 0102 / 0138** (all Accepted, all `foundry-*`): per foundry-eradication these re-route (3-way). Preservable LOGIC, not just mechanics: **0102** canonical `SettingsTemplate` value-type + per-provider renderers + `sref://` secret-resolution + atomic tempfile-rename → **preserve, re-home under cloud-intelligence**; **0138** strangler/atomic-consolidation pattern → preserve as a reusable migration pattern. **0097** (layer-token-last rename) + **0101** (direct-hyper mountpoint, explicitly temporary) = historical mechanics, nothing net-new to keep.
- **"declined AEC" ADR:** **does not exist** — full-corpus scan finds AEC only as a competitor description (0321:19134, "Egnyte for Construction"). Nothing was dropped here. (My earlier note of a "declined AEC" item was mistaken — verified false.)
- **"Bominal Train" / dropped "Train" product:** **does not exist** in any ADR — "train" appears only as release-train/merge-train/ML-train. The ralplan "drop Bominal Train" was a lost-context backlog item, not an ADR; **nothing in the ADR corpus to preserve.**

**≥400 BAND + DROPPED-ITEMS COMPLETE.** Net: the high band is mostly clean/doctrine-aligned (0512/0515 KEEP; 0510 keep-the-decision-scrub-the-vocab). Its drift = the same two classes as the low band (oya-cli/jenkins/forgejo/oya-vcs substrate + foundry vocab) plus **two citation-graph integrity defects** (phantom 0409, asymmetric 0509↔0512). Dropped-items audit: only **0102 SettingsTemplate** + **0138 strangler-pattern** carry logic worth preserving (re-homed); AEC + Train confirmed non-existent. **PERSONAL ADR VERIFICATION COMPLETE across the full corpus.**

## Batch 5 — ADR-0160…0182, verified (terse; patterns established)
- **CLI-governance class (`oya gate` → CaC/CaaS pipeline):** 0161, 0162, 0163, 0164, 0166, 0174, 0175, 0176, 0177, 0178 — **AMEND** (logic preserved, CLI form replaced).
- **Flagger → Argo-Rollouts (D10):** 0160 (the ADR IS "progressive-delivery-flagger" → heavy AMEND), 0165, 0180, 0181 — **AMEND**.
- **`[NEW]` 0167 tenant-cli — SUPERSEDE/heavy-AMEND:** a tenant-facing `oya` CLI directly contradicts "we don't keep things cli" + oya-CLI-retired; the tenant interface is cloud-native API/console. (Audit said KEEP.) LLM→cloud-intelligence; foundry.
- **Kafka→Pulsar:** 0166, 0169. **LLM→cloud-intelligence:** 0164, 0167. **foundry vocab:** 0161-0164,0167-0171,0174,0178,0180,0181.
- **obs: Mimir (0160/0164/0168/0172) vs code's VictoriaMetrics** — the standing obs contradiction.
- **KEEP/clean:** 0172 (cqrs), 0179 (pgcat), 0182 (api-gw/mesh separation), 0168/0171/0176/0177 (minor: oya-gate-ref/foundry).
- dropped-worth-preserving: none.

## Batch 4 — ADR-0119…0159, verified (founder: "we don't keep things CLI — do what Go does, cloud-native, in Rust" — the dominant <400 amendment theme)
| ADR | status | verdict | conversation-authority check (cited) |
|---|---|---|---|
| 0119 specs-flat-root | Accepted | **KEEP**(minor) | flat specs/ sound; foundry refs → intelligence. |
| 0122 ontology-crate-rename | Accepted | **KEEP**(minor) | clean rename; foundry; Kafka(L46)→Pulsar. |
| 0123 hyperscaler-maturity-claim-gate | Accepted | **AMEND [NEW]** | the claim-gate (= claim-ceiling CaC) → **CaC pipeline** (not the implied CLI); eradicate grit/Oya-VCS/foundry (L21-37); GitHub-Action ref now OK (audit said KEEP). |
| 0128 hyperscaler-arch-invariants | Accepted | **AMEND** | invariants good but enforcement = `oya gate`/`oya-dev-cli` (L23,L163-4) → **CaC pipeline**; "enforcement is vapor" → wire it as CaC. |
| 0129 changeset-dag + honest-claims-gate | Accepted | **AMEND [NEW]** | gate logic is gold (active RED/GREEN) BUT it's `oya gate`/`oya-dev-cli` (L21,L123-6) → **CaC pipeline** (audit said KEEP — missed the CLI form). |
| 0131 per-µsvc-flat-layout | Accepted | **KEEP**(minor) | pure-split-amended; CLI(L185) ref → pipeline. |
| 0132 bundle-dissolution | Accepted | **AMEND** | STILL says `microservices/<ms>/` (stale vs pure-split {oya,cloud}); `oya-dev-cli`(L98,L100)→pipeline; foundry. |
| 0133 conformance-program | Accepted | **AMEND** | 6-axis conformance → **CaC/CaaS pipeline**; `oya-dev-cli`(L196-8)→pipeline; Argo-Rollout(L110)=bridge-OK (D10). |
| 0135 aspirational-enforcement-gate | Accepted | **AMEND [NEW]** | gate is real+landed (gold) BUT `oya gate validate`/`oya-dev-cli` (L20,L125-8) → **CaC pipeline** (audit said KEEP). |
| 0139 slo-gated-promotion | Accepted | **AMEND** | grit/oya-vcs(L174)→eradicate; LLM(L104-5)→cloud-intelligence; **mimir(L131,164) vs the code's VictoriaMetrics — obs contradiction**; agentic-gate→CaC/CaaS. |
| 0142 crdt-portability | Accepted | **KEEP** | CRDT port + Loro/Yjs/Automerge adapters — ports/adapters mobility exemplar. |
| 0144 eu-ai-act-risk-tier | Accepted | **AMEND** | exemplary 5-tier; LLM(L160,209)→cloud-intelligence; risk-classification→CaC; foundry. |
| 0145 inter-µsvc-comm-reform | Accepted | **KEEP**(minor) | exemplary ESB-reshape; foundry→intelligence; Cedar→PARC. |
| 0148 service-mesh-cilium-ambient | Accepted | **KEEP**(minor) | exemplary mesh layering; fix phantom-0150→0183; foundry. |
| 0157 api-gateway-tier | Accepted | **AMEND** | **substrate-class in oya/ → should be cloud/** (sprawl); `oya gate`(L135)→pipeline. |
| 0159 feature-flags | Accepted | **AMEND** | **Flagger(L47,50) → Argo-Rollouts (D10)**; `oya gate`(L149)→pipeline; reconcile owned-vs-Flipt. |

**Batch-4 dropped-worth-preserving:** none dropped.
**Batch-4 cross-cutting:** (1) **CLI-governance class now CONFIRMED-PERVASIVE + founder-RULED** ("we don't keep things cli") → every `oya gate`/`oya check`/`oya verify`/`oya-dev-cli`/`cli`-layer ref across the <400 band = AMEND → cloud-native CaC/CaaS pipeline; the gate LOGIC (0129/0135 honest-claims/aspirational-enforcement) is gold and is PRESERVED — only the CLI delivery form is replaced. (2) `0132` still says `microservices/` (stale pure-split). (3) `0139` VictoriaMetrics(code)-vs-Mimir(ADR) obs contradiction. (4) `0159` Flagger→Argo-Rollouts. (5) `0157` api-gateway = oya/→cloud/ sprawl.

## Batch 3 — ADR-0094…0118, verified (founder heuristic: LOWER ADR # = HIGHER drift/staleness — confirmed)
| ADR | status | verdict | conversation-authority check (cited) |
|---|---|---|---|
| 0094 handler-trait | accepted | **KEEP** | typed Handler; clean. |
| 0095 tenant-slug | accepted | **KEEP** | clean (CLI L32 = passing ref). |
| 0096 supervisor-rust-not-node | accepted | **AMEND** | foundry-**supervisor** = agent/LLM → **cloud-intelligence** (D-LLM-DOMAIN); salvage Rust-vs-Node; grit retired. |
| 0098 supervisor-dep-policy | accepted | **AMEND** | [hyperscaler] best-effort durability accepts power-loss data-loss → fix; foundry→cloud-intelligence. |
| 0099 cedar-foundry-supervisor | accepted | **AMEND** | the `.cedar` policies = **PaC** under central PaaS (D-GOVERNANCE-CENTRAL); foundry→cloud-intelligence; Accepted-on-Proposed-0022. |
| 0100 supervisor-public-contract | Accepted | **AMEND** | foundry-supervisor → cloud-intelligence (D-LLM-DOMAIN); salvage zero-surface-change doctrine. |
| 0103 grit-cutover-inventory | Accepted | **SUPERSEDE** | grit/icm RETIRED (0116) + cloud-native-not-CLI; historical. |
| 0104 ecosystem-expansion-toolchain | Accepted | **AMEND [NEW]** | rule is good (no-stub/reachability) BUT `oya gate`/`oya-dev-cli` (L115,L119-120) → **CaC pipeline** (audit said KEEP — missed the CLI refs); foundry→intelligence. |
| 0105 13-layer-enum + check-family | Accepted | **AMEND [NEW]** | **3rd foundational grammar (w/ 0056,0092): enshrines `cli` layer + the `oya-check-*` check-family** (L45,L157,L173) → reconcile to CaC pipeline gates (audit said KEEP — missed the CLI-grammar). |
| 0106 application→usecase | Accepted | **KEEP**(minor) | clean rename; `cli`-layer ref part of the enum class. |
| 0108 sunset-lifecycle-automation | Accepted | **AMEND [NEW]** | sunset = governance lifecycle → **CaC/CaaS pipeline**; CLI refs (L110-236) → pipeline; grit/foundry residue (audit said KEEP). |
| 0109 lifecycle-automation-framework | Accepted | **AMEND [NEW]** | EXPLICITLY "per-lifecycle **dev-CLI** wrappers" + "**the CLI is the IO ring**" — THE governance-via-CLI pattern D-CLOUD-NATIVE/D-GOVERNANCE-CENTRAL retires → CaC/CaaS pipeline (audit said KEEP — missed it; KEY governance-CLI ADR). |
| 0115 registry-consolidation | Accepted | **AMEND [NEW]** | registry has a `vcs/` class (retired) + foundry/grit/icm residue → **scrub** (confirms founder "registry needs scrubbing"); consolidation itself fine (audit said KEEP). |
| 0116 retire-grit/icm/vox | accepted | **AMEND** | keep the retirement (aligns cloud-native-not-CLI); "Foundry pipeline" target → the cloud-native **CaC/CaaS pipeline**; foundry→intelligence. |
| 0117 repo-hygiene | Accepted | **KEEP**(minor) | hygiene sound; oya-vcs-admission→retired; ArgoCD (L51) = CD bridge (OK per D10). |
| 0118 retire-archive-orphan-lane | Accepted | **KEEP**(minor) | exemplary anti-false-enforcement; foundry/grit refs historical. |

**Batch-3 dropped-worth-preserving:** 0103 superseded — its CONTENT (the legacy-primitive→replacement inventory) is historical/already-actioned; nothing worth preserving beyond lineage. ✓
**Batch-3 cross-cutting (the dominant low-band finding):**
- **`[NEW-CLASS, HIGHEST-LEVERAGE]` THE CLI-GOVERNANCE CLASS.** A large swath of low-band ADRs is stale vs **D-CLOUD-NATIVE (not-CLI) + D-GOVERNANCE-CENTRAL (PaC/CaC/PaaS/CaaS)** — and the prior audit (pre-today's-rulings) marked most KEEP. Two strata: (1) **FOUNDATIONAL grammar/enum** — 0056 (`oya-check-*` BNF) + 0092 (`cli` layer) + 0105 (`cli` layer + check-family) **must be reconciled FIRST** (they propagate the pattern); (2) **governance/lifecycle/check ADRs** — 0011, 0018, 0063, 0069, 0090, 0091, 0099, 0104, 0108, **0109 ("the CLI is the IO ring")**, 0116 — all → the CaC/CaaS pipeline. This is the #1 amendment theme of the canon; folds into task #26 (CLI-surface → pipeline) + A-GOVERNANCE.
- foundry-**supervisor** band (0096/0099/0100) = agent/LLM → **cloud-intelligence** (D-LLM-DOMAIN), not generic intelligence.
- `registry/vcs/` class (0115) → scrub (founder-confirmed).

## Batch 2 — ADR-0058…0093, verified
| ADR | status | verdict | conversation-authority check (cited) |
|---|---|---|---|
| 0058 flat-µsvc-catalog | accepted | **AMEND** | flat/no-grouping aligns pure-split at catalog level; flat `crates/` LOCATION superseded (0512 nested); foundry (L38-39)→intelligence. |
| 0059 workflow+ontology adapter layer | accepted | **AMEND** | trinity integration; Kafka (L112)→Pulsar + OFF critical path (D-EVENT/D-D1); LLM (L73)→cloud-intelligence. |
| 0060 bominal-inheritance | accepted | **KEEP**(minor) | precedence rule sound; amend inherited Kafka(L75)→Pulsar; LLM(L73)→cloud-intelligence. |
| 0061 application-b2b-shell | accepted | **KEEP**(minor) | Redis→Valkey. |
| 0062 quality/perf/scalability bar | accepted | **AMEND** | [hyperscaler] day-1-hyperscale-for-all over-mandate; Kafka-as-benchmark→Pulsar; foundry(L130)→intelligence. |
| 0063 doc-set-coverage | accepted | **AMEND** | doc-coverage gate → CaC pipeline (D-GOVERNANCE-CENTRAL); foundry(L145); MASTERPLAN-as-planned vs D1. |
| 0064 canonical-base+packs | accepted | **KEEP** | exemplary seam/pack trichotomy. |
| 0065 docs-as-leptos-coemit | accepted | **AMEND** | foundry pervasive→intelligence; `docs` µsvc→`ops` (0067). |
| 0066 live-introspection-docs | accepted | **AMEND [NEW-correction]** | foundry + grit/ICM(retired)→pipeline. **BUT its GitHub-Actions refs (L53,L63) are now CORRECT (D-CICD-AUTHORITY) — the prior audit WRONGLY flagged GitHub-Actions as a problem.** |
| 0067 ops-console (`docs`→`ops`) | accepted | **AMEND** | [hyperscaler] ops mega-console (~18 BCs); foundry pervasive→intelligence; grit→retired; GitHub-Actions ref now correct. |
| 0069 active-machine-readable-artifact-contract | accepted | **AMEND [NEW]** | the validator is `oya-dev-cli` + `oya check` (L56,L134) — **CONTRADICTS D-CLOUD-NATIVE/D-GOVERNANCE-CENTRAL → must be a CaC pipeline gate, not CLI** (audit missed this); foundry→intelligence; GitHub-Action ref now correct. |
| 0083 error-handling-tiers | Accepted | **KEEP** | exemplary. |
| 0090 hyper-http-backbone | accepted | **KEEP**(minor)**[NEW]** | hyper sound; `oya gate` ref (L170) → CaC pipeline. |
| 0091 foundry-write-gate-foundations | accepted | **AMEND** | the write-gate (Proposed→Reviewed→Approved→Executed) IS a governance/admission gate → **CaC/PaaS pipeline** (D-GOVERNANCE-CENTRAL); foundry→governance/PARC; Kafka→Pulsar. |
| 0092 workspace-dependency-seam (12-layer enum) | accepted | **AMEND [NEW]** | **the 12-layer enum includes `cli` as a canonical layer** + "Oya VCS" (L263) — both contradict D-CLOUD-NATIVE (CLI/oya-VCS retired); reconcile the `cli` layer + eradicate Oya-VCS (audit said KEEP — missed this). |
| 0093 latency-reporter-rename | accepted | **KEEP** | honest-naming exemplar. |

**Batch-2 dropped-worth-preserving:** none dropped.
**Batch-2 cross-cutting (HIGH-VALUE) finds:**
- **`[NEW-CLASS]` GitHub-Actions INVERSION:** the prior audit flagged "GitHub-Actions CI" as a *problem* (vs oya-ci) across many ADRs (0063/0066/0067/0069 + more). Under **D-CICD-AUTHORITY (GitHub Actions = SOLE authority until cutover)** those refs are now **CORRECT, not contradictions.** Every audit "GitHub-Actions" finding must be re-polarized. (This corrects the audit; saves wrongly-amending ~N ADRs.)
- **`[NEW-CLASS]` the CLI pattern is baked into the FOUNDATIONAL grammar/enum:** `oya-check-*` in the BNF (0056) + `cli` as a layer in the 12-enum (0092) + `oya gate`/`oya check`/`oya-dev-cli`/`oya verify` refs (0069/0090/0091 + many). D-CLOUD-NATIVE (not-CLI) + D-GOVERNANCE-CENTRAL require reconciling the **foundational** 0056/0092 grammar/enum FIRST (the checks become CaC pipeline gates, not `oya-check-*`/`cli`-layer crates) — else every dependent ADR re-propagates the retired CLI. This is the single highest-leverage structural fix in the canon (foundational; the audit under-weighted it).

**Batch-1 cross-cutting finds:** (1) `[NEW]` ADR-0056 BNF enshrines `oya-check-*` CLI — a foundational propagator of the retired CLI pattern → must reconcile to CaC/pipeline (load-bearing; touches every check crate). (2) foundry vocab + "agent runtime"/"Foundry bridge" concepts pervasive → all → cloud-intelligence (D-LLM-DOMAIN). (3) 2 hyperscaler SPOFs confirmed (0028 same-provider DR, 0031 singleton ads-gate). (4) term-scan "argo"⊂"Cargo" false-positives — discounted (verify-own-tooling).
