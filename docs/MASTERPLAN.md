---
doc_class: MasterPlan
shape: anchor
length_cap: 800
authority_tier: 0
status: Accepted
date: 2026-05-13
owners:
- council-architecture
canonical_authority: docs/CONSTITUTION.md
companion_docs:
- docs/PRD.md
- docs/DESIGN.md
- docs/ROADMAP.md
- docs/RACI-OWNERSHIP.md
- docs/RISK-REGISTER.md
- docs/CHANGELOG.md
authority_chain_declaration: 'docs/CONSTITUTION.md > rest of docs/ > catalog records
  > Redirect-class files > working drafts

  '
foundation_adrs:
- ADR-0052
- ADR-0053
- ADR-0054
- ADR-0056
purpose: "MasterPlan: Oyatie — MASTERPLAN."
doc_status: published
---
# Oyatie — MASTERPLAN

## §Authority-anchor — [CONSTITUTION.md](CONSTITUTION.md)

This is the canonical Master Plan for oyatie. All milestone INDEXes / phase INDEXes / Implementation Plans under `.omc/plans/milestones/M*/` derive their authority chain from this document and ultimately from `docs/CONSTITUTION.md`.

The planning implementation tree lives at `.omc/plans/milestones/`. `docs/MASTERPLAN.md` (this file) is the canonical product/architecture masterplan; `.omc/plans/MASTERPLAN.md` is deleted — this file is the single source.

---

> **Status:** Accepted (canonical at `docs/MASTERPLAN.md`).
> **Owner:** council-architecture (cross-axis); Founder Jason Lee (north-star arbiter).
> **Date:** 2026-05-13.
> **Supersedes:** pre-2026-05-13 masterplan (7-axis / vertical-grouping / platform-terminology model).

---

## 1. Vision

Oyatie is one cohesive **ecosystem-as-a-service**, expressed as a **flat catalog of independent µservices** that integrate via Workflow and Ontology — the two load-bearing adapter primitives. Any tenant enables any µservice subset à-la-carte. No grouping, no arms, no vertical privilege.

**Oyatie ≡ Bominal** — two parallel codebases of the same product family. Bominal ADR decisions are inherited 1:1 with glossary translation unless an explicit oyatie session override exists (see §5).

**Markets:** Korea-first + US parallel; EU after; jurisdiction-pluggable via regional pack seams.

**Operating posture:** No legacy protocols; own payment rails (M04+); complete-product-not-prototype; modular SME→enterprise; Bominal Proof Ladder L0..L7 + 9 architecture planes green at every milestone gate.

---

## 2. Architecture

### 2.1 Flat µservice catalog

There are no Product Groups, no Verticals, no Arms. Every µservice is independent and modular.

```
Foundry (internal-only engine)
  Polyglot GitOps-capable VCS replacement (AST/semantic locks + tests + review/fix + merge queue controller)
  + icm + oya-tooling-agent-read + LEAN check binaries
  + Cedar + Wasmtime + Proof Ladder + 9 planes + Wave integration framework

Application (B2B unified shell — µservice)
  Tenants sign in; enable µservices à-la-carte (AWS-console model).

Flat catalog — customer-facing enable-able µservices (any tenant, any subset):
  medical, pharmacy, healthcare-portal, emergency, clinical, patient
  hr, payroll, accounting, ats, grc, performance, workforce-analytics
  manufacturing, logistics, facility-ops, procurement, security
  payments, insurance, finance-quant, settlement
  connect (dual-context: messenger + mail), community, social-graph, profile-personal
  hospitality, dining, cellar

Workflow µservice (cross-µservice action/orchestration adapter)
  State machines, DAGs, approvals, escalations, SLA timers, handoffs.
  Products publish typed events; Workflow routes them; consumers subscribe.

Ontology µservice (cross-µservice information adapter — Palantir Ontology equivalent)
  Typed Object Types + Link Types + Action Types + Functions.
  Audit-chain provenance, RLS-enforced tenant isolation, jurisdiction overlays.
  Bounded contexts: entity, link, action, function, agent-gateway, audit-chain, pillar.

Substrate µservices (always-on; underpin every other µservice):
  tenancy, identity, audit-chain, eventing, secrets,
  observability, kms, policy (Cedar), search, vector,
  data-boundary, finance-library, capability-registry,
  records (FHIR-canonical), ads, analytics

Cloud µservices (runtime substrate):
  cloud-tenancy, cloud-iam, cloud-kms, cloud-compute, cloud-storage,
  cloud-network, cloud-billing, cloud-cell, cloud-region,
  cloud-observability

Connect Personal (B2C entry path — separate from Application shell)
```

### 2.2 Workflow + Ontology = ecosystem adapter layer (load-bearing rule)

All inter-µservice integration flows through Workflow (action/orchestration) or Ontology (information/data). µservices never call each other directly. This is the central architectural invariant enforced by LEAN-A2 (cross-µservice refusal check).

### 2.3 BNF v4.1

```bnf
crate          ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
microservice   ::= kebab-token ( "-" kebab-token )*    (* 1..3 tokens; registered in [workspace.metadata.oya.microservices] *)
bc-tokens      ::= kebab-token ( "-" kebab-token )*    (* 0..N; OPTIONAL *)
layer          ::= one of 12 canonical layer values per ADR-0056 §"Layer enum"
                   kernel | domain | application | app | adapter | infrastructure
                   cli | rest | grpc | graphql | worker | sdk
```

BC slot is OPTIONAL. Omit when the µservice has a single concept at the layer. Include when the µservice has multiple BC-level splits. Cross-cutting check crates: `oya-check-<rule-name>` (BNF-exempt).

Examples: `oya-medical-encounter-domain`, `oya-payments-ledger-application`, `oya-workflow-state-machine-domain`, `oya-ontology-entity-kernel`, `oya-foundry-grit-cli`, `oya-application-product-enablement-rest`, `oya-connect-messenger-grpc`.

### 2.4 Glossary (hard rules — no exceptions in docs, code, or plans)

| Retired term | Canonical term |
|---|---|
| platform (architectural) | shared or specific µservice name |
| Object Graph | Ontology |
| Shell / Modular Product Shell | Application |
| Workspace (µservice) | Connect |
| Vertical / Arm / Product Group | flat µservice catalog |
| shared\|vertical slot2 enum | µservice name (open kebab) |

Sales-segmentation labels (Healthcare / Enterprise / FinTech / Social) are GTM-only — NOT architecture. They do not appear in crate names, directory names, or architectural docs.

### 2.5 Canonical global base + localization seams / adapters / packs (load-bearing rule)

**The expectation, set globally:** every oyatie µservice has a **canonical global base** that expresses the universal business model, and zero or more **localization overlays** that bind jurisdiction-specific concerns. The overlay form is chosen per-concern — three forms exist, all valid:

| Form | When to use | Example |
|---|---|---|
| **Seam** | Canonical base declares a port (trait); the jurisdiction plugs in a value or thin trait impl via DI | `payroll-run-domain` calls `StatutoryRateProvider`; KR pack supplies the impl with 4대보험 rates + 간이세액표 |
| **Adapter** | A separate adapter crate translates jurisdiction-specific I/O (protocol / format / portal) into canonical domain types | `oya-payroll-kr-edi-adapter` translates NPS EDI v5.0 ↔ canonical `PayrollFinalized` event |
| **Pack** | Coherent bundle of seams + adapters + Cedar fragments + Workflow templates + Typst templates, shipped as one deployable unit per jurisdiction | `kr` pack = all KR seams + adapters + policies + templates for hr / payroll / accounting / medical / pharmacy / etc. |

**Choosing the form** (per-concern, whichever is most appropriate): use a **seam** when the variation is a value or small trait impl. Use an **adapter** when there is a discrete I/O surface (EDI, API protocol, government portal). Use a **pack** for the deployable bundle and the doc-suite + audit-chain unit. The forms layer cleanly — a pack composes seams + adapters.

**Canonical global base = universal product.** No statutory rates baked in. No jurisdiction codes in business logic (only in pack adapters). No language strings in domain types (i18n keys only; locale resolution at presentation). No regulatory-authority names in domain types. CI lane `oya-check-architecture --canonical-base-neutrality` (per ADR-0064 §8) enforces.

**Pluggability rule:** A customer-facing µservice ships to a paying tenant only when (a) its canonical base passes the M02 substrate quality bar **AND** (b) at least one localization pack exists for it OR an explicit ADR declares it pack-neutral (e.g., `connect-messenger` core protocol is pack-neutral; only retention windows are pack-specific). The canonical base alone is **not** shippable to a paying tenant.

