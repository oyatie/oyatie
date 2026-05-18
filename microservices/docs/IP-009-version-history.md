---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-009-version-history
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-merkle-chain-continuity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: version-history BC (8 crates)

## Intent

Implement Merkle-chained version snapshots + revert + CRDT op-log compaction per ADR-DOCS-0001 (version-aligned compaction policy).

## ChangeSet boundary

8 crates per layer mapping.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-version-history-domain/src/{merkle_chain,compaction_planner,revert_validator}.rs` | create |
| `microservices/docs/src/crates/oya-docs-version-history-worker/src/{lib,compaction_worker,integrity_scan_worker}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-version-history-domain -- merkle_chain_continuity
cargo nextest run -p oya-docs-version-history-domain -- compaction_byte_equality  # AC-02
cargo run -p oya-dev-cli -- gate validate merkle-chain-continuity --microservice docs
```

## References

- ADR-0028 (Bominal audit-chain).
- ADR-DOCS-0001 (compaction policy).
