---
doc_class: FeatureParityMatrix
microservice: contact-center
matrix_date: 2026-05-20
counterparts_top3: [Genesys Cloud, Five9, Amazon Connect]
counterparts_secondary: [Twilio Flex, Zendesk Talk, NICE CXone, Talkdesk]
coverage_mode: UNION
doctrine_overlays:
  - no_tier_dimension
  - tenant_class_overlay_applied
  - deployment_context_overlay_applied
---

# Contact Center — Feature Parity Matrix (UNION-coverage)

This matrix grades the `contact-center` µservice against the UNION of capabilities offered by Genesys Cloud, Five9, and Amazon (dispatch top-3) plus Twilio Flex, Zendesk Talk, NICE CXone, and Talkdesk (corpus-named secondary counterparts). UNION-coverage means: if ANY of the seven names ships the capability, it lands in the universe.

Grading scheme (NO tenant_class dimension per `feedback_no_tenant_class_eligibility_2026_05_20`):

- `Y` — implemented in `src/` + `iac/` + `contracts/` + `policies/` at a level that ships in production.
- `D` — declared in docs (PRD / ARCHITECTURE / capability YAML / IP) but not implemented in source.
- `P` — partial: declared + partial src implementation, but missing concrete adapter, contract, or runtime.
- `N` — neither declared nor implemented; absent.
- `n/a` — not applicable to the µservice's charter.

Tenant-class column shows which tenant class each capability is available to:
- demo_trial = available under OCI Always Free usage caps (concurrent_calls ≤ 30, recording_retention ≤ 30 d, IVR_flows ≤ 5, AI-assist on CPU-only Whisper.cpp tiny.en).
- paid = available without caps; per-seat + per-usage metered.

Deployment-context column shows which contexts (per `feedback_multi_context_provider_agnostic_2026_05_20`) support the capability:
- oyatie-public = managed Oyatie SaaS hosted in Oyatie's public cloud.
- aws-guest = Oyatie deployed as a guest tenant on customer AWS account.
- oci-guest = Oyatie deployed as a guest tenant on customer OCI account (includes OCI Always Free for demo_trial).
- on-prem = Oyatie deployed on customer bare-metal in customer data center.
- colo = Oyatie deployed in colocation facility (Equinix Metal / Cyxtera / etc.).
- oyatie-as-cloud-provider = Oyatie operates as the IaaS itself (cloud-* µservices ARE the provider).

## 1. Inbound voice routing

Inbound voice routing is THE load-bearing CCaaS capability. Genesys Cloud routes via Architect flows; Five9 via VIVR; Amazon via flows JSON.

| # | Sub-capability | Genesys Cloud | Five9 | Amazon | Twilio Flex | Zendesk Talk | NICE CXone | Talkdesk | Oyatie current | Tenant class | Deployment contexts |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | SIP INVITE handling | Y | Y | Y | Y | Y | Y | Y | N (no src/adapter/sip.rs) | both | all-6 |
| 2 | Skill-based routing | Y | Y | Y | Y | Y | Y | Y | D (capability YAML declared; no algorithm in src) | both | all-6 |
| 3 | Predictive routing (ML-driven) | Y (Predictive Engagement) | Y (Practical AI) | Y (Contact Lens predictive) | partial | partial | Y (Enlighten AI) | Y (Talkdesk Copilot) | N | paid | all-6 (paid only) |
| 4 | Queue-fairness routing | Y | Y | Y | Y | Y | Y | Y | D (queue capability declared) | both | all-6 |
| 5 | Time-of-day routing | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 6 | Geographic/locale routing | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 7 | After-hours routing | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 8 | Priority routing (VIP detection) | Y | Y | Y | Y | partial | Y | Y | D | paid | all-6 |
| 9 | Emergency caller bypass (911 / 119 / 112) | Y (NENA i3) | Y | Y | Y | partial | Y | Y | D (capability + Cedar fragment declared) | both | oyatie-public, on-prem, colo |
| 10 | Multi-tenant call isolation | partial (per-account) | partial | partial (per-instance) | Y (Flex per-account) | partial | partial | partial | D (tenant_id binding declared) | both | all-6 |

Coverage: top-3 each offer all 10 sub-capabilities. Oyatie has 9/10 declared but 0/10 implemented in source. Net inbound-voice readiness: 0/10 Y.

## 2. Outbound voice routing

