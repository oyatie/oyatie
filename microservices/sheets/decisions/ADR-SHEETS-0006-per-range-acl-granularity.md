---
id: ADR-SHEETS-0006
title: Per-range ACL granularity — named-range ACL via Cedar policy fragments
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, ops-security, council-design-system, council-privacy
related: [ADR-0028, ADR-0135, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/sheets/PRD.md (FR-11, AC-04)
  - microservices/sheets/policy/tenant-scope.cedar
  - microservices/sheets/policy/editor-isolation.md
  - microservices/sheets/runbooks/share-acl-drift.md
purpose: Resolve PRD Open Question 6 — choose the granularity tier for ACL on workbook ranges (per-cell vs per-range named-ACL vs whole-sheet).
doc_status: published
---

# ADR-SHEETS-0006: Per-range ACL granularity — per-range named-ACL via Cedar policy fragments

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sheets workbooks frequently contain mixed-sensitivity data: a financial workbook may include PII customer columns next to aggregate metric columns; a clinical workbook may include PHI patient identifiers next to operational summary columns. Tenant operators need to share the workbook with broader audiences while restricting access to the sensitive columns.

Excel + Google Sheets offer "protected ranges" — a coarse mechanism. Airtable offers "field permissions" — per-column. The ACL granularity choice affects:
- Tenant authoring UX (how complex is the ACL configuration).
- Server-side evaluation cost (how many Cedar evaluations per cell read).
- Audit-chain emission volume.
- Drift detection cost.

Three granularity tiers were considered:

1. **Per-cell ACL**: every cell has its own ACL row.
2. **Per-range named-ACL**: tenant defines named ranges + per-range ACL.
3. **Whole-sheet ACL**: ACL is at sheet level only.

## Decision

Adopt **per-range named-ACL** via Cedar policy fragments.

### Model

- Tenant defines named ranges via the named-ranges BC.
- Tenant attaches ACL entries per named range: `{principal, decision: allow_read | allow_edit | deny}`.
- Cedar policy fragment synthesised from Postgres-stored ACL rows; deployed per-(tenant, workbook).
- Default-deny: ranges without explicit ACL are accessible only to workbook owner (per tenant-scope.cedar PERMIT 1+2).

### Cedar enforcement

`tenant-scope.cedar` PERMIT 9 + FORBID block (excerpted in `policy/editor-isolation.md`):

```cedar
permit (
  principal in TenantOperator::?p,
  action in [Action::"read_cell", Action::"read_range"],
  resource in CellOrRange::?cr
) when {
  resource has range_id &&
  principal has allowed_range_acls &&
  resource.range_id in principal.allowed_range_acls
};

forbid (
  principal in TenantOperator::?p,
  action in [Action::"read_cell", Action::"read_range", Action::"write_cell", Action::"write_range", Action::"write_formula"],
  resource in CellOrRange::?cr
) when {
  resource has range_id &&
  principal has allowed_range_acls &&
  !(resource.range_id in principal.allowed_range_acls)
};
```

### Server-side filter at render path

- Every cell-grid render call applies ACL filter BEFORE returning payload to client.
- Cells outside requestor's allowed ranges are masked (`#N/A` returned; tenant-side UI surfaces "Hidden by ACL").
- Client-side ACL is presentation-only; never load-bearing.

### Audit + drift detection

- Every ACL change emits `share_acl_changed` event with old/new ACL hashes (Ed25519-sealed).
- Quarterly drift audit compares Cedar fragments to Postgres-stored ACL rows; mismatch fires Sev-1 alert (FM-05).

### XLSX export ACL-aware masking

- XLSX export applies the per-range ACL of the requestor; masked cells are not present in the exported file (or are blanked depending on tenant policy).

## Alternatives Considered

### Alternative A — Per-cell ACL

- **Pros**: maximum granularity.
- **Cons**:
  - 1M-cell workbook = 1M ACL rows; Postgres + Cedar evaluator load explodes.
  - Authoring UX impossible at scale (tenant cannot click 1M cells to set ACL).
  - Cedar evaluation per cell read at 60fps cell-edit-render budget infeasible.
- **Rejected**: cost prohibitive at scale + UX intractable.

### Alternative B — Whole-sheet ACL

- **Pros**: trivially simple; one ACL per sheet.
- **Cons**:
  - Tenants cannot restrict subsets within a sheet.
  - Forces tenants to split mixed-sensitivity data into multiple sheets, breaking analytical workflows (cross-sheet formulas + pivots become awkward).
- **Rejected**: too coarse for tenant needs; matches the limited Excel + Google Sheets "protected ranges" mental model only partially.

### Alternative C — Hybrid (per-sheet ACL with per-cell exceptions)

- **Pros**: covers both coarse + fine-grained cases.
- **Cons**:
  - Two ACL systems to maintain.
  - Ambiguity when exceptions overlap.
- **Rejected**: complexity vs benefit unfavourable. Per-range named-ACL covers the same use cases more cleanly.

### Alternative D — RBAC-only (no per-resource ACL; only role-based)

- **Pros**: simpler.
- **Cons**: tenants need per-workbook fine-grained sharing; RBAC alone is too coarse.
- **Rejected**: doesn't match tenant ask.

## Consequences

### Architectural

- `oya-sheets-sharing-acl-*` BC owns per-range ACL storage + Cedar fragment generation.
- `oya-sheets-named-ranges-*` BC owns range definitions; named-ACL references named-range ids.
- Cedar fragments are deployed dynamically per-(tenant, workbook); evaluated server-side at every read/write.

### Downstream impact

1. **IP-010** authors sharing-acl + named-ranges full BCs.
2. **IP-013 (cell-grid app)** — render path applies ACL filter server-side.
3. **IP-009 (import-export)** — XLSX export applies ACL masking.
4. **IP-011 (AI-formula)** — AI cannot draft formulas referencing ranges outside requestor's ACL.
5. **runbooks/share-acl-drift.md** — handles ACL drift detection.

### CI lanes + SLOs

- `oya-governance-sheets-range-acl-cedar-required` — BLOCKER lane on dev; validates render-path ACL evaluation.
- `sheets.range_acl_drift_total` — Sev-1 alert on any non-zero count.
- `sheets.range_acl_violation_total` — informational; non-zero is normal (the gate is doing its job), but a spike indicates probing.

### Risk register

- **Risk**: Cedar fragment generator regressed; drift produces broader access. **Mitigation**: drift audit quarterly; Sev-1 page per FM-05.
- **Risk**: Tenant misconfigures ACL (grants broader access than intended). **Mitigation**: ACL UI preview before save; LEAN test corpus.
- **Risk**: XLSX export bypasses ACL masking. **Mitigation**: export pipeline always applies ACL; LEAN check.

## References

- PRD `microservices/sheets/PRD.md` FR-11 + AC-04.
- `microservices/sheets/policy/tenant-scope.cedar`.
- `microservices/sheets/policy/editor-isolation.md`.
- `microservices/sheets/runbooks/share-acl-drift.md`.
- `microservices/sheets/threat-model.md` T-T-07.
- Google Sheets protected ranges — `support.google.com/docs/answer/1218656`.
- Excel sheet protection — `support.microsoft.com/en-us/office/protect-a-worksheet`.
- Airtable field permissions — `support.airtable.com`.
- Cedar v4.2 LTS — `cedarpolicy.com`.
- ADR-0028 — audit-chain.
- ADR-0135 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
- ADR-0140 — Cedar policy enforcement.
