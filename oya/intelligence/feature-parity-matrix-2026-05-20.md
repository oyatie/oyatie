# Intelligence Feature-Parity Matrix
Date: 2026-05-20
Batch: Wave 3 Batch 3.2
µservice: `intelligence`
Counterparts: OpenAI Platform / Anthropic Claude Platform / Google Vertex AI
Method: union-coverage audit against current `microservices/intelligence/` artifacts plus public counterpart documentation.

## Citation Anchor Block
1. Local product scope: `microservices/intelligence/PRD.md:15-31`, `microservices/intelligence/README.md:13-20`, `microservices/intelligence/ARCHITECTURE.md:52-64`.
2. Local contracts: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:209`, `microservices/intelligence/contracts/proto/intelligence-v1.proto:57-97`.
3. Local policies and operations: `microservices/intelligence/policy/provider-routing.cedar:20-175`, `microservices/intelligence/policy/byok-gating.cedar:20-92`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml:37-45`.
4. OpenAI public references: `https://platform.openai.com/docs/guides/tools-file-search`, `https://openai.com/api/pricing/`.
5. Anthropic and Google public references: `https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching`, `https://docs.anthropic.com/en/api/rate-limits`, `https://cloud.google.com/vertex-ai/generative-ai/docs`, `https://cloud.google.com/vertex-ai/docs/vector-search/overview`.

