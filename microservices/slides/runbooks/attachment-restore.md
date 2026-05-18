---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: attachment-restore
status: Accepted
severity: Sev-2
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
related_artifacts:
  - microservices/slides/failure-modes.md FM-11
  - microservices/slides/multi-region.md
doc_status: published
---

# Runbook — Attachment (asset) restore

## When to use

- Tenant reports missing image / video / audio asset in deck.
- ClamAV / OPSWAT scan failure rate spike → potentially quarantined assets.
- S3 object 404 from slides-rest during deck render.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Single asset 404 | S3 object lifecycle transition OR cross-region replication lag | step 1 |
| Many assets 404 in pack | Pack-region S3 outage | step 2 |
| Asset quarantined by scanner | ClamAV / OPSWAT detection (could be true-positive or signature-update false-positive) | step 3 |
| Cross-tenant access denied | RLS / IAM condition correct? | step 4 |

## Step 1 — Single-asset 404

```bash
ASSET_ID=<asset_id>
TENANT_ID=<tenant_id>

# Inspect S3 object metadata
oya vcs --service slides --action describe-asset --asset-id $ASSET_ID --tenant-id $TENANT_ID

# Likely fix: restore from cross-region replica
oya vcs --service slides --action restore-asset --asset-id $ASSET_ID --tenant-id $TENANT_ID --source cross-region-replica
```

## Step 2 — Pack-wide S3 outage

Per `multi-region.md`:

```bash
PACK=<pack>

# Failover S3 reads to secondary region
oya vcs --service slides --action s3-region-failover --pack $PACK --target secondary

# Tenant banner
oya vcs --service slides --action announce-asset-failover --pack $PACK
```

## Step 3 — Quarantined asset

```bash
ASSET_ID=<asset_id>

# Inspect scan verdict
oya vcs --service slides --action describe-asset-scan --asset-id $ASSET_ID

# If ClamAV true-positive — keep quarantined; notify tenant
oya vcs --service slides --action notify-tenant-quarantine --asset-id $ASSET_ID

# If OPSWAT scanner regression suspected — escalate to ops-security; do NOT release without dual-confirm
```

ClamAV signature update false-positive flow:
- ClamAV maintainers' advisory check.
- Re-scan with prior signature version.
- If false-positive confirmed: release from quarantine + audit; restore asset; tenant notify.

## Step 4 — Cross-tenant access denied

Per `policy/tenant-scope.cedar` + `threat-model.md` T-I-04.

```bash
# Verify per-tenant IAM condition + S3 prefix
oya vcs --service slides --action verify-tenant-isolation --asset-id $ASSET_ID

# Expected: tenant_id in IAM condition matches; prefix matches
```

If cross-tenant access succeeded incorrectly → Sev-1; treat as data leak.

## Re-enable

After fix:

```bash
# Slide re-render with restored asset
oya vcs --service slides --action re-render-deck --deck-id <deck_id>

# Health verify
oya vcs --service slides --action asset-health
```

## Verification

- Asset 404 rate < 0.1% over 10m.
- Cross-region replication lag < 1min.
- ClamAV/OPSWAT scan success rate > 99%.

## References

- failure-modes.md FM-11.
- threat-model.md T-I-04.
- multi-region.md.
