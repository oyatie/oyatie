---
id: ADR-NET-0005
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, ops-security, axis-network, axis-audit-chain, council-privacy
owner: axis-network + axis-audit-chain
supersedes: []
superseded_by: []
related:
  - ADR-0028
  - ADR-0135
  - ADR-0131
  - ADR-NET-0001
  - ADR-NET-0002
related_artifacts:
  - microservices/network/PRD.md (§Audit + Compliance)
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/runbooks/connection-graph-corruption.md (FM-14)
  - microservices/network/incident-response.md (§"Endorsement-Chain Integrity Compromise = Sev-1")
purpose: Establish the cryptographic integrity model for the endorsement chain — per-endorser Ed25519 signature + Merkle-style sealed chain via the audit-chain µservice — so that endorsement-aggregation (a recruiter-ranker input per ADR-NET-0002) is verifiable, revocation-aware, and tamper-evident.
---

# ADR-NET-0005: Endorsement-chain integrity — per-endorser Ed25519 signature + Merkle-style chain via audit-chain µservice; revocation tombstone semantics; pluggable verification

## Status

Accepted — 2026-05-17.

## Context

Endorsements are a core Professional-network feature: a connection 1-clicks "endorse" for a skill (e.g., "Python", "Public Speaking"). Aggregated endorsement counts contribute to recruiter-stub ranker feature vectors (per ADR-NET-0002). This makes endorsement integrity load-bearing for B2B trust:

- An attacker who can forge an endorsement on a user's profile can artificially inflate their recruiter-rank (potential EU AI Act + EEOC bias regression).
- An attacker who can corrupt the canonical endorsement count breaks the recruiter-stub feature-vector reproducibility — bias-audit replay would diverge.
- A user who endorsed earlier and later revokes the endorsement must produce a verifiable revocation that the audit-chain replay respects.
- LinkedIn's endorsement count is widely understood as low-friction social signal (no cryptographic integrity); oyatie differentiates by providing cryptographic verifiability — a Hyperscaler-grade conformance + audit-grade trust differentiator per ADR-0133.

Constraints:

1. **Tamper-evident**: any modification of historical endorsements must be detectable.
2. **Replay-able**: the audit-chain µservice's authoritative replay must reproduce the canonical endorsement count.
3. **Per-endorser attribution**: each endorsement is signed by the endorser's keypair; signatures are independently verifiable.
4. **Revocation-safe**: a user can revoke their endorsement; revocation is a tombstone in the chain; never removes the cryptographic record.
5. **Pluggable verification**: third parties (recruiters, regulators, the endorsee themselves) can independently verify the endorsement chain without trusting oyatie's runtime.
6. **Per-tenant Merkle**: tenant-scoped Merkle trees; cross-tenant Merkle nodes never share state.
7. **eIDAS 910/2014 alignment**: Ed25519 signatures = AdES; supports EU electronic-signature regulation per Bominal ADR-0028.
8. **DSR cascade alignment**: GDPR Art. 17 erasure tombstones the endorsement record (body-wiped); cryptographic chain intact for verifiability.

## Decision

oyatie network's `endorsement-engine` BC implements:

1. **Per-endorser Ed25519 keypair**:
   - On first endorsement, the endorser's Ed25519 keypair is generated in KMS (OCI KMS or HSM-backed).
   - Public key stored alongside the user's profile (publishable via SDK helper `getEndorserPublicKey`).
   - Private key NEVER exfiltrates the KMS boundary; signing happens via KMS Sign API.
   - Keypair rotation: tenant policy; default 36mo rotation; old public keys retained 7y for verification of historical endorsements (audit-chain replay).
2. **Per-endorsement signature**:
   - Endorsement body: `(endorser_user_ref, endorsee_user_ref, skill_ref, tenant_id, created_at)`.
   - Signature: Ed25519 over `SHA-256(endorsement_body)`.
   - Signature stored in `endorsement_records.signature` column (base64url; 64 bytes Ed25519 = 88 chars base64).
   - Signature stored in audit-chain seal.
