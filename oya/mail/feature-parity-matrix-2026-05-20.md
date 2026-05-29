# Mail Feature Parity Matrix

Audit date: 2026-05-20
Target microservice: `mail`
Counterpart 1: Gmail
Counterpart 2: Microsoft Outlook
Counterpart 3: Proton Mail
Purpose: compare Oyatie mail against the union surface of the three required counterparts.
Scope: product, protocol, admin, privacy, compliance, operations, migration, and developer surfaces.
Local anchors: `PRD.md:27-34`, `ARCHITECTURE.md:43-55`, `contracts/openapi/mail.yaml:1`, `contracts/asyncapi/mail-events.yaml:1`, `contracts/proto/mail.proto:5`.
Counterpart anchors: Gmail Workspace public support docs, Microsoft Learn Exchange Online limits, and Proton Mail support docs.
Interpretation: a present Oyatie artifact is not treated as production-complete unless it has matching implementation, test, IaC, and operational evidence.

## 1. Counterpart-1 Capability Surface: Gmail

1. Gmail sets the highest public consumer expectation for fast inbox search, spam filtering, labels, conversation threading, and large-scale deliverability.
2. Gmail also sets a Workspace administration expectation for domain-managed mail, routing, retention, Vault, compliance controls, and API automation.
3. Public source anchor: Google Workspace Gmail sending limits at `https://support.google.com/a/answer/166852`.
4. Public source anchor: Gmail attachment behavior at `https://support.google.com/mail/answer/6584`.
5. Public source anchor: Google Workspace storage and Gmail bandwidth support pages for quota and client access context.
6. Oyatie local anchor: `PRD.md:78-120` contains compose, reading, threading, search, and attachment feature rows.
7. Oyatie local anchor: `PRD.md:160-170` contains protocol and client posture.
8. Oyatie local anchor: `PRD.md:952-968` contains latency, throughput, and workflow targets.
9. Gmail baseline family: hosted mailbox, personal and business accounts, web app, mobile apps, offline access, and third-party client access.
10. Gmail baseline family: SMTP send, IMAP access, Gmail API, Google Workspace admin controls, and domain authentication.
11. Gmail baseline family: labels, filters, search operators, categories, priority inbox, and conversation views.
12. Gmail baseline family: attachment scan, Drive handoff for large files, phishing warnings, and spam quarantine.
13. Gmail baseline family: delegation, group aliases, send-as, shared mail-style workflows through Groups or delegation.
14. Gmail baseline family: Google Workspace Vault, retention, legal hold, export, and audit.
15. Gmail baseline family: admin policy for routing, compliance, DLP, TLS, and S/MIME in supported Workspace plans.
16. Gmail baseline family: broad migration from IMAP and Workspace Admin tools.
17. Gmail baseline family: global anti-abuse and reputation learning with a very large training base.
18. Gmail baseline family: mobile push, web push, and offline-friendly client behavior.
19. Gmail gap pressure for Oyatie: spam and phishing precision must be exceptional, not merely rule-based.
20. Gmail gap pressure for Oyatie: search syntax and ranking need to feel instant and explainable.
21. Gmail gap pressure for Oyatie: domain onboarding must hide DNS complexity without losing custody.
22. Gmail gap pressure for Oyatie: import tooling must be trustworthy for large mailboxes and Google Workspace migration.
23. Gmail gap pressure for Oyatie: admin audit and Vault-like workflows must be precise enough for legal teams.
24. Oyatie strength against Gmail: `PRD.md:1003-1010` gives stronger custody and encryption ambition than baseline Gmail.
25. Oyatie strength against Gmail: `ARCHITECTURE.md:171` names BYOK posture, though the free/paid wording is stale.
26. Oyatie strength against Gmail: `policy/*.cedar` inventory shows first-class authorization policy rather than only admin console posture.
27. Oyatie weakness against Gmail: no implementation source is present under `src/`.
28. Oyatie weakness against Gmail: no test harness is present under `tests/`.
29. Oyatie weakness against Gmail: `PRD.md:1021-1023` leaves the mail-server backend decision stale relative to accepted ADRs and current tenant-class doctrine.
30. Oyatie weakness against Gmail: existing benchmark rows use retired rank language and missing harness evidence.
31. Parity requirement: Gmail-like search must include full-text, phrase, sender, recipient, date, attachment, label, operator, and saved search behavior.
32. Parity requirement: Gmail-like spam must include user feedback, global reputation, tenant reputation, attachment scanning, URL scanning, and model governance.
33. Parity requirement: Gmail-like migration must include IMAP, Google Workspace export, label mapping, thread mapping, aliases, filters, and delegation mapping.
34. Parity requirement: Gmail-like admin must include domain verification, DKIM, SPF, DMARC, routing, aliases, hold, export, audit, and alerting.
35. Parity requirement: Gmail-like client quality must include fast compose, undo send, drafts, autosave, attachment progress, offline queueing, keyboard workflow, and mobile push.
36. Current Oyatie posture: product specs aim at this surface; evidence does not yet prove execution.
37. Gmail family verdict: parity target is plausible on paper but blocked by implementation, IaC, OS, and benchmark evidence gaps.

## 2. Counterpart-2 Capability Surface: Microsoft Outlook