## §1 Counterpart 1 — OpenAI Platform Capability Surface
1. OpenAI surface includes direct model invocation for text, image, audio, and multimodal workflows.
2. OpenAI surface includes tool calling and structured outputs for application workflows.
3. OpenAI surface includes hosted file search and vector store workflows.
4. OpenAI surface includes embeddings.
5. OpenAI surface includes batch processing.
6. OpenAI surface includes fine-tuning and model customization options.
7. OpenAI surface includes moderation and safety controls.
8. OpenAI surface includes project-scoped keys, organization management, usage, and pricing controls.
9. OpenAI surface includes realtime or streaming model interaction.
10. OpenAI surface includes model and tool documentation for developer onboarding.
11. Intelligence has comparable model-dispatch intent through provider routing and dispatch contracts: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`.
12. Intelligence has provider choice intent through provider hints and routing policy: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:64`, `microservices/intelligence/policy/provider-routing.cedar:20-175`.
13. Intelligence has embedding and retrieval intent through capability files and benchmark docs: `microservices/intelligence/capabilities/context-aware-retrieval.yaml`, `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:32-52`.
14. Intelligence has safety-control intent through guardrails, refusal policy, and abuse defense: `microservices/intelligence/capabilities/guardrails.yaml`, `microservices/intelligence/policy/refusal-baseline.cedar`, `microservices/intelligence/policy/abuse-defence.cedar`.
15. Intelligence has audit and attribution surfaces beyond basic counterpart parity: `microservices/intelligence/capabilities/audit-tap.yaml`, `microservices/intelligence/capabilities/attribution.yaml`.
16. Intelligence has SLO and runbook targets for latency, first token, streaming, audit, and refusal quality: `microservices/intelligence/slos/first-token-latency.openslo.yaml:37-45`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml:37-45`.
17. Intelligence has BYOK policy and credential resolver intent: `microservices/intelligence/policy/byok-gating.cedar:20-92`.
18. Intelligence has provider-specific adapter implementation plans, including OpenAI: `microservices/intelligence/IP-012-adapter-openai.md`.
19. Intelligence gap versus OpenAI: no service-local source implementation is present.
20. Intelligence gap versus OpenAI: no executable test tree is present.
21. Intelligence gap versus OpenAI: no hosted vector-store implementation evidence is present in source.
22. Intelligence gap versus OpenAI: no direct batch job execution implementation is present in source.
23. Intelligence gap versus OpenAI: no first-class tenant-class semantics appear in contracts.
24. Intelligence gap versus OpenAI: current IaC does not provide the six required deployment contexts.
25. Intelligence gap versus OpenAI: current docs do not prove production rate-limit management across providers.
26. Intelligence comparative advantage: cross-provider policy routing is a native design goal rather than a single-provider API.
27. Intelligence comparative advantage: Cedar policy and audit tap are first-order service artifacts.
28. Intelligence comparative advantage: local compliance pack routing is stronger in documentation than many generic provider quickstarts.
29. OpenAI parity verdict: documented partial coverage.
30. OpenAI parity risk: high until OpenTofu context modules, Rust implementation, tests, and tenant-class overlays exist.

## §2 Counterpart 2 — Anthropic Claude Platform Capability Surface
1. Anthropic surface includes Messages API for Claude model invocation.
2. Anthropic surface includes tool use and structured interactions.
3. Anthropic surface includes prompt caching and context-management economics.
4. Anthropic surface includes files, citations, vision-capable workflows, and batch processing.
5. Anthropic surface includes safety policy and model behavior documentation.
6. Anthropic surface includes rate-limit documentation and usage administration.
7. Anthropic surface includes streaming responses.
8. Anthropic surface includes enterprise controls through account, key, and usage management.
9. Anthropic surface includes model-family differentiation and latency/cost tradeoffs.
10. Intelligence has Anthropic adapter planning: `microservices/intelligence/IP-011-adapter-anthropic.md`.
11. Intelligence has provider routing policy that can select or deny Anthropic by region, pack, and credential mode: `microservices/intelligence/policy/provider-routing.cedar:20-175`.
12. Intelligence has BYOK and platform-default gating around regulated use cases: `microservices/intelligence/policy/byok-gating.cedar:20-92`.
13. Intelligence has streaming transport plans: `microservices/intelligence/IP-016-streaming-sse-transport.md`, `microservices/intelligence/IP-017-streaming-websocket-transport.md`.
14. Intelligence has first-token and streaming SLOs: `microservices/intelligence/slos/first-token-latency.openslo.yaml:37-45`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml:37-45`.
15. Intelligence has refusal-quality SLOs that map to safety behavior: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml:23-59`, `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml:23-58`.
16. Intelligence has prompt-injection and refusal runbooks: `microservices/intelligence/runbooks/prompt-injection-detected.md`, `microservices/intelligence/runbooks/refusal-false-positive-cascade.md`.
17. Intelligence has an eval canonicalen-set plan: `microservices/intelligence/IP-021-eval-canonicalen-set.md:16-53`.
18. Intelligence has a capacity model that records provider QPS, request, token, retry, and streaming assumptions: `microservices/intelligence/capacity-model.md:21-98`.
19. Intelligence gap versus Anthropic: prompt caching is not a first-class contract field.
20. Intelligence gap versus Anthropic: provider-specific cache economics are not reflected in tenant billing policy.
21. Intelligence gap versus Anthropic: current adapter plan is not backed by source.
22. Intelligence gap versus Anthropic: no executable Anthropic contract tests are present under the service path.
23. Intelligence gap versus Anthropic: rate-limit behavior is documented but not implemented in service-local code.
24. Intelligence gap versus Anthropic: tenant-class fields are absent from dispatch, events, cost, and policy.
25. Intelligence comparative advantage: intelligence can fail over between providers if the router is implemented as documented.
26. Intelligence comparative advantage: audit row sealing and attribution are richer than a basic model API wrapper.
27. Intelligence comparative advantage: regulated-pack routing is expressed in policy, not just application prose.
28. Anthropic parity verdict: strong design-level overlap, implementation proof missing.
29. Anthropic parity risk: medium-high because prompt caching and rate-limit controls are not yet first-class in contracts.
30. Anthropic parity action: add cache-policy metadata and provider-rate-limit contract tests when implementing the Anthropic adapter.

## §3 Counterpart 3 — Google Vertex AI Capability Surface
1. Vertex AI surface includes Gemini model invocation through Google Cloud.
2. Vertex AI surface includes Model Garden and managed model deployment patterns.
3. Vertex AI surface includes embeddings and Vector Search.
4. Vertex AI surface includes context caching.
5. Vertex AI surface includes provisioned throughput and quota management.
6. Vertex AI surface includes tuning, batch, evaluation, and monitoring workflows.
7. Vertex AI surface includes IAM, service-account identity, regional controls, and cloud perimeter controls.
8. Vertex AI surface includes enterprise billing and Cloud-native operational integration.
9. Vertex AI surface includes managed observability and logging paths.
10. Intelligence has a Google Vertex adapter plan: `microservices/intelligence/IP-013-adapter-google-vertex.md`.
11. Intelligence has provider enum and routing hints for Vertex-oriented routing: `microservices/intelligence/contracts/proto/intelligence-v1.proto:73-97`.
12. Intelligence has region and pack routing policy that can express residency constraints: `microservices/intelligence/policy/provider-routing.cedar:20-175`.
13. Intelligence has data-residency documentation for region and exception handling: `microservices/intelligence/policy/data-residency.md:43-76`.
14. Intelligence has compliance docs covering EU, KR, CN, HIPAA, FedRAMP, PCI, and audit readiness: `microservices/intelligence/compliance.md:40-149`.
15. Intelligence has cost budget entries for Google model usage and pricing assumptions: `microservices/intelligence/cost-budget.md:21-50`.
16. Intelligence has retrieval benchmarks and capability files: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`, `microservices/intelligence/capabilities/context-aware-retrieval.yaml`.
17. Intelligence has multi-region documentation: `microservices/intelligence/multi-region.md`.
18. Intelligence has dashboards for provider latency and refusal rates: `microservices/intelligence/dashboards/provider-latency-heatmap.json`, `microservices/intelligence/dashboards/refusal-rate-by-pack.json`.
19. Intelligence gap versus Vertex AI: no Google-cloud-specific provisioned-throughput contract or implementation is present.
20. Intelligence gap versus Vertex AI: no managed vector-search implementation is present under source.
21. Intelligence gap versus Vertex AI: no service-account or cloud perimeter context module exists under canonical OpenTofu paths.
22. Intelligence gap versus Vertex AI: no six-context deployment module exists for guest-on-aws, guest-on-oci, on-prem, colo, public cloud, or Oyatie provider context.
23. Intelligence gap versus Vertex AI: no supported OS manifest is present.
24. Intelligence gap versus Vertex AI: no executable monitoring export implementation is present in source.
25. Intelligence comparative advantage: Vertex AI is strong inside Google Cloud, while intelligence intends provider-agnostic dispatch across all six Oyatie contexts.
26. Intelligence comparative advantage: Cedar policy can normalize provider controls under tenant and compliance facts.
27. Intelligence comparative advantage: Foundry substrate reuse is designed into intelligence contracts and policy.
28. Vertex parity verdict: credible policy and routing design, missing managed-cloud parity implementation.
29. Vertex parity risk: high until canonical OpenTofu and runtime integration are present.
30. Vertex parity action: express provider-specific throughput and residency controls as deployment-context overlays, not feature-quality classes.

