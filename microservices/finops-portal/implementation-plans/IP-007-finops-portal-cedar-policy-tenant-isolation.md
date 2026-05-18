---
ip_id: IP-007
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/policy/cedar
related_adrs: [ADR-0162, ADR-0183, ADR-0199]
depends_on: [IP-005]
target_lines: 150
---

# IP-007 — Cedar policies for finops-portal authz

## Why this slice

ADR-0183 (cedar-policy-discipline) requires every µservice that
handles per-tenant data to author and ship Cedar policies at
`policy/cedar/`. For `finops-portal` the policy surface is
non-trivial because four distinct principal classes interact with
overlapping resources:

1. **Tenant admin** — sees only own tenant's invoices, drilldowns,
   credit ledger.
2. **ops-finops** — sees fleet-wide; can finalize invoices; can
   apply credits; can trigger regulator emit.
3. **customer-success** — can apply negotiated credits to a tenant;
   can view (not edit) tenant invoices.
4. **regulator** — read-only export of quarterly regulator evidence
   for ALL tenants; explicit allow.

Four policy files cover these surfaces, each scoped narrowly per
the dual-context discipline (request context + resource attributes).

## Acceptance criteria

1. Four files authored:
   - `policy/cedar/tenant-isolation.cedar` (60 lines).
   - `policy/cedar/customer-success-credit-application.cedar` (60 lines).
   - `policy/cedar/regulator-evidence-emit.cedar` (60 lines).
   - `policy/cedar/ops-finops-dashboard-access.cedar` (60 lines).
2. Each policy file declares:
   - The principal entity types it scopes (`TenantAdmin`,
     `OpsFinops`, `CustomerSuccess`, `Regulator`).
   - The action set.
   - The resource entity types (`Tenant`, `Invoice`,
     `CostAllocationPolicy`, `CreditApplication`,
     `RegulatorEvidence`).
   - One or more `permit` statements with explicit `when` clauses.
3. Dual-context fields appear on every `when`:
   - Request context: `context.regulatory_pack`,
     `context.residency_region`.
   - Resource attrs: `resource.tenant_id`, `resource.classification`.
4. Schema file `policy/cedar/schema.cedarschema.json` declares the
   entity + action types and is referenced by every policy file via
   header comment.
5. Unit tests (in the api crate from IP-005) cover ≥ 12 scenarios:
   - tenant-admin reads own invoice — PERMIT.
   - tenant-admin reads other tenant invoice — DENY.
   - tenant-admin tries to finalize — DENY.
   - ops-finops reads any invoice — PERMIT.
   - ops-finops finalizes invoice — PERMIT.
   - customer-success applies credit — PERMIT.
   - customer-success edits cost-allocation-policy — DENY.
   - regulator reads quarterly evidence — PERMIT (residency-aware).
   - regulator reads tenant invoice directly — DENY.
   - cross-pack regulator emit (EU regulator on KR data) — DENY.
   - dashboard access by tenant-admin — PERMIT scoped.
   - dashboard access by ops-finops — PERMIT fleet.
6. `cedar validate` is run in CI per policy file; CI lane
   `lean-a8-cedar-policy-validate` green.

## File-level work plan

1. `policy/cedar/schema.cedarschema.json` — entity/action schema.
2. `policy/cedar/tenant-isolation.cedar` — primary isolation policy.
3. `policy/cedar/customer-success-credit-application.cedar`.
4. `policy/cedar/regulator-evidence-emit.cedar`.
5. `policy/cedar/ops-finops-dashboard-access.cedar`.

## Residency-aware authz

Each policy's `when` clause checks
`principal.residency_region == resource.residency_region` for the
regulator/auditor principals. This implements the
multi-region-strategy.md constraint that an EU regulator never sees
KR tenant data even if granted regulator privilege at the global
level. Failure to match returns DENY.

## Dual-context shape (the field pattern)

```cedar
permit (
  principal in OpsFinops::"group",
  action in [Action::"FinalizeInvoice", Action::"ReadInvoice"],
  resource is Invoice
) when {
  context.regulatory_pack == resource.regulatory_pack &&
  context.residency_region == resource.residency_region &&
  principal.tenant_scope == "fleet"
};
```

The `context.regulatory_pack` field is supplied by the API layer's
auth middleware (derived from the JWT iss claim mapped through the
pack registry).

## Pack-specific overlays

- KR pack: regulator entity name is `KrFssRegulator`; PIPA
  evidence-emit additional allow.
- EU pack: regulator entity is `EuGdprDpa`; GDPR data-export
  permission additional allow.
- US-healthcare pack: PHI redaction enforced via a separate `deny`
  rule against principals lacking the `phi_authorized` attribute.

## Risk + mitigation

- **Risk**: policy drift between file and runtime. **Mitigation**:
  the api crate loads from `policy/cedar/*.cedar` at startup; CI
  reloads the same files in a unit test to detect mismatch.
- **Risk**: implicit deny is too coarse. **Mitigation**: emit a
  CEDAR_DECISION_DENY audit-chain event on every DENY so the
  ops-finops team has a forensic trail.

## Out-of-scope

- The Cedar runtime itself — shared crate `oya-cedar-runtime`.
- Per-tenant role administration UI — separate µservice.

## References

- ADR-0183 — cedar-policy-discipline.
- ADR-0162 — per-tenant audit-log slicing.
- `docs/standards/cedar-policy-authoring.md`.

## Verification

- `cedar validate --schema policy/cedar/schema.cedarschema.json
  --policies policy/cedar/<file>.cedar` per file.
- `oya gate cedar-policy-unit-tests --microservice finops-portal`.
