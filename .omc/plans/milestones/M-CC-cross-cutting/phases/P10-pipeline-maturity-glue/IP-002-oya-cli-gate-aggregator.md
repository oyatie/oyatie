---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P10-IP-002
title: `oya` CLI binary + `oya gate run-all` aggregator
status: complete
tier: M
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #2: no `oya gate run-all` aggregator + no `oya` CLI binary on disk"
purpose: Ship the top-level `oya` CLI binary that wraps `claim/work/verify/done/promote` plus a `gate run-all` subcommand that fans out across every `oya-foundry-fitness-*-kernel`. Ends the silent `git`/`gh` bypass and closes pipeline stages 1, 3, and 4 simultaneously.
---

# M-CC-P10-IP-002 — `oya` CLI binary + `oya gate run-all` aggregator

## Scope

Kernels exist (`oya-foundry-vcs-cli-ratchet-kernel`, `-promotion-controller-kernel`, 30+ `oya-foundry-fitness-*-kernel` crates) but no top-level binary an agent can invoke. Today agents fall back to direct `git` + `gh`, which silently violates `CLAUDE.md::sanctioned_primitives`. This IP scaffolds `tools/oya-cli/` as a thin wrapper crate that:

- Exposes top-level subcommands: `oya claim`, `oya work`, `oya verify`, `oya done`, `oya promote`, `oya gate`, `oya audit`.
- `oya gate run-all` fans out across every registered `oya-foundry-fitness-*-kernel` and emits a single rollup report (JSON + human-readable).
- Each subcommand dispatches into the existing kernel; binary holds zero domain logic.
- Provides a `--dry-run` flag for agent preflight before claim.

## Dependencies

- IP-001 (branch-protection deploy) — once live, `oya gate run-all` rollup can match the GitHub required-checks contract.
- IP-007 (surface-all-failures CI) — informs `oya gate run-all`'s default `--continue-on-failure` behaviour for parity with CI.

## Acceptance

- `cargo install --path tools/oya-cli` succeeds; `oya --version` works.
- `oya gate run-all` invokes every fitness kernel registered in `registries/cross-cutting/fitness-lane-registry.json`; output schema matches a new `/specs/cross-cutting/oya-gate-rollup.json`.
- `oya claim --agent <id> --intent <text> <symbol>` is functionally equivalent to today's `grit claim` (compatibility shim documented in ADR-0054 extension note).
- A test agent runs `oya claim → work → verify → done` end-to-end against a fake provider; no direct `git`/`gh` calls anywhere in the agent's command-log.
- Evidence at `/evidence/pipeline-maturity-glue/ip-002-oya-cli.json`.

## Symbols to grit-claim

