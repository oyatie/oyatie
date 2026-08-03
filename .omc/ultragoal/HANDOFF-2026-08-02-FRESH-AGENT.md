# HANDOFF — 2026-08-02 — written for an agent with ZERO prior context

## 0-LATEST-CODEX-PAUSE — 2026-08-02T10:04:50Z — READ THIS FIRST

**Newest state wins.** This block was written after the rest of this handoff. Where any older claim
below conflicts with this block, use this block and then re-measure the live repository/forge state.

### Terminal state: `PAUSED_PRESERVED`

The founder explicitly requested: put the work on a PR, preserve it, update this handoff with every
backlog and in-flight item, and pause. That request has been carried out.

- No PR was merged after the pause request.
- No deployment, rollout, cluster mutation, or production action was performed.
- The three in-flight read-only #1523 review lanes were interrupted before verdict; do not infer an
  approval or completed review from their existence.
- The signed W0-B -> W0-C -> W0-D stack is preserved remotely in draft PR **#1524**.
- The admission workflow started by preservation PR #1524 was intentionally cancelled after the
  signed head was safely pushed, so a do-not-merge pause artifact would not consume runner capacity.
- The aggregate ultragoal remains active. It was not marked complete or blocked merely because work
  paused.

### Exact live source and forge state at pause

`origin/dev` was freshly fetched and resolved to:

```text
e6230244ff472b35d8ee705e1d3606ce81131f6c
```

Never assume it remains there. Fetch and rebase/rehearse from the then-current `origin/dev` before
resuming any delivery action.

