# Intelligence Performance Benchmark Numbers
Date: 2026-05-20
Batch: Wave 3 Batch 3.2
µservice: `intelligence`
Counterparts: OpenAI Platform / Anthropic Claude Platform / Google Vertex AI
Method: current local benchmark corpus plus public counterpart documentation, with explicit source and estimate labels.

## Citation Anchor Block
1. Local benchmark corpus: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-112`.
2. Local capacity corpus: `microservices/intelligence/capacity-model.md:21-98`.
3. Local SLO corpus: `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml:37-45`, `microservices/intelligence/slos/first-token-latency.openslo.yaml:37-45`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml:37-45`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml:37-45`.
4. Public provider docs: `https://openai.com/api/pricing/`, `https://platform.openai.com/docs/guides/tools-file-search`, `https://docs.anthropic.com/en/docs/about-claude/pricing`, `https://docs.anthropic.com/en/api/rate-limits`.
5. Public provider docs and benchmark reference: `https://cloud.google.com/vertex-ai/generative-ai/docs`, `https://cloud.google.com/vertex-ai/generative-ai/docs/provisioned-throughput`, `https://cloud.google.com/vertex-ai/docs/vector-search/overview`, `https://tokenmix.ai/blog/ai-api-latency-benchmark`.

## Methodology Disclosure
1. This document defines benchmark numbers for the intelligence microservice as a target and audit artifact.
2. The numbers are not a production-measurement claim for a completed service because `microservices/intelligence/` currently has no `src/` directory and no executable `tests/` directory.
3. Local benchmark numbers are taken from the existing benchmark document where present.
4. Public provider numbers are cited as public docs or public latency benchmark references where available.
5. Where public provider docs do not publish a single comparable latency or throughput number, this document marks a value as estimated from local benchmark corpus or account-dependent provider behavior.
6. The target set is a single industry-leader-grade set.
7. The target set is not split by retired feature classes.
8. Deployment-context overlays describe infrastructure constraints, not quality downgrades.
9. Tenant-class overlays describe billing, caps, and entitlement constraints, not model-quality downgrades.
10. The three tenant classes are `demo_trial`, `paid`, and `revenue_share`.
11. `demo_trial` is allowed hard usage and time caps, best-effort SLO, no compliance packs, and no BYOK.
12. `paid` is allowed contractual SLOs, compliance packs, BYOK, and scaling by payment.
13. `revenue_share` is allowed at-cost or zero-margin substrate economics tied to revenue-share terms.
14. The canonical six deployment contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
15. The OCI Always Free profile is treated as demo-trial infrastructure for `guest-on-oci`.
16. The local canonical dispatch latency target comes from `dispatch-api-latency.openslo.yaml`.
17. The local first-token target comes from `first-token-latency.openslo.yaml`.
18. The local streaming target comes from `streaming-throughput.openslo.yaml`.
19. The local audit target comes from `audit-emission-success.openslo.yaml`.
20. Provider model names, price points, context limits, and managed-service quotas drift over time.
21. Launch readiness therefore requires a recurring provider-refresh job before external benchmark claims.
22. Workload W1 is chat dispatch with policy routing and audit emission.
23. Workload W2 is first-token latency for a consumer assist-draft request.
24. Workload W3 is streaming throughput after first token.
25. Workload W4 is context-aware retrieval plus model call.
26. Workload W5 is embedding batch throughput.
27. Workload W6 is refusal classification and policy decision.
28. Workload W7 is provider failover after outage or rate-limit saturation.
29. Workload W8 is audit emission and Merkle-seal path.
30. Workload W9 is BYOK credential resolution.
31. Workload W10 is Foundry internal substrate dispatch under `internal-foundry` audience.
32. OS disclosure: target OS coverage is governed by the master plan OS matrix, but the service currently lacks `supported-oses.json`.
33. Architecture disclosure: target architecture assumes Rust backend implementation.
34. Deployment disclosure: target deployment assumes OpenTofu modules for all six contexts, but current service IaC is not yet aligned.