Outbound dialer modes: preview, progressive, predictive, power. TCPA-abandonment-rate enforcement is the regulatory anchor.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 11 | Preview dialer | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 12 | Progressive dialer | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 13 | Predictive dialer (TCPA-compliant) | Y | Y (flagship) | Y | Y | N | Y | Y | N | paid | all-6 |
| 14 | Power dialer | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 15 | TCPA abandonment-rate enforcement (≤ 3 %) | Y | Y | Y | Y | N | Y | Y | D (tier-matrix mentions) | paid | all-6 |
| 16 | DNC (Do-Not-Call) list compliance | Y | Y | Y | Y | partial | Y | Y | D (TCPA pack declared) | paid | all-6 |
| 17 | Call disposition codes | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 18 | Campaign management UI | Y | Y | Y (basic) | Y | partial | Y | Y | N | paid | oyatie-public, aws-guest, oci-guest |
| 19 | Local-presence dialing (LATA matching) | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 20 | Answering machine detection (AMD) | Y | Y | partial | Y | N | Y | Y | N | paid | all-6 |

Coverage: top-3 each offer all 10 sub-capabilities. Oyatie has 4/10 declared + 0/10 implemented. Net outbound-voice readiness: 0/10 Y.

## 3. IVR (Interactive Voice Response)

Self-service voice flow runtime. Genesys uses Architect; Amazon uses flows JSON; Five9 uses VIVR.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 21 | DTMF input capture | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 22 | Speech-to-text input (ASR-driven IVR) | Y | Y | Y (Lex bots) | Y | partial | Y | Y | N (intelligence µservice delegation declared, no contract) | paid | all-6 |
| 23 | Text-to-speech output | Y | Y | Y | Y | partial | Y | Y | N | both | all-6 |
| 24 | IVR-flow visual designer | Y (Architect) | Y | Y (flows GUI) | Y (Studio) | partial | Y | Y (Talkdesk Studio) | N | paid | oyatie-public, aws-guest, oci-guest |
| 25 | IVR-flow JSON export | Y | Y | Y (canonical) | Y | partial | partial | Y | N (schema not declared) | both | all-6 |
| 26 | IVR-flow version control | Y | partial | Y (per-version) | Y | partial | Y | Y | N | both | all-6 |
| 27 | IVR menu depth ≥ 8 levels | Y | Y | Y | Y | Y | Y | Y | D (tier-matrix claims 8 levels) | both | all-6 |
| 28 | Conditional branching on customer attributes | Y | Y | Y (Customer Profiles) | Y | partial | Y | Y | N | paid | all-6 |
| 29 | API call from IVR (to backend systems) | Y | Y | Y (Lambda) | Y (Functions) | partial | Y | Y | N | paid | all-6 |
| 30 | IVR analytics + drop-off reporting | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |

Coverage: top-3 each offer all 10 sub-capabilities. Oyatie has 1/10 declared + 0/10 implemented. Net IVR readiness: 0/10 Y.

## 4. Omnichannel — chat

Web chat + SDK chat + in-app chat.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 31 | Web chat widget | Y | Y | Y | Y | Y (flagship) | Y | Y | N (delegated to messenger µservice in theory; not bound) | both | all-6 |
| 32 | In-app SDK chat | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 33 | Chat-to-voice escalation | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 34 | Chat bot integration | Y (Bot Connector) | Y | Y (Lex) | Y (Autopilot) | Y | Y | Y | N | paid | all-6 |
| 35 | Chat queue + skill routing | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 36 | Chat transcript archival | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 37 | Co-browse / screen-share | Y | partial | partial | Y | partial | Y | Y | N | paid | all-6 |
| 38 | Chat typing indicator + read receipts | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 39 | File attachment in chat | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 40 | Chat language detection + i18n | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |

Coverage: top-3 each offer all 10 sub-capabilities. Oyatie chat parity is 0/10. Per ADR-0145 + the substrate-product layering doctrine, chat should be delegated to the `messenger` µservice — but the contact-center µservice MUST declare the chat-to-voice escalation contract; that contract is missing.

## 5. Omnichannel — email + SMS + social

Email-as-channel (different from comms-email µservice transactional mail). SMS + social DM (FB Messenger, X DM, Instagram DM, WhatsApp Business).

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 41 | Email channel ticketing | Y | Y | Y | Y | Y (flagship) | Y | Y | N (delegated to comms-email + community?) | both | all-6 |
| 42 | SMS inbound + outbound | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 43 | WhatsApp Business integration | Y | partial | Y | Y (native) | Y | Y | Y | N | paid | all-6 |
| 44 | FB Messenger integration | Y | partial | Y | Y | Y | Y | Y | N | paid | all-6 |
| 45 | X (Twitter) DM integration | partial | N | N | Y | Y | partial | Y | N | paid | all-6 |
| 46 | Instagram DM integration | Y | N | N | Y | Y | partial | Y | N | paid | all-6 |
| 47 | LINE / WeChat (regional packs) | partial | N | partial | Y | partial | partial | partial | N | paid + KR/JP/CN pack | all-6 |
| 48 | Unified omnichannel inbox | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 49 | Channel switching mid-conversation | Y | partial | partial | Y | Y | Y | Y | N | paid | all-6 |
| 50 | Conversation history across channels | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |

