---
doc_class: FAQ
microservice: itsm
persona: itsm-admin
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# itsm — ITSM Administrator FAQ

## Q1: What's the difference between an Incident and a Service Request in ITIL v4?

Incident = something is broken. Service Request = something normal that the user needs but isn't broken (e.g. new laptop request, software install request, access grant request). They share the same ticket primitive in oyatie but route through different workflows, have different SLA classes, and report separately.

## Q2: How does AI-deflection work without exposing my company's KB to a third party?

At retired-advanced tier, the deflection model runs locally on Llama-3.1-70B fine-tuned on your tenant's KB. Articles never leave the substrate. The model embeds each article + each incoming ticket text into a 4096-dim vector + computes cosine similarity to surface the top-3 matches. At retired-sovereign, the same local-only path applies; cross-emit to external LLMs is disabled by construction. Substrate refuses to leak KB content to external services unless your tenant explicitly opts in (rare).

## Q3: How does the substrate integrate with our existing Active Directory / Okta?

Configure SCIM v2 sync: portal → Integrations → "Identity provider sync". Provide the SCIM endpoint + bearer token from your IdP. Users sync hourly (configurable); group memberships drive Cedar role bindings. Most IdPs are supported: Azure AD, Okta, OneLogin, JumpCloud, Google Workspace, Auth0, OneIdentity, Ping. For oyatie's own `iam` µservice, the integration is native.

## Q4: Our CMDB has ~ 200 000 CIs from auto-discovery + manual entry. How do I prevent duplicate CIs?

The substrate's de-duplication engine matches on: (a) hardware: MAC address, serial number, asset-tag. (b) software: install-path + version + tenant scope. (c) services: name + DNS + port. When a discovery agent finds a CI matching an existing CI on these keys, it merges rather than creates duplicate. For manual entries that lack identifying keys, the substrate flags potential duplicates in the review queue for admin disposition.

## Q5: We're migrating from ServiceNow. Will oyatie support our existing custom workflows?

Most workflows port via the converter (`oya itsm workflow-import --source servicenow`). The converter handles ~ 75 % of standard workflow nodes automatically. Custom JavaScript in ServiceNow Business Rules don't auto-port — you must rewrite as oyatie workflow steps (Cedar policies, HTTP calls, conditional approvals). Plan 2-4 weeks of workflow review + porting time for medium-complexity tenants; longer for tenants with extensive custom JavaScript.

## Q6: Can we attach files (screenshots, log dumps) to tickets? What's the size limit?

Yes. retired-basic: 25 MB/file. retired-standard: 250 MB/file. retired-advanced: 1 GB/file. Attachments are scanned by ClamAV antivirus before being stored. PHI, PII, and PCI data in attachments are auto-redacted (configurable per pack overlay). Files are encrypted at rest using SeaweedFS encryption + (retired-sovereign) per-tenant HSM keys.

## Q7: How does the change-management workflow interact with our git PRs?

At retired-advanced tier, the substrate integrates with `oya git`. When a PR is opened that touches a CMDB-tracked CI, the substrate auto-creates a Standard Change ticket linked to the PR. The PR can only merge if (a) the Change is approved (auto-approved for standard-change templates; CAB-reviewed for normal changes), and (b) CI tests pass. This is the DevOps-style "change as code" pattern.

## Q8: A regulator wants evidence of our change-management process for the last 12 months. How do I produce it?

Portal → Reports → "Regulator evidence export". Filter by change-type, date range, CIs affected, approvers. Export format: ZIP with (a) every Change ticket with its lifecycle + approvals; (b) audit-chain Merkle proofs; (c) CAB meeting minutes (if recorded); (d) PR references where DevOps-integrated. SLA: ≤ 24 h for ≤ 10 000 changes. This satisfies ITIL v4 + ISO 20000-1 + SOC 2 CC8.1 + ISO 27001 A.5.36 audit requirements.

## Q9: What's the SLA escalation behaviour for tickets approaching breach?

When a ticket is at 75 % of its SLA window without a status change, the assigned agent + their manager get a Slack/Discord/Telegram alert via the `notifications` µservice. At 90 %, the assignment group manager + assignment-group-on-call get alerts. At 100 % (breach), the alert escalates to the ITSM admin and the substrate emits an `itsm::sla::breached` event for reporting.

## Q10: Can multiple agents collaborate on a single ticket?

Yes. The "Assigned to" field has a single primary agent (responsible). Additional agents can be added as "Collaborators" with full read/write access. Tasks within a ticket can be assigned to different agents (e.g. one agent does network diagnostics, another does access verification). All actions are logged with the acting agent.

## Q11: Our pack is KR-PIPA-Finance. Are there special change-management requirements?

Yes. The pack overlay enforces:
- Any change to a system processing 개인정보 requires CISO sign-off (KR-PIPA Art. 31).
- Any change to a system processing financial transactions requires CFO + CISO joint sign-off (전자금융감독규정).
- Change-management evidence retained 7 years minimum.
- The substrate auto-detects changes affecting PIPA-classified or finance-classified CMDB CIs and routes through the elevated approval workflow.
- Emergency changes still require post-hoc sign-off within 24 h.

## Q12: How does the asset/license tracker handle compliance for software-license audits?

The Asset Management module tracks: (a) licensed copies per software product, (b) installed copies (from CMDB discovery), (c) license consumption + expiry. Reports surface compliance gaps (under-licensed = audit risk; over-licensed = wasted spend). For specific compliance regimes:
- Microsoft SAM (Software Asset Management): export in SAM-Lite format.
- Oracle LMS: track CPU + core counts; export the Oracle LMS template.
- IBM IL (International License): track PVU; export the IBM IL template.
- Open-source: track copyleft licenses (GPL/AGPL/LGPL/MPL) with usage scope to ensure license obligations are met.
