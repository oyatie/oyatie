# RALPLAN — Finish the capability reorg, enforcement-first

**Status: PENDING APPROVAL.** Planning artifact only. No execution authorized.
**Date:** 2026-07-27 · **Mode:** deliberate · **Revision:** 5 — APPROVED by critic; 5 mechanical corrections applied (rev 1 UNSOUND — table deleted · rev 2 SOUND-WITH-CHANGES — 4 amendments · rev 3 architect SOUND, critic ITERATE — 10 items applied)
**Evidence base:** `.omc/specs/deep-dive-trace-bun-fleet-delivery-benchmark.md`

## What changed from rev 1, and why

Rev 1's centerpiece was a 466-row precomputed decision table. **It was deleted.** Three verified defects killed it:

1. **The destination column is already authored.** Resolving all 467 legacy crate dirs against *all four* registry rule sources — `capabilities[].absorbs_current_dirs` **plus** `membership_lint_coverage.{app_products, meta_directory_absorbs, absorbs_current_crate_globs}` — yields **407/467 (87%) declared, 60 unmapped**. *(Corpus: `Cargo.toml` count under `oya/ + cloud/ + libs/ + tools/` on `origin/dev` = 467 exactly.)* The 60 are exactly `frozen_unmapped_baseline`, already gate-tracked with `burn_down_target: 0` and blocking on new entries. Rev 1's "49%/51%, 236-crate Track 2" was a measurement error: only the first rule source was read.
2. **It violated principle 1.** Grep of all five cited authorities for `decision table` / `precompute` / `work-list` / `LIFETIMES` returns zero hits. The table was net-new design in a plan whose first principle forbids net-new design.
3. **Wrong unit.** ADR-0562 §10 makes the execution unit the capability, not the crate: *"each capability moves in ONE PR … before the next capability moves."* Capability moves run ~2 per 40 merges.

**Also corrected:** rev 1 called mechanism (f) silent laundering. `TDA-STALE-BASELINE` is a **blocking** regression that fires on exactly this. The real defect is that the sanctioned remedy (`--emit-baseline`) *drops* the row — **one-command sanctioned laundering**, not silent laundering, and therefore cheaper to close.

## Ratified context this plan implements (not proposes)

- **ADR-0627 §4** (2026-07-26): *"The capability migration continues to completion … **reversed** by founder decision 2026-07-27."* Reorg-finishes is recorded.
- **ADR-0563 §2(3):** *"tier-dep endpoints map via **crate-IDENT pairs** with a candidate-edge existence guard"* — *"STRICT NO-OP when there are no renames, per-(gate,code) injective, fail-closed on collisions"*, and it *"can never manufacture a false-GREEN."* **This is the relocation-survival mechanism this plan wires (S3).** Live in `ci/adapters/path-resolver`; not consumed by the tier gate.
- **ADR-0627 §2 — cited as a DEFECT, not as the mechanism.** It claims *"Baseline keys are cargo package names … Package names survive relocation."* That premise fails here: every capability move is move **+ rename**, and under the de-brand rule the cargo name *is* the path tail, so a package-name key is path-derived by construction and goes stale exactly when the path does. ADR-0627's own 35-key facade baseline inherits this. See Follow-ups.
- **ADR-0617 Alternatives:** *"Ratchet-from-debt enforcement. **Rejected by the founder** in favor of fail-closed after a one-time cleanup."*
- **ADR-0512:** structural migration *"MUST run as a dedicated, exclusive … never merged concurrently in a PR drain."*

## Principles