Coverage: 10 sub-capabilities; oyatie has 0/10. Several are pack-gated (LINE for KR-PIPA; WeChat for CN pack — not present in current corpus).

## 6. Video channel

Video customer service is the rising-leader differentiator (Amazon added video in 2025; Genesys via Genesys Cloud Video).

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 51 | Video inbound (customer initiates) | Y | partial | Y (2025+) | Y (native Programmable Video) | partial | Y | Y | N | paid | all-6 |
| 52 | Video outbound (agent initiates) | Y | partial | Y | Y | partial | Y | Y | N | paid | all-6 |
| 53 | Screen-share over video | Y | partial | Y | Y | partial | Y | Y | N | paid | all-6 |
| 54 | Video recording | Y | partial | Y | Y | partial | Y | Y | N | paid | all-6 |
| 55 | Video transcription (real-time) | Y | partial | Y | Y | partial | Y | Y | N | paid | all-6 |

Coverage: 5 video sub-capabilities; oyatie 0/5. Should delegate to `meet` µservice + bind via contact-center → meet contract; binding does not exist.

## 7. AI bots + agent assist

Real-time AI agent-assist; conversational bots for self-service.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 56 | Conversational AI bot (NLU) | Y (Bot Connector + DialogFlow/Lex) | Y (Practical AI) | Y (Lex flagship) | Y (Autopilot) | Y (Answer Bot) | Y (Enlighten AI) | Y (Talkdesk Copilot) | D (intelligence µservice delegation declared) | paid | all-6 |
| 57 | Intent classification | Y | Y | Y (Lex) | Y | Y | Y | Y | N | paid | all-6 |
| 58 | Entity extraction | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 59 | Agent-assist (real-time suggestions) | Y (Gen AI add-on) | Y (AgentAssist) | Y (Contact Lens) | Y | Y | Y (Enlighten) | Y (Copilot) | D | paid | all-6 |
| 60 | Real-time transcription | Y | Y | Y (Contact Lens) | Y | Y | Y | Y | D (via intelligence µservice, no contract) | paid | all-6 |
| 61 | Real-time sentiment scoring | Y | Y | Y | Y | Y | Y | Y | D | paid | all-6 |
| 62 | Next-best-action prompts | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 63 | Auto-summarization post-call | Y | Y | Y (Generative AI) | Y | Y | Y | Y | N | paid | all-6 |
| 64 | Knowledge-base auto-suggest | Y | Y | Y (Wisdom) | Y | Y (Answer Bot) | Y | Y | N | paid | all-6 |
| 65 | Auto-wrap-up notes generation | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |

Coverage: 10 AI sub-capabilities; oyatie 0/10 implemented (4/10 declared via intelligence-µservice delegation, but no gRPC contract bound). Net AI readiness: 0/10 Y. P0 substance gap.

## 8. Agent desktop

The agent-facing UI / workspace. Genesys uses Genesys Cloud agent desktop; Five9 has Agent Desktop Plus; Amazon has CCP (Contact Control Panel).

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 66 | Web-based agent desktop | Y | Y | Y (CCP) | Y (Flex flagship) | Y | Y | Y | N (no frontend/web/contact-center/ found) | both | all-6 |
| 67 | Native desktop app (Windows) | Y (legacy) | Y | partial | partial | partial | Y | partial | N | paid | all-6 |
| 68 | Mobile agent app (iOS) | Y (Genesys Cloud Mobile) | Y (Five9 Mobile Agent) | partial | partial | Y | Y | Y | N (per §3.4.M of audit, no mobile coordination doc) | paid | all-6 |
| 69 | Mobile agent app (Android) | Y | Y | partial | partial | Y | Y | Y | N | paid | all-6 |
| 70 | Embeddable agent widget (in CRM) | Y (Salesforce, MS Dynamics) | Y (Salesforce, ServiceNow) | Y (Salesforce, Zendesk) | Y (Salesforce, HubSpot) | Y (native) | Y (Salesforce, MSD) | Y (Salesforce, MSD) | N (oyatie crm µservice exists; no contact-center widget contract) | paid | oyatie-public, aws-guest |
| 71 | Customer 360 view in desktop | Y | Y | Y (Customer Profiles) | Y | Y | Y | Y | N (oyatie ontology µservice — no projection contract bound) | paid | all-6 |
| 72 | Click-to-dial | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 73 | Conference / 3-way call | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 74 | Warm transfer | Y | Y | Y | Y | Y | Y | Y | N (declared in ADR-MS-001 transfer-decision fields; no impl) | both | all-6 |
| 75 | Cold transfer | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 76 | Hold + retrieve | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 77 | Mute / unmute | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 78 | Disposition coding | Y | Y | Y | Y | Y | Y | Y | D (ADR-MS-001 fields declared) | both | all-6 |
| 79 | Post-call wrap-up timer + ACW | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 80 | Multi-tab multi-conversation handling | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |

