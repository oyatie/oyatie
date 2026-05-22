# Payments Ownership-Coherence Audit — 2026-05-20

## Citation Anchor Block
1. Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4235` for §D-15 multi-context, §D-16 OpenTofu, §D-17 OS matrix, §D-18 Rust-strict, §D-19 OCI Always Free, and §D-20 audit decision tree.
2. Machine-readable master plan: `specs/master-plan-sequencing.json:704-868` for six deployment contexts, OpenTofu substrate, supported OSes, language policy, and OCI Always Free profile.
3. Product source read: `microservices/payments/PRD.md:1-1612`; the product summary, functional requirements, PSP matrix, budget, event surface, and acceptance criteria were audited.
4. Architecture source read: `microservices/payments/ARCHITECTURE.md:1-1720`; the architecture overview, bounded contexts, adapters, deployment shape, edge cases, and cross-service links were audited.
5. Documentation rigor source: `docs/standards/documentation-rigor.md:62-156` and `docs/standards/brief-template.md:666-1855` for intern-buildability, hyperscaler-grade substance, anchor citations, cross-context evidence, and anti-pattern rejection.

## Source Scope
- Target µservice: `microservices/payments/`.
- Counterparts confirmed by assignment and chat: Stripe, Adyen, Braintree.
- Chat history processed: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:552`, `:637`, `:686`, `:15739`.
- Constraint memory files read: provider-agnostic multi-context, OpenTofu-only, OS support matrix, Rust-strict-only, OCI Always Free maximization, ownership coherence, deliverable verification, and documentation substance.
- Inventory evidence: 202 files under `microservices/payments/`; 60,423 lines and 6,719,960 bytes counted with `find microservices/payments -type f -exec wc -l -c {} +`.
- Audit stance: this is a documentation and design coherence audit, not a code implementation or remediation pass.

## §1 µservice Purpose Summary
- Payments is meant to be Oyatie's regulated money-movement substrate, not a narrow checkout widget.
- The PRD defines a multi-PSP, multi-currency, marketplace-facilitator payment service at `microservices/payments/PRD.md:86-88`.
- The PRD says no Oyatie product calls Stripe, Adyen, Toss, or bank APIs directly; all product flows enter through payments at `microservices/payments/PRD.md:96-105`.
- The PRD surface includes charges, subscriptions, transfers, PSP routing, idempotency, retries, SCA, tax, ledger posting, audit, disputes, refunds, payouts, and marketplace settlement at `microservices/payments/PRD.md:119-125`.
- The architecture agrees on the high-level purpose: a Stripe Connect-style platform-facilitator service with multi-PSP routing at `microservices/payments/ARCHITECTURE.md:44-55`.
- The README repeats the substrate claim and names Stripe, Adyen, Toss, KakaoPay, LINE Pay, WeChat Pay, and Alipay as PSP targets at `microservices/payments/README.md:20-33`.
- The product purpose therefore has three simultaneous responsibilities: PSP abstraction, regulated ledger authority, and cross-product settlement platform.
- The current artifacts are strongest on charge/refund/payout/dispute/subscription/sub-merchant/onboarding flows.
- The current artifacts are materially weaker on canonical deployment context support, OpenTofu state/signing, OS manifest coverage, OCI Always Free tiering, and Braintree union parity.
- The service has a real product thesis, but the deployability and canonical-direction artifacts do not yet match the thesis.
- The audit treats payments as in-scope for all six deployment contexts because the brief template explicitly lists payments among examples that require all six contexts at `docs/standards/brief-template.md:711-714`.
- The audit treats payments as Rust-backend-only because ADR-0328 makes Rust the backend/runtime/CLI/validation/codegen/scripting language at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3062`.
- The audit treats Terraform content as drift because ADR-0328 says the canonical engine is OpenTofu and the CLI spelling is `tofu`, not Terraform at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2249`.
- The audit treats the missing supported OS manifest as a release blocker because ADR-0328 requires `microservices/<name>/supported-oses.json` with Tier-1/Tier-2/out-of-scope fields at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2927`.
- The audit treats OCI Always Free demo_trial as a special contract because ADR-0328 defines `guest-on-oci` Always Free as DemoTrial for evaluation/sandbox/dev and requires `iac/oci-guest/always-free/` at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3491-3697`.
- The audit found no forbidden non-Rust source files by extension under payments, but found plan/proto references that need generated-SDK provenance and language-policy clarification.
- The audit found multiple internal contradictions that would lead independent implementers to build different products: 14 bounded contexts versus 7, Braintree parity versus no Braintree adapter contract, `/v1/transfers` versus `/v1/payouts`, and product-facing surface versus `INTERNAL_ONLY` architecture classification.
- The audit conclusion is partial product coherence with canonical deployment incoherence.
- The required remediation shape is not a prose-only patch; it needs machine-readable context manifests, OpenTofu context modules, OS support manifest, Rust build lanes, and contract/version reconciliation.

## §2 Inventory Snapshot
- Total files seen: 202.
- Total lines audited: 60,423.
- Total byte count audited: 6,719,960.
- Inventory completeness: complete recursive file listing under `microservices/payments/`.
- Inventory table size column uses line count from `wc -l`.

