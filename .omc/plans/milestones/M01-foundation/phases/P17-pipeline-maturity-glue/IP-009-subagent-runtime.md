---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-009
title: Subagent runtime (real per-facet Claude API invocation + fix-agent invocation)
status: scaffolded
tier: M
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_amendment_ref: "Cross-cutting follow-up: every IP-004/IP-005/IP-006 emission currently carries subagent_runtime_pending=true because the per-facet subagent invocation isn't wired into a Rust binary."
upstream_kernel: oya-intelligence-subagent-runtime-kernel
purpose: Close the `subagent_runtime_pending=true` gap left by IP-004 / IP-005 / IP-006 by shipping a real per-facet subagent runtime that loads per-facet prompt templates, invokes the Claude API via the existing `oya-foundry-adapter-anthropic-api-*` substrate against an OpenBao-resolved `SecretReference`, parses the per-facet response into a `FacetFinding`, writes it to `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/<facet>.json`, and lets the IP-006 admission gate refuse APPROVE events that still carry the pending flag.
---

# M01-P17-IP-009 — Subagent runtime (per-facet Claude API invocation)

## Acceptance Criteria

- **AC-001**: 21 per-facet prompt templates exist under `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/<facet-id>.md` (F1..F11 + F13 + M1+M2 + A1..A7), each with required frontmatter fields (`facet_id`, `facet_name`, `lens`, `severity_bar`).
  - test_id: `find evidence/pipeline-maturity-glue/ip-004-pr-review/facets -name "*.md" | wc -l` returns `>= 21`
  - verification_command: `find evidence/pipeline-maturity-glue/ip-004-pr-review/facets -name "*.md" | wc -l`
  - status: pending-spec-author
- **AC-002**: `cargo build -p oya-intelligence-subagent-runtime-kernel` and `cargo test -p oya-intelligence-subagent-runtime-kernel` pass with zero errors.
  - test_id: `cargo nextest run -p oya-intelligence-subagent-runtime-kernel --all-features`
  - verification_command: `cargo nextest run -p oya-intelligence-subagent-runtime-kernel --all-features`
- **AC-003**: In deterministic-mock mode, a test PR invocation produces 21 per-facet JSON findings under `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/` and dispatcher rollup carries `subagent_runtime_pending=false`.
  - test_id: `cargo nextest run -p oya-intelligence-subagent-runtime-kernel mock_port_produces_facet_finding`
  - verification_command: `cargo nextest run -p oya-intelligence-subagent-runtime-kernel --all-features`
  - status: pending-spec-author
- **AC-004**: IP-006 admission gate refuses any admission-log event whose rollup carries `subagent_runtime_pending=true`.
  - test_id: `cargo nextest run -p oya-vcs-admission-gate-kernel rejects_pending_subagent_runtime`
  - verification_command: `cargo nextest run -p oya-vcs-admission-gate-kernel --all-features`
  - status: pending-spec-author
- **AC-005**: Raw Anthropic API key never appears in repo, chat, or checkpoint (OpenBao `SecretReference` path only).
  - test_id: `oya gate validate supply-chain` + `git log --all -p | grep -i "ANTHROPIC_API_KEY"` returns empty
  - verification_command: `cargo run -p oya-dev-cli -- gate validate supply-chain`
- **AC-006**: `cargo clippy --workspace --all-targets` exits 0 after this IP merges.
  - test_id: `cargo clippy --workspace --all-targets -- -D warnings`
  - verification_command: `cargo clippy --workspace --all-targets -- -D warnings`

## Scope

IP-004 (reviewer-agent auto-dispatch), IP-005 (CI-failure fix-loop), and IP-006 (merge-queue fix-loop integration) all currently emit a `subagent_runtime_pending=true` marker because the per-facet subagent invocation has not been wired into a Rust binary. Per the autonomous-decision-principles charter ("no stubs / placeholders / `unimplemented!()` within scope"), this IP closes that gap with a real runtime.

The runtime spans two crates per `feedback_clean_architecture_requirements` (12-layer enum + inward-only flow + port-in-kernel):

- **`crates/oya-intelligence-subagent-runtime-kernel`** — I/O-free port-in-kernel:
  - `FacetPromptTemplate` value type (frontmatter + body).
  - `SubagentRequest` (facet id + reviewer id + change id + system prompt + user message + secret reference + model id).
  - `SubagentResponse` (final recommendation enum + free-text findings).
  - `FacetFindingJson` deterministic serializer (no serde) that produces the exact `evidence/debate/<change_id>-<facet_id>-r1.json` shape consumed by IP-004's `parse_recommendation` + the lane validator.
  - `SubagentPort` trait (one method: `complete(request) -> Result<response, error>`); the kernel never calls Anthropic itself.
  - `MockSubagentPort` (deterministic-test path; same Anthropic-API contract; canonical mock infrastructure per the "test path is canonical mock infrastructure" hard-stop in the IP brief).

- **`tools/oya-intelligence-subagent-runtime-app`** — Binary surface:
  - `SubagentPort` implementations: `AnthropicSubagentPort` (live HTTP via the existing `oya-intelligence-adapter-anthropic-api-adapter` substrate) + `MockSubagentPort` re-export for deterministic CI/test mode.
  - CLI that fans out the 21-facet panel for a PR, loads per-facet templates from `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/<facet-id>.md`, builds one `SubagentRequest` per facet, invokes the port, writes per-facet evidence JSON.
  - `--mode anthropic-api | deterministic-mock` flag — production = `anthropic-api`, test/CI = `deterministic-mock`.
  - OpenBao `SecretReference` fetch is delegated to `oya-secrets-file-adapter` (the canonical local-OpenBao path per the SecretReference memory directive).

