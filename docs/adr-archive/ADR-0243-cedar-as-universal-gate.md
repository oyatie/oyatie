---
id: ADR-0243
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-policy-engine
  - axis-identity
  - axis-audit-chain
  - ops-compliance
  - ops-sre-reliability
supersedes: []
amends:
  - ADR-0150-cedar-policy-engine.md (extends scope from authorization to all gates)
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md (clarifies Cedar's scope)
superseded_by: [ADR-0700]
related:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0140-cedar-policy-enforcement.md (retired per ADR-0145; superseded fragments live in policy-engine)
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/cedar-fragment-schema.json
  - /specs/policy-gate-coverage.json
related_memory:
  - feedback_cedar_as_universal_gate
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_automate_everything
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 2-of-14
purpose: >
  Establish that EVERY routing, authorization, activation, attribution,
  retention, eligibility, gate, and decision-with-policy-implication
  in the oyatie platform is evaluated by the Cedar policy engine.
  Code never decides policy; code asks the policy engine and acts on
  the answer. Policy lives in versioned, signed, hot-reloadable Cedar
  fragments. Coverage is CI-enforced — every gate has a permit
  fragment plus a default-deny.
enforcement_status: advisory-until-policy-engine-substrate-lands
enforced_by:
  - oya gate validate cedar-coverage
  - oya gate validate no-policy-in-code
  - oya gate validate cedar-fragment-signature
  - oya gate validate cedar-default-deny-coverage
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0243: Cedar as Universal Gate

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive). Lands as a single multispectrum-reviewed PR.

Enforcement is `advisory-until-policy-engine-substrate-lands`. Becomes
BLOCKER once:

1. `microservices/policy-engine/` is promoted from Ontology BC to peer
   substrate µservice (per ADR-0246).
2. The Cedar coverage lane scans every µservice and produces a per-
   µservice coverage report with no fewer than the minimum-required
   gate set per the catalog in §D-3.
3. Hot-reload + signature-verification + per-tenant-overlay paths are
   exercised by integration tests in `microservices/policy-engine/tests/`.

## Date

2026-05-20.

## Context

### Prior portfolio state

Cedar policy was introduced in three increments:

- **ADR-0140 (Cedar policy enforcement, retired per ADR-0145).**
  Original framing of Cedar as authorization-only.
- **ADR-0150 (Cedar policy engine, 2026 LTS v4.2).** Establishes Cedar
  v4.2 as the *policy engine for app-tier authorization*. Scopes
  permits + forbids; default-deny pattern; per-Action Cedar fragment
  coverage CI lane (`oya-governance-cedar-coverage`); Cedar
  fragment registry in Ontology BC `cedar-fragment-coverage`.
- **ADR-0183 (Cedar app authz + Kyverno admission, 2026).** Separates
  responsibility: Cedar gates app-tier authorization decisions;
  Kyverno gates Kubernetes admission decisions. Both run in parallel.

This was the right starting point but **scoped Cedar narrowly to
"app-tier authorization."** Many other decisions in the portfolio are
*also* policy decisions, but they were authored as imperative code or
configuration:

| Policy decision in code today | Where it lives | Why it's a policy decision |
|---|---|---|
| **Provider routing in Intelligence (data-class → LLM provider)** | hard-coded in foundry-providers BC adapter selection logic | "PHI cannot go to providers outside HIPAA-eligible list" is a policy, not an algorithm |
| **Cell routing (tenant → home_cell)** | per-tenant config in tenancy substrate; ad-hoc dispatcher | "EU-resident tenant must route to EU cells" is a policy |
| **Tax / region routing (which tenant lives in which jurisdiction)** | per-tenant `jurisdiction_code` static field | "KR-FSS-regulated tenant cannot egress data to non-KR cell" is a policy |
| **Audit-stream selection (which event emits to which stream)** | hard-coded in each µservice's audit-emit code | "Source-code-class events emit to oyatie.foundry stream, not customer stream" is a policy |
| **Cost-center attribution (which cost center bills which action)** | hard-coded in finops aggregator | "Sub-scope inheritance rolls cost up to parent unless override" is a policy |
| **Feature flag evaluation (which features for which tenants)** | LaunchDarkly-equivalent SDK calls scattered in product code | Feature gates ARE policies; they belong in policy-engine, not a separate vendor |
| **Schema-revision activation (when does Ontology schema v2 go live)** | imperative migration runner | "Schema v2 activates after coverage gate green + tenant-pinned drains" is a policy |
| **Marketplace surface eligibility (plugins → which app-store surface)** | imperative classification | "Plugin in healthcare category requires HIPAA-pack approval" is a policy |
| **Compliance pack activation per tenant** | tenant config + ad-hoc gating | "Pack activation requires KYB-verified + jurisdiction-match + DPIA signed" is a policy |
| **DSAR cascade scope (which data classes to erase)** | imperative cascade enumeration | "Article 17 cascade must include all data classes carrying subject identifier *unless* under legal hold" is a policy |
| **Retention sunset (what gets cold-storage tiered when)** | per-jurisdiction static config | Retention rules are policies (jurisdiction overlay can override) |
| **Rate limit + quota tiering (which tier for which tenant)** | tenant_tier config + admission control | Tier-to-quota mapping is a policy |
| **Cross-cell traffic permits (which calls may cross cells)** | mTLS + NetworkPolicy | NetworkPolicy enforces; Cedar should *decide* per-call |
| **encryption-BYOK permit (can tenant bring their own KMS key for data class X)** | tenant config + ad-hoc gates | encryption-BYOK approval is a policy |
| **Webhook subscription eligibility (which events can a tenant subscribe to)** | hard-coded allowlist | Event-class eligibility is a policy |
| **Cron-trigger eligibility (can principal schedule recurring workflow)** | imperative checks | Scheduled-action eligibility is a policy |
| **Sandbox tenant lifetime + resource budget** | static config | Per-engineer + per-PR budgets are policies |
| **Bulk import/export eligibility (which data classes are bulk-exportable)** | imperative checks | Data-portability policy decisions |
| **Plugin Wasmtime capability allowlist (which capabilities can a plugin call)** | manifest declaration → static check | Capability allowance is a policy |
| **Audit retention extension (legal hold supersedes sunset)** | imperative logic | Hold supersession is a policy |
| **Cross-tenant collaboration permit (share workflow A→B)** | invite/accept flow | Cross-tenant sharing approval is a policy |
| **Partner-tenant on-behalf-of (agency → customer assume-role)** | new code path | Pre-authorized assume-role is a policy |
| **Reserved-namespace registration refusal (oyatie + variants)** | tenancy admission logic | Reserved-namespace is a policy (per ADR-0242) |

That's **23 policy-class decisions currently authored as imperative
code or static config.** Each is a place where:

- Policy *drifts* over time (intent in head; code lags).
- Audit *misses* the decision rationale (the why, not just the what).
- Tests *don't cover* the decision uniformly (each gate has its own
  test style).
- Per-tenant *customization* requires code change, not config.
- Per-jurisdiction *overlay* requires per-µservice modification.

### What "Cedar as Universal Gate" means

The doctrine establishes:

1. **Every gate listed above (and any new gate introduced) is a Cedar
   evaluation.** Code requests a decision from the policy engine,
   receives `Permit | Forbid | NotApplicable`, and acts.
2. **The policy engine is the source of truth.** No imperative code
   shall encode policy. Coverage CI lane refuses µservice code that
   makes policy decisions.
