---
id: ADR-0056
doc_class: DecisionRecord
shape: ~
length_cap: 500
authority_tier: 2
status: Superseded
doc_status: published
date: 2026-05-13
version: v4.1
purpose: |
  Formalise the canonical BNF v4.1 `oya-<microservice>-(<bc-tokens>-)?<layer>`
  with 12 closed layer values for every oya-* Rust crate. Replaces v4.0
  `shared|vertical` slot2 enum with open microservice kebab token. Establishes
  the bounded-context registry as a living document, the cloud dual-role +
  public_layers mechanism, the build-tooling vs coordination-primitive
  distinction, and the flat oya-check-<rule-name> namespace for cross-cutting
  checks. Sources docs/plans/rename-plan-v4-clean-arch-2026-05-13.md §2 + §11
  and session decisions 2026-05-13.
canonical_authority: docs/CONSTITUTION.md
supersedes: ~
superseded_by: [ADR-700]
amended_by: [ADR-0105, ADR-0106, ADR-0107, ADR-0565, ADR-0632]
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0054
  - ADR-0057
  - ADR-0058
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/standards/code-style-rust.md
  - docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0056: Rust Clean Architecture BNF v4.1 — Flat Microservice Grammar + 12-Layer Enum

> **Status:** Accepted — 2026-05-13 (v4.1 amendment 2026-05-13)
> **Date:** 2026-05-13
> **Owner:** `council-architecture`
> **Supersedes:** The v3 crate-naming BNF in `docs/standards/crate-naming-convention.md`
> and the v4.0 `shared|vertical` slot2 enum (retired per session decision 2026-05-13).

> **F-0029 RECONCILIATION (ratified 2026-06-07, door:one-way).** The layer enum across the
> amendment chain ADR-0056 → ADR-0105 → ADR-0106 is reconciled to a single SSOT: **13 closed
> product-layer values** — `kernel`, `domain`, `usecase`, `app`, `adapter`, `infrastructure`,
> `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api` — **plus the governance-only `check`
> family layer** (used only by `oya-check-*`; a self-layering convention, NOT a product value and
> NOT a BNF terminal token). `application` is **retired** (→ `usecase`, ADR-0106); `runtime` was
> **never canonical** (→ `app`). Resolution order is the ADR chain: ADR-0056 (base 12) → ADR-0105
> (+`api` ⇒ 13) → ADR-0106 (`application` → `usecase`). This banner closes ADR-0106 Follow-ups
> #1–#2; the normative enum/BNF/semantics in this ADR are updated in place below to match.

---

## ADR-0632 product-protocol reconciliation

The crate-layer grammar names implementation boundaries, not public product authorization. Public contracts are HTTPS REST documented by OpenAPI 3.2.0, signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, and WebSocket. GraphQL, public gRPC, gRPC-Web, and Connect are forbidden. The `grpc` layer is internal-only gRPC/proto3 over HTTP/2; the historical `graphql` token remains naming provenance only and cannot authorize a GraphQL crate or owned API surface under ADR-0565.

## Context

The v4.0 BNF used `shared|vertical` as slot2. Session decision 2026-05-13 retired the
`shared|vertical` binary: "we can retire shared or vertical distinction" + "flat
microservice structure." Everything is shared conceptually; every crate belongs to
a microservice. Slot2 becomes the registered microservice name (open kebab token).

v4.1 also formalises the BC optionality rule (BC slot is optional when a µservice has
a single concept at the layer) and adds the check-namespace exemption.

---

## Decision

### Canonical BNF v4.1

```bnf
crate          ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
                 | "oya" "-" "check" "-" rule-name

microservice   ::= kebab-token ( "-" kebab-token )*    (* 1..3 tokens; registry-validated *)

bc-tokens      ::= kebab-token ( "-" kebab-token )*    (* 0..N; OPTIONAL *)

layer          ::= "kernel" | "domain" | "usecase" | "app"
                 | "adapter" | "infrastructure"
                 | "cli" | "rest" | "grpc" | "graphql"
                 | "worker" | "sdk" | "api"

rule-name      ::= kebab-token ( "-" kebab-token )*    (* 1..4 tokens; open *)

kebab-token    ::= [a-z] [a-z0-9]*
```