1. Implement ratified design; net-new design is the failure mode. **Operational test:** *a scope item must name the file and symbol that already implements the mechanism it wires; if it cannot, it is net-new design.* (ADR status is not the discriminator — it admits every Proposed ADR and resolves nothing. This test catches all three violations this plan produced: rev 1's decision table, rev 2's re-key, and S1's slop gate — while admitting S4, which modifies an existing symbol. Repairing a live mechanism is not net-new; inventing one is.)
2. Parallelize decisions; serialize structure.
3. A move may not launder debt — and the fix is relocation-invariant keys, not a frozen snapshot of bad state.
4. Enforcement before throughput.
5. Evidence over narration: any claim of green names the gate, its corpus, and the count.

## Decision drivers

1. **60 undecided crates** are the only place authorship is the ceiling — and `base/` admission reads the **capability's** DAG position, so they are undecidable until the missing capability nodes exist and `base/` exists. (Crates never receive DAG nodes.)
2. **Crates under capability roots are tier-blind.** #1423 made the root config load-bearing but left `service_roots: ["cloud","oya"]`, so `owning_service` returns `None` for every capability root and R1–R4 skip those edges. **Reproducible corpus:** the gate's own emitted `crates[].service == null` from `collect_corpus` — cite that count, not a hand census. (Rev 3 asserted "402"; an independent recount gave 412, and neither is reproducible from a named corpus.) Every move widens it.
3. **Three gate gaps are verified absent and cheap** — and they are the enforcement that must precede any throughput work.

## Rev 3 amendments (architect re-review: SOUND-WITH-CHANGES)

1. **S3 was another principle-1 violation — replaced.** Rev 2 proposed re-keying the tier baseline to cargo package names per ADR-0627 §2. That premise fails on this repo: **every capability move is move + rename** — the de-branded cargo name *is* the path tail (`iam/adapters/cloud-oci` → `iam-cloud-oci`; `os/core/apid-domain` → `os-apid-domain`). A package-name key is therefore a function of the path and is exactly as stale after a move. The ratified rename-aware mechanism already exists: **ADR-0563 §2(3), "tier-dep endpoints map via crate-IDENT pairs with a candidate-edge existence guard"**, implemented live in `ci/adapters/path-resolver/src/lib.rs` (`ManifestBijection`, `crate_ident_pairs`, `MOVE_MANIFEST_SCHEMA`). **Verified gap:** `ci/facade/layer-dependency-acyclicity` does not consume it — zero references; its entire `[dependencies]` is `serde_json`. So S3 is *wiring an existing mechanism to a gate that lacks it*, not inventing one.
   *This also raises a defect against ADR-0627 §2 itself: its 35-key facade baseline inherits the same path-derived-name assumption. File separately.*
2. **S5 rescoped and its cost stated.** Rev 2's criterion "all 60 have DAG nodes" was a category error — **crates do not get DAG nodes; capabilities do.** The DAG holds **10 nodes** at substrate-service granularity, against the 24 that ADR-0562 §2 and ADR-0615 §5 both assert map 1:1. **14 capability nodes are missing**, and authoring one is a *design act*: `dr_tier`, `slo_floor`, `brownout_protocol_version`, `chaos_drill_cadence_days`, plus per-edge `cascade_rule`, `cedar_permit_fragment`, `dependency_weight`. **S5 is the plan's long pole and is founder-gated architecture work, not a fill-in-the-graph chore.**
3. **Pre-mortem 3's mitigation replaced.** Rev 2 proposed emitting a baseline over the ~402 newly-visible violations — that is ratchet-from-debt, which contradicts this plan's own principle 3 and the ADR-0617 ruling quoted in Ratified Context. Replaced with **staged per-root one-time cleanup**: classify one root at a time, clean its violations, move on. If a root's violations are too many for one PR, stage within the root — never baseline the lot.
4. **Order corrected.** "Enforcement before throughput" means *the enforcement that is actively bleeding, first* — not "gates before moves." Driver 2 says every move removes crates from tier enforcement **now**; S1's fanout justification evaporated when rev 1's table was deleted, and nothing blocks on S1. — **SUPERSEDED by rev 4 A5:** the S3→S4 order this amendment set was itself an enforcement regression and is reversed below. The principle stated here still holds; its application did not.

## Rev 4 amendments (critic verdict: ITERATE — 10 items)

**A5 — Order reversed; S3-before-S4 was an enforcement regression.** Verified in `ci/facade/layer-dependency-acyclicity/src/lib.rs`: today a stranded subject fires `TDA-STALE-BASELINE` as a **blocking regression** (the doc comment: *"A phantom row is a blocking regression whose remedy is re-emitting the baseline (`--emit-baseline`), which drops it"*). After S3 alone the relabel anchors the row — but `owning_service` still returns `None` for capability roots, the crate is pushed `"service": null`, and R1–R4 hit `let (Some(src), Some(dst)) = (tier_of(..), tier_of(..)) else { continue; }`. **S3 before S4 converts a visible forced decision into a silent green.** The stated justification ("a baseline ~50× larger") was unmeasured against a live baseline of 8 rows — **claim dropped**.

**A6 — S4 respecified against the code.** `owning_service` returns `<root>/<first-segment>`, so naively adding capability roots to `service_roots` yields *face-granular* services (`iam/core`, `iam/adapters`, `iam/facade`), and `collect_service_tiers` reads `<governed-root>/<name>/manifest.json`. **Measured: 95 face-level dirs** would each need a manifest, duplicating one tier 3–4× per capability with nothing asserting agreement — the drift this plan exists to stop. **S4 is therefore a CODE change, not data authorship:** `owning_service` returns the capability root itself for capability roots (a capability is the ownership unit per ADR-0562 §1; faces are sub-folds, not services), and `collect_service_tiers` reads `<capability>/manifest.json` — **25 manifests (24 capabilities + `os`), not 95**. `os` is `unclassified_roots` index 26 of 27 with 41 crates; omitting it would leave those 41 tier-blind.

**A7 — S1 is the third principle-1 violation; descoped.** No ADR mandates a `todo!()`/`#[ignore]` gate. ADR-0555 D4 is a *surface* ruling ("gates are canonical; CLI bridges transitional") governing how a gate ships, never that this one should — correctly quoted, one clause from what it says. S1 moves to Follow-ups pending an authority. Its principle-3 defect is moot at that point, but recorded: "born-blocking against a **frozen baseline**" over a corpus of **10** `todo!()`/`unimplemented!()` (9 files) and **77** `#[ignore]` (21 files) — ten is one cleanup PR, so frozen-empty after cleanup costs nothing.

**A8 — S7 decoupled from S5/S6.** Consequences serialized structure behind decisions, inverting principle 2. Only capabilities whose crates sit in the 60 depend on S5/S6; the rest can move as soon as S4→S3 land.

**A9 — corpora named, sample replaced by census.** 467 = `Cargo.toml` count under `oya/ + cloud/ + libs/ + tools/`. The 402 tier-blind figure is **not reproducible as written** and is replaced by the gate's own emitted `crates[].service == null`. The de-brand RED-wave sample (147 crates, "≥40 tails") is replaced by the full census: **189 of 412** capability-root package names carry non-allowed role tails — the sample understated by 4.7×, in the safe direction (it under-sold the deferral's justification).

