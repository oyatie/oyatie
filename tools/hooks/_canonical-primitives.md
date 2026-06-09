# Canonical Primitives Cheat Sheet — 2026-05-18

Single source of truth for hook payloads. Hooks `cat` or `grep` this file rather
than duplicating strings. Keep sections machine-parseable (no nested bullets).

---

## VCS / Git

Canonical: plain `git`. The `oya git` wrapper and the `oya vcs` ratchet are
RETIRED per ADR-0363 (substrate = git + GitHub Actions + cloud-scm (interim)). They no
longer exist as commands — do NOT use them.

Coordination (per agent lane):
  isolated worktree branch per agent lane (scaffold-managed; one lane = one worktree)
  commit and push on that lane
  open a PR against `dev`             # enters the governance pipeline
  cloud-ci/oya-ci pipeline posts `oya-ci-required` + reviewer APPROVE gate merge readiness

`oya` local verifier output is shift-left evidence only. It is never protected-branch
merge authority and never replaces the cloud-ci/oya-ci `oya-ci-required` status.

Authority: ADR-0363 (retire bespoke agentic-VCS; GitHub substrate; oya = gate engine)

---

## Contracts

OpenAPI version : 3.2.0  (NOT 3.3, NOT 3.0.0, NOT 3.1.0)
AsyncAPI version: 3.1.0  (NOT 3.0.0, NOT 2.x)
Schema language : proto3  (NOT proto2)
Reference (OpenAPI): https://spec.openapis.org/oas/v3.2.0
Reference (AsyncAPI): https://www.asyncapi.com/docs/reference/specification/v3.1.0

---

## Substrate — in-memory KV / cache / pubsub / streams

Canonical: Valkey 8.x (Linux Foundation BSD-3-Clause fork of Redis 7.2.4; mainline 8.x)
Retired:   Redis 7.4+ (Redis Inc. SSPLv1 / RSALv2 dual-license since 2024-03-20) — RETIRED-IN-FAVOR-OF-VALKEY per ADR-0336
Fallback:  pre-7.4 Redis (BSD-3-Clause) — license-clean but non-canonical (no upstream patches, no hyperscaler-managed offering)
Forbidden: DragonflyDB (BSL-1.1 on forbidden-license list per dependency-policy §2)

Wire protocol:   RESP3 (preserved across Redis→Valkey swap; client libraries unchanged)
Client crates:   redis-rs / fred / deadpool-redis (upstream crate names preserved; in-tree crate name pattern is oya-<microservice>-adapter-valkey[-<topology>])
Hyperscaler:     AWS ElastiCache for Valkey (GA 2024-11-04); Google Memorystore for Valkey (GA 2024-09-24); Oracle Cloud Cache with Valkey (GA 2025-01-21, OCI Always Free)

Authority: ADR-0336 (Valkey-not-Redis substrate); ADR-0013 + ADR-0045 (license substitution precedents); ADR-0211 (Class C OSS substrate preference); dependency-policy §2 + §2.1 + §7
Migration: corpus-wide vocabulary rewrite is Wave 15-Valkey (queued; dispatches after ADR-0336 Accepted; per-µservice codex buckets under ADR-0322 substance-bar + ADR-0324 anti-template discipline)

Counterpart-fact preservation: external-product Redis references (Discord/Twitch/Stripe/GitHub/Shopify) remain quote-bound per ADR-0336 §D-11. Do NOT migrate counterpart facts.

---

## AI Substrate

microservices/intelligence/  — canonical AI substrate (Layer A + Layer B) per ADR-0255 KS#14; absorbs retired AI-runtime scope per ADR-0335 (Wave 15I)
microservices/intelligence/       — RETIRED 2026-05-21 per ADR-0335; see microservices/intelligence/RETIRED.md
Authority: ADR-0255 (intelligence two-layer); ADR-0335 (AI-runtime retirement + Hermes drop); ADR-0247 (self-modification via oyatie.intelligence.* Cedar principals — principal namespace persists)

Hermes terminology: RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36. Do NOT introduce "Hermes" as a canonical primitive in new content.

---

## Pod Runtime Tier (per ADR-0338)

