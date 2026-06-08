---
id: ADR-0136
status: Superseded
deciders: council-architecture, council-product, council-privacy, axis-foundry, axis-foundry-runtime, axis-foundry-supervisor, axis-foundry-eval, axis-foundry-evidence, axis-foundry-guardrails, axis-foundry-providers, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0389]
supersession_note: "Foundry-as-µservice declared dead context (D-FOUNDRY-CLARIFY); superseded by ADR-0389 cloud-intelligence framework (six-BC reasoning salvaged). D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-11."
related: [ADR-0022, ADR-0024, ADR-0025, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0137, ADR-0138]
related_memory: [feedback_no_silent_regression, feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145), feedback_bominal_inheritance_precedence, feedback_quality_performance_scalability_bar, feedback_flat_product_catalog]
related_specs:
  - /specs/microservices/foundry.json
  - /specs/per-microservice-flat-layout.json
  - /specs/hyperscaler-gates.json
session_context:
  authored: 2026-05-18
  prior_topology: |
    Prior to this ADR, the foundry hosted-agent platform was modelled as six
    independent µservices under `microservices/foundry-{runtime,supervisor,
    eval,evidence,guardrails,providers}/`. Total: 493 artefacts.
purpose: |
  Establish that foundry is one µservice with six internal bounded contexts,
  matching the hyperscaler shape (AWS Bedrock, Google Vertex AI Agent Builder,
  Microsoft Azure AI Foundry, Anthropic Console, Palantir AIP, LangSmith +
  LangGraph) — each of which ships one product with many internal BCs. The
  prior 6-way split contradicted this shape and forced cross-µservice
  deployment dances on every change to the platform's operationally
  inseparable hot path.
---

# ADR-0136: Foundry as a single µservice (with internal bounded contexts)

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

Prior state: `microservices/foundry-runtime/`, `microservices/foundry-supervisor/`,
`microservices/foundry-eval/`, `microservices/foundry-evidence/`,
`microservices/foundry-guardrails/`, `microservices/foundry-providers/` —
six independently scaffolded µservices, totalling 493 artefacts (98 + 104
+ 74 + 71 + 71 + 75).

Each µservice carried its own complete substrate (PRD, PHASE-01, threat
model, DPIA, compliance, cost budget, multi-region plan, incident response,
capacity model, failure modes, SDK plan, competitor parity, backfill/replay)
+ its own contracts (openapi + asyncapi + proto) + its own IaC (Helm +
Kustomize + Terraform + Cedar + Postgres migrations) + its own runbooks +
its own dashboards + its own SLOs + its own per-tenant Cedar policies +
its own catalog of crate records + its own 15 implementation plans.

Three observations make this 6-way split a topology error:

1. **Hyperscaler shape disagreement.** Every reference platform we
   benchmark against ships as ONE product surface with internal BCs:

   - **AWS Bedrock**: one product surface. Internal BCs: Agent runtime,
     Knowledge bases, Guardrails, Model catalog, Studio. Tenant calls
     `bedrock-runtime` API; guardrails and knowledge bases are invoked
     internally through the same product perimeter. Source: AWS Bedrock
     architecture documentation, `docs.aws.amazon.com/bedrock`.
   - **Google Cloud Vertex AI Agent Builder**: one product. Internal BCs:
     Agents, Tools, Safety filters, Evals, Deploy. Source:
     `cloud.google.com/vertex-ai/docs/generative-ai/agents/overview`.
   - **Microsoft Azure AI Foundry**: one product. Internal BCs: Agents,
     Model catalog, Safety filters, Evaluation, Deployment, Supervision.
     Source: `learn.microsoft.com/azure/ai-foundry`.
   - **Anthropic Console / Claude API**: one product. Internal BCs:
     Workbench, System prompts, Tools, Evaluations, Usage analytics.
     Source: `docs.anthropic.com/claude/docs`.
   - **Palantir AIP**: one product. Internal BCs: AIP Logic, AIP Threads,
     AIP Evals, AIP Operator, AIP Tools. Source: Palantir AIP product
     documentation.
   - **LangChain LangSmith + LangGraph**: increasingly bundled. Internal
     BCs: LangGraph (runtime), LangSmith (eval + trace), LangServe (deploy).
     Sources: `python.langchain.com`, `smith.langchain.com`.

   The 6-way split contradicts this canonical shape. Every benchmark ships
   ONE product perimeter; oyatie's prior split forced consumers and
   operators to reason about six perimeters for one capability.