3. **Cedar fragments are versioned, signed, hot-reloadable.** Fragment
   evolution follows a controlled lifecycle: author → multispectrum
   review → sign → publish → activate → audit.
4. **Per-tenant overlays compose at evaluation time.** Tenant T's
   active compliance packs contribute Cedar fragments; baseline
   fragments + pack overlays are evaluated together; deny wins ties.
5. **Bootstrap fragment set signed by org root key.** Cedar bootstraps
   itself: the first fragment that gates fragment-registry access is
   signed by an org root key held in a tier-0 HSM (Shamir-shared).
6. **Coverage CI lane: every gate has a permit fragment + default-deny.**
   No gate is in code without a Cedar fragment. No permit without a
   complementary default-deny.
7. **Performance: < 1ms p99 evaluation per gate.** In-cell cache
   (Valkey hot-cache, 1s TTL); per-cell Cedar evaluator pod (DaemonSet
   for hot-path callers; sidecar-equivalent for batch callers).
8. **Audit: every Cedar decision emits to audit chain.** Audit row
   carries `{principal, action, resource, context, decision, applied_fragments[], evaluation_ms}`.

### Why a unified policy engine matters at hyperscaler scale

Three named references:

- **AWS Verified Permissions (launched 2023, GA 2024) uses Cedar as the
  underlying engine.** AWS recognised that having multiple policy
  systems (IAM for auth, SCP for org constraints, Verified Permissions
  for app authz, fine-grained policies for resources) creates drift.
  Verified Permissions consolidates app-tier policy under Cedar.
- **Google Org Policy + IAM Conditions.** Google Cloud consolidated
  organization-level policy under a single Org Policy service with
  IAM Conditions for fine-grained predicates. Multiple-policy-systems
  was retired.
- **Open Policy Agent (OPA) adoption at Netflix, Pinterest, T-Mobile,
  Capital One.** All converged on OPA as the single policy engine
  across authorization, admission, data-filtering, configuration
  validation. The architectural learning is unambiguous.

For oyatie, we use Cedar (not OPA) per ADR-0150 + ADR-0183 because:

- Cedar's type system + analysis tools (cedar-policy-analyzer)
  enable formal reasoning about policy completeness.
- Cedar v4.2 LTS is AWS-backed with long-term maintenance commitment.
- Cedar's design is finite-time decidable (no Turing-complete
  policies); evaluation latency is bounded.
- Cedar fragments compose under a simple union-with-tie-break-by-
  deny semantic, suitable for per-tenant overlay.

### What this is NOT

This ADR is NOT:

- A rejection of Kyverno (admission-tier policy stays Kyverno per
  ADR-0183).
- A demand to retire OPA if it's currently used elsewhere (OPA usage
  in the portfolio is currently zero per `microservices/policy-engine/`
  spec; this ADR continues that).
- A demand that all *configuration* be Cedar (some config genuinely
  is static — e.g., port numbers, container resource limits — and
  remains in Helm values + ConfigMaps + manifests).
- A demand that all *business logic* be Cedar. Business logic (the
  algorithm for, say, computing tax on a marketplace order) remains
  in code; the policy that *decides which tax engine to use* (US sales
  tax vs EU VAT vs KR 부가세) is Cedar.

The bright line: **policy decisions are Cedar; algorithmic decisions
are code; configuration is config; admission is Kyverno.**

## Decision

### D-1. Cedar evaluates every policy-class decision

The 23 policy-class decisions enumerated in §Context are migrated to
Cedar evaluation. New policy-class decisions introduced by future ADRs
are Cedar from inception.

The canonical Cedar evaluation contract:

```rust
// microservices/policy-engine/src/api.rs

pub struct EvaluationRequest {
    pub principal: Principal,             // who is acting
    pub action: Action,                   // what action
    pub resource: Resource,               // on what resource
    pub context: BTreeMap<String, AttributeValue>,  // contextual attributes
    pub tenant_id: TenantId,              // tenant scope
    pub evaluation_id: Uuid,              // for audit trail
}

pub struct EvaluationResponse {
    pub decision: Decision,               // Permit | Forbid | NotApplicable
    pub applied_fragments: Vec<FragmentId>,  // which fragments contributed
    pub determining_policies: Vec<PolicyId>,  // which permit/forbid policies fired
    pub evaluation_ms: u32,
    pub audit_emitted: bool,
    pub annotations: BTreeMap<String, String>,  // attached rationale for human review
}

pub enum Decision {
    Permit,
    Forbid { reason: String },           // human-readable; structured below
    NotApplicable,                       // no policy matched; caller decides default (usually deny)
}
```

### D-2. Fragment lifecycle

Cedar fragments live in `microservices/policy-engine/fragments/` and
follow a versioned + signed lifecycle:

```
fragment_path: microservices/policy-engine/fragments/<scope>/<name>.cedar
scope: baseline | pack/<pack-id> | overlay/<jurisdiction> | reserved | tenant/<tenant-id>
```

**Lifecycle stages:**

1. **Authored.** Human or `oyatie.foundry.adr-drafter` workflow creates
   a fragment. Frontmatter declares: `fragment_id`, `version`, `scope`,
   `applies_to_actions`, `applies_to_resources`, `effective_at`,
   `sunset_at` (optional), `signed_by` (empty initially).
2. **Reviewed.** Multispectrum review v2.4.0 fan-out per facet:
   - F1 (correctness): does the fragment do what its frontmatter claims?
   - F2 (hyperscaler-fitness): does this match AWS Cedar idiom?
   - F5 (security): does this introduce any privilege escalation path?
   - F6 (performance): evaluation cost analysed?
   - A1 (own-policy-adherence-naming): does the fragment_id follow
     BNF v4.1 conventions?
   - A4 (architecture-adherence): does the fragment respect
     scope/overlay/pack layering?
3. **Signed.** Reviewed fragment is signed by the appropriate signing
   key per its scope:
   - `baseline/` fragments: signed by org-baseline-key (held in HSM
     by `oyatie.security.signing`).
   - `pack/<pack-id>/` fragments: signed by pack-owner-key (held by
     the compliance team responsible for that pack).
   - `overlay/<jurisdiction>/` fragments: signed by jurisdiction-
     overlay-owner-key (per-pack regulator-facing team).
   - `tenant/<tenant-id>/` fragments: signed by tenant-admin-key (per
     tenant; usually via tenant admin console).
   Signatures are Ed25519 + cosign attestation referencing the
   fragment hash.
4. **Published.** Signed fragment is published to fragment registry
   (Postgres + Citus shard on `(scope, fragment_id)` + cosign-attested
   immutable blob in SeaweedFS). Publication emits to audit chain
   (class `CedarFragmentPublished`).
4a. **Soaking** *(per ADR-0294 §D-1).* Before full enforcement, the
   fragment enters a mandatory soak window of ≥60s (enforced by the
   fragment-publisher admission gate: `sunset_at - activate_at >= 60s`
   MUST hold). During soaking the fragment participates in **shadow
   evaluation only** — it is applied in a parallel evaluation path and
   its verdicts are recorded to audit for anomaly analysis, but it does
   NOT affect enforcement decisions. The `oyatie.policy-engine.fragment-
   soak-detector` automation monitors permit-rate, denial-rate, and
   latency shifts during this window. If no anomaly (>3σ shift) is
   detected, soak completes and the fragment advances to **Activated**.
   If an anomaly is detected, `oyatie.policy-engine.fragment-anomaly-
   revoker` automatically revokes the fragment (it returns to `Published`
   state with a `soak_anomaly_revocation` annotation) before it ever
   enters enforcement. Soak duration is configurable per-fragment scope
   (baseline: 60s minimum; elevated-risk fragments: up to 10min).
