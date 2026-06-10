---
doc_class: LegacyRecoveryRegister
title: Recovery-Candidate Register — legacy "oyatie" docs portal vs LIVE source catalog
status: synthesized
date: 2026-06-06
inputs:
  - 10-legacy-inventory.md   # trashed legacy portal (56 JSON, architecture/docs only, no runtime)
  - 20-source-catalog.md     # LIVE oya/+cloud/ monorepo (87+25 dirs, flat catalog, ADR-0001..0514)
method: >
  For every named legacy product / capability / decision / source-gap, find its counterpart
  in the live source catalog. Classify source status and recommend a disposition. Lead with
  genuine MISSING product context; demote already-covered and noise.
legend:
  source_status: absent | weaker | renamed-equivalent | already-covered
  recommendation: RECOVER-as-ADR | FOLD-into-product-doc | ALREADY-COVERED | LEGACY-NOISE
---

# Recovery-Candidate Register

## Headline finding

The legacy "oyatie" portal and the live "oyatie" monorepo are the **same project at two
maturity stages**: legacy = a 56-JSON *architecture-only, zero-runtime* foundation deck
(ADR-0001..0031 accepted, 0032..0060 planned); source = a flat, 87+25-dir monorepo with
ADRs to 0514 and real CI gates. **The live source SUPERSEDES the legacy on almost every
axis** — most legacy planes/substrates exist in source under the same or a renamed identity,
and the live decision space (514 ADRs) has already absorbed the legacy's "planned ADR" band
(0032-0060) and gone far beyond it.

So the high-value recoveries are NOT whole missing planes. They are **a small set of
specific, named product-context details and deliberate intents** that the legacy spelled out
and the live catalog either dropped, weakened, or never re-stated. Everything else is
already-covered or noise. Below, MISSING / WEAKER items lead; ALREADY-COVERED and NOISE follow.

---

## TIER 1 — Genuinely MISSING or WEAKER product context (high-value recoveries)

