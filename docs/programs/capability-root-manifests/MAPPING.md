# Capability-root service manifests — mechanical mapping

Founder ruling `oyatie-0s8`. This document is the conversion table for one recurring unit of work:
**give a capability root its own `<root>/manifest.json`.** It is written to be applied mechanically,
and to be checkable by a reviewer who sees only a diff.

It is not a narrative of the change. It is the rule set every unit obeys, plus the traps that make
the obvious translation wrong.

---

## 0. The population, re-derived — the ruling's list is wrong by one, and the one matters

The ruling says *six* roots lack a top-level manifest and names `cell, comms, data, iam, os,
workflow`. Re-derived at `origin/dev` `885794461`:

```
git ls-tree -r --name-only origin/dev | grep -E '^[^/]+/manifest\.json$' | wc -l
15
```

| quantity | ruling | measured |
|---|---|---|
| roots WITH a top-level manifest | 14 | **15** |
| roots WITHOUT | 6 | **5** |
| the missing set | cell comms data iam os workflow | **cell comms data iam workflow** |

Two corrections, both load-bearing:

**(a) Fifteen, not fourteen.** `audit/manifest.json` predates PR #1620; #1620 added the other
fourteen. The ruling counted #1620's diff, not the tree.

**(b) `os` is not in the population at all.** The denominator "twenty capability roots" is exactly
`governed_service_roots` (22) minus `cloud` and `oya`, which hold their services one level down.
That same 22-entry list appears identically in BOTH consumers —
`ci/facade/service-tier-metadata/tier-field-coverage-policy.json` `governed_service_roots` and
`ci/facade/product-protocol-policy/product-protocol-policy.json` `manifest_inventory.governed_roots`.
`os` is in neither. `specs/capability-registry.json` is `closed: true`, lists 24 capabilities with
`os` absent, and lists `os/` under `meta_directories` — the closed allowlist of trees that may
legitimately carry no tier. `ci/facade/layer-dependency-acyclicity` rule R6b *positively asserts*
that classification.

So an `os/manifest.json` would be collected by no gate and read by no consumer. **PR #1607 was the
instance of the defect, not the fix for it** — a file that satisfies a human's file-existence
intuition while no consumer reads a byte of it is the declaration-wired-to-nothing class the ruling
itself warns is worse than the gap. It is correctly excluded, and excluding it is the ruling applied,
not the ruling narrowed.

(For the record: PR #1607 does not even *add* `os/manifest.json`. It moves and edits one — the
rehomed `cloud/cloud-os` service manifest, `+3 −3` inside a 10-file reorg PR.)

### The wider class, surfaced not settled

Four registry capabilities are absent from BOTH consumers' governed-root lists, so no gate looks for
a manifest there at all: **`ci`, `compute`, `messaging`, `policy`** (`policy` has no top-level
directory). Closing the governed gap still leaves 9 of 24 registry capabilities without a root
manifest. Whether those four should be governed cannot be settled by a unit of this work: the
registry's own `_comment` lists `ci`, `compute` and `messaging` among capabilities deliberately left
tier-undeclared, and unlike `stratum`, `tier` has no `forward-declared` escape value. **Do not
invent tiers for them.** Escalate.

---

## 1. The unit

> **UNIT = one capability root gains one `<root>/manifest.json`.**

Five units in this goal: `cell`, `comms`, `data`, `iam`, `workflow`. Each is independent; each is
committed directly to `impl/0s8-capability-manifests`; no unit opens a PR.

---

## 2. Recurring patterns → what they become

