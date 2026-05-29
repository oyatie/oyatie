---
doc_class: Runbook
title: Evidence archive migration — hot→warm→cold cascade or pack-region migration
microservice: foundry-evidence
severity: Sev-3 (planned change); Sev-2 if migration halts mid-flight
status: Accepted
owner_team: axis-foundry-evidence + council-privacy + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-10, FM-11)
  - microservices/intelligence/policy/data-residency.md (DR-02, DR-05)
  - microservices/intelligence/multi-region.md
  - microservices/audit-chain/runbooks/retention-cascade.md (substrate)
doc_status: published
---

# Runbook: Evidence archive migration

## Purpose

Operational procedure for the archive cascade that moves evidence-pack rows + blob references through hot (Postgres + WORM-hot) → warm (Postgres archive partition + WORM-warm) → cold (cold-tier WORM + metadata-only retain) over the pack-retention lifetime; and for pack-region migrations (rare; only for sub-region failover within the same pack).

Two trigger classes:
- FM-10: planned cascade run (daily); failure of a cascade step.
- FM-11: pack-region migration (e.g., Frankfurt → Madrid within pack-eu) — requires careful chain-locality preservation.

## Trigger

- Daily cascade run: scheduled.
- `oya:foundry_evidence_archive_cascade_lag_hours` > 36 (Sev-2 page).
- pack-region migration: planned change with ChangeRequest.

## Severity

- **Sev-3** for planned cascade run.
- **Sev-2** if cascade lag exceeds 36 h or pack-region migration is in-flight and halts.

## Procedure A — Cascade run

### Phase 1: Pre-run check (≤ 15 min)

1. Verify capacity headroom on warm + cold tiers (`capacity-model.md`).
2. Verify audit-chain substrate retention-cascade is healthy:
   ```
   oya audit-chain retention-cascade status --pack <pack>
   ```
   foundry-evidence retention cascade interlocks with the substrate; if substrate cascade is degraded, foundry-evidence cascade defers.
3. Pull a dry-run plan:
   ```
   oya foundry-evidence archive cascade plan --pack <pack> --target-window <window>
   ```
4. Cedar permit evaluation on the cascade RPC (Cedar `data-residency` + `tenant-scope` invariants).

### Phase 2: Run (≤ 12 h for full pack-day)

1. Run cascade:
   ```
   oya foundry-evidence archive cascade run --pack <pack> --target-window <window>
   ```
2. Each tier transition:
   - hot → warm: Postgres rows move to `evidence_pack_warm` partition; WORM blob lifecycle policy applies (S3 IA storage class transition).
   - warm → cold: WORM blob lifecycle to S3 Glacier Deep Archive; metadata row kept in Postgres `evidence_pack_cold_index` (hash + `audit_event_id` + minimal retrieval pointers).
3. Audit-emit a `foundry.evidence.archive.cascade.applied.v1` event per `(tenant_partition, period_window)` so the substrate's retention-cascade observer keeps in sync.
4. Verify Merkle proofs remain valid post-cascade (the cascade moves blobs, not seals; seals never move).

### Phase 3: Verify (≤ 1 h after run)

1. Sample retrieval test: pull a random subset of cascaded packs + verify hash + verify audit-chain inclusion proof.
2. Check substrate retention-cascade in-sync:
   ```
   oya audit-chain retention-cascade verify-foundry-evidence-sync --pack <pack>
   ```
3. Update dashboard `evidence-storage-growth.json` annotations with cascade event marker.

## Procedure B — Pack-region migration (rare)

### Pre-requisites

- ChangeRequest approved by council-architecture + council-privacy + ExecSponsor.
- Receiving sub-region inside the same pack (cross-pack migration is FORBIDDEN per `policy/data-residency.md` DR-02).
- DR drill of the migration plan completed within last 90 days.

### Phase 1: Stand up receiving region (≤ 4 h)

1. Helm-apply foundry-evidence stack in receiving region:
   ```
   helm upgrade --install foundry-evidence \
     microservices/intelligence/iac/helm/evidence-builder \
     -f overlays/pack-<pack>/<receiving-region>.values.yaml
   ```
2. Verify SPIFFE + Cedar + Postgres + audit-chain bridge come up cleanly in receiving region.

### Phase 2: Quiesce sending region (≤ 30 min)

1. Drain in-flight pack-assembly workers in sending region.
2. Stop accepting new `record_invocation` calls (DNS-flip the recorder REST endpoint to a 503-with-Retry-After).
3. Allow audit-chain bridge to drain dead-letter to substrate.

### Phase 3: Migrate data (≤ 8 h)

1. Postgres logical replication primary cut-over (within-pack only; same chain locality).
2. WORM blob lifecycle transition (sending → receiving) — substrate-managed.
3. Audit-chain bridge cut-over: receiving region's bridge picks up the same per-pack substrate.
4. Sample retrieval test in receiving region.

### Phase 4: Reopen + monitor (≤ 1 h)

1. DNS-flip recorder REST to receiving region; Retry-After-driven callers reconnect.
2. Monitor SLI for record_invocation latency + pack-assembly success rate + audit-chain bridge backlog.
3. Hold sending region in standby (read-only) for 30 days before decommission per ADR-0117.

### Phase 5: Decommission sending region (after 30 days)

- Verify zero residual traffic.
- Helm-uninstall sending region foundry-evidence stack.
- WORM blobs and Postgres rows already migrated; nothing else to evacuate.

## Halt conditions

- Cascade run encounters Postgres replica lag > 60 s → halt cascade; engage cloud-secrets.
- Substrate retention-cascade is in Sev-1 → defer foundry-evidence cascade.
- pack-region migration: replication lag > 5 min → halt; revert to sending-region primary.
- Cedar permit refuses cascade RPC → halt; investigate principal entitlement.

## Verification

- Sample retrieval tests for cascaded packs return matching hashes + valid Merkle proofs.
- audit-chain `RetentionApplied` event count matches foundry-evidence cascade row count.
- For pack-region migration: receiving-region `record_invocation` p99 ≤ 500 ms within 30 min of cut-over.

## References

- `microservices/intelligence/multi-region.md`.
- `microservices/intelligence/policy/data-residency.md`.
- `microservices/audit-chain/runbooks/retention-cascade.md` (substrate).
- ADR-0117 (cloud-native infra).