2. **Operational coupling reality.** A single capability invocation
   traverses ALL SIX of the prior split's µservices in sequence:
   supervisor (admit) → runtime (dispatch) → guardrails (pre-check) →
   providers (LLM call) → guardrails (post-check) → evidence (seal) →
   runtime (return). The split treats these as independent µservices,
   but they share a single failure boundary — a SEV-1 in any one breaks
   invocation for the others. Per ADR-0131's per-microservice-flat-layout,
   a µservice is the unit of independent deployment + independent SLO
   ownership; the 6-way split's BCs do not meet that bar — they cannot
   independently SLO because their SLOs are joined by the single
   invocation transaction.

3. **Working Backwards "one product, one team" model.** Per Amazon's
   Working Backwards framework + Spotify's "two-pizza team" tradition +
   Google's SRE single-product-owner model: a product is a coherent
   surface owned by one team. Oyatie's `axis-foundry-runtime` /
   `axis-foundry-supervisor` / `axis-foundry-eval` / `axis-foundry-evidence`
   / `axis-foundry-guardrails` / `axis-foundry-providers` were six teams
   for one product — they need to coordinate every release and every
   incident anyway. Consolidating to `axis-foundry` with per-BC sub-axes
   aligns ownership with the actual coordination surface.

Per `feedback_quality_performance_scalability_bar.md`: the bar is
"industry leaders — Stripe / Palantir / Linear" + "hyperscaler-grade".
Every cited industry leader and every named hyperscaler ships their
hosted-agent platform as ONE product. The 6-way split fails the bar by
construction.

Per `feedback_flat_product_catalog.md`: "Everything is shared; flat
product catalog." A product is one row in the catalog with its full
substrate, not six rows that fan-out to one capability.

## Decision

**Foundry is one µservice with six internal bounded contexts.**

- The single µservice directory is `microservices/foundry/`.
- The six bounded contexts inside it are: `runtime`, `supervisor`, `eval`,
  `evidence`, `guardrails`, `providers`.
- Each BC retains its crate fan-out under BNF v4.1 form
  `oya-foundry-<bc>-<feature>-<layer>` (ADR-0056). Crate names DO NOT
  change across this consolidation — only the parent directory changes
  from `microservices/foundry-<bc>/` to `microservices/foundry/`.
- The per-BC content is preserved verbatim at
  `microservices/foundry/bc-sources/<bc>/` (PRD, PHASE-01, threat-model,
  DPIA, compliance, cost-budget, multi-region, incident-response,
  capacity-model, failure-modes, sdk-plan, competitor-parity-matrix,
  backfill-replay, README — all preserved per BC).
- The canonical top-level documents at `microservices/foundry/` (PRD,
  PHASE-01-FOUNDRY-FOUNDATION, threat-model, dpia, compliance,
  cost-budget, multi-region, incident-response, capacity-model,
  failure-modes, sdk-plan, competitor-parity-matrix, backfill-replay)
  are the product-of-record surface. Each enumerates per-BC contributions
  and points to the bc-sources archive for full per-BC detail.
- The 90 implementation plans (IP-001 through IP-090) are consolidated
  with sequential numbering across BCs and `<bc>` tag in filename:
  `microservices/foundry/IP-NNN-<bc>-<title>.md`. Distribution: runtime
  IP-001..015, supervisor IP-016..030, eval IP-031..045, evidence
  IP-046..060, guardrails IP-061..075, providers IP-076..090.
