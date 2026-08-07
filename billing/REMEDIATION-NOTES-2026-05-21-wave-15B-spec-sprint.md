# REMEDIATION-NOTES — Wave 15B cloud-billing-spec-sprint (2026-05-21)

## Summary

cloud-billing had substantive Rust kernel (~1,491 lines across 3 crates: oya-cloud-billing-domain 1,030 lines + oya-cloud-billing-kernel 185 lines + oya-cloud-billing-tax-app 276 lines) but ZERO IP files in `implementation-plans/`. This is the canonical kernel-ahead-of-spec anti-pattern flagged in the Wave 4 audit + confirmed by Wave 15 IMPL-truth-up.

Wave 15B sprint authored 15 IPs (IP-001 through IP-015) documenting the actual implementation. Each IP references real files / tests / Cedar fragments / OpenAPI / AsyncAPI / proto3 / ADRs — no speculation.

## Wave 15B IP coverage

| IP | Title | Lines | Anchor existing code/contract |
|---|---|---|---|
| IP-001 | Domain layer bounded contexts | ~ 230 | oya-cloud-billing-domain (1,030 lines, 8 tests) |
| IP-002 | Kernel layer line item finalize | ~ 165 | oya-cloud-billing-kernel (185 lines, 6 tests) |
| IP-003 | Tax computation multi-jurisdiction | ~ 175 | TaxInvoiceFormat dispatch + tax-app crate |
| IP-004 | Composable billing_components | ~ 195 | ADR-0330 §B.11 + billing-components-gates.cedar |
| IP-005 | demo_trial tenant_class | ~ 190 | demo-trial-gates.cedar (174 lines) + conversion-gates.cedar |
| IP-006 | OpenAPI invoice endpoint | ~ 180 | contracts/openapi/cloud/cloud-billing-invoice-v1.yaml (544 lines) |
| IP-007 | AsyncAPI billing events | ~ 165 | contracts/asyncapi/cloud/cloud-billing-events-v1.yaml (50 lines) |
| IP-008 | gRPC service surface | ~ 215 | contracts/proto/cloud-billing.proto (700 lines, 11 services) |
| IP-009 | Cedar policy fragments | ~ 230 | 6 Cedar files (881 lines, 70+ gates) |
| IP-010 | Audit-chain emission | ~ 180 | AuditChainHeader proto + ADR-0263 |
| IP-011 | Revenue attribution + settlement | ~ 200 | settlement-gates.cedar (126 lines) + SettlementApi |
| IP-012 | Tenant onboarding billing flow | ~ 195 | composes IP-001/004/005/006/008/010 |
| IP-013 | DSR cascade retention erasure | ~ 175 | Classified<T> data classes + ADR-0244 + ADR-0251 |
| IP-014 | Cell-aware billing-data residency | ~ 175 | ADR-0248 cellular + ADR-0333 shuffle-sharding |
| IP-015 | Counterpart parity Stripe/Recurly/Zuora/Chargebee | ~ 195 | feature-parity-matrix-2026-05-20.md + 50-capability matrix |

Total IPs authored: 15.
Approximate total lines: ~ 2,865.
Counterpart references added: ~ 60+ direct comparisons across 5 counterparts (Stripe Billing, Recurly, Zuora Billing, Chargebee, AWS Billing & Cost Management) + supporting (FinOps Framework, OpenFGA, AWS IAM, Cedar, GCP, Azure, Snowflake, Databricks, etc.).

## Follow-ups identified during sprint

### Cedar gate gaps (not yet authored)

1. **DSR-specific gates** (IP-013): `cap.cloud.billing.dsr.read_subject_data`, `cap.cloud.billing.dsr.apply_tombstone`, `cap.cloud.billing.dsr.deny_during_retention`. Planned for IP-013-extension after privacy-portal µservice publishes its event schema.
2. **Cell-mesh gates** (IP-014): `cap.cloud.billing.cell.deny_cross_cell_write`, `cap.cloud.billing.cell.permit_aggregation_read`. Planned for IP-014-extension after cell-mesh policy fragment publishes.

### Endpoint surface gaps

3. **Missing REST endpoints**: `GET /v1/cloud/billing/invoices/{invoice_id}`, `POST /v1/cloud/billing/invoices/{invoice_id}/void`, `POST /v1/cloud/billing/credit-memos`. proto3 has these via InvoiceApi RPCs; OpenAPI surface only has POST generate. Planned IP-006-extension.
4. **`PreviewSubscription` RPC** (IP-012): preview-before-conversion API. Planned IP-012-extension.
5. **Tenant-facing CloudEvents webhook adapter** (IP-007): `/v1/cloud/billing/webhooks` self-registration. Owned by api-gateway µservice, not cloud-billing; cross-team coordination needed.

### Runbook gaps

6. **Cell-migration runbook** (IP-014): `microservices/cloud-billing/runbooks/cell-migration.md` not yet authored.
7. **UEL-buffer drain runbook** (IP-010): for the post-outage replay of buffered usage events.