5. **Activated.** Fragment becomes effective (enforcement mode) at
   `effective_at` — only after soak completion with no anomaly.
   Activation is per-cell — each cell's policy-engine evaluator hot-
   reloads at the fragment's effective time (cell-local clock,
   tolerance ±2s per HLC budget).
6. **In-force.** Fragment evaluates incoming requests until sunset.
   Each evaluation that applies the fragment emits to audit.
7. **Sunset.** At `sunset_at`, fragment moves to inactive. Activations
   archived (read-only, regulator-retrieval-only).
8. **Tombstoned.** A separate sunset + tombstone ADR ratifies the
   removal if the fragment represented a regulatory commitment. (Most
   fragments simply expire; sunset is just inactivation.)

### D-3. Minimum-required gate set per µservice

Every µservice in the portfolio must register, at a minimum, the
following Cedar fragments covering its actions:

| Gate category | Minimum required fragments | Default-deny required? |
|---|---|---|
| **Authorization** (who can perform action X on resource Y) | One permit + one default-deny per action type | YES |
| **Tenant-scope** (cross-tenant operations) | Permit per cross-tenant action type | YES |
| **Data class** (per data-class-touching action) | Permit per (action, data-class) pair | YES |
| **Jurisdiction overlay** (per-pack-applicable actions) | Pack overlay fragment for each applicable pack | YES, per pack |
| **Compliance pack** (per active compliance pack) | Pack fragment installed when tenant adopts pack | YES |
| **Reserved namespace** (creation/registration actions) | Reserved-namespace fragment | YES |
| **Audit emission** (which events emit to which stream) | Per event class → audit stream selection fragment | YES |
| **Cost attribution** (per action → cost center) | Per-action cost-center selection fragment | YES |
| **Feature activation** (per-tenant feature gate) | Per-feature × per-tenant-tier permit | YES |
| **Rate limit + quota tier** (per principal → tier) | Per-tier permit fragment | YES |
| **Cross-cell traffic** (which cells may call which) | Per cell-pair permit | YES |
| **encryption-BYOK eligibility** (per tenant → encryption-BYOK class) | Per-data-class encryption-BYOK permit | YES |
| **DSAR cascade scope** (per Article 17 → erasure scope) | Per-tenant erasure-scope fragment | YES (legal hold supersedes) |

Any µservice declaring an action without a corresponding permit + default-
deny fails the `oya-check-cedar-coverage` lane. BLOCKER post-bootstrap.

### D-4. Per-tenant overlay composition

Cedar fragments compose per-tenant at evaluation time:

```
effective_policy(tenant T) =
    baseline_fragments
    ∪ overlay_fragments[jurisdiction(T)]
    ∪ overlay_fragments[secondary_jurisdictions(T)]  // e.g., a tenant with KR + EU operations
    ∪ ⋃ pack_fragments[pack ∈ active_packs(T)]
    ∪ tenant_fragments[T]                            // per-tenant overrides where permitted
```

Composition semantics:

- **Permits union.** Any permit in any layer permits.
- **Forbids override.** Any forbid in any layer forbids (deny wins).
- **NotApplicable means "no fragment said anything."** Default-deny
  fragment catches these; no fragment ≠ permit.

Tenant-specific fragments are *restricted*: they can `forbid` (more
restrictive than baseline) or `permit` actions that baseline permits
conditional on attributes. They CANNOT permit actions that baseline
forbids. This is enforced by a structural check in fragment review
(`oya-check-cedar-tenant-fragment-restriction`).

### D-5. Bootstrap chain of trust

The chicken-and-egg of "policy engine policies its own access" is
resolved via a four-layer trust chain anchored by the separately-rooted
meta-trust-root (per ADR-0293) to break the self-referential circular
predicate identified in F5-247-01:

1. **Bootstrap fragment** (`microservices/policy-engine/fragments/bootstrap/genesis.cedar`)
   is signed by the **org root key** held in a tier-0 HSM (offline,
   Shamir-shared **M=5, N=9 across ≥3 jurisdictions** for the meta-
   trust-root key and all trust-chain anchors per ADR-0293 §5.5 + ADR-0295;
   M=3, N=5 retained only for tenant-scoped operational keys).
2. **Meta-trust-root anchor** (`oyatie.foundry.meta-trust-root`):
   Separately-rooted trust anchor whose key is held in a dedicated
   offline HSM (5-of-9 Shamir, ≥3 jurisdictions). Every Cedar fragment
   that gates self-modification actions MUST carry an
   `attested_by_meta_trust_root: true` annotation backed by a witness
   signature from this key. This separates the "workflow publisher
   signs the artifact" trust from the "meta-trust-root sanctions the
   self-modification action" trust, preventing circular-predicate
   exploitation (per ADR-0293 §D-4). The meta-trust-root key is
   distinct from and independently held from the org root key.
3. **Bootstrap-runner intermediate** (`oyatie.foundry.bootstrap-ca`):
   One-shot offline-rooted CA that issues SPIFFE workload identity
   certificates to Stage-1 external CI runners (`oyatie.foundry.bootstrap-
   runner` principal); private key destroyed after Stage-1 Step 1.10
   completes; bootstrap trust roots killed automatically at T+8h via
   `microservices/policy-engine/fragments/bootstrap/bootstrap-trust-roots-
   kill-switch.cedar` published by `oyatie.foundry.bootstrap-kill-switch-
   publisher` (per ADR-0295).
4. **Genesis fragment** grants the bootstrap principal (a service
   identity issued by `microservices/identity/` at step 3 of the
   `oyatie`-tenant bootstrap sequence per ADR-0242 §D-5) the permission
   to publish further fragments.
5. **Intermediate keys** (org-baseline-key, per-pack-owner-keys, per-
   jurisdiction-overlay-keys) are themselves authored as cosign
   attestations of intermediate-key certificates, each signed by the
   org root key.
6. **Subsequent fragment publication** uses the intermediate keys; no
   subsequent fragment requires the root key directly; self-modification
   fragments also require meta-trust-root witness attestation.
7. **Root key rotation** is a planned ceremony (annual; reviewed by
   council-security + ops-compliance + external auditor). New root
   key signs new intermediate certificates; old root key signatures
   remain valid until their issued intermediates rotate. Meta-trust-root
   rotation follows the same ceremony schedule, independently.

### D-6. Performance: in-cell cache + sub-millisecond p99

Cedar evaluation budget: < 1 ms p99 for cached-policy hot path.

Implementation:

- Each cell has its own `policy-engine-evaluator` Deployment (3+
  replicas per cell, HPA scales to traffic).
- Hot cache: Valkey, per-cell, 1s TTL on compiled-policy bundles.
- Cold path (cache miss): Postgres + Citus query (~10ms p99); compile
  fragments to Cedar AST + cache.
- Evaluation: in-process Cedar v4.2 evaluator; per-call evaluation
  hits the cached compiled bundle.
- Per-µservice client: thin SDK (`oya-shared-policy-engine-client`)
  with circuit breaker + connection pool to the cell-local evaluator.

Performance targets (per ADR-0241 T1 + T2 tier capability):

