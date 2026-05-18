---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Blind-Signature Key Ceremony
microservice: anonymous
severity: "Sev-1 (always; ceremony is a P0 cryptographic event)"
status: Accepted
owner_team: ops-security + axis-anonymous + council-architecture
date: 2026-05-17
related_adrs: [ADR-ANON-0001]
related_artifacts:
  - microservices/anonymous/policy/affinity-attestation-verification.md
  - microservices/anonymous/decisions/ADR-ANON-0001-cryptographic-blinding-protocol.md
doc_status: published
---

# Runbook: Blind-Signature Key Ceremony

## Purpose

Generate or rotate the **blind-signature signing-key set** that anchors PRD invariant I1. The ceremony produces the per-pack signing key under which BBS+ / Camenisch-Lysyanskaya credentials are issued; loss / compromise of this key invalidates ALL bindings.

## Trigger

| Scenario | Frequency | Severity |
|---|---|---|
| New pack onboarding (e.g., pack-ae spinning up) | per-pack onboarding | Sev-1 |
| Planned 24-month rotation per ADR-ANON-0001 rotation policy | per-pack annual | Sev-1 |
| Compromise (private-key leakage suspected) | on-detection | Sev-1 (immediate) |
| FIPS 140-3 module re-certification (rare) | per re-cert | Sev-1 |

## Roles required (NEVER fewer than 5; two-person-rule + auditor witness)

| Role | Responsibility | Min count |
|---|---|---|
| Ceremony Master | Drives the ceremony; reads steps aloud | 1 |
| Quorum-Holder A | Holds 1 of N Shamir secret-share fragments | 1 |
| Quorum-Holder B | Holds another fragment | 1 |
| Quorum-Holder C | Holds another fragment | 1 |
| Auditor Witness | Independent observer; records the ceremony; signs the post-ceremony attestation | 1 |
| Security Officer (recording) | Captures audio + screen recording; commits to tamper-evident storage | 1 (may overlap with Auditor Witness) |

Total minimum: **5 distinct human principals**, each from a distinct organisational unit per the two-person-rule + auditor-witness industry pattern (HSM ceremony best practice; CA root-key ceremony precedent).

## Pre-checks

1. Confirm the air-gapped HSM (FIPS 140-3 Level 3) is available and freshly tested.
2. Confirm Shamir share count parameters: `n = 5` (total shares), `k = 3` (threshold to reconstruct).
3. Confirm all roles are present in person at the secure facility.
4. Confirm no network connectivity to the HSM (air-gapped).
5. Confirm all-other-cause maintenance windows have been postponed.
6. Confirm `cargo run -p oya-dev-cli -- anonymous blind-signature ceremony pre-check` returns green.

## Steps

| # | Step | Performed by | Time |
|---|---|---|---|
| 1 | Ceremony Master reads ceremony script aloud; all participants confirm they understand the steps | Ceremony Master | 15 min |
| 2 | Auditor Witness starts audio + screen recording | Auditor Witness + Security Officer | 1 min |
| 3 | On the air-gapped HSM, generate a new BBS+ key pair (curve BLS12-381) using `oya-anonymous-blind-signatures-cli ceremony generate --pack <pack>` | Ceremony Master | 5 min |
| 4 | Compute the public-key fingerprint; the participants each independently verify the fingerprint matches what the HSM displays | Ceremony Master + Quorum-Holders | 5 min |
| 5 | Split the private key into 5 Shamir secret-shares (k=3 threshold) using `oya-anonymous-blind-signatures-cli ceremony shamir-split --shares 5 --threshold 3` | Ceremony Master | 2 min |
| 6 | Each Quorum-Holder receives their share on a single-use, write-once medium (sealed envelope with tamper-evident seal); each Quorum-Holder verifies their share fingerprint | Quorum-Holders | 10 min |
| 7 | Public key is exported from the HSM in a single transfer; transferred via USB to an online machine that publishes it to the platform via `cargo run -p oya-dev-cli -- anonymous blind-signature publish-public-key --pack <pack> --key-bytes <bytes>` | Security Officer | 5 min |
| 8 | Audit-chain seals a `BlindSignatureKeyCeremonyExecuted` event with the public-key fingerprint, the ceremony participants (hashed), and the auditor's signature | Ceremony Master + Auditor | 2 min |
| 9 | Auditor Witness signs the post-ceremony attestation (PDF + audit-chain seal) | Auditor Witness | 5 min |
| 10 | All recording media (audio, screen, tamper-evident envelopes) are sealed in a secure storage container | Security Officer | 10 min |
| 11 | Ceremony Master pronounces the ceremony complete; all participants confirm | All | 1 min |
| 12 | Post-ceremony: ceremony script + signed attestation + audit-chain seal hash are committed to `evidence/key-ceremonies/<ceremony-id>/` | Security Officer | 30 min |

**Total ceremony duration: ~90 minutes (target).**

## Post-ceremony attestation

The signed attestation includes:

```yaml
ceremony_id: <uuid>
pack: <pack-id>
date: 2026-05-17T00:00:00Z
ceremony_master: <hashed-principal-id>
quorum_holders: [<hashed>, <hashed>, <hashed>]
auditor_witness: <hashed-principal-id>
public_key_fingerprint: <sha256>
shamir_threshold: 3-of-5
hsm_serial: <hsm-id>
hsm_fips_level: 3
recording_storage_location: <vault-id>
audit_chain_seal_hash: <merkle-root>
auditor_signature: <ed25519-signature>
```

## Reconstitution protocol (for rotation or emergency)

To use the private key (issuing credentials), 3 of 5 Shamir shares must be reconstituted in the air-gapped HSM. The reconstitution ceremony follows the same role structure as generation (5 distinct principals + auditor) and produces an audit-chain `BlindSignatureKeyReconstituted` event with the same attestation pattern.

## Failure modes

| Failure | Mitigation | Severity escalation |
|---|---|---|
| HSM fails during ceremony | abort; re-test HSM; reschedule | Sev-1 |
| Auditor Witness is unavailable | abort; reschedule (witness is non-negotiable) | Sev-2 → Sev-1 if blocking |
| Shamir share transfer fails | abort; new ceremony required | Sev-1 |
| Public-key fingerprint mismatch | abort; investigate HSM tamper; full forensic | Sev-1 (immediate; compromise suspected) |
| Audit-chain seal fails | abort; investigate audit-chain incident; restart ceremony | Sev-1 |

## References

- ADR-ANON-0001 (cryptographic-blinding protocol)
- NIST SP 800-57 Part 1 (key management — recommended ceremony pattern)
- FIPS 140-3 (Federal Information Processing Standard for cryptographic modules)
- CA/Browser Forum Baseline Requirements (root-key ceremony precedent)
- Shamir's Secret Sharing (Shamir 1979)