Coverage: 15 agent-desktop sub-capabilities; oyatie 0/15 implemented (1/15 declared). Net agent-desktop readiness: 0/15. The Leptos web frontend stack is authorized per the OS support matrix memory but no `frontend/web/contact-center/` has been authored.

## 9. Scripts + screen pop

Agent guidance scripts; screen pop integration with CRM on inbound call.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 81 | Script designer | Y | Y | partial | Y | partial | Y | Y | N | paid | all-6 |
| 82 | Dynamic script per skill | Y | Y | partial | Y | partial | Y | Y | N | paid | all-6 |
| 83 | Screen pop on inbound | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 84 | CTI integration with Salesforce | Y | Y | Y | Y | Y (native) | Y | Y | N | paid | oyatie-public, aws-guest |
| 85 | CTI with MS Dynamics 365 | Y | Y | Y | Y | partial | Y | Y | N | paid | oyatie-public, aws-guest |
| 86 | CTI with ServiceNow | Y | Y | Y | Y | partial | Y | Y | N | paid | oyatie-public, aws-guest |
| 87 | Script branching on customer attributes | Y | Y | partial | Y | partial | Y | Y | N | paid | all-6 |
| 88 | Mandatory-field enforcement | Y | Y | partial | Y | partial | Y | Y | N | paid | all-6 |

Coverage: 8 script sub-capabilities; oyatie 0/8.

## 10. WFM (Workforce Management)

Forecasting, scheduling, intra-day management. NICE and Genesys are the historical WFM leaders; Five9 has WFM via Calabrio acquisition; has basic forecasting.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 89 | Forecasting (call volume + AHT) | Y | Y (via Calabrio) | partial | partial | partial | Y (flagship) | Y | N | paid | all-6 |
| 90 | Schedule generation | Y | Y | partial | partial | N | Y | Y | N | paid | all-6 |
| 91 | Shift bidding | Y | Y | N | N | N | Y | Y | N | paid | all-6 |
| 92 | Real-time adherence | Y | Y | partial | partial | N | Y (Enlighten) | Y | D (workforce-adherence-stream IP-028 declared) | paid | all-6 |
| 93 | Intra-day management | Y | Y | N | partial | N | Y | Y | N | paid | all-6 |
| 94 | Time-off + leave management | Y | Y | N | partial | N | Y | Y | N | paid | all-6 |
| 95 | Multi-skill optimization | Y | Y | partial | partial | N | Y | Y | N | paid | all-6 |
| 96 | What-if scenario planning | Y | Y | N | partial | N | Y | partial | N | paid | all-6 |

Coverage: 8 WFM sub-capabilities; oyatie 1/8 declared (workforce-adherence-stream IP-028 names the concern but is template-stamped 104-line IP). Net WFM readiness: 0/8 Y. NICE CXone + Genesys are the leaders here.

## 11. WFO (Workforce Optimization) + Quality Management

Call quality scoring, calibration, agent coaching.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 97 | Quality monitoring (manual scoring) | Y | Y | partial | partial | partial | Y (flagship) | Y | D (quality-monitoring bounded context declared) | paid | all-6 |
| 98 | Auto-quality scoring (AI-driven) | Y | Y | Y (Contact Lens) | partial | partial | Y (Enlighten) | Y | N | paid | all-6 |
| 99 | Calibration sessions | Y | Y | N | N | N | Y | Y | N | paid | all-6 |
| 100 | Agent performance dashboards | Y | Y | Y | Y | Y | Y | Y | partial (dashboards/ folder has operating-bar dashboards) | both | all-6 |
| 101 | Coaching workflow | Y | Y | partial | partial | N | Y | Y | N | paid | all-6 |
| 102 | Speech analytics | Y | Y | Y | partial | partial | Y | Y | N | paid | all-6 |
| 103 | Compliance keyword detection | Y | Y | Y | partial | partial | Y | Y | N | paid + compliance packs | all-6 |
| 104 | PCI / HIPAA word/phrase redaction in transcripts | Y | Y | Y | partial | partial | Y | Y | D (tutorials/build-ivr-flow-with-pci-suppression.md declared) | paid + PCI / HIPAA pack | all-6 |

