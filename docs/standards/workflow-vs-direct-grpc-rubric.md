# Workflow vs direct gRPC — when to use which

**Status**: Active 2026-05-18
**Owner**: council-architecture
**Source ADR**: [ADR-0145 — inter-microservice communication reform](../decisions/ADR-0145-inter-microservice-communication-reform.md)
**Related**: ADR-0148 (Cilium Service Mesh primary + Istio Ambient Tier-2 waypoint), ADR-0131 (per-microservice flat layout)

## Purpose

ADR-0145 permits BOTH direct sibling-µservice gRPC AND opt-in Workflow orchestration. This rubric decides which path a particular cross-µservice call should take.

The decision matters because the wrong default — universal Workflow mediation — was the #1 12-month regret surfaced in the PR #143 idea-refine review. The right defaults keep Workflow as a *product* (like AWS Step Functions / Google Cloud Workflows / Temporal Cloud) rather than a *gateway*.

## Decision rubric

Use **Workflow** when the call has ANY of these properties:

1. **Durable execution required.** Retries with backoff that may span hours. Workflow persists run state; in-process retries lose state on pod restart.
2. **Multi-step transaction with rollback (saga).** Two or more cross-µservice writes that must succeed-together or rollback-together. Workflow's compensation API is the canonical saga primitive.
3. **Long-running with human-in-loop.** Approval steps, manual review, or wait-for-external-signal. Workflow's `Signal` and `Continue-as-new` primitives are designed for this.
4. **Async with audit-chain causal ordering.** When the audit-chain replay sequence requires a globally-ordered transaction log across multiple µservices. Workflow's deterministic replay grounds the ordering.
5. **Multi-tenant fan-out with bounded concurrency.** Workflow's `child workflow` + concurrency-limit annotations let you fan out N parallel calls per tenant under a global cap.

Use **direct gRPC** when ALL of these are true:

1. **Synchronous request-response.** Caller waits for the result; latency budget under ~2 seconds end-to-end.
2. **Latency-sensitive.** P99 budget under ~500ms; Workflow's persistence write adds ~50-200ms.
3. **Read-only OR at-most-once semantics OK.** No multi-step rollback required.
4. **Transient state acceptable.** A failed call retries idempotently from the caller; no need for durable retry-state across pod restarts.
5. **Single-hop or fan-out without saga.** No compensating action required if a single hop fails.

## Worked examples

### Example 1 — `network` publishes a job posting to ATS µservice

- Multi-µservice handoff with ack semantics required (network → ATS).
- ATS may be down for hours; replay-from-cursor required.
- Audit-chain causal ordering required across `network` and `ats` µservices.

**Decision**: Workflow. ATS handoff publishes `oyatie.network.jobposting.v1.published` to workflow-engine; workflow-engine routes to ATS; ATS ack returns via workflow-engine. This is the canonical pattern documented in IP-011-jobs-handoff-bc.md.

### Example 2 — `tasks` reads a Person entity from `ontology`

- Synchronous read; tasks blocks the user request on it.
- P99 budget under 50ms.
- No multi-step state change.

**Decision**: Direct gRPC. `tasks` calls `ontology` directly under mTLS with W3C traceparent injected, audit-chain seal emitted at `tasks` (Invariant 1).

### Example 3 — `meet` records a session → emits a recording artifact

- Two µservices change state: `meet` finalizes the session, `recordings` ingests + persists the artifact.
- Failure during ingestion must compensate the `meet` finalize (mark session pending-retry).
- Async; latency budget on the order of minutes.

**Decision**: Workflow. The two-step saga lives in workflow-engine; both µservices participate as workflow activities.

### Example 4 — `tasks` posts a comment notification to `messenger`

- Single hop; messenger queues internally for delivery.
- At-most-once is OK (messenger has its own retry queue).
- Synchronous from tasks' view; ~100ms.

**Decision**: Direct gRPC.

## Operational notes

- Direct gRPC calls must satisfy ADR-0145 Invariants 1 + 2 + 3: audit-chain seal at the calling side, W3C traceparent propagation, ontology projection (for entity-owning µservices).
- Workflow calls inherit Invariants 1 + 2 automatically because workflow-engine integrates the canonical clients (`shared-audit-chain-client-kernel`, `shared-tracing-client-kernel`).
- The mesh (Cilium primary + Istio Ambient Tier-2 per ADR-0148) enforces mTLS + Cedar authorization on BOTH paths. The rubric does not change the trust boundary.

## Anti-patterns

1. **"Just use Workflow because it's safer."** No — Workflow is a platform SLO ceiling for the calls that route through it. Putting a synchronous read on Workflow makes the read SLO worse than the direct path.
2. **"Just use direct gRPC because Workflow is heavy."** No — multi-step saga without Workflow leaks rollback responsibility into each µservice's REST surface, which is exactly the duplication ADR-0145 calls out.
3. **"We'll start direct and migrate to Workflow later."** Migration is expensive (every caller changes). Start with the rubric; revisit only when the workload shape genuinely changes.

## References

- ADR-0145 — inter-microservice communication reform.
- ADR-0148 — Cilium Service Mesh (primary) + Istio Ambient waypoint (Tier-2 opt-in).
- AWS Step Functions design guide — durable orchestration patterns.
- Google Cloud Workflows — opt-in orchestration.
- Temporal — durable execution patterns.
- Microservices and the First Law of Distributed Object Design — Martin Fowler 2014.
