# Portable agent lifecycle protocol — Bun-disciplined design — 2026-08-03

State: **DRAFT DESIGN — NOT INDEPENDENTLY REVIEWED — NOT IMPLEMENTABLE — NO CUTOVER**  
User-fixed boundary: `~/.agents` is the centrally managed protocol/bootstrap boundary across sessions, repositories, models, Claude Code, Codex, Grok Build, and Antigravity. It is the canonical user-local store for artifact classes shared by local harnesses; it is not a replacement for repository governance authority or a cross-machine/team database.  
Initial compatibility slice: shared ultragoal artifacts currently duplicated between `.omc/ultragoal` and `.omx/ultragoal`.

## Problem statement

The current system has three distinct defect classes:

1. conversation/task state can disappear or change meaning across sessions and harnesses;
2. `.omc` and `.omx` independently name/copy shared artifacts, so parity depends on dual writes;
3. global skills and procedures can be installed into harness-specific roots and drift.

The live oracle on this worktree proves the duplication defect: `.omx/ultragoal` has 35 files, all 35 are byte-identical to their `.omc/ultragoal` counterparts; `.omc/ultragoal` has 44 additional files; `.omx` has no unique file. Identical copies are compatibility evidence, not one authority.

## Non-goals

- Do not centralize every byte under `.omc`, `.omx`, `.claude`, `.codex`, or another harness-private root.
- Do not move caches, process locks, task UI state, session transcripts, provider credentials, or runtime-private checkpoints into the shared protocol.
- Do not let a model, peer, adapter, task list, snapshot, or projection become authority.
- Do not vendor runtime skills into Oyatie or let repo policy redefine the lifecycle protocol.
- Do not implement a generic distributed database. One append-only local journal with deterministic reduction is sufficient until measured concurrency proves otherwise.

## Authority and storage boundary

```text
~/.agents/
  protocol/lifecycle/
    contract.json              # versioned semantic contract
    event-schemas/             # versioned event payload schemas
    conformance/               # executable cross-adapter fixtures
  state/
    journals/<lifecycle-id>.jsonl
    snapshots/<lifecycle-id>.json
    artifacts/<artifact-id>/manifest.json
    blobs/sha256/<digest>
  adapters/
    claude-code/
    codex/
    grok-build/
    antigravity/
```

Only the portable semantic contract, local shared artifact identities/blobs, local lifecycle journals, and disposable reducer snapshots live here. A stable canonical protocol URI/store ID identifies the store independently of its home-directory mount; adapters must define sync/recovery rather than assume `~` is shared across machines.

Repository machine authorities remain the repository's admitted `/specs`, `/registry`, `/evidence`, protected source revision, and governance pipeline according to the resolved repo policy. In Oyatie specifically, `specs/root-hub-pointers.json` selects `docs/AGENTS.md` as the live operating contract until explicit PHASE-5 promotion, and `specs/masterplan.json#masterplan_v2` is the sole live plan authority; `.omc`, `.omx`, legacy sequencing stores, and `specs/agent-operating-contract.json` are provenance/projections. `~/.agents` references those immutable authorities; it does not supersede them. Cross-machine/team facts require an admitted repository or service-backed URI selected by the repo adapter, not copying a home-directory journal.

Repository `CLAUDE.md` / `AGENTS.md` and repo-native overlays remain versioned policy adapters. They may add constraints and authority references but cannot silently redefine the portable protocol vocabulary. Where repository policy conflicts, repository policy wins for that repository and the adapter records a blocked incompatibility rather than weakening either contract. Their resolved bytes enter a lifecycle as immutable `policy_snapshot_ref` values.

Harness-native `.omc` and `.omx` paths become compatibility views. They may contain runtime-private state, but a shared lifecycle artifact stored there must carry or resolve the canonical `artifact_id`; it is never an independent authority.

## Stable identity

```text
protocol_store_id = immutable opaque identifier for one configured store
protocol_uri = opaque adapter-resolved URI; local default mounts under ~/.agents
scope_key = stable workspace identity; never a path basename alone
lifecycle_id = immutable globally unique identifier within protocol_store_id
subject_ref = opaque immutable repository/workspace + revision + logical subject identity
artifact_id = sha256(canonical artifact manifest bytes)
blob_ref = sha256(content bytes)
```

For the local ultragoal slice, the scope key resolves from a stable `.omc-workspace` ID/anchor when present, otherwise from a Git common-directory identity so linked worktrees converge without collapsing unrelated repositories. This is adapter logic, not core Git vocabulary; non-Git adapters provide an equivalent opaque workspace identity.

The initial local layout is:

```text
~/.agents/artifacts/v1/<scope-key>/ultragoal/<plan-or-legacy>/<artifact-name>
logical id = artifact:v1:<scope-key>:ultragoal:<plan-or-legacy>:<artifact-name>
```