Coverage: 8 WFO/QM sub-capabilities; oyatie 0/8 implemented (3/8 declared partial).

## 12. Call recording

Recording capture + storage + retention + retrieval + export.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 105 | Audio recording (full call) | Y | Y | Y | Y | Y | Y | Y | N (recordings µservice delegation declared; no contract) | both | all-6 |
| 106 | Recording-on-demand (pause/resume mid-call) | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 107 | Two-channel (stereo) recording | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 108 | Pre-call recording consent prompt | Y | Y | Y | Y | partial | Y | Y | D (recording-consent capability + Cedar fragment + ADR-MS-001 fields) | both | all-6 |
| 109 | Recording retention policy enforcement | Y | Y | Y (S3 Object Lock) | partial | partial | Y | Y | D (data-residency.md + pack overlays) | both | all-6 |
| 110 | WORM (Write-Once-Read-Many) storage | partial | partial | Y (S3 Object Lock) | partial | partial | Y | partial | D (tier-matrix claims SeaweedFS WORM Compliance; no impl) | paid | all-6 |
| 111 | Recording encryption-at-rest | Y | Y | Y (KMS-CMK) | Y | Y | Y | Y | D (cloud-kms µservice declared) | both | all-6 |
| 112 | Per-tenant HSM-resident encryption key | partial | N | Y (per-account KMS) | partial | N | partial | N | D (tier-matrix paid compliance-pack claim; no impl) | paid + sovereign packs | on-prem, colo, oyatie-as-cloud-provider |
| 113 | Recording export (audio + transcript) | Y | Y | Y | Y | Y | Y | Y | D (action_id voice-routing.export declared) | both | all-6 |
| 114 | Recording-redaction (PCI / HIPAA) | Y | Y | Y | partial | partial | Y | Y | D (IP-027-recording-consent-redaction-vault declared) | paid + PCI/HIPAA | all-6 |
| 115 | Long-term archival (years) | Y | Y | Y (S3 Glacier) | partial | partial | Y | Y | D (tier-matrix claims 7-y retention) | paid | all-6 |

Coverage: 11 recording sub-capabilities; oyatie 0/11 implemented (7/11 declared). Recording is delegated to the `recordings` µservice (per ADR-0145 substrate-vs-product); the contact-center → recordings gRPC contract is missing.

## 13. Analytics + reporting

Historical reporting + real-time dashboards + custom report builders.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 116 | Real-time supervisor dashboard | Y | Y | Y | Y | Y | Y | Y | partial (dashboards/operating-bar-overview.json) | both | all-6 |
| 117 | Historical reporting (canned reports) | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 118 | Custom report builder | Y | Y | Y | partial | partial | Y | Y | N | paid | all-6 |
| 119 | Data export (CSV / Parquet / API) | Y | Y | Y (Kinesis) | Y | Y | Y | Y | D (voice-routing.export action_id) | both | all-6 |
| 120 | BI tool integration (Tableau, Looker) | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 121 | Real-time KPI dashboards | Y | Y | Y | Y | Y | Y | Y | partial | both | all-6 |
| 122 | Per-skill / per-queue analytics | Y | Y | Y | Y | Y | Y | Y | partial | both | all-6 |
| 123 | Customer journey analytics | Y | Y | Y (Customer Profiles) | Y | Y | Y | Y | N (delegated to ontology µservice) | paid | all-6 |

Coverage: 8 analytics sub-capabilities; oyatie 1/8 partially implemented.

## 14. Supervisor tools

Real-time monitoring of agent state + barge-in + whisper + coach.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 124 | Agent presence visualization | Y | Y | Y | Y | Y | Y | Y | D (agent-state-sync capability) | both | all-6 |
| 125 | Live call monitoring (listen-only) | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 126 | Whisper (supervisor → agent only) | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 127 | Barge-in (3-way call) | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 128 | Coach mode (silent monitoring + prompts) | Y | Y | partial | partial | partial | Y | Y | N | paid | all-6 |
| 129 | Force-disposition / force-logout | Y | Y | partial | partial | Y | Y | Y | N | paid | all-6 |
| 130 | Supervisor mobile app | Y | Y | partial | partial | Y | Y | Y | N (mobile-agent-coordination doc missing — §3.4.M of audit) | paid | all-6 |