### Test fixture gaps

8. **Cedar policy test fixtures** (IP-009): the six Cedar files lack `microservices/cloud-billing/policies/_tests/` per-gate fixtures. Planned in IP-009-test-fixtures.

### Data-class tombstone rule gaps

9. **Tombstone replacement table** (IP-013): currently only tax_registration_id has a defined tombstone; additional personal-data fields (subscription billing contact name, etc.) need explicit rules as they're added to schemas.

### Counterpart-parity capability gaps (substantive, prioritized)

10. **Hosted customer-portal UI parity** (IP-015 §C.2): finops-portal scope but capability list not yet aligned with Stripe/Recurly portals.
11. **Per-invoice email templating** (IP-015 §C.2): owned by finops-portal rendering; cross-team coordination.
12. **Built-in revenue forecasting / churn prediction** (IP-015 §C.2): finops-portal + ml-platform integration.

### Cross-µservice coordination follow-ups

13. **cloud-iam principal cache refresh latency** target verification: ADR-0255 §D-3 says 30 seconds; IP-005/IP-012 cite this — need integration test.
14. **audit-chain Seal RPC contract** stability: cloud-billing depends on this RPC; audit-chain µservice must publish stable gRPC contract.
15. **payments µservice integration shape** for InitiatePayout response: IP-011 references payment_handle return; payments must publish the contract.

## Files created in this sprint

```
microservices/cloud-billing/implementation-plans/IP-001-domain-layer-bounded-contexts.md
microservices/cloud-billing/implementation-plans/IP-002-kernel-layer-invoice-computation.md
microservices/cloud-billing/implementation-plans/IP-003-tax-computation-multi-jurisdiction.md
microservices/cloud-billing/implementation-plans/IP-004-composable-billing-components.md
microservices/cloud-billing/implementation-plans/IP-005-demo-trial-tenant-class.md
microservices/cloud-billing/implementation-plans/IP-006-openapi-invoice-endpoint.md
microservices/cloud-billing/implementation-plans/IP-007-asyncapi-billing-events.md
microservices/cloud-billing/implementation-plans/IP-008-grpc-service-surface.md
microservices/cloud-billing/implementation-plans/IP-009-cedar-policy-fragments.md
microservices/cloud-billing/implementation-plans/IP-010-audit-chain-emission.md
microservices/cloud-billing/implementation-plans/IP-011-revenue-attribution-cost-centers.md
microservices/cloud-billing/implementation-plans/IP-012-tenant-onboarding-billing-flow.md
microservices/cloud-billing/implementation-plans/IP-013-dsr-cascade-retention-erasure.md
microservices/cloud-billing/implementation-plans/IP-014-cell-aware-billing-data-residency.md
microservices/cloud-billing/implementation-plans/IP-015-counterpart-parity-stripe-recurly-zuora-chargebee.md
```

## Existing files referenced (not modified)

```
crates/oya-cloud-billing-domain/src/lib.rs (1,030 lines)
crates/oya-cloud-billing-kernel/src/lib.rs (185 lines)
crates/oya-cloud-billing-tax-app/src/lib.rs (276 lines)
microservices/cloud-billing/contracts/proto/cloud-billing.proto (700 lines)
contracts/openapi/cloud/cloud-billing-invoice-v1.yaml (544 lines)
contracts/asyncapi/cloud/cloud-billing-events-v1.yaml (50 lines)
microservices/cloud-billing/policies/cloud-billing.cedar (195 lines)
microservices/cloud-billing/policies/billing-components-gates.cedar (156 lines)
microservices/cloud-billing/policies/conversion-gates.cedar (142 lines)
microservices/cloud-billing/policies/demo-trial-gates.cedar (174 lines)
microservices/cloud-billing/policies/settlement-gates.cedar (126 lines)
microservices/cloud-billing/policies/tenant-class-binding.cedar (88 lines)
microservices/cloud-billing/PRD.md (referenced for outcomes + personas)
```

## Closure assertions

- [x] Each IP has substantive §A/§B/§C/§D/§E/§F/§G sections (Wave 15-IP-substance bar).
- [x] Each IP cites real files / tests / Cedar fragments / contracts — no hallucinated references.
- [x] Each IP cites relevant ADRs (ADR-0330 tenant_class + billing_components canonical, ADR-0244 tenant scoping, ADR-0243 cedar-as-universal-gate, ADR-0263 audit-chain seal, ADR-0145 direct gRPC, ADR-0248 cellular, etc.).
- [x] Each IP has counterpart parity section comparing to ≥4 of: Stripe Billing, Recurly, Zuora Billing, Chargebee, AWS Billing & Cost Management.
- [x] Each IP has §F evidence section pointing to actual files (paths with line numbers where applicable).
- [x] kernel-ahead-of-spec anti-pattern closed: 0 IPs → 15 IPs documenting actual implementation.
- [x] Follow-ups + Cedar gaps + endpoint gaps documented for future waves.
