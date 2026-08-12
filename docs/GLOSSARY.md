---
purpose: Oyatie — Glossary, Vocabulary, Conventions
doc_status: published
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

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

The fitness-function `oya-governance-glossary` walks every consolidated doc and flags any term with > 1 spelling, any uncited acronym, any synonym used inconsistently.

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
| **VPN / Direct / Interconnect / Cross-Connect** | Customer-to-cloud private link. |
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
| **OIDC** | OpenID (OAuth2 + identity layer). |
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
| **Lakehouse vs Warehouse** | Open-table-format (Apache Iceberg canonical per ADR-0337) over object storage with open compute vs columnar warehouse (Snowflake/BigQuery). |
| **Apache Iceberg** | Canonical Oyatie OLAP table-format write path per ADR-0337; Apache-2.0; hyperscaler-managed via AWS S3 Tables, Snowflake Polaris, Google BigLake, Databricks Unity Catalog Iceberg REST, Azure Synapse Lake. |
| **Iceberg REST Catalog** | Canonical Iceberg catalog binding (v1.7+) per ADR-0337; Apache Polaris is the reference implementation for self-managed deployments. |
| **Apache Polaris** | Open-source Iceberg REST Catalog reference implementation; Snowflake-authored, donated to ASF as incubating 2024-07-23; canonical for self-managed and OCI-guest contexts per ADR-0337. |
| **BigLake** | Google Cloud's managed Iceberg REST Catalog endpoint (GA 2025-01-15); canonical for GCP-guest context per ADR-0337. |
| **Delta UniForm** | Databricks-authored Delta variant that emits Iceberg metadata pointing at Delta data; read-accepted by Oyatie's Iceberg read path per ADR-0337 §D-2.4. |
| **Apache Delta Lake** | Adapter-only OLAP substrate per ADR-0337; canonical write path is Iceberg. Tenants ingesting Delta-formatted data are served by `oya-<ms>-adapter-delta-ingest-to-iceberg`. |
| **Apache Hudi** | Adapter-only OLAP substrate per ADR-0337; canonical write path is Iceberg. Tenants ingesting Hudi-formatted data are served by `oya-<ms>-adapter-hudi-ingest-to-iceberg`. |
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
| **Claim ceiling** | Mechanical block preventing a preview slice from claiming a foundation guarantee that the foundation hasn't shipped. | "Capability gating" / "Feature flag with provenance" | (Oyatie); validator in `crates/oya-governance-claim-ceiling-kernel` |
| **Foundation bypass** | Tracked, expirable carve-out from a foundation gate. | "Tech-debt waiver" / "Exception ticket" | `registry/foundation-bypasses/` |
| **Catalog record** | The YAML manifest describing a flat-crate. | "Service catalog entry" (Backstage) | `registry/catalog/` per ADR-0015/0222 |
| **Capability record** | The YAML manifest declaring an agent capability. | "Tool manifest" / "Function spec" | `registry/capability-templates/` |
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
| AAAA | IPv6 address DNS resource record | (networking) |
| AATL | Adobe Approved Trust List | §7 / e-signature trust |
| AACSB | Association to Advance Collegiate Schools of Business | §7 / education accreditation packs |
| ABAC | Attribute-Based Access Control | §4 |
| AADC | Age Appropriate Design Code | §7 / ADR-0292 |
| ADR | Architecture Decision Record | (project canon) |
| AGPL / GPL / LGPL / MPL / SSPL / BUSL | License families denied or reviewed by the product license policy | §7 / ADR-0013 |
| AGV / AMR | Automated Guided Vehicle / Autonomous Mobile Robot | (logistics) |
| AI | Artificial Intelligence | (industry) |
| AML | Anti-Money-Laundering | §7 |
| ANZ / AU / BE / BR / EU / JP / KR / KSA / NL / SG / UAE / US | Region and jurisdiction codes used in packs, locales, and residency controls | §9 / regional packs |
| API | Application Programming Interface | (universal) |
| ARPA | American Rescue Plan Act | §7 / tax journeys |
| ASN | Advance Shipping Notice (EDI 856) | §7 |
| ASR | Automatic Speech Recognition | §5 / voice transcription |
| AWS / OCI | Amazon Web Services / Oracle Cloud Infrastructure | §3 |
| AZ | Availability Zone | §3 |
| BLS | Bureau of Labor Statistics | §7 / labor data |
| BMC | Baseboard Management Controller | (cloud) |
| BPMN | Business Process Model and Notation | (we *don't* use BPMN per ADR-0035) |
| BSD / MIT | Berkeley Software Distribution / Massachusetts Institute of Technology license families | §7 / ADR-0013 |
| CCPA / LGPD / PDPL | California Consumer Privacy Act / Brazil Lei Geral de Proteção de Dados / Personal Data Protection Law | §7 |
| CDC | Change Data Capture | §5 |
| CDN | Content Delivery Network | §3 |
| CDTFA | California Department of Tax and Fee Administration | §7 / tax journeys |
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
| DACH | Germany, Austria, Switzerland regional market grouping | §9 / regional packs |
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
| DIR | Department of Industrial Relations | §7 / labor compliance |
| DWC | Division of Workers' Compensation | §7 / labor compliance |
| EaaS | Ecosystem-as-a-Service | §8 |
| ECH | Encrypted Client Hello | §3 / ADR-0253 |
| ECOWAS | Economic Community of West African States | §7 / regional packs |
| EDI | Electronic Data Interchange | §7 |
| EIR | Employer Injury Report | §7 / workplace safety |
| FaaS | Function-as-a-Service | (industry) |
| FDA | US Food and Drug Administration | §7 |
| FFI | Foreign Function Interface | (engineering) |
| FHIR | Fast Healthcare Interoperability Resources | §7 |
| FinOps | Financial Operations (cloud cost) | (industry) |
| FROI | First Report of Injury | §7 / workplace safety |
| FSC / FSS | KR Financial Services Commission / Supervisory Service | §7 |
| FTE | Full-Time Equivalent | (planning / finance) |
| GA | General Availability | (release status) |
| GDPR | General Data Protection Regulation | §7 |
| GLOSSARY | Canonical glossary file token | §12 / documentation-rigor |
| GPU | Graphics Processing Unit | §3 / Foundry |
| GTM | Go-To-Market | (planning) |
| HIPAA | US health privacy act | §7 |
| HITECH | US health breach act | §7 |
| HL7 | Health Level 7 | §7 |
| HLC | Hybrid Logical Clock | §5 / ADR-0252 |
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
| IMSI | International Mobile Subscriber Identity | §4 / telecom identity |
| IP | Intellectual Property / Internet Protocol (context-dependent) | §3 / §7 |
| IPA | Interoperable Private Attribution (IAB) | §6 |
| ISO | International Organization for Standardization | §7 |
| ISV | Independent Software Vendor | (marketplace) |
| IT | Information Technology | (operations) |
| IVT | Invalid Traffic | §6 |
| JV | Joint Venture | §7 / corporate structures |
| JSON | JavaScript Object Notation | (data format) |
| JWT | JSON Web Token | §4 |
| KCMVP | KR cryptographic module validation | §7 |
| KFDA | Older alias for MFDS | §7 |
| KISA | KR Internet & Security Agency | §7 |
| KMS | Key Management Service | §3 |
| KPN | Dutch telecommunications provider brand used as a signing trust-root issuer | §7 / regional packs |
| KOSA | Kids Online Safety Act | §7 / ADR-0292 |
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
| NEMA | National Emergency Management Agency | §7 / regional packs |
| NBSP | Non-breaking space | (typography / localization) |
| NCPDP SCRIPT | E-prescribing standard | §7 |
| NIS | KR National Intelligence Service | §7 |
| NIMASA | Nigerian Maritime Administration and Safety Agency | §7 / regional packs |
| NIMET | Nigerian Meteorological Agency | §7 / regional packs |
| NPS | Net Promoter Score | (GTM) |
| OCR | Optical Character Recognition | (AI / document processing) |
| OEE | Overall Equipment Effectiveness | §7 |
| OG | Object Graph | §8 |
| OG-AG | Object Graph Agent Gateway | (ADR-0021) |
| OIDC | OpenID | §4 |
| OLTP / OLAP | Online Transactional / Analytical Processing | §5 |
| OPA | Open Policy Agent | §4 |
| OPC UA | Industrial protocol | §7 |
| OSS | Open Source Software | §7 / ADR-0013 |
| OTel | OpenTelemetry | (industry) |
| PDF | Portable Document Format | (document format) |
| PC | Personal Computer | (end-user device context) |
| PDT | Pacific Daylight Time | (time zone) |
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
| SECA (cloud) | Sovereign European Cloud API — open declarative cloud infrastructure API (EuroStack / IPCEI-CIS); oyatie SECA-capable contracts are Rust-first | [architecture/cloud-provider-full-ecosystem-north-star.md](architecture/cloud-provider-full-ecosystem-north-star.md) · [spec.secapi.cloud](https://spec.secapi.cloud/) |
| SECA (tax) | Self-Employment Contributions Act | §7 / tax journeys |
| SERP | Search Engine Results Page | §6 |
| SES | Simple Email Service | §7 / email communications |
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
| TXT | DNS text resource record | §3 / email deliverability |
| UI / UX | User Interface / User Experience | (product design) |
| UK | United Kingdom jurisdiction code | §7 / regional packs |
| URL | Uniform Resource Locator | (web) |
| VAST / VPAID | Video ad standards | §6 |
| VHF | Very High Frequency | §3 / telecom |
| VPC | Virtual Private Cloud | §3 |
| W2 / W3 / ... | Wave (sequencing) | §8 / ADR-0017 |
| WAF | Web Application Firewall | (industry) |
| WASM | WebAssembly | (plugin substrate) |
| WMS / WCS | Warehouse Management / Control System | §7 |
| X12 EDI | EDI envelope standard | §7 |
| XML | Extensible Markup Language | (data format) |
| XSD | XML Schema Definition | (data format) |
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
| Redis 7.4+ (RSAL/SSPL — Redis Inc. dual-license since 2024-03-20) | **Valkey** (BSD-3-Clause — Linux Foundation fork) | ADR-0013 / ADR-0045 / **ADR-0336** (canonical authority — DragonflyDB removed because BSL-1.1 is on the forbidden-license list) |
| MVP / Milestone (M0..M3) | Wave per PRD §3.1 (W-Foundation, W-Foundry-Preview, ...) | Drawing-board reframing on 2026-05-09 |
| postmortem long-form | mistakes-and-fixes-ledger entry | Per `docs/MISTAKES-LEDGER.md` and CLAUDE.md |
| `oya verify` (slash command) | `repoctl check` (per recent CLAUDE.md sweep) | REV6 of ADR-0015 plan |
| Foundry engineering platform axis (separate) | Foundry (consolidated; ADR-0025 foundry-as-engineering-platform) | Foundry axis consolidation on 2026-05-09 |

## 12. Conventions

### 12.1 Crate naming

Per ADR-0105 + ADR-0106 + ADR-0565: `oya-<context>-<role>[-<capability>]`. Canonical roles: `kernel`, `domain`, `usecase`, `app`, `adapter`, `infrastructure`, `cli`, `rest`, `grpc`, `worker`, `sdk`, `api`. `app -> app` is forbidden; shared orchestration belongs in `usecase`. Examples: `oya-platform-tenant-kernel`, `oya-identity-usecase`, `oya-intelligence-policy-app`, `oya-cloud-iam-rest`.

### 12.2 ADR naming

`decisions/ADR-####-<kebab-title>.md`. Status header is one of `Accepted`, `Proposed`, `Deprecated`, `Superseded`. Always include `Supersedes:` and `Superseded by:` lines (use `-` if none).

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

Per `registry/capability-templates/`, every capability declares `category`, `data_classes_touched`, `autonomy_tier_required`, `evidence_emission_topic`, `regulatory_packs_consumed`.

---

## 13. Sources scanned

- ADRs 0101, 0105, 0106, 0107, 0116, 0121, 0122, 0125, 0130, 0131, 0132, 0148, 0157, 0167, 0168, 0171, 0173, 0174, 0179, 0181, 0184, 0204, 0207, 0210, 0222, 0228, 0231, 0232, 0233 (and the 127-ADR full corpus indexed at [ADR-INDEX.md](ADR-INDEX.md))
- KR PIPA, KR Labor Standards Act, KR Medical Service Act, KR Pharmaceutical Affairs Act, KR Credit Information Act, KR Information & Communications Network Act, KR Juvenile Protection Act, KR public-procurement standards
- Standard industry references: NIST SP 800-53, ISO 27001/27017/27018/27701, GDPR, HIPAA, PCI-DSS v4, FHIR R4, HL7 v2, X12 EDI, NCPDP SCRIPT, ICD-10-CM, SNOMED CT, LOINC, RxNorm, ISA-95, OPC UA, OpenAPI 3, AsyncAPI, OAuth2, OIDC, SAML, SCIM, Cedar
- AWS Well-Architected, AWS SaaS Factory tenancy patterns, Google SRE workbook, DDD canon (Evans, Vernon, Cockburn)
- `CLAUDE.md`, `docs/DOC-CATALOG.md` (per [`DOC-CATALOG.md`](DOC-CATALOG.md)), `docs/DOC-CATALOG.md`
- `/Users/jasonlee/oyatie/docs/raw/*` recon outputs (all 9)

*Footer regenerated whenever this doc is edited.*

<!-- codex-glossary-onboarding:start -->

## 14. Doctrinal substrate appendix - 2026-05-21

This appendix completes the 2026-05-21 glossary expansion for the unified-ecosystem doctrine.
It contributes 285 terms with an explicit binding ADR and binding doc for every row.
The table is reference-shaped so an intern can resolve each term to a decision source before implementation.

| Term | Definition | Binding ADR | Binding doc | Industry analog | Related terms |
|---|---|---|---|---|---|
| **Tenant** | Universal scoping boundary for customer, organization, person, family, program, or sovereign child. | [ADR-0244](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS account plus Salesforce org | principal, tenant_id |
| **Principal** | Actor identity evaluated before any workflow, ontology, or audit mutation. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS IAM principal | tenant, Cedar permit |
| **Cedar permit** | Policy authorization class that grants or refuses a scoped action. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS Verified Permissions policy | principal, policy fragment |
| **Workflow** | State-machine and DAG execution substrate used by durable processes. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Temporal workflow | workflow run, audit event |
| **Ontology** | Typed object graph with role, capability, jurisdiction, and freshness projections. | [ADR-0257](decisions/ADR-0356-amendment-library-first-ontology-read-path.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Palantir Foundry Ontology | object type, projection |
| **Audit-chain** | Append-only evidence stream for identity, policy, workflow, and state transitions. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | CloudTrail plus Rekor | audit event, evidence |
| **SecretReference** | Pointer to secret material without embedding the secret in code or docs. | [ADR-0296](decisions/ADR-0296-library-first-credential-sidecar.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS Secrets Manager ARN | credential sidecar |
| **Pack overlay** | Regional or compliance variant that activates policy, data, and runtime constraints. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Control Tower guardrail | compliance pack |
| **Capability tier** | Tiered exposure level limiting what a surface, plugin, or agent can do. | [ADR-0316](decisions/ADR-0316-capability-tier-over-product-fragmentation.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Salesforce entitlement | role projection |
| **Role projection** | UX and permit view derived from identity under tenant and role context. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Microsoft work profile | principal, persona |
| **Dual-tenant boundary** | Separation between personal and work tenant contexts for one human identity. | [ADR-0311](decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | managed Apple Account | tenant membership |
| **Conglomerate grant** | Delegated permission across parent, child, sovereign, and business-unit tenants. | [ADR-0313](decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS Organizations delegated admin | tenant hierarchy |
| **Audience type** | Declared audience class such as consumer, employee, partner, regulator, system, or agent. | [ADR-0244](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Google Workspace user type | role projection |
| **Policy fragment** | Composable Cedar rule unit published with soak and rollback metadata. | [ADR-0294](decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | OPA bundle fragment | Cedar permit |
| **Fragment soak** | Observation window before a Cedar fragment becomes promotable. | [ADR-0294](decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Envoy staged rollout | policy fragment |
| **Default deny** | Authorization posture where absence of explicit permission means refusal. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS IAM implicit deny | Cedar permit |
| **Provider credential mode** | Declared mode for provider credentials such as sidecar or network opt-in. | [ADR-0296](decisions/ADR-0296-library-first-credential-sidecar.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Workload Identity Federation | SecretReference |
| **Credential sidecar** | Isolated runtime holder for short-lived provider credentials. | [ADR-0296](decisions/ADR-0296-library-first-credential-sidecar.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SPIFFE sidecar | OpenBao TTL |
| **OpenBao TTL** | Short-lived lease applied to credentials and sensitive grants. | [ADR-0296](decisions/ADR-0296-library-first-credential-sidecar.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Vault dynamic secret lease | SecretReference |
| **Library-first dispatch** | In-process shared-library path used before network calls for substrate reads. | [ADR-0246](decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SDK local validation | policy engine |
| **Network opt-in** | Explicitly justified remote call path when local dispatch is insufficient. | [ADR-0246](decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS SDK service call | library-first dispatch |
| **Ontology read mode** | Declared path for reading ontology objects and freshness floors. | [ADR-0257](decisions/ADR-0356-amendment-library-first-ontology-read-path.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Foundry object set read | freshness floor |
| **Freshness floor** | Minimum accepted recency for ontology or workflow state before a decision. | [ADR-0257](decisions/ADR-0356-amendment-library-first-ontology-read-path.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Spanner bounded staleness | ontology read mode |
| **Time coordination tier** | HLC or TrueTime-like choice for ordering distributed actions. | [ADR-0252](decisions/ADR-0252-time-coordination-distributed-consistency.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Spanner TrueTime | HLC |
| **Hybrid logical clock** | Clock combining physical time with logical counters for causal ordering. | [ADR-0252](decisions/ADR-0252-time-coordination-distributed-consistency.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | CockroachDB HLC | time coordination |
| **Tenant membership** | Relationship between one identity and one tenant with scoped roles. | [ADR-0244](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Okta group membership | role projection |
| **Transient identity** | Time-boxed identity projection for apprentice, intern, resident, fellow, or extern roles. | [ADR-0320](decisions/ADR-0320-apprentice-intern-resident-fellow-transient-identity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | temporary access group | persona |
| **Meta-trust-root** | Root of trust attesting agentic self-modification and Foundry authority. | [ADR-0293](decisions/ADR-0293-governance-meta-trust-root.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SLSA provenance root | audit-chain |
| **Oya VCS ChangeSet** | Claimable, verifiable, bundleable, promotable unit of repository work. | [ADR-0223](decisions/ADR-0223-oya-git-drop-in-surface-with-explicit-policy-verbs.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Gerrit change plus PR | claim, promote |
| **ChangeBundle** | Promotion package grouping verified ChangeSets for controller-owned movement. | [ADR-0110](decisions/ADR-0110-changeset-state-machine.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | merge queue batch | ChangeSet |
| **Cell tier** | Deployment class controlling blast radius, compliance eligibility, and workload isolation. | [ADR-0248](decisions/ADR-0248-amazon-shape-cellular-architecture.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS cellular architecture | shuffle sharding |
| **Sovereign cell** | Cell operating under jurisdiction-specific legal, data, and operational constraints. | [ADR-0240](decisions/ADR-0240-sovereign-cloud-per-regional-pack.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Azure sovereign cloud | pack overlay |
| **Marketplace settlement** | Universal transaction, labor, partner, and business deal settlement surface. | [ADR-0314](decisions/ADR-0314-marketplace-as-universal-deal-settlement.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Stripe | audit-chain |
| **Collar-color workspace** | Projection axis covering white, blue, pink, gray, green, and gold collar work. | [ADR-0318](decisions/ADR-0318-collar-color-workspace-universality.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | frontline worker SKU | persona |
| **Information barrier** | Front, middle, and back office separation enforced by policy and audit. | [ADR-0319](decisions/ADR-0319-front-middle-back-office-information-barrier.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Purview information barrier | role projection |
| **Platform-owner indirection** | Rule that owner names are configurable and not hard-coded. | [ADR-0284](decisions/ADR-0284-platform-owner-name-indirection.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | white-label SaaS | canonical base |
| **Build-ahead certification** | Engineering posture where regulated controls ship before certification is granted. | [ADR-0250](decisions/ADR-0250-build-ahead-of-certification-doctrine.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | FedRAMP-ready program | compliance pack |
| **Abuse-defence baseline** | Anti-bot, anti-spoof, and anti-scrape controls for internet-facing surfaces. | [ADR-0297](decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Cloudflare Bot Management | WAF |
| **Emergency bypass** | Life-safety or continuity override with narrow scope and mandatory audit. | [ADR-0298](decisions/ADR-0298-emergency-services-bypass-life-safety.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | break-glass role | audit-chain |
| **Account recovery resilience** | Recovery model preserving one human identity without bypassing tenant policy. | [ADR-0299](decisions/ADR-0299-account-recovery-resilience.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | account recovery contact | dual-tenant boundary |
| **SAP FI** | SAP FI benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP FI | ERP, capability tier |
| **SAP CO** | SAP CO benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP CO | ERP, capability tier |
| **SAP MM** | SAP MM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP MM | ERP, capability tier |
| **SAP SD** | SAP SD benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP SD | ERP, capability tier |
| **SAP PP** | SAP PP benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP PP | ERP, capability tier |
| **SAP QM** | SAP QM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP QM | ERP, capability tier |
| **SAP PM** | SAP PM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP PM | ERP, capability tier |
| **SAP HCM** | SAP HCM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP HCM | ERP, capability tier |
| **SAP PS** | SAP PS benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP PS | ERP, capability tier |
| **SAP PLM** | SAP PLM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP PLM | ERP, capability tier |
| **SAP EHS** | SAP EHS benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP EHS | ERP, capability tier |
| **SAP SRM** | SAP SRM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP SRM | ERP, capability tier |
| **SAP CRM** | SAP CRM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP CRM | ERP, capability tier |
| **SAP SCM** | SAP SCM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP SCM | ERP, capability tier |
| **SAP GTS** | SAP GTS benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP GTS | ERP, capability tier |
| **SAP TM** | SAP TM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP TM | ERP, capability tier |
| **SAP EWM** | SAP EWM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP EWM | ERP, capability tier |
| **SAP TRM** | SAP TRM benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP TRM | ERP, capability tier |
| **SAP RE-FX** | SAP RE-FX benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP RE-FX | ERP, capability tier |
| **SAP IS-*** | SAP IS-* benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP IS-* | ERP, capability tier |
| **SAP SuccessFactors** | SAP SuccessFactors benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP SuccessFactors | ERP, capability tier |
| **SAP Ariba** | SAP Ariba benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP Ariba | ERP, capability tier |
| **SAP Concur** | SAP Concur benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP Concur | ERP, capability tier |
| **SAP BW** | SAP BW benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP BW | ERP, capability tier |
| **SAP S/4HANA** | SAP S/4HANA benchmark row for ERP parity; Oyatie maps the equivalent capability onto shared tenant, workflow, ontology, and audit primitives. | [ADR-0315](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SAP S/4HANA | ERP, capability tier |
| **Salesforce Sales Cloud** | Salesforce Sales Cloud benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Salesforce Sales Cloud | vendor benchmark, role projection |
| **Salesforce Service Cloud** | Salesforce Service Cloud benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Salesforce Service Cloud | vendor benchmark, role projection |
| **Salesforce AppExchange** | Salesforce AppExchange benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Salesforce AppExchange | vendor benchmark, role projection |
| **Workday HCM** | Workday HCM benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Workday HCM | vendor benchmark, role projection |
| **Workday Financials** | Workday Financials benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Workday Financials | vendor benchmark, role projection |
| **ServiceNow ITSM** | ServiceNow ITSM benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | ServiceNow ITSM | vendor benchmark, role projection |
| **ServiceNow HRSD** | ServiceNow HRSD benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | ServiceNow HRSD | vendor benchmark, role projection |
| **Atlassian Jira** | Atlassian Jira benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Atlassian Jira | vendor benchmark, role projection |
| **Atlassian Confluence** | Atlassian Confluence benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Atlassian Confluence | vendor benchmark, role projection |
| **Microsoft 365** | Microsoft 365 benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Microsoft 365 | vendor benchmark, role projection |
| **Microsoft Teams** | Microsoft Teams benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Microsoft Teams | vendor benchmark, role projection |
| **Microsoft Dynamics 365** | Microsoft Dynamics 365 benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Microsoft Dynamics 365 | vendor benchmark, role projection |
| **Adobe Creative Cloud** | Adobe Creative Cloud benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Adobe Creative Cloud | vendor benchmark, role projection |
| **Adobe Experience Cloud** | Adobe Experience Cloud benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Adobe Experience Cloud | vendor benchmark, role projection |
| **HubSpot CRM** | HubSpot CRM benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | HubSpot CRM | vendor benchmark, role projection |
| **Zendesk Support** | Zendesk Support benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Zendesk Support | vendor benchmark, role projection |
| **Snowflake** | Snowflake benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Snowflake | vendor benchmark, role projection |
| **Databricks** | Databricks benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Databricks | vendor benchmark, role projection |
| **Google Workspace** | Google Workspace benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Google Workspace | vendor benchmark, role projection |
| **Google Cloud IAM** | Google Cloud IAM benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Google Cloud IAM | vendor benchmark, role projection |
| **AWS IAM** | AWS IAM benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS IAM | vendor benchmark, role projection |
| **AWS Verified Permissions** | AWS Verified Permissions benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS Verified Permissions | vendor benchmark, role projection |
| **AWS Organizations** | AWS Organizations benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AWS Organizations | vendor benchmark, role projection |
| **Okta** | Okta benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Okta | vendor benchmark, role projection |
| **Stripe Connect** | Stripe benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Stripe | vendor benchmark, role projection |
| **GitHub Enterprise** | GitHub Enterprise benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GitHub Enterprise | vendor benchmark, role projection |
| **Palantir Foundry** | Palantir Foundry benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Palantir Foundry | vendor benchmark, role projection |
| **Cloudflare Zero Trust** | Cloudflare Zero Trust benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Cloudflare Zero Trust | vendor benchmark, role projection |
| **Slack Enterprise Grid** | Slack Enterprise Grid benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Slack Enterprise Grid | vendor benchmark, role projection |
| **Notion Workspace** | Notion Workspace benchmark row used to compare Oyatie coverage against established enterprise software and platform precedents. | [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Notion Workspace | vendor benchmark, role projection |
| **HIPAA** | HIPAA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | HIPAA | compliance pack, audit evidence |
| **GDPR** | GDPR compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR | compliance pack, audit evidence |
| **KR-PIPA** | KR-PIPA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-PIPA | compliance pack, audit evidence |
| **KR-FSS** | KR-FSS compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-FSS | compliance pack, audit evidence |
| **KR-CSAP** | KR-CSAP compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-CSAP | compliance pack, audit evidence |
| **CN-PIPL** | CN-PIPL compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | CN-PIPL | compliance pack, audit evidence |
| **JP-APPI** | JP-APPI compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | JP-APPI | compliance pack, audit evidence |
| **FedRAMP** | FedRAMP compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | FedRAMP | compliance pack, audit evidence |
| **PCI-DSS** | PCI-DSS compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | PCI-DSS | compliance pack, audit evidence |
| **EU-AI-Act** | EU-AI-Act compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU-AI-Act | compliance pack, audit evidence |
| **EU-NIS2** | EU-NIS2 compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU-NIS2 | compliance pack, audit evidence |
| **EU-DSA** | EU-DSA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU-DSA | compliance pack, audit evidence |
| **EU-MiFID-II** | EU-MiFID-II compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU-MiFID-II | compliance pack, audit evidence |
| **SOX** | SOX compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SOX | compliance pack, audit evidence |
| **Dodd-Frank** | Dodd-Frank compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | Dodd-Frank | compliance pack, audit evidence |
| **ISO-27001** | ISO-27001 compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | ISO-27001 | compliance pack, audit evidence |
| **SOC-2** | SOC-2 compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SOC-2 | compliance pack, audit evidence |
| **KSA-PDPL** | KSA-PDPL compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KSA-PDPL | compliance pack, audit evidence |
| **UAE-PDPL** | UAE-PDPL compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | UAE-PDPL | compliance pack, audit evidence |
| **SG-PDPA** | SG-PDPA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SG-PDPA | compliance pack, audit evidence |
| **AU-Privacy** | AU-Privacy compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AU-Privacy | compliance pack, audit evidence |
| **AU-IRAP** | AU-IRAP compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | AU-IRAP | compliance pack, audit evidence |
| **UK-AADC** | UK-AADC compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0292](decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | UK-AADC | compliance pack, audit evidence |
| **US-CCPA** | US-CCPA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | US-CCPA | compliance pack, audit evidence |
| **BR-LGPD** | BR-LGPD compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | BR-LGPD | compliance pack, audit evidence |
| **IN-DPDPA** | IN-DPDPA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | IN-DPDPA | compliance pack, audit evidence |
| **K-ISMS-P** | K-ISMS-P compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | K-ISMS-P | compliance pack, audit evidence |
| **KCMVP** | KCMVP compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KCMVP | compliance pack, audit evidence |
| **NIST SP 800-53** | NIST SP 800-53 compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | NIST SP 800-53 | compliance pack, audit evidence |
| **DORA** | DORA compliance-pack row that binds legal controls to pack overlays, sovereign cells, evidence, and onboarding review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | DORA | compliance pack, audit evidence |
| **GDPR Art 5** | GDPR Art 5 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 5 | regulatory article, compliance pack |
| **GDPR Art 6** | GDPR Art 6 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 6 | regulatory article, compliance pack |
| **GDPR Art 9** | GDPR Art 9 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 9 | regulatory article, compliance pack |
| **GDPR Art 15** | GDPR Art 15 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 15 | regulatory article, compliance pack |
| **GDPR Art 17** | GDPR Art 17 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 17 | regulatory article, compliance pack |
| **GDPR Art 20** | GDPR Art 20 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 20 | regulatory article, compliance pack |
| **GDPR Art 22** | GDPR Art 22 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 22 | regulatory article, compliance pack |
| **GDPR Art 28** | GDPR Art 28 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | GDPR Art 28 | regulatory article, compliance pack |
| **EU AI Act Art 13** | EU AI Act Art 13 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU AI Act Art 13 | regulatory article, compliance pack |
| **EU AI Act Art 14** | EU AI Act Art 14 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU AI Act Art 14 | regulatory article, compliance pack |
| **EU AI Act Art 15** | EU AI Act Art 15 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU AI Act Art 15 | regulatory article, compliance pack |
| **EU AI Act Annex III** | EU AI Act Annex III regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | EU AI Act Annex III | regulatory article, compliance pack |
| **KR-PIPA Art 15** | KR-PIPA Art 15 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-PIPA Art 15 | regulatory article, compliance pack |
| **KR-PIPA Art 17** | KR-PIPA Art 17 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-PIPA Art 17 | regulatory article, compliance pack |
| **KR-PIPA Art 21** | KR-PIPA Art 21 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-PIPA Art 21 | regulatory article, compliance pack |
| **KR-PIPA Art 28** | KR-PIPA Art 28 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | KR-PIPA Art 28 | regulatory article, compliance pack |
| **HIPAA 45 CFR 164.312** | HIPAA 45 CFR 164.312 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | HIPAA 45 CFR 164.312 | regulatory article, compliance pack |
| **HIPAA 45 CFR 164.308** | HIPAA 45 CFR 164.308 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | HIPAA 45 CFR 164.308 | regulatory article, compliance pack |
| **HIPAA 45 CFR 164.502** | HIPAA 45 CFR 164.502 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | HIPAA 45 CFR 164.502 | regulatory article, compliance pack |
| **PCI DSS Req 3** | PCI DSS Req 3 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | PCI DSS Req 3 | regulatory article, compliance pack |
| **PCI DSS Req 8** | PCI DSS Req 8 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | PCI DSS Req 8 | regulatory article, compliance pack |
| **FedRAMP AC** | FedRAMP AC regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | FedRAMP AC | regulatory article, compliance pack |
| **FedRAMP AU** | FedRAMP AU regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | FedRAMP AU | regulatory article, compliance pack |
| **FedRAMP SC** | FedRAMP SC regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | FedRAMP SC | regulatory article, compliance pack |
| **SOC 2 CC6** | SOC 2 CC6 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SOC 2 CC6 | regulatory article, compliance pack |
| **SOC 2 CC7** | SOC 2 CC7 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | SOC 2 CC7 | regulatory article, compliance pack |
| **ISO 27001 Annex A.5** | ISO 27001 Annex A.5 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | ISO 27001 Annex A.5 | regulatory article, compliance pack |
| **ISO 27001 Annex A.8** | ISO 27001 Annex A.8 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | ISO 27001 Annex A.8 | regulatory article, compliance pack |
| **UK AADC Standard 12** | UK AADC Standard 12 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | UK AADC Standard 12 | regulatory article, compliance pack |
| **CN PIPL Art 38** | CN PIPL Art 38 regulatory article row; implementers cite it when the exact article controls a workflow, data, AI, or audit behavior. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | CN PIPL Art 38 | regulatory article, compliance pack |
| **SLSA L3** | SLSA L3 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | SLSA L3 | hyperscaler precedent |
| **sigstore cosign** | sigstore cosign hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | sigstore cosign | hyperscaler precedent |
| **Rekor** | Rekor hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Rekor | hyperscaler precedent |
| **SPIFFE workload identity** | SPIFFE workload identity hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0295](decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | SPIFFE workload identity | hyperscaler precedent |
| **SPIRE** | SPIRE hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0295](decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | SPIRE | hyperscaler precedent |
| **mTLS** | mTLS hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | mTLS | hyperscaler precedent |
| **ECH RFC 9460** | ECH RFC 9460 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | ECH RFC 9460 | hyperscaler precedent |
| **PQC ML-KEM-768** | PQC ML-KEM-768 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | PQC ML-KEM-768 | hyperscaler precedent |
| **PQC ML-DSA-65** | PQC ML-DSA-65 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | PQC ML-DSA-65 | hyperscaler precedent |
| **FIPS 140-3 L3** | FIPS 140-3 L3 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | FIPS 140-3 L3 | hyperscaler precedent |
| **Kata Containers** | Kata Containers hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Kata Containers | hyperscaler precedent |
| **Cloud Hypervisor** | Cloud Hypervisor hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Cloud Hypervisor | hyperscaler precedent |
| **HTTP/3** | HTTP/3 hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | HTTP/3 | hyperscaler precedent |
| **QUIC** | QUIC hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | QUIC | hyperscaler precedent |
| **Alt-Svc** | Alt-Svc hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Alt-Svc | hyperscaler precedent |
| **TLS 1.3 floor** | TLS 1.3 floor hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | TLS 1.3 floor | hyperscaler precedent |
| **HSTS preload** | HSTS preload hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | HSTS preload | hyperscaler precedent |
| **Certificate Transparency** | Certificate Transparency hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Certificate Transparency | hyperscaler precedent |
| **OCSP stapling** | OCSP stapling hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | OCSP stapling | hyperscaler precedent |
| **DNS HTTPS RR** | DNS HTTPS RR hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | DNS HTTPS RR | hyperscaler precedent |
| **Shuffle-sharding** | Shuffle-sharding hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0248](decisions/ADR-0248-amazon-shape-cellular-architecture.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Shuffle-sharding | hyperscaler precedent |
| **Blast radius** | Blast radius hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0248](decisions/ADR-0248-amazon-shape-cellular-architecture.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Blast radius | hyperscaler precedent |
| **Control plane** | Control plane hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Control plane | hyperscaler precedent |
| **Data plane** | Data plane hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Data plane | hyperscaler precedent |
| **OpenTelemetry trace** | OpenTelemetry trace hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | OpenTelemetry trace | hyperscaler precedent |
| **Cardinality budget** | Cardinality budget hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Cardinality budget | hyperscaler precedent |
| **SLO burn rate** | SLO burn rate hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | SLO burn rate | hyperscaler precedent |
| **Canary** | Canary hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Canary | hyperscaler precedent |
| **Blue-green** | Blue-green hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Blue-green | hyperscaler precedent |
| **Dark launch** | Dark launch hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Dark launch | hyperscaler precedent |
| **SBOM** | SBOM hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | SBOM | hyperscaler precedent |
| **Trivy** | Trivy hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Trivy | hyperscaler precedent |
| **Argo CD** | Argo CD hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Argo CD | hyperscaler precedent |
| **Argo Rollouts** | Argo Rollouts hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Argo Rollouts | hyperscaler precedent |
| **Istio Ambient** | Istio Ambient hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Istio Ambient | hyperscaler precedent |
| **Envoy Gateway** | Envoy Gateway hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Envoy Gateway | hyperscaler precedent |
| **OpenBao** | OpenBao hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | OpenBao | hyperscaler precedent |
| **Fulcio** | Fulcio hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Fulcio | hyperscaler precedent |
| **in-toto attestation** | in-toto attestation hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0039](decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | in-toto attestation | hyperscaler precedent |
| **Workload kill-switch** | Workload kill-switch hyperscaler-grade engineering term; implementations cite it when matching the transport, supply-chain, identity, deployment, telemetry, or resilience precedent. | [ADR-0295](decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Workload kill-switch | hyperscaler precedent |
| **Ontology projection** | Ontology projection architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Role-based projection** | Role-based projection architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Audit-event class** | Audit-event class architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Sovereign-cloud overlay** | Sovereign-cloud overlay architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Cellular architecture** | Cellular architecture architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Tenant-scoped row** | Tenant-scoped row architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Data residency** | Data residency architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Portability export** | Portability export architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Minor user tier** | Minor user tier architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Human oversight** | Human oversight architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Model lifecycle** | Model lifecycle architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0308](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Detection substrate** | Detection substrate architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Investigation case** | Investigation case architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Court-warrant piercing** | Court-warrant piercing architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Delegated agent authority** | Delegated agent authority architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Disaster mode** | Disaster mode architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Survivor safety mode** | Survivor safety mode architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Deceased-user inheritance** | Deceased-user inheritance architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Cognitive-impairment resilience** | Cognitive-impairment resilience architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Whistleblower anonymity** | Whistleblower anonymity architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Canonical base** | Canonical base architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Korea localization pack** | Korea localization pack architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Tenant RBAC view** | Tenant RBAC view architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Ops Dashboard** | Ops Dashboard architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Control Center** | Control Center architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Foundry** | Foundry architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Workflow Studio** | Workflow Studio architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Capability record** | Capability record architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Admission gate** | Admission gate architectural vocabulary row used by onboarding, implementation packets, and review evidence. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | platform architecture precedent | architecture, review |
| **Persona: Dossier — Aiyana Singh** | Named persona archetype from `personas/aiyana-singh.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Anya Mironova** | Named persona archetype from `personas/anya-mironova.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Benefits Specialist Aoife Murphy** | Named persona archetype from `personas/benefits-specialist-aoife-murphy.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Board director Patrick O'Reilly** | Named persona archetype from `personas/board-director-patrick-oreilly.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Captain Chen** | Named persona archetype from `personas/captain-chen-pilot.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Carlos Martinez** | Named persona archetype from `personas/carlos-martinez-forklift.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — CEO Aoki Tanaka** | Named persona archetype from `personas/ceo-aoki-tanaka.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — CFO Helena Brandt** | Named persona archetype from `personas/cfo-helena-brandt.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Chris Volkov** | Named persona archetype from `personas/chris-volkov.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — CHRO Linda Foster** | Named persona archetype from `personas/chro-linda-foster.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — CISO Yuki Park** | Named persona archetype from `personas/ciso-yuki-park.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Diana Reyes** | Named persona archetype from `personas/diana-reyes.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Dr. Tanaka** | Named persona archetype from `personas/dr-tanaka-surgeon.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Father Lopez** | Named persona archetype from `personas/father-lopez-priest.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Hiroshi Tanaka** | Named persona archetype from `personas/hiroshi-tanaka.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Investment Banker Yuna Ahn** | Named persona archetype from `personas/investment-banker-yuna-ahn.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Marcus Chen** | Named persona archetype from `personas/marcus-chen.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Medical Resident Dr. Sun-Mi Kim** | Named persona archetype from `personas/medical-resident-dr-sun-mi-kim.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Ms. Patel** | Named persona archetype from `personas/ms-patel-teacher.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Officer Rodriguez** | Named persona archetype from `personas/officer-rodriguez-police.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Outside Counsel Wei-Yi Chen** | Named persona archetype from `personas/outside-counsel-wei-yi-chen.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Priya Krishnan** | Named persona archetype from `personas/priya-krishnan.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Regulator Inspector Sergei Petrov** | Named persona archetype from `personas/regulator-inspector-sergei-petrov.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Sam Okafor** | Named persona archetype from `personas/sam-okafor.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Sarah Kim** | Named persona archetype from `personas/sarah-kim-delivery.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Summer Intern Priscilla Sharma** | Named persona archetype from `personas/summer-intern-priscilla-sharma.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Tomás García Jr.** | Named persona archetype from `personas/tomas-garcia-jr-farmer.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Tomás García** | Named persona archetype from `personas/tomas-garcia.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Trader Mei Lin** | Named persona archetype from `personas/trader-mei-lin.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **Persona: Dossier — Yejin Park** | Named persona archetype from `personas/yejin-park.md`; treated as an identity projection over tenant, role, locale, device, and skill tier. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) | enterprise persona profile | persona, role projection |
| **ADR-0242 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0242](decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0243 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0243](decisions/ADR-0243-cedar-as-universal-gate.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0244 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0244](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0245 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0246 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0246](decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0247 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0247](decisions/ADR-0247-self-hosting-self-modification-doctrine.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0248 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0248](decisions/ADR-0248-amazon-shape-cellular-architecture.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0249 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0249](decisions/ADR-0249-multi-category-marketplace-doctrine.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0250 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0250](decisions/ADR-0250-build-ahead-of-certification-doctrine.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0251 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0252 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0252](decisions/ADR-0252-time-coordination-distributed-consistency.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0253 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0254 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0254](decisions/ADR-0254-deployment-model-spectrum.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0255 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0255](decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0257 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0257](decisions/ADR-0356-amendment-library-first-ontology-read-path.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0258 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0258](decisions/ADR-0258-api-versioning-model.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0263 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0273 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0273](decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0276 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0276](decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0280 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0280](decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0284 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0284](decisions/ADR-0284-platform-owner-name-indirection.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0292 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0292](decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0293 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0293](decisions/ADR-0293-governance-meta-trust-root.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0294 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0294](decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0295 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0295](decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0296 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0296](decisions/ADR-0296-library-first-credential-sidecar.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0311 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0311](decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0313 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0313](decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0316 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0316](decisions/ADR-0316-capability-tier-over-product-fragmentation.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **ADR-0317 doctrine** | Keystone decision row summarized in the doctrine bootcamp and used as a binding source for review. | [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | architecture decision record | keystone bundle |
| **AADC** | Age Appropriate Design Code acronym used by UK child-safety controls and minor-user onboarding review. | [ADR-0292](decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | UK ICO children's code | UK-AADC, minor user tier |
| **CANNOT** | Source-quoted uppercase refusal word; new normative docs SHOULD prefer RFC-2119 `MUST NOT`. | [ADR-0212](decisions/ADR-0212-buildability-doctrine.md) | [documentation-rigor.md](standards/documentation-rigor.md) | RFC-2119 discipline | MUST NOT, doc-style |
| **COPPA** | US child privacy statute acronym used by minor-user refusal and consent-tier controls. | [ADR-0292](decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | US children's privacy law | KOSA, UK-AADC |
| **DKIM** | DomainKeys Identified Mail acronym used by per-tenant email deliverability controls. | [ADR-0273](decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | email signing control | SPF, DMARC |
| **DMARC** | Domain-based Message Authentication, Reporting, and Conformance acronym for tenant mail policy. | [ADR-0273](decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | email authentication policy | DKIM, SPF |
| **ECH** | Encrypted Client Hello acronym for TLS client hello confidentiality where supported. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | Cloudflare ECH deployment | TLS 1.3, DNS HTTPS RR |
| **GLOSSARY** | Uppercase filename token for the canonical glossary file; term semantics live in `docs/GLOSSARY.md`. | [ADR-0212](decisions/ADR-0212-buildability-doctrine.md) | [documentation-rigor.md](standards/documentation-rigor.md) | reference documentation hub | glossary, term row |
| **HLC** | Hybrid Logical Clock acronym for causal time coordination. | [ADR-0252](decisions/ADR-0252-time-coordination-distributed-consistency.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | CockroachDB HLC | time coordination tier |
| **KOSA** | Kids Online Safety Act acronym used by minor-user doctrine and age-tier controls. | [ADR-0292](decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | online safety statute | COPPA, UK-AADC |
| **PQC** | Post-quantum cryptography acronym for hybrid key exchange and signature posture. | [ADR-0253](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | NIST post-quantum migration | ML-KEM, ML-DSA |
| **SET** | Source-quoted uppercase token from ADR summaries; new docs SHOULD expand or lowercase unless it is a protocol name. | [ADR-0212](decisions/ADR-0212-buildability-doctrine.md) | [documentation-rigor.md](standards/documentation-rigor.md) | glossary vocabulary hygiene | acronym citation |
| **SPF** | Sender Policy Framework acronym used by per-tenant mail deliverability controls. | [ADR-0273](decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | email sender authorization | DKIM, DMARC |
| **SPIFFE** | Secure Production Identity Framework For Everyone acronym for workload identity. | [ADR-0295](decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) | [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) | workload identity standard | SPIRE, mTLS |
| **UK** | United Kingdom jurisdiction acronym used by UK-AADC and related pack overlays. | [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) | [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) | jurisdiction code | UK-AADC, compliance pack |
| **URI** | Uniform Resource Identifier acronym used by source paths, identifiers, and external references. | [ADR-0258](decisions/ADR-0258-api-versioning-model.md) | [documentation-rigor.md](standards/documentation-rigor.md) | IETF URI term | URL, contract versioning |

### 14.1 Maintenance rule

A new canonical document that introduces any term in this appendix MUST cite the Binding ADR column and SHOULD cite the Binding doc column.
If a term changes meaning, update the row in place and add audit-chain evidence in the pull request.
If a term is replaced, move the old spelling to the deprecated-term section and point to the replacement row.

<!-- codex-glossary-onboarding:end -->

## 15. Deepened glossary entries - 2026-05-20

These entries deepen thin rows above and add corpus-wide terms that were used in
three or more documents without a standalone glossary entry. The appendix rows
remain index rows; this section is canonical for prose, specs, and review gates
that need the substance behind a term.

### Cedar

Definition: Cedar is Oyatie's default-deny authorization policy engine and the policy language used to evaluate tenant, principal, audience, action, resource, and data-class context before protected behavior runs.

In Oyatie, Cedar is not a decorative policy layer after business logic. ADR-0243 makes Cedar the universal gate, and ADR-0294 defines fragment soak, anomaly detection, and rollback so policy changes can move through the system without becoming silent production experiments.

Every Cedar decision must carry enough context to be auditable: `tenant_id`, `principal_id`, `audience_type`, resource identity, action, data classification, and the compliance-pack overlay that shaped evaluation. Missing context means deny by default.

Named µservices using the term include `identity`, `governance`, `tenancy`, `policy-engine`, `intelligence`, and `cloud-secrets`. Identity uses Cedar fragments for tenant and role projection, Governance owns fragment lifecycle, and Intelligence uses Cedar to keep consumer AI access separate from internal Foundry authority.

Related terms: Cedar permit; policy fragment; fragment soak; default deny; `audience_type`; audit-event class; compliance pack.

Authority citation: ADR-0243 "Cedar as universal gate"; ADR-0294 "Cedar fragment soak anomaly rollback"; Cedar policy language / AWS Verified Permissions documentation.

### audience_type

Definition: `audience_type` is the ADR-0244 closed enum that classifies what kind of audience a tenant represents, such as consumer, business, reseller, or conglomerate scope.

The name is deliberately not just `audience`: ADR-0244 records that `audience_type` is the durable PostgreSQL enum and manifest replacement for vague per-service audience fields. It lets the tenant model describe the type of audience while preserving tenant identity as the universal scoping primitive.

`audience_type` travels with authorization and routing context. Cedar decisions, tenant lifecycle events, compliance-pack activation, role projection, provider-BYOK eligibility, and cross-tenant sharing flows all need it to avoid guessing from product names or UI surfaces.

Named µservices using the term include `identity`, `tenancy`, `governance`, `workflow-engine`, `marketplace`, and `compliance`. Identity requires it in evaluation context, Tenancy owns the tenant lifecycle source, and Governance consumes it for policy overlays.

Related terms: tenant; principal; Cedar; role projection; compliance pack; conglomerate-tenant; capability tier.

Authority citation: ADR-0244 "Tenant as universal scoping primitive", especially the appendix definition for the closed-enum tenant classification.

### Compliance pack

Definition: A compliance pack is a governed bundle of regulatory policy, evidence, residency, certification, and product-overlay requirements that can be activated for a tenant, cell, or capability tier.

Compliance packs are the formal way Oyatie turns jurisdictional or industry requirements into executable platform constraints. They are not just document folders; they drive Cedar overlays, allowed data classes, audit evidence, cell certification eligibility, localization obligations, retention, and reporting posture.

ADR-0251 defines pack-to-cell certification levels, while `specs/compliance-pack-schema.json` defines the machine-readable shape. Packs may be product-local, such as Korea localization requirements, or platform-wide, such as data-residency and regulated-workload overlays.

Named µservices using the term include `compliance`, `tenancy`, `governance`, `cloud-secrets`, `identity`, `workflow-studio`, and `intelligence`. Tenancy activates packs, Governance maps them to policy, Cloud Secrets applies encryption constraints, and product services expose pack-aware UX or workflow defaults.

Related terms: pack overlay; Cedar; `audience_type`; cell; capability tier; audit-event class; BYOK.

Authority citation: ADR-0251 "Compliance pack cell certification levels"; `specs/compliance-pack-schema.json`; Korea localization pack documents.

### Shuffle-sharding

Definition: Shuffle-sharding is the ADR-0248 tenant-to-cell placement pattern that assigns tenants to small overlapping cell sets so one tenant or cell failure has bounded blast radius.

Oyatie uses shuffle-sharding as a reliability and isolation primitive, not only as a load-balancing trick. ADR-0248 adopts the Amazon cellular-architecture shape and cites AWS shuffle-sharding practice so tenant placement can tolerate noisy neighbors, degraded cells, and partial regional failures.

The pattern depends on a stable shard key, cell eligibility, and a configured shuffle width. A tenant gets a deterministic set of cells; unrelated tenants are unlikely to share the exact same set, which reduces correlated failure while keeping operations measurable.

Named µservices using the term include `cell`, `cloud-iac`, `tenancy`, `traffic-router`, `control-plane`, and `observability`. `cloud-iac` carries cell eligibility and shuffle-width defaults, while `cell` and `tenancy` coordinate assignment, migration, and decommissioning.

Related terms: cell; sovereign cell; cell tier; tenant; audit-event class; capability tier.

Authority citation: ADR-0248 "Amazon-shape cellular architecture"; Colm MacCarthaigh, AWS Architecture Blog, shuffle sharding guidance.

### HLC / TrueTime

Definition: HLC is Oyatie's default hybrid logical clock for distributed ordering; TrueTime is the bounded-uncertainty clock tier reserved for workloads that need externally consistent timestamp semantics.

ADR-0252 chooses a dual time-coordination tier: HLC everywhere by default, TrueTime only where the cost and operational requirements are justified. This keeps ordinary cells portable while still allowing Spanner-class consistency for the small set of flows that truly require it.

HLC combines physical time with a logical counter so event ordering survives skew without treating wall-clock time as authority. TrueTime adds an uncertainty interval around clock reads, which lets qualified storage or coordination paths wait out uncertainty before committing externally visible order.

Named µservices using the term include `audit-chain`, `identity`, `workflow-engine`, `observability`, `cloud-iac`, and the shared `oya-shared-time-kernel` crate. Audit Chain uses HLC ordering for tamper-evident emissions, while infrastructure exposes TrueTime provider wiring only for eligible cells.

Related terms: audit-event class; cell; replay safety; observability; time coordination tier; sovereign cell.

Authority citation: ADR-0252 "Time coordination distributed consistency"; Hybrid Logical Clocks, OPODIS 2014; Google Spanner / TrueTime, OSDI 2012.

### MLS

Definition: MLS means Messaging Layer Security, the IETF RFC 9420 protocol Oyatie uses for group key agreement and end-to-end encrypted messaging state.

MLS protects message content and group key evolution; it does not replace tenant authorization. Cedar still decides whether a tenant, principal, device, or cross-tenant channel may participate, while MLS protects the encrypted communication once membership is allowed.

Oyatie uses MLS where collaboration needs scalable group encryption instead of pairwise ad hoc key exchange. Messenger owns the primary MLS message path, Meet uses it for end-to-end encrypted sessions, Notes applies it to collaborative editing hardening, and uses it for cross-tenant channels.

Named µservices using the term include `messenger`, `meet`, `notes`, `connector`, `identity`, and `audit-chain`. Identity and Audit Chain do not own cryptographic content, but they anchor device identity, group creation evidence, and membership-change audit records.

Related terms: Cedar; principal; tenant; audit-event class; cell; compliance pack.

Authority citation: RFC 9420 "The Messaging Layer Security (MLS) Protocol"; messenger ADR-MSG-001; meet IP-012.

### BYOK

Definition: BYOK means Bring Your Own Key, but Oyatie splits it into provider-BYOK and encryption-BYOK because those are different trust boundaries with different operators, risks, and audit duties.

Provider-BYOK lets a tenant bring or reference provider credentials, such as AI-provider keys or external service credentials, for workloads that call a third-party provider. The credential gates provider use; it does not become the platform encryption root.

Encryption-BYOK lets a tenant bring or control cryptographic key material, usually through KMS or envelope-encryption integration, for data protection and retention workflows. This governs encryption posture and key rotation; it does not grant access to external AI providers.

Named µservices using the term include `cloud-secrets`, `intelligence`, `network`, `payments`, `identity`, and `governance`. Cloud Secrets owns encryption-key references and rotation runbooks, Intelligence consumes provider-BYOK for model/provider access, and Identity keeps both meanings explicit in tenant context.

Related terms: SecretReference; compliance pack; Cedar; `audience_type`; cloud secrets; provider credential; encryption key.

Authority citation: ADR-0255 D-4 and ADR-0251 D-10 BYOK disambiguation; `oya-governance-byok-disambiguation` crate; BYOK rotation runbooks.

### Cell

Definition: A cell is an independently operable, failure-bounded deployment unit used to isolate tenants, workloads, compliance packs, and blast radius under ADR-0248.

Cells are the basic unit of Oyatie's cellular architecture. They combine routing, placement, eligibility, observability, and operational controls so a tenant can be assigned, migrated, degraded, or recovered without assuming the whole platform is one shared fate domain.

ADR-0248 defines multiple tiers, including sovereign cells for jurisdictional or customer-controlled environments. Cell assignment interacts with shuffle-sharding, pack certification, HLC/TrueTime availability, and capability-tier eligibility.

Named µservices using the term include `cell`, `cloud-iac`, `tenancy`, `traffic-router`, `observability`, `audit-chain`, and `control-plane`. The `cell` service owns assignment and migration APIs, while Cloud IAC and Traffic Router turn those assignments into deployable infrastructure and routing behavior.

Related terms: shuffle-sharding; sovereign cell; cell tier; compliance pack; HLC / TrueTime; capability tier.

Authority citation: ADR-0248 "Amazon-shape cellular architecture"; ADR-0252 time coordination; Cloud IAC cell eligibility documentation.

### Ontology

Definition: Ontology is Oyatie's tenant-scoped object, relation, action, and projection model for business meaning, similar in intent to Palantir Foundry Ontology but governed as an Oyatie substrate.

The ontology is not only a search index, schema registry, or knowledge graph. It defines the durable object types and relationships that workflows, intelligence, marketplace integrations, and product surfaces can reason about while respecting tenant, policy, and pack boundaries.

ADR-0257 moves the read path toward a library-first ontology model, and `specs/products/ontology.json` defines the product surface. The ontology can support consumer-facing Intelligence and Workflow Studio without routing those experiences through internal Foundry.

Named µservices using the term include `ontology`, `workflow-engine`, `workflow-studio`, `intelligence`, `marketplace`, and `tenancy`. Workflow Engine reads ontology projections for execution context, while Intelligence uses permitted ontology context for retrieval, attribution, and safe response construction.

Related terms: workflow studio; intelligence; capability tier; Cedar; tenant; ontology projection; Foundry.

Authority citation: ADR-0257 "Library-first ontology read path"; `specs/products/ontology.json`; ADR-0220 consumer intelligence boundary.

### Workflow Studio

Definition: Workflow Studio is Oyatie's n8n-class visual workflow authoring surface for composing governed automations over Oyatie services, templates, triggers, and ontology-aware actions.

Workflow Studio is the authoring product; Workflow Engine is the execution substrate. Keeping those terms separate prevents UI, template migration, and human editing concerns from leaking into the runtime engine's determinism, policy checks, and audit obligations.

The Studio imports and maps external workflow concepts, including n8n-style migrations, into Oyatie's governed primitives. Templates, connections, human approvals, pack overlays, and capability-tier limits must compile into executable definitions that Workflow Engine can verify and audit.

Named µservices using the term include `workflow-studio`, `workflow-engine`, `ontology`, `governance`, `audit-chain`, and `marketplace`. Workflow Studio owns authoring UX and template migration; Governance and Audit Chain make the resulting definitions policy-bound and evidence-bearing.

Related terms: workflow; ontology; capability tier; Cedar; audit-event class; compliance pack.

Authority citation: `specs/microservices/workflow-studio.json`; Workflow Studio PRD and architecture; ADR-0244 tenant-scoped sharing example.

### Foundry (RETIRED)

Status: RETIRED 2026-05-21 per ADR-0335 (Wave 15I). The `foundry` µservice is no longer live authority.

Historical definition (pre-retirement): Foundry was Oyatie's internal platform for agentic development, evaluation, evidence collection, and engineering operations — distinct from the consumer AI product surface.

Retirement: ADR-0136 originally consolidated Foundry as one µservice with internal bounded contexts. ADR-0220 amended the product taxonomy to assign consumer-facing AI capability to Intelligence. ADR-0255 KS#14 (intelligence two-layer AI substrate) then established intelligence as the canonical AI substrate that absorbs Foundry. ADR-0335 (Wave 15I) executes that absorption and retires the Foundry µservice.

Successor: `microservices/intelligence/` — the canonical AI substrate. Layer A covers model routing, providers, guardrails, eval, attribution, audit-tap, credential resolver, assist-draft, and context-aware retrieval. Layer B covers the consumer brand UX surface. Self-modification execution remains under the `oyatie.foundry.*` Cedar principal namespace per ADR-0247 (the Cedar principal namespace persists even though the µservice retires).

retired external agent harness terminology: The "retired external agent harness" brand name for the internal pipeline is RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36. No replacement is needed; the underlying capability is now "intelligence" (consumer AI) or "oyatie.foundry workflow library inside dev-tools-cell-N" (self-modification).

Crate transition debt: Existing `oya-foundry-*` workspace crates are retained as transition debt per ADR-0335 D-37..D-50 (following the ADR-0333 D-59 precedent). New AI substrate code lands under `oya-intelligence-*`.

Related terms: intelligence; ontology; workflow studio; Cedar; audit-event class; capability tier; oyatie.foundry.* (Cedar principal namespace).

Authority citation: ADR-0335 (retirement); ADR-0255 (intelligence two-layer substrate); ADR-0247 (self-modification); ADR-0136 (historical consolidation); ADR-0220 (historical scope clarification); ADR-0239 (historical internal-only amendment).

### Intelligence

Definition: Intelligence is the canonical Oyatie AI substrate µservice per ADR-0255 KS#14 (intelligence two-layer AI substrate). It provides Layer A (model routing, providers, guardrails, eval, attribution, audit-tap, credential resolver, assist-draft, context-aware retrieval) and Layer B (consumer brand UX surface).

ADR-0220 originally created the Intelligence boundary to separate consumer AI from the internal Foundry pipeline. ADR-0255 KS#14 then named Intelligence as the canonical AI substrate that absorbs Foundry. ADR-0335 (Wave 15I) executes that absorption and retires the Foundry µservice.

Intelligence must respect Cedar, `audience_type`, provider-BYOK, compliance packs, ontology permissions, and audit-event classes. It can use shared substrates, but those shared substrates must be explicit and policy-bound instead of implied by naming overlap.

Named µservices using the term include `intelligence`, `ontology`, `workflow-studio`, `governance`, `cloud-secrets`, and `audit-chain`. Intelligence owns model routing and guardrails, Cloud Secrets resolves provider credentials, and Audit Chain records AI-use evidence.

Related terms: Foundry (RETIRED); ontology; BYOK; Cedar; capability tier; audit-event class; oyatie.foundry.* (Cedar principal namespace for self-modification).

Authority citation: ADR-0255 "Intelligence two-layer AI substrate"; ADR-0335 "Foundry retired absorbed by intelligence"; ADR-0220 "Consumer intelligence substrate" (historical); Intelligence architecture; microservices/intelligence manifest.

### Conglomerate-tenant

Definition: A conglomerate-tenant is a parent tenant relationship model where child tenants remain sovereign tenants and parent access is represented through explicit grants, not by collapsing tenant boundaries.

ADR-0313 rejects treating subsidiaries, brands, regions, or acquisitions as soft sub-scopes inside one parent tenant. Each child stays a full ADR-0244 tenant with its own policy, data, packs, and operational controls.

Parent visibility or administration is expressed through `conglomerate_grants`, Cedar permits, and revocation. Reorganizations should revoke or add grants, not migrate child data or rewrite tenant identity.

Named µservices using the term include `tenancy`, `governance`, `identity`, `marketplace`, `workflow-engine`, and `audit-chain`. Tenancy owns the hierarchy and grants, Governance evaluates inherited or delegated permissions, and Audit Chain records grant changes.

Related terms: tenant; `audience_type`; conglomerate grant; Cedar; role projection; compliance pack.

Authority citation: ADR-0313 "Conglomerate tenant hierarchy sovereign children"; ADR-0244 tenant doctrine.

### Audit-event class

Definition: An audit-event class is the governed taxonomy label for a state-changing event that must be emitted, linked, and retained as part of Oyatie's audit-chain and observability contract.

ADR-0263 makes emission shape and audit linkage part of the platform contract. Mutating actions must choose an approved event class, carry required tenant and principal labels, and link to an `audit_id` so evidence can survive service-local logging differences.

Audit-event classes are not free-form log messages. They are registry-backed names, usually namespaced by service and context, that let policy, monitoring, compliance, and replay tooling reason about what happened without parsing prose.

Named µservices using the term include `audit-chain`, `observability`, `governance`, `identity`, `workflow-engine`, `payments`, and `cloud-secrets`. Audit Chain stores tamper-evident records, Observability enforces emission contracts, and services register class names before mutating behavior ships.

Related terms: audit-chain; Cedar; HLC / TrueTime; compliance pack; emission contract; done primitive.

Authority citation: ADR-0263 "Observability emission contract"; Audit Chain architecture; Observability audit-class registry rules.

### Claim / work / verify / done / promote primitives

Definition: Claim, work, verify, done, and promote are the Oya VCS lifecycle primitives used to reserve scope, perform edits, attach evidence, close a changeset, and advance a verified bundle.

`claim` reserves the intended scope before editing so concurrent agents can detect collisions. `work` is the edit phase under the claim, even when no separate command is required. `verify` attaches concrete evidence that the change satisfied its checks.

`done` marks the claimed slice complete with evidence, and `promote` advances the verified changeset or bundle toward an environment or merge queue. These verbs make agent work auditable without depending on retired external coordination tools.

Named µservices and surfaces using the terms include `foundry`, `governance`, `audit-chain`, `developer-sdk`, `ci`, and the repository `bin/oya vcs` control surface. The lifecycle is especially important for documentation, policy, and master-plan slices where concurrency safety matters.

Related terms: Oya VCS ChangeSet; ChangeBundle; claim ceiling; merge queue; audit-event class; Foundry.

Authority citation: ADR-0116 external tooling retirement; ADR-0110 ChangeSet state machine; project AGENTS required sequence for Oya VCS.

### Valkey

Definition: Valkey is the canonical Oyatie in-memory key-value, cache, pubsub, and streams substrate per ADR-0336. It is the Linux Foundation BSD-3-Clause fork of Redis 7.2.4 (the last BSD-3-Clause release before the Redis Inc. SSPLv1 / RSALv2 dual relicense of 2024-03-20). Valkey 8.x is the active mainline.

Valkey preserves the RESP3 wire protocol, command surface, cluster slot mapping, sentinel protocol, RDB and AOF persistence formats, and replication semantics from pre-relicense Redis verbatim, so existing Rust client crates (`redis-rs`, `fred`, `deadpool-redis`) work unchanged. Cluster topology, encryption-at-rest, audit-emission, and TLS posture are preserved across the substrate swap.

Hyperscaler-managed offerings: AWS ElastiCache for Valkey (GA 2024-11-04, 20% lower instance pricing than ElastiCache for Redis OSS at equivalent instance class); Google Memorystore for Valkey (GA 2024-09-24); Oracle Cloud Cache with Valkey (GA 2025-01-21, integrated with OCI Always Free perpetual tier).

Crate naming: `oya-<microservice>-adapter-valkey[-<topology>]` per ADR-0336 §D-1. IaC modules at `iac/<context>/valkey/` per ADR-0336 §D-2. Environment variables `VALKEY_URL`, `VALKEY_CLUSTER_ENDPOINTS`, `VALKEY_TLS_CERT_PATH`, etc., per ADR-0336 §D-5. Cedar entity types `ValkeyCluster::"<id>"`, `ValkeyKey::"<pattern>"`, `ValkeyChannel::"<name>"`, `ValkeyStream::"<name>"` per ADR-0336 §D-8. Audit-chain event classes `valkey.*` per ADR-0336 §D-10. Metric label `substrate="valkey"` per ADR-0336 §D-9.

Counterpart-fact preservation: factual references to Redis-based external products (e.g., "Discord uses Redis Cluster for session state", "Twitch uses Redis for chat fanout") remain preserved quote-bound per ADR-0336 §D-11; these are NOT migrated to Valkey because the counterpart product factually uses Redis.

Related terms: Redis (RETIRED — see §11); Memcached (pure-cache alternative without pubsub / streams / transactional semantics); RESP3; OCI Always Free; cell tier; compliance pack.

Authority citation: ADR-0336 "Valkey is the canonical in-memory KV / cache / pubsub substrate"; ADR-0211 in-house tech stack preference (Class C OSS substrate); ADR-0212 buildability doctrine; ADR-0013 + ADR-0045 license substitution precedents; `docs/standards/dependency-policy.md` §2.1 + §7 substitution tables; `feedback_valkey_not_redis_2026_05_21` user directive.

### Redis (RETIRED in favor of Valkey)

Status: RETIRED as a canonical Oyatie substrate 2026-05-21 per ADR-0336. The Redis 7.4+ substrate is forbidden by the existing dependency-policy §2 because Redis Inc. relicensed Redis on 2024-03-20 from BSD-3-Clause to a dual SSPLv1 / RSALv2 license. SSPL is not OSI-approved due to §13 viral copyleft on the operational stack; RSAL is a source-available license that explicitly prohibits managed-service competition with Redis Inc.

Historical definition (pre-retirement): Redis was Oyatie's prior in-memory KV / cache / pubsub / streams substrate, used by messenger, community, workflow-engine, intelligence, and ~15 other µservices for session state, caching, fanout, and streams workloads.

Retirement reason: license drift (SSPL/RSAL forbidden per `docs/standards/dependency-policy.md` §2) + hyperscaler alignment (AWS / Google / Oracle Cloud all shipped managed Valkey offerings in 2024-2025) + Bominal-inheritance precedence (Bominal corpus follows under separate migration).

Pre-7.4 Redis (BSD-3-Clause) remains license-clean but is non-canonical due to absent upstream maintenance (no Redis Inc. security patches on the BSD branch) and absent hyperscaler-managed offering (no AWS / Google / Oracle Cloud managed pre-7.4 Redis service).

Successor: Valkey (see preceding entry).

Counterpart-fact preservation: factual references to Redis-based external products remain preserved quote-bound per ADR-0336 §D-11.

Authority citation: ADR-0336 "Valkey is the canonical in-memory KV / cache / pubsub substrate"; `docs/standards/dependency-policy.md` §2 forbidden-license list (SSPL, RSAL); `feedback_valkey_not_redis_2026_05_21` user directive.

### Capability tier

Definition: A capability tier is a named, tenant-visible projection bundle that exposes a product capability level over shared substrate primitives without creating fragmented product forks.

ADR-0316 replaces product fragmentation with capability tiers. A tier activates a coherent bundle: Cedar permits, ontology projection or workflow-only declaration, workflow templates or read-only declaration, UX shell manifest, compliance overlay mapping, schema revisions, audit evidence, and cost dimensions.


Named µservices using the term include `workflow-studio`, `intelligence`, `cloud-secrets`, `developer-sdk`, `governance`, and `marketplace`. Their tier matrices define what a tenant receives without duplicating service implementations.

Related terms: compliance pack; Cedar; ontology; Workflow Studio; Intelligence; audit-event class; `audience_type`.

Authority citation: ADR-0316 "Capability tier over product fragmentation"; service capability-tier matrices for Workflow Studio, Intelligence, Cloud Secrets, and Developer SDK.