- The 135 catalog records live flat under `microservices/foundry/catalog/`
  without BC sub-folders (the records already carry the BC in their
  filename: `oya-foundry-<bc>-...-yaml`).
- The 36 runbooks, 18 dashboards, 18 capabilities, 4 SLOs, 18 contracts
  (6 each of openapi/asyncapi/proto), 41 policy files (Cedar + markdown),
  and 53 IaC artefacts live under their canonical subdirs with BC-prefix
  in filename, or under `iac/helm/<bc>/<chart>/` for Helm sub-charts.
- IaC structure: per-BC Helm subchart under `iac/helm/<bc>/<component>/`
  with each BC keeping its own Chart.yaml + values.yaml; per-BC kustomize
  base + overlays at `iac/kustomize/{base,overlays/pack-<pack>}/<bc>/`.
  Per-BC Terraform / Cedar / Postgres-migrations are BC-prefixed in
  filename.
- Spec consolidation: `specs/microservices/foundry.json` is the single
  spec-of-record with sections per BC.

## Alternatives Considered

### (a) Keep the 6-way split as separate µservices

- **Pros**:
  - Zero migration cost (the six dirs already exist).
  - Per-BC independent deployment cadence in theory.
  - Per-BC SLO ownership theoretical.
- **Cons**:
  - **Contradicts hyperscaler shape outright.** AWS Bedrock, Vertex AI,
    Azure Foundry, Anthropic Console, Palantir AIP, LangSmith all ship
    ONE product surface. The split fails `feedback_quality_performance_scalability_bar.md`.
  - **Operational coupling is real.** Every invocation traverses all six
    BCs; the "independent deployment" is fiction — every release coordinates
    across six dirs anyway.
  - **Cross-µservice latency added without isolation benefit.** mTLS round
    trips between six µservices add ~5–15ms cumulative to the
    50ms-budget invocation hot path with no isolation gained (a single
    misbehaving BC still breaks invocation for the others).
  - **Six-fold artefact duplication.** 6 × {PRD, threat-model, DPIA,
    compliance, cost-budget, multi-region, incident-response, capacity-
    model, failure-modes, sdk-plan, competitor-parity, backfill-replay}
    = 72 cross-cutting docs that need to stay coherent — they will drift.
  - **Six-fold ADR-0131 substrate audit surface.** Every governance lane
    (authority-cohesion, per-microservice-layout, hyperscaler-claim,
    SLO-gate) runs 6x for one product.
- **Rejected** because the cons are unbounded coordination cost and the
  topology contradicts every named industry reference.

### (b) Two-way split: foundry + foundry-providers

- **Pros**:
  - Argument: providers are external integrations and could plausibly
    isolate from the rest.
  - Provider credential isolation in a separate µservice would have a
    clean blast-radius story.
- **Cons**:
  - **Providers are tightly coupled to runtime.** Every dispatch makes a
    provider call; the round-trip is in the 50ms hot-path budget.
    Splitting adds network latency without isolation benefit because the
    providers BC is on the critical invocation path, not adjacent to it.
  - **Credential isolation is achieved by BC + OpenBao, not by µservice
    boundary.** Per the threat model + per ADR-0136 §"Decision": the
    OpenBao-bound credential never leaves the providers BC's adapter,
    whether the BC is in-µservice or cross-µservice. The cross-µservice
    boundary adds no security guarantee that the BC-boundary doesn't
    already provide.
  - **Doesn't match any benchmark.** No hyperscaler splits provider-
    routing from the agent runtime as a separate product surface.
    Bedrock includes the model catalog; Vertex includes the model garden;
    Azure Foundry includes models; Anthropic's API IS the provider.
- **Rejected** because the cons (latency + non-hyperscaler-shape) outweigh
  the cons of in-µservice isolation, and the credential isolation goal is
  fully met by the BC + OpenBao adapter pattern.

