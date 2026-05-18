---
doc_class: ImplementationPlan
impl_plan_id: IP-005-backlink-graph-kernel-domain
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: backlink-graph kernel + domain (wikilink + adjacency)

## Intent

Land `oya-notes-backlink-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk,app}`. Includes the shared `pulldown-cmark`-based wikilink-extension parser, which is the SINGLE source of truth used by both server worker AND client SDK (via Wasm bindings).

Per ADR-NOTES-0002: server-side adjacency materialisation for Professional-tier; client-side for Personal-tier.

## Port Traits

```rust
pub trait BacklinkRepository {
    fn upsert(&self, edge: Backlink) -> Result<(), BacklinkError>;
    fn delete(&self, edge: BacklinkKey) -> Result<(), BacklinkError>;
    fn list_outgoing(&self, from: NoteId) -> Result<Vec<Backlink>, BacklinkError>;
    fn list_incoming(&self, to: NoteId) -> Result<Vec<Backlink>, BacklinkError>;
}
```

## Wikilink Parser

Shared crate `oya-notes-wikilink-parser` (used by both server worker + Wasm-bound client SDK). Resolves `[[Foo]]` per ADR-NOTES-0002 §3 (recency-rank + picker disambiguation).

## Test Plan

- Wikilink parser deterministic across server + Wasm.
- Backlink fan-in cap enforced at 50,000.
- Dangling link emits `BacklinkBroken` event.

## Acceptance Gates

```bash
cargo check -p oya-notes-backlink-graph-kernel
cargo test  -p oya-notes-wikilink-parser  # shared parser tests
cargo run -p oya-dev-cli -- gate validate port-location --microservice notes
```

## Next IP

[`IP-006-daily-note-template-gallery.md`](IP-006-daily-note-template-gallery.md)