3. **Per-tenant Merkle tree**:
   - Endorsement records appended in order (per ADR-NET-0001 advisory-lock-serialised within tenant).
   - Per-tenant Merkle tree over the chain of (endorsement_id, signature, prev_merkle_root) tuples.
   - Sealed Merkle root committed to audit-chain at batch-boundary (default: every 100 endorsements OR every 1 hour, whichever first).
4. **Revocation semantics**:
   - Revocation is a tombstone: the record stays in the chain; its `revoked: true` + `revocation_reason` + `revocation_signature` (Ed25519 over revocation body) added.
   - Aggregated counts respect revocation (revoked endorsements do not count toward recruiter-rank feature vector).
   - GDPR Art. 17 erasure of the endorsement body wipes the body but retains the cryptographic chain link (audit-chain compatibility per Bominal ADR-0028).
5. **Verification**:
   - SDK helper `verifyEndorsementChain(tenant_id)`: re-derives Merkle root from canonical PG data; checks against sealed Merkle root in audit-chain.
   - Per-endorser signature verification: `verifyEndorsement(endorsement_id)` fetches endorser public key + signature + body; verifies Ed25519.
   - CI lane `oya-check-endorsement-chain-integrity` runs verification across all production tenants daily; emits `oya_network_endorsement_chain_integrity_failure_total` Prometheus counter.
6. **Compromise response**:
   - Per `runbooks/connection-graph-corruption.md` §FM-14, signature-verification failure or Merkle-root mismatch triggers Sev-1.
   - Quarantine affected partition; replay-derive from audit-chain; investigate KMS audit log for unauthorised key access.
7. **eIDAS posture**: Ed25519 signatures meet eIDAS 910/2014 AdES criteria; suitable for EU electronic-signature regulation alignment.
8. **Drill cadence**: quarterly per `incident-response.md` Drills table.

## Alternatives Considered

### A. No cryptographic integrity (LinkedIn precedent: count + display only)

- Pros: simpler; lowest operational complexity; matches industry status quo.
- Cons: no tamper evidence; recruiter-stub feature-vector cannot be independently verified; bias-audit replay cannot trust historical endorsement counts; loses Hyperscaler-grade differentiator per ADR-0133.
- Rejected.

### B. RSA-2048 signature instead of Ed25519

- Pros: more widely-supported in legacy PKI stacks.
- Cons: 5-10x slower; signature 256 bytes vs Ed25519 64 bytes; storage + bandwidth waste; modern best practice favours Ed25519 (RFC 8032); not eIDAS-AdES-preferred.
- Rejected.

### C. ECDSA secp256r1 (NIST P-256) signature

- Pros: NIST-curve familiarity; broader regulator acceptance in some packs.
- Cons: requires constant-time implementation discipline; some historical implementation bugs; non-deterministic signatures (different `k` per call complicates audit-chain replay determinism); Ed25519 is preferred for deterministic signatures.
- Rejected; Ed25519 preferred for determinism + audit-grade simplicity.

### D. Tenant-level chain (no Merkle; single per-tenant signature over chain)

- Pros: simpler.
- Cons: cannot verify individual endorsement without re-verifying entire tenant chain; doesn't scale; impractical for hyperscaler-grade tenants.
- Rejected.

### E. Append-only ledger (Hyperledger Fabric-style)

- Pros: deep transparency; supports inter-tenant verification.
- Cons: heavy operational footprint; cross-tenant ledger violates pack-residency + tenant-isolation; Hyperledger requires a separate consensus layer.
- Rejected.

### F. Per-endorser keypair generated client-side (true E2E signing)

