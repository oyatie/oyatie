---
doc_class: DecisionRecord
shape: ~
length_cap: 300
authority_tier: 2
status: Superseded
doc_status: published
date: 2026-05-13
purpose: |
  Formalise the Hybrid C cutover topology for the v4 rename plan: Shard 0
  pure-tooling precursor + atomic Shard 1 rename/metadata/dep-edge/CI cutover.
  Supersedes ADR-0055 (v3-era taxonomy, fitness/freeze/expedite primitives).
  Documents lockfile-rename xtask, 4-partition reviewer streams, cross-vertical
  refusal enforcement, rollback/expedite protocol.
canonical_authority: docs/CONSTITUTION.md
supersedes: docs/decisions/ADR-0055-rename-plan-v3-cutover.md
superseded_by: ~
supersession_note: "Executed one-shot migration; dangling+colliding supersedes edge to phantom ADR-0055-v3 file. Archived per D-DISPOSITIONS-RATIFIED: ARCHIVE-5, C-4/D11(c)."
related_adrs:
  - ADR-0015
  - ADR-0053
  - ADR-0054
  - ADR-0056
companion_docs:
  - docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
---

# ADR-0057: Cutover Mechanics — Rename Plan v4 (Hybrid C)

> **Status:** Accepted — 2026-05-13
> **Date:** 2026-05-13
> **Owner:** `council-architecture`
> **Supersedes:** ADR-0055 (v3-era rename plan ADR; fitness/freeze/expedite primitives dropped)

---

## Context

ADR-0055 formalised the v3 rename plan's cutover mechanics including the
`expedite_override_token`, `freeze-window-kernel` fitness lane, and
`lane-config-oyatie` ICM topic. v4 replaces v3 entirely (see plan
`docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`). The fitness/freeze/
expedite machinery is dropped because grit's existing claim system already
provides an exclusive symbol-lock for the duration of Shard 1's atomic
squash-merge.

---

## Decision

### Hybrid C Topology

**Shard 0 (pure tooling precursor, this commit)**:
- `tools/xtask-metadata-augment` crate scaffolded with `lockfile-rename` subcommand + 20-cell dependency-form fixture matrix + 8-row lockfile fixture matrix + ICM round-trip test.
- 4 LEAN check crates scaffolded empty: `oya-shared-architecture-check-cli` (LEAN-A1), `oya-shared-bounded-contexts-check-cli` (LEAN-A2), `oya-shared-supply-chain-check-cli` (LEAN-A3), `oya-shared-semver-check-cli` (LEAN-A4).
- ADR-0054 amended to cover rename events (not just new-crate scaffolds).
- ADR-0056 authored (BNF + layer enum + bounded-context registry policy).
- This ADR (ADR-0057) authored.
- Standards co-edits: `clean-architecture.md`, `crate-naming-convention.md`, `code-style-rust.md`.
- No existing crate is renamed in Shard 0.

**Shard 1 (atomic rename, gated on Shard 0 acceptance + 48 h freeze)**:
- All 114 atomic-safe crates renamed (140 total − 26 PROTOCOL-UNKNOWN scheduled-for-distinct-tracked-work to Shard 1.5).
- `[package.metadata.oya]` blocks emitted to all 140 manifests via xtask `--apply`.
- All dep-edges rewritten (est. 200–400 sites).
- Cargo.lock rewritten via `xtask-metadata-augment lockfile-rename`.
- CI workflows, scripts, registry refs updated.
- 4 LEAN check crates populated (moved from scaffold to implementation).
- Standards co-edits finalised (bounded-contexts.md, clean-architecture.md §2.1 port-location fix).
- Single squash-merge; single lockfile event.

### Shard 1.5: PROTOCOL-UNKNOWN scheduled-for-distinct-tracked-work renames

**Scope**: 26 rows = 5 platform-`*-api` + 13 cloud-`*-api` + 4 foundry-`*-api` + 4 workspace-`*-api`, all marked `PROTOCOL-UNKNOWN, scheduled-for-distinct-tracked-work to ADR-0056 §"Protocol classification"` in §3 audit body.

**Gate to enter**: iter-4 src-inspection completes protocol classification for every row (each `-api` crate identifies as `rest`, `grpc`, `graphql`, or other 12-enum-member protocol layer).

**Timing**: post-Shard-1 commit, no freeze window required (these are existing v3 names; Shard 1.5 only renames the protocol-classified subset).

**Cross-reference**: §3.6 row counts must subtract the 26 from each partition's "renamed: N" line; the §3.6 totals are *aspirational* across both Shards.

**BNF effect**: Shard 1 still meets the "atomic rename" property *for its scoped 114 rows*; Shard 1.5 is a successor-IP, not a partial first attempt.

> **Naming justification for "Shard 1.5"**: This is a milestone-naming convention, not a BNF crate name. BNF (ADR-0056) governs `oya-*` crate identifiers only. Milestone labels like "Shard 0", "Shard 1", "Shard 1.5" are coordination vocabulary outside BNF scope. "1.5" denotes a sequentially ordered successor-IP to Shard 1, within the same major-milestone bracket, consistent with the Hybrid C topology naming convention established in this ADR.

### Lockfile-Rename xtask