**Execution order: S4 → S3 → S2 → S5 → S6 → S7** — or **S3+S4 co-landed in one PR**, which is equally safe and preserves relabel-first intent. S7 runs concurrent with S5, excluding capabilities whose crates are in the 60. S1 is descoped to Follow-ups.

## Rev 6 — S4+S3 SHIPPED (commit `0087b2796`, branch `fix/tier-classify-capability-roots`)

**S3 was mis-scoped in every prior revision, and building it proved so.** Rev 3–5 said "the ratified relabel exists and the tier gate simply doesn't consume it." Wrong in three ways, all verified in source:
1. `relabel_tier_dep_gate` **does** exist (`scm-facts-snapshot/src/lib.rs:3023`, wired at :2795) — it was never missing.
2. It is **inert by design**: *"the tier-dep gate is a standalone advisory baseline, not folded into the firewall face — this branch fires only if the gate is ever added to the firewall gates,"* with an existence guard that *"cannot be satisfied"* at that boundary.
3. **Key-shape mismatch**: it maps `crate_ident_pairs`, but tier baseline subjects are crate **dirs** (`oya/intelligence/crates/… -> oya/application/crates/…`). It could never match.

So S3 was writing a **new dir-pair relabel at the gate**, reusing `MoveManifest::crate_dir_pairs()`. Admissible under principle 1's test (names the file and symbol), but it is new code — not wiring.