| File | Size | Role | Coherent with purpose? |
|---|---:|---|---|
| `ARCHITECTURE.md` | 1720 lines | architecture source | partial; strong product model but canonical deployment and bounded-context contradictions |
| `AUDIT-FINDINGS-2026-05-20.json` | 123 lines | machine audit artifact | partial; useful evidence but not enough for intern buildability |
| `CHANGELOG.md` | 49 lines | release/change notes | partial; thin relative to current artifact churn |
| `IP-001-payments-kernel-charge.md` | 86 lines | implementation plan | yes; charge kernel scope aligns |
| `IP-002-payments-domain-charge.md` | 84 lines | implementation plan | yes; charge domain aligns |
| `IP-003-payments-usecase-charge.md` | 76 lines | implementation plan | yes; charge use case aligns |
| `IP-004-payments-adapter-stripe.md` | 77 lines | implementation plan | yes; Stripe adapter aligns |
| `IP-005-payments-domain-refund.md` | 58 lines | implementation plan | yes; refund domain aligns |
| `IP-006-payments-usecase-refund.md` | 43 lines | implementation plan | yes; refund use case aligns |
| `IP-007-payments-domain-payout.md` | 48 lines | implementation plan | yes; payout domain aligns |
| `IP-008-payments-usecase-payout.md` | 45 lines | implementation plan | yes; payout use case aligns |
| `IP-009-payments-domain-dispute.md` | 56 lines | implementation plan | yes; dispute domain aligns |
| `IP-010-payments-usecase-dispute.md` | 45 lines | implementation plan | yes; dispute use case aligns |
| `IP-011-payments-domain-subscription.md` | 49 lines | implementation plan | yes; subscription domain aligns |
| `IP-012-payments-usecase-subscription.md` | 46 lines | implementation plan | yes; subscription use case aligns |
| `IP-013-payments-domain-sub-merchant.md` | 50 lines | implementation plan | yes; sub-merchant domain aligns |
| `IP-014-payments-usecase-sub-merchant.md` | 44 lines | implementation plan | yes; sub-merchant use case aligns |
| `IP-015-payments-rest-grpc-app.md` | 74 lines | implementation plan | partial; app surface lacks canonical six-context bindings |
| `IP-016-payments-settlement-domain.md` | 47 lines | implementation plan | partial; omitted from manifest plan list |
| `IP-017-payments-settlement-worker.md` | 46 lines | implementation plan | partial; omitted from manifest plan list |
| `IP-018-payments-adapter-adyen.md` | 56 lines | implementation plan | partial; omitted from manifest plan list |
| `IP-journey-j07-stripe-subscription-estate-transfer.md` | 804 lines | journey implementation plan | yes; deep journey content aligns with payments |
| `IP-journey-j08-elder-transfer-cooloff.md` | 802 lines | journey implementation plan | yes; elder transfer risk aligns |
| `IP-journey-j10-payment-mutation-freeze.md` | 802 lines | journey implementation plan | yes; freeze semantics align |
| `IP-journey-j100-pack-rollout-first-action.md` | 400 lines | journey implementation plan | partial; pack rollout is adjacent |
| `IP-journey-j101-escrow-and-settlement.md` | 865 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j102-escrow-and-settlement.md` | 862 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j103-escrow-and-settlement.md` | 862 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j105-escrow-and-settlement.md` | 863 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j106-escrow-and-settlement.md` | 860 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j109-escrow-and-settlement.md` | 862 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j110-escrow-and-settlement.md` | 861 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j111-escrow-and-settlement.md` | 861 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j112-escrow-and-settlement.md` | 862 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j113-escrow-and-settlement.md` | 862 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j114-escrow-and-settlement.md` | 861 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j115-escrow-and-settlement.md` | 863 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j116-three-way-connect-settlement.md` | 430 lines | journey implementation plan | yes; platform settlement aligns |
| `IP-journey-j117-credit-memo-settlement.md` | 430 lines | journey implementation plan | yes; settlement aligns |
| `IP-journey-j119-receivable-settlement-waterfall.md` | 430 lines | journey implementation plan | yes; receivables align |
| `IP-journey-j120-per-currency-ledger-posting.md` | 430 lines | journey implementation plan | yes; ledger aligns |
| `IP-journey-j121-repayment-cascade.md` | 430 lines | journey implementation plan | yes; repayment aligns |
| `IP-journey-j122-mass-payout-and-withholding-ledger.md` | 430 lines | journey implementation plan | yes; payout/withholding align |
| `IP-journey-j123-split-settlement.md` | 430 lines | journey implementation plan | yes; split settlement aligns |
| `IP-journey-j128-irs-direct-pay.md` | 425 lines | journey implementation plan | yes; direct pay aligns |
| `IP-journey-j133-severance-disbursement.md` | 425 lines | journey implementation plan | yes; disbursement aligns |
| `IP-journey-j134-stripe-connect-facilitator-placement-fee.md` | 425 lines | journey implementation plan | yes; Stripe Connect scenario aligns |
| `IP-journey-j136-payroll-deduction-and-per-period-execution.md` | 425 lines | journey implementation plan | partial; payroll dependency needs boundary clarity |
| `IP-journey-j137-corporate-internal-audit-sox-controls-test-approval-chain-exporter.md` | 425 lines | journey implementation plan | partial; corporate audit scenario is adjacent but plausible |
| `IP-journey-j138-corporate-audit-vendor-payment-graph-reader.md` | 425 lines | journey implementation plan | yes; vendor payment graph aligns |
| `IP-journey-j142-cross-tenant-severance-payable.md` | 425 lines | journey implementation plan | yes; cross-tenant payable aligns |
| `IP-journey-j145-cross-tenant-employer-pay-link.md` | 425 lines | journey implementation plan | yes; pay link aligns |
| `IP-journey-j146-marketplace-settlement-and-fx.md` | 425 lines | journey implementation plan | yes; marketplace FX aligns |
| `IP-journey-j148-consumer-credit-and-supplier-settlement.md` | 430 lines | journey implementation plan | yes; supplier settlement aligns |
| `IP-journey-j149-multi-platform-payout-ledger.md` | 430 lines | journey implementation plan | yes; payout ledger aligns |
| `IP-journey-j150-minor-protected-revenue-waterfall.md` | 430 lines | journey implementation plan | yes; protected revenue aligns |
| `IP-journey-j23-stripe-connect-payout.md` | 420 lines | journey implementation plan | yes; payout aligns |
| `IP-journey-j24-buyer-charge-escrow.md` | 420 lines | journey implementation plan | yes; escrow aligns |
| `IP-journey-j36-stripe-connect-auto-pay.md` | 420 lines | journey implementation plan | yes; auto-pay aligns |
| `IP-journey-j37-payroll-ledger-hold.md` | 420 lines | journey implementation plan | partial; payroll boundary needs cross-service resolution |
| `IP-journey-j40-per-seat-billing.md` | 420 lines | journey implementation plan | yes; billing aligns |
| `IP-journey-j47-hospital-bill-payment.md` | 420 lines | journey implementation plan | yes; bill payment aligns |
| `IP-journey-j48-kr-fss-threshold-ledger.md` | 420 lines | journey implementation plan | yes; regulated ledger aligns |
| `IP-journey-j50-helper-payroll-setup.md` | 420 lines | journey implementation plan | partial; payroll setup should be cross-handoff scoped |
| `IP-journey-j51-vendor-payout-stripe-connect.md` | 430 lines | journey implementation plan | yes; vendor payout aligns |
| `IP-journey-j52-buyer-charge-and-seller-settlement.md` | 430 lines | journey implementation plan | yes; seller settlement aligns |
| `IP-journey-j53-recurring-invoice-state.md` | 430 lines | journey implementation plan | yes; invoicing state aligns |
| `IP-journey-j54-first-payment.md` | 430 lines | journey implementation plan | yes; first payment aligns |
| `IP-journey-j55-refund-or-chargeback.md` | 430 lines | journey implementation plan | yes; refund/chargeback align |
| `IP-journey-j58-raise-effective-date.md` | 430 lines | journey implementation plan | partial; compensation event needs payroll ownership boundary |
| `IP-journey-j60-comp-change.md` | 430 lines | journey implementation plan | partial; compensation event needs payroll ownership boundary |
| `IP-journey-j62-copay-and-insurance.md` | 430 lines | journey implementation plan | yes; copay payment aligns |
| `IP-journey-j65-receipt-export.md` | 430 lines | journey implementation plan | yes; receipt export aligns |
| `IP-journey-j66-taxable-transaction-ledger.md` | 430 lines | journey implementation plan | yes; tax ledger aligns |
| `IP-journey-j70-contract-payment.md` | 430 lines | journey implementation plan | yes; contract payment aligns |
| `IP-journey-j71-freeze-and-risk.md` | 430 lines | journey implementation plan | yes; risk freeze aligns |
| `IP-journey-j73-publisher-payout.md` | 430 lines | journey implementation plan | yes; publisher payout aligns |
| `IP-journey-j77-regulated-money-movement.md` | 430 lines | journey implementation plan | yes; regulated money movement aligns |
| `IP-journey-j82-regulated-money-movement.md` | 430 lines | journey implementation plan | yes; regulated money movement aligns |
| `IP-journey-j84-regulated-money-movement.md` | 430 lines | journey implementation plan | yes; regulated money movement aligns |
| `IP-journey-j86-regulated-money-movement.md` | 430 lines | journey implementation plan | yes; regulated money movement aligns |
| `IP-journey-j90-regulated-money-movement.md` | 430 lines | journey implementation plan | yes; regulated money movement aligns |
| `IP-journey-j91-us-msb-mtl-overlay.md` | 400 lines | compliance journey | yes; MSB/MTL aligns |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | 400 lines | compliance journey | partial; privacy overlay requires compliance ownership boundary |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | 400 lines | compliance journey | partial; RBI overlay requires compliance boundary |
| `IP-journey-j94-sox404-public-company-controls.md` | 400 lines | compliance journey | partial; SOX overlay requires governance boundary |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | 400 lines | compliance journey | partial; audit overlay requires governance boundary |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | 400 lines | compliance journey | yes; payment onboarding aligns |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | 400 lines | compliance journey | yes; MAS payment overlay aligns |
| `IP-journey-j98-au-privacy-apra-cps234.md` | 400 lines | compliance journey | partial; APRA/privacy split needs boundary |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | 400 lines | compliance journey | partial; pack conflict belongs to orchestration too |
| `PHASE-01-PAYMENTS-MVP.md` | 201 lines | phase plan | yes; MVP scope aligns |
| `PRD.md` | 1612 lines | product requirements | partial; broad product scope but several unresolved contradictions |
| `README.md` | 108 lines | overview | partial; coherent summary but lacks canonical deployment context proof |
| `backfill-replay.md` | 161 lines | replay plan | yes; payments event replay aligns |
| `benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md` | 112 lines | benchmark notes | partial; no measured provenance and counterpart set drifts from Braintree |
| `capabilities/charge.yaml` | 101 lines | capability manifest | yes; charge capability aligns |
| `capabilities/dispute.yaml` | 64 lines | capability manifest | yes; dispute capability aligns |
| `capabilities/payout.yaml` | 93 lines | capability manifest | yes; payout capability aligns |
| `capabilities/refund.yaml` | 75 lines | capability manifest | yes; refund capability aligns |
| `capabilities/sub-merchant-onboarding.yaml` | 68 lines | capability manifest | yes; sub-merchant capability aligns |
| `capabilities/subscription-lifecycle.yaml` | 73 lines | capability manifest | yes; subscription capability aligns |
| `tenant_class adoption record` | 155 lines | tier model | partial; lacks OCI Always Free and has broken contract paths |
| `capacity-model.md` | 256 lines | capacity math | yes; one of the strongest buildability artifacts |
| `catalog/oya-payments-adapter-adyen.yaml` | 22 lines | catalog component | yes; Adyen adapter aligns |
| `catalog/oya-payments-adapter-stripe.yaml` | 21 lines | catalog component | yes; Stripe adapter aligns |
| `catalog/oya-payments-charge-app.yaml` | 24 lines | catalog component | yes; charge app aligns |
| `catalog/oya-payments-charge-domain.yaml` | 18 lines | catalog component | yes; charge domain aligns |
| `catalog/oya-payments-charge-kernel.yaml` | 18 lines | catalog component | yes; charge kernel aligns |
| `catalog/oya-payments-charge-rest.yaml` | 19 lines | catalog component | yes; REST component aligns |
| `catalog/oya-payments-charge-usecase.yaml` | 21 lines | catalog component | yes; usecase aligns |
| `catalog/oya-payments-dispute-domain.yaml` | 19 lines | catalog component | yes; dispute component aligns |
| `catalog/oya-payments-kyc-kyb-domain.yaml` | 19 lines | catalog component | yes; KYC/KYB component aligns |
| `catalog/oya-payments-payout-domain.yaml` | 18 lines | catalog component | yes; payout component aligns |
| `catalog/oya-payments-refund-domain.yaml` | 18 lines | catalog component | yes; refund component aligns |
| `catalog/oya-payments-settlement-domain.yaml` | 18 lines | catalog component | yes; settlement component aligns |
| `catalog/oya-payments-subscription-domain.yaml` | 18 lines | catalog component | yes; subscription component aligns |
| `competitor-parity-matrix.md` | 141 lines | counterpart matrix | partial; compares Checkout/Toss/PayPal/Square/Coinbase more than Braintree |
| `compliance.md` | 1770 lines | compliance design | partial; has useful scope but repeats content-pass scaffold blocks |
| `contracts/asyncapi-v1.yaml` | 242 lines | event contract | yes; event surface aligns |
| `contracts/metric-naming-convention.md` | 78 lines | metrics standard | yes; observability aligns |
| `contracts/openapi-v1.yaml` | 639 lines | HTTP API contract | partial; omits Braintree PSP and uses payouts while PRD names transfers |
| `contracts/payments-v1.proto` | 312 lines | protobuf contract | partial; Go/Java options require generated-SDK provenance |
| `contracts/psp-adapter-trait.md` | 209 lines | adapter contract | yes; adapter abstraction aligns |
| `cost-budget.md` | 179 lines | cost model | partial; good fees but no OCI Always Free reconciliation |
| `cross-microservice-handoffs.md` | 271 lines | handoff map | partial; strong handoffs but broken `policies/` reference |
| `dashboards/dispute-volume.json` | 88 lines | dashboard | yes; dispute observability aligns |
| `dashboards/finops-cost-attribution.md` | 75 lines | dashboard guidance | yes; cost attribution aligns |
| `dashboards/fraud-signals.md` | 70 lines | dashboard guidance | yes; fraud observability aligns |
| `dashboards/payments-overview.json` | 109 lines | dashboard | yes; overview aligns |
| `dashboards/payout-latency.json` | 85 lines | dashboard | yes; payout latency aligns |
| `dashboards/psp-routing.json` | 83 lines | dashboard | yes; PSP routing aligns |
| `dashboards/settlement-reconciliation.json` | 99 lines | dashboard | yes; settlement aligns |
| `dashboards/subscription-health.json` | 143 lines | dashboard | yes; subscription aligns |
| `decisions/ADR-PAY-001-multi-psp-routing-with-failover-cascade.md` | 244 lines | service ADR | yes; multi-PSP decision aligns |
| `dpia.md` | 195 lines | privacy impact | yes; regulated payments data aligns |
| `failure-modes.md` | 264 lines | failure analysis | yes; strong failure semantics |
| `faqs/payments-engineer-faq.md` | 152 lines | onboarding FAQ | yes; useful engineer guidance |
| `iac/ech-config.yaml` | 51 lines | edge config | partial; references Terraform materialization |
| `iac/edge-waf.yaml` | 146 lines | WAF policy | partial; references Terraform materialization and direct vendor WAFs |
| `iac/helm/payments-app/Chart.yaml` | 23 lines | Helm chart | partial; Helm is deployment packaging but not canonical context IaC |
| `iac/helm/payments-app/values.yaml` | 166 lines | Helm values | partial; Kubernetes packaging, no six-context state |
| `iac/helm/payments-webhook-handler/Chart.yaml` | 24 lines | Helm chart | partial; no six-context state |
| `iac/helm/payments-webhook-handler/values.yaml` | 79 lines | Helm values | partial; no six-context state |
| `iac/kustomize/base/kustomization.yaml` | 32 lines | Kustomize base | partial; not OpenTofu context root |
| `iac/kustomize/base/namespace.yaml` | 17 lines | Kubernetes namespace | partial; no canonical context root |
| `iac/kustomize/base/openbao-secret-references.yaml` | 34 lines | secret refs | partial; secret refs only |
| `iac/network-policy/payments-network-policy.yaml` | 161 lines | network policy | yes for K8s; partial for six-context IaC |
| `iac/openbao/payments-policy.hcl` | 78 lines | OpenBao policy | yes; secret governance aligns |
| `iac/pqc-cert.yaml` | 104 lines | certificate policy | partial; references Terraform edge WAF path |
| `iac/terraform/payments-crdb.tf` | 108 lines | Terraform IaC | no; violates OpenTofu-only canonical direction |
| `iac/terraform/payments-edge-waf.tf` | 125 lines | Terraform IaC | no; violates OpenTofu-only canonical direction |
| `iac/terraform/payments-secret-bindings.tf` | 110 lines | Terraform IaC | no; violates OpenTofu-only canonical direction |
| `iac/tls/payments-ech-config.yaml` | 63 lines | TLS/ECH config | partial; useful but not context-root IaC |
| `incident-response.md` | 197 lines | incident process | yes; regulated response aligns |
| `manifest.json` | 378 lines | service manifest | partial; omits deployment contexts and has malformed refs |
| `migration-playbooks/from-adyen.md` | 584 lines | migration guide | yes; Adyen migration aligns |
| `migration-playbooks/from-braintree.md` | 584 lines | migration guide | yes; Braintree migration aligns but contracts lack Braintree adapter |
| `migration-playbooks/from-checkout-com.md` | 584 lines | migration guide | partial; useful extra counterpart but outside top-3 bar |
| `migration-playbooks/from-stripe.md` | 211 lines | migration guide | yes; Stripe migration aligns |
| `multi-region.md` | 210 lines | regional design | partial; useful but not tied to six contexts |
| `onboarding/payments-engineer-first-week.md` | 239 lines | onboarding guide | yes; intern buildability support |
| `policy/abuse-defence.cedar` | 189 lines | policy | yes; authorized Cedar and payments risk aligned |
| `policy/auditor-scope.cedar` | 185 lines | policy | yes; authorized Cedar and audit aligned |
| `policy/charge-authorization.cedar` | 204 lines | policy | yes; authorized Cedar and charge aligned |
| `policy/ci-scope.cedar` | 124 lines | policy | yes; authorized Cedar and CI scope aligned |
| `policy/data-residency.md` | 154 lines | policy design | yes; residency aligns |
| `policy/dispute-authorization.cedar` | 112 lines | policy | yes; authorized Cedar and dispute aligned |
| `policy/emergency-services-bypass.cedar` | 113 lines | policy | partial; payments emergency bypass needs governance proof |
| `policy/payout-authorization.cedar` | 199 lines | policy | yes; authorized Cedar and payout aligned |
| `policy/refund-authorization.cedar` | 175 lines | policy | yes; authorized Cedar and refund aligned |
| `policy/sub-merchant-onboarding.cedar` | 161 lines | policy | yes; authorized Cedar and onboarding aligned |
| `reference-implementations/charge-and-refund-rust-sdk.md` | 248 lines | Rust reference | yes; language policy aligned |
| `runbooks/aml-suspicious-activity-detected.md` | 282 lines | runbook | yes; AML aligns |
| `runbooks/chargeback-cascade-investigation.md` | 282 lines | runbook | yes; chargeback aligns |
| `runbooks/dispute-escalation.md` | 282 lines | runbook | yes; dispute aligns |
| `runbooks/double-charge-detected.md` | 282 lines | runbook | yes; double-charge aligns |
| `runbooks/elder-financial-abuse.md` | 282 lines | runbook | yes; abuse response aligns |
| `runbooks/fraud-spike-detected.md` | 282 lines | runbook | yes; fraud aligns |
| `runbooks/kr-fss-audit-pull.md` | 282 lines | runbook | yes; regulator pull aligns |
| `runbooks/kyc-aml-screening-pipeline-stall.md` | 282 lines | runbook | yes; KYC/AML aligns |
| `runbooks/payout-failed.md` | 282 lines | runbook | yes; payout failure aligns |
| `runbooks/pci-incident-response.md` | 282 lines | runbook | yes; PCI incident aligns |
| `runbooks/psp-failover-cascade-execution.md` | 282 lines | runbook | yes; PSP failover aligns |
| `runbooks/psp-outage.md` | 282 lines | runbook | yes; PSP outage aligns |
| `runbooks/refund-mismatch.md` | 282 lines | runbook | yes; refund mismatch aligns |
| `scorecards/overrides.json` | 150 lines | scorecard config | partial; governance artifact but needs canonical refs |
| `sdk-plan.md` | 213 lines | SDK plan | partial; TypeScript/Python SDK plan needs generated-output/exception provenance |
| `security/threat-model.md` | 424 lines | threat model | yes; payment security aligns |
| `slos/charge-api-availability.openslo.yaml` | 48 lines | SLO | yes; charge availability aligns |
| `slos/charge-api-latency.openslo.yaml` | 46 lines | SLO | partial; p99 500ms differs from PRD 200ms claim |
| `slos/dispute-response-latency.openslo.yaml` | 46 lines | SLO | yes; dispute response aligns |
| `slos/payout-api-latency.openslo.yaml` | 45 lines | SLO | yes; payout latency aligns |
| `slos/payout-completion-success.openslo.yaml` | 47 lines | SLO | yes; payout completion aligns |
| `slos/refund-api-availability.openslo.yaml` | 46 lines | SLO | yes; refund availability aligns |
| `slos/subscription-renewal-success.openslo.yaml` | 45 lines | SLO | yes; subscription renewal aligns |
| `slos/webhook-delivery-success.openslo.yaml` | 47 lines | SLO | yes; webhook delivery aligns |
| `test-plans/contract-test-strategy.md` | 362 lines | test plan | partial; mentions TypeScript stubs without generated provenance |
| `test-plans/integration-test-strategy.md` | 360 lines | test plan | yes; integration strategy aligns |
| `test-plans/unit-test-strategy.md` | 398 lines | test plan | yes; unit strategy aligns |
| `threat-model.md` | 396 lines | threat model | yes; payment security aligns |
| `tutorials/process-cross-currency-charge.md` | 233 lines | tutorial | yes; user-facing flow aligns |