**Pack #1 — Korea (`kr`)** is the foundational localization pack. M01–M07 milestones ship the canonical base **plus** the KR pack in lock-step (oyatie's first paying tenant is KR). M09+ adds US pack; M10+ adds EU pack; JP/SEA/MENA follow under H4.

**Pack composition** (canonical, per ADR-0064):

- `pack.yaml` manifest (regulations covered, supported language(s), connectors, signed `corpus.lock` per ADR-0190)
- Per-µservice seam impls + adapter crates (BNF v4.1: `oya-<microservice>-<pack>-<bc>-<layer>` for seams inline; `oya-pack-<pack>-<microservice>-<layer>` for discrete adapter bundles)
- Per-jurisdiction Cedar policy fragments (PIPA / GDPR / HIPAA legal bases)
- Per-jurisdiction Workflow Studio templates (KR clinical handoff, US W-2 cycle, EU SEPA cycle)
- Per-jurisdiction Typst document templates (KR 급여명세서, US W-2, EU SEPA mandate)
- Acceptance evidence bundle (regulatory submission samples + signed audit-chain segment)

Canonical anchor: `docs/localization-packs/INDEX.md`. Each pack has a dedicated overview doc (`docs/localization-packs/<code>.md`).

---

## 3. Inheritance posture (per ADR-0060)

Default: inherit Bominal ADR decisions 1:1 with glossary translation. Explicit oyatie overrides (higher precedence):

| # | Override | Oyatie decision |
|---|---|---|
| 1 | Workflow placement | shared µservice; `oya-workflow-*` |
| 2 | Object Graph naming | Renamed to **Ontology** |
| 3 | Platform glossary | `platform` retired; `shared` canonical |
| 4 | Vertical/Arm grouping | Flat µservice catalog; Arms retired |
| 5 | BNF `shared\|vertical` slot2 | µservice name (open kebab) — retired binary |
| 6 | Workspace product | Workspace → Connect dual-context |
| 7 | Shell terminology | Application (capital A) |
| 8 | Sales segmentation | GTM only — NOT architecture |
| 9 | Workflow+Ontology centrality | THE ecosystem adapter layer |

Inherited from Bominal 1:1 (with glossary translation): ADR-0011, ADR-0017–ADR-0021, ADR-0028, ADR-0100–ADR-0112, ADR-0116–ADR-0128, ADR-0132, ADR-0140, ADR-0208–ADR-0215, ADR-0223–ADR-0232.

---

## 4. Milestones

### M01 — v4 BNF cutover (IN FLIGHT)

**Scope:** Atomic rename of all `oya-platform-*` / `oya-shared-*` / `oya-workspace-*` crates to BNF v4.1 flat µservice names. Amend ADR-0056 to v4.1. Four LEAN check binaries green.

**Status:** Shard 0 landed (commit ec0aee3). Shard 1 queued (114-row TSV regen pending BNF v4.1 flag in xtask-metadata-augment).

**Exit criteria:** 114-row atomic rename merged + 26-row Shard 1.5 deferred rows resolved + 4 LEAN checks green on `main`.

**Phases:**
- P01 — Shard 0 ✓ (landed)
- P02 — Shard 1 atomic rename (114 rows; regenerate TSV with `--bnf-version v4.1`)
- P03 — Shard 1.5 deferred rows (26 rows)
- P04 — iter-4 src-inspection (BNF v4.1 compliance audit across all crates)
- P05 — Post-cutover hardening (LEAN checks flip from `--report-only` to BLOCKER)

### M02 — Substrate ready

**Scope:** Foundry engine + Cloud-Tenancy substrate + Ontology µservice + Workflow µservice + Application B2B shell + all substrate µservices (tenancy, identity, audit-chain, eventing, secrets, observability, kms, policy, search, vector, data-boundary, finance-library, capability-registry, records, ads, analytics).

**Exit criteria:** Sibling team scaffolds + ships any µservice via the polyglot GitOps-capable, grit-compatible `claim/work/done/promote` path with review/fix, controller-owned rebase, and merge-queue handling with zero build-team help; 9 architecture planes green at L4-L5; all `--report-only` lanes flipped to BLOCKER; Application deployable.

**Phases:** See §7 Implementation-Plan Index for full phase + IP breakdown.

### M03 — First-paying-tenant GA

**Scope** (= Bominal phase-three, per ADR-0210): Enterprise µservices (HR + Payroll + Accounting + broader Corporate per user scope — not merely payroll/HR SaaS) + Connect Professional (Mail + Messenger with legal hold + eDiscovery per Bominal ADR-0215) + Cloud-Tenancy substrate.

**Exit criteria:** 1 KR group paying tenant live; 4대보험 EDI; 연말정산; audit chain Merkle/Ed25519 segmented per (tenant_id, period); Connect Pro Mail with legal-hold/eDiscovery; Application enabling product subset.

**Phases:** See §7 Implementation-Plan Index.

### M04 — Healthcare KR foundation

**Scope:** Activate medical / pharmacy / patient / records (FHIR-canonical substrate) / emergency µservices with full Korean regulatory binding. Workflow Studio gains clinical-handoff, prescription-lifecycle, intake-routing, and pharmacy-DUR templates. Ontology gains FHIR R5 entity types (Patient, Encounter, Observation, MedicationRequest, Prescription, Practitioner, Organization, Coverage, AllergyIntolerance).

**Regulatory:** 의료법, HIRA DUR (의약품안전사용서비스), KFDA (식약처) recall/dispatch, NHIS / 건보공단 청구, KHIRA outcomes, EMR vendor cross-walk (유비케어, 비트컴퓨터, 이지스헬스케어).

**Exit:** 1 KR hospital (≥30-bed) live; DUR realtime check p99 ≤200ms at prescription; HIRA submission automation green; FHIR R5 export pack signed; pharmacy → accounting auto-journal green.

### M05 — Connect Personal launch (B2C)

**Scope:** Activate Personal context of Connect (separate path from Application B2B shell). E2EE messaging (PQXDH + Signal ratchet, user-controlled keys; org cannot decrypt); Personal mail with user-owned audit chain; community channels; social-graph foundation; profile-personal µservice. Cross-context safety invariant: Personal data never flows to org policy engine, never indexed by org Search, never exposed via org Ontology.

**Regulatory:** 개인정보보호법 (PIPA) full B2C posture; cross-border data minimization; child-safety + minor protection (KFTC + KCC).

**Exit:** Personal context GA; cold-start cohort ≥10k MAU; cross-context safety drill passed (red-team verifies no leak in either direction); onboarding <2hr trust threshold.

### M06 — FinTech KR foundation

**Scope:** Activate payments / insurance / finance-quant / settlement µservices. KR payment rails: card acquirer adapters (KEB Hana, Shinhan Card, BC Card), 간편결제 partner APIs (토스/카카오페이/네이버페이), virtual account, recurring billing, 정산 (T+1 settlement), refund cycle, chargeback handling.

**Regulatory:** 전자금융업 등록 → 간편결제업 → 인터넷전문은행 (phased; multi-year ramp). PCI DSS L1 service-provider, KYC/AML, FSC quarterly reporting. Insurance: 보험업법 (손해/생명 separate licenses; phased).

**Exit:** 전자금융업 license registered; 1 SME tenant taking payments via oyatie rails (≥1k tx/day); settlement T+1 green; finance-quant cleanly pluggable to accounting (auto-journal); PCI L1 RoC issued.

### M07 — Industrial Suite KR

**Scope:** Activate manufacturing / logistics / facility-ops / procurement / security µservices. Workflow Studio templates: SOP-execution, shift-handover, defect-routing, last-mile-delivery, vendor-onboarding, security-audit, incident-IR, MES integration. Ontology gains domain entity types: Asset, WorkOrder, Shipment, Defect, Vendor, PO, Receipt.

**Regulatory:** 산업안전보건법, 중대재해처벌법, 화학물질관리법 (manufacturing); 화물자동차운수사업법, 항만운송사업법 (logistics); 개인정보보호법 (security records).

**Exit:** 1 KR manufacturer (≥50 employees) or 3PL logistics tenant live; cross-µservice flow proven (procurement → accounting → payroll); defect-MTTR ≤2h; shipment-tracking p99 ≤300ms.

### M08 — Enterprise breadth + workforce depth

**Scope:** Activate ats / grc / performance / workforce-analytics µservices at Workday / SAP SuccessFactors parity. ATS funnel (candidate → interview → offer → onboarding handoff to HR); GRC controls library + recurring audit cycle (SOC2/ISO27001 templates); performance review cycle (OKR / 360 / calibration); workforce analytics (attrition, engagement, comp-spend).

