---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P10-IP-002
title: `oya` CLI binary + `oya gate run-all` aggregator
status: scaffolded
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
