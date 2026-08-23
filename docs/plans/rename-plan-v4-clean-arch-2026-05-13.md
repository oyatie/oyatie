---
doc_class: HowTo
shape: ~
length_cap: 1200
authority_tier: 3
status: approved
bnf_version: v4.1
execution: approved-by-user-2026-05-13
iteration: 2
consensus_loop: v4-iter-5-approve-fold
bnf_amendment: v4.1-2026-05-13 (BC optional; flat catalog; drop shared|vertical binary; microservice = slot2 open kebab)
last_modified: 2026-05-13
architect_iter_1: 7-conditions-CLOSED (B1–B7 per §15)
critic_iter_1: ITERATE-7 (folded; C1–C7 per §15 closure block; consistency fixes 8–13 per §15a)
critic_iter_2: ITERATE-7 (folded; D1–D7 per §15b closure block)
critic_iter_3: ITERATE-5 (folded; E1–E5 per §15c closure block — actual body-row regeneration of §3.1–§3.5, §3.6 arithmetic 140 base, frontmatter purpose 3-slot rewrite, stale A4 heading rename, open-questions honest claim correction)
critic_iter_4: ITERATE-4 (folded; F1–F4 per §15d closure block — §3.3.2 11-col regeneration with check-namespace exemption, §3.3/§3.3.1/§3.6 arithmetic mechanical fix, STUB/PROTOCOL-UNKNOWN actual-count sync, ADR-0056 outline 3-slot rewrite + Protocol classification sub-section authored)
critic_iter_5: APPROVE-WITH-CONDITIONS (3 conditions, folded; G1–G3 per §15e closure block — §3.6 summary-table cell sync + PROTOCOL-UNKNOWN narrative for rows 60/72/73/74, active `2-slot` references purge to history-only, check-crate-name normalization to 4-LEAN design)
prefold_a_state: 3-open-items-CLOSED (transitive-cross-vertical refusal + verticals deprecation lifecycle + reviewer-hours 4-stream re-sync) + cloud-dual-role-public_layers-mechanism added
postfold_a_state: 7-codex-iter2-execution-consistency-edits-CLOSED (D1 audit-row 3-slot rewrite + D2 §1/§3.6 arithmetic sync + D3 §3.0 metadata schema thing-cleanup + D4 LEAN-A2 transitive-walker explicit gate + D5 BNF vertical-single-token policy + D6 code-style-rust.md inventory addition + D7 open-questions iter-2 refresh + stale check-crate name fix at §6 R10)
iter3_fold_state: 5-codex-iter3-execution-consistency-edits-CLOSED (E1 §3.1–§3.5 body rows REGENERATED to 11-col 3-slot tuples with STUB markers; zero `rest (provisional)` cells remain in audit; E2 §3.6 arithmetic fixed to "140 + 4 = 144" matching §1; E3 frontmatter purpose rewritten to 3-slot grammar; E4 stale A4 §4a heading renamed singular→plural; E5 open-questions honest-claim correction + iter-3 closure section appended)
fold_state: 12-layer-canonical + 4-lean-check-codification + 3-slot-BNF `oyatie-<shared|vertical>-<bc>-<layer>` (single-token verticals, Option A per ADR-0056 §"Vertical naming policy") + verticals-as-open-kebab-registry-with-active/deprecated/retired-lifecycle + public_layers-cross-vertical-exemption mechanism
architect: opus
critic: codex-gpt-5.5-xhigh
supersedes: docs/plans/rename-plan-v3-2026-05-12.md
date: 2026-05-13
purpose: |
  Execution plan v4.1 for the 140-crate workspace cutover (amended 2026-05-13).
  BNF v4.1: `oyatie-<microservice>[-<bc>]-<layer>` — microservice is slot2 open
  kebab (no shared|vertical binary; everything is shared per flat catalog);
  BC slot is OPTIONAL (omit when microservice has a single concept at the layer).
  Check crates remain `check-<rule-name>` flat namespace (BNF-exempt).
  Atomic rename = old crate name GONE on disk; no aliases, no dead code.
  Carries forward Hybrid C topology (Shard 0 + Shard 1 atomic rename),
  xtask-metadata-augment (with `lockfile-rename --bnf-version v4.1` flag),
  4-layer branch pipeline, scripted-rewrite + `--locked --offline` lockfile
  primitive, and deterministic §8.1 acceptance gates.
  Drops: shared|vertical binary slot, verticals registry, cross-vertical
  refusal (LEAN-A2 simplifies; microservice isolation replaces vertical-kind
  enforcement), fitness terminology, freeze-window-kernel lane.
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/standards/git-workflow.md
  - docs/standards/testing.md
  - docs/audits/convention-audit-2026-05-12.md
  - docs/plans/rename-plan-v3-2026-05-12.md
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0054
  - ADR-0056
  - ADR-0057
doc_status: published
---

# Rename Plan v4.1 — Clean Architecture / Flat Catalog BNF (2026-05-13, Hybrid C: Shard 0 + atomic Shard 1)

> **Supersedes** [`rename-plan-v3-2026-05-12.md`](rename-plan-v3-2026-05-12.md).
> v3 reached consensus-approval but a user pressure-test exposed three
> over-engineered layers that did not earn their keep:
>
> 1. The verbose `oyatie-<context>-<feature>-<capability>-<role>` BNF (4–5 segments)
>    produced names like `governance-architecture-conventions-kernel`.
> 2. The `governance-freeze-window-kernel` lane primitive duplicated
> 3. "Fitness" terminology, imported wholesale from *Building Evolutionary
>    Architectures* jargon, never settled into the team's vocabulary and
>    repeatedly produced 6-segment AMBER crates.
>
> v3 was consensus-approved against its own BNF; v4 is **not an iteration of
> v3 — it is a replacement**. v3 transitions to `status: Superseded`,
> `superseded_by: docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` in the
> same Shard 0 commit that authors this plan.
>
> **Hybrid C topology**, the **xtask-metadata-augment Rust crate** (with
> `lockfile-rename` subcommand and the 8-row + 20-cell fixture matrices),
> the **4-layer branch pipeline** (worktree → local-dev → origin/dev →
> staging → prod), the **scripted-rewrite + `cargo check --workspace --locked
> --offline` lockfile primitive**, the **48 h coordinated freeze + Hybrid-C-Lite
> escape hatch**, and the **deterministic §8 acceptance gates** all
> port forward from v3 unchanged. The lane-config/freeze-window/expedite-token
> ADR-0054) holds an exclusive symbol lock for the duration of Shard 1, which
> is what the 48 h window was approximating in software.

---

## RALPLAN-DR Summary (architect/critic alignment payload, step 2)

**Mode**: SHORT consensus loop default. **DELIBERATE** auto-enabled here
because the cutover touches all 140 workspace crates simultaneously and is
classified high-risk by both the user pressure-test (rationale for replacing
v3) and §6 R3 (re-audited blast-radius row).

### Principles (5, ordered)

1. **Clean Architecture self-enforces via Cargo.** Closed `layer` enum +
   open `bounded-context` token. The dependency graph compiles iff the
   layer ordering holds; no runtime fitness function needed.
2. **Closed where bounded, open where evolving.** Layer enum is closed
   (**12 canonical values**: 4 inner / pure — kernel, domain,
   application, app — + 2 outer / external — adapter, infrastructure —
   + 6 presentation/entry-point — cli, rest, grpc, graphql, worker,
   sdk). Each value carries a single canonical meaning per Uncle Bob's
   *Clean Architecture* + DDD tactical patterns; aliases / overlaps
   are forbidden. Bounded context is an open kebab-token (no closed
   registry, no AMBER carve-outs, no compound-features ceremony).
   Adding a bounded context is a 0-ADR action; adding a layer is a
   1-ADR action. Each presentation layer names a wire-format/protocol
   explicitly (no ambiguous `api` token). Each crate's layer is
   assigned by the canonical decision tree (§2.2.4); reviewers cannot
   disagree on layer assignment.
3. **Hyperscaler convergence.** AWS smithy-rs, Azure SDK for Rust, and Google
   Cloud Rust all encode the layer in the crate name (`azure_storage_blob`,
   `google-cloud-pubsub`, `aws-smithy-runtime`); none encode the *evolutionary
   architecture fitness function family*. v4 imports their convention.
4. **Mechanical change cost is one-time; cognitive load is forever.**
   v3's 37 renames + 31 compound-feature ADR rows + the closed enum + the
   AMBER carve-out for `governance-architecture-conventions-kernel`
   pay an ongoing readability tax. v4's higher one-time rename count (~50–90)
   buys a permanently simpler grammar.
5. **Checks are cross-cutting, not layered.** A clean-architecture check
   crate is itself a `cli` plus optional library — but it does not "belong
   to" a layer of the system it inspects. v4 places checks in a flat
   `check-<rule-name>` namespace, outside the layered architecture, so
   they can never accidentally pretend to be domain or application code.

### Decision Drivers (top 3)

1. **The v3 BNF cannot parse the load-bearing CLI.** v3 row 36 forced
   `tooling-agent-read` → `tooling-agent-cli-read` because the BNF
   required a role token. v4's `oyatie-<bounded-context>-<layer>` parses
   `codeview-cli` cleanly: bounded context = `codeview` (a domain noun);
   layer = `cli`.
2. **`fitness` is jargon, not vocabulary.** Every fitness-as-feature crate
   in v3 (rows 2–34) was an ADR-citation, an architectural check, a
   doc-coverage probe, or a supply-chain audit. None was a "fitness
   function" in any sense the team used informally. v4 collapses them all
   under a flat `check-<name>` namespace, where the noun (`check`)
   matches the team's actual vocabulary.
   `governance-freeze-window-kernel` + `expedite_override_token`
   the duration of Shard 1's atomic squash-merge. The fitness lane was
   a parallel implementation of a primitive the workspace already has.

### Viable Options (≥ 2)

**Option C (CHOSEN) — Clean Architecture canonical BNF + 12-value
canonical layer enum + canonical decision tree + flat check namespace.**
- Pros: closed 12-value layer enum (4 inner / pure: kernel, domain,
  application, app; 2 outer / external: adapter, infrastructure; 6
  presentation: cli, rest, grpc, graphql, worker, sdk) gives Cargo +
  cargo-metadata enough information to enforce dependency direction at
  compile/CI time AND names each protocol/wire-format directly (no
  ambiguous `api` token); each layer has a canonical meaning per Uncle
  Bob + DDD with NO aliases or overlaps; canonical decision tree
  (§2.2.4) makes layer assignment deterministic so reviewers cannot
  disagree; bounded contexts grow without ADR overhead; checks live in
  a namespace that never collides with product code; reduces total
  grammar tokens vs. v3 by ~50 %; matches hyperscaler precedent.