**Parser rule**: split crate name on `-`; LAST token MUST be a layer value (one of 13
canonical); FIRST token is `oya`; SECOND token (after `oya-`) begins the microservice
name (1..3 tokens, registry-validated); remaining middle tokens (if any) = BC tokens
(optional). The `oya-check-*` namespace is fully exempt.

### Why slot2 was renamed from `shared|vertical`

v4.0 used `shared|vertical` as a binary. Session decision 2026-05-13: "In essence
everything is 'shared'." Every microservice in the flat catalog is equally shared
conceptually — Cloud, Ontology, Workflow, Medical, Connect, Payments are all the same
kind of thing: an independent modular microservice. There is no architectural
significance to the `shared` vs `vertical` distinction; it was retired to avoid
misleading naming. Slot2 now carries the registered microservice name, which is
unambiguous. See `[[feedback-flat-product-catalog]]` §"BNF resolution".

### BC optionality rule

BC slot is OPTIONAL:
- **Omit BC** when the µservice has a single binary or single concept at the layer.
  Examples: `oya-medical-domain`, `oya-tenancy-kernel`, `oya-cloud-cli`.
- **Include BC** when the µservice has multiple binaries OR multiple BC-level splits
  at the same layer. Examples: `oya-intelligence-grit-cli`, `oya-intelligence-icm-cli`,
  `oya-workflow-state-machine-domain`, `oya-workflow-approvals-usecase`.

### Check-namespace exemption

`oya-check-<rule-name>` is a flat namespace for cross-cutting check rules. These crates
are not bound to any microservice slot2. Rule names are 1..4 kebab tokens. Examples:
`oya-check-architecture`, `oya-check-glossary`, `oya-check-statelessness`,
`oya-check-shardability`, `oya-check-perf-budget`, `oya-check-benchmark`.

### Microservice registry

Slot2 (microservice name) is validated against `[workspace.metadata.oya.microservices]`
in root `Cargo.toml`. Initial registered microservices include:

```toml
[workspace.metadata.oya.microservices]
# Substrate
tenancy = { owner = "council-architecture" }
identity = { owner = "council-architecture" }
audit-chain = { owner = "council-architecture" }
eventing = { owner = "council-architecture" }
policy = { owner = "council-architecture" }
secrets = { owner = "council-architecture" }
observability = { owner = "council-architecture" }
kms = { owner = "council-architecture" }
search = { owner = "council-search" }
vector = { owner = "council-architecture" }
records = { owner = "council-architecture" }
finance-library = { owner = "council-architecture" }
capability-registry = { owner = "council-foundry" }
data-boundary = { owner = "council-privacy" }

# Application shell
application = { owner = "council-architecture" }

# Adapter layer
workflow = { owner = "council-architecture" }
ontology = { owner = "council-architecture" }

# Cloud substrate
cloud = { owner = "council-cloud", public_layers = ["sdk"] }

# Foundry (internal)
foundry = { owner = "council-foundry" }

# Customer-facing products
medical = { owner = "council-healthcare" }
pharmacy = { owner = "council-healthcare" }
connect = { owner = "council-connect" }
hr = { owner = "council-enterprise" }
payroll = { owner = "council-enterprise" }
accounting = { owner = "council-enterprise" }
payments = { owner = "council-fintech" }
ads = { owner = "council-ads" }
analytics = { owner = "council-ads" }
# ... (full list per flat catalog; ADR-0058)
```

### 13-Value Layer Enum (closed) — reconciled per ADR-0105 (+`api`) and ADR-0106 (`application` → `usecase`)