1. Microsoft Outlook sets the broadest enterprise expectation because it includes Outlook clients, Exchange Online, Microsoft 365 administration, Purview, Defender, retention, eDiscovery, and hybrid enterprise operations.
2. Public source anchor: Microsoft Learn Exchange Online limits at `https://learn.microsoft.com/en-us/office365/servicedescriptions/exchange-online-service-description/exchange-online-limits`.
3. Public source anchor: Microsoft 365 Exchange service descriptions and Purview/eDiscovery documentation for enterprise compliance posture.
4. Oyatie local anchor: `PRD.md:1014-1018` contains deliverability and trust targets.
5. Oyatie local anchor: `compliance.md` contains compliance mappings.
6. Oyatie local anchor: `failure-modes.md:27-194` contains operational failure modes.
7. Outlook baseline family: mailbox, calendar-adjacent messaging, contacts, tasks, shared mailboxes, distribution groups, aliases, and mail-enabled resources.
8. Outlook baseline family: Exchange transport rules, journaling, DLP, retention, litigation hold, eDiscovery, audit, and discovery export.
9. Outlook baseline family: Defender phishing protection, safe links, safe attachments, impersonation protection, and quarantine.
10. Outlook baseline family: Outlook desktop, web, mobile, IMAP/POP/SMTP, Exchange protocol ecosystem, and Graph API.
11. Outlook baseline family: tenant admin, RBAC, delegated admin, compliance center, reporting, and hybrid.
12. Outlook baseline family: mailbox restore, inactive mailboxes, shared mailbox governance, and enterprise recovery.
13. Outlook baseline family: large enterprise migration, coexistence, transport connectors, and domain verification.
14. Outlook baseline family: high availability backed by Microsoft global cloud and operational support.
15. Outlook gap pressure for Oyatie: legal hold and eDiscovery need court-order and audit-chain rigor.
16. Outlook gap pressure for Oyatie: transport rules and DLP need enterprise-grade authoring, testing, and audit.
17. Outlook gap pressure for Oyatie: shared mailbox and delegation semantics need to be first-class, not an afterthought.
18. Outlook gap pressure for Oyatie: migration and coexistence need domain, routing, and DNS safety.
19. Outlook gap pressure for Oyatie: hybrid and on-prem contexts need explicit mail-flow contracts.
20. Oyatie strength against Outlook: `PRD.md:1014-1018` aims at DMARC, DKIM, MTA-STS, TLSRPT, and BIMI.
21. Oyatie strength against Outlook: `PRD.md:1003-1010` aims at zero server-side plaintext for personal mail and BYOK for work mail.
22. Oyatie strength against Outlook: `ARCHITECTURE.md:1153-1158` lists cross-service dependencies, which is useful for enterprise integration.
23. Oyatie weakness against Outlook: incident-response runbook references are not path-complete.
24. Oyatie weakness against Outlook: OpenTofu context modules are absent for on-prem, colo, and guest deployments.
25. Oyatie weakness against Outlook: no OS manifest proves the service can run across the required server fleet.
26. Oyatie weakness against Outlook: accepted backend ADR splits quality by stale tenant ranks rather than workload and context.
27. Parity requirement: Outlook-like eDiscovery must include legal hold, preservation, search, export, audit, authorization, evidence chain, and pack overlays.
28. Parity requirement: Outlook-like transport governance must include rules, exception handling, quarantine, external labeling, DLP, and alert routing.
29. Parity requirement: Outlook-like delegated administration must include role scope, break-glass, just-in-time approval, evidence capture, and revocation.
30. Parity requirement: Outlook-like migration must include coexistence, staged cutover, routing, historical archive import, and rollback.
31. Parity requirement: Outlook-like enterprise recovery must include mailbox restore, domain compromise recovery, key rotation, queue drain, and reputation recovery.
32. Current Oyatie posture: strong design ambition, missing evidence for enterprise deployability.
33. Outlook family verdict: Oyatie mail needs the most work here because enterprise operations require exact IaC and runbook path integrity.

## 3. Counterpart-3 Capability Surface: Proton Mail

1. Proton Mail sets the privacy and trust-counterpart expectation.
2. Public source anchor: Proton Mail sending limits at `https://proton.me/support/email-sending-limits`.
3. Public source anchor: Proton Mail attachments at `https://proton.me/support/attachments`.
4. Public source anchor: Proton storage support at `https://proton.me/support/increase-storage-space`.
5. Oyatie local anchor: `decisions/ADR-MAIL-0001-personal-mail-key-recovery.md:56-60` for recovery mode.
6. Oyatie local anchor: `PRD.md:1003-1010` for encryption and key custody targets.
7. Oyatie local anchor: `policy/tenant-scope.cedar` and `policy/public-read.cedar` for policy inventory.
8. Proton baseline family: E2EE between Proton users, password-protected external messages, zero-access encryption posture, and account recovery controls.
9. Proton baseline family: privacy-focused web and mobile apps, Bridge for desktop clients on supported plans, custom domains, aliases, and import/export.
10. Proton baseline family: encrypted contacts, calendars in broader suite, anti-tracking, and simple privacy-first UX.
11. Proton baseline family: recovery phrase, recovery file, account password separation, and account security posture.
12. Proton baseline family: paid plans with larger storage, multiple addresses, custom domains, and VPN/suite bundling.
13. Proton gap pressure for Oyatie: key custody language must be precise and tested.
14. Proton gap pressure for Oyatie: recovery must not create operator plaintext access.
15. Proton gap pressure for Oyatie: personal and work contexts must not leak across each other.
16. Proton gap pressure for Oyatie: mobile and desktop access must not degrade privacy posture.
17. Proton gap pressure for Oyatie: import/export must preserve encryption and audit controls.
18. Oyatie strength against Proton: `PRD.md:1003-1010` explicitly targets client-side E2EE for personal mail and BYOK for work mail.
19. Oyatie strength against Proton: `PRD.md:52-58` models Cedar-isolated personal and work context, though one policy path is stale.
20. Oyatie strength against Proton: `decisions/ADR-MAIL-0001-personal-mail-key-recovery.md` gives a serious recovery design.
21. Oyatie weakness against Proton: the recovery ADR must gate escrow with tenant_class language.
22. Oyatie weakness against Proton: no client implementation evidence proves privacy posture.
23. Oyatie weakness against Proton: no conformance tests prove ciphertext, envelope, recovery, and audit behavior.
24. Oyatie weakness against Proton: no tenant-class semantics express who may use BYOK or compliance packs under the new model.
25. Parity requirement: Proton-like E2EE must include encryption at rest, key isolation, external recipient flows, recovery, device trust, and auditable operator non-access.
26. Parity requirement: Proton-like migration must handle import without losing privacy guarantees.
27. Parity requirement: Proton-like bridge/client posture must be clear for desktop users who expect IMAP/SMTP access.
28. Parity requirement: Proton-like custom-domain setup must be simple but cryptographically honest.
29. Current Oyatie posture: trust design is ambitious and differentiated, but still document-only.
30. Proton family verdict: Oyatie has a strong design thesis but needs tenant-class rewrite and proof of crypto/client behavior.

