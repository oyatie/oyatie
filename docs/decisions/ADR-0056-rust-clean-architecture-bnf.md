---
doc_class: DecisionRecord
shape: ~
length_cap: 500
authority_tier: 2
status: Accepted
date: 2026-05-13
purpose: |
  Formalise the canonical 3-slot BNF `oya-<shared|vertical>-<bounded-context>-<layer>`
  with 12 closed layer values for every oya-* Rust crate, replacing the v3
  4-5-segment BNF. Establishes the bounded-context registry as a living document,
  the vertical naming policy, the cloud dual-role + public_layers mechanism,
  the build-tooling vs coordination-primitive distinction, and the flat
  oya-check-<rule-name> namespace for cross-cutting checks. Sources
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md §2 + §11 as the
  ground-truth outline.
canonical_authority: docs/CONSTITUTION.md
supersedes: ~
superseded_by: ~
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0054
  - ADR-0057
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/standards/code-style-rust.md
  - docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
---

# ADR-0056: Rust Clean Architecture BNF — 3-Slot Grammar + 12-Layer Enum

> **Status:** Accepted — 2026-05-13
> **Owner:** `council-architecture`
> **Supersedes:** The v3 crate-naming BNF in `docs/standards/crate-naming-convention.md`
> (that document is rewritten in Shard 1 to reflect this ADR).

---

## Context

The v3 BNF (`oya-<context>-<feature>-<capability>-<role>`) produced names like
`oya-foundry-fitness-architecture-conventions-kernel` (5 segments) and forced
every check/fitness crate to claim a product layer it did not conceptually
occupy. It could not cleanly parse `oya-tooling-agent-read` (no role token).
The `fitness` jargon did not reflect team vocabulary. v4 replaces it.

---

## Decision

### Canonical BNF (3-slot grammar)

```bnf
crate           ::= "oya" "-" shared-or-vertical "-" bounded-context "-" layer
                  | "oya" "-" "check" "-" rule-name

shared-or-vertical ::= "shared"
                      | vertical

vertical        ::= kebab-token          (* EXACTLY 1 token; registry-validated; "shared" reserved *)

bounded-context ::= kebab-token ( "-" kebab-token )*   (* 1..N tokens; open *)

layer           ::= "kernel" | "domain" | "application" | "app"
                  | "adapter" | "infrastructure"
                  | "cli" | "rest" | "grpc" | "graphql"
                  | "worker" | "sdk"

rule-name       ::= kebab-token ( "-" kebab-token )*   (* 1..4 tokens; open *)

kebab-token     ::= [a-z] [a-z0-9]*
```

**Parser rule**: split crate name on `-`; LAST token MUST be a layer value
(one of 12 canonical); SECOND token (after `oya-`) MUST be `shared` OR a
registered vertical name from `[workspace.metadata.oya.verticals]`; remaining
middle tokens (joined by `-`) = bounded-context. The `oya-check-*` namespace
is exempt.

### 12-Value Layer Enum (closed)

