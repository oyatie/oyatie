---
ip_id: IP-013
microservice: cloud-billing
title: DSR cascade — billing data retention + erasure + audit-chain preservation
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0145, ADR-0330]
counterpart_parity: [Stripe data retention policies, Recurly GDPR erasure, AWS data residency, Chargebee GDPR tools]
capabilities_touched:
  - cap.cloud.billing.read_invoice
  - cap.cloud.billing.read_tenant_class
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-013 — DSR cascade: billing data retention + erasure

## §A Objective

Document cloud-billing's behavior in the cross-µservice Data Subject Rights (DSR) cascade per ADR-0244 tenant scoping + GDPR/CCPA/PIPL retention regimes. cloud-billing holds tenant-scoped financial records subject to regulatory retention floors (SOX-404 = 7 years, K-FSI = 5 years, GDPR Art. 17 = "without undue delay", PCI-DSS = transaction-life + 1 year). The DSR cascade must reconcile "right to erasure" with "regulatory retention."

The canonical rule per ADR-0251 compliance packs: financial records are retained per the active jurisdiction pack's floor; personal-data portions (e.g. names, addresses on invoices) are subject to per-field erasure rules. audit-chain entries are retained immutably; the question is what cloud-billing's database holds after erasure.

## §B Scope

In scope:

- Per-data-class retention rules: FINANCIAL (regulatory floor), INTERNAL_ONLY (tenant-controlled), PUBLIC (no retention floor).
- DSR types: access (Art. 15 GDPR), rectification (Art. 16), erasure (Art. 17), portability (Art. 20), restriction (Art. 18), objection (Art. 21).
- Retention conflict resolution: regulatory retention overrides erasure for FINANCIAL data; replacement-with-tombstone for personal-data fields on retained records.
- Cross-µservice cascade choreography (when cloud-billing receives a DSR request, how it propagates).
- Audit-chain preservation: seals are immutable; DSR does not delete audit-chain entries.

Out of scope:

- DSR intake UI (privacy-portal µservice).
- Cross-µservice DSR ledger (audit-chain µservice).
- Right-to-portability data format (data-export µservice).

## §C Architecture

### §C.1 Per-data-class retention rules

cloud-billing's data is tagged at the `Classified<T>` level (per IP-001 §C.3). DSR rules per class:

| Data class | Examples in cloud-billing | Regulatory floor | DSR erasure behavior |
|---|---|---|---|
| PUBLIC | region, schema_version | None | Erased immediately on DSR |
| INTERNAL_ONLY | identifiers, billing periods, currency, minor_units | None (commercial confidentiality only) | Erased immediately on DSR unless tied to FINANCIAL record |
| FINANCIAL | tax_registration_id, BillingAccount full state, Invoice full state | 7 years (SOX-404), 5 years (K-FSI), transaction-life + 1 year (PCI-DSS) | Personal-data fields replaced with tombstones; structural data retained; audit-chain entries immutable |

### §C.2 Tombstone semantics

When a personal-data field on a FINANCIAL record must be retained but the subject has invoked Art. 17 erasure, cloud-billing replaces the field with a deterministic tombstone:

- `tax_registration_id` → `taxid/electronic/0000000000` (well-formed but vacuous).
- Free-form description fields → empty string.
- Personal name fields (currently none in cloud-billing; finops-portal owns user-display name) → not applicable.

The audit-chain entry retains the original value (immutable); cloud-billing's queryable database retains the tombstone. This satisfies both:

- Auditor: can trace the original via audit-chain replay with appropriate role.
- Data subject: cloud-billing's live queries return tombstone, not the original PII.

### §C.3 Retention floor by deployment context

| Deployment context | Retention floor | Source authority |
|---|---|---|
| oyatie-public-cloud | 7 years (SOX-404 + state of incorporation) | US tax authority + SOX-404 |
| guest-on-aws | 7 years (SOX-404) | Customer's tax authority overrides |
| guest-on-oci | 7 years (SOX-404) | Same as above |
| on-prem (KR-CSAP, KR-K-FSI) | 5 years (K-FSI) | K-FSI minimum; CSAP may extend |
| colo | Per contract; default 7 years | Contract overrides |
| oyatie-as-cloud-provider | 7 years | Oyatie's own retention policy |

Retention floor is set at BillingAccount creation time via `regional_pack` and is immutable.

### §C.4 Cascade choreography

When a tenant's DSR request lands at privacy-portal µservice:

1. privacy-portal validates the request (subject identity, signed consent).
2. privacy-portal emits `dsr.requested.v1` event with `{tenant_id, subject_id, dsr_type, requested_at}`.
3. cloud-billing receives event; constructs the DSR plan per data-class:
   - For PUBLIC + INTERNAL_ONLY data tied to the subject (e.g. seat-counter records): erase.
   - For FINANCIAL data: tombstone personal fields, retain structural.
4. cloud-billing emits `cloud.billing.dsr.applied.v1` with `{plan_id, fields_erased, fields_tombstoned, retention_floor_blocked}`.
5. audit-chain seals both the DSR request and the DSR application.
6. privacy-portal aggregates DSR application events from all µservices and notifies the subject.

### §C.5 Right of access (DSR type 1)

For Art. 15 access requests:

- cloud-billing produces a tenant-scoped export of all FINANCIAL + INTERNAL_ONLY records.
- Export format: JSON or CSV (subject's choice) + PDF rendering of invoices.
- Tax_registration_id and amounts not redacted (subject is the data subject — has right to see).
- Audit-chain entries summarized (full seal hashes + event_class; payload available on request via audit-chain RPC).

### §C.6 Right of portability (DSR type 4)

For Art. 20 portability requests:

- Same export format as access, but in machine-readable structured format.
- Schema version stamped per record.
- proto3 binary encoding optional.

### §C.7 Right of restriction (DSR type 5)

For Art. 18 restriction requests (e.g. data is contested):

- cloud-billing marks tenant or specific records with `restriction.active = true` attribute.
- Cedar evaluator adds restriction check to write paths (out of scope of this IP's policy fragments; lives in a `dsr-restriction-gates.cedar` future fragment).
- Reads continue; writes denied until restriction lifted.

### §C.8 Tenant-deletion vs Subject-erasure

These are distinct operations:

- **Tenant deletion** (after 60-day demo_trial trial + retention): full tenant data erased per retention policy (FINANCIAL data tombstoned per §C.2; INTERNAL_ONLY data erased).
- **Subject erasure** (mid-life): only the subject's PII tombstoned; tenant continues.

## §D Lifecycle

### §D.1 DSR-triggered tombstone application

1. privacy-portal emits `dsr.requested.v1`.
2. cloud-billing-dsr-handler subscribes; constructs erasure plan.
3. For each FINANCIAL record: per-field rule applied (tombstone vs retain).
4. For each PUBLIC / INTERNAL_ONLY record not tied to financial: erased.
5. Plan executed atomically; cloud-billing emits `cloud.billing.dsr.applied.v1`.
6. audit-chain seal.

### §D.2 Tenant deletion (cascading)

1. tenancy emits `tenant.deletion.requested.v1` after 60-day demo_trial expiry + retention.
2. cloud-billing-deletion-handler subscribes; verifies retention floor exhausted.
3. If retention floor still active: deletion-handler emits `cloud.billing.deletion.blocked.v1 { retention_until }`; tenancy holds the deletion.
4. If retention floor exhausted: full erasure per data class.
5. audit-chain entries persist (Merkle root preserves their evidence).

### §D.3 Failure modes

- Retention floor blocks erasure → `cap.cloud.billing.dsr.deny_during_retention` (future Cedar gate, not yet authored in scope).
- audit-chain unreachable during DSR application → fail closed; retry queue.
- Cross-µservice DSR consensus failure (e.g. cloud-iam acknowledges but tenancy doesn't) → DSR plan in `partial` state; manual operator review.

## §E Cedar Policy Bindings

Existing fragments do not yet author DSR-specific gates — this is a Wave-15B gap noted in REMEDIATION-NOTES. Planned future gates (out of scope of this IP's authoring):

- `cap.cloud.billing.dsr.read_subject_data` — Art. 15 access.
- `cap.cloud.billing.dsr.apply_tombstone` — Art. 17 erasure with retention reconciliation.
- `cap.cloud.billing.dsr.deny_during_retention` — fails closed during retention floor.

For now, DSR application uses the existing `cap.cloud.billing.read_invoice` + cloud-billing-dsr-handler being in the operator group.

## §F Evidence

### §F.1 Source files

- `oya-cloud-billing-domain::Classified<T>` wrapping (per IP-001 §C.3) — the substrate that makes per-data-class retention enforceable.
- Cedar fragments: `cloud-billing.cedar` lines 173–195 (read_invoice, read_settlement_statement gates).

### §F.2 Data-class evidence in code

- `oya-cloud-billing-domain/src/lib.rs` lines 31–69 (every identifier struct's `// data_class:` comment).
- `oya-cloud-billing-domain/src/lib.rs` line 169 — tax_registration_id classified as Financial.
- `oya-cloud-billing-tax-app/src/lib.rs` line 27 — tax_registration_id REST API surface tagged `FINANCIAL_REGULATED_CREDIT`.

### §F.3 ADR anchors

- ADR-0244 tenant scoping primitive.
- ADR-0251 compliance packs (per-pack retention).
- ADR-0263 audit-chain immutability.
- ADR-0330 §B.13 DSR cascade.
- ADR-0145 direct gRPC.

### §F.4 REMEDIATION-NOTES

- DSR-specific Cedar gates not yet authored as a fragment — planned for follow-up IP-013-extension after privacy-portal µservice publishes its event schema.
- Tombstone-replacement rule for personal-data fields needs a per-field decision table; currently the only personal-data field is `tax_registration_id`. As cloud-billing adds personal-name fields (subscription billing contact), the table must be extended.

## §G Counterpart parity

| Counterpart | Their DSR approach | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe data retention | Stripe Data Retention API; delete data subject to "required" data retained | Tombstone-on-required-retention; cascade across µservices | Stripe handles single-account scope; oyatie cascades cross-µservice. |
| Stripe Data Pipeline | Bulk export for portability | Per-tenant export via data-export µservice | Same scope. |
| Recurly GDPR erasure | Per-account erasure with retention exception list | Per-data-class rule with tombstone | Direct parity. |
| AWS data residency | Region-scoped data with per-region erasure | Cell + region scoping (ADR-0248) + retention pack | Oyatie's cellular topology adds per-cell erasure. |
| Chargebee GDPR tools | "Forget customer" UI + retention rules | DSR cascade + privacy-portal | Same surface; oyatie's audit-chain stronger. |
| Salesforce Trust Site | Per-org data subject request workflow | DSR cascade with privacy-portal as orchestrator | Direct parity. |
| Google Takeout | Self-serve export for portability | Per-tenant export with proto3 + JSON + CSV | Oyatie supports machine-readable proto3 for B2B. |

## §H Open questions

- Whether retention floor varies per record (e.g. one invoice retained longer due to dispute) or only per-tenant. Current decision: per-record extensible — the retention floor is computed at access time from the regional_pack + any per-record holds (e.g. litigation hold).
- Whether tombstone should be deterministic (same input → same tombstone) or random per application. Current decision: deterministic — supports verification that DSR was applied without leaking original.
