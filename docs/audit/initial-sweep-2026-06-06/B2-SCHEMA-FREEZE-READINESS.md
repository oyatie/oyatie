# B2 SCHEMA-FREEZE READINESS ASSESSMENT

**STATUS: `READINESS = NEEDS-PREREQ` — schema is STABLE and well-grounded; freeze is one founder declaration away, BUT 3 small open knobs (OPEN-1/-3/-5) and one cross-store scrub residue should be resolved IN THE SAME founder sign-off so the freeze is clean.**

> **Role of this doc:** PREP ONLY. Read-only assessment of the B2 schema-freeze readiness for the BIG HYGIENE PASS. It does NOT pull any trigger — no branch-protection change, no merge, no PR, no commit, no mutation of `/Users/jasonlee/Developer/source`. It assesses whether `STORE-SCHEMA.md` (the B1 deliverable) is stable enough to freeze, and what committing the untracked `registry/stores/` into firewall accounting entails.
> **Authority basis:** `decision-record-oyatie-canon.md` (D-DOCORG, D-SSOT-CURRENT-TRUTH, D-DOCTRINE, D-CLOUD-NATIVE); `BIG-HYGIENE-PASS-PLAN.md` §B1/§B2.
> **Evidence date:** 2026-06-07. CWD self-check passed: `/Users/jasonlee/Developer/source`, branch `cleanup/whole-tree-2026-06-07`, HEAD `7adae31fb`.

---

## §0 — EVIDENCE (real git/file/gate output)

### §0.1 — The B1 deliverable exists
- `STORE-SCHEMA.md` — `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/STORE-SCHEMA.md` (14167 bytes, 2026-06-07 15:53). Status line in the doc itself: **`DRAFT — pending door:one-way founder sign-off at the B1 gate`**. (Note: STORE-SCHEMA.md lives in the linux audit dir, the write-permitted location — NOT in `/Users/jasonlee/Developer/source`.)

### §0.2 — The B2 stores exist and are UNTRACKED (as the task premise states)
```
git -C /Users/jasonlee/Developer/source status --porcelain registry/stores/
?? registry/stores/
git ls-files registry/stores/   → 0 files (UNTRACKED)
git check-ignore registry/stores/*  → none (NOT gitignored; deliberately held out of tracking)
```
Working tree is otherwise clean — `registry/stores/` is the ONLY untracked path. This is exactly the "stays UNTRACKED pending a founder schema-freeze before it enters firewall accounting" posture.

`registry/stores/` contents (4 files):
| file | bytes | shape |
|---|---|---|
| `design-store.json` | 201974 | envelope `{_store,_schema_ref,_status,…,entries[343]}` |
| `instructions-store.json` | 62839 | envelope `{store,schema_ref,status,…,entities[68]}` |
| `registry-store.json` | 358108 | envelope `{$schema_ref,store,…,closed_enums,entity_count:628,entities[628]}` |
| `canon-id-crosswalk.json` | 2791 | rename/disambiguation map; self-validates `all_present_in_design_store: true` |

### §0.3 — Structural-invariant validation (run READ-ONLY against the live store JSON)
All three stores were machine-validated against the closed enums + invariants in STORE-SCHEMA.md:

**design-store (343 entries):**
- `doc_axis` off-enum: **NONE** · `kind` off-enum: **NONE** · `lifecycle` off-enum: **NONE**
- excised-lifecycle (`superseded/deprecated/archived/retired`) present: **NONE** (invariant 2 holds)
- forbidden back-edges (`superseded_by/amends/amended_by`): **0** (invariant §1.5 holds structurally)
- dangling edges (`supersedes/depends_on/related` → non-existent key): **0**
- duplicate keys: **0**
- lifecycle dist: `accepted: 232`, `proposed-forward-plan: 111` (forward-plan KEEP rule §1.4 is materialized)
- **CAVEAT:** distribution is `doc_axis: DECISIONS` (343) / `kind: adr` (343) ONLY. PRD / spec_ms / spec_crate / implementation_plan / runbook / idea entity types are SPEC'd (§1.2) but NOT YET POPULATED. design-store today = the 343 surviving ADRs only.

**instructions-store (68 entities):** `scope`/`category`/`applies_when`/`authority` off-enum: **NONE**; dangling edges: **0**; dup keys: **0**.

