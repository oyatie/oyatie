---
doc_class: Runbook
title: Audit export — auditor + tenant + cross-pack bundles
microservice: audit-chain
severity: Sev-3 (operational) / Sev-1 (export bundle tamper-suspect)
status: Accepted
owner_team: council-privacy + axis-audit-chain + ops-compliance
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/policy/auditor-scope.cedar
  - microservices/audit-chain/policy/data-residency.md (cross-pack export exception)
  - microservices/audit-chain/backfill-replay.md
  - microservices/audit-chain/contracts/openapi/audit-chain.yaml /export path
doc_status: published
---

# Runbook: Audit export

## Purpose

Procedures for:
- Issuing scoped export bundles to external auditors.
- Tenant-initiated tenant-controlled export.
- Cross-pack export refusal investigation.
- PII / PHI remediation export (FM-14).
- Source µservice impersonation investigation (FM-15).

## Auditor Engagement — Procedure

### Pre-engagement (≤ 5 business days)

| Step | Owner | Action |
|---|---|---|
| 1 | Auditor firm submits engagement letter | gtm-customer-success |
| 2 | council-privacy reviews scope: which tenants, which frameworks, time-range, scoped event classes | council-privacy |
| 3 | ops-security issues JIT token via OpenBao with claims: `engagement_id`, `scoped_tenants`, `scoped_packs`, `scoped_event_classes`, `audit_framework`, `valid_from`, `valid_to` (TTL ≤ 4h per session; re-issuable during engagement window) | ops-security |
| 4 | Auditor receives onboarding doc: how to authenticate; how to use verification SDK; how to interpret bundle format | gtm-customer-success |
| 5 | mTLS pinning: auditor firm's gateway certificate fingerprint registered | ops-security |

### During engagement

| Step | Owner | Action |
|---|---|---|
| 1 | Auditor invokes `POST /export` with scoped request | auditor |
| 2 | query-rest validates JIT token via Cedar (`policy/auditor-scope.cedar`) | system |
| 3 | query-rest constructs bundle: <br>  a. enumerates events matching scope; <br>  b. assembles Merkle proofs; <br>  c. resolves signing keys per period (KeyResolver); <br>  d. signs bundle metadata with current pack HSM key | system |
| 4 | Bundle written to auditor-engagement object-storage prefix; URL returned with TTL | system |
| 5 | Auditor downloads bundle | auditor |
| 6 | Every read audit-emitted (audit-of-audits per `policy/auditor-scope.cedar` §"PERMIT 1" + recording-rule) | system |
| 7 | Auditor verifies bundle offline using verification SDK | auditor |

### Post-engagement

| Step | Owner | Action |
|---|---|---|
| 1 | JIT token expires; further reads refused | system |
| 2 | Bundle retained for engagement-evidence purpose; per ops-compliance retention | ops-compliance |
| 3 | Audit-of-audits log archived | ops-security |

## Tenant Self-Service Export — Procedure

Tenant operator invokes `POST /export` from SDK or UI. Same flow but:
- Cedar scope: `tenant-scope.cedar` (limited to own tenant).
- Optional: tenant supplies `receiving_bucket_attestation` per `policy/data-residency.md` cross-border export rules.
- Bundle delivered to tenant-controlled bucket (when attestation present) OR tenant-engagement object-storage prefix (when not).

## Cross-Pack Export Refusal Investigation — Procedure

### Trigger

`oya_audit_chain_cross_pack_emission_rejected_total > 0` (FM-09).

### Procedure

| Step | Action |
|---|---|
| 1 | Verify the offending emission attempt: source SPIFFE identity + claimed pack + receiving pack |
| 2 | Engage offending workload µservice owner |
| 3 | Most common root cause: OTel collector or workload config bug routes the wrong pack. Engage axis-observability to verify collector config |
| 4 | If pattern repeats: engage ops-security (intentional bypass suspect) |
| 5 | If EU/KR-pinned data was rerouted: engage council-privacy for breach-notification chain (GDPR 72h, KR PIPA 72h) per `compliance.md` |
| 6 | Routing corrected; verify with synthetic emission |

### Verification

- `oya_audit_chain_cross_pack_emission_rejected_total` rate returns to 0.
- Origin source µservice owner attests corrected config.
- If breach: regulatory notification confirmed delivered.

## PII / PHI Remediation Export (FM-14) — Procedure

### Trigger

Synthetic-PII detector flags unredacted PII / PHI in audit-chain payload.

### Procedure

| Step | Action |
|---|---|
| 1 | Declare Sev-2 (Sev-1 if pack-us-healthcare PHI in production) |
| 2 | Engage council-privacy + offending source µservice owner |
| 3 | Identify affected event range: query Postgres + S3 for events from offending source µservice with matching pattern |
| 4 | Patch the source µservice's redactor (engineering-discipline floor; can't fully prevent — DPIA R-01) |
| 5 | Trigger DSR cascade for affected records: `cargo run -p oya-dev-cli -- audit-chain dsr-cascade-emergency --pack <pack> --tenant <tenant> --event-range <from-to> --reason "pii-remediation" --approver-1 <p1> --approver-2 <p2>` (2-person rule) |
| 6 | Tenant notification per `incident-response.md` |
| 7 | If pack-eu / pack-kr / pack-us-healthcare affected: regulatory notification per `compliance.md` |
| 8 | Postmortem; harden source µservice's redactor |

## Source µservice Impersonation Investigation (FM-15) — Procedure

### Trigger

`oya_audit_chain_tenant_spoofing_attempt_total > 0`.

### Procedure

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage IC (ops-security) + axis-audit-chain SME |
| 2 | Identify the spoofing SPIFFE identity from `oya_audit_chain_tenant_spoofing_attempt_total` labels |
| 3 | Look up the SPIFFE identity in OpenBao: which workload µservice, which pod, which deploy time |
| 4 | Pattern A — legitimate workload with bug: workload owner engaged; bug fixed |
| 5 | Pattern B — compromised credential: revoke SPIFFE identity in OpenBao; investigate compromise source (typical: leaked credential via logs / env vars); rotate all credentials in the affected µservice |
| 6 | Pattern C — insider threat: engage ExecSponsor + council-architecture; investigate via OpenBao audit log + Kubernetes admission log |
| 7 | Tenant notification per Sev-1 template (potential audit-trail integrity question) |
| 8 | Forensic preservation of attempted emissions (rejected, but record-of-attempt useful) |

### Verification

- Spoofing rate returns to 0.
- Implicated credentials rotated.
- Postmortem identifies systemic vs isolated root cause.

## References

- `microservices/audit-chain/policy/auditor-scope.cedar`.
- `microservices/audit-chain/policy/data-residency.md` §"Cross-pack export".
- `microservices/audit-chain/policy/tenant-scope.cedar`.
- `microservices/audit-chain/backfill-replay.md`.
- `microservices/audit-chain/failure-modes.md` FM-09 + FM-14 + FM-15.
- `microservices/audit-chain/incident-response.md`.
- `microservices/audit-chain/dpia.md` R-01 (PII leakage) + R-05 (auditor exfiltration).
