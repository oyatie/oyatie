---
id: ADR-0626
title: "Resolve fixup-ledger merges structurally instead of by hand"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-27
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: [ADR-0544, ADR-0622]
related_specs: [/registry/fixuptasks.jsonl]
milestone: W0
---

# ADR-0626: Resolve fixup-ledger merges structurally instead of by hand

## Context

`registry/fixuptasks.jsonl` is an append-mostly JSONL ledger. Line 1 is a schema header carrying
no `id`; every other line is one task row keyed by `id`. Filing a finding appends a row at the end
of the file.

Two lanes that each file a row therefore touch adjacent bytes, and git's text merge conflicts on
essentially every pair of concurrent PRs even though the change is semantically disjoint. GH #1412
records this. In one working session the same conflict was resolved by hand four times.

Hand resolution is not merely slow, it is unsafe. A resolver that indexes rows by `id` skips the
schema header, because the header has no `id` to key on. That exact bug — `if i and i not in seen`,
where `None` is falsy — silently deleted the header from four branches before anyone noticed. The
loss was invisible in review: the diff showed only the appended rows.

`merge=union` is the shortcut already applied to `evidence/audit-chain.jsonl`, and `.gitattributes`
records why it was deliberately NOT applied here: this ledger's history contains in-place row
replacements (status flips on existing rows), so union would keep both the stale and the updated
variant of one logical record. Every consumer keys on `id`, so that is silent corruption rather
than a visible conflict — strictly worse than the conflict it removes.

## Decision

Ship a structural three-way merge driver, following the two drivers already in the repo
(`tools/oya-cargo-lock-merge-driver-app`, `tools/oya-friction-ledger-merge-driver-app`).

**D1 — Pure kernel, thin binary.** The merge is a pure function over parsed rows with zero I/O, so
it is fixture-drivable; the binary is only the `%O %A %B` git contract plus an atomic write. Same
split as both sibling drivers.

**D2 — The schema header is bound to line 1 structurally, never looked up by `id`.** This is the
whole reason the kernel exists in this shape rather than as a one-off script. The header's absence
of an `id` is exactly what made the hand resolvers drop it, so the kernel asserts the header is
present on the way out and fails closed if it is not.

**D3 — Resolve only what is unambiguous.** Per `id`: an edit on one side wins; identical edits on
both sides collapse to one row; different edits to the same `id` on both sides CONFLICT. The
driver declines rather than guessing, so git falls back to a normal conflict. This is what union
cannot do and is the reason union was rejected here.

**D4 — Deletion never wins.** A row present in the base and dropped by one side is carried through.
The registry's own `_meta` declares it append-only, and a row vanishing during a merge is the
failure this driver exists to prevent. Same trade-off `evidence/audit-chain.jsonl` already makes:
a legitimate redaction must be a single linearised commit on `dev`, never a merge outcome.

**D5 — Byte-preserving output.** Rows are copied verbatim from their source line, never
re-serialised. Re-dumping this file has twice produced enormous phantom diffs by reordering keys
and re-escaping em-dashes; parsing for structure while emitting the original bytes avoids that
class entirely.

**D6 — Registration is per-clone, so this cannot make the status quo worse.** Git merge drivers
are named in `.gitattributes` but bound in local config. An actor without the `merge.fixup-ledger`
binding gets exactly today's behaviour — a normal conflict. There is no state in which enabling
this driver is worse than not having it.

**D7 — Self-validating.** The merged result is re-parsed before it is allowed out, and the driver
refuses to emit it unless the header survived, no `id` present on any side was lost, and no `id`
is duplicated. A driver that emits a subtly wrong ledger is worse than one that declines.

## Consequences

The recurring conflict stops costing hand resolution, and the specific silent-deletion failure
becomes a hard error instead of an unnoticed data loss.

ADR-0622 (Proposed, `planning_impact: false`, nonbinding) proposes a lifecycle contract for what
a row must CONTAIN. This decision governs only how two branches' rows COMBINE, and is agnostic to
row contents, so the two are complementary rather than competing. Nothing here retires the ledger
or presumes ADR-0622's outcome; if a successor surface is later accepted, this driver retires with
the file it serves.

This does not make the ledger schema enforced. `_meta` declares `blocker_for` as part of the
schema and 165 of 412 rows carry it; nothing validates that. The driver preserves whatever rows it
is given and takes no position on their contents. That gap is real and is not addressed here.

## Justified artifacts

This decision governs, and thereby justifies, the following files.

- `ci/facade/fixup-ledger-merge-driver/BUCK`
- `ci/facade/fixup-ledger-merge-driver/Cargo.toml`
- `ci/facade/fixup-ledger-merge-driver/README.md`
- `ci/facade/fixup-ledger-merge-driver/src/lib.rs`
- `ci/facade/fixup-ledger-merge-driver/src/main.rs`
- `ci/facade/fixup-ledger-merge-driver/src/tests.rs`
- `docs/decisions/ADR-0626-structural-merge-driver-for-the-fixup-ledger.md`
