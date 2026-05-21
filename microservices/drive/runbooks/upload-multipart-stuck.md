---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-UPLOAD-MULTIPART-STUCK
severity_class: sev-2
related_adrs: [ADR-DRIVE-0001, ADR-DRIVE-0002]
related_slos: [upload-multipart-throughput]
owner_team: axis-drive + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: Multipart upload stuck

## Symptom

Multipart upload sessions exhibit one of:
- Sessions persist in `pending` / `chunks-staging` state past their normal completion window (sessions older than 4× the median completion time).
- Per-tenant queue depth `oya_drive_upload_session_pending_count{tenant_id}` exceeds 10× baseline.
- `oya_drive_upload_multipart_p99_seconds` exceeds the `upload-multipart-throughput` SLO bound.

## Severity

**Sev-2** by default. **Sev-1** if (i) > 100 tenants impacted simultaneously, (ii) bytes-loss suspected, or (iii) stuck sessions are blocking a legal-hold or WORM ingest.

## First responder

axis-drive on-call. Escalate to ops-sre-reliability if Sev-1 or if object-store backend involvement is suspected.

## Diagnosis

### Step 1 — Identify the stuck-session signature

```bash
# Per-tenant pending session count, sorted descending
kubectl -n drive exec deploy/oya-drive-upload-rest -- \
  curl -s localhost:9090/metrics |
  grep '^oya_drive_upload_session_pending_count'
```

### Step 2 — Inspect a sample stuck session

```bash
# Per-session state from Valkey (upload session in-flight)
kubectl -n drive exec sts/oya-drive-valkey-0 -- valkey-cli \
  --scan --pattern 'upload-session:<tenant>:*' |
  head -5 |
  xargs -I {} valkey-cli HGETALL {}
```

Look for:
- `state=chunks-staging` + `last_chunk_received_at` > 30 min ago → client likely abandoned.
- `state=chunks-complete` + `promotion-pending` → scan worker hot-spot (jump to Case C).
- `state=virus-scan-pending` + stuck > 10 min → scan worker saturation (jump to Case D).

### Step 3 — Inspect object-store staging bucket

```bash
# Per-tenant staging-bucket size
mc admin info oya-drive-garage --json | jq '.staging_bucket_bytes'
mc ls --recursive oya-drive-garage/staging/<tenant>/ | wc -l
```

## Mitigation

### Case A — Client-abandoned sessions

1. Run the abandonment-sweep worker to age out > 24h sessions:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive abandon-sweep \
     --tenant <tenant_id> \
     --older-than 24h
   ```
2. Surface "Upload was abandoned; please retry" to tenant via tenant-portal banner.

### Case B — Tenant rate-storm (legitimate but exceeds quota)

1. Surface "Upload queued; please slow down" via tenant-portal banner.
2. Raise per-tenant upload quota if tenant is on a higher tier; otherwise apply throttle.
3. Add per-tenant rate limit at gateway:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin throttle-add \
     --microservice drive \
     --tenant <tenant_id> \
     --metric upload-rate \
     --rate "100 rpm" \
     --duration 1h
   ```

### Case C — Object-store backend hot-spot (Garage rebalance / SeaweedFS degradation)

1. Inspect object-store health:
   ```bash
   mc admin info oya-drive-garage --json
   ```
2. If degraded, engage `runbooks/object-storage-degraded.md`.

### Case D — Virus-scan worker saturation

1. Engage `runbooks/virus-scan-rollback.md` if signature update suspected.
2. Otherwise scale virus-scan workers:
   ```bash
   kubectl -n drive scale deploy/oya-drive-dlp-virus-scan-worker --replicas=+5
   ```

### Case E — Promotion-worker hot-spot (chunks complete but not promoted to durable)

1. Inspect promotion worker queue:
   ```bash
   kubectl -n drive logs deploy/oya-drive-upload-worker --tail=100 | grep promote
   ```
2. Scale promotion worker:
   ```bash
   kubectl -n drive scale deploy/oya-drive-upload-worker --replicas=+5
   ```

## Verification

```bash
# Pending session count back to baseline
kubectl -n drive exec deploy/oya-drive-upload-rest -- \
  curl -s localhost:9090/metrics |
  grep oya_drive_upload_session_pending_count

# Multipart SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo upload-multipart-throughput
```

## Post-incident

- Update this runbook if stuck-session signature was new.
- File fix-up task if root cause was a code regression (ADR-0114 canary-observability-rollback rules).
- If client-abandonment > 5% of sessions sustained 24h, escalate to tenant-portal UX team.

## References

- ADR-DRIVE-0001 — object-storage substrate.
- ADR-DRIVE-0002 — CDC + delta-sync.
- `slos/upload-multipart-throughput.openslo.yaml`.
- Garage operator docs.
- SeaweedFS administration guide.
- tus.io 1.0 server troubleshooting.
