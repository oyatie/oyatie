---
id: ADR-0145
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-ontology, axis-workflow-engine
date: 2026-05-18
owner: council-architecture
supersedes: [ADR-0140, ADR-0141]
superseded_by: [ADR-701]
amended_by: [ADR-0245, ADR-0252, ADR-0257, ADR-0280]
related: [ADR-0064, ADR-0131, ADR-0132, ADR-0135, ADR-0136, ADR-0139]
related_specs: [/specs/microservices/manifest-schema.json, /specs/hyperscaler-architecture-invariants.json]
retires_feedback_memory: feedback_workflow_objectgraph_adapter_layer
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0145 — Inter-microservice communication: hyperscaler shape with opt-in Workflow + Ontology

## Status

Accepted (2026-05-18). Supersedes the auto-memory `feedback_workflow_objectgraph_adapter_layer` directive ("all inter-product adapters flow through Workflow + Ontology; products never call each other directly").

## Context

Per `feedback_workflow_objectgraph_adapter_layer` (recorded 2026-05-14), oyatie's prior operating rule was that **all** inter-µservice communication must flow through Workflow (orchestration) + Ontology (info); µservices were forbidden from calling each other directly.

PR #143 idea-refine review (2026-05-18) surfaced this pattern as the #1 12-month regret with ~70% probability. The review found:

- AWS, Google, Microsoft, Stripe, Anthropic do NOT use a universal mediator pattern. They use direct service-to-service gRPC/HTTP with mTLS + per-service contracts.
- Palantir's Foundry DOES have an Ontology substrate, but Palantir's Workflow is opt-in orchestration — it does not mediate every read.
- Putting orchestration in the data path (ESB 2.0) makes Workflow the platform SLO ceiling and failure perimeter.

Per user directive 2026-05-18 ("Full Hyper scalar shape. workflow like step functions"), oyatie adopts the hyperscaler shape.

## Decision

**Three weaker invariants replace the universal-mediator rule.**

### Invariant 1 — Audit invariant (decentralized)

Every state-changing inter-µservice call MUST emit an audit-chain seal at the **calling** service (NOT at a mediator). Each µservice owns its own audit emission. The audit-chain µservice provides canonical seal storage but does NOT mediate the call itself.

Enforcement: per-µservice `oya-shared-audit-chain-client-kernel` trait. Validated by `oya-check-audit-chain-seal-coverage` lane (every state-changing capability declares its seal point).

### Invariant 2 — Tracing invariant

Every inter-µservice call MUST propagate the OpenTelemetry trace context for distributed observability. Cross-µservice flow traceability lives in the tracing system (Tempo), not in a central mediator.

Enforcement: per-µservice `oya-shared-tracing-client-kernel` trait. Validated by `oya-check-otel-trace-propagation` lane.

### Invariant 3 — Ontology projection invariant

µservices that own canonical entities (Person, Task, Document, Recording, etc.) MUST project them into Ontology for cross-µservice queryability. Ontology IS the canonical READ substrate for cross-µservice entity data.

But: Ontology is a SUBSTRATE, not a GATEWAY. µservices may also call each other directly via mTLS gRPC for transactional/synchronous needs; Ontology query is the preferred path for cross-µservice entity reads where latency budget permits.

Enforcement: per-µservice manifest.json declares `ontology_projections: [...]`. Validated by `oya-check-ontology-projection-coverage`.

### Workflow becomes opt-in (AWS Step Functions / Google Cloud Workflows model)

µservices MAY use Workflow for durable long-running cross-µservice orchestration (sagas, retry-with-backoff, human-in-the-loop, multi-step transactions). Workflow is NOT a mandatory mediator. Direct µservice-to-µservice calls are permitted under invariants 1+2+3.

### Rubric: when to use Workflow vs direct gRPC

Use **Workflow** when the call has ANY of these properties:

1. Durable execution required (retries spanning hours; state persists across pod restart).
2. Multi-step transaction with rollback (saga pattern).
3. Long-running with human-in-loop (approval steps, manual review).
4. Async with audit-chain causal ordering across multiple µservices.
5. Multi-tenant fan-out with bounded concurrency.

Use **direct gRPC** when ALL of these are true:

1. Synchronous request-response; latency budget under ~2 seconds.
2. Latency-sensitive (P99 < 500ms).
3. Read-only OR at-most-once semantics OK.
4. Transient state acceptable; caller retries idempotently.
5. Single-hop or fan-out without saga.

The full rubric with worked examples lives at `docs/standards/workflow-vs-direct-grpc-rubric.md`.

### Service-mesh substrate

Direct gRPC under mTLS requires a service-mesh tool. **Cilium Service Mesh** (sidecarless eBPF, single-project parity with the Cilium CNI already adopted in ADR-0121) is the canonical PRIMARY substrate; **Istio Ambient waypoint** is an opt-in per-namespace Tier-2 overlay for the small set of µservices needing advanced L7 traffic management — see `ADR-0148-service-mesh-cilium.md` for alternatives considered (Istio classic sidecar, Istio Ambient as-primary, Linkerd, AWS App Mesh, no-mesh) and rationale.

### Direct sibling-µservice egress is permitted

NetworkPolicy egress rules MAY allow direct egress to sibling µservices' gRPC endpoints, subject to:
- mTLS authentication (per cell-µservice SPIFFE-ID issuance)
- Cedar policy authorization (per-call)
- Invariants 1+2 (audit + tracing emission)

## Alternatives considered

### A. Keep universal-mediator rule (status quo before this ADR)
- Pros: centralized audit/auth; single point of observability
- Cons: Workflow becomes platform SLO ceiling and failure perimeter; ESB 2.0 anti-pattern; doesn't match what hyperscalers do; contradicts user's hyperscaler-bar directive
- **Rejected**: 70% 12-month regret probability per PR #143 idea-refine review

### B. Read-path-direct + write-path-Workflow (half-measure)
- Pros: keeps write-path centralized; reads scale
- Cons: still half-mediator; complexity remains; Workflow still bottleneck for writes; defends a wrong default
- **Rejected**: half-measures don't honor "no deferrals" doctrine

### C. Cross-cutting carriers exemption only (ADR-0140-cross-cutting-carriers, partial)
- Pros: small change; keeps most of the existing rule
- Cons: still imposes mediator for app-tier-to-app-tier flows; doesn't address the fundamental anti-pattern
- **Rejected**: partial mitigation; covers symptoms not cause

### D. Pure hyperscaler shape with 3 weaker invariants (accepted)
- Pros: matches AWS / Google / Microsoft / Stripe / Anthropic shape; Ontology aligns with Palantir's actual model; Workflow becomes a product like AWS Step Functions; no platform SLO ceiling; scales to 1000+ µservices
- Cons: distributed audit/auth requires discipline; cross-µservice flow observability via distributed tracing requires OTel discipline
- **Accepted**: aligns with user directive; honors hyperscaler-bar standard

### E. Drop Ontology entirely (services own their data canonical-source)
- Pros: maximum decentralization; matches AWS-pure
- Cons: cross-µservice entity queryability requires per-pair contracts; loses Palantir-class semantic substrate benefit
- **Rejected**: Ontology has real value as a SUBSTRATE; just shouldn't be a GATEWAY

## Consequences

### Positive

1. **Hyperscaler-bar alignment** — matches what AWS/Google/Microsoft/Stripe/Anthropic do. Closes the #1 12-month regret surfaced in PR #143 review.
2. **Scalability** — no platform SLO ceiling. µservices scale independently. Cross-µservice flows scale linearly in the number of services + edges, not in Workflow's central capacity.
3. **Operational simplicity** — direct service-to-service calls are simpler to reason about, debug, and operate. Distributed tracing (OTel) provides cross-flow observability.
4. **Ontology aligns with Palantir's actual model** — substrate, not gateway. Performance budget for Ontology queries is hot-path; not every inter-µservice call hits it.
5. **Workflow becomes a product** — like AWS Step Functions / Google Cloud Workflows / Temporal Cloud. µservices opt in for durable long-running flows. Workflow's own SLO is its own concern.
6. **Cross-cutting carriers no longer need exemption** — drive/mail/messenger/calendar/recordings are just µservices like any other. Their cross-µservice traffic flows under the 3 invariants.

### Negative

