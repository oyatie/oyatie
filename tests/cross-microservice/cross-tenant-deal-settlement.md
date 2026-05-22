---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-002-cross-tenant-deal-settlement
microservices_under_test:
  - marketplace
  - payments
  - audit-chain
  - governance
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children
---

# Cross-Tenant Deal Settlement

## Scenario Description

Rafael Cho, procurement lead for `tenant-orbit-retail-us`, buys a private marketplace data-cleaning workflow from seller `tenant-nami-crafts-kr`; the deal must move from marketplace order acceptance through payments authorization, split settlement, governance policy checks, sanctions screening, and audit-chain dual sealing without giving either tenant unilateral access to the other's private ledger.

## Pre-conditions

### Named tenant fixtures

- Buyer tenant: `tenant-orbit-retail-us`.
- Seller tenant: `tenant-nami-crafts-kr`.
- Marketplace operator tenant: `tenant-oyatie-marketplace`.
- Buyer principal: `principal-rafael-cho-procurement`.
- Seller principal: `principal-eunji-han-seller-admin`.
- Product listing: `listing-nami-data-cleaning-workflow-pro`.
- Offer id: `offer-private-orbit-nami-2026-05`.
- Deal id: `deal-orbit-nami-2026-05-20`.
- Payment intent id: `payint-orbit-nami-001`.
- Escrow ledger id: `escrow-ledger-marketplace-kr-us-001`.
- Buyer region: `us-east-1-cell-a`.
- Seller region: `kr-central-1-cell-a`.
- Settlement currency: `USD`.
- Seller payout currency: `KRW`.
- Exchange quote id: `fx-usd-krw-20260520-001`.
- Idempotency key: `idem-cmit-002-deal-settlement`.

### Named Cedar permits

- `permit-marketplace-cross-tenant-private-offer-accept`.
- `permit-payments-authorize-buyer-work-tenant`.
- `permit-payments-escrow-hold-marketplace`.
- `permit-payments-seller-payout-kr`.
- `permit-governance-sanctions-screen-cross-border`.
- `permit-governance-tax-withholding-evaluate`.
- `permit-audit-dual-seal-cross-tenant-deal`.
- `forbid-seller-read-buyer-private-ledger`.
- `forbid-buyer-read-seller-bank-instrument`.

### Named pack activations

- `pack-PCI-DSS-v4`.
- `pack-SOC2-Type-II-baseline`.
- `pack-KR-PIPA`.
- `pack-US-state-privacy-baseline`.
- `pack-cross-border-tax-withholding`.
- `pack-marketplace-seller-risk-baseline`.
- `pack-sanctions-ofac-eu-un-kr-mofa`.

### Starting state checks

- Buyer tenant is `ACTIVE` and KYB tier is `KYB_VERIFIED_BUSINESS`.
- Seller tenant is `ACTIVE` and seller risk tier is `LOW_RISK_ESTABLISHED`.
- Listing `listing-nami-data-cleaning-workflow-pro` is `PUBLISHED_PRIVATE_OFFER_ELIGIBLE`.
- Private offer is visible only to `tenant-orbit-retail-us`.
- Buyer payment instrument token `pm-orbit-visa-procurement` is active.
- Seller payout instrument token `ba-nami-shinhan-krw` is active.
- Governance sanctions cache is fresh within 15 minutes.
- Audit streams for buyer, seller, and marketplace operator exist and are empty for this deal id.

## Test Steps

1. Read the private marketplace offer.
   - API call: `GET /marketplace/v1/offers/offer-private-orbit-nami-2026-05?tenant_id=tenant-orbit-retail-us`.
   - Expected response: `200 OK` with `listing_id="listing-nami-data-cleaning-workflow-pro"` and `visibility_scope="SINGLE_BUYER_TENANT"`.
   - Assertion: seller cost details exclude bank-instrument metadata.

2. Attempt offer read from an unrelated tenant.
   - API call: `GET /marketplace/v1/offers/offer-private-orbit-nami-2026-05?tenant_id=tenant-random-observer`.
   - Expected response: `404 Not Found`.
   - Assertion: response does not reveal that the private offer exists.

3. Accept the private offer.
   - API call: `POST /marketplace/v1/offers/offer-private-orbit-nami-2026-05:accept`.
   - Expected response: `201 Created` with `deal_id="deal-orbit-nami-2026-05-20"` and `state="PENDING_GOVERNANCE_SCREEN"`.
   - Assertion: marketplace records buyer and seller tenant ids as independent sovereign tenants.

