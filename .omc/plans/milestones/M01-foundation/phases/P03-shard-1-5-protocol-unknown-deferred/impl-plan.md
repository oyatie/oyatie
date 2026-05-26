---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P03-shard-1-5-protocol-unknown-deferred
impl_plan_id: IP-001-shard-1-5-protocol-rename
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P04/IP-001-iter-4-src-inspection
  reason: Protocol classification evidence must come from iter-4 src-inspection
acceptance_lanes:
- cargo-check
- cargo-clippy
- cargo-nextest
- cargo-deny
purpose: "Inspects each of the 26 `*-api` crates deferred from Shard 1, determines the correct protocol layer (`rest`, `grpc`, `graphql`, or `worker`) via src-inspection, and executes the rename."
---
# IP-001-shard-1-5-protocol-rename: Classify and rename 26 PROTOCOL-UNKNOWN crates

## Intent

Inspects each of the 26 `*-api` crates deferred from Shard 1, determines the
correct protocol layer (`rest`, `grpc`, `graphql`, or `worker`) via src-inspection,
and executes the rename. After merge, zero ambiguous `-api` crates remain in the
workspace.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/<old-api-name>/` (×26) | rename dir → `crates/<new-protocol-name>/` | per classification result |
| `crates/<new-name>/Cargo.toml` (×26) | update | `[package] name`; dep keys/paths |
| `Cargo.toml` | update | workspace members for 26 entries |
| `Cargo.lock` | rewrite | lockfile-rename for 26-row TSV |
| `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` | update | Replace PROTOCOL-UNKNOWN cells with final names + evidence cites |

---

## Crate Naming

For each of the 26 rows, naming justification is generated at classification time.
Template:

```
NAME: oya-<microservice>[-<bc>]-<protocol-layer>
JUSTIFICATION:
- microservice = <same as v3 microservice slot>: unchanged
- bc-tokens = <same as v3 bc>: unchanged
- layer = <rest|grpc|graphql|worker>: per src-inspection evidence: <file:line — pattern>
- exemptions claimed: none
```

Specific cases requiring attention:
- `oya-intelligence-rag-api` (row 74): streaming retrieval → likely `grpc` (server-streaming) OR `rest` with SSE; if multi-protocol, split into `oya-foundry-rag-rest` + `oya-foundry-rag-worker`.
- `oya-cloud-observability-api` (row 59): OTLP ingestion typically `grpc`; control-surface `rest`; may require split.
- `oya-workspace-chat-api` / `oya-connect-messenger-api`: WebSocket/GraphQL subscriptions candidate → `graphql` if async-graphql; `worker` if event-stream consumer.

---

## Code Shape

Same as P02 — rename-only; no new logic. Protocol classification changes only
the layer suffix.

---

## Acceptance Gates

```bash
# Zero *-api crates in metadata
cargo metadata --format-version 1 | jq '.packages[].name' | grep '"-api"$' | wc -l  # 0

# Workspace compiles
rtk cargo check --workspace --all-features     # exit 0
rtk cargo clippy --workspace --all-targets -- -D warnings  # exit 0
rtk cargo nextest run --workspace || cargo test --workspace  # exit 0
rtk cargo deny check                           # exit 0
```

---

## Test Plan

Rename-only; existing tests (scaffold-empty) unchanged. Integration: workspace nextest.

---

## Clean Architecture Compliance

Protocol layer assignments per §2.2.3:
- `rest` → depends on axum/actix; has `Router::new()`
- `grpc` → depends on tonic; has `Server::new()` or tonic service impl
- `graphql` → depends on async-graphql; has `Schema::build`
- `worker` → long-running async loop; no Router/Server/Schema

---

## Load Test

Not applicable — rename-only phase.

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent shard-1-5-executor \
  --intent "Shard 1.5: classify + rename 26 PROTOCOL-UNKNOWN *-api crates" \
  --ttl 3600 \
  Cargo.toml::workspace.members \
  Cargo.lock::all
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-shard-1-5-protocol-rename merged. 26 PROTOCOL-UNKNOWN *-api crates reclassified + renamed. Zero *-api crates in workspace. Protocol breakdown: rest=N, grpc=N, graphql=N, worker=N. Any splits documented. Cargo.lock rewritten. All acceptance gates exit 0." \
  -i high \
  -k "M01,P03,IP-001,shard-1.5,26-rows,protocol-classification,merged"
```

---

## Halt Conditions

1. A crate is genuinely multi-protocol (e.g. REST + gRPC) — split into two crates; author micro-ADR justifying split.
2. Classification is ambiguous after src-inspection — escalate to architect agent with full src/ evidence.
3. `cargo check` fails post-rename — check dep-ref cleanup (same pattern as P02).

---

## Next IP Pointer

`../P05-post-cutover-hardening/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 §2.2.3: presentation layer enum
- ADR-0057 §"Shard 1.5"
- Rename plan §3.6 deferral note (26-row breakdown by partition)