- Cons: higher one-time rename count (~139 vs. v3's 37); bounded-context
  names are not centrally registered, so two teams could disagree on
  what the bounded context is for a new feature (mitigated per R10);
  every existing `*-api` crate requires one-time protocol audit AND
  every existing `*-kernel` crate requires `src/`-inspection to confirm
  pure-types-vs-business-logic classification (mitigated by Codex iter-1
  §10 question 1 dedicated to the canonical-decision-tree audit).

**Option D — Status quo (do nothing; keep v3 BNF + lane primitives).**
- Pros: v3 is consensus-approved; no further reviewer cost.
- Cons: the three over-engineered layers persist; new crates inherit the
  6-segment AMBER tax; `tooling-agent-read` rename remains awkward;
  every "fitness" crate enforces a name the team does not use informally.

**Option E — Thing-domain literal (rejected; the `<thing>` slot was removed in v4 iteration sequence and the final v4 BNF settled at 3-slot `oyatie-<shared|vertical>-<bounded-context>-<layer>`).**
- Pros: explicit `oyatie-<bounded-context>-<thing>-<layer>` always, no optional
  slot, easier to parse.
- Cons: forces a `<thing>` token where none semantically exists (e.g.
  `codeview-tool-cli` to satisfy the slot); pessimises the common case;
  no hyperscaler analogue (Azure ships `azure_storage_blob` not
  `azure_storage_blob_object`). Per third correction, the entire `<thing>`
  slot is removed; granularity expressed via multi-token BC names.

**Option F — Drop-verb pattern (rejected, BNF-equivalent but loses common case).**
- Pros: shortest grammar.
- Cons: collapses every crate to 3 segments; cannot disambiguate a domain
  layer from an infrastructure layer of the same bounded context (e.g.
  `policy` could mean either `policy-domain` or `policy-api`).

**Decision**: Option C. Options D, E, F are explicitly rejected per the
drivers above; no single-viable-option invalidation rationale needed.

### Pre-mortem (3 scenarios, DELIBERATE-mode requirement)

1. **Scenario A — Bounded-context names drift across team members.**
   Within 6 months, two teams pick incompatible names for the same domain
   (`audit` vs. `audit-chain`, `eventing` vs. `events`). Probability: M.
   Impact: H (the registry is the team's vocabulary; drift forks vocabulary).
   **Mitigation (per §6 R10)**: ADR-0056 §"Bounded context registry as a
   living document" requires every new bounded context to ship with a
   1-paragraph rationale in `docs/standards/bounded-contexts.md` (new
   doc) and to be cross-referenced from the originating crate's
   `[package.metadata.oya].bounded_context` field. A bounded context that
   appears in zero crates after 90 days is auto-deprecated.

2. **Scenario B — Row 35-equivalent has higher than estimated blast radius.**
   v3 row 35 (`platform-data-boundary-kernel`, 95 consumers) is renamed
   under v4 to `data-boundary-domain`. The xtask re-audit may surface a
   different crate as the new highest-blast-radius row (e.g.,
   `platform-eventing-kernel` may have a higher path-edge count once
   re-keyed by bounded context). Probability: M. Impact: H.
   **Mitigation (per §6 R3)**: Shard 0 step 15a produces the rename map
   AND emits a `cargo metadata` reverse-dep count for every renamed crate.
   The §8.1 reverse-dep gate enforces a per-rename consumer-count
   assertion derived from the pre-rename snapshot. Whichever crate ends up
   with the highest count gets the row-35-equivalent treatment (95-or-N
   manifest scan + named §3 docs/code co-edit + risk = 5).

3. **Scenario C — Layer name conflicts with an existing Rust convention.**
   `domain` is a Rust ecosystem-neutral term, but `application` is
   sometimes reserved for binary crates with a `main.rs` entry point.
   Cargo itself does not reserve `application`. However, IDE tooling
   (rust-analyzer, IntelliJ-Rust) may have heuristics that key on it.
   Probability: L. Impact: M.
   **Mitigation**: pre-cutover smoke test on a `policy-application`
   skeleton in Shard 0; rust-analyzer + IntelliJ-Rust + VS Code rust
   extensions are all exercised in §8.1 IDE smoke gate (new gate). If any
   fails, escape hatch is to use `app` (shorter, v3-compatible). Layer
   enum has formal authority; the IDE heuristic must yield.

### Expanded test plan (DELIBERATE-mode requirement)

- **Unit (per-crate)**: `cargo nextest run -p <crate>` for each of 140
  crates. Acceptance: all PASS. Run in CI matrix; failure flags the
  per-crate rewrite as buggy.
- **Integration (workspace)**: `cargo nextest run --workspace --all-features
  --no-fail-fast --message-format libtest-json + junit`. Acceptance: zero
  failures. Per §8.1 "Tests pass" gate.
- **End-to-end (staging cutover smoke test)**: After Shard 1 merges to
  `staging` (via the 4-layer pipeline: worktree → local-dev → origin/dev →
  staging), exercise the renamed `codeview-cli` + `dev-cli` against
  a synthetic repo; assert all hardcoded `cargo run -p tooling-*`
  invocations in CI workflows + scripts have been rewritten and resolve
  correctly. New §8.1 gate row "E2E staging smoke".
- **Observability (post-cutover, 7-day rolling)**: emit `cargo doc
  --workspace --no-deps` graph + `cargo metadata --no-deps | jq` snapshot
  daily for 7 days post-Shard-1; new fitness-equivalent check
  `check-rename-baseline-reset` (renamed from v3's
  `governance-baseline-reset-kernel`) computes a daily delta and
  zero unexpected deltas. Counts as the §8.2 "Impossible-to-fail score
  over 7 days" gate.

---

## §0 Frontmatter (declared above)

## §1 Scope summary table

| Item | Count | Notes |
|---|---:|---|
| Total workspace crates audited | **140** | `Cargo.toml [workspace] members` array, verified `wc -l Cargo.toml` partition |
| Estimated renames + new check crates = total crate-name ops | **~144 ops** (140 existing crates per Cargo.toml lines 3-142 + 4 new check crates per §4a; corrects v4-iter-1 claims of 145/151) | All 140 workspace crates are renamed under v4 (every crate either changes layer suffix or drops the deprecated context prefix). Plus 4 new check crates scaffolded fresh (`check-architecture`, `check-bounded-contexts`, `check-supply-chain`, `check-semver`). Final count produced by §3 audit + Codex iter-2 `src/`-inspection; xtask consumes `/tmp/rename-map.tsv`. Under the 12-value canonical layer enum, some `*-kernel` crates that turn out to be PURE types + ports may stay `kernel` (no layer change); some may relayer to `domain` if they carry business logic. |
| `[package.metadata.oya]` block additions | **140** | Schema simplified vs. v3 §3.1: drops `feature`, `capability`, `compound`; keeps `bounded_context`, `layer`, `audit_chain`. See §3.0 |
| Dep-edge rewrites | **~200–400** (toml_edit handles all forms per §3.3.1 v3 matrix, carried forward) | Estimate, not gate; final count emerges from `cargo metadata --no-deps` diff in Shard 0 step 15a |
| New `check-*` crates (LEAN per iter-2 fold; collapsed from 11 → 4 per Codex iter-1 ITERATE-7 edit C1: "too verbose") | **4** | `check-architecture` (orchestrator: subcommands for layer-correctness + dependency-direction + naming-collision + metadata-schema + lockfile-parity + lib-name-parity + check-namespace — all 7 inner checks consolidated as xtask subcommands), `check-bounded-contexts` (BC registry validation + overlap governance + shared/vertical-kind dependency enforcement per supplement), `check-supply-chain` (cargo deny wrapper), `check-semver` (cargo-semver-checks rename-baseline-reset classifier). Scaffolded empty in Shard 0; populated in Shard 1; flipped from `--report-only` to BLOCKER in §8.2 follow-up (per B6 chicken-and-egg avoidance). See §4a "4 lean check crates" for full per-crate spec. |
| Bounded-context kind taxonomy + verticals registry (FINAL per iter-2 fold supplement #2 — 3-slot BNF; supersedes the metadata-only kind taxonomy and the 5-axis enum) | **`shared` literal + open verticals registry** | The slot-2 token in `oyatie-<shared\|vertical>-<bc>-<layer>` is either the literal `shared` OR an open kebab vertical name registered in `[workspace.metadata.oyatie.verticals]`. Initial verticals: `cloud` (owner: council-cloud), `foundry` (owner: council-foundry), `workspace` (owner: council-workspace). Future verticals: `healthcare`, `corporate`, etc., added by registry append + ADR cite. `shared` BCs depend only on other `shared` BCs. `<vertical>` BCs depend on `shared` BCs + same-vertical BCs only. Cross-vertical deps refused by `check-bounded-contexts`. xtask refuses crate names whose slot 2 is neither `shared` nor a registered vertical. |
| Cargo.lock churn events | **1** | Hybrid C atomic Shard 1 ⇒ single lockfile regen via `xtask-metadata-augment lockfile-rename` |
| Bounded contexts identified | **~100 entries** (post-3rd-correction expansion) | Initial draft enumerates the v4-draft-4 ~72 single-token BCs PLUS the ~28 multi-token BCs that absorbed the dropped `thing` slot per third correction. Single-token BCs: `cell`, `region`, `compute`, `iam`, `billing`, `capacity`, `finops`, `marketplace`, `dcops`, `kms`, `storage`, `surface`, `network`, `observability`, `audit-chain`, `eventing`, `object-graph`, `policy-cedar`, `residency`, `regulatory-pack`, `secrets`, `tenant`, `identity`, `metering`, `dsr`, `data-boundary`, `foundry`, `codeview`, `dev`, `composition`, plus 23 workspace product axes. Multi-token BCs (new under 3-slot, populating slot 3): `compute-vm`, `compute-k8s`, `compute-functions`, `storage-object`, `storage-block`, `network-vpc`, `network-dns`, `network-lb`, `audit-chain-file`, `eventing-file`, `observability-tracing`, `secrets-file`, `foundry-evidence-file`, `foundry-run-file`, `foundry-step-file`, `billing-tax`, plus foundry sub-contexts (`foundry-adapter`, `foundry-bypass`, `foundry-capability`, `foundry-catalog`, `foundry-cloud-mutation`, `foundry-evidence`, `foundry-eval`, `foundry-mcp-gateway`, `foundry-policy`, `foundry-rag`, `foundry-registry`, `foundry-run`, `foundry-step`, `foundry-api-semver`, `foundry-mdbook`, `foundry-openapi`, `foundry-cargo-prefix`). Plus the flat `check` namespace. Each entry registered with A4 fields (`name`, `owner`, `rationale`, `adr_cite`). |
| CI workflow updates | **3 files** (carry forward from v3 §1) | Same files as v3; the cargo run package targets change per the new rename map |
| Scripts updates | **~30 sites across 3 files** (carry forward from v3 §1) | `scripts/check.sh`, `scripts/hooks/pre-push-repoctl.sh`, `scripts/check-architecture-boundaries.sh` |
| Registry references | **3 files** (carry forward from v3 §1) | `registry/quality/lanes.yaml`, `registry/docs/pipeline.tsv`, OpenAPI bindings under `registry/openapi/` |
| Standards doc co-edits | **2 files + 1 new** | `docs/standards/clean-architecture.md` §3 (named-by-identity row updated); `docs/standards/crate-naming-convention.md` (rewritten or superseded by ADR-0056); new `docs/standards/bounded-contexts.md` (the living registry per Scenario A mitigation) |
| New ADRs | **2** | ADR-0056 (the BNF + bounded-context registry policy); ADR-0057 (supersedes ADR-0055; drops fitness/freeze/expedite) |
| Dependency cycles introduced | **0** (gate per §8.1 R9) | Clean architecture forbids by construction; gate enforces `dep direction monotonically respects layer enum ordering` |

## §2 BNF + layer enum + check namespace formal definition

### 2.1 Canonical BNF (v4.1 — amended 2026-05-13)

> **BNF v4.1 amendment**: the `shared|vertical` binary slot is retired.
> Everything is shared in the flat microservice catalog. Slot2 is now the
> microservice name (open kebab). BC slot is optional. ADR-0056 must be
> amended to v4.1 per [[feedback-flat-product-catalog]].

```bnf
crate          ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
                 | "oya" "-" "check" "-" rule-name
microservice   ::= kebab-token ( "-" kebab-token )*    (* 1..3 tokens; registered in [workspace.metadata.oyatie.microservices] *)
bc-tokens      ::= kebab-token ( "-" kebab-token )*    (* 0..N; OPTIONAL — omit when microservice has single concept at the layer *)
layer          ::= "kernel" | "domain" | "application" | "app"
                 | "adapter" | "infrastructure"
                 | "cli" | "rest" | "grpc" | "graphql"
                 | "worker" | "sdk"
rule-name      ::= kebab-token ( "-" kebab-token )*    (* 1..4 tokens; open *)
kebab-token    ::= [a-z] [a-z0-9]*
```

**BC optionality rule**: omit BC when the microservice has a single
binary or single concept at that layer (e.g., `medical-domain`,
`tenancy-kernel`, `cloud-cli`). Include BC when the microservice
has multiple binaries or multiple BC-level splits at the same layer (e.g.,
`workflow-state-machine-domain`, `workflow-approvals-application`).

**FINAL BNF v4.1**: each crate name encodes
`oyatie-<microservice>[-<bc>]-<layer>`. The microservice slot is an open
kebab registered in `[workspace.metadata.oyatie.microservices]`. There is
no `shared|vertical` binary — every feature/product is a microservice
in the flat catalog.

Parser rule: split crate name on `-`; LAST token MUST be a layer value
(one of 12 canonical); SECOND token (after `oyatie-`) MUST be `shared` OR
a registered vertical name from the workspace verticals registry;
remaining middle tokens (joined by `-`) = bounded-context. The `check-*`
namespace is exempt — checks are cross-cutting and use the
`check-<rule-name>` shape.

Canonical examples (replace draft-5 examples):
- `shared-audit-chain-domain` — shared BC `audit-chain`, layer `domain`
- `shared-eventing-application` — shared BC `eventing`, layer `application`
- `shared-codeview-cli` — shared BC `codeview` (formerly v3 `tooling-agent-read`), layer `cli`
- `shared-composition-app` — shared BC `composition` (formerly v3 `foundation-app`), layer `app`
- `intelligence-policy-evaluator-cedar-domain` — vertical `foundry`, BC `policy-evaluator-cedar`, layer `domain`
- `cloud-compute-vm-rest` — vertical `cloud`, BC `compute-vm`, layer `rest`
- `cloud-storage-object-adapter-aws` — vertical `cloud`, BC `storage-object-adapter-aws` (multi-token BC; AWS-specific adapter), layer (none in this example — actually parses as BC `storage-object`, layer `adapter`, with `aws` as additional BC qualifier; see audit §3 for canonical resolution under the 12-layer enum: ports go in `kernel`, trait impls in `adapter`, framework wrappers in `infrastructure`)
- `workspace-drive-domain` — vertical `workspace`, BC `drive`, layer `domain`
- `workspace-chat-rest` — vertical `workspace`, BC `chat`, layer `rest`
- `healthcare-patient-charting-domain` — future vertical `healthcare`, BC `patient-charting`, layer `domain`

**Critical correction vs. v4-draft-1**: the layer enum has **12 closed
values**, NOT 6. v4-draft-1's `api` layer was ambiguous (REST? gRPC?
GraphQL?) — hyperscaler precedent names the protocol directly. Each
presentation/wire-format gets its own layer value.

**Critical correction vs. v4-draft-2 / draft-3**: layer enum is
finalised at **12 distinct canonical values**, each with a unique
canonical meaning per Uncle Bob's *Clean Architecture* + DDD tactical
patterns. The 12-value enum supersedes the earlier 10-value (and
9-value) drafts. Each crate occupies exactly ONE layer; the decision
tree in §2.2.4 governs assignment.

Constraints:

1. **Segment count.** Total segments (counting `oya` as segment 1) MUST be
   `>=3`; there is NO upper bound (open-ended via multi-token BC names).
   Long names are encouraged when granularity demands them: e.g.,
   `audit-chain-emission-domain` parses as `BC=audit-chain-emission,
   layer=domain` — 5 segments, no AMBER tax, no special handling. The
   grammar treats every kebab token between `oyatie-` and the trailing
   layer token as part of the bounded-context slot.
2. **Layer enum (closed, 12 values).** Adding a layer is a 1-ADR action.
   Layer enum is the ONLY closed set in the grammar. The 12 values
   partition into **4 inner / pure layers** (kernel, domain, application,
   app) + **2 outer / external layers** (adapter, infrastructure) +
   **6 presentation/entry-point layers** (cli, rest, grpc, graphql,
   worker, sdk). Each value carries a CANONICAL meaning; aliases /
   overlaps are forbidden.
3. **Bounded-context slot (open, sole granularity expressor).** Adding a
   bounded context is a 0-ADR action — the author writes the name, the
   xtask records it in `[workspace.metadata.oya].bounded_contexts`
   (auto-generated from `[package.metadata.oya].bounded_context` fields
   per Shard 1's xtask `--apply` run). The registry IS the workspace
   state, not a hand-curated list. ADR-0056 §"Bounded context registry
   as a living document" documents this. BCs may be 1-N kebab tokens;
   granularity is expressed by making the BC name more specific (e.g.
   `policy` → `policy-evaluator` → `policy-evaluator-cedar`). Multi-token
   BCs are first-class citizens; the registry tracks the full kebab
   string as the canonical BC name. Per third correction, this slot
   REPLACES the previously-considered `<thing>` slot (a separate slot
   was removed because granularity-via-BC-name is simpler and matches
   the Rust workspace pattern of `tokio-util`/`tonic-build`).
4. **Check namespace (flat).** `check-<rule-name>` does NOT carry a
   layer suffix. Checks are cross-cutting; a check crate may itself
   contain a `cli` entry point and a small library, but it is not "part
   of" the layered architecture of the workspace it inspects.

### 2.2 Layer semantics (closed enum, 12 canonical values)

Each value carries a single canonical meaning per Uncle Bob's *Clean
Architecture* (ch. 22 "The Clean Architecture") + DDD tactical patterns
(Evans, *Domain-Driven Design*, ch. 5-6 "Layered Architecture" + ch. 14
"Maintaining Model Integrity"). The 12 values partition into 3 groups:
4 inner / pure layers + 2 outer / external layers + 6 presentation /
entry-point layers.

#### 2.2.1 Inner / pure layers (innermost-out, 4 values)

| Layer | Definition | Allowed internal deps | Surface |
|---|---|---|---|
| `kernel` | **Pure types + ports (traits) only.** ZERO business logic. The "shared kernel" in DDD; the "core contracts" in some Rust hexagonal templates. Defines what the system IS, shape-wise. Value objects, identity types, error enums, port trait declarations. | nothing project-internal | library only |
| `domain` | **Business logic on top of kernel:** entities, domain services, business rules, invariant enforcement. Pure, no I/O, no async, no provider deps. | `kernel` only (or nothing if self-contained) | library only |
| `application` | **Use cases / application services** that orchestrate domain to fulfill user intents. Holds port-trait bounds, not concrete adapters. | `domain` + `kernel` | library only |
| `app` | **Composition root binary** that wires application + adapter + infrastructure + presentation into a runnable service. Usually one `app` per deployable service. The ONLY layer that knows about every other layer simultaneously. | every other layer (composition root has unrestricted internal deps) | bin (with optional thin library shim) |

**Concrete worked example** (canonical pattern):

- `policy-kernel` — `Policy`, `Decision`, `PolicyId` value objects +
  `PolicyRepository` trait (port).
- `policy-domain` — `fn evaluate_policy(policy: &Policy, ctx:
  &Context) -> Decision` business logic operating on kernel types.
- `policy-application` — `CreatePolicyUseCase`,
  `EvaluatePolicyUseCase` orchestrators that hold `PolicyRepository`
  trait bounds.
- `policy-app` — the policy service binary running rest + grpc +
  worker wired with postgres adapters via DI.

#### 2.2.2 Outer / external layers (2 values)

| Layer | Definition | Allowed internal deps | Surface |
|---|---|---|---|
| `adapter` | **Interface adapters** (Uncle Bob's "Interface Adapters" ring): trait implementations of `kernel` ports + DTO mappers. Load-bearing classification cue: the crate's primary public surface is `impl <SomeTrait> for <SomeStruct>` blocks. | `application` + `domain` + `kernel` | library only |
| `infrastructure` | **Frameworks & drivers** (Uncle Bob's "Frameworks & Drivers" ring): framework glue, driver wrappers, runtime utilities. NOT trait impls (those go in `adapter`). Load-bearing classification cue: the crate's primary public surface is framework wiring (axum router builders, opentelemetry exporters, tokio runtime helpers) without a trait impl as the headline. | `application` + `domain` + `kernel`; may use `adapter` for non-trait glue that itself depends on trait impls | library only |

**Concrete worked example** (canonical pattern):

- `policy-adapter-postgres` — `impl PolicyRepository for
  PostgresClient` + `Policy ↔ row` mapping.
- `policy-infrastructure-tracing` — OpenTelemetry exporter wiring;
  not bound to a port trait.
- `policy-infrastructure-pool` — postgres pool setup helpers not
  tied to a trait impl.

#### 2.2.3 Presentation / entry-point layers (per protocol, 6 values)

| Layer | Definition | Wire format / protocol | Allowed internal deps | Surface |
|---|---|---|---|---|
| `cli` | CLI binary or CLI library (subcommand handlers + optional `[[bin]]`) | command-line args + stdout/stderr | `application` + `domain` + `kernel` | bin (with optional thin library shim) |
| `rest` | HTTP REST API handlers + axum-style routing | HTTP/1.1 + JSON (typically axum) | `application` + `domain` + `kernel` | bin (with optional thin library shim) |
| `grpc` | gRPC service definitions + tonic handlers | HTTP/2 + Protobuf (typically tonic) | `application` + `domain` + `kernel` | bin (with optional thin library shim) |
| `graphql` | GraphQL schema + resolvers | HTTP/1.1 + JSON GraphQL (typically async-graphql) | `application` + `domain` + `kernel` | bin (with optional thin library shim) |
| `worker` | Long-running background workers: queue consumers, pubsub workers, scheduled tasks | AMQP/Kafka/Pub-Sub/cron message dispatch | `application` + `domain` + `kernel` | bin (with optional thin library shim) |
| `sdk` | Client libraries for external consumers of oyatie services (pure types + traits) | language-native API surface | `kernel` only | library only |

**Why `api` is dropped from v4-draft-1**: `api` does not name a wire
format. v3 had multiple `-api` crates that were actually REST, gRPC,
or GraphQL. Per hyperscaler precedent (`azure_storage_blob` names the
protocol/product; `google-cloud-pubsub` names the service), each
protocol gets its own layer value.

**Why `kernel` and `domain` are BOTH preserved**: per DDD's distinction
between the "shared kernel" (pure types + ports, no logic) and the
"domain layer" (entities with invariants + domain services with rules).
The v3 inventory has many `*-kernel` crates that are actually
`*-domain` crates by Uncle Bob's strict reading (they carry business
logic, not just types). The §3 audit MUST inspect each `*-kernel` crate
and relayer it to `kernel` (pure types/ports) or `domain` (logic) per
the canonical decision tree.

**Why `app` is added as a distinct layer**: composition-root binaries
(the "main" of a deployable service) are architecturally distinct from
either `application` (use-case orchestrators, no DI) or any single
presentation layer (a service often runs rest + grpc + worker
simultaneously). Naming it `app` (not `application` or `runtime`)
preserves Uncle Bob's terminology: the `app` IS the deployable
"application" binary; the `application` layer is the use-case
orchestration code that the `app` wires up. This dichotomy was
implicit in v3 (`foundation-app` was the composition root) and is
made explicit in v4.

#### 2.2.4 Canonical decision tree (per-crate audit rule)

For each crate, ask in order; the first matching answer fixes the layer.
A crate CANNOT occupy two layers. If a crate currently does multiple
things (e.g. a `*-adapter` that also has framework glue, or a `*-kernel`
that carries business logic), SPLIT it OR document as an Exception in
§3 audit with rationale.

1. Pure types + traits (ports), no logic? → `kernel`
2. Business logic on entities + domain services (operating on kernel types)? → `domain`
3. Use cases orchestrating domain via port trait bounds? → `application`
4. Composition root binary wiring multiple layers into a deployable? → `app`
5. Trait implementation (impl of a `kernel` port)? → `adapter`
6. Framework / driver glue without being a trait impl? → `infrastructure`
7. CLI subcommands or CLI binary? → `cli`
8. REST API handlers? → `rest`
9. gRPC service? → `grpc`
10. GraphQL resolvers? → `graphql`
11. Background worker (queue / scheduled / pubsub consumer)? → `worker`
12. Pure client library for external consumers? → `sdk`

**Audit implication** for §3: a v3 crate currently named
`intelligence-policy-kernel` might actually be a `domain` crate if it
has business logic; the audit must inspect `src/` to classify
correctly. This may flip layer assignments for some crates relative to
v3 names. The audit is the source of truth; v3 naming is advisory only.

#### 2.2.5 Dependency direction (12-value rule matrix)

```
kernel ◀── domain ◀── application ◀── { adapter, infrastructure, cli, rest, grpc, graphql, worker }
                                       ◀── adapter ◀── infrastructure (optional, infrastructure may use adapter glue)
                                                                       ◀── app (composition root has unrestricted inward deps)
kernel ◀── sdk
```

Planned advisory check: `check-architecture` (LEAN-A1 per §4a) per §5 and by
`cargo-metadata`-driven workspace lints. Edges allowed:

- `kernel` → nothing internal
- `domain` → `kernel` only
- `application` → `domain` + `kernel`
- `adapter` → `application` + `domain` + `kernel`
- `infrastructure` → `application` + `domain` + `kernel`; MAY also
  depend on `adapter` (infrastructure-as-glue may compose trait-impl
  adapters with framework bindings)
- `cli` / `rest` / `grpc` / `graphql` / `worker` → `application` +
  `domain` + `kernel` (entry points orchestrate use cases; if an entry
  point needs to instantiate a concrete adapter, that wiring belongs in
  the `app` composition root, not in the entry-point crate itself)
- `app` → every other layer (composition root has unrestricted internal
  deps; this is the canonical exception)
- `sdk` → `kernel` only (pure client types + traits)

`adapter`, `infrastructure`, and every presentation layer are
*application's outbound surface*; they are allowed to depend on
`application` traits — but the application MUST NOT depend on any of
them. The `check-architecture` crate (LEAN-A1) refuses any edge from
`application` to `adapter` / `infrastructure` / presentation; from
`domain` to anything except `kernel`; from `kernel` to anything; from
`sdk` to anything except `kernel`; from any presentation layer to
`adapter` / `infrastructure` directly (presentation must go through
`application`, except in `app` composition roots).

### 2.3 Check namespace formal definition

```bnf
check-crate     ::= "oya" "-" "check" "-" rule-name
rule-name       ::= kebab-token ( "-" kebab-token )*   (* 1..4 tokens *)
```

A check crate:
- MAY be a library crate (lints, custom clippy, helpers)
- MAY be a binary crate (CLI runner invokable as `cargo run -p check-<name>`)
- MUST NOT depend on any `application` or `infrastructure` crate (checks
  are runtime-independent — they inspect manifests, ASTs, or external
  artefacts, never the running system)
- MAY depend on any `domain` crate (e.g., a check that verifies an
  invariant on `data-boundary-domain` types may import that domain
  for `proptest` shapes)

The set of `check-*` crates IS the workspace's enforcement surface.
v4 ships six in Shard 0 (scaffolded empty) + Shard 1 (populated); the
team adds more over time as new checks emerge.

> **Check-namespace duality (clarification):** Two crate forms coexist by design:
> 1. **LEAN check binaries** (4 crates, 3-slot BNF, `cli` layer):
>    `shared-architecture-check-cli`, `shared-bounded-contexts-check-cli`,
>    `shared-supply-chain-check-cli`, `shared-semver-check-cli`.
>    These are the toolchain executables that *run* checks. Justification:
>    slot2 = `shared` (cross-vertical toolchain), slot3 = check subject domain,
>    slot4 = `cli` (12-enum value; presentation layer per ADR-0056).
> 2. **Per-rule check crates** (29 crates, BNF-exempt flat namespace):
>    `check-<rule-name>` per ADR-0056 line 79-80. These are the
>    individual rule implementations that the LEAN binaries discover/consume.
>    Exemption claim: ADR-0056 BNF second production `crate ::= ... | "oya" "-"
>    "check" "-" rule-name`.
> Both forms are canonical. They are not interchangeable.

### 2.4 Bounded-context registry (living document)

ADR-0056 §"Bounded context registry as a living document" establishes:

- The registry is `docs/standards/bounded-contexts.md` (new doc, authored
  in Shard 1).
- Every bounded context appearing in any crate's
  `[package.metadata.oya].bounded_context` MUST also appear in the doc
  with a 1-paragraph rationale.
- The xtask `--check` mode refuses any crate whose `bounded_context`
  field is not in the doc.
- A bounded context that appears in zero crates after 90 days is auto-
  deprecated (a 1-line note in the doc); the xtask warns but does not
  fail on stale entries — drift is a doc-hygiene concern, not a
  compile-blocker.

This satisfies Scenario A mitigation. The registry is a doc (Markdown),
not a TOML field, so the team can read it as prose without parsing.

## §3 Per-crate audit (re-classification of 140 crates against v4 BNF)

**Columns** (FINAL post-Codex-iter-2 D1; 3-slot grammar; `thing?` column DROPPED; `vertical` + `kind` + `layer_evidence` + `bc_registry_status` columns ADDED):

The audit-table header row (used in §3.1–§3.5) is now:

```
| # | current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected |
```

Column semantics:
- `current_name` — v3 crate name (unchanged from prior drafts).
- `vertical` — slot 2 of the v4 3-slot name: literal `shared` OR a
  registered single-token vertical name (`cloud`, `foundry`, `workspace`,
  plus future verticals per ADR-0056 §"Vertical naming policy").
- `bounded_context` — kebab; may be 1-N tokens; populates slot 3 of
  the v4 name; matches `[package.metadata.oya].bounded_context`.
- `kind` — `shared` or `vertical`; redundant with the `vertical` column
  for verification (kind == "shared" iff vertical == "shared").
- `layer` — one of 12 canonical values per §2.2.
- `layer_evidence` — file:line cite (e.g., `crates/intelligence-policy-api/src/main.rs:42 — Router::new()`) OR `cargo metadata` query result OR explicit `PROTOCOL-UNKNOWN, deferred to ADR-0056 §<X>` deferral marker. **NO row may ship as `provisional`** post-iter-3.
- `proposed_name` — 3-slot pattern `oyatie-<shared|vertical>-<bc>-<layer>`.
- `bc_registry_status` — `REGISTERED` | `PROPOSED-NEW` | `DEPRECATED`.
- `risk` — 1-5.
- `dep_edges_affected` — estimated edges touched by the rename.

> **Schema-vs-body status (D1 closure)**: The §3.1–§3.5 audit-table
> headers + alignment rows have been REWRITTEN to this 3-slot column
> schema. The per-row BODY cells were authored under the iter-1
> 9-column 2-slot schema (`bounded_context | thing? | layer |
> proposed_name | Y/N | risk | dep_edges_affected`) and currently carry
> stub data using the v3 axis → v4 vertical translation rule below.
> **Codex iter-3 inspection pass populates each row's `layer_evidence`
> + `bc_registry_status` + 3-slot `proposed_name`** per `src/`-inspection
> + verticals-registry-status; the row may not ship "provisional" or
> "rest (provisional)" — every row gets either an evidence cite OR an
> explicit `PROTOCOL-UNKNOWN` deferral marker. This is the iter-3
> open-item #1 from `.omc/plans/open-questions.md`.

**v3 axis → v4.1 translation rule** (BNF v4.1 amendment applied row-by-row):
- `platform-<bc>-<layer>` → `oyatie-<bc>-<layer>` (drop slot2 entirely; BC promoted to slot2)
- `foundation-<bc>-<layer>` → `oyatie-<bc>-<layer>` (same rule; foundation prefix dropped)
- `tooling-<bc>-<layer>` → `oyatie-<bc>-<layer>` (same rule; tooling prefix dropped)
- `foundry-<bc>-<layer>` → unchanged (foundry is the µservice name)
- `cloud-<bc>-<layer>` → unchanged (cloud is the µservice name)
- `workspace-<bc>-<layer>` → `connect-<bc>-<layer>` (workspace renamed to connect per Round 4 decision [[feedback-flat-product-catalog]])
- `shared-<bc>-<layer>` → `oyatie-<bc>-<layer>` (drop redundant shared prefix)
- `check-<rule>` → unchanged (BNF-exempt)

> **Atomic rename rule**: old crate name is DELETED from disk. No aliases.
> No compatibility shims. After Shard 1, `platform-*` directory does
> not exist; only `oyatie-<bc>-*` exists. Old `Cargo.toml` package names
> are gone from `Cargo.lock` (verified by `lockfile-parity` gate).

Examples of v4.1 `proposed_name` after translation:
- `platform-tenant-kernel` → `tenancy-kernel`
  (BC promoted to slot2; domain noun `tenancy` per ADR-0125)
- `platform-identity-kernel` → `identity-kernel`
- `platform-audit-chain-kernel` → `audit-chain-kernel`
- `intelligence-policy-api` → `intelligence-policy-rest`
- `cloud-storage-object-api` → `cloud-storage-object-rest`
- `foundation-app` → `application-app` (B2B shell µservice)
- `tooling-agent-read` → `codeview-cli`
- `workspace-mail-kernel` → `mail-domain`
- `workspace-chat-api` → `connect-chat-rest`

> **Audit table format note (superseded by D1)**: the iter-1-fold-A
> "thing-slot 2-slot column-schema rework" §5.1 step 15c is now
> SUPERSEDED by this D1 schema declaration. Shard 0 step 15c remains
> in the §5.1 checklist for the body-cell xtask-rebuild execution; the
> SCHEMA itself is now declared here as authoritative.

> **Table format note (superseded by Codex iter-3 edit 1)**: the
> §3.1–§3.5 audit tables below have been regenerated row-by-row under
> the 3-slot column schema in this iter-3 pass. Body cells now carry
> the 11-column tuple `current_name | vertical | bounded_context | kind
> | layer | layer_evidence | proposed_name | bc_registry_status | risk
> | dep_edges_affected`. Cells without `src/`-inspection evidence carry
> the explicit marker `STUB-pending-iter-4-src-inspection` (per Codex
> iter-3 edit 1: no bare `provisional` cells remain). The §5.1 step 15c
> xtask-rebuild execution remains, but its scope reduces to the
> `STUB-pending-iter-4-src-inspection` rows whose evidence cite or
> protocol classification is still open.

**Risk key**: 1 = no internal consumers; 2 = ≤ 5 consumers; 3 = 6–20
consumers; 4 = 21–80 consumers; 5 = > 80 consumers (row-35-equivalent).
Final risk numbers produced by Shard 0 step 15a's `cargo metadata` reverse-
dep count; initial assignments below are based on v3 §2 known consumer
counts.

**Rename required**: Y if proposed_name != current_name. N if grammar
already conforms (no change). `dep_edges_affected` is an estimate; the
xtask produces the final count.

**Presentation-layer protocol classification (post-correction)**: Every
existing `*-api` crate is provisionally classified as `rest` (the
workspace's current HTTP+JSON surface) BUT Codex iter-1 audit MUST
confirm protocol per crate. If a crate currently serves gRPC, it
becomes `oyatie-<context>-grpc`; GraphQL → `oyatie-<context>-graphql`;
multi-protocol → split into per-protocol crates OR documented as
exception in ADR-0056 §"Bounded context registry". The default-to-`rest`
classification is a planner-best-guess; it is one of the top-3 expected
Codex pressure-test surfaces (§10).

**Inner-layer classification under the 12-value canonical enum
(provisional defaults, Codex iter-1 must verify by inspecting `src/`)**:

Per the canonical decision tree (§2.2.4), every existing `*-kernel` and
`*-app` crate must be re-classified by ACTUAL code shape. Provisional
audit defaults applied below pending Codex iter-1 `src/`-inspection:

- **v3 `*-kernel` → v4 default `domain`**: most v3 kernels carry
  business logic per `docs/audits/convention-audit-2026-05-12.md`
  rationale, so they are domain crates under the 12-value enum.
  EXCEPTIONS expected (pure types + ports only → `kernel` under v4):
  `platform-data-boundary-kernel`, possibly several
  `foundry-*-kernel` crates that are pure check-rule type bundles.
  These exceptions get re-classified to `oyatie-<bc>-kernel` (v4) during
  Codex iter-1 audit; the row stays in §3 with `domain → kernel`
  override note.
- **v3 `*-app` → v4 default `application`**: most v3 app crates are
  use-case orchestrators (not composition-root binaries). EXCEPTION:
  `foundation-app` IS the composition root binary (it wires
  `intelligence-api` + `tooling-cli-dev-runtime` + downstream
  consumers per `docs/standards/clean-architecture.md`); under the
  12-value enum it is `app`, not `application`. Row 138 reflects this.
- **v3 `*-adapter-<provider>` → v4 `adapter`**: trait-impl crates stay
  in the `adapter` layer per the canonical taxonomy (rows 15, 18, 22,
  28, 67, 76, 78 — already updated above).
- **v3 `*-api` → v4 `rest` / `grpc` / `graphql` / `worker`**: see
  presentation-layer classification note above.
- **No v3 crate currently classifies to v4 `infrastructure` or v4
  `sdk`**: the workspace does not currently have framework-glue-only
  crates or SDK-publish crates. New `*-infrastructure-*` and `*-sdk`
  crates will emerge organically post-Shard-1 as the team writes them.

### 3.1 Platform / shared µservice crates (n = 28) — BNF v4.1: drop `platform-` prefix, BC becomes slot2

> **v4.1 rule**: `platform-<bc>-<layer>` → `oyatie-<bc>-<layer>`. Old
> directory `crates/platform-<bc>-<layer>/` is DELETED and replaced
> by `crates/oyatie-<bc>-<layer>/`. No alias. No compatibility shim.
> Object Graph crates renamed to Ontology per [[feedback-glossary-ontology-not-object-graph]].

| # | current_name | microservice | bounded_context | layer | layer_evidence | proposed_name | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|:-:|--:|
| 1 | `platform-data-boundary-kernel` | `data-boundary` | — | `kernel` | pure types + ports (named-by-identity per clean-architecture.md §3; only kernel allowed cross-layer deps) | `data-boundary-kernel` | **5** | ~95 |
| 2 | `platform-residency-kernel` | `residency` | — | `domain` | `STUB-pending-src-inspection` (v3 kernel with business logic) | `residency-domain` | 3 | est. 10–20 |
| 3 | `platform-dsr-kernel` | `dsr` | — | `domain` | `STUB-pending-src-inspection` | `dsr-domain` | 2 | est. 5–10 |
| 4 | `platform-dsr-app` | `dsr` | — | `application` | `STUB-pending-src-inspection` | `dsr-application` | 2 | est. 3–5 |
| 5 | `platform-tenant-kernel` | `tenancy` | — | `domain` | `STUB-pending-src-inspection` | `tenancy-domain` | 4 | est. 30–50 |
| 6 | `platform-tenant-api` | `tenancy` | — | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST control-plane likely; gRPC event stream possible) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 3 | est. 5–10 |
| 7 | `platform-identity-kernel` | `identity` | — | `domain` | `STUB-pending-src-inspection` | `identity-domain` | 4 | est. 30–50 |
| 8 | `platform-identity-api` | `identity` | — | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST + OIDC typical; gRPC mTLS plausible) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 3 | est. 5–10 |
| 9 | `platform-identity-app` | `identity` | — | `application` | `STUB-pending-src-inspection` | `identity-application` | 3 | est. 5–10 |
| 10 | `platform-metering-kernel` | `metering` | — | `domain` | `STUB-pending-src-inspection` | `metering-domain` | 3 | est. 10–20 |
| 11 | `platform-metering-app` | `metering` | — | `application` | `STUB-pending-src-inspection` | `metering-application` | 2 | est. 3–5 |
| 12 | `platform-cell-kernel` | `cell` | — | `domain` | `STUB-pending-src-inspection` | `cell-domain` | 3 | est. 10–20 |
| 13 | `platform-audit-chain-kernel` | `audit-chain` | — | `domain` | `STUB-pending-src-inspection` | `audit-chain-domain` | 4 | est. 30–50 |
| 14 | `platform-audit-chain-app` | `audit-chain` | — | `application` | `STUB-pending-src-inspection` | `audit-chain-application` | 3 | est. 5–10 |
| 15 | `platform-audit-chain-adapter-file` | `audit-chain` | `file` | `adapter` | trait impl + DTO mapping; classified `adapter` | `audit-chain-file-adapter` | 2 | est. 3–5 |
| 16 | `platform-eventing-kernel` | `eventing` | — | `domain` | `STUB-pending-src-inspection` | `eventing-domain` | 4 | est. 30–50 |
| 17 | `platform-eventing-app` | `eventing` | — | `application` | `STUB-pending-src-inspection` | `eventing-application` | 3 | est. 5–10 |
| 18 | `platform-eventing-adapter-file` | `eventing` | `file` | `adapter` | `STUB-pending-src-inspection` | `eventing-file-adapter` | 2 | est. 3–5 |
| 19 | `platform-object-graph-kernel` | `ontology` | — | `domain` | `STUB-pending-src-inspection` (renamed object-graph → ontology per [[feedback-glossary-ontology-not-object-graph]]) | `ontology-domain` | 3 | est. 10–20 |
| 20 | `platform-object-graph-api` | `ontology` | — | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (GraphQL plausible given typed-entity semantics) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 21 | `platform-observability-kernel` | `observability` | — | `domain` | `STUB-pending-src-inspection` | `observability-domain` | 4 | est. 30–50 |
| 22 | `platform-observability-adapter-tracing` | `observability` | `tracing` | `adapter` | `STUB-pending-src-inspection` | `observability-tracing-adapter` | 2 | est. 3–5 |
| 23 | `platform-policy-cedar-kernel` | `policy` | `cedar` | `domain` | `STUB-pending-src-inspection` | `policy-cedar-domain` | 3 | est. 10–20 |
| 24 | `platform-policy-cedar-api` | `policy` | `cedar` | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST request/response typical) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 25 | `platform-regional-pack-kernel` | `regional-pack` | — | `domain` | `STUB-pending-src-inspection` | `regional-pack-domain` | 2 | est. 5–10 |
| 26 | `platform-regulatory-pack-api` | `regulatory-pack` | — | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST typical) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 27 | `platform-secrets-kernel` | `secrets` | — | `domain` | `STUB-pending-src-inspection` | `secrets-domain` | 3 | est. 10–20 |
| 28 | `platform-secrets-adapter-file` | `secrets` | `file` | `adapter` | `STUB-pending-src-inspection` | `secrets-file-adapter` | 2 | est. 3–5 |

> Note: rows 13–28 actually total 16 (audit-chain trio + eventing trio +
> object-graph pair + observability pair + policy-cedar pair + regional/
> regulatory pair + secrets pair). The header "n = 18" counted the first
> twelve simple platform crates; the audit-chain through secrets cluster
> brings the platform total to 28 rows. Final count: **28 platform crates**.

### 3.2 Cloud context crates (n = 31)

| # | current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|---|---|:-:|--:|
| 29 | `cloud-cell-app` | `cloud` | `cell` | `vertical` | `application` | `STUB-pending-iter-4-src-inspection` | `cloud-cell-application` | PROPOSED-NEW | 2 | est. 5–10 |
| 30 | `cloud-resource-kernel` | `cloud` | `resource` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-resource-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 31 | `cloud-region-kernel` | `cloud` | `region` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-region-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 32 | `cloud-region-api` | `cloud` | `region` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (cloud control-plane region API typically REST + AWS-SDK-style) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 33 | `cloud-compute-kernel` | `cloud` | `compute` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-compute-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 34 | `cloud-compute-vm-api` | `cloud` | `compute-vm` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (VM CRUD typically REST; vm-event streams could be gRPC) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 35 | `cloud-compute-k8s-api` | `cloud` | `compute-k8s` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (k8s API surface is REST + WATCH streams; reasonable to split rest+worker per cluster-event consumer) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 36 | `cloud-compute-functions-api` | `cloud` | `compute-functions` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (FaaS control-plane typically REST) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 37 | `cloud-iam-kernel` | `cloud` | `iam` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-iam-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 38 | `cloud-iam-api` | `cloud` | `iam` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (IAM control-plane typically REST; sigv4 auth) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 39 | `cloud-billing-kernel` | `cloud` | `billing` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-billing-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 40 | `cloud-billing-app` | `cloud` | `billing` | `vertical` | `application` | `STUB-pending-iter-4-src-inspection` | `cloud-billing-application` | PROPOSED-NEW | 2 | est. 5–10 |
| 41 | `cloud-billing-tax-app` | `cloud` | `billing-tax` | `vertical` | `application` | `STUB-pending-iter-4-src-inspection` | `cloud-billing-tax-application` | PROPOSED-NEW | 2 | est. 3–5 |
| 42 | `cloud-capacity-kernel` | `cloud` | `capacity` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-capacity-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 43 | `cloud-finops-kernel` | `cloud` | `finops` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-finops-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 44 | `cloud-finops-api` | `cloud` | `finops` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (finops dashboards typically REST + JSON) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 45 | `cloud-marketplace-kernel` | `cloud` | `marketplace` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-marketplace-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 46 | `cloud-dcops-kernel` | `cloud` | `dcops` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-dcops-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 47 | `cloud-data-kernel` | `cloud` | `data` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` (BC = `data` inside `cloud` vertical disambiguates from `shared/data-boundary`) | `cloud-data-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 48 | `cloud-kms-kernel` | `cloud` | `kms` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-kms-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 49 | `cloud-kms-api` | `cloud` | `kms` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (KMS REST cipher ops typical; AWS KMS precedent) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 50 | `cloud-storage-kernel` | `cloud` | `storage` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-storage-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 51 | `cloud-storage-object-api` | `cloud` | `storage-object` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (S3-style REST + sigv4 typical) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 52 | `cloud-storage-block-api` | `cloud` | `storage-block` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (EBS-style REST control + iSCSI data-plane; control-plane likely REST) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 53 | `cloud-surface-kernel` | `cloud` | `surface` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-surface-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 54 | `cloud-network-kernel` | `cloud` | `network` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `cloud-network-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 55 | `cloud-network-vpc-api` | `cloud` | `network-vpc` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (VPC control-plane typically REST) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 56 | `cloud-network-dns-api` | `cloud` | `network-dns` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (DNS-over-HTTPS REST control; resolver data-plane is DNS-over-TLS/UDP) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 57 | `cloud-network-lb-api` | `cloud` | `network-lb` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (LB control-plane typically REST) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 58 | `cloud-observability-kernel` | `cloud` | `observability` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` (BC = `observability` inside `cloud` vertical disambiguates from `shared/observability` at row 21) | `cloud-observability-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 59 | `cloud-observability-api` | `cloud` | `observability` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (OTLP ingestion is typically gRPC; control surface REST; may require split per supplement-2 multi-protocol exception) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |

> Note: rows 47, 58, 59 use compound bounded contexts `cloud-data` and
> `cloud-observability` because both `data` (cf. row 1's `data-boundary`)
> and `observability` (cf. platform observability rows 21–22) are already
> used by `platform`. Clean Architecture allows disambiguation by
> compound bounded-context tokens; this is the live-registry policy
> (§2.4) handling collision via prefix.

### 3.3 Foundry context crates (n = 52; Codex iter-4 F2 fix — 23 non-check + 29 check = 52, was incorrectly stated as 53)

The foundry context is the workspace's engineering-platform plane: ADR
emission, audit/coverage probes, capability registry, evidence store, MCP
gateway, eval/replay, claim-ceiling enforcement, etc. Under v4, the
foundry's **non-check** crates remain in bounded context `foundry`; the
foundry's **check** crates (every "fitness" crate from v3 rows 2–34) move
to the flat `check` namespace.

#### 3.3.1 Foundry non-check crates (the engineering platform itself; n = 23; Codex iter-4 F2 fix — rows 60-82 are 23 rows, not 22)

| # | current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|---|---|:-:|--:|
| 60 | `intelligence-api` | `foundry` | `meta` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (foundry meta-surface; aggregator API; likely REST but iter-4 must confirm. BC = `meta` disambiguates from per-feature foundry-* BCs) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 3 | est. 5–10 |
| 61 | `intelligence-adapter-kernel` | `foundry` | `adapter` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` (note: BC is `adapter` as a domain noun — this crate is the foundry's pluggable-adapter framework, not itself a `adapter` layer crate) | `intelligence-adapter-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 63 | `intelligence-capability-kernel` | `foundry` | `capability` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-capability-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 64 | `intelligence-catalog-kernel` | `foundry` | `catalog` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-catalog-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 65 | `intelligence-cloud-mutation-kernel` | `foundry` | `cloud-mutation` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-cloud-mutation-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 66 | `intelligence-evidence-kernel` | `foundry` | `evidence` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-evidence-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 67 | `intelligence-evidence-adapter-file` | `foundry` | `evidence-file` | `vertical` | `adapter` | `STUB-pending-iter-4-src-inspection` (v3 `*-adapter-file` = trait impl; classified `adapter`) | `intelligence-evidence-file-adapter` | PROPOSED-NEW | 2 | est. 3–5 |
| 68 | `intelligence-eval-kernel` | `foundry` | `eval` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `governance-eval-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 69 | `intelligence-eval-app` | `foundry` | `eval` | `vertical` | `application` | `STUB-pending-iter-4-src-inspection` | `intelligence-eval-application` | PROPOSED-NEW | 2 | est. 5–10 |
| 70 | `intelligence-mcp-gateway-kernel` | `foundry` | `mcp-gateway` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-mcp-gateway-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 71 | `intelligence-policy-kernel` | `foundry` | `policy` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-policy-domain` | PROPOSED-NEW | 3 | est. 10–20 |
| 72 | `intelligence-policy-api` | `foundry` | `policy` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (foundry policy decision API; likely REST request/response) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 73 | `intelligence-registry-api` | `foundry` | `registry` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (capability registry CRUD; likely REST) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 74 | `intelligence-rag-api` | `foundry` | `rag` | `vertical` | `PROTOCOL-UNKNOWN` | `pending-iter-4-protocol-inspection` (retrieval-augmented generation; streaming retrieval suggests gRPC or SSE-over-HTTP; multi-protocol candidate) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | PROPOSED-NEW | 2 | est. 5–10 |
| 75 | `intelligence-run-kernel` | `foundry` | `run` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-run-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 76 | `intelligence-run-adapter-file` | `foundry` | `run-file` | `vertical` | `adapter` | `STUB-pending-iter-4-src-inspection` | `intelligence-run-file-adapter` | PROPOSED-NEW | 2 | est. 3–5 |
| 77 | `intelligence-step-kernel` | `foundry` | `step` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-step-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 78 | `intelligence-step-adapter-file` | `foundry` | `step-file` | `vertical` | `adapter` | `STUB-pending-iter-4-src-inspection` | `intelligence-step-file-adapter` | PROPOSED-NEW | 2 | est. 3–5 |
| 79 | `intelligence-api-semver-kernel` | `foundry` | `api-semver` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-api-semver-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 80 | `intelligence-mdbook-kernel` | `foundry` | `mdbook` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-mdbook-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 81 | `intelligence-openapi-kernel` | `foundry` | `openapi` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-openapi-domain` | PROPOSED-NEW | 2 | est. 5–10 |
| 82 | `governance-cargo-prefix-kernel` | `foundry` | `cargo-prefix` | `vertical` | `domain` | `STUB-pending-iter-4-src-inspection` | `intelligence-cargo-prefix-domain` | PROPOSED-NEW | 2 | est. 5–10 |

> Conformant rows (60, 72, 73, 74) need only metadata-block additions, not
> renames.

#### 3.3.2 Foundry check crates (every v3 "fitness" crate; n = 29, all rename to flat `check-<rule-name>` namespace per §2.1 BNF "check-crate" production; Codex iter-4 F1 — regenerated to 11-column 3-slot-equivalent schema with check-namespace exemption documented inline)

> **Check-namespace exemption (per §2.1 BNF)**: rows 83-111 use the
> flat `check-<rule-name>` namespace, which is the second
> production in the v4 BNF (`crate ::= ... | "oya" "-" "check" "-"
> rule-name`). These crates are EXEMPT from the standard 3-slot
> grammar — they do NOT carry a `vertical`/`bounded-context`/`layer`
> triple in the name; `rule-name` is the sole non-`check-` slot.
> The 11-column audit schema fills these rows with documented exemption
> markers (`vertical: check-namespace-exempt`, `bounded_context:
> check-namespace-exempt`, `kind: check-namespace-exempt`,
> `layer: check-namespace-exempt`) so the audit table format is
> uniform across §3.1–§3.5; LEAN-A6 (`check-architecture --
> check-namespace`) enforces the actual flat-shape regex
> `^check_[a-z][a-z0-9_]*$`.

| # | current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|---|---|:-:|--:|
| 83 | `governance-adr-citation-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `adr-citation`) | `check-adr-citation` | PROPOSED-NEW | 2 | est. 5 |
| 84 | `governance-adr-index-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `adr-index`) | `check-adr-index` | PROPOSED-NEW | 2 | est. 5 |
| 85 | `governance-authority-cohesion-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `authority-cohesion`) | `check-authority-cohesion` | PROPOSED-NEW | 2 | est. 5 |
| 86 | `governance-brand-residue-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `brand-residue`) | `check-brand-residue` | PROPOSED-NEW | 2 | est. 5 |
| 87 | `governance-claim-ceiling-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `claim-ceiling`) | `check-claim-ceiling` | PROPOSED-NEW | 2 | est. 5 |
| 89 | `governance-cohesion-fitness-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `cohesion`) | `check-cohesion` | PROPOSED-NEW | 2 | est. 5 |
| 90 | `governance-constitution-cite-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | ~~`NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `constitution-cite`)~~ SUNSET 2026-05-15 | ~~`check-constitution-cite`~~ SUNSET — crate deleted in commit `526e4bf` (strike: retire docs/CONSTITUTION.md and its enforcement crate) | SUNSET | n/a | n/a |
| 91 | `intelligence-cost-budget-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `cost-budget`) | `check-cost-budget` | PROPOSED-NEW | 2 | est. 5 |
| 92 | `governance-data-class-fitness-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `data-class`) | `check-data-class` | PROPOSED-NEW | 2 | est. 5 |
| 93 | `governance-doc-catalog-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `doc-catalog`) | `check-doc-catalog` | PROPOSED-NEW | 2 | est. 5 |
| 94 | `governance-documentation-system-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `documentation-system`) | `check-documentation-system` | PROPOSED-NEW | 2 | est. 5 |
| 95 | `governance-glossary-coverage-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `glossary-coverage`) | `check-glossary-coverage` | PROPOSED-NEW | 2 | est. 5 |
| 96 | `governance-glossary-vocabulary-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `glossary-vocabulary`) | `check-glossary-vocabulary` | PROPOSED-NEW | 2 | est. 5 |
| 97 | `governance-license-policy-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `license-policy`) | `check-license-policy` | PROPOSED-NEW | 2 | est. 5 |
| 98 | `intelligence-mobile-native-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `mobile-native`) | `check-mobile-native` | PROPOSED-NEW | 2 | est. 5 |
| 99 | `governance-placeholder-debt-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `placeholder-debt`) | `check-placeholder-debt` | PROPOSED-NEW | 2 | est. 5 |
| 101 | `governance-pre-push-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `pre-push`) | `check-pre-push` | PROPOSED-NEW | 2 | est. 5 |
| 102 | `governance-quality-lane-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `quality-lane`) | `check-quality-lane` | PROPOSED-NEW | 2 | est. 5 |
| 103 | `governance-raci-team-coverage-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `raci-coverage`) | `check-raci-coverage` | PROPOSED-NEW | 2 | est. 5 |
| 104 | `governance-readme-doc-coverage-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `readme-coverage`) | `check-readme-coverage` | PROPOSED-NEW | 2 | est. 5 |
| 105 | `intelligence-release-evidence-pack-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `release-pack`) | `check-release-pack` | PROPOSED-NEW | 3 | est. 10–20 |
| 106 | `governance-runbook-freshness-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `runbook-freshness`) | `check-runbook-freshness` | PROPOSED-NEW | 2 | est. 5 |
| 107 | `governance-runbook-index-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `runbook-index`) | `check-runbook-index` | PROPOSED-NEW | 2 | est. 5 |
| 108 | `governance-slo-coverage-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `slo-coverage`) | `check-slo-coverage` | PROPOSED-NEW | 2 | est. 5 |
| 109 | `governance-supply-chain-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `supply-chain`) | `check-supply-chain` | PROPOSED-NEW | 3 | est. 10–20 |
| 110 | `governance-typescript-workspace-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `typescript-workspace`) | `check-typescript-workspace` | PROPOSED-NEW | 2 | est. 5 |
| 111 | `governance-vendor-contract-recency-kernel` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `check-namespace-exempt` | `NEW-scaffold-shard-1-from-v3-fitness-crate` (rule-name `vendor-recency`) | `check-vendor-recency` | PROPOSED-NEW | 2 | est. 5 |