**registry-store (628 entities, declared `entity_count: 628`):** `role`/`plane`/`api_stability`/`security_review`/`supply_chain`/`data_class` off-enum: **ALL CLEAN**; `application→app` normalization (OPEN-3) ALREADY APPLIED (no `application` role remains); dup keys: **0**. `api_stability=preview`(628), `security_review=unreviewed`(628), `supply_chain=source-only`(628) — uniform, matches the doc's "today only" note.

### §0.4 — ONE real cross-store integrity defect (scrub residue, not a schema defect)
registry-store `traceability.source_adrs` reference **2 design-store keys that do not exist** → **7 dangling cross-store edges**:
| missing key | refs | what it is in source | why it's gone from design-store |
|---|---|---|---|
| `ADR-0349` | 5 | `ADR-0349-jenkins-argocd-…` `status: Superseded` (→ ADR-0515) | EXCISED correctly (superseded + forbidden-vocab `jenkins`) |
| `ADR-0183` | 2 | `ADR-0183-policy-engine-…` `status: Superseded` `superseded_by:[ADR-0379]` | EXCISED correctly (superseded) |

Both predecessors were correctly EXCISED from design-store per invariant 2 (no tombstones). But the registry-store traceability still points at them instead of the survivors (ADR-0515 / ADR-0379). Per invariant 3 ("edges point only to CURRENT entities; inbound refs SCRUBBED"), these 7 refs must be re-pointed to the successors. **This is exactly the class of failure GATE-1 (cross-artifact-agreement) and the §4(d) entity-incremental gate are built to catch** — confirming the gate design is sound, but also that the corpus is NOT yet GATE-1-clean across stores.

(Also: 1 design-store entry — `D-DEAD-CONTEXT-ROUTING` — matched a forbidden-vocab regex. This is a FALSE POSITIVE: it is the meta-directive that DECLARES the words forbidden and carves out "Palantir Foundry"; it matched on the carve-out string itself, not a real violation.)

### §0.5 — Producer/gate accounting impact (the load-bearing finding)
- **No Rust gate or producer reads `registry/stores/` today** (`grep -rl registry/stores **/*.rs` → 0). The firewall producer (`cloud/cloud-ci/gates/accounting-registry-producer`) scans `docs/decisions/` + the catalog; it does NOT emit the three stores. There is **no `--face` that generates design-store / instructions-store / registry-store.**
- **Classification when committed:** the producer's `unit-class-policy.json` has a `registry/` PREFIX rule → `unit_class: "registry"` (an ACCOUNTED class). So committing `registry/stores/*.json` makes them TRACKED → producer assigns each an accounted `registry` row → **total-accounting requires each to have a resolvable `owner` + `justification_ref`.**
- **total-accounting is born-blocking:** its own doc-comment says *"`unowned` — row has no OWNERS-resolvable owner (born-blocking: 0 OWNERS exist today)."* No OWNERS mechanism exists in source yet. So today ANY newly-tracked accounted file is `unowned` RED. The firewall is RED-by-design until OWNERS land — this is intentional (it is the "born-blocking" floor), but it means **committing the stores naively trips `unowned`/`unjustified`/`unaccounted` until OWNERS + justification refs exist.**
- **registry-drift conflict:** registry-drift enforces `committed == regenerated` (a hand-edit is RED). But the producer does NOT regenerate these stores. So a hand-authored, accounted `registry`-class store file has no regenerated counterpart to byte-diff against. Committing them as hand-authored conflicts with the schema's own §8 generation model (stores authored-from-live-state, views generated, drift-gated) until a producer face emits/regenerates them.

---

## §1 — IS THE SCHEMA STABLE ENOUGH TO FREEZE?

**Yes, the data-structure contract is stable, mature, and evidence-grounded.** Strong signals:
1. **Closed-enum discipline is real and already PROVEN against the live data** (§0.3): every enum the doc closes (`doc_axis`, `kind`, `lifecycle`, `scope`, `category`, `applies_when`, `authority`, `role`, `plane`, `api_stability`, `security_review`, `supply_chain`, `data_class`) validates with ZERO off-enum values across 343 + 68 + 628 entities. The schema is not aspirational — the stores already conform to it.
2. **The no-tombstone structural enforcement works:** 0 excised-lifecycle entries, 0 forbidden back-edges, the 22 source `superseded_by` ADRs are excised. Invariant 2 is materialized, not just asserted.
3. **The forward-plan KEEP ruling (§1.4) is materialized:** 111 `proposed-forward-plan` entities coexist cleanly with 232 `accepted` — the founder's 2026-06-07 ruling is encoded.
4. **registry-store is essentially clean** (the OPEN-3 `application→app` normalization is already applied; all enums clean).
5. **The §6 open-decisions table is small and already mostly pre-resolved** (OPEN-2 resolved by founder ruling; OPEN-1 pre-resolved by D-SSOT-CURRENT-TRUTH).

