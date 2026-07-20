# Canonical-Posture + Supersession + Retired-Vocabulary Map

> **Keystone map for the two-repo ADR audit (initial sweep 2026-06-06).**
> Built by the KEYSTONE MAP agent. READ-ONLY synthesis — no audited doc was modified.
> Binding context: SOURCE = `~/Developer/source` (GitHub `jason931225/oyatie`), 346 ADRs in `docs/decisions/`.
> LINUX = `~/Developer/linux`, the substrate PILOT (staging), 26 ADRs (0001–0026) in `docs/decisions/`.
> GOAL (founder): the MASTERPLAN becomes the single source of truth — "if it is not part of the masterplan, it is not needed."
> Every downstream auditor should treat this file as the shared baseline for what is TRUE, RETIRED, or SUPERSEDED.

---

## 0. How to read this map

- **SOURCE supersession is messy and multi-format.** Three notations coexist: front-matter `superseded_by:`/`supersedes:` (newer ADRs, 0100+), Markdown blockquote `> **Superseded-by:**` (Foundation cluster 0002–0050), and prose `**Superseded by ADR-NNNN**` headers. The supersession graph below was assembled from a complete grep across `source/docs/decisions/*.md`, not just the named ADRs.
- **`superseded_by: []` ≠ retired.** Empty arrays are the default for live ADRs. Only non-empty edges or explicit `status: Superseded`/`deprecated`/`retired` count.
- **`amends:` is NOT supersession.** Many retirement ADRs (0333/0334/0335/0347/0362) *amend* peers and *supersede* the retired microservice's `PRD.md`/`ARCHITECTURE.md` files — not other ADRs. Treat the microservice doc-pair as the retired artifact.
- **LINUX pilot ADRs all carry `supersedes: [] / superseded_by: []`** and a `renumber_note`: they renumber into the SOURCE sequence on merge. None supersede source ADRs yet; they are a parallel staging series. Collisions are guaranteed on merge (see §6).

---

## 1. SUPERSESSION GRAPH (SOURCE)

### 1.1 Confirmed ADR→ADR supersession edges (status verified on disk)