### (c) One µservice with six internal BCs  ← **CHOSEN**

- **Pros**:
  - **Matches every hyperscaler shape** named in §Context.
  - **Working Backwards alignment**: one product, one (sub-)teamed axis,
    one SLO.
  - **Single deployment perimeter** for the operationally inseparable
    invocation hot path.
  - **Single audit surface** for ADR-0131 + ADR-0123 + ADR-0139 governance
    lanes (one µservice = one run, not six).
  - **Per-BC ownership preserved** through `bc-sources/<bc>/` archive +
    BC-tagged crate names + BC-prefixed artefacts + per-BC Helm subcharts.
    BC owners (axis-foundry-runtime / -supervisor / -eval / -evidence /
    -guardrails / -providers as sub-axes under axis-foundry) continue to
    own their BC's contract surface.
  - **No cross-µservice network latency added** to the 50ms hot-path
    budget for cross-BC traffic; in-µservice cross-BC traffic is local
    plus-or-minus-one-hop within the foundry deployment.
  - **Audit-grade content preservation**: every artefact from the prior
    6-way split is preserved (493 → 493 with zero loss; see ADR-0138).
- **Cons**:
  - One-time migration cost (this ChangeSet + ADR-0136/0137/0138 +
    deletion of six source dirs).
  - 90-IP top-level numbering forces external IP references to update.
  - Per-BC contributor mental model temporarily complicates: they now
    look in `microservices/foundry/` instead of `microservices/foundry-<bc>/`.
- **Accepted** because the cons are bounded one-time costs, while the
  alternatives' cons are unbounded coordination + shape disagreement.

### (d) One monolithic µservice without internal BC structure

- **Pros**:
  - Simplest possible layout: one µservice, one PRD, no BC concept.
- **Cons**:
  - **Loses per-BC ownership clarity.** Without internal BC structure,
    no clear sub-team ownership of the six concerns (runtime, supervisor,
    eval, evidence, guardrails, providers); they all collapse into one
    axis-foundry team trying to coordinate six concerns.
  - **Loses per-BC contract surface clarity.** Tenants asking "what's
    the eval API?" or "what's the supervisor API?" lose the BC namespace
    that makes the API discoverable.
  - **Contradicts hyperscaler shape from inside.** Even AWS Bedrock,
    Vertex AI, Azure Foundry distinguish their internal BCs in
    documentation, in pricing pages, in API surfaces. Internal BC
    structure is a feature, not bureaucracy.
  - **DDD-compliant BCs are an architecture good.** Per Eric Evans'
    Domain-Driven Design and Vaughn Vernon's Implementing DDD: BCs are
    the unit of model coherence + ubiquitous-language scope; flattening
    them away destroys those properties.
- **Rejected** because the cons are unbounded loss of ownership +
  ubiquitous-language clarity.

## Consequences

### Positive

1. **Hyperscaler-shape alignment.** Foundry now matches the canonical
   product perimeter of AWS Bedrock, Google Vertex AI, Microsoft Azure
   AI Foundry, Anthropic Console, Palantir AIP, and LangSmith — six
   independent industry references all confirming the choice.

2. **Single audit + governance surface.** Per-µservice governance lanes
   (ADR-0123 hyperscaler-maturity-claim, ADR-0139 SLO-gated promotion,
   ADR-0131 per-microservice-flat-layout, authority-cohesion) run once
   for foundry instead of six times.

3. **Single SLO promotion gate.** Foundry as one product has one
   `HG-FOUNDRY` hyperscaler gate, one SLO promotion runway, one OpenSLO
   manifest set (per-BC subsections under `microservices/foundry/slos/`).

4. **Operational coupling acknowledged honestly.** The invocation hot
   path traverses all six BCs; treating them as one µservice acknowledges
   this rather than pretending six independent µservices.

