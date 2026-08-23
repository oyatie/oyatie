---
doc_status: published
---

# Oyatie Runbook — Audit-Chain Integrity Failure

> **Status:** Active (M01-P03 tamper-evidence drill verified)
> **Owner:** `platform-audit-evidence + ops-sre-reliability`
> **Severity scope:** Sev-1
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-14 (M01-P03-IP-003 drill)

## Symptom
audit-chain hash-link mismatch detected on a per-tenant or global shard

## Detection
- Source signal: ADR-0003 chain-integrity check exit code; `presubmit` (retired CLI `gate validate audit-chain-replay`); `governance-audit-emit` block; chain-replay drill anomaly
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev-1 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Freeze writes on the affected shard; switch tenant traffic to read-only; preserve current chain segment as evidence

## Diagnosis
Compare last-known-good Merkle root with current; identify the first divergent block; check for clock skew, signing-key rotation race, or store-tier drift. The canonical one-cycle detector is `verify_chain` replay: the first replay pass over the affected shard MUST reject a modified payload, previous-hash mismatch, Merkle-root mismatch, tenant-shard mismatch, or invalid Ed25519 signature.

## Recovery
Restore from the last-known-good Merkle root; replay events from outbox; compare the recovered shard root with the incident root; emit `EVT-AUDIT-CHAIN-RECOVERED`; re-enable writes only after an independent verifier signs the recovered root.

## Verify-recovery
Run these checks from the repository root and attach output to the incident:

```bash
cargo test -p audit-chain-domain --test merkle_chain merkle_root_advances_with_each_append_and_detects_payload_tamper -- --exact
cargo test -p audit-chain-file-adapter --test file_ledger file_audit_ledger_rejects_divergent_history_and_tampered_records -- --exact
```

Expected proof:
- Domain replay detects a tampered event in a single `verify_chain`/`AuditChain::from_events` pass.
- File-ledger replay rejects divergent append history and rejects a payload tamper as `InvalidChain`.
- The recovered shard's final Merkle root matches the independent verifier's signed recovery note.

Then:
- Confirm SLO error budget recovers within the Sev-1 recovery SLO.
- Confirm audit-chain integrity per ADR-0003 and the `audit.event.emit.v1` AsyncAPI/Proto contract.
- Run the per-axis fitness lane that originally would have caught this; if it did not, file a prevention ticket per [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md).

## Post-incident
- Author postmortem within Sev 1 SLA per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md)
- Add row to [`MISTAKES-LEDGER.md`](../../MISTAKES-LEDGER.md) with mechanical-prevention proposal
- Emit `EVT-PREVENTION-SHIPPED` per ADR-0003 once prevention lands

## Sources
[INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../../SLO-CATALOG.md), [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md), [`templates/runbook-template.md`](../../templates/runbook-template.md), ADR-0003.