**Also corrected:** the baseline holds **11** rows, not the 8 carried since rev 2; its schema is `{_comment, gate_id, frozen_at_ref, burn_down_target, violations}`, not a `keys` array.

**Shipped:** 431 crates brought under enforcement (252 → **683** `service != null`). 24 capability roots classified via a new optional `capability_roots` policy key; `owning_service` returns the capability root itself, so tier lives at `<root>/manifest.json` — **25 manifests, not the 95 face-level ones** the naive `service_roots` addition would have required. Both halves RED-proven. 32 unit + 7 gate tests green, move-free.

**Three roots deliberately not classified:** `policy` (0 crates — a manifest would classify nothing while incurring ADR-0555 born-accounting); `billing` and `marketplace` (**staged out** — classifying them surfaced two genuine ADR-0245 R1 violations, which ADR-0617 requires be *fixed* not baselined, and ADR-0512 forbids mixing that refactor into a structural PR). They land at **705** once those two edges are clean.

**Enforcement gain is partial by design:** R1/R2/R3/R5 are live for all classified roots now; R4 activates per-capability as real S-ranks land with S5's DAG nodes — every stratum ships `forward-declared`, which is rank-exempt and cannot manufacture a false green.

## Scope

**S1 — DESCOPED to Follow-ups (rev 4).** The observation stands: no gate detects `todo!()`, `unimplemented!()`, `#[ignore]`, or deleted assertions, and the only vacuous-test detector is a shell hook scoped to `libs/oya-check-*` that always exits 0. But **no ADR mandates such a gate**, so shipping it here is net-new design under a plan whose first principle forbids it. It returns to scope when an authority exists. If it does: corpus is **10** `todo!()`/`unimplemented!()` across 9 files and **77** `#[ignore]` across 21 files — small enough to clean once and ship **frozen-empty** (matching `core-dependency-isolation`'s posture), never against a frozen baseline.

**S2 — Defuse the `ports/` trap (enforcement, latent blocker).**
`facade-core-layering` emits a different code depending on whether a capability has any `ports/` dir; the two codes have separate baselines. `intelligence/` and `compute/` have none. The first PR adding any crate under `intelligence/ports/` REDs twice — five keys flip to an unbaselined code *and* their old rows go stale. Must be defused before any move or clean-arch work touches those capabilities.

**S3 — Wire the ADR-0563 crate-IDENT relabel to the tier gate (SECOND — after or co-landed with S4).**
Do **not** re-key. The ratified rename-aware relabel exists and is live in `ci/adapters/path-resolver`; the tier gate does not consume it (verified: zero references, `[dependencies]` = `serde_json` only). Wire it, so a move+rename relabels its baseline subjects instead of stranding them as phantoms that `--emit-baseline` silently drops. Its stated properties are exactly what is needed: *"STRICT NO-OP when there are no renames, per-(gate,code) injective, fail-closed on collisions"* and it *"can only ever REMOVE a false-RED for a proven pure-or-shrinking relocation; it can never manufacture a false-GREEN."* Then fix the **8** `TDA-SUBSTRATE-UPWARD` edges (all `oya/intelligence/* → oya/{application,community}/*`) before those three services move.
**Must NOT precede S4** (rev 4, A5): alone, S3 removes the blocking RED that is currently the only signal of the enforcement hole, while `service: null` keeps the edge unevaluated — a silent green. Land it **after S4, or co-landed with S4 in one PR**. Rev 3's "~50× larger baseline" justification for the reverse order was unmeasured against a live baseline of 8 rows and is withdrawn.
Land the relabel change in a **move-free PR, proven by `git diff --find-renames` reporting zero renames under `crate_root_globs`** — not asserted.

**S4 — Classify capability roots (FIRST; the bleeding is now).**
**A code change, not data authorship** (see A6): `owning_service` returns the capability root itself for capability roots — a capability is the ownership unit per ADR-0562 §1, faces are sub-folds — and `collect_service_tiers` reads `<capability>/manifest.json`. That is **25 manifests — 24 capabilities + `os`**; the naive `service_roots` addition would instead require **95** face-level manifests (93 under the 24 capabilities + 2 under `os`) duplicating each tier 3–4× with nothing asserting agreement.
**`os` is explicitly included, not exempted.** It is `unclassified_roots` index 26 of 27, a meta-directory with `owns_crates: true`, and holds **41 crates**. A literal "24 capabilities" reading leaves those 41 tier-blind — the exact hole S4 exists to close.
Restores ADR-0245 enforcement to the crates currently emitted with `"service": null`. **Staged per-root, one-time cleanup — not baseline emission.** Exempt `libs/` and `tools/` from tier authorship (stating the residual blindness cost): `libs` at **185 crates** is the largest `unclassified_root` and is the very root S6 dissolves, so authoring tier metadata there is work S6 then discards.

**S5 — ADR-0280 capability nodes + `base/` (THE LONG POLE — founder-gated design work).**

**Measured, not estimated** (corpus: `specs/capability-registry.json` `dag_node` declarations × `specs/substrate-dependency-dag.json` `.nodes[].name`, `origin/dev`):

- **10 capabilities have a node** — cell→`cell`, iam→`identity`, policy→`policy-engine`, tenancy→`tenancy`, secrets→`cloud-secrets`, audit→`audit-chain`, observability→`observability`, data→`ontology`, intelligence→`intelligence`, workflow→`workflow-engine`. All 10 existing nodes are consumed; the mapping is 1:1 over what exists.
- **14 capabilities have none** — storage, compute, k8s→`control-plane`, network, gateway→`api-gateway`, messaging, ci→`delivery-fabric`, iac, billing, marketplace, console, compliance, comms, flags.

The non-obvious aliases (`identity`/iam, `ontology`/data, `control-plane`/k8s, `api-gateway`/gateway, `delivery-fabric`/ci) are **declared registry data, not inference** — so "which nodes are missing" is settled, not an open design question. Zero DAG nodes are unclaimed, so the mapping is exactly 1:1 over what exists. ADR-0562 §2 and ADR-0615 §5 describe the 24↔24 end state; 10 of 24 are realized.

**Registry schema drift found while measuring this** (small, but it defeats a naive census): 23 capabilities declare `dag_node` (singular); **`secrets` alone declares `dag_nodes: ["cloud-secrets"]`** (plural). ADR-0615 §5 records the plural being collapsed for `iam` — *"iam's `dag_nodes` reduced to the single `dag_node: identity`"* — so `secrets` is the un-migrated remnant. Any tool reading only `dag_node` will report `secrets` as unmapped and `cloud-secrets` as orphaned; both are false. Normalize it. Recorded in Follow-ups.

Author the **14 missing capability nodes** and their edges. This is not a chore: each node requires `dr_tier`, `slo_floor`, `brownout_protocol_version`, `chaos_drill_cadence_days`; each edge requires `cascade_rule`, `cedar_permit_fragment`, `dependency_weight`, `version_compatibility_range`, `rationale`. None is derivable from the dependency graph — these are reliability-engineering decisions requiring founder/architecture authorship. Then create `base/` with its ≥3-consumer admission gate.
**Scope reduction available before committing:** the `base/` rule is "≥3 capability consumers **AND** strictly below all of them in the DAG." The consumer census is mechanical today; only the "strictly below" half needs DAG positions, and a leaf library with no back-edges satisfies it under any acyclic ordering. **Measure the residual that genuinely needs a node before authoring all 14** — the founder ask may be materially smaller than 14.

**S6 — Burn down `frozen_unmapped_baseline` (the 60; the real decision queue).**
Per-crate: ≥3 capability consumers and strictly below all of them in the DAG → `base/`; otherwise a capability home. Sixty rows is genuinely parallelizable and genuinely adversarially reviewable. Target `burn_down_target: 0`, which flips the membership lint from advisory to blocking.

**S7 — Resume capability moves**, one per PR per ADR-0562 §10, with **face decided inside each move's PR** where the dependency evidence is current and a reviewer can see it.
**Runs concurrent with S5** (rev 4, A8), excluding only capabilities whose crates sit in the 60. Rev 3 serialized all throughput behind S5/S6, which inverts principle 2 (structure behind decisions) and made a founder stall block moves indefinitely. Nothing establishes that moving `billing` or `gateway` requires the `libs/` 60 homed first; that dependency is specific to `oya-data-boundary-kernel`, already deferred by name.

**Deferred, with reason:** the de-brand profile flip (rev 1's S5 was defective — see below); `oya-data-boundary-kernel` (fan-in 128, in the 60, needs S5 first); review authority (`required_pull_request_reviews: null`; the reviewer dispatcher names a workflow file that does not exist and its runtime crate has no HTTP dependency — recorded as its own program, not sequenced here); implementing `affected-gated-migration-engine.md` (its auto-merge-on-green organ is illegal for structural migration under ADR-0512, and it scopes out semantic transforms).

## Why the de-brand flip is deferred

Rev 1 proposed `required_prefix = ""` under `profile='oyatie'` as "surgical." **It is defective in two independently verified ways:**
- `artifact-inventory-registry` filters the BNF corpus by `name.starts_with(prefix)`. With `""`, the layer-suffix corpus jumps from ~452 branded to all 928, admitting every de-branded capability name into a gate whose `ALLOWED_ROLES` is the 12-value enum in `libs/oya-governance-predictable-naming-kernel/src/lib.rs`. **Full census, not a sample: 189 of 412** capability-root package names carry non-allowed role tails (`contract` 12, `evidence` 11, `connector` 10, `policy` 8, `postgres` 7, `service` 6, `inmemory` 6, …) — a RED wave, not a clean flip. *(Rev 3 cited "≥40" from a 147-crate sample — understated by 4.7×, in the direction that under-sold this deferral's justification.)*
- `cargo_prefix_scope` returns `"advisory"` when the prefix is empty, flipping the entire cargo-prefix gate advisory — the same silent-darkening class rev 1 cited as its reason to avoid `profile='neutral'`.

Correct sequencing: give the layer-suffix gate its own corpus-scope key decoupled from `required_prefix`, run the expanded corpus, count `bnf_unknown_role`, and decide whether `contract`/`evidence`/`manifest` join `allowed_roles` or those crates rename — **before** widening.

## Pre-mortem

**1 — S6 stalls because S5 under-delivers.** `base/` admission reads the *capability's* DAG position; 14 capability nodes are missing. If they are not authored, S6 has no input and the reorg stalls at 87%.
*Mitigation:* S5's exit criterion is **"the 24 capabilities each have a DAG node with declared edges; per-crate `base/` admission is then a mechanical consumer census against those positions."** Crates never receive DAG nodes. Measured, not narrated.

**2 — S5 is founder-gated and stalls indefinitely.** `dr_tier`, `slo_floor`, `cascade_rule` and `cedar_permit_fragment` are reliability decisions no agent may invent. If the founder does not author them, S5 blocks S6 and S7 forever.
*Mitigation:* S5's first act is the **scope-reduction measurement** — the count of the 60 that genuinely need a DAG position, versus those decidable by consumer census plus acyclicity alone (a leaf library with no back-edges is below its consumers under any acyclic ordering). If that residual is small, S6 proceeds on the decidable majority and only the residual waits. The founder ask is then a measured number of nodes, not a blanket 14. **S4, S3, S2 and S7 are all independent of S5, so a stall blocks neither enforcement work nor throughput** — only the capabilities whose crates sit in the 60 wait. (S1 is descoped.)

**3 — S4 lights up every previously-invisible crate at once** (the `crates[].service == null` population — ~412 under capability roots plus 41 under `os`). Classifying them may surface a large wave of real tier violations.
*Mitigation:* **staged per-root one-time cleanup — NOT baseline emission.** Classify one root, clean its violations, move to the next. If a single root's violations exceed one PR, stage within that root. Emitting a baseline over the wave would be ratchet-from-debt, contradicting principle 3 and the ADR-0617 ruling quoted in Ratified Context; the ratified pattern here is one-time cleanup then fail-closed. Roots are ordered smallest-first — `flags` 2, `iac` 5, `marketplace` 5, `observability` 5, `messaging` 6 … `tools` 30, `os` 41, `workflow` 48, `ci` 51, `iam` 68, **`libs` 185** — so the mechanism is proven cheap before it meets the large roots. **Rev 3 named `iam` at 68 as the terminus; the real tail is `libs` at 185**, and since `libs` is the root S6 dissolves, tier metadata authored there is work S6 discards. Hence the `libs`/`tools` exemption in S4.

## Test plan

**Unit.** S3 relabel wiring: the tier gate resolves a baseline subject through `ManifestBijection`. RED test — a subject whose crate moved+renamed strands as a phantom **before** the wiring, relabels **after**. Assert the ADR-0563 properties hold at this call site: strict no-op on an empty manifest, injective per (gate, code), fail-closed on collision.
**Integration.** S2: a synthetic PR adding `intelligence/ports/x` REDs before the fix, greens after. S4: a synthetic upward edge under a capability root is invisible before, RED after — **proving enforcement was restored, not merely that the gate ran.**
**E2E.** One capability move post-S3/S4 with `buck_ok=true` (not null), `cargo_ok=true`, `clean=true`, born-accounting complete, post-move member resolution proven equal (§10.29's "verified by resolution, not by inspection"), **and its tier-baseline subjects relabelled rather than stranded** — the end-to-end proof that S3's wiring works under a real move.
**Observability.** Two counters distinguish repair from laundering and must be reported per PR, not inferred: **baseline rows relabelled** (S3) and **crates newly classified** (S4, from the gate's emitted `crates[].service`). Without them S3 has exactly the "indistinguishable from laundering" shape ADR-0627 §2 warns about. Plus: every blocked/undecidable crate in S6 logged with reason; the blocked count is a reported metric. No silent caps.
**Adversarial.** Re-run §10.29's own false green: any "scanned at the new home" claim must cite the gate's actual corpus membership, not the gate crate's unit suite.

## Acceptance criteria

- [ ] **S4** (first): `owning_service` returns the capability root for capability roots; the gate's emitted corpus shows `crates[].service != null` rising to **≥690** (238 currently classified + ~412 under the 24 capability roots + 41 under `os`; read from `collect_corpus` output, not asserted — a ≥640 target would pass with `os` still blind); a previously-blind synthetic upward edge REDs; each root **cleaned, not baselined**; `libs`/`tools` exemption recorded with its residual-blindness cost.
- [ ] **S3** (after or co-landed with S4): the tier gate consumes the ADR-0563 crate-IDENT relabel; a synthetic move+rename relabels its baseline subject instead of stranding it. The 8 `TDA-SUBSTRATE-UPWARD` edges fixed. Relabel PR proven move-free by `git diff --find-renames` reporting zero renames under `crate_root_globs`. *(Rev 3's "`--emit-baseline` can no longer drop a violation" is dropped — false at S3; the drop closes only once S4 makes the edge evaluable.)*
- [ ] **S2**: `intelligence/` and `compute/` defused, evidenced by a **committed RED fixture** that fails before and passes after. *(Rev 3's "or … documented" disjunct removed — dischargeable by writing a document.)*
- [ ] **S5**: the residual needing a DAG position is **measured before authoring**; the **14** missing **capability** nodes authored with founder sign-off on `dr_tier`/`slo_floor`/`cascade_rule`/`cedar_permit_fragment`; `base/`'s admission check — **already coded and vacuous** (`ci/facade/module-membership/src/lib.rs`) — becomes non-vacuous.
- [ ] **S6**: `frozen_unmapped_baseline.crates` reaches 0 from 60; membership lint flips blocking.
- [ ] **S7**: one capability move lands with `buck_ok=true` (not null), `cargo_ok=true`, `clean=true`, born-accounting complete, post-move member resolution proven equal, and its tier-baseline subjects relabelled rather than stranded. **Face declared in-PR and reviewed by whatever review mechanism is live at the time** — this criterion does not require the review-authority program the plan declines to sequence.
- [ ] *(S1 descoped to Follow-ups — no criterion.)*

## ADR

**Decision.** Finish the capability reorg enforcement-first, in this order: **classify capability roots** so `owning_service` stops returning `None` for them — a code change returning the capability root, not 95 face-level manifests — cleaning each root rather than baselining it; **then (or co-landed) wire the ratified ADR-0563 crate-IDENT relabel to the tier gate**, which does not consume it today, and fix the 8 upward edges; defuse the `intelligence`/`compute` `ports/` trap; **author the 14 missing capability DAG nodes** and make `base/`'s already-coded admission check non-vacuous; burn `frozen_unmapped_baseline` from 60 to zero; resume capability moves one per PR with face decided in-PR, **concurrent with the DAG work** except for capabilities whose crates sit in the 60.

**Drivers.** 87% of destinations are already ratified data; the only authorship ceiling is 60 crates, blocked on capability DAG positions that do not exist. Every crate under a capability root — plus the 41 under `os` — is emitted `"service": null` and skipped by R1–R4, and every further move widens that. The relabel mechanism that would stop moves from stranding their baselines is built but unwired.

**Alternatives considered.**
*Adopt `affected-gated-migration-engine.md` minus auto-merge* — its bulk-rollout, worktree-lane and auto-triage organs are compatible with ADR-0512; only auto-merge-on-green is illegal for structural migration, and it scopes out semantic transforms (face assignment is judgment). **Rejected narrowly:** with the merge organ removed and the transform being one capability per PR, the remaining engine adds orchestration over a serial queue of ~24 items. Revisit for the next genuinely mechanical repo-wide sweep, where it is the right tool.
*Continue hand-shepherded moves **without S4/S3/S2*** — rejected: every move removes crates from tier enforcement, and the de-brand divergence detector does not exist. (Note: S7 **is** hand-shepherded moves; what this plan rejects is doing them without first closing the enforcement holes.)
*(Rev 1's 466-row decision table is documented under "What changed from rev 1" — it was this plan's own prior draft, not an independent option.)*

**Consequences.** No new artifacts and no second SSOT; every remaining item wires, applies, or repairs a ratified mechanism — the one net-new item (S1's gate) was descoped for exactly that reason. S4 surfaces previously-invisible tier violations, **cleaned per-root as a one-time cleanup, not baselined** — its cost is real and paid up front, per the ADR-0617 ruling. S5 is founder-gated design work and is the long pole; **S4, S3, S2 and S7 do not depend on it** — only the capabilities whose crates sit in the 60 wait. Throughput therefore continues during the DAG work rather than blocking on it.

**Follow-ups.** Normalize `secrets.dag_nodes` (plural) to `dag_node` (singular) — the last remnant of a collapse ADR-0615 §5 already performed for `iam`. Note *why* it bites: a singular-only census reports `secrets` unmapped **and** `cloud-secrets` orphaned — two false findings from one missed field, pointing in opposite directions, each of which independently reads as a real gap. It fails loud in a way that looks like signal, which is how it survived. **Cheap detector worth shipping with the normalization:** assert every capability declares exactly one of `dag_node`/`dag_nodes`, and every DAG node is claimed by exactly one capability. That catches this and the next half-finished normalization. (Both reviewers in this plan's own consensus loop were misled by it, in opposite directions — the empirical case for the detector.) File the ADR-0627 §2 defect (package-name keys are path-derived under the de-brand rule, so its 35-key facade baseline inherits the staleness it was designed to avoid). De-brand profile flip with a decoupled corpus-scope key. Review authority deployment. Split the de-brand residue class — `oya-tenant` and `oya` are semver-protected public contracts, not deferred cleanup. Correct ADR-0562 §10.6 (stale: `facade-core-layering` landed) and ADR-0570 (baseline is 6, not 5).