5. **Audit-grade content preservation.** All 493 artefacts from the
   prior 6-way split preserved under the consolidated tree (rule: zero
   content loss). Per-BC PRDs / PHASE-01s / threat-models / etc.
   preserved under `bc-sources/<bc>/` as authoritative per-BC chapters.

6. **Per-BC ownership preserved without per-µservice fragmentation.**
   BC sub-axes (axis-foundry-runtime, -supervisor, -eval, -evidence,
   -guardrails, -providers) retain ownership of their BC's contract +
   crate fan-out + acceptance criteria; they roll up to axis-foundry
   for cross-BC concerns + product-level SLO.

7. **Per-BC contract surface preserved.** Each BC ships its own
   openapi.yaml + asyncapi.yaml + proto file (now under
   `microservices/foundry/contracts/{openapi,asyncapi,proto}/<bc>-*`);
   tenants and clients can still discover per-BC APIs independently.

### Negative

1. **One-time migration cost.** This ChangeSet executes ~493 file moves
   + 13 consolidated top-level docs + 3 ADRs + 1 spec consolidation +
   any downstream consumer remap (currently zero; see ADR-0138 for
   verification + sunset window).

2. **Larger single µservice directory.** `microservices/foundry/` now
   contains 493 files vs prior ~75-100 per split µservice. Mental-model
   adjustment for contributors; mitigated by the per-BC archive +
   BC-prefixed artefacts.

3. **Per-BC deployment cadence converges.** Per-BC Helm subcharts allow
   per-BC update without full-product deploy, but the foundry-as-product
   release coordinates across six BCs. The prior 6-way split's *theoretical*
   per-µservice deploy cadence is replaced with *actual* per-BC subchart
   cadence — this is honest, but it does mean BC release coordination is
   formally a foundry-product concern.

4. **External IP references to old paths break.** Any external
   reference to `microservices/foundry-<bc>/IP-NNN-...md` needs remap to
   `microservices/foundry/IP-NNN-<bc>-...md` (renumbered). ADR-0138
   ships the Strangler migration for this.

5. **bc-sources archive is auxiliary surface.** Contributors need to
   learn that authoritative BC-internal detail lives at
   `bc-sources/<bc>/` and authoritative cross-BC summary lives at
   `microservices/foundry/<doc>.md`. Mitigated by frontmatter
   `bc_archive` pointer in the top-level PRD + consistent table-of-
   contents in each consolidated doc.

### Operational

- **New CI lanes** (registered in `.github/branch-protection.yaml` via
  successor-IP ChangeSet):
  - `oya-governance-foundry-bc-source-coherence` — verifies no orphan
    reference between `microservices/foundry/<doc>.md` and
    `bc-sources/<bc>/<doc>.md`; BLOCKER from M01.
  - `oya-governance-foundry-six-path-zero-usage` — verifies no caller
    refers to the deprecated `microservices/foundry-<bc>/` paths; per
    ADR-0138 Strangler. REPORT-ONLY until Phase 3 exit; BLOCKER from
    Phase 4 onward.
- **Helm chart structure**: `microservices/foundry/iac/helm/` contains
  six subchart trees (`runtime/`, `supervisor/`, `eval/`, `evidence/`,
  `guardrails/`, `providers/`), each preserving its prior Chart.yaml +
  values.yaml verbatim. Top-level `Chart.yaml` (to be authored in a
  successor-IP under `microservices/foundry/IP-NNN-foundry-helm-rollup.md`)
  declares the six as dependencies.
- **Catalog**: `microservices/foundry/catalog/` contains all 135 crate-
  catalog records; the BC is in each filename; `oya-check-authority-cohesion`
  reads these as before.

## Clean Architecture Impact

