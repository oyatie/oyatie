---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability + ops-security
doc_status: published
---

# Incident response — slides µservice

## Severity tiers

| Sev | Definition | Response time | Escalation |
|---|---|---|---|
| Sev-1 | Data loss / unauthorized disclosure / silent CRDT loss / global outage / cross-tenant leak / Annex III high-risk AI invocation slipped past risk-class | 15 min ack; 30 min mitigation start | on-call + leadership + DPO + legal |
| Sev-2 | Per-pack outage / broadcast-mode degraded / export pipeline failure / chart-live-link inconsistency | 30 min ack; 1h mitigation start | on-call + service owner |
| Sev-3 | Single-tenant impact / SLO burn-rate at 50% budget | 2h ack; 4h mitigation start | on-call |
| Sev-4 | Cosmetic / low-impact / tracked-issue | next business day | service owner |

## Sev-1 playbook

### Silent CRDT loss detected (AC-06 invariant violated)

1. **Detect**: `oya_workflow_studio_collab_silent_loss_attempt_total` (slides equivalent: `oya_slides_collab_silent_loss_attempt_total`) > 0 in 5m window. Per `slos/crdt-merge-no-silent-loss.openslo.yaml`.
2. **Triage**: confirm not a Loro upstream advisory (check RustSec + GitHub Security Advisories); check op stream provenance (HMAC verified vs broken).
3. **Mitigate**:
   - If Loro library bug: pin previous known-good version; revert pod images via Helm rollback; freeze T2 AI authoring temporarily; emit tenant comms.
   - If HMAC tampering: rotate per-session keys; force WS reconnect; emit Sev-1 audit row; investigate compromise.
4. **Verify**: AC-06 property test rerun against last 24h captured op stream.
5. **Post-mortem**: filed within 5 business days per ADR-0123 hyperscaler-maturity gate.

### Cross-tenant leak detected

1. **Detect**: RLS bypass or CDN cache pollution alarm.
2. **Mitigate**: revoke affected CDN cache keys; rotate impacted Postgres connection pool; freeze edits for affected tenants pending verification; notify legal + DPO within 60min.
3. **Tenant notification**: per breach notification SLA (72h GDPR + per-pack as applicable).

### Broadcast-mode degraded — LiveKit signaling drop mid-present

1. **Detect**: `oya_slides_broadcast_signal_health` < 0.95 over 5m.
2. **Triage**: messenger µservice LiveKit cluster status; per-pack SFU health; viewer count vs cap.
3. **Mitigate**: per `runbooks/broadcast-mode-degraded.md` — graceful degradation to non-broadcast present-mode; audience reconnect when signaling recovers; presenter notification banner.

### Export pipeline failure — PPTX cascade

1. **Detect**: export-job error rate > 5% over 10m for PPTX format.
2. **Triage**: gVisor worker OOM trend? Pandoc bridge crash? OOXML serializer assertion fail? OPSWAT/ClamAV signature update issue?
3. **Mitigate**: per `runbooks/export-pipeline-failure-pptx.md` — drain worker pool; rotate Pandoc/serializer container; queue jobs into retry tier; notify tenants of in-flight export with ETA.

### Attachment scan failure — ClamAV / OPSWAT degraded

1. **Detect**: scan failure rate > 1% over 5m.
2. **Mitigate**: switch to dual-scanner-required mode (refuse upload if either scanner unavailable until restored); per `runbooks/attachment-restore.md`.

### Per-pack residency violation

1. **Detect**: overlay enforcement rejects a cross-pack op + Sev-2 alarm; if op succeeded → Sev-1.
2. **Mitigate**: freeze the offending tenant write path; manual root-cause; notify legal + DPO + tenant.

### EU AI Act risk-class slipped

1. **Detect**: `oya-governance-ai-act-risk-class-stamp` lane red OR runtime detection of an Annex III high-risk T2 invocation without explicit pack override.
2. **Mitigate**: roll back T2 capability tier deployment; freeze T2 invocations per pack; manual review; per ADR-SLIDES-0006.

## Runbook index

- `runbooks/collab-conflict-resolution-crdt.md` — CRDT conflict surfaced; tenant-facing UI guidance + ops disambiguation
- `runbooks/broadcast-mode-degraded.md` — LiveKit signaling drop mid-present
- `runbooks/export-pipeline-failure-pptx.md` — PPTX export cascade failure
- `runbooks/attachment-restore.md` — uploaded asset restore from S3 cross-region
- `runbooks/share-acl-drift.md` — ACL state drift between cache + persistent
- `runbooks/theme-corruption.md` — per-pack theme/template signed-bundle corruption
- `runbooks/animation-engine-rollback.md` — animation engine release rollback

## On-call rotation

- Primary: axis-workspace SRE.
- Secondary: ops-sre-reliability.
- Escalation: council-architecture for cross-µservice incidents.
- DPO + Legal: for privacy + AI Act + breach incidents.

## Post-mortem template

- Incident timeline + cause + duration + tenant impact + SLO burn.
- Mitigation steps + rollback decisions.
- Root cause (5-whys analysis).
- Action items + assignees + ETAs.
- Lessons learned + standards updates.

## References

- ADR-0123 hyperscaler-maturity gate (post-mortem retention).
- GDPR Art. 33 breach notification SLA.
- HIPAA §164.404 breach notification (us-healthcare pack).
- ADR-SLIDES-0001 through ADR-SLIDES-0008.