4. Run governance cross-tenant policy preflight.
   - API call: `POST /governance/v1/policy-decisions:check` with action `marketplace.deal.accept`.
   - Expected response: `200 OK` with `decision="ALLOW"` and `cedar_decision_id="cedar-cmit-002-preflight"`.
   - Assertion: decision references `permit-marketplace-cross-tenant-private-offer-accept`.

5. Screen buyer, seller, and beneficial owners.
   - API call: `POST /governance/v1/sanctions/screenings`.
   - Expected response: `202 Accepted` with `screening_id="screen-orbit-nami-001"` and `state="CLEAR"`.
   - Assertion: screening covers OFAC, EU, UN, and KR-MOFA packs.

6. Compute tax and withholding obligations.
   - API call: `POST /governance/v1/tax/withholding:quote` for `deal-orbit-nami-2026-05-20`.
   - Expected response: `200 OK` with `withholding_required=true` and `withholding_rate_bps=300`.
   - Assertion: withholding basis cites seller jurisdiction and buyer marketplace operator role.

7. Create payment authorization intent.
   - API call: `POST /payments/v1/payment-intents`.
   - Expected response: `201 Created` with `payment_intent_id="payint-orbit-nami-001"` and `state="REQUIRES_CONFIRMATION"`.
   - Assertion: payload uses buyer tenant id and marketplace deal id, not seller bank details.

8. Confirm payment authorization.
   - API call: `POST /payments/v1/payment-intents/payint-orbit-nami-001:confirm`.
   - Expected response: `200 OK` with `state="AUTHORIZED"` and `authorization_code="auth-orbit-nami-777"`.
   - Assertion: authorization stores PSP token reference only and emits no PAN.

9. Place funds into marketplace escrow.
   - API call: `POST /payments/v1/escrow/holds`.
   - Expected response: `201 Created` with `escrow_hold_id="hold-orbit-nami-001"` and `state="HELD"`.
   - Assertion: escrow ledger belongs to marketplace operator tenant, not buyer or seller.

10. Seal buyer-side deal audit event.
    - API call: `POST /audit-chain/v1/streams/tenant-orbit-retail-us.marketplace/events`.
    - Expected response: `201 Created` with `event_class="MarketplaceDealAccepted"` and Merkle proof.
    - Assertion: buyer audit event omits seller payout instrument.

11. Seal seller-side deal audit event.
    - API call: `POST /audit-chain/v1/streams/tenant-nami-crafts-kr.marketplace/events`.
    - Expected response: `201 Created` with `event_class="MarketplaceDealReceived"` and Merkle proof.
    - Assertion: seller audit event omits buyer payment instrument.

12. Activate the purchased workflow entitlement.
    - API call: `POST /marketplace/v1/deals/deal-orbit-nami-2026-05-20/entitlements`.
    - Expected response: `201 Created` with `entitlement_id="ent-orbit-nami-workflow-pro"` and `state="ACTIVE"`.
    - Assertion: entitlement grants buyer runtime use, not seller tenant data access.

13. Capture settlement-ready signal.
    - API call: `POST /marketplace/v1/deals/deal-orbit-nami-2026-05-20:settlementReady`.
    - Expected response: `200 OK` with `state="READY_TO_CAPTURE"`.
    - Assertion: settlement cannot proceed until buyer and seller audit seals exist.

14. Capture the authorized payment.
    - API call: `POST /payments/v1/payment-intents/payint-orbit-nami-001:capture`.
    - Expected response: `200 OK` with `state="CAPTURED"` and `capture_id="cap-orbit-nami-001"`.
    - Assertion: capture amount equals offer price plus buyer tax, not seller payout amount.

15. Create settlement split.
    - API call: `POST /payments/v1/settlements/splits`.
    - Expected response: `201 Created` with `split_id="split-orbit-nami-001"`.
    - Assertion: split lines include seller gross, marketplace fee, tax withholding, and FX reserve.

16. Quote foreign exchange for seller payout.
    - API call: `POST /payments/v1/fx/quotes/fx-usd-krw-20260520-001:lock`.
    - Expected response: `200 OK` with `locked_until="2026-05-20T14:15:00Z"`.
    - Assertion: FX quote is tied to seller tenant and cannot be reused by buyer.

17. Release escrow to settlement.
    - API call: `POST /payments/v1/escrow/holds/hold-orbit-nami-001:release`.
    - Expected response: `200 OK` with `state="RELEASED_TO_SETTLEMENT"`.
    - Assertion: release references governance screening id and tax quote id.

