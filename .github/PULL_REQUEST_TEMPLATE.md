<!--
Canonical authority: docs/templates/pull-request-template.md. Keep it short: CI logs and the
review thread are the evidence. No other sections are required or checked.
-->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary

- 1-3 bullets on **what + why**. The diff already shows the *what*; this section adds the *why*.
- Cite the canonical authority read first per `docs/AGENTS.md §Pre-flight checklist` item 2.

## Verification

- `cargo fmt --all --check` / `cargo clippy` / `cargo test` (or targeted equivalents) — `<PASS|FAIL>` — `<excerpt>`
- `oya-ci-required` PR context — `<PASS|PENDING>` — `<check URL>`

## Code Review

- Reviewer: `<agent>` — verdict `<APPROVE|REQUEST CHANGES|PENDING>`
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