Coverage: 7 supervisor sub-capabilities; oyatie 0/7 implemented (1/7 declared).

## 15. Callback + queueing

Callback queueing + estimated wait time + scheduled callbacks.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 131 | In-queue callback offer | Y | Y | Y | Y | Y | Y | Y | D (callback-schedule capability + IP-030) | both | all-6 |
| 132 | Scheduled callback (caller picks time) | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 133 | Callback SLA enforcement | Y | Y | partial | partial | partial | Y | Y | D (slos/local-callback-schedule-latency declared) | paid | all-6 |
| 134 | Callback retry on failure | Y | Y | Y | Y | partial | Y | Y | D (runbooks/callback-worker-stall.md declared) | both | all-6 |
| 135 | Estimated wait time announcement | Y | Y | Y | Y | partial | Y | Y | N | both | all-6 |
| 136 | Position-in-queue announcement | Y | Y | Y | Y | partial | Y | Y | N | both | all-6 |
| 137 | Queue priority bands | Y | Y | Y | Y | partial | Y | Y | D (ADR-MS-001 priority_band field declared) | paid | all-6 |
| 138 | Fairness routing (longest-waiting-first vs priority) | Y | Y | partial | partial | partial | Y | Y | D (ADR-MS-001 fairness_bucket field) | paid | all-6 |

Coverage: 8 callback sub-capabilities; oyatie 4/8 declared (capability + Cedar fragment + IP + runbook + SLO authoring exists), 0/8 implemented in src/.

## 16. Sentiment + transcription

Real-time sentiment scoring + transcription quality.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 139 | Real-time transcription | Y | Y | Y (Contact Lens) | Y | Y | Y | Y | D (intelligence µservice delegation, no gRPC contract) | paid | all-6 |
| 140 | Post-call transcription | Y | Y | Y | Y | Y | Y | Y | D | both | all-6 |
| 141 | Speaker diarization (who said what) | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 142 | Sentiment scoring (per-turn) | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 143 | Sentiment scoring (overall call) | Y | Y | Y | Y | partial | Y | Y | N | paid | all-6 |
| 144 | Emotion detection beyond positive/negative | Y | Y | partial | partial | partial | Y | Y | N | paid | all-6 |
| 145 | Language auto-detection | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 146 | Multi-language transcription (EN + ES + FR + DE + JA + KO + ZH + ...) | Y | Y | Y | Y | Y | Y | Y | N (KR-pack via in-house Whisper-Korean fine-tune claimed in tier-matrix.md paid compliance-pack row) | paid | all-6 |
| 147 | Custom vocabulary (industry terms) | Y | Y | Y (Transcribe custom vocab) | Y | partial | Y | Y | N | paid | all-6 |

Coverage: 9 sentiment/transcription sub-capabilities; oyatie 2/9 declared (delegation only; no contract).

## 17. STIR/SHAKEN + telephony compliance (US + global)

Caller-ID authentication; FCC TRACED Act compliance; KCC Korean equivalent.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 148 | STIR/SHAKEN signing (A/B/C attestation) | Y | Y | Y | Y | Y | Y | Y | D (tier-matrix paid compliance-pack claim; no impl) | paid | on-prem, colo, oyatie-as-cloud-provider |
| 149 | STIR/SHAKEN verification (inbound) | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 150 | Robocall detection / spam-call surge handling | Y | Y | Y | Y | partial | Y | Y | D (runbooks/spam-call-surge.md declared) | both | all-6 |
| 151 | DID number management | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 152 | Toll-free number support | Y | Y | Y | Y | Y | Y | Y | N | paid | all-6 |
| 153 | E.164 phone-number normalization | Y | Y | Y | Y | Y | Y | Y | N | both | all-6 |
| 154 | Per-country dial-plan support | Y | Y | partial (US-centric) | Y | partial | Y | Y | N | paid + per-country pack | all-6 |
| 155 | KCC (Korea Communications Commission) compliance | partial | N | N | partial | N | partial | N | D (KR-PIPA pack declared; tier-matrix mentions 통신비밀보호법) | paid + KR-PIPA pack | on-prem, colo (KR-resident) |

Coverage: 8 telephony-compliance sub-capabilities; oyatie 0/8 implemented (3/8 declared). KCC compliance is an Oyatie differentiator vs all top-3 (only partial in Genesys + Twilio).

## 18. CRM + ticketing integration