## §3 9-Dimension Audit

### §3.1 Dimension 1 — Internal Coherence Within `microservices/payments/`
| Probe | Evidence | Classification | Severity |
|---|---|---|---|
| D1-01: PRD purpose and architecture purpose agree that payments is the PSP abstraction and platform-facilitator substrate. | `PRD.md:86-105`; `ARCHITECTURE.md:44-55` | resolves | P3-positive |
| D1-02: README repeats the same high-level purpose and consumer list. | `README.md:20-55`; `ARCHITECTURE.md:48-55` | resolves | P3-positive |
| D1-03: PRD says 14 bounded contexts and about 80 crates. | `PRD.md:1235-1256` | contradiction input | P1 |
| D1-04: Architecture lists 7 bounded contexts and at least 18 crates. | `ARCHITECTURE.md:57-103` | contradicts PRD scope | P1 |
| D1-05: README table also lists 7 bounded contexts. | `README.md:45-55` | aligns with architecture, contradicts PRD | P1 |
| D1-06: Manifest lists only 7 bounded contexts. | `manifest.json:14-80` | aligns with architecture, contradicts PRD | P1 |
| D1-07: PRD names Braintree in the PSP matrix. | `PRD.md:1381-1410` | contradiction input | P1 |
| D1-08: Tenant-class adoption notes PayPal Braintree in paid production PSPs. | `tenant_class adoption record` | contradiction input | P1 |
| D1-09: Manifest PSP adapters omit Braintree. | `manifest.json:94-102` | contradicts PRD/tenant_class adoption matrix | P1 |
| D1-10: OpenAPI PSP enum omits Braintree. | `contracts/openapi-v1.yaml:322-349` | contradicts PRD/tenant_class adoption matrix | P1 |
| D1-11: Proto PSP enum omits Braintree. | `contracts/payments-v1.proto:34-43` | contradicts PRD/tenant_class adoption matrix | P1 |
| D1-12: PRD says the API includes `/v1/transfers`. | `PRD.md:119-125` | contradiction input | P2 |
| D1-13: OpenAPI exposes `/v1/payouts`, not `/v1/transfers`. | `contracts/openapi-v1.yaml:204-242` | wrong-direction naming | P2 |
| D1-14: Capability tenant_class adoption matrix links `contracts/openapi/payments.yaml`. | `tenant_class adoption record:130-137` | broken reference | P2 |
| D1-15: Actual OpenAPI path is `contracts/openapi-v1.yaml`. | inventory | reference target mismatch | P2 |
| D1-16: Capability tenant_class adoption matrix links `contracts/asyncapi/payments-events.yaml`. | `tenant_class adoption record:130-137` | broken reference | P2 |
| D1-17: Actual AsyncAPI path is `contracts/asyncapi-v1.yaml`. | inventory | reference target mismatch | P2 |
| D1-18: Capability tenant_class adoption matrix links `contracts/proto/payments-control.proto`. | `tenant_class adoption record:130-137` | broken reference | P2 |
| D1-19: Actual proto path is `contracts/payments-v1.proto`. | inventory | reference target mismatch | P2 |
| D1-20: Architecture references `policy/schema.cedarschema`. | `ARCHITECTURE.md:149` | broken reference | P2 |
| D1-21: Actual policy directory has Cedar policies but no schema file. | inventory | missing target | P2 |
| D1-22: Cross-handoff says Cedar policies live under `policies/`. | `cross-microservice-handoffs.md:16` | broken reference | P2 |
| D1-23: Actual directory is `policy/`. | inventory | missing target | P2 |
| D1-24: PRD performance says p99 charge latency at or below 200ms. | `PRD.md:96-105`; `PRD.md:1556-1568` | contradiction input | P2 |
| D1-25: OpenSLO charge latency says p99 under 500ms. | `slos/charge-api-latency.openslo.yaml:1-46` | softer SLO than PRD claim | P2 |
| D1-26: Capacity model gives critical path p99 budget 242ms. | `capacity-model.md:127-138` | between PRD and SLO | P2 |
| D1-27: Architecture frontmatter classifies the doc as `INTERNAL_ONLY`. | `ARCHITECTURE.md:1-40` | contradiction input | P2 |
| D1-28: PRD and README define externally consumable product APIs and partner-facing PSP behavior. | `PRD.md:119-125`; `README.md:20-33` | product/public surface tension | P2 |
| D1-29: Manifest implementation plans list IP-001 through IP-015. | `manifest.json:184-199` | incomplete manifest | P2 |
| D1-30: Files IP-016, IP-017, and IP-018 exist and are in-scope. | inventory | manifest drift | P2 |
| D1-31: Manifest has malformed `hyperscaler_benchmark` refs. | `manifest.json:338` | broken internal reference | P3 |
| D1-32: Manifest repeats compliance/data packs with mixed case. | `manifest.json:312-366` | consistency issue | P3 |
| D1-33: Architecture has content-pass expansion blocks with dictionary-shaped leftovers. | `ARCHITECTURE.md:174-207`; `ARCHITECTURE.md:224-267` | documentation anti-pattern | P2 |
| D1-34: Compliance repeats content-pass expansion blocks. | `compliance.md:64-158` | documentation anti-pattern | P2 |
| D1-35: PRD cross-slice references include entries described as future additions. | `PRD.md:1367-1377` | unresolved reference shape | P2 |
| D1-36: Existing runbook filenames do not include several PRD-named incident docs exactly. | `PRD.md:1367-1377`; inventory | partial mismatch | P2 |
| D1-37: Charge capability, OpenAPI charge path, SLO charge files, and unit/integration plans align. | `capabilities/charge.yaml:1-101`; `contracts/openapi-v1.yaml:44-146`; `slos/charge-api-availability.openslo.yaml:1-48` | resolves | P3-positive |
| D1-38: Refund capability, OpenAPI refund path, and refund SLO align. | `capabilities/refund.yaml:1-75`; `contracts/openapi-v1.yaml:166-203`; `slos/refund-api-availability.openslo.yaml:1-46` | resolves | P3-positive |
| D1-39: Payout capability, OpenAPI payout path, and payout SLO align. | `capabilities/payout.yaml:1-93`; `contracts/openapi-v1.yaml:204-242`; `slos/payout-api-latency.openslo.yaml:1-45` | resolves | P3-positive |
| D1-40: Dispute capability, OpenAPI dispute path, and dispute SLO align. | `capabilities/dispute.yaml:1-64`; `contracts/openapi-v1.yaml:243-279`; `slos/dispute-response-latency.openslo.yaml:1-46` | resolves | P3-positive |
| D1-41: Subscription capability, OpenAPI subscription path, and subscription SLO align. | `capabilities/subscription-lifecycle.yaml:1-73`; `contracts/openapi-v1.yaml:280-300`; `slos/subscription-renewal-success.openslo.yaml:1-45` | resolves | P3-positive |
| D1-42: Sub-merchant capability and OpenAPI sub-merchant path align. | `capabilities/sub-merchant-onboarding.yaml:1-68`; `contracts/openapi-v1.yaml:301-321` | resolves | P3-positive |
| D1-43: Failure modes cover PSP outage, double-charge, payout failure, audit-chain break, Cedar drift, and cross-tenant payout misroute. | `failure-modes.md:27-194` | resolves | P3-positive |
| D1-44: Incident response covers severity triggers and regulator notification timing. | `incident-response.md:25-102` | resolves | P3-positive |
| D1-45: Cost budget covers Stripe and Adyen fees but not Braintree fees. | `cost-budget.md:24-50` | gap | P2 |
| D1-46: Braintree migration playbook exists but no Braintree adapter catalog exists. | `migration-playbooks/from-braintree.md:1-584`; inventory | contradictory readiness | P2 |
| D1-47: Competitor parity matrix includes PayPal and other providers but not the assigned Braintree union row as first-class top-3. | `competitor-parity-matrix.md:19-45` | counterpart drift | P3 |
| D1-48: Benchmark file title mentions Braintree but filename and body emphasize Checkout.com. | `benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md:1-112` | counterpart drift | P3 |
| D1-49: OpenAPI, AsyncAPI, and proto all carry authority metadata and versioning. | `contracts/openapi-v1.yaml:1-17`; `contracts/asyncapi-v1.yaml:1-16`; `contracts/payments-v1.proto:1-15` | resolves | P3-positive |
| D1-50: There is no source-code `src/` directory to sample; current state is design/spec-only. | inventory | implementation gap | P2 |
| D1-51: No internal contradiction found in runbook naming family for the 13 existing runbooks. | inventory | resolves | P3-positive |
| D1-52: The most severe internal coherence risk is not missing prose; it is divergent product contracts that would yield incompatible implementation branches. | D1-03 through D1-13 | aggregate | P1 |

