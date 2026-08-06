---
id: ADR-0141
status: Superseded
deciders: council-architecture, axis-workflow, axis-ontology, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0145]
related: [ADR-0056, ADR-0064, ADR-0067, ADR-0128, ADR-0131, ADR-0139, ADR-0140]
related_memory: [feedback_workflow_objectgraph_adapter_layer, feedback_quality_performance_scalability_bar, feedback_glossary_ontology_not_object_graph, feedback_flat_product_catalog]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/agentic-slo-gated-promotion.json
purpose: |
  Split the "every cross-µservice call goes through Workflow + Ontology" rule
  into a READ path (direct cell-bounded gRPC to Ontology) vs. a WRITE path
  (orchestrated through Workflow). Mitigates the Google Stubby-class concern
  that Workflow becomes the platform's SLO ceiling.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0141: Workflow + Ontology — read path direct; write path orchestrated

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

The standing rule from `feedback_workflow_objectgraph_adapter_layer.md`
("Workflow + Ontology = adapter layer"): all inter-product adapters
flow through Workflow (orchestration) + Ontology (information);
products never call each other directly.

This rule is correct for **state-changing** operations — it gives a
single audit point, a single Cedar evaluation seam, and a single
retry/compensation harness. But applied uniformly to **read** queries,
the rule degrades cross-µservice latency by:

1. **Stubby-class SLO ceiling.** Every microservice's read SLO becomes
   bounded by Workflow's SLO. Google internally hit this exact ceiling
   with its Stubby RPC orchestrator (per the Borg paper and the SRE
   book ch. 11) and addressed it by allowing first-class direct RPCs
   for reads alongside the orchestrated write path. AWS makes the same
   split in its Builders' Library article *"Avoiding overload in
   distributed systems"*: synchronous reads bypass the orchestration
   tier; only writes traverse it.

2. **Indirect adapter overhead.** Each Workflow hop adds at minimum
   one serialization round-trip, one Cedar evaluation, one queue/
   dispatch boundary, and one observability span. For the read paths
   that satisfy >60% of the cross-microservice latency budgets
   established in `docs/standards/cross-microservice-latency-budget.md`,
   that overhead consumes 30-150 ms per hop — multiple budgets break
   if every read pays it.

3. **Cell-bounded reads naturally collide with Ontology placement.**
   Per ADR-0131 per-microservice flat layout + the cell substrate
   (Bominal ADR-0009 inherited), Ontology entities are physically
   co-located with their owning µservice's cell. A direct gRPC from
   `social` → `ontology` within the same cell is a 1-2 ms intra-cell
   call; the same call routed through Workflow becomes a 30+ ms hop
   even on the happy path.

The autonomous-implementation goal
(`feedback_autonomous_implementation_artifacts.md`) and the
hyperscaler-grade bar (`feedback_quality_performance_scalability_bar.md`)
together demand we resolve this before scale-up; the cost of
retrofitting a read/write split later is materially higher than
making the split policy at M01.

## Decision

The Workflow + Ontology adapter rule is amended to a **read path /
write path split**:

1. **WRITE path: orchestrated.** Every state-changing inter-µservice
   call (CREATE / UPDATE / DELETE; any operation that emits an audit
   row; any operation that crosses a Cedar admission boundary on
   Action::write_*) MUST flow through Workflow. Workflow remains the
   canonical orchestrator for compensation, retry, dead-letter, idempotency-
   key persistence, and audit-chain sealing.

2. **READ path: direct.** Every read query against Ontology entities
   (Action::read_*, Action::list_*, Action::query_*) MAY flow direct
   from caller µservice to the Ontology µservice via cell-bounded gRPC,
   subject to:

   a. **Cedar admission at Ontology's ingress.** The Ontology µservice
      MUST evaluate Cedar against the caller's principal and
      requested resource. The caller MUST attach an OIDC service-to-
      service token (per the cloud-iam substrate). Skipping Workflow
      does NOT skip Cedar.

   b. **Cell boundary respected.** Direct reads MUST originate from
      within the same cell as the target Ontology partition. Cross-
      cell reads continue to flow through Workflow so the cross-cell
      orchestration accounting remains complete.

   c. **Audit-chain row optional, sampled.** Read operations are NOT
      sealed individually (per the audit-chain doctrine — sealing
      every read would 10x the chain volume). The Ontology µservice
      emits per-tenant per-day read-summary rows to the audit chain
      (1 row per 10 000 reads or per day, whichever first).

   d. **The seven canonical metrics still emit.** Direct read calls
      MUST emit `oya_<caller>_responses_total`, `oya_<caller>_request_success_total`,
      `oya_<caller>_request_total` (per the hyperscaler metric
      naming convention) so the SLO substrate sees the traffic.

## Alternatives considered

### Alternative 1: Status quo — all cross-µservice traffic through Workflow