18. Initiate seller payout.
    - API call: `POST /payments/v1/payouts`.
    - Expected response: `202 Accepted` with `payout_id="payout-nami-krw-001"` and `state="PROCESSING"`.
    - Assertion: payout payload carries seller tenant id and bank token, never buyer tenant id.

19. Record governance settlement approval.
    - API call: `POST /governance/v1/approvals/settlement`.
    - Expected response: `201 Created` with `approval_id="gov-approval-orbit-nami-001"`.
    - Assertion: approval includes Cedar decision ids for marketplace, payments, sanctions, and tax.

20. Seal marketplace operator audit event.
    - API call: `POST /audit-chain/v1/streams/tenant-oyatie-marketplace.settlement/events`.
    - Expected response: `201 Created` with `event_class="MarketplaceSettlementReleased"`.
    - Assertion: operator event links buyer proof hash and seller proof hash without copying tenant-private payloads.

21. Fetch deal settlement summary as buyer.
    - API call: `GET /marketplace/v1/deals/deal-orbit-nami-2026-05-20/settlement-summary`.
    - Expected response: `200 OK` with buyer-visible totals and entitlement status.
    - Assertion: response excludes seller bank token and seller internal risk score.

22. Fetch deal settlement summary as seller.
    - API call: `GET /marketplace/v1/seller/deals/deal-orbit-nami-2026-05-20/settlement-summary`.
    - Expected response: `200 OK` with seller gross, fees, withholding, FX, and payout state.
    - Assertion: response excludes buyer payment token and buyer procurement notes.

23. Replay cross-tenant settlement trace.
    - API call: `GET /audit-chain/v1/cross-tenant-traces/deal-orbit-nami-2026-05-20`.
    - Expected response: `200 OK` with buyer, seller, and operator proof references.
    - Assertion: replay validates dual sealing without co-mingling tenant audit streams.

24. Verify final deal state.
    - API call: `GET /marketplace/v1/deals/deal-orbit-nami-2026-05-20`.
    - Expected response: `200 OK` with `state="SETTLED"` and `settlement_state="PAYOUT_PROCESSING"`.
    - Assertion: deal cannot be `SETTLED` unless payment capture, governance approval, and audit seals all pass.

## Test Data Fixtures

### Fixture `PrivateOfferFixture`

```json
{
  "offer_id": "offer-private-orbit-nami-2026-05",
  "listing_id": "listing-nami-data-cleaning-workflow-pro",
  "seller_tenant_id": "tenant-nami-crafts-kr",
  "buyer_tenant_id": "tenant-orbit-retail-us",
  "price": {
    "amount_minor": 1250000,
    "currency": "USD"
  },
  "visibility_scope": "SINGLE_BUYER_TENANT",
  "settlement_terms": "ESCROW_CAPTURE_AFTER_ENTITLEMENT",
  "expires_at": "2026-05-27T00:00:00Z"
}
```

### Fixture `GovernanceScreeningFixture`

```yaml
screening_id: screen-orbit-nami-001
deal_id: deal-orbit-nami-2026-05-20
parties:
  buyer: tenant-orbit-retail-us
  seller: tenant-nami-crafts-kr
  operator: tenant-oyatie-marketplace
lists:
  - OFAC-SDN
  - EU-CFSP
  - UN-SC
  - KR-MOFA
expected_result: CLEAR
```

### Fixture `PaymentIntentFixture`

```json
{
  "payment_intent_id": "payint-orbit-nami-001",
  "deal_id": "deal-orbit-nami-2026-05-20",
  "buyer_tenant_id": "tenant-orbit-retail-us",
  "seller_tenant_id": "tenant-nami-crafts-kr",
  "amount_minor": 1250000,
  "currency": "USD",
  "payment_method_token": "pm-orbit-visa-procurement",
  "capture_method": "MANUAL_AFTER_MARKETPLACE_ENTITLEMENT"
}
```

### Fixture `SettlementSplitFixture`

```json
{
  "split_id": "split-orbit-nami-001",
  "gross_amount_minor": 1250000,
  "currency": "USD",
  "lines": [
    {
      "type": "SELLER_GROSS",
      "amount_minor": 1187500
    },
    {
      "type": "MARKETPLACE_FEE",
      "amount_minor": 62500
    },
    {
      "type": "TAX_WITHHOLDING",
      "amount_minor": 37500
    },
    {
      "type": "FX_RESERVE",
      "amount_minor": 5000
    }
  ]
}
```

### Fixture `CrossTenantAuditProofFixture`

