---
id: ADR-0353
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - ops-sre-reliability
  - ops-compliance
  - axis-policy-engine
  - axis-tenancy
  - axis-identity
  - axis-audit-chain
  - axis-ontology
  - axis-intelligence
supersedes: []
amends:
  - ADR-0246-policy-engine-substrate-promotion.md
superseded_by: []
related:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0050-event-bus-and-outbox-canonical.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0355-amendment-library-first-network-opt-in-clarification.md
  - ADR-NNNN-library-first-credential-sidecar
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/cedar-fragment-schema.json
  - /specs/policy-gate-coverage.json
  - /specs/byok-credential-model.json
  - /specs/tenant-model.json
related_memory:
  - feedback_cedar_as_universal_gate
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_bominal_inheritance_precedence
  - feedback_autonomous_implementation_artifacts
doc_class: Architecture-Decision-Record-Amendment
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-library-first-symmetry-1-of-2
amendment_anchor: F4-LV-6 / F4-AP-1 / F-ANTI-2
enforcement_status: advisory-until-policy-engine-client-library-lands
enforced_by:
  - oya gate validate policy-engine-library-first-default
  - oya gate validate policy-engine-network-opt-in-cedar-gated
  - oya gate validate no-unnecessary-policy-engine-service-hop
  - oya gate validate library-only-evaluation-failure-perimeter
  - oya gate validate library-fragment-snapshot-coherence
  - oya gate validate library-audit-emission-coherence-policy-engine
  - oya gate validate library-credential-sidecar-coherence-policy-engine
---

# ADR-0353: Amendment — Library-First / Network-Opt-In Clarification

## Status

Proposed — 2026-05-20.

This is an **amendment** to ADR-0246 (Policy-Engine Substrate Promotion,
2026-05-20). It does not supersede ADR-0246; it clarifies the **delivery
shape** and **runtime call topology** of the Policy-Engine Substrate so
the substrate does not, by accident, re-introduce the universal-mediator
pattern that ADR-0145 retired and that the ADR-0255 amendment closed for
Intelligence.

The amendment is filed as a **promotion-gate fix (1 of 2)** per the
keystone-bundle 2026-05-20 synthesis §5.13 (F4 library-first symmetry).
The defect it prevents — `microservices/policy-engine/` becoming the
platform-wide per-call authorization gateway — is structurally identical
to the pre-ADR-0145 universal-mediator shape that F-ANTI-1 surfaced for
Intelligence and that the ADR-0255 amendment closed. ADR-0246 in its
current text, especially §D-3 ("Centralized µservice (per-cell evaluator
pool) … Chosen") and §D-5 ("downstream µservices call it over gRPC with
an SDK that holds a connection pool + circuit breaker + cache
fallback"), can be read by an implementer as mandating a per-call gRPC
hop to the cell-local evaluator pool on every state-changing action
across the portfolio. That reading must be foreclosed in writing before
any caller code is written against ADR-0246. Per F4-AP-1, the same
universal-gateway threat applies to Cedar evaluation as applied to LLM
dispatch, because ADR-0243 elevated Cedar to a hot-path consultation on
**every** state-changing inter-µservice call.

Enforcement is `advisory-until-policy-engine-client-library-lands`. CI
lanes that enforce this amendment promote to BLOCKER once:

1. The `oya-shared-policy-engine-client-*` crate family is scaffolded
   per §D-2 below with the **library-mode-default** evaluator embedded
   in-process (Cedar v4.2 Rust crate `cedar-policy` as the
   evaluation kernel), and at least one µservice (pilot:
   `microservices/tenancy/` per ADR-0246 §Status item 3) is demonstrated
   to consume the library for per-call evaluation without making a gRPC
   call to `microservices/policy-engine/`.
2. The `oya-check-no-unnecessary-policy-engine-service-hop` static
   analysis lane is authored and exercised against the
   `tenancy → policy-engine-client → evaluate(...)` reference path.
3. The `tenants` table includes the `policy_evaluation_mode` enum
   attribute described in §D-5 below (per-tenant override of the
   library-first default), surfaced in the ADR-0244 tenant DDL.
4. The `secret_references` and Cedar `secret_reference` entity schemas
   accept the `network_side_opt_in: bool` attribute parallel to the
   ADR-0255 amendment's surface — for callers whose evaluation requires
   centralized fragment-set fan-out or cross-cell coverage rollup.
5. The Policy-Engine µservice is re-scoped per §D-4 below to **only**
   the cross-cutting coordination concerns (fragment authoring,
   signing-chain verification, hot-reload distribution, coverage-audit
   scanning, evaluation-audit rollup, untrusted-caller mediation). The
   per-call evaluation endpoint (`Evaluate` gRPC) is retained as the
   opt-in network surface and is **not** the default dispatch path.
6. ADR-0246 §D-1 and §D-3 are annotated with a forward-pointer to this
   amendment so any reader of ADR-0246 lands here before forming a
   runtime-topology mental model.
7. The reference architecture diagram in
   `docs/architecture/policy-engine-substrate-runtime-topology.md` is
   re-drawn to show the library-first path as the default solid edge and
   the network hop (untrusted-caller mediation + cross-cell fan-out
   coordination) as a dashed opt-in edge.
8. The Slice-2 sidecar key-holder primitive (ADR-NNNN-library-first-
   credential-sidecar) lands and its `audit-signing` sidecar surface is
   referenced by the Policy-Engine library's audit-emission path per
   §D-2 below.

Until those eight items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### F-ANTI-2: the F4-architecture finding that triggered this amendment

The 2026-05-20 multispectrum-review v2.4.0 F4-Architecture verdict
(`evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`)
issued finding **F4-LV-6 / F4-AP-1** (also catalogued as F-ANTI-2 in the
keystone-bundle idea-refine deep-dive):

> The library-first / network-opt-in amendment to ADR-0255 closes
> F-ANTI-1 for Intelligence by declaring `oya-shared-intelligence-client-*`
> as the default and the µservice as cross-cutting coordination only.
> The same anti-pattern threat applies to ADR-0246 (Policy-Engine —
> F-ANTI-2 in the idea-refine deep-dive) and to Ontology (F-ANTI-3).
> The idea-refine deep-dive explicitly recommends "ADR-0246 amendment
> clarifying the library-mode default" and "the ontology read-path
> doctrine". As of 2026-05-20 ADR-0246 §D-3 mentions a `oya-shared-
> policy-engine-client-sdk` library crate but does NOT establish
> library-mode as the default with per-call in-process evaluation;
> §D-5 describes a "per-cell evaluator pool" that callers consult via
> gRPC. The same universal-mediator shape that the Intelligence
> amendment closed is therefore still load-bearing for Cedar — every
> state-changing call in the platform takes a per-call gRPC hop to the
> cell-local policy-engine evaluator pool.

The finding is not a defect in ADR-0246's *intent*. ADR-0246's intent
is to consolidate fragment registry, signing-chain verification,
hot-reload distribution, coverage-audit scanning, and evaluator
operation in one substrate µservice so they are not re-implemented per
caller. That consolidation is sound. The defect is in the unspecified
**delivery shape** of the per-call evaluation step: ADR-0246 §D-3's
"Centralized µservice … Chosen" reads as foreclosing the library
alternative, but in fact the alternative analysis there was framed
solely around *fragment registry consistency and hot-reload propagation*
— problems that are correctly solved by a centralized µservice — while
silently inheriting the assumption that *evaluation itself* must also be
mediated. That assumption is unnecessary and produces the universal-
gateway pathology.

This amendment forecloses the µservice-RPC-per-evaluation reading as
the default and establishes the library-link reading as the canonical
delivery shape for **evaluation**, while preserving the µservice's
ownership of fragment authoring + signing-chain + hot-reload +
coverage-audit + cross-cell coordination.

### Why F-ANTI-2 is structurally identical to F-ANTI-1

F-ANTI-1 (Intelligence) and F-ANTI-2 (Policy-Engine) share the same
shape. The shape is:

1. **A substrate concern** that requires *consolidation* of cross-cutting
   state (provider-adapter inventory + credential pool for Intelligence;
   fragment registry + signing-chain + hot-reload for Policy-Engine).
2. **A per-call hot-path step** that is the substrate's primary user-
   visible work (LLM dispatch for Intelligence; Cedar evaluation for
   Policy-Engine).
3. **An implementation ambiguity** between "the per-call work happens
   in-process in the caller via a library" vs "the per-call work
   traverses a gRPC hop to the substrate µservice."

The hyperscaler reference for Intelligence (AWS SDK + Bedrock, Anthropic
SDK, OpenAI SDK) resolves the ambiguity to library-in-process. The
hyperscaler reference for Policy-Engine (AWS Verified Permissions
client SDK, Open Policy Agent embedded mode, Cedar v4.2 Rust crate's
canonical embedded usage, AWS IAM client-side caller-side eval for
session policies) likewise resolves the ambiguity to library-in-process.

The same six failure modes ADR-0255 amendment §Context enumerated apply
to a µservice-mediated evaluation default:

1. **SLO ceiling.** Every state-changing call's availability is bounded
   by Policy-Engine's availability. Because ADR-0243 elevated Cedar to
   a hot-path consultation on **every** state-changing action, this
   ceiling is harsher than for Intelligence (which only bounds AI-
   mediated features). The platform's stated quality bar
   (`feedback_quality_performance_scalability_bar`) becomes unreachable
   for state-changing functionality across the entire portfolio.
2. **Failure perimeter.** A regional outage of `microservices/policy-
   engine/` in cell X cascades to every state-changing µservice in cell
   X. The blast radius is wider than for Intelligence (every product +
   every substrate, not just AI-mediated ones).
3. **Latency tax.** Every state-changing call adds a network round-trip
   (caller → policy-engine → caller). At cell-internal latency budgets
   (~2-10 ms one way over the mesh) plus Cedar evaluation cost (~50-500
   µs in-process; ~5-50 ms with a hop), the tax is ~2-10 ms per call
   compounded across the portfolio's ~1000× QPS expansion that ADR-0246
   §Context predicted. The Workflow Engine alone routinely runs 5-15
   Cedar evaluations per durable workflow step (action permit + tenant
   scope + data-class permit + cell permit + audit-stream permit); at
   ~10 ms per hop the per-step latency tax is 50-150 ms.
4. **Capacity coupling.** Policy-Engine's capacity becomes the platform
   capacity for state-changing operations. Sizing errors at Policy-
   Engine become sizing errors everywhere. The hyperscaler shape ADR-
   0145 established (per-µservice capacity, independent scaling)
   collapses for every state-changing path.
5. **Observability inversion.** The natural span hierarchy for a
   state-changing call is `caller → resource-target`. Inserting Policy-
   Engine makes it `caller → policy-engine → caller → resource-target`,
   which is an artificial extra span that adds no information (the
   Cedar evaluation and audit emission are equally observable in a
   library span).
6. **Distributed monolith.** Policy-Engine's evolution becomes coupled
   to every consumer. Schema changes, evaluator-version changes,
   fragment-context-shape changes require coordinated deploys across N
   callers. ADR-0212's buildability doctrine and
   `feedback_autonomous_implementation_artifacts` become infeasible.

These six failure modes are exactly the modes ADR-0145 §Context cited
as the reason to retire the universal-mediator pattern. Re-introducing
them under the Policy-Engine label is not acceptable.

### What library-first means for Cedar evaluation specifically

The pattern is well-precedented for Cedar. The Cedar v4.2 Rust crate
(`cedar-policy`, AWS open-source) is designed for **in-process
evaluation** with hot-reloadable signed fragment bundles. Its canonical
embedded-mode usage looks like:

```rust
use cedar_policy::{Authorizer, Context, Entities, PolicySet, Request};

