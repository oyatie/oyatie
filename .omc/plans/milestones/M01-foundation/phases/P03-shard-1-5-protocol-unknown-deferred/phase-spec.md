---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P03-shard-1-5-protocol-unknown-deferred
status: Complete
acceptance_lanes: []
entry_gate: "P02-shard-1-atomic-rename complete (all 6 acceptance gates exit 0); P04\n\
  iter-4-src-inspection complete (all 26 PROTOCOL-UNKNOWN rows have protocol\nclassification\
  \ evidence: rest|grpc|graphql|worker per canonical decision tree\n\xA72.2.3); no\
  \ freeze window required (Shard 1.5 is a follow-on, not a parallel\nmerge per ADR-0057\
  \ \xA7\"Shard 1.5\").\n"
exit_gate: 'All 26 PROTOCOL-UNKNOWN rows renamed to final BNF v4.1 names; zero

  `*-api` crates remain in workspace (or: any retained `-api` names carry

  an explicit ADR amendment justifying retention); cargo check --workspace

  exits 0; cargo clippy exits 0; ICM context-oyatie row emitted; grit done

  or direct merge with ICM rationale.

  '
depends_on:
- milestone: M01
  phase: P02-shard-1-atomic-rename
  reason: Shard 1.5 operates on crates already in v4.1-partially-renamed workspace
- milestone: M01
  phase: P04-iter-4-src-inspection
  reason: Protocol classification evidence from iter-4 src-inspection is the gate
    to enter Shard 1.5
owner_team: council-architecture
purpose: "Completes the reclassification of the 26 `*-api` crates whose protocol was marked `PROTOCOL-UNKNOWN` in the §3 audit body and deferred from Shard 1."
---
# P03-shard-1-5-protocol-unknown-deferred: Shard 1.5 — PROTOCOL-UNKNOWN 26-row reclassification

## Purpose

Completes the reclassification of the 26 `*-api` crates whose protocol was
marked `PROTOCOL-UNKNOWN` in the §3 audit body and deferred from Shard 1.
Per ADR-0057 §"Shard 1.5", each `-api` crate receives a protocol-classified
final name (`rest`, `grpc`, `graphql`, or `worker`) derived from iter-4
src-inspection evidence. After this phase, no `*-api` crates with ambiguous
protocol classification remain in the workspace.

Advances Master Plan principles: every crate name names its wire format
explicitly (no ambiguous `api` token); clean architecture protocol layer enum
is complete.

---

## Scope

### In-scope

| Partition | PROTOCOL-UNKNOWN rows | Count |
|---|---|---:|
| platform | rows 6, 8, 20, 24, 26 | 5 |
| cloud | rows 32, 34, 35, 36, 38, 44, 49, 51, 52, 55, 56, 57, 59 | 13 |
| foundry non-check | rows 60, 72, 73, 74 | 4 |
| connect/workspace | rows 115, 122, 125, 128 | 4 |
| **total** | | **26** |

For each row: inspect `src/` to classify protocol; rename `crates/oya-*-api/`
to `crates/oya-*-{rest|grpc|graphql|worker}/`; update `[package] name`, dep
refs, root Cargo.toml members, Cargo.lock.

Naming justifications required per `feedback_naming_justification.md` for all
26 new names before rename executes.

### Out-of-scope

- Any crate not in the 26-row PROTOCOL-UNKNOWN set.
- Architecture ADR amendments (protocol classification is evidence-driven, not design).

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Classify 26 PROTOCOL-UNKNOWN rows + execute rename | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features                        # exit 0
cargo clippy --workspace --all-targets -- -D warnings         # exit 0
cargo nextest run --workspace || cargo test --workspace       # exit 0
cargo deny check                                              # exit 0
```

### Reality verification

```bash
# Zero *-api crates remain (or explicit ADR amendment for retained ones)
cargo metadata --format-version 1 | \
  jq '.packages[].name' | grep '"-api"$' | wc -l   # must be 0
```

---

## Clean Architecture Compliance

### Protocol classification decision tree (§2.2.3)

For each `-api` crate, ask in order:
1. Does `src/` contain `Router::new()` or axum/actix handler? → `rest`
2. Does `src/` contain `tonic::server` or `.proto` service impl? → `grpc`
3. Does `src/` contain `async-graphql Schema` or GraphQL resolvers? → `graphql`
4. Does `src/` contain a long-running event-consumer loop? → `worker`
5. Multi-protocol? → split into per-protocol crates; document in ADR amendment.

### New BCs registered in this phase

None expected — all BCs were registered in P01/P02. If a protocol split creates
a new BC token, register per `docs/templates/bounded-context-registration-template.md`.

---

## Grit Claim Symbols

```
crates/oya-*-api/Cargo.toml::package.name   (26 crates)
Cargo.toml::workspace.members
Cargo.lock::all
```

TTL: 3600s per crate batch (process by partition).

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "P03-shard-1-5-protocol-unknown-deferred START. 26 PROTOCOL-UNKNOWN rows entering classification. Partitions: platform 5, cloud 13, foundry 4, connect 4. Gate: iter-4 src-inspection evidence for each row. Next: rename to rest/grpc/graphql/worker per classification." \
  -i high \
  -k "M01,P03,shard-1.5,protocol-unknown,26-rows"
```

---

## References

- ADR-0057: `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md` §"Shard 1.5"
- ADR-0056: §"Layer semantics" §2.2.3 presentation layers
- Rename plan §3.1 rows 6,8,20,24,26; §3.2 rows 32-59 (PROTOCOL-UNKNOWN subset); §3.3.1 rows 60,72,73,74; §3.4 rows 115,122,125,128
- Memory: `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`
