---
doc_class: BominalReconciliation
title: Bominal product + vertical + roadmap picture (oyatie's own pre-rename history)
status: synthesized
date: 2026-06-06
premise: >
  oyatie WAS bominal; the rename + migration churned/lost context. This recovers
  oyatie's OWN product/vertical/roadmap surface from the bominal clone
  (~/Developer/_recover-bominal) + the live GitHub milestone graph
  (jason931225/bominal, 54 milestones, state=all). This is recovery of own
  history, not adoption of a foreign repo.
inputs:
  - /Users/jasonlee/Developer/_recover-bominal/modules/<m>/README.md + AGENTS.md (16 modules)
  - /Users/jasonlee/Developer/_recover-bominal/portfolio/strategy/*.md (theses, moat, sequencing, dependency-map)
  - /Users/jasonlee/Developer/_recover-bominal/.planning/{ROADMAP,MILESTONES,REQUIREMENTS,PROJECT}.md
  - /Users/jasonlee/Developer/_recover-bominal/product-control/{capabilities,entitlements,lifecycle,metering,topology}/README.md
  - gh api repos/jason931225/bominal/milestones --paginate state=all  (54 milestones)
  - gh api repos/jason931225/bominal/issues per milestone (capability/build-unit/research issue titles)
relation_to_prior_work: >
  Does NOT re-surface canon decisions (decision-record-oyatie-canon.md) nor
  duplicate the 7 legacy-recovery items (00-RECOVERY-REGISTER.md). Those 7 came
  from the trashed JSON portal; THIS doc comes from the live bominal monorepo +
  GitHub roadmap, which the register did not read.
---

# Bominal product + vertical + roadmap picture

## 0. Headline

Bominal (= oyatie pre-rename) is a **single tenant-aware platform substrate** that fans
out into an explicitly **maximal vertical portfolio**, sequenced as a multi-year ratchet:
near-term KR-corporate + clinical + logistics/industrial wedges (2026), then health-AI /
marketplace / vision / robotics (2027), then conglomerate + capital-markets + geospatial
(2028), and a deliberately-gated **far-future tier (2029–2030): facilities/data-center,
agriculture, public-sector, civil-infrastructure + utilities (POWERGRID), and public-safety
+ drones + DEFENSE**. Running underneath, in parallel, is the **infrastructure-sovereignty
ratchet** (ADR-0150): a staged "own the substrate" program (gateway, KMS/secrets, mail,
cache, stream, DB, storage) gated on M0 contract+benchmark evidence before any replacement.

Two governance spines hold it together: the **product-control plane** ("no product behavior
enabled by accident" — capability → module → entitlement → policy → topology → metering) and
the **portfolio plane** (Proof Ladder L0–L8 rungs + five compounding moats + quarterly
kill/fund review). Every far-future milestone is shaped identically: `research (benchmark
matrix) → PHASE M0 (contract+benchmark+evidence locked) → doc(adr) operating contract → EPIC`,
i.e. nothing high-risk ships before an evidence gate.

---

## (a) Product / module inventory

Source: `modules/<m>/README.md` + `AGENTS.md` in the clone. Each module is a Rust (Axum +
Postgres) service crate set; web surfaces standardize on SvelteKit per ADR-0209 (React/Vite
trees are legacy, scheduled for deletion). 16 module roots:

| Module (dir) | Product name | Purpose | Vertical / arm |
|---|---|---|---|
| `hr` | **Bominal Corporate** | Multi-tenant enterprise SaaS: workforce/HCM, ATS, performance, payroll, procurement, AP/AR, accounting, tax, governance/GRC, contracts, documents/e-sign, SCIM. The KR-payroll wedge. | Corporate |
| `workflow` | **Bominal Workflow** | Work-graph / transitions / automation bindings / solution packs. Transitional — folds INTO Corporate (not a standalone arm). | Corporate (platform→corp) |
| `payments` | **Bominal Payments** | Payment processing, invoicing, reconciliation, PCI-DSS. Settlement substrate every other arm bills against. | Payments (substrate) |
| `insurance` | **Bominal Insurance** | Claims lifecycle, adjudication, member eligibility, benefits coordination; integrates Medical/Records/Payments + external payers. | Insurance / Clinical-adjacent |
| `connect` | **Bominal Connect** | Shared suite shell for Mail + Messenger + Community (directory/search/activity/share-intents); first-class web + desktop (win/mac/linux) + iOS/Android client roots. | Social / Communications |
| `mail` | **Bominal Mail** | Mail + calendar product (SMTP/IMAP/JMAP target, M365/Workspace coexistence, retention/eDiscovery/DLP/S-MIME planned). Corporate-Mail replacement pulled into M3 (ADR-0210). | Communications |
| `messenger` | **Bominal Messenger** | Standalone privacy-first messaging (DMs/groups/attachments/calls/realtime). Trust surface = `ExternalPrivacyFirst`. | Communications |
| `community` | **Bominal Community** | TeamBlind-like verified-company community (pseudonymous spaces/boards, moderation, unmask-audit). | Social |
| `medical` | **Bominal Medical** | Provider-facing hospital/clinic suite; **canonical write authority** for clinical record (charts, encounters, CPOE, labs, meds, discharge, DUR/formulary). | Clinical (system of record) |
| `patient` | **Bominal Healthcare** | Patient-facing **released-view** projection of the Medical record (appointments, results, meds, billing, consent, telehealth-planned). NOT a second system of record. | Clinical (patient surface) |
| `pharmacy` | **Bominal Pharmacy** | External/community pharmacy ops (intake, dispense, fulfillment, inventory, claims). In-hospital meds stay in Medical. | Clinical (pharmacy) |
| `emergency` | **Bominal Emergency Services** | Emergency routing/intake orchestration (hospital search, capacity match, paging, accept/deny/reroute, live relay). Orchestration consumer, not a record owner. | Clinical (emergency) |
| `records` | **Bominal Records** | Shared clinical data-plane boundary (common schema, FHIR/gRPC/SSE, released-view filtering, PHI-read audit). Persists; does not own the record (ADR-0016). | Clinical (data plane) |
| `logistics` | **Bominal Logistics** | Supply chain / fulfillment: inventory, shipment planning, routing, tracking. Serves pharmacy supply, med-equipment, corp procurement. | Logistics / Industrial |
| `manufacturing` | **Bominal Manufacturing** | Manufacturing ops: production scheduling, work orders, QC, regulatory compliance; partner-manufacturing-network mgmt. | Manufacturing / Industrial |
| `security` | **Bominal Security Service** | Customer-facing security/compliance: log ingestion, posture, evidence collection, Workflow-backed response playbooks (hexagonal skeleton). | Security / Trust |

Adjacent (non-`modules/`) product/track names that appear in the planning + portfolio:
**Bominal Finance** (reference rewrite product, separate from Corporate), **Bominal Train**
(brownfield product on its own hobby track), **Workflow Studio** (governed workflow
editor/builder + AI-assisted drafts), **Connect** clients, and the portfolio-arm labels
(see §d) `notify`, `documents`, `intelligence`, `platform`.

**Product-control plane** (`product-control/`) is the runtime governance layer over all of
the above — the vocabulary is **Capability → Module → Preset → Plan → Entitlement → Policy →
Feature-flag → Topology → Activation → Metering**, with the core law "no product behavior is
enabled by accident" (no route without a capability; no capability without a module gate; no
access without an entitlement; no billable behavior without metering). It carries a 5-state
record lifecycle (`draft/active/preview/deprecated/retired`), distinct from the 6-state
catalog-authority lifecycle. This is the live realization of the legacy "capability-tier =
activation bundle" doctrine — relevant to legacy-recovery item #1 (KR HR/payroll bundle).

---

## (b) FULL vertical list (near-term → far-future)

The 54-milestone graph + portfolio arms enumerate the **maximal vertical scope**. Grouped by
domain (★ = far-future 2029–2030 tier; ⚠ = explicitly high-risk / evidence-gated):

**Corporate / Business platform**
- Corporate SaaS (HR/HCM, payroll — KR-payroll is THE entry wedge), finance ops, procurement, accounting/tax, GRC/compliance, contracts + e-sign, SCIM
- Workflow Studio (governed builder, agentic-action governance, external developer plugin SDK + sandbox + signing)
- Partner-launched capabilities (Tier-1 plugins; accounting pilot, ADR-0234)

**Payments / Fintech**
- Domestic payments rails (KR banking + cards + tax-withholding), invoicing, reconciliation, PCI
- ⚠ **Capital-markets / capital-markets fintech (2028-Q1)** — instrument master, market/reference data, portfolio positions, NAV/P&L, ETF holdings, hedge/buy-side/sell-side research; Aladdin/Bloomberg benchmarks

**Communications / Social (Connect suite)**
- Mail + calendar, Messenger (privacy-first), Community (TeamBlind-like)
- Email + Messenger mining (org-pillar intelligence, 2027-Q2)

**Clinical / Health**
- Medical (provider record), Healthcare (patient view), Pharmacy, Emergency Services, Records (data plane), Insurance
- Health Entry (2026-Q3 — governed admin/documentation assistant, NO autonomous diagnosis)
- **CDSS** Phase 1 (2027-Q1, DDx-only, safety-gated) → Phase 2 (2027-Q4, imaging + EKG multimodal)
- ⚠ **Healthcare 3D anatomy + specialty planning (2028-Q1)** — imaging→3D, surgical/dentistry/specialty workbenches, SaMD boundary

**Logistics / Transport / Industrial (2026 monthly spine)**
- Logistics Spine → Transport Core → Dock & Yard → Manufacturing Delivery → Warehouse Support → Automated Logistics
- Industry Entry I (Logistics + Transport + Warehouse), GS1 EPCIS/CBV traceability, WMS/TMS/carrier/telematics adapters, control tower
- Manufacturing core (discrete/process/batch/job-shop/electronics/food; MES/QMS/SCADA; OEE; eDHR/eBR; OT-safety no-actuation)

**Marketplace / Consumer**
- Marketplace + Profiling (2027-Q1) — Insurance marketplace, **Real Estate marketplace** (consumer + commercial; Zillow/Naver/CoStar benchmarks), escrow/temporary-funds, search/ads/ranking governance

**Vision / Facility / Robotics intelligence**
- ⚠ **CCTV Vision pipeline (2027-Q2)** — facility vision, fire/life-safety, occupancy, PPE; facility security command center; ALPR; critical-infra/government/military privacy guards
- **AMR + Facility Intelligence (2027-Q3)** — motor intelligence, AMR fleet (Open-RMF, VDA 5050, ROS 2), facility digital twin, robotics safety sandbox
- **Manufacturing AI (2027-Q3)** — defect prediction, visual inspection (semiconductor/weld/PCB), production optimization, OT-safety no-closed-loop-actuation
- **AI Integration (2028)** — model/agent runtime, registry/routing/evals, RL/optimization sandbox, industrial + healthcare + corporate AI agents
- ⚠ **Realtime global tracking + orbital intelligence (2028-Q2)** — ships/flights/satellites, AIS/ADS-B/TLE-OMM, SGP4/SDP4 orbit prediction, defense-COP events, restricted-defense-use guard
- ⚠ **Satellite imagery + geospatial intelligence (2028-Q3)** — optical/SAR/hyperspectral, imagery→3D (Sat-NeRF/3DGS), facility/pipeline/methane monitoring, geospatial financial intelligence, export-controls/OPSEC guard
- **Physical-intelligence ladder / humanoid robotics (2028+)** — vision+motor+sensor+language+sim multimodal fusion

**Conglomerate / Group scale**
- **Conglomerate Tier (2028)** — group rollups, subsidiary governance, intercompany controls, per-subsidiary isolation, cross-org policy, group-scale data residency

**★ Far-future tier (2029–2030, all deferred, all evidence-gated; from #757 + 2026-04-29 Strategic View audit)**
- ★ 2029-Q1 — Consumer + B2B capability expansion (one-stop capability layer)
- ★ 2029-Q2 — **Facilities + Data Center capabilities** (robotics-enabled facilities, automation cells, power/network/safety/maintenance, capital planning; shared Map consumer)
- ★ 2029-Q3 — **Agriculture + Food traceability** (farm-to-market operating layer)
- ★ 2029-Q4 — **Public-sector thin extensions** (permits, benefits, inspections, notices)
- ★ 2030-Q1 — **Civil Infrastructure + Utilities** ⚠ (utility-network operating contract — **POWERGRID**)
- ★ 2030-Q2 — **Public Safety + Drones + DEFENSE boundaries** ⚠⚠ (highest-risk gate: defense strategic map / common operating picture / intelligence fusion / COA assistance; drone + counter-UAS orchestration — Anduril Lattice / Palantir AIP/Gotham/Maven patterns; defense manufacturing + secure software factory CMMC/CUI/AS9100; explicit **prohibited autonomous lethal use**, legal-authority + human-command + civil-liberties guards, non-weapons boundary)

**Cross-cutting / shared capability planes** (consumed by many verticals): shared **Map +
Strategic View**, **Object Graph**, **Connect**, **Workflow Studio**, evidence/audit modules,
**Intelligence** (data/AI eval multiplier), **Security Service**, anomaly-detection program.

**Parked tracks** (pre-L1, idea-only, `#1450`): dining, cellar, pos, retail, hospitality-ops,
fashion, career. **Deferred/killed arms**: dental-clinic-hr (low compounding), generic
international payroll (regulatory complexity exceeds capacity), consumer-marketing (no wedge).

---

## (c) Roadmap sequencing / timeline

Two interleaved timelines: the **vertical roadmap** (GitHub milestones) and the **portfolio
compounding spine** (strategy docs). Note: `.planning/ROADMAP.md` itself is the OLDER v1.1
"client-transition / post-Leptos / React+Vite" engineering milestone — superseded as the
top-level plan by the milestone graph, which is the authoritative multi-year vertical roadmap.

**Vertical timeline (milestone due dates):**

| Window | Milestones | Theme |
|---|---|---|
| 2026 H1 | Core Work/Business MVP (#18); M3 KR Group Payroll + Mail launch (#49); Partner-launched caps (#53); Workflow Studio Advanced (#19); Health Entry (#20) | KR-corporate wedge + governed-assistant + foundation |
| 2026-05→10 | Logistics Spine → Transport Core → Dock & Yard → Manufacturing Delivery → Warehouse Support → Automated Logistics (#1–#6) | **Industrial monthly spine** |
| 2026-Q4 | Industry Entry I logistics/transport/warehouse (#21); Industry Entry II manufacturing (#22) | Industry entries |
| 2027-Q1 | CDSS Phase 1 DDx-only (#23); Marketplace + Profiling incl. Real Estate (#24); Ambulatory GA (#10) | Health AI + marketplace |
| 2027-Q2 | CCTV Vision (#25); Email+Messenger mining (#26) | Vision + comms intelligence |
| 2027-Q3 | AMR + Facility Intelligence (#27); Manufacturing AI (#28) | Robotics + industrial AI |
| 2027-Q4 | CDSS Phase 2 imaging+EKG (#29); Emergency Capability Beta (#12) | Multimodal clinical AI |
| 2028 | AI Integration (#30); Conglomerate Tier (#31); Infrastructure Moat (#13) incl. capital-markets + realtime-tracking/orbital + satellite/geospatial + healthcare-3D | Conglomerate + capital-markets + geospatial moat |
| **2029-Q1→2030-Q2** | ★ Consumer/B2B (#43) → Facilities/Data-Center (#44) → Agriculture/Food (#45) → Public-sector (#46) → Civil-infra/Utilities/POWERGRID (#47) → Public-safety/Drones/DEFENSE (#48) | **Far-future deferred tier** |

Parallel program tracks: **Enterprise Cloud Readiness M0–M3** (2027-Q2→2028-Q1, ADR-0187 —
claim-boundary → business-cloud package → enterprise-beta → one-stop-claim gate; "no broad
public claim before M2 evidence"). **Infrastructure Sovereignty** 2027-Q2→2028-Q2+ (ADR-0150,
see §d). Clinical product track (#7–#12, M0 Contract-Ready → internal alpha → external beta →
readiness/scale) runs as its own ladder.

**Far-future milestone shape (uniform gate).** Every deferred milestone (#23–#48) decomposes
into the SAME evidence-first pattern: `research: <vertical> mature-depth + industry-leader
benchmark matrix` → `PHASE: <vertical> M0 — contract, benchmark, and evidence locked` →
`doc(adr): <vertical> operating contract` → `EPIC: <vertical>`. High-risk sectors add a
dedicated `security`/governance issue (SaMD boundary, OT no-actuation, export controls, OPSEC,
prohibited-autonomous-lethal-use, civil-liberties). **Nothing high-risk ships before its M0
evidence gate passes.**

**Portfolio compounding spine** (strategy/`sequencing-rationale.md`, `dependency-map.md`):
```
platform → payments → corporate → messaging → documents → notify → healthcare-billing → intelligence
```
Read `A → B` as "A's existence makes B cheaper/faster/higher-trust." Build the most-pointed-to
nodes first (platform, payments, corporate). Block edges (`A ⊥ B`): trust-framework ⊥
healthcare-billing (PHI@L4) and ⊥ payments (PCI@L4); catalog-id ⊥ any-arm@L1; ecosystem-surface
⊥ any-arm@L5; audit-vocabulary ⊥ any-arm@L4. Rungs are the **Proof Ladder L0–L8** (ADR-0223;
L4 = Governed = permissions+trust+audit+rollback; L5 = Externalizable). Sequencing changes only
via the **quarterly portfolio review** (13-week cadence; kill/fund/defund with a decision
record). An arm is **killed** if it fails to compound 5/5 moats (catalog, trust, ecosystem,
data+AI, builder-OS).

---

## (d) Infrastructure-sovereignty ratchet (which substrates owned when)

**ADR-0150 "Future Infrastructure Sovereignty"** — a staged *own-the-substrate* program.
Explicitly **NOT MVP scope**; each substrate gated at **M0 = contract + incumbent-benchmark +
crypto/key-custody/evidence locked** BEFORE any replacement, and M1/M2 ("shadow candidates")
only proceed *after* M0 evidence proves continuation is worth it. This is the operational form
of the canon **own-everything ratchet** — own when, and only when, the evidence gate clears.

| When (M0 due) | Milestone | Substrate(s) to own | M0 gate content |
|---|---|---|---|
| **2027-Q2** | #35 | **IaC / replacement foundation** | OpenTofu/Terraform module standards, incumbent benchmarks, submodule policy, replacement-contract baseline |
| **2027-Q3** | #32 | **API Gateway** + **Secrets Vault / KMS** | Rust gateway contract + edge policy + incumbent benchmark; Rust vault/KMS contract + crypto boundary + key-custody evidence; OCI Monitoring/Logging/Functions auto-heal responder plan |
| **2027-Q4** | #33 | **Mail / collaboration** + **Cache / coordination** + **Event streaming** | Stalwart-class parity matrix + mail/groupware benchmark; Rust cache+session+rate-limit+idempotency+coordination baseline; Rust event-log + replay + stream + delivery baseline |
| **2028-Q1** | #34 | **Database + Storage** | Incumbent baseline for Postgres, Cassandra-class, OLAP, time-series, search, object-metadata replacement |
| **2028-Q2+** | #36 | **M1/M2 shadow candidates** | Only the substrates whose M0 evidence justified continuing |

**Substrate-ownership order (earliest-owned → latest):** IaC standards → (API Gateway, KMS/
Secrets) → (Mail, Cache/Coordination, Event Streaming) → (Database, Storage) → shadow
candidates. Deployment substrate today is transitional **OCI Basic OKE** target (current
footprint `1×A1 + 2×E2-Micro` OCI VMs/Quadlet; Cloudflare = DNS/edge only; GCP free-tier =
auxiliary cross-cloud only). Language ratchet: **Rust default** for product APIs/platform/
workers/integrations/audit/security; Go for operators/control-plane/media-plane; Elixir for
presence/dispatch; Python for analytics/AI; TypeScript for desktop/web clients; Swift/Kotlin
native mobile.

Two adjacent "own-it" programs reinforce the ratchet: **Conglomerate Tier (2028, #31)** owns
group-scale residency/isolation/intercompany controls, and **AI Integration (2028, #30)** owns
the model/agent runtime + eval + cost-governance plane rather than renting it.

---

## Reconciliation notes (for the audit)

- **No conflict with canon:** the milestone graph's maximal scope (defense + powergrid +
  geospatial + conglomerate) corroborates the already-ruled `maximal-vertical-scope incl
  defense+powergrid` canon decision. The sovereignty ratchet corroborates `own-everything-
  ratchet` + `framekernel-host-committed` (own the substrate, gated on evidence).
- **No duplication of the 7 legacy-recovery items:** those came from the trashed JSON portal.
  This doc adds the live monorepo's product surface (16 modules) + the 54-milestone vertical
  roadmap + the ADR-0150 sovereignty schedule, none of which the recovery register covered.
  The one touch-point is legacy-recovery #1 (KR HR/payroll bundle) — corroborated here by the
  live `hr`=Bominal Corporate module + product-control activation-bundle plane + milestone #49
  "M3 KR Group Payroll" — confirming the KR-payroll wedge is the live entry vertical.
- **Open item worth a founder decision:** the far-future high-risk tier (DEFENSE/drones,
  POWERGRID/utilities, geospatial/orbital with export-control + OPSEC + prohibited-autonomous-
  lethal-use boundaries) is already milestone-scoped with M0 evidence gates and ADR operating
  contracts pending. These are net-new verticals relative to the kernel-first goal order and
  should be folded into the masterplan vertical-coverage map (existing task #18) rather than
  re-derived.