## §4 Union-Coverage Matrix
| # | Capability family | OpenAI | Anthropic | Vertex AI | Oyatie current coverage | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Text model invocation | Present | Present | Present | Documented partial | `contracts/openapi/intelligence-v1.yaml:52-64` | No source implementation. |
| 2 | Streaming responses | Present | Present | Present | Planned partial | `IP-016-streaming-sse-transport.md`; `IP-017-streaming-websocket-transport.md` | No executable transport. |
| 3 | Multimodal input | Present | Present | Present | Planned partial | `IP-018-multi-modal-audio-video.md` | No implementation proof. |
| 4 | Tool use / function calling | Present | Present | Present | Not clearly contracted | `contracts/openapi/intelligence-v1.yaml:52-64` | Tool schema not first-class. |
| 5 | Structured output | Present | Present | Present | Not clearly contracted | `contracts/proto/intelligence-v1.proto:16-20` | Output validation absent. |
| 6 | Provider routing | Limited to provider ecosystem | Limited to provider ecosystem | Limited to cloud ecosystem | Strong design | `policy/provider-routing.cedar:20-175` | No router source. |
| 7 | Multi-provider failover | Not primary | Not primary | Not primary | Strong design | `failure-modes.md:27-49`; `capacity-model.md:21-98` | No executable failover tests. |
| 8 | BYOK / credential control | Enterprise features | Enterprise features | Cloud IAM/CMEK features | Strong policy design | `policy/byok-gating.cedar:20-92` | No tenant-class integration. |
| 9 | Platform-default credentials | Present by account | Present by account | Present by project | Strong policy design | `policy/byok-gating.cedar:61-74` | Foundry claim namespace needs explicit docs. |
| 10 | Data residency | Provider controls | Provider controls | Cloud region controls | Strong design | `policy/data-residency.md:43-76` | No context IaC proof. |
| 11 | Compliance pack routing | Partial provider support | Partial provider support | Strong cloud support | Strong design | `compliance.md:40-149`; `policy/provider-routing.cedar:20-175` | No tenant-class allowance field. |
| 12 | Moderation / refusal | Present | Safety policy present | Safety tooling present | Strong design | `slos/refusal-false-negative-rate.openslo.yaml:23-59` | No executable classifier. |
| 13 | Prompt-injection defense | Developer-guidance oriented | Policy guidance | Security tooling | Strong runbooks and policy | `runbooks/prompt-injection-detected.md`; `policy/refusal-baseline.cedar` | No source validation. |
| 14 | Evaluation | Present | Present | Present | Strong plan | `IP-021-eval-canonicalen-set.md:16-53` | No runner implementation. |
| 15 | Audit emission | Usage and logs | Usage and logs | Cloud logs | Strong Oyatie-specific design | `capabilities/audit-tap.yaml`; `slos/audit-emission-success.openslo.yaml:37-45` | No source implementation. |
| 16 | Attribution | Limited by provider features | Citations supported | Citation and grounding features | Strong Oyatie-specific design | `capabilities/attribution.yaml`; `IP-006-domain-layer-attribution.md` | No end-to-end tests. |
| 17 | Retrieval / RAG | File search and vector stores | Files/citations | Vector Search | Documented partial | `benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52` | No retrieval source. |
| 18 | Embeddings | Present | Limited by model family | Present | Documented partial | `benchmarks/openai-anthropic-vertex-vs-oyatie.md:32-41` | No service source. |
| 19 | Fine-tuning / tuning | Present | Limited | Present | Planned adjacent only | `benchmarks/openai-anthropic-vertex-vs-oyatie.md:54-63` | Ownership boundary unclear. |
| 20 | Batch processing | Present | Present | Present | Not complete | `capacity-model.md:21-46` | No batch contract. |
| 21 | Prompt caching / context cache | Provider-specific | Present | Present | Not first-class | `cost-budget.md:21-50` | No cache policy field. |
| 22 | Rate-limit management | Present | Present | Present | Planned partial | `failure-modes.md:27-49`; `runbooks/provider-rate-limit-saturation.md` | No implementation. |
| 23 | Cost attribution | Present | Present | Present | Strong design | `cost-budget.md:57-79`; `dashboards/finops-cost-attribution.md` | Needs tenant_class. |
| 24 | Usage caps | Present | Present | Present | Partial design | `cost-budget.md:96-108` | New tenant classes absent. |
| 25 | Enterprise admin | Present | Present | Present | Partial | `policy/tenant-scope.cedar`; `policy/auditor-scope.cedar` | Admin UI absent. |
| 26 | Project or tenant scoping | Present | Present | Present | Strong design | `contracts/openapi/intelligence-v1.yaml:52-64` | Tenant class absent. |
| 27 | Managed model catalog | Present | Present | Present | Partial through provider hints | `manifest.json:68-190` | Catalog not executable. |
| 28 | Model selection UX | Present | Present | Present | Partial | `README.md:37-48`; `ARCHITECTURE.md:66-87` | Ambiguous badge wording. |
| 29 | Developer SDKs | Present | Present | Present | Partial plan | `sdk-plan.md`; `reference-implementations/chat-completion-rust-sdk.md` | Generator governance unresolved. |
| 30 | Webhooks / events | Present in some flows | Present in some flows | Cloud events | Strong event design | `contracts/asyncapi/intelligence-events-v1.yaml:171-220` | Event consumers need tests. |
| 31 | Observability dashboards | Present | Present | Present | Strong docs | `dashboards/intelligence-overview.json`; `dashboards/provider-latency-heatmap.json` | Runtime exporters absent. |
| 32 | Incident response | Provider support | Provider support | Cloud support | Strong local docs | `incident-response.md:29-118` | Drills need environments. |
| 33 | Regional failover | Provider account dependent | Provider account dependent | Strong cloud support | Strong design | `multi-region.md`; `failure-modes.md:27-49` | Context IaC absent. |
| 34 | On-prem support | Not primary | Not primary | Hybrid via cloud patterns | Required but missing IaC | `specs/master-plan-sequencing.json:704-745` | No `iac/on-prem/`. |
| 35 | Colo support | Not primary | Not primary | Not primary | Required but missing IaC | `specs/master-plan-sequencing.json:704-745` | No `iac/colo/`. |
| 36 | Guest cloud support | Not applicable as provider | Not applicable as provider | Cloud-specific | Required but missing IaC | `specs/master-plan-sequencing.json:704-745` | No guest context dirs. |
| 37 | OCI Always Free profile | Not provider feature | Not provider feature | Not provider feature | Required but absent | `specs/master-plan-sequencing.json:857-867` | No profile path. |
| 38 | OS support manifest | Provider-managed | Provider-managed | Provider-managed | Required but absent | `specs/master-plan-sequencing.json:777-815` | No `supported-oses.json`. |
| 39 | OpenTofu deployment | Not counterpart feature | Not counterpart feature | Cloud templates exist | Required but absent | `specs/master-plan-sequencing.json:747-775`; `iac/terraform/openbao-policy.tf:7-14` | Terraform path exists. |
| 40 | Foundry substrate reuse | Not counterpart feature | Not counterpart feature | Not counterpart feature | Partial design | `ARCHITECTURE.md:89-113`; `policy/dispatch-authorization.cedar:41-57` | Owner transfer incomplete. |
| 41 | Self-modification doctrine | Not counterpart feature | Not counterpart feature | Not counterpart feature | Partially referenced | `ARCHITECTURE.md:1080-1111`; `compliance.md:657-665` | Needs absorption ADR link. |
| 42 | Minor safety | Provider safety | Provider safety | Provider safety | Planned | `IP-024-minor-protection-wiring.md` | No implementation. |
| 43 | Regional legal packs | Provider controls | Provider controls | Cloud controls | Strong design | `IP-025-cn-pipl-pack-adapter.md`; journey compliance IPs | No executable pack tests. |
| 44 | Audit row sealing | Logs | Logs | Cloud logs | Strong differentiated plan | `IP-022-audit-tap-merkle-seal.md:16-62` | No source. |
| 45 | Replay/backfill | Provider logs | Provider logs | Cloud logs | Planned | `backfill-replay.md` | No runner. |
| 46 | Customer migration | Docs and partners | Docs and partners | Cloud migration guides | Partial | `migration-playbooks/from-openai-enterprise.md` | Needs tested path. |
| 47 | Developer onboarding | Strong | Strong | Strong | Good docs | `onboarding/ai-platform-engineer-first-week.md`; `faqs/ai-platform-engineer-faq.md` | Retired labels need cleanup. |
| 48 | Launch maturity | Production providers | Production provider | Production cloud | Not proven | `PRD.md:28-31`; inventory source gap | Needs implementation proof. |

