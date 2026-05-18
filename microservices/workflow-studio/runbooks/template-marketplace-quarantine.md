---
doc_class: Runbook
title: Template marketplace quarantine (signed node library + template)
microservice: workflow-studio
severity: "Sev-1 (active malicious library) / Sev-2 (signature anomaly) / Sev-3 (rev/validation drift)"
status: Accepted
owner_team: ops-security + axis-workflow
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/threat-model.md §"T-S-04" + §"T-T-03" + §"T-E-04"
  - microservices/workflow-studio/PRD.md FR-11 + §"Security" + §"Bounded Contexts" node-library-registry
  - /specs/microservices/workflow-studio.json §anti_patterns non_deterministic_node_library_load
  - microservices/workflow-studio/policy/auditor-scope.cedar (revocation event audit)
doc_status: published
---

# Runbook: Template / node-library quarantine

## Purpose

Studio's `node-library-registry` BC distributes per-pack signed node libraries (FR-11) and (subsequent-to-GA-tier-promotion) tenant-authored template marketplace bundles. A compromised or malicious library is a supply-chain attack surface — this runbook contains it.

## Trigger

ONE of:

1. **Signature verification fails** at any Studio editor open (`oya_workflow_studio_node_library_signature_verification_failed_total > 0`).
2. **Per-pack signing key rotation event** discovers an unsanctioned library published under the prior key.
3. **`oya-governance-node-library-signature-verification` LEAN lane fails on a PR.**
4. **Tenant report**: "a node from library X executes unexpected behavior" — investigate per Path B.
5. **3x-reload determinism check fails** (`oya-governance-node-library-determinism`) — non-determinism could be a hot-swap supply-chain attack.
6. **Threat intelligence** (Trivy / Grype / OSV-Scanner) reports CVE on a library dependency.

## Severity

- Active malicious code in published library, tenants loading it: **Sev-1**.
- Signature anomaly OR verification failure but no tenant-impact yet: **Sev-2**.
- Determinism drift / revocation drift: **Sev-3** (signal of weakened control).

## Impact

- Supply-chain risk: Studio renders node configurations from library descriptors; while descriptors are declarative (not executable), a crafted descriptor could still trigger XSS (T-I-02) OR display misleading data-class markers (deceiving tenant author at FR-16).
- Tenant trust: load-bearing on signed library distribution (FR-11 + threat-model T-S-04).
- Audit-chain: every library publish + revocation event Ed25519-sealed; tampering detectable.

## Pre-checks

1. Identify suspect library: `pack`, `library_name`, `version_sha`, `signature_key_id` from alert.
2. Identify tenants who loaded it: `SELECT tenant_id, COUNT(*) FROM library_load_audit WHERE library_sha = <sha> AND loaded_at > NOW() - INTERVAL '24h' GROUP BY tenant_id`.
3. Verify signature lineage: `cargo run -p oya-workflow-studio-node-library-registry-domain --bin verify-signature -- --library <sha> --key-id <kid>`.
4. Check OpenBao key store: `cargo run -p oya-dev-cli -- openbao audit list --path 'secret/workflow-studio/node-library-signing/*'` to confirm key wasn't rotated outside expected schedule.

## Recovery Path A — Active malicious library (Sev-1)

Cause: confirmed malicious code in published library; tenants are loading it.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security IC + axis-workflow + council-privacy. | ≤ 5 min |
| 2 | **Immediate revoke**: `cargo run -p oya-dev-cli -- workflow-studio library revoke --library <sha> --reason "<rfc>" --severity sev-1`. Revocation propagates via CDN purge + Studio session refresh signal. | ≤ 10 min |
| 3 | Studio refuses to load the revoked library on next-open AND signals to active sessions to discard cached descriptors. | ≤ 60s p99 |
| 4 | Verify zero load attempts post-revoke: `rate(oya_workflow_studio_node_library_loaded_total{library_sha="<sha>"}[5m]) == 0` for ≥ 5 min. | ≤ 10 min |
| 5 | Identify scope: how was the malicious library published? Compromised signing key? Compromised publisher account? Engage ops-security forensics. | ≤ 1h |
| 6 | **Rotate the publisher signing key** via OpenBao if compromise confirmed. Per `threat-model.md` §"T-S-04" rotation 90d default; emergency rotation immediate. | ≤ 30 min |
| 7 | **Tenant breach notification per pack**: any tenant who loaded the library is potentially impacted. <br> - pack-kr: PIPA Art. 34 (72h) <br> - pack-eu: GDPR Art. 33 (72h to DPA) + Art. 34 (without undue delay to data subjects if high risk) <br> - pack-us-healthcare: HIPAA §164.404 (60 days max) + §164.408 (HHS notification) <br> - pack-jp: APPI 漏えい等通知 <br> - pack-sg: PDPA Part VIA <br> - pack-au: Notifiable Data Breaches scheme (30 days) <br> - pack-in: DPDPA 2023 §8(6) <br> - pack-br: LGPD Art. 48 <br> - pack-ae/ksa: per local DPA. | per pack |
| 8 | Postmortem within 5 business days. | – |

