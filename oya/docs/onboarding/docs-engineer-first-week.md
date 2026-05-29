# Docs Engineer — First Week on `docs`

Audience: an engineer with collaborative-editor experience (Y.js, Automerge, ProseMirror, Slate, TipTap, BlockNote, Notion-class
block editor) joining the `oya-docs-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0222-docs-collaborative-document-surface.md` — binding scope.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md` — gRPC + 3 invariants.
- Y.js documentation (https://docs.yjs.dev/) — the CRDT family we wire-compat with.
- ProseMirror schema documentation — our block-schema engine derives from ProseMirror's discipline.
- RFC 9000 (QUIC) — we use QUIC streams as the CRDT op transport.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-docs-week1 .worktrees/$USER-docs-week1
```

## Day 2 — walk the editor end-to-end

```bash
make dev-cell.up CELL=docs-loopback-1 PROFILE=docs-dev
make dev-tenant.create T=oyatie.community.dev-sample TENANT_CLASS=demo_trial
```

Open two browser tabs:
```bash
open https://loopback.docs.oyatie.local/d/new?tenant=oyatie.community.dev-sample
open https://loopback.docs.oyatie.local/d/new?tenant=oyatie.community.dev-sample
```

Edit simultaneously; observe live cursors + convergent state. Inspect the underlying CRDT ops:
```bash
./bin/oya docs ops trail --doc-id <doc-id> --window 5m
```

## Day 3 — code walkthrough

Top-down:
1. `crates/oya-docs-domain/src/block.rs` — closed `BlockKind` enum (text, heading, list, table, code, embed, …).
2. `crates/oya-docs-domain/src/schema.rs` — versioned block-schema.
3. `crates/oya-docs-kernel/src/crdt.rs` — Y.js-compatible CRDT engine.
4. `crates/oya-docs-kernel/src/permission.rs` — Cedar-gated permission evaluation per block.
5. `crates/oya-docs-port-realtime/src/lib.rs` — WebSocket + QUIC transport abstraction.
6. `crates/oya-docs-app/src/api.rs` — REST + gRPC surface.
7. `web/docs-editor/` — the React editor (TypeScript) consuming the SDK.

## Day 4 — author a new block type

Pick a block from `microservices/docs/backlog/starter-blocks.md`. Suppose `MermaidDiagram`:

```rust
use oya_docs_block::prelude::*;

#[derive(Block)]
#[block(
    kind = "mermaid",
    schema_version = "1.0.0",
    availability = "paid"
)]
pub struct MermaidBlock {
    #[content]
    pub source: String,
    #[derived]
    pub rendered_svg: Option<String>,
}

impl Block for MermaidBlock {
    fn render(&self, ctx: &RenderCtx) -> RenderResult {
        let svg = mermaid_render(&self.source).map_err(BlockError::Render)?;
        RenderResult::ok_with_derived(svg)
    }
}
```

Add Cedar permit:
```cedar
permit (
  principal,
  action == docs::Action::CreateBlock,
  resource is docs::Block::Mermaid
);
```

Add UI + snapshot tests.

## Day 5 — ship through Foundry

```bash
./bin/oya vcs claim \
  --agent docs-eng-$USER \
  --intent docs-add-mermaid-block \
  crates/oya-docs-block-mermaid microservices/docs
```

Implement + verify + done + PR.

## Done with week 1

- [ ] You walked a multi-cursor collab session and observed convergent CRDT state.
- [ ] You read ADR-0222 + Y.js + ProseMirror docs.
- [ ] You authored, signed, and merged a new block type.
- [ ] You can name the difference between `docs`, `notes`, and `sites`.
- [ ] You traced a CRDT op through the audit chain.

## Rookie traps

1. **Mutating block schema in-place.** Block schemas are versioned; backward-incompat changes need new version + migrator.
2. **Skipping Cedar on block-level.** Every block action is Cedar-gated; bypassing leaks data across permission tiers.
3. **CRDT divergence.** Always run the convergence property test (`cargo test -p oya-docs-kernel --features property-tests`).
4. **Forgetting AI-block policy.** AI prompts inside documents are subject to per-tenant policy; ungated AI calls fail audit.
