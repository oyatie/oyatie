# production-planning remediation notes

## Wave 15J-final-cleanup
- Scope: F-BUCKET-2 ERP stamped PRD template cleanup.
- Replaced retired capability-tier field references with `tenant_class` and paid `billing_components`.
- Canonical field shape applied in PRD policy, FinOps, and activation sections:
  - tenant_class: TenantClass (enum: demo_trial, paid)
  - billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage})
- Renamed stale 2026-05-20 audit artifacts to 2026-05-21 and scrubbed retired B/S/G/P and `capability_tier` vocabulary.
- Scrubbed implementation-plan front matter and manifest field residue inside the assigned service path.
- Verification: assigned Wave 15J grep checks return zero non-remediation residue.
