---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j82-notice-delivery
journey_id: j82-kr-fss-financial-fraud-24h-freeze
microservice: mail
role: notice-delivery
status: draft
date: 2026-05-20
pack_overlay: KR-FSS
jurisdiction: KR
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
layer_enum: ADR-0105 13-layer canonical enum
layout: ADR-0131 flat per-microservice layout
audit_contract: ADR-0263 event classes required
cedar_contract: ADR-0243 deny-wins authorization
---

# IP-journey-j82-notice-delivery - mail

## Goal

Implement the `notice-delivery` slice for `mail` so j82 can satisfy `KR-FSS` without leaking tenant data, collapsing provider-BYOK and encryption-BYOK meanings, or bypassing Cedar.

## PRD row alignment

- PRD anchor: microservices/mail/PRD.md when present, otherwise the service manifest and architecture surface for that microservice.
- Journey anchor: docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/.
- Regulator article focus: Electronic Financial Transactions Act KR fraud response.
- Rigor row: documentation-rigor.md section 2 IP row; one service, one single-PR-sized implementation plan.

## Files to author in the implementation PR

| File | Purpose | Notes |
|---|---|---|
| `microservices/mail/contracts/openapi/j82-notice-delivery-v1.yaml` | OpenAPI 3.2.0 REST surface | External read/write or admin action |
| `microservices/mail/contracts/asyncapi/j82-notice-delivery-events-v1.yaml` | AsyncAPI 3.1.0 event surface | Emits ADR-0263 events |
| `microservices/mail/contracts/proto/j82-notice-delivery-v1.proto` | proto3 internal RPC | Service-to-service call path |
| `microservices/mail/policy/j82-notice-delivery.cedar` | Cedar permit/forbid bundle | Deny-wins gate |
| `microservices/mail/runbooks/j82-notice-delivery-rollback.md` | Rollback and incident path | Includes regulator deadline handling |
| `microservices/mail/tests/j82_notice_delivery_test.rs` | Integration tests | Positive, negative, rollback, audit |

## Data model

Primary object: `kr-fss-fraud-freeze` with tenant_id, subject_id, pack_id, jurisdiction_code, purpose, data_class, deadline_at, byok_provider_ref, byok_encryption_ref, and prior_seal_ref.
The service may store a local projection only when it can be rebuilt from the audit-chain and source-of-truth service. Mutable state must carry tenant_id and data_class.

## Cedar fragment

```cedar
permit (principal is Principal, action == Action::"j82.mail.notice-delivery", resource is JourneyObject) when {
  principal.tenant_id == resource.tenant_id &&
  resource.pack_overlay == "KR-FSS" &&
  context.jurisdiction == "KR" &&
  context.data_class_allowed == true &&
  context.audit_chain_required == true
};
```

## Audit event classes

- `J82MailNoticeDeliveryStarted`.
- `J82MailNoticeDeliveryPermitted`.
- `J82MailNoticeDeliveryDenied`.
- `J82MailNoticeDeliveryCommitted`.
- `J82MailNoticeDeliveryRolledBack`.

## Implementation rows

### IP row 001 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: Electronic Financial Transactions Act KR fraud response.
Critical path: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 002 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-FSS suspicious transaction reporting expectations.
Critical path: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 003 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 29 safety measures.
Critical path: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 004 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 34 incident notice.
Critical path: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 005 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: AML/KYC regulator floor.
Critical path: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 006 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: Electronic Financial Transactions Act KR fraud response.
Critical path: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 007 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-FSS suspicious transaction reporting expectations.
Critical path: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 008 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 29 safety measures.
Critical path: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 009 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 34 incident notice.
Critical path: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 010 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: AML/KYC regulator floor.
Critical path: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 011 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: Electronic Financial Transactions Act KR fraud response.
Critical path: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 012 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-FSS suspicious transaction reporting expectations.
Critical path: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 013 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 29 safety measures.
Critical path: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 014 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 34 incident notice.
Critical path: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 015 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: AML/KYC regulator floor.
Critical path: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 016 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: Electronic Financial Transactions Act KR fraud response.
Critical path: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 017 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-FSS suspicious transaction reporting expectations.
Critical path: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 018 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 29 safety measures.
Critical path: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 019 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 34 incident notice.
Critical path: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 020 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: AML/KYC regulator floor.
Critical path: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 021 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: Electronic Financial Transactions Act KR fraud response.
Critical path: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 022 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-FSS suspicious transaction reporting expectations.
Critical path: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Contract: OpenAPI 3.2.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 023 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 29 safety measures.
Critical path: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Contract: AsyncAPI 3.1.0 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

### IP row 024 - mail notice-delivery
Scope: implement one mail behavior for j82, not a cross-service refactor.
Regulator anchor: KR-PIPA Art 34 incident notice.
Critical path: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Contract: proto3 envelope includes tenant_id, pack_id, data_class, purpose, deadline_at, and prior_seal_ref.
Authorization: Cedar evaluates in-process with deny-wins semantics and signed fragment bundle provenance.
State: local writes are idempotent by journey_id plus service_step_id; duplicate requests return the prior result.
provider-BYOK: external provider credential handles stay in the provider sidecar and never masquerade as encryption keys.
encryption-BYOK: cryptographic key references stay in cloud-secrets and rotate without changing provider routing.
Observability: emit latency histogram, denial counter, retry counter, deadline slack gauge, and audit seal latency.
Failure mode: downstream unavailable causes pause and retry, not partial completion or silent success.
Rollback: reverse only local pending mutation and append a rollback event that cites the original seal.
Tests: positive permit, denial on wrong tenant, denial on stale pack, duplicate idempotency, rollback after injected failure, and schema validation.
Review: doc-style, privacy, security, and compliance reviewers can map this row to one implementation commit.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-journey-j82-notice-delivery.md` matched `openapi, asyncapi, .proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j82-notice-delivery.md` matched `financial`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