1. **Distributed audit discipline required** — every µservice authors `oya-shared-audit-chain-client` integration. Per-service seal emission must be tested.
2. **Distributed tracing discipline required** — OTel trace propagation must be enforced via `oya-check-otel-trace-propagation` lane.
3. **Per-µservice contract surface grows** — each µservice's contracts/openapi or contracts/proto carries the surfaces siblings call. No longer a centralized Workflow-engine contract.
4. **Cell-µservice load grows** — SPIFFE-ID issuance handles every inter-µservice mTLS handshake (not just edge ingress). Capacity-model updated.
5. **Cedar policy authoring grows** — each µservice declares which sibling-µservice principals may call which actions. (Mitigated by canonical sibling-call Cedar policy fragments in governance.)

### Operational

1. ALL existing `microservices/*/iac/helm/*/templates/networkpolicy.yaml` files reviewed + relaxed where appropriate. Direct sibling-µservice egress permitted under invariants.
2. PR #143's `network → ats` and `meet → recordings` networkpolicy entries (flagged by integration review as Workflow+Ontology bypass violations) are RE-CLASSIFIED as compliant under ADR-0145. The integration findings for those 2 violations are RESOLVED-AS-NO-LONGER-APPLICABLE.
3. Fix-C agent's ADR-0140-cross-cutting-carriers-adapter-exemption (in-flight at ADR-0145 authoring time) is SUBSUMED. Carriers no longer need exemption because direct egress is permitted.
4. Fix-D agent's ADR-0141-workflow-ontology-read-path-direct (queued for authoring) is SUBSUMED. The 3 weaker invariants make the read-path-direct split irrelevant.
5. `feedback_workflow_objectgraph_adapter_layer` auto-memory marked superseded.
6. CLAUDE.md updated to reference ADR-0145 in place of the old "products never call each other directly" assertion.

## Rollback

ADR-0145 is reversible. Operators revert via:

```bash
git revert <merge-commit-of-this-ADR-and-the-skeleton-PR>
```

State-change one-way analysis (per shipping-readiness checklist):