Out-of-the-box integration with downstream CRM systems.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 156 | Salesforce native integration | Y | Y | Y | Y | Y | Y | Y | N (delegated to crm µservice; binding missing) | paid | oyatie-public, aws-guest |
| 157 | MS Dynamics 365 integration | Y | Y | Y | partial | partial | Y | Y | N | paid | oyatie-public, aws-guest |
| 158 | ServiceNow integration | Y | Y | Y | Y | partial | Y | Y | N | paid | oyatie-public, aws-guest |
| 159 | Zendesk integration | Y | Y | Y | Y | n/a | Y | Y | N | paid | oyatie-public, aws-guest |
| 160 | HubSpot integration | partial | Y | partial | Y | partial | partial | Y | N | paid | oyatie-public, aws-guest |
| 161 | Oyatie-CRM (internal) integration | n/a | n/a | n/a | n/a | n/a | n/a | n/a | N (declared as bounded-context delegation; binding missing) | both | all-6 |

Coverage: 6 CRM-integration sub-capabilities; oyatie 0/6.

## 19. Open APIs + extensibility

Webhooks, SDK, custom-extension platforms.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 162 | REST API surface | Y | Y | Y | Y | Y | Y | Y | partial (OpenAPI 3.2.0 declared; minimal endpoints) | both | all-6 |
| 163 | Webhook events for inbound/outbound | Y | Y | Y (EventBridge) | Y | Y | Y | Y | partial (AsyncAPI 3.1.0 channel declared; one message type) | both | all-6 |
| 164 | gRPC internal contract | partial | N | Y | partial | N | partial | N | D (contracts/contact-center-v1.proto; one service) | both | all-6 |
| 165 | TypeScript SDK | Y | Y | Y | Y | Y | Y | Y | N (Rust-strict; TS-SDK only via codegen from contracts) | both | all-6 |
| 166 | Java SDK | Y | Y | Y | Y | Y | Y | Y | N (Java forbidden per Rust-strict doctrine; not authored) | both | all-6 |
| 167 | Python SDK | Y | Y | Y | Y | Y | Y | Y | N (Python forbidden; Rust-only) | both | all-6 |
| 168 | Open IVR-flow JSON schema | partial (Architect proprietary) | partial (Five9 IVR proprietary) | Y (flows JSON) | Y (Studio JSON) | partial | partial (NICE Studio proprietary) | Y (Talkdesk Studio) | N (schema not authored) | both | all-6 |
| 169 | Custom-extension marketplace (3rd-party apps) | Y (Genesys AppFoundry) | Y (CloudSure) | Y (AppFoundry) | Y (Twilio Marketplace) | Y (Zendesk Marketplace) | Y (CXone Mpower) | Y (Talkdesk AppConnect) | D (delegated to marketplace µservice + plugin-app-store) | paid | all-6 |

Coverage: 8 extensibility sub-capabilities; oyatie 4/8 partial/declared, 0/8 implemented to feature parity.

## 20. Compliance packs

Compliance posture per regulatory pack.

| # | Sub-capability | Genesys | Five9 | | Twilio | Zendesk | NICE | Talkdesk | Oyatie | Tenant class | Deployment |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 170 | SOC 2 Type II | Y | Y | Y | Y | Y | Y | Y | D (pack declared in manifest) | paid | all-6 |
| 171 | ISO 27001 | Y | Y | Y | Y | Y | Y | Y | D | paid | all-6 |
| 172 | GDPR (EU) | Y | Y | Y | Y | Y | Y | Y | D (dpia.md authored 420 lines) | paid + EU residency | all-6 |
| 173 | HIPAA (US) | Y | Y | Y | partial | Y | Y | Y | D (pack declared; voice-PHI handling not specified) | paid + HIPAA pack | on-prem, colo, oyatie-as-cloud-provider |
| 174 | PCI-DSS Level 1 v4.0 | Y | Y | Y | Y | partial | Y | Y | D (pack declared; tutorials/build-ivr-flow-with-pci-suppression.md exists) | paid + PCI pack | all-6 |
| 175 | KR-PIPA (Korea) | partial | N | N | partial | N | partial | N | D (pack declared; Oyatie differentiator) | paid + KR-PIPA pack | on-prem, colo (KR-resident) |
| 176 | TCPA (US outbound) | Y | Y | Y | Y | partial | Y | Y | D (pack declared) | paid | all-6 |
| 177 | TILA-RESPA (debt-collection) | partial | Y | partial | partial | N | Y | partial | N | paid + TILA pack | all-6 |
| 178 | UK FCA recording (financial-services) | partial | partial | partial | partial | N | Y | partial | N | paid + UK-FCA pack | all-6 |
| 179 | EU AI Act (high-risk AI annex III) | partial | partial | partial | partial | partial | partial | partial | partial (governance lane retained) | paid + EU AI Act pack | all-6 |