Canonical per-µservice manifest field: pod_runtime_tier ∈ {0, 1, 2, 3}
Numbering aligns with ADR-0248 cellular tier convention: Tier 0 = highest blast-radius / most isolated.

Tier 0 — Tenant-customer untrusted code            → Kata Containers + Cloud Hypervisor (kata-pool, RuntimeClass kata-cloud-hypervisor)
  Floor surfaces: Wasmtime sandbox host; workflow-studio user workflows; marketplace plugin executors;
                  agent-runtime tenant capabilities; developer-sdk uploaded modules.
Tier 1 — Substrate touching tenant data plane      → Kata Containers + Cloud Hypervisor (kata-pool, RuntimeClass kata-cloud-hypervisor)
  Floor surfaces: cloud-iam · cloud-kms · cloud-secrets · audit-chain · messenger (MLS keys) ·
                  payments · intelligence (transport / provider-router / BYOK).
Tier 2 — First-party app µservices                  → runc (runc-pool, RuntimeClass runc); DEFAULT for new µservices.
Tier 3 — Edge / static / perf-critical              → runc on dedicated edge-tuned nodepool (runc-edge-pool, RuntimeClass runc-edge)
  Floor surfaces: api-gateway data-plane · Envoy edge · ztunnel · CDN edge cache.

Per-cell nodepool topology: kata-pool + runc-pool [+ runc-edge-pool when Tier 3 workloads present].
RuntimeClass allowlist: {kata-cloud-hypervisor, runc, runc-edge} only.
Admission gate: Kyverno ClusterPolicy enforce-pod-runtime-tier (validationFailureAction audit at landing; enforce after sunset).
CI lane: oya-check-pod-runtime-tier (REPORT-ONLY at landing; BLOCKER after corpus-wide manifest declarations land).

Why this exists: Kata-everywhere costs ~30-40% pod density + 200-500 ms cold-start for no security gain on
trusted first-party code that is already namespace-isolated + mTLS-mediated + Cedar-policed + supply-chain-signed.
Tenant-customer code paths and substrate µservices touching tenant data plane MUST have VM-isolation; the rest run under runc.

Default for new µservices: Tier 2 (no evidence required). Tier 0 / Tier 1 / Tier 3 declarations require manifest
pod_runtime_tier_justification + pod_runtime_tier_surface_evidence citation. Tier 2 → Tier 1 promotion requires the
ADR-0338 D-10 evidence pack at microservices/<name>/IPs/IP-tier-promotion-2-to-1.md and architecture-reviewer +
security-reviewer approval. Quarterly tier review walks the corpus per ADR-0338 D-8.

Authority: ADR-0338 (pod runtime tier 0..3); amends ADR-0254 (K8s + Cloud Hypervisor + Kata invariant); co-varies
with ADR-0248 (cellular tier numbering); admission gate via ADR-0183 (policy-engine separation: Kyverno admission).
Migration sub-wave: 15S-Pod-Runtime-Tier-declaration (queued; dispatches after ADR-0338 Accepted).

---

## Cell Promotion Gates (per ADR-0341)

Canonical: explicit machine-checkable promotion-gate criteria for every cellular tier-edge per ADR-0248 Tier 0..Tier 4
(Tier 0 = highest blast-radius / most isolated; Tier 4 = best-effort / edge / lowest blast-radius — convention preserved verbatim).

Six gate inputs (AND-evaluated for promotion; OR-evaluated for evaluationtion per ADR-0341 §D-5 with stricter thresholds):
  Gate 1 — Error budget intact (≥ 99 % of SLO budget remaining on current tier; OpenSLO + ADR-0186)
  Gate 2 — Warm-soak floor (≥ N days in current tier; per-edge floors below)
  Gate 3 — Canary cohort SLO compliance ≥ 99.5 % over warm-soak window (ADR-0186 canary cohort)
  Gate 4 — Cell-mesh health: cross-cell call success ≥ 99.95 % over warm-soak window (ADR-0044 mesh tunnel)
  Gate 5 — tenant-class coverage: both evaluation_trial + paid present on current tier (ADR-0330)
  Gate 6 — compliance-pack coverage: every applicable pack signed off (ADR-0251)

