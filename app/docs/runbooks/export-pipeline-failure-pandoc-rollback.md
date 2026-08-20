---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-export-pipeline-failure-pandoc-rollback
status: Accepted
date: 2026-05-17
owner_team: axis-docs + ops-sre-reliability
severity_applicable: [Sev-2, Sev-1]
related_failure_modes: [FM-04, FM-11, FM-12]
doc_status: published
---

# Runbook — Export pipeline failure (Pandoc / WeasyPrint / Chromium rollback)

## When this runbook fires

- `oya_docs_export_pipeline_failure_rate > 10%` on rolling 5min window.
- `oya_docs_export_pdf_p99_seconds > 5s` (target 3s).
- gVisor seccomp violation alert (Sev-1 escape attempt).
- Tenant reports "PDF export looks corrupted / missing content."
- OOXML import fidelity below threshold (`oya_docs_ooxml_import_fidelity_ratio < 0.95`).

## Symptoms

- Export jobs return failure or timeout.
- PDF output corrupt (broken pages, missing fonts, missing math).
- DOCX export missing tables / images.
- gVisor sandbox crashes; pod restart loop.
- Tenant import fails or silently loses features.

## Probable causes

1. Pandoc upgrade introduced unsupported feature.
2. WeasyPrint dependency upgrade broke font rendering.
3. Chromium-headless OOMKilled on large doc.
4. gVisor + seccomp policy too restrictive (legitimate syscall blocked).
5. Malicious input designed to escape sandbox (FM-12; Sev-1).
6. OOXML feature not in ADR-DOCS-0006 named edge-case matrix.

## Triage (within 30 min)

1. Acknowledge page.
2. Check Grafana dashboard `export-import-pipeline`: failure rate + per-backend distribution.
3. Identify affected backend:
   ```bash
   kubectl -n docs logs -l app=oya-docs-export-import-worker --tail=200 | grep -E "(pandoc|weasyprint|chromium|gvisor)"
   ```
4. Check Pandoc / WeasyPrint / Chromium version in deployed image:
   ```bash
   kubectl -n docs get deployment oya-docs-export-import-worker -o jsonpath='{.spec.template.spec.containers[0].image}'
   ```
5. Cross-reference with ADR-DOCS-0003 LTS pin.
6. If gVisor seccomp violation > 0: declare Sev-1 + page ops-security.

## Recovery Path A — Pandoc / WeasyPrint regression rollback

Cause: upgrade introduced regression.

| Step | Action |
|---|---|
| 1 | Identify last-known-good image tag from `iac/helm/Chart.yaml` git history. |
| 2 | Update deployed image tag back to previous LTS pin: `helm upgrade oya-docs <chart> --set exportImport.image.tag=<previous-tag>`. |
| 3 | Restart worker pods: `kubectl rollout restart deployment/oya-docs-export-import-worker -n docs`. |
| 4 | Verify failure rate drops within 15 min. |
| 5 | Re-pin ADR-DOCS-0003 LTS pin in chart; emit ADR-amendment if a permanent version change is needed. |

## Recovery Path B — Fallback backend (PDF: WeasyPrint → Chromium or vice-versa)

Cause: backend-specific failure (e.g., WeasyPrint can't render a specific font).

| Step | Action |
|---|---|
| 1 | Identify affected tenant + doc. |
| 2 | Switch tenant's PDF backend preference via Cedar policy `pdf-backend-tenant-override.cedar`. |
| 3 | Re-attempt export. |
| 4 | If both backends fail: investigate doc-specific issue; surface to user. |

## Recovery Path C — gVisor seccomp violation (Sev-1)

Cause: malicious input or legitimate syscall blocked.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security. | ≤ 5 min |
| 2 | Quarantine affected worker pods: `kubectl drain <pod>`. | ≤ 10 min |
| 3 | Capture the input payload from gVisor audit log; transfer to forensic sandbox. | ≤ 15 min |
| 4 | If escape pattern matches CVE: patch gVisor / pinned engine version per ADR-DOCS-0003 LTS guidance. | ≤ 1h |
| 5 | If legitimate-but-blocked syscall: update seccomp profile + re-test escape corpus. | ≤ 1h |
| 6 | Re-deploy with new sandbox config. | – |
| 7 | Tenant + ops-security notification. | – |

## Recovery Path D — OOXML import fidelity drift (ADR-DOCS-0006 named edge-case)

Cause: a DOCX feature lost during import.

| Step | Action |
|---|---|
| 1 | Identify affected feature via `oya_docs_ooxml_import_lost_feature_count` metric (per-feature label). |
| 2 | Cross-reference with ADR-DOCS-0006 named edge-case test matrix. |
| 3 | If unsupported feature: surface "fidelity warning" to user in import-result UI; refuse silent loss. |
| 4 | If supported-but-broken: emit hotfix + add to ADR-DOCS-0006 matrix as a regression-test entry. |
| 5 | If new attack pattern in OOXML parser (XXE / archive bomb): patch + add fuzz corpus entry. |

## Recovery Path E — Export queue saturation

Cause: massive concurrent export demand or per-tenant export storm.

| Step | Action |
|---|---|
| 1 | Check queue depth: `oya_docs_export_queue_depth_seconds`. |
| 2 | Identify storm source: `topk(5, sum by (tenant_id) (rate(oya_docs_export_job_submitted_total[5m])))`. |
| 3 | Apply per-tenant rate-limit. |
| 4 | Scale gVisor pool: `kubectl scale deployment/oya-docs-export-import-worker -n docs --replicas=200`. |
| 5 | Verify queue drains within 30 min. |

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `oya_docs_export_pipeline_failure_rate` | < 1% | within 15 min |
| `oya_docs_export_pdf_p99_seconds` | < 3s | within 15 min |
| `oya_docs_ooxml_import_fidelity_ratio` | ≥ 0.95 | within 15 min |
| gVisor seccomp violations | 0 | should be 0 |
| Worker queue depth | < 5min cadence | within 30 min |

## Post-incident review

- Was the LTS pin policy violated?
- Should the gVisor seccomp profile be tightened or loosened?
- Update threat-model.md FM-12 mitigation if needed.
- Update ADR-DOCS-0006 named edge-case matrix if a new fidelity issue surfaced.
- Update LEAN check `oya-governance-export-sandbox-conformance` if a new sandbox-escape vector discovered.

## Drills

- Bi-annual: gVisor escape simulation against CVE corpus.
- Quarterly: pandoc upgrade dry-run in staging with 100-doc reference corpus.

## References

- `failure-modes.md` FM-04, FM-11, FM-12.
- `threat-model.md` T-T-02, T-I-04, T-E-05.
- ADR-DOCS-0003 (export pipeline architecture; LTS pins).
- ADR-DOCS-0006 (DOCX import fidelity matrix).
- Pandoc release notes — `pandoc.org/releases.html`.
- WeasyPrint changelog — `weasyprint.readthedocs.io/en/latest/changelog.html`.
- Chromium release notes — `chromiumdash.appspot.com/`.
- gVisor security model — `gvisor.dev/docs/architecture_guide/security/`.
