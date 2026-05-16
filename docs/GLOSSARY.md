---
purpose: Oyatie — Glossary, Vocabulary, Conventions
doc_status: published
---

# Oyatie — Glossary, Vocabulary, Conventions

> **Status:** Draft v0.1 — 2026-05-09. Single source of truth for terminology across all consolidated docs, per-product PRDs, ADRs, code comments, PR descriptions, and customer-facing material.
> **Owner:** `council-architecture` (per [DOC-CATALOG.md](DOC-CATALOG.md) `doc.glossary` row).
> **Companion:** [`machine-readable/glossary.json`](machine-readable/glossary.json) for agent consumption.

---

## 0. Vocabulary policy (read this first)

1. **Industry-standard term wins** when one exists and is unambiguous.
2. **Oyatie-specific term** is reserved for genuinely new concepts (or for renames the brand has explicitly chosen, e.g. `Bench` per ADR-0017).
3. **Korean and English are co-equal** for KR-specific terms; the glossary lists both with the legal/industry-canonical form first.
4. **Every Oyatie-specific term** must list its **closest industry equivalent** in the entry. If no equivalent exists, say so.
5. **Renamed terms** carry a "Replaces:" line listing prior Oyatie names (e.g. Bench replaces "shell" — ADR-0017).
6. **Deprecated terms** are kept in §11 with their replacement, never removed.

The fitness-function `oya-foundry-fitness-glossary` walks every consolidated doc and flags any term with > 1 spelling, any uncited acronym, any synonym used inconsistently.

---

## 1. Architecture & engineering vocabulary (industry standard, used as-is)

| Term | Definition | Why we use the industry term |
|---|---|---|
| **Hexagonal architecture** (a.k.a. Ports & Adapters) | Domain core independent of infrastructure; dependency inward. | DDD canon since Alistair Cockburn 2005; ADR-0015/0105 already use this term. |
| **Clean architecture** | Robert C. Martin's layered formulation: Entities → Use Cases → Interface Adapters → Frameworks. | Equivalent to hexagonal; we use both interchangeably in DESIGN.md §4. |
| **Bounded context** | A self-consistent model boundary (Eric Evans, DDD). | We map one bounded context per axis or per vertical. |
| **Aggregate** | A consistency boundary inside a bounded context. | DDD canon. |
| **Value object** | Immutable, identity-less domain object. | DDD canon. |
| **Entity** | Domain object with identity. | DDD canon. |
| **Repository** | Adapter that persists/retrieves aggregates. | DDD canon. |
| **Saga** | Long-running multi-step coordination across aggregates. | DDD + Vaughn Vernon. |
| **Outbox pattern** | Atomic write of domain event with the state change for reliable publishing. | Industry standard; ADR-0050/0174 implement it. |
| **CQRS** | Command-Query Responsibility Segregation. | Industry standard. |
| **Event sourcing** | Persist state as event log; project to read models. | Used selectively (audit chain is event-sourced; tenant state is not). |
| **Idempotency key** | Client-supplied key that lets a retry de-dupe at the server. | Standard. |
| **Backpressure** | Downstream signaling slowdown to upstream. | Reactive Streams canon. |
| **Cell architecture** | Tenant or workload isolated to a "cell" of compute, sized for blast-radius control. | AWS cell-based architecture canon; we use this term in DESIGN §9. |
| **Shuffle sharding** | Per-tenant assignment to a *combination* of shards to reduce blast radius. | Same AWS canon. |
| **Multi-tenant** | A single deployment serves multiple isolated customers. | Standard. |
| **Tenant isolation models** | Pool, Silo, Bridge — three SaaS isolation patterns. | AWS SaaS Factory canon; we'll use these explicitly in SPEC.md per-axis. |

## 2. Operations / SRE vocabulary

| Term | Definition |
|---|---|
| **SLO** (Service Level Objective) | Target reliability for a surface (e.g. 99.9% availability over 30d). |
| **SLI** (Service Level Indicator) | The measurement that proves SLO compliance. |
| **SLA** (Service Level Agreement) | Customer-facing commitment with remediation if violated. |
| **Error budget** | 1 − SLO; the allowance for failures in a window. |
| **Burn rate** | Rate of consuming error budget. Multi-window burn-rate alerting per Google SRE workbook. |
| **Toil** | Manual, repetitive, automatable operational work. |
| **Postmortem** | Post-incident analysis. We use the term per Google SRE convention; "blameless" is implied. |
| **Runbook** | A repeatable procedure. We catalog them in [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md). |
| **On-call** | Rotating responsibility for production incident response. |
| **MTTD / MTTA / MTTR** | Time to Detect / Acknowledge / Resolve. |
| **Game day** | Pre-planned chaos exercise. |
| **Chaos engineering** | Deliberate fault injection. |
| **Progressive delivery** | Canary, blue-green, or dark-launch deploys. ADR-0050 (Argo Rollouts). |
| **GitOps** | Declarative config in Git is the source of truth; reconciler applies. ADR-0042. |
| **Trunk-based development** | One main branch; short-lived feature branches; release cuts at tag time. ADR-0042. |