| Path | p50 | p99 | p999 | budget_evidence |
|---|---|---|---|---|
| Hot path (cache hit) | 0.1 ms | 1 ms [P5..P95: 0.25ms–0.75ms] | 5 ms | modeled; assumptions in docs/performance-budgets/cedar-hot-path-1ms-p99.md |
| Cold path (cache miss + compile) | 10 ms | 50 ms | 100 ms | modeled; Postgres+Citus ~10ms + Cedar AST compile ~20ms; consistent with ADR-0280 §D-6 cold-path estimates |
| Audit emission (async enqueue only) | — | 1 ms (enqueue only; Merkle-seal async, ≤200ms) | — | modeled; Kafka fire-and-forget enqueue; send response does NOT block on Merkle-seal completion |
| Fragment hot-reload (Path A, EMERGENCY push) | 260 ms | 5 s [P5..P95: 1s–5s] | — | modeled; dual-path; see docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md |
| Fragment hot-reload (Path B, constant-work pull) | ~15 s | ≤35 s [P5..P95: 10s–35s] | — | modeled; 30s pull cadence + 5s recompile; see docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md |

**Hot-path budget requires:** DaemonSet co-located evaluator + same-node Valkey sidecar + Cilium
Ambient eBPF mesh (no Envoy sidecar on intra-node path). See decomposition in
`docs/performance-budgets/cedar-hot-path-1ms-p99.md`. Required SLO:
`cedar_evaluator_cache_hit_ratio ≥ 99.9%` per cell.

**Dual-path hot-reload model:** Path A (EMERGENCY push via Kafka pub-sub) achieves 5s p99.
Path B (constant-work 30s snapshot pull, per ADR-0248 §D-9) achieves ≤35s p99. Standard
fragment activations use Path B only. Emergency fragment activations (priority: EMERGENCY)
trigger Path A immediately and are also included in the next Path B snapshot. See
`docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md` for full reconciliation.

The SDK falls back to in-µservice cached decision if the cell-local
evaluator is unreachable for ≥ 200 ms (static stability per AWS Builder's
Library; cached decision TTL = 30 s; rejects all new actions if even
the cache is empty — fail-closed default).

### D-7. Audit emission on every Cedar decision

Every Cedar evaluation emits an audit row:

```json
{
  "event_class": "CedarEvaluation",
  "evaluation_id": "<uuid>",
  "principal": "oyatie.foundry.ci-agent#<instance-id>",
  "action": "WorkflowEngine::Action::TriggerBuild",
  "resource": "Workflow::id/build-3421",
  "context": {
    "data_class": "SOURCE_CODE_INTERNAL",
    "compliance_packs_active": ["soc2-t2"],
    "request_origin_cell": "data-plane-cell-us-west-2-a"
  },
  "tenant_id": "oyatie",
  "decision": "Permit",
  "applied_fragments": [
    "baseline/oyatie-foundry-ci-permits.cedar:v3",
    "overlay/us-de/oyatie-baseline-overlay.cedar:v1",
    "pack/soc2-t2/audit-emission.cedar:v2"
  ],
  "determining_policies": ["oyatie-foundry-ci-permits:permit-trigger-build"],
  "evaluation_ms": 0.4,
  "audit_emitted_at": "2026-05-20T14:32:11.123Z",
  "audit_stream": "oyatie.foundry"
}
```

This is the per-call audit row. Aggregation, retention, and rollup
follow the per-jurisdiction retention rules (per ADR-0241 + per
Compliance Pack).

### D-8. Cedar fragment authoring + multispectrum review integration

Cedar fragments are first-class artifacts in the multispectrum review
v2.4.0 pipeline:

- **F1 (correctness)** facet: structural check + intent declaration
  match.
- **F2 (hyperscaler-fitness)** facet: does the fragment match AWS Cedar
  idiom + best practices (e.g., named entity types, no anonymous
  resources, explicit `when` conditions)?
- **F5 (security)** facet: privilege escalation analysis using
  `cedar-policy-analyzer`'s formal verification tool.
- **F6 (performance)** facet: evaluation-cost analysis (does any
  policy use unbounded entity-set traversals?).
- **F7 (supply chain)** facet: signing-key chain verified.
- **A1 (own-policy-adherence-naming)** facet: fragment_id follows
  `<scope>-<microservice>-<action-family>` convention.
- **A4 (architecture-adherence)** facet: fragment respects scope/
  overlay/pack layering.
- **A6 (schema-adherence)** facet: Cedar entity types match Ontology
  Object Type definitions.

Multispectrum review verdict for a fragment determines whether it can
be signed and published.

### D-9. Coverage CI lane

The `oya-check-cedar-coverage` CI lane scans every µservice and
verifies:

1. Every declared action (in OpenAPI 3.2.0 + AsyncAPI 3.1.0 contracts;
   in `microservices/<ms>/capabilities/*.yaml`; in Cedar action enums)
   has at least one permit fragment + one default-deny.
2. Every Object Type write (per Ontology IP-002) has an Action Type
   per ADR-0028 + a corresponding Cedar fragment.
3. Every cross-µservice call (declared in `microservices/<ms>/calls.yaml`)
   has a permit fragment.
4. No code path makes a policy decision without consulting the
   policy-engine SDK (verified by `oya-check-no-policy-in-code` static
   analysis lane).

Exit code 0 = full coverage; exit code 1 = coverage gap; exit code 2
= policy-in-code violation.

### D-10. Hot-reload semantics

**Amended per ADR-0294:** The <5s hot-reload window now applies to
**soak-phase shadow evaluation only** — fragments entering the soak
window (§D-2 stage 4a) are loaded into the shadow evaluation path
within 5s p99. Full enforcement activation is gated on soak completion;
no fragment bypasses the soak window to enter enforcement directly.

Fragment updates enter the shadow evaluation path within 5 seconds
across all evaluator replicas in a cell (D-6 performance target).
Advancement to enforcement path occurs only after the soak detector
confirms no anomaly (≥60s soak window; longer for elevated-risk
fragments per ADR-0294 §D-1). Implementation:

- New fragment publication emits a `CedarFragmentPublished` event on
  the audit-chain + a notification on the per-cell `fragment-reload`
  pub-sub topic (Kafka per ADR-0050 inheritance).
- Each evaluator replica subscribes; on notification, fetches new
  fragment from registry; recompiles policy bundle; atomic swap into
  **shadow evaluation path** (not enforcement path).
- Shadow-path tail latency during swap: < 5s p99 across replicas.
  Per-evaluator recompile latency < 1s p99 (compiled AST cached in
  Valkey for warm-start).
- After soak window completion with no anomaly: evaluator performs
  a second atomic swap promoting the fragment from shadow path to
  enforcement path. The enforcement-path swap latency budget is the
  same (<5s p99) but the trigger is the soak-completion event from
  `oyatie.policy-engine.fragment-soak-detector`, not the publication
  event.
- If a swap fails (compile error in new fragment), the replica
  remains on previous policy + emits SEV-3 alert. The faulty
  fragment's publication is reversed by the publishing workflow.
- If the soak detector triggers anomaly-revocation (per ADR-0294
  §D-8), the fragment is revoked before ever entering the enforcement
  path; the shadow-path swap is rolled back; a `CedarFragmentSoakRevoked`
  audit event emits.

### D-11. Failure modes + fail-closed default

When the policy engine is unavailable (cell-local evaluator pods
unhealthy + Valkey cache empty), the SDK falls back through three
causal paths:

1. **Last-known-good cached decision** (per-action, per-tenant, last
   1024 decisions, 30s TTL). Hit → use cached decision. Miss →
   continue.