**Exit:** ATS-to-Payroll handoff via Workflow + Ontology proven end-to-end (no direct cross-product import); perf review cycle 1.0 shipped; SOC2-style internal audit cycle complete on tenant.

### M09 — International expansion — United States

**Scope:** Stand up US region (us-east-1 + us-west-2 OCI ARM64 cells). HIPAA-Compliant baseline (medical/pharmacy/records (FHIR-canonical substrate)). PCI DSS L1 (payments). SOC 2 Type II audit (12-month observation). US payroll: federal + 50-state tax tables; W-2/W-4/1099/I-9/ACA; 401(k) recordkeeper integration; ADP / Workday parity. USD primary; settlement via Stripe / Plaid / Dwolla. US healthcare: Epic / Cerner FHIR R5 adapters.

**Exit:** 1 US tenant live; HIPAA BAA signed; PCI DSS L1 certified; SOC 2 Type II report issued; cross-region failover drill passed (us-east-1 ↔ us-west-2 RTO ≤30s).

### M10 — International expansion — European Union

**Scope:** Stand up EU region (eu-frankfurt-1 + eu-zurich-1 cells; Schrems II-safe — no US transfer). GDPR full posture (Articles 5, 6, 9, 17, 28, 32, 33, 35). eIDAS qualified signatures. SEPA Direct Debit + Credit Transfer + Instant. IFRS bindings (accounting). NIS2 (security). DORA (financial). Per-tenant data residency pinning.

**Regulatory:** GDPR, eIDAS, SEPA, IFRS, NIS2, DORA, MDR (medical devices).

**Exit:** 1 EU tenant live; Article 28 DPA template signed; SEPA mandate flow green; cross-border data minimization audit passed.

### M11 — Healthcare expansion US/EU

**Scope:** US: HIPAA-Compliant medical/pharmacy/records (FHIR-canonical substrate); Epic FHIR R5 + USCDI v3; CDA / IPS export; HL7 v2.x cross-walk. EU: GDPR special-category PHI posture; eMedRec / NHS-compatible records; MDR conformance for device-data ingestion; cross-border PHI never transmitted without Article 9(2) basis.

**Exit:** 1 US or EU healthcare tenant live; HL7 v2 + FHIR R5 dual-stack proven; MDR / HIPAA cross-residency audit drill complete.

### M12+ — Hyperscaler maturity (future-horizon)

**Scope:** 100M-user load validated against multi-region active-active. Carbon-aware compute scheduling. Wasmtime-sandboxed third-party agent marketplace. ISV plugin ecosystem (third-party µservice authoring SDK + signed-attestation gate). Open µservice marketplace where tenants enable community-authored µservices. AI agent marketplace with autonomy-ceiling governance.

**Exit:** 100M synthetic-tenant load test green at <30ms p50 read; carbon-aware scheduler 30% reduction; ≥10 third-party ISV µservices live; agent marketplace governance audit clean.

---

## 4.5 Horizons

| Horizon | Milestones | End state |
|---|---|---|
| **H1: KR enterprise foundation** | M01–M03 | 1 KR group paying tenant on HR/Payroll/Accounting + Connect Pro Mail/Messenger; substrate complete; Application + Workflow Studio live |
| **H2: KR domain breadth** | M04–M07 | KR Healthcare, FinTech, Industrial, Connect Personal live; ≥1 design-partner tenant per domain |
| **H3: International + enterprise depth** | M08–M11 | US + EU regions live; HIPAA + GDPR compliant; ATS/GRC/Performance shipped; cross-border data residency |
| **H4: Hyperscaler maturity** | M12+ | 100M-user load validated; multi-region active-active for high-consequence µservices; ISV + AI-agent marketplace |

## 5. Regulatory roadmap (full horizon — visible from day one)

| Jurisdiction | Regime | Milestone |
|---|---|---|
| KR | 4대보험 EDI (NPS / NHIS / 고용 / 산재) | M03 |
| KR | 연말정산 21-category deduction model | M03 |
| KR | Bominal ADR-0215 dual-context legal-hold / retention (Pro) | M03 |
| KR | 의료법, HIRA DUR, KFDA, NHIS 청구, KHIRA | M04 |
| KR | EMR cross-walk (유비케어 / 비트컴퓨터 / 이지스헬스케어) | M04 |
| KR | 개인정보보호법 (PIPA) — B2C posture + child-safety | M05 |
| KR | 전자금융업 → 간편결제업 → 인터넷전문은행 (phased) | M06 |
| KR | PCI DSS L1; 보험업법 (손해/생명 separate licenses) | M06 |
| KR | 산업안전보건법, 중대재해처벌법, 화학물질관리법 | M07 |
| KR | 화물자동차운수사업법, 항만운송사업법 | M07 |
| US | SOC 2 Type II, HIPAA (BAA), PCI DSS L1 | M09 |
| US | Federal + 50-state payroll tax; ACA; W-2/W-4/1099; I-9 | M09 |
| US | USCDI v3, CDA, IPS, Epic / Cerner FHIR R5 | M11 |
| EU | GDPR (Articles 5/6/9/17/28/32/33/35), eIDAS, SEPA, IFRS | M10 |
| EU | NIS2, DORA, MDR | M10–M11 |

All compliance traits pluggable per Bominal ADR-0140 regional-pack pattern; oyatie inherits 1:1.

## 5.5 Localization pack catalog

| Pack | Code | Status | Milestones | Scope |
|---|---|---|---|---|
| **Korea** | `kr` | **Pack #1 — foundational** | M01–M07 | 4대보험 EDI, 연말정산, K-GAAP, HIRA/KFDA/NHIS/KHIRA, PIPA, 전자금융업/간편결제, FSS, 산업안전보건법, 화물자동차운수사업법, 의료법, 119, 더존/유비케어/비트컴퓨터 cross-walk |
| **United States** | `us` | Planned (H3) | M09, M11 | HIPAA-BAA, PCI DSS L1, SOC2 Type II, federal+50-state tax, W-2/W-4/1099/I-9/ACA, 401(k), USCDI v3, Epic/Cerner FHIR R5, ADP/Workday parity |
| **European Union** | `eu` | Planned (H3) | M10, M11 | GDPR (Art 5/6/9/17/28/32/33/35), eIDAS, SEPA DD/CT/Instant, IFRS, NIS2, DORA, MDR, eMedRec/NHS, multi-language (DE/FR/ES/NL/IT) |
| **Japan** | `jp` | Future (H4) | M12+ | 国民健康保険, 厚生年金, 源泉徴収, インボイス制度, FSA, 医療法 (JP) |
| **SEA pilot (SG/MY/TH/VN)** | `sea-*` | Future (H4) | M12+ | Per-country tax + payments rails (NETS, GrabPay, PromptPay) |
| **MENA pilot (SA/AE)** | `mena-*` | Future (H4) | M12+ | Zakat/VAT, NCC, mada |

**Anchor:** `docs/localization-packs/INDEX.md` is the canonical pack catalog. Each pack has a dedicated doc (`docs/localization-packs/<code>.md`) declaring: scope, regulatory binding list, supported µservices, ADRs locking pack-specific decisions, fitness lane coverage, evidence bundle template.

---

## 6. Operating model

| Concern | Canonical source |
|---|---|
| Proof Ladder L0..L7 | Bominal ADR-0223 |
| 9 architecture planes | Bominal ADR-0224..ADR-0231 |
| Wave integration framework | Bominal ADR-0232 |
| Sanctioned primitives | oyatie ADR-0053 (grit/icm/oya-tooling-agent-read) |
| Agent VCS replacement + GitOps promotion | `/specs/gitops-vcs-replacement.json` + M-CC-P00 |
| Naming justification CI | feedback-naming-justification |
| Milestone > Phase > Impl-plan hierarchy | feedback-milestone-phase-hierarchy |
| ImplementationPlan-as-ChangeSet rule | `/specs/master-plan-sequencing.json#implementation_plan_changeset_contract` |
| 4 LEAN check binaries | oya-check-architecture-cli (pending Shard 1) |

**Compound principles:** Final-shape from day one (no prototype → rewrite); provider-agnostic by default (adapter crates only for provider-specific code); distroless + smallest-image containers; hyperscaler-bar engineering (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence); auto-doc + agentic-dev-optimized.

**ImplementationPlan-as-ChangeSet rule:** Every Implementation Plan under `.omc/plans/milestones/*/phases/*/IP-*.md` is a ChangeSet-sized execution unit: claimable, independently verifiable, bundleable, promotable, and small enough to avoid locking an entire tree without graph-proven rationale. Milestones own outcomes; phases own delivery gates; IPs own the actual atomic work slice that Oyatie VCS can schedule, validate, bundle, and promote. Any IP that cannot meet this shape must be split before execution.

