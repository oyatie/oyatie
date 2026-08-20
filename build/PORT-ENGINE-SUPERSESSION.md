# What the port engine supersedes, and what it does not

Decision record for `kernel/` and `os/`, and the burn-down status of the legacy roots. Written from
measurement, not from memory; every number below was counted on this branch. Promote to an ADR before
acting on the deletions.

## The test this applies

The engine's purpose is to generate Rust from Go **and keep it generated as upstream moves** — an
upstream pin, a six-axis receipt, and a drift classification that says `Explained` or `Unexplained`.
So the question for any tree is not "is this code good" but:

> Is there Go upstream that this mirrors, and can anything today tell whether it still mirrors it?

Hand-written Rust that mirrors a Go project and carries no pin is a liability whatever its quality,
because nobody can answer the second half. Hand-written Rust with no Go upstream is simply not the
engine's business.

## `os/` — SUPERSEDED. Freeze now, strangle per package, do not bulldoze.

582 files, 380 Rust, **170,653 lines**, 44 crates. Zero CI workflows reference it. Three external
dependents, all in `iam/`.

It is a hand-written mirror of Talos. Its own source says so — `os/core/machined-domain/src/boot.rs`
opens *"a real, callable boot sequencer that a PID1 process (talos-init) can drive, mirroring
`siderolabs/talos`'s machined sequencer"* — and the package names are Talos's component list:
`machined`, `apid`, `trustd`, `machine-config`, `kubernetes`, `network`, `block`, `time`, `security`.

Talos is Go. This is exactly the thing the engine generates.

What makes it a liability is not that it was hand-written; it is that **there is no upstream pin and
no receipt anywhere in the tree**. `os/manifest.json` is a microservice manifest — tier, owner, DAG
position — not a provenance record. So nothing in the repository can answer whether these 170,653
lines still mirror Talos today, and nothing will notice when they stop. That property is precisely
what the engine exists to supply.

**Decision: shrink-only, effective now.** No new code in `os/`. Each package is deleted when the
engine emits its replacement with a green receipt against a Talos pin — not before. Deleting working
code ahead of its replacement destroys capability and proves nothing; the point is to stop paying to
maintain a mirror nobody can verify, not to have less software.

## `kernel/` — KEEP, FREEZE, RELABEL. Not part of the generated stack, and never can be.

24 files, 11 Rust, **5,354 lines**, 5 crates, **zero external dependents**.

My prior was that this was a hand-written kernel and therefore the same liability as `os/`. Reading it
says otherwise. It is not a kernel at all: it is an ABI **measurement scaffold** around Asterinas —
a versioned syscall/ABI matrix, a boundary pin, an ABI probe and a boot harness. Its own module doc
states it *"does not claim Asterinas is the canonical node kernel (blocked on founder ADR F1(a)).
Scaffold ≠ green matrix."*

A kernel cannot be port-engine output. Asterinas is Rust; there is no Go source to port from, so no
pin and no receipt are possible even in principle. `kernel/` is therefore permanently **out of scope**
for the engine rather than superseded by it.

**Decision: keep it, freeze it, and label it.** It is the only evidence the pending kernel ADR would
have to decide on, it costs 5k lines and nothing depends on it, and deleting it would destroy a
measurement to save almost nothing. Freeze it against growth and mark it measurement-not-
implementation, so that it is never again mistaken for the owned kernel — which is the mistake this
record was written to correct.

## Burn-down status of the legacy roots — two of these cannot go

Measured, and two of the four named for burn-down are load-bearing:

| root | crates | Rust lines | CI refs | external dependents | verdict |
|---|---|---|---|---|---|
| `os/` | 44 | 170,653 | 0 | 3 | shrink-only; strangle per package |
| `oya/` | 173 | 179,871 | 0 | 7 | **burnable** after the 7 are resolved |
| `libs/` | 130 | 91,126 | 0 | **220** | **NOT burnable** — see below |
| `infra/` | — | 1,331 (1 file) | 2 | — | **keep** — GitOps surface |
| `kernel/` | 5 | 5,354 | 1 | 0 | keep, frozen |
| `tools/` | 19 | 29,815 | 1 | — | shrink-only |

**`libs/` cannot be burned down.** 220 crates outside it depend on it by path. It is the shared layer
the new capability directories are built on, and its retirement is the ADR-0562 strangler migration —
each library moving to its capability's `core/ports/adapters/facade` face — not a deletion. Burning it
would break 220 crates in one commit.

**`infra/` should not be burned down.** Two CI workflows reference it, and it is 1,331 lines of which
exactly one file is Rust: it is GitOps configuration, not code. It is cheap, it is referenced, and
"shrink-only root" is not the same as "dead root". (Its apparent modification today was an artifact of
this lane's own accidental deletion and restore, not real activity; its last real change is 2026-08-14.)

**`oya/` is the genuine burn-down candidate**: 173 crates, 179,871 lines, referenced by zero CI
workflows, with only seven external dependents — `marketplace/facade/dev-cli`, three
`libs/oya-shared-backbone-*` adapters, two `iam/facade/tenant-rbac-*`, and `billing/facade/saas-bench`.
Resolve those seven and the root can go in one move.

`cloud/` does not exist on this branch; it has already been drained.

## Why the burn-down is not being done in this lane

This branch carries the port-engine PR, and it has already had one accidental mass deletion — 2354
files staged by `git add -A` while an external process removed the worktree mid-commit (see R2t).
Deleting another ~350,000 lines into the same pull request would make it unreviewable and would be
indistinguishable, in the diff, from a repeat of that accident.

The burn-down is a separate lane with its own PR, and it has a prerequisite that is real work rather
than a formality: seven dependents for `oya/`, three for `os/`. The decision above is what unblocks
that lane; it is not itself the lane.