1. **Audit-chain seals are append-only.** No rollback corruption risk — old seals remain valid as Ed25519-signed leaves of the Merkle tree. Reverting stops new seals on the relaxed code paths; existing seals stay.
2. **Ontology projections are idempotent re-writes.** The projection target tables are re-derivable from the canonical entity source (the entity-owning µservice's own database). Rollback re-runs the canonical projection pipeline; no data loss.
3. **NetworkPolicy egress relaxations are reversible.** Reverting tightens egress; in-flight connections drain via Istio's `terminationDrainDuration` (default 5s, configurable).
4. **Mesh dataplane changes are reversible.** Cilium agent runs as a node DaemonSet — revert removes mesh policy/identity configuration but keeps the CNI dataplane intact. For Tier-2 Istio Ambient namespaces, revert removes the waypoint; mTLS connections drain via Ambient's ztunnel drain window.
5. **Cedar policy fragment changes are reversible.** Cedar fragments are pure-text data; revert restores the prior fragment. No persisted state depends on a specific fragment version.

No one-way state changes. The revert is operationally safe.

If a partial revert is required (e.g. only one µservice rolls back while others stay on the new shape):

- The skeleton-mode audit/tracing clients are designed to be removable per-µservice — they're path-dependencies, not workspace-wide deps.
- The advisory-mode gates (`otel-trace-propagation`, `ontology-projection-coverage`, `audit-chain-seal-coverage`) do not error on partial rollouts; they surface findings without failing CI.

Operator runtime-impact details live in `docs/operators/ADR-0145-runtime-impact-changelog.md`; the per-µservice 6-step adoption guide lives in `docs/operators/microservice-migration-guide-adr-0145.md`.

## References

- AWS Well-Architected Framework — "Use direct service-to-service communication; reserve orchestration for genuinely cross-cutting durable flows" (per AWS Reliability Pillar 2024 + AWS Step Functions design guide).
- Google SRE Workbook Chapter 1 — Stubby/gRPC direct service-to-service with mTLS + IAM; orchestration via Borg scheduling, not API mediation.
- Microsoft Service Fabric Reliable Services — direct service-to-service.
- Stripe engineering blog — Sorbet + mTLS + Twirp/gRPC direct; no universal mediator.
- Anthropic engineering practices (public statements) — direct dependencies between Console/API/Apps services; no publicly described universal mediator.
- Palantir Foundry Ontology — canonical data substrate, NOT a gateway. Workflow Studio is opt-in orchestration product.
- AWS Step Functions design — opt-in durable orchestration; not a universal API mediator.
- Google Cloud Workflows — same pattern; opt-in.
- Temporal — opt-in durable execution; not a universal mediator.
- ESB 2.0 anti-pattern critique — Martin Fowler 2014 "Microservices and the First Law of Distributed Object Design" + IEEE 2017 "Why Enterprise Service Buses Failed."
- OpenTelemetry specification — distributed tracing as the cross-flow observability primitive.
- SPIFFE/SPIRE — workload identity for mTLS; cell-µservice integration.
- Cedar v4.2 LTS — per-call authorization with sibling-µservice principal types.
- PR #143 review evidence: `evidence/pr-143-review-idea-refine.json`, `evidence/pr-143-review-integration.json`.

## Numbering note

ADR-0140, ADR-0141, ADR-0142, ADR-0143, ADR-0144 numbers were ALLOCATED to Fix-C and Fix-D agents during PR #143's "no-deferrals" remediation sweep. ADR-0145 supersedes those allocations conceptually:
- ADR-0140-cross-cutting-carriers-adapter-exemption (Fix-C) → SUBSUMED by ADR-0145 (no exemption needed)
- ADR-0141-workflow-ontology-read-path-direct (Fix-D) → SUBSUMED by ADR-0145
- ADR-0142-crdt-portability-trait (Fix-D) → INDEPENDENT (CRDT concern); proceeds as-is
- ADR-0143-intelligence-per-bc-release-pointer (Fix-D) → INDEPENDENT (foundry concern); proceeds as-is
- ADR-0144-eu-ai-act-graduated-risk-tier-model (Fix-D) → INDEPENDENT (EU AI Act concern); proceeds as-is

Post-Fix-C/Fix-D landing, mark ADR-0140 and ADR-0141 status `superseded_by: ADR-0145` in their frontmatter.

## Historical residual from ADR-140 (E3 fold 2026-08-06)

**Title:** ADR-0140-cross-cutting-carriers-adapter-exemption

**Preserved decision gist:** oyatie defines **CROSS-CUTTING CARRIERS** as a distinct µservice class with five charter members and a defined exemption to the Workflow+Ontology adapter rule: | Carrier µservice | Carries on behalf of | Payload shape | |---|---|---| | `drive` | every µservice that attaches a file | binary blob + content_hash + tenant-DEK envelope | | `mail` | every µservice that share-by-emails | RFC 5322 message + S/MIME envelope | | `messenger` | every µservice that channel-mentions or notify-via-DMs | typed message + reaction + read-receipt | | `calendar` | every µservice that binds a time-slot, due-date, 

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-59 (E3 fold 2026-08-06)

**Title:** ADR-0059-workflow-ontology-ecosystem-adapter-layer

**Preserved decision gist:** **All adapters in the oyatie ecosystem exist through Workflow and Ontology.** Together they are the canonical adapter/integration surface. **Microservices never call each other directly.** Any cross-product integration flows through one of these two primitives: - **Workflow** — the **action/orchestration adapter**. Cross-product or intra-product action flows: state machines, DAGs, approvals, escalations, SLA timers, automation, handoffs. Microservices publish typed events; Workflow routes them; consuming microservices subscribe. - **Ontology** — the **information/data adapter** (= Palantir Ont

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-141 (E3 fold 2026-08-06)

**Title:** ADR-0141-workflow-ontology-read-path-direct

**Preserved decision gist:** The Workflow + Ontology adapter rule is amended to a **read path / write path split**: 1. **WRITE path: orchestrated.** Every state-changing inter-µservice call (CREATE / UPDATE / DELETE; any operation that emits an audit row; any operation that crosses a Cedar admission boundary on Action::write_*) MUST flow through Workflow. Workflow remains the canonical orchestrator for compensation, retry, dead-letter, idempotency- key persistence, and audit-chain sealing. 2. **READ path: direct.** Every read query against Ontology entities (Action::read_*, Action::list_*, Action::query_*) MAY flow direct

_Source file archived after fold; full body in git history / docs/adr-archive/._