**GitOps-capable VCS replacement (moved earlier per 2026-05-14 directive):** M-CC-P00 is now the first cross-cutting prerequisite before broad agent fan-out. It preserves grit's good parts — AST/semantic locks, isolated workspaces, `claim → work → done`, queues, watch events — but is not Rust-only: Rust AST claims are first-class, and Swift, Kotlin, C#, C++/WinUI/XAML, TypeScript, schemas, contracts, and config all get language/artifact-specific AST/parser-backed semantic indexers. It also establishes, documents, machine-encodes, and enforces unit/integration/e2e testing standards as CI/CD admission gates. It upgrades `done` from local merge serialization to a GitOps promotion request: signed change bundle, policy/CI admission, typed review/fix loop, controller-owned rebase, merge-queue enrollment, environment reconciliation, and audited release of locks only after the promotion reaches an accepted terminal state. Agents still do not call `git` or `gh`; they call the grit-compatible interface until the successor binary is ready, then the successor interface. Canonical machine-readable contract: `/specs/gitops-vcs-replacement.json`.

**RALPLAN v5 fold-in (2026-05-14):** The approved object chain is `OyaWorkItem → IssuePlan → ChangeSet → VirtualHead / QueueAwareLease / FixupTask → ChangeBundle → Promotion / ReleaseTrain`. Grit remains the authoritative repo-transition and lock primitive during cutover; Oyatie VCS consumes/projects grit state through ports and owns scheduling, affected-build closure, evidence bundles, promotion, issue digestion, package/deploy lineage, and `ops.oyatie.com` explainability. The master-plan packet home is `.omc/plans/milestones/M-CC-cross-cutting/phases/P00-gitops-vcs-replacement/`; execution starts with IP-001 + IP-009 before queue/controller fan-out.

**Dependency seam + phase-out discipline (round-5 folded):** release-critical products may ship on Hyper/Tokio/Serde-family dependencies first, but every such dependency is conscious debt: layer-contained behind Oyatie wrapper/newtypes, registered in `/registry/tech-debt-ledger.json`, owned by DRI, tied to trigger-based replacement criteria, and guarded by `oya-check-dependency-seam-discipline`. Public APIs expose Oyatie types only; `hyper::*`, `tokio::*`, `bytes::*`, and `http_body::*` do not leak outside adapter-owned boundaries. Phase-out is triggered by ontology stability, parity evidence, p99 budget history, CVE acceleration, and dependent-wave state — not calendar wish-casting. Canonical implementation home: M-CC → P06 → `IP-002-lts-dependency-lane.md`; machine-readable masterplan seed: `/specs/masterplan.json`.

---

## 7. Tech stack

- **Rust** 1.82+; **PostgreSQL** 16 + Citus + RLS; **ClickHouse**; **TimescaleDB**; **Valkey**; **Kafka** KRaft
- **OpenBao** day-1 + HSM per-cell; **Istio** Ambient; **OpenTelemetry** + VictoriaMetrics
- **Wasmtime** + **Firecracker**; **pgroonga** + Tantivy; **pgvector**; Ed25519 + ML-DSA-87; **Cedar**; Typst
- **Trivy** + Cosign + SBOM + Kyverno; distroless containers
- **Clients:** Leptos web + 5 native (Win/Mac/Linux/iOS/Android) + SvelteKit prototype lane
- **Runtime:** OCI A1 → OKE stages (Bominal ADR-0117); on-prem capable; AWS-ready; no GCP/Azure

---

## 8. Quality and Performance Bar

### 8.1 Quality — Industry Leaders

Oyatie's quality bar is set by industry leaders (competitive-benchmarked) and hyperscalers (100M+ user scale). Horizontal scalability is mandatory from day one. No single-instance-only designs. No prototype-quality first releases — feature-complete or not shipped.

| Dimension | Reference standard |
|---|---|
| API design | Stripe (REST/gRPC contracts, idempotency, pagination, error model) |
| Data layer | Palantir Ontology (typed entities + provenance + audit) — Ontology µservice |
| UI/UX craft | Linear / Stripe / Superhuman (flat dual-mode surfaces) |
| Operational telemetry | Palantir Foundry-grade observability + on-call runbooks |
| Auth + identity | Auth0 / Okta capability parity + own-rails per Bominal ADR-0123 |
| Eventing | Confluent Kafka (KRaft) + Apache schema registry parity |
| Search | OpenSearch / Algolia parity; oyatie uses pgroonga + Tantivy |

Every PRD must include a **Competitive Benchmark** section naming the industry leader(s) the µservice targets parity with, listing quality dimensions benchmarked, and citing primary-source research.

### 8.2 Performance — Hyperscaler

| Dimension | Target |
|---|---|
| API p99 latency | ≤50ms read-only (Ontology Functions, per Bominal ADR-0107); ≤200ms write (Action Types) |
| Throughput | 10k+ req/sec per cell baseline; sharding to 100k+ aggregate via cell architecture |
| Concurrency | 100M+ users architecture (Bominal master-plan); cell-bounded blast-radius |
| Event lag | Sub-second propagation outbox → consumer |
| Audit chain | <1s segment-seal latency per (tenant, period) per Bominal ADR-0028 |
| Failover | RTO ≤30s per-cell; RPO ≤5s with outbox + cross-region replication |
| Cold start | ≤500ms per Bominal ADR-0020 multi-runtime standard |
| Tenant onboarding | ≤5min for self-serve SaaS path (Bominal ADR-0118) |

Every PRD must include a **Performance Targets** section with concrete p50/p99/p999 latency targets, throughput targets, error-budget allocation, and SLO burn-rate alarms. Every Implementation Plan must include a `## Load test` section with results meeting declared perf targets before merging to main.

### 8.3 Horizontal Scalability — Mandatory

| Requirement | Enforcement |
|---|---|
| Stateless services | Required for all `application` / `rest` / `grpc` / `graphql` / `worker` layer crates. State lives only in adapter+infrastructure. Enforced by `oya-check-statelessness-cli` (TBD M02-P09). |
| Sharded state | Postgres + Citus per Bominal ADR-0117; ClickHouse + replicas; Valkey cluster. Single-DB-only designs fail `oya-check-shardability-cli` (TBD M02-P09). |
| Event-driven | Outbox → Kafka KRaft. Direct synchronous cross-µservice calls require ADR justification. |
| Cell architecture | All tenant-bound state partitioned per (cell, region); per Bominal ADR-0009 + oyatie ADR-0009. |
| Active-active capable | All `worker` + `adapter` layers declare `active_active_compatibility` per Bominal ADR-0019. |
| Cross-region replication | Required for high-consequence µservices (medical, payments, connect-pro). Per Bominal ADR-0049. |

Every PRD must include a **Horizontal Scalability** section declaring state strategy, active-active compatibility, per-cell capacity envelope, scale-out trigger metrics, and cross-region story.

New CI fitness lanes (authored in M02-P09):
- `oya-check-statelessness-cli` — presentation/application/worker layers have no module-level mutable state
- `oya-check-shardability-cli` — DB designs declare tenant_id partition key + row-level isolation
- `oya-check-perf-budget-cli` — impl plans include load-test results meeting declared perf targets
- `oya-check-benchmark-cli` — PRDs include competitive-benchmark section before µservice graduates Proof-Ladder L4→L5

---

## 9. Industry Competitive Map

| µservice cluster | Oyatie competitive references |
|---|---|
| HR / Payroll | 더존비즈온, ADP, Workday, SAP SuccessFactors |
| Accounting / Finance | 더존 iCUBE, NetSuite, Xero |
| Healthcare (medical/pharmacy/clinical) | 유비케어, 비트컴퓨터, Epic Systems, Cerner |
| FinTech (payments/banking) | Stripe, 토스, 카카오뱅크, 케이뱅크 |
| Connect (messenger/mail/community) | Slack, Gmail (Google Workspace), Signal, Notion |
| Search | Algolia, Elasticsearch/OpenSearch, Naver Search |
| Ontology (data layer) | Palantir Foundry (Ontology + Object Graph) |
| Cloud substrate | AWS, OCI, GCP |
| Identity | Auth0, Okta, Keycloak |
| Eventing | Confluent Kafka, Apache Pulsar |
| Workflow | Temporal, Camunda, AWS Step Functions |

Each µservice PRD lists the competitor set, the specific benchmark dimensions, and the primary-source evidence. Quality parity is a gate on Proof Ladder L4→L5 graduation.