## 4. Union-Coverage Matrix

| ID | Capability | Gmail | Outlook | Proton Mail | Oyatie evidence and gap |
| --- | --- | --- | --- | --- | --- |
| CAP-001 | Hosted mailbox creation | Core | Core | Core | `PRD.md:291` covers signup and primary mailbox; tenant-class language is missing. |
| CAP-002 | Custom domain onboarding | Core Workspace | Core Exchange | Core paid-plan surface | `PRD.md:185` and `PRD.md:291` cover custom domain; needs current tenant-class model. |
| CAP-003 | SMTP inbound receive | Core | Core | Core | `PRD.md:27-34`; `contracts/proto/mail.proto:5`; no implementation proof. |
| CAP-004 | SMTP outbound send | Core | Core | Core | `PRD.md:952-968` gives target latency; no OpenTofu egress modules. |
| CAP-005 | IMAP access | Core | Supported | Bridge/client dependent | `PRD.md:160-170` includes IMAP4rev2; no tests found. |
| CAP-006 | JMAP access | Not primary | Not primary | Not primary public baseline | Oyatie differentiates with JMAP; `IP-016-jmap-rfc-8620-frontend.md` exists. |
| CAP-007 | REST API | Gmail API | Graph API | Limited public emphasis | `contracts/openapi/mail.yaml:1` exists; implementation absent. |
| CAP-008 | Event API | Admin/activity events | Audit/event surfaces | Limited | `contracts/asyncapi/mail-events.yaml:1` exists; consumer proof absent. |
| CAP-009 | Protobuf internal API | Not public | Not public | Not public | `contracts/proto/mail.proto:5` exists; Rust service absent. |
| CAP-010 | Fast compose | Core | Core | Core | `PRD.md:952-968` targets compose latency; UI implementation absent. |
| CAP-011 | Draft autosave | Core | Core | Core | `PRD.md:78-120` covers compose family; needs implementation evidence. |
| CAP-012 | Undo send | Core | Core | Core enough | Present in product expectations; verify in PRD feature table before implementation. |
| CAP-013 | Attachments | Core | Core | Core | `PRD.md:78-120`; benchmark doc covers DKIM/attachment-adjacent throughput, not full implementation. |
| CAP-014 | Large attachment handoff | Drive handoff | OneDrive handoff | Plan-dependent storage | Oyatie needs equivalent object-store handoff via mailbox-store S3 adapter. |
| CAP-015 | Attachment malware scan | Core | Defender | Core security expectation | `policy/abuse-defence.cedar`; `failure-modes.md`; no scan implementation. |
| CAP-016 | URL scanning | Core | Defender Safe Links | Security expectation | `IP-017-anti-phishing-edge-wiring.md`; no implementation proof. |
| CAP-017 | Sender authentication warnings | Core | Core | Core | `PRD.md:1014-1018` and ADR-MAIL-001 cover auth posture. |
| CAP-018 | Conversation threading | Core | Core | Core | `PRD.md:78-120`; needs contract/test coverage. |
| CAP-019 | Labels/folders | Labels | Folders/categories | Folders/labels | Product surface should map Gmail labels and Outlook folders during migration. |
| CAP-020 | Search operators | Core strength | Core | Basic-to-strong | `PRD.md:952-968` target search latency; syntax coverage not proven. |
| CAP-021 | Full-text search | Core | Core | Core | `catalog/oya-mail-search-index-adapter-tantivy.yaml`; no implementation proof. |
| CAP-022 | Saved search | Core | Search folders | Saved filters less central | Not clearly evidenced; add to PRD or declare out of first release. |
| CAP-023 | Spam filtering | Industry leader | Defender | Strong privacy baseline | `ADR-MAIL-0004`; benchmark doc has classifier numbers; proof absent. |
| CAP-024 | User spam feedback | Core | Core | Core | Needs explicit feedback loop and abuse model evidence. |
| CAP-025 | Phishing detection | Core | Defender | Core | `IP-017-anti-phishing-edge-wiring.md`; needs runtime evidence. |
| CAP-026 | Abuse desk workflow | Core platform | Core enterprise | Trust and safety | Missing context-specific runbook coverage for abuse and reputation. |
| CAP-027 | DKIM signing | Core admin | Core admin | Custom domain surface | ADR-MAIL-001 covers signing key custody. |
| CAP-028 | SPF validation | Core | Core | Core | ADR-MAIL-001 and tutorial cover SPF/DMARC posture. |
| CAP-029 | DMARC policy rollout | Core admin | Core admin | Custom domain surface | `tutorials/promote-dmarc-policy-with-soak.md` exists but uses retired rank language. |
| CAP-030 | MTA-STS | Enterprise-grade | Enterprise-grade | Security expectation | FAQ mentions MTA-STS using retired rank language; needs rewrite. |
| CAP-031 | TLSRPT | Enterprise-grade | Enterprise-grade | Security expectation | `PRD.md:1014-1018`; needs implementation evidence. |
| CAP-032 | ARC forwarding | Important for forwarders | Important | Less visible | Capability matrix has ARC but retired model; rewrite needed. |
| CAP-033 | BIMI | Brand/trust feature | Brand/trust feature | Less central | `PRD.md:1014-1018` mentions BIMI; implementation absent. |
| CAP-034 | Admin domain verification | Core | Core | Core paid-plan surface | Onboarding docs cover DNS challenge, but stale language. |
| CAP-035 | Multi-domain tenancy | Core Workspace | Core Exchange | Plan-dependent | `PRD.md:185`; tenant-class model absent. |
| CAP-036 | Aliases | Core | Core | Core | Need explicit artifact mapping for alias ownership and audit. |
| CAP-037 | Send-as identities | Core | Core | Core | Need DKIM and authorization contract coverage. |
| CAP-038 | Delegated mailbox | Core | Core | Privacy-sensitive | Needs Cedar policy and UI flow evidence. |
| CAP-039 | Shared mailbox | Limited by pattern | Core Exchange | Less central | Outlook parity requires stronger treatment than current PRD shows. |
| CAP-040 | Distribution groups | Google Groups | Core Exchange | Less central | Likely belongs to adjacent identity/groups services; handoff must be explicit. |
| CAP-041 | Mailing lists | Groups | Distribution groups | Limited | Cross-service ownership not fully resolved. |
| CAP-042 | Transport rules | Workspace routing | Core Exchange | Limited | Outlook parity gap; add contracts and admin UI specs. |
| CAP-043 | Compliance routing | Workspace admin | Core Exchange | Limited | `compliance.md`; stale Terraform path needs cleanup. |
| CAP-044 | DLP quarantine | Workspace DLP | Purview DLP | Privacy-first filtering | `runbooks/dlp-quarantine-release.md` exists; policy proof needed. |
| CAP-045 | PHI DLP | Enterprise/compliance | Purview | Plan-sensitive | `policy/phi-dlp.cedar`; `packs/HIPAA.md`; no tests. |
| CAP-046 | Retention policies | Vault | Purview retention | Privacy balance | `IP-010-retention-policy.md`; SLO/evidence needed. |
| CAP-047 | Legal hold | Vault | Litigation hold | Privacy-sensitive | `IP-011-legal-hold-engine.md`; SLO file exists. |
| CAP-048 | eDiscovery export | Vault | Purview eDiscovery | Export with privacy constraints | `IP-012-ediscovery-export.md`; SLO file exists. |
| CAP-049 | Audit chain | Workspace audit | Microsoft audit | Security logs | `PRD.md` and runbooks imply; needs implementation. |
| CAP-050 | Court-order evidence | Vault workflow | Purview workflow | Privacy-sensitive | Requires exact Cedar and audit-chain controls. |
| CAP-051 | BYOK | Workspace Customer Key in some contexts | Customer Key in enterprise contexts | Privacy default differs | `ARCHITECTURE.md:171` names BYOK but uses stale free/paid wording. |
| CAP-052 | Personal E2EE | Not default | Not default | Core differentiator | `PRD.md:1003-1010` targets this; implementation absent. |
| CAP-053 | External encrypted message | Limited/confidential mode | Encryption options | Core password/external flows | Needs explicit Oyatie flow. |
| CAP-054 | Recovery phrase/file | Not Proton-like | Enterprise recovery | Core Proton expectation | ADR-MAIL-0001 covers recovery design but stale gating. |
| CAP-055 | Operator non-access proof | Limited | Limited | Core trust claim | Needs tests and evidence for all tenant classes. |
| CAP-056 | Key rotation | Enterprise admin | Enterprise admin | Core security | `runbooks/dkim-key-rotation.md`; mailbox key rotation proof also needed. |
| CAP-057 | DKIM key custody | Admin feature | Admin feature | Custom domain trust | ADR-MAIL-001 is strong; OpenTofu/OpenBao proof absent. |
| CAP-058 | HSM integration | Enterprise advanced | Enterprise advanced | Less visible | Old docs mention HSM via retired model; rewrite around compliance pack/context. |
| CAP-059 | OpenBao integration | Not relevant | Not relevant | Not relevant | `iac/openbao-policy.yaml`; no context module. |
| CAP-060 | Mailbox store | Proprietary | Exchange store | Proton store | `catalog/oya-mail-mailbox-store-*`; no implementation. |
| CAP-061 | Object storage attachments | Drive/Gmail internals | OneDrive/Exchange internals | Proton storage | `catalog/oya-mail-mailbox-store-adapter-s3.yaml`; no OpenTofu. |
| CAP-062 | PostgreSQL metadata | Not public | Not public | Not public | `capacity-model.md` describes Postgres; implementation absent. |
| CAP-063 | Search index | Gmail proprietary | Exchange search | Proton search | Tantivy adapter catalog exists. |
| CAP-064 | Queueing | Gmail internal | Exchange transport | Proton internal | `runbooks/smtp-queue-backup.md`; no context IaC. |
| CAP-065 | Bounce handling | Core | Core | Core | Needs explicit artifact in contracts/runbooks. |
| CAP-066 | Complaint feedback loop | Core sender reputation | Core sender reputation | Trust expectation | Not path-complete; add abuse feedback loop. |
| CAP-067 | IP warm-up | Workspace sender posture | Exchange Online Protection | Custom domain sender posture | ADR-MAIL-0002 mentions IP pool; OpenTofu absent. |
| CAP-068 | Reputation monitoring | Core | Core | Core | `dashboards/dmarc-deliverability.json`; more runbooks needed. |
| CAP-069 | Deliverability dashboard | Admin console | Exchange admin | Limited | Dashboard files exist; runtime evidence absent. |
| CAP-070 | User inbox dashboard | Gmail UX | Outlook UX | Proton UX | `dashboards/inbox-experience.json`; UI absent. |
| CAP-071 | Admin quarantine | Gmail admin | Defender quarantine | Security expectation | DLP runbook exists; admin UI not proven. |
| CAP-072 | User quarantine release | Gmail spam folder | Outlook quarantine | Spam folder | Needs Cedar and abuse policy evidence. |
| CAP-073 | Mobile push | Core | Core | Core | `PRD.md:160-170`; native client plan unclear. |
| CAP-074 | Offline web access | Core | Core | Limited/different | Next.js reference is stale; Leptos posture needed. |
| CAP-075 | Desktop client support | IMAP/Gmail API | Outlook desktop | Bridge | IMAP/JMAP plan exists; bridge-equivalent not explicit. |
| CAP-076 | Native mobile SDK | Gmail APIs | Graph/Exchange | App internals | SDK docs need generated policy and frontend allowlist. |
| CAP-077 | Migration from Gmail | Admin migration | Exchange migration | Import/export | `migration-playbooks/from-gmail-workspace.md` exists. |
| CAP-078 | Migration from Outlook | Possible via IMAP/Exchange tools | Native source | Import/export | Needs explicit Outlook playbook. |
| CAP-079 | Migration from Proton | IMAP/Bridge/export | Import path | Native source | Needs privacy-preserving import plan. |
| CAP-080 | Backfill replay | Workspace migration | Exchange migration | Import/export | `backfill-replay.md`; throughput proof needed. |
| CAP-081 | Cutover rollback | Workspace admin | Exchange hybrid | Domain reset | Needs runbook and OpenTofu DNS module. |
| CAP-082 | Multi-region | Google global | Microsoft global | Proton region posture | `multi-region.md`; OpenTofu absent. |
| CAP-083 | Data residency | Workspace regions | Microsoft regions | Proton jurisdiction | `policy/data-residency.md`; tenant-class integration missing. |
| CAP-084 | HIPAA posture | Workspace BAA | Microsoft BAA | Limited depending product | `packs/HIPAA.md`; BAA flow needs tenant class. |
| CAP-085 | GDPR posture | Workspace | Microsoft | Proton strength | `packs/GDPR.md`; privacy proof absent. |
| CAP-086 | KR PIPA posture | Regional pack | Regional compliance | Privacy alignment | `packs/KR-PIPA.md`; context evidence missing. |
| CAP-087 | SOC 2 posture | Enterprise | Enterprise | Security trust | `packs/SOC2.md`; audit evidence path needed. |
| CAP-088 | EU AI Act classifier scope | Not a direct product feature | Defender classifier governance | Filter governance | ADR-MAIL-0004 exists; missing policy paths. |
| CAP-089 | Minor protection | Family/admin controls | Family/admin controls | Privacy-sensitive | `policy/minor-protection.cedar`; architecture uses stale paid family language. |
| CAP-090 | Dual personal/work context | Google account separation | Microsoft tenant separation | Personal privacy | `PRD.md:42-58`; policy path mismatch exists. |
| CAP-091 | Cross-tenant isolation | Workspace tenants | M365 tenants | Proton account isolation | `slos/dual-context-correctness.openslo.yaml`; tests absent. |
| CAP-092 | Friendly crawler public folders | Less central | Less central | Not central | `ARCHITECTURE.md:171` includes crawler audience type; unique Oyatie surface. |
| CAP-093 | Public-read policy | Not core mail | Not core mail | Not core mail | `policy/public-read.cedar`; needs product clarity. |
| CAP-094 | Workflow handoff | Gmail add-ons/workflows | Microsoft add-ins/Power Automate | Limited | `IP-013-mail-workflow-handoff.md`; event contract exists. |
| CAP-095 | Plugin mail actions | Gmail add-ons | Outlook add-ins | Limited | `IP-journey-j74-plugin-mail-actions.md`; developer surface needs governance. |
| CAP-096 | Billing receipts mail | Workspace app ecosystem | M365 app ecosystem | Generic | Multiple journey IPs prove mail as substrate for business events. |
| CAP-097 | Healthcare notification | Workspace/HIPAA contexts | M365 healthcare | Privacy contexts | Journey IPs and HIPAA pack exist; deployable evidence absent. |
| CAP-098 | Marketplace notification | Gmail business use | Outlook business use | Generic | Journey IP exists; revenue-share tenant class missing. |
| CAP-099 | Authority notices | Generic | Generic | Generic | Multiple journey IPs exist; compliance audit chain needed. |
| CAP-100 | Regulator notifications | Vault/audit support | Purview support | Privacy support | `IP-journey-j66-regulator-notifications.md`; runbook integration needed. |
| CAP-101 | Mail archive on leaver | Google Vault/admin | Exchange inactive mailbox | Export | `IP-journey-j127-mail-archive-on-leaver.md`; needs legal-hold link. |
| CAP-102 | Hiring mail cascade | Gmail workflow | Outlook workflow | Generic | Journey IP exists; not counterpart-critical. |
| CAP-103 | RIF mail templates | Gmail workflow | Outlook workflow | Generic | Journey IP exists; risk is HR/legal content governance. |
| CAP-104 | Corporate audit pull | Vault | Purview | Export | Journey IP exists; eDiscovery engine must be exact. |
| CAP-105 | Support email bridge | Gmail collaborative inbox | Outlook shared mailbox | Generic | `IP-journey-j49-support-email-bridge.md`; shared mailbox model needed. |
| CAP-106 | iMIP invite bridge | Calendar integration | Core Outlook calendar | Calendar support | `IP-journey-j27-imip-invite-bridge.md`; calendar ownership handoff needed. |
| CAP-107 | Auto-translate thread | Gmail translate | Outlook translate | Limited | `IP-journey-j72-auto-translate-thread.md`; AI policy needed. |
| CAP-108 | Auto-reply and digest | Gmail vacation/filters | Outlook rules | Core | `IP-journey-j144-auto-reply-and-digest-delivery.md`; rule engine evidence needed. |
| CAP-109 | Marketplace seller notices | Business use | Business use | Generic | Revenue-share class should own this economics surface. |
| CAP-110 | Affiliate/reseller embedded mail | Business use | Business use | Generic | Revenue-share class missing from artifacts. |
| CAP-111 | On-prem operation | Not customer-hosted baseline | Hybrid history | Not baseline | Required by Oyatie contexts; OpenTofu missing. |
| CAP-112 | Colo operation | Not customer-hosted baseline | Hybrid history | Not baseline | Required by Oyatie contexts; OpenTofu missing. |
| CAP-113 | Guest AWS operation | SaaS internally | SaaS internally | SaaS internally | Required by Oyatie contexts; OpenTofu missing. |
| CAP-114 | Guest OCI operation | SaaS internally | SaaS internally | SaaS internally | Required by Oyatie contexts; Always Free profile missing. |
| CAP-115 | Oyatie public cloud operation | Hosted baseline | Hosted baseline | Hosted baseline | Required; context module missing. |
| CAP-116 | Oyatie-as-cloud-provider operation | Unique | Unique | Unique | Required; context module missing. |
| CAP-117 | OS support manifest | Not public product | Not public product | Not public product | Required by Oyatie; absent. |
| CAP-118 | Rust backend proof | Not public product | Not public product | Not public product | Required by Oyatie; no `src/`. |
| CAP-119 | Rust conformance tests | Not public product | Not public product | Not public product | Required by Oyatie; no `tests/`. |
| CAP-120 | Single quality bar across tenant classes | Not equivalent | Not equivalent | Not equivalent | Required by current directive; no `tenant_class` semantics. |

