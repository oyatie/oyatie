# 10 — JUSTIFICATION REGISTER: decision corpus re-judged under the FOUNDER D-DOCTRINE charter

> **Lane:** JUSTIFY + REEVALUATE. Read-only. Extends (does not redo) `synthesis/01-ADR-DISPOSITION-TABLE.md` + `synthesis/decision-record-oyatie-canon.md` + `backlog-reconciliation/00-BACKLOG-RECONCILIATION.md` with the charter lens: (a) hyperscaler monorepo doctrine, (b) Linus taste, (c) arch invariants, (d) robust-not-false, (e) total accounting.
> **Method:** prior audit gives per-ADR disposition; THIS register surfaces what the prior audit MISSED or under-weighted once each Accepted decision is judged against the charter — unjustified / contradicting / over-abstracted-or-special-cased / arch-invariant-violating / false-completion.
> **Scope covered:** all 345 distinct source ids are inherited from the verified disposition table; this register adds CHARTER-LENS re-judgements on the **51 ADRs** below where the charter lens changes or sharpens the prior verdict, organized by the priority clusters. The **~294 remaining Accepted/Proposed ADRs** whose prior KEEP/AMEND/ARCHIVE verdict is unchanged under the charter lens are **explicitly DEFERRED to the disposition table** (counted in §Coverage) — not re-typed here, to avoid redoing settled work.
> **Status legend (verdict):** KEEP · AMEND · ARCHIVE · DROP. **charter-lens issue** names the specific charter clause; **action** is the concrete amend/archive op (additive / amend-in-place per D13).
> All evidence below was verified live against `~/Developer/source/docs/decisions/` on 2026-06-06 (paths + line numbers cited).

---

## §0 — HIGHEST-SEVERITY VERDICTS (the charter's robust-not-false + total-accounting failures, live-confirmed)

These are the findings a founder must rule on first — each is a CONFIRMED charter violation with file+line evidence, several SHARPER than the prior audit logged.

### S1 — ADR-0363 asserts a FALSE completion in its own body (robust-not-false FAIL) — **AMEND (mandatory)**
- **Evidence:** `ADR-0363-…md:35` verbatim: *"**The Foundry name was eradicated** … `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell."* The SAME sentence both claims eradication AND admits a 597-file foundry shell was kept — a self-contradicting false-completion **inside one line**. Status = `Accepted`.
- **Charter lens (d) robust-not-false + (e) total-accounting:** an Accepted ADR claiming "eradicated" while admitting retained residue is the canonical false-claim the D-DOCTRINE charter forbids; it is exactly the "ADR-0363 claims Foundry eradicated yet residue persists" exhibit, now verified at the line.
- **Verified residue (canonical tree, worktrees excluded; snapshot 2026-06-06):** 11 foundry-named dirs then survived — `docs/foundry`, `docs/products/foundry`, `docs/runbooks/foundry`, `docs/teams/axis-foundry` (a LIVE team name), `contracts/openapi/foundry`, `templates/foundry-supervisor` (**since deleted** — hooks pointed at missing `tools/foundry-supervisor-*`), `evidence/onprem-foundry-readiness`, `oya/intelligence/_legacy-foundry`, + 3 `.omc/plans` milestones. (Note: the 2180/3771-style raw counts are dominated by transient `.claude/worktrees/` copies — 2169 of 2180 — which are NOT canonical; the honest canonical residue at audit time was the 11 above + 4110 file-level mentions corpus-wide. Stating this so the claim is not overstated.)
- **Action:** rewrite 0363:35 to drop "eradicated"; state "rename is IN PROGRESS — N canonical foundry artifacts remain (enumerated), tracked to completion under the D11 foundry-rename + D-INTEL sense-routing (platform→oya-intelligence / fitness→oya-governance / vcs→retired)." Wire a `no-foundry-token` fitness gate proven by RED/GREEN so the claim becomes machine-true, not prose.