| # | Item | What it is | Legacy file | Source status | RECOMMENDATION | Why |
|---|---|---|---|---|---|---|
| 1 | **KR Enterprise/Corporate Workflow Editor + 8 named HR/payroll packs** | The legacy's ONLY concrete app-level module: a governed workflow editor/studio + approval inbox + KR-localized packs (청구서 invoice-claim, 근태 attendance, 급여대장 payroll-ledger, 지급명세서 wage-statement, 지급내역 payment-history, 근로자 등록 worker-registration, 근로자 관리 worker-management, 연차 annual-leave) — explicitly **KR-only market scope**. | enterprise-corporate-workflow-module.json | **weaker** — source has the *capability* spread across `workflow-studio` + `hr` + `payroll` + `workflow-engine`, but NO catalog entry binds them into this named KR HR/payroll **activation bundle**, and the **8 specific Korean packs are not enumerated anywhere** in source. The KR regional pack exists ("full") but does not carry these workflow templates. | **RECOVER-as-ADR** (or FOLD into the KR regional-pack doc) | This is the most concrete go-to-market wedge the project ever named and the legacy's designated first proof slice (SLICE-001 / REQ-MOD-001). Under ADR-0316's "capability-tier = activation bundle" doctrine it is exactly a bundle spec (Cedar permits + workflow templates + compliance overlay). The 8 pack names + KR scope are load-bearing product context not reconstructable from the flat tree. |
| 2 | **First Proof Slice (SLICE-001) end-to-end seam definition** | A single end-to-end reference flow binding identity/tenancy + cloud resource catalog + the KR enterprise workflow module, chosen to prove the platform "seams" (ADR-0006). Defined the exact first vertical to build and what it must exercise (identity, tenant model, control-plane lifecycle, audit/evidence, deployment path, CI/CD, observability, security/policy gates). | first-proof-slice.json, service-taxonomy.json | **absent** — source has 514 ADRs of breadth doctrine but **no equivalent "this is the first end-to-end slice we build to prove the seams" artifact**. The legacy ADR-0006 ("first proof slice") has no live counterpart; live ADR-0006 is the ontology substrate instead. | **RECOVER-as-ADR** | The single most useful planning artifact for a greenfield: it converts unbounded breadth into one buildable vertical. Its absence in source is a real gap given source is still mostly scaffolds. High-value as a delivery-sequencing decision. |
| 3 | **Source-gap / source-requirement closure model (4-lane: planning/implementation/evidence/launch) + 12 capability packs + false-closure validators** | Legacy's explicit honesty instrument: a matrix tracking each requirement across 4 closure lanes, self-reporting **0 of 12 fully closed at runtime**, plus a CLEAN-ROOM-CLOSURE pack with "false-closure validators." Two packs (FIRST-DELIVERY-GOVERNANCE, CLEAN-ROOM-CLOSURE) self-flagged "needs stronger detail." | source-gap-matrix.json, source-requirement-closure.json | **absent** — source has `governance` (~50 `oya-check-*` CI lanes) and `no-grouping` gates, but **no equivalent multi-lane closure ledger** that separates planning vs implementation vs evidence vs launch, and no "false-closure validator" concept. CI gates check fitness, not claim-truthfulness. | **RECOVER-as-ADR** | This is the legacy's most distinctive governance idea and directly matches the founder's standing "verify at each step / no phantom findings / separate verifier lane" rule (project memory). The anti-false-closure validator is a genuinely novel control worth preserving. The two self-flagged-weak packs are the legacy's own admission of where detail is owed. |
| 4 | **"Ecosystem-as-a-Service" product framing + self-tenant rule** | Coined positioning: cloud primitives + managed modules + B2C products + customer workloads all on ONE tenant-aware substrate; **"Oyatie is itself a tenant" with NO internal bypass** (THESIS-001, ADR-0002/0003, REQ-PLAT-001/003). | glossary.json, platform-foundation-prd.json, adr-ledger.json | **weaker** — source enforces tenant-as-universal-scope (`tenancy` ADR-0242/0244, tenant-class ADR-0330) and flat-catalog cohesion (ADR-0001), but the **"Ecosystem-as-a-Service" name and the explicit no-internal-bypass self-tenant invariant are not stated as a live product thesis**. The mechanics survive; the framing + invariant do not. | **FOLD-into-product-doc** | The substrate already realizes it, so not an ADR — but the self-tenant / no-bypass invariant is a security-relevant rule worth restating in the cohesion/tenancy doc so it is not silently lost. The marketing name is optional. |
| 5 | **Trust Plane: 9 named KMS key classes + break-glass + trust-evidence-plane + named residual risks** | Centralized trust plane enumerating **9 KMS key classes**, SPIFFE-style workload identity (SVID-like), trust-root registry, break-glass recovery, a dedicated trust-evidence-plane, plus explicitly logged residual risks (KMS/HSM/vault product selection deferred; SCIM provisioning unspecified; hardware-backed custody varies by sovereign region). | trust-plane-architecture.json | **weaker** — source splits this across `cloud-kms` (CMK/KEK/DEK, HSM, cryptoshred, signing), `cloud-secrets` (OpenBao), `identity`/`oya-identity` (WebAuthn/passkey), and `oya-authn-device-firmware`. Strong on mechanism, but the **"9 key classes" taxonomy, the unified trust-evidence-plane concept, and the named residual risks (esp. SCIM gap) are not enumerated**. | **FOLD-into-product-doc** | Mechanisms are covered, so not a new ADR — but the 9-key-class taxonomy and the SCIM-provisioning-unspecified residual risk are concrete details that cost real analysis to regenerate. Fold into the cloud-kms / identity docs as a checklist. |
| 6 | **Economic substrate: pricing/invoicing/tax/payments deliberately DEFERRED "until usage facts are trustworthy"** | A sequencing *decision*: meter/quota/usage-evidence first; pricing, rating, invoicing, tax, GL, settlement explicitly deferred until metering is trustworthy (ADR-0014; planned ADR-0034 commercial lifecycle). | billing-finops-substrate.json, foundation-adr-roadmap.json | **renamed-equivalent but the DEFERRAL RATIONALE is weaker** — source has `oya-meter`, `oya-cost`, `oya-billing` (ADR-0478/0479/0480), `cloud-billing` + `cloud-billing-tax`, `finops-portal`. Commercial mechanism is **more advanced** than legacy. But the explicit **"don't build pricing/invoicing until usage facts are trustworthy" sequencing principle** is not visibly restated. | **FOLD-into-product-doc** | Capability is already-covered and ahead of legacy; only the deferral *rationale* is worth a sentence in the metering/billing doc to prevent premature invoicing work. |
| 7 | **Canonical 8-stage delivery artifact chain** | ADR Portfolio → Capability Translation Brief → PRD → Technical SPEC → Control+Evidence Mapping → Implementation Plan → Build/Verify/Certify → Launch/Operate/Improve. The legacy's prescribed pipeline from decision to launch. | capability-translation-brief.json, platform-foundation-prd.json | **absent** — source has ADRs + per-product PRDs + CI gates but **no single artifact stating the canonical decision→launch chain**. | **FOLD-into-product-doc** | Lightweight but useful process scaffolding; one diagram in a docs README. Not an ADR-worthy decision on its own. |