| Group | Values |
|---|---|
| Inner / pure (4) | `kernel`, `domain`, `usecase`, `app` |
| Outer / external (2) | `adapter`, `infrastructure` |
| Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api` |

Adding a layer value is a **1-ADR action**. No aliases or overlaps. The governance-only `check`
family (`oya-check-*`) is a self-layering convention (ADR-0105 Amendment 2), NOT one of the 13
product values.

### Layer semantics

- **`kernel`** — Pure types + ports (trait declarations). ZERO business logic, zero I/O, zero async.
- **`domain`** — Business logic on kernel types. Pure; no I/O; no framework deps.
- **`usecase`** *(was `application`; renamed per ADR-0106)* — Use cases / application services orchestrating domain via port-trait bounds. No concrete adapters.
- **`app`** — Composition-root binary wiring every other layer into a deployable service. Unrestricted inward deps.
- **`adapter`** — Trait implementations of kernel ports + DTO mappers.
- **`infrastructure`** — Framework / driver glue without being a trait impl (axum routers, OTel exporters, pool helpers).
- **`cli`** — CLI binary or CLI library (subcommand handlers + optional `[[bin]]`).
- **`rest`** — HTTP REST API handlers + routing.
- **`grpc`** — gRPC service definitions + tonic handlers.
- **`graphql`** — GraphQL schema + resolvers.
- **`worker`** — Long-running background workers: queue consumers, pubsub, scheduled tasks.
- **`sdk`** — Client libraries for external consumers; depends on `kernel` only.
- **`api`** *(added per ADR-0105)* — Protocol-neutral contract surface: typed inputs/outputs/error variants without HTTP/gRPC/GraphQL commitment. Producer of types; depends on `kernel` only.

### Port location: `kernel` (not `domain`)

Port trait declarations (`trait FooRepository: Send + Sync { … }`) live in `kernel`,
NOT `domain`. The domain layer holds business logic that uses those ports.

### Quality/Performance/Scalability gate (per ADR-0062)

Every µservice's PRD MUST include competitive-benchmark, performance-targets, and
horizontal-scalability sections before the µservice graduates from Proof-Ladder L4 → L5.
Every implementation plan MUST include a `## Load test` section. These are enforced by:
- `oya-check-perf-budget-cli` — verifies impl plans include load-test results
- `oya-check-benchmark-cli` — verifies PRDs include competitive-benchmark section

### Clean architecture CI enforcement matrix

The 14 CI lanes that enforce the 12-layer enum + clean-arch rules on every PR
(all start `--report-only`; flip to BLOCKER at M02 exit gate except LEAN-A3/A4 which are BLOCKER day-1):

| Lane | Enforces |
|---|---|
| `oya-shared-architecture-check-cli -- dependency-direction` | Inward-only flow; 12-value layer import matrix |
| `oya-shared-architecture-check-cli -- layer-correctness` | Declared layer value matches code shape |
| `oya-shared-architecture-check-cli -- lib-name-parity` | `[lib] name` = snake_case(`[package] name`) |
| `oya-shared-architecture-check-cli -- port-location` | Port traits live in `kernel`, impls in `adapter` |
| `oya-shared-architecture-check-cli -- cross-product-refusal` | No direct cross-microservice imports (LEAN-A2) |
| `oya-shared-architecture-check-cli -- composition-root-only` | Only `app` layer has unrestricted inward deps |
| `oya-shared-architecture-check-cli -- sdk-kernel-only` | `sdk` layer depends only on `kernel` |
| `oya-shared-bounded-contexts-check-cli` (LEAN-A2) | BC registration + cross-product-refusal at BC level |
| `oya-shared-supply-chain-check-cli` (LEAN-A3) | `cargo-deny` bans + SBOM — BLOCKER day-1 |
| `oya-shared-semver-check-cli` (LEAN-A4) | API stability per ADR-0037 tiers — BLOCKER day-1 |
| `oya-check-statelessness-cli` | No module-level mutable state in presentation/usecase/worker |
| `oya-check-shardability-cli` | DB designs declare `tenant_id` partition key + RLS |
| `oya-check-perf-budget-cli` | Impl plans include load-test results meeting declared targets |
| `oya-check-benchmark-cli` | PRDs include Competitive Benchmark section before L4→L5 |

Inherited from Bominal: ADR-0100 (hexagonal reference impl), ADR-0101 (hexagonal microservice standard),
ADR-0102 (hexagonal migration plan), ADR-0103 (workflow hexagonal), ADR-0105 (clean-architecture layering),
ADR-0125 (domain naming canon). Per ADR-0060 inheritance rule.

### Examples (BNF v4.1)