### §3.2 Dimension 2 — Outbound Cross-References
| Reference | Evidence | Target check | Classification |
|---|---|---|---|
| D2-01: `docs/decisions/ADR-0328...` governs context/IaC/OS/Rust/OCI. | assignment; `ADR-0328:1730-4235` | exists | resolves |
| D2-02: `specs/master-plan-sequencing.json` governs machine-readable sequence. | assignment; `specs/master-plan-sequencing.json:704-868` | exists | resolves |
| D2-03: `docs/standards/brief-template.md` governs microservice audit form. | assignment; `docs/standards/brief-template.md:666-1855` | exists | resolves |
| D2-04: `docs/standards/documentation-rigor.md` governs intern buildability. | `docs/standards/documentation-rigor.md:62-156` | exists | resolves |
| D2-05: `ARCHITECTURE.md` links cloud-secrets. | `ARCHITECTURE.md:48-55`; `ARCHITECTURE.md:1571-1573` | `microservices/cloud-secrets/` exists | resolves |
| D2-06: `ARCHITECTURE.md` links cell. | `ARCHITECTURE.md:1571-1573` | `microservices/cell/` exists | resolves |
| D2-07: `ARCHITECTURE.md` links cloud-iac. | `ARCHITECTURE.md:1571-1573` | `microservices/cloud-iac/` exists | resolves |
| D2-08: Architecture depends on cloud-iam. | `ARCHITECTURE.md:48-55` | `microservices/cloud-iam/` exists | resolves |
| D2-09: Architecture depends on policy-engine. | `ARCHITECTURE.md:48-55` | no `microservices/policy-engine/` in inventory check | orphan/missing target |
| D2-10: Architecture depends on notifications. | `ARCHITECTURE.md:48-55` | no `microservices/notifications/` in inventory check | orphan/missing target |
| D2-11: Architecture names commerce-product-recommendation consumer. | `ARCHITECTURE.md:48-55` | no `microservices/commerce-product-recommendation/` in inventory check | orphan/missing target |
| D2-12: Manifest consumer `messenger`. | `manifest.json:286-295` | `microservices/messenger/` exists | resolves |
| D2-13: Manifest consumer `shorts`. | `manifest.json:286-295` | `microservices/shorts/` exists | resolves |
| D2-14: Manifest consumer `community`. | `manifest.json:286-295` | `microservices/community/` exists | resolves |
| D2-15: Manifest consumer `connect`. | `manifest.json:286-295` | `microservices/connect/` exists | resolves |
| D2-16: Manifest consumer `cloud-billing`. | `manifest.json:286-295` | `microservices/cloud-billing/` exists | resolves |
| D2-17: Manifest consumer `plugin-app-store`. | `manifest.json:286-295` | `microservices/plugin-app-store/` exists | resolves |
| D2-18: Manifest consumer `marketplace`. | `manifest.json:286-295` | `microservices/marketplace/` exists | resolves |
| D2-19: Manifest consumer `commerce-product-recommendation`. | `manifest.json:286-295` | missing target | orphan |
| D2-20: Cross-handoff depends on audit-chain. | `cross-microservice-handoffs.md:22-55` | `microservices/audit-chain/` exists | resolves |
| D2-21: Cross-handoff depends on api-gateway. | `cross-microservice-handoffs.md:22-55` | `microservices/api-gateway/` exists | resolves |
| D2-22: Cross-handoff depends on application. | `cross-microservice-handoffs.md:22-55` | `microservices/application/` exists | resolves |
| D2-23: Cross-handoff depends on developer-sdk. | `cross-microservice-handoffs.md:22-55` | `microservices/developer-sdk/` exists | resolves |
| D2-24: Cross-handoff depends on ops-dashboard. | `cross-microservice-handoffs.md:22-55` | likely `microservices/ops-dashboard-control-center/`; exact name mismatch | wrong-direction |
| D2-25: Cross-handoff depends on identity. | `cross-microservice-handoffs.md:22-55` | `microservices/identity/` exists | resolves |
| D2-26: Cross-handoff names compliance. | `cross-microservice-handoffs.md:22-55` | `microservices/compliance/` exists | resolves |
| D2-27: Events emitted to audit-chain. | `cross-microservice-handoffs.md:56-95`; `contracts/asyncapi-v1.yaml:25-151` | target exists | resolves |
| D2-28: Events consumed from identity/onboarding context are implied by KYC/KYB flows. | `ARCHITECTURE.md:76-103`; `cross-microservice-handoffs.md:56-95` | identity exists | resolves |
| D2-29: PRD references `docs/performance-budgets/payments-charge-budget.md`. | `PRD.md:1556-1561` | not under payments path; not verified in microservice inventory | external unresolved |
| D2-30: PRD references a webhook budget to be authored in M02. | `PRD.md:1556-1561` | unresolved work marker | weak section |
| D2-31: Existing payments references from docs were present in chat and canonical batch notes. | chat `:686`; chat `:15739` | confirms top-3 assignment | resolves |
| D2-32: Chat line `:686` places payments in Wave 3 Batch 3.1 with Stripe/Adyen/Braintree. | chat history | matches current assignment | resolves |
| D2-33: Chat line `:15739` repeats payments in active batch with Stripe/Adyen/Braintree. | chat history | matches current assignment | resolves |
| D2-34: Chat line `:552` states Stripe processes payments through Stripe, anchoring dogfooding doctrine. | chat history | relevant platform doctrine | resolves |
| D2-35: Chat line `:637` lists Stripe as a wedge and repeats oyatie-is-a-tenant principle. | chat history | relevant scope doctrine | resolves |
| D2-36: Memory ownership directive requires one agent own one microservice audit. | `feedback_microservice_ownership_coherence_2026_05_20.md` | followed | resolves |
| D2-37: Memory ownership directive says contradictions create different products. | same memory | supports severity model | resolves |
| D2-38: Memory verify directive rejects line-count-only completion. | `feedback_verify_deliverables_not_just_line_count_2026_05_20.md` | supports report substance | resolves |
| D2-39: Memory substance directive rejects thin scaffold. | `feedback_docs_substance_not_scaffold_2026_05_20.md` | supports anti-pattern findings | resolves |
| D2-40: Provider-agnostic memory says all six contexts must be first-class. | `feedback_multi_context_provider_agnostic_2026_05_20.md` | payments lacks fields | drift |
| D2-41: OpenTofu memory says no Terraform/Pulumi/CloudFormation/null_resource/local-exec/SSH. | `feedback_zero_handroll_opentofu_only_2026_05_20.md` | payments has Terraform files | drift |
| D2-42: OS memory says Tier-1 list must be explicit and exclusions explicit. | `feedback_os_support_matrix_2026_05_20.md` | payments has no manifest | drift |
| D2-43: Rust memory says backend and scripting must be Rust-only. | `feedback_rust_strict_only_no_python_2026_05_20.md` | no forbidden files found; SDK/proto caveat | partial |
| D2-44: OCI memory says OCI Always Free demo_trial equals Always Free. | `feedback_oci_always_free_maximization_2026_05_20.md` | tenant_class adoption matrix lacks this | drift |
| D2-45: Reverse references to payments from other docs were not fully enumerated outside the bounded chat/canonical/doc searches. | scope note | aggregation should run repo-wide cross-ref | open |
| D2-46: No payments source file under `src/` references other services directly because no source tree exists. | inventory | source-level check N/A | resolves |
| D2-47: `iac/edge-waf.yaml` references AWS and Cloudflare constructs directly. | `iac/edge-waf.yaml:1-146` | infra-level vendor surface | drift-fixable |
| D2-48: `iac/terraform/payments-crdb.tf` references GCP backend state. | `iac/terraform/payments-crdb.tf:1-26` | external state drift | P1 |
| D2-49: Existing outbound cross-references are numerous and mostly sensible, but several target names are stale or absent. | D2-05 through D2-28 | mixed | P2 |
| D2-50: Missing reverse references cannot be fully closed without Wave 14 aggregation across all microservices. | D2-45 | open orchestrator item | P3 |