| pattern in the root | becomes, in the manifest |
|---|---|
| crates directly under `<root>/{core,ports,facade,adapters}/` declared by no other manifest | `capability_root_accounting.crates_under_capability_root` + `crates_by_layer`, **counted off the tree** |
| a crate ALSO declared by an absorbed sub-service manifest | named in `crates_also_declared_by_an_absorbed_service`, and **subtracted** from `crates_covered_by_no_other_manifest` |
| `<root>/<service>/manifest.json` sub-services | `capability_root_accounting.absorbed_services[]` — `{service, manifest}` pairs, path-verified |
| divergent tier/DR facets across absorbed services | `absorbed_service_facets[]` + `absorbed_service_facet_divergence: true`, and the divergence is *recorded*, never averaged |
| the strictest `dr_tier` among absorbed services | `dr_tier`, with `dr_tier_derivation` naming the services and values it was taken from |
| the capability's registry tier/stratum | `tier`, `substrate_dag_position.stratum` — **copied from `specs/capability-registry.json`, never invented** |
| OpenSLO files at capability level | `slos[]`, each entry a path that **resolves to a non-empty file** |
| OpenSLO files only under per-service subdirs | `slo_exemption{status,owner,rationale,cutover_on,evidence}` — never an empty `slos: []` alone |
| the root serves no endpoint of its own | `contracts` present with all three carriers EMPTY, plus a `contracts_non_claim` sentence |
| the root runs no process | `sharding_automation` present with every block `enabled: false`, mode `not_claimed_runtime`, plus a `non_claim` |
| anything the root does NOT claim | an explicit `*_non_claim` / `foundation_non_claims` string — **absence is written down, never left blank** |

**The governing rule of this whole table:** a facet is either MEASURED off the tree or DECLARED as a
non-claim in prose inside the manifest. There is no third option. A field that is present because a
gate wants it present, carrying a value nobody measured, is the defect this work exists to close.

---

## 3. Conventions every unit obeys

**Naming.** File is exactly `<root>/manifest.json` — lowercase, no prefix, no suffix.
`collect_named_files` compares `file_name() == "manifest.json"` **exactly**, so
`service-manifest.json`, `client-manifest.json` and `archive-manifest.json` are invisible to the
gate. `microservice` is the bare root name (`"cell"`, not `"oya-cell"`, not `"cell-capability"`).

**Ownership.** `owner` is the literal single token from `<root>/OWNERS`. All five roots already have
their own `OWNERS`, each far under `[owners] max_paths_per_owners_file = 2000`
(cell 122, comms 689, data 619, iam 658, workflow 590 paths), so nearest-ancestor resolution lands
on the root's own file and the oversized repo-root `OWNERS` is never consulted. **Verify this before
writing, do not assume it** — an `OWNERS` over the cap owns *nothing at all* and fails closed with
no fall-through.

**Reachability.** A root manifest is NOT crate-resident, so `cargo-members` cannot reach it. Every
new one needs an **EXACT-path** row in `specs/reachability-registry.json`
(`prefix: "cell/manifest.json"`, not `"cell/"`), inserted at its **sorted** position. A tree prefix
would over-claim the crate and observability subtrees that carry their own separate reasons.

**Anchor text.** The registry anchor names the **CONSUMING GATE**, never the file. "This file
exists" is not an anchor. Correct form: *"…which `cloud-ci-tier-field-coverage` classifies as a
top-level service manifest and reads tier / tier_subtype / dr_tier / substrate_dag_position /
sharding_automation / OpenSLO coverage from, and which `cloud-ci-product-protocol-policy` counts in
its equality-pinned inventory."*

**ADR references.** `adrs[]` entries carry `id`, `title`, `scope` — **and no `file` key**. See
trap T5.

---

## 4. Invariants — checkable on one unit in isolation

Each holds after every unit, independently of the others.

