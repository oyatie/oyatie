# SVC-ADR-005 — MJML + Liquid canonical templating

- Status: Accepted
- Date: 2026-05-18
- Scope: `comms-email` µservice only
- ADR anchors: ADR-0201, IP-006, IP-007, ADR-0064

## Context

The substrate must pick one templating stack for transactional
mail. Multiple options exist (raw HTML, MJML, Maizzle,
Handlebars, Liquid, Mustache, Jinja).

## Decision

- **MJML** (open-source MIT) for responsive-HTML compile.
  Compiled via `mrml` Rust crate.
- **Liquid** (open-source dual MIT/Apache-2.0) for variable
  substitution. Via the `liquid` Rust crate.
- **MJML compiles before Liquid substitutes** — so MJML
  reasoning is purely about layout, Liquid is purely about
  per-send variables.
- **Per-locale templates** per ADR-0064 pack overlay
  structure.

## Alternatives considered

- Raw HTML: rejected — fails across the long-tail client
  matrix.
- Handlebars: rejected — weaker sandbox than Liquid;
  `{{>partial}}` invites composition that breaks the variable
  contract.
- Mustache: rejected — Liquid is a strict superset and we
  use the extra features.
- Jinja: rejected — Python-rooted, no first-class Rust runtime.

## Consequences

- Templates ship as MJML source under
  `microservices/comms-email/templates/`.
- Liquid variable contracts enforced at preflight (IP-007 §2).
- Template CI lint catches missing variables before merge.

## Open

- Visual editor for MJML — Phase 2 (likely Workflow Studio
  integration).