---

## TIER 2 — Already covered in source (renamed / absorbed / equal-or-stronger) — recover NOTHING

| Legacy item | Source counterpart | Source status | Recommendation | Why |
|---|---|---|---|---|
| Oyatie Policy Engine (Cedar kernel + Zanzibar graph) | `policy` / policy-engine, Cedar v4.x (ADR-0150/0243/0246), `feature-flags`/`oya-flags` Cedar fragments | already-covered (stronger) | ALREADY-COVERED | Live Cedar evaluation substrate; legacy ADR-0013 ≈ live policy ADRs. |
| Oyatie Economic Substrate (meter/quota/usage/FinOps) | `oya-meter`, `oya-cost`, `oya-billing`, `cloud-billing`, `cloud-billing-tax`, `finops-portal` | already-covered (stronger) | ALREADY-COVERED | Bespoke-Rust metering/billing/cost is well past legacy's docs-only state. (Deferral rationale → row 6.) |
| Oyatie Trust Plane | `cloud-kms`, `cloud-secrets`, `identity`/`oya-identity`, `oya-authn-device-firmware` | already-covered | ALREADY-COVERED | Mechanism fully present. (Key-class taxonomy + SCIM residual → row 5.) |
| Oyatie Resource Control Plane (envelope/LRO/reconcile/finalizers) | Resource lifecycle via ontology + workflow-engine + ADR-0145 contract reform; CAP-PACK-RESOURCE-LIFECYCLE | already-covered | ALREADY-COVERED | Universal envelope/lifecycle realized in substrate spine. |
| Oyatie Traffic Plane (gateway/egress/resilience) | `api-gateway` (Envoy/Cilium, ADR tier-0 edge) | renamed-equivalent | ALREADY-COVERED | Traffic plane = api-gateway substrate. |
| Oyatie Network Fabric (VPC/IPAM/SG/DNS) | `cloud-network`, `cloud-network-dns` | already-covered | ALREADY-COVERED | First-class network substrate present. |
| Oyatie Compute Substrate (VM/host/K8s) | `cloud-compute`, `cloud-k8s`, `cloud-capacity`, managed-k8s-* | already-covered | ALREADY-COVERED | Compute + managed-K8s product surface (ADR-0376). |
| Oyatie Storage Substrate (block/object/file/snapshot) | `cloud-storage`, `cloud-data` | already-covered | ALREADY-COVERED | Storage primitive present (some crate-only). |
| Oyatie Eventing/Messaging Substrate | `eventing` (ADR-0005 outbox; crate-only scaffold) | weaker-but-present | ALREADY-COVERED | Present as scaffold; legacy adds no detail source lacks. |
| Oyatie Workflow Orchestration Substrate | `workflow-engine` (Step-Functions-class, ADR-0035/0145) + `workflow-studio` UI | already-covered (stronger) | ALREADY-COVERED | Durable orchestration substrate + visual editor both live. |
| Oyatie Metadata/Consistency Substrate (lease/lock/election/watch) | `ontology` (canonical data substrate) + workflow-engine coordination | renamed-equivalent | ALREADY-COVERED | Metadata/consistency concerns absorbed into ontology + engine. |
| Oyatie Reliability/Observability Plane (SLO/error-budget/incident) | `observability` (OTel/Mimir/Loki/Tempo/Grafana, ADR-0042/0383), `incident-management`, `ops-dashboard-control-center` | already-covered (stronger) | ALREADY-COVERED | Observability spine + incident product surface live. |
| Oyatie State/Data Plane (classification/backup/residency) | `cloud-data`, `cloud-storage`, `compliance` residency packs, `consent-graph` | already-covered | ALREADY-COVERED | Data classification/residency via compliance + consent substrates. |
| Oyatie Capacity Admission Plane (quota/placement/autoscale/preempt) | `cloud-capacity`, `managed-k8s-tenant-quota`, capacity model | already-covered | ALREADY-COVERED | Capacity/quota substrate present. |
| Oyatie Artifact Supply Chain (SLSA/Sigstore/SBOM) | `ci-controller`, `ci-tide`, `ci-webhook-gateway`, `cloud-iac`, CAP-PACK-CICD-SUPPLY-CHAIN; ADR-0511/0513 | already-covered (stronger) | ALREADY-COVERED | Prow-in-Rust CI/CD + supply-chain gates exceed legacy. |
| Control-Plane / Cell architecture (blast-radius cells) | `cell-lifecycle`, `cell-rebalancer` (ADR-0276/0351); cell-as-pattern (ADR-0333) | renamed-equivalent | ALREADY-COVERED | Cell µservice retired to a pattern; lifecycle/rebalancer carved into cloud. Legacy "cell" = absorption, not deletion. |
| Org/Tenant/Resource Hierarchy | `tenancy`/`tenant-rbac`, `cloud-tenancy`, tenant-class (ADR-0330), conglomerate hierarchy (ADR-0313) | already-covered (stronger) | ALREADY-COVERED | Tenancy + tenant-class + sovereign-children hierarchy live. |
| `oya` Rust monorepo toolchain (verify/container-validate/promotion) | `governance` ~50 `oya-check-*` lanes + oya-ci + ADR-0511/0512 monorepo | already-covered (stronger) | ALREADY-COVERED | The `oya` verifier doctrine is realized as live CI fitness lanes. |
| Capability Translation Brief — 8 P0 capabilities (CAP-ID-TENANT … CAP-OPERATE-RELIABILITY) | Mapped 1:1 onto live substrates + CAP-PACKs | already-covered | ALREADY-COVERED | The 8 P0 capabilities are the substrate spine; nothing missing. |
| Resource catalog ~100 primitive types (region/vpc/vm_instance/event_bus/…) | Realized across cloud-* substrates + ontology object types | already-covered | ALREADY-COVERED | Primitive taxonomy is implementation detail now owned by live substrates. |
| Identity/tenancy/context primitives (tenant kinds, principal contexts) | `identity`, dual-tenant identity (ADR-0311), tenant-class | already-covered (stronger) | ALREADY-COVERED | Personal-vs-work boundary (ADR-0311) ⊇ legacy principal-context model. |
| Service taxonomy (5 categories) + reference microservice roots | Superseded by flat-catalog doctrine (ADR-0058/0362) — categories are sales labels only | renamed-equivalent | LEGACY-NOISE | Live doctrine explicitly RETIRES taxonomy/grouping as architecture. Recovering it would contradict ADR-0362. |
| Planned ADRs 0032-0060 (edge/commercial/marketplace/social/industry/search/sustainability…) | Almost all realized: ADR-0249/0314 marketplace, 0029/0234-0238 social/connect, 0315/0321 industry, 0030/0031/0046/0047 search/ads, 0376 managed-K8s, 0034→cloud-billing, etc. | already-covered (stronger) | ALREADY-COVERED | The live 514-ADR space has absorbed and surpassed the legacy's planned band. Only sustainability/carbon (0060) has no obvious live ADR — low value, noise-tier. |