| PR | Exact head at pause | State at pause | Required-context evidence |
|---|---|---|---|
| [#1522](https://github.com/jason931225/oyatie/pull/1522) `fix(docs): regenerate ADR projections for ADR-0634` | `b04328f8416be8184dbf16508688071e06f61230` | OPEN, DRAFT, mergeable/CLEAN, no reviews | run `30740389793` completed **success**; `oya-ci-required` green |
| [#1523](https://github.com/jason931225/oyatie/pull/1523) `fix(reorg-codemod): make the oracle see the workspace it is migrating` | `1c308fa4843cb8487db5f8e94557418a054e4920` | OPEN, non-draft, mergeable but BLOCKED, no reviews | run `30740576873` completed **failure**; Buck2 and aggregate red |
| [#1524](https://github.com/jason931225/oyatie/pull/1524) `draft(reorg): preserve W0-C graph and W0-D reset work` | `b1c4664d0570f26fcf492dcd48499a7c21db5470` | OPEN, DRAFT, mergeable but BLOCKED; **PRESERVATION ONLY — DO NOT MERGE** | run `30742952590` intentionally **cancelled** after preservation |

The #1523 failure is not presently evidence that its codemod patch is wrong. The exact merge-ref
Buck2 run was cold (`Cache hits: 0%`) and failed one gate:

```text
root//ci/facade/cross-artifact-agreement:ci-cross-artifact-agreement-gate
71 passed; 2 failed
adr_index_projection_stale: docs/ADR-INDEX.md#row 14: - **Total ADRs:** 443
```

Those two failed assertions are the live ADR-0634 source/projection divergence that #1522 repairs.
Therefore the safe current admission order is **#1522 first**, then rebase/re-run/review #1523. This
does not weaken the older rule that #1523 must be protected-promoted before any destructive reorg
move execution.

Follow-up issue [#1520](https://github.com/jason931225/oyatie/issues/1520), `CI: make ADR source and
projection changes select parity admission`, tracks the affected-set false-green escape discovered
while repairing this train.

### Preserved signed stack and clean worktrees

The remote preservation branch is:

```text
draft/reorg-w0c-w0d-preservation-20260802
```

Exact signed linear lineage:

```text
e6230244ff472b35d8ee705e1d3606ce81131f6c (origin/dev at pause)
└── b04328f8416be8184dbf16508688071e06f61230 (W0-B)
    └── 3f5c4b8a014a3a0a54d99871baef7334c41bf85d (W0-C)
        └── b1c4664d0570f26fcf492dcd48499a7c21db5470 (W0-D / #1524 head)
```

All three commits were freshly verified as good SSH signatures before push. The cumulative diff
passed `git diff --check` and contains no `*.generated.json` path.

| Unit | Worktree | Branch | Exact head | State/evidence |
|---|---|---|---|---|
| W0-B | `/Users/jasonlee/Developer/oyatie-wt-w0b-adr0634-projection-20260802` | `fix/w0b-adr0634-projection-current-20260802` | `b04328f8416be8184dbf16508688071e06f61230` | clean; only `docs/ADR-INDEX.md` and `docs/machine-readable/decisions.json`; producer 443 records/ADR-0635 next; synthetic merge 73/73 |
| W0-C | `/Users/jasonlee/Developer/oyatie-wt-w0c-w0b-integration-rehearsal-20260802` | `rehearse/w0c-w0b-integration-20260802` | `3f5c4b8a014a3a0a54d99871baef7334c41bf85d` | clean; 19 units/five graph faces, request-only acyclicity, exact reverse max-min closure; unit 2/2, gate 11/11, cross 73/73, JSON/rustfmt clean |
| W0-D | `/Users/jasonlee/Developer/oyatie-wt-w0d-w0c-integration-rehearsal-20260802` | `rehearsal/w0d-w0c-integration-20260802` | `b1c4664d0570f26fcf492dcd48499a7c21db5470` | clean; reset evidence stays `eligible=false` and authorization disabled; JSON 8/8, reset unit 1, reset gate 13, affected-set 44, self-conformance 4, graph 2+11, public protocol 12, cross 73 |
| W0-E | `/Users/jasonlee/Developer/oyatie-wt-w0e-foundation-rehearsal-20260802` | `rehearsal/w0e-foundation-20260802` | `b1c4664d0570f26fcf492dcd48499a7c21db5470` | **clean and unmodified**; design exists, implementation never started |

W0-C and W0-D are rehearsal/preservation commits, not admissible final PR shape. #1522 remains the
authoritative W0-B review surface. After prerequisites promote, restack W0-C and W0-D as separate
serial protected PRs and obtain fresh exact-candidate evidence. Do not merge #1524 as-is.

W0-E design only, not implementation:

1. PR A would add a non-authorizing foundation/evaluator/schema/policy/evidence layer bound to exact
   promoted W0-B/W0-C/W0-D receipts, with status `ClosedEligible` and
   `authority_effect:none-until-masterplan-adoption`; it must not edit the masterplan.
2. PR B would be a data-only masterplan adoption preserving open/HOLD, `binding=false`,
   `dispatch=false`, empty waves, and `closed_state_evaluator_proof=false` until all requirements
   truthfully qualify.
3. No founder decision was required for that design, but implementation must use real promoted
   receipts, not the rehearsal commits in #1524.

The canonical checkout remains a preserved dirty workspace on
`preserve/hermes-w1-dirty-20260630`. It had **1766 porcelain paths** at pause. Do not switch, reset,
clean, stash, or commit there; the handoff itself is ignored local runtime state. Resume only in
isolated worktrees.

### Work that was underway when pause was requested

- #1523 exact-code review, architecture review, and admission verification were running read-only.
  All three were interrupted before final reports. Treat them as **no verdict** and rerun fresh after
  #1522 promotion/rebase.
- #1522 had finished protected CI green but remained draft and unreviewed. No ready-for-review or
  merge transition was made after the pause request.
- W0-C and W0-D implementation/rehearsal were complete locally and are now preserved in #1524, but
  their serial final PRs do not exist.
- W0-E had only an architecture design; its clean worktree has no edits.
- W1 was approved/planned but not executed. W2-W4 were queued as plan-only lanes. W5 was queued as a
  truthful boot-evidence lane. The systemic work below was added to the durable ultragoal backlog.

### Durable backlog added in this Codex session

The ultragoal now contains **37 goals**. `G019` remains the active aggregate; the newly appended
items `G023`-`G037` are pending. The ledger also records the Gajae/Bun/it-legal procedure discipline
and current admission-order evidence. Do not delete a rejected G035 steering ledger row: it is an
intentional audit receipt; the corrected G035 below is the accepted goal.

| ID | Backlog item | Required outcome / boundary |
|---|---|---|
| G023 | W1 execute approved cloud-kernel deletion | Only after #1523 protected promotion; delete the unowned 20-crate `cloud/cloud-kernel` framekernel from fresh `origin/dev`, preserve Asterinas and recovery tag, regenerate controller-owned projections, cold Buck2/control-differential verify, review/admit/observe. |
| G024 | W2 author intelligence remainder move plan | Add only `specs/reorg/intelligence-remainder-move-plan.json` for 78 surviving `oya/intelligence` crates; reconcile 51 moved crates and three executed plans; source/destination oracles and executable assertions; no move/shared-registry edit. |
| G025 | W3 disposition libs and author capability move plans | Graph all 129 flat `libs` crates; six-way disposition with confidence/competing homes; plan-only `libs-<capability>-move-plan.json`; no code move or shared-registry edit. |
| G026 | W4 author tools and oya product-tail move plans | Classify 21 tools crates plus surviving `oya` product/CI tail; keep active codemod stationary; plan-only executable move artifacts; no move/shared-registry edit. |
| G027 | W5 establish truthful Asterinas boot evidence | Obtain observed typed boot terminal result where possible; otherwise stop existing targets implying boot proof and land truthful refusal; keep adapters empty and avoid W1/hot-workflow collisions. |
| G028 | Measure ARC churn and resolve protected-CI queue capacity | Observe >=15 minutes; distinguish ephemeral rotation from dead registration; measure utilization/delay and choose the smallest evidence-backed trunk-priority or scaling repair without reverting #1509 or unsafely touching the shared console cluster. |
| G029 | Apply Gajae operational discipline to console development | Audit nine console crates; typed terminal outcomes, verified no-ops, deployed/user-observed evidence distinct from build success, recorded negative findings; preserve Console-local authority and Buck2-only Rust evidence. |
| G030 | Classify and reduce non-code artifact corpus | Disposition roughly 13,950 Markdown/YAML/JSON/TOML artifacts by consumer/authority; graph-wire maintained artifacts; delete/freeze dark bureaucracy; countable masterplan-v2 reduction with anti-vacuity assertions. |
| G031 | Implement per-field data-classification fact layer | Turn the idea document into Rust proc-macro/fact model, regime packs, and derived DPIA/RoPA/SOC2 projections; schemas-as-spec, inline obligations, RED/GREEN tests, graph wiring; replace 82 templated DPIAs only after parity/consumer proof. |
| G032 | Resolve friction-ledger lifecycle | Founder-level choice: machine-mint evidence-backed intake or freeze as provenance archive; preserve cited FRIC identifiers; no time-aging gate or added manual ceremony. |
| G033 | Resolve broken CODEOWNERS lane | Reverify routing, then delete or implement a genuinely effective replacement; reconcile all specs/masterplan/security/tests atomically; no silent zero-routing state. |
| G034 | Classify fixuptasks as backlog or inventory | Reverify `registry/fixuptasks.jsonl`; evidence-backed disposition backfill or truthful non-gating inventory/lifecycle; no ledger-for-ledger and no unactionable gate. |
| G035 | Resolve user-musl history retirement and supply-chain disposition | Founder-authorized choice: reversible collaborator-coordinated signed history purge, or permanent no-purge with enforceable distribution exclusion/source/license controls. Tree deletion alone cannot complete it. |
| G036 | Wire or retire governance check kernels | Reverify 56 kernels and consumers; wire real corpus into required Buck2 graph or retire dark kernels/rows; resolve `gates_root` limitation without laundering findings; born-blocking fixtures. |
| G037 | Make quality lanes binding or retire dark declarations | Audit all 93 quality-lane rows; replace retired CLI declarations with Rust/Buck2 targets or delete/stale-mark; prove every retained lane reaches the single required context with RED/GREEN tests. |

### Founder north-star product/protocol contract captured by the interview

Interview ambiguity was reduced to approximately 3% (below the requested 5%). Preserve these as
the default architecture decisions unless later founder authority explicitly changes them:

- Public synchronous APIs: **REST + OpenAPI 3.2**.
- Public event contracts: **AsyncAPI 3.1 + CloudEvents 1.0.2**, plus signed/versioned webhooks;
  SSE/WebSocket are permitted where interaction semantics require them.
- Do **not** expose public GraphQL, gRPC, gRPC-Web, or Connect.
- Internal service contracts: proto3 **gRPC over HTTP/2**, protected by **SPIFFE mTLS / TLS 1.3**.
- Edge transport: prefer **HTTP/3**, retain HTTP/2 fallback; transport preference is not an excuse
  to duplicate product semantics.
- **Protobuf** is the internal RPC schema contract. **FlatBuffers** is allowed only behind a measured,
  benchmark-gated adapter for a real zero-copy/latency gap; it is not a parallel default IDL.
- Network/telemetry baseline: **Cilium + Hubble + OpenTelemetry**. Custom eBPF requires a measured
  observability/operation gap, explicit safety bounds, and lifecycle ownership.
- Physical fabric remains provider-owned **BGP Clos/EVPN**. Do not build a switch OS/control plane.
- Defer SONiC/SAI, gNMI/gNOI, UEC protocols, RoCEv2, NVMe/RDMA, and Fibre Channel until a measured
  product requirement and ownership boundary justify them.
- Storage-fabric default: **NVMe/TCP first**; do not introduce RDMA complexity speculatively.
- Reorganization is capability-first and dependency-graph driven. Controller-derived move manifests
  and baselines are the source of operational truth; hand-maintained parallel projections are not.

### External procedure lessons that must shape execution, not become copied content

- **Gajae discipline:** typed terminal states; observe deployment/result, not just build; verify
  no-op and negative/non-finding outcomes; historical merge claims must use
  `completedAt < mergedAt`.
- **Bun migration discipline:** prove the source and destination oracle first; prove the oracle can
  see the destination being transformed; refuse unverifiable rewrites; do not create a dark
  temporary bridge; a successor pointer is an assertion that must be observed.
- **it-legal drafting procedure:** obligation -> requirement; inline citations; enumerate large
  surfaces as tables; schemas/assertions are the executable spec; separate pattern from concrete
  implementation; draft macro-to-micro as one coherent document. Reuse the procedure, not its legal
  content.
- Composite rule: no doctrine-only parallel coordinator. Improve enforceability, evidence lineage,
  graph wireability, placement mechanics, and observed outcomes inside the existing authority chain.

### Authority corrections for resumption

- The older W1 deletion plan suggests adding a new multispectrum evidence file. That is stale.
  Current `docs/AGENTS.md` retires that convention: **do not create new
  `evidence/multispectrum/*.json` files**.
- Never hand-edit or commit `*.generated.json`; use the sanctioned materializer/controller and let
  generated-output diff policy fail closed.
- Use safe isolated-worktree deletion mechanics. Never run cleanup against the canonical preserved
  checkout.
- `dev` branch enforcement remains explicitly deferred; do not silently turn this reorg into a
  branch-policy project.
- All final Rust evidence is Buck2-authoritative. Cargo-only evidence is insufficient.

### Exact resume order

1. Read `specs/root-hub-pointers.json`, `docs/AGENTS.md`, root authority, and this entire handoff.
2. Fetch and record fresh `origin/dev`; inspect #1522, #1523, #1524, #1520, reviews, mergeability,
   exact heads, and protected-run timestamps. Treat every state above as a pause-time snapshot.
3. Review #1522 on its exact candidate, mark ready only when appropriate, and squash-merge it only
   with resolved review, no conflict, and `oya-ci-required` green. Record exact postmerge dev green.
4. Rebase/restack #1523 on the new dev, rerun cold/protected evidence, and rerun the interrupted
   exact-code/architecture/admission reviews. Merge only after all conditions are satisfied.
5. Rebuild W0-C from the newly promoted chain, open a serial protected PR, review/admit/observe it;
   then do the same for W0-D. #1524 is a recovery source, never the final merge shape.
6. Implement W0-E only from real promoted W0-B/W0-C/W0-D receipts and preserve its two-PR
   non-authorizing/adoption split.
7. Start W1 only after #1523 promotion. W2-W4 plan authorship may parallelize when file ownership is
   disjoint. W5 is disjoint only while it avoids W1 and the hot required workflow.
8. Schedule G028-G037 from measured dependencies and conflict surfaces; keep a single writer for hot
   shared authority/projection files.
9. For every admitted PR, complete the postmerge packet: promoted exact-SHA required context green,
   rollout/no-deploy result, rollback note, observability check, browser/user-story evidence where
   applicable, release-governance impact, and observation harvest/cards or duplicate links.

**Pause stop condition is satisfied:** remote draft #1524 preserves the exact signed work; the local
worktrees are clean; the newest handoff records the work, backlog, blockers, and resume order; no
merge or deployment followed the pause request.

You are picking up mid-programme on the **oyatie** monorepo (Rust + buck2). This file is
self-contained: everything you need is here or is linked from here by absolute path. Read it top to
bottom once before touching anything.

**Runtime-neutral.** The prior work ran under Claude Code; this handoff is written to be executed by
any agent (Codex included). Nothing here requires that runtime — §3 restates all remaining work as
plain, self-contained units. Where a Claude Code artifact is mentioned it is **reference material
only**, always at an absolute path you can just read. Two conventions of that runtime that you should
know exist but do not need:
- `~/.claude/plans/*.md` — plain markdown plan files. Read them like any file.
- `~/.claude/projects/.../memory/MEMORY.md` — a durable memory index (~139 one-line entries pointing
  at sibling `.md` files). It is **not** auto-loaded for you; read it deliberately (§8) before making
  architectural claims, because it records prior measurements and founder rulings.

Repo entry surfaces that DO apply to you: root `CLAUDE.md` (authoritative project rules — read it),
`AGENTS.md`, and `docs/AGENTS.md` (the operating contract). Codex-specific overlays live in
`~/.codex/skills`, `~/.codex/agents`, and the tracked `.codex/` directory.

**State: nothing is half-applied.** The prior session stopped on an API quota limit before its last
batch of work executed (`agents_done: 0` — every unit errored on quota, none made an edit).
`origin/dev` is clean at **`e6230244f`**. There is no partial edit to clean up anywhere.

---

## 0. HARD SAFETY RULES — violating these destroys work

1. **The canonical checkout `/Users/jasonlee/Developer/oyatie` is a PRESERVED DIRTY WORKSPACE** on
   branch `preserve/hermes-w1-dirty-20260630` with **1598 dirty paths**.
   - NEVER switch its branch. NEVER commit there. NEVER `git checkout`/`reset` in it.
   - **NEVER trust a grep of it** — it does not reflect `origin/dev`.
   - Read canonical content with `git show origin/dev:<path>`, or make your own worktree off
     `origin/dev`.
2. **Never use `git stash`.** It is repo-global across worktrees; sibling agents have collided on it
   and nearly lost work.
3. **Never commit generated artifacts** — `*.generated.json`, `gate-baseline.generated.json`,
   materialized faces. `gate-baseline.signoff.json` is the ONLY human-edited founder-signed file.
4. **Never edit governed JSON/YAML with a serializer** — use targeted line edits. A reserializer
   reorders keys and produces an unreviewable diff.
5. **The Talos lab cluster is SHARED with the console project.** Check ownership before ANY
   cluster-scoped apply.
6. **OCI credentials live in OCI Vault, never `/tmp`.** For private keys, paste the path, never the
   contents.

---

## 1. Verification rules that this repo will punish you for skipping

The dominant defect class here is **machinery that was declared and never ran**. Nineteen dark gates,
a 2,700-line validator that only validated fixtures, 80 lane rows naming a retired command. Assume
any check you did not personally watch fail is not actually checking.

- **`buck2 test` can serve a STALE GREEN.** Quote the `Cache hits: %` line on every verification run.
  A suspiciously fast pass is suspect until proven cold. (Real case: a 4.1s "PASS" was cached; the
  real run took 265s.)
- **A bare local `buck2 test //ci/...` fails ~18 targets that pass in CI.** The required workflow
  materializes generated faces in an out-of-graph pre-step that a local invocation skips. **ALWAYS
  establish a pristine `origin/dev` control worktree and diff the failure sets** before attributing
  any failure to your change.
- **A suspiciously ROUND / TOTAL / ALARMING number means the probe is wrong until proven otherwise.**
  Confirm any load-bearing NEGATIVE with a second, differently-shaped probe.
- **Merge state must be measured with `completedAt < mergedAt`.** A present-tense
  `gh pr view --json statusCheckRollup` reports what checks say NOW, not at the merge instant. The
  previous session got this wrong twice. Measured properly: **only 3 of the last 30 merged PRs (10%)
  merged with the required context observed green.**
- **A relocation needs a WHOLE-TREE referrer sweep, not a cone check.** A previous reorg PR merged
  green and broke `dev` because a stale pointer lived outside the affected cone.
- **Every rule you write must ship its acceptance test inline** — the actual assertion, gate id, or
  config stanza, never a description of one. Prose specs are how the dark-gate problem happened.

---

## 2. Where things stand

`origin/dev` = **`e6230244f`**. Sixteen PRs merged 2026-08-02 (#1505–#1521).

### Open PRs

| PR | what | note |
|---|---|---|
| **#1523** | `fix(reorg-codemod): make the oracle see the workspace it is migrating` | **LAND THIS FIRST** — prerequisite for every remaining reorg move EXECUTION |
| #1522 | ADR projection regen for ADR-0634 | mechanical — **but it is a DRAFT and cannot merge until marked ready** |

### Repo shape

> Every number below was re-verified against `origin/dev` on 2026-08-02 by an independent pass that
> **found eight errors in an earlier draft of this file.** Re-verify anything you are about to act on;
> do not treat this table as more durable than the tree.

**25 top-level dirs carry the capability-first shape and hold 480 crates:** `iam` 68, `ci` 56,
`intelligence` 51, `workflow` 48, `os` 41, `comms` 24, `data` 23, `tenancy` 22, `audit` 18, `k8s` 18,
`billing` 17, `gateway` 10, `secrets` 10, `console` 9, `cell` 8, `compute` 8, `network` 8,
`storage` 8, `compliance` 7, `messaging` 6, `marketplace` 5, `iac` 5, `observability` 5, `kernel` 3,
`flags` 2. Plus **`governance/` 62 crates**, which the shape also uses.

Two precisions that matter:
- **They are not all "capabilities".** The registry holds **24 registered capabilities**. `os` and
  `kernel` are **META directories, not capabilities** — `os-move-plan.json` states this verbatim. The
  meta set is `governance` / `kernel` / `os` / `base` / `build` / `third-party` / `app`. Registered
  capability `policy` has **no directory at all**. This distinction is load-bearing: a dir that is
  not a registered capability cannot become top-level without a registry amendment.
- **Only 12 of the 25 have all four faces.** `os`, `kernel`, `flags` have `core/` only; `gateway` has
  `adapters/` only; `ci` has **no `core/`**; `marketplace` has `core/ facade/`. Do not assume the
  four-face shape when reasoning about a specific dir — look.

**⚠ `cloud/` does NOT disappear when `cloud-kernel` is deleted.** (An earlier draft claimed it did;
that was wrong.) `cloud/` holds **1478 tracked files across 22 subdirs**; `cloud-kernel` is 170 of
them. **1308 files in 21 subdirs survive** — `cloud-iac` 235, `tenancy` 211, `cloud-k8s` 176,
`cloud-secrets` 152, and more. The deletion removes cloud/'s last **crates**, not the root.

| remaining | crates | destination |
|---|---:|---|
| `cloud/cloud-kernel` | 20 crates + 1 nested workspace manifest | **DELETE** — approved plan, §4 |
| `oya/intelligence` | 78 | `intelligence/` (51 already landed) |
| `libs/` (flat) | 129 | disperse across capabilities — hardest block |
| `tools/` | 21 | `build/` meta-dir + capability homes |
| `oya/` product tail (`office` 19, `community` 14, `application` 8, …) | **90** | `app/` or own capability |
| `oya/ci-{webhook-gateway,controller,tide}` | 12 | `ci/` (56 already there) |

**350 crates remain** in the frozen legacy roots (cloud 20 + libs 129 + oya 180 + tools 21). That
figure is confirmed by a second, differently-shaped probe: `legacy_root_freeze.crates` in
`ci/facade/module-membership/capability-membership-policy.json` is **exactly 350** over
`frozen_roots [cloud, libs, oya, tools]`. So the earlier framing "not 351 crates to go" was
misleading — **350 crates genuinely do remain.** The defensible claim is that they move in **~6
batched moves, not 350 individual ones**, because moves are per-CAPABILITY: one PR (#1498) moved 56
crates at once.

**`specs/reorg/` holds 11 files, not 10.** Ten have zero surviving `old_path` — fully executed. The
eleventh, **`kernel-move-plan.BLOCKED.json`, has all 17 of its `old_path`s still present**: it is the
parked `cloud/cloud-kernel` → `kernel/` move that failed (§7.2). The §4 deletion plan lists deleting
that file. So the reorg is blocked on **plan AUTHORSHIP**, not execution.

**Why moves must be SERIAL:** #1498 touched **117 files outside the moved trees** (verified exact).
The breakdown, which an earlier draft got wrong: **63 `registry/` files** (the largest block, and
previously unmentioned), **30** in the `marketplace/facade/dev-cli` hub, 13 `specs/`, 5 `ci/`, 3
`docs/`, plus root `Cargo.toml` / `Cargo.lock` and the global policy JSONs. Every move hits the same
globals, so two concurrent moves conflict. **Plan AUTHORSHIP is read-only and parallelizes perfectly;
plan EXECUTION does not.**

---

## 3. THE WORK — five independent units, fully specified here

> **Note on provenance:** these were authored as prompts for a Claude Code `Workflow` run that died on
> an API quota limit before any unit executed. They are reproduced below as **plain work items with no
> runtime dependency** — ignore any reference elsewhere to a workflow script, `resumeFromRunId`, or
> lane letters; everything you need is in this section. (The original script, if you want to read the
> prompts verbatim, is at `~/.claude/projects/-Users-jasonlee-Developer-oyatie/`
> `25b3f8fe-ade0-43f6-9ff2-0e9311101703/workflows/scripts/finish-the-reorg-wf_4cfadaa6-1cd.js`.
> It is reference material only.)

### Ordering and concurrency — READ BEFORE STARTING

- **W1 is the only unit that writes global registries.** Root `Cargo.toml`, `Cargo.lock`,
  `capability-membership-policy.json`, `crate-catalog-coverage-policy.json`,
  `affected-set-policy.json`, and the `marketplace/facade/dev-cli` hub. **Never run W1 concurrently
  with any other unit that writes those files.**
- **W2, W3, W4 are READ-ONLY.** They author plan files under `specs/reorg/` and nothing else. They can
  run in parallel with each other and with W1, provided they touch no global registry.
- **W5 is disjoint** from all of the above.
- **Land PR #1523 first** (codemod nested-workspace oracle). It is a prerequisite for any move
  EXECUTION, though not for plan authorship.

### W1 — execute the cloud-kernel deletion

Full spec in **§4** plus the approved plan at `.omc/ultragoal/DELETION-PLAN-cloud-kernel.md` (a plain
markdown file; read it directly). It was built from two independent enumeration passes and every
count was verified against `origin/dev`. Verify each count as you apply it and report any drift.

### W2 — author the move plan for `oya/intelligence` (78 crates)

**PLAN FILE ONLY. Move nothing. Touch no global registry.**

Destination is top-level **`intelligence/`**. Verified on `origin/dev`:

| | crates |
|---|---:|
| `intelligence/` — registered capability (`specs/capability-registry.json:330`), faces in place | **51** |
| `oya/intelligence` — still to move | 78 |
| `cloud/cloud-intelligence` | **0 — does not exist** |

⚠ An earlier brief said the destination was `cloud-intelligence`. **That was wrong** — `cloud-` is
precisely the prefix the de-brand doctrine strips. **The generalizable rule: a destination recorded
before the reorg is a destination in the OLD shape.** Before quoting any stored destination, verify
the target directory exists today and that its name survives de-brand.

This is a **HALF-FINISHED** migration, not a new one. Study the 51 crates that already landed, plus
the three EXECUTED plans (`specs/reorg/intelligence-move-plan.json`,
`intelligence-sinkbatch-move-plan.json`, `intelligence-supervisor-move-plan.json`), and be continuous
with their naming, face assignment and de-brand mapping. If you think the existing convention is
wrong, say so explicitly rather than silently diverging. Report why 78 were left behind when 51
moved. Deliverable: `specs/reorg/intelligence-remainder-move-plan.json`.

### W3 — disposition `libs/` (129 flat crates) and author its move plans

**PLAN FILES ONLY. Move nothing. Touch no global registry.**

The hardest remaining block. `libs/` is a flat dumping ground; the capability-first shape has no
`libs/` in it, so every crate needs a destination capability AND a face.

A prior pass measured **38% inter-rater disagreement** on this disposition. Therefore:
- Emit an explicit **CONFIDENCE** per crate: HIGH (fan-in/fan-out and naming agree on one capability),
  MEDIUM, LOW (genuinely ambiguous).
- For LOW, state the competing destinations and what evidence would settle it. **A crate parked as
  "needs a ruling" is a better outcome than a confident wrong home.**

Method (use it, don't just assert): build the real dependency graph per crate; a crate used by exactly
one capability belongs to it; a crate used by 3+ is either a `base/` meta-dir candidate or a sign the
boundary is wrong; check `registry/catalog/*.yaml` and `specs/capability-registry.json` for an already
declared home; and for fan-out-0 crates ask whether they are dead or merely unwired — an earlier pass
dispositioned 74 fan-out-0 `libs` crates, so find that work and build on it rather than redoing it.

Deliverables: a disposition table (crate → capability → face → confidence → evidence) and move-plan
JSONs grouped by DESTINATION capability, named `specs/reorg/libs-<capability>-move-plan.json`.

### W4 — author move plans for `tools/` (21) and the `oya/` product tail (~83)

**PLAN FILES ONLY. Move nothing. Touch no global registry.**

**`tools/`:** the shape has a top-level `build` meta dir. For each tool decide: `build/`,
`governance/`, inside the capability it serves, or DELETE. Several are named `oya-governance-*-app`
and an earlier pass found some such crates dark — **check liveness (invoked by any lane, workflow, or
BUCK target?) before assigning a destination.** `tools/oya-reorg-codemod-app` is actively used and is
modified by open PR #1523 — plan around it, do not propose moving it yet.

**The `oya/` tail:** `office` 19, `community` 14, `application` 8, `itsm` 6, `payroll` 5, `hr` 5,
`crm` 3, plus 2-crate dirs (`warehouse`, `treasury`, `supply-chain-planning`, `real-estate`,
`quality-management`, `production-planning`, `plant-maintenance`) and singletons. Also
`oya/ci-webhook-gateway` 5, `oya/ci-controller` 4, `oya/ci-tide` 3 — **these are CI infrastructure,
not products**, and likely belong under `ci/` (which already holds 56 crates).

The decisive question, to answer with evidence: are these products each their OWN capability, or
compositions belonging under `app/<product>/` (the shape reserves `app/` for 2+-capability tenant
compositions)? **`specs/capability-registry.json` is CLOSED** — a product that is not a registered
capability CANNOT become a top-level dir without a registry amendment. State which are registered and
which are not; that distinction drives everything. List any registry amendment a plan would require.

### W5 — prove the kept Asterinas kernel actually boots

Independent of the reorg. Detail in §5 "Known-broken". Measured: 28 files name asterinas, **0 in
`.github/`**; the two QEMU-driving targets are `rust_binary` so `buck2 test` never runs them; the two
`rust_test` targets parse serial logs the harness itself writes (a closed loop); receipts are
gitignored and none exist on `dev`. **Verify all of that yourself first — if any of it is wrong, that
is the most valuable thing you can report.**

Then land the smallest real improvement with a RED fixture: make a test actually execute the QEMU
targets, or make the harness fail when the boot fails rather than parsing its own output, or commit a
real serial-log golden and assert against it. **If a genuine boot cannot run here** (owned runners are
arm64; Asterinas at the pin has no aarch64 backend and `BOOT_ARCH` is hardcoded x86_64), **say so
plainly and land the honest thing instead: stop the current targets implying they verify a boot when
they do not.** Separately, one small file: the pin declares
`covered_file_source_pointer_required_before_distribution: true` and nothing enforces it.

Do NOT touch `cloud/cloud-kernel` (W1 deletes it). Do NOT edit
`.github/workflows/oya-ci-required.yml`; a NEW workflow file is fine.

---

## 4. The cloud-kernel deletion — APPROVED, ready to execute

**Full plan: `.omc/ultragoal/DELETION-PLAN-cloud-kernel.md`.** Read it; it was built from two
independent enumeration passes and every count was verified.

**Decision (founder, 2026-08-02):** keep the adopted Asterinas substrate in `kernel/`; delete the
bespoke 20-crate framekernel at `cloud/cloud-kernel`.

**Not deleted for lack of quality** — it is 17,000+ lines with zero `todo!()`, two symmetric ISA
backends, a loom-model-checked `ksync`, and real QEMU captures. It goes because it has **no owner and
cannot be worked on**: editing any of its 50 graph-invisible `.rs` files trips `RefuseUnowned` (red
CI), its output dir is a zero-budget `scratch` class so it cannot commit its own evidence, and
**exactly one commit has ever touched its source** — the import `072a66f37`, 2026-06-10.

**Reversibility is secured.** Tag **`kernel-snapshot-2026-06-08` → `26173992778a`** is pushed and
verified. That commit was NOT an ancestor of `dev`, was untagged, and hung off a single branch ref —
one `git push --delete` would have lost it forever. It holds the bring-up harness (QEMU runners for
both ISAs, `assert-talos-boot`, `diff-oracle`, `check-tcb`) that `dev` never had. Separately
`072a66f37` remains an ancestor, so `git checkout 072a66f37 -- cloud/cloud-kernel` restores the
dev-shaped copy.

### Four traps, each worth a wasted ~60–75 min CI round

1. **Do NOT touch `REQUIRED_OWNED_STACK_LAYERS` / `REQUIRED_OWNED_STACK_LADDER_RUNGS`** in
   `ci/facade/cross-artifact-agreement/src/lib.rs`, nor the `specs/masterplan.json` rung. They hold
   masterplan **layer names, not paths** (two of six aren't directories at all). Removing the
   `cloud-kernel` entry shifts every later rung index → cascading RED. Edit anchor STRINGS only.
2. **`crate-catalog-coverage` is 20 rows, not 7.** The gate is NAME-keyed and 13 crates have bare
   names (`arch-aarch64-layout-tests`, `fsbase-worker-x86_64`,
   `user-{clock,exec,fsbase,hello,init,procinfo,signal,smpdemo,spawn}-x86_64`, `user-procinfo`,
   `user-smpdemo`). **No path grep will ever find them.**
3. **`git rm -r` leaves the directory behind** — local `.gitignore`s cover build output. Follow with
   an explicit `rm -rf cloud/cloud-kernel` or the tree survives untracked and re-trips path gates.
4. **`affected-set-policy.json` and `affected_set.rs` are ATOMIC** — a test reads the live policy off
   disk and asserts both `linker.ld` strings are present.

Also: `capability-membership-policy.json` is **producer-emitted** — regenerate, never hand-edit.

Acceptance numbers: manifests 891→871 · include sites 145→124 · hermeticity ceiling 21→0 ·
`uncatalogued` 197→177 · `legacy_root_freeze.crates` 350→330.

---

## 5. FULL BACKLOG

### Open

| item | state |
|---|---|
| **Execute the cloud-kernel deletion** | plan approved — §4 + `.omc/ultragoal/DELETION-PLAN-cloud-kernel.md`, not started (**W1**) |
| **Finish the reorg** | four units fully specified in §3 (**W2–W4**), plus W1 |
| **Prove Asterinas boots** | **W5**, §3 |
| **Dead runner registration + trunk queue priority** | see below |
| **Apply the Gajae operational lessons to CONSOLE development** | founder-directed, **never done** — see §6.4 |
| **Are ~13,950 non-code artifacts justified?** | founder question, deferred pending the code graph — **which now exists**, so it is answerable |
| **Per-field data-classification fact layer** | **spec only**, never built; 82 templated `dpia.md` copies remain |

**Runner detail — and a CORRECTED INFERENCE. Read this before acting on it.**

An earlier draft said: *"`oya-arm64-lh4ch-runner-8d9ds` is offline across 3 samples over 60s with a
stable name — therefore not ARC ephemeral churn, therefore a dead registration to reap."*
**That inference was WRONG, and it is a good example of the trap in §1.** A later check found all
three runner names had rotated (`-2l9mp`, `-blthc`, `-d5svb`), with a *different* one offline. Sixty
seconds was simply too short a window to distinguish a stuck registration from ARC's normal
terminate/replace cycle. **Do not go looking for `-8d9ds`; it no longer exists.**

What survives the correction: **effective capacity is 2 of 3** — one runner is offline whenever
sampled. What is NOT established: whether that is a stuck registration or the steady state of a
3-runner ephemeral pool (one always cycling). **Distinguish them before reaping anything:** sample
over ≥15 minutes and check whether the *set of names* turns over, and whether the offline slot is
always the same registration or a moving one. A moving offline slot = normal churn, nothing to reap,
and the answer is to scale the pool instead.

Separately and independently confirmed: the concurrency fix (#1509) **removed an accidental
throttle**. Pending eviction used to cap trunk to one live run; a 59s burst of 7 pushes now yields
7 runs × 9 jobs = 63 jobs contending with PR jobs for 2 runners (measured 44 queued / 0 running).
Trunk verdicts now exist but arrive slowly. Decide: trunk queue priority, or scale the fleet.
**Do NOT revert #1509** — an unobservable trunk was strictly worse than a slow one.

### Founder decisions outstanding

1. **The friction ledger.** Its GATE is healthy (10/10, in the required lane). Its **corpus is dead**:
   last row 2026-06-21, **498 commits since**, 119 of 189 frictions still open. A lane deliberately
   declined to ship an aging gate — time is not a declared buck2 input, so it would be non-hermetic or
   ceremony pinned to "may be ~500 commits stale". Choose: **(a)** automate intake so rows are minted
   by machinery, or **(b)** demote it to a frozen provenance archive. It is currently treated as (a)
   while behaving as (b). Do not delete: 186 FRIC ids are cited across ~40 commits and several ADRs.
2. **`.github/CODEOWNERS` deletion.** Verified: **111 unknown-owner errors against 111 references**
   (100%), `@teams/*` cannot resolve on a user-owned repo, and PRs #1498–#1507 carry 0 reviews.
   Converting to the sole collaborator also routes zero while REMOVING the error signal — strictly
   worse. ADR-0634 D4 recommends DELETE with an acceptance test. Not executed: it retires a registered
   lane, orphans a gate crate, and touches 5 specs + masterplan + security-program.
3. **`registry/fixuptasks.jsonl`** — 420 rows, 291 open, **203 (70%) with no disposition field**.
   Cannot be gated as-is: the baseline would exceed the entire friction ledger's open set — a ledger
   of the ledger. Needs a disposition backfill (human judgement) or reclassification as an inventory.
4. **History purge of `user-musl.elf`** — 620 KiB unlicensed third-party static musl in
   `cloud/cloud-kernel/out/`, absent from `deny.toml`, `oss-stewardship-registry.json` and
   `dependency-rationales.json`. `git rm` removes it from the tree, **not from history**. The
   sanctioned purge path `registry/history-only-retirement/control-plane.json` is `HOLD(Planning)`,
   `dispatch_authorized: false`. Unblock that control plane first if a purge is wanted.

### DEFERRED by founder — do not pursue

**Dev branch enforcement** (was task #10). Founder, 2026-08-02: *"we are going to defer dev branch
enforcement for now."* Analysis is preserved in **ADR-0634** (merged, #1518) with a concrete PUT
stanza in D8. Measured state at deferral: the entire `required_pull_request_reviews` object is
**absent** (so nothing requires a change to arrive as a PR at all), `enforce_admins: false`,
`rulesets: []`, and 2/30 recent PRs merged with the required context green.

### Known-broken, filed but not scheduled

- **`governance/check/**` — 56 crates observe ZERO repo artifacts in CI.** All 56 kernels are pure by
  design (no `fs::read`/`WalkDir`/`include_str!` anywhere); the corpus harvest lives in
  `oya gate validate <name>`, which **no CI lane invokes**. 57 fixture tests pass while observing
  nothing. Class fix blocked: `ci/facade/gate-self-conformance` has `gates_root` as a single string
  `"ci/facade"`, so widening it emits ~112 blocking findings at once and needs a merge-authority
  decision first.
- **Nothing on `dev` proves Asterinas boots.** 28 files name it, **0 in `.github/`**; the two
  QEMU-driving targets are `rust_binary` so `buck2 test` never runs them; the two `rust_test` targets
  parse serial logs the harness itself writes (a closed loop); receipts are gitignored. Needs an
  amd64+KVM substrate — owned runners are arm64, and Asterinas at the pin has **no aarch64 backend**
  (`BOOT_ARCH` hardcoded x86_64, upstream arm64 PR stalled). Lane E.
- **`kernel_side_adapters: []` must stay EMPTY.** The zero-maintenance property of the Asterinas
  adoption lives there, not in Asterinas. The first adapter starts a ~201-commit-per-release upgrade
  treadmill against an MPL-2.0 upstream requiring published modifications. **That** is the real
  one-way door.
- The Asterinas pin declares `covered_file_source_pointer_required_before_distribution: true` and
  nothing enforces it. One file.
- **No `oya gate validate <lane>` runs in required CI at all.** The workflow executes only
  `buck2 test //ci/...` plus 3 `//libs` targets — zero `//governance/...`, zero `//marketplace/...`.
  All 93 `registry/quality/lanes.yaml` rows are declared but not executed by merge authority.

---

## 6. Founder directives in force

Quoted where the exact wording matters. These are STANDING unless explicitly retired — several
predate 2026-08-02 and are still live.

### 6.1 Decisions made 2026-08-02

- **"debrand kuberos"** — kuberos SURVIVES (you do not de-brand what you delete) and must never appear
  as a brand in a path. That ruled out both `kuberos/` and `kernel/kuberos/`. Superseded in practice
  by the delete decision, but **the naming rule stands for everything else**.
- **Keep Asterinas, delete kuberos** (§4).
- **Dev branch enforcement DEFERRED** — do not pursue; analysis preserved in ADR-0634.
- **"finish the reorg"** and **"parallelize any other work that can be parallelized from backlog"**.

### 6.2 Standing method directives — these govern HOW you work

- **"don't add or maintain unnecessary bureaucracy in our pipeline."** A gate that measures nothing,
  a registry nobody reads, a ceremony nobody performs: **DELETE it or make it computed.** Do not add
  a process step to compensate for a missing mechanism.
- **Adversarial review of anything hand-rolled.** Verbatim: *"hand rolling those is probably
  hyperscaler anti-pattern and anti-best practice. unmaintainable and adds unnecessary friction.
  their existence itself should be questioned as well and whether it should exist in its current
  form."* So the question is never only "is this correct?" but **"should this exist at all, in this
  form?"**
- **"make autonomous decisions aligned on our north star, and hyperscaler monorepo best practices and
  hyperscaler cloud best practices. system design and architecture, scalability, engineering
  excellence researched online."** Research the external precedent; do not reason only from what is
  already in the repo. Cite what you found.
- **"how long a task takes is not an issue. do a thorough job."** Do **not** optimise for a small
  diff or a fast answer. Thoroughness beats speed, explicitly.
- **Disposition is a six-way choice, not a binary.** Verbatim: *"some may be stale, some may be
  irrelevant now, some may need a reorg, some refactor, some rewrite"* + *"some plain delete."* When
  assessing any artifact, pick from **{keep · reorg · refactor · rewrite · delete · stale-mark}** —
  "it exists so keep it" is not an answer.
- **Keep track of the backlog.** The founder has twice asked *"are you keeping track of backlog
  work?"* and *"make sure you don't lose track of backlog work and context."* Maintain the task list;
  file what you find rather than carrying it in your head.
- **Research, don't trust.** Verify claims — including claims in this handoff, in memory files, and
  in ADRs — against the live repo before acting on them.

### 6.3 Standing product/architecture directives

- **NORTHSTAR: "once the reorg is done, we want everything in code graph and build graph for full
  visibility."** This is the goal the whole reorg serves. Graph-wireability is the disposition test:
  an artifact that cannot be graph-wired and that nobody will claim is a DELETE candidate.
- **"where new or existing crates belong should be almost mechanical and easy to classify"** +
  *"follow hyperscaler pattern here as well."* **Directly governs the reorg lanes** — if placing a
  crate requires a judgement call, the SHAPE is wrong, not the placer. Aim for a mechanical rule.
- **"we must draw linters and namecheck boundaries that we have against hyperscaler best practices
  and patterns. it must be durable pattern that is exactly as a hyperscaler would implement them"** —
  and, when asked to go further: *"not just name keys. the entire process, procedures, protocol,
  pipeline, and nuances."* Boundary enforcement must be modelled on real hyperscaler practice
  end-to-end, not just on naming conventions.
- **owned_stack_policy** — the whole stack owned in Rust; upstream k8s/Talos are transitional behind
  stable interfaces.
- **All CLI surfaces are retirement-marked.** Merge authority lives in the cloud-ci gate apps behind
  the single required context `oya-ci-required`; operations ride the console + API. Legacy
  `oya-dev-cli` invocations are local bridge feedback only, **never merge authority**.
- **Policy engine = Cedar (PBAC) + Zanzibar (ReBAC)** merged into one owned decision plane.

### 6.4 Directives with work still OUTSTANDING — do not lose these

| directive | state |
|---|---|
| *"Apply these lessons ... as well as **console development**"* (the Gajae lessons) | **NEVER DONE.** `console/` has 9 crates; no lane ever applied the operational discipline to it. |
| *"why so many yaml json toml and markdowns? are they all justified?"* — *"we would be able to tell once we have code graph"* | **UNANSWERED.** Measured now: **5342 `.md`, 5853 `.yaml`, 1755 `.json`, 1001 `.toml` = ~13,950 non-code artifacts.** The justification pass was deferred pending the code graph, which now exists. |
| Per-field data-classification fact layer (`/spec-driven-development`) | **SPEC ONLY.** `docs/ideas/data-classification-fact-layer.md` landed; the Phase-1 proc-macro, per-regime policy packs, and derived DPIA/RoPA/SOC2 projections were never built. **82 templated `dpia/dpia.md` copies still exist** — the duplication the spec was written to remove. |
| Masterplan v2 goal: countable md/json reduction | tracked in memory, not scheduled |

---

## 7. Three reference disciplines — READ THIS SECTION PROPERLY

The founder repeatedly asks whether these are being applied, and checks. They are **procedures, not
content** — the value is in the method, never in the source material's subject matter. They are
**complementary, not overlapping**: one governs how you operate, one how you migrate, one how you
write. Sources: `blog.gaebal-gajae.dev`, `bun.com/blog/bun-in-rust`,
`github.com/jclab-joseph/it-legal`.

Each is given below with the concrete 2026-08-02 failure that demonstrates it, because the abstract
form does not survive contact with a real task.

### 7.1 Gajae — OPERATIONAL discipline

**Core claims:** a no-op is a claim needing verification · a successful build is not a deployed
result · every outcome resolves to a TYPED TERMINAL STATE, and the evidence chain must end in an
observation.

**How to apply here:**
- `BLOCKED`, `STOPPED`, `premise void`, `refused` are **valid, valuable outcomes** — not failures to
  paper over. Three lanes this session returned exactly those and each was right to.
- Report what you looked for and did **not** find, not only what you found.
- Never mark work done on "declared". Only on "executed and observed".

**The failure that proves it.** The previous session twice reported merge state from a
present-tense rollup query. That is not an observation of the past — it reports what checks say NOW.
It produced the claim *"#1507 merged with three required contexts in FAILURE"*, which was wrong. What
actually happened: exactly one check had concluded red at the merge instant, and `oya-ci-required`
was **PENDING**, concluding red **33m45s after the commit was already on trunk**. Merging past a red
gate is a policy *violation*; this was policy never being *consulted* (`enforce_admins: false`) —
a different and worse defect needing a different fix. **The evidence chain has to terminate in an
observation of the moment in question. Join `completedAt < mergedAt`.**

### 7.2 Bun-in-Rust — MIGRATION discipline

**Core claims:** the "temporary" intermediate IS the dark wiring — land the real thing or land
nothing · rank targets by an **ORACLE** before porting · never ship a bridge you cannot verify.

**ORACLE-FIRST is the most-skipped step and the most expensive to skip.** Both of this session's
worst detours were the same omission at two different altitudes:

1. **A move scheduled without a DESTINATION oracle.** `cloud/cloud-kernel` → `kernel/` was chosen as
   "the safe next move" after verifying source disposition and fan-out. Nobody asked whether the
   destination was free. It was not — another programme had populated `kernel/`, with a
   toolchain-incompatible workspace (edition 2021 vs 2024, pinned nightly vs stable, `build-std`
   bare-metal vs host std). **A move plan needs a destination oracle, not only a source oracle.**
2. **A migration tool whose oracle could not see its own target.** The reorg codemod ran
   `cargo metadata` against the **root** workspace — which *excludes* the nested workspaces it
   migrates. It returned `cargo_ok: true, clean: true` while leaving 5 dangling path deps and 27
   broken `include!` paths. It also silently **hoisted** moved crates into the root workspace, and
   its `package =` alias rename was rebinding every `use` site. **A tool whose oracle cannot observe
   the thing it changes validates nothing.** Fixed in PR #1523.

**The rule to carry:** before any port or move, *name the oracle and prove it can observe the thing
being changed.* If no oracle can verify a transformation — as with relative-path rewriting, which
neither `cargo metadata` nor `buck2 targets` can check because neither compiles — **detect and refuse
loudly instead of rewriting cleverly.** Refusing cannot corrupt a tree; a clever rewrite can. That
was the deliberate choice in #1523's D3, and it was right.

**Also from Bun:** a pointer to a successor must be an **assertion, not a comment** — it should fail
if the successor disappears. See the retirement of the Python BUCK generators (#1512).

### 7.3 it-legal — DRAFTING discipline

**The method, in order:** anchor every rule to a **cited obligation** and derive the requirement from
it (never the reverse) · cite **inline** so a reader verifies without leaving the document · make
anything enumerable a **table** · **THE SCHEMA IS THE SPEC** — ship the actual assertion, DDL, or
gate id, never a description of one · state the **PATTERN** separately from the current
implementation, so the transitional stack can change without rewriting the doctrine · **macro to
micro** · **one complete document**, not a scattered tree.

**Why row four is load-bearing here.** A doctrine that *describes* a mechanical test is something a
reader INTERPRETS. A doctrine that carries the assertion is something a reader IMPLEMENTS. In this
repo that gap is the dominant defect: 19 dark gates, a validator that validated only fixtures, 80
lane rows naming a retired command. **Prose specs are how that happens.**

**Applied well this session:** ADR-0633 ships every decision's runnable acceptance test inline (the
T1/T2 ownership test, a false-positive-rate test over the last 100 merges, RED/GREEN for the
population counter). ADR-0634 cites three passing in-tree tests by exact line and ships four config
stanzas, D8 including the literal PUT with its readback assertion.

**The cleanest instance of the failure it prevents.** A gate's anti-vacuity floor was written as
`min_expected_yaml_files / 2` — a *derived* number silently encoding the assumption *"most YAML sits
outside the build graph."* The assumption was never stated, so it was never reviewed. When a change
pulled 4541 files into packages and falsified it, the guard fired on **real progress**. The repair
was to make it a named, reviewable policy field with its own RED fixture. **A floor guarding an
assumption belongs where a reviewer can see it, not inside an arithmetic expression where changing
the corpus silently changes the guard.**

### 7.4 The composite rule

When you take on a task, ask all three:
- *Gajae:* what terminal state will I report, and what observation will justify it?
- *Bun:* what is my oracle, and can it see what I am changing?
- *it-legal:* what obligation is this derived from, and where is the assertion?

Longer write-ups of these live as individual `.md` files in the memory directory (§8) — read
`drafting-procedure-obligation-to-executable-spec.md` first: it carries all three disciplines and the
oracle-first failures. Also `owned-stack-go-to-rust-bun-discipline.md` and
`northstar-everything-in-code-and-build-graph.md`.

---

## 8. Context files — all plain files, read them directly

**In-repo (tracked, authoritative):**
- `CLAUDE.md` (root) — **authoritative project rules.** Read this early; it is not optional.
- `AGENTS.md`, `docs/AGENTS.md` — the operating contract and authority chain.
- `specs/capability-registry.json` — the **CLOSED** capability registry. A directory that is not a
  registered capability cannot become top-level without an amendment.
- `specs/reorg/*.json` — the ten EXECUTED move plans. **`os-move-plan.json` is both the schema and the
  standard of argument** to imitate when writing a new plan's `_comment` (it explains *why* each face
  assignment is what it is, on evidence).
- `specs/masterplan.json` — the plan of record.
- `registry/quality/lanes.yaml` — the 93 quality lanes and their `check_command`s.

**Local, untracked (this machine only — `.omc/` is gitignored):**
- `.omc/ultragoal/SESSION-HANDOFF-2026-08-01.md` — the longer prior handoff. Its `§0-LATEST` block is
  current; everything below it is history. **This file (the one you are reading) supersedes it.**
- `.omc/ultragoal/friction-ledger.jsonl` — 189 friction rows, 119 open, last append 2026-06-21.

**Prior-runtime artifacts (plain files; read, do not try to execute):**
- `.omc/ultragoal/DELETION-PLAN-cloud-kernel.md` — **the approved deletion plan, authoritative for §4.**
  Markdown. Read it before starting W1.
- `~/.claude/projects/-Users-jasonlee-Developer-oyatie/memory/MEMORY.md` — durable memory **index**,
  ~139 one-line entries, each pointing at a sibling `.md` file in the same directory. Not auto-loaded
  for you. It records prior measurements, founder rulings, and known traps — **read it before making
  an architectural claim**, and treat each entry as a point-in-time observation to re-verify, not as
  live state.