## 3. Cloud / infrastructure vocabulary (industry standard)

| Term | Definition |
|---|---|
| **IaaS / PaaS / SaaS** | Standard NIST cloud service models. Oyatie spans all three. |
| **Hyperscaler** | AWS, Azure, GCP. We *consume* hyperscaler services today; *become* a regional analogue at the Cloud axis. |
| **Region** | A geographic deployment locale (e.g. `KR-Seoul1`). |
| **Availability Zone (AZ)** | An isolated power/network failure domain inside a region. |
| **Cell** | A blast-radius-sized isolated unit inside an AZ. |
| **VPC** | Virtual Private Cloud; tenant network boundary. |
| **VPN / Direct Connect / Interconnect / Cross-Connect** | Customer-to-cloud private link. |
| **CDN** | Content Delivery Network; edge cache. |
| **Object store** | S3-class blob storage. |
| **Block store** | EBS-class attached storage. |
| **File store** | EFS/NFS/SMB-class shared filesystem. |
| **Cold tier / Archive** | Glacier-class long-tail storage. |
| **DEK / KEK** | Data Encryption Key / Key Encryption Key (envelope encryption). |
| **HSM** | Hardware Security Module. KCMVP-certified for KR cloud is 6-9 month lead-time. |
| **mTLS** | Mutual TLS. |
| **Service mesh** | Sidecar/ambient layer for service-to-service traffic (Istio Ambient per ADR-0044). |
| **Ingress / egress / east-west** | Traffic direction. |
| **NACL / SG** | Network ACL / Security Group. |

## 4. Identity & access (industry standard)

| Term | Definition |
|---|---|
| **AuthN** | Authentication. |
| **AuthZ** | Authorization. |
| **OIDC** | OpenID Connect (OAuth2 + identity layer). |
| **SAML** | Security Assertion Markup Language. |
| **STS** | Security Token Service (short-lived credentials). |
| **RBAC** | Role-Based Access Control. |
| **ABAC** | Attribute-Based Access Control. |
| **Cedar** | AWS Cedar policy language; Oyatie's authorization policy DSL per ADR-0008/0140/0157. |
| **OPA** | Open Policy Agent (alternative; not adopted, evaluated). |
| **Kyverno** | Kubernetes admission policy engine. |
| **Break-glass** | Emergency override of access control with mandatory audit. |
| **MFA / 2FA** | Multi-factor authentication. |
| **SSO** | Single Sign-On. |
| **SCIM** | System for Cross-domain Identity Management (user provisioning). |
| **JWT** | JSON Web Token. |

## 5. Data, search, ML vocabulary

| Term | Definition |
|---|---|
| **OLTP / OLAP / HTAP** | Transactional / Analytical / Hybrid stores. |
| **CDC** | Change Data Capture. |
| **Reverse ETL** | Move analytics-store data back to operational systems. |
| **Lakehouse vs Warehouse** | Iceberg/Delta + open compute vs columnar warehouse (Snowflake/BigQuery). |
| **k-anonymity** | A record is indistinguishable from at least k-1 others on the QID columns. |
| **Differential Privacy (DP)** | Bounded ε-budget noise injection that limits individual disclosure. |
| **PII / PHI / PCI** | Personally Identifiable / Protected Health / Payment Card data. |
| **PIPA** | KR Personal Information Protection Act (개인정보보호법). |
| **GDPR DSR** | EU GDPR Data Subject Request (export / delete / restrict). |
| **DSAR** | Data Subject Access Request — equivalent to GDPR DSR (export). |
| **DPIA** | Data Protection Impact Assessment. |
| **BM25** | Lexical ranking function for search. |
| **TF-IDF** | Term Frequency–Inverse Document Frequency. |
| **Inverted index** | term → posting list of doc-ids; the search-engine canonical structure. |
| **HNSW / IVF / PQ** | Hierarchical Navigable Small World / Inverted File / Product Quantization — vector-index algorithms. |
| **FAISS / Milvus / pgvector / Qdrant** | Vector DBs. ADR-0050 (superseded by 0177) and ADR-0006 (pgvector adapter). |
| **RAG** | Retrieval-Augmented Generation. Oyatie Foundry exposes a RAG endpoint per DESIGN §3. |
| **Embedding** | Dense vector representation of a document or query. |
| **Knowledge graph (KG)** | Entity-relation graph used for entity linking and semantic search. |
| **Cross-encoder rerank** | Model that re-scores a candidate set with full query+doc context. |
| **MMR** | Maximal Marginal Relevance (diversity in result lists). |
| **Featurestore** | Centralized store for ML features with online/offline parity. |
| **Model registry** | Versioned model artifact registry with lineage. |
| **Eval harness** | Repeatable scoring against a golden set. |
| **Drift** | Distributional shift in inputs or labels post-deploy. |

