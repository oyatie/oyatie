# payments remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/payments/onboarding/payments-engineer-first-week.md
- microservices/payments/benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): Values mirror manifest `dr`: RTO 900s, RPO 60s, `multi_region_active_active=true`, `dr_tier=T1`, `replication_shape=active-active-multi-az-cross-region-warm`, `failover_runbook=runbooks/psp-failover-cascade-execution.md`. Alternative considered: preserving the older 35m/15m companion-doc number. Rejected because manifest is the current authority. Cost: active-active finance runtime and PSP failover evidence are more expensive but satisfy stricter pack floors.
- Capacity model (ADR-0340): Values mirror manifest `capacity_model`: 0.2 CPU, 768 MiB RAM, 24 GiB storage, connections Valkey 8/Postgres 6/outbound HTTP 12, `scaling_dimension=per_request`, `cell_placement_class=Tier-0`, `pod_runtime_tier=1`, plus companion demand evidence of 648/s sustained, 6480/s flash, and 242 ms Charge::Create p99. Alternative considered: rely only on per-cell aggregate CRDB capacity. Rejected because payment risk is per-tenant PSP and idempotency throttling. Cost: token-bucket state, PSP rate-limit evidence, and edge/finance runtime headroom must be maintained per tenant.
- Sustainability and cost attribution (ADR-0344): Values require `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on charge, refund, payout, dispute, subscription, KYC/KYB, AML, and audit rows, with carbon routing disabled for real-time charge/fraud/PCI/AML paths; manifest `sustainability_emission_model` remains absent. Alternative considered: carbon-aware PSP routing for all calls. Rejected because authorization latency and PCI real-time fraud controls cannot defer. Cost: settlement and regulator bundles carry carbon metadata while hot payment paths only record emissions, and manifest emission fields must still be added.
- API versioning posture (ADR-0342): Values set public carrier triplet, SDK semver, last 3 versions for at least 180 days, paid/regulated tenant pinning, and ADR-0145 internal mesh exemption. Alternative considered: Stripe-style header-only versioning. Rejected because payments also exposes events and proto3 contracts to internal and marketplace consumers. Cost: PSP adapters, SDKs, and webhook fixtures need multi-version compatibility.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 88
- Trigger A additions: 36
- Trigger B additions: 88
- Trigger C additions: 58
- Trigger D additions: 9
- Root IPs unmatched: 0
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/payments/IP-001-payments-kernel-charge.md`: added API Versioning, DR posture.
- `microservices/payments/IP-002-payments-domain-charge.md`: added DR posture.
- `microservices/payments/IP-003-payments-usecase-charge.md`: added DR posture.
- `microservices/payments/IP-004-payments-adapter-stripe.md`: added DR posture.
- `microservices/payments/IP-005-payments-domain-refund.md`: added DR posture.
- `microservices/payments/IP-006-payments-usecase-refund.md`: added DR posture.
- `microservices/payments/IP-007-payments-domain-payout.md`: added DR posture.
- `microservices/payments/IP-008-payments-usecase-payout.md`: added DR posture.
- `microservices/payments/IP-009-payments-domain-dispute.md`: added DR posture.
- `microservices/payments/IP-010-payments-usecase-dispute.md`: added DR posture.
- `microservices/payments/IP-011-payments-domain-subscription.md`: added DR posture.
- `microservices/payments/IP-012-payments-usecase-subscription.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-013-payments-domain-sub-merchant.md`: added DR posture.
- `microservices/payments/IP-014-payments-usecase-sub-merchant.md`: added DR posture.
- `microservices/payments/IP-015-payments-rest-grpc-app.md`: added API Versioning, DR posture.
- `microservices/payments/IP-016-payments-settlement-domain.md`: added DR posture.
- `microservices/payments/IP-017-payments-settlement-worker.md`: added DR posture.
- `microservices/payments/IP-018-payments-adapter-adyen.md`: added DR posture.
- `microservices/payments/IP-journey-j07-stripe-subscription-estate-transfer.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j08-elder-transfer-cooloff.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j10-payment-mutation-freeze.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j100-pack-rollout-first-action.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j101-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j102-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j103-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j105-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j106-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j109-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j110-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j111-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j112-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j113-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j114-escrow-and-settlement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j115-escrow-and-settlement.md`: added DR posture, Sustainability emission, Pod runtime tier.
- `microservices/payments/IP-journey-j116-three-way-connect-settlement.md`: added API Versioning, DR posture, Pod runtime tier.
- `microservices/payments/IP-journey-j117-credit-memo-settlement.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j119-receivable-settlement-waterfall.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/payments/IP-journey-j120-per-currency-ledger-posting.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j121-repayment-cascade.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j122-mass-payout-and-withholding-ledger.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j123-split-settlement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j128-irs-direct-pay.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/payments/IP-journey-j133-severance-disbursement.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j134-stripe-connect-facilitator-placement-fee.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j136-payroll-deduction-and-per-period-execution.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j137-corporate-internal-audit-sox-controls-test-approval-chain-exporter.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j138-corporate-audit-vendor-payment-graph-reader.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j142-cross-tenant-severance-payable.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j145-cross-tenant-employer-pay-link.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j146-marketplace-settlement-and-fx.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j148-consumer-credit-and-supplier-settlement.md`: added API Versioning, DR posture, Pod runtime tier.
- `microservices/payments/IP-journey-j149-multi-platform-payout-ledger.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j150-minor-protected-revenue-waterfall.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/payments/IP-journey-j23-stripe-connect-payout.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j24-buyer-charge-escrow.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j36-stripe-connect-auto-pay.md`: added DR posture, Pod runtime tier.
- `microservices/payments/IP-journey-j37-payroll-ledger-hold.md`: added DR posture.
- `microservices/payments/IP-journey-j40-per-seat-billing.md`: added DR posture, Pod runtime tier.
- `microservices/payments/IP-journey-j47-hospital-bill-payment.md`: added DR posture.
- `microservices/payments/IP-journey-j48-kr-fss-threshold-ledger.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j50-helper-payroll-setup.md`: added DR posture.
- `microservices/payments/IP-journey-j51-vendor-payout-stripe-connect.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j52-buyer-charge-and-seller-settlement.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j53-recurring-invoice-state.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j54-first-payment.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j55-refund-or-chargeback.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j58-raise-effective-date.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j60-comp-change.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j62-copay-and-insurance.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j65-receipt-export.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j66-taxable-transaction-ledger.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j70-contract-payment.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j71-freeze-and-risk.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j73-publisher-payout.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/payments/IP-journey-j77-regulated-money-movement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j82-regulated-money-movement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j84-regulated-money-movement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j86-regulated-money-movement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j90-regulated-money-movement.md`: added API Versioning, DR posture.
- `microservices/payments/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j95-iso27001-soc2-annual-audit.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j96-ksa-uae-mena-onboarding.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j97-sg-pdpa-mas-tenant.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j98-au-privacy-apra-cps234.md`: added DR posture, Sustainability emission.
- `microservices/payments/IP-journey-j99-multi-pack-conflict-resolution.md`: added DR posture, Sustainability emission.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.2 vCPU, 768 MiB RAM, 24 GB storage, Valkey/Postgres/outbound connections 8/6/12, scaling_dimension=per_request, cell_placement_class=Tier-0.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.20 vCPU/768 MiB/24 GB covers PSP fan-out, KYC/KYB, settlement, and webhook queues.
- Rejected: Tier-1 cell placement was rejected because ADR-0340 names payments PCI core as Tier-0 foundation.
- Cost: Commits to highest isolation, larger pools, and active-active standby for revenue-critical money movement.