```yaml
trace_id: trace-cmit-002-deal-orbit-nami
deal_id: deal-orbit-nami-2026-05-20
buyer_stream: tenant-orbit-retail-us.marketplace
seller_stream: tenant-nami-crafts-kr.marketplace
operator_stream: tenant-oyatie-marketplace.settlement
proofs:
  buyer_event: MarketplaceDealAccepted
  seller_event: MarketplaceDealReceived
  operator_event: MarketplaceSettlementReleased
privacy_rule: proof_hashes_link_streams_without_payload_copy
```

### Fixture `NegativeVisibilityFixture`

```json
{
  "tenant_id": "tenant-random-observer",
  "offer_id": "offer-private-orbit-nami-2026-05",
  "expected_status": 404,
  "expected_error_shape": {
    "code": "NOT_FOUND",
    "leaks_offer_existence": false
  }
}
```

## Assertion Catalogue

### What passes

- `PASS-MARKET-001`: private offer is visible only to the intended buyer tenant.
- `PASS-MARKET-002`: accepted deal records buyer and seller as sovereign tenants.
- `PASS-MARKET-003`: entitlement activates only after governance preflight.
- `PASS-MARKET-004`: settlement-ready state requires buyer and seller audit seals.
- `PASS-PAYMENTS-001`: authorization stores no PAN.
- `PASS-PAYMENTS-002`: escrow ledger belongs to marketplace operator tenant.
- `PASS-PAYMENTS-003`: capture amount matches buyer obligation.
- `PASS-PAYMENTS-004`: split lines balance to captured amount.
- `PASS-PAYMENTS-005`: payout payload references seller bank token only.
- `PASS-GOV-001`: sanctions screen includes all required lists.
- `PASS-GOV-002`: tax quote carries jurisdictional basis.
- `PASS-GOV-003`: settlement approval links every Cedar decision.
- `PASS-AUDIT-001`: buyer stream omits seller bank instrument.
- `PASS-AUDIT-002`: seller stream omits buyer payment token.
- `PASS-AUDIT-003`: operator stream links proof hashes, not private payloads.
- `PASS-CROSS-001`: buyer summary excludes seller private data.
- `PASS-CROSS-002`: seller summary excludes buyer private data.
- `PASS-CROSS-003`: unrelated tenant cannot infer private offer existence.
- `PASS-SLO-001`: payment authorization fits latency budget.
- `PASS-SLO-002`: audit dual-seal replay completes within budget.

### What fails

- `FAIL-MARKET-001`: private offer leaks to unrelated tenant.
- `FAIL-MARKET-002`: deal state reaches settled without entitlement.
- `FAIL-PAYMENTS-001`: PAN or raw bank account appears in API response.
- `FAIL-PAYMENTS-002`: escrow ledger belongs to buyer or seller tenant.
- `FAIL-PAYMENTS-003`: split lines do not balance.
- `FAIL-PAYMENTS-004`: payout uses buyer tenant id.
- `FAIL-GOV-001`: sanctions screen omits KR-MOFA.
- `FAIL-GOV-002`: tax quote lacks jurisdictional citation.
- `FAIL-AUDIT-001`: buyer and seller streams are co-mingled.
- `FAIL-AUDIT-002`: operator event copies buyer or seller private payload.
- `FAIL-CEDAR-001`: settlement mutation lacks Cedar decision id.
- `FAIL-CROSS-001`: buyer can read seller bank token.
- `FAIL-CROSS-002`: seller can read buyer payment token.
- `FAIL-REPLAY-001`: audit proof hashes fail verification.
- `FAIL-SLO-001`: settlement capture exceeds SLO budget.

## Failure Mode Coverage

- `FM-DEAL-001`: unrelated tenant learns private offer exists.
- `FM-DEAL-002`: seller tenant is treated as child-owned by buyer.
- `FM-DEAL-003`: marketplace accepts deal before governance screen.
- `FM-DEAL-004`: sanctions list cache is stale but accepted.
- `FM-DEAL-005`: tax withholding is calculated after payout release.
- `FM-DEAL-006`: payment intent stores raw card data.
- `FM-DEAL-007`: escrow ledger is scoped to the wrong tenant.
- `FM-DEAL-008`: capture happens before entitlement activation.
- `FM-DEAL-009`: split line arithmetic loses minor units.
- `FM-DEAL-010`: FX quote is reused across tenants.
- `FM-DEAL-011`: seller payout sees buyer procurement notes.
- `FM-DEAL-012`: buyer summary exposes seller bank token.
- `FM-DEAL-013`: dual-seal audit proof includes copied private payload.
- `FM-DEAL-014`: audit-chain ordering differs from payments settlement ordering.
- `FM-DEAL-015`: governance approval omits one Cedar decision id.
- `FM-DEAL-016`: marketplace state says `SETTLED` while payout creation failed.
- `FM-DEAL-017`: cross-border pack conflict is ignored.
- `FM-DEAL-018`: KR seller payout skips KR-PIPA metadata minimization.
- `FM-DEAL-019`: OFAC hit is soft-warned instead of blocking settlement.
- `FM-DEAL-020`: idempotent accept creates two deals.