## §5 Family Summary
1. Invocation family status: documented partial coverage.
2. Invocation family strength: contract fields for tenant, audience, provider hint, and credential kind.
3. Invocation family weakness: no source implementation.
4. Streaming family status: planned partial coverage.
5. Streaming family strength: dedicated transport implementation plans and throughput SLO.
6. Streaming family weakness: no executable transport path.
7. Safety family status: strong design coverage.
8. Safety family strength: Cedar policies, refusal SLOs, prompt-injection runbooks, and compliance docs.
9. Safety family weakness: no executable safety tests under service path.
10. Retrieval family status: documented partial coverage.
11. Retrieval family strength: benchmark and capability docs.
12. Retrieval family weakness: no retrieval implementation source.
13. Provider-routing family status: strong design coverage.
14. Provider-routing family strength: provider routing policy and multiple adapter plans.
15. Provider-routing family weakness: no adapter source or contract tests.
16. Credential family status: strong design coverage.
17. Credential family strength: BYOK policy, credential resolver plan, and regulated-pack gating.
18. Credential family weakness: tenant-class allowance is missing.
19. Audit family status: strong design coverage.
20. Audit family strength: audit tap capability, event contracts, Merkle seal plan, and audit SLO.
21. Audit family weakness: no source implementation and no replay tests.
22. Cost family status: strong design coverage.
23. Cost family strength: provider price matrix, cost records, dashboards, and caps.
24. Cost family weakness: tenant-class and revenue-share semantics are absent.
25. Deployment family status: weak canonical coverage.
26. Deployment family strength: Kubernetes and Helm assets exist.
27. Deployment family weakness: required OpenTofu six-context modules are absent.
28. OS family status: missing required manifest.
29. OS family strength: no positive evidence beyond general deployment docs.
30. OS family weakness: no `supported-oses.json`.
31. Foundry absorption family status: partial.
32. Foundry absorption family strength: audience, policy, compliance, and architecture hooks exist.
33. Foundry absorption family weakness: explicit `llm-substrate` owner transfer is absent.
34. SDK family status: partial plan.
35. SDK family strength: Rust reference implementation exists.
36. SDK family weakness: generator and allowed-output governance need canonicalization.