| # | invariant | how a reviewer checks it from the diff alone |
|---|---|---|
| I1 | `tier` and `substrate_dag_position.stratum` EQUAL the root's `specs/capability-registry.json` entry | open the registry entry, compare two strings |
| I2 | every value in `tier` / `tier_subtype` / `dr_tier` / `stratum` is a member of the corresponding `*_enum` in `tier-field-coverage-policy.json` | grep the enum |
| I3 | `sharding_automation` is present with all three sub-blocks (`autosharding`, `auto_rebalance`, `dynamic_sharding`) | structural |
| I4 | disabled `autosharding.mode` is `not_claimed_runtime` (the only member of `allowed_disabled_autosharding_modes`) | string compare |
| I5 | either `slos[]` is non-empty AND every entry resolves to an existing non-empty OpenSLO file, OR `slo_exemption` carries all of `status` / `owner` / `rationale` / `cutover_on` / `evidence` with no placeholder text | `ls` each declared path |
| I6 | `schema_version` is `"1.0"` and `contracts` is an object → the manifest counts as live-v1 | structural |
| I7 | every crate count equals the tree: `find <root> -mindepth 3 -maxdepth 3 -name Cargo.toml` | re-run one command |
| I8 | `crates_covered_by_no_other_manifest` = total − overlap, and the overlap list is NAMED | arithmetic on the diff |
| I9 | every `absorbed_services[].manifest` path exists | `git cat-file -e` |
| I10 | `owner` equals the content of `<root>/OWNERS` | two-file compare |
| I11 | an exact-path row exists in `specs/reachability-registry.json`, list still sorted, still valid JSON | one `python3 -c` |
| I12 | no line in the manifest contains a `decisions/ADR-NNNN` path | `grep -c 'decisions/ADR-'` → 0 |
| I13 | every count/path/threshold in the manifest was read off the tree, or is inside an explicit `non_claim` string | read the diff |

Measured crate counts for the five (all confirmed against the tree):
`cell 8` (core 5, ports 3) · `comms 24` (core 12, ports 5, facade 5, adapters 2) ·
`data 23` (core 16, facade 5, ports 2) · `iam 68` (core 18, ports 14, facade 22, adapters 14) ·
`workflow 48` (ports 16, core 12, adapters 11, facade 9). Total 171.

---

## 5. TRAPS — where the obvious translation is subtly wrong

**T1 — A top-level manifest is not a free declaration; it ACTIVATES checks the root currently
escapes.** `is_top_level_service_manifest` accepts BOTH `<root>/manifest.json` and
`<root>/<service>/manifest.json`. Adding the first shape switches on `evaluate_sharding_automation`
and `evaluate_openslo_manifest_refs` for a root that had never been subject to either. The gate's
own doc comment records the mirror-image bug: requiring two path components silently dropped
fourteen substrate services from both checks **while the gate stayed green**. Expect ~10 TFC codes
to become applicable at once. A stub REDs on most of them simultaneously.

**T2 — Tier authority is SPLIT, and the manifest is NOT the authority for a capability root.**
`ci/facade/layer-dependency-acyclicity` states it as doctrine: *"A capability DECLARES its tier;
nothing derives it."* For SERVICE roots the tier comes from `manifest.json`; for CAPABILITY roots it
comes from `specs/capability-registry.json`, and the manifest-derived path was **deliberately
deleted** because it resolved only while absorbed dirs still existed and let migration order pick
the answer. So the `tier` you write here is a MIRROR, not a source. If it disagrees with the
registry you have recreated the second authority that was removed on purpose — and no gate will tell
you, because the acyclicity gate reads the registry and never opens your file.

