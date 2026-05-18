# IP-007 — Liquid substitution engine

> ADR anchor: ADR-0201, ADR-0064.
> Owner: `oya-substrate-comms`.
> Estimate: 2 days.

## Goal

Substitute variables into compiled MJML output using the Liquid
templating language. Liquid is sandboxed by design (no arbitrary
code execution) and has multi-year track record in
transactional-email systems.

## Why this IP

The MJML compile step (IP-006) produces locale-shaped HTML.
Per-send variables (recipient name, magic-link URL, expiry
timestamp, tenant-specific branding tokens) still need to land.
Handlebars is the obvious alternative but lacks Liquid's
deterministic sandbox guarantees.

## Pre-conditions

- IP-006 MJML template renderer lands.
- `crates/oya-shared-email-comms-kernel` lands.

## Tasks

### 1. Wire the engine

- Depend on the `liquid` Rust crate (workspace-pinned; floor
  0.26.x as of 2026-05-18). MIT-licensed.

### 2. Variable contract

- Each `template_id` declares a typed variable contract in the
  template registry (IP-006). Variables are strongly typed:
  `string`, `int`, `decimal`, `bool`, `iso8601-datetime`, `url`.
- On substitute: validate each supplied variable against the
  declared contract; missing required variables = preflight
  rejection.

### 3. Substitution pipeline

- Input: compiled HTML (from IP-006) + variable bag.
- Steps:
  1. Parse HTML through Liquid.
  2. Apply standard Liquid filters
     (`escape`, `upcase`, `downcase`, `date`, `default`).
  3. Apply oyatie custom filters:
     - `oya_format_currency` (locale-aware money formatting)
     - `oya_format_datetime` (locale-aware datetime)
     - `oya_translate` (translation key lookup)
     - `oya_unsubscribe_url` (canonical unsubscribe URL).
- Output: substituted HTML ready for SMTP transmission.

### 4. Locale-aware filters

- Pack-aware (ADR-0064): `oya_translate` resolves through the
  same fallback chain as MJML templates (tenant override → pack
  → canonical-base).

### 5. Plain-text path

- The plain-text alternative built in IP-006 is also Liquid-
  substituted with the same variable bag and same custom
  filters.

### 6. Forbidden constructs

- Liquid `{% include %}` is **disabled** to prevent template
  composition that breaks the registry's variable contract.
- Liquid `{% capture %}` is **disabled** for the same reason.
- Liquid's standard `{% if %}` / `{% for %}` are allowed.

### 7. Tests

- Unit tests: each declared template variable substitutes
  correctly.
- Unit tests: missing required variable rejects at preflight.
- Unit tests: forbidden Liquid construct rejects at compile.
- Integration test against the verification-email template
  end-to-end (MJML compile → Liquid sub → plain-text alt →
  DKIM sign → SMTP).

## Failure modes

- Variable name typo in template: caught at CI by a template
  lint that runs every template through Liquid parse and
  asserts the parser-extracted variable set equals the
  declared contract.
- Type-mismatch (e.g. `int` supplied for an `iso8601-datetime`):
  preflight rejection with explicit error.

## Acceptance criteria

- 100% of canonical-base templates have a declared variable
  contract enforced at preflight.
- Variable name typos are caught by CI lint, not production.
- Forbidden Liquid constructs reject at compile.

## Rollback

If Liquid substitution regresses, parent flips a feature flag
that swaps Liquid for a minimal placeholder substitution
(`{{var}}` literal replacement) for the affected template_id
only. The flag is per-template.

## References

- ADR-0201.
- ADR-0064 packs.
- Liquid templating spec.
- `liquid` upstream documentation.
- IP-006 MJML template renderer.