## §6 Headline Gap Analysis
1. Gap H1: Intelligence has counterpart-level product ambition but not counterpart-level implementation proof.
2. Evidence H1: no `src/` and no executable `tests/` directory were present in the target path.
3. Impact H1: parity claims must remain design claims until Rust source and tests land.
4. Gap H2: The six deployment contexts are not represented in service IaC.
5. Evidence H2: master plan requires six contexts; service has only K8s, Helm, and Terraform-labeled IaC: `specs/master-plan-sequencing.json:704-775`, `microservices/intelligence/iac/terraform/openbao-policy.tf:7-14`.
6. Impact H2: all-context deployability cannot be claimed.
7. Gap H3: OpenTofu doctrine is not satisfied.
8. Evidence H3: existing service IaC includes Terraform syntax and path names: `microservices/intelligence/iac/terraform/openbao-policy.tf:7-14`.
9. Impact H3: deployment artifacts must be migrated before launch readiness.
10. Gap H4: tenant-class model is missing.
11. Evidence H4: current capacity and cost docs use trial/sandbox/production/internal labels rather than the canonical tenant classes: `microservices/intelligence/capacity-model.md:50-56`, `microservices/intelligence/cost-budget.md:96-108`.
12. Impact H4: usage caps, compliance allowance, BYOK allowance, and revenue-share terms are not policy-addressable.
13. Gap H5: Foundry absorption is not fully documented.
14. Evidence H5: chat records absorption; policy and architecture recognize Foundry, but PRD and IP-001 still separate the surfaces: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`, `microservices/intelligence/PRD.md:15-31`, `microservices/intelligence/IP-001-consumer-intelligence-substrate.md:12-28`.
15. Impact H5: Foundry retirement would be premature if based only on current intelligence artifacts.
16. Gap H6: provider-specific advanced economics are not first-class.
17. Evidence H6: prompt/context cache, provisioned-throughput, and provider-rate behavior are not represented as explicit contract fields.
18. Impact H6: parity with Anthropic and Vertex enterprise control surfaces is incomplete.
19. Gap H7: OS support evidence is absent.
20. Evidence H7: master plan requires service OS manifest, and no `supported-oses.json` exists.
21. Impact H7: supported OS claims remain unproven.
22. Gap H8: benchmark docs need current-source refresh gates.
23. Evidence H8: benchmark and cost docs have useful numbers but need recurring provider refresh before launch claims: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-112`, `microservices/intelligence/cost-budget.md:21-50`.
24. Impact H8: parity targets can drift quickly as counterpart model prices and latency change.