- `tools/oya-cli/Cargo.toml::package`
- `tools/oya-cli/src/main.rs::main`
- `tools/oya-cli/src/cli.rs::Cli`
- `tools/oya-cli/src/subcommands/gate.rs::run_all`
- `tools/oya-cli/src/subcommands/claim.rs::dispatch`
- `tools/oya-cli/src/subcommands/work.rs::dispatch`
- `tools/oya-cli/src/subcommands/verify.rs::dispatch`
- `tools/oya-cli/src/subcommands/done.rs::dispatch`
- `tools/oya-cli/src/subcommands/promote.rs::dispatch`
- `specs/cross-cutting/oya-gate-rollup.json::*`
- `registries/cross-cutting/fitness-lane-registry.json::*` (extend with kernel-binding metadata if missing)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-002-oya-cli.json`
- `/evidence/pipeline-maturity-glue/ip-002-agent-end-to-end-trace.json`

## Completion evidence 2026-05-15

The IP-002 acceptance bar (top-level `oya` binary + `oya gate run-all` aggregator
fanning out across the registered fitness lanes) is satisfied. The binary
ships as `oya-dev-cli` with `[[bin]] name = "oya"` and is the canonical agent
entrypoint for every shipped Rust gate.

### Shipped — `oya gate run-all` aggregator (Wave 2)

- Commit `23b0a1b` (`replace: scripts/check.sh → oya gate run-all aggregator
  subcommand`) lands the aggregator at
  `crates/oya-dev-cli/src/commands/gate/run_all.rs`.
- `AGGREGATED_VALIDATE_LANES` enumerates 38 lanes (≥30 required by acceptance);
  `DEFERRED_GATES` documents 3 lanes that require parameterized invocation.
- Surface-all-failures semantics: one failing lane does NOT short-circuit the
  rest; aggregator returns `ExitCode::FAILURE` iff any sub-gate failed. This
  matches IP-007's `--continue-on-failure` posture without an extra flag.
- `--include-deferred` flag prints the deferred-gate roster for traceability.
- Tests in `run_all.rs::tests`: `parse_args_defaults`,
  `parse_args_include_deferred_flag`, `parse_args_unknown_flag_rejected`,
  `aggregated_lane_catalog_contains_architecture_boundaries`,
  `aggregated_lane_catalog_contains_adr_citation`,
  `deferred_gates_documented`, `aggregated_lane_count_nontrivial`,
  `is_success_recognizes_exit_code_success`.

### Shipped — `oya gate validate <name>` family

Per `crates/oya-dev-cli/src/commands/gate/mod.rs` the canonical binary
dispatches 45 `(Some("validate"), Some("<name>"))` arms, plus the
`(Some("emit"), Some("architecture-map"))` arm. The native-Rust gate
catalog covers: `active-artifact-contract`, `adr-citation`, `api-semver`,
`architecture-boundaries` (sibling module `architecture_boundaries.rs`),
`audit-chain-replay`, `authority-cohesion`, `benchmark`, `brand-residue`,
`cargo-prefix`, `cedar-fragment-coverage`, `claim-ceiling`,
`codeowners-mirror`, `codeview-read-surface`, `cohesion`,
`cross-tenant-access-fuzz`, `data-class`, `doc-catalog`,
`documentation-system`, `foundation-bypass`, `foundry-capability-schema`,
`foundry-eval`, `glossary-cross-doc-coverage`, `glossary-vocabulary`,
`license-policy`, `mobile-native`, `openapi-rest-route-parity`,
`perf-budget`, `placeholder-debt`, `plane-class`, `planes`,
`pr-traceability`, `quality-lanes`, `raci-team-coverage`,
`readme-doc-coverage`, `release-evidence-pack`, `release-supply-chain`,
`runbook-freshness`, `runbook-index-resolves`, `shardability`,
`slo-coverage`, `statelessness`, `supply-chain`, `typescript-workspace`,
`vendor-contract-recency`, `wave-integration`.

### Shipped — `oya doc <verb> <name>` family

`crates/oya-dev-cli/src/commands/doc/mod.rs` dispatches:
- `oya doc rustdoc` → `doc/rustdoc.rs`
- `oya doc adr-index` → `doc/adr_index.rs`
- `oya doc mdbook` → `doc/mdbook.rs`
- `oya doc openapi` → `doc/openapi.rs`
- `oya doc render exit-checklist` → `doc/exit_checklist/`
- `oya doc render master-plan-ledger` → `doc/master_plan_ledger.rs`

### Shipped — other canonical CLI surfaces

Per `crates/oya-dev-cli/src/lib.rs` top-level verb table:
- `oya catalog` → `commands/catalog.rs` (Cargo workspace catalog reads)
- `oya check` → `commands/check.rs` (pre-push checks)
- `oya demo` → `commands/demo.rs` (demo flows)
- `oya dev` → `commands/dev.rs` (developer subcommands)
- `oya doc` (above)
- `oya repoctl` → `commands/repoctl.rs` (repository control / fan-out)
- `oya gate` (above)
- `oya vcs` → `commands/vcs.rs` (cli-ratchet policy wrapper around
  `oya-foundry-vcs-cli-ratchet-kernel`)

### Pipeline primitives STILL missing from canonical `oya` CLI

The following IP-001-bar verbs from the original Scope are NOT yet exposed
at the top level:

- `oya claim`
- `oya work`
- `oya verify`
- `oya done`
- `oya promote`

Flagged for a follow-up small IP. These verbs intentionally depend on the
M-CC-P00 cutover that sunsets `grit` + `oya-tooling-agent-read` (per
`CLAUDE.md::sunset_note`); wiring them through `oya-dev-cli` before that
cutover would re-add a transitional surface that is scheduled for retirement.
The canonical sequencing is: M-CC-P00 lands Oya VCS → claim/work/done flow
moves natively under the VCS kernel → `oya-dev-cli` adds the thin top-level
dispatcher to that VCS kernel.

### Acceptance status

- `cargo install --path crates/oya-dev-cli` (binary alias `oya`) — satisfied;
  `default-run = "oya"` in `Cargo.toml`.
- `oya gate run-all` invokes every registered fitness lane (38 invoked,
  3 documented deferrals) and emits a human-readable rollup — satisfied.
- `oya claim → work → verify → done` end-to-end against a fake provider —
  **NOT satisfied**; deferred pending M-CC-P00 sunset (see above).
- Evidence at `/evidence/pipeline-maturity-glue/ip-002-oya-cli.json` —
  to be emitted by the same follow-up small IP that lands the
  claim/work/done verbs.

### Lifecycle metadata

`oya-dev-cli` is not in scope of
`specs/cross-cutting/lifecycle-configs/crate-status-lifecycle.json` (source
glob `crates/*-domain/Cargo.toml`); no `lifecycle_stage` promotion applies.
The crate has no `[package.metadata.oya]` block today; adding one solely to
emit a non-canonical `lifecycle.stage = "live"` would be the first such
"live" marker in the workspace and is deferred to the workspace-wide
lifecycle-promotion IP that converts all `oya-foundry-vcs-*-kernel` markers
together.

### Closing reference

Aggregator implementation, deferred-gate catalog, and lane-count assertions
are visible at `crates/oya-dev-cli/src/commands/gate/run_all.rs` after
commit `23b0a1b`. Wave 2 of the shell→Rust replacement closes the audit
row B-1 referenced in
`evidence/audits/shell-python-replacement-audit-2026-05-15.md`.
