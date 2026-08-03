# Managed K8s SLA Observability — Failure Modes

## Missing or Stale Observation
- **Impact**: summary freshness is unknown, availability evidence is no-data or
  degraded, and downstream rollout/rollback policy must not treat the cluster as
  green.
- **Recovery**: check collector health, source status freshness, broad
  observability prerequisites, and the last accepted snapshot window before
  re-enabling live evidence claims.

## Unknown Tenant or Cluster
- **Impact**: reads and ingestion fail closed with a typed missing-cluster or
  authorization outcome. No empty success summary is synthesized.
- **Recovery**: verify tenant scope, cluster inventory, and control-plane-host
  status source registration.

## Source Disagreement
- **Impact**: when control-plane status and live scrape evidence disagree, the
  lower-claim/higher-risk state wins for alerting and rollout/rollback holds.
- **Recovery**: inspect source timestamps, scrape freshness, adapter health, and
  cluster lifecycle events; keep the measured claim held until the discrepancy is
  resolved.

## OpenSLO Evidence Missing
- **Impact**: target SLO files remain target-only; no measured-production SLO or
  public SLA claim is allowed.
- **Recovery**: attach reviewed evidence records to the current cloud-path OpenSLO
  files and pass the live ingestion Review/fix gate.

## Cross-Tenant Read Attempt
- **Impact**: the read is denied before loading summaries or evidence handles.
- **Recovery**: confirm principal tenant scope and requested `(tenant_id,
  cluster_name)` match; investigate repeated attempts as policy or abuse signals.