## §7 Additive Oyatie Surface
1. Additive surface A1: cross-provider policy router.
2. Evidence A1: `microservices/intelligence/policy/provider-routing.cedar:20-175`.
3. Additive value A1: tenant, region, compliance, BYOK, and provider selection can be evaluated under Oyatie policy rather than provider-specific application code.
4. Additive surface A2: audit tap with Merkle-seal plan.
5. Evidence A2: `microservices/intelligence/IP-022-audit-tap-merkle-seal.md:16-62`.
6. Additive value A2: dispatch evidence can become tamper-evident across providers.
7. Additive surface A3: attribution as a domain capability.
8. Evidence A3: `microservices/intelligence/capabilities/attribution.yaml`; `microservices/intelligence/IP-006-domain-layer-attribution.md`.
9. Additive value A3: generated content can be connected to evidence and policy decisions.
10. Additive surface A4: regulated-pack provider routing.
11. Evidence A4: `microservices/intelligence/policy/provider-routing.cedar:20-175`; `microservices/intelligence/compliance.md:40-149`.
12. Additive value A4: legal and compliance packs are not bolted on after dispatch.
13. Additive surface A5: Foundry internal substrate route.
14. Evidence A5: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`; `microservices/intelligence/policy/dispatch-authorization.cedar:41-57`.
15. Additive value A5: internal development agents can consume the same governed substrate without bypassing product policy.
16. Additive surface A6: cost attribution tied to provider, tenant, pack, and model.
17. Evidence A6: `microservices/intelligence/cost-budget.md:57-79`; `microservices/intelligence/dashboards/finops-cost-attribution.md`.
18. Additive value A6: AI spend can be audited and capped per tenant and use case.
19. Additive surface A7: refusal-quality SLOs.
20. Evidence A7: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml:23-59`; `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml:23-58`.
21. Additive value A7: safety behavior becomes an operational objective rather than a generic policy statement.
22. Additive surface A8: all-context deployment target.
23. Evidence A8: `specs/master-plan-sequencing.json:704-745`.
24. Additive value A8: once implemented, intelligence can run in public, guest, on-prem, colo, and Oyatie-provider contexts.
25. Additive surface A9: OCI Always Free profile for demo-trial substrate.
26. Evidence A9: `specs/master-plan-sequencing.json:857-867`.
27. Additive value A9: the free tenant class can be infrastructure-bounded without reducing product quality.
28. Additive surface A10: service-owned policy proof.
29. Evidence A10: policy directory contains Cedar files for abuse, auditor scope, BYOK, CI, emergency path, dispatch, EU high risk, provider routing, refusal, and tenant scope.
30. Additive value A10: service behavior can be tested as policy, not just code.

## §8 Parity Verdict
1. OpenAI parity: partial by design, not proven by implementation.
2. Anthropic parity: partial by design, with prompt-cache and rate-limit details needing first-class schema.
3. Google Vertex parity: partial by design, with cloud-throughput, vector, IAM, and context-module evidence missing.
4. Union parity: the planned Oyatie surface is broader than any single counterpart because it normalizes multiple providers under policy.
5. Union parity: the current artifact state is not deployable parity because implementation, tests, canonical IaC, OS manifest, and tenant-class overlays are missing.
6. Highest-confidence strength: policy, audit, compliance, and contract design depth.
7. Highest-confidence weakness: executable source and context deployment absence.
8. Most urgent parity correction: add OpenTofu six-context module skeletons with real policy-bearing variables and OCI Always Free profile.
9. Most urgent product correction: amend PRD and architecture for `llm-substrate` ownership transfer.
10. Most urgent commercial correction: introduce `tenant_class` into contracts, policy, cost, capacity, SLO overlays, and onboarding.
11. Most urgent implementation correction: land a narrow Rust dispatch-router slice with provider adapter contract tests and audit emission.
12. Final parity status: strong roadmap, incomplete productization proof.

