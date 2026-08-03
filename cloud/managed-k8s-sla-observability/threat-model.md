# Managed K8s SLA Observability — Threat Model

## ADR-0376 Threat Model

### False Green SLA Evidence
**Risk**: Missing, stale, or disagreeing observations are interpreted as healthy
control-plane availability.
**Mitigation**: Snapshot windows carry explicit sample counts and freshness; the
lower-claim/higher-risk state wins when sources disagree; missing samples do not
count as healthy.

### Cross-Tenant Evidence Read
**Risk**: A tenant reads another tenant's cluster summaries, evidence handles, or
burn-rate verdicts.
**Mitigation**: Authorize `(tenant_id, cluster_name)` before loading summaries;
fleet and regional rollups aggregate only same-tenant records.

### Secret or High-Cardinality Evidence Leakage
**Risk**: Evidence records expose kubeconfigs, provider credentials, bearer tokens,
raw pod names, or tenant-identifying high-cardinality labels.
**Mitigation**: Evidence handles reference collector run IDs, trace IDs, source
types, timestamps, and OpenSLO records only; raw secrets and provider payloads are
never emitted.

### Premature Public SLA Claim
**Risk**: Placeholder contracts or target OpenSLO files are mistaken for measured
production SLO evidence.
**Mitigation**: Contracts and SLO files retain target-only claim ceilings until live
collector proof, independent review, protected CI, and rollout evidence exist.