## Cross-µservice Handoff Validation

- `HANDOFF-MARKET-GOV-OPENAPI`: marketplace acceptance request maps to governance action `marketplace.deal.accept`.
- `HANDOFF-GOV-MARKET-ASYNCAPI`: governance screening emits `governance.screening.clear.v1` consumed by marketplace.
- `HANDOFF-MARKET-PAYMENTS-OPENAPI`: payment intent references marketplace `deal_id` and `offer_id`.
- `HANDOFF-PAYMENTS-MARKET-ASYNCAPI`: `payments.authorization.created.v1` advances marketplace deal to `PAYMENT_AUTHORIZED`.
- `HANDOFF-PAYMENTS-AUDIT-PROTO`: `PaymentAuthorized` proto excludes PAN and raw bank details.
- `HANDOFF-MARKET-AUDIT-PROTO`: `MarketplaceDealAccepted` includes buyer and seller tenant ids.
- `HANDOFF-GOV-PAYMENTS-OPENAPI`: payment capture requires sanctions `CLEAR` and tax quote ids.
- `HANDOFF-PAYMENTS-GOV-ASYNCAPI`: settlement split emits withholding details for governance approval.
- `HANDOFF-AUDIT-MARKET-OPENAPI`: marketplace final state validates proof hashes before settlement-ready.
- `HANDOFF-CEDAR-ALL`: all mutation APIs persist `cedar_decision_id`.
- `HANDOFF-TENANT-SCOPE`: buyer and seller tenant ids remain separate in every contract.
- `HANDOFF-TRACE`: `trace-cmit-002-deal-orbit-nami` is preserved across all services.
- `HANDOFF-IDEMPOTENCY`: private-offer accept idempotency returns same deal id.
- `HANDOFF-ERROR`: sanctions failure maps to `SETTLEMENT_BLOCKED_BY_GOVERNANCE`.
- `HANDOFF-PRIVACY`: summary endpoints redact counterparty private instruments.

## SLO Conformance

- `SLO-OFFER-READ-P95`: private offer read P95 <= 200 ms.
- `SLO-OFFER-ACCEPT-P95`: offer acceptance P95 <= 300 ms before async screening.
- `SLO-GOV-SCREEN-P95`: sanctions clear response P95 <= 1500 ms with warm list cache.
- `SLO-TAX-QUOTE-P95`: tax quote P95 <= 500 ms.
- `SLO-AUTHORIZE-P95`: payment authorization P95 <= 900 ms.
- `SLO-ESCROW-HOLD-P95`: escrow hold P95 <= 400 ms.
- `SLO-CAPTURE-P95`: capture P95 <= 1200 ms.
- `SLO-PAYOUT-CREATE-P95`: payout creation P95 <= 800 ms.
- `SLO-AUDIT-SEAL-P99`: each audit append P99 <= 150 ms.
- `SLO-DUAL-REPLAY-P95`: cross-tenant replay P95 <= 2 seconds.
- `SLO-AVAILABILITY`: marketplace, payments, governance, and audit-chain endpoints target 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: one cell supports 100 cross-tenant settlements per minute with no tenant-scope collision.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests cross_tenant_deal_settlement -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-DEAL-ORBIT-NAMI`.
- Required fixture bundle: `fixtures/cross-msvc/deal-settlement-orbit-nami.yaml`.
- Required policy bundle: `cedar-bundle-2026-05-20-cross-msvc`.
- Required sanctions snapshot: `sanctions-snapshot-2026-05-20T13:45:00Z`.
- Required PSP mode: deterministic sandbox with no live charge.
- Required FX mode: deterministic quote table seeded from `fx-usd-krw-20260520-001`.
- Test isolation: buyer, seller, and operator audit streams are truncated only after proof export.
- Stop condition: deal reaches `SETTLED`, payout reaches `PROCESSING`, and every privacy assertion passes.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`.
- `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 rows 3, 15, 23, 29.
- `microservices/marketplace/contracts/openapi-v1.yaml`.
- `microservices/payments/contracts/openapi-v1.yaml`.
- `microservices/governance/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/asyncapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
