---
doc_class: Standard
title: OpenSLO Authoring Standard
status: Accepted
date: 2026-05-20
owner: axis-observability + ops-sre-reliability
related_oyatie_adrs:
  - ADR-0130
  - ADR-0131
  - ADR-0139
  - ADR-0250
  - ADR-0263
enforced_by:
  - governance-openslo-conformance
  - governance-openslo-promql-feasibility
  - governance-agentic-slo-gated-promotion
canonical_paths:
  - docs/standards/observability-slo.md
  - microservices/*/slos/*.openslo.yaml
  - specs/agentic-slo-gated-promotion.json
external_reference:
  - https://github.com/OpenSLO/OpenSLO
---

# OpenSLO Authoring Standard

This standard is the canonical authoring surface for SLO manifests in Oyatie.
OpenSLO is the external open specification for expressing SLOs as YAML; Oyatie
uses it as the promotion gate input for every microservice that moves past dev.
`docs/standards/observability-slo.md` remains the broader observability standard;
this file is the focused authoring checklist for `*.openslo.yaml`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to `microservices/<ms>/slos/*.openslo.yaml`.

It applies to availability SLOs.

It applies to latency SLOs.

It applies to correctness SLOs.

It applies to freshness SLOs.

It applies to worker and batch SLOs.

It applies to burn-rate alert definitions.

It applies to promotion decisions using SLO evidence.

It applies to dashboards and runbooks referenced by SLOs.

It does not define all telemetry schemas.

It does not replace `observability-slo.md`.

It does not authorize metric names outside the telemetry standard.

## Normative Requirements

S-001. Every promoted microservice MUST have at least one availability SLO unless it is not request-serving.

S-002. Every promoted request-serving microservice MUST have a latency SLO.

S-003. Every promoted microservice MUST have a correctness SLO.

S-004. Every promoted stream, cache, projection, or batch service MUST have a freshness SLO.

S-005. Every SLO manifest MUST use `apiVersion: openslo/v1` unless an ADR upgrades the version.

S-006. Every SLO manifest MUST use `kind: SLO`.

S-007. Every SLO manifest MUST declare `metadata.name`.

S-008. Every SLO manifest MUST declare `metadata.labels.microservice`.

S-009. Every SLO manifest MUST declare `metadata.labels.sli`.

S-010. Every SLO manifest MUST declare `metadata.labels.data_classes`.

S-011. Every SLO manifest MUST declare `metadata.labels.pack` or explicit pack-agnostic posture.

S-012. Every SLO manifest MUST declare `spec.service`.

S-013. Every SLO manifest MUST declare an indicator.

S-014. Every SLO manifest MUST declare at least one objective.

S-015. Every SLO manifest MUST declare a rolling time window.

S-016. Every SLO manifest MUST declare budgeting method.

S-017. Every ratio SLI MUST define good and total metrics.

S-018. Every threshold SLI MUST define threshold semantics.

S-019. Every PromQL expression MUST be tenant-safe.

S-020. Every PromQL expression MUST avoid unbounded cardinality.

S-021. Every PromQL expression MUST use service-owned metrics unless cross-service composition is declared.

S-022. Every target MUST be at least 0.90.

S-023. Targets above 0.99999 MUST cite an ADR or waiver.

S-024. Availability production default SHOULD be at least 0.999.

S-025. Correctness production default SHOULD be at least 0.9999.

S-026. Latency SLOs SHOULD use ratio-under-budget form.

S-027. Burn-rate alerts MUST include fast-burn page.

S-028. Burn-rate alerts MUST include slow-burn page.

S-029. Burn-rate alerts MUST include ticket-burn.

S-030. Burn-rate alerts MUST include budget-exhausted signal.

S-031. Every page alert MUST reference a runbook.

S-032. Every SLO MUST reference an owning team.

S-033. Every SLO MUST reference a dashboard or dashboard stub.

S-034. Every SLO MUST name promotion impact.

S-035. Every SLO MUST name rollback impact.

S-036. Every SLO MUST name synthetic or fixture validation.

S-037. Every SLO MUST name retention for evidence.

S-038. Every SLO MUST name data residency implications.

S-039. Every SLO MUST be contract-testable.

S-040. Every SLO MUST be query-feasible against representative telemetry.

S-041. Worker completion SLOs MUST declare success and total counters.

S-042. Batch duration SLOs MUST declare expected cadence.

S-043. Projection freshness SLOs MUST declare event source.

S-044. Queue lag SLOs MUST declare queue name and partition dimensions.

S-045. SLO manifests MUST NOT use wildcard tenant selectors.

S-046. SLO manifests MUST NOT select metrics from another tenant.

S-047. SLO manifests MUST NOT hide failures by excluding 5xx without total denominator.

S-048. SLO manifests MUST NOT use stale recording rules without owner.

S-049. SLO manifests MUST NOT use environment-specific metric names as canonical expressions.

S-050. SLO manifests MUST NOT omit data-class labels.

S-051. SLO changes that weaken targets MUST cite a decision.

S-052. SLO changes that strengthen targets SHOULD include capacity evidence.

S-053. SLO deletion MUST include service retirement or replacement evidence.

S-054. SLO waivers MUST have expiry.

S-055. SLO waivers MUST identify the missing signal.

S-056. SLO waivers MUST identify compensating control.

S-057. SLO evidence MUST be retained with promotion evidence.

S-058. SLO evidence MUST include query time.

S-059. SLO evidence MUST include data source.

S-060. SLO evidence MUST include error-budget state.

## Worked Examples

### Example 1: Availability SLO

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: workflow-engine-availability
  labels:
    microservice: workflow-engine
    sli: availability
    data_classes: BEHAVIORAL_TENANT_PRODUCT
spec:
  service: workflow-engine
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(http_requests_total{job="workflow-engine",status!~"5.."}[5m]))
        total:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(http_requests_total{job="workflow-engine"}[5m]))
  objectives:
    - target: 0.999
  timeWindow:
    - duration: 30d
      isRolling: true
```

This passes because numerator and denominator are explicit.

### Example 2: Latency ratio SLO

```yaml
ratioMetric:
  good:
    metricSource:
      type: Prometheus
      spec:
        query: sum(rate(http_request_duration_seconds_bucket{job="mail",le="0.2"}[5m]))
  total:
    metricSource:
      type: Prometheus
      spec:
        query: sum(rate(http_request_duration_seconds_count{job="mail"}[5m]))
```

This is preferred over bare p99 threshold because it composes with burn rate.

### Example 3: Projection freshness SLO

```yaml
metadata:
  name: ontology-mail-thread-freshness
  labels:
    microservice: ontology
    sli: freshness
spec:
  indicator:
    spec:
      thresholdMetric:
        metricSource:
          type: Prometheus
          spec:
            query: max(ontology_projection_lag_seconds{projection="MailThread"})
```

This passes when the threshold and objective define maximum tolerated lag.

### Example 4: Invalid wildcard tenant

```promql
sum(rate(http_requests_total{tenant=~".*"}[5m]))
```

This fails because tenant wildcarding hides isolation and cardinality problems.

### Example 5: Worker completion SLO

```promql
sum(rate(workflow_worker_completed_total[5m]))
/
sum(rate(workflow_worker_started_total[5m]))
```

This passes when retries are idempotent and counters use stable labels.

## Verification

Primary command:

```bash
presubmit (retired CLI gate validate) openslo-conformance --scope microservices
```

Companion command:

```bash
presubmit (retired CLI gate validate) openslo-promql-feasibility --scope microservices
```

The checker MUST parse every `*.openslo.yaml`.

The checker MUST validate required fields.

The checker MUST validate labels.

The checker MUST validate data-class values.

The checker MUST validate pack labels.

The checker MUST validate PromQL syntax.

The checker MUST validate metric ownership.

The checker MUST validate target bounds.

The checker MUST validate burn-rate alerts.

The checker MUST validate dashboard references.

The checker MUST validate runbook references.

The checker MUST validate promotion impact.

The checker MUST validate waiver expiry.

The checker MUST reject missing manifests for promoted services.

The checker MUST emit service-by-service SLO coverage.

The checker SHOULD run queries against a representative Mimir snapshot.

The checker SHOULD emit error-budget state evidence.

## Common Anti-Patterns

Using p99 threshold with no denominator is an anti-pattern.

Using target `1.0` without ADR is an anti-pattern.

Using target below `0.90` is an anti-pattern.

Using wildcard tenant labels is an anti-pattern.

Using cross-service metrics without declaration is an anti-pattern.

Using metric names that exist only in dev is an anti-pattern.

Using SLOs without runbooks is an anti-pattern.

Using SLOs without dashboards is an anti-pattern.

Using one generic SLO for all services is an anti-pattern.

Using stale recording rules is an anti-pattern.

Deleting an SLO to pass promotion is an anti-pattern.

Weakening a target without decision evidence is an anti-pattern.

Treating SLO as observability-only instead of promotion input is an anti-pattern.

Treating data-class labels as optional is an anti-pattern.

Treating batch jobs as exempt from SLOs is an anti-pattern.

## Cross-References

External authority: `https://github.com/OpenSLO/OpenSLO`.

`docs/standards/observability-slo.md` is the broader observability standard.

`docs/decisions/ADR-0700-ci-admission-live-apex.md` binds SLO-gated promotion.

`docs/decisions/ADR-0706-observability-live-apex.md` binds telemetry emission.

`docs/standards/trace-sampling-tier.md` binds trace sampling.

`docs/standards/cross-microservice-latency-budget.md` binds latency budgets.

`docs/standards/capability-tier-matrix.md` binds tier SLO impact.

## Substance Bar Compliance Checklist

SLO-SB-001. Verify `apiVersion: openslo/v1`.

SLO-SB-002. Verify `kind: SLO`.

SLO-SB-003. Verify metadata name.

SLO-SB-004. Verify microservice label.

SLO-SB-005. Verify SLI label.

SLO-SB-006. Verify data class label.

SLO-SB-007. Verify pack label.

SLO-SB-008. Verify service name.

SLO-SB-009. Verify indicator shape.

SLO-SB-010. Verify objectives.

SLO-SB-011. Verify rolling window.

SLO-SB-012. Verify budgeting method.

SLO-SB-013. Verify ratio good metric.

SLO-SB-014. Verify ratio total metric.

SLO-SB-015. Verify threshold metric semantics.

SLO-SB-016. Verify PromQL syntax.

SLO-SB-017. Verify tenant-safe query.

SLO-SB-018. Verify bounded cardinality.

SLO-SB-019. Verify target lower bound.

SLO-SB-020. Verify target upper waiver.

SLO-SB-021. Verify fast-burn alert.

SLO-SB-022. Verify slow-burn alert.

SLO-SB-023. Verify ticket-burn alert.

SLO-SB-024. Verify budget-exhausted signal.

SLO-SB-025. Verify page runbook.

SLO-SB-026. Verify owner team.

SLO-SB-027. Verify dashboard.

SLO-SB-028. Verify promotion impact.

SLO-SB-029. Verify rollback impact.

SLO-SB-030. Verify fixture validation.

SLO-SB-031. Verify evidence retention.

SLO-SB-032. Verify residency implications.

SLO-SB-033. Verify worker completion SLOs.

SLO-SB-034. Verify batch duration SLOs.

SLO-SB-035. Verify projection freshness SLOs.

SLO-SB-036. Verify queue lag SLOs.

SLO-SB-037. Verify SLO waiver expiry.

SLO-SB-038. Verify missing-signal waiver reason.

SLO-SB-039. Verify compensating control.

SLO-SB-040. Verify error-budget state.

SLO-SB-041. Check `availability.openslo.yaml`.

SLO-SB-042. Check `latency.openslo.yaml`.

SLO-SB-043. Check `correctness.openslo.yaml`.

SLO-SB-044. Check `freshness.openslo.yaml`.

SLO-SB-045. Check `workflow-engine` SLOs.

SLO-SB-046. Check `ontology` SLOs.

SLO-SB-047. Check `observability` SLOs.

SLO-SB-048. Check `messenger` SLOs.

SLO-SB-049. Check `tenancy` SLOs.

SLO-SB-050. Check `policy-engine` SLOs.

SLO-SB-051. Reject wildcard tenant selectors.

SLO-SB-052. Reject target below 0.90.

SLO-SB-053. Reject five-nines without waiver.

SLO-SB-054. Reject missing data class.

SLO-SB-055. Reject missing runbook.

SLO-SB-056. Reject missing dashboard.

SLO-SB-057. Reject stale recording rules.

SLO-SB-058. Reject cross-service metric without declaration.

SLO-SB-059. Reject SLO deletion without retirement.

SLO-SB-060. Reject target weakening without decision.

SLO-SB-061. Emit manifest count.

SLO-SB-062. Emit availability SLO count.

SLO-SB-063. Emit latency SLO count.

SLO-SB-064. Emit correctness SLO count.

SLO-SB-065. Emit freshness SLO count.

SLO-SB-066. Emit waiver count.

SLO-SB-067. Emit PromQL feasibility count.

SLO-SB-068. Emit error-budget count.

SLO-SB-069. Emit dashboard link count.

SLO-SB-070. Emit runbook link count.

SLO-SB-071. Preserve OpenSLO v1 profile until ADR upgrade.

SLO-SB-072. Preserve promotion gate semantics.

SLO-SB-073. Preserve error budget math.

SLO-SB-074. Preserve tenant-safe metrics.

SLO-SB-075. Preserve data-class linkage.

SLO-SB-076. Preserve pack overlay linkage.

SLO-SB-077. Preserve burn-rate alerts.

SLO-SB-078. Preserve rollback signal.

SLO-SB-079. Preserve runbook binding.

SLO-SB-080. Preserve dashboard binding.

## Extended Worked Example: Workflow Engine Availability SLO

```yaml
apiVersion: openslo/v1
kind: Service
metadata:
  name: workflow-engine
  displayName: Workflow Engine
spec:
  description: Coordinates durable workflow templates, timers, and compensation.
  owner: team-workflow-substrate
---
apiVersion: openslo/v1
kind: SLI
metadata:
  name: workflow-engine-request-success-ratio
spec:
  ratioMetric:
    good:
      metricSource:
        type: prometheus
        spec:
          query: |
            sum(rate(oyatie_workflow_requests_total{result="success"}[5m]))
    total:
      metricSource:
        type: prometheus
        spec:
          query: |
            sum(rate(oyatie_workflow_requests_total[5m]))
---
apiVersion: openslo/v1
kind: SLO
metadata:
  name: workflow-engine-availability
  annotations:
    oyatie.com/data-class: internal
    oyatie.com/adr: ADR-0145
    oyatie.com/runbook: docs/runbooks/workflow-engine-availability.md
spec:
  service: workflow-engine
  indicator:
    metadata:
      name: workflow-engine-request-success-ratio
  budgetingMethod: Occurrences
  objectives:
    - displayName: 30-day availability
      target: 0.999
      timeWindow:
        - duration: 30d
          isRolling: true
  alertPolicies:
    - metadata:
        name: workflow-engine-fast-burn
      spec:
        description: Fast burn for workflow-engine availability.
        alertWhenBreaching: true
        conditions:
          - kind: burnrate
            threshold: 14
            lookbackWindow: 5m
```

## Extended OpenSLO Object Matrix

| ID | Object | Required Oyatie fields | Example path | Checker |
|---|---|---|---|---|
| SLO-MAT-001 | Service | owner and description | `docs/slos/workflow-engine.openslo.yaml` | `check-openslo-authoring` |
| SLO-MAT-002 | SLI | Prometheus query and unit | `docs/slos/workflow-engine.openslo.yaml` | `check-sli-query` |
| SLO-MAT-003 | SLO | target and window | `docs/slos/workflow-engine.openslo.yaml` | `check-slo-window` |
| SLO-MAT-004 | AlertPolicy | burn-rate condition | `docs/slos/workflow-engine.openslo.yaml` | `check-burn-rate-alerts` |
| SLO-MAT-005 | Annotation | data class | `oyatie.com/data-class` | `check-data-class` |
| SLO-MAT-006 | Annotation | ADR reference | `oyatie.com/adr` | `check-adr-links` |
| SLO-MAT-007 | Annotation | runbook path | `oyatie.com/runbook` | `check-runbook-linkage` |
| SLO-MAT-008 | Annotation | pack overlay | `oyatie.com/regulatory-pack` | `check-pack-overlay` |
| SLO-MAT-009 | Query | tenant-safe labels | PromQL | `check-metric-label-safety` |
| SLO-MAT-010 | Query | no high-cardinality actor label | PromQL | `check-metric-cardinality` |
| SLO-MAT-011 | Window | rolling or calendar declared | SLO object | `check-slo-window` |
| SLO-MAT-012 | Target | decimal target | SLO objective | `check-slo-target` |
| SLO-MAT-013 | Budget | budgeting method declared | SLO object | `check-error-budget-method` |
| SLO-MAT-014 | Dashboard | panel id linked | dashboard annotation | `check-dashboard-linkage` |
| SLO-MAT-015 | Runbook | severity path linked | runbook annotation | `check-runbook-linkage` |
| SLO-MAT-016 | Ownership | service owner exists | service catalog | `check-service-owner` |
| SLO-MAT-017 | Release | rollback signal declared | release manifest | `check-rollback-signal` |
| SLO-MAT-018 | Audit | SLO change event emitted | audit chain | `check-audit-emission` |
| SLO-MAT-019 | Pack | regulated target override explicit | pack overlay | `check-pack-slo-overlay` |
| SLO-MAT-020 | Promote | checker output in evidence | VCS bundle | `retired VCS ratchet` |

## Extended SLO Evidence Ledger

SLO-EVID-001. Record SLO object name.

SLO-EVID-002. Record service object name.

SLO-EVID-003. Record SLI object name.

SLO-EVID-004. Record alert policy name.

SLO-EVID-005. Record service owner.

SLO-EVID-006. Record related ADR.

SLO-EVID-007. Record runbook path.

SLO-EVID-008. Record dashboard path.

SLO-EVID-009. Record data class.

SLO-EVID-010. Record regulatory pack overlay.

SLO-EVID-011. Record SLI query hash.

SLO-EVID-012. Record SLI query unit.

SLO-EVID-013. Record SLI good-event expression.

SLO-EVID-014. Record SLI total-event expression.

SLO-EVID-015. Record target percentage.

SLO-EVID-016. Record time window duration.

SLO-EVID-017. Record window type.

SLO-EVID-018. Record budgeting method.

SLO-EVID-019. Record burn-rate threshold.

SLO-EVID-020. Record lookback window.

SLO-EVID-021. Record page route.

SLO-EVID-022. Record ticket route.

SLO-EVID-023. Record rollback condition.

SLO-EVID-024. Record freeze condition.

SLO-EVID-025. Record no-high-cardinality-label check.

SLO-EVID-026. Record tenant-safe-label check.

SLO-EVID-027. Record Prometheus syntax check.

SLO-EVID-028. Record OpenSLO schema check.

SLO-EVID-029. Record dashboard panel check.

SLO-EVID-030. Record runbook anchor check.

SLO-EVID-031. Record alert dry-run result.

SLO-EVID-032. Record historical burn-rate sample.

SLO-EVID-033. Record pack override sample.

SLO-EVID-034. Record CI checker crate version.

SLO-EVID-035. Record VCS changeset id.

SLO-EVID-036. Record promote bundle id.

## Extended Alerting Anti-Patterns

SLO-APX-001. Page on every single failed request instead of budget burn.

SLO-APX-002. Alert without a runbook path.

SLO-APX-003. Use actor id as a metric label.

SLO-APX-004. Use tenant name instead of tenant-safe tier label.

SLO-APX-005. Declare a target without a window.

SLO-APX-006. Declare a window without budget method.

SLO-APX-007. Copy a PromQL query without service owner review.

SLO-APX-008. Set regulated-pack target lower than canonical-base target without ADR.

SLO-APX-009. Treat dashboard existence as alert validation.

SLO-APX-010. Promote SLO changes without burn-rate dry run.

## Extended Promotion Review Checklist

SLO-PROMOTE-001. SLO object name is stable.

SLO-PROMOTE-002. Service object name is stable.

SLO-PROMOTE-003. SLI object name is stable.

SLO-PROMOTE-004. Alert policy name is stable.

SLO-PROMOTE-005. Service owner is recorded.

SLO-PROMOTE-006. Related ADR is recorded.

SLO-PROMOTE-007. Runbook path is recorded.

SLO-PROMOTE-008. Dashboard path is recorded.

SLO-PROMOTE-009. Data class is recorded.

SLO-PROMOTE-010. Regulatory pack overlay is recorded.

SLO-PROMOTE-011. SLI query hash is recorded.

SLO-PROMOTE-012. SLI query unit is recorded.

SLO-PROMOTE-013. Good-event expression is recorded.

SLO-PROMOTE-014. Total-event expression is recorded.

SLO-PROMOTE-015. Target percentage is recorded.

SLO-PROMOTE-016. Time window duration is recorded.

SLO-PROMOTE-017. Window type is recorded.

SLO-PROMOTE-018. Budgeting method is recorded.

SLO-PROMOTE-019. Burn-rate threshold is recorded.

SLO-PROMOTE-020. Lookback window is recorded.

SLO-PROMOTE-021. Page route is recorded.

SLO-PROMOTE-022. Ticket route is recorded.

SLO-PROMOTE-023. Rollback condition is recorded.

SLO-PROMOTE-024. Freeze condition is recorded.

SLO-PROMOTE-025. Metric label safety passes.

SLO-PROMOTE-026. Prometheus syntax passes.

SLO-PROMOTE-027. OpenSLO schema validation passes.

SLO-PROMOTE-028. Dashboard panel check passes.

SLO-PROMOTE-029. Runbook anchor check passes.

SLO-PROMOTE-030. Alert dry-run passes.

SLO-PROMOTE-031. Historical burn-rate sample is attached.

SLO-PROMOTE-032. Pack override sample is attached.

SLO-PROMOTE-033. Checker crate version is recorded.

SLO-PROMOTE-034. VCS changeset id is recorded.

SLO-PROMOTE-035. Promote bundle id is recorded.

SLO-PROMOTE-036. Error-budget owner is recorded.

SLO-PROMOTE-037. Incident severity mapping is recorded.

SLO-PROMOTE-038. Release freeze mapping is recorded.

SLO-PROMOTE-039. Rollback gate mapping is recorded.

SLO-PROMOTE-040. Promotion evidence includes OpenSLO checker output.