let policies = PolicySet::from_str(&fragment_bundle_text)?;
let entities = Entities::from_json_value(entities_json, Some(&schema))?;
let request = Request::new(principal, action, resource, context, Some(&schema))?;
let authorizer = Authorizer::new();
let response = authorizer.is_authorized(&request, &policies, &entities);
// response is `Allow` or `Deny`; no network call has happened.
```

This is the same shape the AWS IAM caller-side evaluation library uses
for session policies, the same shape Open Policy Agent uses in embedded
mode (`opa eval` against a compiled bundle), and the same shape Cedar's
own AWS Verified Permissions client SDK adopts when given a cached
policy set. The hot-reload of compiled bundles can happen at one of
three places:

1. **Inside the µservice evaluator** (status quo per ADR-0246 §D-3):
   evaluator process holds the bundle, callers RPC to it. Every call is
   a network hop.
2. **Inside the caller's process** (library-first): library holds the
   compiled bundle in-process; bundle is refreshed periodically from
   the µservice; per-call evaluation is in-process. **Chosen as the
   default.**
3. **Inside a co-located sidecar**: bundle in sidecar process; caller
   uses loopback. Considered and rejected for the same reasons the
   ADR-0255 amendment rejected sidecars (memory tax + two-process
   complexity + still operationally a mediator).

The library-first default does not weaken hot-reload guarantees. The
fragment-bundle distribution remains a per-cell Kafka pub-sub fan-out
to all library instances; the <5s p99 propagation guarantee per
ADR-0243 §D-10 is preserved because the receivers are *library
instances in caller processes*, not evaluator pods. The fan-out is
larger (every caller pod instead of every evaluator pod) but Kafka
partition-count + per-message-size budgets accommodate this: a typical
cell has ~200 caller pods × ~3 replicas vs ~30 evaluator pods × ~3
replicas, a 7× expansion that is well within Kafka topic partition
limits.

### What ADR-0145 actually said (re-stated)

ADR-0145 (Inter-microservice communication reform, 2026-05-18) made
three structural decisions:

1. **Audit invariant (decentralized).** Every state-changing call
   emits an audit-chain seal at the *calling* service. No central
   mediator owns the seal.
2. **Tracing invariant.** Every call propagates OTel context. No
   central mediator owns the trace.
3. **Ontology projection invariant.** Ontology is a SUBSTRATE for
   cross-µservice read queries, not a GATEWAY. µservices may also
   call each other directly via mTLS gRPC.

ADR-0145 also explicitly states:

> AWS, Google, Microsoft, Stripe, Anthropic do NOT use a universal
> mediator pattern. They use direct service-to-service gRPC/HTTP
> with mTLS + per-service contracts.
>
> Putting orchestration in the data path (ESB 2.0) makes Workflow the
> platform SLO ceiling and failure perimeter.

The same reasoning applies to authorization evaluation. Putting Policy-
Engine in the data path of every state-changing call makes Policy-
Engine the platform SLO ceiling and failure perimeter for the entire
state-changing surface. If the Policy-Engine µservice is down or
degraded, every state-changing feature is down or degraded — even
though the caller has a valid Cedar fragment bundle in memory that
would have permitted the action. Policy-Engine has no business being a
single point of failure on a path where it has nothing to contribute
beyond what a library could contribute in-process, provided the library
holds the same compiled fragment bundle the evaluator would have held.

This amendment makes that doctrine explicit for Policy-Engine. ADR-0145's
"no universal mediator" doctrine remains intact — and is now actively
defended on the second of three structurally identical risk axes.

### What AWS IAM and Verified Permissions actually show

ADR-0246 §Context cites AWS Verified Permissions as a hyperscaler
reference for the centralized evaluator pattern. The citation is
correct for the *control plane* (fragment authoring, schema
validation, fragment distribution) but not for the *data plane* of
per-call evaluation.

In AWS production reality, **IAM evaluation runs caller-side**. Every
AWS service's request-handling code calls the in-process IAM
authorization library against a cached policy set; the in-process
library composes session policy + permissions boundary + identity
policy + resource policy + SCP and produces an Allow/Deny in-process.
The IAM control-plane service (`iam.amazonaws.com`) is consulted for
policy authoring, not for per-call evaluation. Were it otherwise, every
AWS service call would carry the latency tax of an IAM round-trip; the
single-digit-millisecond p99s AWS publishes for S3 / DynamoDB / Lambda
control plane operations would be unreachable.

AWS Verified Permissions (the 2024-Q1-GA managed Cedar service)
similarly publishes a *batch evaluation* and a *cached client* mode
explicitly so that callers can avoid the per-call network hop. The
managed service's role is fragment authoring, schema validation,
fragment distribution, and audit observability — the same role ADR-
0246's Policy-Engine µservice plays. The per-call evaluation is
caller-side library.

The hyperscaler precedent therefore *agrees with this amendment*, not
with the unamended ADR-0246 §D-3 + §D-5 reading.

### Hamilton's static stability principle (re-stated)

James Hamilton's 2007 LISA talk and the AWS Builder's Library "Static
stability using Availability Zones" (2020) formalized the principle
that **a system should continue to function statically when its
coordinator is unavailable**. Applied here: when the Policy-Engine
µservice is down, state-changing calls should still flow — because the
library in the caller's process has everything it needs (compiled
fragment bundle, signing-chain verified at refresh time, Cedar
evaluator) to make the decision. The Policy-Engine µservice's absence
should degrade *cross-cutting* concerns (new fragment publications are
delayed; coverage-audit refresh is stale; cross-cell fan-out
coordination falls back to local-only) — not break the actual
state-changing path.

This is the AZ-static-stability pattern transplanted to the intra-cell
control plane: the data path does not depend on the control path being
up.

## Decision

### D-1. Policy-Engine Substrate evaluation is library-first

The Policy-Engine Substrate's **per-call evaluation surface** is
delivered as a library by default. The canonical entry point is the
`oya-shared-policy-engine-client-*` crate family. Every caller (every
µservice's request handler that performs a Cedar gate, every Workflow
Engine step that consults a policy permit, every Foundry workflow that
applies an autonomy-tier check, every Ontology Function that screens
its action's permit fragment) links the library and calls the library's
in-process `evaluate(...)` / `evaluate_batch(...)` API.

The library, not a µservice, is the user-visible surface of Policy-
Engine for the per-call evaluation path.

The eight bounded contexts established in ADR-0246 §D-2
(fragment-registry, evaluator, signing-chain, hot-reload, coverage-
audit, pack-overlay, tenant-overlay, bootstrap-genesis) are unchanged in
their *responsibility split*. What changes is their *runtime delivery
shape*:

| BC | Where it runs by default |
|---|---|
| `evaluator` | **In the caller's process** (library composition of `cedar-policy` v4.2 + compiled bundle). The µservice retains an `evaluate` RPC endpoint for opt-in callers only. |
| `pack-overlay` | **In the caller's process** (library holds compiled pack bundles in memory; pack-set is part of the fragment bundle refresh). |
| `tenant-overlay` | **In the caller's process** (library holds compiled tenant overlays in memory; tenant overlay set is part of the fragment bundle refresh, scoped by tenant). |
| `fragment-registry` | **In the µservice** (Postgres + Citus + Valkey hot cache). Library pulls compiled bundles via `GET /v1/fragments/bundle/{cell_id}`. |
| `signing-chain` | **In the µservice** for authoring; **in the library** for refresh-time verification (the library verifies the bundle's Ed25519 signature against the org-root → intermediate → publisher chain before accepting the bundle for hot-swap). |
| `hot-reload` | **In the µservice** (publishes `FragmentPublished` events to per-cell Kafka topic); **in the library** (subscribes, fetches, verifies, swaps). |
| `coverage-audit` | **In the µservice** (CI lane + nightly drift detection). Not on the call path. |
| `bootstrap-genesis` | **In the µservice** (one-shot per cell at bootstrap step 5 of ADR-0242 §D-5). Not on the call path. |

The library is the unit of consumption for evaluation. Each BC's crate
remains the unit of authorship.

### D-2. What the library performs in-process

The `oya-shared-policy-engine-client-*` library performs **all** of the
following work in the caller's own process:

| Concern | In-process responsibility |
|---|---|
| Cedar evaluation | Construct `cedar-policy::Request` from caller's `(principal, action, resource, context)` tuple. Evaluate against the compiled `cedar-policy::PolicySet` held in the library's bundle cache. Return `Decision { Permit \| Forbid \| NotApplicable }`. |
| Bundle composition | At refresh time (not per-call), compose baseline ∪ active pack overlays ∪ active tenant overlays into a single compiled `PolicySet` per (cell, tenant) tuple. Hold the per-tenant compiled bundles in an LRU cache (default 1024 tenants per pod). |
| Bundle refresh | Subscribe to per-cell `policy-engine.fragment-reload` Kafka topic. On `FragmentPublished`, fetch new bundle via gRPC from µservice; verify signing-chain via embedded `oya-policy-engine-signing-chain-domain` crate; recompile; atomic-swap into bundle cache. Hot-reload completes within the ADR-0243 §D-10 <5s p99 SLO at the caller's process boundary. |
| Signing-chain verification | Embedded `oya-policy-engine-signing-chain-domain` crate verifies the bundle's Ed25519 signature against the cached org-root certificate (pinned at bootstrap; refreshed via separate `signing-chain.org-root-cert.public.pem` watch). Verification is in-process; no µservice round-trip per evaluation. |
| Pack-overlay resolution | Embedded `oya-policy-engine-pack-overlay-domain` crate composes pack fragments at bundle-refresh time, not per-call. Per-call overhead is zero beyond the in-memory `PolicySet` already containing the composed overlays. |
| Tenant-overlay resolution | Embedded `oya-policy-engine-tenant-overlay-domain` crate composes tenant fragments at bundle-refresh time per tenant. The tenant-fragment-restriction enforcement (tenants can forbid but cannot raise permits beyond baseline) is verified at compile time, not at evaluation time. |
| Audit emission | In-process call to `oya-shared-audit-chain-client` (per ADR-0145 Invariant 1) to seal a `PolicyEvaluated` audit row when the action is gated and the decision is consequential (permit + state-changing, or forbid that surfaces to caller). Per-evaluation audit emission is sampled by Cedar fragment (e.g., `audit.policy_evaluated.sampling_rate = 1.0` for tier-1 actions; 0.01 for high-volume reads). The seal is emitted by the caller's process, not by the Policy-Engine µservice. The audit-signing key is held by the **Slice-2 sidecar key-holder** (ADR-NNNN-library-first-credential-sidecar) co-located with the caller pod; the library calls into the sidecar via UDS to sign. |
| OTel propagation | In-process call to `oya-shared-tracing-client` (per ADR-0145 Invariant 2). The span hierarchy is `caller → resource-target`, with library work as child spans of `caller` (typically `policy-engine.evaluate` span). No artificial `caller → policy-engine-mediator → resource-target` insertion. |
| SecretReference resolution | Policy-Engine evaluation does **not** itself resolve secrets, but the Cedar fragment may reference attributes from a SecretReference (e.g., `secret_reference.tenant_id`). Attribute fetch happens in-process via `oya-shared-secret-reference` against `microservices/cloud-secrets/` (OpenBao) per ADR-0255 amendment §D-2. Policy-Engine µservice is not in the credential-resolution path. |
| Brown-out signal | The library emits ADR-0176 brown-out signal locally when in-process evaluation latency p99 exceeds the per-cell target (default 1 ms). The µservice emits a cell-wide aggregate brown-out signal (opt-in callers only). Two signals are non-conflicting; local trips first; global trips on opted-in callers. |
| Decision cache | Optional LRU cache of `(principal, action, resource, context_hash) → Decision` with short TTL (default 5 s) for repeated evaluations within a request scope (e.g., a batch of 1000 reads each evaluated against the same Cedar fragment). Cache is in-process; no µservice involvement. |
| Coverage telemetry | Library reports per-call coverage telemetry (`fragment_hits[fragment_id] += 1`) to the µservice via batched async POST (default 60 s flush). The µservice aggregates fleet-wide coverage. Telemetry path is non-blocking for the caller; failure to flush does not block evaluation. |

All twelve concerns happen in the caller's process or via non-blocking
async flush. No synchronous network hop to `microservices/policy-
engine/` is required for any of them on the default path.

### D-3. The library composes evaluation in-process

After the library has performed its in-process work (bundle composition
at refresh time, in-process evaluation, audit row emission via sidecar
co-located key-holder, OTel span open), the caller's request continues
on its normal path — typically issuing the state-changing call to the
target resource µservice over mTLS gRPC.

The call topology is:

```
caller process
  │
  │ (in-process)
  │  oya-shared-policy-engine-client-* library
  │   ├─ bundle cache lookup       (in-process; sub-µs)
  │   ├─ tenant overlay merge      (in-process; pre-compiled)
  │   ├─ pack overlay merge        (in-process; pre-compiled)
  │   ├─ cedar-policy evaluate     (in-process; 50-500 µs typical)
  │   ├─ audit-emit                (in-process via UDS to sidecar key-holder
  │   │                              per Slice-2 ADR-NNNN-library-first-
  │   │                              credential-sidecar)
  │   ├─ OTel span open            (in-process)
  │   └─ decision returned         (Permit / Forbid / NotApplicable)
  │
  │  (Permit) caller proceeds to target resource µservice
  │  (Forbid) caller returns 403 to its own caller
  │
  ▼
