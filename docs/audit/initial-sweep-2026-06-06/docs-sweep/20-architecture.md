# 20 — Architecture Cluster Sweep (lane: architecture)

**Scope:** `/Users/jasonlee/Developer/source/docs/architecture/` — **36 top-level entries** (33 `.md`, 1 `.json`, 1 `.html`, 1 `diagrams/` dir with 10 `.md`) = **45 doc files reviewed**. The task brief said "43 docs"; actual on-disk is 36 top-level + 10 diagrams. **No silent truncation.**
**Method:** scripted per-file canon-term heat map (foundry/Jenkins/Forgejo/tenant-tier/Redis/Kafka/Cedar/Zitadel/M0-M3) over all 45 files → line-anchored grep of contradiction phrasings → deep-read of the highest-density standalone-assertion docs (`unified-ecosystem-thesis`, `foundry-fitness-to-governance-transition`, `training-cost-doctrine`, `wave-3-g-executive-briefing-2026-05-21`, the 2 architecturally-loaded diagrams `cedar-policy-evaluation-flow` + `capability-tier-projection-flow`). The 7 large `*-line-audit`/`*-cross-reference`/`*-reachability` docs were **scanned, not fully read** — they are machine-generated graph/line audits that *quote ADR titles verbatim*; their term-hits are mostly path-refs to ADR filenames, NOT standalone architectural assertions (see false-positive note below).