---

## TIER 3 — Legacy noise / superseded (do NOT recover)

| Legacy item | Why it is noise |
|---|---|
| 5-category service taxonomy + 6 named reference service roots (`services/identity/` etc.) | Directly contradicted by the live flat-only, no-grouping doctrine (ADR-0058/0131/0132/0362). Recovering it would reintroduce a retired architecture. |
| Largest unread legacy files (implementation-plan 161KB, foundation-readiness-matrix 117KB, foundation-adr-roadmap 111KB, traceability-matrix, doc-graph) | Process/traceability scaffolding for a zero-runtime deck; their product surface is already captured by the ADR titles + capability packs above. The live repo has its own governance lanes. |
| Clean-room provenance boilerplate (`drafting_policy: clean-room-from-reference` on every file; 5 SRC-FAM source families "adapted not copied") | Process hygiene, not product context. Live repo has its own clean-room posture; nothing to recover. |
| Authority model (README=entrypoint, JSON=SSOT, HTML=render) | Legacy portal mechanics; irrelevant to the live monorepo. |
| Benchmarked-against list (Cedar/Zanzibar/OPA/FOCUS/OpenCost/SPIFFE/SLSA/Sigstore/SAP/ServiceNow…) | Already reflected in live ADRs (Cedar, OpenCost→oya-cost, etc.). No new info. |
| Stack baselines (Rust 1.95/Node 24.16/OpenTofu 1.12/Podman 5.8/Jenkins/ArgoCD) | Live repo has moved on (ADR-0511 Argo supersedes Jenkins; bespoke SCM). Legacy versions are stale. |
| Open questions OQ-002/003/004 (first B2C product? first B2B wedge? which IdP/policy tech?) | Answered by live source: IdP/policy = `identity`+`policy`(Cedar)/`oya-identity`; B2B/B2C wedge = the full product catalog + connect dual-context. Stale questions. |

