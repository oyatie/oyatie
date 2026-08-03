# Pipeline friction: derived affected-set seeds + productized workflow guards

## Problem Statement

**How might we give infrastructure and governance-data PRs a proportionate CI verdict, and let
authors make existing shell *safer*, without weakening either fail-closed rule?**

## The two frictions, measured

**1. Every infrastructure PR pays a full-workspace build.** `gate · affected-set` took **25m42s**
on PR #1450 plus a full `buck2` leg. ADR-0554 round-6 defect 2 is explicit and correct: *"every
changed path must map to a buck2 `owner()` OR an explicit synthetic-dependency declaration here,
otherwise it escalates to FULL."* The rule is fail-closed and right. The **table is incomplete** —
it holds 6 entries (`.github/**`, `docs/**`, `**/*.md`, `**/*.mdx`, `docs/ideas/archive/**`, one
specs file). `infra/**` and `specs/*.json` appear nowhere, so touching a Dockerfile, an ARC values
file, or the reachability registry escalates to `//...`.

Note `.github/**` is *well* mapped — 21 gate targets. The naive diagnosis ("workflows are
unmapped") is wrong.

**2. The inline-shell ratchet penalises safety.** `workflow_inline_shell` is shrink-only and
growth is **born-blocking**. Fixing a genuinely broken step (a bare `git fetch origin` that cannot
authenticate against a PRIVATE repo under `persist-credentials: false`) required *adding* a guard
line — so the fix was blocked at `line count grew from frozen baseline 1 to 2`. It only landed
after being golfed into one dense line. The gate cannot distinguish a line that adds an assertion
from a line that adds risk, so **improving existing shell costs more than leaving it broken**.

## The bar: global · canonical · portable · productized · comprehensive

Founder constraint (2026-07-31): the pipeline **is a product**. Every element of it must be
global (not repo-specific), canonical (one way, not N), portable (runs on any repo), productized
(a product, not a script), and comprehensive (no gaps). Two candidate fixes were rejected against
this bar before the recommendation below:

- **Hand-adding `infra/**` rows** — fails *global* and *canonical*: it encodes one repo's current
  directory names into a governance table, and those names are themselves reorg targets.
- **Deriving seeds from `ci/facade/*` `scan_roots`** — fails *portable*: it hard-codes THIS repo's
  gate layout. An affected-set engine that requires a per-repo path table is a config file with
  extra steps, not a product. A repo with no `ci/facade/` tree must still get correct seeds.
- **`preflight --assert-base-ref`** — fails *comprehensive*: one flag per guard means the next
  guard is another bespoke flag, forever.

## Recommended Direction

**Make input-declaration a first-class, repo-agnostic contract — then derive both the seeds and
the guards from it.**

*Seeds:* the portable invariant is not "read `scan_roots`". It is: **every tool that consumes repo
content declares its input classes, and the engine seeds from declarations plus the build graph's
own `owner()`.** `scan_roots` is merely this repo's current spelling of that declaration. The
engine reads the declaration interface; a repo with different tooling supplies its own declarations
and gets the same behaviour. `synthetic_dependencies` stops being a governance table and becomes a
*derived projection* — which is what makes it survive `infra/` → `iac/` and `specs/` →
`governance/` without an edit.

The decisive argument is **not** elegance. It is that **both paths in question are transitional**:
`infra` is neither a capability nor a meta-dir in the CLOSED registry (it distributes to `iac/`,
`ci/`, `network/`, `secrets/`, `storage/`, `observability/`), and `specs` is likewise absent —
ADR-0562 §114 says the registry's *"eventual home is `governance/capability-registry.json` after
the reorg; held at `specs/` until the `governance/` top-level dir exists."* A hand-written
`infra/**` row is a row that must be rewritten the moment the reorg touches it. **Hand-mapping a
dying path is migration debt** — the exact thing the reorg exists to stop accruing. A derived
table follows the move for free.

*Guards:* keep the ratchet strict and make the productized path cheap — but **data-driven, not
flag-driven**. A preflight product evaluates a declared set of *preconditions* (base ref reachable,
toolchain present at the path the build actually invokes, disk headroom above the measured
footprint, required credentials projected) expressed as data, exactly as the disk-reclaim preflight
already is. Adding a guard becomes a row, not a new flag and not a new shell line. That is what
makes it comprehensive: the twenty remaining inline-shell steps migrate into the same contract
instead of each earning a bespoke shim.

Precedent exists in-tree and is named by the policy's own `_comment`: FRIC-017's disk-reclaim
blocks were productized this way ("pipeline-glue(b)") into
`oya-cloud-ci-runner-disk-reclaim-bin` — *"a build-as-user + sudo-binary invocation of the
data-driven Rust preflight"*. Follow that shape rather than inventing a second one; two preflight
mechanisms would fail *canonical*.

## Key Assumptions to Validate

- [ ] **Gates declare their inputs honestly.** Derivation is only as complete as `scan_roots`. Test:
      diff the derived table against the current hand-written one; every existing entry must be
      reproduced or consciously dropped. A gate that reads a path without declaring it stays
      invisible — that is the failure mode to hunt.
- [ ] **`infra/**` genuinely has consumers.** Test: run `automation-language-policy` (scans it for
      shell) and born-accounting against an `infra/`-only diff and confirm non-empty findings.
      If a class truly has none, it must still NOT be declared `[]` — PR #1389 declared `.github/**`
      inert and a workflow-only PR walked past the no-new-shell ratchet; it was reverted.
- [ ] **A preflight binary is cheaper than the shell it replaces.** Test: replace the two base-ref
      assertions first (smallest real case) and measure the inline-shell baseline shrink.

## MVP Scope

**In:** a producer emitting `synthetic_dependencies` from gate `scan_roots`, plus a completeness
gate asserting the derived table covers every tracked top-level path class. Then
`oya-cloud-ci-preflight-bin` with ONE subcommand (`--assert-base-ref`) replacing the two inline
guards.

**Out of the MVP:** migrating the other 20 inline-shell steps to the binary. Prove the seam on the
two guards first.

## Not Doing (and Why)

- **Hand-adding `infra/**` and `specs/**` rows** — both directories are reorg targets; the rows
  would be rewritten within the program. This is the option the founder's own question killed.
- **Declaring any class inert (`[]`)** — a documented false green with a reverted PR (#1389) behind
  it.
- **Exempting "assertion-only" lines from the shell ratchet** — it makes a mechanical gate
  adjudicate intent, and every author believes their line is the safe kind.
- **Retuning affected-set tiering while the required lane is mid-migration** — the flip to owned
  runners is the only CI signal that currently exists; changing tier derivation underneath it
  would confuse the very evidence being relied on.

## Open Questions

- Does any gate read a path it does not declare in `scan_roots`? That set is the derivation's blind
  spot and should be measured before the hand-written table is deleted.
- Should the completeness gate be born-blocking on day one, or advisory until the reorg settles the
  top-level path set? (Born-blocking is the repo's default; advisory is the exception that needs
  justification.)

## ADR verdict — asked and answered

**No ADR blocks best practice here.** ADR-0554's fail-closed unmapped-path rule is *correct*; only
its data is incomplete. The one genuine tension is the inline-shell ratchet's incentive — and the
policy already names the right escape (productize to Rust), it was simply more expensive than the
two-line fix. Funding the preflight binary removes the tension without weakening the ratchet.
