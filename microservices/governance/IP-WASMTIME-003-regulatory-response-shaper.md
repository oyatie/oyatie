# IP-WASMTIME-003 — Regulatory response-shape filter

> ADR anchor: ADR-0200, ADR-0144 (EU AI Act Annex III).
> Owner: `oya-governance`.
> Estimate: 3 days.

## Goal

Implement an Envoy WASM filter that injects regulatory-refusal
or disclosure text into LLM responses based on jurisdiction-
specific rules (e.g. EU AI Act Annex III refusal text).

## Why this IP

ADR-0144 mandates jurisdiction-specific response shaping for
high-risk AI Act categories. Doing this at the gateway level
(via a WASM filter) keeps individual µservices unaware of the
regulatory variance.

## Tasks

### 1. Rule set

- Per-pack rule declarations (EU pack ships AI Act Annex III
  rules; others may add jurisdiction-specific rules).

### 2. Filter logic

- On response: detect LLM-output shape; if regulated category
  matches, inject canonical refusal / disclosure text per
  pack rule.

### 3. Tests

- EU pack: regulated request returns the Annex III refusal
  text.
- non-EU pack: same request passes unmodified.

## Failure modes

- Rule mismatch (false-positive injection): pack-overlay can
  disable the rule.

## Acceptance criteria

- AI Act Annex III scenarios return the canonical refusal
  text under the EU pack.
- No injection for non-regulated categories.

## References

- ADR-0144 EU AI Act Annex III refusal.
- ADR-0200, ADR-0182.
