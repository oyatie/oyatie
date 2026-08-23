---
purpose: Oyatie — Tenant Migration Playbook (from competitor stacks)
doc_status: published
---

# Oyatie — Tenant Migration Playbook (from competitor stacks)

> **Owner:** `gtm-customer-success` + per-vertical team.
> **Companion:** [`templates/migration-runbook-template.md`](../templates/migration-runbook-template.md) (planned), per-vertical PRDs §3 + §6, [GTM-PLAN.md §6](../GTM-PLAN.md).

## 1. The migration play

Per [COMPETITIVE-GAP-ANALYSIS.md §9.7](../COMPETITIVE-GAP-ANALYSIS.md), tenant migration tooling is THE single biggest GTM-friction reducer. Per-competitor playbook + Foundry-driven importer + per-tenant validation report → target ≤ 24h tenant migration at preview / ≤ 6h at GA.

## 2. Per-source-stack playbook

| Source | Vertical | Surface(s) migrated | Estimated time per tenant |
|---|---|---|---|
| Google Workspace | Workspace | Mail / Calendar / Docs / Drive / Meet / Chat / Forms / Sites / Tasks | 6-24h depending on data volume |
| Microsoft 365 | Workspace | Mail / Calendar / Docs / Sheets / Slides / OneDrive / Teams / Forms / Lists / SharePoint | 6-24h |
| Naver Works | Workspace | Mail / Calendar / Drive / Talk / Forms | 6-12h (KR-specific) |
| Kakao Work | Workspace | Mail / Calendar / Drive / Chat | 6-12h (KR-specific) |
| Notion | SaaS / Workspace | Wiki / Docs / Tasks / Databases | 4-12h |
| Slack | Workspace | Chat (channels + DMs + threads + bots) | 4-8h |
| Zoom | Workspace (Meet) | Recordings + meeting history | 2-4h |
| Asana / Trello / Jira / Linear | SaaS / Workspace | Tasks / projects / boards | 4-12h |
| Salesforce | Vertical Corporate / SaaS | CRM (accounts / contacts / opps / activities) | 8-24h |
| HubSpot | Vertical Corporate / SaaS | CRM + Marketing | 8-24h |
| Workday | Vertical Corporate | HR + payroll + benefits | 24-72h (regulated; per-region) |
| 더존비즈온 (KR ERP) | Vertical Corporate | KR HR / payroll / GL / 전자세금계산서 | 24-72h KR-specific |
| 영림원 (KR ERP) | Vertical Corporate | KR HR / payroll / GL | 24-72h KR-specific |
| SAP | Vertical Corporate / Industrial | ERP / HR / GL / MES | 72h+ (per-module) |
| Epic / Cerner / KR EMR | Vertical Healthcare | Clinical record / scheduling / billing | 72h+ (regulated; HIPAA + KR MFDS) |
| Toss / KakaoPay / NaverPay | Vertical Fintech | Payment + KYC + transaction history | 24-72h (regulated; FSC + 신용정보법) |
| Adyen / Stripe / Braintree | Vertical Fintech | PG + tokenization | 24-72h (PCI-DSS migration) |
| Manhattan / Blue Yonder / Oracle WMS | Vertical Logistics | Shipment + EDI + warehouse | 72h+ |
| Procore / Autodesk Construction | Vertical Construction | Projects + RFI + submittal | 24-72h |
| Canvas / Blackboard / Google Classroom | Vertical Education | Courses + assignments + grades | 24-72h |

## 3. Per-migration phases

Per `templates/migration-runbook-template.md` (planned):

1. **Discovery** (1-3 days)
   - Inventory of source data + permissions + integrations
   - Per-class data-class assessment per ADR-0008
   - Per-region residency verification
   - Per-vertical regulator binding identified
2. **Pre-flight** (1-2 days)
   - Tenant onboarded into Oyatie per [`../../templates/checklists/tenant-onboarding.md`](../../templates/checklists/tenant-onboarding.md) (planned)
   - Per-cell allocation
   - Per-tenant DPIA completed per [`templates/dpia-template.md`](../templates/dpia-template.md)
   - Migration consent + DSR scope agreed
3. **Migration** (per-source per-vertical, see §2)
   - Source-side export (API / SCIM / OData / vendor SDK)
   - Per-record class annotation
   - Per-record encryption per Data Use Boundary
   - Per-record audit-chain emission
   - Per-batch rollback evidence
4. **Validation** (1-3 days)
   - Per-class post-migration record count + sample audit
   - Per-permission set verification
   - Per-tenant smoke test
   - Per-vertical regulatory pack onboarding evidence
5. **Cutover** (4-8 hours)
   - Source-side read-only mode
   - Final delta sync
   - DNS / SSO / SCIM cutover
   - Per-tenant smoke test (production)
   - Source-side decommission scheduled (post-30-day grace)
6. **Post-cutover** (30 days)
   - Per-tenant SLO baseline
   - Per-tenant DSR cascade test (synthetic)
   - Per-tenant audit-evidence pack regenerated
   - Per-tenant onboarding-experience review

## 4. Foundry-driven importer

Per [DESIGN §3](../DESIGN.md) Foundry-as-accelerator:
- `oyatie.migration.discover` capability inventories source
- `oyatie.migration.export` per-source connector
- `oyatie.migration.transform` per-class annotation + format-shift
- `oyatie.migration.import` per-Oyatie-axis ingest
- `oyatie.migration.validate` per-record audit + count
- `oyatie.migration.cutover` orchestration

## 5. Per-vertical migration regulatory considerations

- Healthcare: HIPAA Privacy Rule § 164.502 disclosure restrictions during migration; KR MFDS evidence; per-record consent verification
- Fintech: PCI-DSS scope migration (CDE handoff); KR FSC 24h notification if any deviation
- Public-sector: per-region procurement rules; 망분리 enforcement during migration
- Education-K12: CHILDREN_UNDER_14 hard-deny per-record; FERPA (US) / KR equivalent

## 6. Anti-patterns

- Migrate first, regulate later — never; regulatory pack binding is pre-flight requirement
- Skip per-record class annotation — never; downstream Data Use Boundary fails
- Single-shot migration without dry-run — never; per-cell phased
- Cutover without source-side read-only window — never; race condition risk

## 7. Sources
[COMPETITIVE-GAP-ANALYSIS.md §9.7](../COMPETITIVE-GAP-ANALYSIS.md), [GTM-PLAN.md §6](../GTM-PLAN.md), per-vertical PRDs §3 + §6, ADR-0008/0009/0010/0028/0029/0033, per-source vendor APIs.
