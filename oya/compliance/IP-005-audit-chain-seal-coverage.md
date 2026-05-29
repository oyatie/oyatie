---
microservice: compliance
ip: IP-005
title: Audit-chain seal coverage (every evidence artifact carries a verifiable seal)
status: Drafting
authority_tier: 3
owner: axis-security
co_owners: [axis-compliance]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0181, ADR-0209]
---

# IP-005 — Audit-chain seal coverage

## Purpose

Wire the audit-chain seal hex (SHA-256 + cosign keyless OIDC chain) into every evidence artifact emitted by the compliance pipeline. Drive the `oya-check-audit-chain-seal-coverage` gate to fail-closed when any artifact ships without a verifiable seal.

## Acceptance criteria

1. Every emitted artifact carries a 64-hex-char `audit_chain_seal_hex` field.
2. Cosign keyless OIDC chain verifier integrated; verification path runs on every auditor portal read.
3. Seal verification failure emits `EVT-AUDIT-SEAL-VERIFY-FAILED` (Sev-1).
4. Coverage gate `oya-check-audit-chain-seal-coverage` integration validated.
5. Cosign keyless trust root pinned via `policy/cosign-trust-root.json` (per ADR-0181).
6. ≥ 5 integration tests: seal-emit + seal-verify-pass + seal-verify-fail + missing-seal-reject + chain-break-Sev-1.

## Seal lifecycle

```
[collector emits artifact] → sha256(content || nonce) = seal_hex
                          → cosign sign-blob --identity-token <OIDC> seal_hex
                          → seal_hex + cosign signature stored alongside artifact in SeaweedFS
                          → auditor reads artifact → cosign verify-blob
                          → success: 200 + verified flag; failure: 500 + Sev-1
```

## Cosign keyless OIDC chain

Per ADR-0181 image-promotion uses cosign keyless OIDC. Same trust root + same chain verifier for evidence artifacts:

- **OIDC issuer:** Sigstore Fulcio (default) OR operator-cluster-bound OIDC issuer.
- **Trust root:** pinned at `policy/cosign-trust-root.json`; quarterly rotation.
- **Rekor log:** transparency log; auditor can prove non-revocation.

## EVT-AUDIT-SEAL-VERIFY-FAILED

Sev-1 incident. Triggers:

1. AlertManager → PagerDuty (on-call security).
2. Auditor portal renders a banner: "Seal verification failed for artifact <id>. Investigation in progress."
3. Artifact remains visible (for forensics) but flagged.
4. Investigation runbook at `runbooks/audit-seal-verify-failure.md`.

## Risk + mitigation

- **Risk:** OIDC issuer compromise. **Mitigation:** sigstore trust root quarterly rotation; multi-issuer fallback (Sigstore + operator OIDC).
- **Risk:** seal pre-computation attack (attacker pre-seals adversarial content). **Mitigation:** seal includes a server-side nonce + emit timestamp.
- **Risk:** Rekor log unavailability. **Mitigation:** operator-cluster mirror of Rekor log (per ADR-0164 sovereign cloud).

## Acceptance evidence

`evidence/ip-005-audit-chain-seal-coverage-acceptance.json`.

## Cross-references

- ADR-0145 — audit-chain seal substrate.
- ADR-0181 — container image promotion pipeline (cosign keyless OIDC).
- ADR-0209 — substrate authority.
- Bominal ADR-0028 — audit chain (inherited).
- `oya-check-audit-chain-seal-coverage` — enforcement gate.