> **KEY METHODOLOGICAL FINDING.** The bulk of high Cedar/foundry/Zitadel counts in this cluster live in the **`adr-cross-reference-graph`** and **`*-line-audit`** docs, which are GENERATED indexes that echo ADR filenames and titles (`ADR-0150-cedar-policy-engine.md`, `ADR-0187-canonical-oidc-idp-zitadel-primary`). Those are **reachability path-refs, not the cluster asserting a position** — the canon violation lives in the *underlying ADR* (out of this lane's scope; flagged for the ADR lane). The genuine architecture-cluster contradictions are the **standalone doctrine/thesis/diagram assertions** listed first below.

---

## A. GENUINE CANON-CONTRADICTIONS (architecture cluster asserts these in its own voice)

### A1. Cedar framed AS the policy ENGINE — violates canon #6 (Cedar = CONTRACT; owned PARC = engine)
The single most pervasive contradiction in the cluster. Multiple standalone-voice docs assert Cedar IS the decision engine, not a contract language fronting an owned engine. **PARC appears ZERO times anywhere in the cluster** — the owned engine is entirely absent from the architecture narrative.

| File:line | Contradicting claim |
|---|---|
| `unified-ecosystem-thesis-2026-05-21.md:104` | "one policy engine (**Cedar; ADR-0243**)" — Cedar named as THE engine in the substrate-invariant list |
| `unified-ecosystem-thesis-2026-05-21.md:111` | "the marketplace by Cedar, and **Cedar by the tenant's pack**" — Cedar as the terminal authority |
| `unified-ecosystem-thesis-2026-05-21.md:272` | "**Definition**: one Cedar policy engine for every authorization and denial path" |
| `unified-ecosystem-thesis-2026-05-21.md:1938` | "**Cedar evaluation engine**: deterministic, replay-safe, sub-10ms p99" |
| `training-cost-doctrine-2026-05-21.md:115,221,432,1240` | "one **Cedar policy engine**" repeated as a substrate invariant ("ONE-POLICY-ENGINE") |
| `wave-3-g-executive-briefing-2026-05-21.md:177` | "One **Cedar-based authorization engine** for every 'may this happen' decision" |
| `wave-3-g-executive-briefing-2026-05-21.md:1212` | Glossary: "**Cedar.** The policy-**engine** language oyatie uses for every authorization decision" |
| `wave-3-g-synthesis-adjudication-2026-05-21.md:659,717` | "one Cedar policy engine"; "**Cedar (AWS) is the engine**; OPA/Rego is [precedent]" |
| `diagrams/cedar-policy-evaluation-flow.md:41` | mermaid participant `Cedar as **policy evaluator**` — the diagram makes Cedar the decision node |
| `diagrams/capability-tier-projection-flow.md:36` | `Tier --> Cedar["Cedar permit set"]` + `Registry->>Cedar: evaluate tier activation policy` (line 64) — Cedar evaluates |

**Fix:** Reframe to canon #6 — **Cedar is the policy CONTRACT/fragment language; the owned PARC is the engine (PDP) that compiles + evaluates**. In the diagrams, the evaluator participant should be **PARC (the policy engine)**, with Cedar as the *fragment format* it consumes. Note `ADR-0191-edge-authz-tier-vs-origin-cedar-pdp` and `ADR-0183` already use the correct "**Cedar PDP**" decomposition (waypoint enforcement point + fragments) — those are closer to canon and should be the template; the thesis/doctrine docs lag them.

### A2. Zitadel framed as CANONICAL/PRIMARY identity — violates canon #6 (oya-identity owned; Zitadel = BRIDGE)
The cluster echoes the ADR title that declares Zitadel canonical, and the keystone walkthrough treats Zitadel as the *authoritative* identity at runtime — not as a bridge.

| File:line | Contradicting claim |
|---|---|
| `adr-cross-reference-graph-2026-05-20.md:251` | "ADR-0187 — **Canonical OIDC IdP: Zitadel primary**" (quoted title — flag the ADR; cluster propagates it) |
| `keystone-bundle-intern-walkthrough.md:119,132` | "**Authoritative identity** \| Zitadel `personal-idp`" / "Zitadel personal-idp" — Zitadel IS the authority, no bridge framing |
| `keystone-bundle-intern-walkthrough.md:223,229,231` | "All authenticated writes carry **Zitadel JWT**"; "refresh token to the cell-local **Zitadel endpoint**"; "**Zitadel issues** a fresh JWT" — runtime identity is Zitadel, not oya-identity |

**Fix:** Reframe per canon #6 — **oya-identity is the owned canonical IdP (endpoint); Zitadel is a transitional BRIDGE adapter**. The "authoritative identity = Zitadel" framing must become "authoritative identity = oya-identity (Zitadel bridge during transition)". JWT-issuance language should target oya-identity with Zitadel as operative-until-cutover.

### A3. Eventing tier is internally contradictory AND off-canon — violates canon #5 (Kafka→Pulsar) + AI-slop internal-contradiction
The cluster names **three mutually inconsistent** eventing backbones. Canon #5 says the endpoint is owned and Kafka→**Pulsar** is the bridge. One doc is canon-aligned (Pulsar), another asserts a different stack (Redpanda+NATS), and raw Kafka persists.

| File:line | Claim | Canon status |
|---|---|---|
| `adr-cross-reference-graph-2026-05-20.md:1924` (quoting ADR-0005/0192) | "**Pulsar = event backbone** per ADR-0005"; "Pulsar + etcd + SeaweedFS dependency stack" | canon-ALIGNED (Pulsar bridge) |
| `keystone-bundle-idea-refine-deep-dive.md:297` | "oyatie uses **Redpanda + NATS**" | CONTRADICTS ADR-0005 Pulsar + canon #5 |
| `keystone-bundle-idea-refine-deep-dive.md:527,530,534,539,1209,1903,2772` | "**NATS JetStream** for asynchronous events" / "background job framework (NATS JetStream)" / "Postgres CDC → Debezium → **NATS JetStream**" | introduces a 2nd uncanonized eventing tech |
| `product-graph.md` / `product-graph.html` | raw **Kafka** mentions (7 / 1) | stale (Kafka→Pulsar bridge framing missing) |

**Fix:** Resolve to one canon line: **Pulsar is the transitional bridge toward the owned eventing endpoint; Kafka→Pulsar.** The Redpanda+NATS/NATS-JetStream framing in `keystone-bundle-idea-refine-deep-dive.md` directly contradicts the ADR-0005 Pulsar backbone and must be reconciled (either NATS is a scoped sub-component for sub-1s jobs and must be stated as such, or it is stale and dropped). This is a **real internal contradiction**, not just a stale term.

### A4. ADR-0023 "Foundry sandbox" + isolation stack lacks the framekernel-host endpoint — partial violation of canon #7
Isolation framing across the cluster is **Wasmtime + Firecracker + Kata + Cloud-Hypervisor microVM** — the assume-breach microVM *default* (canon #7) is correctly present, but the **framekernel-host COMMITTED endpoint is entirely absent**, and the sandbox ADR still carries the retired "foundry" brand.

| File:line | Claim | Canon status |
|---|---|---|
| `adr-cross-reference-graph-2026-05-20.md:111` (quoting ADR-0023) | "**Foundry sandbox** — Wasmtime + WASI Preview 2 … **Firecracker microVMs**" | microVM-default OK; "foundry" brand retired (#2); no framekernel endpoint (#7) |
| `wave-3-g-executive-briefing-2026-05-21.md:350,708,714,825,1151` | "**ADR-0254 (Kubernetes + Cloud Hypervisor + Kata)** is THE deployment shape" | microVM-default OK; framekernel-host endpoint missing (#7) |
| `hyperscaler-pattern-attribution.md:257-258,504-505,941,1446` | "Cloud Hypervisor + Kata … NOT gVisor (rejected)" — VM-per-workload isolation | aligned to microVM-default; no framekernel-host endpoint named |

**Fix:** Per canon #7, **framekernel-host is the COMMITTED isolation endpoint**; Cloud-Hypervisor/Kata/Firecracker microVM is the **assume-breach DEFAULT bridge** (not the terminal architecture). Add framekernel-host as the endpoint to ADR-0254/ADR-0248 framing. Rename ADR-0023 "Foundry sandbox" per canon #2 (this is the *intelligence/AI-substrate* sense → `cloud-intelligence` sandbox). The microVM-default itself is canon-COMPLIANT — do not flag it as native-default (no `native-default`/`secure-by-default-native` phrasing exists in this cluster: 0 hits).

### A5. ArgoCD framed as the CANONICAL progressive-delivery / federation controller — violates canon #4
The cluster echoes ADR titles/bodies that make ArgoCD the canonical GitOps controller, with no "operative-until-cutover bridge toward oya-ci" framing.

| File:line | Claim |
|---|---|
| `adr-cross-reference-graph-2026-05-20.md:224` (ADR-0160) | "Progressive Delivery via Flagger 1.x … **ArgoCD-integrated**" |
| `adr-cross-reference-graph-2026-05-20.md:1627,1629` | "Flagger 1.x as **the canonical progressive-delivery controller**. Integrates with ArgoCD" |
| `adr-cross-reference-graph-2026-05-20.md:235,1751` (ADR-0171) | "Multi-cluster federation via **ArgoCD ApplicationSets**" |
| `adr-cross-reference-graph-2026-05-20.md:2230` (ADR-0240) | "OpenTofu + Helm + Kustomize + **ArgoCD/Flux**" as the sovereign-pack delivery stack |

**Fix:** Per canon #4, **oya-ci (Run+graph; Prow+Tekton+Argo) is the canonical endpoint; Argo is OPERATIVE-until-cutover, not the endpoint (build-first-cutover-later).** These are ADR-quoting echoes — flag the underlying ADRs (0160/0171/0240) for the ADR lane; the cluster needs no edit beyond not presenting ArgoCD as terminal. **Jenkins/Forgejo: ZERO hits in this cluster** (clean here; the Jenkins/Forgejo-as-canonical problem lives in `ideas/`+`products/foundry/` per the footprint, not architecture/).

### A6. Autonomy ceiling enforced "via Cedar policy" — partial tension with canon #8 + #6
The cluster frames the autonomy ceiling as runtime-enforced (canon #8 ✓) but routes enforcement **through Cedar** as the engine (re-triggers A1) and uses the **T1–T4 tier** vocabulary.

| File:line | Claim |
|---|---|
| `adr-cross-reference-graph-2026-05-20.md:110` (ADR-0022) | "Autonomy ceiling — **runtime enforcement via Cedar policy** at every capability invocation" |
| `adr-cross-reference-graph-2026-05-20.md:96` (ADR-0007) | "Cedar policy engine … + persona-tier **autonomy ceiling (T1–T4)** with per-capability runtime enforcement" |
| `adr-cross-reference-graph-2026-05-20.md:166` (ADR-0099) | "Cedar Policy Extension: **Foundry Supervisor** Capabilities at **T1–T4**" |

**Fix:** Runtime-enforced ✓ (canon #8). But (a) enforcement engine must be PARC, Cedar = contract (A1); (b) "Foundry Supervisor" → cloud-intelligence (canon #2); (c) the **autonomy ceiling must be governance-OWNED** (canon #8) — that ownership is not asserted here. `autonomy_tier`/`T1–T4` is a namespaced tier and is **canon-COMPLIANT** (canon #9 keeps namespaced `*_tier`) — do NOT rename to tenant-class.

---

## B. CANON-COMPLIANT (verified false-positives — do NOT amend)

- **`foundry-fitness-to-governance-transition-2026-05-21.md`** (159 "foundry" hits) — this is the **retirement RECORD ITSELF**: it documents renaming `oya-foundry-fitness-*` → `oya-governance-*` (571 files, per ADR-0132 + CLAUDE.md `new_governance_lane_prefix`). This is **canon #2 governance-sense compliance in action**, not a violation. The 159 hits are the old token being inventoried for removal. **CAVEAT:** it only covers the *governance/fitness* sense of foundry; the *intelligence/AI-substrate* sense (ADR-0023 "Foundry sandbox", ADR-0099 "Foundry Supervisor", `products/foundry/`) is **NOT** covered here and remains live (route → cloud-intelligence).
- **`transition-classification-2026-05-21.json`** — the machine-readable classification backing the above; canon-compliant.
- **`autonomy_tier` / `T1–T4` / `capability tier` / `eu_ai_act_risk_tier`** — namespaced `*_tier` vocabulary, explicitly preserved by canon #9. Not a tenant-tier violation.
- **Cloud-Hypervisor/Kata/Firecracker microVM default** — aligned with canon #7 assume-breach microVM default (the gap is only the missing framekernel-host *endpoint*, A4 — not a default-tech violation).
- **Pulsar event backbone (ADR-0005 echo)** — canon-ALIGNED bridge (canon #5). Only the competing Redpanda+NATS framing (A3) is off-canon.

---

## C. AI-SLOP / STALE / PLAIN-WRONG

- **`unified-ecosystem-thesis-2026-05-21.md`** — self-confessed slop in its own `revision_history` (line 57-58): **"v1: clause-loop padded; 7,369 lines, 700 Thesis-clause repetitions"**, v2 "collapse-pass … clause-loop and implementation-note-loop and displacement-clause-loop removed". A `line_floor: 2500` front-matter field (line 9) is fabricated-rigor (mandating minimum length is an anti-pattern that *induces* padding). Status `Proposed`. **Refinement:** verify the v2 collapse actually removed the loops (the 700-repetition admission is a red flag); drop `line_floor`.
- **Fabricated-precision dollar figures** — `unified-ecosystem-thesis-2026-05-21.md:95-101`: "USD 165,000 per employee per year", "USD 38.5 million per year for a 1,000-employee enterprise", "1.5 FTE … USD 500,000". The doc *itself* hedges these as "internal thesis sizing assumptions … legal and procurement must validate" (line 71-74) — so they are flagged-but-unsourced fabricated precision. Acceptable as internal sizing ONLY because explicitly fenced; must not leak to customer-facing.
- **`day-in-the-life-coherent-ecosystem-2026-05-21.md`** — 1.05 MB single markdown file, 862 "Cedar" mentions, status `Proposed`. Not read in full (size); flagged for **size/slop review** — a 1MB narrative is a padding risk and a reachability liability.
- **`enterprise-software-coverage-matrix-2026-05-21.md`** — 2.05 MB, status `Living`, 598 Cedar / 180 foundry. Machine-ish matrix; the 180 "foundry" need the #2 sense-split rename. Flagged for size.
- **Mid-remediation snapshot** — `corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md` is an explicitly transient "mid-remediation" artifact (382 KB) superseded by `corpus-rigor-audit-2026-05-21-post-wave-3-g.md`. **Candidate for archive** (stale-by-construction).

---

## D. REACHABILITY CLASSIFICATION

| Class | Docs | Disposition |
|---|---|---|
| **GENERATED-REFERENCE** (machine-produced indexes; reachable via pipeline) | `adr-cross-reference-graph`, `adr-corpus-line-audit`, `ip-corpus-line-audit`, `ip-cross-reference-sweep`, `microservices-corpus-line-audit`, `standards-corpus-line-audit`, `six-hops-reachability-audit`, `audit-event-coverage-sweep`, `product-graph.{md,html}`, `transition-classification.json` | Keep; these are outputs, term-hits are echoes not assertions. Canon fixes belong in their SOURCES (ADRs). |
| **DECISION-derived → ADR** (doctrine that should be ADR-backed) | `unified-ecosystem-thesis`, `training-cost-doctrine`, `foundry-fitness-to-governance-transition` | Reachable via `related_adrs` front-matter. Doctrine claims (A1 Cedar-engine) must be reconciled with the ADRs they cite. |
| **GENERATED/EPHEMERAL (wave snapshots)** | `wave-3-final-scorecard`, `wave-3-retrospective`, `wave-3-g-executive-briefing-2026-05-20-post-remediation` (superseded by `-2026-05-21`), `wave-3-g-synthesis-adjudication`, `corpus-rigor-audit-2026-05-20*` (superseded by `-05-21`), `keystone-bundle-2026-05-20-*` | Several are **superseded duplicates** → ARCHIVE candidates (05-20 variants where a 05-21 exists). |
| **ORPHAN → not-needed** | `corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md` (explicitly mid-flight) | Archive. |
| **REFERENCE (diagrams, keep)** | `diagrams/*.md` (10) | Keep; but `cedar-policy-evaluation-flow` + `capability-tier-projection-flow` need the Cedar→PARC engine-vs-contract fix (A1). |

---

## E. COUNTS

- **Files reviewed:** 45 (36 top-level + 10 diagrams), of which **deep-read: 9** (thesis, foundry-transition, training-cost, briefing-0521, 2 diagrams, + partial reads), **scanned/grepped: 36** (the large generated audits — honest: NOT fully read; their hits verified as ADR-echoes by line-anchored sampling).
- **Genuine canon-contradictions (cluster's own voice): 6 themes** (A1 Cedar-engine [10 sites], A2 Zitadel-canonical [5 sites], A3 eventing 3-way [11 sites incl. internal contradiction], A4 framekernel-endpoint-missing/foundry-sandbox [3 sites], A5 ArgoCD-canonical [4 ADR-echoes], A6 autonomy-via-Cedar [3 ADR-echoes]).
- **Canon-COMPLIANT false-positives carved out: 5** (foundry-transition record, transition JSON, namespaced `*_tier`, microVM-default, Pulsar backbone).
- **AI-slop / stale / archive candidates: 6** (thesis self-confessed padding, fabricated $ figures, 1MB day-in-the-life, 2MB coverage-matrix, mid-remediation snapshot, 05-20 superseded duplicates).
- **`native-default`/`secure-by-default-native`: 0 hits** — that specific anti-canon phrasing is absent from this cluster (canon #7 not violated by phrasing, only by the missing framekernel endpoint).
- **Jenkins/Forgejo: 0 hits** — clean in architecture/ (the problem is in `ideas/`+`products/foundry/`).

---

## F. TOP-3 FIXES (priority order)

1. **Cedar = CONTRACT, PARC = ENGINE** (A1) — rewrite the "one Cedar policy engine" invariant across thesis/doctrine/briefing + both diagrams; introduce PARC as the owned PDP. *Highest blast radius — PARC is absent everywhere.*
2. **oya-identity owned; Zitadel = bridge** (A2) — strip "authoritative identity = Zitadel" from keystone walkthrough; flag ADR-0187 title "canonical Zitadel primary".
3. **Resolve the 3-way eventing contradiction** (A3) — one canon line: Pulsar bridge (canon #5), reconcile/scope the Redpanda+NATS framing in `keystone-bundle-idea-refine-deep-dive.md`. *Real internal contradiction, not just stale term.*
