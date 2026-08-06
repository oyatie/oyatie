---
id: ADR-0517
title: "One owned AST substrate (tree-sitter-our-way, rowan-style, content-addressed) read by every consumer; one work-area hash = SCM id + buck2/RBE cache key + CD artifact hash"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0516]
amends: []
related: [ADR-0516, ADR-0518, ADR-0520, ADR-0521, ADR-0530]
related_specs:
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0517: One owned AST substrate read by every consumer

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Decomposes ADR-0516 Component-wide. The content-address contract is one of the W1 interfaces
(ADR-0520) with infinite-scale baked in.

## Context

The Agentic Delivery Fabric (ADR-0516) needs a single source of machine-legible structural truth that
every consumer reads — work-area identity (locking), practice enforcement, auto-remediation,
doc-tracking, and the work-area-granular CI affected-set. Reusing tree-sitter fails on two axes: it
violates the minimal-deps doctrine, and it cannot natively give content-addressed node identity (the
node-identity crux that work-area locking requires).

## Decision

OWN the parser. Build a single bespoke AST substrate in safe Rust — **"tree-sitter-our-way,"
rowan-style, with content-addressed node identity** — NOT a reuse of tree-sitter. ONE substrate is
read by every consumer: work-area identity (locking), practice enforcement (the GATE = AST queries for
hyperscaler / cloud-native patterns and anti-patterns), auto-remediation (AST rewrites), doc-tracking,
and the work-area-granular CI affected-set.

Correspondingly, ONE content-addressed **work-area hash** is simultaneously the SCM change id, the
buck2 affected-target + RBE cache key, and the CD artifact hash — hermetic, cacheable, and attestable
end-to-end. This single hash dissolves the work-area-identity crux.

## Drivers

- Minimal-deps doctrine plus the node-identity crux (tree-sitter cannot give content-addressed
  identity natively).
- End-to-end attestable hermeticity: one hash that ties SCM id, build cache key, and artifact hash.
- Agentic-dev-primary: agents need machine-legible structural truth to author and remediate at scale.

## Alternatives considered

- **Reuse tree-sitter as the substrate** — rejected: fails the minimal-deps doctrine and the
  content-addressed identity crux.
- **Separate per-consumer parsers** — rejected: drift across consumers, no shared identity, no single
  hash.

## Consequences

W1 LOCKS the interface (`WorkAreaTree`, ADR-0520). W2 builds the owned Rust + Markdown parser behind
it, plus AST practice / anti-pattern gates with behavior-preserving auto-fix (ADR-0530), AST
doc-tracking, the work-area affected-set, and the auto-remediation bot fleet (ADR-0531). The
content-address contract is one of the W1 interfaces with infinite-scale baked in. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: settled-vision spec (PASSED). Decomposes
ADR-0516.*
