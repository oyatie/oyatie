---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-008-comments-and-suggestions
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: comments-and-suggestions BC (9 crates)

## Intent

Implement comment threads + suggestion (track-changes) state machine with CRDT-aware anchors that survive arbitrary edits.

## ChangeSet boundary

9 crates per layer mapping: kernel + domain + usecase + api + adapter + adapter-postgres + rest + worker + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-comments-and-suggestions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-comments-and-suggestions-domain/src/{anchor_stability,suggestion_state_machine,thread_resolution}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-comments-and-suggestions-domain -- anchor_survives_arbitrary_edits
cargo nextest run -p oya-docs-comments-and-suggestions-domain -- suggestion_state_machine
cargo nextest run -p oya-docs-comments-and-suggestions-domain -- no_auto_acceptance
```

## References

- Migration Hyrum #6 (anchor stability) + Hyrum #7 (no auto-acceptance).