### S2 — 0511↔0513 dual-destination contradiction, supersession missing BOTH ends (Linus stable-contract FAIL + total-accounting FAIL) — **AMEND (mandatory, both files)**
- **Evidence:** `ADR-0511:3` title = *"CI orchestration = **Argo Workflows** … supersede ADR-0359"*; `0511:19-20` founder-basis 2026-05-29 names **Argo Workflows as "the destination CI orchestrator."** `0511` front-matter: `status: Proposed`, `superseded_by: []`. `ADR-0513` (dated 2026-05-30, `status: Accepted`) makes **bespoke-Rust-Prow oya-ci the destination**, and its `relates:` list (`0380,0111,0116,0374,0363,0392`) **does NOT contain 0511** — no `supersedes`/`superseded_by` keys exist at all.
- **Charter lens (b) stable contracts + (e) accounting:** TWO ADRs each declare themselves the canonical CI-orchestration destination (Argo vs bespoke-Prow), one day apart, with **zero supersession edge between them on either end.** This is the precise "0511→0513 supersession missing both ends" exhibit — confirmed. A reader cannot tell which governs.
- **Cross-check:** `0359` correctly has `superseded_by: [ADR-0511]`, but `0511` does NOT carry the reciprocal nor its own onward supersession to 0513 → a broken 2-hop chain `0359→0511→(gap)→0513`.
- **Action (per founder D-CICD + backlog T-CI ruling = bespoke-Prow-only):** set `0511 status: Superseded`, `superseded_by: [ADR-0513]`; add reciprocal `supersedes: [ADR-0511]` + add 0511 to `0513.relates`. Consolidate 0349/0359/0361/0408/0511/0513/0514 → ONE canonical CI ADR (destination 0513); DROP 0349+0361 (Proposed, never-ratified Jenkins debt); AMEND-in-place 0408/0514 (adopted Buck2 substrate). Tekton is named NOWHERE with an Accepted ADR — strike it from any "four faces" framing.

### S3 — dup-0377 (two ADRs, one id) + non-enum free-text status (total-accounting FAIL + status-enum drift) — **AMEND (renumber one)**
- **Evidence:** `ADR-0377-kafka-to-pulsar-via-kop.md` (`id: ADR-0377`, `status: Accepted`, `supersedes: [ADR-0005]`) AND `ADR-0377-forgejo-board-git-ref-cas-fallback.md` (`id: ADR-0377`, `status: Proposed (conditional: Accepted only after ADR-0377-D2 and ADR-0377-D3 code/tests pass)`). Two distinct decisions collide on id 0377; the forge variant's status is **free-text, not a member of the status enum.**
- **Charter lens (e) accounting + (b) good-data-structures:** id is the primary key; a duplicated key + a non-enum status value break every generated index and the no-dangling-ref invariant (D13).
- **Action:** RENUMBER the forge variant into the free block >0514 (per D13 additive); normalize its status to enum `Proposed` and move the conditional into the body. This is the "dup-0377" exhibit — confirmed live.

### S4 — phantom `ADR-0150-cedar-policy-engine.md` cited BY FILENAME across the corpus while 0150 is cursor-pagination (total-accounting / dangling-ref FAIL) — **AMEND (assign real id)**
- **Evidence:** on disk `ADR-0150-cursor-pagination-canonical.md` is the ONLY 0150. Yet `ADR-0243:17,28,101` cites `ADR-0150-cedar-policy-engine.md` (a file that does not exist) as the Cedar-policy-engine anchor; ≥10 ADRs reference 0150 (0297/0341/0348/0255/0294/0337/0313/0251/0292/0250…). The Cedar policy-engine decision has **NO real ADR id** — it is a phantom.
- **Charter lens (e) accounting + (b) stable contracts:** a load-bearing keystone decision (the Cedar engine, D6) is anchored to a non-existent file-id cited by name — the worst dangling-ref class because it looks resolvable.
- **Action:** assign the Cedar-engine decision a REAL id in the free block (per D-EXEC A.0-2 "phantom-0150 cedar real-id assigned at Wave-1 L1.0 MAP"); repoint all `ADR-0150-cedar-policy-engine.md` citations to it; leave 0150-cursor-pagination untouched. Sharper than the prior table (which logged "re-key the map" but did not flag the **by-name filename citation** in 0243).

