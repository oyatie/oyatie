---
doc_status: published
---

# Team: axis-workspace

## Mission
Own the Oyatie Workspace / Productivity Platform (Axis 2): Mail, Calendar, Docs, Sheets, Slides, Drive (Cloud Storage), Meet (video/audio/screen), Chat, Forms, Sites, Tasks, Notes, Translate, Recordings, Address Book. Ship a Google Workspace / Naver Works / Microsoft 365 / AWS Productivity (WorkMail / WorkDocs / Chime) / Kakao Work-class suite that is *natively* integrated with the SaaS platform, Foundry, Cloud, Search, Vertical-pack, and the regional-pack seams. Do NOT own per-vertical productivity (e.g. clinical-only mail) — that lives in the per-vertical team that depends on Workspace.

## Owned axes / surfaces / contracts
- Axis: `workspace` (axis 2 of 7)
- Surfaces: [`products/workspace/PRD.md`](../../products/workspace/PRD.md) — Mail, Calendar, Docs, Sheets, Slides, Drive, Meet, Chat, Forms, Sites, Tasks, Notes, Translate, Recordings, Address Book
- Cross-axis contracts: `WORKSPACE_TENANT_AND_IDENTITY` (consumed from SaaS); `WORKSPACE_KMS_SHRED` (Cloud); `WORKSPACE_SEARCH_BOUNDARY` (Search; consent-gated); `WORKSPACE_FOUNDRY_CAPABILITIES` (Foundry — compose/triage/schedule/transcribe/summarize); `WORKSPACE_REGIONAL_PACK_SEAMS` (per-region mail-security, holiday calendar, language pack, e-invoicing tax format)
- Catalog records: `crates/workspace-{mail,calendar,docs,sheets,slides,drive,meet,chat,forms,sites,tasks,notes,translate,recordings,address-book}-*`
- Runbooks: mail deliverability incident; doc CRDT divergence recovery; drive object integrity check; meet SFU failover; recording archiver
- ADRs: Workspace-axis ADR cluster needed (TBD): Mail server choice, CRDT engine choice (Yrs picked), Meet SFU choice (in-house picked), DLP engine, e-discovery policy

## In-scope work
- Mail server (SMTP / IMAP / JMAP) + spam / phishing / DLP / classifier
- Calendar (CalDAV) + smart scheduling + Foundry-driven scheduling agent
- Doc / Sheet / Slide collaborative editor (Yrs CRDT-based)
- Drive (object storage + folder/permission/sync)
- Meet (WebRTC SFU + recording + Foundry transcription + AI summary)
- Chat (DM + group + channels + threading + bots)
- Forms (data collection routed into Object Graph)
- Sites (lightweight intranet/wiki)
- Tasks + Notes + Keep
- Translate (50+ languages via Foundry adapter)
- Recordings archive
- Address Book (cross-tenant directory under consent)
- Migration tooling from Google Workspace / Microsoft 365 / Naver Works / Kakao Work / Notion / Slack

## Out-of-scope (anti-scope)
- Public consumer mail / calendar / doc (Gmail-class B2C) — never
- Free tier with ads — Workspace is paid-only
- Generic file-sync without enterprise controls — out of scope
- Game / streaming / consumer-social products
- Per-vertical specializations (e.g. clinical-mail rules) — those live with the vertical team

## Key dependencies on other teams

| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | Tenant kernel + Identity + RBAC/Cedar | continuous |
| `axis-foundry` | Capability invocation; provider adapters; transcription/summary capabilities | continuous |
| `axis-cloud` | KMS-shred (DEK per Drive object / Mail body); object storage; SFU placement; outbound IP reputation | continuous |
| `axis-search` | Per-tenant private index for Mail / Doc / Drive / Meet-transcript per consent | per release |
| `council-privacy` | Data Use Boundary class taxonomy + tenant-class overrides | quarterly |
| `regional-packs` | Per-pack mail-security, holiday calendar, language pack, e-invoicing format, identity provider | per pack |
| `ops-sre-reliability` | SLO targets; on-call rotation; mail-deliverability monitoring | continuous |
| `ops-security` | DLP rules; threat model per surface; phishing classifier review | quarterly |

## Teams that depend on us

| Consumer | What they need | Cadence |
|---|---|---|
| All vertical teams | Per-vertical Mail / Doc / Drive policy adoption | per vertical wave |
| `axis-saas` | Workspace plugin substrate; Workspace surfaces in marketplace | continuous |
| `axis-foundry` | Workspace-as-tool-surface for agent capabilities | continuous |
| `axis-search` | Index ingestion for Workspace data per consent | continuous |
| `gtm-customer-success` | Workspace migration playbook; tenant onboarding | per design partner |

## Success metrics

| Metric | Wave-Preview | Wave-Stable | Wave-GA |
|---|---|---|---|
| Mail deliverability | ≥ 99% | ≥ 99.5% | ≥ 99.9% |
| Doc edit-propagation p99 | < 200ms intra-region | < 100ms | < 80ms |
| Drive sync conflict rate | < 0.5% | < 0.1% | < 0.05% |
| Meet RTT p95 | < 250ms intra-region | < 200ms | < 150ms |
| DSR cascade SLA | 30d | 14d | 7d |
| Migration ingest from Google Workspace / M365 / Naver Works | < 24h per tenant | < 12h | < 6h |
| Cross-axis-contract violations | 0 | 0 | 0 |

## Escalation path
- Internal: tech lead → team manager
- Cross-team: `council-architecture` (for cross-axis contract changes)
- Privacy: `council-privacy` (for Data Use Boundary issues)
- Founder: as last resort

## Communication cadence
- Stand-up: daily; per-surface sub-team
- Weekly: Workspace-axis review with axis-saas + axis-foundry counterparts
- Monthly: cross-axis review with vertical teams
- Quarterly: Workspace-axis council review

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: 12-25 across surfaces (Mail / Calendar / Doc-family / Drive / Meet / Chat / supporting)
- Open requisitions: link to [HIRING-CAPACITY-PLAN.md](../../HIRING-CAPACITY-PLAN.md)

## Operating norms
- Per-surface sub-team owns its Cargo crate family
- All surfaces share the `workspace-shared-kernel` for `WorkspaceTenantBinding`, `PermissionSet`, `RetentionPolicy`
- Mandatory: every new surface ships with a DLP rule set + per-class allowlist before going to preview
- Pre-push: `oya verify` + `oya gate validate workspace`
- ADR proposal cadence: monthly batch

## Slice of risk register
Per-Workspace risks from RISK-REGISTER.md, plus surface-specific:
| Risk | Severity | Mitigation |
|---|---|---|
| Outbound mail IP reputation collapse | catastrophic | per-region warm pool; rate cap |
| Doc CRDT divergence | high | Yrs deterministic merge; replay test |
| Meet recording leak | catastrophic | KMS-shred; trust-portal access only |
| Workspace data leak via Foundry agent | catastrophic | per-capability data-class allowlist + Cedar gate |

## Sources scanned
- [products/workspace/PRD.md](../../products/workspace/PRD.md)
- [PRD.md §3.1 W-Workspace-Preview wave](../../PRD.md)
- [DESIGN.md §10 cross-axis contracts](../../DESIGN.md) — Workspace rows planned
- Codex verdict §2 axis coverage requirement