The manifest binds logical kind, subject, producing lifecycle/event, content digest, schema version, provenance, authority refs, and sensitivity class. Paths and basenames are labels, not identities.

No credential bytes, decoded Secret values, bearer tokens, private keys, or provider session material may enter events, manifests, snapshots, fixtures, logs, or projections.

## Lifecycle state

```text
phase  = discover | plan | review | implement | verify | admit | rollout | observe | retire
status = active | blocked | completed
state  = {lifecycle_id, subject_ref, phase, status, attempt, sequence}
initial = discover/active
normal terminal = retire/completed
```

A deterministic reducer over the append-only journal is authority. Snapshots, task boards, status Markdown, `.omc`, `.omx`, and harness UI are disposable projections.

Every append requires the expected previous sequence and an idempotency key. Duplicate idempotency keys are no-ops only when their complete event digest matches. Sequence gaps, stale expected sequence, unknown event type/version, subject mismatch, invalid authority scope, and differing duplicate payloads fail closed.

## Canonical event envelope

```text
spec_version
event_id
event_type
lifecycle_id
subject_ref
sequence
occurred_at
producer_ref
actor_ref
correlation_id
causation_id
phase
attempt
payload_schema_ref
payload
authority_refs[]
evidence_refs[]
policy_snapshot_refs[]
idempotency_key
integrity_ref
```

Events are past-tense facts, never commands disguised as facts:

- `LifecycleCreated`
- `PhaseEntered`
- `DiscoveryRecorded`
- `PlanProposed`
- `ReviewDecided`
- `ImplementationPublished`
- `VerificationDecided`
- `AdmissionDecided`
- `RolloutStarted`
- `RolloutCompleted`
- `RolloutReverted`
- `ObservationDecided`
- `EvidenceAttached`
- `AuthorityReferenced`
- `PhaseBlocked`
- `PhaseResumed`
- `PhaseCompleted`
- `PhaseReopened`
- `RetirementDecided`
- `LifecycleRetired`

Only `PhaseCompleted` advances after the reducer validates the phase guard. A failure records evidence and blocks or reopens; history is never rewritten. `RetirementDecided` may close abandoned, superseded, rolled-back, or decommissioned work without fabricating successful completion.

## Phase guards

1. `discover → plan`: normalized intent/scope, constraints, source/authority refs, and unknowns exist.
2. `plan → review`: versioned plan digest, acceptance criteria, risk, verification, rollout, rollback, and retirement strategies exist.
3. `review → implement`: independent registered review satisfying repository risk policy approves the exact change-set digest (base + patch/plan) and all blocking findings are disposed. Review excludes implementer persuasion but includes authoritative contract, callers, tests, oracle, and evidence; reviewer count is policy-defined, not universally fixed.
4. `implement → verify`: immutable implementation artifact and provenance match the reviewed plan; otherwise reopen review.
5. `verify → admit`: required evidence independently passes, is fresh, and binds the exact artifact plus policy snapshot.
6. `admit → rollout`: a scoped external admission decision authorizes the exact artifact and target. Green checks alone never imply admission.
7. `rollout → observe`: a rollout-controller receipt proves the target reached the intended digest and a rollback pointer exists.
8. `observe → retire`: the observation window and required SLO, security, user-story, audit, cleanup, and release outcomes pass.

## Authority references

```text
{kind, uri, revision, digest, issuer_ref, decision_id, scope, observed_at, expires_at}
```

Reference kinds include requirement, source, plan, review verdict, artifact, verification verdict, policy snapshot, admission decision, rollout receipt, telemetry query receipt, rollback receipt, and retirement decision.

An issuer attests only its registered scope. Models and harnesses are actors, not authorities. A peer message never grants permissions. Mutable pointers are inadmissible unless captured with immutable revision/digest. Expired, stale, mismatched, or scope-incompatible authority fails closed.

## UX parity principle

The shared contract is the **user-facing verb set and its semantics**, not a lowest-common-denominator implementation. The same verb must mean the same thing and produce the same outcome class on every harness, while each adapter implements it with the **best native mechanism that harness offers** — Claude Code's Workflow tool, subagents, plugin skills and hooks; Codex's goal mode, `.codex-plugin` plugins and MCP servers. Parity is judged on observable UX and canonical events, never on identical internals. A harness lacking the capability blocks a named stage with an explicit reason; it never silently substitutes prose for the mechanism.

## Adapter contract

Each harness adapter has only four responsibilities:

1. negotiate supported protocol/event/schema versions;
2. translate native callbacks into proposed canonical events;
3. append through the one canonical expected-sequence/idempotency API;
4. render canonical state/artifacts into native compatibility views.

