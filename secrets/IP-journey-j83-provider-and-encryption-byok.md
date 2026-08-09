---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j83-provider-and-encryption-byok
journey_id: j83-cn-pipl-data-localization-and-cac-assessment
microservice: cloud-secrets
role: provider-and-encryption-byok
status: draft
date: 2026-05-20
pack_overlay: CN-PIPL-2021
jurisdiction: CN
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
layer_enum: ADR-0105 13-layer canonical enum
layout: ADR-0131 flat per-microservice layout
audit_contract: ADR-0263 event classes required
cedar_contract: ADR-0243 deny-wins authorization
---

# IP-journey-j83-provider-and-encryption-byok - cloud-secrets

## Goal

Implement the `provider-and-encryption-byok` slice for `cloud-secrets` so j83 can satisfy `CN-PIPL-2021` without leaking tenant data, collapsing provider-BYOK and encryption-BYOK meanings, or bypassing Cedar.

## PRD row alignment

- PRD anchor: microservices/cloud-secrets/PRD.md when present, otherwise the service manifest and architecture surface for that microservice.
- Journey anchor: docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/.
- Regulator article focus: PIPL Art 28 sensitive personal information.
- Rigor row: documentation-rigor.md section 2 IP row; one service, one single-PR-sized implementation plan.

## Files to author in the implementation PR

| File | Purpose | Notes |
|---|---|---|
| `microservices/cloud-secrets/contracts/openapi/j83-provider-and-encryption-byok-v1.yaml` | OpenAPI 3.2.0 REST surface | External read/write or admin action |
| `microservices/cloud-secrets/contracts/asyncapi/j83-provider-and-encryption-byok-events-v1.yaml` | AsyncAPI 3.1.0 event surface | Emits ADR-0263 events |
| `microservices/cloud-secrets/contracts/proto/j83-provider-and-encryption-byok-v1.proto` | proto3 internal RPC | Service-to-service call path |
| `microservices/cloud-secrets/policy/j83-provider-and-encryption-byok.cedar` | Cedar permit/forbid bundle | Deny-wins gate |
| `microservices/cloud-secrets/runbooks/j83-provider-and-encryption-byok-rollback.md` | Rollback and incident path | Includes regulator deadline handling |
| `microservices/cloud-secrets/tests/j83_provider_and_encryption_byok_test.rs` | Integration tests | Positive, negative, rollback, audit |

## Data model

Primary object: `cn-pipl-cac-assessment` with tenant_id, subject_id, pack_id, jurisdiction_code, purpose, data_class, deadline_at, byok_provider_ref, byok_encryption_ref, and prior_seal_ref.
The service may store a local projection only when it can be rebuilt from the audit-chain and source-of-truth service. Mutable state must carry tenant_id and data_class.

## Cedar fragment

```cedar
permit (principal is Principal, action == Action::"j83.cloud-secrets.provider-and-encryption-byok", resource is JourneyObject) when {
  principal.tenant_id == resource.tenant_id &&
  resource.pack_overlay == "CN-PIPL-2021" &&
  context.jurisdiction == "CN" &&
  context.data_class_allowed == true &&
  context.audit_chain_required == true
};
```

## Audit event classes

- `J83CloudSecretsProviderAndEncryptionByokStarted`.
- `J83CloudSecretsProviderAndEncryptionByokPermitted`.
- `J83CloudSecretsProviderAndEncryptionByokDenied`.
- `J83CloudSecretsProviderAndEncryptionByokCommitted`.
- `J83CloudSecretsProviderAndEncryptionByokRolledBack`.

## Implementation rows

### IP row 001 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 28 sensitive personal information.
Critical path: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
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

### IP row 002 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 38 cross-border transfer pathways.
Critical path: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
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

### IP row 003 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 40 localization for CIIOs and threshold processors.
Critical path: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
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

### IP row 004 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 45 access and copy.
Critical path: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
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

### IP row 005 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 51 personal information impact assessment.
Critical path: documentation-rigor.md section 3.2.5 row 30 regional outage.
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

### IP row 006 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 57 breach notice.
Critical path: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
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

### IP row 007 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 28 sensitive personal information.
Critical path: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
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

### IP row 008 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 38 cross-border transfer pathways.
Critical path: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
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

### IP row 009 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 40 localization for CIIOs and threshold processors.
Critical path: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
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

### IP row 010 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 45 access and copy.
Critical path: documentation-rigor.md section 3.2.5 row 30 regional outage.
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

### IP row 011 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 51 personal information impact assessment.
Critical path: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
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

### IP row 012 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 57 breach notice.
Critical path: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
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

### IP row 013 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 28 sensitive personal information.
Critical path: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
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

### IP row 014 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 38 cross-border transfer pathways.
Critical path: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
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

### IP row 015 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 40 localization for CIIOs and threshold processors.
Critical path: documentation-rigor.md section 3.2.5 row 30 regional outage.
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

### IP row 016 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 45 access and copy.
Critical path: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
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

### IP row 017 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 51 personal information impact assessment.
Critical path: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
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

### IP row 018 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 57 breach notice.
Critical path: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
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

### IP row 019 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 28 sensitive personal information.
Critical path: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
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

### IP row 020 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 38 cross-border transfer pathways.
Critical path: documentation-rigor.md section 3.2.5 row 30 regional outage.
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

### IP row 021 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 40 localization for CIIOs and threshold processors.
Critical path: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
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

### IP row 022 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 45 access and copy.
Critical path: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
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

### IP row 023 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 51 personal information impact assessment.
Critical path: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
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

### IP row 024 - cloud-secrets provider-and-encryption-byok
Scope: implement one cloud-secrets behavior for j83, not a cross-service refactor.
Regulator anchor: PIPL Art 57 breach notice.
Critical path: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
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

## Grep-recognized counterpart anchor

GitHub Actions Secrets is cited only for CI secret-distribution verification in this BYOK journey: provider credentials may enter tests as workflow secrets, but implementation must convert them to cloud-secrets handles before routing or encryption logic. The primary comparator truth remains OpenBao/Vault, managed secret stores, and KMS/HSM BYOK behavior.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-journey-j83-provider-and-encryption-byok.md`.
