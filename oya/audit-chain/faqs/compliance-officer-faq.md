---
doc_class: FAQ
microservice: audit-chain
persona: Diana-as-auditor / Dimitri-as-auditor-for-client-A / Hyo-Jin-as-auditor-A / Jakub-as-IT-auditor
date: 2026-05-20
doc_status: published
---

# Compliance Officer FAQ — audit-chain

## Why SHA-256 + Ed25519 and not RSA / ECDSA?

Per `/specs/audit-chain-merkle-ed25519.json` and local ADR-AUD-001, the Merkle tree uses RFC 6962-shaped SHA-256 and the root signature uses Ed25519. SHA-256 is the cross-jurisdiction audit default auditors can reproduce with standard Unix/OpenSSL tooling, and it matches the emitted proof schema in `contracts/openapi/audit-chain.yaml`. Alternative tree hashes may be useful elsewhere, but they are not the audit-chain Merkle proof algorithm.

Ed25519 vs RSA-3072: Ed25519 signing is ~ 80× faster than RSA-3072 (single-core: ~ 30 k sigs/sec vs ~ 380 sigs/sec). At our envelope we need this. Ed25519 vs ECDSA-P256: comparable signing speed but Ed25519 is deterministic (no nonce reuse risk; ECDSA's k-value reuse has caused PlayStation 3 + Sony Bitcoin wallet compromises in the wild). FIPS 140-3 has approved Ed25519 since FIPS 186-5 (2023-02), making it eligible for our HSM-resident path.

## Why per-cell rather than per-tenant signing keys at demo_trial/paid tenant_class?

Per ADR-0028 + ADR-0263. Per-tenant signing keys give cryptographic guarantee that no other tenant's emission can forge against this tenant's chain — but they multiply HSM signing-context overhead by tenant count. At 10 000 tenants/cell on a Thales Luna 7 A790 HSM, per-tenant context-switching costs ~ 8 ms per sign — collapses our throughput from 12 k sigs/sec to ~ 125 sigs/sec. The Cedar + Pulsar-topic + tenant-ID indexing path already enforces tenant isolation; the signing key is a coarse-grained "this cell vouches for the chain" claim, not a per-tenant claim.

At paid tenant_class we DO use per-tenant keys (sovereign-pack tenants are higher-stakes and lower-count, ~ 10-100 per pack), so the per-tenant overhead is manageable.

## How does the chain prove non-repudiation against the EU AI Act Art. 12 logging obligations?

EU AI Act Art. 12(1) requires high-risk AI systems to automatically record events that "enable identification of situations that may result in the AI system presenting a risk." Article 12(2)-(3) require retention for ≥ 6 months and tamper-evidence. The `audit-chain` µservice meets these via:

- Tamper-evidence: every event has `prev_hash` linking to the previous event; every batch has a Merkle root signed by an Ed25519 key that's HSM-resident for paid tenant_class deployments. Tampering with any event invalidates the chain from that point forward; verification reproduces the chain in ≤ 1 s per 1 M events for paid tenant_class deployments.
- Retention: demo_trial 1 y; paid tenant_class 7 y default or pack-defined retention (≥ 6 months minimum).
- The `intelligence` µservice's high-risk-AI events (`intelligence.model.invoked`, `intelligence.refusal.emitted`, `intelligence.policy.gate.evaluated`) terminate in audit-chain per ADR-0220.

Mapping to specific Annex III high-risk categories happens at the `compliance` µservice (per ADR-0143 high-risk-AI-system registration); audit-chain is the receiving substrate.

## Why is the seal interval 1 s and not per-event real-time?

Per ADR-0028 § "Batch sealing rationale". A per-event sign would cost one HSM signing call per event; at 100 k events/sec that's beyond Thales Luna 7 A790's 12 k sigs/sec headroom and would saturate the HSM. Batch sealing every 1 s aggregates the period into one signed root while keeping the `seal-cycle-latency` SLO aligned with the PRD.

The 1 s interval is a SLO trade-off: a tenant verifying immediately after emission may see "event present but not yet sealed" (the event is in the chain head but the most-recent period root has not yet been signed). Tenants who need per-event signatures at paid tenant_class can request "per-event seal" mode at additional HSM cost and reduced throughput envelope.

## Can a tenant export their chain and have an external auditor verify it without our help?

Yes. The `oya audit regulator-export` command emits a tarball containing every event, every Merkle batch, every signing-key public component, plus a standalone `verification.sh` shell script that re-verifies the chain using only `openssl`, `sha256sum`, and `jq`. The external auditor needs ZERO oyatie tooling to verify. This is mandatory under SEC 17a-4(f) and audit-best-practice under SOC 2 CC4.x.

The export bundle is itself audit-emitted (`audit_chain.regulator_export.emitted` event) so any export is traceable.

## What's the dual-tenant seal pattern and when do I use it?

When an event involves two tenants — e.g., a marketplace transaction (`buyer_tenant`, `seller_tenant`), a tenant-to-tenant messenger DM, a cross-tenant data subject request — we emit ONE Merkle leaf with a `tenant_ids: [t1, t2]` array, not two separate leaves. The query view projects per-tenant via the indexed array. This avoids double-counting at the chain level and preserves the canonical event_id.

Use the dual-seal in the emitter via:

```rust
audit_emitter.emit(AuditEvent {
    event_class: "marketplace.transaction.settled".into(),
    tenant_ids: vec![buyer_tenant, seller_tenant],
    payload,
    ..
}).await?;
```

The adapter (`oya-audit-emission-adapter`) handles the array semantics; the verifier projects correctly. Do NOT emit two single-tenant events for the same transaction — that breaks chain causality.

## How do I respond when a tenant says "Splunk Audit shows different counts than your audit-chain"?

This usually means one of:

1. Splunk Audit ingests events from a different source (e.g., raw Kubernetes audit logs, not the per-µservice emission adapter). Compare what's being ingested where.
2. Some `intelligence` µservice events emit only to audit-chain (not to Splunk), because they exceed the Splunk per-event 32 KiB hard limit. audit-chain has no per-event size limit (events > 64 KiB spill to SeaweedFS-S3 with the chain leaf containing the hash).
3. Splunk's de-duplication on `_meta.event_id` may collapse events that audit-chain treats as distinct (e.g., dual-tenant events appearing once in Splunk, twice in audit-chain query views).

Use `migration-playbooks/from-splunk-audit.md` for the dual-emit reconciliation model and `runbooks/audit-export.md` for export-side evidence handling. In 95 % of cases the discrepancy resolves to one of the three above.

## What happens if the HSM is destroyed (fire / earthquake / dual-control loss)?

paid tenant_class: HSM cluster is ≥ 3 HSMs across ≥ 2 AZs. Single-HSM loss is non-blocking. Cluster-wide loss follows `runbooks/hsm-key-rotation.md` for key continuity and `runbooks/merkle-seal-recovery.md` for seal resumption. RTO depends on pack custody procedures and is reported through the incident runbook evidence.

paid tenant_class pack-bound HSM with M-of-N Shamir custody. The Shamir shards are held by 5 named pack operators in geographically separated facilities. Activation requires 3 of 5. Loss of > 2 shards is unrecoverable; the chain becomes permanently "verifiable-only" (no new signs possible). At that point the affected tenants migrate to a fresh chain instance with a key-rotation event marking the transition.

The DR drill runs annually through `runbooks/hsm-key-rotation.md`, `runbooks/chain-replay-from-snapshot-protocol.md`, and `runbooks/merkle-seal-recovery.md`.

## Why don't we use blockchain (e.g., Ethereum, Hyperledger) as the substrate?

Per ADR-0028 § "Why not blockchain". Three reasons:

1. Throughput: public blockchains commit ≤ 100 tx/s (Ethereum mainnet sustained 15-30 tx/s); private blockchains (Hyperledger Fabric) commit ≤ 3 500 tx/s with careful tuning. We need ≥ 100 k events/sec sustained.
2. Cost: per-event commit on Ethereum costs USD 0.5-5 in gas. At 100 k events/sec that's USD 4-40 M / hour. Not viable.
3. Privacy: blockchain is by definition multi-party-visible. Per-tenant chains in a shared ledger require ZK proofs (zkSNARK, Cairo, etc.) at additional throughput cost. We chose per-cell Ed25519 + Merkle which is functionally equivalent for our threat model (we are NOT trying to prove against an oyatie-internal compromise; we are proving against tenant repudiation and external-adversary tampering).

The audit chain is "blockchain-like" in that it's Merkle-sealed + signed + non-rewritable, but it is single-party-trust (the cell operator) not multi-party-consensus. SOC 2 + SEC 17a-4(f) + EU AI Act Art. 12 all accept this model.

## How do I prove that an event from 2025-12 is still intact today?

```sh
oya audit verify-chain --cell <cell> --tenant <tenant> \
    --since 2025-12-01 --until 2025-12-31
```

The verifier replays every event in that range, checks `prev_hash` chains correctly, recomputes every batch Merkle root, and verifies every batch signature against the signing key active at that time. The verifier compiles to a stand-alone binary that an auditor can run; it doesn't need any audit-chain runtime services — just the exported event bundle + signing-key history.

For ad-hoc spot-checks: `oya audit prove --event-id <id>` emits a single-event Merkle proof; pipe to `oya audit external-verify proof.json --pubkey <signing-key-public>` and you get PASS/FAIL.

## What's our exposure if a signing key leaks?

A leaked demo_trial signing key (sealed-secret) means an attacker can sign forged events that look authentic. Mitigation: rotation. demo_trial tenant_class keys rotate every 30 d; the rotation event itself is signed by the outgoing key and the new key — verifier accepts events signed by any key active at emission time, but rejects events whose claimed signing-key was rotated-out before claimed emission timestamp.

A leaked paid tenant_class HSM key is non-leakable by construction (the key never leaves the HSM). The HSM API allows signing but not exporting. Compromise scenario: an attacker with HSM access + signing-PED. Mitigation: HSM signing-PED quorum + HSM operator role-separation; the audit-chain itself audits every signing operation (`hsm_sign_called` events stored OUTSIDE the audit-chain at Splunk Audit, to break the circular-trust).

A leaked paid dual-control quorum (3 of 5 Shamir shards) is the worst case — equivalent to the HSM signing key being readable. Mitigation: the dual-control quorum members are geographically + organisationally separated; combining 3 requires either a court order (in which case lawful) or a 3-party physical conspiracy (in which case detectable via the per-operator quorum-attempt log).

## How does this differ from `observability` traces?

- `observability`: span-level traces with sampling. Optimised for read-by-SRE. Retention 90 d default. Sampled at 1-10 %. Replay-by-trace-id within the sampling window.
- `audit-chain`: every event of consequence, no sampling. Optimised for cryptographic verification. Retention 1-25 y depending on tenant_class and pack. Replay-by-event-id forever within retention.

A trace span has `trace_id, parent_span_id, attributes`; an audit event has `event_id, prev_hash, signature, payload`. Both are emitted from the same µservice code paths but to different substrates. A `workflow.step.completed` emits BOTH (one trace span for SRE, one audit event for compliance).