Corollary: `layer-dependency-acyclicity`'s module doc at line 111 still says the tier comes from
`<capability>/manifest.json`. That prose is **stale** — line 496 resolves via `registry_entry(…)`.
Believe the code. (Separate fix, not this goal's.)

**T3 — `iam` has NO registry tier and NO stratum, and the honest move is not to invent one.**
`iam` is a registered capability that the closed registry deliberately leaves untiered — its
`_comment` explains it spans S1+S3 with three untiered absorbed services. `substrate_dag_stratum_enum`
contains the value **`forward-declared`** for exactly this case; use it, and record the split in
`absorbed_service_facet_divergence` rather than picking S1 or S3. Writing a plausible stratum to make
a gate pass is the silent-exemption pattern R6c exists to RED.

**T4 — `covered_by_no_other_manifest: true` is a boolean that is FALSE for `data` and looks true.**
`data-cloud-domain` and `data-cloud-kernel` are also declared by `data/cloud-data/manifest.json`, so
the honest number is **21 of 23, not 23**. A boolean cannot express that, which is why this field is
a computed integer plus a named overlap list. This is the entire class in miniature: the flat
assertion passed every check and was untrue. (Fixed on the branch by `4f59f99ba`.)

**T5 — The `adrs[].file` key is a live citation, and `docs/decisions/` holds only ADR-0700..0709.**
Any `decisions/ADR-NNNN` path naming a non-apex is a **new `adr_citation_dangling_path` finding** and
REDs the census gate. Cite by bare `id` only (which registers no citation line at all on a
non-authority surface), or by a `docs/adr-archive/…` path. Note that `flags/manifest.json` already
carries the broken form — an `ADR-0131` id paired with an ADR-0701 apex path — so **copying an
existing manifest wholesale propagates a defect.** Read before you copy.

**T6 — `specs/microservices/manifest-schema.json` is the wrong target and is enforced on nothing.**
It appears only as `artifacts.manifest_schema`; there is no per-manifest validation loop. Proof that
it is inert: `flags/manifest.json` is missing 6 of its 19 required fields and is live and green.
Worse, it is `additionalProperties: false` and FORBIDS the very `tier` / `tier_subtype` / `dr_tier`
fields the tier-field-coverage gate MANDATES. Author to the **gate**, not to the schema.

**T7 — `min_expected_service_manifests: 95` is a FLOOR, not a pin, and it is slack by six.** It will
not catch a loss. What catches loss is `product-protocol-policy`'s `expected_total`, an `assert_eq`.
Do not read a green tier-coverage run as evidence that the inventory is intact.

**T8 — The census moves and must be RE-FROZEN BY MEASUREMENT, in the same commit.** Every
`<root>/manifest.json` is a `.json` outside the exempt prefixes, so it is `+1` to
`adr-citation-closure` `files_scanned` — pinned by EQUALITY, so it REDs immediately. Three pins move
for the five manifests, all confirmed by running the gates:

| pin | before | after | cause |
|---|---|---|---|
| `manifest_inventory.expected_total` | 96 | **101** | +5 governed manifests |
| `manifest_inventory.expected_live_v1_total` | 58 | **63** | same five, all `schema_version` 1.0 |
| `measured.files_scanned` | 16524 | **16529** | five new `.json` files |
| `measured.citation_lines` | 8896 | **8896** | unchanged — bare ids, no `decisions/` paths (T5) |

A `+5` in `expected_total` that failed to move `expected_live_v1_total` would be the signature of a
manifest that is NOT live-v1 — chase it, do not average it. Edit these policies as **TEXT keyed by
name**; round-tripping through JSON reformats the whole file.

**T9 — Born accounting fires ON THE NEW FILE, and it fires late.** The five manifests are not
crate-resident, so ADR-0555 accounting reports them `unreachable`, and `unreachable` implies
`unjustified` — **ten regressions from five files.** This reached CI as a RED
`cloud-ci-firewall` on PR #1629 before it was caught locally. Two doors are offered and only one is
true: REGISTER (these files are genuinely read by two gates) rather than RAISE THE BASELINE (which
would be a silent exemption dressed as a fix). Run the firewall locally, before the push.

**T10 — Location for any accompanying DOCUMENT is an accounting decision, not a filing preference.**
A top-level `docs/*.md` is structurally unownable: ownership resolves to the nearest-ancestor
`OWNERS`, a `docs/OWNERS` would cover 2631 paths, and an `OWNERS` over the 2000-path cap owns
NOTHING and fails closed. Adding one would look like a fix while changing nothing. Hence this
document sits in `docs/programs/capability-root-manifests/` with its own `OWNERS` and its own
registry prefix — the same shape `docs/security-program/` and `docs/programs/k8s-port/` already use,
for the same reason.

---

## 6. DEFINITION OF DONE — one unit

A reviewer holding only the diff can apply every line of this.

1. **One new file**, `<root>/manifest.json`, plus the co-required rows listed in 4–6 below. Nothing
   else in the diff.
2. `schema_version` `"1.0"`; `microservice` is the bare root name; `owner` matches `<root>/OWNERS`.
3. **I1–I13 all hold.** In particular I1 (tier mirrors the registry), I7/I8 (crate counts equal the
   tree, overlap subtracted and named) and I12 (no `decisions/ADR-` path).
4. **`specs/reachability-registry.json`** carries an EXACT-path row for the new manifest, at its
   sorted position, with an anchor that names the consuming gates.
5. **Every equality pin the file moves is re-frozen in the SAME commit**, with the delta ATTRIBUTED
   in the policy's own `_comment` — the named file, the named cause. Never by arithmetic. A narrowed
   scan and a genuine add produce the same number and only one is legitimate.
6. **No `*_non_claim` string is missing.** For every gate-mandated block the root does not actually
   do (`contracts`, `sharding_automation`, `dr`), the block is present, disabled/empty, and carries a
   sentence saying why that is the true state.
7. **Gate evidence in the commit message, literal buck2 output including the `Commands:` line**, for:
   `ci/facade/service-tier-metadata`, `ci/facade/product-protocol-policy`,
   `governance/check/adr-citation-closure`, and the `cloud-ci-firewall` born-accounting leg.
   A green count alone is not evidence — for anything that MOVES a file, diff the failing set at the
   untouched base against the failing set at head.
8. **Commit with a pathspec**: `git commit -- <paths>`. Never `git add` then `git commit` — the index
   is per-worktree and therefore shared between lanes, and `git commit` commits the INDEX.

### The one-line reviewer test

> Pick any number, path or threshold in the manifest at random. Can you name the command that
> produced it, or find the `non_claim` sentence that declares it unclaimed?

If neither, the unit is not done — it is the declaration-wired-to-nothing defect wearing a manifest
costume, which the ruling explicitly rates **worse than the gap it closes**.

---

## 7. Per-root findings — the five do not lack manifests for the same reason

The ruling asked for this to be surfaced separately rather than flattened.

| root | crates | shape | SLO disposition |
|---|---|---|---|
| `cell` | 8 | **The "documentation with stub crates" case the ruling anticipated — and it is `cell`, not `os`.** 121 files, 51 `.md`, 22 `.rs`. Both named services (`cell-lifecycle`, `cell-rebalancer`) are documentation-and-helm only: ARCH/PRD/README/IPs/runbooks/contracts/dpia plus a manifest each, and **no `.rs` at all**. The capability's real code is in `core/` and `ports/`. | exemption |
| `comms` | 24 | four full layers; 71 OpenSLO files, all under per-service subdirs | exemption |
| `data` | 23 | the ONLY root with genuine crate overlap (T4) — 21 of 23 uncovered | **real SLO** (1) |
| `iam` | 68 | largest; registry-untiered by design (T3); stratum `forward-declared` | **real SLOs** (6) |
| `workflow` | 48 | four full layers; 35 OpenSLO files, per-service | exemption |

And the inverse finding, worth stating because it is what the held-open PR was actually about:
**`os/` is all code and zero prose** — 558 files, 41 `Cargo.toml`, 373 `.rs`, **0 `.md`**, 0 nested
manifests, 0 OpenSLO. Its 41 crates genuinely carry no tier, DR or ownership accounting anywhere.
That gap is real; a capability-root manifest is simply the wrong instrument for it, because `os` is a
meta directory and no consumer would read the file. Fixing it means a governance ruling on whether
`os` joins the closed registry — not a JSON file added to a tree nothing scans.

---

*Authored against `origin/dev` `885794461` and `impl/0s8-capability-manifests` `825b0606a`. Every
count in §0, §4 and §5 was re-derived on that tree; the pin values in T8 are the values the gates
reported, not arithmetic.*
