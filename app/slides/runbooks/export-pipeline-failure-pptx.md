---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: export-pipeline-failure-pptx
status: Accepted
severity: Sev-2
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
related_artifacts:
  - microservices/slides/decisions/ADR-SLIDES-0003-export-pipeline-fidelity.md
  - microservices/slides/slos/export-pptx-latency.openslo.yaml
  - microservices/slides/failure-modes.md FM-08
doc_status: published
---

# Runbook — Export pipeline failure (PPTX)

## When to use

- PPTX export error rate > 5% over 10m.
- gVisor export-worker OOM rate > 3 / 5min.
- Tenant reports stuck export job.
- AC-15 lane `slides-pptx-roundtrip-subset` red.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Pandoc bridge crash on import-side | Malformed OOXML OR Pandoc upgrade regression | step 1 |
| OOXML serializer panic on export-side | Bespoke serializer edge case | step 2 |
| gVisor OOM | Memory budget exceeded by malicious or pathological deck | step 3 |
| Slow exports (queue depth growing) | Worker pool saturated | step 4 |

## Step 1 — Pandoc bridge crash

```bash
# Inspect last 10 failed import-side jobs
oya vcs --service slides --action export-failures --component pandoc-bridge --last 10

# Sample a failed input file (sandboxed; do NOT open locally)
oya vcs --service slides --action sample-failed-import --job-id <job_id> --to-sandbox

# Pandoc version + advisory check
oya vcs --service slides --action pandoc-version
```

If Pandoc upgrade regression: rollback to last-known-good Pandoc image; re-queue failed jobs.

```bash
helm rollback slides-export-pool <known_good_revision> -n workflow-studio
```

If malformed OOXML class detected: ClamAV/OPSWAT scan verdict; quarantine input + audit.

## Step 2 — OOXML serializer panic on export-side

```bash
oya vcs --service slides --action export-failures --component ooxml-serializer --last 10

# Inspect deck shape
oya vcs --service slides --action describe-deck --deck-id <deck_id>
```

If deck has unsupported feature (e.g., unsupported animation type, deeply nested grouping > 32 levels): serializer should emit a structured `EmitDiagnostic` not a panic. File issue + temporary refusal at boundary; tenant fallback to PDF export.

## Step 3 — gVisor OOM

Per failure-modes.md FM-08 + threat-model.md T-T-03.

```bash
# OOM source — input PPTX
oya vcs --service slides --action export-failures --component gvisor-oom --last 10

# Pull job input metadata (size + page count)
oya vcs --service slides --action describe-import-job --job-id <job_id>
```

If input >> typical:
- Raise per-job memory budget temporarily (max 4 GiB for import; 8 GiB for MP4); audit + alarm.
- If input is malicious-grade: refuse + ClamAV/OPSWAT scan + quarantine + tenant notify.

If OOM is widespread: gVisor or worker memory leak; rotate worker pod images.

```bash
# Rotate all export-worker pods
kubectl rollout restart deployment/slides-export-pool -n workflow-studio
```

## Step 4 — Queue saturation

```bash
oya vcs --service slides --action export-queue-depth

# If > 500, HPA should scale; verify
kubectl get hpa -n workflow-studio | grep slides-export-pool

# Manual scale if HPA stuck
kubectl scale deployment slides-export-pool --replicas=20 -n workflow-studio
```

Tenant communication: in-app banner for affected jobs with ETA.

## Re-enable

After fix:

```bash
# Re-queue failed jobs
oya vcs --service slides --action re-queue-export --filter "status=failed AND last_attempt > 1h"

# Health verify
oya vcs --service slides --action export-health
```

## Verification

- Export error rate < 1% over 10m.
- gVisor OOM rate < 1 / 5min.
- AC-15 lane green.
- Queue depth < 100.

## Escalation

- Sev-2: on-call.
- Sev-1: if security-related (malicious upload causing crash); ops-security + DPO.

## References

- ADR-SLIDES-0003 (export pipeline fidelity).
- threat-model.md T-T-03.
- failure-modes.md FM-08.
- ECMA-376 OOXML PresentationML.