## 5. Family Summary

1. Compose and send family: Oyatie has strong latency targets and protocol intent, but lacks implementation and test evidence.
2. Compose and send family: Gmail parity requires autosave, undo, attachments, rich compose, send-as, aliases, and trust warnings.
3. Compose and send family: Outlook parity requires enterprise transport rules, delegated send, shared mailbox, and compliance routing.
4. Compose and send family: Proton parity requires encryption posture and external secure-message behavior.
5. Read and organize family: Oyatie has search/index ambitions through Tantivy and product tables.
6. Read and organize family: Gmail sets the most demanding label/search/operator behavior.
7. Read and organize family: Outlook sets folder, category, shared mailbox, and enterprise retention expectations.
8. Read and organize family: Proton sets privacy-preserving mailbox access expectations.
9. Protocol family: Oyatie is strongest when it leads with JMAP plus IMAP4rev2 compatibility.
10. Protocol family: Gmail API and Microsoft Graph are API expectation anchors, even when protocols differ.
11. Protocol family: Proton Bridge creates a user expectation that privacy systems still interoperate with desktop clients.
12. Admin and domain family: Oyatie has DKIM/SPF/DMARC plans and tutorial artifacts.
13. Admin and domain family: the current docs need context-specific OpenTofu modules before they can claim deployability.
14. Admin and domain family: DNS automation must preserve tenant custody and audit.
15. Security and abuse family: Oyatie has strong policy and ADR ambitions.
16. Security and abuse family: Gmail remains the bar for spam precision and user feedback learning loops.
17. Security and abuse family: Outlook remains the bar for enterprise Defender-style admin response.
18. Security and abuse family: Proton remains the bar for user trust and operator non-access claims.
19. Compliance family: Oyatie has packs for HIPAA, GDPR, KR PIPA, SOC 2, and EU AI Act.
20. Compliance family: `compliance.md:85` contains a stale Terraform path and must be corrected.
21. Compliance family: legal hold and eDiscovery exist as plans and SLOs, but implementation proof is missing.
22. Recovery family: ADR-MAIL-0001 is strong design work.
23. Recovery family: ADR-MAIL-0001 must be rewritten away from tenant_class-absent gating.
24. Migration family: Gmail migration has a playbook.
25. Migration family: Outlook and Proton migration need explicit playbooks to satisfy the union bar.
26. Operations family: failure modes and incident response are strong starts.
27. Operations family: runbook path mismatches reduce incident usability.
28. Deployment family: all six contexts remain in scope.
29. Deployment family: context-specific OpenTofu is the largest gap.
30. Tenant economics family: `demo_trial`, `paid`, and `revenue_share` are absent.
31. Tenant economics family: quality must be uniform while caps and economics differ.
32. Developer family: contracts exist, but generated SDK policy and Rust implementation proof are missing.
33. UI family: current web wording is stale against Leptos SSR plus selective island hydration.
34. Mobile family: native client claims need to stay in allowed frontend stacks and should probably live outside the backend microservice path.
35. Overall family result: Oyatie mail is concept-rich but not yet counterpart-complete in execution evidence.