Adapters cannot mint review, admission, rollout, or credential authority unless the issuer and scope are registered in the active policy snapshot. Adapter-only extensions live in namespaced payloads and cannot weaken core guards.

Claude Code, Codex, Grok Build, and Antigravity conformance uses the same fixture corpus. A missing capability blocks at a named phase; it never silently drops evidence or invents success.

## `.omc` / `.omx` compatibility rule

Initial scope is only shared ultragoal artifacts. Runtime-private state stays native.

For every shared logical artifact:

```text
read authority = canonical ~/.agents artifact identity/blob
legacy read = allowed only during shadow/adoption and bound to activation SHA-256
write authority = canonical ~/.agents append/artifact API only
projection write = generated/link operation after canonical commit
both legacy copies differ = hard conflict; never newest-wins or merge
canonical absent + one legacy present = adoption candidate, not automatic authority
```

An adoption manifest records each legacy path and its activation SHA. After adoption, an unchanged legacy copy may remain readable during compatibility; any independent legacy mutation is a conflict. A crash after canonical commit but before projection is recoverable by replay. A crash before canonical commit changes nothing authoritative.

Do not symlink whole `.omc` and `.omx` trees: that would merge unrelated private state and locks. Only shared artifact identities receive generated views or links.

## Plugin and MCP boundary

Measured 2026-08-03. Codex already treats the central store as its plugin home: its bundled plugin README states the default marketplace is `.agents/plugins/marketplace.json`, and `~/.agents/plugins/marketplace.json` exists and resolved `personal/agent-skills` into `~/.codex/plugins/cache/personal/agent-skills/1.0.0+codex.local-…`. Its declared source path `./plugins/agent-skills` is now **missing**, so the central marketplace is currently dangling. Claude Code keeps a parallel registry in `~/.claude/plugins/{known_marketplaces.json,installed_plugins.json,marketplaces/,cache/}` — 5 marketplaces, 8 installed plugins. Only `ponytail` is installed on both, and its payloads differ (1.9 MB Claude vs 4.0 MB Codex), so it is duplicated, not shared. `claude_design` and `open-design` are declared twice, in `~/.claude.json` and `~/.codex/config.toml`.

Manifest formats are genuinely per-harness: `.codex-plugin/plugin.json` for Codex, `.claude-plugin/` for Claude. Multi-harness plugins already solve this by shipping several manifests beside one payload — the `ponytail` marketplace checkout carries `.devin-plugin/plugin.json`, `gemini-extension.json`, and `opencode.json` together.

Therefore the boundary is:

- **Centralize declarations** — the marketplace registry, the intended plugin set, and MCP server definitions belong in `~/.agents` as one machine-readable source, projected into each harness's native config.
- **Do not centralize caches** — installed plugin payloads are installer-owned, versioned, and harness-specific; merging them would recreate the dual-authority defect and buy nothing (276 MB + 357 MB of legitimately per-harness bytes).
- **Fix, do not ignore, a dangling central pointer** — a marketplace entry whose source path does not exist must fail loudly, not resolve from a stale cache.

## Global skill boundary

`~/.agents/skills` is the sole user-global installer write target and lock owner. Harness discovery order is:

1. explicit repository-native overlay;
2. `~/.agents/skills`;
3. legacy user root during migration only.

Full skill trees are content-hashed. Same name plus different digest at the same user scope is a hard conflict. Repo `.agents/skills/rust-skills` remains a repo-owned override, not a user-global copy.

## Bun-disciplined migration

Bun's lesson applies as discipline rather than its compiler-error queue mechanics:

### M0 — Freeze the oracle

- inventory shared `.omc`/`.omx` artifacts and full-tree digests;
- record identical, only-left, only-right, and conflict cases;
- capture failure/resume, duplicate event, stale authority, digest mismatch, rollback, and secret-redaction fixtures;
- make no authority change.

### M1 — Minimal semantic kernel

Implement the smallest owned-Rust contract parser, deterministic reducer, append primitive, artifact manifest/blob store, and conformance runner. No harness adapter writes yet.

### M2 — Vertical protocol slices

In order, implement and independently review:

1. discover → plan → review;
2. implement → verify;
3. admit → rollout;
4. observe → retire.

Each slice must run across all four adapters before the next slice. Preserve compatibility before adding improvements.

### M3 — Shadow reads and writes

- existing native behavior remains active;
- adapters propose canonical events and compare reduced projections without controlling execution;
- compare artifact identity, bytes, state, failure, and resume behavior;
- emit counts for matches, fallback reads, conflicts, rejected stale events, and redactions;
- never dual-write two authorities.

### M4 — Read cutover

After zero unexplained mismatches on representative real workloads, adapters read canonical state/artifacts first. Legacy fallback is allowed only for activation-hash-bound adoption candidates. Mutation remains disabled if conflict exists.

