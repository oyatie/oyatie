# Wave-3 Final Corpus Completeness Scorecard — 2026-05-20

Audit-only artifact for Wave-3-G plus remediation. This file is intentionally a bespoke corpus scorecard, not a source-of-truth replacement.

Evidence basis: local working tree under `/Users/jasonlee/oyatie`; root hub pointers, docs, specs, registry, microservice folders, packs, contracts, crates, tests, and benchmarks were scanned with build-output directories excluded.
Calendar note: the live working tree contains several 2026-05-21 artifacts; they are treated as present corpus evidence, not as a claim about wall-clock date.

Final verdict preview: NEEDS-WAVE-4. The corpus is broad and substantially remediated, but numeric completeness gates still show named gaps in ADR continuity, service/tier registry parity, per-service test and threat coverage, and CI lane implementation depth.

## §1 Methodology + Workstream Definitions
- Audit mode: read-only source audit plus one authored scorecard file
- Corpus roots scanned: docs, specs, registry, microservices, packs, regional-packs, contracts, crates, tests, benchmarks
- Text files read: 15984
- Readable text files: 15982
- Skipped/decoded notes: 2
- ADR expected range: ADR-0001 through ADR-0321, plus recursively discovered ADR-MS-* files
- Microservice expected source: live `microservices/*` directories plus `specs/microservices/*.json` overlays
- Journey expected source: `docs/user-journeys/j01..j180` live journey directories
- Persona expected source: `docs/personas/*.md` excluding README/master roster/index control files
- Compliance expected source: 8 registry compliance packs plus localization/regional packs
- Scoring dimensions: presence, depth, cross-reference density, policy/contract/test/threat evidence, and registry conformance
- Cross-reference density: count of explicit ADR, microservice, registry/spec, journey, persona, compliance, policy, and contract anchors
- Depth grading: A-rigorous >=120 lines with broad markers and cross refs; B-complete >=80 lines; C-partial >=40 lines; D-thin >0; F-missing
- Named-gap policy: every incomplete row names the concrete missing evidence rather than using generic follow-up language
- Stop condition: scorecard file >=3000 lines, lifecycle evidence commands run, no source file edits beyond this scorecard

Workstreams defined for this audit: ADR decisions, microservice specifications, journeys, personas, compliance packs, standards, capability-tier registry, Foundry pipeline specs, CI gate crates, customer-facing material, tests/benchmarks, cumulative remediation line estimate, Wave-4 priority queue, and final verdict.