### §3.3 Dimension 3 — Substance Bar: Intern-Buildability
| Buildability Question | Evidence | Assessment | Severity |
|---|---|---|---|
| D3-01: Can a cold intern state what payments does? | `PRD.md:86-105`; `README.md:20-33` | yes | P3-positive |
| D3-02: Can a cold intern identify the core APIs? | `contracts/openapi-v1.yaml:44-349` | yes, with transfer/payout naming caveat | P2 |
| D3-03: Can a cold intern identify event channels? | `contracts/asyncapi-v1.yaml:25-151` | yes | P3-positive |
| D3-04: Can a cold intern identify gRPC/proto messages? | `contracts/payments-v1.proto:1-312` | yes | P3-positive |
| D3-05: Can a cold intern identify PSP adapter shape? | `contracts/psp-adapter-trait.md:1-209`; `ARCHITECTURE.md:104-135` | yes | P3-positive |
| D3-06: Can a cold intern know which PSPs are required day one? | `PRD.md:1381-1410`; `manifest.json:94-102`; `contracts/openapi-v1.yaml:322-349` | no, Braintree conflict | P1 |
| D3-07: Can a cold intern know the bounded-context count? | `PRD.md:1235-1256`; `ARCHITECTURE.md:57-103` | no, 14 vs 7 contradiction | P1 |
| D3-08: Can a cold intern start implementation from plans? | IP-001 through IP-018 | mostly yes for core slices | P3-positive |
| D3-09: Can a cold intern trust manifest completeness? | `manifest.json:184-199`; inventory | no, IP-016..018 missing | P2 |
| D3-10: Can a cold intern provision the service in all six contexts? | `ADR-0328:1730-2239`; inventory under `iac/` | no, context dirs missing | P1 |
| D3-11: Can a cold intern use canonical IaC engine? | `iac/terraform/*.tf`; `ADR-0328:2241-2494` | no, Terraform drift | P1 |
| D3-12: Can a cold intern run canonical build command? | `ADR-0328:3288-3289`; payments no Rust workspace/src | no implementation build surface | P2 |
| D3-13: Can a cold intern determine Tier-1 OS support? | no `supported-oses.json`; `ADR-0328:2907-2927` | no | P1 |
| D3-14: Can a cold intern map package formats per OS? | `ADR-0328:2881-2905`; payments missing OS manifest | no | P1 |
| D3-15: Can a cold intern know OCI Always Free capacity? | `ADR-0328:3514-3577`; `tenant_class adoption record:13-39` | no, DemoTrial mismatch | P1 |
| D3-16: Can a cold intern know charge latency target? | `PRD.md:96-105`; `capacity-model.md:127-138`; `slos/charge-api-latency.openslo.yaml:1-46` | not precisely | P2 |
| D3-17: Can a cold intern know PSP outage behavior? | `failure-modes.md:27-39`; `runbooks/psp-outage.md:1-282` | yes | P3-positive |
| D3-18: Can a cold intern know double-charge behavior? | `failure-modes.md:40-51`; `runbooks/double-charge-detected.md:1-282` | yes | P3-positive |
| D3-19: Can a cold intern know payout failure behavior? | `failure-modes.md:53-64`; `runbooks/payout-failed.md:1-282` | yes | P3-positive |
| D3-20: Can a cold intern know regulator pull timing? | `incident-response.md:46-102`; `runbooks/kr-fss-audit-pull.md:1-282` | yes | P3-positive |
| D3-21: Can a cold intern know PCI scope? | `compliance.md:159-184`; `runbooks/pci-incident-response.md:1-282` | yes with scaffold caveat | P2 |
| D3-22: Can a cold intern know data retention? | `PRD.md:874-885`; `dpia.md:1-195` | mostly yes | P3-positive |
| D3-23: Can a cold intern know cost envelope? | `cost-budget.md:95-149` | yes for paid tiers, not OCI free | P2 |
| D3-24: Can a cold intern know per-tenant cost attribution? | `dashboards/finops-cost-attribution.md:1-75`; `cost-budget.md:95-149` | yes | P3-positive |
| D3-25: Can a cold intern know marketplace settlement model? | `PRD.md:1471-1499`; `ARCHITECTURE.md:754-765` | yes | P3-positive |
| D3-26: Can a cold intern know ledger invariants? | `PRD.md:1471-1499`; `failure-modes.md:157-194` | yes | P3-positive |
| D3-27: Can a cold intern know idempotency semantics? | `PRD.md:119-125`; `contracts/openapi-v1.yaml:44-79` | partially; needs explicit idempotency key rules | P2 |
| D3-28: Can a cold intern know refund semantics? | `contracts/openapi-v1.yaml:166-203`; `failure-modes.md:1-264` | yes | P3-positive |
| D3-29: Can a cold intern know dispute evidence rules? | `contracts/openapi-v1.yaml:243-279`; `runbooks/dispute-escalation.md:1-282` | yes | P3-positive |
| D3-30: Can a cold intern know subscription retries/dunning? | `contracts/openapi-v1.yaml:280-300`; `capabilities/subscription-lifecycle.yaml:1-73` | partial; Braintree/Stripe billing parity needs expansion | P2 |
| D3-31: Can a cold intern know SDK languages? | `sdk-plan.md:1-213`; `ADR-0328:3235-3286` | ambiguous because Python/TypeScript plan conflicts with strict policy without exception | P2 |
| D3-32: Can a cold intern know frontend constraints? | no `frontend/` under payments | N/A today | P3 |
| D3-33: Can a cold intern know CI lanes? | `test-plans/*.md`; missing OS manifest | partial; tests exist but not per Tier-1 OS | P1 |
| D3-34: Can a cold intern know source layout? | no `src/` | no implementation skeleton | P2 |
| D3-35: Can a cold intern know migration from Stripe? | `migration-playbooks/from-stripe.md:1-211` | yes | P3-positive |
| D3-36: Can a cold intern know migration from Adyen? | `migration-playbooks/from-adyen.md:1-584` | yes | P3-positive |
| D3-37: Can a cold intern know migration from Braintree? | `migration-playbooks/from-braintree.md:1-584` | yes, but adapter contract absent | P2 |
| D3-38: Can a cold intern know onboarding sequence? | `onboarding/payments-engineer-first-week.md:1-239` | yes | P3-positive |
| D3-39: Can a cold intern know tutorial happy path? | `tutorials/process-cross-currency-charge.md:1-233` | yes | P3-positive |
| D3-40: Can a cold intern know threat model? | `security/threat-model.md:1-424`; `threat-model.md:1-396` | yes, possible duplicate ownership | P3 |
| D3-41: Can a cold intern know dashboard expectations? | `dashboards/*.json`; `dashboards/*.md` | yes | P3-positive |
| D3-42: Can a cold intern know SLO alert names? | `slos/*.openslo.yaml`; `contracts/metric-naming-convention.md:1-78` | mostly yes | P3-positive |
| D3-43: Can a cold intern resolve all internal links? | D1 references | no | P2 |
| D3-44: Can a cold intern determine release maturity? | `README.md:90-96`; `PRD.md:1-84` | yes, proposed/MVP direction | P3 |
| D3-45: Can a cold intern determine authoritative service status? | `PRD.md:1-84`; `ARCHITECTURE.md:1-40` | partial; proposed vs accepted tension | P2 |
| D3-46: Can a cold intern determine who owns cross-service handoffs? | `cross-microservice-handoffs.md:192-263` | yes with missing service caveats | P2 |
| D3-47: Can a cold intern determine Braintree gap? | PRD/tier/migration vs contracts | yes after audit, not from one source | P1 |
| D3-48: Can a cold intern determine provider-agnostic deployment? | missing six-context manifest/IaC | no | P1 |
| D3-49: Can a cold intern build enough of the product from docs alone? | aggregate | partial, but not safely deployable | P1 |
| D3-50: Intern-buildability verdict. | docs rigor `docs/standards/documentation-rigor.md:133-156` | strong domain docs, failing canonical deployability | P1 |

