---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-OBJECT-STORAGE-DEGRADED
severity_class: sev-2
related_adrs: [ADR-DRIVE-0001]
related_slos: [download-first-byte-latency, upload-multipart-throughput]
owner_team: axis-drive + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: Object-store degraded (Garage / SeaweedFS / SeaweedFS cell loss)

## Symptom

One or more of:
- Garage / SeaweedFS / SeaweedFS cell health metric degraded.
- `oya_drive_object_store_replica_count{tenant}` drops below replication-factor 3.
- `oya_drive_object_store_request_failure_rate` rises.
- Download / upload SLO burn-rate alert.

## Severity

**Sev-2** for single-cell loss (replication absorbs). **Sev-1** for dual-cell loss (only one healthy copy left; write refusal possible) or multi-cell loss.

## First responder

ops-sre-reliability on-call. Escalate to axis-drive if persistent or cross-tenant impact.

## Diagnosis

### Step 1 — Identify the failed cell

```bash
# Garage cluster status
kubectl -n drive exec sts/oya-drive-garage-0 -- garage status

# SeaweedFS cluster status
mc admin info oya-drive-minio --json | jq .

# SeaweedFS health
kubectl -n drive exec deploy/oya-drive-seaweedfs-master -- curl -s localhost:9333/cluster/healthz
```

### Step 2 — Determine replication state

```bash
# Garage: per-bucket replication health
kubectl -n drive exec sts/oya-drive-garage-0 -- garage bucket info <bucket>

# Verify replication-factor 3 is currently enforced
grep replication-factor /etc/oya-drive-garage/config.toml
```

### Step 3 — Identify scope

- Single-tenant impact? Multi-tenant? Whole-pack?

## Mitigation

### Case A — Single cell down (replication absorbs)

1. Replication-factor 3 absorbs; reads + writes continue.
2. Trigger rebuild on neighbour cells (Garage handles automatically; verify).
3. Replace failed node hardware via cloud-iac runbook (typically 30 min - 4h depending on cloud provider).
4. Monitor replication backlog metric:
   ```bash
   kubectl -n drive exec sts/oya-drive-garage-0 -- garage stats | grep -i pending
   ```

### Case B — Dual-cell down (replication-factor 3 → only one healthy copy)

1. **Sev-1**. Engage incident-response.md.
2. Place affected buckets in **read-only mode** (writes refused until rebuild):
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive object-store-readonly \
     --pack <pack> \
     --reason "dual-cell-loss-rebuild"
   ```
3. Trigger emergency cell-add to restore replication:
   ```bash
   cargo run -p oya-dev-cli -- cloud-iac apply --pack <pack> --emergency-cell-add
   ```
4. Tenant comms within 1h: "Drive is in read-only mode; uploads paused pending storage rebuild."

### Case C — Pack-wide outage

1. **Sev-1**. Engage incident-response.md + multi-region.md DR procedure.
2. Failover to secondary region within pack (DNS swing).
3. RPO ≤ 60s; RTO ≤ 15 min (single-region).
4. Cross-cell replication ensures secondary region has the data.

### Case D — SeaweedFS (pack-us-healthcare) degraded

1. SeaweedFS single-cluster deployment for pack-us-healthcare per ADR-DRIVE-0001.
2. Engage `cloud-iac/runbooks/minio-cluster-degraded.md` for SeaweedFS-specific recovery.

### Case E — SeaweedFS archive tier degraded

1. Lower-criticality archive tier.
2. Reads from archive tier degrade to "archive temporarily unavailable; retry in 1h".
3. Engage SeaweedFS recovery per `cloud-iac/runbooks/seaweedfs-cluster-degraded.md`.

## Verification

```bash
# Cell health restored
kubectl -n drive exec sts/oya-drive-garage-0 -- garage status

# Replication backlog cleared
kubectl -n drive exec sts/oya-drive-garage-0 -- garage stats | grep pending

# SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo download-first-byte-latency
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo upload-multipart-throughput
```

## Post-incident

- Cause analysis: hardware failure, network partition, software bug.
- Cell-add ChangeSet via cloud-iac.
- Post-mortem at `evidence/postmortem/<incident_id>.md`.
- Update replication-factor / cell topology if patterns recur.

## References

- ADR-DRIVE-0001 — object-storage substrate (Garage primary, SeaweedFS secondary, SeaweedFS archive).
- `slos/download-first-byte-latency.openslo.yaml`.
- `slos/upload-multipart-throughput.openslo.yaml`.
- `multi-region.md`.
- `incident-response.md` IR-4.
- Garage operator docs.
- SeaweedFS operator docs.
- SeaweedFS operator docs.