### M5 — Write cutover

All shared writes append/store once under `~/.agents`; native files are generated projections. Installer writes target only `~/.agents/skills`. Rollback returns to canonical-read shadow mode and disables mutation; it never re-enables independent legacy writes after the authority marker exists.

### M6 — Retirement

After two released adapter versions or 30 days (whichever is longer) with zero legacy writes, fallbacks, and conflicts; green parity/conformance; and all installers single-writing canonical:

- remove legacy fallback/write code;
- delete only duplicate shared subtrees after an exact inventory review;
- retain native private state and repo overlays;
- emit a retirement event and deletion manifest.

Transitional machinery without a measured retirement condition is a defect.

## Required executable oracles

- deterministic replay produces byte-identical state twice;
- predicates are defined in the core contract and exercised by one real reducer/conformance runner with negative fixtures; shell/jq/rg snippets are neither protocol authority nor proof;
- duplicate identical event is idempotent; duplicate key with differing bytes fails;
- out-of-order/gap/stale expected sequence fails;
- unknown schema/event/version fails;
- plan/artifact/review/admission digest mismatch fails;
- reviewer and admission issuer scope mismatch fails;
- mutable authority without captured revision/digest fails;
- projection deletion/rebuild preserves canonical state;
- `.omc`/`.omx` different bytes for one adopted identity hard-fail;
- crash before/after canonical append is replay-safe;
- Secret/token/private-key fixtures are rejected/redacted;
- all four adapters reduce the same fixture journal to identical state;
- repository policy may strengthen but cannot weaken core guards;
- disabling every harness hook does not bypass canonical transition guards.

Buck2 is the Oyatie admission authority for repository implementation; Cargo is supplementary local feedback only. Cross-repository protocol conformance must also run without assuming Oyatie paths.

## Degraded-mode matrix

| Condition | Required behavior |
|---|---|
| store corrupt or unsupported version | read-only diagnostics; block all authority-changing appends/cutover |
| oracle unavailable | proposal/discovery may continue; parity, verification, and cutover block |
| reviewer unavailable | review/implementation admission blocks; no self-approval fallback |
| CI/admission issuer unavailable | verification/admission/integration blocks |
| reliable clock unavailable | existing leases remain active; no expiry-based authority gain |
| adapter capability missing | emit named `PhaseBlocked`; never substitute chat/prose success |
| offline segment | append only against pinned head/sequence if policy permits; reconciliation rejects conflicts fail-closed |
| legacy projection conflict | canonical mutation disabled until independently resolved |

## Security and failure policy

- local file permissions restrict journals/blobs containing non-secret operational metadata;
- sensitive references are opaque identifiers, never values;
- manifests have explicit sensitivity and retention classes;
- integrity digest covers canonical bytes; append uses atomic temp/write/fsync/rename discipline where supported;
- corrupted/truncated journal tails fail closed with last valid sequence reported; no silent repair;
- locks have owner/lease metadata and stale-lock recovery evidence;
- no adapter executes external mutation from a proposed event; only separately authorized controllers consume admitted commands/artifacts.

## Implementation sequence

1. Freeze schemas, identity rules, fixture corpus, and compatibility inventory.
2. Independent architecture/security review of exact design digest.
3. Implement Rust reducer/store/conformance kernel under a fresh isolated lane.
4. Add read-only adapters one harness at a time, with conformance after each.
5. Add ultragoal artifact adoption/shadow projection.
6. Run real workloads across session restart and at least two repositories.
7. Independently verify parity and failure cases.
8. Admit read cutover, then separately admit write cutover.
9. Observe retirement criteria; delete duplicate shared stores only after exact review.

## Unresolved before review freeze

- exact protocol schema locations and package ownership under `~/.agents`;
- stable repository identity rule for clones/worktrees/remotes;
- registered authority/issuer policy format and trust bootstrap;
- retention and encryption requirements by artifact sensitivity;
- exact Grok Build and Antigravity adapter capability surfaces;
- canonical API/process boundary for concurrent harness writers;
- inventory of every artifact class shared by `.omc` and `.omx` beyond ultragoal;
- rollout owner, rollback owner, and independent reviewers;
- whether the 30-day/two-release retirement floor needs a stricter organization policy.

No placeholder is an implementation choice. Fill these fields, freeze packet bytes, bind a digest, and obtain independent design APPROVE before code or cutover.

## Non-actions

- No `~/.agents` state/protocol directories created by this packet.
- No plugin, runtime, config, skill, `.omc`, or `.omx` code changed.
- No symlinks, copy, deletion, installer change, or write-authority cutover.
- No peer design treated as user approval.
- No transport failure treated as independent review.