```
cargo run --release -p xtask-metadata-augment -- lockfile-rename \
  --rename-map /tmp/rename-map.tsv \
  --lockfile Cargo.lock \
  --inplace
```

Deterministic rewrite via `toml_edit`; preserves version, source, checksum.
Reverse via `--reverse` flag. Post-rewrite gate: `cargo check --workspace --locked --offline`.

### 4-Partition Reviewer Streams (R11 mitigation; rebalanced iter-2)

| Stream | Partition | Reviewer |
|---|---|---|
| 1a | Platform/shared (~28 crates, §3.1) | reviewer-platform |
| 1b | Cloud vertical (~31 crates, §3.2) | reviewer-cloud |
| 1c | Foundry vertical (~51 crates, §3.3) | reviewer-foundry |
| 1d | Workspace + tooling/shared + 4 hotspots (~30 crates + ADR-0056 + clean-arch amendment + xtask spec + 4 LEAN check crates) | reviewer-lead (full-PR scope) |

Atomic squash-merge requires **all 4 partition sign-offs**.

### Cross-Vertical Refusal Enforcement

`oya-shared-bounded-contexts-check-cli` (LEAN-A2) enforces:
1. Direct cross-vertical deps refused (`vertical-A → vertical-B` where A ≠ B, both ≠ `shared`).
2. Transitive cross-vertical via shared refused (`vertical-A → shared-X → vertical-B` — the `shared-X → vertical-B` edge is the proximate violation).
3. `public_layers` exemption applied at every cross-vertical hop (not just chain endpoints).

Runs in `--report-only` mode during Shard 1 merge; flipped to BLOCKER in a successor-IP PR within 24 h of Shard 1 merge (§8.2 global gate).

### Dropped Machinery (vs. ADR-0055 / v3)

| Dropped | Replacement |
|---|---|
| `expedite_override_token` | grit claim's exclusive-lock authority (existing) |
| `oya-governance-freeze-window-kernel` lane | grit claim's symbol-lock for the 48 h window |
| `lane-config-oyatie` ICM topic | not needed; grit claim replaces it |
| `oya-governance-fitness-*` crate family | flat `oya-check-*` namespace (4 LEAN crates) |
| "fitness" terminology | "check" (matches team vocabulary) |

---

## Decision Drivers

1. **grit already enforces freeze windows.** The `oya-governance-freeze-window-kernel` primitive was a parallel implementation of grit's existing claim system.
2. **`fitness` jargon.** Every fitness crate was a check, audit probe, or supply-chain gate. The `oya-check-*` namespace names them honestly.
3. **Single lockfile event.** Hybrid C's atomic Shard 1 produces exactly one Cargo.lock churn event vs. 6 under Option B (sequential context-shards).

---

## Rollback / Expedite Protocol

**Shard 0 revert** (rare; < 15 min):
```
git revert <shard-0-sha>
```
Removes xtask, ADR-0054 amendment, ADR-0056, this ADR, check crate scaffolds, registry block.

**Shard 1 revert — standard** (< 60 min; full gate):
```
git revert <shard-1-sha>
cargo run --release -p xtask-metadata-augment -- lockfile-rename \
  --rename-map /tmp/rename-map.tsv --lockfile Cargo.lock --inplace --reverse
cargo check --workspace --locked --offline
```
Then run §8.1 gates against pre-Shard-1 state.

**Shard 1 revert — emergency lane** (< 15 min; CI bypass):
Pre-conditions (ALL three required):
1. The grit claim on the rename-cutover symbol-lock is currently held by a Security Council member.
2. Operator possesses standing Security Council authority (ADR-0054 icm-coordination-lock authority).
3. ICM rationale row logged BEFORE the admin-merge:
   ```
   icm store -t direct-tool-invocations \
     -c "EMERGENCY revert of Shard 1 via admin-merge, rationale: <reason>; grit-symbol-lock-held; security-council-authority" \
     -i critical
   ```
Then: `gh pr merge --admin` under the named exception.

Post-emergency: full §8 gate set runs non-blocking (BLOCKING if staging already reached, per R8).

**Security P0 during 48 h freeze**:
The on-call security council member releases the grit symbol-lock via
`grit done --agent <id> --force`, lands the P0, and re-acquires the lock.
ICM rationale row at `direct-tool-invocations` topic. No fitness-lane token needed.

---

## Consequences

### Positive
- Single lockfile event; single 48 h coordination window; clean revert path.
- Reviewer load bounded by 4-partition scheme (~100–250 files per reviewer vs. 500–700 full PR).
- `oya-check-*` names match team vocabulary; no fitness-function jargon.

### Negative
- Higher one-time rename count (~144 ops) than v3 (~37).
- 48 h window requires scheduling; Hybrid-C-Lite escape hatch (48 h unschedulable within 2 weeks → async merge with branch-protection relaxation) documented in §7.3 of the v4 plan.

---

## References

- `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §4, §5, §7, §8 — execution plan source of truth
- ADR-0054 — scaffold-claim pattern (amended in Shard 0 to cover rename events)
- ADR-0056 — 3-slot BNF + 12-layer enum (authored in Shard 0)
- ADR-0053 — grit + icm as sanctioned coordination primitives
- ADR-0055 — superseded v3 cutover ADR
