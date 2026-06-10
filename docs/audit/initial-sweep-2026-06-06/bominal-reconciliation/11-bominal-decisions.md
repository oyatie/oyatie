---
doc_class: BominalReconciliation
title: Key bominal (oyatie's earlier self) decisions — grouped, with present/weaker/absent flags vs oyatie canon
status: synthesized
date: 2026-06-06
source: /Users/jasonlee/Developer/_recover-bominal/decisions/ (132 ADRs, ADR-0010..ADR-0233)
method: >
  Read every product/vertical/architecture-bearing ADR in full; skimmed the rest by title.
  Each row is a decision atom + present/weaker/absent estimate in the oyatie monorepo.
  Estimates are approximate; the diff phase confirms. Already-canon (decision-record-oyatie-canon.md)
  and already-recovered (00-RECOVERY-REGISTER.md) items are NOT re-surfaced here except where a
  bominal ADR sharpens a known item. INTENT-bearing product/vertical decisions are flagged ★.
legend:
  status: PRESENT | WEAKER | ABSENT  (estimate; oyatie = the live jason931225/oyatie monorepo)
  ★ = encodes product/vertical INTENT (the load-bearing recoveries)
---

# Key bominal decisions (oyatie's own pre-rename history)

## Headline

bominal IS oyatie before the rename + migration. The 132-ADR bominal set is **earlier, more
product-concrete, and more KR-grounded** than the post-migration oyatie canon. The migration kept
the *architecture spine* (Object Graph, hexagonal, RLS, audit-chain, Cedar, tenancy, data-tier,
cloud-native infra, multi-runtime) — those are PRESENT and usually stronger in oyatie. What the
churn LOST is the **product/vertical intent layer**: the specific verticals bominal committed to
(clinical/CDSS, manufacturing/AMR/CCTV, marketplace, Connect, bid-pricing, optimization platform),
the **KR-first regulatory depth** that motivated them, and several **cross-cutting product doctrines**
(persona tiers, data-ownership pillars, Workflow+Object-Graph-as-operating-kernel, Ecosystem-as-a-Service
Bench + industry presets). oyatie's "maximal vertical scope, sequenced" canon (D9) *asserts* breadth
but does not carry bominal's concrete per-vertical specifications — those are the recoveries.

The single most important reframe: in bominal the verticals are **named, specified, and regulator-
grounded** (each with KR statute citations, market benchmarks, model choices, safety gates). oyatie's
canon treats verticals as an abstract in-scope set. **The bominal ADRs ARE the vertical-coverage map
that oyatie task #18 is trying to (re)build.**

---

## GROUP A — Product / Vertical decisions (the INTENT layer — highest recovery value)

| # | Decision atom | bominal ADR | Status in oyatie | Note |
|---|---|---|---|---|
| A1 ★ | **Six business arms** are the portfolio: Healthcare, Corporate SaaS, FinTech, Platform/Ops, Hospitality+Lifestyle, Communications/Connect — each with Tier-0 foothold / Tier-1 wedge / Tier-2+ deepening per sub-vertical | 0203, 0185 | **WEAKER** | oyatie canon D9 says "maximal vertical scope, sequenced" but the *named six-arm taxonomy with per-arm wedge* is not restated. This is the missing vertical-coverage map. |
| A2 ★ | **Healthcare vertical = full clinical platform**: Medical (canonical record authority) + Records (shared data plane) + Pharmacy + Emergency + Healthcare (patient released-view). "One medical domain, multiple surfaces." | 0016, 0011 | **WEAKER** | oyatie has clinical-canonical-record canon, but the 5-surface product split (Medical/Records/Pharmacy/Emergency/patient-portal) + released-view-as-projection + scoped write-authority per surface is not crisply carried. |
| A3 ★ | **CDSS clinical diagnostic assistance**: DDx engine + multimodal imaging (CXR) + EKG + red-flag escalation; self-hosted open-weight medical LLMs (Meditron/Med42, never cloud Med-PaLM for PHID-residency); MFDS "Informing"-tier strategy; HITL non-negotiable; Bominal = data processor not controller | 0137 | **ABSENT** | No oyatie counterpart found. Deep product+regulatory intent (KFDA path, IRB cycles, prospective-validation ship gate, Wong-2021 Epic-Sepsis negative example). Strong recovery candidate. |
| A4 ★ | **Connect = native dual-context comms platform** (Personal vs Professional, strict architectural isolation): symmetric Messenger/Mail/Community; Personal=E2EE user-DEK (Bominal cannot decrypt), Professional=tenant-DEK auditable w/ four-eyes; builds Slack/Discord/Reddit/Teamblind/KakaoTalk/Signal/Telegram patterns natively | 0208, 0215 | **WEAKER** | oyatie has connect dual-context (ADR-0311 personal-vs-work) per recovery register, but the *native-rebuild-of-10-platforms product ambition* + symmetric 3-product model + per-context encryption matrix is weaker/unstated. |
| A5 ★ | **Manufacturing operations AI**: 6 capabilities — probabilistic defect detection, predictive maintenance, fault/root-cause, accountability, workflow optimization, tax/financial optimization; edge inference; KR competitors (SUALAB/MAKINAROCKS) | 0143, 0217 | **ABSENT** | No oyatie counterpart. Manufacturing data model (ISA-95/OPC-UA/OEE/NCR/CAPA) + OT safety boundary also absent. |
| A6 ★ | **AMR + facility intelligence**: 3D SLAM facility mapping, obstacle/hazard detection, autonomous servicing, traffic optimization, pathfinding, storage optimization; ROS2 + edge perception + fleet mgmt; KR robotics precedents | 0142 | **ABSENT** | No oyatie counterpart. Physical-safety-critical robotics vertical. |
| A7 ★ | **CCTV vision pipeline**: 5 escalating-risk stages (motion→object→personnel→facial→identity-match); facial/biometric OFF by default, jurisdiction-gated (BIPA/EU-AI-Act/PIPA Art.23); edge for stages 1-2, centralized for 3-5; encrypted identity gallery | 0141 | **ABSENT** | No oyatie counterpart. Encodes a hard biometric-default-off safety posture. |
| A8 ★ | **Marketplace operating model** + Korean payment integration: catalog/order/payment/settlement; PortOne (IMP) PG aggregator → Toss primary; double-entry settlement ledger; KYB via NTS; 전자상거래법/통신판매업 compliance; B2C/B2B/HR-services orientations open | 0135 | **WEAKER** | oyatie has marketplace (ADR-0249/0314 per recovery register) but the KR-payment-rail specifics (PortOne, Toss, 통신판매업자 registration, double-entry settlement ledger) are not carried. |
| A9 ★ | **Contract Bid Pricing Engine** (platform-wide, all verticals): hard/soft cost model, margin/budget, **live labor-rate integration from corp-pay** (the differentiator vs Procore/Deltek), KR 국가계약법 낙찰하한율 statutory floor enforcement; bid→contract→budget lifecycle | 0115 | **ABSENT** | No oyatie counterpart. Named go-to-market wedge with live-payroll moat. |
| A10 ★ | **QA + Customer Support Ticketing** (platform-wide): Tier-1 internal QA + Tier-2 customer support (Tier-3 CRM deferred); Workflow Studio routing + SLA engine under KR 소비자기본법 first-response windows | 0114 | **ABSENT** | No oyatie counterpart found. |
| A11 ★ | **Tenant-configurable optimization & ML platform** (Workflow Studio extension): tenants bring data+parameter-space+objectives, platform supplies RSM/Bayesian-opt/DOE/Pareto/bandit templates; the "welding sweet-spot" generalization — don't pre-ship per-process models | 0145 | **ABSENT** | No oyatie counterpart. Key "customers encode their company into us" (Foundry-parity) doctrine made self-serve. |
| A12 ★ | **AI surfaces catalog** (~17+ intelligence domains, method-tagged): document understanding, voice/transcription, translation, etc.; "some are pure algorithmic" — OR/MIP/graph before ML where it fits | 0144 | **ABSENT** | Enumeration ADR; the breadth-of-AI-surface intent is unstated in oyatie. |
| A13 ★ | **FinTech arm**: own PG → 전자금융업, quant/asset-mgmt, insurance marketplace, banking-license path (인터넷전문은행), tax filing (종합소득세), BNPL, cross-border FX, crypto custody | 0203, 0120, 0124 | **WEAKER** | oyatie has billing/cost/finops substrate (stronger) but the *FinTech-as-a-product-arm* ladder (PG license → bank license) is not carried as product intent. |
| A14 ★ | **Quant extracted to standalone repo** (`bominal-quant`) — isolated, no inbound deps; finance MATH split into `oya-kernel-finance` library (shared) vs quant (isolated) | 0124, 0120 | **PRESENT** | oyatie canon ratifies own-everything; quant separation likely survives. Finance-library boundary worth confirming. |
| A15 ★ | **Hospitality + Lifestyle arm**: dining, cellar (beverage), POS, retail, hotel PMS, fashion, career marketplace | 0203 | **ABSENT** | Lowest-maturity arm (Discovery tier) but explicitly in-scope; no oyatie trace. |
| A16 ★ | **Bominal Law** = future product on `oya-kernel-legal` substrate: AI-assisted legal exploration (cite source, not legal advice); law/compliance-as-code splits statutory params from executable logic | 0190, 0220 | **WEAKER** | oyatie has legal-corpus governance but "Bominal Law as a product surface" intent may be lost. |
| A17 ★ | **Email/messenger mining** as product: F1 org-pillar (BEC defense, revenue-intel, workforce analytics) BUILD; F2 person-pillar EXCLUSION ZONE (do-not-build default, 통신비밀보호법 1-10yr) | 0136 | **ABSENT** | No oyatie counterpart. The org/person split is canon-adjacent but the *mining product + hard person-pillar exclusion* is product intent. |
| A18 ★ | **User/Org profile architecture**: UserProfile entity + behavioral event pipeline; OrganizationProfile + firmographic enrichment + health_score (feeds marketplace/fraud) | 0138, 0139 | **ABSENT** | Profiling-program product surfaces; no oyatie trace. |

---

## GROUP B — Architecture decisions (the spine — mostly PRESENT, often stronger in oyatie)

| # | Decision atom | bominal ADR | Status in oyatie | Note |
|---|---|---|---|---|
| B1 | **Object Graph** = engine-enforced, cryptographically auditable typed-entity layer (Object/Link/Action/Function types); the "Palantir-Ontology-but-better" architecture; NEVER call it "ontology"; 5 differentiators (engine-RLS, Merkle audit, rule-packs-as-primitives, portable semantics, plugin/multi-renderer) | 0106, 0192 | **PRESENT** | oyatie `ontology` is the canonical data substrate (recovery register confirms). The *anti-Palantir-naming* + 5-differentiator framing may be weaker. |
| B2 | **Workflow + Object Graph = the operating kernel**: all products integrate via adapters/ports/solution-packs/event-contracts; no product duplicates a workflow engine or graph model | 0192, 0148, 0149, 0164 | **PRESENT** | oyatie has workflow-engine + workflow-studio (stronger). The "these two ARE the kernel" doctrine worth confirming as explicit. |
| B3 | **OG-AG (Object Graph Agent Gateway)**: LLM tool surface; LLM can't call tools directly; runs under invoking user's auth; every tool call = Action Type = audit-chain leaf; `principal_kind=llm` recorded | 0107 | **PRESENT** | oyatie has agent-gateway / intelligence-consumes-governance pattern. Confirm `principal_kind` audit tagging survives. |
| B4 | **Hexagonal microservice standard** (repo-wide): sealed ports ≤6 methods, clean-architecture inner-ring layering, no app-crate→app-crate deps, cross-product composition only at server root | 0101, 0100, 0102, 0103, 0105 | **PRESENT** | Foundational; oyatie almost certainly inherits (the std-first/clean-arch posture is in project memory). |
| B5 | **Specialised OG property types** as first-class (not serde_json blobs): Vector(pgvector), Geo(PostGIS), TimeSeries(Timescale), CipherText(KMS-envelope), Struct(schemars) — one sub-ADR each | 0108-0112, 0133 | **PRESENT** | Property-type tiering exists; CipherText/KMS maps to oyatie cloud-kms. |
| B6 | **Audit-chain**: per-tenant daily Merkle root, Ed25519-signed, chained day-to-day, externally verifiable offline; every Action Type → outbox → leaf | 0028, 0106, 0225 | **PRESENT (stronger)** | oyatie canon has Merkle-sealed-Ed25519 audit chain explicitly (recovery register). |
| B7 | **Tenancy + RLS posture**: Postgres FORCE ROW LEVEL SECURITY on every multi-tenant table; `TenantScopedPool::begin()` is the only transaction path; cross-tenant leak structurally impossible | 0018 | **PRESENT (stronger)** | oyatie has tenant-as-universal-scope (ADR-0242/0244) + tenant-class. |
| B8 | **Differential-privacy query gateway** + ε-budget composition (T3 enforcement pathway) | 0134 | **WEAKER/UNKNOWN** | oyatie has a differential-privacy-query-gateway in canon list — confirm ε-budget composition carries. |
| B9 | **Event streaming**: Redpanda→Kafka(gated); outbox poller is day-1 substitute; ClickHouse CQRS read path | 0116(superseded), 0174 | **PRESENT** | oyatie uses Pulsar/eventing (own-endpoint ratchet). Substrate pick differs (Pulsar vs Kafka) but pattern PRESENT. |
| B10 | **Cloud-native infrastructure**: polyglot data tier, service mesh (Istio Ambient pulled to Phase 1), OCI A1→OKE scaling path | 0117, 0184, 0167, 0168 | **PRESENT** | oyatie cloud substrate is the endpoint; mesh/gateway picks are vendored bridges. |
| B11 | **Data tier assignment matrix**: definitive per-workload store selection + OCI managed-service mapping + cloud transition path | 0119, 0175-0182 | **PRESENT (stronger)** | oyatie D4 owns entire data tier via vendored→owned ratchet (canon). bominal matrix = the transition detail. |
| B12 | **Multi-runtime platform standard** + runtime-target metadata model | 0019, 0020 | **PRESENT** | Maps to oyatie multi-runtime + D-META ownership ratchet (ADR-0019 cited in canon). |
| B13 | **Tenant activation + data import**: self-service wizard, multi-format (.xls/.csv/.json/.xml) parsing, LLM-optional entity mapping, mapping templates, row-level errors, rollback | 0118 | **WEAKER** | Concrete onboarding product surface; oyatie tenant-activation (ADR-0118-equiv) may be weaker on the import wizard. |
| B14 | **Form schema standard** — Bench-native typed JSON over Object Graph | 0151 | **UNKNOWN** | Confirm form-schema standard survives. |
| B15 | **Plugin system**: manifest schema + trust tiers + Cosign-keyless signing + Rekor + Wasmtime/WASI-P2 capability-gated sandbox | 0156, 0157, 0161, 0162 | **PRESENT** | oyatie has plugin-trust-tiers/signing/wasm-sandbox in canon list. |

---

## GROUP C — Platform / Infrastructure decisions

| # | Decision atom | bominal ADR | Status in oyatie | Note |
|---|---|---|---|---|
| C1 | **Ecosystem-as-a-Service**: customer-facing category = tenant-owned operating ecosystem (one Bench, one module catalog, one OG, one workflow fabric, one Connect, one policy plane, industry/regional presets); "Core Work/Business MVP" preset is the first wedge | 0121, 0191 | **WEAKER** | Recovery register row #4 already flags EaaS framing + self-tenant invariant as WEAKER (fold-into-doc). bominal 0121 is the canonical source for it. |
| C2 | **Bench** = unified host surface (renames "shell"); owns login + module activation; modules mount per tenant module-graph; no per-module credential login | 0130, 0121, 0123 | **WEAKER/ABSENT** | The "Bench" naming + module-shell-owns-session contract is product-shaped; confirm oyatie carries an equivalent host surface concept. |
| C3 | **Rust-first platform sovereignty** with performance-gated replacement: incumbents stay certified until first-party Rust replacement clears its own gates; nothing displaced because "it's ours" | 0150 | **PRESENT (stronger)** | This IS the oyatie own-everything-ratchet (D-META: own-endpoint/vendor-bridge/ratchet-when-proven). bominal 0150 is the seed. |
| C4 | **Time-horizon delivery model**: end-state strict-best-no-compromise; high-migration-cost work lands early (even if consumer surface gated later); each milestone gets stage-right tools; Phase1=corp+connect, health/industry milestone-gated | 0185, 0210 | **WEAKER** | oyatie has time-horizon-delivery (ADR-0185-equiv) + sequencing canon (D8/D9) but the explicit 3-principle phasing rule may be weaker. |
| C5 | **M3 launch scope** = KR group payroll + corporate Mail production; ≥1 paid KR group customer (~3000 wage employees) closes real payroll before M3 done; payroll-first public claim | 0210 | **WEAKER/ABSENT** | Concrete first-customer go-to-market target. Maps loosely to recovery register's "First Proof Slice" gap. |
| C6 | **Client stack policy**: Leptos web + native platform clients (per-native split: SwiftUI/Kotlin/WinUI3/GTK4-rs); React/Compose-Multiplatform retired | 0209, 0201, 0205 | **WEAKER/UNKNOWN** | Confirm oyatie client-stack policy (Leptos + per-native). |
| C7 | **Multi-platform / multi-form-factor**: iPad/macOS/watchOS/visionOS/WearOS/AndroidXR/CarPlay/Auto/ChromeOS roadmap; additive shared-code structure | 0202 | **ABSENT/UNKNOWN** | Forward-looking form-factor roadmap; likely not carried. |
| C8 | **Hybrid on-prem + cloud compute**: workstation-first ML/AI (RTX 5080 dev ceiling), cloud-default for everything else; "PHI never leaves the building" for early CDSS | 0147 | **WEAKER/ABSENT** | Maps to oyatie hybrid-onprem-cloud (ADR-0147-equiv). Confirm workstation-first ML posture survives. |
| C9 | **IaC profiles** multi-cloud + on-prem, air-gap-first shared plane (OpenTofu); HIPAA-shape from day zero; KR-OCI now / US-OCI planned / customer on-prem | 0233, 0224, 0021, 0183 | **PRESENT (stronger)** | oyatie cloud-iac + sovereign/air-gapped (D9 raises air-gapped CI/isolation reqs). |
| C10 | **At-scale stack ADRs** (gated substrates behind ports): Kafka, Cassandra, TiDB/Vitess, Milvus, Mimir/Loki/Tempo, Temporal, OpenSearch, Iceberg — each with a day-1 substitute | 0174-0182 | **PRESENT (stronger)** | Exactly the oyatie vendored→owned ratchet (D4). |

---

## GROUP D — Governance / Cross-cutting doctrine decisions

| # | Decision atom | bominal ADR | Status in oyatie | Note |
|---|---|---|---|---|
| D1 ★ | **Persona tier model T1/T2/T3/T4**: marketable / collectable-non-marketable / anonymous-aggregate / non-collection; Kantara consent receipts; Cedar policy gateway; T4 enforced at ingest (zero events) | 0131 | **WEAKER/ABSENT** | NOTE: oyatie `tier` is namespaced (D12) into autonomy_tier/eu_ai_act_risk_tier/etc — bominal's *persona/data-collection* T1-T4 may have been lost in that namespacing. High-value: this is the platform-wide data-classification axis. |
| D2 ★ | **Data-ownership pillars** Org vs Person, **cross-pillar join prohibition** at policy layer; immutable `ownership_pillar` on every event/OG-property/feature-row/consent-receipt/audit-entry; worker-rights override (PIPA 22-2/37) | 0132 | **WEAKER** | Recovery register notes tenant-as-universal-scope present, but the *org/person pillar discriminator + cross-pillar prohibition + worker-rights override* is a distinct security invariant worth confirming. |
| D3 | **Cedar as policy engine**: per-tenant bundle, per-request Decide API, in-process (chosen over OPA's Go-only out-of-process Rego) | 0131, 0140 | **PRESENT (stronger)** | oyatie D6: Cedar = permanent external CONTRACT, own evaluation engine (PARC). bominal seeds the Cedar choice. |
| D4 ★ | **Multi-jurisdiction policy** KR/EU/US/CN/+: layered Cedar bundles (global→jurisdiction→tenant→receipt); per-jurisdiction data-residency; cross-border transfer mechanically gated (SCC/BCR/adequacy); sectoral overlays (HIPAA/GDPR/MFDS/EU-AI-Act); strictest-wins | 0140 | **WEAKER** | oyatie has multi-jurisdiction-policy + sovereign-per-regional-pack (D9: 0240). The KR-primary + strictest-wins + cross-border-gate mechanics worth confirming. |
| D5 ★ | **KR-first regulatory grounding** is the DNA: 근로기준법, PIPA, 통신비밀보호법, MFDS/의료법, 국가계약법, 4대보험, KCMVP (ARIA/LEA not bare AES), 52시간제 — every vertical cites KR statute | 0104, 0126, 0127, 0136, 0137, 0140, 0190 | **WEAKER** | oyatie data-tier-own-all + compliance packs exist, but the *KR-home-market-first* posture + specific statute depth is the most churned context (recovery register #1 KR HR/payroll packs already flagged). |
| D6 ★ | **Employment classification** (8-class KR taxonomy: 정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원) driving payroll/4대보험/leave/52h/severance/withholding-stream | 0126, 0127 | **WEAKER/ABSENT** | Maps to recovery-register #1 KR HR/payroll packs. The typed `EmploymentClassification` enum is concrete lost product context. |
| D7 | **Domain naming canon**: Tenant/Organization/User/Person/Employee/Employment/EmploymentClassification | 0125 | **PRESENT** | oyatie has domain-naming-canon (D-series). Confirm the Person-vs-User-vs-Employee split survives. |
| D8 | **Isolation-compatible operating model**: shared runtime OK now, isolatable from day one; **unit of isolation = customer × domain**; dedicated = separate cluster+db+network+keys; shared auth+observability control planes | 0011 | **PRESENT (stronger)** | oyatie D7 framekernel + Capsule + assume-breach microVM is the endpoint; bominal customer×domain = the seed. |
| D9 | **Trust framework foundation**: 5-class data vocabulary (public/internal/confidential/restricted/regulated), audit-event standard, evidence ledger, break-glass, GDPR-style privacy-rights workflows | 0225 | **PRESENT (stronger)** | Accepted+promoted in bominal; maps to oyatie trust/governance. |
| D10 | **Product Control Plane**: capabilities, entitlements, topology, metering as first-class architecture | 0226 | **PRESENT (stronger)** | oyatie has oya-meter/cost/billing (recovery register, stronger). |
| D11 | **Data + AI governance**: semantic metrics, knowledge packs, AI context routing, lineage, synthetic data | 0228, 0219 | **PRESENT** | oyatie data-ai-governance canon. AI/ML governance umbrella (0219) for regulated+operational intel. |
| D12 | **Builder Operating System**: ownership registry, 5-class decision rights, 10-class PR taxonomy, 7 council charters, agent operating model (agents never self-approve, security-reviewers read-only, pause at approval-required actions) | 0229 | **PRESENT** | Strongly matches oyatie's multi-agent operational protocol + founder's separate-verifier-lane rule. |
| D13 | **Portfolio & Capital Allocation Plane**: investment theses, L0-L7 maturity gates, kill criteria, launch readiness, parked-tracks (allowed/forbidden/promotion-requires), quarterly reviews | 0231, 0223 | **PRESENT (stronger)** | Maps to oyatie portfolio governance + Proof Ladder. The day-0 capacity-budget question (oyatie D8, still owed) is exactly bominal's portfolio-sequencing concern. |
| D14 | **Proof Ladder** — 8-rung product-readiness model (L0..L7) | 0223 | **PRESENT** | Maps to oyatie proof-ladder / readiness gates. |
| D15 | **Evolution & Simplification Plane**: lifecycle, deprecation, fitness functions, complexity budgets | 0230 | **PRESENT** | Matches oyatie fitness-lane CI doctrine. |
| D16 | **Ecosystem integration plane**: contract-first APIs, webhooks, connectors, SDKs, MCP, sandbox, deprecation policy | 0227 | **PRESENT** | Maps to oyatie ecosystem-integration-plane. |
| D17 | **Multi-agent operational protocol** + universal PR traceability/accounting + GitOps best-practice baseline | 0187, 0206, 0207, 0022 | **PRESENT (stronger)** | Matches oyatie governance CI lanes + founder verify-each-step rule. |
| D18 | **Operational Intelligence layer** over Workflow + Object Graph | 0221 | **PRESENT/UNKNOWN** | Confirm OI layer survives. |

---

## What the diff phase should confirm first (ranked by intent-loss risk)

1. **The six-arm vertical map + per-arm wedge (A1)** — this IS oyatie task #18's deliverable, already authored in bominal 0203/0185. Recover before re-inventing.
2. **The named, regulator-grounded verticals that appear ABSENT (A3 CDSS, A5 manufacturing, A6 AMR, A7 CCTV, A9 bid-pricing, A11 optimization platform, A12 AI-catalog, A17 mining, A18 profiles)** — these are the concrete product specs the churn most likely dropped. oyatie D9 "maximal scope" asserts them in principle but carries none of the specification.
3. **Persona tier model T1-T4 (D1)** — at risk because oyatie namespaced `tier` (D12) away from the data-collection axis; the platform-wide consent/collection classification may be orphaned.
4. **Data-ownership pillars + cross-pillar prohibition (D2)** and **KR-first regulatory DNA (D5/D6)** — the most churned context per the legacy recovery (#1 KR packs); confirm the org/person invariant and KR statute depth.
5. **Ecosystem-as-a-Service + Bench host surface (C1/C2)** — recovery register already flags EaaS framing WEAKER; bominal 0121/0130 is the authoritative source to fold back.
6. **Connect native-rebuild ambition (A4)** and **Marketplace KR payment rails (A8)** — present-but-weaker; the *product ambition* (not the mechanism) is what churned.

## Caveats

- Status flags are **estimates** keyed off the oyatie canon doc + legacy recovery register, not a fresh read of the live oyatie monorepo. The diff phase confirms each.
- "PRESENT (stronger)" means oyatie has advanced past bominal on the architecture spine (expected — oyatie is the later, own-everything stage).
- bominal ADR IDs (0010-0233) collide with the oyatie ID space; under oyatie's ID-discipline ruling (canon D13) any recovered decision re-enters as a clean ADR-0000+ record, not by bominal's old number.
- The ~40 substrate/stack/CI/mobile-parity ADRs not deep-read here are almost all "PRESENT (stronger)" spine items already absorbed by oyatie's own-everything-ratchet; the recovery value concentrates in Group A + the starred Group D rows.