## §1 Methodology
1. Benchmark dimension: dispatch latency, measured at p50, p95, and p99.
2. Benchmark dimension: first-token latency, measured at p50, p95, and p99.
3. Benchmark dimension: streaming throughput, measured as tokens per second at p50 and p99.
4. Benchmark dimension: request throughput, measured as sustained requests per second per cell.
5. Benchmark dimension: concurrent in-flight operations, measured per service cell.
6. Benchmark dimension: retrieval latency, measured as context fetch and rerank latency.
7. Benchmark dimension: embedding throughput, measured as documents per second for normalized chunks.
8. Benchmark dimension: refusal false-negative rate.
9. Benchmark dimension: refusal false-positive rate.
10. Benchmark dimension: audit emission success.
11. Benchmark dimension: provider failover recovery time.
12. Benchmark dimension: BYOK credential resolution latency.
13. Benchmark dimension: per-request cost attribution latency.
14. Benchmark dimension: replay/backfill throughput for audit records.
15. Benchmark dimension: infrastructure footprint under OCI Always Free profile.
16. Test workload W1 uses 1 KB prompt, 1 KB policy context, provider routing, and audit emission.
17. Test workload W2 uses 4 KB prompt plus retrieval citations for assist-draft.
18. Test workload W3 uses a streaming answer of 512 generated tokens.
19. Test workload W4 uses retrieval over a 10 million chunk corpus with filtered tenant corpus selection.
20. Test workload W5 uses 100,000 document chunk embedding batch.
21. Test workload W6 uses regulated-pack refusal and risk classification.
22. Test workload W7 induces provider timeout and validates alternate provider routing.
23. Test workload W8 validates audit write, hash chain update, and event emission.
24. Test workload W9 validates OpenBao handle resolution and no raw secret disclosure.
25. Test workload W10 validates internal Foundry audience without product-policy bypass.
26. OS target: all required OS entries in the master plan need manifest coverage before launch.
27. Architecture target: x86_64 and aarch64 should both be measured where the OS matrix requires them.
28. Deployment-context target: each of the six contexts should publish capacity overlays.
29. Tenant-class target: each benchmark report should include `tenant_class`, but the target quality numbers remain uniform.
30. Source rule: local artifact citations are authoritative for current Oyatie target intent.
31. Source rule: public counterpart documentation is authoritative for provider features and public prices.
32. Source rule: public latency blogs are supporting evidence and must be rechecked before launch claims.
33. Source rule: provider account quotas are account-dependent unless documented as a published limit.
34. Reporting rule: every headline target must state whether Oyatie is ahead, at parity, catching up, or blocked by missing evidence.

## §2 Counterpart Numbers

### §2.1 OpenAI Platform Numbers
1. OpenAI chat latency p50: 1,720 ms in local benchmark corpus for GPT-4o; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
2. OpenAI chat latency p99: 3,512 ms in local benchmark corpus for GPT-4o; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
3. OpenAI embedding throughput: 1,700 docs/sec in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:32-41`.
4. OpenAI RAG end-to-end p50: 2,410 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
5. OpenAI RAG end-to-end p99: 6,890 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
6. OpenAI high-risk classifier accuracy: 93.4 percent in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:65-77`.
7. OpenAI prompt input price reference: model-dependent public pricing; source: `https://openai.com/api/pricing/`.
8. OpenAI file-search/vector-store capability: public feature exists; source: `https://platform.openai.com/docs/guides/tools-file-search`.
9. OpenAI batch cost behavior: public pricing page advertises batch economics by model and workload; source: `https://openai.com/api/pricing/`.
10. OpenAI request quota: account and model dependent; source: estimated from provider account policy, not a universal public number.
11. OpenAI context capacity: model dependent; source: public model documentation and pricing page, not stable as a single platform-wide number.
12. OpenAI streaming throughput: model and account dependent; source: estimated from local benchmark corpus and provider behavior.
13. OpenAI failover recovery: not a native cross-provider feature; source: inferred from product surface.
14. OpenAI BYOK equivalence: not a direct local credential resolver; source: inferred from public platform controls.
15. OpenAI parity note: strong hosted primitives, weaker cross-provider governance by design.