## Wiring

- **IP-004** — `tools/oya-intelligence-pr-review-dispatcher-app` now drives the runtime: on workflow_run completion, the GitHub workflow invokes `oya-intelligence-subagent-runtime-app fan-out --pr <N> --change-id <ID>` BEFORE invoking the dispatcher. The dispatcher then reads the per-facet `<facet>.json` files that the runtime wrote, runs `rollup_verdict`, and emits a real Verdict with `subagent_runtime_pending=false`.
- **IP-005** — `tools/oya-vcs-ci-fix-loop-dispatcher-app` posts the bundle to the agent dispatch queue AND invokes `oya-intelligence-subagent-runtime-app fix-loop --bundle <PATH>` which loads the bundle, asks the model for a patch, and emits the fix-agent's response into `evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/<attempt>-agent-response.json`. The fix is claimed via `oya verify` (the canonical pre-merge gate) before push.
- **IP-006** — `tools/oya-vcs-merge-queue-fix-loop-app` adds an admission-gate assertion: when an `pr-review-approved` event arrives, the integration crate reads the per-PR `rollup.json` and REFUSES to admit if `subagent_runtime_pending=true`. Convergence guarantee: a PR never merges without real subagent findings.

## Dependencies

- IP-004 (reviewer-agent dispatcher) — wires the runtime fan-out before dispatcher rollup.
- IP-005 (CI-failure fix-loop) — wires the runtime fix-agent invocation.
- IP-006 (merge-queue integration) — adds the admission-gate refusal.
- `oya-intelligence-adapter-anthropic-api-kernel` / `-adapter` — existing canonical Claude API substrate; this IP EXTENDS those (no parallel client).
- `oya-intelligence-account-kernel::SecretReference` — opaque reference; secrets domain (`oya-secrets-domain` + `oya-secrets-file-adapter`) resolves it to OpenBao-backed material at runtime.

## Acceptance

- 21 per-facet prompt templates exist under `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/<facet-id>.md` (F1..F11 + F13 + M1+M2 + A1..A7), each carrying frontmatter (`facet_id` + `facet_name` + `lens` + `severity_bar`) + body (the actual prompt).
- A test PR opens; the GitHub workflow invokes the runtime in deterministic-mock mode; 21 per-facet JSON findings appear under `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/`; the dispatcher rollup emits `subagent_runtime_pending=false` with a real verdict.
- Production mode hits the real Anthropic API using the OpenBao `SecretReference`; raw API key never appears in repo / chat / checkpoint.
- IP-006 admission gate REFUSES any admission-log event whose corresponding rollup carries `subagent_runtime_pending=true`.
- Cargo lanes pass: `cargo build -p oya-intelligence-subagent-runtime-kernel -p oya-intelligence-subagent-runtime-app`, `cargo test` on both, `cargo clippy --workspace --all-targets`.
- Naming-justification doc-comment present on both new crates.

## Symbols to grit-claim

- `crates/oya-intelligence-subagent-runtime-kernel/Cargo.toml::package`
- `crates/oya-intelligence-subagent-runtime-kernel/src/lib.rs::*` (`FacetPromptTemplate`, `SubagentRequest`, `SubagentResponse`, `FacetRecommendation`, `SubagentPort`, `MockSubagentPort`, `FacetFindingJson::render`)
- `tools/oya-intelligence-subagent-runtime-app/Cargo.toml::package`
- `tools/oya-intelligence-subagent-runtime-app/src/main.rs::main`
- `tools/oya-intelligence-subagent-runtime-app/src/anthropic.rs::AnthropicSubagentPort`
- `tools/oya-intelligence-pr-review-dispatcher-app/src/main.rs::fan_out_facets_then_load`
- `tools/oya-vcs-ci-fix-loop-dispatcher-app/src/main.rs::dispatch_bundle` (invoke fix-agent + drop pending marker)
- `tools/oya-vcs-merge-queue-fix-loop-app/src/main.rs::assert_runtime_complete` (reject pending APPROVEs)
- `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/F1_linus.md` (and 20 sibling files)

## Hard-stops

- Anthropic API substrate cannot accept per-facet message-completion shape → STOP, NEEDS_CONTEXT (extend the existing kernel/adapter pair).
- API key cannot be resolved from OpenBao via `SecretReference` → STOP, NEEDS_CONTEXT (need OpenBao integration first).
- Network egress denied in CI / test → the deterministic-mock port is canonical mock infrastructure (not a stub); production hits real Anthropic.

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-009-subagent-runtime.json` — rollup
- `/evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/<facet>.json` — 21 per-facet findings for the test PR (deterministic-mock mode).

## Naming justification

- Crate `oya-intelligence-subagent-runtime-kernel` — BNF v4.1: `oya` (root) - `foundry` (slot-2 µservice, registered) - `subagent-runtime` (bc-tokens; kebab tokens; bounded context = per-facet subagent invocation substrate) - `kernel` (layer ∈ 13-value enum {kernel, domain, usecase, app, adapter, infrastructure, cli, rest, grpc, graphql, worker, sdk, api} ✓). Sibling pattern: every other `oya-foundry-*-kernel` crate.
- Crate `tools/oya-intelligence-subagent-runtime-app` — same root/µservice/bc-tokens; layer = `app` (binary that orchestrates the kernel per ADR-0107 §"tools/ binding"; binary tools SHALL use `-app`).