## 6. Headline Gap Analysis

1. Gap A: deployment evidence is below product ambition.
2. Evidence: all counterpart-grade mail systems require exact DNS, egress, reputation, queue, storage, and recovery operations; mail has no six-context OpenTofu modules.
3. Impact: a paper feature cannot safely become production mail without context-specific deliverability and abuse controls.
4. Gap B: tenant-class model is absent.
5. Evidence: no `tenant_class`, `demo_trial`, or `revenue_share` matches exist under mail.
6. Impact: current docs cannot represent the current business model or OCI Always Free profile correctly.
7. Gap C: retired capability-rank language is pervasive.
8. Evidence: 73 exact references are listed in the coherence audit.
9. Impact: future implementers will keep building against a retired strategy unless this is scrubbed.
10. Gap D: Outlook enterprise parity is under-specified in shared mailbox, transport rule, and hybrid/on-prem behavior.
11. Evidence: product docs cover compliance and eDiscovery, but no OpenTofu, OS, or test evidence proves deployment.
12. Impact: enterprise mail buyers will compare this surface directly with Exchange Online and Microsoft Purview.
13. Gap E: Gmail anti-abuse parity is unproven.
14. Evidence: ADR-MAIL-0004 and benchmark docs have classifier ambition, but no runtime harness is present.
15. Impact: mail quality fails in practice if spam, phishing, and reputation handling are weak.
16. Gap F: Proton privacy parity is design-only.
17. Evidence: PRD and ADRs discuss E2EE and recovery, but no implementation or conformance tests prove operator non-access.
18. Impact: privacy claims become risky without cryptographic evidence.
19. Gap G: runbook integrity is incomplete.
20. Evidence: incident/failure docs reference runbook names not present in the inventory.
21. Impact: incident response under stress will lose time resolving paths.
22. Gap H: frontend direction is stale.
23. Evidence: `PRD.md:169-170` names Next.js and Tauri while canonical direction requires Leptos web SSR plus selective island hydration and allowed native stacks.
24. Impact: future UI implementation can drift into forbidden or non-canonical surfaces.
25. Gap I: SDK direction needs generated-governance wording.
26. Evidence: FAQ lists multiple SDK languages using stale capability ordering.
27. Impact: this can be valid only if generated from contracts and governed by the current developer SDK doctrine.
28. Gap J: counterpart matrix should distinguish product gaps from proof gaps.
29. Evidence: many features are present in PRD/ADR form but not in source/test/IaC form.
30. Impact: status dashboards should not mark document-level presence as production readiness.

