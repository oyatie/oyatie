---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: kb-attachment-restore
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_artifacts:
  - microservices/community/failure-modes.md (FM-04, FM-10)
  - microservices/community/multi-region.md
doc_status: published
---

# Runbook: kb-attachment-restore

## When to use

- FM-04 (KB attachment store outage)
- FM-10 (ClamAV scanner failure)

## Symptoms

- S3 PUT/GET p99 > 5 s; error rate > 1 %.
- KB article publish queue depth growing.
- ClamAV scan latency > 30 s; scan queue depth > 1 k.

## Detection

- Grafana alert `community-s3-error-rate-critical`.
- Grafana alert `community-clamav-scan-queue-depth`.

## Triage

1. Identify whether outage is:
   - Regional S3 outage (cloud provider)
   - Bucket-level (permissions / corruption)
   - ClamAV scanner pool issue
   - Network path issue
2. Check S3 status page + provider incident feed.

## Mitigation

### Regional S3 outage

1. For tenants with cross-region replication: failover bucket pointer.
2. For tenants without: enable degraded mode in `kb-article-store-rest` (KB articles serve without attachments; banner notice).
3. Block new uploads; show retry-later to authors.
4. When S3 restored: drain retry queue; verify sha256 integrity per attachment.

### Bucket-level corruption

1. Engage cloud-secrets for IAM verification.
2. List objects vs. Postgres attachment registry; reconcile missing or modified.
3. Restore from cross-region replica.
4. Audit-chain witnesses each restore.

### ClamAV scanner failure

1. Failover to backup scanner replica.
2. If signature DB outdated: pull latest via cloud-secrets-controlled update path.
3. Reject uploads on extended outage (> 30 min); show notice.
4. Drain queue when restored.

## Verification

- S3 error rate < 0.01 %.
- KB article publish p99 < 500 ms.
- ClamAV scan latency p99 < 10 s.

## Post-Incident

- If structural: capacity revision; cross-region replication review.
- Per-tenant attachment integrity audit.

## Owner

axis-community (primary) + ops-sre.