2. **Default-deny.** No fragment-derived permit available → forbid.
3. **Fragment-revoked-by-soak-anomaly** *(per ADR-0294 §D-8 — third
   causal fallback path):* When a published fragment is revoked by
   the soak-anomaly detector during its soak window, all evaluator
   replicas that loaded the fragment into their shadow evaluation path
   MUST roll back to the previous enforcement policy immediately. The
   `oyatie.policy-engine.fragment-anomaly-revoker` emits a
   `CedarFragmentSoakRevoked` event on the per-cell `fragment-reload`
   pub-sub topic; each evaluator replica treats this identically to a
   publish-failure: remove the shadow-path entry, retain current
   enforcement policy unchanged, emit SEV-3 alert. Because the
   anomaly-revoked fragment never entered the enforcement path, callers
   are unaffected — the rollback is invisible to enforcement semantics.
   The audit record of both the shadow-evaluation period and the
   revocation event is retained for forensic review.

This is fail-closed by default per AWS Builder's Library "Static
Stability" guidance.

Brown-out signal (ADR-0176) emits `degraded` for the µservice when
falling back to cache; emits `outage` when falling back to default-
deny on > 1% of requests over 30s; emits `fragment-soak-revocation`
event class when path 3 triggers (advisory only — enforcement was
never disrupted, but the anomaly warrants investigation).

### D-12. Per-tenant fragment authoring boundary

Tenants may author fragments scoped to themselves with these limits:

- Tenant can `forbid` actions that baseline permits (more restrictive).
- Tenant can `permit` actions that baseline permits *conditional on
  attributes* (e.g., baseline permits writes to any document; tenant
  permits only after a specific approval flag).
- Tenant CANNOT `permit` actions that baseline forbids.
- Tenant fragments are reviewed by the tenant's own admin console
  workflow (configured per tenant); they do not require oyatie
  council-security review.
- Tenant fragments cannot extend tenant's own permissions beyond what
  the tenant's compliance packs already permit.

This is planned to be enforced by `oya-check-cedar-tenant-fragment-restriction`
during fragment review.

### D-13. Feature flag replacement

Feature flag evaluation moves entirely into Cedar. The pattern:

```cedar
permit (
  principal,
  action == Feature::"WorkflowStudioCanvasV2",
  resource is User
)
when {
  resource.tenant_id in TenantTier::["enterprise", "pro"]
  && resource.feature_opted_in
  && (resource.tenant_compliance_packs.contains("soc2-t2") || resource.tenant_audience == "internal")
};
```

The application code:

```rust
let decision = policy_engine.evaluate(EvaluationRequest {
    principal: caller,
    action: Action::Feature("WorkflowStudioCanvasV2"),
    resource: user.into(),
    context: default_context(),
    tenant_id: caller.tenant_id,
    evaluation_id: Uuid::new_v4(),
}).await?;

if decision.decision == Decision::Permit {
    render_canvas_v2(...).await
} else {
    render_canvas_v1(...).await
}
```

LaunchDarkly / Flagsmith / open-source feature-flag SDKs are NOT
adopted. They would introduce a parallel policy system; the doctrine
explicitly rejects this.

## Alternatives considered

### Alt-1. Cedar for authorization only (status quo from ADR-0150)

Keep Cedar scoped narrowly to authorization decisions; leave other
policy-class decisions in code or in separate systems (LaunchDarkly,
configuration files, ad-hoc gates).

**Pros:**

- Zero migration cost (already in place).
- Cedar evaluation surface stays small + reviewable.
- Avoids forcing every team to learn Cedar fragment authoring.

**Cons:**

- **23 policy-class decisions drift in code.** Each becomes its own
  drift surface. New regulatory requirements trigger N µservice
  changes instead of one fragment.
- **Audit-chain gap.** Non-Cedar policy decisions miss the audit row,
  losing forensic + compliance evidence.
- **Per-tenant overlay impossible.** Code-encoded policy can't be
  per-tenant without bespoke per-tenant code branches.
- **Compliance Pack abstraction breaks.** Compliance Packs (per
  ADR-0251) include Cedar fragments. If only some policy is Cedar,
  packs only cover some compliance.