| Group | Values |
|---|---|
| Inner / pure (4) | `kernel`, `domain`, `application`, `app` |
| Outer / external (2) | `adapter`, `infrastructure` |
| Presentation / entry-point (6) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk` |

Adding a layer value is a **1-ADR action**. No aliases or overlaps.

### Layer semantics (canonical, per Uncle Bob ch. 22 + DDD ch. 5–6)

- **`kernel`** — Pure types + ports (trait declarations). ZERO business logic, zero I/O, zero async. The shared kernel in DDD.
- **`domain`** — Business logic on kernel types: entities, domain services, invariant enforcement. Pure; no I/O; no framework deps.
- **`application`** — Use cases / application services orchestrating domain via port-trait bounds. No concrete adapters.
- **`app`** — Composition-root binary wiring every other layer into a deployable service. Unrestricted inward deps.
- **`adapter`** — Trait implementations of kernel ports + DTO mappers. Primary surface: `impl <KernelTrait> for <Struct>`.
- **`infrastructure`** — Framework / driver glue without being a trait impl (axum routers, OTel exporters, pool helpers).
- **`cli`** — CLI binary or CLI library (subcommand handlers + optional `[[bin]]`).
- **`rest`** — HTTP REST API handlers + routing (axum-style).
- **`grpc`** — gRPC service definitions + tonic handlers.
- **`graphql`** — GraphQL schema + resolvers.
- **`worker`** — Long-running background workers: queue consumers, pubsub, scheduled tasks.
- **`sdk`** — Client libraries for external consumers; depends on `kernel` only.

### Port location: `kernel` (not `domain`)

Port trait declarations (`trait FooRepository: Send + Sync { … }`) live in
`kernel`, NOT `domain`. The domain layer holds business logic that uses
(calls through) those ports; it does not define them. This supersedes the
wording in `docs/standards/clean-architecture.md` §2.1 which placed ports in
`domain`; that wording is updated in Shard 1 (step 15a).

---

## Decision Drivers

1. **The v3 BNF cannot parse the load-bearing CLI.** `oya-tooling-agent-read`
   had no role token; v4 parses it as `oya-shared-codeview-cli` cleanly.
2. **`fitness` is jargon, not vocabulary.** Every fitness crate was a check or
   probe; the `oya-check-*` namespace names them honestly.
3. **grit already enforces freeze windows via symbol locks.** The
   `oya-foundry-fitness-freeze-window-kernel` lane primitive is dropped; grit's
   claim system handles the coordination window natively.

---

## Alternatives Considered

### Option D — Status quo (do nothing; keep v3 BNF)
Pros: v3 is consensus-approved; no further cost.
Cons: 6-segment AMBER tax persists; `oya-tooling-agent-read` remains awkward;
`fitness` crates continue to misrepresent their purpose.
**Verdict:** Rejected.

### Option E — Thing-domain literal (`oya-<bc>-<thing>-<layer>`)
Forces a `<thing>` token where none semantically exists; pessimises the common
case; no hyperscaler analogue.
**Verdict:** Rejected; granularity expressed via multi-token BC names instead.

### Option F — Drop-verb pattern (3-segment always)
Cannot disambiguate domain from infrastructure of the same BC without a layer
suffix; collapses too aggressively.
**Verdict:** Rejected.

---

## Why Chosen (Option C)

- Closed 12-value layer enum gives Cargo + cargo-metadata enough information to
  enforce dependency direction at compile/CI time AND names each protocol/wire-
  format directly (no ambiguous `api` token).
- Bounded contexts grow without ADR overhead (0-ADR action to add a BC).
- Checks live in `oya-check-*` namespace that never collides with product code.
- Reduces total grammar tokens vs. v3 by ~50 %.
- Matches hyperscaler precedent (AWS smithy-rs, Azure SDK for Rust, Google Cloud Rust).

---

## Consequences

### Positive
- Every crate name encodes its layer and vertical unambiguously.
- Dependency direction is mechanically enforceable by `oya-shared-architecture-check-cli` (LEAN-A1).
- Bounded-context registry is auto-derived from `[package.metadata.oya].bounded_context` fields.

### Negative
- Higher one-time rename count (~140 existing + 4 new = ~144 ops in Shard 1).
- Without a closed BC enum, teams could disagree on BC names (mitigated by §"Bounded context registry as a living document" + R10 5-layer permanent-controls ledger).

---

## Bounded Context Registry as a Living Document

The registry is `docs/standards/bounded-contexts.md`. Every bounded context
appearing in any crate's `[package.metadata.oya].bounded_context` MUST appear
in that document with:
- `name` (kebab; matches the slot-3 token)
- `kind`: `shared` or `vertical`
- `vertical` (required iff `kind == "vertical"`; single-token kebab from `[workspace.metadata.oya.verticals]`)
- `owner` (default: `council-architecture`)
- `rationale` (1 paragraph)
- `adr_cite` (one-line ADR reference)
- `parent: <bc>` (optional; if BC is a prefix-child of another BC)

Adding a bounded context is a **0-ADR action** — the author writes the name
and adds a registry entry. The xtask `--check-bc` mode refuses unregistered BCs.
A BC with zero crates after 90 days is auto-deprecated (soft; warn only).

**BC arbitrator clause (B3)**: If two PRs propose conflicting BC names for the
same crate cluster, `council-architecture` reviews both; tie-breaker is the
proposal with the more specific `rationale` paragraph + ADR cite; ultimate
tiebreaker is the earlier-PR timestamp.

---

## Vertical Naming Policy

The slot-2 token (shared-or-vertical) MUST be:
- The literal `shared` (reserved; cross-vertical; not a vertical name), OR
- A single kebab token registered in `[workspace.metadata.oya.verticals]`.

**Option A (CHOSEN)**: exactly 1 token per vertical name. Multi-word verticals
(e.g. `healthcare`) use a single token. `shared` is RESERVED and refused as a
vertical name. Adding a vertical requires: registry append in root `Cargo.toml`
`[workspace.metadata.oya.verticals]` + 1 ADR cite.

Initial verticals: `cloud` (owner: council-cloud), `foundry` (owner:
council-foundry), `workspace` (owner: council-workspace).

---

## Cloud Dual-Role + `public_layers` Mechanism

The `cloud` vertical plays a dual role: it is a product vertical (billing,
compute, IAM) AND a potential infrastructure substrate for other verticals
(e.g., `workspace-drive` might legitimately consume `cloud-storage-sdk`).

Cross-vertical dependencies are normally refused by `oya-shared-bounded-contexts-check-cli`
(LEAN-A2). The `public_layers` exemption allows a registered vertical to declare
specific layer values as its public surface:

```toml
[workspace.metadata.oya.verticals.cloud]
public_layers = ["sdk"]
```

A `workspace-*` crate depending on `cloud-storage-sdk` is allowed because `sdk`
is in `cloud.public_layers`. LEAN-A2 checks the `public_layers` allowlist at
EVERY cross-vertical hop in both direct AND transitive dep chains.

---

## Build Tooling vs Coordination Primitives

`xtask-metadata-augment` is a build tool (not a coordination primitive):
- It reads workspace manifests and rewrites `[package.metadata.oya]` blocks.
- It rewrites Cargo.lock crate names per a rename-map TSV.
- It generates `/tmp/old-crate-names.txt` and `/tmp/rename-map.tsv` from the §3 audit table.

Coordination primitives remain `grit` + `icm` per ADR-0053 + ADR-0054.
The xtask does NOT make scheduling decisions, does NOT hold locks, and does NOT
replace ICM scaffold-claim windows.

---

## BNF Accommodation (B7 closure — 4 gap cases)

| Gap case | Canonical layer assignment |
|---|---|
| **Proc-macros** (`proc-macro = true`) | Layer = `infrastructure` (framework-level code generation; not a port trait impl; not business logic). Name: `oya-<v>-<bc>-infrastructure`. |
| **Codegen crates** (build.rs heavy; schema-to-Rust generation) | Layer = `infrastructure` (same rationale as proc-macros; code generation is framework glue). |
| **Test-fixture crates** (`#[cfg(test)]` helpers shared across crates) | Layer = `domain` if fixtures are pure domain types; `adapter` if fixtures are in-memory port impls. Name: `oya-<v>-<bc>-<layer>` with `purpose` field noting "test fixtures only". |
| **Library + binary split** (crate has both `[lib]` and `[[bin]]`) | Assign layer by the PRIMARY surface: if the primary surface is a CLI binary, layer = `cli`; if the primary surface is a library consumed by other crates, layer = the library's semantic layer. The `app` layer explicitly allows both surfaces (composition-root binary MAY export a thin library shim). |

