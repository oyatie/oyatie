---
doc_id: finops-portal/compliance-matrix
authored: 2026-05-18
status: ready
authority: per-pack regulatory frameworks
classification: internal
---

# Compliance matrix — finops-portal

Per-pack regulatory obligations + the evidence finops-portal emits
to satisfy them.

## KR pack — PIPA + FSS guidance

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| PIPA Art. 28 personal-data inventory      | quarterly emit `pii_redaction_applied: true`       | quarterly  |
| FSS quarterly cost report                 | `FinOpsQuarterlyReport` (KR overlay)               | quarterly  |
| FSS audit-log retention 5y                | audit-chain retention policy (ADR-0162)            | continuous |
| Data residency in KR                      | helm `values-kr.yaml` nodeSelector + Cedar policy  | per-deploy |
| KR FSS evidence portal upload             | manual upload of the sealed envelope               | quarterly  |

## EU pack — GDPR

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| Article 6 lawful basis                    | `dpia.md` §2                                        | annual     |
| Article 30 records of processing          | quarterly emit `gdpr_article_30_record`            | quarterly  |
| Article 32 security measures              | `threat-model.md` + audit-chain seals              | continuous |
| Article 33 breach notification (72h)      | `runbooks/quarterly-regulator-emit-miss.md` §Path B| ad-hoc     |
| Article 35 DPIA                           | `dpia.md`                                          | annual     |
| Right of access / portability             | tenant-invoice public API + FOCUS export           | continuous |
| Cross-border transfer controls            | helm `values-eu.yaml` networkPolicy + Cedar        | per-deploy |

## US-healthcare pack — HIPAA

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| §164.312 access controls                  | Cedar `tenant-isolation.cedar` PHI clauses         | continuous |
| §164.312 audit controls                   | audit-chain seal classes                           | continuous |
| §164.312 integrity                        | append-only credit-ledger + Ed25519 envelope sig    | continuous |
| §164.312 transmission security            | encrypted-hipaa storageClass + TLS                 | continuous |
| §164.404 breach notification              | `runbooks/quarterly-regulator-emit-miss.md` §Path D| ad-hoc     |
| BAA requirements                          | helm `values-us-healthcare.yaml` toggles           | per-deploy |
| Min necessary (per-line redaction)        | `features.phiRedaction: true`                      | continuous |

## US-financial pack — SOX

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| ICFR control attestation                  | quarterly emit `control_attestation` block         | quarterly  |
| Segregation of duties (policy reviewers)  | IP-010 2-reviewer quorum                           | per-promote|
| Audit trail completeness                  | audit-chain seal classes                           | continuous |
| Change management                         | git audit log + PR review                          | continuous |

## US-public-sector pack — FedRAMP moderate

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| AC-2 account management                   | Cedar policies + audit-chain                       | continuous |
| AU-2 audit events                         | 5 seal_events declared in manifest                 | continuous |
| AU-9 protection of audit info             | Ed25519 + HSM                                       | continuous |
| CM-3 configuration change control          | helm rollout + IP-010 quorum                       | per-deploy |
| SC-7 boundary protection                  | NetworkPolicy + per-region pin                     | continuous |

## Generic pack — internal controls

| Obligation                                | Evidence emitter                                   | Cadence    |
|-------------------------------------------|----------------------------------------------------|------------|
| Quarterly internal audit                  | `FinOpsQuarterlyReport`                            | quarterly  |
| Chargeback fairness                       | IP-009 / IP-010 + ADR-0174                         | continuous |
| Anomaly response                          | `runbooks/tenant-cost-anomaly-spike.md`            | ad-hoc     |
| Credit ledger reconciliation              | `runbooks/credit-application-reconciliation.md`    | weekly     |

## Signing key + envelope verification

Every quarterly envelope carries:

- `signed_by`: Ed25519 public key fingerprint.
- `signature`: detached signature over the envelope's canonical
  serialization.
- `key_published_at`: timestamp the public key was sealed to
  audit-chain under class `FinOpsQuarterlyKeyPublished`.

Verifiers (regulators, auditors) follow this procedure:

1. Fetch the audit-chain `FinOpsQuarterlyKeyPublished` event for
   the quarter; extract the public key.
2. Canonicalize the envelope (sort fields, UTF-8 NFC).
3. Verify the Ed25519 signature.
4. Confirm `key_published_at <= envelope.emitted_at`.

## References

- ADR-0162 audit-log slicing + signing.
- ADR-0174 chargeback formula.
- ADR-0199 FinOps canonical.
- `threat-model.md`.
- `dpia.md`.
- IP-015 quarterly emit.