### §3.4 Dimension 4 — Canonical-Direction Alignment
| Constraint | Required Source | Payments Evidence | Classification |
|---|---|---|---|
| D4-01: Six contexts must be considered for payments. | `ADR-0328:1730-2239`; `brief-template.md:711-714` | no manifest deployment context field | drifted-fixable |
| D4-02: `oyatie-public-cloud` must have context IaC. | `ADR-0328:1738-1750` | no `iac/oyatie-public-cloud/` | drifted-fixable |
| D4-03: `guest-on-aws` must have context IaC. | `ADR-0328:1785-1797` | no `iac/guest-on-aws/` | drifted-fixable |
| D4-04: `guest-on-oci` must have context IaC. | `ADR-0328:1834-1848` | no `iac/oci-guest/` | drifted-fixable |
| D4-05: `on-prem` must have context IaC. | `ADR-0328:1882-1895` | no `iac/on-prem/` | drifted-fixable |
| D4-06: `colo` must have context IaC. | `ADR-0328:1932-1945` | no `iac/colo/` | drifted-fixable |
| D4-07: `oyatie-as-cloud-provider` must have context IaC. | `ADR-0328:1981-1994` | no `iac/oyatie-iaas/` | drifted-fixable |
| D4-08: Context manifest should name supported contexts. | `ADR-0328:2079-2084` | `manifest.json` lacks deployment-context field | drifted-fixable |
| D4-09: OpenTofu is the canonical engine. | `ADR-0328:2241-2249`; `master-plan:747-776` | `iac/terraform/*.tf` and Terraform backend | incoherent |
| D4-10: Required context files are `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, `README.md`. | `ADR-0328:2296-2309` | absent for all six contexts | drifted-fixable |
| D4-11: No Terraform Cloud or hand-edited state. | `ADR-0328:2391-2397` | backend `gcs` under Terraform file | drifted-fixable/incoherent naming |
| D4-12: Cloud-IaC owns orchestration. | `ADR-0328:2405-2434` | no context modules registered to cloud-iac | drifted-fixable |
| D4-13: Forbidden null_resource/local-exec/remote-exec/SSH. | `ADR-0328:2464-2494` | no such patterns found in current evidence | aligned |
| D4-14: Supported OS manifest required. | `ADR-0328:2907-2927`; `master-plan:777-816` | no `supported-oses.json` | drifted-fixable |
| D4-15: Tier-1 OSes blocking. | `ADR-0328:2949-2959` | no per-OS CI lane manifest | drifted-fixable |
| D4-16: Out-of-scope OS exclusions explicit. | `ADR-0328:2838-2854` | no explicit exclusions | drifted-fixable |
| D4-17: Rust backend only. | `ADR-0328:3047-3062`; `master-plan:817-856` | no forbidden source extensions found | aligned |
| D4-18: Authorized file types include tf/yaml/json/proto/openslo/sql/md/cedar. | `ADR-0328:3085-3107` | current files are mostly authorized types | aligned with caveat |
| D4-19: Forbidden backend languages include Python/JS/TS/Ruby/Go/Java/Scala/Groovy/PHP/F#. | `ADR-0328:3235-3286` | no matching source files found | aligned |
| D4-20: Proto options for Go/Java are generated client metadata. | `contracts/payments-v1.proto:9-11` | allowed only if generation is controlled and not backend source | drifted-fixable |
| D4-21: SDK plan lists TypeScript and Python client SDKs. | `sdk-plan.md:20` | needs generated-SDK exception/provenance | drifted-fixable |
| D4-22: Contract test validates TypeScript stubs. | `test-plans/contract-test-strategy.md:332` | needs generated-output classification | drifted-fixable |
| D4-23: Canonical build invocation must be Cargo workspace release locked. | `ADR-0328:3288-3289` | no Rust workspace/source tree under payments | drifted-fixable |
| D4-24: OCI Always Free path required. | `ADR-0328:3666-3697`; `master-plan:857-868` | no `iac/oci-guest/always-free/` | drifted-fixable |
| D4-25: OCI Always Free demo_trial equals Always Free. | `ADR-0328:3500-3504` | tenant_class adoption matrix DemoTrial is 3 nodes/8 vCPU each and not Always Free | incoherent |
| D4-26: OCI Always Free compute 4 OCPU/24GB. | `ADR-0328:3514-3527` | tenant_class adoption matrix DemoTrial exceeds this envelope | incoherent |
| D4-27: OCI Always Free storage 200GB block plus 10GB object/archive. | `ADR-0328:3532-3549` | tenant_class adoption matrix DemoTrial DB/storage not reconciled | drifted-fixable |
| D4-28: OCI Always Free requires zero-cost alerting/billing events. | `ADR-0328:3708-3736` | cost docs do not include OCI zero-cost events | drifted-fixable |
| D4-29: Audit decision tree says missing context/IaC/OS/language findings are P1 for in-scope non-P0 services. | `ADR-0328:4106-4118` | applied here | aligned audit severity |
| D4-30: Documentation anti-pattern rejects scaffold content. | `brief-template.md:1727-1793` | architecture/compliance content-pass leftovers violate | drifted-fixable |
| D4-31: Documentation anti-pattern rejects contradiction. | `brief-template.md:1821-1843` | product scope contradictions exist | drifted-fixable |
| D4-32: Multi-context provider agnostic memory says no provider-first shortcut. | memory file | Terraform/GCP backend and vendor WAF references drift | drifted-fixable |
| D4-33: OpenTofu memory says no Terraform spelling. | memory file | `iac/terraform` directory drift | incoherent |
| D4-34: OS memory says macOS M5+ only, Intel/pre-M5 out of scope. | memory file | no explicit support/exclusion manifest | drifted-fixable |
| D4-35: Rust memory says no script generation of durable behavior. | memory file | audit used no content scripting; service has no forbidden files | aligned |
| D4-36: OCI memory says Always Free is not optional for DemoTrial OCI. | memory file | tenant_class adoption record missing | drifted-fixable |
| D4-37: Payments has Helm and Kustomize. | inventory `iac/helm`, `iac/kustomize` | useful deployment packaging but not a substitute for OpenTofu contexts | partial |
| D4-38: Payments has OpenBao policy. | `iac/openbao/payments-policy.hcl:1-78` | aligned with secret governance | aligned |
| D4-39: Payments has network policies. | `iac/network-policy/payments-network-policy.yaml:1-161` | aligned for K8s security, context-incomplete | partial |
| D4-40: Payments has TLS/ECH configs. | `iac/tls/payments-ech-config.yaml:1-63`; `iac/ech-config.yaml:1-51` | useful edge config, but Terraform refs drift | partial |
| D4-41: Payments has no `tests/` root but has `test-plans/`. | inventory | documentation-only test state | partial |
| D4-42: Payments has no `src/` root. | inventory | implementation absent | partial |
| D4-43: Payments current artifacts can define what to build but not how to deploy canonically. | aggregate D4 | drifted-fixable | P1 |
| D4-44: The most canonical-aligned areas are policies, contracts, runbooks, failure modes, and capacity math. | inventory and cited docs | aligned | P3-positive |
| D4-45: The least canonical-aligned areas are IaC engine, context roots, OS manifest, OCI Always Free demo_trial, and counterpart parity. | D4-01 through D4-28 | incoherent/drifted | P1 |
| D4-46: There is no evidence that any of the six deployment contexts are correctly N/A. | `ADR-0328:2116-2119`; `brief-template.md:711-714` | all should be in-scope | P1 |
| D4-47: The service should not claim hyperscaler maturity until D4-01 through D4-28 are remediated. | `documentation-rigor.md:143-156` | maturity gate | P1 |
| D4-48: The canonical action is to repair machine-readable surfaces first, not add prose-only appendices. | `master-plan:704-868`; memory directives | remediation direction | P2 |
| D4-49: Classification summary: multi-context drifted-fixable, OpenTofu incoherent, OS drifted-fixable, Rust mostly aligned, OCI drifted-fixable/incoherent. | aggregate | final D4 verdict | P1 |
| D4-50: Canonical-direction headline: payments is product-rich but substrate-incomplete. | aggregate | audit conclusion | P1 |

### §3.5 Dimension 5 — Industry-Counterpart Parity
| Capability | Stripe | Adyen | Braintree | Oyatie payments evidence | Gap |
|---|---|---|---|---|---|
| D5-01: Online card payments | Stripe docs; Adyen docs; Braintree docs | yes | yes | `contracts/openapi-v1.yaml:44-79` | present |
| D5-02: Hosted/prebuilt checkout | Stripe Checkout; Adyen Drop-in; Braintree Drop-in | yes | yes | no hosted checkout contract | gap |
| D5-03: Embedded payment elements/components | Stripe Elements; Adyen Web Components; Braintree Hosted Fields | yes | yes | no frontend component contract | gap |
| D5-04: Payment links/no-code collection | Stripe Payment Links | partial | partial | no pay-link API except journey docs | gap |
| D5-05: Payment Intents/session state | Stripe PaymentIntents; Adyen Sessions | Braintree transactions | `charge` state exists | partial |
| D5-06: Authorize/capture | Stripe; Adyen capture; Braintree submit for settlement | yes | `contracts/openapi-v1.yaml:122-146` | present |
| D5-07: Void/cancel | Stripe cancel; Adyen cancel; Braintree void | yes | `contracts/openapi-v1.yaml:147-165` | present |
| D5-08: Refunds | all three | yes | `contracts/openapi-v1.yaml:166-203` | present |
| D5-09: Partial refunds | all three | yes | refund contract needs amount semantics | partial |
| D5-10: Disputes/chargebacks | all three | yes | `contracts/openapi-v1.yaml:243-279`; runbooks | present |
| D5-11: Evidence upload | all three | yes | `contracts/openapi-v1.yaml:257-279` | present |
| D5-12: 3D Secure/SCA | all three | yes | PRD and capability docs mention SCA | partial |
| D5-13: Fraud/risk tools | Stripe Radar; Adyen Risk; Braintree fraud tools | yes | fraud dashboards/runbooks | partial |
| D5-14: Token vault | Stripe payment methods; Adyen tokenization; Braintree Vault | yes | no explicit vault API | gap |
| D5-15: Network tokenization/account updater | Stripe/Adyen strong; Braintree via card updater features | partial | no explicit network token contract | gap |
| D5-16: Subscriptions | Stripe Billing; Adyen recurring; Braintree recurring billing | yes | `contracts/openapi-v1.yaml:280-300` | partial |
| D5-17: Invoicing | Stripe Invoicing; Braintree recurring; Adyen less central | partial | no invoice API | gap |
| D5-18: Usage-based billing | Stripe Meter Events | partial | partial | no metering API | gap |
| D5-19: Tax calculation | Stripe Tax; Adyen tax partners; Braintree less central | partial | PRD tax/ledger only | partial |
| D5-20: Marketplace/platform onboarding | Stripe Connect; Adyen Platforms; Braintree Marketplace | yes | `contracts/openapi-v1.yaml:301-321` | partial |
| D5-21: Connected account capabilities | Stripe Connect capabilities; Adyen account holders | Braintree sub-merchants | no capability API | gap |
| D5-22: Split payments | Stripe Connect; Adyen split transactions; Braintree marketplace | yes | settlement journey docs | partial |
| D5-23: Payouts | Stripe payouts; Adyen payouts; Braintree disbursements | yes | `contracts/openapi-v1.yaml:204-242` | present |
| D5-24: Instant payouts | Stripe; Adyen instant card payouts | limited | no instant payout contract | gap |
| D5-25: Balance accounts | Stripe balances; Adyen balance accounts | Braintree merchant accounts | ledger docs | partial |
| D5-26: Multi-currency | all three | yes | PRD/capacity/migration docs | present |
| D5-27: Local payment methods | Stripe and Adyen broad; Braintree PayPal/Venmo/etc. | yes | PSP list includes local wallets but contract enum limited | partial |
| D5-28: In-person payments | Stripe Terminal; Adyen POS; Braintree in-store | yes | no terminal/POS contract | gap |
| D5-29: PayPal/Venmo acceptance | Braintree strong; Stripe/Adyen support regional alternatives | yes | no PayPal/Venmo enum | gap |
| D5-30: Bank debits/ACH/SEPA | all three | yes | not first-class in contract | gap |
| D5-31: Crypto/onramp | Stripe has crypto docs; others variable | partial | no crypto contract | acceptable gap unless strategy expands |
| D5-32: Reporting/export | all three | yes | dashboards and audit events | partial |
| D5-33: Webhooks | all three | yes | `contracts/openapi-v1.yaml:322-349`; AsyncAPI | present |
| D5-34: API idempotency | Stripe strong; Adyen/Braintree have request references | yes | PRD mentions; contract needs explicit header | partial |
| D5-35: API rate limiting guidance | Stripe official rates; others account-specific | yes | capacity model has PSP limits | partial |
| D5-36: Sandbox/test mode | all three | yes | DemoTrial sandbox tier | present |
| D5-37: Client/server SDK ecosystem | all three | yes | `sdk-plan.md:1-213` with language-policy caveat | partial |
| D5-38: Dashboard/admin tools | all three | yes | dashboards exist; no product UI contract | partial |
| D5-39: Compliance and PCI scope tooling | all three | yes | `compliance.md:159-184`; runbooks | present |
| D5-40: KYC/KYB onboarding | Stripe Connect and Adyen Platforms strong; Braintree Marketplace | yes | bounded context and OpenAPI sub-merchants | partial |
| D5-41: Sanctions/AML screening | platform counterparts support risk/compliance | partial | runbooks and compliance docs | partial |
| D5-42: Multi-region resilience | counterpart opaque; Oyatie explicit | `multi-region.md:1-210` | additive |
| D5-43: Tenant-isolated cells | counterparts abstract; Oyatie explicit | `ARCHITECTURE.md:1597-1657` | additive |
| D5-44: Cedar policy gates | not counterpart-native | Oyatie explicit | `policy/*.cedar` | additive |
| D5-45: Audit-chain Merkle evidence | not counterpart-native | Oyatie explicit | `failure-modes.md:157-168` | additive |
| D5-46: OCI Always Free DemoTrial | not counterpart feature | Oyatie required but missing | `ADR-0328:3491-3697`; tenant_class adoption matrix lacks | gap |
| D5-47: OpenTofu six-context deployability | not counterpart feature | Oyatie required but missing | `iac/terraform/*.tf`; missing context dirs | gap |
| D5-48: Headline parity verdict | union coverage bar | Oyatie partial | Braintree and platform/admin gaps material | partial |
| D5-49: Top missing capability family | hosted checkout/components/vault/network tokens/Braintree adapter | local evidence above | gap cluster |
| D5-50: Additive capability family | cell isolation/Cedar/audit-chain/regulated journeys | local evidence above | ahead of counterparts |

### §3.6 Dimension 6 — Multi-Context Deployment Support
| Context | Required Path | Payments Evidence | Status |
|---|---|---|---|
| D6-01: `oyatie-public-cloud` | `iac/oyatie-public-cloud/` | absent | missing IaC |
| D6-02: `guest-on-aws` | `iac/guest-on-aws/` | absent | missing IaC |
| D6-03: `guest-on-oci` | `iac/oci-guest/` | absent | missing IaC |
| D6-04: `guest-on-oci` Always Free | `iac/oci-guest/always-free/` | absent | missing IaC |
| D6-05: `on-prem` | `iac/on-prem/` | absent | missing IaC |
| D6-06: `colo` | `iac/colo/` | absent | missing IaC |
| D6-07: `oyatie-as-cloud-provider` | `iac/oyatie-iaas/` | absent | missing IaC |
| D6-08: Manifest context field | `manifest.json` should name contexts | absent | missing declaration |
| D6-09: Correctly N/A contexts | none for payments by brief template | none marked N/A | no N/A accepted |
| D6-10: Public cloud support | required | no OpenTofu root | unsupported |
| D6-11: AWS guest support | required | no AWS guest module | unsupported |
| D6-12: OCI guest support | required | no OCI guest module | unsupported |
| D6-13: On-prem support | required | no on-prem module | unsupported |
| D6-14: Colo support | required | no colo module | unsupported |
| D6-15: Oyatie provider support | required | no Oyatie IaaS module | unsupported |
| D6-16: Kubernetes packaging | present in Helm/Kustomize | `iac/helm`, `iac/kustomize` | useful but insufficient |
| D6-17: Network policy | present | `iac/network-policy/payments-network-policy.yaml:1-161` | useful but context-neutral |
| D6-18: OpenBao policy | present | `iac/openbao/payments-policy.hcl:1-78` | useful but context-neutral |
| D6-19: Edge WAF | present | `iac/edge-waf.yaml:1-146` | vendor-tinted; not context root |
| D6-20: ECH/TLS | present | `iac/ech-config.yaml:1-51`; `iac/tls/payments-ech-config.yaml:1-63` | useful but context-neutral |
| D6-21: Direct AWS WAF naming | present | `iac/edge-waf.yaml:1-146` | forbidden-pattern risk |
| D6-22: Cloudflare edge naming | present | `iac/edge-waf.yaml:1-146` | provider-specific IaC concern |
| D6-23: GCP state backend | present in Terraform file | `iac/terraform/payments-crdb.tf:18-19` | provider-specific drift |
| D6-24: Direct cloud-vendor APIs from business logic | no source tree | cannot confirm in source; no direct code evidence | unresolved but no finding |
| D6-25: Direct cloud-vendor APIs from docs/IaC | vendor configs present | WAF/GCP backend evidence | drift |
| D6-26: Per-context state backend | required by ADR-0328 | absent | missing |
| D6-27: Per-context plan evidence | required by brief template | absent | missing |
| D6-28: Per-context apply gate | required by ADR-0328 | absent | missing |
| D6-29: Per-context secret wiring | required | OpenBao generic only | partial |
| D6-30: Per-context policy bundle | required | Cedar exists but not context-bound | partial |
| D6-31: Per-context database substrate | required | Terraform CRDB only | drift |
| D6-32: Per-context event bus substrate | required | no context module | missing |
| D6-33: Per-context audit-chain substrate | required | handoff docs only | missing IaC |
| D6-34: Per-context observability substrate | required | dashboards only | missing IaC |
| D6-35: Per-context payment PSP secret delivery | required | OpenBao references only | partial |
| D6-36: Per-context WAF substrate | required | generic/vendor WAF config | partial/drift |
| D6-37: Per-context load balancer substrate | required | Helm ingress values only | partial |
| D6-38: Per-context DNS/cert substrate | required | ECH/cert docs only | partial |
| D6-39: Per-context CI matrix | required | no manifest | missing |
| D6-40: Multi-context cost profiles | required | cost-budget lacks context rows | missing |
| D6-41: Multi-context capacity overlays | required | capacity model not context-specific | partial |
| D6-42: Multi-context runbook overlays | expected | runbooks are generic | partial |
| D6-43: Multi-context failure modes | expected | failure modes generic | partial |
| D6-44: Multi-context compliance overlays | expected | compliance doc broad, not six-context exact | partial |
| D6-45: Multi-context tenant onboarding | required | no `tofu` onboarding flow | missing |
| D6-46: Multi-context provider-agnostic abstraction | required | architecture mentions substrate dependencies, but IaC not aligned | partial |
| D6-47: Severity basis | `ADR-0328:4106-4118` | payments is in-scope non-P0 | P1 |
| D6-48: Headline context verdict | all six expected | zero canonical context roots | P1 missing support |
| D6-49: Remediation hint | create six OpenTofu roots and manifest fields | source ADR | actionable |
| D6-50: Stop condition | no claim of deployable-context completeness until all six roots pass plan checks | audit | required |

### §3.7 Dimension 7 — OpenTofu IaC Coverage
| Check | Evidence | Result | Severity |
|---|---|---|---|
| D7-01: `iac/` exists. | inventory | yes | P3-positive |
| D7-02: `iac/helm/` exists. | inventory | yes, packaging only | P3 |
| D7-03: `iac/kustomize/` exists. | inventory | yes, packaging only | P3 |
| D7-04: `iac/network-policy/` exists. | inventory | yes | P3-positive |
| D7-05: `iac/openbao/` exists. | inventory | yes | P3-positive |
| D7-06: `iac/tls/` exists. | inventory | yes | P3-positive |
| D7-07: `iac/terraform/` exists. | inventory | yes | P1 |
| D7-08: `payments-crdb.tf` title says Terraform. | `iac/terraform/payments-crdb.tf:1` | forbidden naming/engine drift | P1 |
| D7-09: `payments-crdb.tf` uses `terraform {}` block. | `iac/terraform/payments-crdb.tf:6` | Terraform syntax under Terraform directory | P1 |
| D7-10: `payments-crdb.tf` uses GCS backend `oyatie-terraform-state`. | `iac/terraform/payments-crdb.tf:18-19` | forbidden Terraform-state drift | P1 |
| D7-11: `payments-crdb.tf` provisions Cockroach cluster. | `iac/terraform/payments-crdb.tf:26` | useful target, wrong engine/context | P1 |
| D7-12: `payments-edge-waf.tf` title says Terraform. | `iac/terraform/payments-edge-waf.tf:1` | forbidden naming/engine drift | P1 |
| D7-13: `payments-edge-waf.tf` uses `terraform {}` block. | `iac/terraform/payments-edge-waf.tf:6` | engine drift | P1 |
| D7-14: `payments-secret-bindings.tf` title says Terraform. | `iac/terraform/payments-secret-bindings.tf:1` | engine drift | P1 |
| D7-15: `payments-secret-bindings.tf` uses `terraform {}` block. | `iac/terraform/payments-secret-bindings.tf:6` | engine drift | P1 |
| D7-16: Edge WAF says backed by Terraform. | `iac/edge-waf.yaml:3-7` | engine drift | P1 |
| D7-17: ECH config says materialized by Terraform. | `iac/ech-config.yaml:28` | engine drift | P1 |
| D7-18: PQC cert references Terraform WAF file. | `iac/pqc-cert.yaml:10` | engine drift | P1 |
| D7-19: Required `iac/oyatie-public-cloud/main.tf`. | ADR required | absent | P1 |
| D7-20: Required `iac/guest-on-aws/main.tf`. | ADR required | absent | P1 |
| D7-21: Required `iac/oci-guest/main.tf`. | ADR required | absent | P1 |
| D7-22: Required `iac/on-prem/main.tf`. | ADR required | absent | P1 |
| D7-23: Required `iac/colo/main.tf`. | ADR required | absent | P1 |
| D7-24: Required `iac/oyatie-iaas/main.tf`. | ADR required | absent | P1 |
| D7-25: Required `versions.tf` per context. | `ADR-0328:2296-2309` | absent | P1 |
| D7-26: Required `variables.tf` per context. | `ADR-0328:2296-2309` | absent | P1 |
| D7-27: Required `outputs.tf` per context. | `ADR-0328:2296-2309` | absent | P1 |
| D7-28: Required context README per context. | `ADR-0328:2296-2309` | absent | P1 |
| D7-29: State backend per context. | `ADR-0328:2373-2389` | absent, except Terraform GCS drift | P1 |
| D7-30: Sigstore signing wiring. | `ADR-0328:3897-3940` | absent | P1 |
| D7-31: Module provenance/signature references. | `ADR-0328:3897-3940` | absent | P1 |
| D7-32: `null_resource` pattern. | grep evidence from investigation | not found | aligned |
| D7-33: `local-exec` pattern. | grep evidence from investigation | not found | aligned |
| D7-34: `remote-exec` pattern. | grep evidence from investigation | not found | aligned |
| D7-35: SSH provisioner pattern. | grep evidence from investigation | not found | aligned |
| D7-36: Hand-edited tfstate. | no file evidence | not found | aligned |
| D7-37: Terraform Cloud. | no file evidence | not found | aligned |
| D7-38: Pulumi reference. | investigation grep | not found | aligned |
| D7-39: CloudFormation reference. | investigation grep | not found | aligned |
| D7-40: ARM/Bicep reference. | investigation grep | not found | aligned |
| D7-41: Direct business logic cloud SDK call. | no source tree | no evidence | unresolved |
| D7-42: OpenBao secret policy exists. | `iac/openbao/payments-policy.hcl:1-78` | useful context module input | P3-positive |
| D7-43: Network policy exists. | `iac/network-policy/payments-network-policy.yaml:1-161` | useful context module input | P3-positive |
| D7-44: Helm app values exist. | `iac/helm/payments-app/values.yaml:1-166` | useful deployment input | P3-positive |
| D7-45: Helm webhook values exist. | `iac/helm/payments-webhook-handler/values.yaml:1-79` | useful deployment input | P3-positive |
| D7-46: Kustomize base exists. | `iac/kustomize/base/kustomization.yaml:1-32` | useful deployment input | P3-positive |
| D7-47: OpenTofu coverage verdict. | D7-07 through D7-31 | failing canonical bar | P1 |
| D7-48: Remediation hint. | ADR required | rename/port Terraform modules into six OpenTofu context roots with state/signing | P1 |
| D7-49: Verification needed after remediation. | `tofu init`, `tofu plan`, signed module check | not available now | P1 |
| D7-50: Claim boundary. | audit | payments cannot claim OpenTofu coverage today | P1 |

### §3.8 Dimension 8 — OS Support Matrix
| OS/Check | Required Status | Payments Evidence | Result |
|---|---|---|---|
| D8-01: Manifest path `microservices/payments/supported-oses.json`. | required | absent | P1 |
| D8-02: Tier-1 Ubuntu LTS on x86_64. | blocking | no manifest | missing |
| D8-03: Tier-1 Ubuntu LTS on arm64. | blocking | no manifest | missing |
| D8-04: Tier-1 Debian stable on x86_64. | blocking | no manifest | missing |
| D8-05: Tier-1 Debian stable on arm64. | blocking | no manifest | missing |
| D8-06: Tier-1 RHEL-compatible on x86_64. | blocking | no manifest | missing |
| D8-07: Tier-1 RHEL-compatible on arm64. | blocking | no manifest | missing |
| D8-08: Tier-1 Fedora latest on x86_64. | blocking | no manifest | missing |
| D8-09: Tier-1 Fedora latest on arm64. | blocking | no manifest | missing |
| D8-10: Tier-1 SUSE/openSUSE on x86_64. | blocking | no manifest | missing |
| D8-11: Tier-1 SUSE/openSUSE on arm64. | blocking | no manifest | missing |
| D8-12: Tier-1 Talos Linux on x86_64. | blocking | no manifest | missing |
| D8-13: Tier-1 Talos Linux on arm64. | blocking | no manifest | missing |
| D8-14: Tier-1 Flatcar Container Linux on x86_64. | blocking | no manifest | missing |
| D8-15: Tier-1 Flatcar Container Linux on arm64. | blocking | no manifest | missing |
| D8-16: Tier-1 Bottlerocket on x86_64. | blocking | no manifest | missing |
| D8-17: Tier-1 Bottlerocket on arm64. | blocking | no manifest | missing |
| D8-18: Tier-1 Windows 11 client on x86_64. | blocking if desktop/support tool exists | no manifest | missing |
| D8-19: Tier-1 Windows 11 client on arm64. | blocking if desktop/support tool exists | no manifest | missing |
| D8-20: Tier-1 macOS Apple Silicon M5+ on arm64. | blocking for developer/admin clients if any | no manifest | missing |
| D8-21: Tier-2 ppc64le. | test-only | no manifest | missing declaration |
| D8-22: Tier-2 s390x. | test-only | no manifest | missing declaration |
| D8-23: Intel macOS out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-24: Apple Silicon M1-M4 out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-25: FreeBSD out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-26: OpenBSD out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-27: Windows Server out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-28: Solaris/illumos out-of-scope. | must be explicit | no manifest | missing exclusion |
| D8-29: RPM package format. | required for RHEL/Fedora/SUSE family | no manifest | missing |
| D8-30: DEB package format. | required for Ubuntu/Debian family | no manifest | missing |
| D8-31: `.pkg` package format. | required for macOS if relevant | no manifest | missing |
| D8-32: Homebrew cask/tap. | required for macOS if relevant | no manifest | missing |
| D8-33: Talos extension. | required | no manifest | missing |
| D8-34: Flatcar extension. | required | no manifest | missing |
| D8-35: Bottlerocket settings/user-data. | required | no manifest | missing |
| D8-36: Container image fallback. | likely required | Helm images imply containers, but no OS matrix | partial |
| D8-37: Tier-1 CI lanes. | required blocking | no manifest | missing |
| D8-38: Tier-2 CI lanes. | test-only | no manifest | missing |
| D8-39: Out-of-scope CI skips. | explicit | no manifest | missing |
| D8-40: Rust build portability. | required | no source workspace | unproved |
| D8-41: No Python interpreter dependency. | `ADR-0328:2964-2968` | no `.py` files | aligned |
| D8-42: No generic Linux claim. | `ADR-0328:2985-2986` | no supported OS doc | missing positive evidence |
| D8-43: Helm chart OS assumptions. | container/K8s | no host OS mapping | partial |
| D8-44: OpenBao policy OS assumptions. | none | context-neutral | partial |
| D8-45: Runbooks OS assumptions. | generic | no per-OS instructions | partial |
| D8-46: Developer onboarding OS assumptions. | onboarding doc exists | no canonical OS matrix | partial |
| D8-47: Severity basis. | `ADR-0328:3023-3030` | in-scope service with missing manifest | P1 |
| D8-48: Remediation hint. | create `supported-oses.json` with Tier-1/Tier-2/exclusions/package/CI fields | actionable | P1 |
| D8-49: Verification hint. | each Tier-1 lane must have build/test evidence | absent today | P1 |
| D8-50: OS verdict. | aggregate | no OS support matrix coverage | P1 |

### §3.9 Dimension 9 — Rust-Strict Language Coverage
| Check | Evidence | Result | Severity |
|---|---|---|---|
| D9-01: `.py` files. | extension scan | none found | aligned |
| D9-02: `.js` files. | extension scan | none found | aligned |
| D9-03: `.ts` files. | extension scan | none found | aligned |
| D9-04: `.tsx` files. | extension scan | none found | aligned |
| D9-05: `.rb` files. | extension scan | none found | aligned |
| D9-06: `.go` files. | extension scan | none found | aligned |
| D9-07: `.java` files. | extension scan | none found | aligned |
| D9-08: `.scala` files. | extension scan | none found | aligned |
| D9-09: `.groovy` files. | extension scan | none found | aligned |
| D9-10: `.php` files. | extension scan | none found | aligned |
| D9-11: `.fs`/`.fsx` files. | extension scan | none found | aligned |
| D9-12: `.cs` files outside WinUI frontend. | extension scan | none found | aligned |
| D9-13: `.kt` files outside Android frontend. | extension scan | none found | aligned |
| D9-14: `.swift` files outside Apple frontend. | extension scan | none found | aligned |
| D9-15: Markdown files. | inventory | authorized doc type | aligned |
| D9-16: YAML files. | inventory | authorized config/contract type | aligned |
| D9-17: JSON files. | inventory | authorized config/dashboard type | aligned |
| D9-18: Proto file. | `contracts/payments-v1.proto:1-312` | authorized contract type | aligned |
| D9-19: OpenSLO YAML. | `slos/*.openslo.yaml` | authorized SLO type | aligned |
| D9-20: Cedar policy files. | `policy/*.cedar` | authorized policy type | aligned |
| D9-21: HCL OpenBao policy. | `iac/openbao/payments-policy.hcl` | not in ADR authorized extension list; infrastructure config | P2 classification needed |
| D9-22: `.tf` files. | `iac/terraform/*.tf` | extension authorized but engine/name violates OpenTofu policy | P1 |
| D9-23: Proto `go_package`. | `contracts/payments-v1.proto:9` | generated SDK metadata; no Go source | P2 provenance needed |
| D9-24: Proto `java_package`. | `contracts/payments-v1.proto:10-11` | generated SDK metadata; no Java source | P2 provenance needed |
| D9-25: SDK plan lists Rust plus TypeScript/iOS/Android/Python client SDKs. | `sdk-plan.md:20` | needs strict generated/client exception | P2 |
| D9-26: Contract test validates Rust and TypeScript stubs nightly. | `test-plans/contract-test-strategy.md:332` | needs generated-stub boundary | P2 |
| D9-27: Reference implementation is Rust SDK. | `reference-implementations/charge-and-refund-rust-sdk.md:1-248` | aligned | P3-positive |
| D9-28: Canonical build invocation. | `ADR-0328:3288-3289` | no source workspace to run | P2 |
| D9-29: Forbidden backend build invocations. | `ADR-0328:3303-3320` | not found | aligned |
| D9-30: Frontend path `frontend/<platform>/`. | inventory | absent | N/A |
| D9-31: Swift frontend allowance. | ADR policy | no Swift files | N/A |
| D9-32: Kotlin frontend allowance. | ADR policy | no Kotlin files | N/A |
| D9-33: WinUI3 frontend allowance. | ADR policy | no C#/XAML files | N/A |
| D9-34: Leptos/frontend Rust allowance. | ADR policy | no frontend Rust files | N/A |
| D9-35: Shell scripts. | inventory | none observed in payments file list | aligned |
| D9-36: SQL files. | inventory | none observed | N/A |
| D9-37: `Chart.yaml` files. | Helm | config, not runtime source | allowed with context caveat |
| D9-38: Dashboard JSON. | dashboards | config, not source | allowed |
| D9-39: OpenAPI YAML. | contracts | contract, authorized | aligned |
| D9-40: AsyncAPI YAML. | contracts | contract, authorized | aligned |
| D9-41: Markdown sample code. | reference implementation | Rust sample only in sampled file | aligned |
| D9-42: WAF rule name contains `PHPRuleSet`. | `iac/edge-waf.yaml:52` | vendor rule label, not PHP source | not violation |
| D9-43: No `src/` means no backend source language violation can be found. | inventory | documentation/spec-only state | partial |
| D9-44: Rust crate list appears in PRD/architecture but no Cargo workspace exists in payments. | `PRD.md:1235-1256`; `ARCHITECTURE.md:57-74` | implementation gap | P2 |
| D9-45: Build invocation absent from microservice docs. | docs scan evidence | no local cargo command surface | P2 |
| D9-46: Language policy classification. | aggregate | aligned on files, partial on SDK/generation, failing on build evidence | P2 |
| D9-47: Remediation hint for SDKs. | ADR exception process | mark client SDKs as generated artifacts with Rust codegen provenance or move out of backend scope | P2 |
| D9-48: Remediation hint for proto options. | generated metadata | document no committed Go/Java backend source and generated outputs are distribution artifacts | P2 |
| D9-49: Remediation hint for build. | `cargo build --workspace --release --all-features --locked` | add workspace/package manifest or explain service remains design-only | P2 |
| D9-50: Rust-strict verdict. | extension scan and docs | no forbidden source files; policy documentation still incomplete | P2 |

## §4 Findings Summary
| Severity | Dimension | Finding | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | D6/D7 | Missing all six canonical deployment context IaC roots. | `ADR-0328:1730-2239`; inventory `iac/` | Add `iac/oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `on-prem`, `colo`, `oyatie-iaas` OpenTofu roots. |
| P1 | D7 | Terraform engine and state drift in payments IaC. | `iac/terraform/payments-crdb.tf:1-26`; `iac/edge-waf.yaml:3-7` | Port to OpenTofu naming and context modules; remove Terraform state naming. |
| P1 | D8 | Missing `supported-oses.json` and Tier-1/Tier-2/out-of-scope OS matrix. | `ADR-0328:2907-2927`; inventory | Add machine-readable OS manifest and CI/package evidence. |
| P1 | D4/D6 | OCI Always Free DemoTrial path and tier reconciliation missing. | `ADR-0328:3491-3697`; `tenant_class adoption record:13-39` | Add `iac/oci-guest/always-free/` and DemoTrial resource profile capped to Always Free. |
| P1 | D1/D3 | Braintree required by top-3/PRD/tier/migration but absent from adapters and contracts. | `PRD.md:1381-1410`; `manifest.json:94-102`; `contracts/openapi-v1.yaml:322-349` | Add Braintree adapter/catalog/enum/API semantics or explicitly demote with canonical approval. |
| P1 | D1/D3 | Bounded-context count contradiction: PRD 14/~80 crates vs architecture/README/manifest 7. | `PRD.md:1235-1256`; `ARCHITECTURE.md:57-103`; `README.md:45-55` | Choose one bounded-context map and update PRD, architecture, README, manifest, catalog. |
| P2 | D1 | API naming contradiction: PRD says `/v1/transfers`; OpenAPI has `/v1/payouts`. | `PRD.md:119-125`; `contracts/openapi-v1.yaml:204-242` | Decide transfer-vs-payout terminology and add aliases/deprecation if needed. |
| P2 | D1 | Broken contract references in tenant_class adoption matrix. | `tenant_class adoption record:130-137` | Point to `contracts/openapi-v1.yaml`, `asyncapi-v1.yaml`, `payments-v1.proto`. |
| P2 | D1 | Broken policy schema reference. | `ARCHITECTURE.md:149`; inventory | Add `policy/schema.cedarschema` or remove reference. |
| P2 | D1 | Cross-handoff uses `policies/` but actual directory is `policy/`. | `cross-microservice-handoffs.md:16`; inventory | Rename reference or directory consistently. |
| P2 | D1/D3 | Charge latency target is inconsistent across PRD, capacity model, and OpenSLO. | `PRD.md:96-105`; `capacity-model.md:127-138`; `slos/charge-api-latency.openslo.yaml:1-46` | Select one tiered latency contract and propagate. |
| P2 | D1/D4 | Architecture/compliance contain content-pass scaffolding. | `ARCHITECTURE.md:174-267`; `compliance.md:64-158` | Replace scaffold blocks with bespoke payment design or delete. |
| P2 | D2 | Outbound references include missing `policy-engine`, `notifications`, and `commerce-product-recommendation`. | `ARCHITECTURE.md:48-55`; `manifest.json:286-304` | Confirm service names or create explicit external/deferred classification. |
| P2 | D9 | SDK plan and proto generated-language metadata need Rust-strict provenance. | `sdk-plan.md:20`; `contracts/payments-v1.proto:9-11` | Document generated client boundary and forbid committed non-Rust backend source. |
| P2 | D3/D9 | No `src/` or Cargo workspace means build invocation cannot be verified. | inventory; `ADR-0328:3288-3289` | Add Rust workspace skeleton or mark current state as design-only. |
| P3 | D1 | Manifest malformed benchmark refs and duplicate pack names reduce machine readability. | `manifest.json:312-338` | Normalize refs and pack names. |
| P3 | D5 | Existing parity docs drift toward Checkout.com/Toss/PayPal rather than assigned Braintree union. | `competitor-parity-matrix.md:19-45`; `benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md:1-112` | Replace counterpart baseline with Stripe/Adyen/Braintree union. |
| P3 | D2 | Repo-wide reverse references to payments need Wave 14 aggregation. | chat history; local bounded search | Run orchestrator-wide reverse reference pass. |

Severity counts:
- P0: 0.
- P1: 6.
- P2: 9.
- P3: 3.

## §5 Open Questions for Wave 14 Aggregation
- Is payments officially a P0 µservice, or should ADR-0328's P1 non-P0 severity classification stand for this wave?
- Should Braintree be a required adapter in MVP because the user assigned it as a top-3 counterpart, or can Braintree remain migration-only until Paid/Paid?
- Should `/v1/transfers` be an alias for `/v1/payouts`, or should the PRD be rewritten to use payout terminology consistently?
- Which bounded-context map is canonical: PRD's 14 contexts and about 80 crates, or architecture/README/manifest's 7 contexts and about 18 crates?
- Should `policy-engine`, `notifications`, and `commerce-product-recommendation` be created services, renamed references, or explicitly external/deferred dependencies?
- Should the architecture classification remain `INTERNAL_ONLY`, or should API/SDK/marketplace docs get a public/partner contract split?
- Should TypeScript/Python client SDK references remain as generated distribution artifacts, or should they be removed under Rust-strict policy until an ADR exception exists?
- Should existing Terraform modules be ported in place to OpenTofu context roots, or should they be archived as provenance-only examples after canonical OpenTofu modules land?
- Should OCI Always Free DemoTrial permit sandbox-only no-PAN flows, or should DemoTrial have a separate paid/sandbox split outside `guest-on-oci`?
- Should payments own tax calculation, or only post tax ledger metadata from a separate tax/compliance service?

<!-- ORCHESTRATOR REPORT
  µservice: payments
  deliverables_landed: microservices/payments/coherence-audit-2026-05-20.md (795 lines); microservices/payments/feature-parity-matrix-2026-05-20.md (412 lines); microservices/payments/performance-benchmark-numbers-2026-05-20.md (312 lines); microservices/payments/tenant_class-deltas-vs-counterparts-2026-05-20.md (357 lines)
  inventory_files_seen: 202
  inventory_lines_read: 60423
  chat_history_matches_processed: 4
  findings_p0: 0
  findings_p1: 6
  findings_p2: 9
  findings_p3: 3
  top_3_counterparts_confirmed: Stripe / Adyen / Braintree
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1876
-->
