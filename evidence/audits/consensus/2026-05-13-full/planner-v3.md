# Full-project Ralplan Planner v3 (2026-05-13)

**Revision of v2 absorbing critic r1's 5 narrow fixes.** Critic r1 Torvalds-lens → ITERATE; recommendation: "After those edits, APPROVE is likely."

Architect r2 (hyperscaler lens) APPROVED v2. Critic r1 (Torvalds lens) ITERATED v2 with 5 narrow concrete fixes. v3 below closes them.

## §1 Current state inventory (unchanged from v1/v2; see planner-v1.md §1)

## §2 Accumulated user directives (unchanged; 25 chronologically in v1 §2)

## §3 Viable Options (NEW per critic r1 fix #2 — fair alternatives scored)

Critic r1 finding #2: v2 imported alternatives by reference. v3 restates + scores 4 alternatives against 5 planner principles + 4 user-mandated rules.

| Option | Description | Principles satisfied | User rules | Decision |
|---|---|---|---|---|
| **α (amended)** | Continue meta-layer direction WITH 10 architect amendments + freeze-net-new-classes narrowing. Next slice = VL. | P1 (becomes operational), P2 (sound), P3 (becomes enforceable), P4 (consumer-led migration), P5 (single grit path) — ALL satisfied AT END OF VL | (i) honest-claims ✓ (ii) Linus-grade ✓ (iii) verified-claims ✓ post-tracking-fix (iv) honest-introspection ✓ | **ADOPTED** |
| β (pivot to operational) | Stop meta-layer; wire validator + lane + active; restart implementation per masterplan. | P1 partial (validator without contract); P5 (same as α); others FAIL (no contract, no graph, no DRY, no retirement) | (i) ✗ (claims operational without contract spec); (iii) ✓ | rejected: throws away 14 artifacts + 1 ADR + 1 crate; α-with-amendment IS β functionally |
| γ (freeze-only, no VL) | Freeze all work; user decides direction. | P1-P5 all unsatisfied | (iv) ✓ honest about stall | rejected: user explicitly said "iterate until approved" |
| δ (rollback/defer) | Revert today's 7 commits; defer architecture work. | All FAIL | (iv) ✓ | rejected: erases 9000 lines of validated work; user has not requested rollback |

**Decision: Option α (amended)** — same as direction-consensus-v1 + 10 architect r1 amendments + 5 critic r1 fixes.

## §4 Standards in force (unchanged from v2 §3)

## §5 VL acceptance criteria (CRITIC R1 FIX #3 — executable per-step rows)

Critic r1 finding #3: v2 steps 3-6 were "mush". v3 provides Command / Expected failure / Expected success / Artifact path for each step.

