# ADR-SVC-CG-005: Self-revocation — data-subject-initiated revocation for B2C grants

- Status: Accepted
- Scope: service
- Date: 2026-05-18
- Authority: ADR-0214 §2.1 + §1 (B2C use case), GDPR Art. 7(3) (right to withdraw consent).

## Context

For B2C agreements (consumer-initiated grants of personal data to a service), the data subject must
be able to withdraw consent at any time and have the revocation take effect immediately. GDPR Art.
7(3) is explicit: withdrawal must be as easy as giving consent.

For B2B agreements, only grantor + grantee tenants may revoke; data subject revocation does not apply.

## Decision

`DataSharingAgreement` may set `data_subject_self_revocation=true`. When true:
- The data subject (identified by `subject_principal_id` field on the agreement) may revoke via a
  consumer-facing API endpoint without grantor approval.
- Self-revocation has the same legal weight as grantor revocation.
- Revocation reason recorded as `DataSubjectInitiated` in the audit-chain.

Implementation:
- Consumer-facing UI (in ops-portal's consumer-companion surface) presents one-click revoke.
- Self-revoke API: `POST /v1/agreements/{id}/self-revoke` authenticated via the subject's identity
  (not tenant-level).
- Audit emission: bilateral chain entry on both grantor + grantee chains, marked as
  `actor_class=data_subject`.
- Notification: grantor + grantee receive notification within 1min (so they can update internal
  records).

## Alternatives

- Only tenant-level revocation (rejected: violates GDPR Art. 7(3) for B2C).
- Self-revoke requires grantor approval (rejected: violates "as easy as consent" requirement).
- Self-revoke through grantee only (rejected: grantee may be hostile/non-responsive; subject must
  have direct primitive).

## Consequences

- Identity µservice must support data-subject (consumer) authentication, not just tenant-principal
  authentication. (Already in place per identity µservice spec.)
- Audit-chain entries from data-subject actors are first-class.
- UI for data subjects required in ops-portal consumer surface (PHASE-02 deliverable).
- Tenant SLAs cannot block self-revocation; this is a regulatory primitive, not a contractual one.

## Verification

- Test: B2C agreement → self-revoke API → propagation within 1s → grantee read denied.
- Test: B2B agreement (data_subject_self_revocation=false) → self-revoke attempt → rejected.
- DSAR cascade includes self-revoke as a permitted erasure mechanism.
