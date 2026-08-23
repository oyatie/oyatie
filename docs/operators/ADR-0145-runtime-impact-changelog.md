# ADR-0145 runtime-impact changelog (for operators)

**Status**: Active 2026-05-18
**Source ADR**: [ADR-0145 — inter-microservice communication reform](../decisions/ADR-0145-inter-microservice-communication-reform.md)
**Audience**: operators, SREs, incident commanders.

This document captures what changes for operators when ADR-0145 (hyperscaler-shape inter-microservice communication) lands. Pair this with the [microservice-migration-guide-adr-0145.md](microservice-migration-guide-adr-0145.md) for the per-µservice adoption steps.

## What changed

Previously, every cross-µservice call was REQUIRED to flow through Workflow (orchestration) + Ontology (info). Under ADR-0145, direct sibling-µservice gRPC under mTLS is permitted. Workflow remains available as an opt-in product (similar to AWS Step Functions).

## Direct operator impact

### 1. NetworkPolicy egress rules

- Direct egress to sibling µservices' gRPC endpoints is now permitted.
- ALL `microservices/*/iac/helm/*/templates/networkpolicy.yaml` files are being reviewed and relaxed where appropriate.
- The `ats` namespace egress from the community jobs/recruiter surface was previously refused (carrier-exemption gap); per ADR-0145 it is now compliant. The active networkpolicy currently routes through workflow-engine pending the first Tier-G ATS tenant onboarding (see microservices/community/PRD.md Note "ATS µservice activates per ADR-0132 forward policy").

### 2. Service-mesh requirements (Cilium primary + Istio Ambient Tier-2 per ADR-0148)

- Every µservice now requires `ciliumnetworkpolicy.yaml` declaring identity-based L4 rules + (where applicable) L7 HTTP/gRPC rules. STRICT mTLS is the cluster-level default.
- Every µservice declares its `policy/tenant-scope.cedar` fragment; the cell µservice's policy-compiler emits CNP rules (and, for Tier-2 namespaces, Istio Ambient AuthorizationPolicy) from the same Cedar source.
- Tier-2 namespaces (initially `workflow-engine`, `foundry-orchestrator`) additionally ship `istio-waypoint.yaml` + `istio-authorizationpolicy.yaml` for advanced L7 traffic management.
- Mesh overhead: Cilium agent runs as a node DaemonSet (no per-pod sidecar) — fleet-wide RAM/CPU savings vs Istio sidecar mode. Tier-2 Ambient waypoint is a per-namespace singleton (~40-80MB RAM + ~50m CPU per waypoint, not per pod).

### 3. Audit-chain integration (Invariant 1)

- Every state-changing inter-µservice call MUST emit an audit-chain seal at the CALLING site.
- Each µservice integrates `shared-audit-chain-client-kernel` (skeleton authored; production impl tracked under `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-client-impl`).
- During the skeleton phase, the gate `presubmit` (retired CLI `gate validate audit-chain-seal-coverage`) runs in DEFERRED (advisory) mode.

### 4. Trace propagation (Invariant 2)

- Every inter-µservice call MUST propagate the W3C `traceparent` header.
- Hubble's OpenTelemetry exporter + the canonical `shared-tracing-client-kernel` integrate the surface.
- Tempo backend (in `observability` µservice) is the destination.
- Gate `presubmit` (retired CLI `gate validate otel-trace-propagation`) runs in DEFERRED (advisory) mode pending strict-mode parser landing.

### 5. Ontology projection (Invariant 3)

- µservices that own canonical entities (Person, Task, Document, Recording, etc.) MUST declare `ontology_projections: [...]` in their `manifest.json`.
- Schema updated at `specs/microservices/manifest-schema.json`.
- Gate `presubmit` (retired CLI `gate validate ontology-projection-coverage`) runs in DEFERRED (advisory) mode.

## Rollback

ADR-0145 is reversible:

```bash
git revert <merge-commit-of-adr-0145>
```

State-change analysis:

- **Audit-chain seals**: append-only; no rollback corruption risk. Old seals remain valid.
- **Ontology projections**: idempotent re-writes from canonical source. Rollback re-projects from the canonical entity source; no data loss.
- **NetworkPolicy egress relaxations**: reverting tightens egress; no in-flight call breakage (Cilium mesh policy is identity-aware and drains gracefully on policy change).
- **Mesh dataplane changes**: reverting removes Cilium mesh policy/identity configuration (CNI dataplane stays intact). For Tier-2 Istio Ambient namespaces, revert removes the waypoint; in-flight mTLS drains via ztunnel's drain window.

No one-way state changes. The revert is safe.

## Day-2 observability checklist

After ADR-0145 lands and the first µservice migrates, operators should confirm:

- [ ] Tempo backend shows trace spans crossing the migrated µservice → at least one sibling.
- [ ] audit-chain query API returns seals for the migrated µservice's state-changing capabilities.
- [ ] ontology projection lag (per `manifest.json#ontology_projections.lag_budget_seconds`) sits within budget.
- [ ] Cilium agent (node DaemonSet) memory/CPU stays within the capacity-model bound; Tier-2 Ambient waypoint footprint (per-namespace singleton) stays within bound for enrolled namespaces.
- [ ] No P99 latency regression on direct-gRPC calls vs the previous Workflow-mediated path.

## Incident response

If an inter-µservice call fails after ADR-0145 lands:

1. **Check mTLS** — `cilium identity get <id>` + `cilium endpoint list` confirms SPIFFE-ID-aware identity binding. For Tier-2 Ambient namespaces, `istioctl ztunnel-config secret` confirms waypoint mTLS.
2. **Check policy** — `cilium policy get` + `hubble observe --drop` flags identity/policy mismatches; for Tier-2 namespaces, `istioctl analyze` flags AuthorizationPolicy mismatches.
3. **Check W3C traceparent** — Tempo span search by `traceparent` correlates caller and callee (Hubble OTel exporter is the source).
4. **Check audit-chain seal** — query the audit-chain µservice for the calling capability's recent seal.
5. **Rubric check** — was this call routed correctly per `docs/standards/workflow-vs-direct-grpc-rubric.md`? Synchronous reads on Workflow are an anti-pattern.

## References

- ADR-0145 — inter-microservice communication reform.
- ADR-0148 — Cilium Service Mesh (primary) + Istio Ambient waypoint (Tier-2 opt-in).
- docs/standards/workflow-vs-direct-grpc-rubric.md — when to use which path.
- docs/operators/microservice-migration-guide-adr-0145.md — per-µservice 6-step adoption.
- registry/placeholder-debt/adr-follow-ups.yaml — skeleton-impl tracking.