target resource µservice
  (microservices/ontology/, microservices/workflow-engine/, ...)
```

There is **no network hop to `microservices/policy-engine/`** on this
path. The Policy-Engine µservice is not in the request flow.

This is the **default** path. It is what ≥99% of evaluations take
unless the caller has explicitly opted in to the network-side
coordination features described in §D-4 + §D-5.

### D-4. The Policy-Engine µservice exists for cross-cutting state only

`microservices/policy-engine/` continues to exist, and its bounded-
context split per ADR-0246 §D-2 is preserved. What this amendment
changes is the **runtime responsibility surface** — which BCs serve
their work over the network and which BCs ship their work as library
code. Specifically:

| Cross-cutting concern | Why it cannot live entirely in the library | Network-side responsibility |
|---|---|---|
| Fragment registry storage | Fragments are authored centrally, signed by the publisher chain, and persisted in Citus-sharded Postgres for queryability + diffability + audit. Per-process storage is wasteful and inconsistent. | `fragment-registry` BC: Postgres + Citus shard on `(scope, fragment_id)`; Valkey hot cache for compiled bundles; REST/gRPC for authoring + listing + getting fragments. |
| Fragment authoring + signing | Fragment publication requires offline-key signature ceremonies (Shamir M-of-N at org root for baseline; intermediate key for day-to-day; publisher key for tenant overlays). The signing ceremony cannot happen in a caller's process. | `signing-chain` BC owns the signing path; OpenBao for intermediate keys; PKCS#11 HSM client for the org root key; Sigstore cosign for attestations. Library handles only refresh-time verification. |
| Hot-reload distribution | When a fragment is published, every library instance in every caller process across the cell must receive the new bundle within <5s p99. The fan-out coordination requires a central publisher. | `hot-reload` BC: Kafka `policy-engine.fragment-reload` topic per cell; `FragmentPublished` event with bundle URL + signature + version. Library subscribers consume; library is the *consumer*, not the publisher. |
| Coverage-audit scanning | Determining whether every µservice's declared actions have a permit fragment + default-deny requires fleet-wide manifest enumeration + cross-µservice intersection. A caller's process has no visibility into other callers' manifests. | `coverage-audit` BC: scheduled scan of every µservice's `capabilities/*.yaml` + OpenAPI + AsyncAPI + Cedar fragment store; emits `CoverageReport` rows. CI lane + nightly drift detection. Read-only consumer of µservice manifests + fragment store; not a per-call participant. |
| Cross-cell fragment fan-out | A new fragment must propagate not only within a cell but across cells (subject to per-pack jurisdictional restrictions). The cross-cell publisher coordinates this. | `hot-reload` BC's cross-cell publisher emits fragment-reload events to peer cells' `policy-engine.fragment-reload` topics per ADR-0248 §D-9 constant-work pattern. Library does not participate in cross-cell coordination. |
| Cross-cell coverage rollup | Per-call coverage telemetry emits at the caller's library; cross-cell aggregate views (fleet-wide fragment-hit rate by tenant by action) require a rollup process. | Subscribe to the coverage telemetry stream; emit aggregate rows + dashboards. Read-only consumer of coverage telemetry; not a per-call participant. |
| Untrusted-caller mediation | A caller deployed in an *untrusted* cell tier (per ADR-0248 §D-7 cell-tier taxonomy: certain edge-tier or sovereign-pack cells with reduced caller-trust posture) cannot be trusted to hold the compiled policy bundle in-process without leaking it or evaluating against a stale bundle. For these callers, evaluation must go to a centrally-attested evaluator. | `evaluator` BC's opt-in `Evaluate` gRPC endpoint serves untrusted-tier callers. Per-tenant `policy_evaluation_mode = network_only` callers also use this endpoint per §D-5. |
| Bootstrap genesis | At cell creation (bootstrap step 5 of ADR-0242 §D-5), the genesis fragment must be loaded from the bootstrap log + verified against the org-root signature + seeded into the fragment registry + emitted as a bootstrap-completion audit event. This is a one-shot µservice operation. | `bootstrap-genesis` BC owns the bootstrap path; Kubernetes Job, not Deployment. |
| Evaluation-audit rollup | Per-call audit rows emit at the caller; cross-cell aggregate views (fleet-wide permit/forbid ratio by fragment by tenant by audience) require a rollup process. | Subscribe to the audit-chain stream's `PolicyEvaluated` rows; aggregate to compliance dashboards. Read-only consumer of audit-chain; not a per-call participant. |

None of the ten concerns above is on the **synchronous per-call Cedar
evaluation path** for the default caller. They are control-plane and
batch concerns. The runtime topology preserves the static-stability
property: when the Policy-Engine µservice is unavailable, the per-call
evaluation path continues to function (degraded only in the cross-cutting
sense — new fragments are delayed; coverage-audit refresh is stale;
cross-cell fan-out coordination falls back to local-only).

### D-5. Callers opt in to network-side Policy-Engine per Cedar policy and per tenant attribute

Most callers default to the library-only path. A caller that **needs**
network-side coordination opts in per call (or per credential, or per
audience tag, or per tenant) via two surfaces: a per-SecretReference
attribute and a new per-tenant attribute.

**Per-SecretReference opt-in (parallel to the ADR-0255 amendment).**
The `secret_references` table (per ADR-0255 §D-4 + amendment §D-5)
gains an attribute parallel to the Intelligence-amendment's:

```sql
ALTER TABLE secret_references
    ADD COLUMN policy_evaluation_network_opt_in BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE secret_references
    ADD COLUMN policy_evaluation_network_opt_in_reasons TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
    -- enum-validated subset of:
    --   'untrusted-cell-tier'
    --   'fragment-publisher-attestation-unavailable-locally'
    --   'cross-cell-fragment-fan-out-required'
    --   'centralized-policy-distribution-required'
    --   'audit-signing-key-holder-co-residency-unavailable'
```

When `policy_evaluation_network_opt_in = TRUE`, the library performs a
gRPC `Evaluate` call to the Policy-Engine µservice's evaluator endpoint
instead of evaluating in-process. When `FALSE`, the library evaluates
in-process per §D-2.

**Per-tenant attribute opt-in (new in this amendment).** The `tenants`
table (per ADR-0244) gains a column governing the tenant-wide
evaluation mode:

```sql
ALTER TABLE tenants
    ADD COLUMN policy_evaluation_mode policy_evaluation_mode_t NOT NULL DEFAULT 'library_first';

CREATE TYPE policy_evaluation_mode_t AS ENUM (
    'library_first',
    'network_only',
    'library_first_with_attested_fallback'
);
```

Semantics:

- `library_first` (default): the library evaluates in-process per §D-2.
  This is the canonical mode for the overwhelming majority of tenants.
- `network_only`: every evaluation for this tenant goes via gRPC to the
  Policy-Engine µservice's `Evaluate` endpoint. This is the mode for
  tenants in untrusted-cell-tier deployments, or for tenants whose
  compliance pack mandates centralised evaluation (e.g.,
  `pack/kr-fss-iss-il5/` may declare `requires_network_only_policy_evaluation: true`
  for centralised audit posture).
- `library_first_with_attested_fallback`: library evaluates in-process,
  but if the local bundle's freshness is older than a pack-declared
  threshold (e.g., 24h), the library falls back to the network endpoint.
  This is the mode for compliance packs that require maximum freshness
  guarantees without giving up the static-stability property entirely.

The follow-up ADR-0244 tenant DDL extension that introduces
`policy_evaluation_mode` is tracked separately (see §F-1 below); this
amendment lists the attribute as required but does not edit ADR-0244.

**Per-call Cedar opt-in.** A Cedar fragment in `policy-engine/fragments/
baseline/` governs the opt-in decision:

```cedar
permit (
    principal,
    action == PolicyEngine::Action::"NetworkSideEvaluation",
    resource is Tenant
) when {
    resource.policy_evaluation_mode == "network_only" ||
    (resource.policy_evaluation_mode == "library_first_with_attested_fallback"
        && context.local_bundle_age_seconds > resource.attested_fallback_threshold_seconds)
};
```

The caller's library checks the tenant's mode + the local bundle's
freshness; Cedar evaluates; permit routes to the network endpoint;
forbid (NotApplicable) routes to the in-process evaluator.

This per-policy opt-in keeps the network side as an *explicit*
escalation, not an *implicit default*. The hyperscaler shape that ADR-
0145 established is preserved: direct calls are the default; coordination
is opt-in.

### D-6. Default is library-only; network-side is opt-in

The library defaults to **local-only in-process evaluation** for every
caller whose tenant `policy_evaluation_mode = library_first` and whose
SecretReference (where applicable) has `policy_evaluation_network_opt_in
= FALSE`.

The default's properties:

1. The Policy-Engine µservice has zero per-call network traffic from
   the caller for evaluation.
2. The Policy-Engine µservice's availability does not bound the
   caller's evaluation availability.
3. The caller's latency budget does not include a Policy-Engine
   round-trip for evaluation.
4. The caller's failure perimeter for evaluation does not include the
   Policy-Engine µservice.

Opt-in's properties (for callers under `network_only` or
`library_first_with_attested_fallback` modes, or for SecretReferences
with `policy_evaluation_network_opt_in = TRUE`):

1. The library performs a gRPC `Evaluate` (and `EvaluateBatch` where
   applicable) RPC before proceeding.
2. The RPC is on the synchronous path; the caller's latency budget
   includes one extra round-trip per evaluation (or per batch).
3. The caller's failure perimeter includes the Policy-Engine µservice
   *for evaluation*. If the RPC fails open (per Cedar fallback policy),
   the caller proceeds with a per-fragment default-deny or last-good-
   in-process bundle; if it fails closed, the caller fails the state-
   changing call.

The opt-in decision is per-tenant + per-SecretReference, so a B2C
consumer tenant in a standard cell stays library-first while a regulated
tenant in a sovereign-cloud cell can declare `network_only` without
affecting any other tenant.

### D-7. ADR-0145 alignment statement

ADR-0145 established the doctrine: **Authorization is a SPECIALIZED
concern with a SPECIALIZED library, NOT a universal mediator.**

This amendment makes that doctrine explicit for Policy-Engine. The
Policy-Engine library is parallel in shape to:

- The `oya-shared-intelligence-client-*` library family (per ADR-0255
  amendment). The Intelligence µservice does not mediate every LLM
  call; the library does the dispatch in-process.
- The `oya-shared-audit-chain-client` library. The audit-chain µservice
  does not mediate every seal; the library emits seals directly.
- The `oya-shared-tracing-client` library. Tempo does not mediate every
  span; the library propagates and emits OTel directly.
- The `oya-shared-secret-reference` library. The cloud-secrets µservice
  does not mediate every fetch beyond the OpenBao request itself; the
  library handles caching and resolution.

Policy-Engine joins this family. The library is the per-call surface;
the µservice is the coordination + storage + signing + fan-out +
audit-rollup surface.

The ADR-0145 three invariants (audit emission, OTel propagation,
Ontology projection) apply unchanged. The library satisfies all three
in-process at the caller. The Policy-Engine µservice does **not** emit
`PolicyEvaluated` audit seals on behalf of the caller (the caller emits
its own seals, per Invariant 1, signed by the Slice-2 sidecar key-
holder).

**The "no universal mediator" doctrine remains intact.** This amendment
does not introduce a new mediator; it removes the implicit mediator
that ADR-0246 §D-3 + §D-5 could be read as introducing. The pre-
ADR-0246 state of the world (Cedar evaluation via library in caller
process per ADR-0150's original framing) is restored as the default.
ADR-0246's substrate-promotion benefits (fragment authoring, signing-
chain, hot-reload, coverage-audit) are preserved by keeping those
concerns in the µservice.

### D-8. SLO + failure perimeter consequences

The library-first default produces three operational properties:

1. **SLO ceiling is removed.** The platform-wide SLO for state-changing
   functionality is bounded by the caller's own SLO + the target
   resource µservice's SLO, not by Policy-Engine's SLO. If the caller is
   99.95% available and the target is 99.95% available, the composed SLO
   for the state-changing feature is ~99.90% — independent of Policy-
   Engine's own SLO.
2. **Failure perimeter contracts.** When the Policy-Engine µservice is
   down, the per-call evaluation path continues to function. The
   degradation is limited to: new fragment publications are delayed
   (callers continue evaluating against last-published bundle); coverage-
   audit refresh is stale (fleet-wide coverage telemetry catches up
   when Policy-Engine recovers); cross-cell fan-out coordination falls
   back to local-only (in-cell fragment-set continues to apply). None
   of these block evaluation.
3. **Latency tax is removed.** The default path adds ~50-500 µs of
   in-process Cedar evaluation per call, compared to the ~2-10 ms
   network round-trip + ~50-500 µs Cedar evaluation that the
   µservice-mediated reading would have imposed. For a portfolio
   averaging ~5 Cedar evaluations per state-changing call (per ADR-
   0246 §Context QPS analysis), the per-call tax saved is ~10-50 ms
   when measured against the µservice-mediated alternative.

These three properties are required to honor the hyperscaler-bar
quality target (`feedback_quality_performance_scalability_bar`) for
state-changing functionality, which is to say for every product surface
in the portfolio.

## Alternatives

### Alternative 1 — Keep ADR-0246 as-is (status quo without this amendment)

**Description.** Accept ADR-0246 §D-3 + §D-5's centralized-evaluator
language at face value. Implement the `evaluator` BC as a gRPC endpoint
on `microservices/policy-engine/`. Every state-changing call from every
µservice issues a gRPC `Evaluate` to Policy-Engine, which then performs
the in-process Cedar evaluation against its compiled bundle and returns
Permit/Forbid.

**Pros.**

1. Single binary to test and operate for the evaluation surface.
2. Fragment version pinning is centralized (no library version skew
   across callers).
3. Emergency-permit propagation is naturally fleet-wide because all
   evaluations pass through Policy-Engine.
4. Coverage-audit telemetry is naturally complete because every
   evaluation is observable at the central µservice.

**Cons.**

1. **Re-introduces the universal-mediator anti-pattern.** ADR-0145
   §Context cites this as the #1 12-month regret with ~70% probability.
   Re-introducing it under the Policy-Engine label does not change the
   pathology; F4-LV-6 / F4-AP-1 / F-ANTI-2 expressly catalogue this
   risk as identical to F-ANTI-1.
2. **SLO ceiling.** Every state-changing feature's availability is
   bounded by Policy-Engine's availability — harsher than for
   Intelligence because Cedar is on every state-changing call (per
   ADR-0243), not just AI-mediated ones.
3. **Failure perimeter.** A regional Policy-Engine outage cascades to
   every state-changing feature in the cell.
4. **Distributed monolith.** Policy-Engine evolution becomes coordinated
   with every caller; ADR-0212's buildability and
   `feedback_autonomous_implementation_artifacts` become infeasible.
5. **Latency tax** on every call, compounding across the ~1000× QPS
   expansion ADR-0246 §Context predicted.
6. **Capacity coupling.** Policy-Engine becomes the platform capacity
   for authorization, defeating the per-µservice independent-scaling
   property ADR-0145 established.

**Rejected.** This is exactly the anti-pattern ADR-0145 retired and
that the ADR-0255 amendment closed for Intelligence. The defect ADR-
0145 closed in PR #143 is the same defect this alternative would re-
open for Policy-Engine.

### Alternative 2 — Full µservice-only with sidecar mitigation

**Description.** Implement the `evaluator` BC as a gRPC endpoint on
Policy-Engine (as in Alternative 1), but co-locate a Policy-Engine
sidecar in every caller's pod to absorb the network hop locally. The
sidecar runs the Cedar evaluator in a separate process; the caller
talks to the sidecar over localhost.

**Pros.**

1. Eliminates the cross-pod network hop's latency.
2. The sidecar's failure perimeter is per-pod, not cell-wide.
3. Fragment version pinning remains centralized (sidecar image is the
   central artifact).

**Cons.**

1. **Sidecar memory + CPU tax on every pod.** A sidecar replicating the
   compiled bundle across N caller pods is wasteful relative to a
   library that shares the address space.
2. **Two-process complexity.** The caller process and the sidecar
   process must remain in lockstep on schema version, bundle refresh,
   signing-chain anchor refresh, etc. Library-in-process avoids the IPC
   entirely.
3. **Still operationally a mediator.** The sidecar pattern is
   structurally a service-mesh-style mediator; debugging an evaluation
   failure means looking at the sidecar's logs, not the caller's
   process. The `feedback_no_silent_regression` doctrine penalizes the
   diagnostic indirection.
4. **Re-introduces universal-mediator pattern, just per-pod.** The
   sidecar version becomes the platform-wide evaluator bottleneck — an
   upgrade requires rolling the sidecar across all state-changing pods.

**Rejected.** A sidecar is a worse library for evaluation. The Slice-2
sidecar-credential-isolation pattern is appropriate for *credential
handling* (audit-signing key isolation) but not for *Cedar evaluation*,
which has no key material to isolate.

### Alternative 3 — Library-first, network-opt-in (CHOSEN)

**Description.** Per §D-1 through §D-8 above. The library is the
default; the µservice exists for fragment authoring + signing-chain +
hot-reload + coverage-audit + cross-cell fan-out + untrusted-caller
mediation; opt-in is per-tenant + per-SecretReference + per-Cedar.

**Pros.**

1. **Aligns with ADR-0145.** Direct calls are the default; coordination
   is opt-in. Same shape as Intelligence client (per ADR-0255
   amendment), audit-chain client, tracing client, secret-reference
   client.
2. **SLO ceiling removed.** State-changing feature SLO is bounded by
   caller + target, not by Policy-Engine.
3. **Failure perimeter contracts.** Policy-Engine µservice outage does
   not block evaluation.
4. **No latency tax** on the default path.
5. **Cross-cutting concerns retained.** Fragment authoring, signing-
   chain, hot-reload, coverage-audit, cross-cell fan-out, untrusted-
   caller mediation remain centralized where they genuinely need to be.
6. **Hyperscaler-shape parity.** AWS IAM caller-side eval + AWS
   Verified Permissions cached client + Cedar v4.2 embedded mode + Open
   Policy Agent embedded mode + Google Zanzibar's distributed
   authorization. Library in caller's process; control plane in
   µservice.
7. **`feedback_autonomous_implementation_artifacts` preserved.**
   Library version pinning per-caller means caller upgrades are
   independent; the build-everything-to-green property is maintained.

**Cons.**

1. **Library version skew across callers.** Mitigated by the workspace-
   wide single-version policy (`feedback_no_silent_regression` + Cargo
   workspace `[workspace.dependencies]` pinning) and by the library
   being designed so version-N+1 reads version-N's compiled bundles
   without break.
2. **Bundle distribution discipline required.** New fragments land as
   `FragmentPublished` events; libraries must consume promptly.
   Mitigated by ADR-0212 buildability + the Kafka topic's <5s p99
   propagation guarantee.
3. **Untrusted-tier callers require opt-in.** Callers in untrusted cell
   tiers (per ADR-0248 §D-7) must opt into `network_only` mode via
   tenant attribute. Acceptable because the opt-in is a one-time per-
   tenant declaration aligned with the tenant's compliance pack.

**Accepted.** This is the shape that honors ADR-0145, preserves
hyperscaler-bar SLO, and retains the cross-cutting consolidation ADR-
0246 intended.

## Consequences

### Positive

1. **ADR-0145 universal-mediator retirement is preserved on the
   authorization axis.** F-ANTI-2 (F4-LV-6 / F4-AP-1) is closed in
   writing before any caller code is authored against ADR-0246. The
   "no universal mediator" doctrine remains intact for the second of
   three structurally identical risk axes (Intelligence closed by
   ADR-0255 amendment; Policy-Engine closed by this amendment; Ontology
   closed by the parallel ADR-0257 amendment per keystone-bundle
   §5.13 promotion-gate fix 2-of-2).
2. **Hyperscaler-bar SLO for state-changing functionality is
   reachable.** Caller SLO + target SLO compose without Policy-Engine
   as a third multiplicand.
3. **Static stability per Hamilton 2007.** The data path does not
   depend on the control path being up. Policy-Engine µservice can be
   under maintenance, rolling-restarted, or in a regional outage
   without blocking state-changing operations.
4. **Latency budget unchanged from baseline.** The default path is
   in-process Cedar evaluation; no additional network round-trip is
   introduced relative to a hypothetical no-Policy-Engine baseline
   (per the AWS IAM caller-side eval reference shape).
5. **Diagnostic locality.** When a Cedar evaluation forbids an action,
   the forbid surfaces in the caller's OTel span hierarchy as
   `policy-engine.evaluate` child span of `caller`, with library-
   internal annotations showing the determining policy fragment. No
   artificial `caller → policy-engine → caller` indirection.
6. **Per-µservice scaling preserved.** ADR-0145's "no platform SLO
   ceiling" + "µservices scale independently" applies to Cedar
   evaluation.
7. **Fragment authoring + signing remain centralized** in the µservice
   where signing ceremonies + offline key handling + Shamir M-of-N
   recovery genuinely need central state.
8. **Coverage telemetry remains centralized** via the async batched
   flush pattern. Fleet-wide coverage reports are unchanged.

### Negative

1. **Library version pinning discipline.** Workspace-wide pinning
   already exists; this amendment makes the discipline binding for
   `oya-shared-policy-engine-client-*`. Operators must roll library
   updates across the workspace per ADR-0212 buildability cadence.
2. **Bundle refresh cadence.** New fragment publications require library
   instances to fetch + verify + recompile. Mitigated because Cedar
   bundle compilation is fast (~10-50 ms typical for a 1000-fragment
   bundle on a modern x86 core) and is amortized over the bundle's
   lifetime (typical fragment lifetime is hours-to-days, not seconds).
3. **Per-tenant opt-in authoring.** Tenants in untrusted-cell-tier
   deployments must author the `policy_evaluation_mode = network_only`
   declaration. The default for new tenants is `library_first`, which
   preserves the "no surprise coupling" principle
   (`feedback_no_silent_regression`).

### Consequences for ADR-0145's "no universal mediator" doctrine

ADR-0145's load-bearing doctrine is **explicitly preserved** by this
amendment. The amendment introduces no new mediator. It removes the
implicit mediator that ADR-0246 §D-3 + §D-5 could be read as
introducing. Specifically:

1. **Direct service-to-service calls remain the default.** Caller →
   target resource µservice remains the canonical inter-µservice shape.
   Policy-Engine evaluation happens *inside the caller process* before
   the call, not as a separate mediating hop.
2. **The three weaker invariants apply unchanged.** Audit emission
   (Invariant 1): caller emits its own `PolicyEvaluated` seal via
   library + sidecar key-holder. Tracing (Invariant 2): caller emits
   its own span hierarchy. Ontology projection (Invariant 3): Policy-
   Engine entities (fragments, coverage reports) are projected into
   Ontology for read queryability, but the read path itself is governed
   by the parallel ADR-0257 amendment per F4-LV-6 / F4-AP-1.
3. **Workflow remains opt-in.** Policy-Engine evaluation does not flow
   through Workflow Engine. The Workflow vs direct-gRPC rubric (per
   ADR-0145 §"Rubric: when to use Workflow vs direct gRPC") is unchanged
   by this amendment.
4. **Service-mesh substrate is unaffected.** mTLS handshakes per ADR-
   0148 Cilium happen for the µservice-to-µservice surfaces (audit-
   chain client, secret-reference client, fragment-bundle fetch on
   refresh) — not for per-call evaluation, which is in-process.

If a future amendment to this amendment proposes to re-introduce a
per-call gRPC mediator under any label, that amendment must explicitly
overturn ADR-0145's "no universal mediator" doctrine. This amendment's
existence documents the position that such an overturning would have to
clear.

### Operational

1. **Authoring sequence.** `oya-shared-policy-engine-client-*` crates
   are scaffolded with the library-mode-default evaluator embedded
   (Cedar v4.2 Rust crate `cedar-policy` as the kernel) **before** the
   Policy-Engine µservice's `Evaluate` gRPC endpoint is exercised
   against any caller. The µservice's runtime surface for evaluation is
   scoped to opt-in callers from day one. If a future requirement
   genuinely needs centralized per-call evaluation as the default, that
   requirement amends this amendment with a fresh ADR.
2. **CI lane authoring.** The `oya-check-no-unnecessary-policy-engine-
   service-hop` lane scans for unconditional
   `PolicyEngineClient::evaluate_via_service(...)` calls (or equivalent
   gRPC client invocations) that are not gated by a per-tenant
   `policy_evaluation_mode != library_first` check or a per-
   SecretReference `policy_evaluation_network_opt_in = TRUE` check.
   Failures block merge.
3. **Documentation updates.** ADR-0246 §D-1 + §D-3 + §D-5 gain forward-
   pointers to this amendment in their frontmatter and prose. The
   reference architecture diagram at `docs/architecture/policy-engine-
   substrate-runtime-topology.md` is re-drawn to show library-first
   default + opt-in network edge.
4. **Brown-out signal authoring.** The brown-out signal per ADR-0176
   is emitted by the *library* (local-only on the default path) and by
   the *µservice* (cell-wide aggregate, opt-in callers only). The two
   signals do not conflict; the local signal trips first; the global
   signal trips on opted-in callers when the cell-wide aggregate is
   hot.
5. **Cell-µservice load is reduced.** Per ADR-0148 Cilium Service Mesh,
   mTLS handshake count for Policy-Engine drops by ~99% on the library-
   first default. SPIFFE-ID issuance budget at cell-µservice eases
   correspondingly.
6. **Service-mesh egress.** Policy-Engine µservice egress (fragment
   bundle fetch on refresh, async coverage telemetry flush, opt-in
   evaluation gRPC) flows through Cilium service mesh per ADR-0148.
   NetworkPolicy permits library → µservice for these specific
   endpoints only.
7. **Multi-cell deployment.** Each cell ships the library to its
   callers and a Policy-Engine µservice for its own cross-cutting
   state. Cross-cell coordination (fragment fan-out, coverage rollup,
   audit rollup) happens at the µservice layer via Kafka pub-sub +
   audit-chain stream subscription.
8. **Failure-mode runbook.** A new runbook at
   `docs/operators/policy-engine-substrate-failure-modes.md`
   enumerates: bundle staleness (library detects; caller surfaces stale-
   bundle warning if older than per-tenant threshold); signing-chain
   verification failure (library refuses bundle swap; caller continues
   on last-good bundle; SEV-2 alert fires); cell-local Policy-Engine
   µservice down (library proceeds local-only; coverage telemetry
   buffer fills; alarms on the µservice itself); cell-wide network
   partition to fragment store (library circuit-breaks fragment refresh;
   stale-bundle warnings escalate). The runbook is referenced from
   ADR-0246 §F and from the keystone-bundle 2026-05-20 synthesis §5.9
   runbook coverage gate.

## Implementation surface

### Library crates (workspace `crates/`)

The library is delivered as a family of crates rather than a single
mega-crate, so callers can depend only on the surfaces they use. The
crate naming follows ADR-0246 §D-3's existing `oya-shared-policy-engine-
client-*` naming, with new crate additions for the embedded evaluator
and the network-opt-in path:

| Crate | Layer (per ADR-0105) | Responsibility |
|---|---|---|
| `oya-shared-policy-engine-client-domain` | domain | Caller-facing types: `Principal`, `Action`, `Resource`, `Context`, `Decision`, `EvaluationError`, `BundleVersion`. Pure types; no I/O. |
| `oya-shared-policy-engine-client-kernel` | kernel | Trait `PolicyEvaluator` with the async `evaluate` / `evaluate_batch` surface. Trait `PolicyDecisionCache`. Trait `BundleSubscriber`. Pure trait; no concrete adapter. |
| `oya-shared-policy-engine-client-evaluator-app` | app | Default composition of in-process Cedar v4.2 evaluator + bundle cache + tenant overlay merger + pack overlay merger + audit-emit + OTel propagation + brown-out signal. The library's default `PolicyEvaluator` impl. **This crate is the library-first default.** |
| `oya-shared-policy-engine-client-bundle-cache` | adapter | LRU cache of compiled `cedar-policy::PolicySet` per (cell, tenant). Watcher thread for `FragmentPublished` Kafka events. |
| `oya-shared-policy-engine-client-bundle-fetcher` | adapter | gRPC fetcher for `GET /v1/fragments/bundle/{cell_id}/{tenant_id?}`. Signature-verifies the bundle via `oya-policy-engine-signing-chain-domain` before accepting. |
| `oya-shared-policy-engine-client-tenant-overlay` | adapter | Re-export of `oya-policy-engine-tenant-overlay-domain` for in-process tenant overlay merge at bundle-refresh time. |
| `oya-shared-policy-engine-client-pack-overlay` | adapter | Re-export of `oya-policy-engine-pack-overlay-domain` for in-process pack overlay merge at bundle-refresh time. |
| `oya-shared-policy-engine-client-signing-chain` | adapter | Re-export of `oya-policy-engine-signing-chain-domain` for in-process signing-chain verification at refresh time. |
| `oya-shared-policy-engine-client-audit-emit` | adapter | Audit emission via `oya-shared-audit-chain-client` + Slice-2 sidecar key-holder UDS call for signing. Emits `PolicyEvaluated`, `PolicyForbid`, `PolicyEmergencyPermitApplied` rows. |
| `oya-shared-policy-engine-client-network-opt-in` | adapter | Optional crate for callers that opt in to network-side evaluation (per-tenant `network_only` or per-SecretReference `policy_evaluation_network_opt_in = TRUE`). Wraps `Evaluate` and `EvaluateBatch` gRPC against the Policy-Engine µservice. Includes circuit breaker per ADR-0243 §D-11 fail-closed default. |
| `oya-shared-policy-engine-client-sdk` | sdk | High-level Rust SDK exposing `policy_engine.evaluate(...)`, `evaluate_batch(...)`, `permit_or_forbid_call`, `feature_gate`, `data_class_check`. Composes the above crates. **This is the public façade callers depend on.** |

Each crate is independently version-pinned in the workspace. Callers
depend on `oya-shared-policy-engine-client-sdk`; the SDK transitively
pulls the evaluator-app + bundle-cache + signing-chain + audit-emit by
default. The `network-opt-in` crate is an optional dependency added
only by callers in untrusted-cell-tier deployments or tenants under
`network_only` mode.

### Rust trait + module surface

The canonical evaluator trait:

```rust
// crates/oya-shared-policy-engine-client-kernel/src/lib.rs
use async_trait::async_trait;
use crate::domain::{Action, Context, Decision, EvaluationError, Principal, Resource};

/// The library-first evaluator surface. Every caller depends on this
/// trait via the SDK façade and obtains an implementation from the
/// composition root (typically `LibraryFirstPolicyEvaluator` from the
/// evaluator-app crate).
#[async_trait]
pub trait PolicyEvaluator: Send + Sync {
    /// Single-evaluation call. In-process by default; network-opt-in
    /// when the tenant attribute or SecretReference flag selects it.
    async fn evaluate(
        &self,
        principal: &Principal,
        action: &Action,
        resource: &Resource,
        context: &Context,
    ) -> Result<Decision, EvaluationError>;

    /// Batch evaluation. The library batches in-process for the default
    /// path; the network-opt-in adapter batches at the gRPC layer.
    async fn evaluate_batch(
        &self,
        requests: &[(Principal, Action, Resource, Context)],
    ) -> Result<Vec<Decision>, EvaluationError>;
}

/// Network-opt-in selector. The library composes this with the in-
/// process evaluator to choose, per call, whether to evaluate locally
/// or via the µservice gRPC. The default implementation reads the
/// tenant's `policy_evaluation_mode` + the SecretReference's
/// `policy_evaluation_network_opt_in` + the per-cell `local_bundle_age`
/// + the Cedar fragment `intelligence-network-side-opt-in.cedar`
/// analogue (`policy-engine-network-side-opt-in.cedar`).
#[async_trait]
pub trait NetworkOptInSelector: Send + Sync {
    async fn select_path(
        &self,
        principal: &Principal,
        resource: &Resource,
        context: &Context,
    ) -> Result<EvaluationPath, EvaluationError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationPath {
    LibraryInProcess,
    NetworkViaMicroservice,
}
```

The library-first composition root:

```rust
// crates/oya-shared-policy-engine-client-evaluator-app/src/lib.rs
use cedar_policy::{Authorizer, PolicySet, Request, Schema};
use crate::bundle_cache::CompiledBundleCache;
use crate::network_opt_in::DefaultNetworkOptInSelector;

pub struct LibraryFirstPolicyEvaluator {
    bundle_cache: Arc<CompiledBundleCache>,
    cedar_authorizer: Authorizer,
    schema: Arc<Schema>,
    network_selector: Arc<dyn NetworkOptInSelector>,
    network_evaluator: Option<Arc<dyn PolicyEvaluator>>, // None unless network-opt-in crate is wired.
    audit_emitter: Arc<dyn AuditEmitter>,
    tracer: Arc<dyn Tracer>,
}

#[async_trait]
impl PolicyEvaluator for LibraryFirstPolicyEvaluator {
    async fn evaluate(
        &self,
        principal: &Principal,
        action: &Action,
        resource: &Resource,
        context: &Context,
    ) -> Result<Decision, EvaluationError> {
        let _span = self.tracer.start_span("policy_engine.evaluate");

        // Step 1: network-opt-in selector chooses the path.
        let path = self.network_selector
            .select_path(principal, resource, context)
            .await?;

        // Step 2a: if the network path is selected, delegate.
        if path == EvaluationPath::NetworkViaMicroservice {
            let network = self.network_evaluator
                .as_ref()
                .ok_or(EvaluationError::NetworkPathSelectedButCrateNotWired)?;
            return network.evaluate(principal, action, resource, context).await;
        }

        // Step 2b: library-first default. Evaluate in-process.
        let bundle = self.bundle_cache.get_for_tenant(resource.tenant_id()).await?;
        let cedar_request = Request::new(
            principal.into(),
            action.into(),
            resource.into(),
            context.into(),
            Some(&self.schema),
        )?;
        let response = self.cedar_authorizer.is_authorized(
            &cedar_request,
            bundle.policy_set(),
            bundle.entities(),
        );

        let decision: Decision = response.into();

        // Step 3: emit audit row via sidecar key-holder UDS.
        self.audit_emitter
            .emit_policy_evaluated(principal, action, resource, &decision)
            .await?;

        Ok(decision)
    }

    async fn evaluate_batch(
        &self,
        requests: &[(Principal, Action, Resource, Context)],
    ) -> Result<Vec<Decision>, EvaluationError> {
        // Batch evaluation: group by tenant for bundle reuse; evaluate
        // each in-process; emit a single audit row carrying the batch
        // hash. Implementation omitted.
        unimplemented!()
    }
}
```

The default network-opt-in selector:

```rust
// crates/oya-shared-policy-engine-client-evaluator-app/src/network_opt_in.rs
pub struct DefaultNetworkOptInSelector {
    tenant_attribute_cache: Arc<TenantAttributeCache>,
    secret_reference_cache: Arc<SecretReferenceCache>,
    local_bundle_age_provider: Arc<dyn LocalBundleAgeProvider>,
    cedar_evaluator_for_opt_in_gate: Arc<dyn PolicyEvaluator>,
        // Self-referential cell: the opt-in gate itself is a Cedar
        // evaluation, but it always runs library-first. The recursion
        // bottoms out because the opt-in-gate fragment is part of the
        // baseline bundle that always loads at library startup.
}

#[async_trait]
impl NetworkOptInSelector for DefaultNetworkOptInSelector {
    async fn select_path(
        &self,
        principal: &Principal,
        resource: &Resource,
        context: &Context,
    ) -> Result<EvaluationPath, EvaluationError> {
        let tenant = self.tenant_attribute_cache
            .get(resource.tenant_id())
            .await?;

        // Fast path: tenant explicitly opted out of network.
        if tenant.policy_evaluation_mode == PolicyEvaluationMode::LibraryFirst {
            return Ok(EvaluationPath::LibraryInProcess);
        }

        // Fast path: tenant explicitly opted into network-only.
        if tenant.policy_evaluation_mode == PolicyEvaluationMode::NetworkOnly {
            return Ok(EvaluationPath::NetworkViaMicroservice);
        }

        // Conditional path: attested fallback. Consult local bundle age
        // and tenant-declared threshold.
        let bundle_age = self.local_bundle_age_provider
            .age_for_tenant(resource.tenant_id())
            .await?;
        if bundle_age > tenant.attested_fallback_threshold {
            return Ok(EvaluationPath::NetworkViaMicroservice);
        }

        // Per-SecretReference opt-in (when the resource carries a
        // SecretReference reference).
        if let Some(secret_ref_id) = resource.secret_reference_id() {
            let secret_ref = self.secret_reference_cache.get(secret_ref_id).await?;
            if secret_ref.policy_evaluation_network_opt_in {
                return Ok(EvaluationPath::NetworkViaMicroservice);
            }
        }

        // Default: library-in-process.
        Ok(EvaluationPath::LibraryInProcess)
    }
}
```

### Opt-in µservice surface (`microservices/policy-engine/`)

The µservice retains the eight BCs from ADR-0246 §D-2 unchanged in
their *responsibility split*. What changes is the **default consumption
shape**: the `Evaluate` gRPC endpoint is *opt-in only* and is not the
default caller surface. The runtime endpoints are:

| µservice endpoint | Purpose | Caller-facing path |
|---|---|---|
| `POST /v1/evaluate` | Opt-in caller submits an evaluation request for network-side processing. | Opt-in only. Caller library calls when tenant `policy_evaluation_mode != library_first` OR SecretReference `policy_evaluation_network_opt_in = TRUE`. |
| `POST /v1/evaluate-batch` | Batch evaluation for opt-in callers. | Opt-in only. Same gating. |
| `GET /v1/fragments/bundle/{cell_id}` | Library polls / pulls the canonical fragment bundle for a cell. | Non-call-path; refresh-time. |
| `GET /v1/fragments/bundle/{cell_id}/{tenant_id}` | Library polls / pulls the per-tenant compiled overlay bundle. | Non-call-path; refresh-time. |
| `POST /v1/fragments/publish` | Fragment authoring: publish a new fragment to the registry. | Authoring path only; not on caller hot path. |
| `POST /v1/fragments/activate` | Activate a published fragment for hot-reload. | Authoring path only. |
| `POST /v1/fragments/sunset` | Sunset a fragment per the deprecation handshake. | Authoring path only. |
| `GET /v1/fragments/{fragment_id}` | Fragment retrieval (diff, audit). | Non-call-path. |
| `GET /v1/coverage/report` | Fleet-wide coverage report (CI lane + nightly drift). | Non-call-path. |
| `POST /v1/coverage/telemetry-flush` | Library flushes batched coverage telemetry. | Non-call-path; async batched. |
| `GET /v1/signing-chain/org-root-cert` | Library polls the org root cert (refresh on rotation). | Non-call-path. |
| `GET /v1/admin/observability/evaluation-rollup` | Cross-cell evaluation rollup (read-only consumer of audit-chain stream). | Compliance dashboards + tenant admin. |

The default per-call evaluation endpoint that would have existed under
the natural reading of ADR-0246 §D-3 + §D-5 (`POST /v1/evaluate` as
the universal mediator) is **deliberately retained but marked opt-in
only**. The CI lane `oya-check-no-unnecessary-policy-engine-service-
hop` enforces this: any caller that issues `Evaluate` without a
matching `tenant.policy_evaluation_mode != library_first` declaration
or a `SecretReference.policy_evaluation_network_opt_in = TRUE` is
flagged.

### Cedar fragments (`policy-engine/fragments/baseline/`)

Three Cedar fragments govern the library + µservice boundary:

1. `policy-engine-library-evaluate.cedar` — governs the in-process
   evaluation decision (this is the baseline + per-µservice action set
   that every caller's library evaluates against; this fragment is
   self-applied at library bundle composition time).
2. `policy-engine-network-side-opt-in.cedar` — governs the opt-in
   decision per §D-5. Permit on tenant `policy_evaluation_mode !=
   library_first` OR resource `policy_evaluation_network_opt_in == true`.
3. `policy-engine-untrusted-tier-mediation.cedar` — governs the
   untrusted-cell-tier mediation path. Permit on caller residing in a
   cell whose `cell.trust_tier == untrusted` (per ADR-0248 §D-7
   trust-tier taxonomy) regardless of tenant attribute.

The three fragments are part of the baseline bundle that every library
instance loads at startup. The opt-in-gate fragment's evaluation always
runs library-first; the recursion bottoms out because the opt-in-gate
fragment is in the baseline bundle.

## Verification

### CI lanes

Seven advisory-until-bootstrap lanes promote to BLOCKER after the
items in §Status are complete:

1. **`oya-check-policy-engine-library-first-default`** (static
   analysis). Scans the workspace for any caller that issues a Policy-
   Engine `Evaluate` µservice RPC on the per-call evaluation path
   without a corresponding tenant `policy_evaluation_mode !=
   library_first` declaration or SecretReference
   `policy_evaluation_network_opt_in = TRUE`. Flags violations.
2. **`oya-check-policy-engine-network-opt-in-cedar-gated`** (static
   analysis). Confirms that every network-side `Evaluate` site is gated
   by a Cedar evaluation of `policy-engine-network-side-opt-in.cedar`
   with a declared `context.coordination_reason`. Flags ungated RPCs.
3. **`oya-check-no-unnecessary-policy-engine-service-hop`** (integration
   test). Runs the `tenancy → policy-engine-client → evaluate(...)`
   reference workflow end-to-end and asserts that zero gRPC calls hit
   `microservices/policy-engine/` on the per-call evaluation path.
   Asserts that the `Evaluate` gRPC is invoked only when the test
   fixture's tenant has `policy_evaluation_mode = network_only`.
4. **`oya-check-library-only-evaluation-failure-perimeter`** (chaos
   test). Brings down `microservices/policy-engine/` in a test cell;
   asserts that Cedar evaluation through the library continues to
   succeed for default callers; asserts that opted-in callers either
   fall back per Cedar policy or fail closed per declared policy.
5. **`oya-check-library-fragment-snapshot-coherence`** (unit + integration
   test). Confirms that the library's in-process compiled bundle
   matches the µservice's authoritative bundle within the <5s p99 hot-
   reload SLO. Asserts no policy drift between library and µservice.
6. **`oya-check-library-audit-emission-coherence-policy-engine`** (unit
   + integration test). Confirms that the library emits the
   `PolicyEvaluated`, `PolicyForbid`, `PolicyEmergencyPermitApplied`
   audit rows from the *caller's* process (verified by checking the
   audit-chain seal's emitting principal matches the caller's SPIFFE-
   ID, not Policy-Engine's SPIFFE-ID).
7. **`oya-check-library-credential-sidecar-coherence-policy-engine`**
   (integration test). Confirms that the library's audit-signing path
   uses the Slice-2 sidecar key-holder (per ADR-NNNN-library-first-
   credential-sidecar) and never holds the audit-signing key in the
   caller's main process memory beyond the immediate signing UDS
   call. Verifies that RCE in the caller's main process does not expose
   the audit-signing key.

### Manual verification gates

1. The reference architecture diagram at
   `docs/architecture/policy-engine-substrate-runtime-topology.md`
   shows the library-first path as the default solid edge and the
   network hop (untrusted-tier mediation + per-tenant `network_only`
   opt-in) as a dashed opt-in edge.
2. The ADR-0246 §D-1 + §D-3 + §D-5 frontmatter is annotated with a
   forward pointer to this amendment.
3. The `tenants` migration ships the `policy_evaluation_mode` column
   per the follow-up ADR-0244 tenant DDL extension (tracked separately
   in §F-1).
4. The `secret_references` migration ships the
   `policy_evaluation_network_opt_in` column and the
   `policy_evaluation_network_opt_in_reasons` array column.
5. The Foundry CI agent + tenancy admission reference workflows are
   documented in Appendix B as canonical worked examples.

## Migration

### F-1. ADR-0244 tenant DDL extension required (follow-up, not edited here)

This amendment introduces a new column on the `tenants` table:

```sql
CREATE TYPE policy_evaluation_mode_t AS ENUM (
    'library_first',
    'network_only',
    'library_first_with_attested_fallback'
);

ALTER TABLE tenants
    ADD COLUMN policy_evaluation_mode policy_evaluation_mode_t NOT NULL DEFAULT 'library_first';

ALTER TABLE tenants
    ADD COLUMN attested_fallback_threshold INTERVAL
        NOT NULL DEFAULT '24 hours'::INTERVAL;
```

The column **is not added in this amendment** — this amendment defers
the DDL change to a follow-up ADR-0244 extension. The follow-up extends
ADR-0244 §D-3 (tenant DDL) with the new column + type, the corresponding
Cedar entity-schema attribute, and the spec/tenant-model.json property.

The follow-up is tracked as keystone-bundle synthesis §5.13 follow-up
"library-first symmetry promotion-gate fix tenant DDL extension."

### F-2. SecretReference DDL extension required (follow-up, not edited here)

Parallel to F-1, the `secret_references` table needs the two columns
per §D-5. This is also a separate follow-up, paralleling the
ADR-0255-amendment's `network_side_opt_in` columns.

### F-3. Migration path for existing callers

All existing callers default to library-first. No caller code changes
are required for the default path. Callers in untrusted-cell-tier
deployments + tenants under `network_only` mode opt in by declaring
the new tenant attribute; the library handles the selector
automatically.

### F-4. Sunset of any in-flight `evaluate-via-service` paths

If any in-flight branch has authored `evaluate_via_service(...)` calls
against the µservice gRPC as the default path, those calls are
rewritten to use the `oya-shared-policy-engine-client-sdk::evaluate(...)`
SDK façade. The SDK façade selects library-first by default. The
sunset is a one-time sweep PR, separate from this amendment.

### F-5. Coverage of ADR-0246 §Status items

ADR-0246 §Status item 3 reads "the SDK (`oya-shared-policy-engine-client`)
is consumable by at least one downstream µservice (pilot:
`microservices/tenancy/`)". This amendment adds the requirement that
the pilot's consumption is **in-process evaluation**, not gRPC mediation.
The integration test under §Verification 3 above is the gating evidence
for this Status item.

## References

- **ADR-0145** — Inter-microservice communication reform (2026-05-18).
  Retires the universal-mediator pattern; establishes three weaker
  invariants (audit, tracing, ontology projection); direct service-
  to-service is the default. This amendment applies that doctrine to
  Policy-Engine.
- **ADR-0150** — Cedar policy engine. The canonical library client
  pattern this amendment extends.
- **ADR-0176** — Brown-out degradation signal. Library emits local;
  µservice emits cell-wide aggregate on opt-in.
- **ADR-0211** — In-house tech stack preference. OpenBao for
  signing-chain intermediate keys; HSM for org root.
- **ADR-0212** — Buildability doctrine. Library version pinning
  preserves the build-to-green property across callers.
- **ADR-0242** — `oyatie` is a tenant. Bootstrap sequence puts policy-
  engine at step 5; the library + µservice split preserves the sequence.
- **ADR-0243** — Cedar as universal gate. Every state-changing call
  consults Cedar; this amendment specifies that the consultation is
  library-first.
- **ADR-0244** — Tenant as universal scoping primitive. The new
  `policy_evaluation_mode` enum extends the tenant DDL (per F-1).
- **ADR-0245** — Substrate-vs-product layering. Policy-Engine remains
  a substrate; the library shape is the consumption surface.
- **ADR-0247** — Self-hosting / self-modification doctrine.
  `oyatie.foundry.*` workflows evaluate Cedar in-process via the library
  on the default path.
- **ADR-0248** — Amazon-shape cellular architecture. The untrusted-tier
  mediation path in §D-4 references ADR-0248 §D-7 cell-trust-tier
  taxonomy.
- **ADR-0255** — Intelligence as two-layer AI substrate.
- **ADR-0355-amendment-library-first-network-opt-in-clarification** —
  The structural twin of this amendment for Intelligence. This
  amendment is 1-of-2 in the F4 library-first symmetry promotion-gate
  fix; the ADR-0257 amendment is 2-of-2.
- **ADR-NNNN-library-first-credential-sidecar** (Slice-2, number
  pending assignment) — Sidecar key-holder primitive for audit-signing
  key + provider-credential isolation. Referenced by this amendment's
  §D-2 audit-emission row.
- **AWS IAM caller-side evaluation** — Public IAM policy evaluation
  library (`software.amazon.awssdk.auth.policy`) referenced by every
  AWS SDK; runs policy evaluation in-process against a cached policy
  set; control plane consulted only for policy authoring + refresh.
  The reference shape this amendment adopts.
- **AWS Verified Permissions** (2024-Q1 GA) — Managed Cedar service.
  Publishes batch evaluation and cached client modes specifically so
  callers can avoid the per-call network hop. Control plane for
  fragment authoring; data plane is caller-side.
- **Cedar v4.2 Rust crate** (`cedar-policy`, AWS open-source) —
  Designed for in-process evaluation with hot-reloadable signed bundle.
  The evaluation kernel embedded by `oya-shared-policy-engine-client-
  evaluator-app`.
- **Google Zanzibar** (Pang et al., USENIX ATC 2019) — Distributed
  authorization with caller-side cached relations + central namespace
  service for relation publishing. The architectural shape this
  amendment follows (caller-side eval + central authoring).
- **Open Policy Agent (OPA)** — Embedded mode (`opa eval` against a
  compiled bundle) is the canonical library-first pattern for policy
  evaluation across the industry. Referenced by Netflix, Pinterest,
  and Stripe public engineering posts.
- **Stripe Connect** — Caller-side rate-limit token bucket pattern;
  central service for budget authoring; caller library for per-call
  enforcement. The same architectural separation this amendment
  applies to Cedar evaluation.
- **James Hamilton 2007 LISA** — "On Designing and Deploying Internet-
  Scale Services." Formalizes static stability and the principle
  that the data path must not depend on the control path being up.
- **AWS Builder's Library** — "Avoiding Cascading Failures" (2019)
  and "Avoiding insurmountable queue backlogs" (2020). Document
  the SLO ceiling + failure perimeter pathology that arises when a
  central coordinator sits on the data path; establish circuit
  breaker + hedged requests + static stability as the canonical
  mitigations.
- **AWS Builder's Library** — "Static stability using Availability
  Zones" (2020). Same principle at the AZ scope; this amendment
  transplants it to the intra-cell control-plane scope.
- **Martin Fowler 2014** — "Microservices and the First Law of
  Distributed Object Design." Critique of the ESB 2.0 anti-pattern;
  argues against centralized smart mediators in favor of dumb pipes
  + smart endpoints.
- **IEEE 2017** — "Why Enterprise Service Buses Failed." Empirical
  retrospective on ESB deployments.
- **`feedback_cedar_as_universal_gate`** — Memory establishing Cedar
  as universal gate; this amendment specifies the library-first
  delivery shape.
- **`feedback_quality_performance_scalability_bar`** — Hyperscaler-
  grade performance + horizontal scalability. The library-first
  default is required to honor this bar for state-changing
  functionality.
- **`feedback_no_silent_regression`** — Linus-style protection of
  public contracts. The amendment's `oya-check-no-unnecessary-policy-
  engine-service-hop` lane is the CI enforcement.
- **`feedback_autonomous_implementation_artifacts`** — Long-term
  goal that "Implement the masterplan" runs without coordination.
  Library version pinning per-caller preserves this.
- **F4-Architecture verdict** —
  `evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`
  finding F4-LV-6 / F4-AP-1 / F-ANTI-2. Authority for this amendment.
- **Keystone-bundle 2026-05-20 synthesis** —
  `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.13
  promotion-gate fix library-first-symmetry-1-of-2.

## Appendix A — Hyperscaler-pattern attribution

The library-first / network-opt-in shape is not novel. It is the
canonical pattern across every hyperscaler reference cited in the
ADR-0246 + ADR-0150 + ADR-0145 corpus. The attribution is explicit so
that future readers do not re-derive the pattern by trial and error.

| Reference | Library | Network coordination | Shape |
|---|---|---|---|
| **AWS IAM** (production reality) | In-process IAM authorization library bundled with every AWS SDK. | `iam.amazonaws.com` control plane for policy authoring + refresh; not on per-call path. | Library-first; control plane for authoring. |
| **AWS Verified Permissions** (2024-Q1 GA) | Cached client mode in caller's process; in-memory policy set; batch evaluation. | Verified Permissions control plane for fragment authoring + schema + audit. | Library-first; cached client is the explicit recommendation. |
| **Cedar v4.2 Rust crate** (`cedar-policy`) | In-process evaluator; `PolicySet::from_str(&bundle)` + `Authorizer::new()` + `is_authorized(...)`. | None at the crate level; integration with a registry is left to the consumer. | Library-only by design; the registry is a separate concern. |
| **Open Policy Agent (OPA)** | Embedded mode: `opa eval` against a compiled bundle in-process. | OPA control plane for bundle publishing + decision-log aggregation. | Library-first; control plane for bundle distribution. |
| **Google Zanzibar** (USENIX ATC 2019) | Caller-side cached relation tuples in every service's process. | Zanzibar central service for namespace authoring + relation publishing. | Library-first; central authoring + caller-side eval. |
| **Stripe Connect** (caller-side rate-limit token bucket) | In-process token bucket per credential. | Central budget service for opt-in fan-out coordination. | Library-first; central coordination opt-in. |
| **Netflix Falcor / Tartan / Sleipnir** (public posts) | Caller-side cached policy in every service. | Central policy publishing. | Library-first. |
| **Pinterest OPA-at-edge** (public post 2021) | OPA sidecar embedded mode + bundle pull from central. | Central bundle publishing. | Sidecar-embedded; same shape modulo sidecar vs library choice. |
| **`oya-shared-audit-chain-client`** (per ADR-0145 Invariant 1) | Library in caller's process. | Audit-chain µservice for canonical Merkle storage. | Library-first; the audit µservice is the storage tier, not a mediator. |
| **`oya-shared-tracing-client`** (per ADR-0145 Invariant 2) | OTel SDK in caller's process. | Tempo for storage. | Library-first; Tempo is not on the call path. |
| **`oya-shared-intelligence-client-*`** (per ADR-0255 amendment) | Library family for LLM dispatch. | Intelligence µservice for cross-cutting coordination. | Library-first; the structural twin of this amendment. |

The convergence is unambiguous. Every reference at the hyperscaler bar
uses the library-first / network-opt-in shape for *per-call evaluation*
while retaining the central service for *fragment authoring +
distribution + audit*. Policy-Engine joins the pattern.

## Appendix B — Worked example: Tenancy admission check via the library

This appendix walks through a single Cedar evaluation from
`microservices/tenancy/` (per ADR-0246 §Status item 3 pilot µservice)
checking whether `oyatie.tenant.admin@acme-corp` may invoke
`Tenancy::Action::CreateSubScope` on `tenant::acme-corp`. The example is
the canonical reference path the `oya-check-no-unnecessary-policy-
engine-service-hop` lane asserts against.

### Setup

- **Caller principal:** `oyatie.tenant.admin@acme-corp` (per ADR-0242
  tenant doctrine; `acme-corp` is a B2B tenant).
- **Action:** `Tenancy::Action::CreateSubScope`.
- **Resource:** `tenant::acme-corp`.
- **Cell:** `cell-us-east-1` (Tier 3 data-plane cell;
  `cell.trust_tier == standard`).
- **Tenant attribute:** `acme-corp.policy_evaluation_mode = library_first`
  (default).
- **Cedar fragments active:** `policy-engine-library-evaluate.cedar`,
  `tenancy-sub-scope-creation-permits.cedar`,
  `tenancy-baseline-tenant-default-deny.cedar`.

### Step-by-step

1. **Caller constructs the request.** The Tenancy µservice's request
   handler (admission endpoint for `POST /v1/tenants/{id}/sub-scopes`)
   constructs a `(Principal, Action, Resource, Context)` tuple via
   `oya-shared-policy-engine-client-domain`.
2. **Library opens an OTel span.** `oya-shared-tracing-client` opens a
   child span under the caller's existing span: `policy_engine.evaluate`.
   Span attributes: principal, action, resource, cell, tenant.
3. **Library evaluates network-opt-in selector.** `DefaultNetworkOptInSelector`
   reads `acme-corp.policy_evaluation_mode = library_first` from the
   in-process tenant attribute cache. Selector returns
   `EvaluationPath::LibraryInProcess`.
4. **Library looks up compiled bundle.** `bundle_cache.get_for_tenant(
   "acme-corp")` returns the in-memory compiled `PolicySet` for
   `(cell-us-east-1, acme-corp)`. Bundle age: 312 seconds (well within
   ADR-0243 §D-10 hot-reload SLO).
5. **Library constructs Cedar request.** `cedar_policy::Request::new(...)`
   wraps the tuple.
6. **Library evaluates.** `Authorizer::is_authorized(...)` against the
   compiled `PolicySet`. Evaluation completes in ~120 µs. Determining
   policy: `tenancy-sub-scope-creation-permits.cedar::permit-tenant-
   admin-sub-scope-create`. Decision: `Allow`.
7. **Library emits audit row via sidecar key-holder.** UDS call to the
   Slice-2 sidecar (per ADR-NNNN-library-first-credential-sidecar)
   carrying the canonical seal payload. Sidecar signs with the audit-
   chain Ed25519 key (held only in the sidecar process); returns the
   signed seal. Library forwards the sealed row to `oya-shared-audit-
   chain-client` for transmission. The seal's `emitting_principal`
   field is `tenancy.us-east-1.svc` (Tenancy's SPIFFE-ID), not
   `policy-engine.us-east-1.svc`.
8. **Library closes the OTel span.** Span ends with status `OK`.
   Decision attached as a span attribute.
9. **Caller proceeds with the state-changing call.** The Tenancy
   request handler accepts the request and creates the sub-scope.

### What did NOT happen

- No gRPC call from `microservices/tenancy/` to
  `microservices/policy-engine/`.
- No `PolicyEvaluated` audit seal emitted by Policy-Engine's SPIFFE-ID
  (the seal was emitted by Tenancy's own SPIFFE-ID; the seal's
  `emitting_principal` reflects this).
- No artificial `policy-engine-mediator` span between Tenancy and the
  Cedar evaluator in the OTel trace.
- No SLO contribution from Policy-Engine's availability to the Tenancy
  request's per-call success budget.

### What happened separately (asynchronously)

- The coverage telemetry batch flush sends per-fragment hit counters to
  Policy-Engine's `/v1/coverage/telemetry-flush` endpoint every 60
  seconds. The flush is non-blocking; failure to flush does not block
  evaluation.
- The audit-chain stream consumer at Policy-Engine ingests the
  `PolicyEvaluated` row asynchronously. The cross-cell evaluation
  rollup catches up.

### Hypothetical opt-in variant: untrusted-cell-tier caller

If the same evaluation were attempted by a caller residing in
`cell-edge-mumbai-1` whose `cell.trust_tier = untrusted`:

- Step 3 selector reads cell trust tier from cell context. Per
  `policy-engine-untrusted-tier-mediation.cedar`, untrusted-tier
  callers route to network regardless of tenant attribute. Selector
  returns `EvaluationPath::NetworkViaMicroservice`.
- Step 4-6 are replaced with a single gRPC `Evaluate` call to
  `policy-engine.cell-edge-mumbai-1.svc`. The µservice's evaluator pod
  performs the evaluation against the centrally-attested compiled
  bundle and returns the decision.
- Step 7 audit emission still happens at the caller (Tenancy in
  untrusted cell) via its co-located sidecar key-holder. The emitting
  principal is Tenancy's SPIFFE-ID; the seal carries an extra attribute
  `evaluation_path = network_via_microservice` for compliance
  observability.

### Hypothetical opt-in variant: `network_only` tenant

If the same evaluation were attempted for a tenant with
`policy_evaluation_mode = network_only`:

- Step 3 selector returns `EvaluationPath::NetworkViaMicroservice`
  immediately upon reading the tenant attribute.
- Steps 4-6 as in the untrusted-tier variant.
- Step 7 audit emission as in the untrusted-tier variant.

### Why this matters

This worked example demonstrates that the canonical authorization-
mediated workflow in the platform (a Tenancy admission check) does
**not** require `microservices/policy-engine/` to be up. The platform's
state-changing capability — the property that every state-changing call
proceeds without an additional mediator hop — is preserved across
Policy-Engine µservice outages, upgrades, schema migrations, and even
across regional Policy-Engine partitions. That is the static-stability
guarantee Hamilton 2007 prescribed, that ADR-0145 codified for the
platform's inter-µservice surface, that the ADR-0255 amendment extended
to AI-mediated functionality, and that this amendment now extends to
authorization-mediated functionality.

ADR-0145's "no universal mediator" doctrine is intact and is now
actively defended on the second of three structurally identical risk
axes.

---

*End of amendment.*