---

## Counts per recommendation

| Recommendation | Count | Items |
|---|---|---|
| **RECOVER-as-ADR** | **3** | #1 KR enterprise workflow module + 8 HR/payroll packs; #2 First Proof Slice (SLICE-001) seam; #3 4-lane source-closure model + false-closure validators |
| **FOLD-into-product-doc** | **4** | #4 Ecosystem-as-a-Service / self-tenant invariant; #5 Trust-plane 9 key-classes + SCIM residual; #6 economic-substrate deferral rationale; #7 canonical 8-stage delivery chain |
| **ALREADY-COVERED** | **24** | All 18 owned planes/engines + oya toolchain + 8 P0 capabilities + resource catalog + identity primitives + planned ADR band 0032-0060 (see Tier 2) |
| **LEGACY-NOISE** | **8** | 5-category taxonomy + 6 service roots; large process files; clean-room boilerplate; portal authority model; benchmark list; stale stack baselines; stale open questions; sustainability ADR-0060 |

**Net:** of ~40+ distinct legacy items, only **7 carry recoverable product context** (3 ADR-worthy, 4 fold-into-doc). The remaining ~32 are already-covered (and usually stronger) in the live source, or are superseded process/portal noise. The legacy's own source-gap-matrix honesty (0/12 closed at runtime) is itself one of the recoveries (#3).