> Note: 29 check crates here (rows 83–111). Post-iter-2 LEAN collapse
> (per §4a iter-2-fold-B), the workspace ships **4 LEAN check crates**
> scaffolded fresh in Shard 0 (not renamed from existing crates):
> `check-architecture` (LEAN-A1; 7 subcommands consolidating
> layer-correctness + dependency-direction + naming-collision +
> metadata-schema + lockfile-parity + lib-name-parity + check-namespace),
> `check-bounded-contexts` (LEAN-A2; BC registry + cross-vertical
> refusal + overlap governance), `check-supply-chain` (LEAN-A3;
> cargo-deny wrapper), `check-semver` (LEAN-A4; cargo-semver-checks
> rename-baseline-reset classifier). The iter-1-fold-A 6-check list
> (`check-clean-architecture` + 5 siblings) and the iter-1-fold-A
> 11-check expansion are both SUPERSEDED — see §4a LEAN-A1–LEAN-A4 for
> current canonical and §15a/§15b for the journey history.

### 3.4 µservice crates (formerly workspace; n = 26) — BNF v4.1: `workspace-*` → `connect-*`

> **v4.1 rule**: workspace renamed to connect per Round 4 session decision
> [[feedback-flat-product-catalog]]. Old `crates/workspace-<bc>-<layer>/`
> directory is DELETED; replaced by `crates/connect-<bc>-<layer>/`.
> No alias. covers dual-context: Professional (B2B) + Personal (B2C)
> per Bominal ADR-0208.

| # | current_name | microservice | bounded_context | layer | layer_evidence | proposed_name | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|:-:|--:|
| 112 | `workspace-address-book-kernel` | `connector` | `address-book` | `domain` | `STUB-pending-src-inspection` | `address-book-domain` | 2 | est. 5–10 |
| 113 | `workspace-calendar-kernel` | `connector` | `calendar` | `domain` | `STUB-pending-src-inspection` | `calendar-domain` | 2 | est. 5–10 |
| 114 | `workspace-chat-kernel` | `connector` | `messenger` | `domain` | `STUB-pending-src-inspection` (chat → messenger per ADR-0208 dual-context nomenclature) | `messenger-domain` | 2 | est. 5–10 |
| 115 | `workspace-chat-api` | `connector` | `messenger` | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (WebSocket / GraphQL subscriptions plausible) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 116 | `workspace-collab-runtime-kernel` | `connector` | `collab-runtime` | `domain` | `STUB-pending-src-inspection` | `collab-runtime-domain` | 2 | est. 5–10 |
| 117 | `workspace-document-format-kernel` | `connector` | `document-format` | `domain` | `STUB-pending-src-inspection` | `document-format-domain` | 2 | est. 5–10 |
| 118 | `workspace-dlp-kernel` | `connector` | `dlp` | `domain` | `STUB-pending-src-inspection` | `dlp-domain` | 2 | est. 5–10 |
| 119 | `workspace-ediscovery-kernel` | `connector` | `ediscovery` | `domain` | `STUB-pending-src-inspection` | `ediscovery-domain` | 2 | est. 5–10 |
| 120 | `workspace-docs-kernel` | `connector` | `docs` | `domain` | `STUB-pending-src-inspection` | `docs-domain` | 2 | est. 5–10 |
| 121 | `workspace-drive-kernel` | `connector` | `drive` | `domain` | `STUB-pending-src-inspection` | `drive-domain` | 2 | est. 5–10 |
| 122 | `workspace-drive-api` | `connector` | `drive` | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST typical; GraphQL possible for UI surface) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 123 | `retention-dsr-kernel` | `connector` | `dsr` | `domain` | `STUB-pending-src-inspection` | `dsr-domain` | 2 | est. 5–10 |
| 124 | `workspace-forms-kernel` | `connector` | `forms` | `domain` | `STUB-pending-src-inspection` | `forms-domain` | 2 | est. 5–10 |
| 125 | `workspace-forms-api` | `connector` | `forms` | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST typical) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 126 | `workspace-mail-kernel` | `connector` | `mail` | `domain` | `STUB-pending-src-inspection` | `mail-domain` | 2 | est. 5–10 |
| 127 | `workspace-meet-kernel` | `connector` | `meet` | `domain` | `STUB-pending-src-inspection` | `meet-domain` | 2 | est. 5–10 |
| 128 | `workspace-meet-api` | `connector` | `meet` | `PROTOCOL-UNKNOWN` | `pending-protocol-inspection` (REST signaling + WebRTC data-plane split candidate) | `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` | 2 | est. 5–10 |
| 129 | `workspace-notes-kernel` | `connector` | `notes` | `domain` | `STUB-pending-src-inspection` | `notes-domain` | 2 | est. 5–10 |
| 130 | `workspace-recordings-kernel` | `connector` | `recordings` | `domain` | `STUB-pending-src-inspection` | `recordings-domain` | 2 | est. 5–10 |
| 131 | `workspace-retention-kernel` | `connector` | `retention` | `domain` | `STUB-pending-src-inspection` | `retention-domain` | 2 | est. 5–10 |
| 132 | `workspace-sheets-kernel` | `connector` | `sheets` | `domain` | `STUB-pending-src-inspection` | `sheets-domain` | 2 | est. 5–10 |
| 133 | `workspace-sites-kernel` | `connector` | `sites` | `domain` | `STUB-pending-src-inspection` | `sites-domain` | 2 | est. 5–10 |
| 134 | `workspace-slides-kernel` | `connector` | `slides` | `domain` | `STUB-pending-src-inspection` | `slides-domain` | 2 | est. 5–10 |
| 135 | `workspace-tasks-kernel` | `connector` | `tasks` | `domain` | `STUB-pending-src-inspection` | `tasks-domain` | 2 | est. 5–10 |
| 136 | `workspace-translate-kernel` | `connector` | `translate` | `domain` | `STUB-pending-src-inspection` | `translate-domain` | 2 | est. 5–10 |
| 137 | `workspace-trust-portal-kernel` | `connector` | `trust-portal` | `domain` | `STUB-pending-src-inspection` | `trust-portal-domain` | 2 | est. 5–10 |

> Note: row 123 (`retention-dsr-kernel`) keeps the `workspace-`
> prefix because row 3 already claims `dsr` for the platform-DSR
> bounded context; the workspace-axis DSR uses compound bounded context
> `retention-dsr`. 26 product-axis crates (rows 112–137); plus the
> additional foundation/tooling crates (rows 138–140) total exactly 140.

### 3.5 Foundation + tooling crates (n = 3) — BNF v4.1: drop `foundation-`/`tooling-` prefix

> **v4.1 rule**: `foundation-<bc>-<layer>` → `oyatie-<bc>-<layer>`;
> `tooling-<bc>-<layer>` → `oyatie-<bc>-<layer>`. Old directories
> DELETED. No alias. `foundation-app` becomes `application-app`
> (the B2B Application shell µservice per [[feedback-flat-product-catalog]]).

| # | current_name | microservice | bounded_context | layer | layer_evidence | proposed_name | risk | dep_edges_affected |
|--:|---|---|---|---|---|---|:-:|--:|
| 138 | `foundation-app` | `application` | — | `app` | composition-root binary per decision tree §2.2.4 step 4; wires all other layers | `application-app` | 3 | est. 10–20 |
| 139 | `tooling-cli-dev-runtime` | `dev` | — | `cli` | hosts `oya` + `repoctl` bins; touches 3 CI workflows + ~30 script sites + test fixtures | `dev-cli` | **5** | est. 30+ |
| 140 | `tooling-agent-read` | `codeview` | — | `cli` | sanctioned-primitive READ slot per git-workflow.md §2-3; every agent/script invoking `tooling-agent-read` needs update | `codeview-cli` | 3 | est. 10–20 |

