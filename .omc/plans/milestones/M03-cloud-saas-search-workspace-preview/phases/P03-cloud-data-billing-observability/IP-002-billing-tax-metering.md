---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P03-IP-002
title: Cloud Billing tax-invoice + metering
status: regional-pack-tax-format-green; metering-outbox-runtime-pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Per-region tax-invoice format via regional pack; per-resource metering via outbox.
---

# M03-P03-IP-002 — Cloud Billing tax-invoice + metering

## Purpose
Per-region tax-invoice format via regional pack; per-resource metering via outbox.

## Symbols-to-grit-claim
```
crates/oya-cloud-billing-tax-app/src/lib.rs::CloudBillingTaxInvoiceFormatPolicy
crates/oya-cloud-billing-tax-app/src/lib.rs::generate_cloud_billing_invoice_from_api
crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs::regional_pack_tax_invoice_contract
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p oya-cloud-billing-kernel --all-features
cargo test -p oya-cloud-billing-domain --all-features
cargo test -p oya-cloud-billing-tax-app --all-features
cargo clippy -p oya-cloud-billing-tax-app --all-features --all-targets -- -D warnings
cargo run -q -p oya-dev-cli -- gate validate cohesion
cargo run -q -p oya-dev-cli -- gate validate planning-closure
oya verify --ci-required
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P03-IP-002 Cloud Billing tax-invoice + metering shipped; acceptance commands green' -i high -k 'M03-P03-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- Cloud Billing tax-invoice API no longer accepts arbitrary tax invoice formats
  detached from the selected regional pack; the runtime has one provider-neutral
  regional-pack-to-tax-format policy table covering KR/JP/EU/IN/BR/KSA/UAE.
- Account/invoice regional pack mismatch is rejected before invoice issuance.

Remaining boundary:
- This ChangeSet does not introduce a deployed billing runtime, live tax
  authority integration, persisted idempotency ledger, or provider-specific
  billing adapter. Those remain follow-up slices.
