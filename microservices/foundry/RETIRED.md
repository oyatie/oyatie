# foundry µservice — RETIRED

Retired 2026-05-21 per ADR-0335.

## Reason

`foundry` was the internal Hermes pipeline name per ADR-0136-amendment + ADR-0220 + ADR-0239 + ADR-0247. Per ADR-0255 KS#14 (intelligence two-layer AI substrate), foundry absorbs into the `intelligence` µservice as the canonical AI substrate.

The retirement removes a service boundary. It does NOT remove any AI capability, eval substrate, training surface, RLHF workflow, red-team workflow, model registry, or guardrail stack. All of those live on under `intelligence`.

The "Hermes" name is retired corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0328 D-12.22..D-12.24.

## Absorbed by

`microservices/intelligence/`

## Authority

- `docs/decisions/ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md` — this retirement
- `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md` — intelligence two-layer substrate (KS#14)
- `docs/decisions/ADR-0247-self-hosting-self-modification-doctrine.md` — self-modification via `oyatie.foundry.*` Cedar principals (D-10 retires Hermes name)
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` — D-9.18 sequences this wave; D-9.22 + D-12.22..D-12.24 drop Hermes as canonical primitive
- `docs/decisions/ADR-0138-foundry-six-path-deprecation.md` — strangler discipline pattern
- `docs/decisions/ADR-0136-amendment` — foundry-internal scope clarification (historical)
- `docs/decisions/ADR-0220-consumer-intelligence-substrate.md` — consumer AI substrate split (historical)
- `docs/decisions/ADR-0239-amendment-foundry-internal-scope-clarification-2026-05-18.md` — internal-only amendment (historical)

## Precedent

- Wave 15K — `network` → `community` (merged-into-successor + RETIRED.md + content absorption)
- Wave 15L — `cell` retired (ADR-0333 — pattern not service)
- Wave 15O — `shorts` → `social` (ADR-0334 — flavor not separate concern)

## Successor pointers

Substantive content absorbed under `microservices/intelligence/`:

- Model routing → `microservices/intelligence/manifest.json#bounded_contexts.model-routing`
- Providers (18 first-class providers) → `microservices/intelligence/manifest.json#bounded_contexts.providers`
- Guardrails → `microservices/intelligence/manifest.json#bounded_contexts.guardrails`
- Eval → `microservices/intelligence/manifest.json#bounded_contexts.eval`
- Attribution → `microservices/intelligence/manifest.json#bounded_contexts.attribution`
- Audit-tap → `microservices/intelligence/manifest.json#bounded_contexts.audit-tap`
- Credential resolver (OpenBao) → `microservices/intelligence/manifest.json#bounded_contexts.credential-resolver`
- Brand UX surface → `microservices/intelligence/manifest.json#bounded_contexts.brand-ux-surface`
- Assist-draft → `microservices/intelligence/manifest.json#bounded_contexts.assist-draft`
- Context-aware retrieval → `microservices/intelligence/manifest.json#bounded_contexts.context-aware-retrieval`

Self-modification execution remains under the `oyatie.foundry.*` Cedar principal namespace per ADR-0247 (the Cedar principal namespace persists even though the µservice retires).

Agentic-pipeline doctrine (changeset state, admission gate, merge queue, completion gate, webhook-driven invocation, VCS orchestrator) lives in ADRs 0110, 0111, 0112, 0113, 0116, 0247, 0255 and is implemented by the cross-cutting substrate µservices (`vcs-orchestrator`, `intelligence`, `workflow`, `audit-chain`, `observability`, `identity`, `tenancy`, `policy-engine`).

## Crate transition debt

`oya-foundry-*` workspace crates are retained as transition debt per ADR-0335 D-37..D-50 (following the ADR-0333 D-59 precedent established by Wave 15L). The crate names are namespaces, not service boundaries. New code does not generate `oya-foundry-*` crates; new AI substrate code lands under `oya-intelligence-*`. The full rename cascade is sequenced as a separate cleanup wave.

## Historical evidence preserved

The historical foundry artifacts (PRD, ARCHITECTURE, PHASE-* docs, IPs, contracts, dashboards, runbooks, slos, policy, iac, scorecards, capabilities, threat-model, dpia, compliance, capacity-models, faqs, onboarding, spec) remain in place as historical evidence but are no longer live authority. Live authority for AI substrate concerns is `microservices/intelligence/` per ADR-0255.

## Do not

- Do NOT cite `microservices/foundry/PRD.md` as live authority.
- Do NOT cite `microservices/foundry/ARCHITECTURE.md` as live authority.
- Do NOT generate new `oya-foundry-*` crates.
- Do NOT introduce "Hermes" as a canonical primitive in new content.
- Do NOT treat `microservices/foundry/` as a destination for new IPs, runbooks, dashboards, or SLOs.