## §9 Evidence Ledger By Capability
1. Ledger L1 dispatch request: OpenAPI exposes request shape, but source handlers are absent.
2. Ledger L1 citation: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`.
3. Ledger L1 parity result: partial.
4. Ledger L1 next proof: Rust handler plus contract test.
5. Ledger L2 provider hint: OpenAPI and proto expose provider selection fields.
6. Ledger L2 citation: `microservices/intelligence/contracts/proto/intelligence-v1.proto:73-97`.
7. Ledger L2 parity result: partial.
8. Ledger L2 next proof: router selection test across three primary counterparts.
9. Ledger L3 audience routing: consumer, developer, and internal Foundry audiences are modeled.
10. Ledger L3 citation: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`.
11. Ledger L3 parity result: differentiated design.
12. Ledger L3 next proof: Cedar decision trace for each audience.
13. Ledger L4 credential kind: credential mode exists in contract.
14. Ledger L4 citation: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:209`.
15. Ledger L4 parity result: strong design.
16. Ledger L4 next proof: OpenBao handle resolution test.
17. Ledger L5 BYOK gating: policy allows tenant and platform-default credential paths.
18. Ledger L5 citation: `microservices/intelligence/policy/byok-gating.cedar:20-92`.
19. Ledger L5 parity result: differentiated design.
20. Ledger L5 next proof: tenant-class allowance in policy facts.
21. Ledger L6 provider routing: Cedar policy covers region and regulated-pack constraints.
22. Ledger L6 citation: `microservices/intelligence/policy/provider-routing.cedar:20-175`.
23. Ledger L6 parity result: strong design.
24. Ledger L6 next proof: executable policy tests.
25. Ledger L7 data residency: service has region-handling policy prose.
26. Ledger L7 citation: `microservices/intelligence/policy/data-residency.md:43-76`.
27. Ledger L7 parity result: strong design.
28. Ledger L7 next proof: six-context OpenTofu variables and tests.
29. Ledger L8 refusal baseline: service has Cedar refusal policy.
30. Ledger L8 citation: `microservices/intelligence/policy/refusal-baseline.cedar`.
31. Ledger L8 parity result: strong design.
32. Ledger L8 next proof: canonicalen-set runner with false-positive and false-negative metrics.
33. Ledger L9 abuse defense: service owns abuse-defense policy.
34. Ledger L9 citation: `microservices/intelligence/policy/abuse-defence.cedar`.
35. Ledger L9 parity result: partial.
36. Ledger L9 next proof: hostile-input test corpus.
37. Ledger L10 audit tap: capability exists.
38. Ledger L10 citation: `microservices/intelligence/capabilities/audit-tap.yaml`.
39. Ledger L10 parity result: differentiated design.
40. Ledger L10 next proof: audit row write and event emission test.
41. Ledger L11 Merkle seal: implementation plan exists.
42. Ledger L11 citation: `microservices/intelligence/IP-022-audit-tap-merkle-seal.md:16-62`.
43. Ledger L11 parity result: differentiated design.
44. Ledger L11 next proof: hash-chain validation test.
45. Ledger L12 attribution: capability exists.
46. Ledger L12 citation: `microservices/intelligence/capabilities/attribution.yaml`.
47. Ledger L12 parity result: strong design.
48. Ledger L12 next proof: generated answer with citations and attribution record.
49. Ledger L13 retrieval: capability and benchmarks exist.
50. Ledger L13 citation: `microservices/intelligence/capabilities/context-aware-retrieval.yaml`; `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
51. Ledger L13 parity result: partial.
52. Ledger L13 next proof: retrieval engine and corpus isolation tests.
53. Ledger L14 embeddings: benchmark corpus includes throughput numbers.
54. Ledger L14 citation: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:32-41`.
55. Ledger L14 parity result: target-level only.
56. Ledger L14 next proof: batch embedding harness.
57. Ledger L15 eval: canonicalen-set plan exists.
58. Ledger L15 citation: `microservices/intelligence/IP-021-eval-canonicalen-set.md:16-53`.
59. Ledger L15 parity result: strong plan.
60. Ledger L15 next proof: eval runner and result artifact.
61. Ledger L16 safety SLOs: refusal false-negative and false-positive SLOs exist.
62. Ledger L16 citation: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml:23-59`; `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml:23-58`.
63. Ledger L16 parity result: strong target.
64. Ledger L16 next proof: measured safety report.
65. Ledger L17 dispatch SLO: latency and availability SLOs exist.
66. Ledger L17 citation: `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml:37-45`; `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml:17-20`.
67. Ledger L17 parity result: strong target.
68. Ledger L17 next proof: deployed service measurement.
69. Ledger L18 streaming SLO: first-token and streaming throughput targets exist.
70. Ledger L18 citation: `microservices/intelligence/slos/first-token-latency.openslo.yaml:37-45`; `microservices/intelligence/slos/streaming-throughput.openslo.yaml:37-45`.
71. Ledger L18 parity result: strong target.
72. Ledger L18 next proof: streaming transport test.
73. Ledger L19 cost attribution: cost record schema exists in prose.
74. Ledger L19 citation: `microservices/intelligence/cost-budget.md:57-79`.
75. Ledger L19 parity result: strong design.
76. Ledger L19 next proof: emitted cost record with tenant_class.
77. Ledger L20 usage caps: cap concepts exist but old vocabulary remains.
78. Ledger L20 citation: `microservices/intelligence/cost-budget.md:96-108`.
79. Ledger L20 parity result: partial.
80. Ledger L20 next proof: demo-trial cap enforcement.
81. Ledger L21 capacity model: provider budgets and streaming assumptions exist.
82. Ledger L21 citation: `microservices/intelligence/capacity-model.md:21-98`.
83. Ledger L21 parity result: strong plan.
84. Ledger L21 next proof: load-test results per deployment context.
85. Ledger L22 failure modes: 20-mode risk set exists.
86. Ledger L22 citation: `microservices/intelligence/failure-modes.md:27-49`.
87. Ledger L22 parity result: strong ops plan.
88. Ledger L22 next proof: game-day evidence.
89. Ledger L23 incident response: severity and regulator tables exist.
90. Ledger L23 citation: `microservices/intelligence/incident-response.md:29-118`.
91. Ledger L23 parity result: strong ops plan.
92. Ledger L23 next proof: drill record and alert routing.
93. Ledger L24 compliance: framework matrix exists.
94. Ledger L24 citation: `microservices/intelligence/compliance.md:40-149`.
95. Ledger L24 parity result: strong design.
96. Ledger L24 next proof: pack-specific acceptance tests.
97. Ledger L25 DPIA: processing and treatment corpus exists.
98. Ledger L25 citation: `microservices/intelligence/dpia.md:94-120`; `microservices/intelligence/dpia.md:194-239`.
99. Ledger L25 parity result: strong privacy design.
100. Ledger L25 next proof: data-flow test and retention enforcement.
101. Ledger L26 Foundry substrate: policy admits internal Foundry audience.
102. Ledger L26 citation: `microservices/intelligence/policy/dispatch-authorization.cedar:41-57`.
103. Ledger L26 parity result: differentiated design.
104. Ledger L26 next proof: explicit `llm-substrate` owner-transfer note.
105. Ledger L27 self-modification doctrine: architecture checklist references it.
106. Ledger L27 citation: `microservices/intelligence/ARCHITECTURE.md:1080-1111`.
107. Ledger L27 parity result: partial.
108. Ledger L27 next proof: ADR cross-reference in the absorption docs.
109. Ledger L28 OpenTofu contexts: required by master plan.
110. Ledger L28 citation: `specs/master-plan-sequencing.json:704-775`.
111. Ledger L28 parity result: missing.
112. Ledger L28 next proof: six context modules under service IaC.
113. Ledger L29 OS manifest: required by master plan.
114. Ledger L29 citation: `specs/master-plan-sequencing.json:777-815`.
115. Ledger L29 parity result: missing.
116. Ledger L29 next proof: `supported-oses.json`.
117. Ledger L30 OCI Always Free profile: required by master plan.
118. Ledger L30 citation: `specs/master-plan-sequencing.json:857-867`.
119. Ledger L30 parity result: missing.
120. Ledger L30 next proof: `iac/oci-guest/always-free/`.
121. Ledger L31 source implementation: expected for production parity.
122. Ledger L31 citation: inventory has no service-local `src/` directory.
123. Ledger L31 parity result: missing.
124. Ledger L31 next proof: first Rust crate or module owned by this service.
125. Ledger L32 executable tests: expected for parity proof.
126. Ledger L32 citation: inventory has no service-local `tests/` directory.
127. Ledger L32 parity result: missing.
128. Ledger L32 next proof: contract and policy tests.
129. Ledger L33 tenant_class: required by batch directive.
130. Ledger L33 citation: `microservices/intelligence/capacity-model.md:50-56`; `microservices/intelligence/cost-budget.md:96-108`.
131. Ledger L33 parity result: missing.
132. Ledger L33 next proof: contract, policy, cost, and SLO overlays.
133. Ledger L34 provider-pricing path: cost doc points to non-canonical IaC.
134. Ledger L34 citation: `microservices/intelligence/cost-budget.md:52-53`.
135. Ledger L34 parity result: drift.
136. Ledger L34 next proof: move pricing config into canonical service config or OpenTofu inputs.
137. Ledger L35 launch claim: PRD currently excludes maturity claims.
138. Ledger L35 citation: `microservices/intelligence/PRD.md:28-31`.
139. Ledger L35 parity result: blocked by proof gap.
140. Ledger L35 next proof: updated PRD plus measured deployment evidence.