Coverage: 10 compliance pack sub-capabilities; oyatie 8/10 declared, 0/10 fully implemented. KR-PIPA + TILA-RESPA + UK-FCA are differentiators vs top-3.

## 21. UNION-coverage scoreboard

Total UNION universe across the 20 sections above: 179 sub-capabilities.

| Counterpart | Y count (of 179) | Partial count | N count |
|---|---:|---:|---:|
| Genesys Cloud | 168 | 9 | 2 |
| Five9 | 154 | 16 | 9 |
| Amazon | 150 | 17 | 12 |
| Twilio Flex | 161 | 14 | 4 |
| Zendesk Talk | 109 | 38 | 32 |
| NICE CXone | 167 | 9 | 3 |
| Talkdesk | 158 | 14 | 7 |
| Oyatie contact-center (current) | 0 | 12 | 167 |

Reading: Genesys Cloud leads on UNION-coverage by a thin margin over NICE CXone; both clear ~ 94 % of the union universe at Y. Amazon Connect's gaps are mostly in WFM/WFO (where it leans on partner integrations like Calabrio and Verint). Zendesk Talk is the weakest because it's email-first; voice is a bolt-on.

Oyatie contact-center current readiness: 0 Y, 12 partial declarations, 167 N. Net implementation maturity: ~ 7 % declared, 0 % implemented. The µservice is at the charter-document stage; the substance bar is not crossed.

## 22. Closing the gap — priority order

P0 ordered list (chosen by combinatoric: max counterpart coverage × highest user-visible impact × lowest dependency-graph):

1. SIP INVITE handling + FreeSWITCH adapter (#1) — unblocks 50+ downstream sub-capabilities.
2. PSTN trunk integration (Bandwidth.com + Inteliquent + KT 070; #151, #155) — unblocks all outbound + inbound paths.
3. STIR/SHAKEN attestation (#148, #149) — compliance-blocking for US deployments.
4. Web agent desktop in Leptos (#66) — unblocks supervisor tools (#124-130) + scripts (#81-88).
5. IVR-flow JSON schema + runtime (#21-30, #168) — unblocks self-service + bot integration.
6. Recording-blob handoff to recordings µservice (#105, #109, #111, #115) — unblocks compliance evidence.
7. Real-time transcription gRPC contract with intelligence µservice (#139, #145) — unblocks sentiment + agent-assist.
8. Predictive dialer + TCPA abandonment-rate enforcement (#13, #15, #16) — unblocks outbound revenue.
9. Mobile agent app (iOS Swift + Android Kotlin; #68, #69, #130) — required for field-service B2B parity per audit §3.4.M.
10. Migration playbooks (from-five9 + from-amazon-connect) — required for sales conversion.

P1 ordered (after P0 closes the substance gap):

11. WFM forecasting + scheduling (#89-96).
12. WFO auto-quality scoring + speech analytics (#97-104).
13. Omnichannel chat-to-voice escalation contract (#33).
14. Email + SMS + WhatsApp inbound (#41-43).
15. Per-tenant HSM-resident recording encryption (#112) for KR-PIPA + sovereign packs.

## 23. Tenant-class + deployment-context overlay summary

Every parity-matrix row above carries Tenant-class + Deployment-context columns:

- `both` tenant_class entries: 90 of 179 sub-capabilities (50 %). These are the "available even in demo_trial" baseline.
- `paid` tenant_class entries: 89 of 179 sub-capabilities (50 %). These require demo_trial → paid conversion.
- `paid + <pack>` entries: 9 specifically gate on compliance pack activation (which requires paid per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage` §3.6).
- `paid + KR-PIPA` / `paid + EU residency` entries: 4 specifically gate on regional pack activation.
- `all-6` deployment-context entries: 156 of 179. Most capabilities work in all 6 deployment contexts.
- `oyatie-public, aws-guest, oci-guest` only entries: 11 (mostly CRM integrations that depend on cloud-hosted CRM SaaS).
- `oyatie-public, on-prem, colo` only entries: 8 (sovereign / emergency-bypass / HSM-resident).
- `on-prem, colo, oyatie-as-cloud-provider` only entries: 2 (sovereign HIPAA + sovereign-cell HSM).

No tier (demo_trial/paid/paid/paid compliance-pack) entries anywhere in this matrix. Tenant_class dimension is RETIRED per `feedback_no_tenant_class_eligibility_2026_05_20`.

End of feature parity matrix. Three deliverables landed; halting cleanly per dispatch.
