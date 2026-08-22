---
purpose: Oyatie — Competitive Gap Analysis
doc_status: published
---

# Oyatie — Competitive Gap Analysis

> **Status:** Draft v0.1 — 2026-05-09. Authored per user directive: identify product/service areas where Oyatie lacks competitive edge or maturity, propose concrete expansion. Companion to [PRD.md](PRD.md), [DESIGN.md](DESIGN.md), [ROADMAP.md](ROADMAP.md), per-product PRDs.
> **Owner:** `council-architecture` + `gtm-sales-se` + Founder.

---

## 1. Methodology

For each of the 7 axes + cross-cutting substrates, compare Oyatie's planned/current capability against the strongest per-segment incumbent (per regional pack where relevant). Score:

- **Edge** = where Oyatie is unique (cohesion thesis = baseline strength)
- **Parity** = where Oyatie matches incumbents
- **Gap** = where Oyatie is meaningfully behind
- **Anti-gap** = where Oyatie has explicitly chosen NOT to compete (anti-scope)

Each Gap row gets a proposed expansion + estimated investment + wave placement.

---

## 2. Axis 1 — SaaS multi-tenant platform

| Surface | Incumbent benchmark | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Workflow engine | Microsoft Power Automate / ServiceNow / Salesforce Flow | ADR-0035 state-machine + DAG hybrid; planned in `saas-workflow-*` | Parity (planned) | **Gap: visual editor maturity.** Expand: Workflow Studio with Foundry-driven authoring (#566); add workflow-template marketplace per vertical (8-15 templates per vertical at GA) |
| Plugin substrate | Salesforce AppExchange / Slack App Directory / Microsoft AppSource | ADR-0036/0157/0161/0162 manifest + trust + signing + Wasmtime sandbox | Edge (sandbox + signing) | **Gap: developer experience + revenue share economics.** Expand: plugin SDK in 4 languages (Rust + TS + Python + Go), per-plugin sandboxed dev environment, marketplace revenue share (industry 30%), curated review queue |
| Tenant onboarding self-serve | HubSpot / Zendesk free-tier signup | Issue #1204 / #1212 KR-first manual onboarding | **Gap (high)** | Self-serve onboarding ≤ 5min; per-vertical guided setup; Foundry-driven configuration agent; design-partner referral flow |
| Marketplace economics | Salesforce AppExchange (30%) / Apple App Store (15-30%) | undefined | **Gap (high)** | Per-tier revenue share matrix; ISV onboarding playbook; payout cadence; per-region tax handling |
| / collaboration surface | Slack / Discord / Microsoft Teams | Issue #749/#750/#1288 partial; planned ad-free per LEDG-021 | Parity (planned) | **Gap: voice+video integration depth.** Personal as an "ad-free Discord-class messaging" differentiator |
| Per-tenant white-label / theming | Stripe / Salesforce Lightning brand customization | undefined | **Gap (medium)** | Per-tenant logo / brand-color / custom-domain in Workspace surfaces |

**Edge to lean into:** engine-enforced object-graph isolation (ADR-0006) is a real moat — competitors have application-layer isolation only.

---

## 3. Axis 2 — Workspace / Productivity Platform

| Surface | Incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Mail | Google Gmail / Microsoft Outlook / Naver Mail | Greenfield; planned `workspace-mail-*` | **Gap (catastrophic without depth)** | Build out: per-tenant deliverability ≥ 99.9%; per-region mail-security (KR 메일 보안 / JP / EU); agent-driven triage (Foundry capability); migration wizard from Gmail / Outlook / Naver Mail (must be ≤ 1h per tenant) |
| Calendar | Google Calendar / Outlook Calendar / Naver Calendar | Greenfield | Gap | Smart scheduling (Foundry); team-availability views; resource booking; conferencing room integration |
| Docs / Sheets / Slides | Google Docs / Microsoft 365 / Naver Docs | Yrs CRDT planned | Gap | Real-time collaboration parity ≤ 80ms p99; export to PPTX/DOCX/XLSX/HWP/HWPX; offline mode; revision history; comment threading; granular permissions |
| Drive / Cloud Storage | Google Drive / Dropbox / OneDrive / Naver MyBox | Planned per `workspace-drive-*` | Gap | Native sync clients (Win/Mac/Linux/iOS/Android); team-drive concept; advanced sharing; selective-sync; offline-first |
| Meet / Video | Google Meet / Zoom / Microsoft Teams Meetings / Naver Whale Meet | Planned WebRTC SFU | Gap | Sub-150ms RTT KR-intra; AI summary + transcription; recording vault; live captioning per-language; webinar mode (1000+ attendees); meeting room hardware integration (Logitech / Poly) |
| Chat | Slack / Microsoft Teams / Discord / KakaoWork | Planned `workspace-chat-*` | Gap | Bot ecosystem; thread-first UX; voice channels; cross-tenant federation |
| Forms | Google Forms / SurveyMonkey / Typeform / Naver Form | Planned `workspace-forms-*` | Gap | Conditional logic; payment integration; vertical-form templates (clinical intake, KR HR forms, education enrollment) |
| Sites / Wiki | Google Sites / Notion / Confluence / Naver Cafe | Planned `workspace-sites-*` | **Gap (high)** | Notion-class structured note-taking + database views; Confluence-class wiki; per-tenant intranet templates |
| Tasks / Notes / Keep | Google Tasks + Keep / Microsoft To Do / Notion / Naver Memo | Planned shallow | **Gap** | Calendar integration; per-team kanban; project-management bridge to vertical-construction etc. |
| Translate | Google Translate / DeepL / Naver Papago / Kakao i Translation | Planned via Foundry adapter | Parity (via providers) | Per-pack quality benchmarks vs Papago for KR↔JP↔EN |
| **Edge to build** | (none yet) | (build the cohesion: Mail + Doc + Drive + Calendar + Foundry agent natively integrated, not bolted on) | — | This is THE differentiator: agent-native productivity from day one |

**Critical expansion:** Workspace is a NEW axis with the biggest greenfield gap. Make Foundry-native productivity (every Doc / Mail / Calendar action invokable + composable by agents) the differentiator vs Google Workspace's bolted-on Gemini and M365's bolted-on Copilot.

---

## 4. Axis 3 — Vertical industry cloud

| Vertical | Strongest incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Corporate (KR HR/payroll/GL/mail) | KR: 더존비즈온 / 영림원 / SAP / Workday | products/vertical-corporate deep | Edge (KR-anchor) | **Edge: deepen.** KR statutory depth (통상임금, 휴일/야간, 5/6일제, 주52시간, 연차 사용촉진) + Foundry-driven payroll close per #1156 |
| Corporate (global HR) | Workday / SAP SuccessFactors / Rippling | partial | Gap | Global payroll integration; multi-jurisdiction tax engines; SCIM / HRIS API parity |
| Healthcare | Epic / Cerner / KR EMR vendors | ADR-0016 + partial; #871 / #975 / #137 | Gap (high) | FHIR R4 server (read first, write at stable); HL7 v2 parity; clinical CDSS per ADR-0033; KR-MFDS pathway |
| Industrial | Siemens Opcenter / Rockwell Plex / SAP Digital Manufacturing / DELMIA Apriso / AVEVA MES / GE Proficy / Tulip | weak prose only; source-backed benchmark added in `evidence/autoresearch/industrial-mes-benchmark-1779034103.json` | Gap | OPC UA + ISA-95 + MES parity; connected-worker templates; multi-plant governance; OEE/yield/quality/takt telemetry; AMR integration |
| Logistics | Manhattan / Blue Yonder / Oracle WMS / KR-LX 판토스 | partial; #1157 | Gap | EDI 214/990/997 + route opt + HOS + cold-chain |
| Fintech | Toss / KakaoPay / NaverPay / PayPal / Stripe / Robinhood | depth in [`standards/fintech-compliance.md`](standards/fintech-compliance.md) | Parity (planned) | Per-region license stack; per-rail adapters; AML rules engine; KYC/KYB |
| Legal | Thomson Reuters / LexisNexis / Practical Law / KR LXLAW | partial; ADR-0033 | Gap | Regulated corpus management; contract analysis; e-discovery |
| Retail | Shopify / Square / Lightspeed / KR 큐텐 | skeleton | **Gap (high)** | POS + inventory + omnichannel; KR-payment integration; BNPL |
| Education | Google Classroom / Canvas / Blackboard / KR EBS | skeleton | **Gap (high)** | LMS + SIS + grading + parent portal; KR 학사관리 integration |
| Public Sector | KR 조달청 / 정부24 / Tyler / OpenGov | skeleton | **Gap (high)** | KR public-procurement; per-region gov form filing |
| Hospitality | Oracle Hospitality / Mews / Cloudbeds | skeleton | Gap | PMS + booking + F&B + housekeeping |
| Construction | Procore / Autodesk Construction / KR 건설365 | skeleton | Gap | Project mgmt + RFI + submittal + safety |
| Real Estate | Yardi / RealPage / KR 직방 / 다방 | skeleton | Gap | Leasing + asset mgmt |
| Agriculture | John Deere Operations Center / KR 스마트팜 | skeleton | Gap | Farm-mgmt + traceability + per-region subsidy filing |
| Food | Trace Register / KR HACCP | skeleton | Gap | HACCP + supply-chain compliance + recall mgmt |

**Highest-ROI expansion:** Healthcare + Fintech in KR (regulator depth + design-partner gravity); Retail + Education + Public-Sector cross-vertical (large TAM in KR + regional packs).

---

## 5. Axis 4 — Foundry

| Surface | Incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Agent runtime | LangChain/LangGraph / AutoGen / CrewAI / Bedrock Agents | strong consolidation | Parity (planned) | **Gap: live execution disabled per ADR-0025**; flip with clear gate |
| Capability registry | OpenAI Functions / Anthropic MCP / Google ADC | MCP-compatible per [TOOLCHAIN §4.A](TOOLCHAIN.md) | Edge (MCP gateway as discoverable surface) | Build out 100+ capabilities; eval set per capability |
| Multi-provider routing | LiteLLM / OpenRouter / Portkey | planned per provider-adapter trait | Parity (planned) | Cost-aware routing; failover per autonomy tier |
| Observability for agents | LangSmith / Helicone / Langfuse / Phoenix / Trulens / Logfire | weak | **Gap (high)** | OTel `gen_ai` semconv; per-step replay; per-tenant dashboard; cost waterfall |
| Eval harness | OpenAI Evals / Anthropic Constitutional AI / Apollo Research patterns | weak | Gap | Per-capability golden + adversarial + per-region linguistic; nightly run; regression gate |
| Sandbox | Anthropic Computer Use / OpenAI Code Interpreter | Wasmtime + Firecracker per ADR-0023 | Edge (Wasmtime + Firecracker hybrid) | Per-tool resource caps; per-tool egress allowlist |
| Foundry as a product | (none — this is unique) | possible per Foundry-improvements §H.7 | **Edge** | Sell Foundry capabilities directly via MCP; meaningful TAM expansion |
| In-house models (long-horizon) | OpenAI / Anthropic / Google / Naver HyperCLOVA-X / Upstage Solar / EXAONE | W-AI-Model-Substrate planned | Gap (long-horizon) | KR-first foundation LLM; embedding models for Search; STT/TTS; vision |
| Engineering platform (consolidated Foundry engineering platform) | Backstage / Port / OpsLevel / Cortex / Humanitec | consolidated into Foundry per [DESIGN §3](DESIGN.md) | Edge (recursive: same agent runtime authors + governs) | Build out: scorecards, fitness functions per axis, dev portal Leptos UI |

---

## 6. Axis 5 — Cloud provider

| Surface | Incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| IaaS compute | AWS EC2 / GCP Compute / OCI Compute / Naver Cloud / NHN Cloud / KT Cloud | greenfield | **Gap (catastrophic)** | Phased per [DESIGN §3.0.4](DESIGN.md): consume OCI+AWS now → own colo at scale → own greenfield mega-DC. Build IAM + region/AZ/cell + control plane in-house from day one |
| Object storage | AWS S3 / GCP GCS / OCI Object Storage / Cloudflare R2 | greenfield | Gap | KMS-shred per object; regional replication policy; lifecycle tiering |
| Block storage | AWS EBS / GCP PD / OCI Block | greenfield | Gap | Per-tier IOPS (gp3 / io2-class) |
| Network (VPC / LB / DNS / CDN) | AWS VPC + ELB + Route53 + CloudFront / Cloudflare | greenfield | Gap | KR 망분리 enforcement; per-region peering |
| Managed K8s | EKS / GKE / OKE / Naver Cloud K8s | greenfield | Gap | Foundry-driven cluster ops; per-tenant nodepool isolation |
| Functions | AWS Lambda / GCP Cloud Functions / OCI Functions / Cloudflare Workers | greenfield | Gap | Cold-start budget; per-region; sub-100ms cold |
| KMS / HSM | AWS KMS + CloudHSM / GCP KMS / OCI Vault | greenfield | Gap (KCMVP HSM lead 6-9 mo) | KCMVP-validated HSM (KR); FIPS 140-3 (global); per-tenant key isolation |
| Billing | AWS Billing / GCP Billing | greenfield | Gap | Per-resource per-tenant metering; per-region tax-invoice format |
| Observability | CloudWatch / Stackdriver / Datadog / New Relic | OTel + VictoriaMetrics + Grafana per ADR-0045/0178 | Parity (with license caveat per drafted ADR-0013-product-license-policy: Grafana is AGPL — replace with in-house Leptos UI long-horizon) | Build out in-house observability portal; Datadog-class APM |
| FinOps | CloudHealth / Vantage / Apptio | greenfield | Gap | Per-tenant cost explorer; budget alerts; right-sizing |
| Marketplace | AWS Marketplace / GCP Marketplace / Azure Marketplace | greenfield | Gap | ISV onboarding; per-vertical curation |
| DCIM (long-horizon) | Sunbird DCIM / Schneider EcoStruxure / Nlyte / Google internal | greenfield per W-DataCenter-Operations | Gap (long-horizon) | In-house DCIM; per-rack inventory; PUE/WUE/CUE; sustainability metrics |
| Edge / CDN POPs | Cloudflare / Fastly / Akamai / Naver Cloud CDN | greenfield | Gap | Per-region POPs; image transforms; security WAF |
| Bare-metal | AWS Outposts / OCI Bare Metal / KR colo | greenfield | Gap (long-horizon) | Per-tenant bare-metal lease; KCMVP-eligible |
| GPU fleet (post W-AI-Model-Substrate) | AWS P5 / GCP A3 / OCI GPU shapes | greenfield | Gap (long-horizon) | Per-cell GPU partition; reserved capacity |

**Critical expansion:** Cloud is greenfield. The cohesion play (cloud + tenancy + identity + audit chain shared) is the differentiator vs hyperscalers who can't credibly own the SaaS layer that runs on top. KR sovereignty + CSAP path is the regulatory wedge.

---

## 7. Axis 6 — Search

| Surface | Incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Crawler | Google / Bing / Naver / Daum | greenfield | **Gap (catastrophic without depth)** | Per-host politeness; per-host budget; spam detection; per-corpus rights ledger |
| Parser (HTML / PDF / OCR / KR morphology) | Google parsers / Naver Mecab-ko | partial (mecab-ko/khaiii via FFI per ADR-0047) | Gap | Per-language morphology; HWPX (KR gov format); image OCR |
| Inverted index | Google internal / Naver internal / Elasticsearch / OpenSearch / Tantivy | pgroonga day-1 + OpenSearch gated per ADR-0047 | Gap | Sharding; per-tenant + cross-tenant boundary; freshness pipeline |
| Vector index | FAISS / Milvus / pgvector / Qdrant | pgvector day-1 per ADR-0050/0177 | Parity | HNSW / IVF in-house long-horizon; per-tenant vector |
| Ranker | Google Brain / Naver / Bing | greenfield | Gap | BM25 + cross-encoder rerank + freshness + authority + diversity + KR signals |
| Query understanding | Google QU / Naver QU | greenfield | Gap | Spelling, expansion, intent, autocomplete (per-locale, freshness-aware) |
| SERP + features | Google / Naver | greenfield | Gap | Snippets / featured / KG panel / image / video / news / maps / shopping |
| Knowledge graph | Google KG / Naver 지식백과 | greenfield | Gap | Per-vertical KG; consent-bound entity linking |
| Per-tenant private search | Algolia / Elasticsearch managed | partial | Edge | Engine-enforced isolation per ADR-0006 |
| RAG endpoint | Pinecone / Weaviate / Vespa | partial | Edge (cohesion: Foundry consumes per [DESIGN §3.0.1](DESIGN.md)) | Per-tenant + per-class allowlist; consent-receipt cited |
| Korean morphology + tokenizer | Naver Mecab-ko / Daum Kkma / Khaiii | partial | Parity (planned via FFI; in-house long-horizon) | Port mecab-ko in Rust + Tantivy at scale |
| RTBF / DSR cascade | Google RTBF / Bing RTBF | planned | Edge (cohesion: per-tenant cascade automatic) | Cascade-purge + Cosign-signed proof-of-erasure |
| Voice search | Google Voice / Naver / Siri | greenfield | Gap | STT integration via Foundry speech substrate |
| Image / video search | Google Images / Naver / Bing | greenfield | Gap | Image embedding + perceptual hash; video keyframe + transcript |
| Maps / local | Google Maps / Naver Maps / Kakao Maps | greenfield | **Gap (catastrophic for KR consumer search)** | Defer or partner; Naver Maps integration as adapter |

**Critical expansion:** Search is greenfield with massive incumbents. Don't try to beat Google globally. Focus: (a) per-tenant private search as a SaaS feature; (b) Foundry-RAG endpoint as agent-internal; (c) eventual KR consumer search as differentiator only if regulator + market window opens.

---

## 8. Axis 7 — Ads + Analytics

| Surface | Incumbent | Oyatie posture | Score | Gap + expansion |
|---|---|---|---|---|
| Ad serving + auction | Google Ads / Meta Ads / Naver 검색광고 / Kakao Moment | greenfield | **Gap (catastrophic)** | Sub-100ms auction; per-cell-routed; per-policy gate |
| Ad targeting | Google Ads / Meta / Criteo | greenfield | Gap | DECLARED_PREFERENCE-only by default; per-class allowlist (HARD_DENY for PHI/PCI/PIPA-Art23) |
| Attribution | Google Analytics / Adobe Analytics / SKAdNetwork / IAB IPA | greenfield | Gap | Privacy-preserving aggregation per [PRIVACY-PROGRAM §2.2.6](PRIVACY-PROGRAM.md); cross-device with consent |
| Advertiser console | Google Ads UI / Naver 검색광고센터 | greenfield | Gap | Campaign mgmt; recommendations engine; budget mgmt; reporting |
| Publisher network | Google AdSense / Naver Smart 채널 | greenfield (skip if Oyatie owns inventory) | Gap | Header bidding; revenue share with publishers |
| Smart bidding ML | Google Smart Bidding | greenfield (post W-AI-Model-Substrate) | Gap | Per-tenant ROAS-target bidding |
| Click fraud / IVT | DoubleVerify / Integral Ad Science / IAS | greenfield | Gap | Per-impression IVT filter; per-click fraud detection |
| Brand safety | DoubleVerify / IAS / GroupM | greenfield | Gap | Per-page content classifier; brand-safety inventory tier |
| KR adtech ecosystem | KR KODA / 한국디지털광고협회 | greenfield | Gap | KR 의료광고 / 금융광고 / 정치광고 review workflows |
| Analytics — event ingestion | Segment / mParticle / Snowplow | greenfield | Gap | Per-tenant event schema; consent-tier-bound |
| Analytics — warehouse | Snowflake / BigQuery / ClickHouse / Databricks | ClickHouse per ADR-0045 (verify license post-fork) | Parity (planned) | Per-tenant DP-budget; cross-tenant aggregate per consent |
| Analytics — BI | Tableau / Looker / PowerBI / Naver DataLab | greenfield | Gap | Per-tenant dashboard; embedding |
| Analytics — streaming | Materialize / Flink / Arroyo | greenfield | Gap | Real-time analytics dashboards; per-tenant materialized views |
| DP / k-anonymity | Apple SKAdNetwork / Google Privacy Sandbox / IAB IPA | planned per [PRIVACY-PROGRAM §2.2.6](PRIVACY-PROGRAM.md) | **Edge** | Per-tenant per-class ε-budget (cleaner than Google); central ledger |

**Critical expansion:** Ads + Analytics is greenfield. Focus on the privacy moat: per-class boundary + DP/k-anonymity built in. Don't try to beat Google globally on ad-tech volume; differentiate on regulated-vertical advertiser + KR-first compliance review.

---

## 9. Cross-cutting weak areas

### 9.1 Mobile + native + edge

- **Status:** ADR-0033 Pure-Leptos console is the canonical UI; mobile/native client strategy is ADR-0051 plus #770 native phone/tablet execution.
- **Gap:** Mobile native parity weak; Leptos mobile story unclear
- **Expansion:** dedicated mobile team per axis (not per platform); KMP for shared business logic; Native UI per platform

### 9.2 Voice / speech + Vision

- **Status:** Folded into Foundry per [DESIGN §3.0.2](DESIGN.md)
- **Gap:** No in-house production yet; depends on W-AI-Model-Substrate + W-Robotics-Vision-Speech (long-horizon)
- **Expansion:** STT + TTS for KR / JP / EN / ES / PT / HI / AR (Foundry adapter day-1 from Whisper/Naver Clova); in-house vision models post W-AI-Model-Substrate

### 9.3 Lakehouse vs Warehouse

- **Status:** Issue #130 lakehouse-vs-warehouse decision pending
- **Gap:** Currently undecided; blocks BI roadmap
- **Expansion:** Decide via ADR (Iceberg + DataFusion lakehouse OR ClickHouse warehouse); recommended: lakehouse (Apache 2 + open formats + license-clean)

### 9.4 CRM / Marketing automation / Sales tools

- **Status:** Out of explicit scope today (corporate vertical light coverage)
- **Gap:** Tenant-side CRM is a typical adjacency to corporate; not declared anti-scope
- **Expansion:** Lightweight CRM as a vertical-corporate sub-module (Salesforce-Lite for SMB tenants)

### 9.5 Crypto / blockchain / Web3

- **Status:** Adjacent to fintech; not declared in/out of scope
- **Gap:** No crypto exchange / VASP work; KR 특금법 + EU MiCA + US FinCEN MSB deferred
- **Expansion:** Council decision: in-scope as a fintech sub-vertical (per `LEDG-013`/`LEDG-020` open) or anti-scope

### 9.6 Twilio-class CPaaS (SMS / voice / WebRTC)

- **Status:** Workspace Meet covers internal video; no SMS / programmable voice / SIP
- **Gap:** Verticals (especially fintech for OTP, healthcare for appointment reminders, retail for loyalty) need SMS + voice
- **Expansion:** CPaaS sub-axis or partner with KR carrier API (KT / SKT / LGU+ messaging API)

### 9.7 Migration tooling from competitors (most underrated)

- **Status:** vertical-PRDs mention it; depth weak
- **Gap:** Tenant migration from Google Workspace / Microsoft 365 / Naver Works / Salesforce / Workday / KR 더존비즈온 etc. is the single biggest GTM friction
- **Expansion:** Per-competitor migration playbook + Foundry-driven importer + per-tenant validation report; target ≤ 24h tenant migration at preview, ≤ 6h at GA

### 9.8 Trust portal + customer-facing compliance UX

- **Status:** Planned at `trust.oyatie.com` per [DOCUMENTATION.md §3](DOCUMENTATION.md)
- **Gap:** No spec yet for customer-facing compliance dashboards
- **Expansion:** Trust portal preview slice shipping with W-Foundation gate; per-tenant evidence-pack download self-serve; chain-anchor proofs visible; SOC2 / ISO / ISMS-P attestations summarized

### 9.9 Developer Experience (SDKs, sandbox tenants, dev surface)

- **Status:** ADR-0025 dev.oyatie.com Engineering Development Surface; SDK gen planned in TOOLCHAIN
- **Gap:** SDKs not authored; sandbox tenants undefined; dev surface partial
- **Expansion:** TS + Python + Go SDK auto-generated from OpenAPI; per-developer sandbox tenant; webhook console; changelog feed

### 9.10 Internal admin console for support staff

- **Status:** Issue #764 / #754 mvp-admin-surface
- **Gap:** Read-only internal admin partial; per-tenant troubleshooting weak
- **Expansion:** `oya admin support` CLI + Leptos web UI; per-tenant view (read-only first; gated write with audit)

### 9.11 KR-specific consumer products (where Naver / Kakao dominate)

- **Status:** Out of scope per anti-scope (no consumer social network; no consumer search at preview)
- **Gap:** If we ever serve KR consumers, the gap vs Naver / Kakao is enormous
- **Expansion:** Defer; revisit only on substantial evidence per PRD §3.3

---

## 10. Recommended expansion priorities (by ROI)

| Priority | Expansion | Rationale | Wave placement |
|---|---|---|---|
| P0 | Workspace cohesion (Mail + Doc + Drive + Calendar + Foundry-native from day one) | Biggest greenfield gap; differentiator vs Google/M365 bolted-on AI | W-Workspace-Preview (parallel) |
| P0 | Foundry observability + eval harness | Without these, agent reliability collapses | W-Foundry-Preview |
| P1 | Per-vertical migration tooling (Google Workspace / M365 / Naver Works / Salesforce / Workday / KR 더존비즈온) | Removes biggest GTM friction | W-Vertical-Pilot + continuous |
| P1 | Trust portal preview slice | Customer-facing compliance UX | W-Foundation |
| P1 | Cloud cell architecture + IAM + KMS depth | Cloud-axis credibility floor | W-Cloud-Preview |
| P1 | Healthcare + Fintech vertical depth (KR-first) | KR design-partner gravity | W-Vertical-Pilot |
| P2 | Plugin SDK in 4 languages + marketplace economics | ISV ecosystem unlock | W-SaaS-Preview |
| P2 | Search per-tenant + Foundry-RAG | Enables agent reliability | W-Search-Preview |
| P2 | Mobile native parity per axis | Tenant employees | W-Workspace-Stable |
| P3 | Retail / Education / Public-Sector vertical depth | TAM expansion | W-Vertical-Fan-Out |
| P3 | KR adtech compliance review workflows + Naver/Kakao integration | KR ad-axis credibility | W-Ads-Preview |
| P3 | CPaaS sub-axis OR carrier-API partnership | Fintech / healthcare / retail use-cases | W-Vertical-Fan-Out |
| P4 | In-house AI models (KR-first foundation LLM, embedding, STT/TTS, vision) | Cost + sovereignty | W-AI-Model-Substrate |
| P4 | DCIM software | Long-horizon scale | W-DataCenter-Operations |
| P5 | Robotics control plane + simulator | Industrial / logistics future | W-Robotics-Vision-Speech |
| Anti | Consumer search / social / Maps in KR (where Naver dominates) | Gap too large; not winnable | (anti-scope) |

---

## 11. Sources scanned

All consolidated docs at `docs/`; per-product PRDs at `products/`; v2 plan at `docs/raw/plan-v2-draft.md`; Foundry-improvements research at `docs/raw/foundry-improvements.md`; Codex verdict at `docs/raw/codex-verdict.md`; per-vertical incumbent landscape (industry research).

*Footer regenerated whenever this doc is edited.*