## 6. Ads vocabulary (industry standard)

| Term | Definition |
|---|---|
| **CPM** | Cost Per Mille (per 1000 impressions). |
| **CPC** | Cost Per Click. |
| **CPA** | Cost Per Acquisition. |
| **ROAS** | Return On Ad Spend. |
| **DSP / SSP / Ad Exchange** | Demand-Side Platform / Supply-Side Platform / Auction venue. |
| **Header bidding** | Pre-auction bidding by multiple SSPs in the page header. |
| **VAST / VPAID / OpenRTB** | Video ad serving / programmatic protocols. |
| **Viewability** | Whether the ad was actually visible to a user. |
| **IVT** | Invalid Traffic (bots, click farms). |
| **Brand safety** | Avoiding adjacency to objectionable content. |
| **Frequency capping** | Limit how often a user sees the same ad. |
| **Pacing** | Spending budget evenly over campaign duration. |
| **Lookalike audience** | Audience expanded from a seed via similarity. |
| **Retargeting** | Re-engaging users who already showed intent. |
| **Multi-touch attribution (MTA)** | Distribute conversion credit across touch-points. |
| **Last-click vs Position-based vs Data-driven attribution** | Attribution models. |
| **SKAdNetwork / Privacy Sandbox** | Apple / Google privacy-preserving attribution. |
| **IPA (IAB)** | Interoperable Private Attribution. |

## 7. Compliance & regulatory vocabulary

| Term | Definition |
|---|---|
| **HIPAA** | US Health Insurance Portability and Accountability Act. |
| **HITECH** | US health-data breach notification act. |
| **PCI-DSS v4** | Payment Card Industry Data Security Standard. |
| **SOC2 Type II** | AICPA Service Organization Controls audit, ongoing. |
| **ISO 27001 / 27017 / 27018 / 27701** | InfoSec / Cloud / PII-Cloud / PIMS standards. |
| **NIST CSF** | NIST Cybersecurity Framework. |
| **NIST SP 800-53** | US federal control catalog. |
| **GDPR** | EU General Data Protection Regulation. |
| **CCPA / CPRA** | California Consumer Privacy Act / Rights Act. |
| **PIPA (KR)** | 개인정보보호법; KR Personal Information Protection Act. |
| **KISA** | 한국인터넷진흥원; Korea Internet & Security Agency. |
| **CSAP** | Cloud Security Assurance Program — KR cloud certification (3 levels: 하/중/상; 6-12 month cycle). |
| **K-ISMS-P** | 정보보호 및 개인정보보호 관리체계 인증; KR InfoSec + Privacy management cert. |
| **KCMVP** | 한국암호모듈검증; KR cryptographic module validation. KCMVP-certified HSM is the gating procurement for cloud-axis KMS. |
| **MFDS** | 식품의약품안전처; KR Ministry of Food and Drug Safety. |
| **KFDA** | Older alias for MFDS. |
| **FSC** | 금융위원회; KR Financial Services Commission. |
| **KCC** | 방송통신위원회; KR Korea Communications Commission. |
| **NIS** | 국가정보원; KR National Intelligence Service. |
| **FIPS 140-3** | US crypto module validation. |
| **FedRAMP** | US gov cloud authorization. |
| **STAR (CSA)** | Cloud Security Alliance Security Trust Assurance Registry. |
| **신용정보법** | KR Credit Information Use and Protection Act. |
| **정보통신망법** | KR Information & Communications Network Act. |
| **청소년보호법** | KR Juvenile Protection Act. |
| **의료법** | KR Medical Service Act. |
| **약사법** | KR Pharmaceutical Affairs Act. |
| **의료광고심의위원회** | KR Medical Ad Review Committee. |
| **금융감독원** | KR Financial Supervisory Service. |
| **공공정보법** | KR Act on the Promotion of Provision and Use of Public Data. |
| **정보공개법** | KR Official Information Disclosure Act. |
| **망분리** | Network separation requirement (KR financial sector + public sector). |
| **전자세금계산서** | KR Electronic Tax Invoice (NTS-mandated). |
| **조달청** | KR Public Procurement Service. |
| **본인확인서비스** | KR identity-verification service used for regulated account onboarding. |
| **마이데이터** | KR MyData regime for consented personal-data portability. |
| **NTS** | KR National Tax Service (국세청). |
| **NHIS** | KR National Health Insurance Service (국민건강보험공단). |
| **HL7 v2** | Healthcare messaging standard. |
| **FHIR R4** | Fast Healthcare Interoperability Resources, current revision. |
| **DICOM** | Medical imaging standard. |
| **NCPDP SCRIPT** | E-prescribing standard. |
| **X12 EDI** | Healthcare/logistics EDI envelope (270/271 eligibility, 278 prior auth, 837 claim, 835 remittance, 214/990/997 logistics events). |
| **ICD-10-CM / SNOMED CT / LOINC / RxNorm** | Clinical coding systems. |
| **ISA-95** | Manufacturing operations integration model. |
| **OPC UA** | Industrial communication standard. |
| **MES / OEE / SCADA** | Manufacturing Execution System / Overall Equipment Effectiveness / Supervisory Control and Data Acquisition. |
| **EDI 856 / 940 / 944 / 990** | Logistics EDI: ASN / warehouse shipping order / WSC ack / load tender response. |
| **NACHA / SWIFT / RTP** | US ACH / international wire / real-time payments. |
| **KYC / KYB / AML** | Know Your Customer / Business / Anti-Money-Laundering. |
| **PEP** | Politically Exposed Person (sanctions-screening category). |