---

## 10. Sales segmentation (GTM only — NOT architecture)

| Label | GTM bucket for |
|---|---|
| Healthcare (의료) | medical / pharmacy / portal / emergency / clinical µservices |
| Enterprise (기업) | hr / payroll / accounting / manufacturing / logistics / facility-ops / procurement / security / grc / ats |
| FinTech (금융) | payments / insurance / finance-quant µservices |
| Social | connect dual-context µservice + future social-graph µservices |

---

## 11. Risk register (top items)

| ID | Description | Prob | Impact | Owner |
|---|---|---|---|---|
| RM-01 | Cross-µservice contract drift | High | High | council-architecture |
| RM-02 | Tenant data leak via PHI/PII into search/ads | Med | Catastrophic | council-privacy |
| RM-03 | Agent runtime escapes autonomy ceiling | Med | Catastrophic | axis-foundry |
| RM-04 | BNF v4.1 rename breaks `main` | Med | High | council-architecture |
| RM-05 | Workflow or Ontology adapter boundary violated | Med | High | council-architecture |

Full register: `docs/RISK-REGISTER.md`.

---

## 12. RACI summary

| Milestone | Responsible | Accountable |
|---|---|---|
| M01 | axis-foundry (rename execution) | council-architecture |
| M02 | platform-substrate + axis-foundry | council-architecture |
| M03 | axis-enterprise + axis-connect + axis-cloud | council-architecture + gtm-customer-success |
| M-CC | per-phase owner | council-architecture |

Full RACI: `docs/RACI-OWNERSHIP.md`.

---

## 13. Implementation-Plan Index

This section lists every (Milestone, Phase, Impl-Plan) tuple. Files marked **[EXISTS]** are already authored under `.omc/plans/milestones/`. Files marked **[TBD]** need authoring in a Wave 2 planning session — the path is the canonical target location.

### M-CC — Cross-cutting workstreams

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| **P00 gitops-vcs-replacement** | `.omc/plans/milestones/M-CC-cross-cutting/phases/P00-gitops-vcs-replacement/` | IP-001-symbol-lock-domain.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-002-remote-lock-store-events.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-003-change-bundle-attestation.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-004-gitops-promotion-controller.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-005-grit-compat-cli-and-migration-ratchet.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-006-polyglot-indexers.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-007-review-fix-rebase-merge-queue-loop.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-008-test-standard-enforcement.md | **[EXISTS — ChangeSet planned]** |
| P00 | | IP-009-ast-index-contract.md | **[EXISTS — ChangeSet planned]** |
| P01 agentic-pipeline-cutover | `.omc/plans/milestones/M-CC-cross-cutting/phases/P01-agentic-pipeline-cutover/` | IP-001-adr-0054-scaffold-claim.md | **[EXISTS]** |
| P01 | | IP-002-inventory-adr-0052.md | **[EXISTS]** |
| P01 | | IP-003-oya-tooling-agent-read.md | **[EXISTS]** |
| P01 | | IP-004-bidirectional-prd-cite.md | **[EXISTS]** |
| P01 | | IP-005-foundry-corpus-cross-cite.md | **[EXISTS]** |
| P01 | | IP-006-agent-facing-memory.md | **[EXISTS]** |
| P01 | | IP-007-hook-skill-audit.md | **[EXISTS]** |
| P01 | | IP-008-archive-glue.md | **[EXISTS]** |
| P01 | | IP-009-delete-active-path.md | **[EXISTS]** |
| P01 | | IP-010-parallel-claim-demo.md | **[EXISTS]** |
| P01 | | IP-011-upstream-grit-bug.md | **[EXISTS]** |
| P01 | | IP-012-authoritative-tracked-audit.md | **[EXISTS]** |
| P02 doc-automation-freshness | `.omc/plans/milestones/M-CC-cross-cutting/phases/P02-doc-automation-freshness/` | IP-001-mdbook-pipeline.md | **[EXISTS]** |
| P02 | | IP-002-doc-freshness-lane.md | **[EXISTS]** |
| P02 | | IP-003-doc-style-lane.md | **[EXISTS]** |
| P03 purpose-orphan-detection | `.omc/plans/milestones/M-CC-cross-cutting/phases/P03-purpose-orphan-detection/` | IP-001-purpose-frontmatter-audit.md | **[EXISTS]** |
| P03 | | IP-002-orphan-detection-lane.md | **[EXISTS]** |
| P04 agentic-navigability | `.omc/plans/milestones/M-CC-cross-cutting/phases/P04-agentic-navigability/` | IP-001-navigability-lane.md | **[EXISTS]** |
| P04 | | IP-002-predictable-naming.md | **[EXISTS]** |
| P05 provider-agnosticism | `.omc/plans/milestones/M-CC-cross-cutting/phases/P05-provider-agnosticism/` | IP-001-provider-coupling-lane.md | **[EXISTS]** |
| P05 | | IP-002-cloud-multi-provider-audit.md | **[EXISTS]** |
| P05 | | IP-003-adapter-substitution-harness.md | **[EXISTS]** |
| P06 distroless-lts-image | `.omc/plans/milestones/M-CC-cross-cutting/phases/P06-distroless-lts-image/` | IP-001-distroless-image-lane.md | **[EXISTS]** |
| P06 | | IP-002-lts-dependency-lane.md (dependency-seam discipline + tech-debt ledger + LTS roster) | **[EXISTS]** |
| P06 | | IP-003-static-musl-build.md | **[EXISTS]** |
| P07 hyperscaler-practices | `.omc/plans/milestones/M-CC-cross-cutting/phases/P07-hyperscaler-practices/` | IP-001-prfaq-designdoc-postmortem.md | **[EXISTS]** |
| P07 | | IP-002-1es-ci-templates.md | **[EXISTS]** |
| P07 | | IP-003-eng-excellence-merge-gate.md | **[EXISTS]** |
| P07 | | IP-004-rust-toolchain-gates.md | **[EXISTS]** |
| P08 supply-chain-security | `.omc/plans/milestones/M-CC-cross-cutting/phases/P08-supply-chain-security/` | IP-001-cosign-rekor.md | **[EXISTS]** |
| P08 | | IP-002-sbom-pipeline.md | **[EXISTS]** |
| P08 | | IP-003-license-policy-lane.md | **[EXISTS]** |
| P08 | | IP-004-slsa-attestation.md | **[EXISTS]** |
| P09 visualization-as-code | `.omc/plans/milestones/M-CC-cross-cutting/phases/P09-visualization-as-code/` | IP-001-architecture-map-walkers.md | **[EXISTS]** |
| P09 | | IP-002-mermaid-d2-graphviz-emitters.md | **[EXISTS]** |
| P09 | | IP-003-mdbook-publish-integration.md | **[EXISTS]** |
| P09 | | IP-004-architecture-map-freshness-lane.md | **[EXISTS]** |

### M01 — v4 BNF cutover

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 data-use-boundary-tenancy | `.omc/plans/milestones/M01-foundation/phases/P01-data-use-boundary-tenancy/` | IP-001-data-use-boundary-adr.md | **[EXISTS]** |
| P01 | | IP-002-tenant-kernel-contracts.md | **[EXISTS]** |
| P01 | | IP-003-dsr-cascade-engine.md | **[EXISTS]** |
| P02 identity-cedar | `.omc/plans/milestones/M01-foundation/phases/P02-identity-cedar/` | IP-001-identity-kernel.md | **[EXISTS]** |
| P02 | | IP-002-sts-rotation.md | **[EXISTS]** |
| P02 | | IP-003-cedar-policy-substrate.md | **[EXISTS]** |
| P03 audit-chain-evidence | `.omc/plans/milestones/M01-foundation/phases/P03-audit-chain-evidence/` | IP-001-merkle-ed25519-kernel.md | **[EXISTS]** |
| P03 | | IP-002-audit-asyncapi-proto.md | **[EXISTS]** |
| P03 | | IP-003-tamper-evidence-drill.md | **[EXISTS]** |
| P04 eventing-ontology | `.omc/plans/milestones/M01-foundation/phases/P04-eventing-object-graph/` | IP-001-outbox-topic-registry.md | **[EXISTS]** |
| P04 | | IP-002-object-graph-property-tiers.md | **[EXISTS]** (note: "object-graph" slug is legacy; content = Ontology) |
| P04 | | IP-003-eventing-adapters.md | **[EXISTS]** |
| P05 cell-plane | `.omc/plans/milestones/M01-foundation/phases/P05-cell-plane/` | IP-001-cell-routing-primitive.md | **[EXISTS]** |
| P05 | | IP-002-plane-separation-lane.md | **[EXISTS]** |
| P06 regional-pack-flattening | `.omc/plans/milestones/M01-foundation/phases/P06-regional-pack-flattening/` | IP-001-regional-pack-adr-kernel.md | **[EXISTS]** |
| P06 | | IP-002-flat-crates-guard.md | **[EXISTS]** |
| **P-Shard1** BNF-v4.1-rename | `.omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/` | IP-001-tsv-regen-v4.1.md | **[TBD]** |
| P-Shard1 | | IP-002-atomic-rename-114-rows.md | **[TBD]** |
| P-Shard1 | | IP-003-shard-1.5-deferred-26-rows.md | **[TBD]** |
| P-Shard1 | | IP-004-iter4-src-inspection.md | **[TBD]** |
| P-Shard1 | | IP-005-lean-checks-blocker-flip.md | **[TBD]** |