### Block 2: dr
- Value: RTO 900s, RPO 60s, active_active=true, backup_substrate=valkey_cluster, postgres_wal_g, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal, failover_runbook=runbooks/psp-failover-cascade-execution.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Fifteen-minute RTO and one-minute RPO are tighter than PCI floors because money movement and webhook delivery are revenue-critical.
- Rejected: PCI 24-hour floor was rejected as only a compliance minimum, not an operational recovery target.
- Cost: Requires active-active regions, OpenBao recovery, Valkey cluster backup, and audit-chain seals.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=1; evidence=microservices/payments/manifest.json, microservices/payments/ARCHITECTURE.md, microservices/payments/IP-016-payments-settlement-domain.md, microservices/payments/IP-journey-j77-regulated-money-movement.md, microservices/payments/slos/webhook-delivery-success.openslo.yaml.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: Payments does not execute tenant-customer code, but it owns regulated tenant money-movement, PSP secrets, KYC/KYB, payout, settlement, and webhook state.
- Rejected: Tier 2 was rejected because payments is tenant-data substrate for regulated financial flows.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-0.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Payments has public charge, payout, webhook, and proto surfaces that must honor per-tenant pinning.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-kata@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, on-prem/openbao-secret-binding@v1, aws-guest/edge-waf@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Kata nodepool and OpenBao secret binding are selected because Tier 1 pod isolation handles PSP secrets and PCI tenant data.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