**The schema is a SPEC that the data already satisfies.** That is the strongest possible freeze signal — you are not freezing a guess, you are freezing a contract the corpus has been built to.

---

## §2 — OPEN SCHEMA QUESTIONS (must be settled BY / WITHIN the freeze sign-off)

These are the four guard contracts + the §6 ratification table. The four guards (accessor / formatter / merge-driver / entity-incremental gate) are CONTRACTS in the doc, BUILT later in Workstream C — they do NOT block the schema freeze, but the schema must commit to their shape so C can build to it. Status:

| # | Open item (from STORE-SCHEMA.md §6 + the doc-as-data model) | Assessment | Resolve before freeze? |
|---|---|---|---|
| **OPEN-1** | No `superseded_by`/`amended_by`; model supersession as EXCISION. | Pre-resolved by D-SSOT-CURRENT-TRUTH; **VERIFIED materialized** in the data (0 back-edges, 22 excised). | **Confirm in sign-off** (1 line). |
| **OPEN-2** | Forward-plan KEEP vs DESTROY. | **RESOLVED** (founder ruled KEEP 2026-06-07); materialized (111 entities). | No — already ruled. |
| **OPEN-3** | registry `role` near-dupe collapse. | `application→app` **ALREADY APPLIED**; `api`/`rest`/`grpc` kept distinct per recommendation. | **Confirm the keep-distinct call** (1 line) — touches 903 catalog + gate baselines. |
| **OPEN-4** | Store file location/format `registry/stores/*.json`. | The stores ARE at that path; the `registry/` prefix rule already accounts that subtree. One-way (accessor/merge-driver/gate hard-code the path). | **Confirm** (1 line) — it's a one-way door. |
| **OPEN-5** | Doctrine says "two-store"; reality is THREE (design·instructions·registry). | Stale doctrine phrase. | **Doc-fix the "two-store" line in the SSOT** (B-lane edit) — should be queued, not a blocker. |
| **GUARD-(a) ACCESSOR** | `read <store> <id> [<section>]` — keyed/enum accessor (§5). | Contract defined; built in C. Schema must freeze the KEY scheme (§1.7) so accessor + crosswalk agree. crosswalk already self-validates against design-store. | Freeze the KEY scheme; build later. |
| **GUARD-(b) FORMATTER** | idempotent byte-identical canonical formatter; key-sorted; stable field order. | Contract defined; built in C. **Envelope-key INCONSISTENCY across the 3 stores** (`_store` vs `store` vs `$schema_ref`; `entries` vs `entities`) means the formatter spec must pick ONE canonical envelope or the formatter cannot be uniform. | **Resolve the envelope-key convention before freeze** (small, see §3 RECO-2). |
| **GUARD-(c) MERGE-DRIVER** | `.gitattributes` union-merge for `registry/stores/*.json`. | Contract defined; built in C. Depends on OPEN-4 path + the formatter. | Freeze path; build later. |
| **GUARD-(d) ENTITY-INCREMENTAL GATE** | validate only changed-key entities; closed-enum + no-dangling + no-forbidden-vocab + no-excised-lifecycle + forward-plan-not-mislabeled. | Contract defined; built in C. The §0.4 cross-store dangling defect is precisely what this gate (and GATE-1) must catch — it is currently RED on the live corpus. | Freeze the predicate set; build later. The corpus is NOT yet clean for it. |

**Net:** no LARGE open question remains. The remaining items are (a) three 1-line founder confirmations (OPEN-1/3/4), (b) one stale-doc fix (OPEN-5), (c) one schema decision the formatter needs (the envelope-key convention, GUARD-b), and (d) the C-lane guards which freeze-by-contract now and build later.

---

## §3 — WHAT COMMITTING `registry/stores/` INTO ACCOUNTING ENTAILS (producer/gate impact)

The transition is **untracked → tracked**, and that is a real accounting event because of the `registry/` classification rule:

1. **Each store file becomes an accounted `registry`-class row** (not husk/generated/ephemeral). The producer will demand `owner` + `justification_ref` per row.
2. **total-accounting will RED with `unowned`** for all 4 files until an OWNERS mechanism resolves an owner — because the gate is born-blocking (0 OWNERS exist today). This is the intended floor, but it means the commit is NOT gate-green on its own.
3. **registry-drift tension:** these files are hand-authored, but registry-drift expects `committed == regenerated`. Until a producer `--face` emits the three stores, there is no regenerated artifact to byte-diff. Either (a) the stores must be declared producer-GENERATED (and a face built to emit them — Workstream C), or (b) explicitly exempted from registry-drift like a spec/source-input, with a justification ref. This decision is part of the freeze.
4. **GATE-1 / entity-incremental gate would RED on the §0.4 defect:** the 7 dangling cross-store `source_adrs` (ADR-0349/ADR-0183) must be scrubbed to ADR-0515/ADR-0379 FIRST, or the very first accounted run is RED on cross-artifact-agreement.
5. **One-way doors fire:** OPEN-4 hard-codes `registry/stores/*.json` into the accessor, merge-driver `.gitattributes`, and the entity-incremental gate. Committing is the door:one-way moment the task is gating.

**Conclusion on impact:** committing is safe ONLY when (a) the 7-ref scrub is done, (b) an owner + justification_ref story for the 4 store files exists (or the born-blocking floor is explicitly accepted as the documented RED baseline), and (c) the regenerate-vs-exempt decision for registry-drift is made. None of these is large; all are prerequisites the founder should fold into the freeze declaration.

---

## §4 — RECOMMENDATION

**DO NOT freeze-blind. Declare the freeze WITH a 5-line rider that closes the residue in the same door:one-way sign-off. The schema itself is ready; the corpus has one scrub gap and the accounting-entry has prerequisites.**

Ordered prerequisites BEFORE the freeze declaration takes effect (all small, all in source — to be EXECUTED later by the mutation lane, not now):
- **RECO-1 (scrub):** re-point the 7 registry-store `traceability.source_adrs` from ADR-0349→ADR-0515 and ADR-0183→ADR-0379 (the excised predecessors' survivors). This makes the corpus GATE-1-clean. [BLOCKS a green accounted commit.]
- **RECO-2 (formatter precondition):** pick ONE canonical store envelope (unify `_store`/`store`/`$schema_ref` and `entries`/`entities`) so GUARD-(b) formatter can be uniform. Recommend `{store, schema_ref, status, authority, …, entities}` (drop the `_`/`$` prefixes; standardize on `entities`). [BLOCKS the formatter; cheap to do.]
- **RECO-3 (accounting story):** decide registry-drift disposition for the 4 store files — GENERATED-by-future-producer-face (preferred, matches §8) vs spec-like-exempt-with-justification. Document the chosen owner + justification_ref so the first accounted run is explainable (even if `unowned` stays RED under the born-blocking floor). [Required to interpret the first gate run.]
- **RECO-4 (1-line confirmations):** founder confirms OPEN-1 (no-tombstone), OPEN-3 (keep `api`/`rest`/`grpc` distinct), OPEN-4 (path is one-way).
- **RECO-5 (doc-fix):** correct the stale "two-store" phrase → "three-store" in the SSOT (OPEN-5), queued for the B-lane.

If the founder accepts the born-blocking `unowned` RED as the DOCUMENTED accounting baseline (consistent with the existing `gate-baseline.signoff.json` door), then RECO-1 + RECO-2 + RECO-3 are the true blockers and RECO-4/-5 are confirmations.

**Verdict: NEEDS-PREREQ.** Flip to READY the moment RECO-1 (scrub) + RECO-2 (envelope) land and RECO-3 (drift disposition) is decided. The schema does not need redesign — it needs the founder to declare the freeze and the corpus to get its last scrub.

---

## §5 — FOUNDER ACTION

**Declare the B1/B2 schema-freeze (door:one-way) by signing off STORE-SCHEMA.md** — i.e. change its status line from `DRAFT — pending door:one-way founder sign-off at the B1 gate` to FROZEN/RATIFIED — **with the §4 rider**: confirm OPEN-1/-3/-4, order RECO-1 (scrub the 7 ADR-0349/ADR-0183 refs → ADR-0515/ADR-0379), RECO-2 (canonical envelope), and RECO-3 (registry-drift disposition for the store files), and queue RECO-5 (two-store→three-store doc-fix). Only AFTER that declaration may a later mutation lane commit `registry/stores/` into firewall accounting. This doc is PREP ONLY and pulls no trigger.
