# IP-DOCS-003 — Redoc + AsyncAPI renderer

> ADR anchor: ADR-0203.
> Owner: `oya-docs`.
> Estimate: 3 days.

## Goal

Render OpenAPI 3.2 contracts via Redoc and AsyncAPI 3.1
contracts via the AsyncAPI rendering CLI, both embedded in
the SvelteKit Tier-3 site.

## Why this IP

Every µservice ships an OpenAPI + AsyncAPI contract under
`microservices/<ms>/contracts/`. Per ADR-0203 those contracts
render into the public docs via Redoc / AsyncAPI tools.

## Tasks

### 1. Build-time render

- For each µservice's `contracts/openapi.yaml`: run Redoc
  CLI → static HTML.
- For each `contracts/asyncapi.yaml`: run AsyncAPI Generator
  → static HTML.

### 2. SvelteKit integration

- Rendered HTML embedded as iframes (or hydrated server-side)
  inside the SvelteKit site under `/api/<ms>`.

### 3. Tests

- Sanity: comms-email OpenAPI + AsyncAPI both render.
- Per-µservice contracts validate against OpenAPI 3.2 +
  AsyncAPI 3.1 schemas at CI.

## Acceptance criteria

- Every µservice's contracts surface as an API reference page
  on the public docs site.

## References

- ADR-0203.
- Redoc upstream.
- AsyncAPI Generator upstream.