| Superseded ADR | Status on disk | Superseded / replaced by | Domain | Archive candidate? |
|---|---|---|---|---|
| ADR-0042 observability OTel + in-house UI | `superseded` | **ADR-0383** (Loki/Tempo/Mimir/Grafana, AGPL-3) | Observability | YES — archive |
| ADR-0046 vector-store strategy | `Superseded` | **ADR-0192** (Milvus canonical) | Data/vector | YES — archive |
| ADR-0052 inventory grit-cutover | `Superseded` | **ADR-0118** (retire archive/orphan fitness lane) | CI/tooling | YES — archive |
| ADR-0054 grit scaffold-claim pattern | `deprecated` | **ADR-0116** (retire external agent-coordination tooling) | CI/tooling | YES — archive |
| ADR-0107 tools implicit-app convention | `Superseded` | **ADR-0105** (13-layer enum) | Repo structure | YES — archive |
| ADR-0110 changeset state machine | `Superseded` | **ADR-0363** (retire agentic-VCS) | CI/VCS | YES — archive |
| ADR-0112 webhook-driven foundry agent invocation | `Superseded` | **ADR-0363** | CI/VCS | YES — archive |
| ADR-0113 vcs-orchestrator end-to-end | `Superseded` | **ADR-0363** | CI/VCS | YES — archive |
| ADR-0140 cross-cutting carriers adapter exemption | `Superseded` | **ADR-0145** (inter-µsvc comms reform) | Architecture | YES — archive |
| ADR-0141 workflow-ontology read-path direct | `Superseded` | **ADR-0145** | Architecture | YES — archive |
| ADR-0170 developer portal (Backstage) | `Superseded` | **ADR-0394** (bespoke-Rust IDP; Backstage quarantined) | DevPortal | YES — archive |
| ADR-0183 policy-engine separation (Cedar/Kyverno) | `Superseded` | **ADR-0379** (Kubewarden default admission) | Policy/admission | YES — archive (Cedar split principle survives) |
| ADR-0359 Jenkins completely replaces GH Actions | `Superseded` | **ADR-0511** (Argo Workflows; Jenkins transitory) | CI/CD | YES — archive |
| ADR-0316 capability-tier over product-fragmentation | `Proposed` + `superseded_by:[ADR-0329]` | **ADR-0329** (tier-system retired → tenant-class) | Tenancy | YES — archive |
| ADR-0015 architectural-flattening-target | `accepted` (PARTIAL) | **ADR-0131** (per-µsvc flat layout) — *only the docs-vs-crates split; BC/layer rules remain in force* | Repo structure | PARTIAL — keep BC rules |
| ADR-0121 onprem k8s (kubeadm/containerd/istio) | `Superseded` | **ADR-0375** (Talos+CAPI+ArgoCD) | Orchestration | YES — archive |
| ADR-0120 rust-first onprem tooling | `Superseded` | **ADR-0375** | Orchestration | YES — archive |
| ADR-0372 frontend SolidJS+Rust/WASM | `Superseded` | (see ADR-0372 body) | Frontend | YES — archive |
| ADR-0005 eventing-backbone Kafka/outbox | `proposed` (retired-in-fact) | **ADR-0377-kafka-to-pulsar** → ADR-0195/0397 (Pulsar+Oxia) | Eventing | PARTIAL — outbox pattern survives, Kafka retired |
| ADR-0055 object-graph → ontology rename | `accepted` | (rename ADR; absorbed by ADR-0122 ontology-crate-rename) | Naming | Historical record |
| ADR-0358 ideal-roadmap (Bazel/strangler) | `Proposed` | **ADR-0392** (Buck2 build graph) + **ADR-0408** (Buck2 CI/CD) reverse §2 | Toolchain | PARTIAL reversal |

### 1.2 Retirement / rename ADRs that supersede *microservice doc-pairs* (not ADRs)

| Retirement ADR | Status | What it retires | Absorbed/renamed into | Notes |
|---|---|---|---|---|
| **ADR-0329** tier-system retired → tenant-class | `Accepted` | `tier`/`tier-system` vocabulary; supersedes **ADR-0316** | `tenant_class` (`demo_trial`\|`paid`) + composable `billing_components` | Tenancy keystone |
| **ADR-0333** cell-µsvc retired → pattern-not-service | `Accepted` (`completed-locally`) | `microservices/cell/{PRD,ARCHITECTURE}.md` | "cell" becomes a deployment *pattern*; amends ADR-0248/0138/0131 | |
| **ADR-0334** shorts-µsvc merged into social | `Accepted` (`completed-locally`) | `microservices/shorts/{PRD,ARCHITECTURE}.md` | folded into `social`; amends ADR-0238/0132 | |
| **ADR-0335** foundry-µsvc retired → absorbed by intelligence | `Accepted` (`completed-locally`) | `microservices/foundry/{PRD,ARCHITECTURE,PHASE-01,PHASE-02}.md` | AI-agent platform → **intelligence**; **Governance** stays separate; amends ADR-0136/0138/0220/0239/0247/0255 | **Brand "foundry" RETIRED** |
| **ADR-0347** foundry-fitness → governance bulk rename | `Proposed` | all `oya-foundry-fitness-*` CI lanes/crates/catalog | `oya-governance-*` | Doctrine-only; 34 per-lane IPs collapsed |
| **ADR-0362** full grouping retirement → flat-only catalog | `Accepted` | ALL product grouping (suite/family/bundle/vertical) as architecture artifact | flat-only; grouping = presentation tag, revivable only by future ADR | amends ADR-0132 |
| **ADR-0363** retire agentic-VCS foundry → intelligence/Forgejo | `Accepted` (amended_by 0510/0513) | bespoke `oya vcs`/`oya git`/changeset-SM/merge-queue/webhook; supersedes **ADR-0110/0112/0113** | plain git + Forgejo PRs + Prow-shaped cloud-ci; Foundry→Intelligence | Forge keystone (see §5) |
| **ADR-0116** retire external agent-coordination tooling | `Accepted` | grit/rtk/icm/vox; supersedes **ADR-0054** | in-repo Foundry pipeline (now "intelligence/oya-ci") | |
| **ADR-0118** retire archive/orphan fitness lane | `Accepted` | supersedes **ADR-0052** | — | |
| **ADR-0130** deprecate knowledge-graph registry file | — | knowledge-graph-registry file | migrate to ontology | |
| **ADR-0138** foundry six-path deprecation | — | six foundry paths | — | pre-0335 |
| **ADR-0336** Redis retired → Valkey | (per GLOSSARY) | Redis 7.4+ (SSPL/RSAL relicense) | **Valkey** (BSD-3) | License-driven |
| **ADR-0377-kafka-to-pulsar-via-kop** | `Accepted` | standalone Kafka; supersedes **ADR-0005** | Pulsar 4.x + Oxia (KoP wire-compat) | Eventing |