## §2 ADR Coverage Scorecard (ADR-0001..ADR-0321 + ADR-MS-*)
- ADR numeric files discovered: 264
- ADR numeric expected: 321
- ADR numeric missing count: 57
- ADR-MS files discovered: 8
- Primary named gap: ADR sequence is not fully materialized through ADR-0321
### ADR-0001 — ADR-0001-cohesion-thesis-one-product-flat-catalog.md
- status: accepted
- depth: A-rigorous; lines=150; substance_markers=9
- cross-ref-density: 38 refs / 150 lines = 25.33 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: verification evidence absent
### ADR-0002 — ADR-0002-tenant-and-identity-kernel.md
- status: proposed
- depth: A-rigorous; lines=170; substance_markers=12
- cross-ref-density: 42 refs / 170 lines = 24.71 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0003 — ADR-0003-audit-chain-and-evidence-emission.md
- status: proposed
- depth: A-rigorous; lines=152; substance_markers=9
- cross-ref-density: 39 refs / 152 lines = 25.66 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0004 — ADR-0004-plane-separation-control-data-analytics.md
- status: proposed
- depth: A-rigorous; lines=165; substance_markers=9
- cross-ref-density: 34 refs / 165 lines = 20.61 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0005 — ADR-0005-eventing-backbone-outbox-pattern.md
- status: proposed
- depth: A-rigorous; lines=170; substance_markers=8
- cross-ref-density: 41 refs / 170 lines = 24.12 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0006 — ADR-0006-ontology-typed-entity-layer.md
- status: accepted
- depth: B-complete; lines=130; substance_markers=6
- cross-ref-density: 42 refs / 130 lines = 32.31 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0007 — ADR-0007-cedar-authorization-policy-and-persona-tier.md
- status: proposed
- depth: A-rigorous; lines=202; substance_markers=9
- cross-ref-density: 87 refs / 202 lines = 43.07 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0008 — ADR-0008-data-use-boundary.md
- status: accepted
- depth: A-rigorous; lines=215; substance_markers=11
- cross-ref-density: 47 refs / 215 lines = 21.86 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0009 — ADR-0009-cell-architecture-per-tenant-per-region.md
- status: proposed
- depth: A-rigorous; lines=159; substance_markers=9
- cross-ref-density: 46 refs / 159 lines = 28.93 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0010 — ADR-0010-regional-pack-architecture.md
- status: proposed
- depth: A-rigorous; lines=187; substance_markers=11
- cross-ref-density: 39 refs / 187 lines = 20.86 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0011 — ADR-0011-cross-microservice-contract-registry.md
- status: accepted
- depth: A-rigorous; lines=159; substance_markers=8
- cross-ref-density: 88 refs / 159 lines = 55.35 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0012
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0013 — ADR-0013-product-license-policy.md
- status: proposed
- depth: B-complete; lines=184; substance_markers=5
- cross-ref-density: 26 refs / 184 lines = 14.13 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0014 — ADR-0014-build-vs-buy-policy.md
- status: proposed
- depth: A-rigorous; lines=189; substance_markers=8
- cross-ref-density: 45 refs / 189 lines = 23.81 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0015 — ADR-0015-architectural-flattening-target.md
- status: accepted
- depth: A-rigorous; lines=200; substance_markers=10
- cross-ref-density: 54 refs / 200 lines = 27.0 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0016 — ADR-0016-wave-and-plane-integration-framework.md
- status: proposed
- depth: A-rigorous; lines=164; substance_markers=8
- cross-ref-density: 43 refs / 164 lines = 26.22 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0017 — ADR-0017-brand-naming-and-repo-layout.md
- status: accepted
- depth: B-complete; lines=120; substance_markers=5
- cross-ref-density: 16 refs / 120 lines = 13.33 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0018 — ADR-0018-glossary-and-terminology-canon.md
- status: accepted
- depth: B-complete; lines=118; substance_markers=6
- cross-ref-density: 30 refs / 118 lines = 25.42 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0019 — ADR-0019-doc-catalog-and-update-protocol.md
- status: proposed
- depth: A-rigorous; lines=179; substance_markers=9
- cross-ref-density: 48 refs / 179 lines = 26.82 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: verification evidence absent
### ADR-0020 — ADR-0020-intelligence-multi-provider-adapter-model.md
- status: proposed
- depth: A-rigorous; lines=176; substance_markers=7
- cross-ref-density: 26 refs / 176 lines = 14.77 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0021 — ADR-0021-intelligence-capability-registry-and-mcp-gateway.md
- status: proposed
- depth: A-rigorous; lines=145; substance_markers=11
- cross-ref-density: 32 refs / 145 lines = 22.07 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0022 — ADR-0022-autonomy-ceiling-runtime-enforcement.md
- status: proposed
- depth: A-rigorous; lines=176; substance_markers=9
- cross-ref-density: 67 refs / 176 lines = 38.07 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0023 — ADR-0023-intelligence-sandbox-wasmtime-firecracker.md
- status: proposed
- depth: A-rigorous; lines=160; substance_markers=8
- cross-ref-density: 18 refs / 160 lines = 11.25 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0024 — ADR-0024-intelligence-eval-harness-and-replay.md
- status: proposed
- depth: A-rigorous; lines=159; substance_markers=7
- cross-ref-density: 23 refs / 159 lines = 14.47 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0025 — ADR-0025-intelligence-as-engineering-platform.md
- status: proposed
- depth: A-rigorous; lines=167; substance_markers=9
- cross-ref-density: 23 refs / 167 lines = 13.77 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0026 — ADR-0026-in-house-ai-model-substrate-roadmap.md
- status: proposed
- depth: B-complete; lines=162; substance_markers=5
- cross-ref-density: 19 refs / 162 lines = 11.73 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0027 — ADR-0027-robotics-vision-speech-sub-substrates.md
- status: proposed
- depth: A-rigorous; lines=219; substance_markers=9
- cross-ref-density: 28 refs / 219 lines = 12.79 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0028 — ADR-0028-cloud-microservice-architecture.md
- status: accepted
- depth: A-rigorous; lines=147; substance_markers=7
- cross-ref-density: 40 refs / 147 lines = 27.21 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: verification evidence absent
### ADR-0029 — ADR-0029-connect-dual-context-architecture.md
- status: accepted
- depth: B-complete; lines=176; substance_markers=5
- cross-ref-density: 48 refs / 176 lines = 27.27 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0030 — ADR-0030-search-microservice-architecture.md
- status: accepted
- depth: B-complete; lines=155; substance_markers=6
- cross-ref-density: 35 refs / 155 lines = 22.58 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0031 — ADR-0031-ads-and-analytics-microservice-architecture.md
- status: accepted
- depth: B-complete; lines=148; substance_markers=5
- cross-ref-density: 38 refs / 148 lines = 25.68 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0032 — ADR-0032-dcim-software-for-own-dc-ops.md
- status: proposed
- depth: A-rigorous; lines=214; substance_markers=11
- cross-ref-density: 49 refs / 214 lines = 22.9 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0033
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0034 — ADR-0034-per-microservice-data-class-overrides.md
- status: accepted
- depth: B-complete; lines=163; substance_markers=6
- cross-ref-density: 41 refs / 163 lines = 25.15 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0035 — ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
- status: proposed
- depth: A-rigorous; lines=183; substance_markers=11
- cross-ref-density: 58 refs / 183 lines = 31.69 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0036 — ADR-0036-plugin-substrate-wasm-and-trust.md
- status: proposed
- depth: A-rigorous; lines=227; substance_markers=10
- cross-ref-density: 48 refs / 227 lines = 21.15 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0037 — ADR-0037-public-api-stability-tiers-and-deprecation.md
- status: proposed
- depth: B-complete; lines=249; substance_markers=6
- cross-ref-density: 53 refs / 249 lines = 21.29 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md
- named gaps: verification evidence absent
### ADR-0038 — ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md
- status: proposed
- depth: A-rigorous; lines=226; substance_markers=7
- cross-ref-density: 67 refs / 226 lines = 29.65 per 100 lines
- artifact: `docs/decisions/ADR-0703-cas-cache-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0039 — ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md
- status: proposed
- depth: A-rigorous; lines=235; substance_markers=7
- cross-ref-density: 47 refs / 235 lines = 20.0 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0040 — ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md
- status: proposed
- depth: A-rigorous; lines=222; substance_markers=8
- cross-ref-density: 39 refs / 222 lines = 17.57 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0041 — ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md
- status: proposed
- depth: B-complete; lines=232; substance_markers=5
- cross-ref-density: 47 refs / 232 lines = 20.26 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md
- named gaps: verification evidence absent
### ADR-0042 — ADR-0042-observability-stack-otel-and-in-house-ui.md
- status: proposed
- depth: A-rigorous; lines=234; substance_markers=8
- cross-ref-density: 52 refs / 234 lines = 22.22 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0043 — ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
- status: proposed
- depth: A-rigorous; lines=219; substance_markers=9
- cross-ref-density: 53 refs / 219 lines = 24.2 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: verification evidence absent
### ADR-0044 — ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md
- status: proposed
- depth: A-rigorous; lines=220; substance_markers=8
- cross-ref-density: 68 refs / 220 lines = 30.91 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0045 — ADR-0045-database-tier-strategy.md
- status: proposed
- depth: A-rigorous; lines=208; substance_markers=7
- cross-ref-density: 48 refs / 208 lines = 23.08 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0046 — ADR-0046-vector-store-strategy.md
- status: proposed
- depth: B-complete; lines=218; substance_markers=6
- cross-ref-density: 33 refs / 218 lines = 15.14 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0047 — ADR-0047-search-backend-strategy.md
- status: proposed
- depth: B-complete; lines=220; substance_markers=5
- cross-ref-density: 41 refs / 220 lines = 18.64 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: verification evidence absent
### ADR-0048 — ADR-0048-korean-morphology-and-multilingual-tokenization.md
- status: proposed
- depth: B-complete; lines=230; substance_markers=6
- cross-ref-density: 29 refs / 230 lines = 12.61 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0049 — ADR-0049-cross-region-replication-and-residency.md
- status: proposed
- depth: A-rigorous; lines=232; substance_markers=9
- cross-ref-density: 57 refs / 232 lines = 24.57 per 100 lines
- artifact: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0050 — ADR-0050-automation-first-pipeline.md
- status: proposed
- depth: A-rigorous; lines=259; substance_markers=10
- cross-ref-density: 54 refs / 259 lines = 20.85 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md
- named gaps: none named; retain in regression audit
### ADR-0051 — ADR-0051-mobile-and-native-client-strategy.md
- status: accepted
- depth: B-complete; lines=107; substance_markers=7
- cross-ref-density: 40 refs / 107 lines = 37.38 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0052 — ADR-0052-inventory-grit-cutover.md
- status: Superseded
- depth: A-rigorous; lines=582; substance_markers=15
- cross-ref-density: 68 refs / 582 lines = 11.68 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0053 — ADR-0053-grit-icm-as-sanctioned-primitives.md
- status: Accepted
- depth: A-rigorous; lines=142; substance_markers=7
- cross-ref-density: 15 refs / 142 lines = 10.56 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0054 — ADR-0054-grit-scaffold-claim-pattern.md
- status: deprecated
- depth: B-complete; lines=276; substance_markers=5
- cross-ref-density: 22 refs / 276 lines = 7.97 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0055 — ADR-0055-branch-pipeline.md
- status: Accepted
- depth: C-partial; lines=172; substance_markers=4
- cross-ref-density: 27 refs / 172 lines = 15.7 per 100 lines
- artifact: `docs/advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md`
- named gaps: none named; retain in regression audit
### ADR-0056 — ADR-0056-rust-clean-architecture-bnf.md
- status: Accepted
- depth: B-complete; lines=326; substance_markers=6
- cross-ref-density: 24 refs / 326 lines = 7.36 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0057 — ADR-0057-cutover-mechanics-rename-plan-v4.md
- status: Accepted
- depth: B-complete; lines=195; substance_markers=6
- cross-ref-density: 24 refs / 195 lines = 12.31 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0058 — ADR-0058-flat-microservice-catalog.md
- status: accepted
- depth: B-complete; lines=176; substance_markers=6
- cross-ref-density: 19 refs / 176 lines = 10.8 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0059 — ADR-0059-workflow-ontology-ecosystem-adapter-layer.md
- status: accepted
- depth: C-partial; lines=157; substance_markers=4
- cross-ref-density: 30 refs / 157 lines = 19.11 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0060 — ADR-0060-bominal-inheritance-precedence.md
- status: accepted
- depth: A-rigorous; lines=138; substance_markers=9
- cross-ref-density: 75 refs / 138 lines = 54.35 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0061 — ADR-0061-application-b2b-unified-shell.md
- status: accepted
- depth: A-rigorous; lines=150; substance_markers=7
- cross-ref-density: 50 refs / 150 lines = 33.33 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0062 — ADR-0062-quality-performance-scalability-bar.md
- status: accepted
- depth: A-rigorous; lines=204; substance_markers=9
- cross-ref-density: 52 refs / 204 lines = 25.49 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0063 — ADR-0063-documentation-set-coverage.md
- status: accepted
- depth: A-rigorous; lines=216; substance_markers=8
- cross-ref-density: 28 refs / 216 lines = 12.96 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0064 — ADR-0064-canonical-base-and-localization-packs.md
- status: accepted
- depth: A-rigorous; lines=237; substance_markers=7
- cross-ref-density: 41 refs / 237 lines = 17.3 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0065 — ADR-0065-docs-as-leptos-webapp-with-machine-readable-coemit.md
- status: accepted
- depth: A-rigorous; lines=308; substance_markers=9
- cross-ref-density: 40 refs / 308 lines = 12.99 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0066 — ADR-0066-live-code-introspection-docs-portal.md
- status: accepted
- depth: A-rigorous; lines=256; substance_markers=9
- cross-ref-density: 41 refs / 256 lines = 16.02 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0067 — ADR-0067-ops-oyatie-com-hyperscaler-operations-console.md
- status: accepted
- depth: A-rigorous; lines=214; substance_markers=11
- cross-ref-density: 100 refs / 214 lines = 46.73 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0068
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0069 — ADR-0069-active-machine-readable-artifact-contract.md
- status: accepted
- depth: A-rigorous; lines=175; substance_markers=8
- cross-ref-density: 61 refs / 175 lines = 34.86 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0070
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0071
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0072
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0073
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0074
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0075
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0076
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0077
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0078
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0079
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0080
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0081
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0082
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0083 — ADR-0083-rust-error-handling-tier-decision.md
- status: Accepted
- depth: B-complete; lines=266; substance_markers=5
- cross-ref-density: 24 refs / 266 lines = 9.02 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0084
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0085
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0086
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0087
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0088
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0089
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0090 — ADR-0090-hyper-canonical-http-backbone.md
- status: accepted
- depth: C-partial; lines=130; substance_markers=4
- cross-ref-density: 9 refs / 130 lines = 6.92 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0091 — ADR-0091-governance-write-gate-foundations.md
- status: accepted
- depth: B-complete; lines=116; substance_markers=6
- cross-ref-density: 10 refs / 116 lines = 8.62 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0092 — ADR-0092-workspace-dependency-seam-policy.md
- status: accepted
- depth: A-rigorous; lines=351; substance_markers=8
- cross-ref-density: 54 refs / 351 lines = 15.38 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0093 — ADR-0093-latency-budget-reporter-rename.md
- status: accepted
- depth: C-partial; lines=73; substance_markers=3
- cross-ref-density: 5 refs / 73 lines = 6.85 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0094 — ADR-0094-handler-trait-with-associated-error.md
- status: accepted
- depth: C-partial; lines=125; substance_markers=4
- cross-ref-density: 7 refs / 125 lines = 5.6 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0095 — ADR-0095-tenant-slug-in-tenancy-kernel.md
- status: accepted
- depth: C-partial; lines=142; substance_markers=4
- cross-ref-density: 9 refs / 142 lines = 6.34 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0096 — ADR-0096-supervisor-language-rust-not-node.md
- status: accepted
- depth: B-complete; lines=141; substance_markers=6
- cross-ref-density: 22 refs / 141 lines = 15.6 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0097 — ADR-0097-intelligence-account-adapter-rename-target-slot-last.md
- status: accepted
- depth: D-thin; lines=156; substance_markers=1
- cross-ref-density: 11 refs / 156 lines = 7.05 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0098 — ADR-0098-supervisor-dep-policy-Y-zero-deps-best-effort-durability.md
- status: accepted
- depth: C-partial; lines=209; substance_markers=4
- cross-ref-density: 20 refs / 209 lines = 9.57 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0099 — ADR-0099-cedar-policy-extend-supervisor-capabilities.md
- status: accepted
- depth: A-rigorous; lines=253; substance_markers=8
- cross-ref-density: 102 refs / 253 lines = 40.32 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0100 — ADR-0100-supervisor-public-contract-lean-a10.md
- status: Accepted
- depth: D-thin; lines=34; substance_markers=3
- cross-ref-density: 6 refs / 34 lines = 17.65 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: thin rationale body; verification evidence absent
### ADR-0101 — ADR-0101-supervisor-mountpoint-direct-hyper.md
- status: Accepted
- depth: D-thin; lines=28; substance_markers=1
- cross-ref-density: 2 refs / 28 lines = 7.14 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: thin rationale body; low cross-reference density; alternatives/rejections not explicit; verification evidence absent
### ADR-0102 — ADR-0102-intelligence-settings-template-canonical-rendering.md
- status: Accepted
- depth: D-thin; lines=33; substance_markers=1
- cross-ref-density: 2 refs / 33 lines = 6.06 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: thin rationale body; low cross-reference density; alternatives/rejections not explicit; verification evidence absent
### ADR-0103 — ADR-0103-grit-cutover-inventory.md
- status: Accepted
- depth: C-partial; lines=66; substance_markers=4
- cross-ref-density: 11 refs / 66 lines = 16.67 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0104 — ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
- status: Accepted
- depth: D-thin; lines=133; substance_markers=2
- cross-ref-density: 16 refs / 133 lines = 12.03 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0105 — ADR-0105-13-layer-enum-and-check-family-patterns.md
- status: Accepted
- depth: A-rigorous; lines=187; substance_markers=7
- cross-ref-density: 43 refs / 187 lines = 22.99 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0106 — ADR-0106-rename-application-to-usecase.md
- status: Accepted
- depth: C-partial; lines=111; substance_markers=3
- cross-ref-density: 22 refs / 111 lines = 19.82 per 100 lines
- artifact: `docs/decisions/ADR-0703-cas-cache-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0107 — ADR-0107-tools-implicit-app-convention.md
- status: Superseded
- depth: C-partial; lines=177; substance_markers=4
- cross-ref-density: 59 refs / 177 lines = 33.33 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0108 — ADR-0108-sunset-lifecycle-automation.md
- status: Accepted
- depth: B-complete; lines=263; substance_markers=5
- cross-ref-density: 54 refs / 263 lines = 20.53 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0109 — ADR-0109-lifecycle-automation-framework.md
- status: Accepted
- depth: B-complete; lines=280; substance_markers=6
- cross-ref-density: 27 refs / 280 lines = 9.64 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0110 — ADR-0110-changeset-state-machine.md
- status: Proposed
- depth: C-partial; lines=245; substance_markers=4
- cross-ref-density: 25 refs / 245 lines = 10.2 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0111 — ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
- status: Proposed
- depth: C-partial; lines=212; substance_markers=4
- cross-ref-density: 10 refs / 212 lines = 4.72 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0112 — ADR-0112-webhook-driven-intelligence-agent-invocation.md
- status: Proposed
- depth: B-complete; lines=228; substance_markers=5
- cross-ref-density: 21 refs / 228 lines = 9.21 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0113 — ADR-0113-vcs-orchestrator-end-to-end.md
- status: Proposed
- depth: C-partial; lines=270; substance_markers=4
- cross-ref-density: 25 refs / 270 lines = 9.26 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0114 — ADR-0114-canary-observability-rollback.md
- status: Proposed
- depth: C-partial; lines=297; substance_markers=4
- cross-ref-density: 24 refs / 297 lines = 8.08 per 100 lines
- artifact: `docs/decisions/ADR-0706-observability-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0115 — ADR-0115-registry-consolidation-flat-singular.md
- status: Accepted
- depth: B-complete; lines=166; substance_markers=6
- cross-ref-density: 40 refs / 166 lines = 24.1 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0116 — ADR-0116-retire-external-agent-coordination-tooling.md
- status: accepted
- depth: B-complete; lines=158; substance_markers=6
- cross-ref-density: 48 refs / 158 lines = 30.38 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0117 — ADR-0117-repo-hygiene-gitignore-audit-config-and-kyverno-consolidation.md
- status: Accepted
- depth: B-complete; lines=90; substance_markers=6
- cross-ref-density: 16 refs / 90 lines = 17.78 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0118 — ADR-0118-retire-archive-orphan-fitness-lane.md
- status: Accepted
- depth: B-complete; lines=81; substance_markers=5
- cross-ref-density: 22 refs / 81 lines = 27.16 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0119 — ADR-0119-specs-flat-root-topology.md
- status: Accepted
- depth: B-complete; lines=119; substance_markers=7
- cross-ref-density: 91 refs / 119 lines = 76.47 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0120 — ADR-0120-rust-first-onprem-tooling-with-paired-uninstall.md
- status: accepted
- depth: D-thin; lines=105; substance_markers=2
- cross-ref-density: 7 refs / 105 lines = 6.67 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0121 — ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md
- status: accepted
- depth: B-complete; lines=129; substance_markers=5
- cross-ref-density: 33 refs / 129 lines = 25.58 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0122 — ADR-0122-ontology-crate-rename-from-object-graph.md
- status: Accepted
- depth: C-partial; lines=86; substance_markers=4
- cross-ref-density: 37 refs / 86 lines = 43.02 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0123 — ADR-0123-hyperscaler-maturity-claim-gate.md
- status: Accepted
- depth: B-complete; lines=81; substance_markers=7
- cross-ref-density: 13 refs / 81 lines = 16.05 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0124 — ADR-0124-own-merge-queue-webhook-driven.md
- status: accepted
- depth: C-partial; lines=163; substance_markers=4
- cross-ref-density: 17 refs / 163 lines = 10.43 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0125
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0126
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0127
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0128 — ADR-0128-hyperscaler-architecture-invariants.md
- status: Accepted
- depth: B-complete; lines=183; substance_markers=5
- cross-ref-density: 24 refs / 183 lines = 13.11 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0129 — ADR-0129-changeset-plan-dag-and-honest-claims-gate.md
- status: Accepted
- depth: C-partial; lines=128; substance_markers=4
- cross-ref-density: 12 refs / 128 lines = 9.38 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0130 — ADR-0130-deprecate-knowledge-graph-registry-file-migrate-to-ontology.md
- status: accepted
- depth: B-complete; lines=91; substance_markers=5
- cross-ref-density: 30 refs / 91 lines = 32.97 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0131 — ADR-0131-per-microservice-flat-layout.md
- status: Accepted
- depth: A-rigorous; lines=392; substance_markers=10
- cross-ref-density: 160 refs / 392 lines = 40.82 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0132 — ADR-0132-product-platform-and-bundle-dissolution.md
- status: Accepted
- depth: C-partial; lines=112; substance_markers=4
- cross-ref-density: 74 refs / 112 lines = 66.07 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0133 — ADR-0133-industry-best-practice-conformance-program.md
- status: Accepted
- depth: A-rigorous; lines=221; substance_markers=10
- cross-ref-density: 68 refs / 221 lines = 30.77 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0134 — ADR-0134-portfolio-hyperscaler-pattern-remediation-backlog.md
- status: Proposed
- depth: B-complete; lines=99; substance_markers=8
- cross-ref-density: 9 refs / 99 lines = 9.09 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0135 — ADR-0135-aspirational-enforcement-gate.md
- status: Accepted
- depth: B-complete; lines=130; substance_markers=5
- cross-ref-density: 13 refs / 130 lines = 10.0 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0136 — ADR-0136-intelligence-as-single-microservice.md
- status: Accepted
- depth: A-rigorous; lines=426; substance_markers=11
- cross-ref-density: 115 refs / 426 lines = 27.0 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0137 — ADR-0137-intelligence-bounded-contexts.md
- status: Accepted
- depth: A-rigorous; lines=366; substance_markers=9
- cross-ref-density: 86 refs / 366 lines = 23.5 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0138 — ADR-0138-intelligence-six-path-deprecation.md
- status: Accepted
- depth: A-rigorous; lines=369; substance_markers=10
- cross-ref-density: 170 refs / 369 lines = 46.07 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0139 — ADR-0139-agentic-slo-gated-promotion.md
- status: Accepted
- depth: A-rigorous; lines=261; substance_markers=10
- cross-ref-density: 81 refs / 261 lines = 31.03 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0140 — ADR-0140-cross-cutting-carriers-adapter-exemption.md
- status: Superseded
- depth: A-rigorous; lines=351; substance_markers=7
- cross-ref-density: 72 refs / 351 lines = 20.51 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0141 — ADR-0141-workflow-ontology-read-path-direct.md
- status: Superseded
- depth: B-complete; lines=216; substance_markers=6
- cross-ref-density: 40 refs / 216 lines = 18.52 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0142 — ADR-0142-crdt-portability-trait.md
- status: Accepted
- depth: B-complete; lines=228; substance_markers=5
- cross-ref-density: 14 refs / 228 lines = 6.14 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0143 — ADR-0143-intelligence-per-bc-release-pointer.md
- status: Accepted
- depth: A-rigorous; lines=232; substance_markers=7
- cross-ref-density: 34 refs / 232 lines = 14.66 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0144 — ADR-0144-eu-ai-act-graduated-risk-tier-model.md
- status: Accepted
- depth: A-rigorous; lines=283; substance_markers=7
- cross-ref-density: 37 refs / 283 lines = 13.07 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0145 — ADR-0145-inter-microservice-communication-reform.md
- status: Accepted
- depth: A-rigorous; lines=198; substance_markers=10
- cross-ref-density: 60 refs / 198 lines = 30.3 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0146 — ADR-0146-container-base-image-distroless-nonroot.md
- status: accepted
- depth: C-partial; lines=132; substance_markers=3
- cross-ref-density: 11 refs / 132 lines = 8.33 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0147 — ADR-0147-container-sandboxing-runtime-ladder.md
- status: Amended
- depth: B-complete; lines=417; substance_markers=6
- cross-ref-density: 40 refs / 417 lines = 9.59 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0148 — ADR-0148-service-mesh-cilium-ambient-layered.md
- status: Accepted
- depth: A-rigorous; lines=259; substance_markers=7
- cross-ref-density: 130 refs / 259 lines = 50.19 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0149 — ADR-0149-idempotency-keys-canonical.md
- status: Accepted
- depth: D-thin; lines=66; substance_markers=1
- cross-ref-density: 11 refs / 66 lines = 16.67 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0150 — ADR-0150-cursor-pagination-canonical.md
- status: Accepted
- depth: D-thin; lines=63; substance_markers=0
- cross-ref-density: 2 refs / 63 lines = 3.17 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: low cross-reference density; verification evidence absent
### ADR-0151 — ADR-0151-request-id-propagation.md
- status: Accepted
- depth: D-thin; lines=65; substance_markers=1
- cross-ref-density: 5 refs / 65 lines = 7.69 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0152 — ADR-0152-rpo-rto-canonical.md
- status: Accepted
- depth: D-thin; lines=61; substance_markers=1
- cross-ref-density: 8 refs / 61 lines = 13.11 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0153 — ADR-0153-outbox-pattern.md
- status: Accepted
- depth: D-thin; lines=70; substance_markers=1
- cross-ref-density: 3 refs / 70 lines = 4.29 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0154 — ADR-0154-event-schema-versioning.md
- status: Accepted
- depth: D-thin; lines=65; substance_markers=1
- cross-ref-density: 7 refs / 65 lines = 10.77 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0155 — ADR-0155-per-tenant-resource-quotas.md
- status: Accepted
- depth: D-thin; lines=59; substance_markers=2
- cross-ref-density: 2 refs / 59 lines = 3.39 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: low cross-reference density; verification evidence absent
### ADR-0156 — ADR-0156-pii-registry-canonical.md
- status: Accepted
- depth: D-thin; lines=62; substance_markers=2
- cross-ref-density: 9 refs / 62 lines = 14.52 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0157 — ADR-0157-api-gateway-tier.md
- status: Accepted
- depth: A-rigorous; lines=159; substance_markers=8
- cross-ref-density: 72 refs / 159 lines = 45.28 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0158 — ADR-0158-multi-region-active-active.md
- status: Accepted
- depth: B-complete; lines=165; substance_markers=6
- cross-ref-density: 46 refs / 165 lines = 27.88 per 100 lines
- artifact: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0159 — ADR-0159-feature-flag-substrate.md
- status: Accepted
- depth: A-rigorous; lines=174; substance_markers=9
- cross-ref-density: 82 refs / 174 lines = 47.13 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0160 — ADR-0160-progressive-delivery-flagger.md
- status: Accepted
- depth: A-rigorous; lines=156; substance_markers=8
- cross-ref-density: 62 refs / 156 lines = 39.74 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0161 — ADR-0161-csi-storage-class-canonical.md
- status: Accepted
- depth: B-complete; lines=168; substance_markers=5
- cross-ref-density: 37 refs / 168 lines = 22.02 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0162 — ADR-0162-per-tenant-audit-log-slicing.md
- status: Accepted
- depth: A-rigorous; lines=164; substance_markers=8
- cross-ref-density: 73 refs / 164 lines = 44.51 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0163 — ADR-0163-tenant-environment-tiers.md
- status: Accepted
- depth: A-rigorous; lines=166; substance_markers=8
- cross-ref-density: 58 refs / 166 lines = 34.94 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0164 — ADR-0164-sovereign-cloud-air-gapped.md
- status: Accepted
- depth: B-complete; lines=187; substance_markers=6
- cross-ref-density: 57 refs / 187 lines = 30.48 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0165 — ADR-0165-chaos-engineering-substrate.md
- status: Accepted
- depth: A-rigorous; lines=167; substance_markers=7
- cross-ref-density: 44 refs / 167 lines = 26.35 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0166 — ADR-0166-schema-registry.md
- status: Accepted
- depth: C-partial; lines=191; substance_markers=4
- cross-ref-density: 65 refs / 191 lines = 34.03 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0167 — ADR-0167-tenant-cli.md
- status: Accepted
- depth: A-rigorous; lines=213; substance_markers=7
- cross-ref-density: 55 refs / 213 lines = 25.82 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0168 — ADR-0168-public-status-page.md
- status: Accepted
- depth: B-complete; lines=211; substance_markers=5
- cross-ref-density: 44 refs / 211 lines = 20.85 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0169 — ADR-0169-webhook-dlq-retry.md
- status: Accepted
- depth: B-complete; lines=222; substance_markers=6
- cross-ref-density: 36 refs / 222 lines = 16.22 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0170 — ADR-0170-developer-portal.md
- status: Accepted
- depth: A-rigorous; lines=236; substance_markers=7
- cross-ref-density: 65 refs / 236 lines = 27.54 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md
- named gaps: none named; retain in regression audit
### ADR-0171 — ADR-0171-multi-cluster-federation.md
- status: Accepted
- depth: B-complete; lines=208; substance_markers=5
- cross-ref-density: 43 refs / 208 lines = 20.67 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0172 — ADR-0172-cqrs-read-replicas.md
- status: Accepted
- depth: C-partial; lines=218; substance_markers=3
- cross-ref-density: 41 refs / 218 lines = 18.81 per 100 lines
- artifact: `docs/decisions/ADR-0703-cas-cache-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0173 — ADR-0173-vendor-lock-in-avoidance-and-stack-ownership.md
- status: accepted
- depth: A-rigorous; lines=397; substance_markers=10
- cross-ref-density: 95 refs / 397 lines = 23.93 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0174 — ADR-0174-finops-cost-attribution-chargeback.md
- status: Accepted
- depth: B-complete; lines=245; substance_markers=5
- cross-ref-density: 38 refs / 245 lines = 15.51 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0175 — ADR-0175-tenant-lifecycle-workflow.md
- status: Accepted
- depth: B-complete; lines=224; substance_markers=6
- cross-ref-density: 33 refs / 224 lines = 14.73 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: verification evidence absent
### ADR-0176 — ADR-0176-brownout-degradation-signal-api.md
- status: Accepted
- depth: C-partial; lines=258; substance_markers=4
- cross-ref-density: 27 refs / 258 lines = 10.47 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: verification evidence absent
### ADR-0177 — ADR-0177-internal-external-api-surface-separation.md
- status: Accepted
- depth: B-complete; lines=207; substance_markers=6
- cross-ref-density: 41 refs / 207 lines = 19.81 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0178 — ADR-0178-layered-throttling-tiers.md
- status: Accepted
- depth: C-partial; lines=246; substance_markers=3
- cross-ref-density: 34 refs / 246 lines = 13.82 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0179 — ADR-0179-postgres-connection-pooling-pgcat.md
- status: Accepted
- depth: B-complete; lines=126; substance_markers=6
- cross-ref-density: 37 refs / 126 lines = 29.37 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0180 — ADR-0180-slo-composition-inheritance-arithmetic.md
- status: Accepted
- depth: C-partial; lines=149; substance_markers=4
- cross-ref-density: 29 refs / 149 lines = 19.46 per 100 lines
- artifact: `docs/decisions/ADR-0706-observability-live-apex.md`
- named gaps: verification evidence absent
### ADR-0181 — ADR-0181-container-image-promotion-pipeline.md
- status: Accepted
- depth: B-complete; lines=153; substance_markers=5
- cross-ref-density: 46 refs / 153 lines = 30.07 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0182 — ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md
- status: Accepted
- depth: A-rigorous; lines=176; substance_markers=7
- cross-ref-density: 67 refs / 176 lines = 38.07 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0183 — ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
- status: Accepted
- depth: B-complete; lines=176; substance_markers=6
- cross-ref-density: 124 refs / 176 lines = 70.45 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0184 — ADR-0184-storage-tier-layering.md
- status: Accepted
- depth: A-rigorous; lines=199; substance_markers=8
- cross-ref-density: 40 refs / 199 lines = 20.1 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0185 — ADR-0185-workflow-studio-client-stack.md
- status: Accepted
- depth: A-rigorous; lines=331; substance_markers=7
- cross-ref-density: 48 refs / 331 lines = 14.5 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0186 — ADR-0186-observability-backplane-layering.md
- status: Accepted
- depth: B-complete; lines=217; substance_markers=6
- cross-ref-density: 41 refs / 217 lines = 18.89 per 100 lines
- artifact: `docs/decisions/ADR-0706-observability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0187 — ADR-0187-canonical-oidc-idp-zitadel-primary.md
- status: Accepted
- depth: A-rigorous; lines=175; substance_markers=8
- cross-ref-density: 61 refs / 175 lines = 34.86 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0188 — ADR-0188-passkey-webauthn-substrate.md
- status: Accepted
- depth: A-rigorous; lines=174; substance_markers=7
- cross-ref-density: 23 refs / 174 lines = 13.22 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0189 — ADR-0189-step-up-authentication-acr-classes.md
- status: Accepted
- depth: A-rigorous; lines=176; substance_markers=7
- cross-ref-density: 57 refs / 176 lines = 32.39 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0190 — ADR-0190-scim-2-provisioning-enterprise-tenants.md
- status: Accepted
- depth: A-rigorous; lines=152; substance_markers=7
- cross-ref-density: 24 refs / 152 lines = 15.79 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0191 — ADR-0191-edge-authz-tier-vs-origin-cedar-pdp.md
- status: Accepted
- depth: C-partial; lines=178; substance_markers=4
- cross-ref-density: 88 refs / 178 lines = 49.44 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0192 — ADR-0192-vector-database-canonical-milvus.md
- status: Accepted
- depth: A-rigorous; lines=291; substance_markers=8
- cross-ref-density: 82 refs / 291 lines = 28.18 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0193 — ADR-0193-olap-analytics-warehouse-clickhouse.md
- status: Accepted
- depth: A-rigorous; lines=276; substance_markers=8
- cross-ref-density: 63 refs / 276 lines = 22.83 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0194 — ADR-0194-tenant-facing-timeseries-timescaledb.md
- status: Accepted
- depth: A-rigorous; lines=244; substance_markers=8
- cross-ref-density: 65 refs / 244 lines = 26.64 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0195 — ADR-0195-stream-processing-tier.md
- status: Accepted
- depth: B-complete; lines=224; substance_markers=6
- cross-ref-density: 40 refs / 224 lines = 17.86 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit; verification evidence absent
### ADR-0196 — ADR-0196-object-storage-canonical-seaweedfs-primary-ceph-scale-up.md
- status: Accepted
- depth: A-rigorous; lines=305; substance_markers=9
- cross-ref-density: 32 refs / 305 lines = 10.49 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0197 — ADR-0197-backup-substrate-velero-pgbackrest-restic.md
- status: Accepted
- depth: A-rigorous; lines=330; substance_markers=10
- cross-ref-density: 41 refs / 330 lines = 12.42 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0198 — ADR-0198-k8s-node-autoscaling-karpenter.md
- status: Accepted
- depth: B-complete; lines=297; substance_markers=5
- cross-ref-density: 26 refs / 297 lines = 8.75 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0199 — ADR-0199-per-tenant-cost-attribution-finops-substrate.md
- status: Accepted
- depth: A-rigorous; lines=332; substance_markers=8
- cross-ref-density: 37 refs / 332 lines = 11.14 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0200 — ADR-0200-wasm-runtime-canonical-wasmtime.md
- status: Accepted
- depth: B-complete; lines=202; substance_markers=6
- cross-ref-density: 25 refs / 202 lines = 12.38 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0201 — ADR-0201-email-transactional-comms-adapter-substrate.md
- status: Accepted
- depth: A-rigorous; lines=266; substance_markers=8
- cross-ref-density: 34 refs / 266 lines = 12.78 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0202 — ADR-0202-gitops-iac-cluster-lifecycle-three-tier.md
- status: Accepted
- depth: C-partial; lines=223; substance_markers=3
- cross-ref-density: 26 refs / 223 lines = 11.66 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0203 — ADR-0203-documentation-engine-three-tier.md
- status: Accepted
- depth: C-partial; lines=220; substance_markers=4
- cross-ref-density: 32 refs / 220 lines = 14.55 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0204 — ADR-0204-workflow-studio-canvas-library.md
- status: Accepted
- depth: A-rigorous; lines=159; substance_markers=8
- cross-ref-density: 31 refs / 159 lines = 19.5 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0205 — ADR-0205-code-editor-canonical-codemirror.md
- status: Accepted
- depth: C-partial; lines=143; substance_markers=4
- cross-ref-density: 26 refs / 143 lines = 18.18 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0206 — ADR-0206-i18n-substrate-fluent-icu.md
- status: Accepted
- depth: D-thin; lines=162; substance_markers=2
- cross-ref-density: 14 refs / 162 lines = 8.64 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: verification evidence absent
### ADR-0207 — ADR-0207-accessibility-wcag-2-2-aa.md
- status: Accepted
- depth: C-partial; lines=161; substance_markers=4
- cross-ref-density: 19 refs / 161 lines = 11.8 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0208 — ADR-0208-realtime-transport-tier.md
- status: Accepted
- depth: C-partial; lines=178; substance_markers=3
- cross-ref-density: 23 refs / 178 lines = 12.92 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: verification evidence absent
### ADR-0209 — ADR-0209-compliance-evidence-automation.md
- status: Accepted
- depth: B-complete; lines=164; substance_markers=5
- cross-ref-density: 68 refs / 164 lines = 41.46 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0210 — ADR-0210-otel-tail-sampling.md
- status: Accepted
- depth: B-complete; lines=185; substance_markers=5
- cross-ref-density: 36 refs / 185 lines = 19.46 per 100 lines
- artifact: `docs/decisions/ADR-0706-observability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0211 — ADR-0211-in-house-tech-stack-policy.md
- status: accepted
- depth: A-rigorous; lines=226; substance_markers=10
- cross-ref-density: 74 refs / 226 lines = 32.74 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0212 — ADR-0212-buildability-doctrine.md
- status: accepted
- depth: A-rigorous; lines=124; substance_markers=12
- cross-ref-density: 16 refs / 124 lines = 12.9 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0213 — ADR-0213-ecosystem-as-a-service-architecture.md
- status: Proposed
- depth: A-rigorous; lines=345; substance_markers=13
- cross-ref-density: 168 refs / 345 lines = 48.7 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0214 — ADR-0214-cross-tenant-real-time-visibility.md
- status: Proposed (target: Accepted upon PR #143 merge to dev)
- depth: A-rigorous; lines=269; substance_markers=12
- cross-ref-density: 49 refs / 269 lines = 18.22 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0215 — ADR-0215-multi-context-platform-architecture.md
- status: accepted
- depth: A-rigorous; lines=123; substance_markers=8
- cross-ref-density: 48 refs / 123 lines = 39.02 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: verification evidence absent
### ADR-0216 — ADR-0216-open-integration-and-migration-out-policy.md
- status: accepted
- depth: B-complete; lines=111; substance_markers=7
- cross-ref-density: 29 refs / 111 lines = 26.13 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0217 — ADR-0217-vertical-slice-rollout-order.md
- status: accepted
- depth: A-rigorous; lines=131; substance_markers=13
- cross-ref-density: 39 refs / 131 lines = 29.77 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0218 — ADR-0218-tenant-granular-control-surface.md
- status: accepted
- depth: B-complete; lines=117; substance_markers=9
- cross-ref-density: 39 refs / 117 lines = 33.33 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0219 — ADR-0219-no-code-first-ux-with-optional-ai-assist.md
- status: accepted
- depth: B-complete; lines=124; substance_markers=6
- cross-ref-density: 23 refs / 124 lines = 18.55 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0220 — ADR-0220-consumer-intelligence-substrate.md
- status: accepted
- depth: A-rigorous; lines=128; substance_markers=8
- cross-ref-density: 35 refs / 128 lines = 27.34 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0221 — ADR-0221-agentic-development-pipeline-hardening.md
- status: accepted
- depth: B-complete; lines=171; substance_markers=6
- cross-ref-density: 30 refs / 171 lines = 17.54 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0222 — ADR-0222-saga-compensation-portfolio-policy.md
- status: Accepted
- depth: A-rigorous; lines=241; substance_markers=8
- cross-ref-density: 42 refs / 241 lines = 17.43 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0223 — ADR-0223-oya-git-drop-in-surface-with-explicit-policy-verbs.md
- status: Accepted
- depth: C-partial; lines=102; substance_markers=3
- cross-ref-density: 14 refs / 102 lines = 13.73 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0224
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0225
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0226
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0227
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0228
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0229
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0230
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0231
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0232
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0233
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0234 — ADR-0234-community-social-expansion-planning-contract.md
- status: Accepted
- depth: C-partial; lines=75; substance_markers=7
- cross-ref-density: 34 refs / 75 lines = 45.33 per 100 lines
- artifact: `docs/adr-archive/ADR-0234-connect-social-expansion-planning-contract.md`
- named gaps: none named; retain in regression audit
### ADR-0235 — ADR-0235-connect-core-public-contracts.md
- status: Accepted
- depth: C-partial; lines=75; substance_markers=7
- cross-ref-density: 37 refs / 75 lines = 49.33 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0236 — ADR-0236-op11-corpus-remediation-planning-contract.md
- status: Proposed
- depth: B-complete; lines=106; substance_markers=5
- cross-ref-density: 13 refs / 106 lines = 12.26 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0237 — ADR-0237-connect-dissolution-strangler-migration.md
- status: Accepted
- depth: B-complete; lines=425; substance_markers=6
- cross-ref-density: 86 refs / 425 lines = 20.24 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0238 — ADR-0238-connect-super-app-expansion.md
- status: Accepted
- depth: A-rigorous; lines=371; substance_markers=9
- cross-ref-density: 151 refs / 371 lines = 40.7 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0239 — ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md
- status: accepted
- depth: B-complete; lines=106; substance_markers=5
- cross-ref-density: 35 refs / 106 lines = 33.02 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: verification evidence absent
### ADR-0240 — ADR-0240-sovereign-cloud-per-regional-pack.md
- status: Accepted
- depth: A-rigorous; lines=270; substance_markers=7
- cross-ref-density: 53 refs / 270 lines = 19.63 per 100 lines
- artifact: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
- named gaps: verification evidence absent
### ADR-0241 — ADR-0241-dr-business-continuity-portfolio-policy.md
- status: Accepted
- depth: A-rigorous; lines=255; substance_markers=8
- cross-ref-density: 37 refs / 255 lines = 14.51 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: verification evidence absent
### ADR-0242 — ADR-0242-oyatie-is-a-tenant-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=1099; substance_markers=13
- cross-ref-density: 311 refs / 1099 lines = 28.3 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0243 — ADR-0243-cedar-as-universal-gate.md
- status: Proposed
- depth: A-rigorous; lines=1103; substance_markers=12
- cross-ref-density: 567 refs / 1103 lines = 51.41 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0244 — ADR-0244-tenant-as-universal-scoping-primitive.md
- status: Proposed
- depth: A-rigorous; lines=2359; substance_markers=15
- cross-ref-density: 580 refs / 2359 lines = 24.59 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0245 — ADR-0245-substrate-vs-product-layering.md
- status: Proposed
- depth: A-rigorous; lines=1901; substance_markers=15
- cross-ref-density: 519 refs / 1901 lines = 27.3 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0246 — ADR-0246-policy-engine-substrate-promotion.md
- status: Proposed
- depth: A-rigorous; lines=2118; substance_markers=13
- cross-ref-density: 879 refs / 2118 lines = 41.5 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0247 — ADR-0247-self-hosting-self-modification-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=2023; substance_markers=15
- cross-ref-density: 776 refs / 2023 lines = 38.36 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md
- named gaps: none named; retain in regression audit
### ADR-0248 — ADR-0248-amazon-shape-cellular-architecture.md
- status: Proposed
- depth: A-rigorous; lines=2296; substance_markers=13
- cross-ref-density: 414 refs / 2296 lines = 18.03 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0249 — ADR-0249-multi-category-marketplace-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=2987; substance_markers=14
- cross-ref-density: 524 refs / 2987 lines = 17.54 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0250 — ADR-0250-build-ahead-of-certification-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=1786; substance_markers=15
- cross-ref-density: 449 refs / 1786 lines = 25.14 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0251 — ADR-0251-compliance-pack-cell-certification-levels.md
- status: Proposed
- depth: A-rigorous; lines=2648; substance_markers=12
- cross-ref-density: 614 refs / 2648 lines = 23.19 per 100 lines
- artifact: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0252 — ADR-0252-time-coordination-distributed-consistency.md
- status: Proposed
- depth: A-rigorous; lines=2028; substance_markers=11
- cross-ref-density: 243 refs / 2028 lines = 11.98 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0253 — ADR-0253-network-topology-edge-service-mesh.md
- status: Proposed
- depth: A-rigorous; lines=1796; substance_markers=14
- cross-ref-density: 364 refs / 1796 lines = 20.27 per 100 lines
- artifact: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0254 — ADR-0254-deployment-model-spectrum.md
- status: Proposed
- depth: A-rigorous; lines=2222; substance_markers=14
- cross-ref-density: 487 refs / 2222 lines = 21.92 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0255 — ADR-0355-amendment-library-first-network-opt-in-clarification.md
- status: Proposed
- depth: A-rigorous; lines=1187; substance_markers=10
- cross-ref-density: 244 refs / 1187 lines = 20.56 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0256
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0257 — ADR-0257-ontology-object-type-versioning-deprecation-handshake.md
- status: Proposed
- depth: A-rigorous; lines=1832; substance_markers=10
- cross-ref-density: 282 refs / 1832 lines = 15.39 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0258 — ADR-0258-api-versioning-model.md
- status: Accepted
- depth: A-rigorous; lines=1108; substance_markers=12
- cross-ref-density: 226 refs / 1108 lines = 20.4 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0259
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0260
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0261
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0262
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0263 — ADR-0263-observability-emission-contract.md
- status: Proposed
- depth: A-rigorous; lines=1907; substance_markers=12
- cross-ref-density: 358 refs / 1907 lines = 18.77 per 100 lines
- artifact: `docs/decisions/ADR-0706-observability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0264
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0265
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0266
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0267
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0268
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0269
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0270
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0271
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0272 — ADR-0272-cookie-consent-per-purpose-analytics-opt-in.md
- status: Proposed
- depth: A-rigorous; lines=1846; substance_markers=14
- cross-ref-density: 275 refs / 1846 lines = 14.9 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0273 — ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md
- status: Proposed
- depth: A-rigorous; lines=1856; substance_markers=11
- cross-ref-density: 175 refs / 1856 lines = 9.43 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0274
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0275
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0276 — ADR-0276-backup-portability-format-gdpr-article-20.md
- status: Proposed
- depth: A-rigorous; lines=2083; substance_markers=12
- cross-ref-density: 240 refs / 2083 lines = 11.52 per 100 lines
- artifact: `docs/decisions/ADR-0704-k8s-port-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0277
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0278
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0279
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0280 — ADR-0280-substrate-of-substrate-dependency-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=2246; substance_markers=11
- cross-ref-density: 473 refs / 2246 lines = 21.06 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0281
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0282
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0283
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0284 — ADR-0284-platform-owner-name-indirection.md
- status: Proposed
- depth: A-rigorous; lines=1755; substance_markers=12
- cross-ref-density: 288 refs / 1755 lines = 16.41 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0285
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0286
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0287
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0288
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0289
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0290
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0291
- status: MISSING
- depth: F-missing
- cross-ref-density: 0 refs / 0 lines
- named gaps: missing decision artifact; cannot verify rationale, alternatives, constraints, tests, or cross-service effect
- remediation gate: author ADR shell only after source-of-truth owner confirms decision exists; otherwise mark reserved/deprecated explicitly
### ADR-0292 — ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
- status: Proposed
- depth: A-rigorous; lines=1945; substance_markers=13
- cross-ref-density: 346 refs / 1945 lines = 17.79 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0293 — ADR-0293-governance-meta-trust-root.md
- status: Proposed
- depth: A-rigorous; lines=1349; substance_markers=11
- cross-ref-density: 210 refs / 1349 lines = 15.57 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0294 — ADR-0294-cedar-fragment-soak-anomaly-rollback.md
- status: Proposed
- depth: A-rigorous; lines=1114; substance_markers=11
- cross-ref-density: 253 refs / 1114 lines = 22.71 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0295 — ADR-0295-bootstrap-ci-spiffe-kill-switch.md
- status: Proposed
- depth: A-rigorous; lines=1243; substance_markers=12
- cross-ref-density: 139 refs / 1243 lines = 11.18 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0296 — ADR-0296-library-first-credential-sidecar.md
- status: Proposed
- depth: A-rigorous; lines=1378; substance_markers=10
- cross-ref-density: 160 refs / 1378 lines = 11.61 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0297 — ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
- status: Proposed
- depth: A-rigorous; lines=3115; substance_markers=13
- cross-ref-density: 497 refs / 3115 lines = 15.96 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0298 — ADR-0298-emergency-services-bypass-life-safety.md
- status: Proposed
- depth: A-rigorous; lines=1669; substance_markers=13
- cross-ref-density: 321 refs / 1669 lines = 19.23 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0299 — ADR-0299-account-recovery-resilience.md
- status: Proposed
- depth: A-rigorous; lines=1557; substance_markers=13
- cross-ref-density: 294 refs / 1557 lines = 18.88 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0300 — ADR-0300-whistleblower-press-freedom-anonymity.md
- status: Proposed
- depth: A-rigorous; lines=1650; substance_markers=13
- cross-ref-density: 293 refs / 1650 lines = 17.76 per 100 lines
- artifact: `docs/decisions/ADR-0707-trust-safety-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0301 — ADR-0301-survivor-safety-domestic-abuse-mode.md
- status: Proposed
- depth: A-rigorous; lines=1534; substance_markers=14
- cross-ref-density: 263 refs / 1534 lines = 17.14 per 100 lines
- artifact: `docs/decisions/ADR-0707-trust-safety-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0302 — ADR-0302-deceased-user-inheritance-doctrine.md
- status: Proposed
- depth: A-rigorous; lines=1596; substance_markers=13
- cross-ref-density: 298 refs / 1596 lines = 18.67 per 100 lines
- artifact: `docs/decisions/ADR-0707-trust-safety-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0303 — ADR-0303-cognitive-impairment-decision-resilience.md
- status: Proposed
- depth: A-rigorous; lines=1829; substance_markers=12
- cross-ref-density: 331 refs / 1829 lines = 18.1 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0304 — ADR-0304-cross-jurisdiction-conflict-resolution.md
- status: Proposed
- depth: A-rigorous; lines=1527; substance_markers=11
- cross-ref-density: 262 refs / 1527 lines = 17.16 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0305 — ADR-0305-delegated-agent-authority-chain.md
- status: Proposed
- depth: A-rigorous; lines=1560; substance_markers=11
- cross-ref-density: 275 refs / 1560 lines = 17.63 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0306 — ADR-0306-disaster-mode-cell-resilience.md
- status: Proposed
- depth: A-rigorous; lines=1640; substance_markers=13
- cross-ref-density: 303 refs / 1640 lines = 18.48 per 100 lines
- artifact: `docs/decisions/ADR-0707-trust-safety-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0307 — ADR-0307-detection-substrate-streaming-batch.md
- status: Proposed
- depth: A-rigorous; lines=1866; substance_markers=13
- cross-ref-density: 368 refs / 1866 lines = 19.72 per 100 lines
- artifact: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0308 — ADR-0308-ml-model-lifecycle-ai-act-compliance.md
- status: Proposed
- depth: A-rigorous; lines=1904; substance_markers=12
- cross-ref-density: 267 refs / 1904 lines = 14.02 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0309 — ADR-0309-detection-fairness-audit-civil-rights.md
- status: Proposed
- depth: A-rigorous; lines=1783; substance_markers=11
- cross-ref-density: 205 refs / 1783 lines = 11.5 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0310 — ADR-0310-investigation-case-management.md
- status: Proposed
- depth: A-rigorous; lines=2013; substance_markers=13
- cross-ref-density: 348 refs / 2013 lines = 17.29 per 100 lines
- artifact: `docs/decisions/ADR-0703-cas-cache-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0311 — ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
- status: Proposed
- depth: A-rigorous; lines=1803; substance_markers=13
- cross-ref-density: 600 refs / 1803 lines = 33.28 per 100 lines
- artifact: `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0312 — ADR-0312-court-warrant-scoped-piercing.md
- status: Proposed
- depth: A-rigorous; lines=1510; substance_markers=12
- cross-ref-density: 312 refs / 1510 lines = 20.66 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0313 — ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
- status: Proposed
- depth: A-rigorous; lines=2987; substance_markers=13
- cross-ref-density: 535 refs / 2987 lines = 17.91 per 100 lines
- artifact: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0314 — ADR-0314-marketplace-as-universal-deal-settlement.md
- status: Proposed
- depth: A-rigorous; lines=1801; substance_markers=11
- cross-ref-density: 1514 refs / 1801 lines = 84.06 per 100 lines
- artifact: `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0315 — ADR-0315-erp-coverage-doctrine-sap-parity.md
- status: Proposed
- depth: A-rigorous; lines=2001; substance_markers=14
- cross-ref-density: 6576 refs / 2001 lines = 328.64 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0316 — ADR-0316-capability-tier-over-product-fragmentation.md
- status: Proposed
- depth: A-rigorous; lines=2145; substance_markers=16
- cross-ref-density: 792 refs / 2145 lines = 36.92 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0317 — ADR-0317-role-based-projection-unified-ux-shell.md
- status: Proposed
- depth: A-rigorous; lines=2152; substance_markers=11
- cross-ref-density: 411 refs / 2152 lines = 19.1 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0318 — ADR-0318-collar-color-workspace-universality.md
- status: Proposed
- depth: A-rigorous; lines=2951; substance_markers=13
- cross-ref-density: 856 refs / 2951 lines = 29.01 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: none named; retain in regression audit
### ADR-0319 — ADR-0319-front-middle-back-office-information-barrier.md
- status: Proposed
- depth: A-rigorous; lines=2268; substance_markers=14
- cross-ref-density: 557 refs / 2268 lines = 24.56 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0320 — ADR-0320-apprentice-intern-resident-fellow-transient-identity.md
- status: Proposed
- depth: A-rigorous; lines=1559; substance_markers=12
- cross-ref-density: 796 refs / 1559 lines = 51.06 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-0321 — ADR-0321-b2b-saas-industry-leader-coverage.md
- status: Proposed
- depth: A-rigorous; lines=20851; substance_markers=17
- cross-ref-density: 3527 refs / 20851 lines = 16.92 per 100 lines
- artifact: `docs/decisions/ADR-0709-general-live-apex.md`
- named gaps: alternatives/rejections not explicit
### ADR-MS — ADR-MS-001-edge-admission-policy-and-pqc-contract.md
- status: Proposed
- depth: A-rigorous; lines=286; substance_markers=13
- cross-ref-density: 129 refs / 286 lines = 45.1 per 100 lines
- artifact: `microservices/api-gateway/decisions/ADR-MS-001-edge-admission-policy-and-pqc-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-secret-reference-namespace-and-rotation-contract.md
- status: Proposed
- depth: A-rigorous; lines=293; substance_markers=11
- cross-ref-density: 100 refs / 293 lines = 34.13 per 100 lines
- artifact: `microservices/cloud-secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-connector-broker-webhook-and-dlq-contract.md
- status: Proposed
- depth: A-rigorous; lines=294; substance_markers=10
- cross-ref-density: 71 refs / 294 lines = 24.15 per 100 lines
- artifact: `microservices/connector/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-omnichannel-routing-queue-and-consent-contract.md
- status: Proposed
- depth: A-rigorous; lines=294; substance_markers=11
- cross-ref-density: 87 refs / 294 lines = 29.59 per 100 lines
- artifact: `microservices/contact-center/decisions/ADR-MS-001-omnichannel-routing-queue-and-consent-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-customer-record-mutation-and-revenue-lineage-contract.md
- status: Proposed
- depth: A-rigorous; lines=286; substance_markers=11
- cross-ref-density: 89 refs / 286 lines = 31.12 per 100 lines
- artifact: `microservices/crm/decisions/ADR-MS-001-customer-record-mutation-and-revenue-lineage-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md
- status: Proposed
- depth: A-rigorous; lines=288; substance_markers=11
- cross-ref-density: 85 refs / 288 lines = 29.51 per 100 lines
- artifact: `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-tenant-olap-freshness-and-lineage-contract.md
- status: Proposed
- depth: A-rigorous; lines=292; substance_markers=11
- cross-ref-density: 100 refs / 292 lines = 34.25 per 100 lines
- artifact: `microservices/data-warehouse/decisions/ADR-MS-001-tenant-olap-freshness-and-lineage-contract.md`
- named gaps: none named; retain in regression audit
### ADR-MS — ADR-MS-001-flag-evaluation-killswitch-and-experiment-contract.md
- status: Proposed
- depth: A-rigorous; lines=285; substance_markers=10
- cross-ref-density: 89 refs / 285 lines = 31.23 per 100 lines
- artifact: `microservices/feature-flags/decisions/ADR-MS-001-flag-evaluation-killswitch-and-experiment-contract.md`
- named gaps: none named; retain in regression audit

## §3 µservice Coverage Scorecard
- Live microservice directories discovered: 78
- User requested reference count: 79 µservices
- Registry declared count: 70
- Named corpus-level gap: live service count, user reference count, and capability registry declared count are not in full parity
### µservice `analytics`
- current-state: complete; artifacts=137; corpus_lines=18250
- surface coverage: api, events, data, policy, contract, ops; api=1528 events=1271 data=714 policy=5679 contract=1909 ops=1864
- capability-tier mapping: mentions=108; status=mapped
- Cedar policy coverage: policy_files=9; policy_mentions=3164; status=present
- contract version conformance: contract_mentions=280; status=evidenced
- threat-model: threat_mentions=96; status=evidenced
- test-plan: test_mentions=41; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `api-gateway`
- current-state: complete; artifacts=134; corpus_lines=16630
- surface coverage: api, events, data, policy, contract, ops; api=11414 events=1030 data=1010 policy=7113 contract=1922 ops=1384
- capability-tier mapping: mentions=31; status=mapped
- Cedar policy coverage: policy_files=13; policy_mentions=4110; status=present
- contract version conformance: contract_mentions=477; status=evidenced
- threat-model: threat_mentions=320; status=evidenced
- test-plan: test_mentions=31; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `application`
- current-state: complete; artifacts=135; corpus_lines=16768
- surface coverage: api, events, data, policy, contract, ops; api=2037 events=875 data=605 policy=5826 contract=1642 ops=1099
- capability-tier mapping: mentions=6; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3236; status=present
- contract version conformance: contract_mentions=334; status=evidenced
- threat-model: threat_mentions=151; status=evidenced
- test-plan: test_mentions=43; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `audit-chain`
- current-state: complete; artifacts=210; corpus_lines=59178
- surface coverage: api, events, data, policy, contract, ops; api=3917 events=10998 data=6014 policy=32808 contract=8162 ops=6079
- capability-tier mapping: mentions=20; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=16107; status=present
- contract version conformance: contract_mentions=2521; status=evidenced
- threat-model: threat_mentions=308; status=evidenced
- test-plan: test_mentions=1678; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `calendar`
- current-state: complete; artifacts=140; corpus_lines=27252
- surface coverage: api, events, data, policy, contract, ops; api=1752 events=4131 data=1060 policy=11133 contract=2998 ops=1638
- capability-tier mapping: mentions=26; status=mapped
- Cedar policy coverage: policy_files=9; policy_mentions=5522; status=present
- contract version conformance: contract_mentions=585; status=evidenced
- threat-model: threat_mentions=186; status=evidenced
- test-plan: test_mentions=219; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `cell`
- current-state: complete; artifacts=146; corpus_lines=26297
- surface coverage: api, events, data, policy, contract, ops; api=1930 events=1896 data=1728 policy=8809 contract=3293 ops=1658
- capability-tier mapping: mentions=47; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=5049; status=present
- contract version conformance: contract_mentions=554; status=evidenced
- threat-model: threat_mentions=151; status=evidenced
- test-plan: test_mentions=111; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `cloud-billing`
- current-state: thin; artifacts=7; corpus_lines=1154
- surface coverage: api, events, data, policy, contract, ops; api=4 events=69 data=32 policy=41 contract=20 ops=4
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=27; status=present
- contract version conformance: contract_mentions=12; status=evidenced
- threat-model: threat_mentions=1; status=evidenced
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry
### µservice `cloud-billing-tax`
- current-state: thin; artifacts=7; corpus_lines=1231
- surface coverage: api, events, data, policy, contract, ops; api=20 events=19 data=2 policy=39 contract=28 ops=2
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=25; status=present
- contract version conformance: contract_mentions=17; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry; threat model absent
### µservice `cloud-data`
- current-state: thin; artifacts=7; corpus_lines=1322
- surface coverage: api, events, data, policy, contract, ops; api=71 events=15 data=79 policy=66 contract=21 ops=4
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=45; status=present
- contract version conformance: contract_mentions=8; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=5; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry; threat model absent
### µservice `cloud-iac`
- current-state: complete; artifacts=168; corpus_lines=21404
- surface coverage: api, events, data, policy, contract, ops; api=1791 events=1313 data=611 policy=6184 contract=2073 ops=1433
- capability-tier mapping: mentions=47; status=mapped
- Cedar policy coverage: policy_files=7; policy_mentions=3493; status=present
- contract version conformance: contract_mentions=351; status=evidenced
- threat-model: threat_mentions=160; status=evidenced
- test-plan: test_mentions=58; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `cloud-iam`
- current-state: partial; artifacts=10; corpus_lines=1834
- surface coverage: api, events, data, policy, contract, ops; api=62 events=74 data=97 policy=424 contract=140 ops=256
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=278; status=present
- contract version conformance: contract_mentions=12; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=15; status=evidenced
- named gaps: not mapped in capability-tier registry; threat model absent
### µservice `cloud-k8s`
- current-state: complete; artifacts=121; corpus_lines=19047
- surface coverage: api, events, data, policy, contract, ops; api=1841 events=1028 data=395 policy=6188 contract=1756 ops=1209
- capability-tier mapping: mentions=3; status=mapped
- Cedar policy coverage: policy_files=10; policy_mentions=3661; status=present
- contract version conformance: contract_mentions=354; status=evidenced
- threat-model: threat_mentions=154; status=evidenced
- test-plan: test_mentions=155; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `cloud-kms`
- current-state: thin; artifacts=7; corpus_lines=1144
- surface coverage: api, events, data, policy, contract, ops; api=13 events=10 data=17 policy=66 contract=45 ops=3
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=48; status=present
- contract version conformance: contract_mentions=29; status=evidenced
- threat-model: threat_mentions=1; status=evidenced
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry
### µservice `cloud-network`
- current-state: thin; artifacts=7; corpus_lines=1140
- surface coverage: api, events, data, policy, contract, ops; api=36 events=26 data=5 policy=231 contract=23 ops=2
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=1; policy_mentions=158; status=present
- contract version conformance: contract_mentions=7; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry; threat model absent
### µservice `cloud-network-dns`
- current-state: thin; artifacts=7; corpus_lines=1226
- surface coverage: api, events, data, policy, contract, ops; api=94 events=8 data=1 policy=50 contract=13 ops=4
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=35; status=present
- contract version conformance: contract_mentions=7; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry; threat model absent
### µservice `cloud-secrets`
- current-state: complete; artifacts=134; corpus_lines=20473
- surface coverage: api, events, data, policy, contract, ops; api=1579 events=1416 data=546 policy=6588 contract=2232 ops=1302
- capability-tier mapping: mentions=17; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3777; status=present
- contract version conformance: contract_mentions=313; status=evidenced
- threat-model: threat_mentions=164; status=evidenced
- test-plan: test_mentions=60; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `cloud-storage`
- current-state: thin; artifacts=7; corpus_lines=1347
- surface coverage: api, events, data, policy, contract; api=65 events=6 data=7 policy=73 contract=126 ops=0
- capability-tier mapping: mentions=0; status=unmapped
- Cedar policy coverage: policy_files=0; policy_mentions=54; status=present
- contract version conformance: contract_mentions=106; status=evidenced
- threat-model: threat_mentions=0; status=missing
- test-plan: test_mentions=4; status=evidenced
- named gaps: low artifact count; not mapped in capability-tier registry; threat model absent
### µservice `comms-email`
- current-state: complete; artifacts=134; corpus_lines=14701
- surface coverage: api, events, data, policy, contract, ops; api=1131 events=1225 data=302 policy=5845 contract=1521 ops=1291
- capability-tier mapping: mentions=5; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=3349; status=present
- contract version conformance: contract_mentions=183; status=evidenced
- threat-model: threat_mentions=216; status=evidenced
- test-plan: test_mentions=44; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `community`
- current-state: complete; artifacts=201; corpus_lines=43376
- surface coverage: api, events, data, policy, contract, ops; api=2912 events=4977 data=3429 policy=18722 contract=6563 ops=4477
- capability-tier mapping: mentions=95; status=mapped
- Cedar policy coverage: policy_files=13; policy_mentions=10183; status=present
- contract version conformance: contract_mentions=1822; status=evidenced
- threat-model: threat_mentions=474; status=evidenced
- test-plan: test_mentions=1221; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `compliance`
- current-state: complete; artifacts=199; corpus_lines=43547
- surface coverage: api, events, data, policy, contract, ops; api=3526 events=6889 data=3705 policy=25571 contract=5938 ops=6418
- capability-tier mapping: mentions=72; status=mapped
- Cedar policy coverage: policy_files=18; policy_mentions=12979; status=present
- contract version conformance: contract_mentions=1870; status=evidenced
- threat-model: threat_mentions=305; status=evidenced
- test-plan: test_mentions=1172; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `connector`
- current-state: complete; artifacts=183; corpus_lines=32250
- surface coverage: api, events, data, policy, contract, ops; api=2723 events=4145 data=2077 policy=14417 contract=3929 ops=2280
- capability-tier mapping: mentions=5; status=mapped
- Cedar policy coverage: policy_files=13; policy_mentions=7795; status=present
- contract version conformance: contract_mentions=814; status=evidenced
- threat-model: threat_mentions=398; status=evidenced
- test-plan: test_mentions=1172; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `consent-graph`
- current-state: complete; artifacts=135; corpus_lines=21742
- surface coverage: api, events, data, policy, contract, ops; api=1520 events=1741 data=1845 policy=6906 contract=2011 ops=1356
- capability-tier mapping: mentions=6; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3774; status=present
- contract version conformance: contract_mentions=359; status=evidenced
- threat-model: threat_mentions=203; status=evidenced
- test-plan: test_mentions=45; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `contact-center`
- current-state: complete; artifacts=169; corpus_lines=17561
- surface coverage: api, events, data, policy, contract, ops; api=4350 events=2998 data=873 policy=3095 contract=2077 ops=2004
- capability-tier mapping: mentions=17; status=mapped
- Cedar policy coverage: policy_files=20; policy_mentions=1824; status=present
- contract version conformance: contract_mentions=256; status=evidenced
- threat-model: threat_mentions=668; status=evidenced
- test-plan: test_mentions=5333; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `contract-lifecycle-management`
- current-state: complete; artifacts=157; corpus_lines=23185
- surface coverage: api, events, data, policy, contract, ops; api=4370 events=3749 data=1612 policy=7461 contract=19024 ops=2418
- capability-tier mapping: mentions=19; status=mapped
- Cedar policy coverage: policy_files=23; policy_mentions=5344; status=present
- contract version conformance: contract_mentions=1175; status=evidenced
- threat-model: threat_mentions=923; status=evidenced
- test-plan: test_mentions=5344; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `crm`
- current-state: complete; artifacts=141; corpus_lines=18366
- surface coverage: api, events, data, policy, contract, ops; api=2819 events=3806 data=2175 policy=7476 contract=1553 ops=4227
- capability-tier mapping: mentions=79; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=4687; status=present
- contract version conformance: contract_mentions=637; status=evidenced
- threat-model: threat_mentions=507; status=evidenced
- test-plan: test_mentions=652; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `data-pipeline`
- current-state: complete; artifacts=150; corpus_lines=20080
- surface coverage: api, events, data, policy, contract, ops; api=1589 events=1647 data=1927 policy=4144 contract=2503 ops=2403
- capability-tier mapping: mentions=60; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2574; status=present
- contract version conformance: contract_mentions=339; status=evidenced
- threat-model: threat_mentions=683; status=evidenced
- test-plan: test_mentions=4769; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `data-warehouse`
- current-state: complete; artifacts=153; corpus_lines=22169
- surface coverage: api, events, data, policy, contract, ops; api=2066 events=1808 data=2023 policy=4230 contract=2631 ops=2274
- capability-tier mapping: mentions=61; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2584; status=present
- contract version conformance: contract_mentions=481; status=evidenced
- threat-model: threat_mentions=750; status=evidenced
- test-plan: test_mentions=5015; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `design-collaboration`
- current-state: complete; artifacts=150; corpus_lines=18947
- surface coverage: api, events, data, policy, contract, ops; api=2099 events=2482 data=1392 policy=5450 contract=4525 ops=2554
- capability-tier mapping: mentions=11; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=3588; status=present
- contract version conformance: contract_mentions=1436; status=evidenced
- threat-model: threat_mentions=997; status=evidenced
- test-plan: test_mentions=5381; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `detection`
- current-state: complete; artifacts=127; corpus_lines=12388
- surface coverage: api, events, data, policy, contract, ops; api=1007 events=1101 data=365 policy=810 contract=1295 ops=547
- capability-tier mapping: mentions=1; status=mapped
- Cedar policy coverage: policy_files=21; policy_mentions=639; status=present
- contract version conformance: contract_mentions=804; status=evidenced
- threat-model: threat_mentions=320; status=evidenced
- test-plan: test_mentions=10; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `developer-sdk`
- current-state: complete; artifacts=137; corpus_lines=14303
- surface coverage: api, events, data, policy, contract, ops; api=1565 events=849 data=430 policy=4955 contract=1814 ops=1082
- capability-tier mapping: mentions=45; status=mapped
- Cedar policy coverage: policy_files=9; policy_mentions=2597; status=present
- contract version conformance: contract_mentions=275; status=evidenced
- threat-model: threat_mentions=83; status=evidenced
- test-plan: test_mentions=140; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `docs`
- current-state: complete; artifacts=128; corpus_lines=20224
- surface coverage: api, events, data, policy, contract, ops; api=1529 events=991 data=603 policy=5568 contract=2103 ops=1248
- capability-tier mapping: mentions=38; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3142; status=present
- contract version conformance: contract_mentions=619; status=evidenced
- threat-model: threat_mentions=157; status=evidenced
- test-plan: test_mentions=33; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `drive`
- current-state: complete; artifacts=175; corpus_lines=41157
- surface coverage: api, events, data, policy, contract, ops; api=2730 events=6505 data=3462 policy=24145 contract=6090 ops=2494
- capability-tier mapping: mentions=67; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=11219; status=present
- contract version conformance: contract_mentions=1791; status=evidenced
- threat-model: threat_mentions=312; status=evidenced
- test-plan: test_mentions=760; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `feature-flags`
- current-state: complete; artifacts=130; corpus_lines=16519
- surface coverage: api, events, data, policy, contract, ops; api=1499 events=1131 data=296 policy=6429 contract=1814 ops=1292
- capability-tier mapping: mentions=16; status=mapped
- Cedar policy coverage: policy_files=14; policy_mentions=3771; status=present
- contract version conformance: contract_mentions=237; status=evidenced
- threat-model: threat_mentions=264; status=evidenced
- test-plan: test_mentions=52; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `financial-planning`
- current-state: complete; artifacts=150; corpus_lines=20971
- surface coverage: api, events, data, policy, contract, ops; api=1737 events=3498 data=1498 policy=3827 contract=7947 ops=2072
- capability-tier mapping: mentions=27; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2241; status=present
- contract version conformance: contract_mentions=6048; status=evidenced
- threat-model: threat_mentions=738; status=evidenced
- test-plan: test_mentions=8163; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `finops-portal`
- current-state: complete; artifacts=158; corpus_lines=27297
- surface coverage: api, events, data, policy, contract, ops; api=1966 events=3169 data=1179 policy=14912 contract=3077 ops=5213
- capability-tier mapping: mentions=28; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=8821; status=present
- contract version conformance: contract_mentions=569; status=evidenced
- threat-model: threat_mentions=231; status=evidenced
- test-plan: test_mentions=637; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `forms`
- current-state: complete; artifacts=143; corpus_lines=20313
- surface coverage: api, events, data, policy, contract, ops; api=1701 events=2551 data=886 policy=10742 contract=2018 ops=1430
- capability-tier mapping: mentions=9; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=5118; status=present
- contract version conformance: contract_mentions=459; status=evidenced
- threat-model: threat_mentions=171; status=evidenced
- test-plan: test_mentions=48; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `foundry`
- current-state: complete; artifacts=589; corpus_lines=72823
- surface coverage: api, events, data, policy, contract, ops; api=4997 events=9000 data=2531 policy=16857 contract=4084 ops=3529
- capability-tier mapping: mentions=54; status=mapped
- Cedar policy coverage: policy_files=61; policy_mentions=10895; status=present
- contract version conformance: contract_mentions=884; status=evidenced
- threat-model: threat_mentions=627; status=evidenced
- test-plan: test_mentions=1091; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `global-trade`
- current-state: complete; artifacts=136; corpus_lines=14779
- surface coverage: api, events, data, policy, contract, ops; api=2146 events=4392 data=1485 policy=7878 contract=1925 ops=4185
- capability-tier mapping: mentions=4; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=5140; status=present
- contract version conformance: contract_mentions=1234; status=evidenced
- threat-model: threat_mentions=500; status=evidenced
- test-plan: test_mentions=501; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `governance`
- current-state: complete; artifacts=203; corpus_lines=29110
- surface coverage: api, events, data, policy, contract, ops; api=1986 events=2410 data=1199 policy=9781 contract=2904 ops=3813
- capability-tier mapping: mentions=31; status=mapped
- Cedar policy coverage: policy_files=29; policy_mentions=5714; status=present
- contract version conformance: contract_mentions=777; status=evidenced
- threat-model: threat_mentions=166; status=evidenced
- test-plan: test_mentions=162; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `healthcare-integration`
- current-state: complete; artifacts=157; corpus_lines=21138
- surface coverage: api, events, data, policy, contract, ops; api=2770 events=2609 data=1301 policy=4331 contract=2604 ops=2517
- capability-tier mapping: mentions=5; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2667; status=present
- contract version conformance: contract_mentions=475; status=evidenced
- threat-model: threat_mentions=830; status=evidenced
- test-plan: test_mentions=4794; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `identity`
- current-state: complete; artifacts=237; corpus_lines=74693
- surface coverage: api, events, data, policy, contract, ops; api=4528 events=11237 data=43003 policy=41354 contract=13294 ops=6377
- capability-tier mapping: mentions=98; status=mapped
- Cedar policy coverage: policy_files=12; policy_mentions=20413; status=present
- contract version conformance: contract_mentions=3757; status=evidenced
- threat-model: threat_mentions=693; status=evidenced
- test-plan: test_mentions=2481; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `incident-management`
- current-state: complete; artifacts=157; corpus_lines=16692
- surface coverage: api, events, data, policy, contract, ops; api=1345 events=3997 data=910 policy=4820 contract=2162 ops=12874
- capability-tier mapping: mentions=22; status=mapped
- Cedar policy coverage: policy_files=20; policy_mentions=3337; status=present
- contract version conformance: contract_mentions=262; status=evidenced
- threat-model: threat_mentions=664; status=evidenced
- test-plan: test_mentions=5532; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `intelligence`
- current-state: complete; artifacts=172; corpus_lines=37369
- surface coverage: api, events, data, policy, contract, ops; api=3187 events=4055 data=2636 policy=15298 contract=4827 ops=4992
- capability-tier mapping: mentions=99; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=8469; status=present
- contract version conformance: contract_mentions=838; status=evidenced
- threat-model: threat_mentions=390; status=evidenced
- test-plan: test_mentions=685; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `itsm`
- current-state: complete; artifacts=160; corpus_lines=18687
- surface coverage: api, events, data, policy, contract, ops; api=1858 events=1264 data=1661 policy=3503 contract=2675 ops=8906
- capability-tier mapping: mentions=57; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2072; status=present
- contract version conformance: contract_mentions=420; status=evidenced
- threat-model: threat_mentions=664; status=evidenced
- test-plan: test_mentions=5665; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `learning-management`
- current-state: complete; artifacts=149; corpus_lines=14679
- surface coverage: api, events, data, policy, contract, ops; api=1254 events=1047 data=948 policy=2901 contract=1917 ops=2084
- capability-tier mapping: mentions=35; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=1669; status=present
- contract version conformance: contract_mentions=215; status=evidenced
- threat-model: threat_mentions=662; status=evidenced
- test-plan: test_mentions=5311; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `mail`
- current-state: complete; artifacts=209; corpus_lines=56908
- surface coverage: api, events, data, policy, contract, ops; api=3485 events=10592 data=4684 policy=37100 contract=7969 ops=4461
- capability-tier mapping: mentions=37; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=17135; status=present
- contract version conformance: contract_mentions=2012; status=evidenced
- threat-model: threat_mentions=645; status=evidenced
- test-plan: test_mentions=1061; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `marketing-automation`
- current-state: complete; artifacts=161; corpus_lines=16181
- surface coverage: api, events, data, policy, contract, ops; api=1400 events=2142 data=941 policy=2942 contract=1979 ops=1953
- capability-tier mapping: mentions=47; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=1700; status=present
- contract version conformance: contract_mentions=251; status=evidenced
- threat-model: threat_mentions=667; status=evidenced
- test-plan: test_mentions=5319; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `marketplace`
- current-state: complete; artifacts=131; corpus_lines=22324
- surface coverage: api, events, data, policy, contract, ops; api=1720 events=3583 data=1355 policy=10733 contract=3080 ops=4762
- capability-tier mapping: mentions=66; status=mapped
- Cedar policy coverage: policy_files=11; policy_mentions=4673; status=present
- contract version conformance: contract_mentions=256; status=evidenced
- threat-model: threat_mentions=80; status=evidenced
- test-plan: test_mentions=948; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `meet`
- current-state: complete; artifacts=139; corpus_lines=24179
- surface coverage: api, events, data, policy, contract, ops; api=1622 events=3381 data=1128 policy=11691 contract=3233 ops=1550
- capability-tier mapping: mentions=21; status=mapped
- Cedar policy coverage: policy_files=11; policy_mentions=5622; status=present
- contract version conformance: contract_mentions=700; status=evidenced
- threat-model: threat_mentions=161; status=evidenced
- test-plan: test_mentions=183; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `messenger`
- current-state: complete; artifacts=166; corpus_lines=43838
- surface coverage: api, events, data, policy, contract, ops; api=2695 events=5452 data=4111 policy=20333 contract=5847 ops=2900
- capability-tier mapping: mentions=64; status=mapped
- Cedar policy coverage: policy_files=12; policy_mentions=10443; status=present
- contract version conformance: contract_mentions=2159; status=evidenced
- threat-model: threat_mentions=339; status=evidenced
- test-plan: test_mentions=751; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `network`
- current-state: complete; artifacts=127; corpus_lines=24021
- surface coverage: api, events, data, policy, contract, ops; api=1480 events=1499 data=799 policy=6227 contract=2078 ops=1396
- capability-tier mapping: mentions=15; status=mapped
- Cedar policy coverage: policy_files=9; policy_mentions=3598; status=present
- contract version conformance: contract_mentions=399; status=evidenced
- threat-model: threat_mentions=215; status=evidenced
- test-plan: test_mentions=64; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `notes`
- current-state: complete; artifacts=160; corpus_lines=26555
- surface coverage: api, events, data, policy, contract, ops; api=1767 events=3600 data=1857 policy=13972 contract=3288 ops=1814
- capability-tier mapping: mentions=16; status=mapped
- Cedar policy coverage: policy_files=14; policy_mentions=6631; status=present
- contract version conformance: contract_mentions=835; status=evidenced
- threat-model: threat_mentions=313; status=evidenced
- test-plan: test_mentions=261; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `observability`
- current-state: complete; artifacts=213; corpus_lines=45523
- surface coverage: api, events, data, policy, contract, ops; api=2937 events=3882 data=3453 policy=15763 contract=10058 ops=29692
- capability-tier mapping: mentions=64; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=8690; status=present
- contract version conformance: contract_mentions=1377; status=evidenced
- threat-model: threat_mentions=317; status=evidenced
- test-plan: test_mentions=982; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `ontology`
- current-state: complete; artifacts=155; corpus_lines=34506
- surface coverage: api, events, data, policy, contract, ops; api=2717 events=3001 data=21394 policy=13656 contract=2924 ops=4719
- capability-tier mapping: mentions=664; status=mapped
- Cedar policy coverage: policy_files=21; policy_mentions=7865; status=present
- contract version conformance: contract_mentions=322; status=evidenced
- threat-model: threat_mentions=342; status=evidenced
- test-plan: test_mentions=584; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `ops-dashboard-control-center`
- current-state: complete; artifacts=154; corpus_lines=23377
- surface coverage: api, events, data, policy, contract, ops; api=2086 events=2267 data=1046 policy=10746 contract=2975 ops=14271
- capability-tier mapping: mentions=14; status=mapped
- Cedar policy coverage: policy_files=26; policy_mentions=6250; status=present
- contract version conformance: contract_mentions=738; status=evidenced
- threat-model: threat_mentions=350; status=evidenced
- test-plan: test_mentions=63; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `payments`
- current-state: complete; artifacts=198; corpus_lines=60200
- surface coverage: api, events, data, policy, contract, ops; api=5198 events=9332 data=4535 policy=31075 contract=8033 ops=7319
- capability-tier mapping: mentions=73; status=mapped
- Cedar policy coverage: policy_files=11; policy_mentions=16222; status=present
- contract version conformance: contract_mentions=1850; status=evidenced
- threat-model: threat_mentions=1062; status=evidenced
- test-plan: test_mentions=2625; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `performance-management`
- current-state: complete; artifacts=158; corpus_lines=16140
- surface coverage: api, events, data, policy, contract, ops; api=1324 events=1144 data=941 policy=2950 contract=1949 ops=1947
- capability-tier mapping: mentions=31; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=1714; status=present
- contract version conformance: contract_mentions=229; status=evidenced
- threat-model: threat_mentions=801; status=evidenced
- test-plan: test_mentions=5311; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `plant-maintenance`
- current-state: complete; artifacts=137; corpus_lines=19169
- surface coverage: api, events, data, policy, contract, ops; api=2160 events=3627 data=1610 policy=7583 contract=1104 ops=4500
- capability-tier mapping: mentions=7; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=4750; status=present
- contract version conformance: contract_mentions=409; status=evidenced
- threat-model: threat_mentions=501; status=evidenced
- test-plan: test_mentions=763; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `plugin-app-store`
- current-state: complete; artifacts=147; corpus_lines=19680
- surface coverage: api, events, data, policy, contract, ops; api=1548 events=2615 data=955 policy=9921 contract=2120 ops=1716
- capability-tier mapping: mentions=28; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=5264; status=present
- contract version conformance: contract_mentions=320; status=evidenced
- threat-model: threat_mentions=114; status=evidenced
- test-plan: test_mentions=470; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `production-planning`
- current-state: complete; artifacts=149; corpus_lines=20047
- surface coverage: api, events, data, policy, contract, ops; api=2220 events=3832 data=1552 policy=7491 contract=1234 ops=4305
- capability-tier mapping: mentions=16; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=4557; status=present
- contract version conformance: contract_mentions=504; status=evidenced
- threat-model: threat_mentions=502; status=evidenced
- test-plan: test_mentions=868; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `quality-management`
- current-state: complete; artifacts=137; corpus_lines=17614
- surface coverage: api, events, data, policy, contract, ops; api=2100 events=3883 data=1659 policy=7220 contract=1053 ops=4133
- capability-tier mapping: mentions=6; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=4558; status=present
- contract version conformance: contract_mentions=309; status=evidenced
- threat-model: threat_mentions=502; status=evidenced
- test-plan: test_mentions=546; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `real-estate`
- current-state: complete; artifacts=137; corpus_lines=15192
- surface coverage: api, events, data, policy, contract, ops; api=2030 events=5213 data=1559 policy=6933 contract=3047 ops=4170
- capability-tier mapping: mentions=3; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=4323; status=present
- contract version conformance: contract_mentions=283; status=evidenced
- threat-model: threat_mentions=500; status=evidenced
- test-plan: test_mentions=469; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `recordings`
- current-state: complete; artifacts=128; corpus_lines=18923
- surface coverage: api, events, data, policy, contract, ops; api=1238 events=1045 data=795 policy=6834 contract=2197 ops=1169
- capability-tier mapping: mentions=8; status=mapped
- Cedar policy coverage: policy_files=12; policy_mentions=3822; status=present
- contract version conformance: contract_mentions=233; status=evidenced
- threat-model: threat_mentions=136; status=evidenced
- test-plan: test_mentions=76; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `sheets`
- current-state: complete; artifacts=126; corpus_lines=21686
- surface coverage: api, events, data, policy, contract, ops; api=1398 events=905 data=618 policy=5791 contract=1932 ops=1257
- capability-tier mapping: mentions=30; status=mapped
- Cedar policy coverage: policy_files=10; policy_mentions=3233; status=present
- contract version conformance: contract_mentions=464; status=evidenced
- threat-model: threat_mentions=234; status=evidenced
- test-plan: test_mentions=53; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `shorts`
- current-state: complete; artifacts=125; corpus_lines=23562
- surface coverage: api, events, data, policy, contract, ops; api=1475 events=1314 data=636 policy=6477 contract=2148 ops=1426
- capability-tier mapping: mentions=14; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3867; status=present
- contract version conformance: contract_mentions=308; status=evidenced
- threat-model: threat_mentions=229; status=evidenced
- test-plan: test_mentions=78; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `sites`
- current-state: complete; artifacts=123; corpus_lines=19516
- surface coverage: api, events, data, policy, contract, ops; api=1371 events=835 data=583 policy=5346 contract=1848 ops=1272
- capability-tier mapping: mentions=11; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=2969; status=present
- contract version conformance: contract_mentions=368; status=evidenced
- threat-model: threat_mentions=180; status=evidenced
- test-plan: test_mentions=32; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `slides`
- current-state: complete; artifacts=129; corpus_lines=18128
- surface coverage: api, events, data, policy, contract, ops; api=1270 events=806 data=552 policy=5339 contract=1641 ops=1147
- capability-tier mapping: mentions=6; status=mapped
- Cedar policy coverage: policy_files=7; policy_mentions=2920; status=present
- contract version conformance: contract_mentions=293; status=evidenced
- threat-model: threat_mentions=148; status=evidenced
- test-plan: test_mentions=29; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `social`
- current-state: complete; artifacts=145; corpus_lines=23351
- surface coverage: api, events, data, policy, contract, ops; api=1517 events=1380 data=666 policy=7124 contract=2207 ops=1619
- capability-tier mapping: mentions=17; status=mapped
- Cedar policy coverage: policy_files=17; policy_mentions=4260; status=present
- contract version conformance: contract_mentions=271; status=evidenced
- threat-model: threat_mentions=508; status=evidenced
- test-plan: test_mentions=49; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `supply-chain-planning`
- current-state: complete; artifacts=135; corpus_lines=15776
- surface coverage: api, events, data, policy, contract, ops; api=2006 events=4267 data=1320 policy=7776 contract=1823 ops=4165
- capability-tier mapping: mentions=14; status=mapped
- Cedar policy coverage: policy_files=15; policy_mentions=5109; status=present
- contract version conformance: contract_mentions=1124; status=evidenced
- threat-model: threat_mentions=501; status=evidenced
- test-plan: test_mentions=4559; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `tasks`
- current-state: complete; artifacts=124; corpus_lines=20692
- surface coverage: api, events, data, policy, contract, ops; api=1495 events=1326 data=789 policy=5592 contract=1738 ops=1280
- capability-tier mapping: mentions=27; status=mapped
- Cedar policy coverage: policy_files=8; policy_mentions=3135; status=present
- contract version conformance: contract_mentions=243; status=evidenced
- threat-model: threat_mentions=139; status=evidenced
- test-plan: test_mentions=69; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `tenancy`
- current-state: complete; artifacts=194; corpus_lines=47549
- surface coverage: api, events, data, policy, contract, ops; api=3408 events=6191 data=3732 policy=23277 contract=6490 ops=5637
- capability-tier mapping: mentions=3; status=mapped
- Cedar policy coverage: policy_files=17; policy_mentions=12866; status=present
- contract version conformance: contract_mentions=1776; status=evidenced
- threat-model: threat_mentions=405; status=evidenced
- test-plan: test_mentions=1286; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `translate`
- current-state: complete; artifacts=122; corpus_lines=19721
- surface coverage: api, events, data, policy, contract, ops; api=1803 events=1632 data=558 policy=6358 contract=13456 ops=1358
- capability-tier mapping: mentions=1; status=mapped
- Cedar policy coverage: policy_files=7; policy_mentions=3367; status=present
- contract version conformance: contract_mentions=266; status=evidenced
- threat-model: threat_mentions=148; status=evidenced
- test-plan: test_mentions=74; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `treasury`
- current-state: complete; artifacts=137; corpus_lines=14852
- surface coverage: api, events, data, policy, contract, ops; api=2217 events=4384 data=1507 policy=8158 contract=1819 ops=4178
- capability-tier mapping: mentions=7; status=mapped
- Cedar policy coverage: policy_files=17; policy_mentions=5412; status=present
- contract version conformance: contract_mentions=1135; status=evidenced
- threat-model: threat_mentions=500; status=evidenced
- test-plan: test_mentions=530; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `warehouse`
- current-state: complete; artifacts=137; corpus_lines=15949
- surface coverage: api, events, data, policy, contract, ops; api=2187 events=3601 data=1584 policy=7232 contract=1069 ops=4138
- capability-tier mapping: mentions=15; status=mapped
- Cedar policy coverage: policy_files=16; policy_mentions=4485; status=present
- contract version conformance: contract_mentions=306; status=evidenced
- threat-model: threat_mentions=528; status=evidenced
- test-plan: test_mentions=482; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `whiteboard`
- current-state: complete; artifacts=149; corpus_lines=21237
- surface coverage: api, events, data, policy, contract, ops; api=1701 events=1789 data=1309 policy=4675 contract=2480 ops=2332
- capability-tier mapping: mentions=9; status=mapped
- Cedar policy coverage: policy_files=19; policy_mentions=2924; status=present
- contract version conformance: contract_mentions=381; status=evidenced
- threat-model: threat_mentions=880; status=evidenced
- test-plan: test_mentions=5037; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `workflow-engine`
- current-state: complete; artifacts=226; corpus_lines=65082
- surface coverage: api, events, data, policy, contract, ops; api=4854 events=12710 data=5507 policy=41580 contract=9408 ops=7203
- capability-tier mapping: mentions=133; status=mapped
- Cedar policy coverage: policy_files=9; policy_mentions=19519; status=present
- contract version conformance: contract_mentions=2964; status=evidenced
- threat-model: threat_mentions=413; status=evidenced
- test-plan: test_mentions=2310; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `workflow-studio`
- current-state: complete; artifacts=220; corpus_lines=40627
- surface coverage: api, events, data, policy, contract, ops; api=2338 events=1805 data=1298 policy=8307 contract=3655 ops=4294
- capability-tier mapping: mentions=25; status=mapped
- Cedar policy coverage: policy_files=12; policy_mentions=4982; status=present
- contract version conformance: contract_mentions=730; status=evidenced
- threat-model: threat_mentions=237; status=evidenced
- test-plan: test_mentions=442; status=evidenced
- named gaps: none named; retain in regression audit
### µservice `workplace-integration`
- current-state: complete; artifacts=132; corpus_lines=22759
- surface coverage: api, events, data, policy, contract, ops; api=1807 events=3189 data=1668 policy=10162 contract=3486 ops=4796
- capability-tier mapping: mentions=43; status=mapped
- Cedar policy coverage: policy_files=12; policy_mentions=4568; status=present
- contract version conformance: contract_mentions=493; status=evidenced
- threat-model: threat_mentions=70; status=evidenced
- test-plan: test_mentions=938; status=evidenced
- named gaps: none named; retain in regression audit

## §4 Journey Coverage Scorecard (j01..j180)
- Journey directories discovered: 180
- Expected journey range: j01..j180
- Corpus-level named gap: presence is complete for j01..j180
### j01 — j01-emergency-911-dispatch
- file count: 13; line count: 4201
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=api-gateway, audit-chain, calendar, cell, comms-email, compliance, connect, consent-graph, docs, drive
- regulatory anchor density: 87 anchors / 4201 lines = 2.07 per 100 lines
- named gaps: missing explicit persona field
### j02 — j02-healthcare-code-blue-ehr-break-glass
- file count: 11; line count: 3535
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=api-gateway, audit-chain, cell, compliance, consent-graph, docs, foundry, identity, intelligence, mail
- regulatory anchor density: 71 anchors / 3535 lines = 2.01 per 100 lines
- named gaps: missing explicit persona field
### j03 — j03-988-crisis-line-minor-self-report
- file count: 10; line count: 3269
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, audit-chain, cell, community, compliance, connect, consent-graph, docs, drive, foundry
- regulatory anchor density: 8 anchors / 3269 lines = 0.24 per 100 lines
- named gaps: missing explicit persona field
### j04 — j04-dv-survivor-shelter-mode
- file count: 8; line count: 3289
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, compliance, consent-graph, docs, drive, foundry, identity, mail
- regulatory anchor density: 5 anchors / 3289 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j05 — j05-whistleblower-anonymous-ethics-report
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, community, compliance, docs, foundry, identity, mail, messenger
- regulatory anchor density: 5 anchors / 3285 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j06 — j06-press-source-securedrop-class
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, community, compliance, docs, drive, foundry, identity, mail
- regulatory anchor density: 5 anchors / 3285 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j07 — j07-deceased-user-inheritance-handoff
- file count: 8; line count: 3293
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=api-gateway, audit-chain, cell, compliance, docs, drive, foundry, identity, mail, messenger
- regulatory anchor density: 5 anchors / 3293 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j08 — j08-elder-financial-abuse-detection
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, compliance, detection, docs, foundry, identity, mail, messenger
- regulatory anchor density: 5 anchors / 3285 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j09 — j09-account-recovery-phishing-resistant
- file count: 8; line count: 3277
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, identity, mail, messenger, network
- regulatory anchor density: 79 anchors / 3277 lines = 2.41 per 100 lines
- named gaps: missing explicit persona field
### j10 — j10-account-takeover-SIM-swap-detected
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, identity, mail, messenger, network
- regulatory anchor density: 5 anchors / 3285 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j100 — j100-pack-rollout-from-tenant-onboarding-to-first-action
- file count: 9; line count: 2809
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 473 anchors / 2809 lines = 16.84 per 100 lines
- named gaps: missing explicit persona field
### j101 — j101-multi-tier-supply-chain-formation
- file count: 6; line count: 5764
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, compliance, connect, detection, docs, identity, mail, marketplace, network, observability
- regulatory anchor density: 6 anchors / 5764 lines = 0.1 per 100 lines
- named gaps: missing explicit persona field
### j102 — j102-raw-material-purchase-with-quality-attestation
- file count: 6; line count: 5732
- persona resolved: inferred-from-title-only
- µservice citations: 13 detected; sample=audit-chain, compliance, connect, detection, docs, drive, identity, marketplace, network, observability
- regulatory anchor density: 11 anchors / 5732 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j103 — j103-just-in-time-procurement-automation
- file count: 6; line count: 5771
- persona resolved: inferred-from-title-only
- µservice citations: 12 detected; sample=audit-chain, compliance, connect, detection, docs, identity, marketplace, network, observability, ontology
- regulatory anchor density: 11 anchors / 5771 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j104 — j104-supplier-vendor-onboarding-kyb-cascade
- file count: 6; line count: 5724
- persona resolved: inferred-from-title-only
- µservice citations: 12 detected; sample=audit-chain, compliance, connect, detection, docs, identity, marketplace, network, observability, ontology
- regulatory anchor density: 11 anchors / 5724 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j105 — j105-dispute-cross-tenant-arbitration
- file count: 6; line count: 5756
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=audit-chain, compliance, connect, detection, docs, drive, identity, mail, marketplace, messenger
- regulatory anchor density: 406 anchors / 5756 lines = 7.05 per 100 lines
- named gaps: missing explicit persona field
### j106 — j106-multi-currency-cross-border-payment
- file count: 6; line count: 5712
- persona resolved: inferred-from-title-only
- µservice citations: 13 detected; sample=audit-chain, compliance, connect, detection, docs, identity, marketplace, network, observability, ontology
- regulatory anchor density: 17 anchors / 5712 lines = 0.3 per 100 lines
- named gaps: missing explicit persona field
### j107 — j107-supply-chain-disruption-and-failover
- file count: 6; line count: 5753
- persona resolved: inferred-from-title-only
- µservice citations: 12 detected; sample=audit-chain, compliance, connect, detection, docs, identity, mail, marketplace, network, observability
- regulatory anchor density: 6 anchors / 5753 lines = 0.1 per 100 lines
- named gaps: missing explicit persona field
### j108 — j108-supplier-rating-and-marketplace-discovery
- file count: 6; line count: 5759
- persona resolved: inferred-from-title-only
- µservice citations: 13 detected; sample=audit-chain, community, compliance, connect, detection, docs, identity, intelligence, marketplace, network
- regulatory anchor density: 16 anchors / 5759 lines = 0.28 per 100 lines
- named gaps: missing explicit persona field
### j109 — j109-construction-co-hires-freelance-specialist
- file count: 6; line count: 5888
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, community, compliance, connect, detection, docs, identity, marketplace, network, observability
- regulatory anchor density: 6 anchors / 5888 lines = 0.1 per 100 lines
- named gaps: missing explicit persona field
### j11 — j11-disaster-zone-offline-first-sync
- file count: 8; line count: 3289
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=api-gateway, audit-chain, cell, compliance, connect, docs, drive, foundry, identity, mail
- regulatory anchor density: 5 anchors / 3289 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j110 — j110-traveling-nurse-multi-employer-roster
- file count: 6; line count: 5757
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=audit-chain, community, compliance, connect, detection, docs, identity, marketplace, network, observability
- regulatory anchor density: 11 anchors / 5757 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j111 — j111-staffing-agency-as-tenant-facilitator
- file count: 6; line count: 5750
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, community, compliance, connect, detection, docs, identity, marketplace, network, observability
- regulatory anchor density: 11 anchors / 5750 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j112 — j112-tenant-to-tenant-rfq-and-bid
- file count: 6; line count: 5764
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, community, compliance, connect, detection, docs, identity, marketplace, network, observability
- regulatory anchor density: 6 anchors / 5764 lines = 0.1 per 100 lines
- named gaps: missing explicit persona field
### j113 — j113-cross-tenant-internship-from-handshake
- file count: 6; line count: 5768
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=audit-chain, calendar, community, compliance, connect, detection, docs, identity, marketplace, messenger
- regulatory anchor density: 6 anchors / 5768 lines = 0.1 per 100 lines
- named gaps: missing explicit persona field
### j114 — j114-employee-secondment-cross-tenant
- file count: 6; line count: 5755
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, compliance, connect, detection, docs, identity, marketplace, network, observability, ontology
- regulatory anchor density: 11 anchors / 5755 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j115 — j115-saas-vendor-sells-api-to-multiple-tenant-customers
- file count: 6; line count: 5801
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, compliance, connect, detection, docs, finops-portal, identity, marketplace, network, observability
- regulatory anchor density: 21 anchors / 5801 lines = 0.36 per 100 lines
- named gaps: missing explicit persona field
### j116 — j116-plugin-marketplace-developer-publishes-and-monetizes
- file count: 8; line count: 3119
- persona resolved: inferred-from-title-only
- µservice citations: 20 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3119 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j117 — j117-api-customer-tenant-incident-response
- file count: 8; line count: 3125
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3125 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j118 — j118-tenant-to-tenant-data-sharing-via-ontology-projection
- file count: 8; line count: 3119
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3119 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j119 — j119-invoice-financing-marketplace
- file count: 8; line count: 3125
- persona resolved: inferred-from-title-only
- µservice citations: 20 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3125 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j12 — j12-mass-casualty-incident-10x-traffic
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, identity, mail, messenger, network
- regulatory anchor density: 5 anchors / 3285 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j120 — j120-tenant-treasury-multi-currency-fx-hedge
- file count: 8; line count: 3119
- persona resolved: inferred-from-title-only
- µservice citations: 20 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3119 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j121 — j121-business-loan-application-from-bank-tenant
- file count: 8; line count: 3131
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3131 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j122 — j122-vendor-payment-batch-with-tax-withholding
- file count: 8; line count: 3125
- persona resolved: inferred-from-title-only
- µservice citations: 20 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3125 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j123 — j123-multi-tenant-coordinated-product-launch
- file count: 8; line count: 3131
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, drive, finops-portal
- regulatory anchor density: 4 anchors / 3131 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j124 — j124-supply-chain-disruption-emergency-coordination
- file count: 8; line count: 3119
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3119 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j125 — j125-marketplace-acquires-supplier-tenant-merger
- file count: 8; line count: 3137
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, drive, finops-portal
- regulatory anchor density: 4 anchors / 3137 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j126 — j126-government-auditor-3pao-conducts-fedramp-audit
- file count: 10; line count: 3441
- persona resolved: Diana Reyes.
- µservice citations: 25 detected; sample=api-gateway, application, audit-chain, calendar, cell, comms-email, community, compliance, connect, detection
- regulatory anchor density: 247 anchors / 3441 lines = 7.18 per 100 lines
- named gaps: none named; retain in regression audit
### j127 — j127-dual-tenant-identity-employee-resigns-and-keeps-personal
- file count: 8; line count: 2753
- persona resolved: Marcus tenant engineer.
- µservice citations: 20 detected; sample=api-gateway, audit-chain, calendar, cell, comms-email, compliance, detection, docs, drive, identity
- regulatory anchor density: 124 anchors / 2753 lines = 4.5 per 100 lines
- named gaps: none named; retain in regression audit
### j128 — j128-auditor-personal-side-uses-workflow-studio-for-family-taxes
- file count: 8; line count: 2727
- persona resolved: Diana Reyes.
- µservice citations: 18 detected; sample=api-gateway, audit-chain, calendar, cell, compliance, connect, docs, drive, identity, intelligence
- regulatory anchor density: 115 anchors / 2727 lines = 4.22 per 100 lines
- named gaps: none named; retain in regression audit
### j129 — j129-court-warrant-pierces-personal-tenant-with-judicial-oversight
- file count: 8; line count: 2759
- persona resolved: Diana Reyes.
- µservice citations: 20 detected; sample=api-gateway, audit-chain, cell, comms-email, community, compliance, connect, detection, docs, drive
- regulatory anchor density: 116 anchors / 2759 lines = 4.2 per 100 lines
- named gaps: none named; retain in regression audit
### j13 — j13-cross-jurisdiction-eu-cloud-act-conflict
- file count: 8; line count: 3281
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, identity, intelligence, mail, messenger
- regulatory anchor density: 20 anchors / 3281 lines = 0.61 per 100 lines
- named gaps: missing explicit persona field
### j130 — j130-auditor-receives-bribery-attempt-via-personal-messenger
- file count: 8; line count: 2747
- persona resolved: Diana Reyes.
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, comms-email, community, compliance, connect, docs, governance, identity
- regulatory anchor density: 139 anchors / 2747 lines = 5.06 per 100 lines
- named gaps: none named; retain in regression audit
### j131 — j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy
- file count: 8; line count: 2737
- persona resolved: Diana Reyes.
- µservice citations: 13 detected; sample=api-gateway, audit-chain, cell, comms-email, compliance, connect, detection, docs, identity, network
- regulatory anchor density: 218 anchors / 2737 lines = 7.96 per 100 lines
- named gaps: none named; retain in regression audit
### j132 — j132-hr-mass-hiring-event-100-roles
- file count: 11; line count: 3027
- persona resolved: Priya Krishnan.
- µservice citations: 28 detected; sample=api-gateway, application, audit-chain, calendar, cell, cloud-iac, community, compliance, connect, detection
- regulatory anchor density: 178 anchors / 3027 lines = 5.88 per 100 lines
- named gaps: none named; retain in regression audit
### j133 — j133-hr-conducts-layoff-with-dignity-and-compliance
- file count: 11; line count: 3034
- persona resolved: Priya Krishnan.
- µservice citations: 26 detected; sample=audit-chain, calendar, cell, cloud-iac, community, compliance, connect, detection, docs, drive
- regulatory anchor density: 79 anchors / 3034 lines = 2.6 per 100 lines
- named gaps: none named; retain in regression audit
### j134 — j134-hr-cross-tenant-recruitment-via-staffing-agency
- file count: 10; line count: 2864
- persona resolved: Priya Krishnan.
- µservice citations: 19 detected; sample=audit-chain, calendar, cell, community, compliance, connect, drive, finops-portal, identity, mail
- regulatory anchor density: 85 anchors / 2864 lines = 2.97 per 100 lines
- named gaps: none named; retain in regression audit
### j135 — j135-hr-handles-harassment-complaint-with-dual-tenant-boundary
- file count: 10; line count: 2898
- persona resolved: Priya Krishnan.
- µservice citations: 16 detected; sample=audit-chain, calendar, cell, community, compliance, connect, drive, identity, mail, marketplace
- regulatory anchor density: 102 anchors / 2898 lines = 3.52 per 100 lines
- named gaps: none named; retain in regression audit
### j136 — j136-hr-administers-benefits-open-enrollment
- file count: 10; line count: 2865
- persona resolved: Priya Krishnan.
- µservice citations: 20 detected; sample=audit-chain, calendar, cell, community, compliance, connect, docs, drive, finops-portal, forms
- regulatory anchor density: 96 anchors / 2865 lines = 3.35 per 100 lines
- named gaps: none named; retain in regression audit
### j137 — j137-corporate-internal-audit-sox-controls-test
- file count: 10; line count: 3297
- persona resolved: Sam Okafor.
- µservice citations: 19 detected; sample=api-gateway, application, audit-chain, calendar, cell, compliance, detection, docs, governance, identity
- regulatory anchor density: 200 anchors / 3297 lines = 6.07 per 100 lines
- named gaps: none named; retain in regression audit
### j138 — j138-corporate-audit-fraud-investigation-via-pattern-detection
- file count: 8; line count: 2826
- persona resolved: Sam Okafor.
- µservice citations: 19 detected; sample=api-gateway, audit-chain, cell, community, compliance, connect, detection, docs, drive, governance
- regulatory anchor density: 102 anchors / 2826 lines = 3.61 per 100 lines
- named gaps: none named; retain in regression audit
### j139 — j139-internal-audit-policy-violation-cedar-permit-misuse
- file count: 8; line count: 2752
- persona resolved: Sam Okafor.
- µservice citations: 14 detected; sample=audit-chain, cell, community, detection, governance, identity, mail, meet, network, notes
- regulatory anchor density: 126 anchors / 2752 lines = 4.58 per 100 lines
- named gaps: none named; retain in regression audit
### j14 — j14-delegated-llm-agent-acting-for-yejin
- file count: 8; line count: 3289
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, identity, intelligence, mail, messenger
- regulatory anchor density: 5 anchors / 3289 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j140 — j140-internal-audit-data-loss-prevention-egress-trip
- file count: 8; line count: 2768
- persona resolved: Sam Okafor.
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, community, detection, docs, drive, identity, mail, meet
- regulatory anchor density: 112 anchors / 2768 lines = 4.05 per 100 lines
- named gaps: none named; retain in regression audit
### j141 — j141-internal-audit-respects-employee-personal-tenant-boundary
- file count: 8; line count: 2859
- persona resolved: Sam Okafor.
- µservice citations: 11 detected; sample=audit-chain, cell, compliance, docs, governance, identity, mail, meet, messenger, notes
- regulatory anchor density: 163 anchors / 2859 lines = 5.7 per 100 lines
- named gaps: none named; retain in regression audit
### j142 — j142-layoff-day-zero-from-employees-side
- file count: 8; line count: 2797
- persona resolved: Chris Volkov.
- µservice citations: 22 detected; sample=audit-chain, calendar, cell, community, compliance, connect, detection, docs, drive, finops-portal
- regulatory anchor density: 73 anchors / 2797 lines = 2.61 per 100 lines
- named gaps: none named; retain in regression audit
### j143 — j143-laid-off-imports-work-portfolio-into-personal-tenant
- file count: 6; line count: 2711
- persona resolved: Chris Volkov.
- µservice citations: 12 detected; sample=application, audit-chain, cell, compliance, drive, identity, mail, network, notes, ops-dashboard-control-center
- regulatory anchor density: 92 anchors / 2711 lines = 3.39 per 100 lines
- named gaps: none named; retain in regression audit
### j144 — j144-laid-off-builds-job-search-pipeline-in-workflow-studio
- file count: 6; line count: 2704
- persona resolved: Chris Volkov.
- µservice citations: 16 detected; sample=application, audit-chain, calendar, cell, community, compliance, connect, detection, intelligence, mail
- regulatory anchor density: 74 anchors / 2704 lines = 2.74 per 100 lines
- named gaps: none named; retain in regression audit
### j145 — j145-laid-off-applies-via-community-handshake-linkedin-mode
- file count: 6; line count: 2694
- persona resolved: Chris Volkov.
- µservice citations: 20 detected; sample=application, audit-chain, calendar, cell, community, compliance, connect, drive, identity, intelligence
- regulatory anchor density: 86 anchors / 2694 lines = 3.19 per 100 lines
- named gaps: none named; retain in regression audit
### j146 — j146-laid-off-uses-marketplace-as-temporary-income
- file count: 6; line count: 2710
- persona resolved: Chris Volkov.
- µservice citations: 16 detected; sample=audit-chain, cell, community, compliance, connect, drive, finops-portal, identity, intelligence, mail
- regulatory anchor density: 114 anchors / 2710 lines = 4.21 per 100 lines
- named gaps: none named; retain in regression audit
### j147 — j147-laid-off-cohort-mutual-aid-community-channel
- file count: 6; line count: 2703
- persona resolved: Chris Volkov.
- µservice citations: 16 detected; sample=application, audit-chain, cell, community, detection, forms, governance, identity, mail, marketplace
- regulatory anchor density: 116 anchors / 2703 lines = 4.29 per 100 lines
- named gaps: none named; retain in regression audit
### j148 — j148-supply-chain-circular-economy-electronics-recycling
- file count: 8; line count: 3131
- persona resolved: inferred-from-title-only
- µservice citations: 19 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3131 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j149 — j149-gig-economy-multi-platform-worker
- file count: 8; line count: 3131
- persona resolved: inferred-from-title-only
- µservice citations: 22 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, forms
- regulatory anchor density: 4 anchors / 3131 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j15 — j15-bug-bounty-researcher-submission
- file count: 8; line count: 3274
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, community, compliance, docs, foundry, identity, mail, messenger
- regulatory anchor density: 5 anchors / 3274 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j150 — j150-creator-economy-shorts-creator-monetization-stack
- file count: 8; line count: 3137
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=api-gateway, application, audit-chain, cell, community, compliance, connect, docs, finops-portal, foundry
- regulatory anchor density: 4 anchors / 3137 lines = 0.13 per 100 lines
- named gaps: missing explicit persona field
### j151 — j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow
- file count: 1; line count: 176
- persona resolved: inferred-from-title-only
- µservice citations: 12 detected; sample=audit-chain, cell, compliance, connect, docs, finops-portal, identity, messenger, observability, payments
- regulatory anchor density: 2 anchors / 176 lines = 1.14 per 100 lines
- named gaps: missing explicit persona field
### j152 — j152-ahmad-hassan-construction-site-incident-bilingual
- file count: 10; line count: 2505
- persona resolved: inferred-from-title-only
- µservice citations: 20 detected; sample=application, audit-chain, cell, compliance, connect, docs, drive, forms, identity, incident-management
- regulatory anchor density: 17 anchors / 2505 lines = 0.68 per 100 lines
- named gaps: missing explicit persona field
### j153 — j153-devon-williams-hvac-side-business-tax-end-of-year
- file count: 10; line count: 2096
- persona resolved: inferred-from-title-only
- µservice citations: 19 detected; sample=application, audit-chain, cell, community, compliance, connect, docs, drive, finops-portal, identity
- regulatory anchor density: 33 anchors / 2096 lines = 1.57 per 100 lines
- named gaps: missing explicit persona field
### j154 — j154-tomas-pieter-channel-partner-co-marketing-launch
- file count: 10; line count: 3297
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=analytics, application, audit-chain, cell, comms-email, community, compliance, connect, consent-graph, crm
- regulatory anchor density: 126 anchors / 3297 lines = 3.82 per 100 lines
- named gaps: missing explicit persona field
### j155 — j155-stefan-kovacs-college-night-shift-and-finals-week
- file count: 10; line count: 2975
- persona resolved: inferred-from-title-only
- µservice citations: 24 detected; sample=analytics, application, audit-chain, calendar, cell, community, compliance, consent-graph, detection, docs
- regulatory anchor density: 73 anchors / 2975 lines = 2.45 per 100 lines
- named gaps: missing explicit persona field
### j156 — j156-carlos-reyes-ii-maintenance-emergency-after-hours
- file count: 10; line count: 2962
- persona resolved: inferred-from-title-only
- µservice citations: 24 detected; sample=application, audit-chain, cell, compliance, connect, consent-graph, detection, docs, drive, forms
- regulatory anchor density: 88 anchors / 2962 lines = 2.97 per 100 lines
- named gaps: missing explicit persona field
### j157 — j157-diana-lazar-print-operator-batch-defect-and-quality-recall
- file count: 10; line count: 2991
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=analytics, application, audit-chain, calendar, cell, compliance, contract-lifecycle-management, crm, detection, docs
- regulatory anchor density: 76 anchors / 2991 lines = 2.54 per 100 lines
- named gaps: missing explicit persona field
### j158 — j158-print-shop-cell-rebalance-shorts-creator-spike
- file count: 10; line count: 2968
- persona resolved: inferred-from-title-only
- µservice citations: 22 detected; sample=analytics, application, audit-chain, cell, compliance, crm, docs, drive, governance, identity
- regulatory anchor density: 116 anchors / 2968 lines = 3.91 per 100 lines
- named gaps: missing explicit persona field
### j159 — j159-saanvi-mehta-mba-application-spans-personal-and-work
- file count: 10; line count: 4084
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=analytics, application, audit-chain, calendar, cell, community, compliance, connect, crm, docs
- regulatory anchor density: 142 anchors / 4084 lines = 3.48 per 100 lines
- named gaps: missing explicit persona field
### j16 — j16-disability-accommodation-voice-only-signup
- file count: 8; line count: 3281
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, application, audit-chain, cell, compliance, docs, foundry, identity, intelligence, mail
- regulatory anchor density: 5 anchors / 3281 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j160 — j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
- file count: 10; line count: 4025
- persona resolved: inferred-from-title-only
- µservice citations: 26 detected; sample=analytics, application, audit-chain, calendar, cell, community, compliance, contract-lifecycle-management, crm, docs
- regulatory anchor density: 156 anchors / 4025 lines = 3.88 per 100 lines
- named gaps: missing explicit persona field
### j161 — j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
- file count: 10; line count: 4055
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=analytics, application, audit-chain, cell, community, compliance, connect, contract-lifecycle-management, crm, detection
- regulatory anchor density: 188 anchors / 4055 lines = 4.64 per 100 lines
- named gaps: missing explicit persona field
### j162 — j162-print-operator-diana-lazar-night-shift-onboarding
- file count: 10; line count: 3792
- persona resolved: inferred-from-title-only
- µservice citations: 24 detected; sample=analytics, application, audit-chain, cell, compliance, crm, data-warehouse, docs, drive, forms
- regulatory anchor density: 120 anchors / 3792 lines = 3.16 per 100 lines
- named gaps: missing explicit persona field
### j163 — j163-av-coordinator-jordan-park-board-meeting-cross-time-zone
- file count: 10; line count: 3685
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=application, audit-chain, calendar, cell, compliance, docs, drive, governance, identity, intelligence
- regulatory anchor density: 204 anchors / 3685 lines = 5.54 per 100 lines
- named gaps: missing explicit persona field
### j164 — j164-retired-hiroshi-tanaka-yearly-tax-and-pension
- file count: 10; line count: 3615
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=application, audit-chain, calendar, cell, compliance, detection, docs, drive, identity, intelligence
- regulatory anchor density: 32 anchors / 3615 lines = 0.89 per 100 lines
- named gaps: missing explicit persona field
### j165 — j165-cco-naveen-iyer-board-quarterly-compliance-report
- file count: 10; line count: 3186
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=application, audit-chain, calendar, cell, compliance, docs, drive, governance, identity, intelligence
- regulatory anchor density: 261 anchors / 3186 lines = 8.19 per 100 lines
- named gaps: missing explicit persona field
### j166 — j166-cso-mira-goldberg-strategic-acquisition-go-no-go
- file count: 10; line count: 3589
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=application, audit-chain, cell, community, compliance, connect, detection, docs, drive, financial-planning
- regulatory anchor density: 101 anchors / 3589 lines = 2.81 per 100 lines
- named gaps: missing explicit persona field
### j167 — j167-cto-diego-vargas-platform-major-version-cutover
- file count: 10; line count: 3177
- persona resolved: inferred-from-title-only
- µservice citations: 24 detected; sample=analytics, application, audit-chain, cell, cloud-iac, cloud-k8s, compliance, detection, docs, drive
- regulatory anchor density: 48 anchors / 3177 lines = 1.51 per 100 lines
- named gaps: missing explicit persona field
### j168 — j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
- file count: 10; line count: 2338
- persona resolved: inferred-from-title-only
- µservice citations: 23 detected; sample=analytics, application, audit-chain, calendar, cell, cloud-k8s, compliance, crm, detection, docs
- regulatory anchor density: 48 anchors / 2338 lines = 2.05 per 100 lines
- named gaps: missing explicit persona field
### j169 — j169-cmo-felix-ng-multi-country-launch-with-locale-pack
- file count: 10; line count: 2287
- persona resolved: inferred-from-title-only
- µservice citations: 24 detected; sample=analytics, application, audit-chain, cell, cloud-data, community, compliance, crm, docs, drive
- regulatory anchor density: 52 anchors / 2287 lines = 2.27 per 100 lines
- named gaps: missing explicit persona field
### j17 — j17-activist-dissident-high-risk-mode
- file count: 8; line count: 3281
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, community, compliance, docs, drive, foundry, identity, mail
- regulatory anchor density: 5 anchors / 3281 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j170 — j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
- file count: 10; line count: 2336
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=analytics, application, audit-chain, calendar, cell, cloud-data, compliance, connect, contract-lifecycle-management, crm
- regulatory anchor density: 66 anchors / 2336 lines = 2.83 per 100 lines
- named gaps: missing explicit persona field
### j171 — j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege
- file count: 10; line count: 3869
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=application, audit-chain, cell, community, compliance, docs, drive, governance, identity, meet
- regulatory anchor density: 80 anchors / 3869 lines = 2.07 per 100 lines
- named gaps: missing explicit persona field
### j172 — j172-lev-kahn-investor-relations-shareholder-meeting-livestream
- file count: 10; line count: 3435
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=analytics, application, audit-chain, cell, community, compliance, docs, drive, governance, identity
- regulatory anchor density: 29 anchors / 3435 lines = 0.84 per 100 lines
- named gaps: missing explicit persona field
### j173 — j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure
- file count: 10; line count: 3642
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=application, audit-chain, cell, compliance, contract-lifecycle-management, docs, drive, governance, identity, intelligence
- regulatory anchor density: 7 anchors / 3642 lines = 0.19 per 100 lines
- named gaps: missing explicit persona field
### j174 — j174-sven-eriksson-treasury-eod-position-reconciliation
- file count: 10; line count: 3075
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=application, audit-chain, cell, compliance, docs, finops-portal, identity, intelligence, messenger, notes
- regulatory anchor density: 6 anchors / 3075 lines = 0.2 per 100 lines
- named gaps: missing explicit persona field
### j175 — j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
- file count: 10; line count: 3155
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=application, audit-chain, calendar, cell, compliance, connect, docs, drive, finops-portal, identity
- regulatory anchor density: 5 anchors / 3155 lines = 0.16 per 100 lines
- named gaps: missing explicit persona field
### j176 — j176-migration-from-sap-s4hana-to-oyatie-finance-month-1
- file count: 10; line count: 3190
- persona resolved: inferred-from-title-only
- µservice citations: 13 detected; sample=audit-chain, compliance, data-pipeline, detection, docs, drive, identity, observability, ops-dashboard-control-center, tasks
- regulatory anchor density: 145 anchors / 3190 lines = 4.55 per 100 lines
- named gaps: missing explicit persona field
### j177 — j177-migration-from-salesforce-sales-cloud-to-oyatie-crm
- file count: 10; line count: 3186
- persona resolved: inferred-from-title-only
- µservice citations: 14 detected; sample=audit-chain, compliance, crm, data-pipeline, detection, docs, identity, mail, messenger, observability
- regulatory anchor density: 276 anchors / 3186 lines = 8.66 per 100 lines
- named gaps: missing explicit persona field
### j178 — j178-migration-from-workday-hcm-to-oyatie-workforce
- file count: 10; line count: 3165
- persona resolved: inferred-from-title-only
- µservice citations: 13 detected; sample=audit-chain, compliance, data-pipeline, detection, docs, drive, identity, messenger, observability, ops-dashboard-control-center
- regulatory anchor density: 99 anchors / 3165 lines = 3.13 per 100 lines
- named gaps: missing explicit persona field
### j179 — j179-migration-from-servicenow-itsm-to-oyatie-itsm
- file count: 10; line count: 3180
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=audit-chain, calendar, compliance, connect, data-pipeline, detection, docs, feature-flags, identity, incident-management
- regulatory anchor density: 145 anchors / 3180 lines = 4.56 per 100 lines
- named gaps: missing explicit persona field
### j18 — j18-child-safety-mandatory-reporter
- file count: 8; line count: 3285
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, community, compliance, docs, foundry, identity, mail, messenger
- regulatory anchor density: 7 anchors / 3285 lines = 0.21 per 100 lines
- named gaps: missing explicit persona field
### j180 — j180-migration-from-atlassian-jira-confluence-to-oyatie-workspace
- file count: 10; line count: 3166
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=audit-chain, compliance, connect, data-pipeline, detection, docs, drive, identity, messenger, notes
- regulatory anchor density: 238 anchors / 3166 lines = 7.52 per 100 lines
- named gaps: missing explicit persona field
### j19 — j19-tenant-break-glass-locked-out-tenant-admin
- file count: 8; line count: 3281
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=api-gateway, audit-chain, cell, compliance, docs, foundry, governance, identity, mail, messenger
- regulatory anchor density: 5 anchors / 3281 lines = 0.15 per 100 lines
- named gaps: missing explicit persona field
### j20 — j20-data-residency-violation-detection
- file count: 8; line count: 3289
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=api-gateway, audit-chain, cell, compliance, detection, docs, foundry, identity, mail, messenger
- regulatory anchor density: 57 anchors / 3289 lines = 1.73 per 100 lines
- named gaps: missing explicit persona field
### j21 — j21-personal-signup-passkey-first-dm
- file count: 9; line count: 3114
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal`. Center of gravity: `identity`.
- µservice citations: 16 detected; sample=analytics, audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail
- regulatory anchor density: 6 anchors / 3114 lines = 0.19 per 100 lines
- named gaps: none named; retain in regression audit
### j22 — j22-personal-mail-inbox-first-week
- file count: 6; line count: 2883
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal`. Center of gravity: `mail`.
- µservice citations: 13 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2883 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j23 — j23-marketplace-listing-and-first-sale
- file count: 6; line count: 2900
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal-seller`. Center of gravity: `marketplace`.
- µservice citations: 15 detected; sample=audit-chain, cell, community, compliance, connect, detection, docs, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2900 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j24 — j24-marketplace-purchase-as-buyer
- file count: 6; line count: 2900
- persona resolved: Aiyana Singh. Locale: `en-IN`. Tenant mode: `personal-buyer`. Center of gravity: `payments`.
- µservice citations: 14 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2900 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j25 — j25-personal-notes-daily-journaling-with-e2e
- file count: 6; line count: 2883
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal`. Center of gravity: `notes`.
- µservice citations: 15 detected; sample=audit-chain, cell, cloud-secrets, community, compliance, detection, docs, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2883 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j26 — j26-drive-family-photo-backup
- file count: 6; line count: 2884
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal`. Center of gravity: `drive`.
- µservice citations: 15 detected; sample=audit-chain, cell, community, compliance, connect, detection, docs, drive, identity, intelligence
- regulatory anchor density: 2 anchors / 2884 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j27 — j27-calendar-cross-context-family-and-work
- file count: 6; line count: 2885
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `dual-context`. Center of gravity: `calendar`.
- µservice citations: 14 detected; sample=audit-chain, calendar, cell, community, compliance, detection, docs, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2885 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j28 — j28-meet-family-video-call
- file count: 6; line count: 2884
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal-family`. Center of gravity: `meet`.
- µservice citations: 15 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2884 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j29 — j29-workflow-studio-personal-automation
- file count: 6; line count: 2884
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal-seller`. Center of gravity: `workflow-engine`.
- µservice citations: 16 detected; sample=audit-chain, cell, community, compliance, connect, detection, docs, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2884 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j30 — j30-shorts-creator-first-post
- file count: 6; line count: 2885
- persona resolved: Yejin Park daughter. Locale: `ko-KR`. Tenant mode: `minor-personal`. Center of gravity: `shorts`.
- µservice citations: 14 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2885 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j31 — j31-social-broadcast-vs-DM
- file count: 6; line count: 2883
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `personal`. Center of gravity: `social`.
- µservice citations: 15 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2883 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j32 — j32-community-teamblind-employer-anonymous
- file count: 6; line count: 2884
- persona resolved: Yejin Park. Locale: `ko-KR`. Tenant mode: `verified-employer-anonymous`. Center of gravity: `community`.
- µservice citations: 13 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2884 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j33 — j33-b2b-sso-saml-onboarding
- file count: 6; line count: 2900
- persona resolved: Marcus Chen. Locale: `en-US`. Tenant mode: `b2b-work`. Center of gravity: `identity`.
- µservice citations: 14 detected; sample=audit-chain, cell, community, compliance, detection, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 2 anchors / 2900 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j34 — j34-b2b-team-channel-with-files
- file count: 6; line count: 2899
- persona resolved: Marcus Chen. Locale: `en-US`. Tenant mode: `b2b-work`. Center of gravity: `messenger`.
- µservice citations: 16 detected; sample=audit-chain, cell, community, compliance, detection, docs, drive, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2899 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j35 — j35-b2b-workplace-mail-and-calendar
- file count: 6; line count: 2886
- persona resolved: Marcus Chen. Locale: `en-US`. Tenant mode: `b2b-work`. Center of gravity: `mail`.
- µservice citations: 15 detected; sample=audit-chain, calendar, cell, community, compliance, detection, docs, identity, intelligence, mail
- regulatory anchor density: 2 anchors / 2886 lines = 0.07 per 100 lines
- named gaps: none named; retain in regression audit
### j36 — j36-b2b-workflow-engine-approval-cascade
- file count: 6; line count: 3004
- persona resolved: Marcus Chen
- µservice citations: 16 detected; sample=audit-chain, cell, community, connect, docs, identity, mail, marketplace, messenger, network
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j37 — j37-b2b-clocking-and-attendance
- file count: 6; line count: 3004
- persona resolved: Marcus Chen
- µservice citations: 16 detected; sample=audit-chain, cell, community, connect, docs, identity, mail, marketplace, messenger, network
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j38 — j38-b2b-e-signing-contract
- file count: 6; line count: 3004
- persona resolved: Marcus Chen
- µservice citations: 16 detected; sample=audit-chain, cell, community, docs, drive, identity, mail, marketplace, messenger, network
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j39 — j39-b2b-meeting-with-transcription
- file count: 6; line count: 3047
- persona resolved: Marcus Chen
- µservice citations: 19 detected; sample=audit-chain, cell, community, docs, drive, identity, intelligence, mail, marketplace, meet
- regulatory anchor density: 19 anchors / 3047 lines = 0.62 per 100 lines
- named gaps: none named; retain in regression audit
### j40 — j40-b2b-marketplace-vendor-billing
- file count: 6; line count: 2961
- persona resolved: Marcus Chen
- µservice citations: 16 detected; sample=audit-chain, cell, community, connect, docs, identity, mail, marketplace, messenger, network
- regulatory anchor density: 200 anchors / 2961 lines = 6.75 per 100 lines
- named gaps: none named; retain in regression audit
### j41 — j41-b2b-developer-builds-on-platform
- file count: 6; line count: 3004
- persona resolved: Marcus Chen
- µservice citations: 16 detected; sample=audit-chain, cell, community, developer-sdk, docs, foundry, identity, mail, marketplace, messenger
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j42 — j42-b2b-finops-portal-spend-attribution
- file count: 6; line count: 2961
- persona resolved: Marcus Chen
- µservice citations: 15 detected; sample=audit-chain, cell, community, docs, finops-portal, identity, mail, marketplace, messenger, network
- regulatory anchor density: 200 anchors / 2961 lines = 6.75 per 100 lines
- named gaps: none named; retain in regression audit
### j43 — j43-healthcare-nurse-patient-handoff
- file count: 6; line count: 3047
- persona resolved: Yejin Park
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, docs, foundry, identity, intelligence, mail, marketplace
- regulatory anchor density: 784 anchors / 3047 lines = 25.73 per 100 lines
- named gaps: none named; retain in regression audit
### j44 — j44-healthcare-telemedicine-consultation
- file count: 6; line count: 3047
- persona resolved: Yejin Park
- µservice citations: 19 detected; sample=audit-chain, cell, community, compliance, connect, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 393 anchors / 3047 lines = 12.9 per 100 lines
- named gaps: none named; retain in regression audit
### j45 — j45-healthcare-patient-portal-records
- file count: 6; line count: 3047
- persona resolved: Yejin Park
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, docs, drive, identity, mail, marketplace, messenger
- regulatory anchor density: 61 anchors / 3047 lines = 2.0 per 100 lines
- named gaps: none named; retain in regression audit
### j46 — j46-healthcare-prescription-renewal-workflow
- file count: 6; line count: 3047
- persona resolved: Yejin Park
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, identity, mail, marketplace, messenger
- regulatory anchor density: 19 anchors / 3047 lines = 0.62 per 100 lines
- named gaps: none named; retain in regression audit
### j47 — j47-healthcare-billing-and-insurance
- file count: 6; line count: 3004
- persona resolved: Yejin Park
- µservice citations: 16 detected; sample=audit-chain, cell, community, compliance, connect, docs, identity, mail, marketplace, messenger
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j48 — j48-sidebusiness-stripe-tax-and-invoicing
- file count: 6; line count: 3004
- persona resolved: Yejin Park
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, finops-portal, identity, mail, marketplace
- regulatory anchor density: 1511 anchors / 3004 lines = 50.3 per 100 lines
- named gaps: none named; retain in regression audit
### j49 — j49-sidebusiness-customer-support-omnichannel
- file count: 6; line count: 3047
- persona resolved: Yejin Park
- µservice citations: 17 detected; sample=audit-chain, cell, community, connect, docs, identity, intelligence, mail, marketplace, messenger
- regulatory anchor density: 19 anchors / 3047 lines = 0.62 per 100 lines
- named gaps: none named; retain in regression audit
### j50 — j50-sidebusiness-employee-hires-first-helper
- file count: 6; line count: 3004
- persona resolved: Yejin Park
- µservice citations: 14 detected; sample=audit-chain, cell, community, docs, identity, mail, marketplace, messenger, network, observability
- regulatory anchor density: 109 anchors / 3004 lines = 3.63 per 100 lines
- named gaps: none named; retain in regression audit
### j51 — j51-procure-to-pay-po-extraction-and-approval
- file count: 8; line count: 3140
- persona resolved: "Marcus Chen, acting procurement lead at Acme SaaS"
- µservice citations: 19 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, intelligence, mail
- regulatory anchor density: 0 anchors / 3140 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j52 — j52-order-to-cash-marketplace-to-fulfillment
- file count: 8; line count: 3140
- persona resolved: "Yejin Park, buyer and seller operating across personal and work contexts"
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 0 anchors / 3140 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j53 — j53-invoice-to-cash-recurring-subscription
- file count: 8; line count: 3128
- persona resolved: "Marcus Chen, finance owner for a recurring B2B SaaS customer"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, finops-portal, identity, mail
- regulatory anchor density: 0 anchors / 3128 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j54 — j54-quote-to-contract-to-payment-saas
- file count: 8; line count: 3146
- persona resolved: "A prospect buying Marcus's SaaS under a new tenant"
- µservice citations: 20 detected; sample=application, audit-chain, cell, community, compliance, connect, docs, drive, forms, identity
- regulatory anchor density: 0 anchors / 3146 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j55 — j55-refund-and-dispute-resolution-cascade
- file count: 8; line count: 3140
- persona resolved: "A buyer disputing a marketplace charge with the seller and platform involved"
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 0 anchors / 3140 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j56 — j56-job-application-to-offer
- file count: 8; line count: 3152
- persona resolved: "Aiyana Robinson applying to Marcus's company through community Handshake mode"
- µservice citations: 21 detected; sample=application, audit-chain, calendar, cell, community, compliance, connect, docs, drive, identity
- regulatory anchor density: 0 anchors / 3152 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j57 — j57-employee-onboarding-day-one-to-week-one
- file count: 8; line count: 3146
- persona resolved: "Aiyana Robinson during her first week as a new employee"
- µservice citations: 21 detected; sample=audit-chain, calendar, cell, community, compliance, connect, docs, drive, identity, mail
- regulatory anchor density: 0 anchors / 3146 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j58 — j58-quarterly-performance-review-cycle
- file count: 8; line count: 3152
- persona resolved: "Aiyana Robinson and her manager completing quarterly review"
- µservice citations: 23 detected; sample=application, audit-chain, calendar, cell, community, compliance, connect, docs, drive, finops-portal
- regulatory anchor density: 0 anchors / 3152 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j59 — j59-offboarding-and-knowledge-transfer
- file count: 8; line count: 3140
- persona resolved: "An employee leaving Marcus's company with personal tenant preserved"
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 0 anchors / 3140 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j60 — j60-internal-mobility-promotion-cascade
- file count: 8; line count: 3146
- persona resolved: "Aiyana Robinson promoted into a new internal role"
- µservice citations: 20 detected; sample=application, audit-chain, cell, community, compliance, connect, docs, drive, finops-portal, forms
- regulatory anchor density: 0 anchors / 3146 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j61 — j61-patient-intake-to-followup
- file count: 8; line count: 3158
- persona resolved: "Yejin Park, physician treating a referred patient"
- µservice citations: 21 detected; sample=application, audit-chain, cell, community, compliance, connect, docs, drive, forms, identity
- regulatory anchor density: 476 anchors / 3158 lines = 15.07 per 100 lines
- named gaps: none named; retain in regression audit
### j62 — j62-prescription-to-pharmacy-to-payment
- file count: 8; line count: 3140
- persona resolved: "Yejin Park prescribing medication for a patient"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 0 anchors / 3140 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j63 — j63-clinical-trial-recruitment-to-consent
- file count: 8; line count: 3140
- persona resolved: "A research coordinator recruiting through a verified community network"
- µservice citations: 21 detected; sample=application, audit-chain, cell, community, compliance, connect, docs, drive, forms, identity
- regulatory anchor density: 339 anchors / 3140 lines = 10.8 per 100 lines
- named gaps: none named; retain in regression audit
### j64 — j64-hospital-network-cross-tenant-referral
- file count: 8; line count: 3140
- persona resolved: "Yejin Park referring a patient to a specialist at another hospital"
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 670 anchors / 3140 lines = 21.34 per 100 lines
- named gaps: none named; retain in regression audit
### j65 — j65-gdpr-dsar-cascade-across-all-services
- file count: 8; line count: 3164
- persona resolved: "An EU resident exercising GDPR Article 15 and Article 20 rights"
- µservice citations: 17 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 2988 anchors / 3164 lines = 94.44 per 100 lines
- named gaps: none named; retain in regression audit
### j66 — j66-tax-quarterly-filing-multi-jurisdiction
- file count: 8; line count: 3146
- persona resolved: "Marcus's finance team filing KR, EU, and US quarterly obligations"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, finops-portal, identity, mail
- regulatory anchor density: 8 anchors / 3146 lines = 0.25 per 100 lines
- named gaps: none named; retain in regression audit
### j67 — j67-law-enforcement-warrant-response
- file count: 8; line count: 3140
- persona resolved: "Legal operations responding to a scoped warrant"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, governance, identity, mail
- regulatory anchor density: 8 anchors / 3140 lines = 0.25 per 100 lines
- named gaps: none named; retain in regression audit
### j68 — j68-regulator-audit-pull-hippa-soc2-pci
- file count: 8; line count: 3140
- persona resolved: "An external auditor pulling HIPAA, SOC2, and PCI records"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, mail, marketplace
- regulatory anchor density: 6109 anchors / 3140 lines = 194.55 per 100 lines
- named gaps: none named; retain in regression audit
### j69 — j69-llm-agent-managing-yejins-week
- file count: 8; line count: 3146
- persona resolved: "Yejin Park delegating weekly coordination to an Intelligence agent"
- µservice citations: 21 detected; sample=audit-chain, calendar, cell, community, compliance, connect, docs, drive, identity, intelligence
- regulatory anchor density: 0 anchors / 3146 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j70 — j70-ai-drafted-contract-human-finalized
- file count: 8; line count: 3146
- persona resolved: "Marcus using Intelligence to draft but not finalize a vendor contract"
- µservice citations: 19 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, intelligence, mail
- regulatory anchor density: 0 anchors / 3146 lines = 0.0 per 100 lines
- named gaps: no regulatory anchor
### j71 — j71-ai-detected-fraud-pattern-response
- file count: 8; line count: 3140
- persona resolved: "Yejin receiving a suspicious-card alert and appeal path"
- µservice citations: 18 detected; sample=audit-chain, cell, community, compliance, connect, detection, docs, drive, identity, mail
- regulatory anchor density: 339 anchors / 3140 lines = 10.8 per 100 lines
- named gaps: none named; retain in regression audit
### j72 — j72-ai-translation-cross-locale-business
- file count: 8; line count: 3140
- persona resolved: "Tom\u00e1s emailing Marcus's company in Portuguese under LGPD context"
- µservice citations: 21 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, identity, intelligence, mail
- regulatory anchor density: 1895 anchors / 3140 lines = 60.35 per 100 lines
- named gaps: none named; retain in regression audit
### j73 — j73-third-party-developer-publishes-plugin
- file count: 8; line count: 3146
- persona resolved: "A third-party developer publishing a tenant-safe plugin"
- µservice citations: 19 detected; sample=audit-chain, cell, community, compliance, connect, docs, drive, foundry, identity, mail
- regulatory anchor density: 8 anchors / 3146 lines = 0.25 per 100 lines
- named gaps: none named; retain in regression audit
### j74 — j74-tenant-installs-plugin-and-it-spans-services
- file count: 8; line count: 3146
- persona resolved: "Marcus installing a CRM plugin across work surfaces"
- µservice citations: 19 detected; sample=audit-chain, cell, community, compliance, connect, crm, docs, drive, identity, mail
- regulatory anchor density: 8 anchors / 3146 lines = 0.25 per 100 lines
- named gaps: none named; retain in regression audit
### j75 — j75-plugin-revoked-during-incident-response
- file count: 8; line count: 3140
- persona resolved: "Marcus responding to a plugin CVE during an active incident"
- µservice citations: 20 detected; sample=audit-chain, cell, community, compliance, connect, detection, docs, drive, foundry, identity
- regulatory anchor density: 8 anchors / 3140 lines = 0.25 per 100 lines
- named gaps: none named; retain in regression audit
### j76 — j76-eu-gdpr-dsar-full-cascade
- file count: 11; line count: 3165
- persona resolved: inferred-from-title-only
- µservice citations: 21 detected; sample=analytics, api-gateway, application, audit-chain, cell, community, compliance, consent-graph, docs, governance
- regulatory anchor density: 776 anchors / 3165 lines = 24.52 per 100 lines
- named gaps: missing explicit persona field
### j77 — j77-eu-ai-act-high-risk-credit-decision
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 15 detected; sample=analytics, audit-chain, cell, compliance, docs, identity, intelligence, marketplace, notes, observability
- regulatory anchor density: 251 anchors / 2844 lines = 8.83 per 100 lines
- named gaps: missing explicit persona field
### j78 — j78-eu-nis2-breach-three-stage-cadence
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=analytics, api-gateway, audit-chain, cell, compliance, docs, governance, identity, mail, marketplace
- regulatory anchor density: 524 anchors / 2844 lines = 18.42 per 100 lines
- named gaps: missing explicit persona field
### j79 — j79-eu-dsa-transparency-semi-annual-report
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=analytics, audit-chain, cell, community, compliance, docs, identity, intelligence, mail, marketplace
- regulatory anchor density: 525 anchors / 2844 lines = 18.46 per 100 lines
- named gaps: missing explicit persona field
### j80 — j80-kr-pipa-personal-info-cross-border-transfer
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=analytics, audit-chain, cell, cloud-iac, cloud-secrets, compliance, consent-graph, docs, drive, identity
- regulatory anchor density: 611 anchors / 2844 lines = 21.48 per 100 lines
- named gaps: missing explicit persona field
### j81 — j81-kr-csap-sovereign-cell-audit-pull
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=analytics, audit-chain, cell, cloud-iac, cloud-k8s, cloud-secrets, compliance, docs, governance, identity
- regulatory anchor density: 482 anchors / 2844 lines = 16.95 per 100 lines
- named gaps: missing explicit persona field
### j82 — j82-kr-fss-financial-fraud-24h-freeze
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=analytics, audit-chain, cell, compliance, docs, finops-portal, identity, intelligence, mail, marketplace
- regulatory anchor density: 444 anchors / 2844 lines = 15.61 per 100 lines
- named gaps: missing explicit persona field
### j83 — j83-cn-pipl-data-localization-and-cac-assessment
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=analytics, audit-chain, cell, cloud-iac, cloud-secrets, compliance, consent-graph, docs, governance, identity
- regulatory anchor density: 14 anchors / 2844 lines = 0.49 per 100 lines
- named gaps: missing explicit persona field
### j84 — j84-jp-appi-elder-user-consent
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=analytics, audit-chain, cell, community, compliance, consent-graph, docs, identity, mail, marketplace
- regulatory anchor density: 476 anchors / 2844 lines = 16.74 per 100 lines
- named gaps: missing explicit persona field
### j85 — j85-hipaa-end-to-end-phi-workflow
- file count: 6; line count: 2843
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=analytics, audit-chain, cell, compliance, consent-graph, docs, drive, identity, mail, marketplace
- regulatory anchor density: 695 anchors / 2843 lines = 24.45 per 100 lines
- named gaps: missing explicit persona field
### j86 — j86-pci-dss-l1-tokenized-payment-flow
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 16 detected; sample=analytics, audit-chain, cell, cloud-secrets, compliance, docs, finops-portal, identity, marketplace, network
- regulatory anchor density: 535 anchors / 2844 lines = 18.81 per 100 lines
- named gaps: missing explicit persona field
### j87 — j87-fedramp-high-il5-air-gap-deployment
- file count: 6; line count: 2843
- persona resolved: inferred-from-title-only
- µservice citations: 18 detected; sample=analytics, audit-chain, cell, cloud-iac, cloud-k8s, cloud-secrets, compliance, docs, governance, identity
- regulatory anchor density: 357 anchors / 2843 lines = 12.56 per 100 lines
- named gaps: missing explicit persona field
### j88 — j88-au-irap-protected-tenant
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 17 detected; sample=analytics, audit-chain, cell, cloud-iac, cloud-secrets, compliance, docs, governance, identity, marketplace
- regulatory anchor density: 312 anchors / 2844 lines = 10.97 per 100 lines
- named gaps: missing explicit persona field
### j89 — j89-uk-aadc-minor-ux-adaptation
- file count: 6; line count: 2844
- persona resolved: inferred-from-title-only
- µservice citations: 19 detected; sample=analytics, audit-chain, cell, community, compliance, consent-graph, docs, identity, intelligence, mail
- regulatory anchor density: 61 anchors / 2844 lines = 2.14 per 100 lines
- named gaps: missing explicit persona field
### j90 — j90-us-ccpa-cpra-do-not-sell-opt-out
- file count: 6; line count: 2843
- persona resolved: inferred-from-title-only
- µservice citations: 19 detected; sample=analytics, audit-chain, cell, community, compliance, consent-graph, docs, identity, intelligence, marketplace
- regulatory anchor density: 466 anchors / 2843 lines = 16.39 per 100 lines
- named gaps: missing explicit persona field
### j91 — j91-us-state-money-transmitter-licensing
- file count: 9; line count: 2821
- persona resolved: inferred-from-title-only
- µservice citations: 47 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 7 anchors / 2821 lines = 0.25 per 100 lines
- named gaps: missing explicit persona field
### j92 — j92-br-lgpd-dsar-with-us-parent
- file count: 9; line count: 2815
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 2912 anchors / 2815 lines = 103.45 per 100 lines
- named gaps: missing explicit persona field
### j93 — j93-in-dpdpa-rbi-financial-overlay
- file count: 9; line count: 2809
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 2674 anchors / 2809 lines = 95.19 per 100 lines
- named gaps: missing explicit persona field
### j94 — j94-sox-404-public-company-controls
- file count: 9; line count: 2803
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 376 anchors / 2803 lines = 13.41 per 100 lines
- named gaps: missing explicit persona field
### j95 — j95-iso-27001-soc-2-annual-audit
- file count: 9; line count: 2807
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 1542 anchors / 2807 lines = 54.93 per 100 lines
- named gaps: missing explicit persona field
### j96 — j96-ksa-uae-mena-tenant-onboarding
- file count: 9; line count: 2813
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 355 anchors / 2813 lines = 12.62 per 100 lines
- named gaps: missing explicit persona field
### j97 — j97-sg-pdpa-mas-singapore-tenant
- file count: 9; line count: 2813
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 7 anchors / 2813 lines = 0.25 per 100 lines
- named gaps: missing explicit persona field
### j98 — j98-au-privacy-apra-cps-234-tenant
- file count: 9; line count: 2813
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 438 anchors / 2813 lines = 15.57 per 100 lines
- named gaps: missing explicit persona field
### j99 — j99-cross-jurisdiction-multi-pack-conflict-resolution
- file count: 9; line count: 2812
- persona resolved: inferred-from-title-only
- µservice citations: 45 detected; sample=analytics, api-gateway, application, audit-chain, calendar, cell, cloud-iac, cloud-k8s, cloud-secrets, comms-email
- regulatory anchor density: 1294 anchors / 2812 lines = 46.02 per 100 lines
- named gaps: missing explicit persona field

## §5 Persona Coverage Scorecard
- Persona dossiers discovered: 129
- User requested reference count: 129 personas
- Corpus-level named gap: presence matches 129 persona dossiers
### Persona — Accountant Ravi Iyer
- artifact: `docs/personas/accountant-ravi-iyer.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 68 detected by title/name token scan; sample=j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111, j112
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ahmad Hassan
- artifact: `docs/personas/ahmad-hassan.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 3 detected by title/name token scan; sample=j126, j152, j169
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Aiyana Singh
- artifact: `docs/personas/aiyana-singh.md`; lines=409
- substance markers: 10 / 17 tracked markers present
- journey appearances: 9 detected by title/name token scan; sample=j113, j135, j149, j24, j56, j57, j58, j60, j93
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Anya Mironova
- artifact: `docs/personas/anya-mironova.md`; lines=408
- substance markers: 11 / 17 tracked markers present
- journey appearances: 7 detected by title/name token scan; sample=j06, j155, j159, j17, j175, j76, j77
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Apprentice Jakob Bauer
- artifact: `docs/personas/apprentice-jakob-bauer.md`; lines=467
- substance markers: 10 / 17 tracked markers present
- journey appearances: 4 detected by title/name token scan; sample=j154, j162, j76, j77
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Auditor It Specialist Jakub Nowak
- artifact: `docs/personas/auditor-it-specialist-jakub-nowak.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 92 detected by title/name token scan; sample=j100, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Av Coordinator Jordan Park
- artifact: `docs/personas/av-coordinator-jordan-park.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 77 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Bank Compliance Officer Rishi Bhattacharya
- artifact: `docs/personas/bank-compliance-officer-rishi-bhattacharya.md`; lines=469
- substance markers: 12 / 17 tracked markers present
- journey appearances: 168 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Bank Ops Officer Olamide Adebanjo
- artifact: `docs/personas/bank-ops-officer-olamide-adebanjo.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 116 detected by title/name token scan; sample=j01, j02, j03, j06, j100, j101, j102, j103, j104, j105, j106, j107
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Bank Risk Manager Anders Pedersen
- artifact: `docs/personas/bank-risk-manager-anders-pedersen.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 148 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Banker External Hideki Watanabe
- artifact: `docs/personas/banker-external-hideki-watanabe.md`; lines=469
- substance markers: 10 / 17 tracked markers present
- journey appearances: 98 detected by title/name token scan; sample=j100, j102, j103, j104, j106, j107, j116, j117, j118, j119, j120, j121
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Benefits Specialist Aoife Murphy
- artifact: `docs/personas/benefits-specialist-aoife-murphy.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 18 detected by title/name token scan; sample=j109, j126, j132, j133, j134, j135, j136, j146, j152, j153, j159, j160
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Board Director Patrick Oreilly
- artifact: `docs/personas/board-director-patrick-oreilly.md`; lines=407
- substance markers: 10 / 17 tracked markers present
- journey appearances: 179 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Board Secretary Florence Akinsanya
- artifact: `docs/personas/board-secretary-florence-akinsanya.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 178 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Business Analyst Aditya Verma
- artifact: `docs/personas/business-analyst-aditya-verma.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 105 detected by title/name token scan; sample=j01, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cafeteria Manager Soyeon Kim
- artifact: `docs/personas/cafeteria-manager-soyeon-kim.md`; lines=469
- substance markers: 10 / 17 tracked markers present
- journey appearances: 97 detected by title/name token scan; sample=j02, j104, j107, j114, j118, j122, j126, j127, j132, j133, j134, j135
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Captain Chen Pilot
- artifact: `docs/personas/captain-chen-pilot.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 79 detected by title/name token scan; sample=j01, j100, j114, j118, j119, j121, j123, j125, j126, j127, j128, j129
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Captain Olufemi
- artifact: `docs/personas/captain-olufemi.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 1 detected by title/name token scan; sample=j151
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Carlos Martinez Forklift
- artifact: `docs/personas/carlos-martinez-forklift.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 3 detected by title/name token scan; sample=j146, j156, j160
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cco Naveen Iyer
- artifact: `docs/personas/cco-naveen-iyer.md`; lines=470
- substance markers: 12 / 17 tracked markers present
- journey appearances: 135 detected by title/name token scan; sample=j01, j02, j03, j04, j07, j09, j10, j116, j117, j118, j119, j12
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ceo Aoki Tanaka
- artifact: `docs/personas/ceo-aoki-tanaka.md`; lines=407
- substance markers: 11 / 17 tracked markers present
- journey appearances: 45 detected by title/name token scan; sample=j01, j02, j03, j126, j128, j130, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cfo Helena Brandt
- artifact: `docs/personas/cfo-helena-brandt.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 16 detected by title/name token scan; sample=j126, j133, j134, j136, j137, j138, j139, j163, j166, j167, j168, j170
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Channel Partner Tomas Pieter
- artifact: `docs/personas/channel-partner-tomas-pieter.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 110 detected by title/name token scan; sample=j01, j03, j04, j06, j10, j100, j113, j114, j116, j117, j118, j119
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Chris Volkov
- artifact: `docs/personas/chris-volkov.md`; lines=408
- substance markers: 10 / 17 tracked markers present
- journey appearances: 16 detected by title/name token scan; sample=j127, j132, j133, j142, j143, j144, j145, j146, j147, j153, j155, j159
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Chro Linda Foster
- artifact: `docs/personas/chro-linda-foster.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 88 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j11, j12
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ciso Yuki Park
- artifact: `docs/personas/ciso-yuki-park.md`; lines=406
- substance markers: 12 / 17 tracked markers present
- journey appearances: 74 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cleaning Supervisor Tomas Horak
- artifact: `docs/personas/cleaning-supervisor-tomas-horak.md`; lines=467
- substance markers: 11 / 17 tracked markers present
- journey appearances: 16 detected by title/name token scan; sample=j02, j03, j128, j154, j155, j156, j160, j162, j163, j167, j168, j169
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cmo Felix Ng
- artifact: `docs/personas/cmo-felix-ng.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 5 detected by title/name token scan; sample=j154, j167, j169, j171, j172
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Co Op Student Liam Murphy
- artifact: `docs/personas/co-op-student-liam-murphy.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 10 detected by title/name token scan; sample=j113, j128, j132, j153, j155, j158, j159, j161, j173, j175
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Coach Park
- artifact: `docs/personas/coach-park.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 76 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Commercial Banker Frederik Hartmann
- artifact: `docs/personas/commercial-banker-frederik-hartmann.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 48 detected by title/name token scan; sample=j116, j117, j118, j119, j120, j121, j122, j123, j124, j125, j134, j148
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Communications Specialist Charlotte Dubois
- artifact: `docs/personas/communications-specialist-charlotte-dubois.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 19 detected by title/name token scan; sample=j109, j129, j130, j133, j134, j137, j141, j152, j157, j158, j159, j160
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Compliance Analyst Yui Hayashi
- artifact: `docs/personas/compliance-analyst-yui-hayashi.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 168 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Compliance Officer Tunde Bello
- artifact: `docs/personas/compliance-officer-tunde-bello.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 169 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Consultant Adekunle Adebayo
- artifact: `docs/personas/consultant-adekunle-adebayo.md`; lines=467
- substance markers: 11 / 17 tracked markers present
- journey appearances: 5 detected by title/name token scan; sample=j138, j151, j162, j166, j170
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Coo Akira Watanabe
- artifact: `docs/personas/coo-akira-watanabe.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 53 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j11, j117
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Corp Dev Senior Analyst Saanvi Mehta
- artifact: `docs/personas/corp-dev-senior-analyst-saanvi-mehta.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 179 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Corporate Relations Director Soo Yeon Han
- artifact: `docs/personas/corporate-relations-director-soo-yeon-han.md`; lines=468
- substance markers: 12 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Credit Analyst Hina Mori
- artifact: `docs/personas/credit-analyst-hina-mori.md`; lines=470
- substance markers: 11 / 17 tracked markers present
- journey appearances: 39 detected by title/name token scan; sample=j108, j116, j117, j118, j119, j120, j121, j122, j123, j124, j125, j126
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cs Ic Lin Chen
- artifact: `docs/personas/cs-ic-lin-chen.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cso Mira Goldberg
- artifact: `docs/personas/cso-mira-goldberg.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 8 detected by title/name token scan; sample=j117, j154, j155, j166, j167, j168, j170, j173
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Cto Diego Vargas
- artifact: `docs/personas/cto-diego-vargas.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Customer Champion Akemi Sato
- artifact: `docs/personas/customer-champion-akemi-sato.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 59 detected by title/name token scan; sample=j01, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Customer Success Manager Sofia Rezende
- artifact: `docs/personas/customer-success-manager-sofia-rezende.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 150 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — D And I Director Maya Okoroafor
- artifact: `docs/personas/d-and-i-director-maya-okoroafor.md`; lines=470
- substance markers: 11 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Data Analyst Felipe Andrade
- artifact: `docs/personas/data-analyst-felipe-andrade.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 179 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Data Scientist Yu Chen
- artifact: `docs/personas/data-scientist-yu-chen.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 179 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Devon Williams
- artifact: `docs/personas/devon-williams.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 3 detected by title/name token scan; sample=j132, j153, j175
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Devops Engineer Olukayode Adejumo
- artifact: `docs/personas/devops-engineer-olukayode-adejumo.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 82 detected by title/name token scan; sample=j114, j116, j117, j118, j119, j120, j121, j122, j123, j124, j125, j126
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Devops Manager Pavel Korsak
- artifact: `docs/personas/devops-manager-pavel-korsak.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 48 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Diana Reyes
- artifact: `docs/personas/diana-reyes.md`; lines=405
- substance markers: 11 / 17 tracked markers present
- journey appearances: 13 detected by title/name token scan; sample=j126, j127, j128, j129, j130, j131, j152, j156, j157, j162, j166, j170
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Dr Tanaka Surgeon
- artifact: `docs/personas/dr-tanaka-surgeon.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 8 detected by title/name token scan; sample=j134, j152, j164, j168, j172, j173, j175, j84
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Engineering Manager Aisha Ali
- artifact: `docs/personas/engineering-manager-aisha-ali.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Executive Assistant Olivia Reyes
- artifact: `docs/personas/executive-assistant-olivia-reyes.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 28 detected by title/name token scan; sample=j126, j128, j129, j130, j131, j133, j135, j152, j153, j156, j160, j161
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — External Auditor Dimitri Volkov
- artifact: `docs/personas/external-auditor-dimitri-volkov.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 137 detected by title/name token scan; sample=j100, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — External Auditor Hyo Jin Lee
- artifact: `docs/personas/external-auditor-hyo-jin-lee.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 178 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Father Lopez Priest
- artifact: `docs/personas/father-lopez-priest.md`; lines=407
- substance markers: 11 / 17 tracked markers present
- journey appearances: 15 detected by title/name token scan; sample=j03, j07, j141, j156, j157, j158, j160, j161, j162, j163, j164, j167
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Fellow Dr Tobias Klein
- artifact: `docs/personas/fellow-dr-tobias-klein.md`; lines=470
- substance markers: 11 / 17 tracked markers present
- journey appearances: 4 detected by title/name token scan; sample=j128, j161, j82, j86
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Finance Director Mei Ling Wu
- artifact: `docs/personas/finance-director-mei-ling-wu.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Financial Analyst Wendy Lee
- artifact: `docs/personas/financial-analyst-wendy-lee.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 159 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j104
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Hiroshi Tanaka
- artifact: `docs/personas/hiroshi-tanaka.md`; lines=407
- substance markers: 10 / 17 tracked markers present
- journey appearances: 8 detected by title/name token scan; sample=j134, j152, j164, j168, j172, j173, j175, j84
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Hr Specialist Aoife Murphy
- artifact: `docs/personas/hr-specialist-aoife-murphy.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 10 detected by title/name token scan; sample=j109, j133, j152, j159, j160, j162, j168, j175, j61, j64
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Hrbp Jamal Carter
- artifact: `docs/personas/hrbp-jamal-carter.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 2 detected by title/name token scan; sample=j132, j178
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Intern Manager Felicia Adamou
- artifact: `docs/personas/intern-manager-felicia-adamou.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 162 detected by title/name token scan; sample=j01, j02, j03, j100, j101, j102, j103, j104, j105, j106, j107, j108
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Internal Comms Lead Ji Ho Yoon
- artifact: `docs/personas/internal-comms-lead-ji-ho-yoon.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 160 detected by title/name token scan; sample=j01, j02, j03, j100, j101, j102, j103, j104, j105, j106, j107, j108
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Investment Banker Yuna Ahn
- artifact: `docs/personas/investment-banker-yuna-ahn.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 12 detected by title/name token scan; sample=j108, j134, j154, j155, j156, j163, j170, j172, j173, j174, j175, j91
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Investor Lp Aanya Kapoor
- artifact: `docs/personas/investor-lp-aanya-kapoor.md`; lines=471
- substance markers: 10 / 17 tracked markers present
- journey appearances: 6 detected by title/name token scan; sample=j167, j168, j170, j172, j173, j175
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ir Manager Lev Kahn
- artifact: `docs/personas/ir-manager-lev-kahn.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 132 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j104
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ir Specialist Unnamed
- artifact: `docs/personas/ir-specialist-unnamed.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 10 detected by title/name token scan; sample=j109, j133, j152, j159, j160, j162, j168, j175, j61, j64
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — It Manager Jamie O Connor
- artifact: `docs/personas/it-manager-jamie-o-connor.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 49 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Jordan Lee
- artifact: `docs/personas/jordan-lee.md`; lines=464
- substance markers: 10 / 17 tracked markers present
- journey appearances: 155 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j104
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Leave Specialist Margarethe Reinhart
- artifact: `docs/personas/leave-specialist-margarethe-reinhart.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 38 detected by title/name token scan; sample=j01, j02, j100, j109, j126, j127, j133, j136, j137, j138, j139, j142
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Legal Counsel Anika Mehta
- artifact: `docs/personas/legal-counsel-anika-mehta.md`; lines=467
- substance markers: 10 / 17 tracked markers present
- journey appearances: 154 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Legal Operations Stephen Park
- artifact: `docs/personas/legal-operations-stephen-park.md`; lines=467
- substance markers: 11 / 17 tracked markers present
- journey appearances: 170 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Mailroom Hae Won Kim
- artifact: `docs/personas/mailroom-hae-won-kim.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 75 detected by title/name token scan; sample=j02, j03, j107, j122, j126, j132, j133, j134, j136, j154, j157, j158
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Maintenance Tech Carlos Reyes Ii
- artifact: `docs/personas/maintenance-tech-carlos-reyes-ii.md`; lines=467
- substance markers: 10 / 17 tracked markers present
- journey appearances: 53 detected by title/name token scan; sample=j01, j100, j126, j128, j129, j130, j131, j132, j133, j134, j141, j142
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Marcus Chen
- artifact: `docs/personas/marcus-chen.md`; lines=409
- substance markers: 11 / 17 tracked markers present
- journey appearances: 88 detected by title/name token scan; sample=j01, j100, j114, j118, j119, j120, j121, j123, j125, j126, j127, j128
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Maria Santos
- artifact: `docs/personas/maria-santos.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 6 detected by title/name token scan; sample=j133, j152, j157, j160, j162, j169
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Marketing Manager Olu Adeyemi
- artifact: `docs/personas/marketing-manager-olu-adeyemi.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 140 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j104
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Marketing Specialist Riya Sharma
- artifact: `docs/personas/marketing-specialist-riya-sharma.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 89 detected by title/name token scan; sample=j02, j03, j04, j05, j06, j07, j08, j09, j10, j109, j11, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Medical Resident Dr Sun Mi Kim
- artifact: `docs/personas/medical-resident-dr-sun-mi-kim.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 98 detected by title/name token scan; sample=j01, j02, j107, j122, j126, j127, j128, j129, j13, j132, j133, j134
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ms Patel Teacher
- artifact: `docs/personas/ms-patel-teacher.md`; lines=407
- substance markers: 10 / 17 tracked markers present
- journey appearances: 8 detected by title/name token scan; sample=j03, j134, j159, j160, j161, j162, j167, j168
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Office Coordinator Phoebe Lin
- artifact: `docs/personas/office-coordinator-phoebe-lin.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Office Manager Priya Ramanathan
- artifact: `docs/personas/office-manager-priya-ramanathan.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 77 detected by title/name token scan; sample=j02, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Officer Rodriguez Police
- artifact: `docs/personas/officer-rodriguez-police.md`; lines=406
- substance markers: 11 / 17 tracked markers present
- journey appearances: 38 detected by title/name token scan; sample=j02, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ombudsperson Felix Tan
- artifact: `docs/personas/ombudsperson-felix-tan.md`; lines=463
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Outside Counsel Wei Yi Chen
- artifact: `docs/personas/outside-counsel-wei-yi-chen.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 111 detected by title/name token scan; sample=j01, j02, j03, j100, j108, j114, j118, j119, j121, j123, j125, j126
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Paralegal Tomas Novak
- artifact: `docs/personas/paralegal-tomas-novak.md`; lines=467
- substance markers: 10 / 17 tracked markers present
- journey appearances: 6 detected by title/name token scan; sample=j154, j156, j160, j167, j171, j92
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Pr Firm Beatriz Fernandez
- artifact: `docs/personas/pr-firm-beatriz-fernandez.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 128 detected by title/name token scan; sample=j01, j02, j03, j100, j101, j102, j103, j104, j105, j106, j107, j108
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Pr Manager Helena Sato
- artifact: `docs/personas/pr-manager-helena-sato.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 48 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Print Operator Diana Lazar
- artifact: `docs/personas/print-operator-diana-lazar.md`; lines=469
- substance markers: 10 / 17 tracked markers present
- journey appearances: 92 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j11
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Priya Krishnan
- artifact: `docs/personas/priya-krishnan.md`; lines=408
- substance markers: 11 / 17 tracked markers present
- journey appearances: 22 detected by title/name token scan; sample=j111, j115, j127, j132, j133, j134, j135, j136, j137, j138, j139, j142
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Procurement Manager Wei Liu
- artifact: `docs/personas/procurement-manager-wei-liu.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 63 detected by title/name token scan; sample=j101, j102, j103, j104, j105, j106, j107, j108, j114, j116, j118, j122
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Procurement Specialist Beata Kowalski
- artifact: `docs/personas/procurement-specialist-beata-kowalski.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 25 detected by title/name token scan; sample=j101, j102, j103, j104, j105, j106, j107, j108, j109, j116, j133, j138
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Product Designer Akihiro Sato
- artifact: `docs/personas/product-designer-akihiro-sato.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 141 detected by title/name token scan; sample=j01, j100, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Product Manager Lily Chang
- artifact: `docs/personas/product-manager-lily-chang.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 160 detected by title/name token scan; sample=j01, j100, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Project Manager Soo Jin Park
- artifact: `docs/personas/project-manager-soo-jin-park.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 135 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Public Affairs Director Carlos Mendez
- artifact: `docs/personas/public-affairs-director-carlos-mendez.md`; lines=470
- substance markers: 11 / 17 tracked markers present
- journey appearances: 142 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Receptionist Daria Volkova
- artifact: `docs/personas/receptionist-daria-volkova.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 1 detected by title/name token scan; sample=j167
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Recruiter Marcus Iv
- artifact: `docs/personas/recruiter-marcus-iv.md`; lines=469
- substance markers: 10 / 17 tracked markers present
- journey appearances: 74 detected by title/name token scan; sample=j100, j114, j118, j119, j120, j121, j123, j125, j126, j127, j130, j132
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Recruiting Manager Hina Suzuki
- artifact: `docs/personas/recruiting-manager-hina-suzuki.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 74 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Regulator Inspector Sergei Petrov
- artifact: `docs/personas/regulator-inspector-sergei-petrov.md`; lines=405
- substance markers: 11 / 17 tracked markers present
- journey appearances: 132 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j11
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Retail Banker Sebastian Vega
- artifact: `docs/personas/retail-banker-sebastian-vega.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 12 detected by title/name token scan; sample=j108, j115, j123, j148, j152, j154, j155, j156, j168, j170, j172, j174
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Retirement Plan Admin Bryce Williams
- artifact: `docs/personas/retirement-plan-admin-bryce-williams.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Returning Intern Jia Han
- artifact: `docs/personas/returning-intern-jia-han.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sales Ae Maya Lindqvist
- artifact: `docs/personas/sales-ae-maya-lindqvist.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 16 detected by title/name token scan; sample=j128, j132, j135, j136, j137, j146, j153, j154, j156, j160, j161, j170
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sales Manager Anthony Costa
- artifact: `docs/personas/sales-manager-anthony-costa.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 53 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j128, j132, j133, j134, j135, j136
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sam Okafor
- artifact: `docs/personas/sam-okafor.md`; lines=407
- substance markers: 11 / 17 tracked markers present
- journey appearances: 141 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sarah Kim Delivery
- artifact: `docs/personas/sarah-kim-delivery.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 129 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j103
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sdr Kofi Asante
- artifact: `docs/personas/sdr-kofi-asante.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 1 detected by title/name token scan; sample=j82
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Security Analyst Anna Petrova
- artifact: `docs/personas/security-analyst-anna-petrova.md`; lines=468
- substance markers: 12 / 17 tracked markers present
- journey appearances: 111 detected by title/name token scan; sample=j01, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Security Guard Stefan Kovacs
- artifact: `docs/personas/security-guard-stefan-kovacs.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 135 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Software Engineer Hugo Tanaka
- artifact: `docs/personas/software-engineer-hugo-tanaka.md`; lines=463
- substance markers: 11 / 17 tracked markers present
- journey appearances: 87 detected by title/name token scan; sample=j114, j116, j117, j118, j119, j120, j121, j122, j123, j124, j125, j126
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Strategic Advisor Rita Almeida
- artifact: `docs/personas/strategic-advisor-rita-almeida.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 119 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Summer Intern Priscilla Sharma
- artifact: `docs/personas/summer-intern-priscilla-sharma.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 160 detected by title/name token scan; sample=j01, j02, j03, j100, j101, j102, j103, j104, j105, j106, j107, j108
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Support Rep Nadia Hassani
- artifact: `docs/personas/support-rep-nadia-hassani.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Sustainability Officer Aiko Brown
- artifact: `docs/personas/sustainability-officer-aiko-brown.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 42 detected by title/name token scan; sample=j02, j101, j102, j103, j104, j105, j106, j107, j108, j109, j110, j111
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Tax Analyst Ji Sung Park
- artifact: `docs/personas/tax-analyst-ji-sung-park.md`; lines=469
- substance markers: 10 / 17 tracked markers present
- journey appearances: 130 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Tomas Garcia Jr Farmer
- artifact: `docs/personas/tomas-garcia-jr-farmer.md`; lines=407
- substance markers: 11 / 17 tracked markers present
- journey appearances: 6 detected by title/name token scan; sample=j154, j156, j160, j165, j167, j92
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Tomas Garcia
- artifact: `docs/personas/tomas-garcia.md`; lines=409
- substance markers: 11 / 17 tracked markers present
- journey appearances: 6 detected by title/name token scan; sample=j154, j156, j160, j165, j167, j92
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Total Rewards Manager Nilufer Demir
- artifact: `docs/personas/total-rewards-manager-nilufer-demir.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 104 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j104, j11
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Trader Mei Lin
- artifact: `docs/personas/trader-mei-lin.md`; lines=406
- substance markers: 10 / 17 tracked markers present
- journey appearances: 180 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j100, j101
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Training Specialist Mehmet Yilmaz
- artifact: `docs/personas/training-specialist-mehmet-yilmaz.md`; lines=467
- substance markers: 10 / 17 tracked markers present
- journey appearances: 22 detected by title/name token scan; sample=j01, j02, j109, j132, j133, j135, j140, j141, j144, j152, j153, j159
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, dual-tenant, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Treasury Ops Sven Eriksson
- artifact: `docs/personas/treasury-ops-sven-eriksson.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 95 detected by title/name token scan; sample=j01, j02, j03, j06, j100, j103, j106, j115, j116, j117, j118, j119
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Ux Researcher Adaeze Nwosu
- artifact: `docs/personas/ux-researcher-adaeze-nwosu.md`; lines=468
- substance markers: 11 / 17 tracked markers present
- journey appearances: 4 detected by title/name token scan; sample=j138, j15, j172, j63
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Venture Partner Lucas Muller
- artifact: `docs/personas/venture-partner-lucas-muller.md`; lines=470
- substance markers: 10 / 17 tracked markers present
- journey appearances: 36 detected by title/name token scan; sample=j100, j114, j126, j132, j133, j134, j136, j137, j148, j154, j155, j156
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Wealth Manager Aamir Khan
- artifact: `docs/personas/wealth-manager-aamir-khan.md`; lines=469
- substance markers: 11 / 17 tracked markers present
- journey appearances: 54 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j132, j133, j134, j135, j136, j137
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Wellness Program Manager Akira Sato
- artifact: `docs/personas/wellness-program-manager-akira-sato.md`; lines=468
- substance markers: 10 / 17 tracked markers present
- journey appearances: 55 detected by title/name token scan; sample=j104, j114, j118, j122, j126, j127, j129, j132, j133, j134, j135, j136
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, cross-context, jurisdiction
- named gaps: none named; retain in regression audit
### Persona — Yejin Park
- artifact: `docs/personas/yejin-park.md`; lines=413
- substance markers: 11 / 17 tracked markers present
- journey appearances: 78 detected by title/name token scan; sample=j01, j02, j03, j04, j05, j06, j07, j08, j09, j10, j101, j102
- cross-context bridges: personal, work, tenant, family, marketplace, regulator, healthcare, finance, cross-context, jurisdiction
- named gaps: none named; retain in regression audit

## §6 Compliance Pack Coverage Scorecard
- Registry compliance packs discovered: 0
- Expected named packs: HIPAA, GDPR, SOC2, EU AI Act, KR PIPA, CSAP, PCI, EU CSRD

### Localization / regional packs
- Expected localization packs: au, br, eu, in, jp, kr, mx, us
- Discovered localization/regional packs: eu, jp, kr, ksa, us-government
- `au`: MISSING; named gaps: no localization pack directory detected
- `br`: MISSING; named gaps: no localization pack directory detected
- `eu`: present as `regional-packs/eu`; artifacts=1; lines=114; policy_markers=0; named gaps=thin localization pack
- `in`: MISSING; named gaps: no localization pack directory detected
- `jp`: present as `regional-packs/jp`; artifacts=1; lines=46; policy_markers=3; named gaps=thin localization pack; data model delta not explicit
- `kr`: present as `regional-packs/kr`; artifacts=2; lines=222; policy_markers=5; named gaps=none named; retain in regression audit
- `mx`: MISSING; named gaps: no localization pack directory detected
- `us`: present as `regional-packs/us-government`; artifacts=1; lines=99; policy_markers=4; named gaps=thin localization pack
- additional localization/regional evidence: ksa, us-government; named gap: reconcile whether additional packs are promoted into the canonical 8-pack matrix

## §7 Standards Coverage Scorecard
- Standard-related files discovered: 195
### Standard lane `documentation-rigor`
- artifacts: 108; lines=63825
- marker hits: 8143
- representative artifacts: docs/STANDARDS-AND-TEMPLATES.md, docs/decisions/ADR-0701-monorepo-capability-live-apex.md, docs/decisions/ADR-0709-general-live-apex.md, docs/architecture/standards-corpus-line-audit-2026-05-21.md, docs/localization-packs/INDEX.md, docs/localization-packs/kr.md, docs/standards/migration-playbook.md, docs/standards/fintech-compliance.md
- named gaps: none named; retain in regression audit
### Standard lane `naming-justification`
- artifacts: 18; lines=15743
- marker hits: 244
- representative artifacts: docs/standards/multispectrum-review-v2.4.0-cadence.md, docs/standards/naming-convention-bnf-v4.md, docs/standards/multi-agent-tool-map.md, docs/standards/capability-tier-matrix.md, docs/standards/documentation-rigor.md, docs/standards/fips-hsm-substrate-root-signing.md, docs/standards/messenger-e2e-encryption-mls.md, docs/standards/anti-patterns.md
- named gaps: none named; retain in regression audit
### Standard lane `layer-enum`
- artifacts: 62; lines=39077
- marker hits: 598
- representative artifacts: docs/decisions/ADR-0701-monorepo-capability-live-apex.md, docs/decisions/ADR-0709-general-live-apex.md, docs/architecture/standards-corpus-line-audit-2026-05-21.md, docs/localization-packs/kr.md, docs/standards/fintech-compliance.md, docs/standards/regulatory-pack-authzpolicy-overlays.md, docs/standards/layer-enum-adr-0105.md, docs/standards/cedar-policy-discipline.md
- named gaps: none named; retain in regression audit
### Standard lane `localization`
- artifacts: 189; lines=79014
- marker hits: 21772
- representative artifacts: docs/STANDARDS-AND-TEMPLATES.md, docs/decisions/ADR-0701-monorepo-capability-live-apex.md, docs/decisions/ADR-0709-general-live-apex.md, docs/architecture/standards-corpus-line-audit-2026-05-21.md, docs/localization-packs/INDEX.md, docs/localization-packs/kr.md, docs/standards/security-review.md, docs/standards/helm-chart-convention.md
- named gaps: none named; retain in regression audit
### Standard lane `canonical-primitives`
- artifacts: 99; lines=60543
- marker hits: 1699
- representative artifacts: docs/STANDARDS-AND-TEMPLATES.md, docs/decisions/ADR-0709-general-live-apex.md, docs/architecture/standards-corpus-line-audit-2026-05-21.md, docs/localization-packs/INDEX.md, docs/localization-packs/kr.md, docs/standards/helm-chart-convention.md, docs/standards/fintech-compliance.md, docs/standards/release-management.md
- named gaps: none named; retain in regression audit

## §8 Capability-Tier Registry Scorecard
- Tier files discovered: 8
- Tier files: registry/capability-tiers/bronze.json, registry/capability-tiers/checkpoint.json, registry/capability-tiers/gold.json, registry/capability-tiers/index.json, registry/capability-tiers/microservice-tier-mapping.yaml, registry/capability-tiers/platinum.json, registry/capability-tiers/silver.json, registry/capability-tiers/vendor-tier-mapping.yaml
- Expected tier matrix: 4 tiers x 70 µservices x 295 vendors
- tier `bronze`: artifacts=1; lines=99; service_mentions=6; vendor_markers=0; named gaps=low explicit service coverage
- tier `silver`: artifacts=1; lines=103; service_mentions=6; vendor_markers=0; named gaps=low explicit service coverage
- tier `gold`: artifacts=1; lines=106; service_mentions=4; vendor_markers=0; named gaps=low explicit service coverage
- tier `platinum`: artifacts=1; lines=109; service_mentions=4; vendor_markers=0; named gaps=low explicit service coverage
- Live service count: 78
- Registry microservice rows mentioned by slug: 70
- Vendor row marker: 295
- Named gaps: registry remains calibrated to 70-service matrix while live corpus now exposes a different service count; reconcile before claiming final parity

## §9 Foundry Pipeline Spec Scorecard
### Foundry spec `specs/microservices/intelligence.json`
- status: present; lines=278; control markers=71
- coverage: pipeline=0 checkpoint=0 supervisor=23 evidence=23
- named gaps: verification path thin
### Foundry spec `registry/capabilities/foundry-internal.json`
- status: present; lines=345; control markers=69
- coverage: pipeline=0 checkpoint=0 supervisor=0 evidence=63
- named gaps: verification path thin
### Foundry spec `registry/capabilities/foundry-supervisor.toml`
- status: present; lines=21; control markers=0
- coverage: pipeline=0 checkpoint=0 supervisor=0 evidence=0
- named gaps: low Foundry pipeline marker density; verification path thin
### Foundry spec `docs/foundry/governance-pipeline-substrate-checkpoint-2026-05-20.md`
- status: present; lines=44; control markers=21
- coverage: pipeline=4 checkpoint=4 supervisor=0 evidence=4
- named gaps: none named; retain in regression audit
### Foundry spec `docs/products/foundry/PHASE-00-SPEC.md`
- status: present; lines=406; control markers=51
- coverage: pipeline=1 checkpoint=1 supervisor=0 evidence=16
- named gaps: none named; retain in regression audit
### Foundry spec `docs/products/foundry/PRD.md`
- status: present; lines=2648; control markers=860
- coverage: pipeline=6 checkpoint=2 supervisor=1 evidence=422
- named gaps: none named; retain in regression audit
### Foundry spec `docs/foundry/supervisor/architecture.md`
- status: present; lines=19; control markers=7
- coverage: pipeline=0 checkpoint=0 supervisor=6 evidence=0
- named gaps: low Foundry pipeline marker density; verification path thin
### Foundry spec `docs/advanced-cicd/branch-pipeline/governance-pipeline-mirror.md`
- status: present; lines=154; control markers=45
- coverage: pipeline=7 checkpoint=0 supervisor=0 evidence=9
- named gaps: verification path thin

## §10 CI Gate Lane Crate Scorecard
- Governance crates discovered: 8
- Expected scaffold count: 8
- Expected implementation count: 4
- Implementation-grade crates by audit heuristic: 0
### Crate `oya-governance-audit-event-emission`
- scaffold status: present; files=4; rust_files=2; lines=114
- implementation status: scaffold; cli=False; test_markers=4
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-byok-disambiguation`
- scaffold status: present; files=7; rust_files=3; lines=674
- implementation status: partial-implementation; cli=False; test_markers=8
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-capability-tier-coverage`
- scaffold status: present; files=4; rust_files=2; lines=113
- implementation status: scaffold; cli=False; test_markers=4
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-cedar-coverage`
- scaffold status: present; files=4; rust_files=2; lines=113
- implementation status: scaffold; cli=False; test_markers=4
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-naming-justifications`
- scaffold status: present; files=7; rust_files=3; lines=712
- implementation status: partial-implementation; cli=False; test_markers=7
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-no-template-stamping`
- scaffold status: present; files=11; rust_files=3; lines=683
- implementation status: partial-implementation; cli=False; test_markers=8
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-pack-overlay-completeness`
- scaffold status: present; files=4; rust_files=2; lines=113
- implementation status: scaffold; cli=False; test_markers=4
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected
### Crate `oya-governance-substance-bar`
- scaffold status: present; files=7; rust_files=3; lines=662
- implementation status: partial-implementation; cli=False; test_markers=8
- named gaps: not implementation-grade by line/CLI/test heuristic; no CLI surface detected

## §11 Customer-Facing Material Scorecard
- Customer-facing files discovered by path/name scan: 358
- Total customer-facing lines: 123945
### Customer lane `demo scripts`
- artifacts: 17; lines=6649
- persona/journey anchors: persona=460 journey=6
- compliance anchors: 286
- representative artifacts: docs/runbooks/demo-environment-reset.md, docs/customer-success/demo-scripts/conglomerate-tenant-demo.md, docs/customer-success/demo-scripts/healthcare-system-demo.md, docs/customer-success/demo-scripts/fortune-500-erp-replacement-demo.md, docs/customer-success/demo-scripts/compliance-pack-activation-demo.md, docs/customer-success/demo-scripts/agentic-workflow-studio-demo.md, docs/customer-success/demo-scripts/unified-ecosystem-thesis-demo.md, docs/customer-success/demo-scripts/financial-services-vertical-demo.md, docs/customer-success/demo-scripts/mid-market-crm-replacement-demo.md, docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md
- named gaps: none named; retain in regression audit
### Customer lane `tutorials`
- artifacts: 73; lines=19413
- persona/journey anchors: persona=251 journey=9
- compliance anchors: 621
- representative artifacts: docs/tutorials/consent-cascade-across-microservices.md, docs/tutorials/ai-assisted-document-summarization.md, docs/tutorials/cross-tenant-channel-setup.md, docs/tutorials/marketplace-list-sell-buy.md, docs/tutorials/ontology-projection-from-external-source.md, docs/tutorials/capability-tier-upgrade-bronze-to-platinum.md, docs/tutorials/multi-pack-tenant-activation.md, docs/tutorials/quickstart-new-user-day-one.md, docs/tutorials/data-subject-erasure-request-handling.md, docs/tutorials/workflow-studio-build-employee-onboarding.md
- named gaps: none named; retain in regression audit
### Customer lane `onboarding`
- artifacts: 215; lines=77665
- persona/journey anchors: persona=6031 journey=18700
- compliance anchors: 3617
- representative artifacts: templates/checklists/tenant-onboarding.md, templates/checklists/new-team-onboarding.md, templates/checklists/regional-pack-onboarding.md, templates/checklists/vertical-onboarding.md, docs/runbooks/tenant-onboarding.md, docs/runbooks/pack-onboarding.md, docs/runbooks/external-dep-onboarding.md, docs/runbooks/design-partner-onboarding.md, docs/tutorials/workflow-studio-build-employee-onboarding.md, docs/gtm/tenant-onboarding-90-day-program.md
- named gaps: none named; retain in regression audit
### Customer lane `investor materials`
- artifacts: 19; lines=8988
- persona/journey anchors: persona=62 journey=239
- compliance anchors: 121
- representative artifacts: docs/personas/investor-lp-aanya-kapoor.md, docs/investor/market-sizing-tam-sam-som.md, docs/investor/company-overview-deck.md, docs/investor/unit-economics-and-pricing-model.md, docs/investor/competitive-landscape-and-positioning.md, docs/investor/moat-and-defensibility.md, docs/investor/ask-and-use-of-funds.md, docs/user-journeys/j172-lev-kahn-investor-relations-shareholder-meeting-livestream/README.md, docs/user-journeys/j172-lev-kahn-investor-relations-shareholder-meeting-livestream/handshake.md, docs/user-journeys/j172-lev-kahn-investor-relations-shareholder-meeting-livestream/story.md
- named gaps: none named; retain in regression audit
### Customer lane `go-to-market`
- artifacts: 47; lines=19251
- persona/journey anchors: persona=385 journey=824
- compliance anchors: 943
- representative artifacts: docs/GTM-PLAN.md, docs/personas/customer-success-manager-sofia-rezende.md, docs/personas/sales-ae-maya-lindqvist.md, docs/personas/sales-manager-anthony-costa.md, docs/gtm/customer-success-quarterly-business-review-template.md, docs/gtm/expansion-and-upsell-playbook.md, docs/gtm/tenant-prospect-to-active-stages.md, docs/gtm/migration-from-incumbent-playbook.md, docs/gtm/solutions-engineering-runbook.md, docs/gtm/tenant-onboarding-90-day-program.md
- named gaps: none named; retain in regression audit

## §12 Test Coverage Scorecard
- Test/benchmark/scenario files discovered: 469
- Test/benchmark/scenario lines: 142926
- Cross-service integration scenario candidates: 223
- Benchmark candidates: 80
- µservice `analytics`: test-plan=present; test_mentions=41; integration_markers=59; benchmark_markers=379; named gaps=none named; retain in regression audit
- µservice `api-gateway`: test-plan=present; test_mentions=31; integration_markers=83; benchmark_markers=355; named gaps=none named; retain in regression audit
- µservice `application`: test-plan=present; test_mentions=43; integration_markers=42; benchmark_markers=47; named gaps=none named; retain in regression audit
- µservice `audit-chain`: test-plan=present; test_mentions=1678; integration_markers=1135; benchmark_markers=1078; named gaps=none named; retain in regression audit
- µservice `calendar`: test-plan=present; test_mentions=219; integration_markers=491; benchmark_markers=324; named gaps=none named; retain in regression audit
- µservice `cell`: test-plan=present; test_mentions=111; integration_markers=972; benchmark_markers=750; named gaps=none named; retain in regression audit
- µservice `cloud-billing`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=8; named gaps=cross-service integration scenario absent
- µservice `cloud-billing-tax`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=16; named gaps=cross-service integration scenario absent
- µservice `cloud-data`: test-plan=present; test_mentions=5; integration_markers=0; benchmark_markers=28; named gaps=cross-service integration scenario absent
- µservice `cloud-iac`: test-plan=present; test_mentions=58; integration_markers=175; benchmark_markers=481; named gaps=none named; retain in regression audit
- µservice `cloud-iam`: test-plan=present; test_mentions=15; integration_markers=3; benchmark_markers=12; named gaps=none named; retain in regression audit
- µservice `cloud-k8s`: test-plan=present; test_mentions=155; integration_markers=88; benchmark_markers=334; named gaps=none named; retain in regression audit
- µservice `cloud-kms`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=16; named gaps=cross-service integration scenario absent
- µservice `cloud-network`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=19; named gaps=cross-service integration scenario absent
- µservice `cloud-network-dns`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=33; named gaps=cross-service integration scenario absent
- µservice `cloud-secrets`: test-plan=present; test_mentions=60; integration_markers=418; benchmark_markers=562; named gaps=none named; retain in regression audit
- µservice `cloud-storage`: test-plan=present; test_mentions=4; integration_markers=0; benchmark_markers=30; named gaps=cross-service integration scenario absent
- µservice `comms-email`: test-plan=present; test_mentions=44; integration_markers=76; benchmark_markers=89; named gaps=none named; retain in regression audit
- µservice `community`: test-plan=present; test_mentions=1221; integration_markers=1363; benchmark_markers=930; named gaps=none named; retain in regression audit
- µservice `compliance`: test-plan=present; test_mentions=1172; integration_markers=746; benchmark_markers=930; named gaps=none named; retain in regression audit
- µservice `connector`: test-plan=present; test_mentions=1172; integration_markers=782; benchmark_markers=188; named gaps=none named; retain in regression audit
- µservice `consent-graph`: test-plan=present; test_mentions=45; integration_markers=221; benchmark_markers=662; named gaps=none named; retain in regression audit
- µservice `contact-center`: test-plan=present; test_mentions=5333; integration_markers=42; benchmark_markers=6471; named gaps=none named; retain in regression audit
- µservice `contract-lifecycle-management`: test-plan=present; test_mentions=5344; integration_markers=49; benchmark_markers=6019; named gaps=none named; retain in regression audit
- µservice `crm`: test-plan=present; test_mentions=652; integration_markers=145; benchmark_markers=294; named gaps=none named; retain in regression audit
- µservice `data-pipeline`: test-plan=present; test_mentions=4769; integration_markers=63; benchmark_markers=5968; named gaps=none named; retain in regression audit
- µservice `data-warehouse`: test-plan=present; test_mentions=5015; integration_markers=61; benchmark_markers=6179; named gaps=none named; retain in regression audit
- µservice `design-collaboration`: test-plan=present; test_mentions=5381; integration_markers=156; benchmark_markers=6302; named gaps=none named; retain in regression audit
- µservice `detection`: test-plan=present; test_mentions=10; integration_markers=0; benchmark_markers=73; named gaps=cross-service integration scenario absent
- µservice `developer-sdk`: test-plan=present; test_mentions=140; integration_markers=90; benchmark_markers=122; named gaps=none named; retain in regression audit
- µservice `docs`: test-plan=present; test_mentions=33; integration_markers=40; benchmark_markers=354; named gaps=none named; retain in regression audit
- µservice `drive`: test-plan=present; test_mentions=760; integration_markers=694; benchmark_markers=571; named gaps=none named; retain in regression audit
- µservice `feature-flags`: test-plan=present; test_mentions=52; integration_markers=88; benchmark_markers=260; named gaps=none named; retain in regression audit
- µservice `financial-planning`: test-plan=present; test_mentions=8163; integration_markers=2750; benchmark_markers=6691; named gaps=none named; retain in regression audit
- µservice `finops-portal`: test-plan=present; test_mentions=637; integration_markers=214; benchmark_markers=442; named gaps=none named; retain in regression audit
- µservice `forms`: test-plan=present; test_mentions=48; integration_markers=40; benchmark_markers=437; named gaps=none named; retain in regression audit
- µservice `foundry`: test-plan=present; test_mentions=1091; integration_markers=233; benchmark_markers=639; named gaps=none named; retain in regression audit
- µservice `global-trade`: test-plan=present; test_mentions=501; integration_markers=114; benchmark_markers=236; named gaps=none named; retain in regression audit
- µservice `governance`: test-plan=present; test_mentions=162; integration_markers=197; benchmark_markers=468; named gaps=none named; retain in regression audit
- µservice `healthcare-integration`: test-plan=present; test_mentions=4794; integration_markers=54; benchmark_markers=5979; named gaps=none named; retain in regression audit
- µservice `identity`: test-plan=present; test_mentions=2481; integration_markers=3802; benchmark_markers=1006; named gaps=none named; retain in regression audit
- µservice `incident-management`: test-plan=present; test_mentions=5532; integration_markers=39; benchmark_markers=6657; named gaps=none named; retain in regression audit
- µservice `intelligence`: test-plan=present; test_mentions=685; integration_markers=1140; benchmark_markers=780; named gaps=none named; retain in regression audit
- µservice `itsm`: test-plan=present; test_mentions=5665; integration_markers=66; benchmark_markers=6629; named gaps=none named; retain in regression audit
- µservice `learning-management`: test-plan=present; test_mentions=5311; integration_markers=34; benchmark_markers=6456; named gaps=none named; retain in regression audit
- µservice `mail`: test-plan=present; test_mentions=1061; integration_markers=1737; benchmark_markers=774; named gaps=none named; retain in regression audit
- µservice `marketing-automation`: test-plan=present; test_mentions=5319; integration_markers=34; benchmark_markers=6423; named gaps=none named; retain in regression audit
- µservice `marketplace`: test-plan=present; test_mentions=948; integration_markers=706; benchmark_markers=311; named gaps=none named; retain in regression audit
- µservice `meet`: test-plan=present; test_mentions=183; integration_markers=361; benchmark_markers=464; named gaps=none named; retain in regression audit
- µservice `messenger`: test-plan=present; test_mentions=751; integration_markers=667; benchmark_markers=400; named gaps=none named; retain in regression audit
- µservice `network`: test-plan=present; test_mentions=64; integration_markers=151; benchmark_markers=618; named gaps=none named; retain in regression audit
- µservice `notes`: test-plan=present; test_mentions=261; integration_markers=480; benchmark_markers=521; named gaps=none named; retain in regression audit
- µservice `observability`: test-plan=present; test_mentions=982; integration_markers=2493; benchmark_markers=1168; named gaps=none named; retain in regression audit
- µservice `ontology`: test-plan=present; test_mentions=584; integration_markers=384; benchmark_markers=791; named gaps=none named; retain in regression audit
- µservice `ops-dashboard-control-center`: test-plan=present; test_mentions=63; integration_markers=279; benchmark_markers=537; named gaps=none named; retain in regression audit
- µservice `payments`: test-plan=present; test_mentions=2625; integration_markers=1013; benchmark_markers=711; named gaps=none named; retain in regression audit
- µservice `performance-management`: test-plan=present; test_mentions=5311; integration_markers=34; benchmark_markers=6412; named gaps=none named; retain in regression audit
- µservice `plant-maintenance`: test-plan=present; test_mentions=763; integration_markers=94; benchmark_markers=223; named gaps=none named; retain in regression audit
- µservice `plugin-app-store`: test-plan=present; test_mentions=470; integration_markers=163; benchmark_markers=395; named gaps=none named; retain in regression audit
- µservice `production-planning`: test-plan=present; test_mentions=868; integration_markers=247; benchmark_markers=207; named gaps=none named; retain in regression audit
- µservice `quality-management`: test-plan=present; test_mentions=546; integration_markers=102; benchmark_markers=224; named gaps=none named; retain in regression audit
- µservice `real-estate`: test-plan=present; test_mentions=469; integration_markers=89; benchmark_markers=168; named gaps=none named; retain in regression audit
- µservice `recordings`: test-plan=present; test_mentions=76; integration_markers=303; benchmark_markers=484; named gaps=none named; retain in regression audit
- µservice `sheets`: test-plan=present; test_mentions=53; integration_markers=46; benchmark_markers=351; named gaps=none named; retain in regression audit
- µservice `shorts`: test-plan=present; test_mentions=78; integration_markers=320; benchmark_markers=586; named gaps=none named; retain in regression audit
- µservice `sites`: test-plan=present; test_mentions=32; integration_markers=34; benchmark_markers=444; named gaps=none named; retain in regression audit
- µservice `slides`: test-plan=present; test_mentions=29; integration_markers=39; benchmark_markers=361; named gaps=none named; retain in regression audit
- µservice `social`: test-plan=present; test_mentions=49; integration_markers=368; benchmark_markers=516; named gaps=none named; retain in regression audit
- µservice `supply-chain-planning`: test-plan=present; test_mentions=4559; integration_markers=4171; benchmark_markers=224; named gaps=none named; retain in regression audit
- µservice `tasks`: test-plan=present; test_mentions=69; integration_markers=43; benchmark_markers=369; named gaps=none named; retain in regression audit
- µservice `tenancy`: test-plan=present; test_mentions=1286; integration_markers=1292; benchmark_markers=852; named gaps=none named; retain in regression audit
- µservice `translate`: test-plan=present; test_mentions=74; integration_markers=51; benchmark_markers=448; named gaps=none named; retain in regression audit
- µservice `treasury`: test-plan=present; test_mentions=530; integration_markers=113; benchmark_markers=265; named gaps=none named; retain in regression audit
- µservice `warehouse`: test-plan=present; test_mentions=482; integration_markers=96; benchmark_markers=224; named gaps=none named; retain in regression audit
- µservice `whiteboard`: test-plan=present; test_mentions=5037; integration_markers=120; benchmark_markers=6175; named gaps=none named; retain in regression audit
- µservice `workflow-engine`: test-plan=present; test_mentions=2310; integration_markers=840; benchmark_markers=1098; named gaps=none named; retain in regression audit
- µservice `workflow-studio`: test-plan=present; test_mentions=442; integration_markers=405; benchmark_markers=283; named gaps=none named; retain in regression audit
- µservice `workplace-integration`: test-plan=present; test_mentions=938; integration_markers=119; benchmark_markers=105; named gaps=none named; retain in regression audit

## §13 Cumulative Substantive Lines Authored Estimate (this remediation session — by workstream)
- ADR decisions and ADR-MS files: estimated substantive corpus lines=162678; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- µservice specs/docs/contracts/policies: estimated substantive corpus lines=1879627; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Journey dossiers: estimated substantive corpus lines=588298; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Persona dossiers: estimated substantive corpus lines=58490; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Compliance and localization packs: estimated substantive corpus lines=0; audit confidence=low; note=live-line estimate from remediation corpus, not a git-diff attribution
- Standards/control-surface docs: estimated substantive corpus lines=79074; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Foundry pipeline specs: estimated substantive corpus lines=3915; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- CI gate lane crates: estimated substantive corpus lines=3184; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Customer-facing material: estimated substantive corpus lines=123945; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Tests, scenarios, benchmarks: estimated substantive corpus lines=142926; audit confidence=medium; note=live-line estimate from remediation corpus, not a git-diff attribution
- Total estimated substantive lines across scored workstreams: 3042137
- Named limitation: line estimates are corpus-presence estimates; exact session-authored deltas require a clean baseline or commit-range diff, which the dirty working tree does not provide safely

## §14 Recommended Wave-4 Top-30 Priorities
1. owner: Architecture Decision Owner; ETA: 2026-05-21; deliverable: Close ADR numeric continuity for missing ADR-0001..ADR-0321 slots with present/reserved/deprecated status records; verification: update relevant manifest/spec and rerun corpus scorecard delta.
2. owner: Architecture Decision Owner; ETA: 2026-05-22; deliverable: Normalize ADR status fields and Lore-style decision trailers across thin ADRs; verification: update relevant manifest/spec and rerun corpus scorecard delta.
3. owner: Service Registry Owner; ETA: 2026-05-21; deliverable: Reconcile live microservice count with 70-service capability matrix and the user-requested 79-service gate; verification: update relevant manifest/spec and rerun corpus scorecard delta.
4. owner: Service Registry Owner; ETA: 2026-05-21; deliverable: Publish generated service roster manifest with explicit inclusions, exclusions, aliases, and retired service names; verification: update relevant manifest/spec and rerun corpus scorecard delta.
5. owner: Capability Tier Owner; ETA: 2026-05-22; deliverable: Regenerate four-tier registry after roster reconciliation and preserve vendor-row count evidence; verification: update relevant manifest/spec and rerun corpus scorecard delta.
6. owner: Policy Owner; ETA: 2026-05-23; deliverable: Add Cedar/policy artifact references for every service row missing policy evidence; verification: update relevant manifest/spec and rerun corpus scorecard delta.
7. owner: Contract Owner; ETA: 2026-05-23; deliverable: Add explicit contract version conformance blocks for every service missing contract markers; verification: update relevant manifest/spec and rerun corpus scorecard delta.
8. owner: Threat Modeling Owner; ETA: 2026-05-24; deliverable: Create per-service threat-model stubs for services with zero threat markers; verification: update relevant manifest/spec and rerun corpus scorecard delta.
9. owner: QA Owner; ETA: 2026-05-24; deliverable: Create per-service test-plan stubs for services with zero test markers; verification: update relevant manifest/spec and rerun corpus scorecard delta.
10. owner: Integration QA Owner; ETA: 2026-05-25; deliverable: Create cross-service integration scenario matrix linking journeys to service pairs; verification: update relevant manifest/spec and rerun corpus scorecard delta.
11. owner: Performance QA Owner; ETA: 2026-05-25; deliverable: Create benchmark matrix for latency/throughput-sensitive services; verification: update relevant manifest/spec and rerun corpus scorecard delta.
12. owner: Journey Owner; ETA: 2026-05-26; deliverable: Normalize explicit persona fields in journeys currently inferred from titles only; verification: update relevant manifest/spec and rerun corpus scorecard delta.
13. owner: Journey Owner; ETA: 2026-05-26; deliverable: Raise microservice citation density for journeys below two service citations; verification: update relevant manifest/spec and rerun corpus scorecard delta.
14. owner: Regulatory Owner; ETA: 2026-05-26; deliverable: Add regulatory anchors to journeys with zero compliance markers; verification: update relevant manifest/spec and rerun corpus scorecard delta.
15. owner: Persona Owner; ETA: 2026-05-27; deliverable: Patch persona dossiers with weak journey appearance evidence; verification: update relevant manifest/spec and rerun corpus scorecard delta.
16. owner: Persona Owner; ETA: 2026-05-27; deliverable: Patch persona dossiers with weak cross-context bridges; verification: update relevant manifest/spec and rerun corpus scorecard delta.
17. owner: Compliance Owner; ETA: 2026-05-28; deliverable: Verify manifest schema completeness for each of the eight compliance packs; verification: update relevant manifest/spec and rerun corpus scorecard delta.
18. owner: Localization Owner; ETA: 2026-05-28; deliverable: Promote or retire additional regional packs outside the canonical eight-pack matrix; verification: update relevant manifest/spec and rerun corpus scorecard delta.
19. owner: Data Model Owner; ETA: 2026-05-29; deliverable: Add explicit data-model delta tables for compliance and localization packs missing them; verification: update relevant manifest/spec and rerun corpus scorecard delta.
20. owner: Standards Owner; ETA: 2026-05-29; deliverable: Promote documentation-rigor/naming/layer/localization/canonical-primitive rules into machine-readable checks; verification: update relevant manifest/spec and rerun corpus scorecard delta.
21. owner: Foundry Owner; ETA: 2026-05-30; deliverable: Add acceptance tests to every Foundry pipeline spec that has low verification marker density; verification: update relevant manifest/spec and rerun corpus scorecard delta.
22. owner: CI Gate Owner; ETA: 2026-05-30; deliverable: Finish implementation-grade status for the two governance crates nearest completion; verification: update relevant manifest/spec and rerun corpus scorecard delta.
23. owner: CI Gate Owner; ETA: 2026-05-31; deliverable: Add CLI/test surfaces to scaffold-only governance crates or explicitly mark them deferred; verification: update relevant manifest/spec and rerun corpus scorecard delta.
24. owner: Customer Success Owner; ETA: 2026-05-31; deliverable: Bind demo scripts to j-codes, personas, regulatory proof points, and service citations; verification: update relevant manifest/spec and rerun corpus scorecard delta.
25. owner: Product Education Owner; ETA: 2026-06-01; deliverable: Normalize tutorials/onboarding to include prerequisites, expected outputs, and failure recovery; verification: update relevant manifest/spec and rerun corpus scorecard delta.
26. owner: Investor Narrative Owner; ETA: 2026-06-01; deliverable: Tie investor materials to quantified coverage evidence instead of broad platform claims; verification: update relevant manifest/spec and rerun corpus scorecard delta.
27. owner: Evidence Automation Owner; ETA: 2026-06-02; deliverable: Add a reproducible scorecard generator or manifest so future audits are not manual-only; verification: update relevant manifest/spec and rerun corpus scorecard delta.
28. owner: VCS Governance Owner; ETA: 2026-06-02; deliverable: Wire scorecard line-count evidence into Oya VCS promote bundles; verification: update relevant manifest/spec and rerun corpus scorecard delta.
29. owner: Review Owner; ETA: 2026-06-03; deliverable: Run adversarial review of the Wave-4 patch set against this scorecard gaps list; verification: update relevant manifest/spec and rerun corpus scorecard delta.
30. owner: Release Gate Owner; ETA: 2026-06-05; deliverable: Declare Wave-4 PASS only after ADR, service registry, policy, threat, and test gaps hit zero or named waivers; verification: update relevant manifest/spec and rerun corpus scorecard delta.

## §15 Verdict (PASS / NEEDS-WAVE-4 / BLOCKED)
- Verdict: NEEDS-WAVE-4
- Blocked?: no hard blocker to remediation; corpus is auditable and promotable as a scorecard artifact
- PASS criteria not yet met because: 57 ADR numeric slots missing from ADR-0001..ADR-0321; live microservice count is 78, not requested 79; capability tier registry still declares/targets a 70-service matrix while live services differ; only 0 governance crates are implementation-grade by audit heuristic, below requested 4; 6 services lack threat-model markers
- Clean halt condition: after Oya VCS verify/done/promote evidence succeeds with this file line count

Named evidence samples for highest-risk gaps:
- Missing ADR slots: ADR-0012, ADR-0033, ADR-0068, ADR-0070, ADR-0071, ADR-0072, ADR-0073, ADR-0074, ADR-0075, ADR-0076, ADR-0077, ADR-0078, ADR-0079, ADR-0080, ADR-0081, ADR-0082, ADR-0084, ADR-0085, ADR-0086, ADR-0087, ADR-0088, ADR-0089, ADR-0125, ADR-0126, ADR-0127, ADR-0224, ADR-0225, ADR-0226, ADR-0227, ADR-0228, ADR-0229, ADR-0230, ADR-0231, ADR-0232, ADR-0233, ADR-0256, ADR-0259, ADR-0260, ADR-0261, ADR-0262, ADR-0264, ADR-0265, ADR-0266, ADR-0267, ADR-0268, ADR-0269, ADR-0270, ADR-0271, ADR-0274, ADR-0275, ADR-0277, ADR-0278, ADR-0279, ADR-0281, ADR-0282, ADR-0283, ADR-0285, ADR-0286, ADR-0287, ADR-0288, ADR-0289, ADR-0290, ADR-0291
- Services missing policy markers: none
- Services missing test markers: none
- Services missing threat markers: cloud-billing-tax, cloud-data, cloud-iam, cloud-network, cloud-network-dns, cloud-storage

_Scorecard generated from local corpus scan; final_line_count=4888._