> Note on row 140: v4.1 reads `tooling-agent-read` as: microservice =
> `codeview` (domain noun for the agent's read-only code view), layer = `cli`.
> Under v4.1 BNF the BC slot is omitted (single concept at cli layer).
> Old crate dir `crates/tooling-agent-read/` deleted; replaced by
> `crates/codeview-cli/`.

### 3.6 Audit summary — BNF v4.1 (amended 2026-05-13)

| Group | Crates | Rename required | Old dir deleted |
|---|---:|---:|---|
| Platform µservices (drop `platform-` prefix; BC becomes slot2; Ontology rename) | 28 | 28 | `crates/platform-*/` ALL deleted |
| Cloud µservice (unchanged; foundry is µservice name) | 31 | 31 | dirs renamed per new name |
| Foundry non-check µservice (unchanged; foundry is µservice name) | 23 | 23 | dirs renamed per new name |
| Foundry check crates (move to flat `check-*` namespace) | 29 | 29 | `crates/foundry-*-kernel/` (check subset) deleted |
| µservice (formerly workspace; ALL `workspace-*` → `connect-*`) | 26 | 26 | `crates/workspace-*/` ALL deleted |
| Foundation + tooling (drop prefix; `foundation-app` → `application-app`; `tooling-*` → µservice names) | 3 | 3 | `crates/foundation-*/` + `crates/tooling-*/` deleted |
| **Subtotal — existing crates renamed** | **140** | **140** | **140 old dirs gone** |
| **NEW — 4 LEAN check crates scaffolded fresh** (`check-architecture`, `check-bounded-contexts`, `check-supply-chain`, `check-semver`) | **+4** | n/a (new scaffolds) | n/a |
| **Total crate-name-affecting ops** | **144** | **140 + 4 new = 144** | — |

> **Atomic rename = old name GONE**: after Shard 1 merges, zero crate
> directories matching `platform-*`, `workspace-*`, `foundation-*`,
> `tooling-*`, `shared-*` (the old `shared` prefix from v4 iter-1–5
> interim names) may exist. Verified by §8.1 "Zero old-names" and
> "Cargo.lock zero old-names" gates. No alias crates. No re-export shims.

> Audit cross-check (Codex iter-4 F2 mechanical-arithmetic fix): v3
> Cargo.toml lists **exactly 140 workspace members** (Cargo.toml lines
> 3-142). Per-group subtotals: Platform 28 (rows 1-28) + Cloud 31
> (rows 29-59) + Foundry non-check **23** (rows 60-82) + Foundry check
> 29 (rows 83-111) + Workspace 26 (rows 112-137) + Foundation+tooling
> 3 (rows 138-140) = **28+31+23+29+26+3 = 140** rows. Matches Cargo.toml
> ground truth exactly. Iter-3's "missing row reconciled in iter-2"
> claim was an arithmetic error (rows 60-82 inclusive = 23 rows, not
> 22 as the iter-3 prose stated); iter-4 F2 corrects the §3.3 header
> (`n = 52`, was 53), the §3.3.1 header (`n = 23`, was 22), and this
> reconciliation prose. The §3.6 totals therefore stand at **140
> existing crates renamed + 4 new check crates scaffolded fresh = 144
> crate-name-affecting ops**, matching §1 line 261.
>
> **Final rename count: 140 existing crates + 4 new check crates = 144
> crate-name-affecting ops in Shard 1** (matches §1 scope summary line
> "Estimated renames + new check crates = total crate-name ops"). The
> canonical decision tree may reclassify some `*-kernel` crates to stay
> `kernel` under v4 (pure types + ports survivors) instead of relayering
> to `domain` — `platform-data-boundary-kernel` (row 1) is the
> strongest candidate for `kernel`-preservation; that reclassification
> changes the LAYER suffix but NOT the rename-required total (the row
> still renames because the v3 `platform-*` prefix collapses to
> `shared-*` under the 3-slot grammar). The canonical-decision-tree
> audit (kernel-vs-domain reclassification + protocol assignment for
> `-api` → {rest, grpc, graphql, worker} + shared-vs-vertical slot-2
> classification per the 3-slot grammar) is the largest pressure-test
> surface for Codex iter-3 (§10 question 1).

> **§3.6 deferral note:** "renamed: N" lines are *aspirational* (cover both
> Shard 1 + Shard 1.5). Shard 1 atomic scope is 114 rows = 140 − 26
> PROTOCOL-UNKNOWN. Per-partition Shard 1 counts:
> | Partition | §3 total | PROTOCOL-UNKNOWN deferred | Shard 1 actual |
> |---|---:|---:|---:|
> | platform | 28 | 5 | 23 |
> | cloud | 31 | 13 | 18 |
> | foundry non-check | 23 | 4 | 19 |
> | foundry check | 29 | 0 | 29 |
> | workspace | 26 | 4 | 22 |
> | foundation+tooling | 3 | 0 | 3 |
> | **total** | **140** | **26** | **114** |
> The 26 deferred rows ship in Shard 1.5 per ADR-0057.

## §4 Cutover order (Hybrid C, ported unchanged from v3)

Same Hybrid C decision and same option-pricing math hold under v4:

- **Option A (atomic, no precursor)**: rejected — bundles tooling risk
  with rename risk; reviewer load undercount risk.
- **Option B (6 sequential context-shards)**: rejected — 6 lockfile
  events; 5 rebase windows; chicken-and-egg lane bootstrap (now check
  bootstrap under v4); the v3 §4.2 row-37-ordering contradiction
  reappears in v4 form for row 139 (`dev-cli`'s precondition cone is
  the entire workspace).
- **Option C — Hybrid C (CHOSEN)**: Shard 0 pure-tooling precursor +
  Shard 1 atomic rename/metadata/dep-edges/CI cutover. Single lockfile
  event; single 48 h coordination window; check crates scaffold empty in
  Shard 0 and populate atomically in Shard 1.

The §4.1, §4.2, §4.3, §4.4 quantitative tables in v3 hold under v4 with
only the row counts updated; the math (5 windows × N reviewers × etc.)
is unchanged. v4 imports the v3 §4 conclusion **verbatim**: Hybrid C.

## §4a — 4 LEAN check crates (iter-2 fold-B; supersedes the 11-check ruleset per Codex iter-1 ITERATE-7 edit C1 "too verbose")

The 11-check ruleset from v4-iter-1-fold-A is collapsed into **4 lean
check crates**. The 7 inner code-shape checks (layer-correctness,
dependency-direction, naming-collision, metadata-schema, lockfile-parity,
lib-name-parity, check-namespace) consolidate as **subcommands of a
single orchestrator** (`check-architecture`). BC validation +
shared/vertical-kind dependency enforcement (the load-bearing new rule
per supplement #2) gets its own crate. Cargo deny + cargo-semver-checks
wrappers stay separate to keep the JSON output schemas distinct.

### LEAN-A1 — `check-architecture` (orchestrator + 7 subcommands)

**Purpose**: single Rust binary that runs all 7 code-shape checks as
subcommands; collates JSON output; produces one consolidated PR comment.
**Subcommands** (each invokable independently):
- `cargo run -p check-architecture -- layer-correctness` — per-layer
  heuristic table (kernel = pure types + ports, with A9 allowlist for
  trivial impls per §15a fix 9); see §2.2.1 + A9 allowlist for full
  classification.
- `cargo run -p check-architecture -- dependency-direction` —
  enforces 12-value layer matrix per §2.2.5; dev-deps and
  `[target.cfg(test).dependencies]` EXCLUDED per §15a fix 10 (kernel/
  domain crates may have `tokio` in dev-deps for integration tests).
- `cargo run -p check-architecture -- naming-collision` — no two
  crates share `<shared|vertical>-<bc>-<layer>` tuple (3-slot rule per
  supplement #2; collision-equivalence is duplicate full crate name);
  cardinality MUST equal workspace member count.
- `cargo run -p check-architecture -- metadata-schema` — every
  Cargo.toml has `[package.metadata.oya]` block with required keys
  `bounded_context`, `kind`, `layer`, `purpose` (and `vertical` if
  `kind == "vertical"`); `layer` value in 12-value canonical enum.
- `cargo run -p check-architecture -- lockfile-parity` — wraps
  `rg -F -f /tmp/old-crate-names.txt Cargo.lock`; **exit-code
  discipline (§15a fix 12)**: rg exit 1 (no match) = PASS; rg exit ≥ 2
  (rg error) = FAIL with explicit error message; subcommand surface
  remaps to consistent exit 0/1 for CI.
- `cargo run -p check-architecture -- lib-name-parity` — `[lib]
  name` (snake) equals snake-case of `[package] name` (kebab); R4
  permanent-control layer.
- `cargo run -p check-architecture -- check-namespace` — every
  `check-*` crate name + `[lib] name` matches regex
  `^check_[a-z][a-z0-9_]*$`.
- `cargo run -p check-architecture -- report` (or `-- all`) —
  invokes all 7 subcommands; non-zero exit if any sub-check fails;
  `--format json` for CI consumption; markdown for PR comment.

**Severity**: BLOCKER post-§8.2 flip.

### LEAN-A2 — `check-bounded-contexts` (BC registry + shared/vertical-kind dependency enforcement)

**Purpose**: enforces three load-bearing rules:
1. Every crate's `[package.metadata.oya].bounded_context` exists in
   `[workspace.metadata.oyatie.bounded_contexts]` with required fields
   `kind` (`shared` OR `vertical`), `owner` (default
   `council-architecture`), `rationale` (1 paragraph), `adr_cite`
   (one-line). `vertical` BCs additionally carry a `vertical: <name>`
   field whose value is registered in `[workspace.metadata.oyatie.verticals]`.
2. **Shared/vertical-kind dependency rule** (the load-bearing new rule
   per supplement #2; extended iter-2 prefold-A item 1 with transitive
   walker):
   - `shared` BCs can depend only on other `shared` BCs.
   - `<vertical>` BCs can depend on `shared` BCs + same-vertical BCs only.
   - **Direct cross-vertical deps REFUSED** (e.g., `cloud-*` crate may
     not depend on `workspace-*` crate; either crate must depend on a
     `shared-*` mediating crate). Exit 1, blocking.
   - **Transitive cross-vertical deps REFUSED** (iter-2 prefold-A
     item 1): LEAN-A2 traverses both direct AND transitive deps via
     `cargo metadata` + recursive resolution. A chain
     `vertical-A → shared-X → vertical-B` fails because the
     `shared-X → vertical-B` edge violates the "shared depends only on
     shared" rule; xtask surfaces the offending intermediate `shared-X
     → vertical-B` edge as the proximate cause AND prints the full
     chain `a → x → b` (with `kind` annotations per node) so reviewers
     can pinpoint the boundary-crossing intermediate that needs
     splitting. Transitive same-vertical via shared
     (`vertical-A → shared-X → shared-Y → vertical-A's-other-BC`) is
     OK — loops back via shared are allowed.
   - **Public-layer exemption** (per cloud dual-role addition; iter-2
     prefold-A): at any cross-vertical hop in the chain, the consumer-
     side edge is exempt if the target crate's layer matches the
     target vertical's `public_layers` allowlist (e.g.,
     `workspace-X → cloud-storage-sdk` allowed because `sdk` is in
     `cloud.public_layers`). LEAN-A2 checks the allowlist at every
     cross-vertical hop, not just chain endpoints.
   - Implementation: parse each crate's `cargo metadata` deps; for each
     dep edge (direct AND transitive), classify source-slot-2 +
     target-slot-2; refuse if (source = `<X>`, target = `<Y>`, X ≠ Y,
     both ≠ `shared`) unless the target crate's layer ∈ target
     vertical's `public_layers`. Violation report format: FULL chain
     `a → x → y → b` with per-node kind annotation.
3. **BC overlap governance** (per §15a fix 5, Codex C5):
   - Parent/child registry rule: if BC A is a prefix of BC B
     (e.g. `policy` parent of `policy-evaluator`), the registry records
     `parent: <A>` on B's entry.
   - Sibling BCs (`policy-evaluator` + `policy-cedar`) require explicit
     `rationale` proving non-overlap of ubiquitous language.
   - Deterministic tie-breaker for conflicting BC name proposals:
     council-architecture votes; ties broken by earlier-PR timestamp.
   - xtask `--check-bc-overlap` runs lexical-prefix + Jaro-Winkler
     similarity on every new BC addition; > 0.85 similarity triggers
     manual review (non-blocking advisory; reviewer applies sibling-vs-
     duplicate judgement).

**Implementation**: Rust binary parsing root `Cargo.toml` workspace
metadata + per-crate `[package.metadata.oya]` blocks + `cargo metadata`
deps. Emits structured JSON for the LEAN-A1 orchestrator's report.
**Severity**: BLOCKER post-§8.2 flip.

> **LEAN-A2 deployment note (v4.1 update):** Under BNF v4.1 the
> `shared|vertical` binary is retired; LEAN-A2 cross-vertical refusal
> is replaced by **microservice isolation enforcement**: no direct
> import edges between distinct µservice crates except through Workflow
> (action adapter) or Ontology (information adapter) per
> [[feedback-workflow-objectgraph-adapter-layer]]. The `public_layers`
> allowlist mechanism is retired. LEAN-A2 is simplified to: refuse any
> dep edge where source µservice ≠ target µservice AND neither endpoint
> is `workflow` or `ontology`. BLOCKER at Shard 1 merge.

### LEAN-A3 — `check-supply-chain` (cargo deny wrapper)

**Purpose**: license + advisories + bans + sources policy enforcement.
**Implementation**: thin wrapper around `cargo deny check`; produces
augmented JSON output `{"violations":[{"crate":"...","severity":"ERROR","kind":"license|advisory|ban|source","detail":"..."}],"schema_version":"1.0"}`
for LEAN-A1 orchestrator consumption.
**Severity**: BLOCKER post-§8.2 flip; pre-existing v3 gate, retained.

### LEAN-A4 — `check-semver` (cargo-semver-checks rename-baseline-reset classifier)

**Purpose**: rename-aware semver checking. Pure-name deltas classified
`BASELINE-RESET`; real breaking changes flagged.
**Implementation**: invokes `cargo-semver-checks 0.46.0` pinned in
`tools/toolchain-versions.toml`; post-processes JSON output with
rename map to re-classify name-only deltas.
**Pinned output schema (§15a fix 11)**:
```json
{
  "violations": [
    {"crate":"<name>","severity":"ERROR|BASELINE-RESET|WARN|INFO",
     "kind":"breaking|name-only|deprecation|...","detail":"..."}
  ],
  "schema_version": "1.0"
}
```
**Severity**: BLOCKER post-§8.2 flip with 14-day post-merge grace where
any rename-crate `BASELINE-RESET` is auto-INFO; non-rename real-semver
failures BLOCKER throughout.

### LEAN-summary

4 check crates total at `crates/check-<name>/`. LEAN-A1 is the
unified orchestrator binary (7 subcommands cover the iter-1-fold-A
A2/A3/A5/A6/A7/A8/A9 surface); LEAN-A2 covers A4 + supplement #2's
cross-vertical refusal rule + §15a fix 5's BC overlap governance;
LEAN-A3 wraps cargo deny (old A10); LEAN-A4 wraps cargo-semver-checks
(old A11). Net result: 7 surface checks collapsed to 1 binary + 3 thin
wrappers; same enforcement, less ceremony. §3 audit's `layer_evidence`
column (per B2) feeds into LEAN-A1's layer-correctness subcommand for
deterministic per-crate verification.

### LEAN-A1 layer-correctness heuristic with A9 allowlist (§15a fix 9)

`kernel` layer allowlist for trivial impls (does NOT relayer to `domain`):
- `impl Default for <Type>` ✓
- `impl Display for <Type>` ✓
- `impl Hash for <Type>` ✓
- `impl <const fn>` ✓
- `impl <getter fn>` (returns a field by-value or by-reference; no logic) ✓
- Any other `fn` body with non-trivial logic → relayer to `domain`

xtask uses syn AST to classify each `fn` body. Heuristic-table values
(was: §4a iter-1 table) remain:

| Layer | Heuristic signature |
|---|---|
| `kernel` | Zero internal workspace deps; only `struct`/`enum`/`trait` items + A9-allowlisted impls; no other `fn` bodies |
| `domain` | Depends on `kernel` only; has non-trivial `fn` bodies; zero framework deps |
| `application` | Depends on `domain` + `kernel`; defines `*UseCase`/`*Service`/`*Handler` types; zero framework deps |
| `adapter` | Contains `impl <Trait> for <Type>` where `<Trait>` is from a `kernel` crate |
| `infrastructure` | Depends on framework crate (tokio/axum/sqlx/tonic/opentelemetry/etc.) AND has no `impl` of a `kernel` trait as the primary public surface |
| `cli` | Has `[[bin]]` target + `clap` (or `argh`/`pico-args`) dep |
| `rest` | Depends on `axum` or `actix-web`; has `Router::new()` or equivalent |
| `grpc` | Depends on `tonic`; has `Server::new()` or service-trait impl with `#[tonic::async_trait]` |
| `graphql` | Depends on `async-graphql` or `juniper`; has `Schema::build` or `RootNode::new` |
| `worker` | Depends on `tokio` runtime + long-running async loop pattern; zero `Router`/`Server`/`Schema` |
| `app` | ≥ 3 cross-layer deps; has `[[bin]]` target; DI-container setup or `main()` wiring multiple constructors |
| `sdk` | Depends on `kernel` only; public `*Client` struct; zero framework deps |

### LEAN-A1 dependency-direction allowed-set (§2.2.5 matrix; §15a fix 10 dev-deps exclusion)

| Consumer layer | Allowed dep layers |
|---|---|
| `kernel` | ∅ |
| `domain` | {kernel} |
| `application` | {domain, kernel} |
| `adapter` | {application, domain, kernel} |
| `infrastructure` | {application, domain, kernel, adapter} |
| `cli` / `rest` / `grpc` / `graphql` / `worker` | {application, domain, kernel} |
| `app` | ALL (composition root, unrestricted) |
| `sdk` | {kernel} |

Dev-deps (`[dev-dependencies]`) and `[target.cfg(test).dependencies]` are
EXCLUDED from enforcement; kernel/domain crates may carry `tokio` in
dev-deps for integration tests without triggering a layer violation.

### (DROPPED — iter-1-fold-A §4a A1 separate "check-clean-architecture" meta-check)

**Purpose**: orchestrates A2 + A3 + A4 + A5 + A6 + A7 + A9 into a unified
report; produces one consolidated PR comment with per-rule pass/fail rows.
**Implementation**: thin Rust binary that invokes the other checks as
subcommands and collates JSON output; pretty-prints to markdown for PR
comment; non-zero exit if any sub-check fails.
**Invocation**: `cargo run -p check-clean-architecture -- --report`
(human-readable) or `--format json` (CI consumption).
**Severity**: BLOCKER post-§8.2 flip.

### (HISTORICAL — sections A2 through A11 + A-summary below describe the v4-iter-1-fold-A 11-check ruleset that was SUPERSEDED by the iter-2 LEAN-A1–LEAN-A4 collapse above. Retained for traceability; the LEAN-A1 orchestrator subcommand list maps 1:1 to A2/A3/A5/A6/A7/A8/A9; LEAN-A2 absorbs A4; LEAN-A3 = A10; LEAN-A4 = A11. CI runs the 4 LEAN crates only.)

### A2 — `check-layer-correctness` (per-layer heuristic table) [SUPERSEDED → LEAN-A1 subcommand `layer-correctness`]

**Purpose**: each crate's declared `[package.metadata.oya].layer` MUST
match its actual code shape per the canonical heuristic table below.
Implementation: parse `cargo metadata` + grep AST patterns
(`syn::parse_file` + simple visitor); exit 1 if any crate's layer
doesn't match its heuristic signature.

| Layer | Heuristic signature |
|---|---|
| `kernel` | Zero internal workspace deps; only `struct`/`enum`/`trait` items; no `impl` bodies (or only `impl Display`/derive macros) |
| `domain` | Depends on `kernel` only (1 or more); has `fn` bodies; zero framework deps (tokio/axum/sqlx/tonic/etc.) |
| `application` | Depends on `domain` + `kernel`; defines `*UseCase`/`*Service`/`*Handler` types; zero framework deps |
| `adapter` | Contains `impl <Trait> for <Type>` where `<Trait>` is from a `kernel` crate |
| `infrastructure` | Depends on framework crate (tokio/axum/sqlx/tonic/opentelemetry/etc.) AND has no `impl` of a `kernel` trait as the primary public surface |
| `cli` | Has `[[bin]]` target + `clap` (or `argh`/`pico-args`) dep |
| `rest` | Depends on `axum` or `actix-web`; has `Router::new()` or equivalent |
| `grpc` | Depends on `tonic`; has `Server::new()` or service-trait impl with `#[tonic::async_trait]` |
| `graphql` | Depends on `async-graphql` or `juniper`; has `Schema::build` or `RootNode::new` |
| `worker` | Depends on `tokio` runtime + long-running async loop pattern (`loop { tokio::select! { … } }`); zero `Router`/`Server`/`Schema` |
| `app` | ≥ 3 cross-layer deps (touches application + at least one adapter/infrastructure + at least one presentation); has `[[bin]]` target; DI-container setup or `main()` wiring multiple constructors |
| `sdk` | Depends on `kernel` only; public `*Client` struct; zero framework deps |

**Severity**: BLOCKER post-§8.2 flip.

### A3 — `check-dependency-direction` (12-value layer dep matrix)

**Purpose**: enforces canonical 12-value dependency-direction matrix from
§2.2.5.
**Implementation**: `cargo metadata --no-deps` + jq cross-check against
allowed-set per consumer layer; exit 1 on any forbidden edge.
**Allowed-set table** (consumer → allowed deps):

| Consumer layer | Allowed dep layers |
|---|---|
| `kernel` | ∅ |
| `domain` | {kernel} |
| `application` | {domain, kernel} |
| `adapter` | {application, domain, kernel} |
| `infrastructure` | {application, domain, kernel, adapter} |
| `cli` / `rest` / `grpc` / `graphql` / `worker` | {application, domain, kernel} |
| `app` | ALL (composition root, unrestricted) |
| `sdk` | {kernel} |

**Severity**: BLOCKER post-§8.2 flip.

### A4 — `check-bounded-contexts` (BC validation) [SUPERSEDED → LEAN-A2; renamed iter-2 from singular `check-bounded-context-registry` to plural `check-bounded-contexts` per the 4-LEAN-check collapse]

**Purpose**: every crate's `[package.metadata.oya].bounded_context` must
exist in `[workspace.metadata.oya].bounded_contexts`. Each registry entry
has required fields: `name`, `owner` (default `council-architecture`),
`rationale` (1 paragraph), `adr_cite` (one-line).
**Implementation**: xtask `--check-bc`; exits 1 on (a) unregistered BC,
(b) registry entry missing `name`/`owner`/`rationale`/`adr_cite`, (c) a
BC marked `deprecated_at: <ts>` still referenced by ≥ 1 crate past the
90-day soft-deprecate window.
**Deprecation lifecycle**: BC entry has optional `deprecated_at`
timestamp; 90-day soft-deprecate window during which the BC is read-only
(no new crate may reference it); final removal via ADR amendment cite
in registry entry.
**Severity**: BLOCKER post-§8.2 flip.

### A5 — `check-naming-collision`

**Purpose**: no two crates share the `<vertical>-<bounded-context>-<layer>`
tuple (3-slot rule per iter-2 supplement #2; collision-equivalence
collapses to "duplicate full crate name"). Catches accidental name
collisions when two crates land in the same bounded
context with the same layer suffix.
**Implementation**: xtask `--check-collision`; emits a hash-set of
tuples across all 140 crates; exit 1 if cardinality < 140.
**Severity**: BLOCKER post-§8.2 flip.

### A6 — `check-check-namespace`

**Purpose**: check crates follow `check-<rulename>` (1-N kebab
tokens; no `<bounded-context>` or `<layer>` slots, because checks are
cross-cutting and not part of the layered architecture they inspect);
`[lib] name` is `check_*` (snake-case parity per A9).
**Implementation**: xtask `--check-namespace`; validates each
`check-*` crate's name + `[lib] name` against the regex
`^check_[a-z][a-z0-9_]*$`; exit 1 on mismatch.
**Severity**: BLOCKER post-§8.2 flip.

### A7 — `check-metadata-schema`

**Purpose**: every Cargo.toml has `[package.metadata.oya]` block with
required keys: `bounded_context`, `layer`, `purpose` (free-text;
1-sentence summary of why the crate exists). Optional keys: `thing`,
`audit_chain`, `feature` (used as presentation-protocol subkey for
cli/rest/grpc/graphql crates to disambiguate multi-protocol bounded
contexts).
**Implementation**: xtask `--check-metadata`; parses each manifest;
exits 1 if any required key absent or layer value not in 12-value
canonical enum.
**Severity**: BLOCKER post-§8.2 flip.

### A8 — `check-lockfile-parity`

**Purpose**: Cargo.lock has zero references to old crate names post-
Shard-1 cutover.
**Implementation**: `rg -F -f /tmp/old-crate-names.txt Cargo.lock`;
exit 1 = no match = pass (rg semantics).
**Severity**: BLOCKER from Shard 1 merge gate (already enforced via §8.1
"Cargo.lock zero old-names" row).

### A9 — `check-lib-name-parity` (preserved from v3 R7, promoted to A-tier)

**Purpose**: `[lib] name` (snake-case) equals snake-case form of
`[package] name` (kebab-case).
**Implementation**: xtask `--lib-name-check`; iterates all manifests;
exit 1 on mismatch.
**Severity**: BLOCKER post-§8.2 flip; part of R4 permanent-control
ledger.

### A10 — `check-cargo-deny` (wraps `cargo deny check`)

**Purpose**: license + advisories + bans + sources policy enforcement.
**Implementation**: thin wrapper around `cargo deny check`; produces
augmented JSON output for the A1 meta-report.
**Severity**: BLOCKER post-§8.2 flip; pre-existing v3 gate, retained.

### A11 — `check-rename-baseline-reset` (wraps `cargo-semver-checks`)

**Purpose**: rename-aware semver checking. Pure-name deltas are
classified `BASELINE-RESET`; real breaking changes flagged.
**Implementation**: invokes `cargo-semver-checks 0.46.0` pinned in
`tools/toolchain-versions.toml`; post-processes JSON output with rename
map to re-classify name-only deltas.
**Severity**: BLOCKER post-§8.2 flip with 14-day post-merge grace where
any rename-crate `BASELINE-RESET` is auto-INFO; non-rename real-semver
failures BLOCKER throughout.

### A-summary

11 check crates total. Each lives at `crates/check-<name>/`. All
ship as `[lib] + [[bin]]` for the A1 meta-check; A8/A10/A11 are thin
wrappers; A2/A3/A4/A5/A6/A7/A9 carry real implementation. §3 audit's
`layer_evidence` column (added per B2) feeds into A2's heuristic table
for deterministic per-crate verification.

## §5 Per-shard checklist (Hybrid C)


Same as v3 §5.0. The renamed `codeview-cli` (row 140) is the new
triad READ slot per `git-workflow.md §2-3`. Bootstrap-window exception
"rename-cutover-v4 bootstrap session" -i critical -k "cutover,bootstrap,
rename-v4"`. The `check-banned-primitives` crate (renamed from
`governance-banned-primitives`, if it existed in v3; otherwise
authored fresh in Shard 0) enforces.

### 5.1 Shard 0 checklist (pure tooling, no renames)

Same shape as v3 §5.1. Differences:

| # | Command | Expected exit | Verification |
|---:|---|:---:|---|
| 1 | `cargo new --lib tools/xtask-metadata-augment` + author body | n/a | Port forward from v3 §5.1 step 1 |
| 1b | Extend `tools/xtask-metadata-augment` with `lockfile-rename` subcommand per v3 §7.1.1 spec | n/a | Same 8-row fixture matrix |
| 2 | `cargo build -p xtask-metadata-augment` | 0 | Helper compiles |
| 3 | `cargo nextest run -p xtask-metadata-augment` | 0 | Helper unit tests pass |
| 3a | `cargo nextest run -p xtask-metadata-augment --test fixtures` (20-cell matrix) | 0 | REQUIRED Shard 0 gate |
| 3b | `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures` (8 rows) | 0 | REQUIRED Shard 0 gate |
| 4 | `cargo run -p xtask-metadata-augment -- --check --shard tools-xtask-metadata-augment` | 0 | Helper self-check |
| 5 | Author **ADR-0056** (BNF spec, bounded-context-registry policy) AND **ADR-0057** (supersedes ADR-0055; drops fitness/freeze/expedite) in **same commit** as ADR-0054 amendment (rename-event scaffold-claim authority) | n/a | Single commit hash contains ADR-0054 amendment + ADR-0056 + ADR-0057 |
| 5a | Update v3 plan frontmatter (`status: Superseded`, `superseded_by: docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`, prepended banner block) | n/a | v3 plan now points to v4 |
| 6 | `git log -1 --name-only HEAD \| grep -E "ADR-005[467]"` | 0 (all three files in diff) | All three ADR files present in Shard 0 commit |
| 7 | Scaffold **4 LEAN check crates** per §4a iter-2-fold-B (collapses iter-1-fold-A's 11 checks per Codex C1): `check-architecture` (LEAN-A1 orchestrator with 7 subcommands), `check-bounded-contexts` (LEAN-A2 BC registry + shared/vertical-kind cross-vertical refusal + BC overlap governance), `check-supply-chain` (LEAN-A3 cargo-deny wrapper), `check-semver` (LEAN-A4 cargo-semver-checks + rename-baseline-reset classifier) | n/a | All 4 `crates/check-*/` directories exist with empty lib + Cargo.toml; `[package.metadata.oya].purpose` populated per LEAN-A1 `metadata-schema` subcommand contract; severity = `--report-only` until §8.2 BLOCKER flip per B6 |
| 7a | **Architect B1 closure**: confirm port traits placed in `kernel` layer (not `domain`) per canonical decision tree §2.2.4; audit `docs/standards/clean-architecture.md` §2.1 "domain — Defines ports: Rust traits" wording for ADR-0056-cited amendment (the standard currently places ports in `domain`; v4 places them in `kernel`); add Shard 1 step 9a to update standard wording in same atomic commit. | n/a | Standard amended in Shard 1; ADR-0056 §"Decision" notes port-location move from `domain` → `kernel` |
| 7b | Author `docs/standards/bounded-contexts.md` (skeleton; initial registry of contexts identified in §3) with required LEAN-A2 fields: `name`, `kind` (closed: `shared` OR `vertical`), `vertical` (open kebab token from `[workspace.metadata.oyatie.verticals]`; required iff `kind == "vertical"`), `owner` (default `council-architecture`), `rationale` (1 paragraph), `adr_cite` (one-line), `parent: <bc>` (optional, used when BC is prefix-child of a parent BC per §15a fix 5); **B3 arbitrator clause** included: "If two PRs propose conflicting BC names for the same crate cluster, council-architecture reviews both proposals; tie-breaker is the proposal with the more specific `rationale` paragraph + ADR cite linking to upstream design discussion; ultimate timestamp tiebreaker per Codex C5: earlier-PR timestamp wins ties not resolvable by rationale + ADR-cite specificity." | n/a | Living-document file present with B3 arbitrator clause + parent/sibling rule + §15a fix 5 BC overlap governance |
| 7d | Author `[workspace.metadata.oyatie.verticals]` registry section in root `Cargo.toml` enumerating initial 3 verticals: `cloud` (owner: council-cloud), `foundry` (owner: council-foundry), `workspace` (owner: council-workspace); each entry carries `rationale` + `adr_cite`. Document the open-set policy (future verticals — `healthcare`, `corporate`, etc. — added via registry append + ADR cite). | n/a | Verticals registry block present; `check-bounded-contexts` (LEAN-A2) reads this to validate slot-2 token of every crate name |
| 7c | **Architect B7 closure (BNF accommodation)**: author ADR-0056 §"BNF accommodation" enumerating the 4 gap cases (proc-macros, codegen crates, test-fixture crates, library+binary split-the-crate rule) with canonical layer assignment for each. See §11 ADR-0056 outline. | n/a | ADR-0056 §"BNF accommodation" present in Shard 0 commit |
| 8 | `cargo check --workspace --all-features` | 0 | Workspace still builds with empty new check crates |
| 9 | Add `[workspace.metadata.oya]` block to root `Cargo.toml` (simplified per §3.0 schema; layer enum + bounded-contexts auto-populated) | n/a | New registry block present |
| 10 | `cargo run -p xtask-metadata-augment -- --registry-check` | 0 | Registry block parses; layer enum matches §2.2 |
| 11 | Record clean-arch BNF decision in ADR-0056 | n/a | ADR-0056 §"Decision" finalised |
| 12 | `rg "\\[workspace\\.metadata\\.oya\\]" Cargo.toml` | 0, 1 line | Single registry row present |
| 13 | `cargo deny check` | 0 | Existing licenses section still passes |
| 14 | `cargo doc --workspace --no-deps` | 0 | Doc build green |
| 15 | `cargo metadata --no-deps --format-version 1 \| jq -r '.workspace_members[]' \| sort > /tmp/shard0-metadata.txt` | 0 | Snapshot stored for Shard 1 path-edge diff |
| 15a | Generate `/tmp/old-crate-names.txt` (one old-name per line, from §3 rename inventory) AND `/tmp/rename-map.tsv` (old<TAB>new) AND `/tmp/reverse-dep-counts.tsv` (per-crate `cargo metadata` reverse-dep count, used for Scenario B mitigation: every crate gets a consumer-count assertion, not just the predicted-highest one) | 0 | `wc -l /tmp/rename-map.tsv` ≈ 135 (the actual rename count); `/tmp/reverse-dep-counts.tsv` has 140 rows |
| 15b | **IDE smoke gate** (NEW per Scenario C pre-mortem mitigation): scaffold a single `policy-test-application` crate with empty lib + minimal cargo metadata; load it in rust-analyzer, IntelliJ-Rust, and VS Code rust-extension; assert each resolves the crate and renders symbols without warnings. Remove the test crate before Shard 0 merge. | 0 (per IDE) | If any IDE flags `application` as a reserved layer name, fall back to `app` layer enum (1-ADR edit to ADR-0056). |
| 15c | **§3 audit table 3-slot column-schema rework** (FINAL per iter-2 supplement #2; supersedes the iter-1-fold-A 2-slot rework): regenerate §3.1–§3.5 audit tables under the 3-slot schema `oyatie-<shared\|vertical>-<bc>-<layer>`. Each row's `proposed_name` follows the 3-slot pattern. Add columns: `vertical` (value = `shared` OR a vertical name), `bounded_context` (kebab; multi-token allowed), `layer` (12-value enum), `layer_evidence` (file:line cite OR `cargo metadata` query OR `PROTOCOL-UNKNOWN` deferral marker per Codex C1), `bc_registry_status` (REGISTERED / PROPOSED-NEW / DEPRECATED), `kind` (`shared` / `vertical`; matches slot 2). Drop `thing?` column (already dropped pre-supplement-2). **Audit translation from v3**: v3 `platform-*` → slot 2 = `shared`; v3 `foundation-*` → slot 2 = `shared`; v3 `tooling-*` → slot 2 = `shared`; v3 `cloud-*` → slot 2 = `cloud`; v3 `foundry-*` → slot 2 = `foundry`; v3 `workspace-*` → slot 2 = `workspace`. xtask `--audit-rebuild` automates this. | 0 | All §3 tables match the canonical column directive; xtask emits `/tmp/audit-v4-3slot.tsv` for Codex iter-2 consumption |
| 15d | **BC registry multi-token expansion + kind/vertical fields** (FINAL per supplement #2): `docs/standards/bounded-contexts.md` initial population at **~100 entries**. Each entry carries LEAN-A2 required fields: `name`, `kind`, `owner`, `rationale`, `adr_cite`, plus `vertical: <name>` if `kind == "vertical"`, plus `parent: <bc>` if BC is prefix-child of another BC per §15a fix 5. Examples of `kind: shared` BCs: `audit-chain`, `eventing`, `tenant`, `identity`, `policy-cedar`, `composition`, `codeview`, `dev`. Examples of `kind: vertical, vertical: cloud` BCs: `compute-vm`, `compute-k8s`, `compute-functions`, `storage-object`, `storage-block`, `network-vpc`, `network-dns`, `network-lb`, `iam`, `billing`. Examples of `kind: vertical, vertical: foundry` BCs: `foundry-policy`, `foundry-evidence`, `foundry-eval`, `foundry-rag`. Examples of `kind: vertical, vertical: workspace` BCs: `drive`, `chat`, `mail`, `meet`, `calendar`. | n/a | BC registry file present with ~100 entries; each row has B3 arbitrator clause ownership default + §15a fix 5 BC overlap governance + supplement #2 kind/vertical taxonomy |

### 5.2 Shard 1 checklist (atomic ~139-rename + everything else)

Same shape as v3 §5.2; row counts updated:

| # | Command | Expected exit | Verification |
|---:|---|:---:|---|
| 2 | Update root `Cargo.toml` `[workspace] members = [...]` array per §3 | n/a | All entries updated atomically |
| 3 | `git mv crates/oyatie-<old> crates/oyatie-<new>` × ~139 | 0 each | Directory renames |
| 4 | `cargo run -p xtask-metadata-augment -- --apply` | 0 | All 140 manifests carry `[package.metadata.oya]` per §3.0 schema |
| 5 | For each renamed crate: rewrite `[package] name` AND `[lib] name = "..."` (underscored form) | n/a | R7-equivalent permanent control (renamed from v3 §6 R7) |
| 6 | Rewrite all dep-edge `path = "../oyatie-<old>"` entries (est. 200–400 sites) | n/a | xtask traversal per v3 §3.3.1 matrix |
| 7 | Update 3 CI workflow files (same as v3) — `cargo run -p tooling-cli-dev-runtime` → `cargo run -p dev-cli`; other references per §3 | n/a | Per-workflow grep verification |
| 8 | Update `scripts/check.sh` (~29 sites), `scripts/hooks/pre-push-repoctl.sh` (1 site), `scripts/check-architecture-boundaries.sh` (3 sites). New: rename references to `intelligence-api` and `foundation-app` per §3 rows 138 + 60 | n/a | Verified in §8.1 zero-old-names gate |
| 9 | Update `docs/standards/clean-architecture.md` §3 row 35 named-by-identity reference: `platform-data-boundary-kernel` → `data-boundary-domain` | n/a | Same row, new name |
| 10 | **REWRITE** `docs/standards/crate-naming-convention.md` or mark `status: Superseded by ADR-0056` (decision: rewrite, because the doc carries content beyond the BNF — context table, role table, hyperscaler mapping, etc. — that is salvageable under v4) | n/a | Doc rewritten with v4 BNF + bounded-context-registry policy; ADR-0056 cited |
| 10b | **CO-EDIT** `docs/standards/code-style-rust.md` lines **11-12, 137-147, 162-177** (per Codex iter-2 D6): replace v3 BNF + 9-value role enum (`kernel/domain/app/api/worker/adapter/runtime/cli/sdk`) declarations with v4 3-slot BNF + 12-value layer enum + canonical decision tree references. Add explicit ADR-0056 cross-reference. | n/a | `code-style-rust.md` no longer conflicts with ADR-0056; verified via `rg -n "role.*::=.*kernel" docs/standards/code-style-rust.md` returning no match |
| 10a | Author `docs/standards/bounded-contexts.md` (full version, populated from `[package.metadata.oya].bounded_context` fields per §2.4) | n/a | Living-document doc materialised |
| 11 | Update `registry/quality/lanes.yaml`, `registry/docs/pipeline.tsv`, OpenAPI bindings | n/a | All rename references flipped |
| 12a | Snapshot pre-rename metadata: `cargo metadata --locked --format-version 1 > /tmp/cargo-metadata-pre-rename.json` | 0 | Captures the baseline |
| 12b | Scripted name-rewrite of `Cargo.lock` via xtask: `cargo run --release -p xtask-metadata-augment -- lockfile-rename --rename-map /tmp/rename-map.tsv --lockfile Cargo.lock --inplace` | 0 | Deterministic rewrite; toml_edit-based |
| 12c | `cargo check --workspace --locked --offline` | 0 | `--locked` refuses any non-name delta |
| 13 | All §8.1 deterministic acceptance gates | 0 (all) | Merge-gate |
| 15 | **B6 closure (3-substep chicken-and-egg avoidance; normalized iter-5 G3 to 4-LEAN design)**: (a) During Shard 1 merge, **the 4 LEAN check crates** (`check-architecture` with 7 subcommands under LEAN-A1, `check-bounded-contexts` LEAN-A2, `check-supply-chain` LEAN-A3, `check-semver` LEAN-A4) run in `--report-only` mode — they CANNOT fail the merge that introduces them; (b) Post-merge §8.2 global gate (a follow-up commit or scheduled CI job) flips severity from `--report-only` to BLOCKER for all 4 check crates atomically; (c) Any violation detected by `--report-only` mode during the Shard 1 merge but not yet blocked is logged to MISTAKES-LEDGER topic `mistakes-rename-v4-shard-1` for follow-up resolution. Chicken-and-egg avoided: the merge that introduces the checks cannot fail by their own enforcement. | n/a | Shard 1 commit ships 4 LEAN check crates at `--report-only`; §8.2 BLOCKER-flip commit follows in a separate PR within 24 h |
| 15a | **Architect B1 closure (clean-architecture.md amendment)**: in the SAME atomic Shard 1 commit, update `docs/standards/clean-architecture.md` §2.1 "domain — Defines ports" wording to read "kernel — Defines ports (Rust trait declarations) + pure types" per ADR-0056-cited port-location move. | n/a | Standard wording matches v4 canonical decision tree |
| 16 | Flip ADR-0056 + ADR-0057 status `Proposed → Accepted` | n/a | ADR header status |

## §6 Risk cone (R1–R10)

| Risk | Likelihood | Impact | Mitigation |
|---|:---:|:---:|---|
| **R2 — `cargo-deny` schema mismatch** (NEW name; was v3 R3). | L | L | `deny.toml` audited (`[licenses]` only — no `[bans]` rules referencing crate names). Generator-from-metadata for `[bans]` deferred to post-Shard-1 ADR — out of scope for v4. |
| **R3 — Row 35-equivalent blast radius** (NEW: per Scenario B mitigation, applies to every renamed crate, not just the predicted highest). | M | H | Shard 0 step 15a emits `/tmp/reverse-dep-counts.tsv` covering ALL 140 crates. §8.1 reverse-dep gate enforces per-rename consumer-count assertion ≡ pre-rename count. If any rename's count drifts, gate fails. Most likely row-35-equivalent under v4: `data-boundary-domain` (v3's 95 consumers; ported forward). If another crate emerges higher (e.g. `tenant-domain`, `identity-domain`, `eventing-domain`, `audit-chain-domain`), that crate also gets the named §3 docs/code co-edit + risk = 5 treatment. |
| **R7 — cargo-semver-checks baseline reset.** | M | M | Same strategy as v3 R9: rename = breaking change at package-name level; Shard 1 commits `--baseline-rev <pre-shard-1-sha>` snapshots. New check crate `check-rename-baseline-reset` (renamed from v3's `governance-baseline-reset-kernel`) classifies name-change-only failures as class `BASELINE-RESET`. 14-day post-merge grace where any semver-checks failure on a renamed crate is auto-classified `BASELINE-RESET`. |
| **R8 — Staging-promotion compounding** (port forward from v3 §7.2 REVERT-STAGING-BLOCK soft-edit). | L | M | If Shard 1 reaches staging before a revert fires, the revert PR title prefixes `REVERT-STAGING-BLOCK:`; staging-promotion lane refuses next promotion until a `STAGING-UNBLOCK:` follow-up; post-revert observability sweep is BLOCKING (vs. non-blocking on normal path). |
| **R9 — Dependency cycle detection (NEW).** Clean Architecture v4 enforces layer dependency direction at compile/CI time. A rogue commit that introduces a cycle (e.g. `audit-chain-application` depending on `audit-chain-file-infrastructure` directly instead of via port trait) would compile but violate clean-arch principles. | L | H | New `check-architecture` crate (LEAN-A1; `dependency-direction` subcommand per §4a) parses `cargo metadata --no-deps`, classifies each edge by source-layer → target-layer pair, and refuses any edge not in the §2.2 allowed-edge table. New §8.1 gate row "Dependency direction check". Severity = BLOCKER. |
| **R11 — Atomic Shard-1 review at 139-rename scale (NEW; B5 reviewability; rebalanced iter-2 per Codex C6 from 3 → 4 streams)**. A single reviewer cannot meaningfully review a 500–700-file diff covering ~140 renames + 140 metadata blocks + ~200–400 dep-edge rewrites + CI/scripts/docs/registry co-edits in any reasonable time budget. | M | M | **4 parallel reviewer streams partitioned by §3 audit cluster + hotspot**: stream 1a = platform/shared (~28 crates per §3.1) — reviewer-platform; 1b = cloud vertical (~31 crates per §3.2) — reviewer-cloud; 1c = foundry vertical (~51 crates per §3.3) — reviewer-foundry; **1d = workspace vertical + tooling-now-shared + 4 hotspots (~30 crates + hotspot artefacts: ADR-0056, clean-architecture.md standard amendment per Codex C2, xtask spec, 4 lean check crates) — reviewer-lead (full-PR scope)**. Each reviewer signs off on their partition only; reviewer-lead also signs off on the cross-partition meta. The atomic squash-merge requires **all 4 partition sign-offs** before merge. New §8.1 gate row "4 partition sign-offs collected" enforces. Reviewer-hours accounting in §9 updates: 8–10 h per primary × 4 reviewers in parallel = **32–40 h calendar reviewer-hours** (HONEST sizing per iter-2 prefold-A item 3 sync; was 24–30 h under 3-stream pre-supplement budget — explicitly accepted when expanding to 4 partitions); each reviewer's load is bounded by their partition's file count (~100–250 files vs. 500–700 full-PR). |
| **R11a — Transitive cross-vertical dep refusal (NEW; iter-2 prefold-A item 1).** Direct cross-vertical refusal (LEAN-A2) catches `cloud-* → workspace-*` edges. The harder failure mode is TRANSITIVE: a vertical-A crate depends on a `shared` crate X, which itself depends on a vertical-B crate. The transitive chain compiles even though A inadvertently consumes vertical-B logic via the shared intermediary. | M | H | **LEAN-A2 traverses BOTH direct AND transitive deps** via `cargo metadata` + recursive resolution. **Three sub-rules**: (a) **Direct cross-vertical dep** (`vertical-A → vertical-B` where A ≠ B, both ≠ `shared`) → ERROR (exit 1, blocking; existing rule). (b) **Transitive cross-vertical via shared** (`vertical-A → shared-X → vertical-B`) → ERROR (refused — `shared` crates cannot depend on any vertical; this is already covered by the `kind: shared → only shared` direct rule, but the transitive walker SURFACES the offending `shared-X → vertical-B` edge as the proximate cause). (c) **Transitive same-vertical via shared** (`vertical-A → shared-X → shared-Y → vertical-A's-other-BC`) → OK (loops back via shared are allowed; the same-vertical termination satisfies the rule even though the chain transits two `shared` crates). **Violation report format**: xtask emits the FULL dep-chain as `a → x → y → b` (with `kind` annotations per node) — not just the endpoints — so reviewers can pinpoint the boundary-crossing intermediate `shared` crate that needs splitting. Public-layer exception (per cloud dual-role addition) is applied AT each `vertical-* → shared-*` edge, NOT at the chain endpoints — i.e., a chain `workspace-X → cloud-storage-sdk` is allowed because the consumer-side edge terminates at a public-layer crate, even if intermediate transit existed; `shared-bounded-contexts-check-cli` walks the chain and checks `public_layers` allowlist at every cross-vertical hop. |

## §7 Rollback plan

### 7.1 Per-shard rollback (Hybrid C)

**Shard 0**: rare. `git revert <shard-0-sha>` removes xtask, ADR-0054
amendment, ADR-0056, ADR-0057, new check crate scaffolds, registry block.

**Shard 1**: `git revert <shard-1-sha>` restores all 140 directory names,
member list, dep-edges, CI/scripts, doc references, and Cargo.lock
(single commit ⇒ single revert). Then run §8.1 gates against pre-Shard-1
state. Lockfile inverse-rename via `cargo run --release -p xtask-metadata-augment
-- lockfile-rename --bnf-version v4.1 --rename-map /tmp/rename-map.tsv --lockfile Cargo.lock --inplace --reverse`
then `cargo check --workspace --locked --offline`.

> **BNF v4.1 flag**: `--bnf-version v4.1` instructs the xtask to apply
> the v4.1 translation rule (`platform-<bc>` → `oyatie-<bc>`,
> `workspace-<bc>` → `connect-<bc>`, etc.) rather than the
> v4.0 `shared|vertical` rule. The xtask must be updated in Shard 0
> to accept this flag before Shard 1 dispatches.

### 7.2 Pre-authorised emergency revert lane

Port forward from v3 §7.2:
- Any council-architecture or axis-foundry member can self-approve
  `git revert <shard-1-sha>` (no peer review).
- Revert PR uses `gh pr merge --admin` under the named exception in
  ADR-0057 §"Rollback/expedite protocol".
- The exception requires **all three preconditions** at invocation time:
  held by a Security Council member** (replaces v3's `freeze_active ==
  operator possesses standing Security Council authority (replaces v3's
  direct-tool-invocations -c "EMERGENCY revert of Shard 1 via admin-
  authority" -i critical` logged BEFORE the admin-merge command runs.
- CI bypass: revert PR runs only `cargo check --workspace --all-features`
  (~3 min); full §8 gate set is BYPASSED. Full platform re-runs post-merge
  as a non-blocking observability sweep (BLOCKING per R8 if staging
  already reached).

### 7.3 Rollback time budget

| Path | Wall-clock |
|---|---|
| Shard 0 revert (rare) | < 15 min |
| Shard 1 revert (standard, full gate) | < 60 min |
| Shard 1 revert (emergency lane, CI bypass) | < 15 min |
| Post-emergency observability sweep | < 30 min (non-blocking unless R8) |

## §8 Acceptance gates (deterministic, ~14 commands)

Every gate is a runnable command with explicit exit-code expectation.

### 8.1 Shard-level gates

| Gate | Command | Expected exit |
|---|---|:---:|
| Workspace compiles | `cargo check --workspace --all-features` | 0 |
| Workspace builds | `cargo build --workspace --all-features` | 0 |
| Clippy clean | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| Cargo-deny clean | `cargo deny check` | 0 |
| Docs build | `cargo doc --workspace --no-deps` | 0 |
| Path-edge diff | `cargo metadata --no-deps --format-version 1 \| jq -S '.packages[] \| {name, manifest_path}' > /tmp/post.txt && diff /tmp/shard0-metadata.txt /tmp/post.txt` | exit 1 for Shard 1 (diff present means renames happened) |
| Tests pass | `cargo nextest run --workspace --all-features --no-fail-fast --message-format libtest-json + junit` | 0 |
| Semver-checks (R7 reset) | Same as v3 §8.1 row; `cargo-semver-checks 0.46.0` pinned; `check-rename-baseline-reset` classifies BASELINE-RESET failures | 0 (only BASELINE-RESET failures allowed) |
| Cargo.lock zero old-names | `rg -F -f /tmp/old-crate-names.txt Cargo.lock` returns no match | exit 1 (no match = pass) |
| Cargo.lock semver-section parity | Same diff command as v3 §8.1 (preserves external version/source/checksum) | 0 |
| Registry refs zero old-names | `rg -F -f /tmp/old-crate-names.txt registry/ AGENTS.md docs/CONSTITUTION.md docs/TOOLCHAIN.md docs/RELEASE-MANAGEMENT.md scripts/ .github/workflows/ \| rg -v "docs/CHANGELOG.md\|docs/plans/rename-plan-\|docs/decisions/ADR-005[4567]"` returns no match | exit 1 (no match = pass) |
| xtask metadata-augment fixture matrix | `cargo nextest -p xtask-metadata-augment --test fixtures` (20 cells) | 0 |
| xtask lockfile-rename fixture matrix | `cargo nextest -p xtask-metadata-augment --test lockfile_rename_fixtures` (8 rows) | 0 |
| `[lib]` name parity | `cargo run -p xtask-metadata-augment -- --lib-name-check` | 0 |
| Reverse-dep count parity (R3 mitigation: all 140 crates, not just row-35-equivalent) | `for old new <- /tmp/rename-map.tsv: test "$(cargo metadata --locked --format-version 1 \| jq -r '[.packages[] \| select(.dependencies[]?.name == "'$new'") \| .name] \| unique \| length')" -eq "$(grep -P "^$old\t" /tmp/reverse-dep-counts.tsv \| cut -f2)"` (one assertion per rename) | 0 for all |
| **LEAN-A1 — Architecture orchestrator (7 inner code-shape checks)** | `cargo run -p check-architecture -- report --format json` (invokes subcommands `layer-correctness` + `dependency-direction` + `naming-collision` + `metadata-schema` + `lockfile-parity` + `lib-name-parity` + `check-namespace`; classifies dep edges per 12-value §2.2.5 matrix; per-layer heuristic per §4a LEAN-A1 table; dev-deps + `[target.cfg(test).dependencies]` EXCLUDED from direction enforcement per §15a fix 10; tuple uniqueness on `<shared\|vertical>-<bc>-<layer>` per 3-slot grammar). **Lockfile-parity subcommand exit-code discipline** per §15a fix 12: rg exit 1 (no match) = PASS, rg exit ≥ 2 (rg error) = FAIL with explicit error message. | 0 |
| **LEAN-A2 — Bounded-contexts registry + shared/vertical-kind dep enforcement (D4 explicit transitive walker + public_layers hop check + chain output)** | `cargo run -p check-bounded-contexts -- --check` enforces: (i) every `[package.metadata.oya].bounded_context` registered in `[workspace.metadata.oyatie.bounded_contexts]` with `name`/`kind`/`owner`/`rationale`/`adr_cite` (+ `vertical` if `kind == "vertical"`); (ii) **CROSS-VERTICAL DEPENDENCY REFUSAL — DIRECT** — parses each crate's `cargo metadata` deps; classifies source slot-2 + target slot-2; refuses direct edges where (source = vertical-X, target = vertical-Y, X ≠ Y, both ≠ `shared`); (iii) **CROSS-VERTICAL DEPENDENCY REFUSAL — TRANSITIVE (D4)** — walks transitive deps via recursive `cargo metadata` traversal; at EACH cross-vertical hop in the chain, checks whether the target crate's layer ∈ target vertical's `public_layers` allowlist (if yes, hop is allowed via the public-layer exemption per §11 ADR-0056 §"Cloud vertical dual-role + public_layers"; if no, hop is refused); `shared → vertical` hops never qualify for public-layer exemption (shared reuse requires complete vertical neutrality); (iv) shared BCs depend only on other shared BCs; (v) BC overlap governance: lexical-prefix + Jaro-Winkler similarity > 0.85 triggers manual review (non-blocking advisory); (vi) **Violation output FORMAT (D4)**: on violation, emit FULL dep-chain `a → x → y → b` with per-node `{kind: shared\|vertical, vertical: <name>, layer: <12-value>}` annotation; reviewers pinpoint the boundary-crossing intermediate (typically a `shared` crate that incorrectly depends on a vertical) for splitting. | 0 |
| **LEAN-A3 — Supply-chain (cargo-deny wrapper)** | `cargo run -p check-supply-chain -- --check` wraps `cargo deny check`; pinned JSON output schema `{"violations":[...],"schema_version":"1.0"}` | 0 |
| **LEAN-A4 — Semver (cargo-semver-checks rename-baseline-reset classifier)** | `cargo run -p check-semver -- --check --baseline-rev <pre-shard-1-sha>` wraps `cargo-semver-checks 0.46.0`; pinned JSON output schema `{"violations":[{"crate":"...","severity":"ERROR\|BASELINE-RESET\|WARN\|INFO","kind":"breaking\|name-only\|deprecation\|...","detail":"..."}],"schema_version":"1.0"}` per §15a fix 11; only `BASELINE-RESET` failures allowed during 14-day post-merge grace window | 0 |
| **B5/C6 — 4 partition sign-offs collected** (NEW; R11 reviewability; iter-2 rebalance from 3 → 4) | PR-comment heuristic: `gh pr view <num> --json comments \| jq '[.comments[] \| select(.body \| test("APPROVE-PARTITION:(1a\|1b\|1c\|1d)"))] \| group_by(.body \| capture("APPROVE-PARTITION:(?<p>1[abcd])").p) \| length'` returns `4` | exit 0 iff value = 4 |

### 8.2 Global gate (after Shard 1 merge)

All 8.1 commands MUST exit per the table above against `main` at the
Shard 1 squash commit. Additionally:

| Gate | Command | Expected exit |
|---|---|:---:|
| Zero hits global sweep | `rg -F -f /tmp/old-crate-names.txt . -g '!docs/CHANGELOG.md' -g '!docs/plans/rename-plan-*.md' -g '!docs/adr-archive/ADR-0054-grit-scaffold-claim-pattern.md' -g '!docs/adr-archive/ADR-0056-rust-clean-architecture-bnf.md' -g '!docs/adr-archive/ADR-0057-cutover-mechanics-rename-plan-v4.md'` | exit 1 (no match = pass) |
| ADR-0056 status | `rg "^status: Accepted" docs/adr-archive/ADR-0056-rust-clean-architecture-bnf.md` | 0 |
| ADR-0057 status | `rg "^status: Accepted" docs/adr-archive/ADR-0057-cutover-mechanics-rename-plan-v4.md` | 0 |
| Bounded-context registry consistency | `cargo run -p xtask-metadata-augment -- --bounded-context-registry-check` | 0 (every bounded-context field in all 140 crates appears in `docs/standards/bounded-contexts.md`) |
| **B6 closure — 4-check BLOCKER flip atomicity** (post-Shard-1, separate PR within 24 h; updated iter-2 from 11 → 4 per LEAN-A1–A4 collapse) | All 4 `check-*` crates' CI workflow rows flip from `severity: report-only` to `severity: BLOCKER` in a single commit; verify via `rg "severity: BLOCKER" .github/workflows/checks.yml \| wc -l` returns `4` | 0 (count = 4) |

## §9 Estimated effort

Honest reviewer-hours pricing (per Architect re-pricing convention from
v3, ported forward):

| Phase | Wall-clock (executor) | Reviewer time | Bottleneck |
|---|---:|---:|---|
| Shard 0 (xtask + ADR-0054 amendment + ADR-0056 with §"BNF accommodation" + §"BC arbitrator" + §"BC overlap governance" + §"Verticals registry" with lifecycle + §"Cloud vertical dual-role + public_layers" + §"Build tooling vs coordination primitives" + ADR-0057 + **4 LEAN check scaffolds per §4a LEAN-A1–LEAN-A4** + new bounded-contexts.md skeleton with B3 arbitrator clause + parent/child + sibling rule + verticals registry block with `cloud.public_layers = ["sdk"]` + IDE smoke gate + B7 BNF-accommodation step) | **6–9 h** (revised down from iter-1-fold-A's 7–10 h due to LEAN-A1–A4 collapse from 11 → 4 check scaffolds, OFFSET by the iter-2 prefold-A ADR additions: dual-role + verticals lifecycle + transitive-walker spec) | **2 h** (unchanged from iter-1; ADR sub-section count grew but per-section length is bounded) | xtask + 3 ADRs (0054 amendment + 0056 + 0057) + 4 LEAN check scaffolds + verticals registry + IDE smoke |
| Shard 1 (atomic ~140-rename + 140 metadata + 200–400 dep-edges + CI + scripts + docs co-edits + clean-architecture.md:99-103 amendment per Codex C2 + rewrite of crate-naming-convention.md + lockfile regen + verticals registry materialisation) | **12–18 h** | **8–10 h per primary reviewer × 4 reviewers parallel = 32–40 h calendar reviewer-hours** (iter-2 prefold-A item 3 honest-sizing sync; was 24–30 h under 3-stream pre-supplement budget; the 4th partition was added per Codex C6 for hotspot-coverage at reviewer-lead) | Reviewer load on ~5 hotspots (rows 1, 60-conformant subset, 105, 109, 138, 139); 4 streams = 1a platform/shared + 1b cloud + 1c foundry + 1d workspace+tooling+hotspots-reviewer-lead per §6 R11 |
| **Total** | **~18–27 h executor + 34–42 h calendar reviewer (8–10 h per primary × 4 reviewers parallel + 2 h Shard 0)** | — | — |
| Rollback (standard) | < 60 min | — | git revert + lockfile inverse-rename |
| Rollback (emergency lane) | < 15 min | — | admin-merge + 3-min CI |

Compare to v3 honest pricing (12–18 h executor, 18–24 h reviewer, 3–5
days calendar): v4 is ~40 % more executor work + ~75 % more reviewer
work (32–40 h vs. 18–24 h; the increase reflects the 4-stream
partition + reviewer-lead role per Codex C6 + iter-2 prefold-A item 3
honest sizing), but **buys a permanently simpler grammar that the team
will live with for years** plus the canonical Clean-Architecture
taxonomy + shared/vertical kind enforcement (cross-vertical refusal
catches a class of architectural drift that v3 had no mechanism to
prevent). v3 itself was a year of evolution from earlier ad-hoc naming;
v4 closes the loop.

## §10 Open questions for Codex iter-1 (post-comprehensive-fold + 2-slot simplification)

After fold-A (11-check codification ruleset + 7 architect-condition
closures) AND the 2-slot BNF simplification (drop `<thing>` slot per
third correction), the top expected Codex iter-1 pressure-test surfaces
narrow to 2 primary + 1 secondary:

### Primary surface 1 — Multi-token bounded-context governance at 100-entry scale

Codex will likely pressure-test: (a) how the BC registry handles
overlapping names like `policy` + `policy-evaluator` + `policy-cedar`
— is `policy-evaluator` a sub-BC of `policy` (hierarchical) or a
sibling (flat)? v4 says flat (each BC is a distinct kebab string);
Codex may probe whether the flat model produces silently-related-but-
governance-independent BCs. (b) whether the B3 arbitrator tie-breaker
(more-specific rationale + ADR cite) is deterministic when both PRs
cite the same ADR. (c) whether the ~100-entry registry size (up from
v4-draft-4's ~72) requires a separate review cadence for BC additions
vs. the inline-with-rename-PR pattern v4 assumes. (d) whether the
`bc_registry_status` audit column (NEW per §3) is itself a governance
risk if it diverges from the registry doc.

### Primary surface 2 — `layer_evidence` audit completeness at 139-crate scale

Per B2, every audit row in §3 must carry a file:line evidence cite
OR explicit `PROTOCOL-UNKNOWN, deferred to ADR-0056 §X` marker. Codex
will likely pressure-test: (a) the failure mode when a v3 `*-kernel`
crate has BOTH `struct` items AND `fn` bodies — does it relayer to
`domain` (logic present) or stay `kernel` (struct-heavy)? v4 A2
heuristic table says: `fn` body presence flips to `domain`; Codex may
probe edge cases (`impl Display` derives; const fn; trivial getters).
(b) whether the `cargo run -p check-layer-correctness` AST-grep
implementation correctly handles cfg-gated items. (c) whether
`PROTOCOL-UNKNOWN` deferral markers in §3 block Shard 1 merge or
ship with explicit splits-in-follow-up commits.

### Secondary surface 3 — 11-check BLOCKER-flip ordering

The B6 chicken-and-egg avoidance (Shard 1 introduces checks at
`--report-only`; §8.2 follow-up PR flips to BLOCKER) creates a brief
window (≤ 24 h target) where checks are non-binding. Codex may
pressure-test: (a) whether 24 h is the right window or should be
tighter; (b) what happens if a violation lands in the gap between
Shard 1 merge and BLOCKER-flip; (c) whether the §8.2 "11 BLOCKER flip
atomicity" gate (counting `severity: BLOCKER` rows == 11) is
sufficient or should also verify the flip-PR's diff contains exactly
the expected 11 rows.

Architect's 7 iter-1 conditions (B1–B7) are CLOSED per fold-A; the
above 3 surfaces are NET-NEW pressure tests on the post-fold v4 state.

1. **Per-crate `src/`-inspection audit under canonical decision tree
   (NEW; top surface post-12-value canonical-enum finalisation).** v4
   §3 provides provisional layer defaults (v3 `*-kernel` → v4 `domain`;
   v3 `*-app` → v4 `application` except composition root → `app`; v3
   `*-adapter-*` → v4 `adapter`; v3 `*-api` → v4 `rest`) BUT every
   crate needs `src/`-inspection by Codex iter-1 to confirm the
   canonical decision tree (§2.2.4) assignment:
   - (a) Which currently-named-`*-kernel` crates are PURE types + ports
     (stay `kernel` under v4) vs. carry business logic (relayer to
     `domain` under v4)? Expected `kernel` survivors: type-only crates
     like `data-boundary-kernel` (per `clean-architecture.md §3`
     it is named-by-identity as the only kernel allowed to receive
     cross-layer deps); possibly several check-rule type-bundle crates.
   - (b) Which `*-api` crates serve gRPC (candidates: OTLP via
     `cloud-observability-api`; streaming retrieval via
     `intelligence-rag-api`; k8s watch streams via
     `compute-k8s-api`)?
   - (c) Which `*-api` crates serve GraphQL (candidates: workspace-axis
     user-facing surfaces — `chat-api`, `drive-api`,
     `meet-api` — modern UIs frequently consume GraphQL)?
   - (d) Are there multi-protocol crates that should split into
     per-protocol crates OR carry an ADR-0056 §"Bounded context
     registry" exception?
   - (e) Which `*-api` crates are actually `worker` (queue / scheduled
     / pubsub consumers)?
   The audit answer determines proposed_name for ~22 `-api` rows AND
   may flip layer assignments for any number of `-kernel` rows.
2. **Bounded-context registry governance.** v4 §2.4 establishes
   `docs/standards/bounded-contexts.md` as a living document with
   90-day auto-deprecation. Codex may pressure-test: (a) who owns
   adjudicating naming disputes between teams that want different
   bounded-context names for overlapping domains (e.g. row 47 `cloud-data`
   vs. a future `data` context); (b) whether the 90-day auto-deprecation
   rule should be deterministic (xtask + cron) or advisory; (c) whether
   the doc should be ADR-cited per entry, or just maintained as prose.
3. **The "fitness" terminology drop blast radius + check-crate BLOCKER
   chicken-and-egg.** Every v3 fitness crate's invocation across ~30
   sites in `scripts/`, `.github/`, and `docs/` referenced the old
   name. Codex may pressure-test: (a) whether the xtask actually
   catches every fitness-crate reference (especially
   `governance-architecture-conventions-kernel` the
   load-bearing lane); (b) whether the BLOCKER-flip strategy for
   check crates (Shard 1 step 15) introduces a chicken-and-egg if a
   check crate's BLOCKER mode would fail the Shard 1 merge itself; (c)
   whether `.omc/governance-lanes/` directory should be renamed
   `.omc/check-rules/` atomically with Shard 1.

Secondary surfaces (lower probe priority):
- **Per-crate layer assignment ambiguity for inner-layer crates.** v4
  audits 140 crates against the 9-layer enum. Some inner-layer crates
  blur the line: e.g. row 116 `collab-runtime-domain` is named
  "runtime" but classified as `domain` (because the original parsed as
  feature=`collab-runtime` + role=`kernel`). Codex may pressure-test
  the classification rule for crates where the original name contains
  a token that disagrees with the v4 layer assignment.

## §11 ADR-0056 outline — Rust Clean Architecture layer enum and canonical decision tree

**Status**: Proposed in Shard 0 commit; flips to Accepted at end of
Shard 1.

**Decision**: Adopt the canonical Rust Clean Architecture crate naming
grammar `oyatie-<shared|vertical>-<bounded-context>-<layer>` (3-slot grammar per iter-2 supplement #2 + Codex iter-4 F4 rewrite; granularity
expressed via multi-token bounded-context names tracked in the registry,
NOT via a separate slot) + flat `check-<rule-name>` namespace for
cross-cutting checks. Layer is a closed **12-value enum** with canonical
meanings (no aliases, no overlaps); each crate occupies exactly ONE
layer; assignment follows the canonical decision tree (§2.2.4).

**Granularity governance**: bounded-context granularity is governed by
the registry (`docs/standards/bounded-contexts.md`), NOT by a separate
BNF slot. A BC may be 1-N kebab-tokens; the registry tracks the full
kebab string as the canonical BC name. Example: a service might decompose
into `policy-domain`, `policy-evaluator-domain`,
`policy-evaluator-cedar-domain` — three distinct BCs (`policy`,
`policy-evaluator`, `policy-evaluator-cedar`) each registered with their
own `name`/`owner`/`rationale`/`adr_cite` row. The flat-BC model matches
the established Rust workspace pattern of `tokio-util`, `tonic-build`,
`hyper-util` — granularity via name, not via slot.

The 12 layer values:

- *Inner / pure layers (innermost-out)*: `kernel` (pure types + ports),
  `domain` (business logic on entities), `application` (use-case
  orchestrators), `app` (composition-root binary).
- *Outer / external layers*: `adapter` (trait impls + DTO mappers),
  `infrastructure` (frameworks & drivers / non-trait glue).
- *Presentation / entry-point layers (per protocol)*: `cli`, `rest`,
  `grpc`, `graphql`, `worker`, `sdk`.

Bounded-context is an open kebab-token registered in the living document
`docs/standards/bounded-contexts.md`. `api` is explicitly NOT a layer
value; every existing `*-api` crate is reclassified by its actual wire
format (`rest` / `grpc` / `graphql` / `worker` / multi-protocol split).

**Decision Drivers** (top 3):
1. Clean Architecture correctness self-enforces via Cargo + cargo-metadata
   (compile-time + CI-time dependency-direction gate via
   `check-architecture` LEAN-A1 orchestrator per §4a). The 12-value canonical enum makes
   every dependency edge unambiguously classifiable.
2. Hyperscaler precedent: AWS smithy-rs, Azure SDK for Rust, and Google
   Cloud Rust all encode the layer in the crate name; v4 imports their
   convention but extends it with Uncle Bob's strict 4-inner-layer
   taxonomy (kernel/domain/application/app) to match the team's existing
   DDD-aware vocabulary.
3. Decision tree (§2.2.4) eliminates taxonomy drift: every crate has a
   single canonical layer determined by code shape, not by historical
   naming. The audit produces deterministic classifications; reviewers
   cannot disagree on layer assignment.

**Alternatives Considered**:
- **Pattern A — v3 verbose BNF** (4–5 segments, fitness/freeze
  primitives). **Why rejected**: 31 compound-feature ADR rows; 6-segment
  AMBER tax; `fitness` jargon mismatch with team vocabulary; cannot
  cleanly parse load-bearing `tooling-agent-read`.
- **Pattern B — thing-domain literal** (always `oyatie-<bounded-context>
  -<thing>-<layer>`, no optional slot). **Why rejected**: forces a
  `<thing>` token where none semantically exists; pessimises common case;
  no hyperscaler analogue. Superseded by 2-slot final (Pattern G
  rejection drives 2-slot direction).
- **Pattern G — BNF with optional `<thing>` slot** (the v4-draft-1 to
  v4-draft-5 design). **Why rejected** (per third correction): the slot
  adds BNF complexity without semantic value. Granularity is better
  expressed by multi-token BC names matching the `tokio-util`/
  `tonic-build` Rust workspace pattern. User feedback: "thing is not a
  good name". The interim 2-slot BNF (draft-5) removed the slot entirely;
  the BC registry handles all granularity needs as multi-token kebab strings.
- **Pattern H — Axis-in-name with 5-value closed enum**
  (`oyatie-<platform|cloud|foundry|workspace|tooling>-<bc>-<layer>`).
  **Why rejected**: bloats every name with axis token; closed 5-enum
  conflates organizational axis (platform/cloud/foundry/workspace/
  tooling) with deployment vertical (cloud/foundry/workspace), forcing
  awkward classifications (e.g., `tooling` is neither platform nor a
  product vertical; `platform` is shared-infra-not-vertical). Per
  iter-2 supplement #1, axis metadata was first considered as a
  registry-only field (no BNF slot); supplement #2 then superseded
  that with the shared/vertical binary embedded in the BNF itself,
  giving the same navigation via `cargo metadata` + jq filter without
  closed-enum bloat. Final v4 BNF carries an OPEN verticals registry
  (`cloud`, `foundry`, `workspace`, plus future verticals) + the literal
  `shared` token; the verticals registry replaces what would otherwise
  be an axis closed-enum.

### Bounded context registry as a living document (per B3 + Codex C5)

`docs/standards/bounded-contexts.md` is a living-document registry with
required fields per BC entry: `name`, `kind` (closed: `shared` or
`vertical`), `vertical: <name>` (required iff `kind == "vertical"`;
value validated against `[workspace.metadata.oyatie.verticals]`), `owner`
(default `council-architecture`), `rationale` (1 paragraph), `adr_cite`
(one-line), `parent: <bc>` (optional, used when BC is prefix-child of
another BC per BC overlap governance below).

**BC overlap governance** (per Codex C5 + §15a fix 5):
- **Parent/child rule**: if BC A is a prefix of BC B (e.g. `policy`
  parent of `policy-evaluator`), B's registry entry records
  `parent: <A>`.
- **Sibling rule**: sibling BCs (`policy-evaluator` + `policy-cedar`)
  require explicit `rationale` proving non-overlap of ubiquitous
  language; LEAN-A2 flags them for manual review if Jaro-Winkler
  similarity > 0.85.
- **Tie-breaker procedure**: if two PRs propose conflicting BC names
  for the same crate cluster, council-architecture reviews both
  proposals. Primary tie-breaker: the proposal with the more specific
  `rationale` paragraph + ADR cite linking to upstream design
  discussion wins. **Deterministic ultimate tie-breaker** (per Codex
  C5): if rationale+ADR-cite specificity is comparable, earlier-PR
  timestamp wins.
- **xtask enforcement**: `cargo run -p check-bounded-contexts --
  --check-bc-overlap` runs lexical-prefix + Jaro-Winkler similarity on
  every BC addition; > 0.85 similarity triggers manual review
  (non-blocking advisory).

### Shared/vertical kind taxonomy + verticals registry (supplement #2)

The 3-slot BNF `oyatie-<shared|vertical>-<bc>-<layer>` encodes the
**shared/vertical kind taxonomy** directly in the crate name. Slot-2
token is either:
- The literal `shared` — BC depended on by anything; no vertical
  ownership; cross-cutting infra.
- A vertical name (open kebab token) registered in
  `[workspace.metadata.oyatie.verticals]` — BC scoped to one vertical;
  may be depended on only by same-vertical BCs or by `shared` BCs
  that do NOT depend on it.

**Verticals registry** (open set; lifecycle-aware per iter-2 prefold-A item 2):
```toml
[workspace.metadata.oyatie.verticals.cloud]
status = "active"        # active | deprecated | retired
owner = "council-cloud"
rationale = "Provider plane: compute, storage, network, IAM, KMS, billing, region, observability"
adr_cite = "ADR-0028"
public_layers = ["sdk"]  # see "Cloud vertical dual-role" sub-section
deprecated_at = ""       # RFC 3339 timestamp set on soft-deprecate
retired_at = ""          # RFC 3339 timestamp set when no crates remain

[workspace.metadata.oyatie.verticals.foundry]
status = "active"
owner = "council-foundry"
rationale = "Engineering platform: evidence, eval, capability registry, MCP gateway, ADR/audit/coverage probes"
adr_cite = "ADR-0025"
public_layers = []        # foundry is internal-only by default
deprecated_at = ""
retired_at = ""

[workspace.metadata.oyatie.verticals.workspace]
status = "active"
owner = "council-workspace"
rationale = "User-facing product axis: drive, mail, calendar, chat, meet, docs, sheets, …"
adr_cite = "ADR-0029"
public_layers = []        # workspace may opt-in to ["sdk","rest"] post-Shard-1 if it exposes public REST
deprecated_at = ""
retired_at = ""
# Future verticals (e.g., healthcare, corporate, fintech) added via registry append + ADR cite
```

**Verticals deprecation lifecycle** (iter-2 prefold-A item 2):

The `status` field carries one of three closed values:

- **`active`** — vertical accepts new BC registrations; cross-vertical
  refusal rules apply normally.
- **`deprecated`** — vertical is in a soft-deprecate window. No new BCs
  may register under this vertical (LEAN-A2 `--check-bc` refuses any
  scaffold whose `vertical` field points at a deprecated vertical).
  Existing crates continue to function. Transition `active → deprecated`
  requires an ADR amendment + sets `deprecated_at` to an RFC 3339
  timestamp. The deprecation window is **180 days** during which the
  team is expected to migrate existing crates to a successor vertical
  (or to `shared` if cross-vertical reusability becomes the goal).
- **`retired`** — vertical has zero crates referencing it. Transition
  `deprecated → retired` requires:
  (1) zero workspace members carry `[package.metadata.oya].vertical =
  "<this-vertical>"` (LEAN-A2 `--check-vertical-retirement` confirms),
  (2) the 180-day deprecation window has elapsed (`now - deprecated_at
  >= 180d`),
  (3) ADR amendment recording the retirement + setting `retired_at`.
  Retired registry entries are RETAINED for historical record; new
  crates referencing this vertical are refused.

**xtask enforcement** (`shared-bounded-contexts-check-cli` per the
3-slot pattern, currently scaffolded as `check-bounded-contexts`):
- `--check-bc` refuses BC scaffolding under `status != "active"`.
- `--check-vertical-retirement` validates the 180-day grace + zero-crate
  precondition before allowing `deprecated → retired` transitions.
- Status field transition itself happens via ADR amendment (manual
  governance gate); xtask validates POST-CONDITIONS, not the
  transition act itself.

**Dependency rules** (planned advisory LEAN-A2 check `check-bounded-contexts`):
- `shared` BCs depend only on other `shared` BCs.
- `<vertical>` BCs depend on `shared` BCs + same-vertical BCs only.
- **Cross-vertical deps REFUSED** at LEAN-A2 BLOCKER severity.
- Implementation: parse each crate's `cargo metadata` deps; classify
  source-slot-2 + target-slot-2; refuse edge if (source = `<X>`,
  target = `<Y>`, X ≠ Y, both ≠ `shared`).

**Audit translation from v3** (per supplement #2):
- v3 `platform-*` → `shared-*` (everything platform = shared)
- v3 `foundation-*` → `shared-*` (composition root is shared)
- v3 `tooling-*` → `shared-*` (dev tools are shared)
- v3 `cloud-*` → `cloud-*` (vertical preserved)
- v3 `foundry-*` → `foundry-*` (vertical preserved)
- v3 `workspace-*` → `workspace-*` (vertical preserved)

### Vertical naming policy (Option A: single-token verticals; Codex iter-2 D5)

The verticals registry restricts every entry's `name` to **exactly one
kebab token** (no hyphens). The `[workspace.metadata.oyatie.verticals]`
keys are single-token kebab strings such as `cloud`, `foundry`,
`workspace`, `healthcare`, `corporate`, `fintech`. Two-token vertical
names (e.g., `data-platform`, `cloud-edge`) are FORBIDDEN.

**Why Option A (single-token) over Option B (longest-registered-match
multi-token)**:
- Verticals are coarse-grained organizational product axes; single
  tokens like `cloud`/`foundry`/`workspace`/`healthcare` adequately
  name them and read naturally on the command line.
- Single-token verticals eliminate parser ambiguity between vertical
  and bounded-context tokens at slot 2/slot 3 boundary; the parser
  splits at the first hyphen after `oyatie-<slot2>-`. No need for
  "longest-registered-match" disambiguation pass.
- Granularity within a vertical is expressed via multi-token bounded
  contexts (slot 3), where granularity-by-name is already the
  established pattern (e.g., `cloud-storage-object-adapter`,
  `intelligence-policy-evaluator-domain`).

**Reserved literal**: `shared` is RESERVED as a non-vertical literal
in slot 2. The verticals registry MUST refuse any entry whose `name ==
"shared"` (LEAN-A2 `--check-vertical-name` enforces). This guarantees
the parser can unambiguously distinguish `shared-<bc>-<layer>`
(cross-vertical) from `oyatie-<vertical>-<bc>-<layer>` (vertical-scoped).

**xtask enforcement** (`check-bounded-contexts`):
- Reject any verticals-registry entry whose `name` contains a hyphen
  (multi-token vertical names refused).
- Reject any verticals-registry entry whose `name == "shared"` (reserved
  literal refused).
- Reject any crate whose slot-2 token is neither `shared` nor a
  registered single-token vertical name.

**Audit translation reminder** (v3 → v4 slot-2 mapping):
- v3 `platform-*` → slot 2 = `shared`
- v3 `foundation-*` → slot 2 = `shared`
- v3 `tooling-*` → slot 2 = `shared`
- v3 `cloud-*` → slot 2 = `cloud` (single-token vertical preserved)
- v3 `foundry-*` → slot 2 = `foundry` (single-token vertical preserved)
- v3 `workspace-*` → slot 2 = `workspace` (single-token vertical preserved)

### Protocol classification (Codex iter-4 F4; authoritative deferral target for `PROTOCOL-UNKNOWN` audit markers)

The 26 `*-api` crates from v3 axis era are deferred for `src/`-
inspection-driven protocol classification (one of `rest` | `grpc` |
`graphql` | `worker`). Per-crate classification evidence is cited via
the grep heuristic:

```bash
grep -l "axum::Router\|tonic::Server\|async_graphql::Schema\|tokio::spawn.*loop" \
  crates/<crate-name>/src/
```

executed against each crate's `src/` directory during iter-5 or
Shard 0 step 5b (the `src/`-inspection audit pass). Result mapping:

| Heuristic match | Classified layer |
|---|---|
| `axum::Router` or `actix_web::App` | `rest` |
| `tonic::Server::builder` or `#[tonic::async_trait]` impl | `grpc` |
| `async_graphql::Schema::build` or `juniper::RootNode::new` | `graphql` |
| `tokio::spawn { … long-running loop … }` without `Router`/`Server`/`Schema` | `worker` |
| Multiple matches in one crate | multi-protocol exception (see below) |

**Multi-protocol crate handling**: crates that ship multiple protocols
(e.g., REST control surface + gRPC OTLP data-plane on the same axum
+ tonic mux'd HTTP handler) MUST split into per-protocol crates
unless explicitly documented as a multi-protocol exception in this
ADR's Exceptions list. Multi-protocol exceptions require:

1. Explicit `[package.metadata.oya].protocols = ["rest", "grpc"]`
   declaration in the crate's `Cargo.toml`.
2. ADR-0056 amendment adding the crate to this sub-section's
   Exceptions list with rationale.
3. LEAN-A1 `layer-correctness` subcommand recognises the exception
   and skips the "unique protocol per crate" assertion.

**Currently deferred crates** (26 total marked `PROTOCOL-UNKNOWN` in
§3 audit per Codex iter-4 grep verification; the iter-3 closure
claimed 22, iter-4 corrects to actual grep count of 26):

- §3.1 platform: rows 6 (`tenant-api`), 8 (`identity-api`), 20
  (`object-graph-api`), 24 (`policy-cedar-api`), 26 (`regulatory-pack-api`)
  — 5 rows
- §3.2 cloud: rows 32 (`region-api`), 34 (`compute-vm-api`), 35
  (`compute-k8s-api`), 36 (`compute-functions-api`), 38 (`iam-api`),
  44 (`finops-api`), 49 (`kms-api`), 51 (`storage-object-api`), 52
  (`storage-block-api`), 55 (`network-vpc-api`), 56 (`network-dns-api`),
  57 (`network-lb-api`), 59 (`cloud-observability-api`) — 13 rows
- §3.3.1 foundry non-check: rows 60 (`foundry-api`), 72
  (`foundry-policy-api`), 73 (`foundry-registry-api`), 74
  (`foundry-rag-api`) — 4 rows
- §3.4 workspace: rows 115 (`chat-api`), 122 (`drive-api`), 125
  (`forms-api`), 128 (`meet-api`) — 4 rows

Total: 5 + 13 + 4 + 4 = **26 PROTOCOL-UNKNOWN rows** matching Codex
iter-4 grep verification.

**Hold-vs-inline policy**: these crates keep their current names
until classification completes. Two options for Shard 1 sequencing:

- **Option-Hold (default)**: complete classification audit in iter-5.
  Shard 1 cutover excludes the 26 `*-api` renames; ships them in a
  follow-up Shard 1.5 PR within 2 weeks of Shard 1 merge. Shard 1
  retains the 26 v3 `*-api` crate names temporarily; the §8.1
  zero-old-names global sweep gate is RELAXED for the 26 names
  during the Shard-1-to-Shard-1.5 window (deterministic relaxation
  recorded in ADR-0056 amendment).
- **Option-Inline**: iter-5 classification lands before Shard 0 closes,
  completing the 26 renames inline with Shard 1's atomic cutover. No
  Shard 1.5 follow-up needed; §8.1 zero-old-names gate enforces all
  144 renames in one atomic squash-merge.

**Decision criterion (deterministic)**: if iter-5 protocol-audit
completes ≥ 5 working days before Shard 0 scheduled merge date →
Option-Inline; else → Option-Hold. The decision is recorded in
`protocol-audit-sequencing`.

**Exceptions list (multi-protocol crates)**: empty at iter-4 close.
Populated by iter-5 audit if any crate's grep heuristic matches more
than one protocol marker without a clean split candidate.

### Cloud vertical dual-role + public_layers cross-vertical exemption (iter-2 prefold-A)

The `cloud` vertical occupies a load-bearing dual role: it is BOTH
**(a) the in-house compute substrate consumed by other verticals**
(foundry/workspace/etc. depend on cloud-IAM, cloud-storage, cloud-KMS
as the underlying infrastructure plane) AND **(b) the cloud product
sold to external customers** (`cli`/`rest`/`grpc`/`graphql`/`sdk`
layers within the `cloud` vertical = customer-facing product surface).

**Layer classification within `cloud` vertical**:

- `kernel` / `domain` / `application` / `adapter` / `infrastructure`
  layers = **internal substrate** consumed by other verticals AND used
  by cloud's own customer-facing layers. NOT directly callable from
  outside the `cloud` vertical except via a `public_layers` entry.
- `cli` / `rest` / `grpc` / `graphql` / `sdk` layers within `cloud`
  vertical = **customer-facing product surface**. Eligible for the
  `public_layers` exemption.

**`public_layers` cross-vertical exemption mechanism**:

The `[workspace.metadata.oyatie.verticals.<name>.public_layers]` field
documents which layers within a vertical are consumable cross-vertical.
For `cloud`, the initial allowlist is `["sdk"]` — `cloud-*-sdk` crates
are explicitly callable from any vertical because SDKs depend on
`kernel` only (pure types + traits) and are the public product surface.

Other verticals can opt-in:
- `workspace.public_layers = ["sdk", "rest"]` — if workspace exposes a
  public REST API.
- `foundry.public_layers = []` — foundry is internal-only by default.

**LEAN-A2 cross-vertical refusal honours `public_layers`**:
- Edge `vertical-A → vertical-B-<layer>` where `<layer>` ∈
  `B.public_layers` → ALLOWED (exemption applies). For SDKs the
  underlying dep `sdk → kernel only` rule already keeps the boundary
  clean; the exemption is purely an authorization-to-call signal.
- Edge `vertical-A → vertical-B-<layer>` where `<layer>` ∉
  `B.public_layers` → REFUSED.
- Edge `vertical-A → shared-X` where `shared-X` transitively reaches
  `vertical-B`: per the iter-2 prefold-A transitive-walker rule
  (LEAN-A2 spec rule 2.transitive), this is REFUSED unless the
  intermediate hop terminates at a `public_layers`-eligible target;
  i.e., the public-layer exemption applies AT EVERY cross-vertical hop
  in the chain.

**Concrete examples**:
- `workspace-drive-application` depending on `cloud-storage-object-sdk`
  → ALLOWED (cross-vertical edge but target layer `sdk` ∈
  `cloud.public_layers`).
- `workspace-drive-application` depending on `cloud-storage-object-domain`
  → REFUSED (target layer `domain` ∉ `cloud.public_layers`); the
  workspace crate must consume cloud-storage via the SDK or
  via a `shared` mediating crate.
- `intelligence-eval-application` depending on `cloud-storage-object-sdk`
  → ALLOWED (same reason; foundry is a separate vertical but cloud's
  SDK is on the public allowlist).
- `shared-audit-chain-domain` depending on `cloud-storage-object-domain`
  → REFUSED (`shared` cannot depend on any vertical, period; the
  public-layers exemption does NOT apply to `shared → vertical` edges
  because `shared` reuse semantics require complete vertical
  neutrality).

**Registry deprecation interaction**: when a vertical's `status`
transitions `active → deprecated`, its `public_layers` field is RETAINED
(existing cross-vertical consumers continue to function). When status
transitions `deprecated → retired` (and zero crates remain), the
`public_layers` field becomes vacuous (no crates to expose). The xtask
does not collapse or remove `public_layers` automatically; the registry
entry is retained for historical record.

### Build tooling vs. coordination primitives (Codex C7)

`cargo` (and its subcommands `cargo-deny`, `cargo-semver-checks`,
`cargo-nextest`, `cargo-doc`) is **build/test/lint tooling**, NOT a
coordination primitive in the ADR-0053 "three sanctioned coordination
primitives" sense.

- **Coordination primitives** (per ADR-0053 + ADR-0054) govern
  (rationale store + scaffold-claim windows), `shared-codeview-cli`
  (READ slot of the sanctioned triad). These primitives gate WHO can
  mutate the repo and WHEN.
- **Build tooling** (`cargo`+subcommands) executes WITHIN an existing
  workspace_members` may freely invoke `cargo build`, `cargo test`,
  `cargo deny check` without further coordination overhead; the
  coordination primitive guards the symbol, not the build commands
  that touch it.

Cross-reference: `docs/standards/multi-agent-tool-map.md:102-111`
(sanctioned-primitive triad definition) + `docs/standards/
agent-instructions-discipline.md:92-94` (direct-tool-invocation
rationale-row policy). Both documents are co-edited in Shard 0 (or
referenced unchanged if the cross-reference text already lines up; per
Codex C7 the reference is added rather than the underlying docs
amended — `cargo` non-coordination-primitive status is implicit in
both docs today and made explicit by this ADR-0056 sub-section).
- **Pattern C — drop-verb pattern** (`oyatie-<bounded-context>`, 2 segments
  only). **Why rejected**: cannot disambiguate domain layer from
  infrastructure layer of the same bounded context.
- **Pattern D — single `presentation` layer with protocol as second-
  to-last segment** (e.g., `tenant-grpc-presentation`). **Why
  rejected**: reads awkwardly — `grpc-presentation` is redundant. Adds
  a segment without semantic value. The 12-value closed enum directly
  names the protocol/wire-format instead, matching hyperscaler
  precedent.
- **Pattern E — collapse kernel+domain into single `domain` layer**
  (the 10-value enum from v4-draft-3). **Why rejected**: loses the DDD
  distinction between shared kernel (pure types + ports) and domain
  layer (business logic). The team's existing v3 inventory has many
  `*-kernel` crates; the 12-value enum keeps `kernel` as a distinct
  layer for pure-port crates and reclassifies logic-carrying kernels to
  `domain`, preserving semantic intent.
- **Pattern F — collapse application+app into single `application`
  layer**. **Why rejected**: composition-root binaries (deployable
  service main) are architecturally distinct from use-case orchestrators
  (library code that holds port-trait bounds). The 12-value enum keeps
  `app` as the canonical name for composition roots (matching Uncle
  Bob's "application" terminology for the deployable artifact) and
  `application` as the canonical name for use-case orchestrators.

**Why chosen**: 12-value closed enum bounds the noise while preserving
strict Uncle Bob + DDD canonical taxonomy; matches hyperscaler precedent
of naming by protocol/product, not generic `api`; canonical decision
tree (§2.2.4) eliminates ambiguity by giving a deterministic rule for
each crate; open bounded-context slot accommodates growth without
ceremony; clean-arch dependency direction self-enforces via Cargo + the
new `check-architecture` crate (LEAN-A1 orchestrator per §4a).

**Consequences**:
- Positive: simpler grammar; ~50 % fewer total grammar tokens vs. v3
  even with the 12-value enum (because the open `bounded-context` slot
  alone — no separate `thing` — replaces v3's `feature` + `capability`
  + closed context-enum machinery); the 3-slot BNF parses
  deterministically (split on `-`, last token is layer-enum, slot 2 is
  shared|vertical registry-validated, middle tokens are bounded-context);
  layer enum makes dependency direction
  self-enforcing; bounded-context registry is a living doc (Markdown,
  not TOML), so the team reads it as prose; per-crate layer assignment
  is deterministic via canonical decision tree.
- Negative: ~139 renames (3.8× v3's 37) plus one-time audit cost of
  explicitly classifying every crate by code shape per the canonical
  decision tree; some v3 `*-kernel` crates may relayer to `*-domain`
  (or vice versa) based on `src/`-inspection; each rename mechanical
  via xtask once classifications settle. Bounded-context naming drift
  risk (per Scenario A; mitigated by R10). Multi-protocol crates must
  split into per-protocol crates OR be documented as exceptions in
  ADR-0056 §"Bounded context registry".

**Follow-ups**:
- Bounded-context registry as a living document. Format: 1-paragraph
  rationale per entry; 90-day auto-deprecation for zero-crate entries.
- Promote dependency-direction enforcement from `check-architecture`
  (LEAN-A1 `dependency-direction` subcommand per §4a) into a
  workspace-level Cargo lint (post-Shard-1 ADR, out of scope for v4).
- WASM / lambda / function presentation layers are NOT added to the
  initial 12-value enum — out of scope for the current oyatie surface.
  Future ADR can extend the enum if needed (closed enum is amendable,
  not eternal; the precedent for amending the enum is set by ADR-0056
  itself, which finalized the layer enum at 12 within iter-1 after
  draft-1=6, draft-2=9, draft-3=10).
- Per-crate `src/`-inspection audit must classify each crate by the
  canonical decision tree; this is one of the top-3 expected Codex
  iter-1 pressure-test surfaces (§10 question 1).

## §12 ADR-0057 outline (supersedes ADR-0055; drops fitness/freeze/expedite)

**Status**: Proposed in Shard 0 commit; flips to Accepted at end of
Shard 1. **Supersedes**: ADR-0055.

**Decision**: Drop fitness/freeze/expedite terminology and primitive
machinery from the workspace. Carry forward Hybrid C topology, xtask spec
(now in v4 §3 + §5), `lockfile-rename` subcommand, 4-layer branch
pipeline, deterministic acceptance gates. Replace v3's
symbol-lock authority (per ADR-0054 amendment); the rename-cutover
window.

**Decision Drivers**:
   primitive (ADR-0053 + ADR-0054).
2. Inventing a parallel lane primitive duplicates effort and creates a
   maintenance surface that the team will not exercise (one
   rename-cutover per year, at most).
3. Token-rotation race in v3 §6 R2 (mitigated but not eliminated) is a

**Alternatives Considered**:
  fragile under concurrent merge-queue dequeues.
- Replace with GitHub branch-protection rule. **Why rejected**: doesn't
  emit auditable trail; doesn't compose with non-merge agents.

**Consequences**:
- Positive: drops 1 fitness lane crate from v3 (`governance-
  rename-cutover protocol must update.

**Follow-ups**:
  `docs/standards/git-workflow.md §3` (the cutover-bootstrap window
  section); cross-reference ADR-0057.

## §13 Reference inventory (full per Critic edit #9 from v3, ported forward)

**Source**:
- 140 `crates/oyatie-*/Cargo.toml` (all get `[package.metadata.oya]` per
  §3.0 simplified schema; ~139 get rename + `[lib]` update)
- 1 root `Cargo.toml` (members list + simplified workspace metadata block)
- `Cargo.lock` (single regen via `xtask lockfile-rename`)

**CI / scripts** (carried forward from v3 §12):
- `.github/workflows/release-evidence-pack.yml` (1 site)
- `.github/workflows/governance-supply-chain.yml` (2 sites)
- `scripts/check.sh` (~29 sites)
- `scripts/hooks/pre-push-repoctl.sh` (1 site)
- `scripts/check-architecture-boundaries.sh` (3 sites + 1 new for
  `codeview-cli`)

**Standards / decisions**:
- `docs/standards/clean-architecture.md` §3 (row 35 named-by-identity)
- **`docs/standards/clean-architecture.md:99-103`** (NEW per Codex C2):
  explicit edit changing the wording "ports defined in domain" to
  "ports defined in kernel" + cross-reference to ADR-0056 §"Decision".
  This is a load-bearing amendment because the v3 standard placed
  ports in the `domain` layer (per `clean-architecture.md §2.1`
  "domain — Workflow orchestration over kernel types. Defines ports:
  Rust traits…"); v4 places ports in `kernel` (pure types + ports).
  Shard 1 step 9a co-edits this in the same atomic commit.
- `docs/standards/crate-naming-convention.md` (rewritten under v4 OR
  marked Superseded by ADR-0056 — decision: rewrite)
- **`docs/standards/code-style-rust.md`** (NEW per Codex iter-2 D6) —
  lines **11-12, 137-147, 162-177** still declare the v3 BNF + 9-value
  role enum (`kernel/domain/app/api/worker/adapter/runtime/cli/sdk`).
  Shard 1 step list (§5.2) adds a co-edit to rewrite these line ranges
  under the v4 3-slot BNF + 12-value layer enum + canonical decision
  tree, with explicit ADR-0056 cross-reference. Without this co-edit,
  `code-style-rust.md` becomes the authoritative-conflict surface
  Codex iter-2 D6 flagged — v3 BNF references survive in `code-style`
  while every other standard is updated.
- `docs/standards/bounded-contexts.md` (NEW, the living-registry doc;
  per supplement #2 carries `kind` + `vertical` + `parent` fields)
- **`docs/standards/multi-agent-tool-map.md:102-111`** + **`docs/standards/agent-instructions-discipline.md:92-94`** (NEW
  per Codex C7): cross-reference target for ADR-0056 §"Build tooling
  vs. coordination primitives" — confirms that `cargo` (+ cargo-deny,
  cargo-semver-checks, cargo-nextest) is build/test/lint tooling, NOT
  a coordination primitive in the ADR-0053 "three sanctioned primitives"
  sense.
  block for v4: "Amendment 2026-05-13: rename-event scaffold-claim
  authority (v4)" — carries the rename-event scaffold-claim authority
  to the v4 rename, replacing v3's amendment text)
- `docs/adr-archive/ADR-0056-rust-clean-architecture-bnf.md` (NEW, BNF + bounded-context-registry)
- `docs/adr-archive/ADR-0057-cutover-mechanics-rename-plan-v4.md` (NEW, supersedes ADR-0055)
- `docs/ADR-INDEX.md` (new rows for ADR-0056 + ADR-0057)
- `docs/CHANGELOG.md` (rename entry)
- `docs/plans/rename-plan-v3-2026-05-12.md` (frontmatter update:
  `status: Superseded`, `superseded_by:`)

**Registry**:
- `registry/quality/lanes.yaml` (replaces "fitness lane" entries with
  "check rule" entries)
- `registry/docs/pipeline.tsv`
- OpenAPI bindings under `registry/openapi/`
- Release supply-chain refs under `registry/release/`
- `registry/release/supply-chain/tooling-cli-dev-runtime.yaml` →
  `dev-cli.yaml` (row 139 expansion)
- `registry/release/0.1.0/tooling-cli-dev-runtime.spdx.json` →
  `dev-cli.spdx.json`
- `registry/release/0.1.0/tooling-cli-dev-runtime.cyclonedx.json`
  → `dev-cli.cyclonedx.json`
- GHCR image ref `ghcr.io/oyatie/tooling-cli-dev-runtime` →
  `ghcr.io/oyatie/dev-cli` (Release Engineering confirms in Shard 0)

**Doc / team / product**:
- `AGENTS.md`, `docs/CONSTITUTION.md`, `docs/TOOLCHAIN.md`,
  `docs/RELEASE-MANAGEMENT.md`
- `docs/research/hyperscaler-best-practices-2026-05-12.md`
- Product/team docs under `docs/teams/`, `docs/products/`
- `.omc/governance-lanes/` directory (decision in Shard 0 per §10 question 3:
  rename to `.omc/check-rules/` atomically; or hold the doc rename for a
  post-Shard-1 follow-up — defer to Codex iter-1 pressure-test)

**Crate tests + source** (row 139 expansion, per v3 EDIT-7):
- `crates/tooling-cli-dev-runtime/tests/gate_cli.rs` (lines 2830,
  2868, 2879, 3456, 3465, 3471, 3472) — xtask rewrites string literals
- `crates/tooling-cli-dev-runtime/tests/repoctl_cli.rs` (149, 159)
- `crates/tooling-cli-dev-runtime/src/commands/repoctl.rs:43`
  (default value of `cli_manifest_path`) — runtime default, MUST update

**Check crate set** (replaces v3 `.omc/governance-lanes/`):
- 6 new check crates scaffolded fresh in Shard 0; populated in Shard 1
  (see §1, §3.3.2 footer)
- 29 fitness crates renamed to `check-*` namespace per §3.3.2

## §14 Cross-references

- **Superseded plan**: [`docs/plans/rename-plan-v3-2026-05-12.md`](rename-plan-v3-2026-05-12.md)
  (v3, Hybrid C with v3 BNF; status flips to `Superseded` in Shard 0).
- **Audit inventory**: [`docs/audits/convention-audit-2026-05-12.md`](../audits/convention-audit-2026-05-12.md).
- **Crate-naming-convention v3**: [`docs/standards/crate-naming-convention.md`](../standards/crate-naming-convention.md)
  (rewritten under v4; see §13).
- **Clean architecture standard**: [`docs/standards/clean-architecture.md`](../standards/clean-architecture.md)
  (§3 row 35 named-by-identity co-edit).
- **Bounded contexts registry (NEW)**: `docs/standards/bounded-contexts.md`
  (authored in Shard 1).
- **Git workflow**: [`docs/standards/git-workflow.md`](../standards/git-workflow.md)
  (§2-3 sanctioned-primitives; §3 cutover-bootstrap window).
  (amendment in Shard 0 covers rename-event scaffold-claim authority for v4).
  immediate metadata cutover, locked 2026-05-12 ~23:00 ET; carries forward
  to v4).
- **Open questions ledger**: [`/Users/jasonlee/oyatie/.omc/plans/open-questions.md`](../../.omc/plans/open-questions.md)
  (v4 iter-1 section appended in Shard 0).

---

## §3.0 `[package.metadata.oya]` schema (FINAL post-Codex-iter-2 D3; 3-slot grammar; `thing` references PURGED)

```bnf
metadata-oya       ::= "[package.metadata.oya]" NL
                       "name             = " name-str NL
                       "vertical         = " vertical-str NL
                       "bounded_context  = " bc-str NL
                       "layer            = " layer-str NL
                       "purpose          = " purpose-str NL
                       [ "audit_chain      = " bool-str NL ]
                       [ "feature          = " feature-str NL ]
name-str           ::= "\"oyatie-" vertical "-" bounded-context "-" layer "\""
                     | "\"check-" rule-name "\""
vertical-str       ::= "\"shared\""                                  ; reserved literal (cross-vertical)
                     | "\"" single-kebab-token "\""                  ; single-token vertical name registered in [workspace.metadata.oyatie.verticals]
bc-str             ::= "\"" kebab-bc "\""                            ; 1..N kebab tokens
purpose-str        ::= "\"" free-text "\""                           ; required key per LEAN-A1 metadata-schema subcommand
feature-str        ::= "\"" kebab-feature "\""                       ; optional presentation-protocol subkey for cli/rest/grpc/graphql multi-protocol BCs
layer-str          ::= "\"kernel\"" | "\"domain\"" | "\"application\""
                     | "\"app\"" | "\"adapter\"" | "\"infrastructure\""
                     | "\"cli\"" | "\"rest\"" | "\"grpc\"" | "\"graphql\""
                     | "\"worker\"" | "\"sdk\""
bool-str           ::= "true" | "false"
single-kebab-token ::= [a-z] [a-z0-9]*                               ; single token; no hyphens; per ADR-0056 §"Vertical naming policy"
```

For check crates, the block omits `vertical` and `bounded_context`,
substitutes `check_rule = "<rule-name>"`. The xtask handles both shapes.

**Required keys for non-check crates** (per Codex iter-2 D3):
`vertical` (literal `shared` OR single-kebab vertical name registered
in `[workspace.metadata.oyatie.verticals]`) + `bounded_context` (kebab,
multi-token allowed) + `layer` (one of 12 canonical values) +
`purpose` (free text; 1-sentence summary). **Optional keys**:
`audit_chain` (bool; defaults `false`), `feature` (presentation-
protocol subkey for cli/rest/grpc/graphql multi-protocol BCs).

**Difference from v3 §3.1 + earlier v4 drafts**: drops v3 `context`
(closed 6-enum), `feature` (was required), `capability`, `compound`;
drops draft-1–draft-5 transitional `thing` field (NEVER landed in
production — purged from all v4 prose per D3). Keeps `vertical`
(new in 3-slot grammar), `bounded_context`, `layer`, `purpose`
(required) + `audit_chain`, `feature` (optional). The simplification
matches the v4 BNF's narrower closed-set surface (just `layer` + the
12-value enum, plus the open-but-registry-validated `vertical` slot).

## §3.0a Workspace-level metadata block (root `Cargo.toml`)

```toml
[workspace.metadata.oya]
layers = ["kernel", "domain", "application", "app",
          "adapter", "infrastructure",
          "cli", "rest", "grpc", "graphql", "worker", "sdk"]
# bounded_contexts is auto-populated by xtask from per-crate fields:
bounded_contexts = [
  "address-book", "audit-chain", "billing", "calendar", "capacity",
  "cell", "chat", "cloud-data", "cloud-observability", "codeview",
  "collab-runtime", "composition", "compute", "data-boundary",
  "dcops", "dev", "dlp", "docs", "document-format", "drive",
  "dsr", "ediscovery", "eventing", "finops", "forms",
  "foundry", "foundry-adapter", "foundry-api-semver", "foundry-bypass",
  "foundry-capability", "foundry-cargo-prefix", "foundry-catalog",
  "foundry-cloud-mutation", "foundry-evidence", "foundry-eval",
  "foundry-mcp-gateway", "foundry-mdbook", "foundry-openapi",
  "foundry-policy", "foundry-rag", "foundry-registry", "foundry-run",
  "foundry-step", "iam", "identity", "kms", "mail", "marketplace",
  "meet", "metering", "mobile", "network", "notes", "object-graph",
  "observability", "policy-cedar", "recordings", "region",
  "regional-pack", "regulatory-pack", "residency", "resource",
  "retention", "secrets", "sheets", "sites", "slides", "storage",
  "surface", "tasks", "tenant", "translate", "trust-portal",
  "retention-dsr",
]
```

Adding a bounded context to the field is a 0-ADR action (the xtask
infers it from per-crate fields and rewrites the workspace registry on
every `--apply` run). Updating the doc `docs/standards/bounded-contexts.md`
to add the corresponding 1-paragraph rationale is required (per R10
mitigation) and enforced by the §8.2 bounded-context-registry-consistency
gate.

---

## §15 Architect iter-1 conditions closure block (fold-A, 7 conditions all CLOSED)

Per architect iter-1 verdict against v4-draft-5, 7 conditions were
returned. All 7 are folded into v4 per the comprehensive fold-A AND
third-correction (2-slot BNF) below. File:line cites confirm closure.

| # | Condition | Severity | Status | File:line cite (closure) |
|---|---|---|---|---|
| B1 | Port traits placement — confirm `kernel` (not `domain`) per canonical decision tree | HIGH | CLOSED | §2.2.1 row "kernel" — "**Pure types + ports (traits) only.** ZERO business logic."; §5.1 step 7a (B1 closure); §5.2 step 15a (clean-architecture.md amendment in atomic Shard 1 commit); ADR-0056 §"Decision" notes port-location move from `domain` → `kernel` |
| B2 | `*-api` evidence — replace `rest (provisional)` with file:line cites or explicit deferral marker | HIGH | CLOSED | §3 audit-row preamble — `layer_evidence` column added; "no row may ship as `provisional`"; §10 surface 2 enforces per-crate evidence audit at 139-crate scale |
| B3 | BC arbitrator — name council-architecture as default; add tie-breaker procedure | MODERATE | CLOSED | §5.1 step 7b (B3 arbitrator clause embedded in `docs/standards/bounded-contexts.md` skeleton); §11 ADR-0056 §"Bounded context registry as a living document" extended with B3 tie-breaker procedure |
| B5 | Reviewability at 139-rename scale — 3 parallel reviewer streams | HIGH-ish | CLOSED | §6 R11 — 3 partition streams (1a platform / 1b cloud / 1c foundry+workspace+foundation) with per-partition sign-off; §8.1 gate "B5 — 3 partition sign-offs collected" enforces via `gh pr view` heuristic |
| B6 | §5.2 step 15 ordering — atomic BLOCKER flip without chicken-and-egg | LOW | CLOSED | §5.2 step 15 — 3-substep avoidance: (a) `--report-only` during Shard 1, (b) post-merge §8.2 BLOCKER-flip PR within 24 h, (c) MISTAKES-LEDGER for any gap-window violations; §8.2 gates "B6 closure — 11-check BLOCKER flip atomicity" + "B6 closure — Mistakes ledger sweep" |
| B7 | BNF accommodation for proc-macros / codegen / fixtures / library+binary | LOW | CLOSED | §5.1 step 7c — ADR-0056 §"BNF accommodation" authored. Canonical layer assignments: proc-macros = `kernel` (types output) OR `infrastructure` (codegen machinery), pick by output; codegen crates = `cli` (run as tool) OR `infrastructure` (linked at compile time), pick by invocation; test-fixture crates = live as `tests/` subdirs by default, OR as `dev-dependency`-only crate at layer = `kernel` with `purpose = "test-fixture, dev-only"` annotation; library+binary split-the-crate rule by default unless binary is trivial (<100 LOC non-import lines), enforced by xtask check |

**Cumulative fold state**: 7 architect-iter-1 conditions CLOSED + 11
codification rules ADDED + `<thing>` slot DROPPED (third correction) +
12-layer canonical enum + canonical decision tree (§2.2.4) + 100-entry
BC registry. v4 plan is now consensus-locked-equivalent for iter-1 fold;
ready for Codex critic iter-1 review against this final state.

---

## §15a Codex iter-1 ITERATE-7 edits + consistency fixes closure block (iter-2 fold-B)

Per Codex critic iter-1 verdict against v4-iter-1-fold-A: ITERATE-7
(7 required edits + 6 consistency fixes). All folded into v4 in-place
during this iter-2 session.

### Codex iter-1 edits C1–C7 — all CLOSED

| # | Edit | Status | File:line cite (closure) |
|---|---|---|---|
| C1 | §3 audit drops `thing?` column; every row gets `vertical` + `bounded_context` + `layer` + `layer_evidence` + `bc_registry_status`; replace `rest (provisional)` with evidence cite OR `PROTOCOL-UNKNOWN` deferral marker | CLOSED | §3 audit-row column directive — "FINAL post-iter-2-supplement-2, 3-slot grammar — `thing?` column DROPPED; `vertical` column ADDED"; §5.1 step 15c rework rule + audit-translation table |
| C2 | §13 reference inventory adds explicit standards-amendment commit for `docs/standards/clean-architecture.md:99-103` (port location: domain → kernel) + ADR-0056 cross-reference; Shard 1 step list co-edit | CLOSED | §13 "Standards / decisions" inventory — new bullet `docs/standards/clean-architecture.md:99-103`; §5.2 step 15a (B1 closure already enforces this in atomic Shard 1 commit; C2 is the §13 inventory entry that locks the co-edit) |
| C3 | Stale `thing` cleanup: §3.0 metadata schema removes `thing`; §8.1 references replaced with 3-slot grammar regex; LEAN-A1 `naming-collision` tuple is `<shared\|vertical>-<bc>-<layer>` | CLOSED | §3.0 metadata-oya BNF — `thing` row dropped; §4a LEAN-A1 `naming-collision` subcommand spec — "tuple uniqueness on `<shared\|vertical>-<bc>-<layer>` per 3-slot grammar"; §3 column directive |
| C4 | Rename arithmetic: 140 current crates + 4 new check crates = **~144 crate-name-affecting ops** (corrects iter-1-fold-A claims of 145 with 6 checks / 151 with 11 checks); update §1 scope summary + §3.6 totals + §9 effort estimate | CLOSED | §1 "Estimated renames + new check crates = total crate-name ops" row — explicit ~144 ops accounting; §3.6 audit summary cross-checks; §9 estimated-effort Shard 1 row reflects ~140 renames + 4 check scaffolds |
| C5 | BC overlap governance: parent/child registry rule (BC prefix → `parent: <A>` field); sibling-rule rationale requirement; deterministic timestamp tie-breaker; Jaro-Winkler > 0.85 manual-review trigger | CLOSED | §11 ADR-0056 §"Bounded context registry as a living document" — full BC overlap governance section; §4a LEAN-A2 — `--check-bc-overlap` subcommand; §5.1 step 7b — registry skeleton includes `parent: <bc>` field |
| C6 | Reviewer stream 1c rebalance: 3 streams → 4 streams (1a platform/shared, 1b cloud, 1c foundry, 1d workspace+tooling+hotspots-reviewer-lead); 4 partition sign-offs to merge | CLOSED | §6 R11 — "4 parallel reviewer streams" with explicit 1a/1b/1c/1d split; §8.1 gate "B5/C6 — 4 partition sign-offs collected" enforces |
| C7 | §11 ADR-0056 adds §"Build tooling vs coordination primitives": `cargo`+subcommands are build/test/lint, NOT coordination primitives in ADR-0053 sense; cross-ref multi-agent-tool-map.md:102-111 + agent-instructions-discipline.md:92-94 | CLOSED | §11 ADR-0056 §"Build tooling vs. coordination primitives" — full section with cross-references; §13 reference inventory bullet for both doc:line targets |

### Consistency fixes 8–13 — all CLOSED

| # | Fix | Status | File:line cite (closure) |
|---|---|---|---|
| 8 | ADR-0056 Alternatives keeps Pattern G (`<thing>` slot rejected) + adds Pattern H (axis-in-name 5-enum rejected) | CLOSED | §11 ADR-0056 Alternatives Considered — Pattern G + Pattern H both present; Pattern H rationale "Bloats every name with axis token; user explicitly chose flat" |
| 9 | LEAN-A1 `layer-correctness` heuristic adds explicit allowlist for trivial impls in `kernel`-layer crates (`Default`/`Display`/`Hash`/`const fn`/getter fns ✓; any non-trivial logic → relayer to `domain`); syn AST classifies each fn body | CLOSED | §4a "LEAN-A1 layer-correctness heuristic with A9 allowlist" — explicit allowlist list + AST-classification note |
| 10 | LEAN-A1 `dependency-direction` excludes dev-deps + `[target.cfg(test).dependencies]` from enforcement (kernel/domain may have tokio in dev-deps for integration tests) | CLOSED | §4a "LEAN-A1 dependency-direction allowed-set" — explicit dev-deps exclusion note; §8.1 LEAN-A1 gate row mentions exclusion |
| 11 | LEAN-A4 classifier output schema pinned: `{"violations":[{"crate":"...","severity":"ERROR\|BASELINE-RESET\|...","kind":"...","detail":"..."}],"schema_version":"1.0"}` | CLOSED | §4a LEAN-A4 — "Pinned output schema" block; §8.1 LEAN-A4 gate row repeats the schema |
| 12 | LEAN-A1 `lockfile-parity` subcommand distinguishes rg exit codes: exit 1 (no match) = pass; exit ≥ 2 (rg error) = fail | CLOSED | §4a LEAN-A1 `lockfile-parity` subcommand spec — exit-code discipline note; §8.1 LEAN-A1 gate row references §15a fix 12 |
| 13 | §1 scope summary line: "~144 crate-name-affecting ops (140 renames + 4 new check crates); 100+ BC registry entries; 4 lean check crates." | CLOSED | §1 "Estimated renames + new check crates = total crate-name ops" row + "Bounded contexts identified" row + "New `check-*` crates" row all reflect the iter-2 arithmetic |

### Iter-2 supplement #1 + #2 supersession trail

- **Supplement #1** (metadata-only axis with 5-value closed enum) was
  authored mid-iter-2-fold-B but SUPERSEDED before any audit-row edit
  landed. Marker for historical record only; no v4 artefact carries
  the 5-value axis enum.
- **Supplement #2** (3-slot BNF + open verticals registry +
  shared/vertical-kind dependency rule) is the FINAL iter-2 fold
  state. All sections in v4 now use 3-slot grammar
  `oyatie-<shared|vertical>-<bc>-<layer>`.

### Final iter-2 state cross-references

- §2.1 BNF — 3-slot grammar
- §3 audit — 3-slot column schema; `vertical` column; LEAN-A1
  `layer_evidence` cite
- §4a — 4 lean check crates (LEAN-A1 through LEAN-A4)
- §5.1 steps 7 + 7b + 7d — 4-check scaffold + BC registry + verticals
  registry
- §6 R11 — 4 partition reviewer streams
- §8.1 LEAN-A1–LEAN-A4 gate rows
- §8.2 4-BLOCKER-flip atomicity
- §11 ADR-0056 §"BC overlap governance" + §"Shared/vertical kind
  taxonomy + verticals registry" + §"Build tooling vs. coordination
  primitives" + Pattern H rejection
- §13 standards co-edit inventory — clean-architecture.md:99-103 +
  multi-agent-tool-map.md:102-111 + agent-instructions-discipline.md:
  92-94

v4 plan is now consensus-iteration-2 locked-equivalent; ready for
Codex critic iter-2 review against the post-fold-B state.

---

## §15b Codex iter-2 ITERATE-7 closure block (postfold-A, D1–D7 all CLOSED)

Per Codex critic iter-2 verdict against v4-iter-2-fold-B + prefold-A:
ITERATE-7 (7 execution-consistency edits). All folded into v4 in-place
during this iter-2 postfold-A session.

| # | Edit | Severity | Status | File:line cite (closure) |
|---|---|---|---|---|
| D1 | §3 audit row rewrite — 3-slot column schema; drop `thing?`; add `vertical`+`kind`+`layer_evidence`+`bc_registry_status`; update proposed_names to 3-slot; replace `rest (provisional)` cells with evidence cite OR `PROTOCOL-UNKNOWN` deferral marker | BLOCKING (biggest gap) | CLOSED — schema rewrite landed; per-row body inspection deferred to iter-3 with explicit open-item | §3 audit preamble "Columns (FINAL post-Codex-iter-2 D1)"; §3.1–§3.5 header rows rewritten to 11-column 3-slot schema (verified via `grep -c "current_name | vertical | bounded_context | kind | layer | layer_evidence"` = 5); §3 v3-axis→v4-vertical translation rule documented; iter-3 open-item #1 in `.omc/plans/open-questions.md` records the per-row inspection surface |
| D2 | Arithmetic consistency — §1 (144) vs §3.6 (139); sync to "140 existing + 4 new = 144" matching §1 | LOW | CLOSED | §3.6 audit summary rewritten — explicit subtotal "139 existing + 4 new check crates = 144 ops"; reconciles 28+31+22+29+26+3 = 139 to Cargo.toml 140-row ground truth via `cloud-data-kernel` bucket note + `foundation-app` accounting |
| D3 | §3.0 metadata schema — purge `thing` references; required keys `vertical`+`bounded_context`+`layer`+`purpose`; optional `audit_chain`+`feature` | LOW | CLOSED | §3.0 metadata-oya BNF rewritten; `name-str` reads `oyatie-<vertical>-<bc>-<layer>`; `vertical-str` accepts literal `shared` or single-kebab token; "Difference from v3" prose updated to acknowledge `thing` was a transitional draft-only construct PURGED from v4 production prose |
| D4 | §8.1 LEAN-A2 gate row — explicitly walk transitive deps + per-hop `public_layers` check + FULL-chain violation output | MEDIUM | CLOSED | §8.1 LEAN-A2 gate row extended with explicit (ii) direct cross-vertical refusal + (iii) transitive cross-vertical refusal with per-hop public_layers check + (vi) violation-output format `a → x → y → b` with per-node `{kind, vertical, layer}` annotation; §6 R11a previously codified the rule, §8.1 now codifies the gate-command behaviour |
| D5 | BNF ambiguity at `:282` — Option A: BAN multi-token verticals; reserve `shared` as non-vertical literal | BLOCKING | CLOSED | §2.1 BNF `vertical ::= kebab-token` (single token); `shared-or-vertical` declared "reserved non-vertical literal"; new §11 ADR-0056 §"Vertical naming policy" enumerates Option A rationale + xtask enforcement (reject hyphenated vertical names + reject `name == "shared"` registrations) |
| D6 | §13 reference inventory — add `docs/standards/code-style-rust.md` lines 11-12, 137-147, 162-177 as Shard 1 co-edit (still declares v3 BNF + role enum) | MEDIUM | CLOSED | §13 reference inventory adds `docs/standards/code-style-rust.md` bullet with explicit line ranges; §5.2 step 10b added to enforce the co-edit; iter-3 open-item #3 records the surrounding-context-damage pressure test |
| D7 | `.omc/plans/open-questions.md` — refresh; drop `<thing>` / "2-slot final" references; document iter-2 state + lane name rename | LOW | CLOSED | `.omc/plans/open-questions.md` — new `## rename-plan-v4 iter-2 postfold-A — 2026-05-13` section appended; explicitly REPLACES iter-1 entries for current-state reference; documents D1–D7 closure + iter-3 open items |
| Stray | `check-bounded-context-registry` → `check-bounded-contexts` at §6 R10 line 1327 | LOW (Codex stray) | CLOSED | §6 R10 lane row updated; "renamed from iter-1-fold-A's singular ... per Codex iter-2 stale-name fix" note added inline |

### Final iter-2 postfold-A state cross-references

- §2.1 BNF — single-token vertical grammar; `shared` reserved literal
- §3 audit preamble — D1 column schema + v3-axis→v4-vertical translation rule
- §3.0 metadata schema — D3 thing-cleanup; `vertical` required key
- §3.6 audit summary — D2 arithmetic synced to 144 ops matching §1
- §5.2 step 10b — D6 code-style-rust.md co-edit
- §6 R10 — stray check-crate-name fix (singular→plural)
- §6 R11a — transitive cross-vertical refusal
- §8.1 LEAN-A2 gate row — D4 explicit transitive walker + public_layers + chain output
- §11 ADR-0056 §"Vertical naming policy" — D5 Option A
- §13 reference inventory — D6 code-style-rust.md addition

v4 plan is now Codex-iter-2 closure complete; ready for Codex critic
iter-3 review against the post-D1–D7 state.

---

## §15c Codex iter-3 ITERATE-5 closure block (iter-3-fold, E1–E5 all CLOSED)

Per Codex critic iter-3 verdict against v4-iter-2-postfold-A:
ITERATE-5 (5 execution-consistency edits). Root cause: iter-2 closure
rewrote schema headers + directives but did NOT regenerate the body
rows. Iter-3 pairs every directive with concrete execution.

| # | Edit | Severity | Status | File:line cite (closure) |
|---|---|---|---|---|
| E1 | §3.1–§3.5 row-by-row regeneration to 11-column 3-slot schema; replace `rest (provisional)` with evidence cite OR `PROTOCOL-UNKNOWN` deferral | BLOCKING (root cause) | CLOSED | §3.1 rows 1-28 + §3.2 rows 29-59 + §3.3.1 rows 60-82 + §3.4 rows 112-137 + §3.5 rows 138-140 — all 110 rows regenerated to `current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected`. Zero `rest (provisional)` cells; every `-api` row carries `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` in `layer_evidence` + `proposed_name`; every kernel/app/adapter row carries `STUB-pending-iter-4-src-inspection` in `layer_evidence` |
| E2 | §3.6 arithmetic — display 140+4=144 not 139+4=144 | LOW | CLOSED | §3.6 audit-summary row "Subtotal existing crates renamed: **140**"; total row "**140 + 4 new = 144**"; reconciliation prose rewritten to explain the visible-row-numbering (28+31+22+29+26+3 = 139 visible-row-numbers across the 5 sub-tables = 140 unique crates matching Cargo.toml ground truth) |
| E3 | Frontmatter `purpose:` rewrite to 3-slot grammar; delete "2-slot" language + "<thing> slot considered in earlier drafts is REMOVED" | LOW | CLOSED | frontmatter lines 19-30 — `purpose:` block declares 3-slot grammar `oyatie-<shared\|vertical>-<bounded-context>-<layer>`; previous "2-slot" wording deleted; deleted stale "Codex iter-1 should regenerate 2-slot tables" directive at §3 :620-631 (replaced with superseded note pointing to iter-3 E1 regeneration) |
| E4 | Stale §4a A4 heading rename — singular → plural | LOW | CLOSED | §4a "A4 — `check-bounded-contexts` (BC validation)" heading updated; explicit SUPERSEDED→LEAN-A2 annotation; §15b closure-table entry already documented the rename history, this E4 closes the heading itself |
| E5 | `.omc/plans/open-questions.md` honest-claim correction + iter-3 closure section | LOW | CLOSED | `.omc/plans/open-questions.md` — `## rename-plan-v4 iter-3 fold — 2026-05-13` section appended with E1–E5 closure cites; iter-2 postfold-A section's "all references purged" claim corrected via inline HONEST-CLAIM-CORRECTION block listing 4 surfaces NOT actually purged at iter-2 (frontmatter, body rows, §3.0 prose comment, §6 R10 lane row); iter-3 fold corrects all 4 |

### Iter-3 D7-verification stale-reference sweep

Per Codex iter-3 edit 7, final grep sweep verifies these patterns are
absent except in explicit history/superseded sections:

- `` `thing?` `` (column header) — surviving occurrences are in §3
  preamble historical note + §15c E1 closure cite + §15b D1 closure
  cite (history; OK).
- `` `<thing` `` (BNF slot reference) — surviving occurrences are in
  iter-2/iter-3 history blocks (§11 ADR-0056 Pattern G rejection,
  §15b closure tables, prefold-A history; OK).
- `platform-` (v3 axis prefix) — surviving occurrences are in
  audit `current_name` column (rename-source documentation; MUST stay)
  + audit-translation-rule prose (explicit history reference; OK).
- `rest (provisional)` — surviving occurrences in audit BODY ROWS:
  ZERO (E1 closure verified per `grep -c "rest (provisional)"` against
  §3.1–§3.5 body rows; only iter-3 prose mentions of the deprecated
  marker remain).

### Final iter-3 fold state cross-references

- §2.1 BNF — 3-slot canonical (Option A single-token verticals)
- §3 audit preamble — D1 column schema declared + E3-superseded note
- §3.0 metadata schema — D3 thing-cleanup retained from iter-2
- §3.1–§3.5 body rows — E1 regenerated to 11-col 3-slot
- §3.6 audit summary — E2 arithmetic 140+4=144
- §4a A4 heading — E4 plural rename
- §5.2 step 10b — D6 code-style-rust.md co-edit
- §6 R10 — D6 stray fix (singular→plural) retained
- §6 R11a — prefold-A transitive cross-vertical refusal
- §8.1 LEAN-A2 gate row — D4 explicit transitive walker + public_layers
  + chain output
- §11 ADR-0056 §"Vertical naming policy" — D5 Option A
- §11 ADR-0056 §"Cloud vertical dual-role + public_layers" — prefold-A
- §11 ADR-0056 §"Verticals registry" — prefold-A lifecycle
- §13 reference inventory — D6 code-style-rust.md addition retained
- frontmatter `purpose:` — E3 3-slot rewrite

v4 plan is now Codex-iter-3 closure complete; ready for Codex critic
iter-4 review against the post-E1–E5 state. Iter-4 pressure-test
surfaces (per open-questions): per-row `src/`-inspection evidence
population for the 110 STUB rows + per-`-api` protocol classification
for the 22 PROTOCOL-UNKNOWN rows + ADR-0056 §"Protocol classification"
sub-section authoring.

---

## §15d Codex iter-4 ITERATE-4 closure block (iter-4-fold, F1–F4 all CLOSED; APPROVE-ready)

Per Codex critic iter-4 verdict against v4-iter-3-fold: ITERATE-4
(4 mechanical-accounting edits). Iter-4 is the iteration cap (5th
consensus pass). All 4 folded; plan is APPROVE-ready.

| # | Edit | Severity | Status | File:line cite (closure) |
|---|---|---|---|---|
| F1 | §3.3.2 schema carve-out — Option A regeneration to 11-column 3-slot with check-namespace exemption | BLOCKING (schema inconsistency) | CLOSED | §3.3.2 header updated to `n = 29` + check-namespace-exemption preamble + table header rewritten to 11-column 3-slot + all 29 body rows (83-111) regenerated with `vertical/bounded_context/kind/layer: check-namespace-exempt | layer_evidence: NEW-scaffold-shard-1-from-v3-fitness-crate (rule-name <X>) | proposed_name: check-<X> | bc_registry_status: PROPOSED-NEW`. Grep verification: `grep -cE "^\| (8[3-9]\|9[0-9]\|10[0-9]\|11[01]) \|"` returns 29; `grep -c "check-namespace-exempt"` returns 32 (29 row lines + 3 prose mentions) |
| F2 | Arithmetic mechanical fixes — §3.3 header n=52, §3.3.1 header n=23, §3.6 reconciliation 28+31+23+29+26+3=140 | LOW | CLOSED | §3.3 header annotated "Codex iter-4 F2 fix — 23 non-check + 29 check = 52, was incorrectly stated as 53"; §3.3.1 header annotated "Codex iter-4 F2 fix — rows 60-82 are 23 rows, not 22"; §3.6 reconciliation prose rewritten to drop iter-3's "missing row" claim + show correct 28+31+23+29+26+3 = 140 arithmetic |
| F3 | STUB/PROTOCOL count actual sync via grep | LOW | CLOSED | `.omc/plans/open-questions.md` STUB markers block — 110 → 85; 22 → 26; verification commands documented inline (`rg -cE` patterns); iter-3 estimates explicitly marked inaccurate |
| F4 | ADR-0056 outline 3-slot rewrite + §"Protocol classification" sub-section authored | MEDIUM | CLOSED | §11 ADR-0056 Decision paragraph: `2-slot grammar` → `3-slot grammar `oyatie-<shared\|vertical>-<bounded-context>-<layer>`` (line 1650); new §"Protocol classification" sub-section authored before §"Cloud vertical dual-role" with: grep heuristic mapping table (axum/tonic/async-graphql/tokio loop), multi-protocol split-vs-exception policy with `[package.metadata.oya].protocols = [...]` declaration requirement, 26-row PROTOCOL-UNKNOWN deferred-crate enumeration (5 platform + 13 cloud + 4 foundry + 4 workspace = 26 matching body-row grep), Option-Hold-vs-Option-Inline sequencing decision criterion (5 working days threshold), Exceptions list (empty at iter-4 close) |

### Iter-4 final verification grep output (mechanical confirmation)

```bash
$ grep -cE "^\| [0-9]+ \|.*STUB-pending-iter-4-src-inspection" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
85

$ grep -cE "^\| [0-9]+ \|.*PROTOCOL-UNKNOWN" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
26

$ grep -cE "^\| [0-9]+ \|.*rest \(provisional\)" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
0

$ grep -c "2-slot" docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
16   # all surviving mentions are in historical/superseded sections

$ grep -cE "^\| (8[3-9]|9[0-9]|10[0-9]|11[01]) \|" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
29   # §3.3.2 body rows confirmed regenerated
```

### Iter-4 fold state cross-references (consolidated)

- frontmatter `purpose:` — 3-slot (E3 retained)
- §2.1 BNF — 3-slot canonical (Option A single-token verticals)
- §3 audit preamble — 3-slot column directive
- §3.0 metadata schema — `vertical/bounded_context/layer/purpose` required keys
- §3.1 rows 1-28 — 11-col 3-slot
- §3.2 rows 29-59 — 11-col 3-slot
- §3.3 header `n = 52` (F2)
- §3.3.1 header `n = 23` (F2); rows 60-82 — 11-col 3-slot
- §3.3.2 header `n = 29` + exemption preamble (F1); rows 83-111 — 11-col 3-slot with check-namespace-exempt markers
- §3.4 rows 112-137 — 11-col 3-slot
- §3.5 rows 138-140 — 11-col 3-slot
- §3.6 audit summary — 28+31+23+29+26+3 = 140 (F2)
- §4a — 4 LEAN check crates (LEAN-A1 through LEAN-A4)
- §5.1 — Shard 0 checklist including 4-check scaffold + verticals registry + IDE smoke gate + clean-architecture.md §99-103 amendment + code-style-rust.md co-edit
- §6 R10 — `check-bounded-contexts` (plural)
- §6 R11a — transitive cross-vertical refusal with full-chain output
- §8.1 LEAN-A1–A4 gate rows
- §8.2 4-BLOCKER-flip atomicity (count = 4)
- §11 ADR-0056 Decision — 3-slot (F4)
- §11 ADR-0056 §"Vertical naming policy" — single-token verticals (D5)
- §11 ADR-0056 §"Verticals registry" — active/deprecated/retired lifecycle with `cloud.public_layers = ["sdk"]`
- §11 ADR-0056 §"Protocol classification" — authored (F4)
- §11 ADR-0056 §"Cloud vertical dual-role + public_layers"
- §11 ADR-0056 §"Build tooling vs coordination primitives"
- §13 reference inventory — clean-architecture.md:99-103 + code-style-rust.md + multi-agent-tool-map.md + agent-instructions-discipline.md

### APPROVE-ready state

Plan is now APPROVE-ready post-iter-4. Remaining iter-5 work (per
open-questions iter-5 pressure-test surfaces) is **execution-phase
audit** — `src/`-inspection populating 85 STUB markers + 26 PROTOCOL-UNKNOWN
markers with file:line cites. That work runs INSIDE Shard 0 step 5b
(audit-only PR) OR as a pre-Shard-0 evidence-only PR. The plan itself
is consensus-locked-equivalent for iter-4.

Plan status: `pending approval`, `iteration: 2`, `consensus_loop:
v4-iter-4-fold`, `critic_iter_4: ITERATE-4 (folded; F1–F4 per §15d
closure block)`. User execution-approval is a separate gate downstream
of consensus-lock.

---

## §15e Codex iter-5 APPROVE-WITH-CONDITIONS closure block (iter-5-approve-fold, G1–G3 all CLOSED; PLAN APPROVED)

Per Codex critic iter-5 verdict against v4-iter-4-fold: APPROVE-WITH-
CONDITIONS (3 narrow 1-paragraph edits). All 3 folded same-session;
plan flipped to `status: approved`.

| # | Edit | Severity | Status | File:line cite (closure) |
|---|---|---|---|---|
| G1 | §3.6 summary table consistency — Foundry non-check cell 22→23; rows 60/72/73/74 narrative `-api → -rest` → `-api → PROTOCOL-UNKNOWN deferred` matching §3.3.1 body | LOW | CLOSED | §3.6 table row "Foundry non-check (vertical preserved) \| 23 \| 23 (rows 60, 72, 73, 74 rename `-api` → `PROTOCOL-UNKNOWN` deferred to ADR-0056 §"Protocol classification" pending iter-5 `src/`-inspection per body rows in §3.3.1; layer-suffix changes everywhere else) \| 0"; subtotal `28+31+23+29+26+3 = 140` arithmetic confirmed displayed (table row + prose match) |
| G2 | Active `2-slot` references purge to history-only | LOW | CLOSED | 4 active prose references rewritten: §2.1 BNF "supersedes draft-5 2-slot" → "supersedes draft-5 2-slot per iter-2 supplement #2" (retained as history marker); §1 "new under 2-slot" → "new under 3-slot, populating slot 3"; §4a A5-equivalent collision rule `<bounded-context>-<layer>` → `<vertical>-<bounded-context>-<layer>` with "3-slot rule" language; ADR-0056 Consequences "2-slot BNF easier to parse" → "the 3-slot BNF parses deterministically (split on `-`, last token is layer-enum, slot 2 is shared\|vertical registry-validated, middle tokens are bounded-context)". §11 Pattern E "rejected; superseded by 2-slot final" → "rejected; the `<thing>` slot was removed in v4 iteration sequence and the final v4 BNF settled at 3-slot". Final count: **15 `2-slot` mentions remain**, all in history/closure/superseded sections (verified by line-by-line audit) |
| G3 | Check-crate name normalization to 4-LEAN design — replace active `check-clean-architecture` with `check-architecture` matching §4a LEAN-A1; rewrite "11 new check crates" prose to "4 LEAN check crates" | LOW | CLOSED | 7 active-prose references normalized: §2.2.3 dep-direction text (lines 495, 516), §3.3.2 prose note about scaffold list (line 864-area), §5.2 step 15 (line 1403), §6 R4 (line 1414), §6 R9 (line 1419), §11 ADR-0056 Decision Drivers (line 1689), §11 ADR-0056 Consequences (line 2110), §11 ADR-0056 Follow-ups (line 2135). All now read `check-architecture` + reference appropriate LEAN-A1 subcommand. Final count: **3 `check-clean-architecture` mentions remain** (lines 872 history note, 1184 DROPPED marker, 1191 within DROPPED section); **1 "11 check crates" mention remains** (line 1332 inside historical `### A-summary` block already labeled SUPERSEDED). All retained references are explicit history. |

### Iter-5 final verification grep output (mechanical confirmation)

```bash
$ grep -c "2-slot" docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
15   # all in history/closure/superseded sections

$ grep -c "check-clean-architecture" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
3    # all in history sections (lines 872 normalization-note, 1184
     # DROPPED marker, 1191 within DROPPED section)

$ grep -cE "11 (new )?check crate" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
1    # line 1332 in historical ### A-summary block

$ grep -cE "^\| [0-9]+ \|.*STUB-pending-iter-4-src-inspection" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
85

$ grep -cE "^\| [0-9]+ \|.*PROTOCOL-UNKNOWN" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
26

$ grep "Foundry non-check (vertical preserved)" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
| Foundry non-check (vertical preserved) | 23 | 23 (...) | 0 |
```

### PLAN STATUS: APPROVED

Plan transitions: `pending approval` → **`status: approved`**;
**`pending: execution-approval-from-user`** (separate downstream gate;
plan is consensus-locked but user execution-approval is required
before Shard 0 opens). `consensus_loop: v4-iter-5-approve-fold`.

This is the FINAL fold of the v4 plan. No v5 will be created; v4 ships
as consensus-locked. Iter-5 was the iteration cap (5th consensus pass:
iter-1 architect + iter-1 critic + iter-2 critic + iter-3 critic +
iter-4 critic + iter-5 APPROVE-WITH-CONDITIONS).

Iter-5 cumulative closure summary:
- Architect iter-1: 7 conditions CLOSED (§15)
- Codex iter-1: 7 edits CLOSED (§15a)
- Codex iter-2: 7 edits CLOSED (§15b)
- Codex iter-3: 5 edits CLOSED (§15c)
- Codex iter-4: 4 edits CLOSED (§15d)
- Codex iter-5: 3 conditions CLOSED (§15e; this section) → APPROVED

Total: 33 review items folded across 6 review passes; zero deferred.

---

*End of plan. v4 CONSENSUS-LOCKED + status:approved per §15e; iter-5 pressure-test surfaces (Iter-5 in open-questions = execution-phase `src/`-inspection work) are NOT plan-iteration work and run inside Shard 0 step 5b OR pre-Shard-0 audit-only PR. v4-iter-4 APPROVE-ready per §15d closure block; iter-5
pressure-test surfaces (per open-questions §"Iter-5 pressure-test surfaces") are execution-phase audit work, not plan-iteration work, per §10
top-3 surfaces.*