## 8. Oyatie-specific terms (with industry analog)

These are terms we use that don't have a clean industry term, or that we've explicitly renamed.

| Oyatie term | Definition | Industry analog | Source |
|---|---|---|---|
| **Oyatie** | The product. | (brand) | User directive 2026-05-08 |
| **oYa** | The logo abbreviation. | (brand) | User directive 2026-05-08 |
| **Bench** | The user-facing app shell that hosts vertical workspaces. Replaces "shell". | "Workspace shell" or "App shell" | ADR-0017 |
| **Object Graph (OG)** | Oyatie's typed-entity, engine-enforced, cryptographically auditable domain-data layer. | "Domain model store" with audit; closest industry analog is Apache Atlas + DDD aggregate persistence; no direct equivalent. | ADR-0006 |
| **Foundry** | Oyatie's AI agent runtime + control plane (axis 3). | "Agent platform" / "AI orchestration runtime" / "AI gateway"; closest commercial analog is LangSmith + AWS Bedrock Agents. *Note: ADR-0006 has a "no Palantir vocabulary" clause that some readers interpret as restricting "Foundry"; rename evaluation pending — see DOC user-input questions.* | (Oyatie name) |
| **Foundry Furnace** | The self-improvement loop within Foundry. | "RLHF + agent self-evaluation pipeline" | (Oyatie name; verify use in code) |
| **Capability** | A discrete unit of agent-invocable functionality with declared inputs/outputs/policy. | "Tool" (LangChain) / "Function" (OpenAI) / "Skill" (Microsoft Copilot Studio) | ADR-0021 OG-AG |
| **Capability namespace** | A scoped collection of capabilities a tenant binds to. | "API surface area" or "service catalog scope" | (Oyatie) |
| **Autonomy ceiling** | Per-tenant maximum tier of agent autonomy (T1..T4). | "Permission tier" / "Agent governance level" | ADR-0022 |
| **Persona tier (T1..T4)** | Agent-action authority levels: T1 view-only, T2 advisory, T3 execute-with-approval, T4 auto-execute. | (Oyatie-specific) | ADR-0022 |
| **Pillar (data ownership)** | Org-owned vs Person-owned data segregation. | "Data domain" / "Data product" (Data Mesh) | ADR-0008 |
| **Plane** | Control / Data / Analytics. | Industry standard term used per ADR-0017. | ADR-0017 |
| **Plane gate** | A CI gate that triggers when a surface changes plane class. | "Cross-plane review" | ADR-0017 |
| **Wave** | A coordinated sequence of work landing together (W2 / W3 / W4 ...). | "Release train" / "Increment" (SAFe) | ADR-0017 |
| **Milestone (M0..M3)** *(RETIRED 2026-05-09)* | Legacy commercial-launch gates dropped during drawing-board re-framing. Replaced by descriptive wave names — see [PRD.md §3.1](PRD.md). Listed here for forensic reference only. | "Milestone" (PMI/PMBOK) | ADR-0050 / Issue #1219 / ADR-0040 (legacy refs) |
| **Band (P0..P20)** | Backlog priority tiers; P0 highest. | "Priority tier" | This consolidation; v1 plan |
| **Team** | A coordinated multi-worker work bundle with a shared brief; standing or tactical. *(The legacy "CUG / Closed-User-Group" terminology was retired 2026-05-09 and superseded by "Team" everywhere.)* | "Cross-functional team" / "Pod" / "Squad" | CLAUDE.md "Team Worker Brief Standards"; per-team charters under [`teams/`](teams/) |
| **Claim ceiling** | Mechanical block preventing a preview slice from claiming a foundation guarantee that the foundation hasn't shipped. | "Capability gating" / "Feature flag with provenance" | (Oyatie); validator in `crates/oya-foundry-claim-ceiling-kernel` |
| **Foundation bypass** | Tracked, expirable carve-out from a foundation gate. | "Tech-debt waiver" / "Exception ticket" | `registry/foundation-bypasses/` |
| **Catalog record** | The YAML manifest describing a flat-crate. | "Service catalog entry" (Backstage) | `registry/catalog/` per ADR-0015/0222 |
| **Capability record** | The YAML manifest declaring an agent capability. | "Tool manifest" / "Function spec" | `product-control/capabilities/` |
| **Repoctl** | The internal CLI for everyday engineering tasks (check, push, validate, etc.). | "Developer CLI" | `crates/oya-tooling-cli-dev-runtime/` compatibility binary; persona split planned under `crates/oya-tooling-cli-*` |
| **Ecosystem-as-a-Service (EaaS)** | The thesis that Oyatie's 7 axes form one cohesive product. | (industry uses "Platform-as-a-Service" / "Vertical SaaS"; EaaS is Oyatie's framing) | PRD §1 |

