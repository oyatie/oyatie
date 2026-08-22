---
doc_class: Runbook
runbook_class: Foundry-Pipeline-Preflight
id: RB-SANCTIONED-PRIMITIVES-PREFLIGHT
parent: ../../AGENTS.md
status: active
control_id: MISTAKES-LEDGER-CONTROL-1
memory_ref: feedback_repeat_mistake_prevention
audit_ref: evidence/audits/pipeline-maturity-audit-2026-05-15.md
adr_ref: docs/decisions/ADR-0709-general-live-apex.md
---

# Foundry pipeline preflight

**Status:** active for build / CI regression controls; external coordination-tool checks retired 2026-05-16 by ADR-0116.
**Severity scope:** Sev-2
**Last verified:** 2026-05-16

This runbook keeps the non-retired preflight controls that prevent repeated CI-infrastructure failures. Historical rows for the retired external coordination tooling are retained as explicit tombstones so ledger references remain stable and agents do not resurrect those tools as prerequisites.

Rows #8-#10 are live mistake-prevention controls for MFL-0014, MFL-0015, and MFL-0016. They are unrelated to the retired coordination tools and MUST remain executable until the controls are relocated to a machine-readable registry.

## Canonical preflight sequence

Run the active rows in order. A failing active probe blocks the session until repaired. Retired rows are documentation-only tombstones.

| # | Primitive / control | Version probe | Smoke probe | Ledger key on failure |
|---|---|---|---|---|
| 1 | `rustc` | `rustc --version` (expects `1.95.x`) | `cargo --version && rustup show active-toolchain` | `cargo-clippy::stable-toolchain-mismatch` |
| 2 | `cargo` | `cargo --version` | `cargo metadata --no-deps --format-version 1 > /dev/null` | `cargo::metadata-failure` |
| 3 | `rustup` | `rustup --version` | `rustup show active-toolchain` matches `rust-toolchain.toml` | `rustup::toolchain-drift` |
| 4 | external lock tool surface | retired by ADR-0116 | no probe; Foundry pipeline admission owns coordination | `external-coordination::retired-lock-tool` |
| 5 | external memory-store surface | retired by ADR-0116 | no probe; durable evidence belongs in the Foundry/Oya VCS artifacts | `external-coordination::retired-memory-tool` |
| 6 | `dev-cli` | `cargo run --quiet -p dev-cli -- --help \| head -1` | `cargo run --quiet -p dev-cli -- gate run-all --include-deferred \| tail -1` | `dev-cli::gate-aggregator-missing` |
| 7 | legacy read helper | compatibility/provenance only during cutover | no prescribed preflight; do not make it a forward closure authority | `legacy-read-helper::unexpected-required-surface` |
| 8 | GitHub Actions pins | `grep -RnE 'uses:\s+[^@]+@[a-f0-9]{40}' .github/workflows/ \| head -1` returns rc 0 | every `uses: <action>@<sha>` SHA resolves (`gh api /repos/<action>/commits/<sha>`) | `gha::broken-action-sha` |
| 9 | nextest profile | `grep -q '\[profile.ci\]' .config/nextest.toml` | `cargo nextest list --profile ci --workspace > /dev/null` | `nextest::missing-profile-ci` |
| 10 | shell shebangs | `grep -RLn '^#!' scripts/ \| head -1` returns no rows | `find scripts -type f -name '*.sh' -exec sh -c 'head -1 "$0" \| grep -q "^#!"' {} \;` returns rc 0 | `bash::missing-shebang` |

## Failure protocol

When any active probe fails:

1. STOP. Do not create commits or PRs until the recurrence check is complete.
2. Search the ledger: `jq '.entries[] | select(.failure_mode == "<ledger-key>")' registry/mistakes-ledger.json`.
3. If a row exists, this is a recurrence — escalate immediately, do not patch ad hoc. Apply the linked control.
4. If no row exists, this is a first occurrence — fix in place, then append a new ledger row using `docs/templates/mistakes-ledger-row-template.md`.
5. Record evidence in the PR verification notes or the active Foundry/Oya VCS changeset artifact.
6. Re-run the active rows in this runbook before resuming work.

## Wiring

- Citation: `docs/AGENTS.md` D17 (mistakes-ledger row D17 lane).
- Fitness lane: `governance-mistakes-ledger-kernel` verifies every ledger row carries a preflight reference here.
- Verify gate: `oya gate validate mistakes-ledger` invokes the kernel as a required check.
- Backfill rows: `gha::broken-action-sha`, `nextest::missing-profile-ci`, `bash::missing-shebang` — see `registry/mistakes-ledger.json` and `docs/MISTAKES-LEDGER.md` MFL-0014..MFL-0016.

## Naming justification

Filename `preflight.md` — `<artifact:runbook>-<topic:sanctioned-primitives>-<step:preflight>`; conforms to v4 BNF for runbooks under `docs/runbooks/<topic>/<step>.md`. The directory is retained for stable ledger links; active coordination authority moved to ADR-0116 and the Foundry pipeline.