- Pros: server cannot forge endorsements; user has cryptographic ownership.
- Cons: key management on client (browser / mobile) is hard; key loss = endorsement loss; UX friction; doesn't match user expectation of "log in and endorse"; client-side signing also creates supply-chain vulnerability via injected scripts in browser context.
- Rejected for P01; revisit at M05-onward if cryptographic UX matures (FIDO2 / Passkeys eventually enable this).

## Consequences

### Positive

- Endorsement chain is tamper-evident + replay-able + independently verifiable.
- Recruiter-stub feature-vector (per ADR-NET-0002) is reproducible from canonical store + audit-chain replay.
- Hyperscaler-grade trust differentiator vs LinkedIn / Xing / Wantedly (none provide cryptographic verifiability).
- eIDAS 910/2014 AdES posture aligns with EU electronic-signature regulation.
- DSR cascade GDPR Art. 17 erasure compatible (body-wipe + chain-link intact).
- KMS-bound signing prevents private-key exfiltration; KMS audit log + KMS-bound rotation discipline.
- Pluggable verification (third-party regulators can verify).

### Negative

- KMS Sign API adds ~10-30ms latency per endorsement; mitigated by batched-seal worker.
- KMS cost: ~$0.03 per 10000 sign operations; trivial at network scale (~$10/month at XS tier).
- Per-endorser keypair lifecycle management is operational overhead.
- Drilling Merkle integrity quarterly + daily LEAN lane is ongoing CI cost.
- Compromise response (FM-14) is high-severity; rare but high-impact.

### Operational

- Cargo workspace: `oya-network-endorsement-engine-*` per BNF v4.1.
- KMS integration: per-endorser keypair under `oyatie/network/endorser/{user_ref}` KMS path; managed by ops-security.
- Postgres: `endorsement_records` table with `signature`, `revoked`, `revocation_signature`, `merkle_chain_position`, `prev_merkle_root` columns.
- Audit-chain: `oya.network.endorsement.v1.added` events sealed; per-tenant Merkle root sealed at batch-boundary.
- LEAN lane: `oya-check-endorsement-chain-integrity` (daily).
- Dashboard: `dashboards/professional-graph-health.json` exposes endorsement-chain integrity panels.
- Runbook: `runbooks/connection-graph-corruption.md` §FM-14.
- Drill: quarterly Merkle-integrity verification.

### Regulatory

- **eIDAS 910/2014**: Ed25519 signatures = AdES; suitable for EU electronic-signature regulation.
- **GDPR Art. 17**: body-wipe tombstone + chain-intact cryptographic record satisfies erasure obligation while preserving audit-trail.
- **EU AI Act Art. 12** (record-keeping): endorsement signatures = audit-grade trail for recruiter-stub feature input.
- **EEOC UGESP**: endorsement-chain replay enables 2y record-keeping verification.
- **SOC 2 CC7.2 + CC9.1**: audit-chain integrity supports SOC 2 monitoring + risk-mitigation.
- **ISO 27001 A.5.34**: privacy + PII protection — endorsement chain preserves attribution without leaking endorser PII to third parties (signature is per-endorser pseudonymous).

## References

- ADR-0028 (Bominal audit-chain).
- ADR-0135 (Connect dissolution, parallel).
- ADR-0131 (per-microservice flat layout).
- ADR-NET-0001 (Postgres adjacency-list + per-tenant advisory lock).
- ADR-NET-0002 (recruiter-stub feature pipeline; endorsement is input).
- `microservices/network/runbooks/connection-graph-corruption.md` §FM-14.
- `microservices/network/incident-response.md` §"Endorsement-Chain Integrity Compromise = Sev-1".
- `microservices/network/dashboards/professional-graph-health.json`.
- `microservices/network/slos/professional-context-isolation-correctness.openslo.yaml` (covers endorsement-chain integrity counters).
- RFC 8032 (Ed25519).
- RFC 6962 (Certificate Transparency Merkle-tree pattern; reference).
- eIDAS Regulation (EU) 910/2014.
- GDPR Art. 17.
- audit-chain µservice docs (under `microservices/audit-chain/`).