## 9. Korean ↔ English term parity (canonical pairs)

For KR terms with English equivalents, both are acceptable in docs; use whichever is clearer for the audience.

| Korean (canonical) | English (canonical) |
|---|---|
| 개인정보보호법 | KR Personal Information Protection Act (PIPA) |
| 한국인터넷진흥원 | Korea Internet & Security Agency (KISA) |
| 식품의약품안전처 | Ministry of Food and Drug Safety (MFDS) |
| 금융위원회 | Financial Services Commission (FSC) |
| 금융감독원 | Financial Supervisory Service (FSS) |
| 방송통신위원회 | Korea Communications Commission (KCC) |
| 국가정보원 | National Intelligence Service (NIS) |
| 망분리 | Network separation |
| 전자세금계산서 | Electronic tax invoice |
| 조달청 | Public Procurement Service |
| 국세청 | National Tax Service (NTS) |
| 국민건강보험공단 | National Health Insurance Service (NHIS) |
| 본인확인서비스 | Identity verification service |
| 마이데이터 | MyData |
| 통상임금 | Ordinary wage (KR Labor Standards Act) |
| 휴일/야간 근로 | Holiday / Night work (KR Labor) |
| 주52시간 | 52-hour workweek (KR Labor) |
| 연차 사용촉진 | Annual leave usage promotion (KR Labor) |
| 청소년보호법 | Juvenile Protection Act |
| 의료법 | Medical Service Act |
| 약사법 | Pharmaceutical Affairs Act |
| 의료광고 | Medical advertisement |
| 금융광고 | Financial advertisement |
| 정치광고 | Political advertisement |
| 신용정보법 | Credit Information Use and Protection Act |
| 정보통신망법 | Information & Communications Network Act |
| 공공정보법 | Public Data Provision Act |
| 정보공개법 | Official Information Disclosure Act |
| 도로명/지번 | Road-name / Lot-number address (KR addressing) |
| 실명인증 | Real-name verification |
| 사업자등록 | Business registration |
| HWP / HWPX | Hangul Word Processor formats (KR-government default) |

## 10. Acronym index (alphabetical, with section pointer)

