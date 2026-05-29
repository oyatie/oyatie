---
microservice: compliance
doc: ThreatModel
status: Drafting
authority_tier: 2
owner: axis-security
co_owners: [axis-compliance, council-architecture]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0183, ADR-0209]
---

# Compliance — Threat Model

## STRIDE per-asset

### Asset A1 — Audit-chain seal hex

Tamper-evident hash linking every artifact to the audit chain. Compromise here invalidates the auditor's ability to verify integrity.

| Threat | Mitigation |
|---|---|
| **Spoofing** — adversary emits an artifact with a fake seal | Cosign keyless OIDC (per ADR-0181); seal verification chain on every auditor portal read |
| **Tampering** — adversary modifies an artifact post-seal | SeaweedFS WORM (write-once-read-many) tier for hot evidence; cold tier re-seals on archive |
| **Repudiation** — emitter denies emitting an artifact | Cosign keyless OIDC links emission to the workload's SPIFFE-ID (per ADR-0148) |
| **Info disclosure** — seal hex leaks subject identity | Seal hex is SHA-256 of artifact content + nonce; no subject identity material |
| **DoS** — adversary floods seal verification endpoint | Rate limit; circuit-break; HPA on Backstage auditor portal |
| **Elevation** — adversary uses seal to assume auditor identity | Cedar per-engagement role binding; auditor identity is OIDC-bound per engagement |

### Asset A2 — DSAR request (GDPR Art. 12)

Subject's data subject access request. Compromise leads to either (i) cross-tenant data leak (catastrophic), (ii) wrong-subject disclosure (catastrophic), (iii) statutory SLA breach (regulatory fine).

| Threat | Mitigation |
|---|---|
| **Spoofing** — adversary opens DSAR pretending to be the subject | Zitadel passwordless auth + email-link verification |
| **Tampering** — adversary modifies DSAR request mid-flight | TLS 1.3 + signed request envelope |
| **Repudiation** — subject denies opening DSAR | Audit-chain seal on EVT-DSAR-REQUEST-OPENED |
| **Info disclosure (cross-tenant)** | **Kernel-level `tenant_id` invariant** (`oya-shared-compliance-evidence-kernel::coverage_gaps`); Ontology projection traversal carries `tenant_id` at every step; assembly rejects mismatch |
| **Info disclosure (wrong subject)** | Subject identity verified via Zitadel passwordless; Ontology projection only matches subjects with matching pseudonym |
| **DoS** — adversary floods DSAR endpoint | Per-tenant rate limit (10 DSARs / tenant / day); circuit-break at backlog > 100 |
| **Elevation** — adversary chains DSAR to admin escalation | DSAR API has no admin surface; export-only / delete-only / rectify-only paths |

### Asset A3 — Auditor portal access

Read-only access for external auditors during an engagement. Compromise = unauthorized auditor reads or post-engagement access creep.

| Threat | Mitigation |
|---|---|
| **Spoofing** — adversary impersonates auditor | Per-engagement OIDC identity; passwordless |
| **Repudiation** — auditor denies reading a specific artifact | EVT-AUDITOR-ARTIFACT-VIEWED in audit chain |
| **Info disclosure** — auditor reads other tenant's artifacts | Cedar per-engagement policy scopes to the engagement's tenant set |
| **Elevation** — auditor escalates to write | Read-only Cedar role binding; auditor-portal endpoints are GET-only |
| **Access creep post-engagement** | Engagement-end webhook revokes Cedar role binding; integration test asserts revoke |

### Asset A4 — Minimum-necessary access log (HIPAA)

Per-PHI-access log used for HIPAA §164.514(d) minimum-necessary audit.

| Threat | Mitigation |
|---|---|
| **Tampering** — accessor or admin modifies log to hide unauthorized access | Append-only JSONL + cosign seal per entry |
| **Info disclosure** — log content leaks PHI | Log records subject pseudonym (NOT name); access purpose; Cedar decision |
| **DoS** — log volume overwhelms storage | Continuous compaction to cold tier; retention 6 years (HIPAA statutory) |

### Asset A5 — Manual evidence upload (pen-test, BAA inventory)

Authenticated human uploads of evidence that can't be auto-collected.

| Threat | Mitigation |
|---|---|
| **Spoofing** — adversary uploads fake pen-test report | Uploader identity bound to SPIFFE-ID + Cedar `compliance:admin` role; pen-test report PDF SHA-256 in audit chain |
| **Tampering** — adversary modifies uploaded report later | Cosign keyless OIDC seal at upload; SeaweedFS WORM |
| **Info disclosure** — uploaded pen-test reveals exploitable vuln to non-auditor | Cedar policy: pen-test artifacts visible only to `compliance:admin` + `auditor:engagement-X` |

## Cross-tenant isolation invariant (highest priority)

Most catastrophic failure mode. Belt-and-suspenders:

1. **Kernel invariant** — `oya-shared-compliance-evidence-kernel::coverage_gaps` filters by `tenant_id` before evaluating coverage.
2. **Domain invariant** — DSAR aggregation walks the Ontology projection (per ADR-0145) with `tenant_id` predicate at every step.
3. **API invariant** — REST handler asserts `request.tenant_id == subject.tenant_id`; rejects otherwise.
4. **Cedar invariant** — `dsar.exec` capability requires `principal.tenant_id == resource.tenant_id`.
5. **Integration test** — `tests/cross_tenant_dsar.rs` builds two tenants with overlapping subject pseudonyms; asserts each DSAR returns only matching-tenant data.
6. **Audit-chain test** — every DSAR artifact carries `tenant_id`; coverage gate rejects cross-tenant artifact emission.

## Trust boundaries

- **External auditor → auditor portal** (Backstage; Zitadel OIDC + Cedar policy).
- **External subject → DSAR endpoint** (Zitadel passwordless; rate-limited).
- **Internal collector → evidence-collector tier** (SPIFFE-ID + Cedar `compliance:emit`).
- **Internal collector → SeaweedFS** (mTLS via service mesh per ADR-0148).
- **External tenant admin → tenancy admin UI** (separate µservice; this µservice consumes tenancy events).

## Residual risks

- Audit-chain seal source compromise (downstream of ADR-0145; not within this µservice's mitigation surface).
- Cosign keyless OIDC issuer compromise (industry-wide risk; tracked in cross-cutting threat model).

## References

- ADR-0145 — audit-chain seal.
- ADR-0148 — service mesh (mTLS + SPIFFE-ID).
- ADR-0183 — Cedar policy.
- ADR-0209 — compliance evidence automation.
