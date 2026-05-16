---
doc_class: Runbook
runbook_class: Sanctioned-Primitive-Preflight
id: RB-SANCTIONED-PRIMITIVES-PREFLIGHT
parent: ../../AGENTS.md
status: active
control_id: MISTAKES-LEDGER-CONTROL-1
memory_ref: feedback_repeat_mistake_prevention
audit_ref: evidence/audits/pipeline-maturity-audit-2026-05-15.md
adr_ref: docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md
---

# Sanctioned-primitives preflight

**Status:** active
**Severity scope:** Sev-2
**Last verified:** 2026-05-16

Per [`feedback_repeat_mistake_prevention.md`](../../../../.claude/projects/-Users-jasonlee-oyatie/memory/feedback_repeat_mistake_prevention.md) Control 1: every agent MUST run this preflight BEFORE the first `grit claim`, `icm store`, `cargo`, or `oya-tooling-agent-read` invocation in a session. Each row is a `--version` probe plus a smoke check that exits non-zero on first failure. The preflight is the single authority for "is this primitive callable today?".

Today's session paid 3 commit cycles to debug 3 CI-infrastructure regressions (broken action SHA, missing nextest profile, missing shebang) — exactly the repeat-class signature this runbook exists to prevent. Each backfilled row is recorded in `registry/mistakes-ledger.json` and indexed below.

## Canonical preflight sequence

Run these in order. The first failing probe blocks the session until repaired.

| # | Primitive | Version probe | Smoke probe | Ledger key on failure |
|---|---|---|---|---|
| 1 | `rustc` | `rustc --version` (expects `1.95.x`) | `cargo --version && rustup show active-toolchain` | `cargo-clippy::stable-toolchain-mismatch` |
| 2 | `cargo` | `cargo --version` | `cargo metadata --no-deps --format-version 1 > /dev/null` | `cargo::metadata-failure` |
| 3 | `rustup` | `rustup --version` | `rustup show active-toolchain` matches `rust-toolchain.toml` | `rustup::toolchain-drift` |
| 4 | `grit` | `grit --version` (expects ≥ v0.3.0) | `ls .grit/registry.db && [ $(stat -f%z .grit/registry.db) -lt 1073741824 ]` (1 GB ceiling) | `grit::registry-bloat` |
| 5 | `icm` | `icm --version` | `icm list --limit 1` returns rc 0 | `icm::store-unreachable` |
| 6 | `oya-dev-cli` | `cargo run --quiet -p oya-dev-cli -- --help \| head -1` | `cargo run --quiet -p oya-dev-cli -- gate run-all --include-deferred \| tail -1` | `oya-dev-cli::gate-aggregator-missing` |
| 7 | `oya-tooling-agent-read` | `cargo run --quiet -p oya-tooling-agent-read -- --version` | `cargo run --quiet -p oya-tooling-agent-read -- diff --base HEAD --head HEAD --paths .` | `oya-tooling-agent-read::diff-failure` |
| 8 | GitHub Actions pins | `grep -RnE 'uses:\s+[^@]+@[a-f0-9]{40}' .github/workflows/ \| head -1` returns rc 0 | every `uses: <action>@<sha>` SHA resolves (`gh api /repos/<action>/commits/<sha>`) | `gha::broken-action-sha` |
| 9 | nextest profile | `grep -q '\[profile.ci\]' .config/nextest.toml` | `cargo nextest list --profile ci --workspace > /dev/null` | `nextest::missing-profile-ci` |
| 10 | shell shebangs | `grep -RLn '^#!' scripts/ \| head -1` returns no rows | `find scripts -type f -name '*.sh' -exec sh -c 'head -1 "$0" \| grep -q "^#!"' {} \;` returns rc 0 | `bash::missing-shebang` |

## Failure protocol

When any probe fails:

1. STOP. Do NOT issue `grit claim`, `git commit`, or `gh pr create`.
2. Search the ledger: `jq '.entries[] | select(.failure_mode == "<ledger-key>")' registry/mistakes-ledger.json`.
3. If a row exists, this is a recurrence — escalate immediately, do not patch ad-hoc. Apply the linked control.
4. If no row exists, this is a first occurrence — fix in place, then APPEND a new ledger row using `docs/templates/mistakes-ledger-row-template.md`.
5. ICM record: `icm store -t mistakes-prevention -c "<symptom>"` keyed by `error-class,<primitive>,<symptom>`.
6. Re-run this runbook from #1 before resuming work.

## Wiring

- Citation: `docs/AGENTS.md` D17 (mistakes-ledger row D17 lane).
- Fitness lane: `oya-foundry-fitness-mistakes-ledger-kernel` verifies every ledger row carries a preflight reference here.
- Verify gate: `oya gate validate mistakes-ledger` invokes the kernel as a required check.
- Backfill rows (today's session): `gha::broken-action-sha`, `nextest::missing-profile-ci`, `bash::missing-shebang` — see `registry/mistakes-ledger.json`.

## Naming justification

Filename `preflight.md` — `<artifact:runbook>-<topic:sanctioned-primitives>-<step:preflight>`; conforms to v4 BNF for runbooks under `docs/runbooks/<topic>/<step>.md`. The directory `sanctioned-primitives/` matches the CLAUDE.md fenced `sanctioned_primitives:` list (the BNF topic token).