| Lane | Impact | Action |
|---|---|---|
| `dependency-direction` (LEAN-A1) | unchanged | per-BC crate fan-out preserves layer rules; no crate moved |
| `cross-product` (LEAN-A2) | unchanged | foundry remains one product; cross-product rule still refuses non-foundry crate imports |
| `per-microservice-layout` (ADR-0131) | adapted | one µservice with internal BCs is still a flat layout per ADR-0131; ADR-0137 documents the BC structure as a foundry-specific overlay |
| `authority-cohesion` (ADR-0123) | improved | HG-FOUNDRY registers once instead of HG-FR + HG-FS + HG-FE + HG-FEV + HG-FG + HG-FP |
| `foundry-bc-source-coherence` (NEW) | new BLOCKER | enforces bc-sources archive + top-level coherence |
| `foundry-six-path-zero-usage` (NEW) | new REPORT-ONLY→BLOCKER | ADR-0138 Strangler enforcement |

## Verification

- [ ] `find microservices/foundry-* -type d` returns empty (source dirs
      deleted).
- [ ] `find microservices/foundry -type f | wc -l` returns 493 (full
      preservation).
- [ ] `grep -rn "microservices/foundry-\(runtime\|supervisor\|eval\|evidence\|guardrails\|providers\)" microservices/ docs/ specs/ registry/` returns zero
      hits outside this ADR, ADR-0137, ADR-0138, and the bc-sources
      archive (where historical PRDs / PHASE-01s reference their own
      former paths internally).
- [ ] `cargo run -p oya-check-authority-cohesion -- --repo-root .` exits 0.
- [ ] Each of the 6 BC contract surfaces lints clean.
- [ ] `specs/microservices/foundry.json` exists and validates against
      the per-microservice spec schema.
- [ ] ADRs 0137 + 0138 exist with full Status / Date / Context / Decision
      / Alternatives / Consequences / References.

## References

- ADR-0022: Autonomy tiers T0-T4 (referenced by runtime + guardrails +
  supervisor BCs).
- ADR-0024: Foundry eval harness (eval BC origin).
- ADR-0025: Foundry runtime consolidation (runtime BC origin).
- ADR-0056: BNF v4.1 naming (crate names preserved).
- ADR-0105: 13-layer enum.
- ADR-0106: application→usecase rename.
- ADR-0110: ChangeSet state machine.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward policy.
- ADR-0133: Industry best-practice conformance.
- ADR-0134: Connect-dissolution Strangler (analogous migration pattern).
- ADR-0137: Foundry bounded contexts (companion ADR — names the six BCs +
  inter-BC dependency rules).
- ADR-0138: Foundry six-path deprecation (companion ADR — Strangler
  migration for any external references to the old paths).
- AWS Bedrock product documentation — `docs.aws.amazon.com/bedrock`.
- Google Cloud Vertex AI Agent Builder — `cloud.google.com/vertex-ai/docs/generative-ai/agents/overview`.
- Microsoft Azure AI Foundry — `learn.microsoft.com/azure/ai-foundry`.
- Anthropic Console / Claude API — `docs.anthropic.com/claude/docs`.
- Palantir AIP product documentation.
- LangChain LangSmith + LangGraph — `python.langchain.com`, `smith.langchain.com`.
- Eric Evans, *Domain-Driven Design* (Addison-Wesley, 2003) — bounded
  contexts.
- Vaughn Vernon, *Implementing Domain-Driven Design* (Addison-Wesley,
  2013) — BC integration patterns.
- `feedback_quality_performance_scalability_bar.md` — hyperscaler-grade
  bar; industry-leader bar.
- `feedback_workflow_objectgraph_adapter_layer.md` — cross-BC traffic
  through Workflow + Ontology adapter layer (cross-µservice rule preserved
  within foundry; cross-BC rule remains: BCs talk over typed event +
  ontology objects, not direct kernel-port imports across BCs).
- `feedback_flat_product_catalog.md` — flat product catalog rule.
- `feedback_bominal_inheritance_precedence.md` — Bominal ADR inheritance
  baseline.
- `feedback_no_silent_regression.md` — public-contract preservation.
