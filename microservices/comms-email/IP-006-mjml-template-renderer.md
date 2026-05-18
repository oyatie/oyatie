# IP-006 — MJML template renderer

> ADR anchor: ADR-0201, ADR-0064.
> Owner: `oya-substrate-comms`.
> Estimate: 3 days.

## Goal

Compile MJML templates to cross-client responsive HTML before
the Liquid substitution step (IP-007). Templates live in the
canonical-base directory + per-pack overlays per ADR-0064.

## Why this IP

Hand-authored email HTML is brittle across the long tail of
clients (Outlook, Gmail, Apple Mail, AOL, on and on). MJML is
the de-facto standard responsive-email markup; its compiled
output is the most reliable HTML across the client matrix.

## Pre-conditions

- `crates/oya-shared-email-comms-kernel` lands.
- ADR-0064 pack overlay structure exists.

## Tasks

### 1. Wire the renderer

- Depend on the `mrml` Rust crate (workspace-pinned; floor 5.x
  as of 2026-05-18). `mrml` is the canonical MJML implementation
  in Rust, MIT-licensed.

### 2. Template directory layout

- Per ADR-0064:
  - `microservices/comms-email/templates/canonical-base/{template_id}/{locale}.mjml`
  - `microservices/comms-email/templates/packs/{pack}/{template_id}/{locale}.mjml`
- Resolution order: tenant override → pack-locale → pack-default
  → canonical-base → canonical-base/en-US (final fallback).

### 3. Template registry

- A static registry maps `template_id` → metadata
  (description, mandatory variables, optional variables,
  CAN-SPAM unsubscribe-footer requirement).

### 4. Compile pipeline

- On send: resolve template + locale → load MJML source →
  `mrml::parse` + `mrml::render` → resulting HTML is the
  `html_body` of the `OutboundMessage`.

### 5. Cache

- Compiled HTML is cached for 5 minutes per
  `(template_id, pack, locale)` to avoid re-compiling MJML on
  every send. Cache lives in-process.

### 6. Plain-text alternative

- For each compiled HTML, also generate a plain-text fallback
  using a deterministic HTML-to-text shim
  (`html2text` crate, workspace-pinned).

### 7. Mandatory footer

- CAN-SPAM compliance: the renderer always appends a
  pack-specific unsubscribe footer with the tenant's
  unsubscribe URL. Missing footer = preflight rejection.

### 8. Tests

- Unit test: compile each canonical-base template for each
  locale; assert no `mrml::Error`.
- Integration test: render the verification-email template
  with pack=kr, locale=ko-KR, and assert the output contains
  the Korean translation strings.
- Tests for the resolution-order fallback chain.

## Failure modes

- MJML source rejected by `mrml`: emit a CI-time lint failure
  so bad templates never reach production.
- Template missing for resolved locale: fallback chain
  guarantees a render; the missing-locale event is logged
  for follow-up translation work.

## Acceptance criteria

- 100% of canonical-base templates render cleanly under
  `mrml`.
- The verification-email template renders the recipient's
  preferred locale ≥ 99% of the time across the supported
  pack matrix.
- CAN-SPAM unsubscribe footer is present on 100% of sends.

## Rollback

If MJML rendering regresses, parent flips a feature flag to
pass through hand-authored HTML for the affected
template_id only. The flag is per-template so only the
regressing template degrades.

## References

- ADR-0201.
- ADR-0064 packs.
- MJML spec.
- `mrml` upstream documentation.
- IP-007 Liquid substitution engine.