## Recovery Path B — Suspected malicious behavior (Sev-2 → potential Sev-1)

Cause: tenant reports unexpected library behavior; not yet confirmed malicious.

| Step | Action |
|---|---|
| 1 | Engage ops-security + axis-workflow; declare Sev-2 pending investigation. |
| 2 | **Soft-quarantine**: `cargo run -p oya-dev-cli -- workflow-studio library quarantine --library <sha> --soft`. Tenants see "library temporarily under review" banner; can opt to continue with explicit acknowledgment (2-person rule on tenant side via tenancy SDK). |
| 3 | Replicate the reported behavior in a sandbox tenant. |
| 4 | If confirmed malicious: escalate to Path A (Sev-1 hard revoke). |
| 5 | If false alarm: lift quarantine; explain to reporting tenant. |

## Recovery Path C — Signature verification failure on PR

Cause: `oya-governance-node-library-signature-verification` LEAN lane reports a library descriptor's signature does not verify against the per-pack signing key.

| Step | Action |
|---|---|
| 1 | Verify the descriptor's `pack` matches the signing key used. |
| 2 | If wrong pack key used by mistake: re-sign with the correct key; resubmit PR. |
| 3 | If key not in allowed-publisher set: check `microservices/workflow-studio/iac/terraform/node-library-publishers.tf` — only listed publishers can sign per-pack. |
| 4 | If suspicious: escalate to Path A. |

## Recovery Path D — Determinism drift (3x re-load returns different descriptors)

Cause: `oya-governance-node-library-determinism` fails — same library version produces non-byte-identical descriptors across 3 loads.

| Step | Action |
|---|---|
| 1 | Validate determinism check: `cargo nextest run -p oya-workflow-studio-node-library-registry-domain --test test_load_determinism` locally. |
| 2 | If fails: a CDN edge OR object-storage layer is mutating bytes (highly suspicious — potential MITM OR cache-pollution). |
| 3 | Engage ops-security + cloud-iac. |
| 4 | Verify SRI hash of served library against expected; if mismatch: Path A (Sev-1 supply-chain). |

## Recovery Path E — Threat-intel CVE on library dependency

Cause: Trivy / Grype / OSV-Scanner reports a CVE in a transitive dep of a library descriptor (descriptors reference Ontology types + capability descriptors; these chains can carry CVEs).

| Step | Action |
|---|---|
| 1 | Verify CVE applicability: does the affected dep path actually execute in Studio's render context? (Descriptors are declarative; many deps are dev-only). |
| 2 | If applicable: re-publish library with patched dep; revoke old version. |
| 3 | If not applicable: file an exception in `cargo deny` config with documented rationale + sunset date. |

## Verification

After recovery:
- Revoked library: zero load attempts (`rate(... == 0)` for ≥ 5 min).
- Allowed-publisher set: matches expected (no unauthorized publisher entries).
- Signing key rotation completed (if applicable); new key in OpenBao + audit-logged.
- Tenant notifications complete per regulatory timeline (Sev-1 path).
- Audit-chain seal: revocation event sealed; quarantine event sealed.
- Studio editor open works for non-affected libraries.

## Post-incident updates

- Postmortem within 5 business days (or per regulatory minimum if Sev-1).
- Document the attack vector in `threat-model.md`.
- If publisher account compromised: tighten publisher-account access controls (FIDO2 mandatory, 2-person rule on key access via OpenBao JIT).
- If CVE chain new: extend `oya-governance-node-library-dep-scan` lane.
- Action item: review revocation propagation SLI — was 60s p99 met?

## References

- `microservices/workflow-studio/threat-model.md` T-S-04, T-T-03, T-E-04.
- `microservices/workflow-studio/PRD.md` FR-11, §"Security" — node library supply-chain.
- `/specs/microservices/workflow-studio.json` §anti_patterns non_deterministic_node_library_load.
- OWASP Top 10 A06 (Vulnerable Components).
- NIST SP 800-218 SSDF — software supply-chain practices.
- SLSA Level 3 spec — `slsa.dev/spec/v1.0/levels`.
- in-toto attestation framework — `in-toto.io`.