### S5 — ADR-0335 carries TWO status lines (status-enum integrity FAIL) — **AMEND**
- **Evidence:** `ADR-0335-…md:3` `status: Accepted` AND `:771` `status: completed-locally`. Two status declarations in one ADR; `completed-locally` is not an enum member.
- **Charter lens (d) robust-not-false + (e) accounting:** a generated status projection (D1/D365) reading this file gets an ambiguous/invalid status → the masterplan generator silently picks one. This is a latent false-green.
- **Action:** remove the `:771` stray; if it records a real sub-state, move it to a body field, not `status:`. (Prior table marked 0335 "KEEP top-tier governing" and MISSED the second status line.)

### S6 — `axes_count` drift 6≠7 live in the machine-readable SSOT (generated-not-hand-maintained FAIL) — **AMEND + GATE**
- **Evidence:** `docs/machine-readable/catalog.json:12` `"axes_count": 6` vs `docs/machine-readable/contracts.json:9` `"axes_count": 7`. The 7-product-axis model (D-DOCORG) is contradicted by a stale `6` in catalog.json.
- **Charter lens (a) generated-not-hand-maintained + (d) robust:** two hand-maintained machine-readable files disagree on a count that should be GENERATED from one source — the exact D-DOCTRINE "catalog axes_count:6 stale vs 7" exhibit, confirmed.
- **Action:** generate `axes_count` from the single axis registry; add a cross-artifact-agreement gate (backlog #11 / D1) proven by RED/GREEN. Do not hand-fix one number — that re-creates the drift.

---

## §1 — CI/CD CLUSTER (D-CICD; charter priority) — re-judged

| id | verdict | charter-lens issue | action |
|---|---|---|---|
| 0349 | **DROP** | (e) accounting + (a) one-version: Jenkins+ArgoCD substrate, Proposed, never ratified; competes with 0513 destination. Jenkins-half is dead debt. | DROP per D-EXEC A.0-2; resolve the `registry/foundation-bypasses/byp_adr_0349` bypass record so no orphan gate references it. |
| 0359 | **ARCHIVE** | Jenkins-replaces-GHA; already `Superseded`. Edge `superseded_by:[0511]` is correct but 0511 itself is now Superseded → chain must re-point. | Keep tombstone; ensure the kill-GHA principle survives in the 0513 canonical ADR; re-point chain 0359→0513. |
| 0361 | **DROP** | Jenkins-native revamp, Proposed; Kyverno→Kubewarden stale; pure Jenkins debt. | DROP per D-EXEC; salvage the OSI-strict supply-chain-shift-left language into 0513/0039. |
| 0408 | **AMEND→KEEP** | (c) parallelizable-builds: Buck2-driven CI engine is correct + hyperscaler-aligned; only the orchestrator clause is Jenkins-stale. Standalone adopted substrate (A.0-2). | AMEND-in-place: orchestrator clause off Jenkins/0359 → oya-ci/0513; keep Buck2-RBE/cquery-rdeps engine. NOT archived. |
| 0511 | **AMEND (Superseded)** | **S2** — dual-destination contradiction vs 0513; supersession missing both ends. | Per S2: `Superseded`/`superseded_by:[0513]`; Argo confined to CD-only (ArgoCD/Rollouts bridges per D10); Argo-Workflows rejected for CI. |
| 0513 | **KEEP** (amend metadata) | (b) stable-contract: the ONE Accepted CI destination (bespoke-Rust-Prow); founder-locked. But missing supersedes keys → cannot prove canon mechanically. | KEEP; add `supersedes:[ADR-0511,ADR-0359?]` + reciprocal relates; fold 0111/0116; reconcile 0380 back-edge. |
| 0514 | **AMEND→KEEP** | (e) accounting: depends on unwritten `ADR-0488` (linker dep, absent) + uses retired `microservices/` path. | AMEND-in-place: author the 0488 linker dep OR inline it; rewrite `microservices/`→`{oya,cloud}/`. |
| 0360 | **AMEND→KEEP** | (c) affected-targets/parallelizable: 7 optimizations correct; O1/O3 must bind Buck2-RBE, O6→Tide. | Rebind tool refs; RATIFY. |
| 0366 | **KEEP** | (c) one-lane-one-path: single-owner-agent-per-service on disjoint paths = charter-aligned (matches D-CONFORM). | KEEP. |
| 0367 | **KEEP** | (d) robust-not-false: trustless re-execution (producer never self-certifies) IS the anti-false-green primitive the charter demands. | KEEP — promote as the reference enforcement pattern. |
| 0374/0387 | **AMEND (FOUNDER-CALL)** | (b) contracts: two CI-webhook-gateway ADRs with conflicting sinks (Forgejo vs GitHub commit-status) + dead 0112 cite. Forge fault-line. | Pick one sink per D2 (GitHub-now→bespoke-later); supersede the loser; scrub Jenkins-orchestrator → 0513. |
| 0380 | **AMEND** | (d): "Jenkins farm on Talos" is a phase-superseded bridge presented without the cutover edge. | Add 0513 back-edge; mark transitory build-first-cutover-later. |
| 0392 | **KEEP** | (c) parallelizable-builds + (a) one-version: Buck2 canonical graph = hyperscaler-correct; supersedes 0358 §2. | KEEP (governing). |

**CI-cluster net:** DROP 3 (0349/0361 + 0359-tombstone), AMEND-metadata 2 (0511 Superseded, 0513 canon), AMEND-in-place 2 (0408/0514). Consolidate to ONE ADR (0513). This MATCHES founder D-CICD + backlog T-CI; the charter adds the **machine-proof requirement** (no "consolidated" claim until the supersession graph is acyclic + complete, gate-verified).

---

## §2 — STRUCTURE CLUSTER (0131/0132/0512/0509/0357) — re-judged

| id | verdict | charter-lens issue | action |
|---|---|---|---|
| 0131 | **AMEND** | (a) one-version + (c) min-blast-radius: per-service flat layout is correct, BUT references `microservices/` **15×** while D-PURESPLIT rules exactly `oya/`+`cloud/` — an Accepted structure ADR contradicting the founder pure-split. | AMEND-in-place: rewrite all `microservices/` examples → `{oya,cloud}/<service>/`; state the two-tree-only rule; bind 0512. |
| 0132 | **KEEP** (amend) | (b) no-special-cases: no-grouping flat-catalog is Linus-clean; drop grandfather language. | KEEP; drop grandfather carve-out (a special-case). |
| 0512 | **KEEP** (amend) | (c) one-workspace + min-coupling: canonical monorepo (crate=BC, Buck2 per-crate graph) is the charter's structural backbone, BUT still references `microservices/` **6×**. | KEEP-governing; AMEND the `microservices/` references to the pure-split `{oya,cloud}/` per D-PURESPLIT (supersedes 0357/0509, amends 0131). |
| 0509 | **KEEP** | (c) min-blast-radius: single-crate decomposition (121→13 collapse) is hyperscaler-aligned. | KEEP. |
| 0357 | **KEEP→note** | superseded-by-0512 trajectory; 546-vs-734 crate-count drift is a stale data point. | KEEP as history; mark superseded_by 0512; the count drift = a generated-index target, not hand-fix. |

**Structure net:** the charter surfaces a MISS the prior table soft-pedaled — **0131 AND 0512 (both Accepted) still encode the retired `microservices/` 3rd-tree** while D-PURESPLIT (founder, one-way) mandates exactly two trees. This is an arch-invariant contradiction (min-shared-blast-radius / pure-split), not just "drop examples." It must be amended in BOTH before any structural gate can be honest.

---

## §3 — ENFORCEMENT / GOVERNANCE CLUSTER (0365/0363/0247/0335/0123/0135/0368/0109) — re-judged

| id | verdict | charter-lens issue | action |
|---|---|---|---|
| 0363 | **AMEND (mandatory)** | **S1** false "eradicated" claim + forge fault-line. | Per S1: drop "eradicated," enumerate residue, wire a proven `no-foundry-token` gate. |
| 0335 | **AMEND** | **S5** dual status line (`Accepted`+`completed-locally`). Correctly supersedes 0136/0247/0239 (verified `:15-49`) — that half is sound. | Per S5: remove stray `:771` status. Keep the supersession (the 0136/0247 dissolution tension IS properly resolved here — no action there). |
| 0247 | **KEEP/RATIFY** | (e): self-modification doctrine; `:94` cleanly states 0136/0239 DISSOLVE. Resolved by 0335 supersession. | RATIFY; foundry/retired external agent harness vocab scrub; self-mod ceiling = door:one-way. |
| 0365 | **KEEP** | (a) generated-not-hand-maintained: automated ADR lifecycle = the charter's drift-proof backbone; `status: Accepted`. | KEEP; ensure the lifecycle gate is RED/GREEN-proven (not advisory) — robust-not-false. |
| 0123 | **KEEP** | (d) robust-not-false: maturity-claim-gate forbids "hyperscaler mature" without evidence — IS the charter's no-false-promise primitive. | KEEP; verify the gate actually blocks (RED fixture). |
| 0135 | **KEEP** | (d): aspirational-enforcement gate (fail-closed on claims naming non-existent surfaces) = the meta-gate the whole charter relies on. | KEEP — promote as the keystone robust-not-false enforcer; PROVE it blocks. |
| 0368 | **AMEND** | **(b) Linus-taste FAIL:** `:20` "the fleet is kept at **maximum safe concurrency at all times**" — the idle=defect framing is over-abstracted org-doctrine masquerading as architecture; distrust-over-abstraction + no-special-cases. | AMEND: reframe "max concurrency at all times" → "capacity-bounded, M0-gated parallelism" (matches D8: builds parallelize, NOT one-at-a-time, NOT all-at-once). Drop the idle-as-defect value judgment. |
| 0109 | **AMEND** | **(b) Linus special-case FAIL:** `:114` "Both patterns are canonical" + the Pattern-B carve-out the ADR ITSELF flags as tension with `no-exceptions-canonical` (`:38`). One generic kernel + a named "Pattern-B exception" = the special-case Linus rejects. | AMEND: collapse to ONE canonical lifecycle-kernel shape parameterized by config; if sunset-lifecycle (0108) genuinely needs a dedicated kernel, justify it as an INSTANCE of the one shape, not a second canonical pattern. Resolve the §Decision-6 vs §Migration-policy self-contradiction. |

**Enforcement net:** the charter confirms the prior "22 oya-governance-* crates unwired" fear is REAL but the count is conservative — **59 `oya-governance-*` crate dirs + 70 fitness-lane docs** exist in the canonical tree, and 59 appear in BUCK files; the gap is whether those BUCK targets are in the **required gate roster** vs merely defined. The `diataxis-doc-class` and `prd-axis-coverage` lanes resolve only inside `.claude/worktrees/` (transient) — **no canonical-tree active-blocking wiring found** → confirmed "defined-not-active." Action: a single gate-roster manifest (generated) that lists every required lane + a RED/GREEN proof each one blocks; anything defined-not-rostered auto-flags (total-accounting).

---

## §4 — IDENTITY / POLICY / DATA / EVENT CLUSTERS (charter spot-re-judgements)

| id | verdict | charter-lens issue | action |
|---|---|---|---|
| 0476 | **AMEND** | (b) stable-contract: oya-identity correct endpoint, BUT `supersedes:[ADR-0421]` (absent on this branch) + does NOT yet carry `supersedes:[0187]`; Cedar mis-cited as 0083 (error-handling ADR, not Cedar). | Add `supersedes:[0187]`; fix phantom 0421 (branch-locality, re-resolve at merge); fix Cedar cite 0083→real Cedar id (the S4 phantom-0150 target). |
| 0187 | **AMEND** | (a) own-endpoint/vendor-bridge: Zitadel must demote canonical→bridge, `superseded_by:[0476]` — edge missing today. | Set superseded-as-endpoint/bridge-retained; resolve C-4. |
| 0243/0246 | **AMEND** | **S4** — both anchor Cedar to phantom `ADR-0150-cedar-policy-engine.md`. | Repoint to the real Cedar-engine id; preserve Cedar-as-permanent-contract (D6) / own-PARC-engine. |
| 0006 | **AMEND** | (e) accounting + (b): self-referential rename "Ontology→Ontology" tautology (×2) — a no-op edge that pollutes the rename graph. | Fix the tautology; repoint vector tier→Milvus 0192. |
| 0005 | **ARCHIVE** | Kafka backbone retired-in-fact; 0377-kafka supersedes. Patterns (outbox/CloudEvents) survive. | ARCHIVE broker; ensure outbox 0153 + 0377 carry the patterns. |
| 0457/0429/0443/0428 refs (via 0478/0479/0480/0481) | **AMEND (branch-locality)** | (b): bespoke-billing/meter/cost/flags supersede predecessors that live on `origin/dev`; supersede-edges dangle on this branch + Cedar mis-cited 0083 across all four. | Re-resolve supersede-edges at merge; fix the four Cedar 0083 mis-cites → real Cedar id. |
| 0045 | **AMEND** | (e): claims "Citus=AGPL" (factual error) + stale OLAP/TS/pool/vector repoints. | Fix the Citus license claim; repoint to 0193/0194/0179/0192. |

---

## §5 — WHAT THE PRIOR AUDIT MISSED (charter-lens deltas, summarized)

1. **0363 false-completion is in ONE self-contradicting line** (`:35`), not just "stale glossary" — prior table marked it TRUE-core/PARTIAL; charter elevates to mandatory-AMEND robust-not-false failure.
2. **0511↔0513 is a live DUAL-DESTINATION contradiction** (Argo vs bespoke-Prow), not merely "reconcile" — both claim canon, neither supersedes; verified via `0511:3,19` vs `0513.relates` omission.
3. **phantom-0150 is cited BY FILENAME** (`ADR-0150-cedar-policy-engine.md` in `0243:17,28,101`) — prior table said "re-key the map"; charter shows the dangling-ref is worse (looks resolvable, governs D6 Cedar keystone).
4. **0335 has TWO status lines** — prior table KEEP'd it without catching `:771 status: completed-locally`.
5. **0131 AND 0512 (Accepted) still encode `microservices/` 3rd-tree** — contradicts D-PURESPLIT two-tree invariant; prior table softened to "drop examples."
6. **0368 idle=defect / "max concurrency at all times"** — Linus over-abstraction the prior table marked only "questionable (framing)"; charter says AMEND to capacity-bounded.
7. **0109 "Both patterns are canonical"** — a self-admitted special-case (Pattern-B) violating no-exceptions-canonical; prior table said "resolve self-contradiction" without naming the Linus-taste violation.
8. **axes_count 6≠7 live in two machine-readable files** — confirmed generated-not-hand-maintained failure; must be GATED, not hand-fixed.
9. **dup-0377 forge variant has a non-enum free-text status** — prior table flagged the dup but not the status-enum violation.
10. **Foundry residue must be stated honestly:** 11 canonical dirs (not the 2180 worktree-inflated count) — a charter total-accounting correction that PREVENTS over-claiming while still proving S1.

---

## §COVERAGE / HONESTY

- **Re-judged under charter lens here:** 51 ADRs (CI cluster 12, structure 5, enforcement 8, identity/policy/data/event 9, + the 17 cross-referenced inside §0/§5 highest-severity items). Each carries a file+line evidence cite verified live 2026-06-06.
- **Inherited unchanged from the verified disposition table (DEFERRED, not re-typed):** the remaining ~294 of 345 source ids whose prior KEEP/AMEND/ARCHIVE verdict the charter lens does NOT change. Their dispositions stand in `synthesis/01-ADR-DISPOSITION-TABLE.md`. This is a deliberate bound to avoid redoing settled work, per the lane instruction to EXTEND not redo.
- **LINUX side (L-0001..L-0026):** NOT re-judged here — they renumber to 0515+ on merge and the prior table's verdicts are charter-consistent (own-when-proven ratchet = D-META). One residual: L-0001's half-applied Postgres "eliminate→retain" scrub (line 36) is a robust-not-false micro-failure → AMEND (already in table).
- **NOT COVERED (explicit, no silent caps):** (a) I did not open every one of the 345 ADR bodies — for the 294 deferred ADRs I relied on the prior verified table + targeted greps, NOT full reads; a body-level charter pass on those could surface more Linus-taste/over-abstraction cases (e.g., breadth ADRs 0027/0249/0314/0315 flagged "questionable-breadth" in the table were not re-litigated here — they are founder-scope per D9, not charter-fail). (b) I did not verify the `oya-governance-*` BUCK targets are actually in the REQUIRED gate roster (only that 59 appear in BUCK files) — the defined-vs-rostered distinction needs the gate-roster manifest to confirm; flagged as the §3 action. (c) Branch-locality: 0421/0457/0429/0443/0428/0488 are absent on this branch (live on origin/dev) — their supersede-edges can only be verified at merge.
- **Status distribution (corpus-wide, verified):** 135 Accepted · 99 Proposed · 14 Superseded (of 348 .md files / 345 distinct ids).

---

## RETURN DIGEST

**Highest-severity verdicts (all CONFIRMED live, file+line):**
1. **S1 — ADR-0363:35 false "eradicated" claim** (robust-not-false) — self-contradicting in one line; AMEND-mandatory + wire proven no-foundry gate.
2. **S2 — 0511↔0513 dual CI-destination, supersession missing both ends** (0511:3,19 Argo-destination vs 0513 bespoke-Prow; 0513.relates omits 0511) — AMEND both: 0511→Superseded-by-0513.
3. **S3 — dup-0377** (two files share id ADR-0377; forge variant non-enum status) — renumber forge variant >0514.
4. **S4 — phantom `ADR-0150-cedar-policy-engine.md` cited by filename in 0243:17,28,101** while 0150 is cursor-pagination — assign Cedar-engine a real id, repoint.
5. **S5 — ADR-0335 two status lines** (:3 Accepted / :771 completed-locally) — remove stray.
6. **S6 — axes_count 6≠7** (catalog.json:12 vs contracts.json:9) — generate + gate, don't hand-fix.
7. **Structure — 0131 (×15) AND 0512 (×6) still encode retired `microservices/` 3rd-tree** vs D-PURESPLIT two-tree invariant — AMEND both.
8. **Linus-taste — 0368 "max concurrency at all times" + 0109 "both patterns canonical"** — AMEND (capacity-bounded; one canonical kernel).

**Coverage:** 51 ADRs re-judged under charter lens (CI 12 / structure 5 / enforcement 8 / identity-policy-data-event 9 / cross-ref-in-§0 17). **Deferred (verdict unchanged, stand on prior table):** ~294 of 345 source ids. **Not covered (no silent cap):** body-level charter pass on the 294 deferred ADRs; gate-roster-membership of the 59 oya-governance crates; branch-local supersede-edges (0421/0457/0429/0443/0428/0488 on origin/dev). LINUX L-0001..0026 deferred to table (charter-consistent).

**Artifact:** `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/justify-account-robustness/10-justify-decisions.md`
