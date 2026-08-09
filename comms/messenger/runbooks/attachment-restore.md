---
doc_class: Runbook
title: Attachment store outage / restore
microservice: messenger
severity: "Sev-2"
status: Accepted
owner_team: ops-sre-reliability + cloud-secrets + axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-04)
  - microservices/messenger/multi-region.md
doc_status: published
---

# Runbook: Attachment restore (FM-04)

## Trigger

- S3 endpoint outage in pack region.
- `messenger_attachment_upload_failure_rate` > 5 % for ≥ 2 min.
- Mass blob 5xx errors on fetch.

## Severity

Sev-2.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify provider status (Oracle OCI status page; primary region) | ≤ 5 min |
| 2 | Check S3 endpoint reachability from a messenger pod: `aws --endpoint-url ... s3 ls` | ≤ 5 min |
| 3 | If quorum / cluster degraded: surface tenant-visible "attachments degraded" banner | ≤ 5 min |
| 4 | Enable upload-queue buffering: file-attachment-worker buffers uploads on local disk until S3 returns | ≤ 5 min |
| 5 | If outage > 30 min AND pack has DR pair: initiate DR failover for attachments per `multi-region.md` | ≤ 35 min |
| 6 | Engage cloud-secrets for IAM / KMS issues if cause is auth-related | – |

## DR Failover for Attachments

For DR-pair packs:

| Step | Action |
|---|---|
| 1 | Update Helm runtime config: `attachment.s3.endpoint = <DR-pair-endpoint>` |
| 2 | Confirm S3 CRR replica is up-to-date (≤ 5 min RPO) |
| 3 | Switch read traffic to DR-pair bucket |
| 4 | New uploads write to DR-pair bucket; original lazy-hydrated on demand |
| 5 | When primary returns: re-replicate; reverse switch |

## Restore from Corrupted / Missing Blobs

For T-T-02 / FM-05-style block corruption:

| Step | Action |
|---|---|
| 1 | Identify affected attachment IDs via `messenger_attachment_digest_mismatch_total` label |
| 2 | Restore from S3 versioning history if Object Lock enabled |
| 3 | If not recoverable: mark attachment metadata as `lost_blob`; notify channel members |
| 4 | Audit-chain seal of the loss event |

## Recovery Verification

- `messenger_attachment_upload_p99_seconds` ≤ 0.3.
- `messenger_attachment_upload_failure_rate` ≤ 1 % for ≥ 30 min.
- Upload buffer queue drained to 0.

## Postmortem

- If pack provider reliability < SLA: assess multi-provider strategy.
- If quarantine pattern emerges: investigate scanner false-positives or attack.

## References

- `microservices/messenger/failure-modes.md` FM-04.
- `microservices/messenger/multi-region.md` §"DR Failover".
- OCI Object Storage docs.
- `runbooks/attachment-malware-quarantine.md`.