Per-edge warm-soak + quiet-window floors:
  Tier 0 → 1: warm-soak 7 days + quiet window 24 hours
  Tier 1 → 2: warm-soak 14 days + quiet window 48 hours
  Tier 2 → 3: warm-soak 28 days + quiet window 96 hours
  Tier 3 → 4: warm-soak 56 days + quiet window 168 hours
  Inverse edges (Tier 4 → 3 → 2 → 1 → 0; cell graduating into more-critical tier): symmetric floors apply.

Auto-promotion: cell-orchestrator µservice (running inside tenancy + observability per ADR-0148; oyatie.intelligence.* Cedar
namespace per ADR-0247; pod_runtime_tier 1 per ADR-0338) evaluates gates every 60 s + fires promotion event when all six
gates pass AND quiet window elapses without alert burst. Signed cell.promotion.executed audit-chain row per ADR-0263 +
Kyverno-admitted node label mutation + manifest cell_promotion_history update via self-modification PR per ADR-0247.

Evaluationtion: same evaluator with STRICTER thresholds (error budget < 95 %; canary SLO < 99 %; mesh < 99.9 %; pack revocation);
no quiet window (evaluationtion is immediate to protect blast-radius); 24-hour cooldown before re-entering promotion path.

Emergency override (rare): multi-party authorization (incident commander + on-call SRE + security-reviewer signatures);
emits cell.promotion.override audit-chain event; skips warm-soak floor + gate AND-condition but NOT the audit trail;
evidence pack required at microservices/<name>/IPs/IP-cell-promotion-override-<cell-id>-<timestamp>.md.

CI lane: oya-check-cell-promotion-gates (REPORT-ONLY at landing; BLOCKER after Wave 15T-Cell-Promotion-Gates lands).
Kyverno admission: enforce-cell-promotion-gates ClusterPolicy refuses cellular topology mutations without cell-orchestrator-signed
promotion-event attestation.

Manifest fields (per ADR-0341 §D-10): cell_promotion_gates {applicable_tiers, cellular_deployment_pattern, default_initial_tier,
promotion_window_per_edge_seconds, compliance_pack_floor} + cell_promotion_history [{event_id, from_tier, to_tier,
evaluator_version, gate_snapshot_sha256}].

Authority: ADR-0341 (cellular promotion gates explicit tier criteria); amends ADR-0248 (cellular topology); binds to
ADR-0148 (cell-orchestrator control plane), ADR-0186 (canary cohort), ADR-0044 (inter-cell mesh), ADR-0263 (observability
emission contract), ADR-0251 (compliance pack certification), ADR-0244 (tenant scoping primitive), ADR-0330 (tenant_class).
Migration sub-wave: 15T-Cell-Promotion-Gates (queued; dispatches after ADR-0341 Accepted; cell-orchestrator full µservice
implementation is a SEPARATE follow-on sub-wave under ADR-0148).

---

## API Versioning (per ADR-0342)

HYBRID model: date-based versions on the public boundary + semver on SDK packages.

**Public APIs** (OpenAPI 3.2.0 endpoints, AsyncAPI 3.1.0 channels, proto3 services exposed externally):
  Version format = YYYY-MM-DD (ISO-8601 calendar date; UTC; no time component)
  Three canonical carriers (all three mandatory on every public surface):
    1. HTTP request header   : Oyatie-Version: 2026-05-21
    2. URL prefix            : /v/<YYYY-MM-DD>/...
    3. proto3 message field  : string oyatie_version = 8001;  (reserved tag 8001)
  AsyncAPI channel carrier   : message header `oyatie-version` (kebab-case per AsyncAPI) + channel URL `/v/<YYYY-MM-DD>/`
  Supported window           : N=3 versions in parallel; ≥180 days post-deprecation before sunset
  Hyperscaler precedent      : Stripe (since 2011) · Anthropic (since 2023-06-01) · OpenAI/Azure OpenAI · AWS (since 2006-04-10) · Google Cloud · GitHub (X-GitHub-Api-Version since 2022-11-28)