```
oya-medical-encounter-domain        — Medical µservice, Encounter BC, domain
oya-payments-ledger-usecase         — Payments µservice, Ledger BC, usecase
oya-workflow-state-machine-domain   — Workflow µservice, State-Machine BC, domain
oya-ontology-entity-kernel          — Ontology µservice, Entity BC, kernel
oya-cloud-tenancy-adapter           — Cloud µservice, Tenancy BC, adapter
oya-cloud-storage-sdk               — Cloud µservice, Storage BC, sdk (public_layers)
oya-intelligence-grit-cli                — Foundry µservice, Grit BC, CLI
oya-application-product-enablement-rest — Application µservice, Product-Enablement BC, rest
oya-messenger-grpc          — µservice, Messenger BC, grpc
oya-tenancy-kernel                  — Tenancy µservice, no BC (single concept), kernel
oya-medical-domain                  — Medical µservice, no BC (single concept), domain
oya-check-architecture              — Check namespace (BNF-exempt)
oya-check-glossary                  — Check namespace (BNF-exempt)
oya-check-statelessness             — Check namespace (BNF-exempt)
```

### Cloud Dual-Role + `public_layers` Mechanism

The `cloud` microservice plays a dual role: product microservice AND infrastructure
substrate. Cross-microservice dependencies are normally refused by `oya-check-architecture`
(LEAN-A2). The `public_layers` exemption allows `cloud` to declare specific layer values
as its public surface:

```toml
[workspace.metadata.oya.microservices.cloud]
public_layers = ["sdk"]
```

A `connect-*` crate depending on `oya-cloud-storage-sdk` is allowed because `sdk` is
in `cloud.public_layers`. LEAN-A2 checks the `public_layers` allowlist at every
cross-microservice hop in both direct AND transitive dep chains.

---

## Decision Drivers

1. **`shared|vertical` binary retired.** Session decision 2026-05-13: flat microservice
   catalog; no vertical/shared distinction. Slot2 = registered microservice name.
2. **BC is optional.** Single-concept microservices don't need a BC token.
3. **Check namespace is flat.** `oya-check-*` never collides with product code.

---

## Consequences

### Positive
- Every crate name encodes its microservice and layer unambiguously.
- Dependency direction mechanically enforceable by `oya-check-architecture` (LEAN-A1).
- No `shared|vertical` disambiguation needed; everything is a named microservice.

### Negative
- Higher one-time rename count vs v4.0 (Shard 1 regenerates TSV with v4.1 flag).
- Shard 1 must regenerate rename-map TSV before dispatch:
  `tools/xtask-metadata-augment generate-rename-map --bnf-version v4.1`

### Concrete migration

Shard 1 atomic rename TSV changes under v4.1:
```
oya-platform-tenant-kernel     → oya-tenancy-kernel
oya-platform-identity-kernel   → oya-identity-kernel
oya-platform-audit-chain-kernel → oya-audit-chain-kernel
oya-shared-workflow-kernel     → oya-workflow-kernel
oya-shared-object-graph-kernel → oya-ontology-entity-kernel
oya-workspace-*                → oya-connect-*
```
(Full 114-row TSV in `/tmp/rename-map.tsv`; regenerated by xtask with `--bnf-version v4.1`.)

---

## Bounded Context Registry as a Living Document

The registry is `docs/standards/bounded-contexts.md`. Every BC appearing in any crate's
`[package.metadata.oya].bounded_context` MUST appear in that document. Adding a BC is a
**0-ADR action**.

---

## Follow-ups

1. **Shard 1**: rewrite `docs/standards/crate-naming-convention.md` to reflect BNF v4.1.
2. **Shard 1**: update `docs/standards/code-style-rust.md` lines 11–12, 137–147, 162–177.
3. **Shard 1**: author `docs/standards/bounded-contexts.md` full version.
4. **Post-Shard-1**: flip `oya-check-architecture` + 3 sibling LEAN check crates from `--report-only` to BLOCKER.
5. **M02**: author and deploy `oya-check-statelessness`, `oya-check-shardability`, `oya-check-perf-budget`, `oya-check-benchmark` (per ADR-0062).

---

## References

- `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §2, §4a, §11
- ADR-0015 — flat crates layout
- ADR-0017 — `oya-` prefix
- ADR-0054 — scaffold-claim pattern
- ADR-0057 — cutover mechanics
- ADR-0058 — Flat microservice catalog (Product Groups retired)
- ADR-0062 — Quality/Performance/Scalability bar (PRD + impl-plan gates)
- `[[feedback-flat-product-catalog]]` §"BNF resolution" — slot2 `shared|vertical` retired
- Robert C. Martin — *Clean Architecture* ch. 22
- Eric Evans — *Domain-Driven Design* ch. 5–6