## 7. Additive Surface for Oyatie

1. Oyatie can exceed Gmail, Outlook, and Proton if it combines enterprise-grade operations with Proton-grade trust and first-class self-hostable contexts.
2. The differentiator is not a lower-cost clone of any one counterpart.
3. The differentiator is a provider-agnostic mail substrate deployable across public cloud, guest cloud, on-prem, colo, and Oyatie provider contexts.
4. The differentiator is uniform feature quality across `demo_trial`, `paid`, and `revenue_share`.
5. The differentiator is context-aware economics rather than quality segmentation.
6. The differentiator is Cedar-governed dual-context isolation between personal and work mail.
7. The differentiator is cryptographic recovery with audit-chain proof and no casual operator plaintext path.
8. The differentiator is OpenTofu-controlled deployment evidence for DNS, MX, DKIM, MTA-STS, TLSRPT, queues, object storage, OpenBao, and observability.
9. The differentiator is support for customer-controlled infrastructure without losing Oyatie-grade operational standards.
10. The differentiator is Rust-strict backend and conformance tooling.
11. The differentiator is JMAP-first design with IMAP4rev2 compatibility, instead of being trapped in legacy mail UX.
12. The differentiator is explicit legal-hold and eDiscovery under encryption-aware custody.
13. The differentiator is compliance-pack behavior without feature-quality downgrade.
14. The differentiator is a clean revenue-share model for marketplace sellers, B2C operators, embedded SaaS resellers, and affiliate partners.
15. The additive path requires deleting stale capability ranks before implementation teams rely on them.
16. The additive path requires the missing context OpenTofu modules before deployment claims are made.
17. The additive path requires replacing shell-script acceptance criteria with Rust test binaries and Cargo-governed checks.
18. The additive path requires a `supported-oses.json` manifest.
19. The additive path requires a privacy conformance harness for encryption, recovery, and operator non-access claims.
20. The additive path requires a deliverability conformance harness for DKIM, SPF, DMARC, MTA-STS, TLSRPT, ARC, queue drain, bounce, and abuse feedback.
21. The additive path requires exact runbook names and incident links.
22. The additive path requires a tenant-class adoption PRD/ADR/manifest pass.
23. The additive path requires a benchmark harness that uses context and tenant-class overlays, not retired ranks.
24. The additive path requires feature parity evidence to be separated into designed, implemented, tested, deployed, and operated states.
25. Final feature-parity verdict: mail is a strong product plan with high counterpart ambition, but it is not yet counterpart-coherent until canonical substrate gaps and stale strategy language are resolved.

