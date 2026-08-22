# observability-axe-pa11y-runner

WCAG 2.2 AA accessibility CI runner per ADR-0207.

## What it does

Runs **axe-core + pa11y** as a Kubernetes Job against the preview deployment URLs from a PR.
Emits an axe-core JSON report; hashes it into the audit-chain ledger (per ADR-0145); fails
the PR CI lane on any AA criterion violation.

## Per-stack modes

- `web` — axe-core/playwright + pa11y (in-cluster).
- `gtk4` — AT-SPI conformance test (in-cluster Linux).
- `swiftui`, `compose`, `winui3` — out-of-cluster runners; this Job emits the spec the
  external runner consumes.

## WCAG target

- `aa` — production minimum (ADR-0207).
- `aaa` — regulated packs (HIPAA / EU AI Act / government).

## Evidence

Per ADR-0145 audit-chain seal: every run emits a JSON report; the report SHA-256 lands in
the audit chain ledger at `evidence/a11y/pr-<n>.json`.

## Cross-references

- ADR-0207 — a11y bar (WCAG 2.2 AA).
- `docs/standards/a11y-canonical.md` — canonical a11y rules.
- `docs/standards/wcag-2-2-aa-checklist.md` — per-criterion mapping.