| Step | Action | Command | Expected failure | Expected success | Artifact path |
|---|---|---|---|---|---|
| 1 | Wire oya-dev-cli subcommand | `cargo run -p oya-dev-cli -- gate validate active-artifact-contract --help` | exit 2 (no subcommand registered) | exit 0 + help text shows the subcommand | `crates/oya-dev-cli/src/commands/gate.rs` |
| 2 | Failing fixture for missing-row artifact | `cargo run -p oya-dev-cli -- gate validate active-artifact-contract --registry tests/fixtures/missing-row-registry.json` | exit 0 (false negative) | exit 1 + violation `R03-missing-capability` or `R01-artifact-path-not-in-head` printed | `crates/oya-dev-cli/tests/fixtures/missing-row-registry.json` |
| 3 | Flip `lean-a-active-artifact-contract` to active | `grep "lean-a-active-artifact-contract" registry/quality/lanes.yaml \| grep "status: active"` | grep returns empty | grep returns the line | `registry/quality/lanes.yaml` |
| 4 | Grit pre-done validation OR bounded ICM fallback | `scripts/hooks/grit-pre-done-validate-artifact-contract.sh` (or scaffold-lock with `expires_at` field within 24h) | exit non-zero on violation | exit 0 + scaffold-lock includes `expires_at: 2026-05-14T00:00:00Z` (24h max) | `scripts/hooks/grit-pre-done-validate-artifact-contract.sh` |
| 5 | Emit evidence/status bundle | `cargo run -p oya-dev-cli -- gate validate active-artifact-contract --emit-evidence /evidence/lane-run-${RUN_ID}.json` | no file written or schema-invalid | file written; contains `outcome`, `validation_duration_ms`, `head_commit_sha`, `green_ci_run_url` | `/evidence/lane-run-${RUN_ID}.json` |
| 6 | ONE tracked graph artifact + ONE checker assertion (critic r1 fix #5 narrowed) | `cargo run -p oya-dev-cli -- gate validate active-artifact-contract --emit-graph-edges /registry/graph/active-artifact-contract-edges.json && cargo run -p oya-check-active-artifact-contract --test graph_edge_emission` | no graph file OR test fails | graph file present with `[{source: artifact_id, target: capability_id, edge_type: declares}]`; test passes | `/registry/graph/active-artifact-contract-edges.json` |
| 7 | Gate before resuming migrations | manual review of steps 1-6 all green | any step red | all 6 green; commit message references `VL-OPERATIONAL` keyword | (no artifact; gate event) |

## §6 Architectural amendments — narrowed per critic r1 (fixes #4 + #5 + #6)

### Amendment 1 — Resource-controller pattern (unchanged from v2)

### Amendment 2 — VL is first controller (unchanged)

### Amendment 3 — Registry sharding (unchanged)

### Amendment 4 — Graph materialization layer (CRITIC R1 FIX #5 — narrowed)

**v3 narrowing:** VL emits exactly ONE graph artifact (`/registry/graph/active-artifact-contract-edges.json`) + ONE checker assertion (test `graph_edge_emission`). The full materialization layer (nodes/edges/reverse_indexes/owners/freshness/impact_queries) is a SEPARATE post-VL slice. v3 does NOT claim full materialization in VL.

### Amendment 5 — spec/status separation (unchanged)

### Amendment 6 — Admission severity (unchanged)

### Amendment 7 — DRY enforceability (unchanged)

### Amendment 8 — Markdown retirement consumer-led (unchanged)

### Amendment 9 — ICM fallback NARROWED (CRITIC R1 FIX #4)

**v3 narrowing:** ICM scaffold-lock fallback is:
- **One expiring record per claim** with required fields: `agent_id`, `intent`, `paths`, `symbols`, `created_at`, `expires_at` (24h after `created_at`), `validation_command`, `owner`
- **Hard fail at expiry** — lane `lean-a-grit-fallback-stale` flags records past `expires_at`
- **No VCS-replacement decision inside VL** — that's a separate Decision Record `ADR-0070-oyatie-native-vcs-decision` (planned post-VL; not gated by VL completion)

### Amendment 10 — Control-plane scale SLOs (CRITIC R1 FIX #6 — measurement hooks added)

**v3 measurement hooks:**
- `oya-dev-cli gate validate active-artifact-contract` emits `validation_duration_ms` field into evidence bundle (step 5 above)
- `oya-gen-graph-materialize` (post-VL) emits `graph_build_duration_ms`
- Stale-state window tracked via evidence-bundle `emitted_at` vs previous run timestamp
- Lane `lean-a-control-plane-slo` (post-VL) compares emissions vs SLOs from `/specs/control-plane-slos.json`

## §7 Scale-failure-mode coverage (unchanged from v2 §7)

## §8 Honest gaps (refined per critic r1)

Existing v2 gaps + 2 NEW from critic r1:

11. **Consensus artifacts not HEAD-tracked** (critic r1 finding #1; architect r2 dismissed; critic upgraded to blocking). FIX: this commit batch tracks all `/evidence/audits/consensus/2026-05-13-full/*` + `/evidence/audits/consensus/2026-05-13-direction/*` artifacts.
12. **VL step-3-to-6 ambiguity** (critic r1 finding #3) — FIX: v3 §5 replaces mush with executable per-step rows.

## §9 Standardization audit (unchanged from v2 §9)

## §10 Conclusion (proposed full consensus, CRITIC-R1-CONDITIONS-MET version)

Per critic r1 recommendation: ITERATE once more narrowly. v3 closes all 5 fixes:
- ✅ Fix #1 (track artifacts): handled in commit batch landing v3
- ✅ Fix #2 (real alternatives): v3 §3 NEW
- ✅ Fix #3 (executable VL steps): v3 §5 NEW
- ✅ Fix #4 (narrow grit fallback): v3 amendment #9 narrowed
- ✅ Fix #5 (one tracked graph artifact): v3 amendment #4 narrowed
- ✅ Fix #6 (SLO measurement hooks): v3 amendment #10 measurement hooks added

After v3, dispatch architect r3 + critic r2 for final-layer APPROVE+APPROVE.

---

**Awaiting architect r3 + critic r2.**
