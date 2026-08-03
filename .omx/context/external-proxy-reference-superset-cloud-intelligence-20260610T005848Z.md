# Context Snapshot: External Proxy Reference Superset Cloud Intelligence

## Task statement
Create a methodical `$ralplan` consensus plan to capture all features/capabilities of `askalf/dario` and reimplement them as a Rust, Kubernetes/cloud-native `cloud-intelligence` superset with worker pods in mind.

## Desired outcome
A durable consensus planning artifact that inventories external proxy reference capabilities, maps them to current Oyatie evidence, identifies gaps, and defines an execution-ready Rust/Kubernetes plan with tests, acceptance criteria, risks, ADR, staffing, launch hints, and team verification path.

## Constraints
- Planning mode only: write only `.omx/context`, `.omx/plans`, `.omx/specs`, `.omx/state` records.
- No source-code implementation in this turn.
- Must use Architect then Critic review sequentially before final consensus.
- Must not ask for confirmation; non-interactive `$ralplan` outputs final plan and stops.
- Current repo governance: read `/specs/root-hub-pointers.json` first; `docs/AGENTS.md` is agent operating contract until PHASE-5.
- external reference and provider behavior may have changed; use current upstream repo evidence, not memory alone.

## Known local facts before fresh inventory
- `cloud/cloud-intelligence` exists with Rust crates for kernel, rest, app, authz, Codex adapter, event sinks, OpenBao adapter.
- Current router exposes `/v1/messages`, admin pool routes, health/livez/readyz/metrics; broader OpenAI-compatible surface appears specified but not fully implemented.
- Current kernel provider enum is Anthropic + Codex; broader provider list appears in older Intelligence substrate ADRs.
- ADR-0384 lists external reference repos named in ADR-0384/ADR-0255 references, but not external reference.

## Unknowns/open questions to resolve by inspection
- Exact external proxy reference capability inventory from upstream source/docs/tests.
- Whether the external reference includes features not captured by existing ADR-0384 references.
- The minimum set of new Oyatie artifacts needed to make the external proxy reference baseline a first-class reference target without source edits now.
- Worker pod topology: which functions belong in gateway pods vs workers/controllers/watchers.

## Likely codebase touchpoints
- `cloud/cloud-intelligence/**`
- `docs/decisions/ADR-0373-llm-gateway-production-design.md`
- `docs/decisions/ADR-0384-llm-gateway-oauth-subscription-pool-redesign.md`
- `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- `specs/microservices/intelligence.json`
- `cloud/cloud-intelligence/contracts/*`
- `cloud/cloud-intelligence/k8s/*`, `iac/k8s/helm/*`

## User update: source-driven model routing target
The user clarified that the external reference model routing is closer to the desired Oyatie model proxy/routing shape. Activate `$source-driven-development`: plan decisions must be grounded in external reference upstream source/docs plus official provider/Kubernetes/cloud-native documentation where relevant. external reference routing semantics should be treated as the comparative target for model proxy/routing, not merely as an OAuth subscription-pool example.

## User update: planning discipline skills
The user explicitly activated `$spec-driven-development`, `$superpowers:test-driven-development`, `$incremental-implementation`, and `$api-and-interface-design`. The plan must be spec-first, API/interface-first, test-first, and split into incremental slices with disjoint verification gates before implementation. No source-code edits in ralplan.