- **Pros:** One adapter rule, one audit seam, simplest mental model.
- **Cons:** Workflow becomes the platform-wide SLO ceiling; every
  per-hop latency budget at `docs/standards/cross-microservice-
  latency-budget.md` breaks by 30-150 ms; the autonomous-implementation
  goal cannot ship hyperscaler-grade reads.
- **Rejected because:** Demonstrably exceeds the per-hop p99 budgets
  for Flow A, Flow B, Flow C even before traffic-shaping overhead.

### Alternative 2: Skip Workflow for both reads AND writes

- **Pros:** Maximum performance; no orchestration overhead.
- **Cons:** Compensation, retry, dead-letter, idempotency-key
  persistence have to be re-implemented per caller; cross-µservice
  audit row gaps appear; Cedar evaluation surfaces multiply 10x.
- **Rejected because:** This is the n8n / Zapier mistake — losing
  the orchestrator entirely lost them the ability to ship reliable
  compensation, which is one of Workflow Studio's primary moats per
  `feedback_workflow_studio_scope.md`.

### Alternative 3: Read-replica fan-out with no direct gRPC

- **Pros:** Reads always hit a local replica; very fast.
- **Cons:** Replication lag becomes user-visible (read-your-writes
  violations); replicas multiply storage cost by N-1; the canonical
  Ontology data model (Palantir-class entity graph) doesn't admit
  trivial replication.
- **Rejected because:** Linear, Stripe, and Anthropic Console all
  serve their read paths from primary stores (per public engineering
  blogs 2022–2024); the latency budget already accommodates primary
  reads, and the operational simplicity of one source of truth
  outweighs the marginal latency gain.

## Consequences

### Positive

1. **Cross-microservice latency budgets become achievable.** The 1 s
   social-post p99, the 500 ms messenger DM p99, and the 1.5 s task-
   creation p99 all become budget-compliant once read hops drop from
   60-180 ms (through Workflow) to 5-30 ms (direct).
2. **Workflow SLO surface shrinks to writes only.** ADR-0139 SLO-gated
   promotion can refuse the Workflow release pointer based on write-
   path burn-rate alone; read-path burn-rate is owned per Ontology
   partition. This is the same shape as Google's Borg → Borgmon
   split (write traffic governed by control plane; read traffic
   governed by data plane).
3. **Ontology's read API becomes the canonical query surface.**
   Future products (Workflow Studio, Search, Vector) consume the
   Ontology read API directly, matching how Palantir AIP consumes
   the Ontology service.

### Negative

1. **Two adapter rules instead of one.** Engineers must internalise
   "is this a read or write?" before choosing the path. Mitigation:
   Cedar admission paths name actions Action::read_*, Action::write_*;
   the path choice is deterministic from the action prefix.
2. **Cedar evaluation now happens at two seams** (Workflow for writes,
   Ontology ingress for reads). Mitigation: the canonical Cedar
   policy fragments at `microservices/<ms>/policy/*.cedar` are the
   single source; both seams import them via the canonical import
   envelope.
3. **Cross-cell reads remain orchestrated.** Engineers building
   federated queries must still understand the cell-bounded read
   constraint. Mitigation: documented in the Ontology µservice's
   `microservices/ontology/runbooks/cross-cell-read.md` runbook (to
   be authored at M02-P09 substrate landing).

### Comparisons to industry-standard practice

- **AWS:** *Builders' Library*: "Reads should bypass orchestration
  unless they require cross-resource consistency." Direct precedent.
- **Google:** Borg → Stubby → Spanner: read traffic skips Stubby for
  in-cell reads via direct fronted-service-to-Spanner gRPC; only
  cross-region writes traverse Stubby. (Google SRE Workbook ch. 11.)
- **Anthropic:** Claude Console reads go direct to its primary
  DataStore (per the 2024 architecture overview); only writes hit
  the orchestrated tool/eval surface.
- **Palantir AIP:** Ontology service exposes a direct query API
  (consumed by AIP Logic, AIP Threads); only ontology *mutations*
  flow through Workflow orchestration. Direct precedent.

## References

- ADR-0056 — substrate architecture (port-in-kernel).
- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0067 — perf authority.
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0131 — per-microservice flat layout.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0140 — Cedar policy enforcement substrate (referenced by capabilities).
- `docs/standards/cross-microservice-latency-budget.md`.
- `feedback_workflow_objectgraph_adapter_layer.md` (the rule this ADR amends).
- Google SRE Workbook ch. 11 — managing load.
- AWS Builders' Library — *Avoiding overload in distributed systems*.
- Linear engineering blog 2023 — *Building a fast and reliable real-time sync engine*.
- Anthropic public engineering posts on Claude API architecture (2024).
- Palantir AIP product documentation — Ontology service direct query API.