- **Multiple policy engines emerge** (one in each µservice's code).
  This is precisely the anti-pattern AWS Verified Permissions, GCP
  Org Policy, Netflix OPA, etc. were created to resolve.

**Rejected** because the cons accumulate over time + every industry
reference recommends consolidation.

### Alt-2. Multi-engine: Cedar for app authz + OPA for admission/policy + LaunchDarkly for feature flags

Use Cedar for authorization; OPA for general policy + admission;
LaunchDarkly (or similar) for feature flags. Each in its native
strength.

**Pros:**

- Each tool used for what it's strongest at.
- Familiar pattern (many SaaS adopt all three).

**Cons:**

- **Three policy stores, three review workflows, three audit
  emission shapes.** Drift between systems guaranteed.
- **Cross-engine evaluation impossible.** A decision that depends on
  both authorization (Cedar) and feature-flag state (LaunchDarkly)
  requires two evaluations + reconciliation in code.
- **Compliance Pack would need three artifacts** (Cedar fragment + OPA
  Rego + LaunchDarkly config) per pack — expensive + drift-prone.
- **Per-tenant overlay split** across three systems.
- **Vendor risk** for LaunchDarkly (third-party SaaS for policy data —
  violates [[bominal-inheritance-precedence]] in-house preference).

**Rejected** because consolidation is the consistent hyperscaler-
grade pattern + multi-engine cost compounds.

### Alt-3. Move to OPA (Rego) instead of Cedar

Replace Cedar with OPA + Rego language. OPA has broader community,
broader integration (admission via Gatekeeper; data filtering;
deployment validation).

**Pros:**

- Larger ecosystem.
- OPA Gatekeeper is established Kubernetes admission solution.
- Rego is more flexible (Turing-complete subset; though OPA enforces
  decidable subset in practice).

**Cons:**

- **Migration cost.** ADR-0150 already adopted Cedar; OPA migration
  would invalidate existing fragments + tooling.
- **Cedar's formal-verification tools (cedar-policy-analyzer) are
  ahead of OPA equivalents.** AWS-funded; integrates with cedar-policy
  Rust crate.
- **Cedar v4 LTS commitment from AWS.** OPA's CNCF-graduated status
  is strong but Cedar's specific AWS-backed LTS gives stronger
  enterprise commitment.
- **Cedar's design is decidable by construction.** Rego requires
  external analysis to prove termination.
- **The portfolio already uses Cedar (ADR-0150, ADR-0183, Ontology
  cedar-fragment-coverage BC, ADR-0140 retired fragments).** Switching
  invalidates the in-flight investment.

**Rejected** because the migration cost outweighs OPA's ecosystem
advantage and Cedar's formal properties align with the platform's
quality bar.

### Alt-4. Universal Cedar (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **Single source of truth for policy decisions.**
- **Compliance Pack abstraction works cleanly.**
- **Per-tenant overlay composes uniformly.**
- **Audit-chain coverage is comprehensive.**
- **Hyperscaler-grade pattern.** Matches AWS Verified Permissions
  (Cedar), GCP Org Policy (consolidated), Netflix + Pinterest (OPA
  but unified).
- **Cedar's formal-verification properties** enable doctrine
  enforcement at fragment-review time.
- **In-house** per ADR-0211.

**Cons:**

- **Author + reviewer learning curve.** Mitigation: Cedar v4.2 has
  excellent docs + tooling + community. The 23 policy migrations are
  bounded.
- **Performance budget tight.** Mitigation: in-cell cache + per-cell
  evaluator pool + < 1ms p99 budget proven achievable per AWS Verified
  Permissions production data.
- **Fragment proliferation risk.** Mitigation: fragment naming
  convention (BNF v4.1) + coverage CI lane + sunset lifecycle prevent
  unbounded growth.
- **Bootstrap chain-of-trust requires HSM ceremony.** Mitigation: HSM
  ceremony is annual; well-understood operational pattern (similar to
  CA root key ceremony).

**Accepted** as the foundational keystone for policy.

## Consequences

### Positive

1. **Compliance Pack first-class.** ADR-0251 Compliance Pack
   abstraction works because Cedar fragments are the unit of policy.
   Packs ship as fragment bundles.
2. **Audit-chain coverage comprehensive.** Every policy decision
   emits to audit. Forensic + regulatory + DSAR queries hit one
   audit corpus.
3. **Per-tenant overlay clean.** Tenants compose policy by activating
   compliance packs + authoring restricted tenant fragments.
4. **Per-jurisdiction overlay clean.** Pack overlays per ADR-0240
   layer naturally with baseline.
5. **Drift detection via coverage CI lane.** New action without
   fragment = CI fail.
6. **Hot-reload enables policy evolution without code deploys.**
   Critical for incident response (e.g., emergency narrow-permit during
   security investigation).
7. **Hyperscaler-shape.** Matches AWS Verified Permissions, GCP Org
   Policy, OPA-at-scale patterns.

### Negative

1. **Performance budget tight.** <1ms p99 evaluation budget consumed by
   network hop + compile + Cedar evaluator overhead. Mitigation:
   per-cell evaluator + Valkey cache; SDK static-stability fallback.
2. **Authoring complexity.** Cedar requires learning Cedar grammar +
   entity-type modelling. Mitigation: dedicated `oyatie.foundry.adr-drafter`
   workflow that drafts initial fragments from action declarations.
3. **Fragment proliferation.** Many fragments. Mitigation: convention +
   coverage lane + sunset lifecycle.
4. **Bootstrap HSM ceremony.** Annual operational overhead.
   Mitigation: well-understood pattern; ops-compliance runbook.

### Operational

1. **New CI lanes:**
   - `oya-check-cedar-coverage` (advisory until bootstrap; BLOCKER post-bootstrap)
   - `oya-check-no-policy-in-code` (static-analysis lane; identifies
     imperative policy decisions)
   - `oya-check-cedar-fragment-signature` (verifies signed-by-correct-
     key chain)
   - `oya-check-cedar-default-deny-coverage` (every permit has a
     corresponding default-deny)
   - `oya-check-cedar-tenant-fragment-restriction` (tenant fragments
     can't permit baseline-forbidden actions)
2. **New µservice surfaces:**
   - `microservices/policy-engine/` (per ADR-0246) — promoted from
     Ontology BC.
   - `oya-shared-policy-engine-client` (Rust + TS SDKs).
3. **Observability:**
   - Per-cell policy-engine evaluator pod metrics (eval p99, cache hit
     rate, fragment load count, audit emission lag).
   - Per-fragment evaluation count + decision distribution.
4. **Tooling:**
   - `cedar-policy-analyzer` integration in CI for formal verification.
   - `cedar-policy-cli` for local fragment authoring + testing.
   - Fragment registry browser in tenancy-admin-console.
5. **HSM ceremony:** annual root-key rotation; ops-compliance owns
   runbook.

### Sustainability

- Per-cell evaluator pods consume ~0.1% of cell compute. Carbon impact
  minimal vs the prior multi-system policy overhead.

### Compliance

- **GDPR Article 22 (automated individual decision-making).** Cedar
  decisions on consumer-facing actions are now individually auditable;
  per-decision rationale emitted.
- **EU AI Act Article 14 (transparency).** AI-mediated decisions
  (LLM tool-call permits, autonomy_tier escalations) emit applied-
  fragments list for human review.
- **HIPAA Security Rule §164.312 (access control).** Every PHI-touching
  action authorized via Cedar permit + audit emission.
- **SOC 2 CC6.1 (logical access).** All authorization is Cedar-mediated.
- **ISO 27001 A.9 (access control).** Cedar provides the control.
- **KR-PIPA Article 22 (consent).** Consent records influence Cedar
  evaluation via context attributes.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/cedar-fragment-schema.json` | NEW — canonical fragment frontmatter schema |
| `/specs/policy-gate-coverage.json` | NEW — per-µservice gate enumeration schema |
| `/specs/microservices/policy-engine.json` | NEW — per ADR-0246 |
| `microservices/policy-engine/` µservice | NEW — per ADR-0246 promotion |
| `microservices/policy-engine/fragments/baseline/*.cedar` | NEW — baseline fragment set per gate category |
| `microservices/policy-engine/fragments/bootstrap/genesis.cedar` | NEW — genesis fragment signed by org root |
| `microservices/policy-engine/src/api.rs` | NEW — evaluation API |
| `microservices/policy-engine/src/evaluator.rs` | NEW — in-process Cedar v4.2 evaluator |
| `microservices/policy-engine/src/hot_reload.rs` | NEW — fragment hot-reload subscriber |
| `microservices/policy-engine/src/signing_chain.rs` | NEW — signature verification |
| `crates/oya-shared-policy-engine-client/` | NEW — per-µservice SDK with circuit breaker + cache fallback |
| `microservices/policy-engine/iac/helm/policy-engine-evaluator/` | NEW — per-cell evaluator Deployment Helm chart |
| `tools/oya-check-cedar-coverage/` | NEW |
| `tools/oya-check-no-policy-in-code/` | NEW |
| `tools/oya-check-cedar-fragment-signature/` | NEW |
| `tools/oya-check-cedar-default-deny-coverage/` | NEW |
| `tools/oya-check-cedar-tenant-fragment-restriction/` | NEW |
| `docs/standards/cedar-fragment-authoring.md` | NEW — full standards doc with examples |
| `docs/runbooks/cedar-hsm-root-key-ceremony.md` | NEW — annual ceremony procedure |
| `docs/runbooks/cedar-fragment-incident-response.md` | NEW — emergency-permit + emergency-forbid procedures |
| `docs/runbooks/cedar-evaluation-failure-mode.md` | NEW — fail-closed cache fallback procedure |
| Migration sweep: removal of LaunchDarkly / Flagsmith / similar SDK calls (currently ~0; preemptive) | SWEEP |
| Migration sweep: removal of in-code policy decisions from ~23 sites | SWEEP — per the §Context inventory |

## 2026-06-25 Tenant Quota Adapter Evidence Addendum

Wave A cloud-iac/cloud-k8s verified that
`k8s/adapters/tenant-quota-adapter-cedar/src/lib.rs` enforces same-tenant
quota reads and writes for tenant roles and reserves cross-tenant quota read
authority to the platform quota scope. The accounting registration evidence is
`evidence/multispectrum/waveA-iac-k8s-tenant-quota-rbac-20260625-1782430279.json`;
it closes the ADR-0243 quota-tiering policy decision gap for
tenant/project/account isolation without adding generated faces, new
dependencies, provider adapters, workflows, or root-hub spec churn.

## Verification

- [ ] `microservices/policy-engine/` is promoted to peer substrate µservice (ADR-0246 completed).
- [ ] `/specs/cedar-fragment-schema.json` exists + validates real fragments.
- [ ] `microservices/policy-engine/fragments/bootstrap/genesis.cedar` exists + signed by org root key.
- [ ] `oya gate validate cedar-coverage` reports ≥ 95% coverage across all µservices' declared actions (bootstrap target; goal 100% by post-keystone +90 days).
- [ ] `oya gate validate no-policy-in-code` reports zero in-code policy decisions in pilot µservice (e.g., `microservices/tenancy/`); expand to all µservices in subsequent sweeps.
- [ ] `oya gate validate cedar-fragment-signature` succeeds for all published fragments.
- [ ] `oya gate validate cedar-default-deny-coverage` reports every permit has a corresponding default-deny.
- [ ] `oya gate validate cedar-tenant-fragment-restriction` succeeds for all tenant-scoped fragments.
- [ ] Hot-reload p99 < 5s measured under load.
- [ ] Hot-path evaluation p99 < 1ms measured at 10k QPS per cell.
- [ ] Audit-chain emits `CedarEvaluation` for every decision; sampled regulatory query returns all decisions for a given tenant within 30 days.
- [ ] HSM ceremony runbook drilled at least once before BLOCKER promotion.
- [ ] cedar-policy-analyzer integration in CI passes for baseline fragment set.

## References

### Industry sources

- **AWS Verified Permissions** (GA 2024-Q1). The reference implementation of Cedar at hyperscale. Docs: `docs.aws.amazon.com/verifiedpermissions`.
- **Cedar v4.2 LTS** (2025). Documentation + grammar + reasoning tools. `cedarpolicy.com`.
- **AWS Cedar SDK Rust crate** (`cedar-policy` on crates.io v4.x).
- **cedar-policy-analyzer** (AWS-funded, open-source 2024). Formal verification of Cedar policies.
- **AWS Builder's Library — "Avoiding insurmountable queue backlogs"** (Marc Brooker). Per-call gate doctrine.
- **AWS Builder's Library — "Static stability using Availability Zones"** (Becky Weiss + Mike Furr). Fail-closed default + cache fallback pattern.
- **Open Policy Agent (OPA)** documentation. Comparative reference; not adopted.
- **Google Cloud Org Policy + IAM Conditions** documentation. Consolidated-policy pattern.
- **Microsoft Azure Policy** documentation. Comparative reference.
- **Netflix Tech Blog — "OPA in production at Netflix" (2020-2023 series).** Multi-engine consolidation lessons.
- **CNCF Open Policy Agent project** (graduated 2021). Maturity reference.
- **Pinterest Engineering Blog — "Authorization at Pinterest"** (2022). OPA-based consolidation.

### Regulatory sources

- **GDPR Article 22 — Automated individual decision-making.** Individual-decision auditability requirement.
- **EU AI Act 2024/1689 Article 14 — Transparency obligations.** Applies to AI-mediated decisions.
- **HIPAA Security Rule §164.312 — Access control.** Authorization audit trail.
- **SOC 2 Type II — CC6.1 (Logical Access Controls).** Access policy enforcement evidence.
- **ISO 27001 Annex A.9 — Access Control.** Standard access-control evidence.
- **KR-PIPA Article 22 — Consent.** Consent state as policy input.
- **NIST SP 800-162 — Attribute Based Access Control (ABAC).** Cedar is ABAC.
- **NIST SP 800-207 — Zero Trust Architecture.** Per-call policy evaluation principle.

### Cedar / formal-policy academic + practitioner sources

- **Cedar specification (1.0 published 2023; v4 published 2025).** Language reference.
- **"Cedar: A New Language for Authorization" (AWS, OOPSLA 2024).** Academic paper introducing Cedar's design.
- **"Formal verification of authorization policies"** (various authors, 2024-2025).
- **Mark Stamp, *Information Security: Principles and Practice* (3rd ed.).** Foundational ABAC + authorization theory.

### Internal portfolio ADRs

- **ADR-0028 — Cloud microservice architecture.** Cedar lives in policy-engine µservice.
- **ADR-0099 — Data class registry.** Data classes are Cedar entity attributes.
- **ADR-0105 — Thirteen-layer canonical enum.** Cedar evaluator is a kernel-layer concern.
- **ADR-0128 — Hyperscaler architecture invariants.** Policy-as-code is an invariant.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Tier evaluation is Cedar-mediated.
- **ADR-0145 — Inter-microservice communication reform.** Direct gRPC + Cedar evaluation at caller.
- **ADR-0150 — Cedar policy engine.** Original scope; this ADR extends.
- **ADR-0176 — Brown-out + degradation signal.** Cedar evaluator availability signal.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Separation preserved.
- **ADR-0240 — Sovereign cloud per regional pack.** Per-pack Cedar overlays.
- **ADR-0241 — DR + BC portfolio.** Policy-engine is T1.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** `oyatie.*` principal fragments live in policy-engine.
- **ADR-0244 — Tenant as universal scoping primitive (companion).**
- **ADR-0246 — Policy-engine substrate promotion (companion).**
- **ADR-0247 — Self-hosting / self-modification (companion). Cedar fragments evolve via Foundry workflows under Cedar gates.**
- **ADR-0251 — Compliance Pack + Cell Certification Levels (companion).** Packs are Cedar fragment bundles.

### Auto-memory feedback

- `feedback_cedar_as_universal_gate` — NEW.
- `feedback_oyatie_is_a_tenant_doctrine` — applies; oyatie principals follow Cedar gates.
- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade.
- `feedback_no_silent_regression` — reinforced; Cedar fragment versioning + sunset prevents silent policy change.
- `feedback_automate_everything` — reinforced; policy in fragments rather than code enables automation.

---

## Appendix A: Hyperscaler-pattern attribution matrix

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Cedar evaluates every policy-class decision) | "Single Policy Engine Consolidation" | AWS Verified Permissions design; GCP Org Policy consolidation; Netflix OPA-at-scale | "Multiple Policy Engines Drift" — each subsystem hand-rolls policy |
| D-2 (fragment lifecycle) | "Signed Policy Authoring Lifecycle" | AWS Verified Permissions policy store; Sigstore + cosign attestations | "Imperative Policy Patching" — ad-hoc policy changes without provenance |
| D-3 (minimum gate set per µservice) | "Coverage-Required Authorization" | NIST SP 800-162 ABAC; AWS Well-Architected SEC | "Implicit Permit" — actions without explicit permit policy |
| D-4 (per-tenant overlay composition) | "Layered Policy Composition" | AWS SCP + IAM intersection; Cedar fragment union | "Per-Tenant Code Branch" — per-tenant logic embedded in shared code |
| D-5 (bootstrap chain of trust) | "PKI Root + Intermediate Certificate Chain" | RFC 5280 X.509; Sigstore Rekor; AWS KMS key hierarchy | "Implicit Bootstrap Trust" — undocumented signing key emergence |
| D-6 (in-cell cache + sub-millisecond p99) | "Edge-Cached Policy Evaluation" | AWS Verified Permissions production cache; Cloudflare Workers KV | "Synchronous Round-Trip to Global Policy Store" — cross-region policy fetch on hot path |
| D-7 (audit emission on every decision) | "Audit-Every-Decision" | NIST SP 800-92 audit log standards; SOC 2 CC7.2 | "Audit Sampling" — only some decisions audited |
| D-8 (multispectrum review integration) | "Multi-Facet Policy Review" | oyatie multispectrum review v2.4.0 doctrine | "Single-Reviewer Policy Change" — drift via insufficient review |
| D-9 (coverage CI lane) | "Coverage-Enforced Policy" | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | "Untested Policy Surface" — gates discovered missing in production |
| D-10 (hot-reload semantics) | "Hot-Reload Configuration Distribution" | etcd watch pattern; Kubernetes ConfigMap watch; Apollo / Argo CD sync | "Restart-To-Apply" — policy changes require service restart |
| D-11 (fail-closed default) | "Static Stability + Fail-Closed" | AWS Builder's Library "Static stability"; NIST SP 800-207 deny-by-default | "Fail-Open on Policy Unavailable" — security holes during outage |
| D-12 (per-tenant fragment authoring boundary) | "Restricted Tenant Self-Policy" | AWS SCP + IAM permission boundary pattern | "Tenant Privilege Escalation" — tenant raises own permissions above baseline |
| D-13 (feature flag replacement) | "Unified Policy + Feature Gate" | AWS Verified Permissions docs explicitly cover feature gates | "Separate Feature-Flag System" — LaunchDarkly + similar |

---

## Appendix B: Worked example — emergency narrow-permit during security incident

**Scenario:** A security incident is detected: a malicious dependency
in a third-party library is exfiltrating data via the messenger
µservice. Incident response wants to narrow permits on `messenger`
emergent-write actions to oyatie internal principals only, blocking all
tenant access, until the dependency is removed.

**Without Cedar-as-universal-gate:** the team would need to identify
every code path that gates messenger writes, push a code change
through CI (hours; risks introducing new bugs under pressure), wait
for canary deploy, then watch for fallout. Or — worst case — they
do it via emergency kill-switch in code that bypasses the standard CI
gates (anti-pattern: bypass path acquired during incident, retained
afterward).

**With Cedar-as-universal-gate:**

1. **2026-XX-XX 14:32 UTC** — Incident commander
   `oyatie.security.incident-response.<oncall>` invokes emergency
   workflow.
2. **14:32** — Workflow drafts emergency fragment:
   ```cedar
   // emergency-fragments/incident-2026-xx-xx-messenger-narrow.cedar
   // SUNSET: 2026-XX-XX 18:00 UTC (4 hours from publication)
   // SIGNED BY: incident-response-emergency-key (HSM, 2-of-3 quorum)
   forbid (
     principal,
     action == Messenger::Action::Write,
     resource is MessengerMessage
   )
   when {
     !(principal in Tenant::"oyatie".sub_scopes("security", "platform-ops", "engineering-lead"))
   };
   ```
3. **14:33** — Fragment review (abbreviated emergency-mode
   multispectrum review v2.4.0 — F5 security + A4 architecture facets
   only, fan-out to 2 reviewers + 1 incident commander).
4. **14:34** — Fragment signed by incident-response-emergency-key
   (per the HSM-quorum runbook). Signing key has its own audit-emit
   that triggers a SEV-3 alert to council-security (so emergency-key
   use is itself observable).
5. **14:34** — Fragment published. Hot-reload propagates to all cell-
   local evaluator replicas within 5s.
6. **14:35** — New messenger Write actions evaluate against the
   updated fragment set. Non-`oyatie.security/platform-ops/engineering-
   lead` principals receive `Forbid`. Audit emits the deny.
7. **14:36** — Incident response observes the deny rate on a dashboard.
   Tenant complaints triaged (some legitimate tenant traffic blocked;
   per emergency protocol, this is expected and the tradeoff is
   accepted).
8. **18:00** — Fragment auto-sunsets per its `effective_at + sunset_at`
   declaration. Permits return to baseline.
9. **18:30** — Incident post-mortem reviews the emergency-fragment use;
   ratifies (or revokes) the emergency authorization; emits a
   `EmergencyFragmentUseReview` audit row.

**Why this matters:** the emergency-permit pattern is well-defined,
reversible, auditable, and cannot become a permanent bypass. The
multispectrum review + HSM-quorum signing + sunset enforcement ensure
emergency policy use doesn't degrade the overall policy hygiene.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oya-check-cedar-coverage` | N/A (check-family) | `check`.`cedar-coverage` | CI fitness-check per ADR-0105 Amendment 2 `oya-check-*` flat namespace; verifies every µservice action has a Cedar permit + default-deny fragment. |
| `oya-check-no-policy-in-code` | N/A (check-family) | `check`.`no-policy-in-code` | CI fitness-check; static-analysis lane refusing imperative policy decisions not routed through the Cedar evaluator per §D-1. |
| `oya-check-cedar-fragment-signature` | N/A (check-family) | `check`.`cedar-fragment-signature` | CI fitness-check; verifies every active Cedar fragment has a valid Ed25519 + cosign signature per §D signing-chain. |
| `oya-check-cedar-default-deny-coverage` | N/A (check-family) | `check`.`cedar-default-deny-coverage` | CI fitness-check; verifies every permit fragment has a corresponding default-deny fragment. |
| `oya-check-cedar-tenant-fragment-restriction` | N/A (check-family) | `check`.`cedar-tenant-fragment-restriction` | CI fitness-check; verifies tenant-authored fragments cannot raise permits above baseline per Cedar restriction semantics. |
| `oya-shared-policy-engine-client` | `sdk` | `shared`.`policy-engine-client`.`sdk` | Cross-µservice SDK crate; slot2=`shared` per ADR-0056 §"Microservice registry"; BC=`policy-engine-client`; high-level Rust SDK wrapping gRPC client to cell-local evaluator. Also cited in ADR-0246 §F.0 registry diff. |
| `microservices/policy-engine/fragments/baseline/*.cedar` | N/A (file paths) | N/A | Baseline Cedar fragment set; `baseline/` subdirectory under `fragments/` separates platform-baseline permits from pack/tenant overlays; `.cedar` extension per Cedar v4.2. |

---

## Change log

- **2026-05-20 (Wave-3-A cross-reference wiring):** Applied four surgical amendments per ADR-0293, ADR-0294, ADR-0295:
  - §D-2 fragment lifecycle: Inserted `Soaking` stage (4a) between `Published` (4) and `Activated` (5) per ADR-0294 §D-1. Soak window ≥60s mandatory; shadow evaluation only during soak; anomaly detector monitors permit/denial/latency shifts >3σ; soak anomaly triggers automatic pre-enforcement revocation.
  - §D-5 chain-of-trust: Expanded to four-layer trust chain; added separately-rooted `oyatie.foundry.meta-trust-root` anchor (ADR-0293); added `oyatie.foundry.bootstrap-ca` bootstrap-runner intermediate + T+8h kill-switch fragment (ADR-0295); Shamir threshold expanded from M=3,N=5 to M=5,N=9 across ≥3 jurisdictions for all trust-chain anchors.
  - §D-10 hot-reload: Scoped the <5s window to soak-phase shadow evaluation only; full enforcement requires soak completion with no anomaly per ADR-0294.
  - §D-11 failure modes: Added `fragment-revoked-by-soak-anomaly` as the third causal fallback path per ADR-0294 §D-8.


## 2026-06-26 PDP Same-Tenant Policy Evidence Addendum

Wave B IAM/PDP verified that the structured Cedar/PDP DSL can encode
request-time same-tenant resource access with
`principal.tenant_id == resource.tenant_id`, and that tenant-quota default
policies enforce same-tenant read/write through the PDP policy path while the
#872 adapter guard remains defense-in-depth. The accounting registration
evidence is
`evidence/multispectrum/waveB-pdp-same-tenant-879-20260626-1782441461.json`,
with supplemental public API/gRPC projection evidence in
`evidence/multispectrum/waveB-pdp-same-tenant-api-879-20260626-1782443622.json`; together they close the ADR-0243 same-tenant policy-layer
follow-up for issue #879 without adding generated faces, new dependencies,
provider adapters, workflows, or root-hub spec churn.

*End of ADR-0243.*