**SDK packages** (10 idiomatic languages per feedback_developer_sdk_stainless_generator_2026_05_20):
  Format: MAJOR.MINOR.PATCH (semver 2.0.0 strict)
  Languages: TypeScript · Python · Go · Java · Kotlin · Swift · Rust · .NET-C# · C · C++
  Bump rule:
    MAJOR = breaking interface change (signature / type removed; pinned date dropped support)
    MINOR = additive change (new methods / types / optional params; pinned date added endpoints)
    PATCH = bug fix (no interface or type change)
  Pin discipline: each SDK semver release pins exactly one public-API date version under the hood;
                  recorded in microservices/developer-sdk/release-manifests/<sdk-package>-<semver>.json;
                  generated SDK source includes const OYATIE_API_DATE = "<YYYY-MM-DD>";
                  consumers MAY override via client option (new OyatieClient({ apiVersion: "..." }))
  Distribution channels: npm (TS) · PyPI (Python) · Go modules · Maven Central (Java + Kotlin) ·
                         Swift Package Manager + CocoaPods (Swift) · crates.io (Rust) · NuGet (.NET-C#) ·
                         vcpkg + Conan (C + C++)

Per-tenant pinning: tenant manifest declares api_version_pinning {
  default_oyatie_version          : "<YYYY-MM-DD>"
  per_microservice_overrides      : { "<ms>": "<YYYY-MM-DD>", ... }
  auto_advance_policy             : "pinned_until_sunset" (default) | "auto_advance_at_sunset"
}

Per-µservice declaration: manifest.json gains an `api_versioning` / `tenant_version_pinning` block enumerating
  declared_versions[], default_version, supported_window_size (≥3), supported_window_minimum_days (≥180),
  deprecation_calendar[] (each entry with version, deprecated_on, sunset_on, successor, breaking_change_adr, migration_doc),
  public_surface_files{} (per-version OpenAPI / AsyncAPI / proto3 file paths),
  supports_per_tenant_pinning (bool; default true).

Per-µservice contract layout:
  OpenAPI : microservices/<name>/contracts/openapi/<YYYY-MM-DD>.yaml
  AsyncAPI: microservices/<name>/contracts/asyncapi/<YYYY-MM-DD>.yaml
  proto3  : microservices/<name>/contracts/proto/<YYYY-MM-DD>/<service>.proto

api-gateway owns:
  - version routing (header + URL prefix + tenant default resolution; conflict → 400; unsupported → 410 Gone)
  - discovery endpoint GET /v/versions (OpenAPI 3.2.0; UN-versioned)
  - discovery proto3 service oyatie.versions.v1.VersionsService.ListVersions
  - RFC 8594 Sunset: + RFC 9745 Deprecation: + RFC 5988 Link: header injection on deprecated responses

developer-sdk owns:
  - Stainless-class generator emitting 10 idiomatic SDKs
  - semver bump rule enforcement
  - cosign-attested release artifacts (ADR-0181)
  - per-SDK-release date-version pinning recorded in release-manifests/

Audit-chain event classes: api.version.created · api.version.deprecated · api.version.sunset ·
                            api.version.pin_change · api.version.carrier_conflict · api.version.carrier_missing
Observability label      : oyatie_version on every audit-chain row + metric + tracing span at the public boundary;
                            cardinality bound ≤ 5 per µservice (N=3 supported + ≤ 2 in transition).
Internal mesh exemption  : ADR-0145 inter-µservice traffic does NOT carry the triplet;
                            internal proto3 evolves under tag-number backward-compatibility rules.

CI lanes:
  oya-check-public-api-date-version           — manifest block presence + ≥3 supported + ≥180-day window
  oya-check-public-api-version-triplet        — every public OpenAPI path / proto3 file / AsyncAPI channel carries the carrier
  oya-check-public-api-supported-window       — refuses drops below N=3 or windows below 180 days
  oya-check-public-api-sunset-adr             — deprecation_calendar entries require paired sunset-class ADR
  oya-check-sdk-semver-bump                   — major-on-breaking / minor-on-additive / patch-on-fix bump rule
  oya-check-sdk-language-coverage             — minor/major releases publish all 10 languages
  oya-check-tenant-version-pinning            — tenant manifest declares api_version_pinning block
  oya-governance-version-routing-canonical-carriers — refuses non-canonical carriers (?api_version=, X-API-Version, sub-domain)

Why this exists: there is no canonical public-API version carrier today; per `feedback_no_silent_regression`
silent breaking changes break tenants without notice; per `feedback_developer_sdk_stainless_generator_2026_05_20`
SDK release engineering across 10 languages needs semver. Date-on-boundary + semver-on-SDK is the canonical
hyperscaler split (Stripe / Anthropic / OpenAI / AWS / Google / GitHub).

Authority: ADR-0342 (API versioning hybrid date-public + semver-SDK); amends ADR-0145 (carrier triplet on public boundary only),
           ADR-0212 (manifest declaration), ADR-0244 (tenant pinning), ADR-0263 (audit-chain + observability emission).
Migration sub-wave: 15V-API-Versioning-Adoption (queued; dispatches after ADR-0342 Accepted + api-gateway router + developer-sdk pipeline land).

---

## Taxonomy

plugin-app-store   — curated plugin distribution channel (distinct µservice)
marketplace        — general commerce surface (distinct µservice)
community          — social/forum surface (distinct µservice)
These three are NOT synonyms. Each is a separate µservice with its own contracts.

---

## Quality Bar

Artifact threshold: 100+ artifacts per µservice (files across docs/, src/, slos/, contracts/)
Authority: ADR-0212 (Buildability Doctrine)

---

## Doctrines In-Flight (ADR-0211..ADR-0221)

ADR-0211: In-house tech stack preference (Rust-primary)
ADR-0212: Buildability doctrine — every µservice buildable end-to-end, 100+ artifacts
ADR-0215: Multi-context platform — same engine, multiple deployment contexts
ADR-0216: Open integration — standard APIs; no vendor lock-in
ADR-0217: Vertical-slice rollout — ship one slice at a time, not horizontal sprawl
ADR-0218: Tenant granular control — per-tenant feature flags + policy
ADR-0219: No-code-first UX with optional AI-assist layer
ADR-0220: Intelligence µservice scope — consumer-facing only (historical; per ADR-0335 Wave 15I, intelligence now absorbs the full AI substrate)
ADR-0221: Agentic pipeline hardening — hooks are GUIDANCE, not enforcement; CI gates enforce
ADR-0136-amendment: Legacy internal AI-runtime scope — Hermes pipeline only (historical; superseded by ADR-0335 retirement)
ADR-0255: Intelligence two-layer AI substrate (KS#14) — absorbs retired AI-runtime scope
ADR-0335: retired AI-runtime µservice retired (Wave 15I) — AI substrate absorbed into intelligence; Hermes terminology dropped corpus-wide

---

## Forbidden Primitives

See: specs/master-plan-sequencing.json#forbidden_primitives
Summary: Bash agent commands use plain `git` for ordinary git operations. The
retired local `oya` wrappers must not be used. Governance verification is the
cloud-ci/oya-ci produced `oya-ci-required` status; local
retired local verifier/retired local gate/retired local check/dev-cli wrappers are retired authority
mechanisms and must not be used as local substitutes for that required context.
OpenAPI must be 3.2.0; AsyncAPI must be 3.1.0.

---

## Common Pitfalls

1. Using retired local `oya` wrapper surfaces instead of plain git + PR + GitHub Actions cloud-ci governance gates.
2. Writing `openapi: 3.3.0` (no such released version as of 2026-05-18)
3. Writing `asyncapi: 3.0.0` (use 3.1.0)
4. Treating microservices/intelligence/ as a live µservice (ADR-0335 absorbed it into microservices/intelligence/)
5. Conflating plugin-app-store / marketplace / community
6. Creating µservices with <100 artifacts (buildability bar)
7. Bundling multiple concerns into one µservice (ADR-0132 no-grouping policy)
8. ADR references in docs without corresponding docs/decisions/ADR-NNNN-*.md files
9. Vacuous-green gates: test passes on empty input (M-08 per ADR-0221)
10. Scope creep: creating new µservices outside the current PR's declared vertical slice

---

## shared Rust gate logic Invocation Pattern

Local dev-cli invocation is retired as an authority mechanism. Prefer Buck2
targets for local confidence and the cloud-ci/oya-ci `oya-ci-required` status
for protected-branch evidence. Do not add new dev-cli hook or checklist
requirements.

---

## Wave 15-ZF Doctrine Primitives (ADR-0346..ADR-0349)

ADR-0346 legacy local-verifier wording is superseded for active work by
ADR-0515 and the current canon stores: `oya-ci-required` is the one canonical
blocking status, produced by the cloud-ci/oya-ci pipeline. Retired local oya
wrappers must not be used as protected-branch authority or revived as a local
authority mechanism.

ADR-0347: every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in one Wave 15-ZB
  bulk-rename pull request; the deterministic inventory path is .omc/state/oya-governance-rename-inventory-2026-05-21.json.
Enforced by: oya-governance-retired-vocab-residue; oya-governance-lane-prefix-vocabulary;
  oya-governance-rename-inventory-presence.

ADR-0348: cellular topology MUST support three control-plane-driven automation modes under ADR-0341 cell-level promotion gates:
  AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING. Every µservice manifest.json gains a `sharding_automation`
  block declaring per-automation-mode configuration; automation honors residency + compliance packs and emits audit-chain events.
Enforced by: oya-governance-sharding-automation-coverage; oya-governance-autosharding-manual-mode-refusal;
  oya-governance-auto-rebalance-residency-honored; oya-governance-dynamic-sharding-threshold-coverage;
  oya-governance-audit-chain-emit-on-automation-events; oya-governance-tenant-migration-reversibility.

ADR-0349 (superseded by ADR-0515): GitHub Actions is the sole CI surface. ArgoCD is the canonical GitOps CD orchestrator
  and replaces manual kubectl apply / Helm CLI deploy paths across all contexts. ArgoCD is provisioned via OpenTofu modules
  under cloud-iac/modules/<context>/argocd/. Self-hostable CI contexts use cloud-ci (oya-ci) per ADR-0515.
Enforced by: oya-governance-argocd-application-cosign-verified;
  oya-governance-argocd-tenant-namespace-isolation;
  oya-governance-deploy-audit-chain-emit.

---

## Lifecycle Skill Map

Vendored at tools/agent-skills/skills/
Source: https://github.com/addyosmani/agent-skills (MIT — Addy Osmani and contributors)

Define phase:
  interview-me                  — extract real requirements before writing code
  idea-refine                   — stress-test ideas before committing to a plan
  spec-driven-development       — write spec before writing code

Plan phase:
  planning-and-task-breakdown   — break work into ordered atomic tasks

Build phase:
  incremental-implementation    — build one step at a time with verification
  test-driven-development       — failing tests first, then implementation
  source-driven-development     — implementation grounded in source evidence
  doubt-driven-development      — challenge assumptions before proceeding
  context-engineering           — optimize agent context for quality output
  api-and-interface-design      — design contracts before implementation
  frontend-ui-engineering       — UI-specific build patterns

Verify phase:
  browser-testing-with-devtools — browser-based test execution
  debugging-and-error-recovery  — systematic root-cause diagnosis

Review phase:
  code-review-and-quality       — multi-axis review (correctness/readability/security/perf)
  code-simplification           — reduce complexity without changing behavior
  security-and-hardening        — security review with remediation
  performance-optimization      — measure first, then optimize

Ship phase:
  git-workflow-and-versioning   — branching, commits, tagging
  ci-cd-and-automation          — pipeline setup and quality gates
  deprecation-and-migration     — safe removal of old APIs/systems
  documentation-and-adrs        — ADR authoring and doc coverage
  shipping-and-launch           — final checklist before merge/release

Persona agents (tools/agent-skills/agents/):
  code-reviewer    — use for review tasks
  security-auditor — use for security tasks
  test-engineer    — use for testing tasks

Discovery rule: invoke the skill matching the task phase BEFORE producing output.
Process skills (Define/Plan) come before implementation skills (Build/Verify/Ship).