## 8. Evidence-State Ledger

1. Hosted mailbox creation: designed in `PRD.md:291`; implementation evidence absent; status is designed-only.
2. Custom domain onboarding: designed in `PRD.md:185` and `PRD.md:291`; OpenTofu DNS evidence absent; status is designed-only.
3. SMTP inbound receive: designed in `PRD.md:27-34`; Rust service absent; status is designed-only.
4. SMTP outbound send: designed in `PRD.md:952-968`; egress and reputation modules absent; status is designed-only.
5. IMAP4rev2 access: designed in `PRD.md:160-170`; conformance tests absent; status is designed-only.
6. JMAP access: planned through `IP-016-jmap-rfc-8620-frontend.md`; protocol test proof absent; status is planned.
7. REST API: specified in `contracts/openapi/mail.yaml:1`; handler proof absent; status is contract-only.
8. Event API: specified in `contracts/asyncapi/mail-events.yaml:1`; producer/consumer proof absent; status is contract-only.
9. Protobuf API: specified in `contracts/proto/mail.proto:5`; Rust service proof absent; status is contract-only.
10. Search: designed in PRD and cataloged through Tantivy adapter inventory; index implementation absent; status is designed-only.
11. Spam classification: governed by ADR-MAIL-0004; classifier implementation absent; status is decision-only.
12. Phishing defense: planned by `IP-017-anti-phishing-edge-wiring.md`; runtime proof absent; status is planned.
13. DKIM signing: governed by ADR-MAIL-001; DNS/OpenBao/OpenTofu proof absent; status is decision-only.
14. SPF/DMARC: designed in ADR-MAIL-001 and tutorial; test harness absent; status is decision-only.
15. MTA-STS/TLSRPT: referenced in PRD and FAQ; stale language needs scrub; status is designed-only.
16. ARC and forwarding: present in old capability docs; current replacement model absent; status is stale-doc-only.
17. Legal hold: planned by `IP-011-legal-hold-engine.md`; SLO exists; implementation proof absent; status is planned.
18. eDiscovery export: planned by `IP-012-ediscovery-export.md`; SLO exists; export proof absent; status is planned.
19. DLP quarantine: runbook exists; policy and implementation proof need coupling; status is partial-doc.
20. BYOK: named in `ARCHITECTURE.md:171`; tenant-class adoption absent; status is designed-only.
21. E2EE personal mail: targeted in `PRD.md:1003-1010`; cryptographic tests absent; status is designed-only.
22. Recovery: ADR-MAIL-0001 is substantive; stale tenant gating remains; status is decision-needs-rewrite.
23. Migration from Gmail: playbook exists; benchmark and rollback proof absent; status is documented.
24. Migration from Outlook: no explicit counterpart playbook found; status is gap.
25. Migration from Proton: no explicit counterpart playbook found; status is gap.
26. Backfill replay: document exists; harness proof absent; status is documented.
27. Incident response: document exists; runbook path integrity gap remains; status is partial-doc.
28. Failure modes: document exists; runbook path integrity gap remains; status is partial-doc.
29. Dashboards: JSON dashboards exist; live telemetry proof absent; status is artifact-only.
30. OpenTofu context coverage: absent; status is blocker.
31. OCI Always Free profile: absent; status is blocker.
32. OS manifest: absent; status is blocker.
33. Rust backend source: absent; status is blocker for implementation claims.
34. Rust tests: absent; status is blocker for verification claims.
35. Tenant-class model: absent; status is canonical-direction gap.
36. Retired rank language: present in many files; status is retirement backlog.
37. Shared mailbox: not sufficiently specified for Outlook parity; status is product gap.
38. Transport rules: not sufficiently specified for Outlook parity; status is product gap.
39. Proton-style external encrypted messages: not sufficiently specified; status is product gap.
40. Google-style search operators: not sufficiently specified; status is product gap.
41. Abuse feedback loop: not sufficiently specified; status is product gap.
42. Reputation recovery: referenced but runbook path not complete; status is operations gap.
43. IP warm-up: referenced in ADR-MAIL-0002; context IaC proof absent; status is operations gap.
44. Cross-tenant isolation: SLO exists; test proof absent; status is partial-doc.
45. Dual personal/work context: PRD and policies exist; policy path mismatch remains; status is partial-doc.
46. Developer SDKs: plan exists; generated-governance wording needs current policy; status is partial-doc.
47. Web client: stale framework wording remains; status is canonical-direction gap.
48. Native clients: allowed-language boundaries need clearer ownership; status is product/ownership gap.
49. Revenue-share mail economics: absent; status is tenant-class gap.
50. Demo trial mail limits: absent; status is tenant-class gap.
51. Paid class SLO mapping: absent; status is tenant-class gap.
52. Compliance-pack eligibility: documented in packs, not mapped to tenant class; status is partial-doc.
53. Provider-agnostic deployment: intended by master plan, not represented in mail IaC; status is blocker.
54. Feature parity conclusion: design coverage is broad, but proof coverage is not yet sufficient for counterpart-grade claims.
55. Evidence-state action: convert each designed-only surface into contract, source, test, IaC, runbook, and benchmark states before marking it complete.
56. Evidence-state action: do not treat broad journey IP coverage as proof that the core mail substrate works.
57. Evidence-state action: keep the counterpart union matrix as a living acceptance checklist after the canonical-direction cleanup.
58. Evidence-state action: move product gaps into implementation plans only after the substrate blockers are removed.
59. Evidence-state action: resolve stale strategy language before adding more feature surfaces.
60. Evidence-state action: keep Gmail, Outlook, and Proton Mail as the union bar for this microservice until a later directive changes the bar.