---

## Protocol Classification

Every existing `*-api` crate requires a one-time protocol audit per the
canonical decision tree (§2.2.4):
- HTTP/1.1 + JSON → `rest`
- HTTP/2 + Protobuf (tonic) → `grpc`
- HTTP/1.1 + JSON GraphQL → `graphql`
- AMQP/Kafka/Pub-Sub/cron → `worker`
- Multi-protocol → split into per-protocol crates OR document exception in ADR-0056 §"Bounded context registry" with rationale.

Rows with `PROTOCOL-UNKNOWN` in the §3 audit table are deferred to Shard 1;
the protocol is confirmed by `src/`-inspection before the rename is applied.
The default-to-`rest` classification for existing `-api` crates is a
planner-best-guess; Codex iter-1 must verify per crate.

---

## Follow-ups

1. **Shard 1**: update `docs/standards/clean-architecture.md` §2.1 to reflect port-location move (`kernel` not `domain`); cite this ADR.
2. **Shard 1**: rewrite `docs/standards/crate-naming-convention.md` to reflect 3-slot BNF + 12-layer enum; cite this ADR.
3. **Shard 1**: update `docs/standards/code-style-rust.md` lines 11–12, 137–147, 162–177 to replace v3 BNF + 9-value role enum with v4 3-slot BNF + 12-value layer enum; cite this ADR.
4. **Shard 1**: author `docs/standards/bounded-contexts.md` full version (populated from `[package.metadata.oya].bounded_context` fields).
5. **Post-Shard-1**: flip `oya-shared-architecture-check-cli` + 3 sibling LEAN check crates from `--report-only` to BLOCKER (§8.2 global-gate follow-up commit).

---

## References

- `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §2, §4a, §11 — source of truth for BNF + layer semantics
- ADR-0015 — flat crates layout
- ADR-0017 — `oya-` prefix
- ADR-0054 — scaffold-claim pattern (amended in this Shard 0 commit to cover rename events)
- ADR-0057 — cutover mechanics (supersedes ADR-0055)
- Robert C. Martin — *Clean Architecture* ch. 22
- Eric Evans — *Domain-Driven Design* ch. 5–6
- AWS smithy-rs, Azure SDK for Rust, Google Cloud Rust — hyperscaler naming precedent