### §2.2 Anthropic Claude Platform Numbers
1. Anthropic chat latency p50: 2,312 ms in local benchmark corpus for Claude 3.5 Sonnet; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
2. Anthropic chat latency p99: 4,824 ms in local benchmark corpus for Claude 3.5 Sonnet; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
3. Anthropic RAG end-to-end p50: 2,840 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
4. Anthropic RAG end-to-end p99: 7,940 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
5. Anthropic high-risk classifier accuracy: 94.1 percent in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:65-77`.
6. Anthropic prompt caching feature: public feature exists; source: `https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching`.
7. Anthropic pricing reference: model-dependent public pricing; source: `https://docs.anthropic.com/en/docs/about-claude/pricing`.
8. Anthropic rate-limit behavior: account and model dependent; source: `https://docs.anthropic.com/en/api/rate-limits`.
9. Anthropic batch support: public feature exists in platform docs; source: public Anthropic documentation.
10. Anthropic request quota: account dependent; source: rate-limit docs.
11. Anthropic streaming throughput: model and load dependent; source: estimated from local benchmark corpus.
12. Anthropic cache-hit latency improvement: workload dependent; source: prompt caching docs.
13. Anthropic failover recovery: not a native cross-provider feature; source: inferred from product surface.
14. Anthropic BYOK equivalence: not a direct local credential resolver; source: inferred from public platform controls.
15. Anthropic parity note: strong safety and long-context economics, not a multi-provider governance substrate.

### §2.3 Google Vertex AI Numbers
1. Vertex chat latency p50: 1,920 ms in local benchmark corpus for Gemini 1.5 Pro; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
2. Vertex chat latency p99: 3,824 ms in local benchmark corpus for Gemini 1.5 Pro; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:17-30`.
3. Vertex RAG end-to-end p50: 2,280 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
4. Vertex RAG end-to-end p99: 6,120 ms in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
5. Vertex high-risk classifier accuracy: 92.7 percent in local benchmark corpus; source: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:65-77`.
6. Vertex generative AI model access: public feature exists; source: `https://cloud.google.com/vertex-ai/generative-ai/docs`.
7. Vertex provisioned throughput: public managed-capacity feature exists; source: `https://cloud.google.com/vertex-ai/generative-ai/docs/provisioned-throughput`.
8. Vertex Vector Search: public managed vector search exists; source: `https://cloud.google.com/vertex-ai/docs/vector-search/overview`.
9. Vertex context caching: public feature exists; source: `https://cloud.google.com/vertex-ai/generative-ai/docs/context-cache/context-cache-overview`.
10. Vertex request quota: project, region, and model dependent; source: Google Cloud quota controls.
11. Vertex streaming throughput: model and region dependent; source: estimated from local benchmark corpus.
12. Vertex failover recovery: cloud-region and customer-architecture dependent; source: inferred from Vertex operational model.
13. Vertex BYOK equivalence: Cloud KMS and IAM-based controls exist; source: inferred from Google Cloud enterprise controls.
14. Vertex latency public benchmark cross-check: public blog data should be treated as supporting evidence; source: `https://tokenmix.ai/blog/ai-api-latency-benchmark`.
15. Vertex parity note: strong managed-cloud controls, not a provider-agnostic substrate across all Oyatie contexts.

