# C6 — Exportable sealing-root custody design (feeds Group-F1 / ADR-0624)

Design-only (read-only). Verified on origin/dev `b1c2c2a7`. Founder decision needed on destination (A/B/C); S0+S1 ship NOW regardless.

## Defect (adapter only — kernel is sound)
`secrets/adapters/kms-openbao/src/root_custody.rs`: `:128` root born `exportable=true`; `:139-148` standing full-root plaintext export endpoint (`/export/encryption-key/`); `:181` always stamps `OpenBaoTransitionalSingleCustodian`. Root unseals every tenant KEK→DEK→secret in the cell ⇒ single OpenBao token = total cell compromise. Kernel enclave (`secrets/core/kms-enclave`) is strong: mlock+zeroize, one-way `from_key_bytes` ingress, typed provenance.

## Two secondary findings (must be in ADR-0624)
1. **Inert backstop:** `provenance.rs:54` `satisfies_quorum_doctrine()` has ZERO non-test callers; operator boot (`kms-operator-app/src/main.rs`) never checks provenance. The "boot refuses on posture" doc claim is inert at runtime. → **S0 wires it = cheapest highest-leverage fix, NOT founder-gated.**
2. **`exportable=true` invisible to all 38 gates** (false-green class, same as AUTH-005/#768). → **S1 make-impossible gate.**

## Options (kernel one-way door unchanged in all)
- **A HSM/PKCS#11 non-exportable:** root bytes NEVER materialize; but INVERTS wrap topology (wrap/unwrap→HSM ops, touches all callers + dek_cache); FIPS/regulated-pack grade; vendor transitional (cryptoki/SoftHSM→CloudHSM, IP-011). Effort HIGH.
- **B Shamir M-of-N:** owned-Rust constant-time SSS; ZERO kernel-topology change (reuses `from_key_bytes`, only byte-assembly differs); retires SingleCustodian directly; but root briefly materializes in mlock'd memory at unseal; not FIPS-partition. Effort MEDIUM. Kernel already models `ShamirQuorumCeremony{M,N}`.
- **C Hybrid (RECOMMENDED):** HSM steady-state + Shamir break-glass DR; B's SSS work reused as C's break-glass (no thrown-away work); best DR. Effort HIGHEST but convergent.

## Recommendation
C as destination, sequenced **B-first as the transitional fix**, **S0+S1 immediately** (independent of A/B/C). S0+S1+S3+S4 = full CRITICAL remediation; S5/S6 = regulated-pack escalation gated on the A-vs-C choice.

## Make-impossible gate (S1, model on crypto-backend-purity)
`oya-cloud-ci-sealing-root-custody-app` + `sealing-root-custody-policy.json`; pure `evaluate_keyed`; TWO signals (both required, gate-baseline-asymmetry trap): (1) SOURCE — no prod custody ctor emits `exportable=true` / `/export/encryption-key/`; (2) POSTURE (load-bearing) — every prod `KmsSealingRoot` CRD declares `hsm-non-exportable` or `shamir-quorum{M>=2}`, `single-custodian` forbidden in prod. Codes: SRC-EXPORTABLE-ROOT-CONSTRUCTED / SRC-SINGLE-CUSTODIAN-IN-PROD / SRC-EXPORT-COMMAND-PRESENT / SRC-POLICY-MALFORMED / SRC-EMPTY-SCAN. 7-property bar.

## Existing-root rotate-then-shred (zero downtime)
Re-root = KEK-layer re-wrap only (DEKs/payloads untouched, AWS-KMS envelope pattern). Dual-root window: provision root_v2 (non-exportable/quorum) → boot both, new writes under v2, reads fall back v1 (root-id binding `material.rs:144`) → idempotent re-wrap sweep v1→v2 (operator-driven via `KmsSealingRoot` CRD activeVersion/observedVersion) → drain+verify zero-under-v1 → crypto-shred v1 (reuse `shred.rs` ScheduledKeyDeletion+quorum) + revoke export policy. CRD `health.state` drives Ambiguous→Healthy.

## Strangler slices
- **S0** boot fail-closed in prod on SingleCustodian/exportable (wire `satisfies_quorum_doctrine`) — immediate, smallest diff
- **S1** make-impossible CI gate — frozen-empty after S0
- **S2** owned `SealingRootCustody` port (wrap/unwrap/provenance, W5-shaped)
- **S3** owned Rust Shamir SSS + quorum custody adapter (Option B transitional fix; retires SingleCustodian)
- **S4** rotate-then-shred migration job (eliminates EXISTING exportable roots)
- **S5** HSM/PKCS#11 adapter (Option A destination, IP-011) — inverts wrap topology
- **S6** Hybrid break-glass (Option C convergence)

## FOUNDER DECISION (not blocking S0/S1; gates S5/S6)
A (HSM-only, FIPS, heaviest) vs **B (Shamir-only, owned/hermetic, fastest, root briefly in memory)** vs **C (Hybrid, recommended)**. Secondary: is `single-custodian` provenance OK in non-prod behind S0 prod-refusal, or delete from prod-constructible code entirely at W5 (kernel plans test-only demote, `provenance.rs:30`)?

## Sequencing
S0/S1 are face-touching PRs → land AFTER the keystone (de-commit scm-facts, #141) to avoid the cascade. S0+S1 = Wave-3 Group-D/F lane once keystone lands.
