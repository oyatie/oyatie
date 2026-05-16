# Critic r1 (Torvalds lens) — Full-project Consensus 2026-05-13

## Verdict
ITERATE

## Session
n/a

## Quality-Criteria Check
- Principle↔option consistency:    PASS — v2 consistently narrows on the controller primitive: `desired registry row -> admission validation -> reconcile -> status/evidence` at planner-v2.md:17, 50, and blocks net-new classes until VL is operational at lines 27-28, 54.
- Fair alternatives:                 FAIL — v2 has no real alternatives section. It imports direction consensus by reference and jumps to “Adopt α + 10 amendments” at planner-v2.md:159-166. That is not fair-option evaluation; it is conclusion reuse.
- Risk-mitigation clarity:           FAIL — pre-mortem scenarios are named, but several mitigations are placeholders: “rollups + freshness windows” with no schema/threshold at planner-v2.md:125, “OR audit-chain” at line 43, and “investigate grit SQLite FK root cause OR build Oyatie-native VCS replacement” at line 108. That is two giant branches stuffed into one sentence.
- Acceptance-criteria testability:   FAIL — some VL steps are testable, but not each one. Step 3 says “CI green run URL recorded” without naming the CI lane or file target; step 4 accepts either grit hook or ICM fallback; step 5 references an `evidence-bundle-template` path that is not cited; step 6 accepts `/registry/graph/edges-2026-05-13.json` OR audit-chain. See planner-v2.md:40-43.
- Verification concreteness:         FAIL — v2 names a few files and commands, but not enough executable verification. It never gives the exact test command for the fixture, the `scripts/check.sh` invocation shape, the grit hook command, or the graph assertion. “CI green run URL” is not a local verification plan.

## User-Mandated-Rule Check
- (i)   honest-claims:           PASS — v2 admits 0/10 HG gates operational, 0 capabilities operational, no drift detector, no OTel exporters, and consensus pending at planner-v2.md:131-140.
- (ii)  Linus-grade:             FAIL — untracked consensus inputs and missing branch specificity allow silent regression in the plan itself. Architect r2 noticed the untracked full-consensus artifacts at architect-r2.md:31-32, then waved it away.
- (iii) verified-claims:         FAIL — `git ls-files` returned tracked entries for canonical specs/registries such as `/specs/active-machine-readable-artifact-contract.json`, `/specs/root-hub-pointers.json`, `docs/AGENTS.md`, `docs/DOC-CATALOG.md`, `/registry/artifact-capabilities-registry.json`, `/registry/reusable-building-blocks-registry.json`, `/specs/master-plan-sequencing.json`, and `registry/quality/lanes.yaml`. It returned no tracked entries for `/evidence/audits/consensus/2026-05-13-full/planner-v2.md`, `architect-r1.md`, `architect-r2.md`, or `planner-v1.md`.
- (iv)  honest-introspection:    PASS — v2 tracks real gaps at planner-v2.md:129-140 and records mechanical-enforcement holes at lines 146-157.

## Linus-style findings
1. Defect: The deliverable being approved is not HEAD-durable. Evidence: `git ls-files /evidence/audits/consensus/2026-05-13-full/planner-v2.md ... architect-r2.md` returned nothing, while `find` shows those files exist in the worktree. Architect r2 calls this “procedural” at architect-r2.md:31-32. No. If the consensus artifact is not tracked, nobody can verify it later. Concrete fix: land the full-consensus artifacts through the grit lane before APPROVE, or change verdict to conditional ITERATE.

2. Defect: v2 pretends alternatives were evaluated, but §3 is inherited rather than present. Evidence: planner-v2.md:9-13 says §2 and §3 are “unchanged” from v1; planner-v2.md:159-166 jumps directly to adoption. Direction consensus has alternatives at /evidence/audits/consensus/2026-05-13-direction/consensus-v1.md:79-83, but v2 does not restate, update, or test them against the 10 amendments. Concrete fix: add a compact §3: α amended, β operational pivot, freeze-only, rollback/defer; score each against the five principles and user rules.

3. Defect: The VL acceptance table is not fully testable. Evidence: planner-v2.md:38-44. Steps 1 and 2 are testable. Steps 3-6 are mush: “CI green run URL,” “or narrow ICM fallback,” “conforming to evidence-bundle-template,” and “OR audit-chain.” Concrete fix: for every step add `Command`, `Expected failure`, `Expected success`, and `Artifact path`. If audit-chain is blocked, do not make it an acceptance alternative.

4. Defect: The grit fallback mitigation is too broad for a state-transition rule. Evidence: planner-v2.md:101-108 allows ICM fallback and then says either investigate grit FK or build an Oyatie-native VCS replacement. That is planning theater unless bounded to the VL slice. Concrete fix: make fallback a single expiring file/ICM record with owner, expiry, validation command, and hard fail after 24h; move VCS replacement to a separate decision record.

5. Defect: Graph materialization is a fake abstraction until storage/query ownership is pinned. Evidence: planner-v2.md:60-62 lists `nodes`, `edges`, `reverse_indexes`, `unresolved_refs`, `owners`, `freshness`, `impact_queries`; planner-v2.md:43 accepts one edge file or audit-chain. `.omc/graph` currently has no files from `find .omc/graph -maxdepth 2 -type f`. Concrete fix: VL should produce exactly one tracked graph artifact and one checker assertion before naming the full materialization layer.

6. Defect: Scale SLOs are specified without measurement hooks. Evidence: planner-v2.md:110-117 adds p99 validation/runtime SLOs; no command or evidence file captures timings. Concrete fix: require `scripts/check.sh` or `oya-dev-cli gate validate active-artifact-contract --json` to emit duration fields into `/evidence/lane-run-${RUN_ID}.json`.

## Architect r2 disagreements
Architect r2 goes too easy on the tracking gap. At architect-r2.md:31-32 it says untracked consensus artifacts do not invalidate architecture. For this loop they do invalidate approval, because the user explicitly required verified claims via `git ls-files`.

Architect r2 also treats “specified” as “concrete.” Lines 10-19 pass every amendment because the words exist in v2. That misses the core Torvalds test: can a contributor implement and verify it without inventing half the missing contracts? For steps 3-6 of VL, the answer is no.

Architect r2 over-accepts the hyperscaler bar at lines 34-35. A hyperscaler-grade plan does not leave the admission hook, evidence template, graph write target, and fallback mode as branches. It chooses the boring first implementation and proves it.

## Recommended consensus position
ITERATE once more, narrowly. Do not rewrite the direction. Keep the resource-controller primitive and VL-first freeze. Required fixes:

1. Track or explicitly route the full-consensus artifacts through the grit-governed lane before final approval.
2. Add a real alternatives section to planner v2, even if short, and score alternatives against the five principles plus four user rules.
3. Replace VL steps 3-6 with executable acceptance rows: command, file, failure case, success case, and evidence path.
4. Collapse fallback branches: one grit path, one expiring degraded fallback, no VCS-replacement branch inside VL.
5. Require one tracked graph output plus one checker assertion before claiming graph materialization.

After those edits, APPROVE is likely. As written, architect r2 over-delivered rhetorically and under-verified mechanically.
