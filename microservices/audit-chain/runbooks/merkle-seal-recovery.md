---
doc_class: Runbook
title: Merkle seal recovery — cross-channel divergence + genesis mismatch
microservice: audit-chain
severity: Sev-1 (chain-integrity)
status: Accepted
owner_team: ops-security + axis-audit-chain
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/failure-modes.md (FM-03, FM-04)
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/incident-response.md
doc_status: published
---

# Runbook: Merkle seal recovery

## Purpose

Recovery for cross-channel root divergence (FM-03) and genesis record mismatch (FM-04). Both are Sev-1 chain-integrity events.

## Trigger

- FM-03: `oya:audit_chain_root_cross_channel_match:rate < 1.0` for any `(pack, tenant_partition, period_id)`.
- FM-04: sealing-worker logs `genesis_record_mismatch` at boot.

## Severity

**Sev-1 always.** These events indicate potential tamper of one or more publication channels OR a fundamental misconfiguration that compromises chain integrity.

## Cross-channel divergence (FM-03) — Procedure

### Phase 1: Halt + Engage (≤ 5 min)

1. Declare Sev-1; open `#inc-<id>` Slack.
2. Engage IC (ops-security primary) + ops-sre-reliability + axis-audit-chain SME + Cryptography SME + council-privacy.
3. Halt sealing-worker for affected partition (preserve forensic state): `kubectl scale deployment sealing-worker-<pack>-<shard> --replicas=0`.
4. Halt continuous-validator's auto-rollback path (manual decision required for chain-state changes).

### Phase 2: Channel Forensics (≤ 1h)

| Channel | What to check | Tool |
|---|---|---|
| S3 WORM blob | Read the SealRecord; compare `root_hash`, `signature`, `signer_public_key_fp` against the canonical Postgres SealRecord row | `aws s3 cp s3://<bucket>/<key> -` + `psql ... -c 'SELECT ... FROM seal_record WHERE ...'` |
| Mimir series | Query `oya_audit_chain_root_hash{pack=<pack>, period_id=<period>}` | Grafana / Mimir CLI |
| GitHub-pinned manifest | Read `evidence/audit-chain-roots/<pack>/<epoch>.json` at HEAD + at any historical commit | `git log -p ...` |
| Postgres SealRecord | Cross-check primary + replica | psql against both |

Determine WHICH channel diverges.

### Phase 3: Diagnose root cause

| Suspect cause | Indicators | Action |
|---|---|---|
| Config drift / accidental Helm change | Recent Helm rollout near divergence time | Roll back the Helm change |
| Sealing-worker bug | Sealing-worker logs show error/exception near sealing time | Pin to known-good version; engage axis-audit-chain to bug-hunt |
| Intentional tamper | No legitimate operation near divergence time; channel state inconsistent with reference Postgres | Engage ops-security forensic team; treat as breach |
| Mimir series tampering | `oya:audit_chain_root_cross_channel_match:rate` Mimir series diverges from S3 + Postgres + GitHub | Investigate Mimir's tenant boundary; check for unauthorized writes |

### Phase 4: Recovery

**If divergence is in Mimir series or GitHub manifest (not S3 + Postgres):**
1. Postgres + S3 are the canonical source-of-truth per `policy/seal-integrity.md` §"SI-04".
2. Re-publish from canonical: `cargo run -p oya-dev-cli -- audit-chain republish-roots --pack <pack> --from <period> --to <period>` (2-person rule; emits `RootRepublished` event sealed in chain).
3. Verify three-channel match returns to 1.0.
4. Postmortem.

**If divergence is in S3 (S3 WORM blob mutated despite Object Lock):**
1. This is a fundamental control failure — Object Lock Compliance mode is designed to prevent this.
2. Engage Oracle Cloud-HSM team + Oracle Object Storage team.
3. Treat as confirmed tamper attempt.
4. Forensic preservation of all relevant artifacts.
5. Possible emergency key revocation per `runbooks/hsm-key-rotation.md` (if tamperer also has signing-key access).

**If divergence is in Postgres SealRecord:**
1. Postgres replica may have rolled back to inconsistent state; check `pg_replication_lag_seconds`.
2. Restore from S3 WORM canonical: SealRecord index is recomputable from S3 raw blobs.
3. Reindex: `cargo run -p oya-dev-cli -- audit-chain reindex-from-s3 --pack <pack> --from <period> --to <period>` (2-person rule).
4. Verify.

### Phase 5: Tenant + regulator comms

Per `incident-response.md` Sev-1 template.

## Genesis record mismatch (FM-04) — Procedure

### Phase 1: Halt sealing (≤ 5 min)

1. Sealing-worker refused to start — chain advance is already halted.
2. Declare Sev-1; engage ExecSponsor + ops-security director + Cryptography SME + council-architecture chair.
3. DO NOT proceed without full incident command.

### Phase 2: Channel comparison

For the affected `(pack, tenant_partition)`:
1. Read genesis record from S3 (`s3://<bucket>/genesis/<pack>/<partition>.json`).
2. Read genesis record from GitHub-pinned manifest (`evidence/audit-chain-genesis/<pack>/<partition>.json`).
3. Read genesis record from Postgres (`SELECT ... FROM genesis_record WHERE pack=... AND partition=...`).
4. Compute the deterministic genesis: `sha256("oyatie-audit-chain-genesis|<pack>|<partition>|<epoch_id>")`.

Identify which channel(s) diverge.

### Phase 3: Recovery

**Critical: only the deterministic genesis is unrelinquishable.** If a channel disagrees with the deterministic genesis, that channel is wrong.

1. Re-compute deterministic genesis.
2. Restore divergent channel(s) from the deterministic value.
3. 2-person rule sign-off recorded in chain.
4. Restart sealing-worker.
5. Verify boot completes; new periods seal.

### Phase 4: Forensic

If the divergence cannot be explained by drift / corruption, treat as **fundamental tampering**:
- Engage ExecSponsor + council-architecture for chain-wide audit.
- Quarantine the affected `(pack, partition)`'s sealing-worker + HSM partition.
- Consider chain reset for the affected partition (declare new epoch with new genesis; old chain remains for verification of pre-incident events).
- Notify all affected tenants; regulatory notification chain.

## Verification (post-recovery)

- All three channels' roots agree for the affected period range.
- Three-channel match rate returns to 1.0 sustained ≥ 1h.
- Sealing-worker boots cleanly; no genesis_record_mismatch.
- Sample verification calls succeed.
- Postmortem within 5 business days.

## References

- `microservices/audit-chain/policy/seal-integrity.md` §"SI-03..SI-05".
- `microservices/audit-chain/failure-modes.md` FM-03 + FM-04.
- `microservices/audit-chain/incident-response.md`.
- Bominal ADR-0028 §"Genesis recording" + §"Cross-channel transparency".