## §3 Oyatie Target Numbers — Single Industry-Leader Target Set
1. Target T1 dispatch latency p50: <= 75 ms service overhead.
2. Target T1 dispatch latency p95: <= 150 ms service overhead.
3. Target T1 dispatch latency p99: <= 250 ms service overhead.
4. Target T1 source: local dispatch latency SLO p99 below 250 ms: `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml:37-45`.
5. Target T2 first-token latency p50: <= 1.2 seconds for consumer assist-draft when provider is healthy.
6. Target T2 first-token latency p95: <= 1.6 seconds for consumer assist-draft when provider is healthy.
7. Target T2 first-token latency p99: <= 2.0 seconds for consumer assist-draft when provider is healthy.
8. Target T2 source: local first-token SLO p99 below 2.0 seconds: `microservices/intelligence/slos/first-token-latency.openslo.yaml:37-45`.
9. Target T3 streaming throughput p50: >= 45 tokens/sec after first token.
10. Target T3 streaming throughput p99: >= 30 tokens/sec after first token.
11. Target T3 source: local streaming SLO p99 at or above 30 tokens/sec: `microservices/intelligence/slos/streaming-throughput.openslo.yaml:37-45`.
12. Target T4 dispatch availability: >= 99.95 percent monthly.
13. Target T4 source: local dispatch availability SLO: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml:17-20`.
14. Target T5 audit emission success: >= 99.99 percent.
15. Target T5 source: local audit SLO: `microservices/intelligence/slos/audit-emission-success.openslo.yaml:37-45`.
16. Target T6 refusal false-negative rate: <= 0.1 percent.
17. Target T6 source: local refusal SLO: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml:23-59`.
18. Target T7 refusal false-positive rate: <= 2.0 percent.
19. Target T7 source: local refusal SLO: `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml:23-58`.
20. Target T8 embedding throughput: >= 8,200 docs/sec for self-hosted embedding path when hardware matches local benchmark assumptions.
21. Target T8 source: local benchmark self-hosted embedding number: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:32-41`.
22. Target T9 RAG end-to-end p50: <= 2.25 seconds for standard retrieval-augmented assist response.
23. Target T9 RAG end-to-end p99: <= 6.0 seconds for standard retrieval-augmented assist response.
24. Target T9 source: local benchmark comparison against OpenAI, Anthropic, and Vertex RAG rows: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:43-52`.
25. Target T10 high-risk classifier accuracy: >= 95 percent.
26. Target T10 source: local benchmark target compared to counterpart rows: `microservices/intelligence/benchmarks/openai-anthropic-vertex-vs-oyatie.md:65-77`.
27. Target T11 provider outage recovery time: <= 5 minutes for non-regulated failover where alternate provider is policy-eligible.
28. Target T11 source: failure-mode and runbook corpus: `microservices/intelligence/failure-modes.md:27-49`, `microservices/intelligence/runbooks/provider-outage-openai.md`.
29. Target T12 provider rate-limit saturation detection: <= 60 seconds from sustained saturation.
30. Target T12 source: capacity model and rate-limit runbook: `microservices/intelligence/capacity-model.md:21-46`, `microservices/intelligence/runbooks/provider-rate-limit-saturation.md`.
31. Target T13 BYOK credential resolution p99: <= 100 ms for cached OpenBao handle path.
32. Target T13 source: architecture deployment shape and BYOK policy: `microservices/intelligence/ARCHITECTURE.md:878-886`, `microservices/intelligence/policy/byok-gating.cedar:20-92`.
33. Target T14 cost attribution emission p99: <= 500 ms after dispatch completion.
34. Target T14 source: cost record schema and dashboards: `microservices/intelligence/cost-budget.md:57-79`, `microservices/intelligence/dashboards/finops-cost-attribution.md`.
35. Target T15 audit replay throughput: >= 10,000 audit rows/minute per worker for backfill jobs.
36. Target T15 source: backfill target inferred from `microservices/intelligence/backfill-replay.md`; implementation measurement still required.
37. Target T16 concurrent in-flight dispatch operations per standard cell: >= 2,000 when provider quotas allow.
38. Target T16 source: capacity model concurrency and provider budget assumptions: `microservices/intelligence/capacity-model.md:21-98`.
39. Target T17 regulated-pack policy decision p99: <= 50 ms for Cedar evaluation after facts are loaded.
40. Target T17 source: policy corpus and refusal SLOs: `microservices/intelligence/policy/provider-routing.cedar:20-175`, `microservices/intelligence/policy/refusal-baseline.cedar`.
41. Target T18 internal Foundry substrate dispatch overhead p99: <= 250 ms before provider latency.
42. Target T18 source: internal-foundry audience contract and policy: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml:52-64`, `microservices/intelligence/policy/dispatch-authorization.cedar:41-57`.

### §3.1 Deployment-Context Overlays
1. Overlay `oyatie-public-cloud`: full target set applies when deployed on Oyatie-managed elastic substrate.
2. Overlay `oyatie-public-cloud`: provider quota exhaustion remains a provider/account constraint and should trigger failover or throttling.
3. Overlay `guest-on-aws`: full target set applies only when customer AWS substrate meets CPU, memory, network, OpenBao, NATS, Valkey, and egress requirements.
4. Overlay `guest-on-aws`: context module must expose provider egress, private networking, policy sidecars, and observability before launch claim.
5. Overlay `guest-on-oci`: full target set applies on sized customer OCI tenancy.
6. Overlay `guest-on-oci`: OCI Always Free profile caps demo-trial tenants to the free compute and memory envelope defined by the master plan.
7. Overlay `guest-on-oci`: demo-trial OCI profile should cap sustained request throughput rather than lower model-quality targets.
8. Overlay `on-prem`: target set applies when customer hardware and network are certified by the OS and capacity manifest.
9. Overlay `on-prem`: provider egress latency and facility routing can dominate first-token latency.
10. Overlay `colo`: target set applies when colo networking, HSM/OpenBao path, and provider egress are certified.
11. Overlay `colo`: failover time depends on provider route availability and facility peering.
12. Overlay `oyatie-as-cloud-provider`: full target set applies with Oyatie-owned cloud substrate and internal cell operations.
13. Overlay all contexts: p99 service overhead target remains 250 ms, but provider latency remains external.
14. Overlay all contexts: audit emission success target remains 99.99 percent.
15. Overlay all contexts: refusal false-negative and false-positive targets remain unchanged.
16. Overlay all contexts: tenant-class caps are commercial and infrastructure limits, not quality limits.
17. Overlay all contexts: OpenTofu module evidence is required before the overlay can be claimed deployable.
18. Overlay all contexts: `supported-oses.json` is required before OS coverage can be claimed.

### §3.2 Tenant-Class Overlays
1. Overlay `demo_trial`: quality target set remains the same as paid service behavior.
2. Overlay `demo_trial`: usage caps may limit requests per minute, tokens per day, file count, corpus size, and total runtime.
3. Overlay `demo_trial`: OCI Always Free profile may cap sustained throughput to fit 4 OCPU and 24 GB memory envelope from the master plan.
4. Overlay `demo_trial`: best-effort SLO means no contractual remedy, not lower service correctness.
5. Overlay `demo_trial`: compliance packs are unavailable.
6. Overlay `demo_trial`: BYOK is unavailable.
7. Overlay `paid`: full target set applies subject to purchased capacity and provider quotas.
8. Overlay `paid`: contractual SLOs are allowed.
9. Overlay `paid`: compliance packs are allowed.
10. Overlay `paid`: BYOK is allowed.
11. Overlay `paid`: scale ceiling increases with paid capacity and customer deployment context.
12. Overlay `revenue_share`: full target set applies subject to contract capacity.
13. Overlay `revenue_share`: substrate is at-cost or zero-margin by commercial design.
14. Overlay `revenue_share`: revenue-linked usage must still emit cost attribution and audit records.
15. Overlay `revenue_share`: compliance packs and BYOK availability should follow contract and regulatory scope, not reduced quality.
16. Overlay all tenant classes: no model-quality stratification is allowed.
17. Overlay all tenant classes: policy refusal correctness targets are uniform.
18. Overlay all tenant classes: audit emission targets are uniform.
19. Overlay all tenant classes: deployment constraints should be disclosed as capacity caps.
20. Overlay all tenant classes: benchmark reports must record tenant_class for interpretation.

## §4 Comparison Narrative
1. Headline dispatch overhead: Oyatie target p99 <= 250 ms is ahead of a raw app-side wrapper target and comparable to a well-built policy gateway.
2. Headline dispatch overhead status: blocked by missing source implementation and tests.
3. Headline first-token latency: Oyatie p99 <= 2.0 seconds is ahead of local OpenAI, Anthropic, and Vertex p99 chat rows when provider is healthy.
4. Headline first-token latency status: catch-up because provider latency dominates and current numbers are target posture.
5. Headline streaming throughput: Oyatie p99 >= 30 tokens/sec is a measurable parity target with modern streaming platforms.
6. Headline streaming throughput status: blocked by missing streaming transport implementation.
7. Headline embedding throughput: Oyatie target >= 8,200 docs/sec is ahead of local OpenAI embedding row.
8. Headline embedding throughput status: conditional on self-hosted embedding hardware and corpus pipeline implementation.
9. Headline RAG p50: Oyatie target <= 2.25 seconds is ahead of local OpenAI and Anthropic rows and near Vertex local row.
10. Headline RAG p50 status: catch-up because source implementation and retrieval corpus execution are absent.
11. Headline RAG p99: Oyatie target <= 6.0 seconds is ahead of local OpenAI and Anthropic p99 rows and slightly ahead of local Vertex p99 row.
12. Headline RAG p99 status: blocked by missing retrieval source, vector index ownership, and tests.
13. Headline high-risk classifier accuracy: Oyatie target >= 95 percent is ahead of local counterpart rows.
14. Headline high-risk classifier status: catch-up because classifier implementation and eval runner are absent.
15. Headline audit emission: Oyatie target >= 99.99 percent is differentiated versus generic provider usage logs.
16. Headline audit emission status: blocked by missing audit tap implementation.
17. Headline failover recovery: Oyatie target <= 5 minutes is ahead of single-provider surfaces because it can route across providers.
18. Headline failover status: conditional on provider adapters, policy eligibility, and rate-limit telemetry.
19. Headline cost attribution: Oyatie target p99 <= 500 ms after dispatch is differentiated because it attaches cost to tenant, provider, model, and policy context.
20. Headline cost attribution status: partial design, missing tenant_class and source implementation.
21. Headline BYOK resolution: Oyatie p99 <= 100 ms is a concrete enterprise target.
22. Headline BYOK status: partial policy, missing implementation and tenant-class allowance.
23. Headline deployment coverage: Oyatie target all six contexts is ahead of single-provider platforms in portability.
24. Headline deployment coverage status: behind because canonical OpenTofu context directories are absent.
25. Headline OS coverage: Oyatie target broad OS matrix is ahead of hosted-only provider surfaces.
26. Headline OS coverage status: behind because the service lacks `supported-oses.json`.
27. Headline OCI Always Free profile: Oyatie target gives demo-trial infrastructure economics without lowering quality.
28. Headline OCI Always Free profile status: behind because profile directory is absent.
29. Headline Foundry absorption: Oyatie target integrates internal `llm-substrate` consumption under the same policy substrate.
30. Headline Foundry absorption status: partial because architecture and policy exist, but owner-transfer prose is incomplete.
31. Overall comparison: Oyatie can target industry-leader parity only if the missing implementation and deployment evidence is created.
32. Overall comparison: current docs are strong enough to guide implementation, not strong enough to prove live parity.
33. Overall comparison: the strongest differentiator is governed multi-provider routing with audit and compliance controls.
34. Overall comparison: the weakest evidence area is deployable runtime proof across the six contexts.
35. Overall comparison: no benchmark should be published externally without source implementation, test harnesses, and current provider refresh.

## §5 Benchmark Acceptance Gates
1. Gate G1: Rust dispatch source exists and compiles.
2. Gate G2: provider adapter contract tests exist for OpenAI, Anthropic, and Vertex paths.
3. Gate G3: dispatch policy tests cover region, pack, tenant, audience, and credential mode.
4. Gate G4: audit emission tests prove event write and hash-chain update.
5. Gate G5: refusal-quality tests run the canonicalen set and publish false-positive and false-negative rates.
6. Gate G6: retrieval tests run tenant-filtered corpus fetch and citation attribution.
7. Gate G7: streaming tests measure first token and tokens/sec.
8. Gate G8: rate-limit tests simulate provider saturation.
9. Gate G9: failover tests validate alternate eligible provider selection.
10. Gate G10: BYOK tests validate OpenBao handle resolution without raw secret exposure.
11. Gate G11: cost attribution tests emit per-request cost records.
12. Gate G12: OpenTofu modules exist for all six deployment contexts.
13. Gate G13: OCI Always Free profile exists for demo-trial infrastructure.
14. Gate G14: `supported-oses.json` exists and CI lanes map to the master plan OS matrix.
15. Gate G15: benchmark report records deployment context, tenant_class, provider, model, OS, arch, and workload.
16. Gate G16: provider prices and model names are refreshed from public docs before release.
17. Gate G17: public latency references are refreshed and source-dated before external claims.
18. Gate G18: Foundry internal dispatch benchmark runs under `internal-foundry` audience.
19. Gate G19: demo-trial caps are enforced as usage limits.
20. Gate G20: paid and revenue-share tenants demonstrate the same quality target set under appropriate capacity.

## §6 Final Benchmark Position
1. OpenAI comparison: Oyatie targets lower dispatch overhead and richer governance, but lacks implementation proof.
2. Anthropic comparison: Oyatie targets comparable streaming and stronger cross-provider governance, but lacks prompt-cache contract fields.
3. Vertex comparison: Oyatie targets broader deployment portability, but lacks managed-cloud throughput implementation.
4. Current confidence in local target numbers: medium for SLO-derived targets.
5. Current confidence in local benchmark numbers: medium-low until rerun on current provider models.
6. Current confidence in deployment overlays: low until OpenTofu context modules exist.
7. Current confidence in tenant-class overlays: low until tenant_class is added to contracts and policy.
8. Current confidence in Foundry absorption benchmark: medium-low because policy hooks exist but owner-transfer language is incomplete.
9. Launch recommendation: use these numbers as implementation gates, not as public production claims.
10. Audit recommendation: fix IaC, OS manifest, tenant_class, implementation source, executable tests, and Foundry transfer language before claiming industry-leader parity.

## §7 Provider Refresh And Measurement Cadence
1. Refresh R1: provider model names must be refreshed before each benchmark publication.
2. Refresh R2: provider price pages must be refreshed before each cost comparison.
3. Refresh R3: public latency references must be refreshed before external claims.
4. Refresh R4: local benchmark runs must record provider, model, deployment context, tenant_class, OS, architecture, workload, and timestamp.
5. Refresh R5: latency runs must include p50, p95, and p99.
6. Refresh R6: throughput runs must include warm-up duration and sustained measurement window.
7. Refresh R7: refusal runs must include corpus version and policy revision.
8. Refresh R8: audit runs must include event schema version and hash-chain verifier version.
9. Refresh R9: BYOK runs must include OpenBao policy version and credential-handle cache state.
10. Refresh R10: Foundry internal runs must include the principal namespace and `internal-foundry` audience.
11. Refresh R11: demo-trial runs must include OCI Always Free profile limits when that context is used.
12. Refresh R12: paid runs must include purchased capacity and provider quota facts.
13. Refresh R13: revenue-share runs must include revenue-linked capacity contract facts without lowering quality targets.
14. Refresh R14: failed runs must preserve raw measurements and error categories.
15. Refresh R15: launch report must separate target numbers, measured numbers, and provider-published numbers.
16. Refresh R16: every public number should carry a source URL or local file citation.
17. Refresh R17: every estimated number should be labeled as estimated from a named source.
18. Refresh R18: every deployment-context cap should point to an OpenTofu variable or capacity manifest.
19. Refresh R19: every OS-specific result should point to `supported-oses.json`.
20. Refresh R20: no benchmark should imply a feature-quality split between tenant classes.
21. Refresh R21: benchmark dashboards should alert on provider price drift.
22. Refresh R22: benchmark dashboards should alert on p99 regression before customer SLO breach.
23. Refresh R23: benchmark dashboards should alert on audit emission misses immediately.
24. Refresh R24: benchmark dashboards should alert on refusal false-negative movement before release.
25. Refresh R25: benchmark dashboards should alert when demo-trial caps are reached without treating the cap as a quality failure.