| Acronym | Expansion | See |
|---|---|---|
| ABAC | Attribute-Based Access Control | §4 |
| ADR | Architecture Decision Record | (project canon) |
| AGPL / GPL / LGPL / MPL / SSPL / BUSL | License families denied or reviewed by the product license policy | §7 / ADR-0013 |
| AGV / AMR | Automated Guided Vehicle / Autonomous Mobile Robot | (logistics) |
| AI | Artificial Intelligence | (industry) |
| AML | Anti-Money-Laundering | §7 |
| ANZ / AU / BR / EU / JP / KR / KSA / SG / UAE / US | Region and jurisdiction codes used in packs, locales, and residency controls | §9 / regional packs |
| API | Application Programming Interface | (universal) |
| ASN | Advance Shipping Notice (EDI 856) | §7 |
| AWS / OCI | Amazon Web Services / Oracle Cloud Infrastructure | §3 |
| AZ | Availability Zone | §3 |
| BMC | Baseboard Management Controller | (cloud) |
| BPMN | Business Process Model and Notation | (we *don't* use BPMN per ADR-0035) |
| BSD / MIT | Berkeley Software Distribution / Massachusetts Institute of Technology license families | §7 / ADR-0013 |
| CCPA / LGPD / PDPL | California Consumer Privacy Act / Brazil Lei Geral de Proteção de Dados / Personal Data Protection Law | §7 |
| CDC | Change Data Capture | §5 |
| CDN | Content Delivery Network | §3 |
| CDSS | Clinical Decision Support System | §7 (ADR-0033) |
| CI / CD | Continuous Integration / Continuous Delivery | (engineering) |
| CIS | Center for Internet Security | §7 |
| CLI | Command-Line Interface | (engineering) |
| CPA / CPC / CPM | Cost Per Acquisition / Click / Mille | §6 |
| CPU | Central Processing Unit | §3 / Cloud |
| CRDT | Conflict-free Replicated Data Type | (distributed systems) |
| CSAP | KR Cloud Security Assurance Program | §7 |
| CUG (legacy alias) | Team | §8 (retired 2026-05-09) |
| CVE | Common Vulnerabilities and Exposures | (security) |
| DAG | Directed Acyclic Graph | (workflow shape, ADR-0035) |
| DB | Database | (engineering) |
| DEK / KEK | Data / Key Encryption Key | §3 |
| DICOM | Digital Imaging and Communications in Medicine | §7 |
| DLP | Data Loss Prevention | §5 / §7 |
| DNS | Domain Name System | (networking) |
| DP | Differential Privacy | §5 |
| DPIA | Data Protection Impact Assessment | §5 |
| DR | Disaster Recovery | §2 / runbooks |
| DSAR | Data Subject Access Request | §5 |
| DSP / SSP | Demand-Side / Supply-Side Platform | §6 |
| DSR | Data Subject Request | §5 |
| EaaS | Ecosystem-as-a-Service | §8 |
| EDI | Electronic Data Interchange | §7 |
| FaaS | Function-as-a-Service | (industry) |
| FDA | US Food and Drug Administration | §7 |
| FFI | Foreign Function Interface | (engineering) |
| FHIR | Fast Healthcare Interoperability Resources | §7 |
| FinOps | Financial Operations (cloud cost) | (industry) |
| FSC / FSS | KR Financial Services Commission / Supervisory Service | §7 |
| FTE | Full-Time Equivalent | (planning / finance) |
| GA | General Availability | (release status) |
| GDPR | General Data Protection Regulation | §7 |
| GPU | Graphics Processing Unit | §3 / Foundry |
| GTM | Go-To-Market | (planning) |
| HIPAA | US health privacy act | §7 |
| HITECH | US health breach act | §7 |
| HL7 | Health Level 7 | §7 |
| HNSW | Hierarchical Navigable Small Worlds | §5 / Search |
| HR | Human Resources | (workspace / operations) |
| HSM | Hardware Security Module | §3 |
| HTAP | Hybrid Transactional/Analytical | §5 |
| HTML | HyperText Markup Language | (data format) |
| HTTP | Hypertext Transfer Protocol | (networking) |
| IAM | Identity and Access Management | §4 |
| ICD | International Classification of Diseases | §7 |
| ID | Identifier / Identity Document (context-dependent) | §4 / §7 |
| IDP / IdP | Identity Provider | §4 |
| IM / CM | Incident Manager / Comms Manager | §2 |
| IP | Intellectual Property / Internet Protocol (context-dependent) | §3 / §7 |
| IPA | Interoperable Private Attribution (IAB) | §6 |
| ISO | International Organization for Standardization | §7 |
| ISV | Independent Software Vendor | (marketplace) |
| IT | Information Technology | (operations) |
| IVT | Invalid Traffic | §6 |
| JSON | JavaScript Object Notation | (data format) |
| JWT | JSON Web Token | §4 |
| KCMVP | KR cryptographic module validation | §7 |
| KFDA | Older alias for MFDS | §7 |
| KISA | KR Internet & Security Agency | §7 |
| KMS | Key Management Service | §3 |
| KOSPI / KOSDAQ | KR exchanges (financial-vertical context) | (industry) |
| KYC / KYB | Know Your Customer / Business | §7 |
| LLM | Large Language Model | (AI) |
| LMS | Learning Management System | (vertical-education) |
| LOINC | Logical Observation Identifiers (lab) | §7 |
| MCP | Model Context Protocol | §4 / Foundry |
| MES | Manufacturing Execution System | §7 |
| MFA | Multi-Factor Authentication | §4 |
| MFDS | KR Ministry of Food and Drug Safety | §7 |
| MFL | Mistakes-and-Fixes Ledger identifier | MISTAKES-LEDGER.md |
| ML | Machine Learning | (industry) |
| MTA | Multi-Touch Attribution | §6 |
| MTTD/MTTA/MTTR | Time to Detect / Acknowledge / Resolve | §2 |
| NACHA | US ACH governing body | §7 |
| NCPDP SCRIPT | E-prescribing standard | §7 |
| NIS | KR National Intelligence Service | §7 |
| NPS | Net Promoter Score | (GTM) |
| OCR | Optical Character Recognition | (AI / document processing) |
| OEE | Overall Equipment Effectiveness | §7 |
| OG | Object Graph | §8 |
| OG-AG | Object Graph Agent Gateway | (ADR-0021) |
| OIDC | OpenID Connect | §4 |
| OLTP / OLAP | Online Transactional / Analytical Processing | §5 |
| OPA | Open Policy Agent | §4 |
| OPC UA | Industrial protocol | §7 |
| OSS | Open Source Software | §7 / ADR-0013 |
| OTel | OpenTelemetry | (industry) |
| PDF | Portable Document Format | (document format) |
| PII / PHI / PCI | Personally Identifiable / Protected Health / Payment Card | §5 / §7 |
| PIPA | KR Personal Information Protection Act | §7 |
| PMS | Property Management System (hospitality) | (vertical-hospitality) |
| POS | Point Of Sale | (vertical-retail) |
| PR | Pull Request | (engineering) |
| PRD | Product Requirements Document | (canon) |
| QID | Quasi-Identifier (privacy) | §5 |
| QUERY | OpenAPI HTTP QUERY method | §10 / API design |
| RACI | Responsible / Accountable / Consulted / Informed | (ownership) |
| RAG | Retrieval-Augmented Generation | §5 |
| RBAC | Role-Based Access Control | §4 |
| REST | Representational State Transfer | (API style) |
| RLS | Row-Level Security (Postgres) | (DB term) |
| ROAS | Return On Ad Spend | §6 |
| RTP | Real-Time Payments | §7 |
| SaaS / PaaS / IaaS | Software / Platform / Infrastructure as a Service | §3 |
| SAML | Security Assertion Markup Language | §4 |
| SBOM | Software Bill of Materials | §7 / ADR-0039 |
| SCADA | Supervisory Control and Data Acquisition | §7 |
| SCIM | Cross-domain identity provisioning | §4 |
| SDK | Software Development Kit | (engineering) |
| SERP | Search Engine Results Page | §6 |
| SLO / SLI / SLA | Service Level Objective / Indicator / Agreement | §2 |
| SMTP | Simple Mail Transfer Protocol | (email) |
| SNOMED CT | Clinical terminology | §7 |
| SOAP | Subjective/Objective/Assessment/Plan (clinical note format; not the protocol) | §7 |
| SOC2 | AICPA Service Org Controls audit | §7 |
| SQL | Structured Query Language | (database) |
| SRE | Site Reliability Engineering | §2 |
| SSO | Single Sign-On | §4 |
| STS | Security Token Service | §4 |
| TF-IDF | Term Frequency–Inverse Document Frequency | §5 |
| TLS | Transport Layer Security | (security) |
| TS | TypeScript | (engineering) |
| TTL | Time To Live | (systems) |
| UI / UX | User Interface / User Experience | (product design) |
| URL | Uniform Resource Locator | (web) |
| VAST / VPAID | Video ad standards | §6 |
| VPC | Virtual Private Cloud | §3 |
| W2 / W3 / ... | Wave (sequencing) | §8 / ADR-0017 |
| WAF | Web Application Firewall | (industry) |
| WASM | WebAssembly | (plugin substrate) |
| WMS / WCS | Warehouse Management / Control System | §7 |
| X12 EDI | EDI envelope standard | §7 |
| XML | Extensible Markup Language | (data format) |
| XSS | Cross-Site Scripting | (security) |
| YAML | YAML Ain’t Markup Language | (data format) |

## 11. Deprecated / renamed terms (kept for forensic use)

| Old | New | Reason |
|---|---|---|
| Pre-directive brand aliases | Oyatie | Brand standardization per ADR-0017 (user directive 2026-05-08) |
| oyatie-* (Cargo prefix) | oya-* | ADR-0017 |
| shell (UI) | Bench | ADR-0017 |
| Caddy (gateway) | Envoy | ADR-0013 (supersedes ADR-0004) |
| WireGuard (bastion) | OCI Bastion | ADR-0045 (supersedes earlier WireGuard plan) |
| Linkerd (mesh) | Istio Ambient | ADR-0044 (supersedes ADR-0044, ADR-0044) |
| HashiCorp Vault | OpenBao | ADR-0043 |
| HashiCorp Vault 1.14+ (BUSL) | OpenBao (MPL-2) | ADR-0043 |
| HashiCorp Terraform 1.6+ (BUSL) | OpenTofu (MPL-2) | ADR-0013 |
| Redis 7.4+ (RSAL) | Valkey (BSD-3) or DragonflyDB (BSD-3) | ADR-0013 / ADR-0045 |
| MVP / Milestone (M0..M3) | Wave per PRD §3.1 (W-Foundation, W-Foundry-Preview, ...) | Drawing-board reframing on 2026-05-09 |
| postmortem long-form | mistakes-and-fixes-ledger entry | Per `docs/MISTAKES-LEDGER.md` and CLAUDE.md |
| `oya verify` (slash command) | `repoctl check` (per recent CLAUDE.md sweep) | REV6 of ADR-0015 plan |
| Foundry engineering platform axis (separate) | Foundry (consolidated; ADR-0025 foundry-as-engineering-platform) | Foundry axis consolidation on 2026-05-09 |

## 12. Conventions

### 12.1 Crate naming

Per ADR-0105 + ADR-0106: `oya-<context>-<role>[-<capability>]`. Canonical roles: `kernel`, `domain`, `usecase`, `app`, `adapter`, `infrastructure`, `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api`. `app -> app` is forbidden; shared orchestration belongs in `usecase`. Examples: `oya-platform-tenant-kernel`, `oya-identity-usecase`, `oya-foundry-policy-app`, `oya-cloud-iam-rest`.

### 12.2 ADR naming

`decisions/ADR-NNNN-<kebab-title>.md`. Status header is one of `Accepted`, `Proposed`, `Deprecated`, `Superseded`. Always include `Supersedes:` and `Superseded by:` lines (use `-` if none).

### 12.3 Issue / PR vocabulary

| Phrase | Meaning |
|---|---|
| `Refs #N` | Soft reference; does not auto-close |
| `Closes #N` | Will auto-close on merge |
| `Blocks #N` | This blocks the linked issue |
| `Blocked-by #N` | Cannot proceed until linked issue resolves |
| `# review-bypass: <reason>` | Skips the agent review gate (per `guard-pr-merge-review.mjs`); always logged |

### 12.4 PR sections (PR template)

Mandatory H2s (per CLAUDE.md): `## Issue`, `## Summary`, `## Verification`, `## Traceability`, `## Evidence`. Lead-only optional `## Code Review`. Worker PRs MUST NOT add `## Code Review`.

### 12.5 Date format

ISO 8601 (`YYYY-MM-DD`) in all docs. KR-locale UI may render `YYYY년 MM월 DD일`.

### 12.6 Locale

Default `ko-KR` for KR-served surfaces; default `en-US` for global. UTC for all server times; user-locale for display.

### 12.7 Currency

KRW for KR; USD for global. Internal accounting in both; reporting per tenant region.

### 12.8 Plane labels (catalog)

Per [`registry/catalog/<crate>.yaml`](../registry/catalog/), the `plane:` field is one of `control`, `data`, `analytics`. Cross-plane calls require explicit declaration.

### 12.9 Capability labels (registry)

Per `product-control/capabilities/`, every capability declares `category`, `data_classes_touched`, `autonomy_tier_required`, `evidence_emission_topic`, `regulatory_packs_consumed`.

---

## 13. Sources scanned

- ADRs 0101, 0105, 0106, 0107, 0116, 0121, 0122, 0125, 0130, 0131, 0132, 0148, 0157, 0167, 0168, 0171, 0173, 0174, 0179, 0181, 0184, 0204, 0207, 0210, 0222, 0228, 0231, 0232, 0233 (and the 127-ADR full corpus indexed at [ADR-INDEX.md](ADR-INDEX.md))
- KR PIPA, KR Labor Standards Act, KR Medical Service Act, KR Pharmaceutical Affairs Act, KR Credit Information Act, KR Information & Communications Network Act, KR Juvenile Protection Act, KR public-procurement standards
- Standard industry references: NIST SP 800-53, ISO 27001/27017/27018/27701, GDPR, HIPAA, PCI-DSS v4, FHIR R4, HL7 v2, X12 EDI, NCPDP SCRIPT, ICD-10-CM, SNOMED CT, LOINC, RxNorm, ISA-95, OPC UA, OpenAPI 3, AsyncAPI, OAuth2, OIDC, SAML, SCIM, Cedar
- AWS Well-Architected, AWS SaaS Factory tenancy patterns, Google SRE workbook, DDD canon (Evans, Vernon, Cockburn)
- `CLAUDE.md`, `docs/DOC-CATALOG.md` (per [`DOC-CATALOG.md`](DOC-CATALOG.md)), `docs/DOC-CATALOG.md`
- `/Users/jasonlee/oyatie/docs/raw/*` recon outputs (all 9)

*Footer regenerated whenever this doc is edited.*