### 1.3 Supersede-chains worth flagging (multi-hop / churn)

- **CI/CD churn (the long chain):** ADR-0349 (Jenkins+ArgoCD *augment* GH Actions) → ADR-0359 (Jenkins *fully replaces* GH Actions; now `Superseded`) → ADR-0361 (execute Jenkins-native revamp) → ADR-0408 (Buck2-driven CI/CD reverses ADR-0358 §2) → ADR-0511 (Argo Workflows = destination; Jenkins transitory; supersedes 0359) → ADR-0513 (oya-ci bespoke-Rust Prow; phased replacement of ADR-0380's Jenkins path) → ADR-0514 (target architecture). **Net current truth:** Buck2 (build/RBE) + Argo Workflows (k8s-native CI orchestration) + ArgoCD/Argo-Rollouts (CD), with `oya` gate engine as governance overlay and Forgejo Commit Status as the gate sink; Jenkins is **transitory bootstrap only**.
- **VCS/forge chain:** ADR-0363 (Forgejo "canonical" host, retire agentic-VCS) → ADR-0510 (Forgejo reframed **transitory**; bespoke hyperscaler monorepo-VCS is the *declared destination*, cutover numerically triggered) → ADR-0513 (oya-ci on the Forgejo substrate). See §5 fault-line.
- **Foundry dissolution:** ADR-0136 (foundry as single µsvc) → ADR-0239 (internal-scope amendment) → ADR-0242 (oyatie-is-a-tenant) + ADR-0247 (self-hosting doctrine) + ADR-0255 (intelligence two-layer) → **ADR-0335** (foundry retired, absorbed by intelligence) → ADR-0347 (CI-lane rename). NOTE: ADR-0136 still shows `status: Accepted, superseded_by: []` on disk despite 0335/0247 declaring it superseded — a **stale-front-matter drift** auditors should flag.
- **Policy/authz chain:** ADR-0150 (Cedar engine) → ADR-0183 (Cedar app-authz vs Kyverno admission separation; now `Superseded`) → ADR-0243 (Cedar as universal gate) + ADR-0246 (policy-engine substrate promotion) → **ADR-0379** (Kubewarden default admission; Cedar app-authz separation principle retained).

---

## 2. RETIRED VOCABULARY

| Dead term | Replacement | Governing ADR / source | Notes |
|---|---|---|---|
| **foundry** (the BRAND/µservice) | **cloud-intelligence** (consumer AI) + **governance** (CI/gates) | ADR-0335, ADR-0347; GLOSSARY §"Foundry (RETIRED)" L1032 | Founder: "cloud-intelligence is the valid name." Brand is dead even though hundreds of `oya-foundry-*` / "Foundry" strings persist corpus-wide. |
| **Foundry Furnace** / "Furnace" | (self-improvement loop inside intelligence) | CONTRADICTION-LEDGER LEDG-013 ("retire Furnace branding"); GLOSSARY L241 | |
| **retired external agent harness** (internal-pipeline brand) | "intelligence" (consumer AI) / "oyatie.foundry workflow library inside dev-tools-cell-N" | GLOSSARY L1042 (ADR-0247 D-10 + ADR-0328 + ADR-0335) | No replacement brand needed |
| **tier / tier-system / capability-tier** | **tenant-class** (`demo_trial`\|`paid`) + composable `billing_components` (`per_seat`,`per_usage`) | ADR-0329 (supersedes ADR-0316); GLOSSARY L6 | Autonomy *tiers* T1–T4 are a DIFFERENT, live concept (policy autonomy ceiling) — do not conflate |
| **cell (as a microservice)** | **cell (as a deployment pattern only)** | ADR-0333 | `microservices/cell/*` retired |
| **shorts (as a microservice)** | merged into **social** | ADR-0334 | |
| **M0 / M1 / M2 / M3 / Milestone / MVP** | descriptive **Wave names** (W-Foundation, W-Foundry-Preview, …) | GLOSSARY L250/L504 (RETIRED 2026-05-09); MFL-0003 | "Foundry-Preview" wave name itself now anachronistic post-0335 |
| **CUG / Closed-User-Group** | **Team** | GLOSSARY L252/L336 (retired 2026-05-09); MFL-0004 | |
| **Redis** (as canonical substrate) | **Valkey** | ADR-0336; GLOSSARY L1122 | SSPL/RSAL relicense; OSI-strict policy |
| **Kafka** (standalone) | **Pulsar 4.x + Oxia** (KoP wire-compat) | ADR-0377-kafka-to-pulsar (supersedes ADR-0005) | |
| **object-graph / knowledge-graph-registry** | **ontology** | ADR-0055, ADR-0122, ADR-0130 | |
| **grit / rtk / icm / vox** (external coord tools) | in-repo pipeline (now intelligence/oya-ci) | ADR-0116 (supersedes ADR-0054) | |
| **product grouping** (suite/family/bundle/vertical as arch artifact) | flat-only catalog; grouping = presentation tag | ADR-0362 (amends ADR-0132) | |
| **Jenkins** (as destination CI) | **Argo Workflows** (destination); Jenkins = transitory bootstrap | ADR-0511 | |
| **GitHub Actions** (as CI) | self-hostable CI (Jenkins→Argo Workflows) | ADR-0359/0361/0511 | GH Actions budget SPOF was the trigger |
| **Bazel / rules_rust** | **Buck2** | ADR-0392 (reverses ADR-0358 §2) | |
| **Backstage** (dev portal) | bespoke-Rust IDP (Leptos + ops-BFF) | ADR-0394 (supersedes ADR-0170) | Backstage = feature reference only |
| **`oya-foundry-fitness-*`** (CI lane prefix) | **`oya-governance-*`** | ADR-0347 | |
| **ADR-number-keyed gate/lane names** (`adr-0145-*`, `M01-P18`) | function-named semantic ids | planning-ssot-consolidation.md §"Canonical naming" | FORBIDDEN antipattern going forward |

> **Lint signal:** GLOSSARY §"§Note (2026-05-21 transition)" appears at the foot of many docs declaring `oya-governance-*` (vs old `oya-foundry-*`) is the live prefix. Treat residual `oya-foundry-*` in *new* work as retired-vocab leakage (MFL-0002/0003 brand-residue lanes).

---

## 3. CANONICAL POSTURE (current TRUE high-level decision per domain, SOURCE)

| Domain | Current canonical decision | Governing ADR(s) |
|---|---|---|
| **Isolation / runtime** | K8s-everywhere; runtime ladder native→sandbox→microvm→confidential; wasmtime canonical WASM; firecracker/Kata + Cloud-Hypervisor microVMs | ADR-0147 (runtime ladder), ADR-0200 (wasmtime), ADR-0254 (deployment-model spectrum), ADR-0254-kubernetes-everywhere |
| **Data / storage** | Best-of-breed per workload: **Milvus** (vector >10M; pgvector ≤10M), **SeaweedFS** primary + **Ceph RGW** scale-up (object), **ClickHouse** (OLAP), **TimescaleDB** (tenant TS), **Postgres + pgcat** (relational pooling) | ADR-0192, ADR-0196, ADR-0193, ADR-0194, ADR-0179 |
| **Identity / crypto** | **Zitadel** primary OIDC IdP; OIDC/SAML/SCIM 2.0/Passkeys/WebAuthn first-class; step-up ACR; KCMVP HSM for KR | ADR-0187, ADR-0188, ADR-0189, ADR-0190 |
| **Policy / authz** | **Cedar** = universal authorization gate (app-authz PDP); **Kubewarden** = default k8s admission (supersedes ADR-0183's Kyverno) | ADR-0243, ADR-0246, ADR-0379 (sup. ADR-0183), ADR-0191 |
| **CI/CD** | **Buck2** (build/RBE) + **Argo Workflows** (k8s-native CI orchestration) + **ArgoCD/Argo-Rollouts** (CD); `oya` = governance-gate engine; Jenkins transitory bootstrap; bespoke-Rust **oya-ci** (Prow-shaped) is the target platform | ADR-0392, ADR-0408, ADR-0511, ADR-0513, ADR-0514 |
| **Forge / SCM** | **CONTESTED** — Forgejo (self-hosted) is the *transitory* canonical host per ADR-0363; bespoke hyperscaler monorepo-VCS is the *declared destination* per ADR-0510 (cutover numerically triggered). Founder's migration directive = **GitHub** `jason931225/oyatie`. See §5. | ADR-0363, ADR-0510, ADR-0374, ADR-0387 |
| **Orchestration / k8s** | **Talos** immutable node-OS + **CAPI** + **ArgoCD** fleet substrate (bare-metal via Sidero); replaces kubeadm/containerd/istio onprem stack | ADR-0375 (sup. ADR-0121/0120), ADR-0370, ADR-0378, ADR-0382 |
| **Intelligence / AI** | **Intelligence** = two-layer AI substrate (consumer AI + internal self-modification); absorbs retired Foundry; Governance stays separate | ADR-0255, ADR-0335, ADR-0220 (historical), ADR-0293 (meta-trust-root) |
| **Tenancy** | **tenant** = universal scoping primitive; **oyatie-is-a-tenant** doctrine; **tenant-class** (demo_trial/paid) not tiers; per-tenant audit-log slicing | ADR-0244, ADR-0242, ADR-0329, ADR-0162, ADR-0163 |
| **Masterplan / SSOT** | `MASTERPLAN.md` = human compatibility projection (NOT authority); `/specs/masterplan.json` = canonical authority. **Authored-vs-generated is an OPEN founder question** (see §4). | MASTERPLAN.md front-matter, planning-ssot-consolidation.md, planning-ssot-drift-prevention.md |
| **Eventing** | Pulsar 4.x + Oxia (KoP wire-compat); transactional-outbox pattern retained; Kafka retired | ADR-0377-kafka-to-pulsar (sup. ADR-0005), ADR-0195, ADR-0397 |
| **Observability** | Loki/Tempo/Mimir/Grafana (AGPL-3 carve-out); OTel emission contract | ADR-0383 (sup. ADR-0042), ADR-0263 |
| **License posture** | OSI-strict; no AGPL/GPL in product code (carve-outs for server-side substrate w/ evidence); Class-C OSS stewardship | ADR-0013, ADR-0211, ADR-0345 |

---

## 4. MASTERPLAN FACTS

- **`source/docs/MASTERPLAN.md`** — front-matter: `doc_class: MasterPlan`, `shape: compatibility_projection`, `authority_tier: 0`, `canonical_authority: /specs/masterplan.json`. Body line 1: *"This file is a compatibility projection for humans. It is not the implementation authority."* Agents resolve truth through `/specs/root-hub-pointers.json`, `/specs/masterplan.json`, `master-plan-sequencing.json`, `planning-closure-contract.json`, `planning-closure-status-closure-ledger.json`, and the live gate output. FD-001 = Tenant RBAC at full production depth (NOT a preview). Development order is vertical-slice; promotion is via plain-git branch → PR against `dev` → **Jenkins** required checks (note: stale — CI is now Argo Workflows per ADR-0511) → `oya gate`/`oya verify` evidence.
- **`/specs/masterplan.json`** — 385 KB; the declared canonical authority. Lives alongside ~118 other `/specs/*.json` machine-readable artifacts (root-hub-pointers, master-plan-sequencing, planning-closure-contract/status, tenant-model, cedar-policy-schema, hyperscaler-architecture-invariants, ci-farm-substrate-canonical, etc.).
- **THE OPEN FOUNDER QUESTION (authored-as-SSOT vs generated-from-ADRs) — UNRESOLVED, two designs contradict:**
  - **`planning-ssot-consolidation.md`** (2026-05-26) designs masterplan as **GENERATED from ADR front-matter** (`planning_impact`, `status`, `supersedes`, `deliverables`, `milestone`). ADRs = the **authored, immutable SSOT** (append-only; supersede, never edit). Status is *derived* from `verified_by` gate output, NOT stored in the ADR. Precedent: Kubernetes KEP. Even proposes re-founding the ADR log from ADR-0000 (re-author survivors with `consolidates:` provenance; archive old series frozen).
  - **`planning-ssot-drift-prevention.md`** says *"masterplan.json **is the one planning authority**; ADRs + canonical specs **bind into it**"* via a `planning-ssot-coverage` gate (frontmatter `masterplan_ref` + bidirectional + supersession-aware). Found only **8.8% ADR binding** today.
  - **These are opposite directions** (ADRs-generate-masterplan vs masterplan-is-authority-ADRs-bind-in). The founder's stated GOAL ("masterplan = single source of truth; backfill it with true+relevant decisions") leans toward **masterplan-as-authority**, but the consolidation design (and ADR immutability doctrine) leans **generated-from-ADRs**. **DO NOT ASSUME — flag every masterplan-related decision under both readings.**

---

## 5. KNOWN FAULT-LINES (cross-side tensions auditors must watch)

1. **Data tier: LINUX owned-DB vs SOURCE best-of-breed.** LINUX **ADR-0001** wants a from-scratch Rust multi-model engine that *eliminates the PostgreSQL/sqlx dependency* (cites Spanner/CockroachDB). SOURCE picks best-of-breed managed substrates: Milvus (ADR-0192), SeaweedFS/Ceph (ADR-0196), ClickHouse (ADR-0193), TimescaleDB (ADR-0194), Postgres+pgcat (ADR-0179). **Direct scope/ownership tension** — "own the substrate" vs "assemble proven OSS." LINUX ADR-0020 even flags Milvus as an UNSAFE deferral with a hard vector-count gate.
2. **Policy: LINUX owned-policy (ADR-0021) vs SOURCE Cedar.** LINUX ADR-0021 designs a typed **compile-to-Rust, tier-aware** policy language — but explicitly **"Cedar-compatible"** (extends Cedar's PARC model + Lean soundness; `cedar-policy` is the vendored adapter *now*, owned port later). SOURCE makes **Cedar the universal gate** (ADR-0243/0246) + Kubewarden admission (ADR-0379). **Tension is "own vs reuse Cedar," not a flat contradiction** — LINUX positions itself as the owned successor to the same model. Watch: LINUX adds autonomy-tier T1–T4 as a first-class policy dimension (this is NOT the retired tenant "tier-system" of ADR-0329 — different axis).
3. **Isolation/runtime: LINUX Capsule/framekernel vs SOURCE Talos+containerd+firecracker+wasmtime.** LINUX **ADR-0018** adopts (with reservations) a "we are the host; framekernel's own isolation, no separate containerd" Capsule model — but ground-truth re-verified that the framekernel currently **boots as a QEMU guest** (the literal host claim is time-boxed to an uncommitted H2). LINUX **ADR-0014** is more reconcilable: one OCI/CRI frontend + pluggable `IsolationBackend` port (native/sandbox/microvm/confidential), evolving the pilot's `stack/containerd`. SOURCE uses Talos node-OS (ADR-0375) + K8s-everywhere + Kata/firecracker/Cloud-Hypervisor + wasmtime (ADR-0200/0147/0254). **Tension: own-the-host/kernel (framekernel) vs assemble-the-substrate (Talos+containerd).** LINUX ADR-0025 wants a *Rust "Talos"* (beat-or-parity) — competes with SOURCE's adoption of actual Talos.
4. **Forge: GitHub vs Forgejo vs bespoke-VCS (THREE-way).** Founder migration decision = **GitHub** `jason931225/oyatie`. SOURCE canon: **Forgejo** self-hosted is canonical host (ADR-0363/0374/0387), GitHub is mirror; ADR-0510 then reframes Forgejo as **transitory** with a **bespoke hyperscaler monorepo-VCS** (Piper/Sapling/Mononoke-class, Rust) as the *declared destination*. So the founder's GitHub directive conflicts with **even the transitory** Forgejo canon, and the long-horizon canon is "own the VCS entirely." **Surface, do not resolve.**
5. **"Own everything" scope/breadth.** LINUX repeatedly chooses OWN_DAY0 (DB engine, policy language, kernel/framekernel, node-OS, gRPC framing eventually, container runtime). SOURCE is more "best-of-breed OSS now, own when proven" (ADR-0019 universal-port-ratchet on the LINUX side actually agrees with this — vendored now, owned when proven). The breadth tension is whether the pilot's day-0 ownership ambition is consistent with source's staged-ownership ratchet. **Both repos share the "own when proven" ratchet language (LINUX ADR-0019/0020, SOURCE ADR-0211/0173) — the disagreement is the *trigger threshold*, not the principle.**
6. **Stale front-matter / supersession drift (SOURCE-internal, but poisons any merge).** ADR-0136 reads `Accepted, superseded_by:[]` while ADR-0335/0247 declare it superseded. ADR-0005 reads `proposed` while ADR-0377-kafka declares it superseded. ADR-0145 §body says "post-landing, mark ADR-0140/0141 superseded_by:ADR-0145" — done in those files but the cross-ref discipline is inconsistent. Auditors must trust the *superseding* ADR's claim over the *superseded* ADR's stale front-matter.

### Verdict on the LINUX auto-reconciliation (wm4gkcey5) — NOT "plain wrong"
The recent linux edits are internally coherent and self-aware (each carries `review_note` from a critic loop; ADR-0018 honestly records `consensus=FALSE` and time-boxes the host claim; ADR-0021 removed phantom citations). They are **deliberately divergent** from source (own-DB, own-policy, framekernel) rather than erroneous — these are genuine architecture tensions to surface, not reconciliation bugs. No linux ADR fabricates a source posture; ADR-0014 and ADR-0021 explicitly position as *evolutions/owned-successors* of the source substrate. The one item to watch: ADR-0001's "eliminate PostgreSQL" framing is the sharpest unflagged conflict with source's Postgres+best-of-breed posture.

---

## 6. NUMBER COLLISIONS

### 6.1 Within `source/docs/decisions/` (same directory — real ID collision)
- **ADR-0377 — DUPLICATE.** Two distinct ADRs share the number:
  - `ADR-0377-forgejo-board-git-ref-cas-fallback.md` (status: `Proposed (conditional)`, 2026-05-27, forge board)
  - `ADR-0377-kafka-to-pulsar-via-kop.md` (status: `Accepted`, 2026-05-28, eventing)
  - **This is a genuine collision** — two authoritative ADRs, same id, different domains. Must be resolved (one renumbers). The Kafka→Pulsar one is `Accepted` and supersedes ADR-0005; the forge-board one is conditional-Proposed.

### 6.2 Cross-directory (ADR-numbered files outside `decisions/`)
- **ADR-0055**: `decisions/ADR-0055-object-graph-renamed-to-ontology.md` **and** `advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md` — different doc series, same prefix.
- **ADR-0145**: `decisions/ADR-0145-inter-microservice-communication-reform.md` **and** `operators/ADR-0145-runtime-impact-changelog.md` — different series.
- These cross-dir ones are *namespace overlaps* (a non-`decisions/` doc reuses the ADR-NNNN convention), lower-severity than 6.1 but they break any flat `ADR-NNNN` lookup/index.

### 6.3 Numbering gaps & non-contiguous allocations (declared, not collisions, but index-poisoning)
- `decisions.json next_adr` is **STALE** (records ADR-0377 or ADR-0392 depending on the reader) while origin/dev carries ADRs through **ADR-0509**, and ADR-0392/0408 were forward-allocated by founder convention (Buck2 reversal), ADR-0510/0511/0513/0514 above that. Gaps ADR-0377–0391 and ADR-0393–0407 are documented-open. **`decisions.json next_adr` must be re-derived from the on-disk corpus, never trusted at face value.**

### 6.4 LINUX ↔ SOURCE collision on merge (guaranteed)
- LINUX pilot ADRs are **ADR-0001…ADR-0026**, every one carries `renumber_note` ("renumber into source's ADR sequence on merge"). SOURCE already occupies 0001–0514. **All 26 linux numbers collide with existing source ADRs** (e.g. linux ADR-0001 distributed-DB vs source ADR-0001 foundation; linux ADR-0014 container-runtime vs source ADR-0014 build-vs-buy; linux ADR-0021 owned-policy vs source ADR-0021 foundry-capability-registry). The pilot series MUST be renumbered (likely to ADR-0515+) on merge — never merged at face value.

---

## 7. Sources read (load-bearing citations)
- SOURCE: `docs/MASTERPLAN.md` (L1–113), `docs/CONTRADICTION-LEDGER.md`, `docs/ADR-CONSOLIDATION-PLAN.md`, `docs/MISTAKES-LEDGER.md`, `docs/GLOSSARY.md` (§11 L490+, L1032 Foundry-RETIRED, L1122 Redis-RETIRED, L250/L504 M0–M3, L252/L336 CUG), `docs/ideas/planning-ssot-consolidation.md`, `docs/ideas/planning-ssot-drift-prevention.md`.
- SOURCE ADR front-matter (grep across all `decisions/*.md`): 0316/0329/0333/0334/0335/0347/0362/0363, 0042/0046/0052/0054/0107/0110/0112/0113/0140/0141/0170/0183/0359/0015/0121/0120/0372 (superseded set), 0349/0359/0361/0408/0511/0513/0514 (CI churn), 0374/0377×2/0387/0510 (forge), 0187/0192/0196/0200/0243/0246/0254/0375/0379/0383/0392 (canonical posture).
- LINUX: all 26 pilot ADR front-matter + decision bodies of ADR-0001/0014/0018/0021/0025 (fault-lines).

---
*End of keystone map. Downstream auditors: trust the superseding ADR over stale front-matter; treat `foundry`/`tier`/`M0-M3`/`Jenkins-as-destination`/`Kafka`/`Redis`/`Backstage` as retired; treat masterplan authored-vs-generated as OPEN; renumber all linux pilot ADRs on merge.*