## §10 Minimum Parity Slice
1. Slice M1: implement dispatch request parsing in Rust.
2. Slice M1 proof: OpenAPI contract test passes.
3. Slice M2: implement provider-router decision for OpenAI, Anthropic, and Vertex adapters.
4. Slice M2 proof: routing policy test covers allowed, denied, and failover cases.
5. Slice M3: implement credential handle resolution through the existing BYOK policy.
6. Slice M3 proof: raw secret material never appears in audit or logs.
7. Slice M4: implement audit event emission.
8. Slice M4 proof: AsyncAPI payload validates and audit SLO counter increments.
9. Slice M5: implement refusal baseline evaluation.
10. Slice M5 proof: canonicalen-set sample returns expected false-positive and false-negative counters.
11. Slice M6: implement first streaming path.
12. Slice M6 proof: first-token and tokens/sec metrics are emitted.
13. Slice M7: implement tenant_class propagation.
14. Slice M7 proof: `demo_trial`, `paid`, and `revenue_share` are present in policy facts and cost records.
15. Slice M8: implement internal Foundry dispatch path.
16. Slice M8 proof: `internal-foundry` audience passes only with the Foundry principal namespace.
17. Slice M9: add OpenTofu context directories.
18. Slice M9 proof: all six deployment contexts run static validation.
19. Slice M10: add OS manifest.
20. Slice M10 proof: CI matrix maps to the supported OS list and explicit exclusions.
21. Slice M11: add OCI Always Free profile.
22. Slice M11 proof: demo-trial caps are enforced in capacity config.
23. Slice M12: refresh provider pricing and latency references.
24. Slice M12 proof: benchmark report includes source dates and provider model versions.
25. Slice M13: update Foundry absorption prose.
26. Slice M13 proof: PRD, architecture, and microservice ADR agree on `llm-substrate` ownership.
27. Slice M14: remove retired feature-class files or translate them.
28. Slice M14 proof: no retired labels remain outside cited historical migration records.
29. Slice M15: add launch gate summary.
30. Slice M15 proof: every parity claim links to source, test, deployment, and SLO evidence.
