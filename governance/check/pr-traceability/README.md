# check-pr-traceability

Pure-Rust kernel + CLI that validates a PR body against the Oyatie PR traceability contract
(`docs/templates/pull-request-template.md`): five required H2 sections in order (Issue, Summary,
Verification, Traceability, Evidence), the literal Traceability/Evidence field labels, an issue
reference, and — when `--require-code-review` is set (the CI default) — a `## Code Review`
section with reviewer, verdict, resolved items, and deferred items.

This is what `oya-ci-required`'s PR-metadata preflight runs (`.github/workflows/oya-ci-required.yml`)
against every PR's live body before the rest of the gate fleet runs.

## Author workflow

Stop discovering body-contract violations for the first time in CI. Scaffold, edit, and check
locally before opening the PR:

```
buck2 run //governance/check/pr-traceability:pr-traceability-admission-bin -- --scaffold > body.md
# ...edit body.md: fill in the issue ref, summary, verification evidence, traceability/evidence
# fields, and leave `## Code Review` / `Verdict: pending` for the reviewer to stamp...
buck2 run //governance/check/pr-traceability:pr-traceability-admission-bin -- \
  --check body.md --all-violations
```

`--scaffold` emits a template generated from the exact same label constants the validator
checks (`REQUIRED_SECTIONS`, `REQUIRED_TRACEABILITY_FIELDS`, `REQUIRED_EVIDENCE_FIELDS` in
`src/lib.rs`) — so the scaffold and the validator can never drift apart. `--check <path>` is an
alias for `--pr-body <path>`: the same file-input validation path CI uses, runnable locally, under
the same default `--require-code-review` policy CI uses. `--all-violations` prints every
violation found instead of stopping at the first, so a local author sees every defect in one pass
instead of one CI round-trip per fix.

Run against the untouched scaffold, the check above reports exactly one violation:
`MissingCodeReviewApproval`. That is expected and correct — the scaffold's `## Code Review`
section is left at `Verdict: pending` on purpose, since no review has happened yet. Once every
OTHER violation is gone, the body is ready to open as a PR; `MissingCodeReviewApproval` is the one
violation only a reviewer, not the author, can close.

## Flags

- `--scaffold` — print an admission-passing PR-body template to stdout and exit (ignores all
  other flags).
- `--pr-body <path>` / `--check <path>` — the PR body file to validate (default:
  `docs/templates/pull-request-template.md`).
- `--pr-title <title>` — the PR title (checked for blocked-review markers).
- `--require-code-review` (default, matches CI) / `--forbid-code-review` — `--forbid-code-review`
  rejects a body that has a `## Code Review` section at all, so it only applies to a body that
  intentionally omits that section (not the `--scaffold` output, which always includes it).
- `--all-violations` — report every violation instead of stopping at the first.

None of the above changes CI-invoked default behavior: with no new flags, exit semantics are
byte-identical to before.