### M02 — Substrate ready

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P00 account-auth | `.omc/plans/milestones/M02-foundry-preview/phases/P00-account-auth/` | IP-001-clean-arch-skeleton.md | **[EXISTS]** |
| P00 | | IP-002-domain-types-state-machine.md | **[EXISTS]** |
| P00 | | IP-003-secret-store-port.md | **[EXISTS]** |
| P01 provider-gateway | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/` | IP-001-anthropic-adapter.md | **[EXISTS]** |
| P01 | | IP-002-openai-adapter.md | **[EXISTS]** |
| P01 | | IP-003-gemini-adapter.md | **[EXISTS]** |
| P01 | | IP-004-usage-window-route-policy.md | **[EXISTS]** |
| P02 multi-subscription-pool | `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/` | IP-001-provider-account-pool-kernel.md | **[EXISTS]** |
| P02 | | IP-002-anthropic-compat-adapter.md | **[EXISTS]** |
| P02 | | IP-003-openai-compat-adapter.md | **[EXISTS]** |
| P02 | | IP-004-oauth-subscription-capture.md | **[EXISTS]** |
| P02 | | IP-005-upstream-api-drift-lane.md | **[EXISTS]** |
| P02 | | IP-006-tos-policy-audit-chain.md | **[EXISTS]** |
| P02-vis visibility-operator-plane | `.omc/plans/milestones/M02-foundry-preview/phases/P02-visibility-operator-plane/` | IP-001-readonly-api-kernel.md | **[EXISTS]** |
| P02-vis | | IP-002-dashboard-svelte.md | **[EXISTS]** |
| P02-vis | | IP-003-dry-run-surface.md | **[EXISTS]** |
| P03 gates-validators-evidence | `.omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/` | IP-001-phase00-evidence-validator.md | **[EXISTS]** |
| P03 | | IP-002-foundry-fitness-lane-ratchet.md | **[EXISTS]** |
| P03 | | IP-003-adr-template-bypass-ledger.md | **[EXISTS]** |
| P04 transport-parity-write-gates | `.omc/plans/milestones/M02-foundry-preview/phases/P04-transport-parity-write-gates/` | IP-001-rest-graphql-transports.md | **[EXISTS]** |
| P04 | | IP-002-sse-websocket-transports.md | **[EXISTS]** |
| P04 | | IP-003-write-gate-foundations.md | **[EXISTS]** |
| P05 capability-registry-autonomy | `.omc/plans/milestones/M02-foundry-preview/phases/P05-capability-registry-autonomy/` | IP-001-capability-registry.md | **[EXISTS]** |
| P05 | | IP-002-autonomy-ceiling.md | **[EXISTS]** |
| P05 | | IP-003-rag-endpoint.md | **[EXISTS]** |
| **P06** ontology-µservice | `.omc/plans/milestones/M02-foundry-preview/phases/P06-ontology-microservice/` | IP-001-ontology-entity-link-kernel.md | **[TBD]** |
| P06 | | IP-002-ontology-action-function-kernel.md | **[TBD]** |
| P06 | | IP-003-ontology-agent-gateway.md | **[TBD]** |
| P06 | | IP-004-ontology-rls-audit-chain.md | **[TBD]** |
| **P07** workflow-µservice | `.omc/plans/milestones/M02-foundry-preview/phases/P07-workflow-microservice/` | IP-001-workflow-state-machine-domain.md | **[TBD]** |
| P07 | | IP-002-workflow-approvals-escalations.md | **[TBD]** |
| P07 | | IP-003-workflow-sla-automation.md | **[TBD]** |
| P07 | | IP-004-workflow-adapter-kafka.md | **[TBD]** |
| **P08** application-shell | `.omc/plans/milestones/M02-foundry-preview/phases/P08-application-shell/` | IP-001-application-product-enablement-api.md | **[TBD]** |
| P08 | | IP-002-application-tenant-onboarding-flow.md | **[TBD]** |
| P08 | | IP-003-application-capability-menu.md | **[TBD]** |
| **P09** substrate-µservices | `.omc/plans/milestones/M02-foundry-preview/phases/P09-substrate-microservices/` | IP-001-search-vector-substrate.md | **[TBD]** |
| P09 | | IP-002-finance-library-capability-registry.md | **[TBD]** |
| P09 | | IP-003-records-data-boundary.md | **[TBD]** |
| P09 | | IP-004-ads-analytics-substrate.md | **[TBD]** |

### M03 — First-paying-tenant GA

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 cloud-foundations | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P01-cloud-foundations/` | IP-001-kms-api-adapters.md | **[EXISTS]** |
| P01 | | IP-002-storage-api-adapters.md | **[EXISTS]** |
| P01 | | IP-003-network-api-adapters.md | **[EXISTS]** |
| P01 | | IP-004-iam-cedar-sso-sts.md | **[EXISTS]** |
| P01 | | IP-005-region-az-cell-taxonomy.md | **[EXISTS]** |
| P02 cloud-compute | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P02-cloud-compute/` | IP-001-vm-api-adapters.md | **[EXISTS]** |
| P02 | | IP-002-k8s-functions-api.md | **[EXISTS]** |
| P02 | | IP-003-capacity-management.md | **[EXISTS]** |
| P03 cloud-data-billing-observability | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P03-cloud-data-billing-observability/` | IP-001-cloud-data-adapters.md | **[EXISTS]** |
| P03 | | IP-002-billing-tax-metering.md | **[EXISTS]** |
| P03 | | IP-003-observability-otel.md | **[EXISTS]** |
| P03 | | IP-004-finops-report.md | **[EXISTS]** |
| P03 | | IP-005-marketplace-isv.md | **[EXISTS]** |
| P04 saas-platform-preview | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P04-saas-platform-preview/` | IP-001-workflow-engine.md | **[EXISTS]** |
| P04 | | IP-002-plugin-substrate.md | **[EXISTS]** |
| P04 | | IP-003-marketplace-listing.md | **[EXISTS]** |
| P05 search-preview | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P05-search-preview/` | IP-001-pgroonga-morphology.md | **[EXISTS]** |
| P05 | | IP-002-pgvector-tenant-private.md | **[EXISTS]** |
| P05 | | IP-003-rag-endpoint-data-boundary.md | **[EXISTS]** |
| P06 workspace-14-surfaces | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P06-workspace-14-surfaces/` | IP-001-mail-calendar.md | **[EXISTS]** |
| P06 | | IP-002-docs-sheets-slides-sites.md | **[EXISTS]** |
| P06 | | IP-003-drive-kms-shred.md | **[EXISTS]** |
| P06 | | IP-004-meet-chat-recordings.md | **[EXISTS]** |
| P06 | | IP-005-forms-address-tasks-notes-translate.md | **[EXISTS]** |
| P07 regional-pack-onboarding | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P07-regional-pack-onboarding/` | IP-001-kr-pack.md | **[EXISTS]** |
| P07 | | IP-002-second-pack.md | **[EXISTS]** |
| P08 cross-axis-contracts | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P08-cross-axis-contracts/` | IP-001-saas-pairs.md | **[EXISTS]** |
| P08 | | IP-002-cloud-pairs.md | **[EXISTS]** |
| P08 | | IP-003-search-ads-pairs.md | **[EXISTS]** |
| P08 | | IP-004-vertical-workspace-pairs.md | **[EXISTS]** |
| **P09** enterprise-µservices-hr-payroll | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P09-enterprise-hr-payroll/` | IP-001-hr-domain-kernel.md | **[TBD]** |
| P09 | | IP-002-payroll-4대보험-edi.md | **[TBD]** |
| P09 | | IP-003-payroll-연말정산.md | **[TBD]** |
| P09 | | IP-004-accounting-domain-kernel.md | **[TBD]** |
| **P10** connect-professional | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P10-connect-professional/` | IP-001-connect-mail-legal-hold.md | **[TBD]** |
| P10 | | IP-002-connect-messenger-ediscovery.md | **[TBD]** |
| P10 | | IP-003-connect-dual-context-boundary.md | **[TBD]** |
| **P11** audit-chain-tenant-segmentation | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P11-audit-chain-tenant-segmentation/` | IP-001-merkle-segmented-per-tenant-period.md | **[TBD]** |
| P11 | | IP-002-ed25519-signing-rotation.md | **[TBD]** |
| **P12** first-paying-tenant-onboarding | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P12-first-paying-tenant-onboarding/` | IP-001-kr-group-tenant-onboarding.md | **[TBD]** |
| P12 | | IP-002-go-live-evidence-pack.md | **[TBD]** |

### M04 — Healthcare KR foundation

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 medical-clinical | `.omc/plans/milestones/M04-healthcare-kr/phases/P01-medical-clinical/` | medical encounter kernel + clinician UI + DUR hook | **[TBD]** |
| P02 pharmacy-dur | `.../P02-pharmacy-dur/` | pharmacy prescription kernel + realtime DUR (의약품안전사용서비스) | **[TBD]** |
| P03 records-kr-healthcare | `.../P03-records-kr-healthcare/` | FHIR R5 entity types in `records` substrate + KR healthcare-pack adapters (EMR cross-walk 유비케어/비트컴퓨터/이지스) | **[TBD]** |
| P04 patient-portal-b2c | `.../P04-patient-portal-b2c/` | patient record access + appointment booking + Connect Personal linkage | **[TBD]** |
| P05 emergency-handoff | `.../P05-emergency-handoff/` | 119 routing + handoff workflow + cross-clinic dispatch | **[TBD]** |
| P06 kr-regulatory-binding | `.../P06-kr-regulatory-binding/` | HIRA / KFDA / NHIS / KHIRA submission + recall adapters | **[TBD]** |
| P07 kr-hospital-acceptance | `.../P07-kr-hospital-acceptance/` | 30-bed tenant onboarding + ADR-style evidence bundle | **[TBD]** |

### M05 — Connect Personal launch (B2C)

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 personal-context-bootstrap | `.omc/plans/milestones/M05-connect-personal/phases/P01-personal-context-bootstrap/` | Personal context flag + dual-context boundary enforcement (Bominal ADR-0208) | **[TBD]** |
| P02 e2ee-user-keys | `.../P02-e2ee-user-keys/` | PQXDH + Signal ratchet under user-controlled keys (org cannot decrypt) | **[TBD]** |
| P03 personal-mail-audit | `.../P03-personal-mail-audit/` | Personal mail + user-owned audit chain (separate Merkle root per user) | **[TBD]** |
| P04 community-social-graph | `.../P04-community-social-graph/` | community channels + social-graph foundation + profile-personal µservice | **[TBD]** |
| P05 cross-context-safety | `.../P05-cross-context-safety/` | cross-context safety drill (red-team verifies no leak either direction) | **[TBD]** |
| P06 cold-start-launch | `.../P06-cold-start-launch/` | Personal context GA; 10k MAU cold-start cohort; <2hr trust onboarding | **[TBD]** |

### M06 — FinTech KR foundation

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 payments-kernel | `.omc/plans/milestones/M06-fintech-kr/phases/P01-payments-kernel/` | payment intent / charge / refund / chargeback domain | **[TBD]** |
| P02 kr-acquirer-adapters | `.../P02-kr-acquirer-adapters/` | KEB Hana / Shinhan / BC Card adapters; 토스/카카오페이/네이버페이 partner APIs | **[TBD]** |
| P03 settlement-t1 | `.../P03-settlement-t1/` | T+1 settlement + reconciliation + finance-quant auto-journal | **[TBD]** |
| P04 insurance-kernel | `.../P04-insurance-kernel/` | 손해/생명 insurance kernel; policy/claim/underwriting | **[TBD]** |
| P05 finance-quant-pluggable | `.../P05-finance-quant-pluggable/` | finance-quant µservice pluggable to accounting via Ontology | **[TBD]** |
| P06 fsc-regulatory-binding | `.../P06-fsc-regulatory-binding/` | 전자금융업 등록 → 간편결제업 (phased); PCI DSS L1; KYC/AML | **[TBD]** |
| P07 sme-acceptance | `.../P07-sme-acceptance/` | 1 SME tenant taking payments live; ≥1k tx/day; settlement green | **[TBD]** |

### M07 — Industrial Suite KR

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 manufacturing-mes | `.omc/plans/milestones/M07-industrial-kr/phases/P01-manufacturing-mes/` | manufacturing MES integration + SOP execution + defect-routing | **[TBD]** |
| P02 logistics-tms | `.../P02-logistics-tms/` | TMS / WMS adapters; last-mile-delivery; carrier integrations | **[TBD]** |
| P03 facility-ops | `.../P03-facility-ops/` | facility-ops µservice; shift handover; incident-IR | **[TBD]** |
| P04 procurement-flow | `.../P04-procurement-flow/` | procurement → accounting → payroll cross-µservice flow proven | **[TBD]** |
| P05 security-physical | `.../P05-security-physical/` | physical security + audit; cross-walks 개인정보보호법 records | **[TBD]** |
| P06 kr-industrial-regulatory | `.../P06-kr-industrial-regulatory/` | 산업안전보건법 / 중대재해처벌법 / 화학물질관리법 bindings | **[TBD]** |
| P07 industrial-acceptance | `.../P07-industrial-acceptance/` | 1 KR manufacturer ≥50 emp or 3PL logistics tenant live | **[TBD]** |

### M08 — Enterprise breadth + workforce depth

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 ats-funnel | `.omc/plans/milestones/M08-enterprise-breadth/phases/P01-ats-funnel/` | applicant tracking funnel (candidate → interview → offer → onboarding) | **[TBD]** |
| P02 grc-controls | `.../P02-grc-controls/` | GRC controls library + recurring audit cycle (SOC2/ISO27001 templates) | **[TBD]** |
| P03 performance-cycle | `.../P03-performance-cycle/` | performance review cycle (OKR / 360 / calibration) | **[TBD]** |
| P04 workforce-analytics | `.../P04-workforce-analytics/` | attrition / engagement / comp-spend analytics | **[TBD]** |
| P05 enterprise-handoffs | `.../P05-enterprise-handoffs/` | ATS → HR → Payroll handoff via Workflow + Ontology end-to-end | **[TBD]** |

### M09 — International expansion — United States

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 us-region-cells | `.omc/plans/milestones/M09-us-expansion/phases/P01-us-region-cells/` | us-east-1 + us-west-2 OCI ARM64 cells; cross-region failover RTO ≤30s | **[TBD]** |
| P02 us-payroll-tax | `.../P02-us-payroll-tax/` | federal + 50-state tax; W-2/W-4/1099/I-9/ACA; 401(k) recordkeeper integration | **[TBD]** |
| P03 us-payments-rails | `.../P03-us-payments-rails/` | USD primary; Stripe / Plaid / Dwolla; ACH; same-day ACH; wire | **[TBD]** |
| P04 hipaa-baa-baseline | `.../P04-hipaa-baa-baseline/` | HIPAA-Compliant medical/pharmacy/records (FHIR-canonical substrate); BAA template | **[TBD]** |
| P05 soc2-pci-certification | `.../P05-soc2-pci-certification/` | SOC 2 Type II (12-mo observation); PCI DSS L1 service-provider RoC | **[TBD]** |
| P06 us-tenant-acceptance | `.../P06-us-tenant-acceptance/` | 1 US tenant live; cross-region failover drill passed | **[TBD]** |

### M10 — International expansion — European Union

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 eu-region-cells | `.omc/plans/milestones/M10-eu-expansion/phases/P01-eu-region-cells/` | eu-frankfurt-1 + eu-zurich-1; Schrems II-safe (no US transfer) | **[TBD]** |
| P02 gdpr-posture | `.../P02-gdpr-posture/` | GDPR Articles 5/6/9/17/28/32/33/35; per-tenant data-residency pinning | **[TBD]** |
| P03 eidas-sepa | `.../P03-eidas-sepa/` | eIDAS qualified signatures; SEPA Direct Debit + Credit Transfer + Instant | **[TBD]** |
| P04 ifrs-accounting | `.../P04-ifrs-accounting/` | IFRS bindings (accounting); cross-walks to K-GAAP / US GAAP | **[TBD]** |
| P05 nis2-dora-mdr | `.../P05-nis2-dora-mdr/` | NIS2 (security), DORA (financial), MDR (medical devices) | **[TBD]** |
| P06 eu-tenant-acceptance | `.../P06-eu-tenant-acceptance/` | 1 EU tenant live; Article 28 DPA signed; SEPA mandate green | **[TBD]** |

### M11 — Healthcare expansion US/EU

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 hipaa-fhir-r5-usa | `.omc/plans/milestones/M11-healthcare-intl/phases/P01-hipaa-fhir-r5-usa/` | Epic / Cerner FHIR R5 adapters; USCDI v3; CDA / IPS export | **[TBD]** |
| P02 hl7v2-crosswalk | `.../P02-hl7v2-crosswalk/` | HL7 v2.x dual-stack with FHIR R5; legacy hospital interop | **[TBD]** |
| P03 gdpr-phi-eu | `.../P03-gdpr-phi-eu/` | GDPR special-category PHI posture (Article 9(2) bases) | **[TBD]** |
| P04 emedrec-nhs-eu | `.../P04-emedrec-nhs-eu/` | eMedRec / NHS-compatible records; cross-border PHI minimization | **[TBD]** |
| P05 mdr-device-ingestion | `.../P05-mdr-device-ingestion/` | MDR conformance for medical-device data ingestion | **[TBD]** |
| P06 intl-healthcare-acceptance | `.../P06-intl-healthcare-acceptance/` | 1 US or EU healthcare tenant live; dual-stack proven | **[TBD]** |

### M12+ — Hyperscaler maturity (future-horizon)

| Phase | Phase path | Scope summary | Status |
|---|---|---|---|
| P01 100m-load-validation | `.omc/plans/milestones/M12-hyperscaler-maturity/phases/P01-100m-load-validation/` | 100M synthetic-tenant load test; multi-region active-active | **[TBD]** |
| P02 carbon-aware-scheduling | `.../P02-carbon-aware-scheduling/` | carbon-aware compute scheduler; 30%+ carbon reduction | **[TBD]** |
| P03 isv-marketplace | `.../P03-isv-marketplace/` | ISV plugin ecosystem; signed-attestation gate; ≥10 third-party µservices | **[TBD]** |
| P04 agent-marketplace | `.../P04-agent-marketplace/` | Wasmtime-sandboxed agent marketplace; autonomy-ceiling governance | **[TBD]** |

> **Note on legacy M04 directory:** The old `.omc/plans/milestones/M04-vertical-pilot-korea/` phases were authored under the pre-2026-05-13 "Vertical-Pilot Korea" model. That model retires the "vertical/arm" terminology and is superseded by the flat µservice catalog. The legacy directory is scheduled for physical removal in a dedicated cleanup phase (no compat seams; stale removed in reality, per `feedback_autonomous_implementation_artifacts.md`). M04 is now Healthcare KR foundation.

---

## 13.5 Documentation suite coverage (CI-enforced)

Every planned µservice ships with a complete documentation suite. Coverage is **CI-enforced** via `oya-check-documentation-cli` (LEAN-A5; report-only until M02-P22, BLOCKER thereafter). See ADR-0063.

### 13.5.1 Per-µservice canonical artifact suite

For every µservice registered in `[workspace.metadata.oya.microservices]`:

| Artifact | Path convention | Template |
|---|---|---|
| Microservice record | `docs/microservices/<microservice>.md` | `docs/templates/microservice-template.md` |
| Product Requirements (canonical, pack-neutral) | `docs/prds/<microservice>.md` | `docs/templates/prd-template.md` |
| Naming-scope ADR | `docs/decisions/ADR-NNNN-microservice-<microservice>.md` | `docs/templates/adr-template.md` |
| Bounded-context registrations (one per BC) | `docs/bounded-contexts/<microservice>-<bc>.md` | `docs/templates/bounded-context-registration-template.md` |
| Phase-Specs (≥1 referencing the µservice) | `.omc/plans/milestones/M*/phases/*/phase-spec.md` | `docs/templates/phase-spec-template.md` |
| Impl-Plans (one per IP) | `.omc/plans/milestones/M*/phases/*/impl-plan.md` | `docs/templates/impl-plan-template.md` |

### 13.5.2 Per-localization-pack overlay suite (per pack × per µservice in pack scope)

| Artifact | Path convention |
|---|---|
| Pack overlay PRD | `docs/prds/<microservice>-<pack>.md` (required when pack adds material scope; optional otherwise) |
| Pack regulatory ADR | `docs/decisions/ADR-NNNN-<pack>-<microservice>-regulatory.md` |
| Pack acceptance evidence | `docs/localization-packs/<pack>/evidence/<microservice>.md` |

Pack `pack.yaml` manifests at `docs/localization-packs/<pack>/pack.yaml` declare which µservices the pack covers; the CI lane derives the required (pack × µservice) cross-product from there.

### 13.5.3 Per-milestone artifacts

| Artifact | Path convention |
|---|---|
| Milestone README | `.omc/plans/milestones/M<NN>-<slug>/README.md` (`milestone-readme-template.md`) |
| Acceptance evidence bundle | `.omc/plans/milestones/M<NN>-<slug>/acceptance-evidence/` |

### 13.5.4 Enforcement — `oya-check-documentation-cli` (LEAN-A5)

Lane registered in `registry/quality/lanes.yaml`. Runs on every PR.

Algorithm:

1. Parse `[workspace.metadata.oya.microservices]` for canonical µservice list
2. For each µservice, verify every row in §13.5.1 exists; report missing as violation
3. Parse `docs/localization-packs/INDEX.md` for active packs + each `pack.yaml` for scope
4. For each (pack × µservice) pair in pack scope, verify §13.5.2 rows; report missing
5. Per-milestone: verify §13.5.3 rows exist for every milestone directory in `.omc/plans/milestones/`
6. Section-completeness checks: every PRD has a `## Competitive Benchmark` section (per quality bar); every Impl-Plan has a `## Load test` section (per perf bar); every Phase-Spec frontmatter declares `acceptance_lanes:`
7. Exit nonzero in BLOCKER mode if any required artifact missing or any required section absent

Coverage snapshot (auto-emitted by the lane): `docs/DOC-COVERAGE.md`.

### 13.5.5 Suite-completeness is a phase exit gate

A phase that registers a new µservice (or new BC) is **not Complete** until the doc-coverage lane is green for that µservice. Per `feedback_autonomous_decision_principles.md` scope-completion rule: no stubs, no placeholders, no deferrals. The doc suite ships in the same commit that introduces the µservice.

---

## 14. References

- Memory files: `~/.claude/projects/-Users-jasonlee-oyatie/memory/MEMORY.md`
- ADRs: `docs/decisions/ADR-*.md` (especially ADR-0056 v4.1, ADR-0058..0061 overrides)
- Bominal cross-reference: `/Users/jasonlee/bominal/decisions/` and `/Users/jasonlee/bominal/docs/`
- Planning tree: `.omc/plans/milestones/`

---

## 15. Status footer

Status: **Accepted** (canonical at `docs/MASTERPLAN.md`).
Iteration: 10 — enforced ImplementationPlan-as-ChangeSet semantics for all IPs and materialized M-CC-P00 Oyatie VCS phase/IP packet files from approved ralplan v5. Iteration: 9 — added documented/enforced unit/integration/e2e test standard plus explicit AST handling to M-CC-P00. Iteration: 8 — expanded M-CC-P00 to cover polyglot semantic indexing plus CI/CD review-fix loop, controller-owned rebase, and merge queue handling. Iteration: 7 — moved the grit-compatible VCS replacement ahead of other cross-cutting work as M-CC-P00 and upgraded its target from local merge serialization to GitOps promotion/reconciliation per 2026-05-14 user directive. Iteration: 6 — folded dependency-seam phaseout round-5 into §6/§8/M-CC hierarchy and seeded `/specs/masterplan.json` per Markdown-retirement target. Iteration: 5 — extended 2026-05-13 with M04–M12 milestone scope (Healthcare KR, Connect Personal B2C, FinTech KR, Industrial Suite KR, Enterprise breadth, US/EU expansion, Healthcare US/EU, Hyperscaler maturity), §2.5 Canonical base + localization packs (KR pack #1; ADR-0064), §5.5 Localization pack catalog, §13.5 Documentation suite coverage CI-enforced (ADR-0063 / LEAN-A5 `oya-check-documentation-cli`). Iteration 4 (earlier on 2026-05-13): full rewrite per /deep-interview session consensus — flat µservice catalog, BNF v4.1, Ontology/Workflow adapter layer, Bominal inheritance posture, M01-M03 phase+IP index.
