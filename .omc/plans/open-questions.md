# Open Questions

## oyatie-mega-plan-execution — 2026-05-12

- [ ] **Canonical hook-name replacements** for `WorktreeCreated` and `WorktreeRemoved` — Wave 0 #1 needs the exact new names from the mega-plan; planner inferred drift but not the target tokens. — Blocks W0-T1 implementation.
- [ ] **Canonical layer-count** (11, 12, or 13 layers) — Wave 0 #6 needs to know which doc is the source of truth before reconciliation. — Blocks W0-T6.
- [ ] **omx preflight crate location** — is it `crates/omx-preflight`, a script in `agents/`, or part of an existing crate? — Affects W0-T3 file path enumeration.
- [ ] **`session-start` hook owner crate** — Wave 0 #7 needs to know which crate owns the session-start path that will seed the `claims` row workaround. — Blocks W0-T7.
- [ ] **Skeleton state of `oya-foundry-account-*` crates** — are `Cargo.toml` deps already pointing in the clean-arch direction, or do they need re-wiring? — Affects estimate for P00-01..P00-08.
- [ ] **`/ultrawork` concurrency cap** — confirm 4 is the right concurrency for P00-03..P00-07 vs. the team's preferred default. — Tuning, not blocking.
- [ ] **ADR target location** — should the final ADR land in `/Users/jasonlee/bominal/docs/adr/` or in the plan file itself? — Affects hand-off step.

## ralplan-oyatie-sst-consolidation — 2026-05-12

- [ ] **Helper implementation language for `tools/oya-agent-read/`** — Rust (matches workspace crate idiom) vs Node/TS (matches typical CLI helper idiom). — Decided at P2 scaffold time; affects symbol-claim identifiers and test runner.
- [ ] **Next free ADR slot numbers** — Plan assumes ADR-0026 (inventory) and ADR-0027 (cutover). Must be reconciled against `ADR-INDEX.md` at write time; bump if taken. — Mechanical, blocks P1 + ADR landing.
- [ ] **Human-orchestrator carve-out scope** — P6/P7 file moves and P9 upstream issue filing require non-agent invocations of `git mv`/`git rm`/`gh issue create`. Confirm the rule reads as "agents do not invoke git/gh"; humans orchestrating the cutover do. — Flagged inline at P6/P7/P9; affects whether the cutover can proceed at all.
- [ ] **CI extension to flag archive-path tokens** — Pre-mortem #2 mitigation requires the banned-primitives fitness lane to also grep for archive-path tokens before P7 merges. Confirm the lane crate's scope. — Affects P7 gating.
- [ ] **Demo symbol selection** — P8 demo must claim two real grit-indexed `file::Identifier` symbols in non-overlapping crates. Specific symbols TBD at demo-script-author time. — Affects P8 reproducibility.
- [ ] **Archive retention policy** — ADR Follow-up 3 proposes 60 days. Confirm duration with user before policy lands. — Tuning, not blocking.
- [ ] **`oya-agent-write` future surface** — Spec §Assumptions item 10 floats `oya-agent-write pr-finalize` if `grit done` is local-only. Defer to P9 follow-up; out of scope for this cutover. — Tracking.

## convention-audit-and-rename — 2026-05-12

- [ ] **Compound-feature policy choice (Policy A vs Policy B)** — Sub-plan A admits 28 new compound features into `[workspace.metadata.oya] compound_features` via a single ADR (low churn). Sub-plan B (Policy B alternative) collapses the foundry-fitness kernel family under a single `fitness` feature umbrella (large refactor; consensus-required). Both satisfy the grammar. — Blocks execution of the rename plan; the lane stays in `--report-only` until adjudicated. See `docs/audits/convention-audit-2026-05-12.md` §7 and `docs/plans/rename-plan-2026-05-12.md` §1, §8.
- [ ] **`oya-foundry-api` future** — Plan §2.1 surfaces ambiguity: rename to a distinct feature (e.g. `oya-foundry-meta-api`) or merge into the existing `oya-foundry-policy-api`. Architect call. — Blocks Sub-plan B step 1.
- [ ] **`oya-tooling-cli-dev-runtime` final name** — Plan §2.2 enumerates parses; recommendation is `oya-tooling-dev-runtime`. High-risk because the crate hosts the `oya` and `repoctl` bin entries that every CI workflow names. — Blocks Sub-plan B step 9; consensus-gated.
- [ ] **Sub-plan A ADR number** — `ADR-FND-008` is proposed for "Workspace compound-feature registry extension". Reconcile against `ADR-INDEX.md` at write time; bump if taken. — Mechanical.
- [ ] **AMBER-metadata cutover date** — Sub-plan A defers the per-crate `[package.metadata.oya]` block requirement to Q3-2026 BLOCKER promotion. Confirm the target date with the user. — Tuning. **(SUPERSEDED by `rename-plan-v2-2026-05-12.md`: AMBER-metadata cutover is now IMMEDIATE per Policy B / ICM `01KRFMEVN49BB6J0QWKNGATC1K`. Retained for v1 audit trail only.)**

## rename-plan-v2 (Policy B + immediate metadata cutover) — 2026-05-12

- [ ] **Sharded vs atomic cutover (force consensus)** — v2 §4.3 recommends Option B (6 shards). Codex critic to pressure-test the 2-week calendar cost vs. atomic-PR bisectability loss. — Blocks shard-1 open.
- [ ] **ADR-0055 slot + title** — v2 §11 reserves ADR-0055 for "Adopt Policy B fitness-umbrella crate taxonomy and immediate metadata cutover". `ADR-INDEX.md` says "Next ADR number: 0055". Confirm slot is still free and title scope is correct at shard-1 author time. — Mechanical, blocks shard-1.
- [ ] **xtask-metadata-augment scheduling** — v2 §3.3 recommends shipping the metadata helper in shard 1 (alongside the high-risk row 37 CI cutover). Should it be a shard 0 precursor instead? — Risk-balance call; blocks shard-1 scope.
- [ ] **48 h freeze window length** — v2 §5 item 14 suggests 48 h between shard merges. Confirm or override (24 h / 72 h alternatives). — Tuning.
- [ ] **Context-enum extension** — v2 §3.1 keeps `fitness` as a feature within context `foundry` (six-context enum). User-supplied BNF proposed `fitness` as a seventh context. Architect/critic decision required **before shard 6 opens**. — Blocks shard 6.
- [ ] **`oya-platform-data-boundary-kernel` rename (row 35)** — v2 §10 question 6: the only kernel that may receive cross-layer deps per `clean-architecture.md` §3. Rename in shard 4, or defer to a dedicated follow-up PR with registered `data-boundary` compound? — Blocks shard 4 scope.
- [ ] **AMBER-row optional cleanup** — v2 row 39 reservation: rename `oya-platform-audit-chain-adapter-file` to a 5-segment form? Default: no. Critic to confirm. — Tuning, blocks shard 4 close.
- [ ] **CI workflow file names** — v2 §10 question 8: keep `.github/workflows/{release-evidence-pack,supply-chain}.yml` file names unchanged while flipping their `cargo run -p` invocations to the renamed binaries? — Confirm; affects shard 1 + shard 6 PR scope.
- [ ] **Rename-count reconciliation** — v2 §2 final-cohort note: user direction said "37 RED + 2 named = 39 renames" but the 2 named decisions are **already inside the 37-row RED list** (rows 4 and 37 of the audit). Plan adopts 37 unique renames. Confirm with user before ADR-0055 lands. — Mechanical but load-bearing; blocks ADR title.

## rename-plan-v3 (Hybrid C: Shard 0 + atomic Shard 1) — 2026-05-12 — iteration 2

These supersede the rename-plan-v2 entries above (kept in tree as historical record). v3 §10 enumerates the top-3 questions most likely to surface in `/ralplan --critic` iter-2 pressure-testing.

- [ ] **xtask-metadata-augment specification completeness** — The xtask is the atomic rewriter for 140 manifests; if its derivation rule for `feature`, `layer`, or especially `audit_chain` has any edge case, the atomic Shard 1 produces 140 wrong blocks at once. Codex iter-2 likely demands: (a) unit-test matrix for the xtask's derivation rule, (b) `audit_chain` derivation source (cargo-metadata feature flags? hand-annotated allow-list? per-crate ADR cite?), (c) dry-run diff sample for 5 representative crates with sign-off **before Shard 1 opens**. — Blocks Shard 1 merge gate.
- [ ] **48 h freeze enforcement mechanism** — v3 §6 R2 says "single 48 h freeze" but does not specify HOW the freeze is enforced. Options: (a) GitHub branch protection rule preventing merges to `main` for 48 h before Shard 1; (b) fitness lane `oya-foundry-fitness-rename-freeze-window` that fails any PR merged in the freeze window; (c) merge-queue label `freeze-rename-v3` blocking dequeue. Critic iter-2 will pressure-test which mechanism is authoritative and how the security-expedite lane (R10) cleanly overrides it. — Blocks Shard 1 open.
- [ ] **Row 35 (`oya-platform-data-boundary-kernel`) 95-consumer dep-edge correctness verification** — Rename rewrites 95 path edges. v3 trusts the xtask. Codex iter-2 will likely demand: (a) `cargo metadata` reverse-dep query that proves all 95 consumers are accounted for, (b) xtask diff against `cargo metadata` to confirm zero orphan edges, (c) plan for the (currently unobserved) case of a `build.rs` constructing a dep-name string dynamically. — Blocks Shard 1 §8.1 acceptance gate for path-edge diff.
- [ ] **Cargo.lock single-event regen strategy** — v3 §1 + §5.2 step 12 claim a single `cargo update --workspace --offline` regen event. Confirm `--offline` produces a deterministic lockfile against the renamed-only delta (no unrelated minor-version bumps slip in). If `--offline` is too conservative for the dependency graph at rename time, fall back to documented `--package <name>` per-crate regen — but lose the single-event property. — Tuning, may affect §8.1 Cargo.lock zero-old-names gate definition.
- [ ] **R9 cargo-semver-checks baseline strategy operational details** — v3 §6 R9 commits "rename PRs reset the semver baseline" and §8.1 gate allows only `BASELINE-RESET` class failures. Confirm: (a) is `cargo-semver-checks` versioned ≥ 0.30 (which supports `--baseline-rev`)? (b) does the BASELINE-RESET class need to be encoded as a custom semver-checks rule or is it captured by the existing rule set? (c) 14-day post-merge grace window — who closes the grace and how? — Affects §8.1 gate definition + post-merge operations.
- [ ] **R10 security-expedite lane authority chain** — v3 §7.2 pre-authorises admin-merge under ADR-0055 §"Rollback/expedite protocol". Confirm: (a) which exact roster (council-architecture, axis-foundry, security-council) holds expedite authority; (b) does the `gh pr merge --admin` exemption need a separate ADR or is the ADR-0055 cross-ref sufficient; (c) post-emergency observability sweep is non-blocking — under what conditions does it become blocking? — Blocks final ADR-0055 §"Rollback/expedite protocol" wording.
- [ ] **Compound-features registry approval gate** — v3 §3.2 enumerates 31 new compound capabilities for `[workspace.metadata.oya].compound_features`. Per `crate-naming-convention.md §7.1`, additions REQUIRE an ADR cite in CHANGELOG. v3 assumes ADR-0055 is the single cite. Critic iter-2 may push for per-compound individual rationale rows (32 rows × ADR cite each) vs. v3's batch cite. — Affects ADR-0055 §"Compound capability audit" verbosity.
- [ ] **Row 16 (`documentation-system`) AMBER carve-out** — v3 §2 row 16 cap-tok=2 sits at the 2-token capability cap AND the name is 6 segments (AMBER per `crate-naming-convention.md §2` constraint 1). v3 cites ADR-0055 as the AMBER carve-out. Codex iter-2 may pressure-test: is 6-segment AMBER acceptable atomically, or should row 16 be deferred / decomposed further? — Affects row 16 final-form decision.

## rename-plan-v3 — iter-2 closure (folded into v3 in-place, 2026-05-12)

Both reviewer reports converged: Architect iter-2 = SOUND-WITH-CONDITIONS with 3 residuals (freeze primitive, lockfile regen, xtask matrix); Codex iter-2 = ITERATE with 7 required edits (all 1-paragraph / 1-line). Codex's 7 edits subsume Architect's 3 residuals. v3 folded all 7 in-place (no v4 rewrite). Plan status remains `pending approval` pending iter-3 consensus.

**Edits folded** (all 7 closed, zero deferred):

- [x] **EDIT 1 — Lockfile regen primitive** — v3 §1 row 10 + §5.2 steps 12a/12b/12c + §7.1 + §8.1 "Cargo.lock semver-section parity" gate. Replaces `cargo update --workspace --offline` with scripted name-rewrite + `cargo check --workspace --locked --offline` + jq-based metadata-diff gate. Architect residual #2 closed as byproduct. — Status: CLOSED.
- [x] **EDIT 2 — xtask fixture-test coverage matrix** — v3 §3.3.1 specification table (20 cells across 9 dep-edge forms × 4 table types + negative fixtures) + Shard 0 step 3a acceptance gate `cargo nextest run -p xtask-metadata-augment --test fixtures`. Architect residual #3 closed as byproduct. — Status: CLOSED.
- [x] **EDIT 3 — Lane-health gate determinism** — v3 §8.1 lane-health row replaced with deterministic shell predicate (`test ... -eq 0`); threshold pinned at `impossible_to_fail_count == 0` for 30-day window. — Status: CLOSED.
- [x] **EDIT 4 — cargo-semver-checks pinning** — v3 §8.1 Semver-checks row pinned to `cargo-semver-checks 0.46.0` + `--format json > /tmp/semver-output.json` + new fitness lane `oya-foundry-fitness-rename-baseline-reset` defined as the BASELINE-RESET classifier (scaffolded in Shard 0 step 7b). — Status: CLOSED.
- [x] **EDIT 5 — Freeze-enforcement primitive** — v3 §6 R2 rewrites with new fitness lane `oya-foundry-fitness-rename-freeze-window` (scaffolded in Shard 0 step 7a). Config = freeze_active + freeze_end_ts + expedite_override_token (single-use rotation). Merge-queue calls lane on dequeue. Architect residual #1 closed as byproduct. — Status: CLOSED.
- [x] **EDIT 6 — Row 35 evidence command + reverse-dep gate** — v3 §1 row 35 command corrected to unquoted, manifest-scoped, excluding root + crate's own manifest; new §8.1 row "Row 35 reverse-dep count == 95" enforces `cargo metadata`-derived consumer count post-rename. — Status: CLOSED.
- [x] **EDIT 7 — Row 37 test/release fixture rewrites** — v3 §2 row 37 expanded to list `tests/gate_cli.rs` (lines 2830/2868/2879/3456/3465/3471/3472), `tests/repoctl_cli.rs` (149/159), `src/commands/repoctl.rs:43`. §12 reference inventory updated with the test fixtures + release supply-chain YAML/SBOM/GHCR image refs. — Status: CLOSED.

**Finalisation edits also applied**:

- [x] **EDIT 3-finalisation (Codex #3 sanctioned-primitives partial)** — v3 §5.0 + ADR-0055 §"Rollback/expedite protocol" both now require all three preconditions (freeze_active=true, Security-Council-minted token, ICM rationale store) for any `gh pr merge --admin` invocation; absent any precondition, the invocation remains a banned-primitives violation. — Status: CLOSED.
- [x] **EDIT 10-finalisation (Codex per-compound rationale)** — v3 §2 + ADR-0055 §"Compound capability audit" now carry the explicit one-liner: 31 compounds admitted as one taxonomy family per Policy B; individual rationale only for AMBER exceptions (row 16). Closes Codex edit #10 as APPROVE-WITH-CONDITION. — Status: CLOSED.

**Soft edits applied** (1-line each, no deferrals):

- [x] **Reviewer-hours disambiguation** — All v3 references to "6–8 reviewer hours" (§4.1, §4.3 twice, §9 row, §11 consequence) now read "6–8 h per primary reviewer, ~3 reviewers parallel = 18–24 h calendar reviewer-hours". — Status: APPLIED.
- [x] **Staging-promotion fallback** — v3 §7.2 carries the 1-paragraph fallback: if Shard 1 reached staging before revert, the revert PR title prefixes `REVERT-STAGING-BLOCK:`, the staging-promotion lane refuses subsequent promotion until a `STAGING-UNBLOCK:` follow-up, and the observability sweep is BLOCKING (vs. non-blocking on the normal path). — Status: APPLIED.
- [x] **Hybrid C-Lite escape hatch** — v3 §11 ADR-0055 carries the escape hatch: if the 48 h freeze cannot be scheduled within 2 weeks of Shard 0 merge, Shard 1 is held in a long-lived feature branch with daily `git rebase main` cadence (xtask is idempotent against rebased base). — Status: APPLIED.

**Deferred items**: NONE. All 7 required edits + 2 finalisations + 3 soft edits were 1-paragraph-or-shorter and folded in-place this session.

**Next**: iter-3 — Architect + Codex critic re-review against the folded v3. Plan stays `pending approval`. Expected pressure-test surfaces for Codex iter-3:
1. The `tools/lockfile-rename.py` script — does it actually exist or is it a forward reference? (currently a forward reference; needs to be authored in Shard 0 alongside the xtask).
2. Whether the 95-reverse-dep gate's `jq` query produces *unique* consumer names — Codex may probe duplicates.
3. The `oya-foundry-fitness-rename-baseline-reset` lane's classifier — does it correctly distinguish a name-only delta from a real semver delta? Needs the lane's algorithm spec.

## rename-plan-v3 — iter-3 pre-fold closure (folded into v3 in-place, 2026-05-12)

Architect iter-3 verdict: SOUND-WITH-CONDITIONS with 3 residuals (1 HIGH, 1 MODERATE, 1 BLOCKER). All 3 folded into v3 in-place before Codex critic iter-3 ran (no v4 rewrite). Frontmatter updated: `architect_iter_3: SOUND-WITH-CONDITIONS (3 residuals, pre-folded)`. Plan status remains `pending approval`; iteration stays at 3 (sub-revision).

**Residuals folded** (all 3 closed, zero deferred):

- [x] **RESIDUAL 1 — HIGH: BNF grammar violation in new lane crate names** — The two new fitness-lane crates introduced in the iter-2 fold (`oya-foundry-fitness-rename-freeze-window` and `oya-foundry-fitness-rename-baseline-reset`) violated `docs/standards/crate-naming-convention.md §2` BNF: missing role token and 3-token capability tail (cap is 2). Renamed both per Architect-recommended fix, dropping the `rename-` prefix and appending `-kernel` role token: `oya-foundry-fitness-freeze-window-kernel` and `oya-foundry-fitness-baseline-reset-kernel`. Both parse as feature=`fitness-<*>` (3 tokens at feature cap), role=`kernel`, no capability (forbidden for kernels per constraint 4). Updated all 7 occurrence sites in v3 (§1 inventory, §5.0 ADR-exception, §5.1 Shard 0 steps 7a/7b, §6 R2, §8.1 semver gate, §10 open-questions excerpt, §11 ADR-0055 protocol). Also added 3 new feature-compound entries to `[workspace.metadata.oya].compound_features` in §3.2 to satisfy `crate-naming-convention.md §6` rule 1: `fitness-architecture-conventions`, `fitness-freeze-window`, `fitness-baseline-reset`. — Status: CLOSED.

- [x] **RESIDUAL 2 — MODERATE: `oya-tooling-agent-read lane-config set` write-through-read** — §6 R2 had used the READ-named sanctioned-primitive (`oya-tooling-agent-read`, triad READ slot per `git-workflow.md §2-3`) for a write operation (lane-config token mint). Chose OPTION A per Architect recommendation: routed the token-mint through `icm store` (already a sanctioned primitive triad member; the same operation auto-satisfies the Directive 12 rationale-row requirement). Token mint via `icm store -t lane-config-oyatie -c "freeze_window:expedite_token=$(uuidgen)" -i critical -k "lane=oya-foundry-fitness-freeze-window-kernel"`; lane runtime reads via `icm recall ... | jq -r '.[] | select(.content | contains("expedite_token=")) | .content | sub(".*expedite_token=";"")'`; rotation via tombstone row `expedite_token=REVOKED` (latest-row-wins). No §5.0 sanctioned-primitive triad table needed editing (the table is in `git-workflow.md`, not v3). — Status: CLOSED.

- [x] **RESIDUAL 3 — BLOCKER: `tools/lockfile-rename.py` forward reference** — §5.2 step 12b and §7.1 referenced a script that did not exist. Added Shard 0 step 1b (author script) + step 3b (pytest unit tests as REQUIRED acceptance gate) + step 15a (generate `/tmp/old-crate-names.txt` and `/tmp/rename-map.tsv` via awk over the §2 rename inventory). Added new §7.1.1 with full script spec: CLI (`--rename-map`, `--lockfile`, `--inplace`, `--reverse`), behaviour (parse `Cargo.lock` via stdlib `tomllib`; rewrite `[[package]] name` for workspace members + `dependencies` array entries; preserve `version`/`source`/`checksum`; missing-map entry → no-op + warning), and 6-row unit-test matrix. Updated §5.2 step 12b + §7.1 revert path to use the standardized CLI flags. Added new §8.1 gate row "Lockfile-rename script unit tests" (`pytest tools/lockfile-rename_test.py -q` exit 0) gating Shard 1 entry. Also de-forward-referenced the `/tmp/old-crate-names.txt` and `/tmp/rename-map.tsv` generators by inlining awk one-liners in Shard 0 step 15a. — Status: CLOSED.

**Soft conditions NOT yet addressed** (kept open for Codex iter-3 visibility):
- The low-severity rotation race in §6 R2 (between `icm store` REVOKED tombstone write and the next `icm recall` from a parallel merge-queue dequeue) is mitigated by `latest-row-wins` semantics + single-use minting but is not strictly serializable. ICM does not guarantee strict ordering across concurrent writers; if two merge-queue dequeues race on the same expedite token consumption, both could observe the live token before the REVOKED tombstone lands. Mitigation: Security Council mints tokens with PR-id bound in the content payload (`expedite_token=<uuid>;pr=<n>`) and the lane runtime cross-checks `pr=<n>` against the requesting PR. Documented here for Codex iter-3 scrutiny; not promoted to a v3 edit because the operational expedite cadence (≤1 per Shard 1 cutover window) makes the race statistically negligible — but Codex may push for explicit PR-id binding to make the invariant deterministic rather than statistical.

**Top-2 expected pressure tests for Codex iter-3**:
1. **`tools/lockfile-rename.py` `--reverse` mode correctness when the same crate name appears as both an old name AND a new name across the 37-row map** — Codex will likely construct an adversarial rename map (e.g. row N maps `A → B`; row M maps `B → C`) and probe whether the reverse-pass produces a consistent inverse. Spec needs an explicit "no-cycle / no-collision" invariant on the map, or the script must topologically order applications. v3 §7.1.1 does not currently mandate this invariant.
2. **`icm recall` JSON Lines stability for the lane runtime read path** — Codex will pressure-test whether `icm recall --format jsonl` is a stable contract (since the runtime parses content strings with a fragile `sub(".*expedite_token=";"")` jq filter). If `icm` ever changes the `content` field shape or escaping rules, the lane silently misreads the token. Mitigation paths (none committed yet): (a) use a structured field instead of regex-encoded content; (b) pin the `icm` version in `tools/toolchain-versions.toml` alongside `cargo-semver-checks 0.46.0`; (c) add a Shard 0 contract test that round-trips a known token through icm-store → icm-recall.

## rename-plan-v3 — iter-3 approve-fold closure (folded into v3 in-place, 2026-05-12)

**Final consensus state**: Architect iter-3 SOUND-WITH-CONDITIONS (3 residuals, pre-folded → CLOSED). Codex critic iter-3 APPROVE-WITH-CONDITIONS (3 conditions, folded in same session → CLOSED). All conditions closed; no v4 created; no re-litigation of Hybrid C or prior decisions.

**3 Critic iter-3 conditions closed** (folded in-place into v3):

- [x] **CONDITION 1 — Lockfile pytest matrix coverage gap (CLOSED)**. Added 2 new pytest rows to the `tools/lockfile-rename.py` matrix in v3 §7.1.1: row 7 (dependency entry with version disambiguator: `"old-name 0.1.0"` → `"new-name 0.1.0"`) and row 8 (dependency entry with version+source disambiguator: `"old-name 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"` → `"new-name 0.1.0 (registry+...)"`). Added explicit disambiguator-preservation invariant to §7.1.1 narrative ("script splits each dependency-array entry on the first whitespace; only the leading name token is rewritten; version and source-disambiguator suffixes are preserved character-for-character"). Updated Shard 0 step 3b acceptance gate text + §8.1 Lockfile-rename script unit tests row to confirm 8-row matrix coverage.

- [x] **CONDITION 2 — PR-bound expedite tokens (CLOSED)**. Changed token payload from `expedite_token=<uuid>` to `expedite_token=<uuid>;pr=<n>` in v3 §6 R2 + §11 ADR-0055 §"Rollback/expedite protocol". Updated mint command (Security Council adds `;pr=${PR_NUM}` to content + `,pr=${PR_NUM}` to key), updated lane runtime check (merge accepted ONLY when both UUID matches AND requesting PR number matches `pr=` field), updated tombstone (per-PR `REVOKED` row keyed on `pr=${PR_NUM}` so cross-PR tokens are independent). ADR-0055 §"Rollback/expedite protocol" §3rd precondition extended: "and the requesting PR number must match the token's `pr=` field" alongside the existing 3 preconditions. Eliminates the statistical rotation race surfaced in iter-3 soft-condition (line 101) by making the invariant deterministic per-PR rather than statistical across PRs.

- [x] **CONDITION 3 — ICM JSONL round-trip contract test + icm version pin (CLOSED, BOTH applied for minimal cost + maximal safety)**. (a) Pinned `icm` version in `tools/toolchain-versions.toml` (referenced in §8.1 semver-checks row alongside `cargo-semver-checks 0.46.0` pin; the pin row will be added during Shard 0 step 1b). (b) Added new Shard 0 step 7c as REQUIRED acceptance gate: synthetic-PR-token round-trip through `icm store` → `icm recall --format jsonl | jq | sed -E 's/.*expedite_token=([^;]+);pr=([0-9]+).*/\1,\2/'` → assert `RECOVERED == EXPECTED` → tombstone cleanup. Added matching §8.1 gate row "ICM JSONL round-trip contract" so the contract test is enforced at the Shard 0 merge gate. Anchors the lane-runtime's PR-bound-token parse contract against a pinned `icm` JSONL schema.

**Plan transition**:
- `status: pending approval` → **`status: approved`** (consensus-locked: Architect SOUND-WITH-CONDITIONS-CLOSED + Critic APPROVE-WITH-CONDITIONS-CLOSED)
- New frontmatter row: **`pending: execution-approval-from-user`** (separate user gate; plan is consensus-locked but NOT yet user-approved for execution)
- Frontmatter updates: `architect_iter_3: SOUND-WITH-CONDITIONS (3 residuals, pre-folded → CLOSED)`, `critic_iter_3: APPROVE-WITH-CONDITIONS (3 conditions, folded)`

**Total loop cost**:
- 3 iterations (iter-1 → iter-2 → iter-3)
- ~16 hours agent compute (architect + critic + planner fold cycles)
- 13 residuals closed across iter-1/iter-2 (Architect 3 conditions + Codex 7 edits + Architect-iter-2 3 residuals) + 3 conditions closed in iter-3 (Critic) = **16 total residuals closed**, zero deferred
- Zero re-litigation; Hybrid C decision held from iter-1 dominant convergence through iter-3 approve

**Next gate**: user execution-approval (separate from consensus approval). Plan stays gated; no rename runs until user explicitly approves execution.

## rename-plan-v4-clean-arch (Clean Architecture canonical BNF) — 2026-05-13 — iter-1

v3 is SUPERSEDED by v4. Pivot rationale (read first): user pressure-test
on v3 exposed three over-engineered layers that the consensus loop never
caught because reviewers iterated within v3's own framing rather than
against the simpler external standard. Specifically:

1. **Verbose 4–5 segment BNF** (`oya-<context>-<feature>-<capability>-<role>`)
   produced names like `oya-foundry-fitness-architecture-conventions-kernel`
   that nobody used in conversation. Replaced in v4 by
   `oya-<bounded-context>-<layer>` (with optional `<thing>` slot) +
   `oya-check-<rule-name>` flat namespace.
2. **`oya-foundry-fitness-freeze-window-kernel` lane** + ICM
   `lane-config-oyatie` topic + `expedite_override_token` rotation
   duplicated grit's existing claim/symbol-lock system (ADR-0054).
   Dropped in v4; grit's normal authority handles the 48 h coordination
   window.
3. **"Fitness" terminology** imported wholesale from
   *Building Evolutionary Architectures* never settled into the team's
   vocabulary; every v3 "fitness" crate was an ADR-cite probe, an
   architectural check, or a doc-coverage audit. Replaced in v4 by
   the plain noun `check`.

Additional corrections during v4 iter-1 author session (cumulative;
final state is draft-4 = **12-value canonical enum + canonical decision
tree**):

- **draft-1 → draft-2 (6 → 9 values)**: user feedback "make sure you
  account for all presentation layers as well. like grpc etc. we have
  more than just api, cli". Expanded enum from 6 to 9 values (inner:
  domain/application/infrastructure; presentation: cli/rest/grpc/
  graphql/worker/sdk). `api` dropped because it does not name a
  wire format.
- **draft-2 → draft-3 (9 → 10 values)**: user feedback "we can leave
  adapter/infrastructure as is. i think that is still within the bounds
  of clean architecture". Re-added `adapter` alongside `infrastructure`
  to preserve Uncle Bob's strict separation of interface adapters
  (trait impls + DTO mappers) from frameworks & drivers (non-trait
  glue).
- **draft-3 → draft-4 (10 → 12 values; FINAL)**: user feedback "just
  make sure when one is more appropriate over the other and establish a
  standardization using the canonical and conventions of clean
  architecture". Finalized to 12 canonical values with NO aliases /
  overlaps + canonical decision tree (§2.2.4) for deterministic
  per-crate layer assignment:
  - *Inner / pure (4)*: `kernel` (pure types + ports), `domain`
    (business logic on entities), `application` (use-case
    orchestrators), `app` (composition root binary)
  - *Outer / external (2)*: `adapter` (trait impls + DTOs),
    `infrastructure` (frameworks & drivers / non-trait glue)
  - *Presentation / entry-point (6)*: `cli`, `rest`, `grpc`, `graphql`,
    `worker`, `sdk`
  Audit implication: every `*-kernel` crate must be `src/`-inspected by
  Codex iter-1 to decide kernel-vs-domain; every `-api` must be
  classified by protocol; every `-app` must distinguish use-case
  orchestrator (`application`) vs. composition root (`app`).

- **draft-4 → draft-5 (12-value enum + 2-slot BNF; FINAL state for
  iter-1-fold-A)**: combined three corrections in one fold session.
  - **Fold-A: 11-check codification ruleset** (replaces 6-check list):
    A1 `oya-check-clean-architecture` (meta), A2 `-layer-correctness`
    (per-layer heuristic table), A3 `-dependency-direction` (12-value
    matrix), A4 `-bounded-context-registry` (BC validation + 90-day
    deprecation), A5 `-naming-collision`, A6 `-check-namespace`, A7
    `-metadata-schema`, A8 `-lockfile-parity`, A9 `-lib-name-parity`
    (R4 5-layer permanent control), A10 `-cargo-deny`, A11
    `-rename-baseline-reset` (R7 semver baseline).
  - **Fold-A: 7 architect-iter-1 conditions CLOSED**: B1 ports placed
    in `kernel` not `domain`; B2 `layer_evidence` audit column added;
    B3 BC arbitrator clause (council-architecture default + tie-breaker
    procedure); B4 R10 5-layer permanent controls parity with R4; B5
    3-partition parallel reviewer streams (1a platform / 1b cloud /
    1c foundry+workspace+foundation) + sign-off gate; B6 chicken-and-egg
    avoidance via `report-only`-then-BLOCKER-flip with 24 h target; B7
    BNF accommodation for proc-macros, codegen crates, test-fixtures,
    library+binary split-the-crate rule.
  - **Third correction: 2-slot BNF (`<thing>` slot DROPPED)** — user
    feedback "thing is not a good name". Final BNF is
    `oya-<bounded-context>-<layer>` only. Granularity expressed via
    multi-token BC names registered in `docs/standards/bounded-contexts.md`.
    BC registry grows from ~72 → **~100 entries**. Pattern G (BNF
    with optional `<thing>` slot) added to ADR-0056 Alternatives as
    REJECTED. The on-disk crate names did NOT change (e.g.
    `oya-compute-vm-api` parse shifts from `BC=compute, thing=vm` to
    `BC=compute-vm` — same name, simpler parse).
  v4 plan frontmatter updates: `consensus_loop: v4-iter-1-fold-A`,
  `architect_iter_1: 7-conditions-CLOSED`,
  `fold_state: 12-layer-canonical + 11-check-codification + 2-slot-BNF
  (thing-slot DROPPED)`.

Top-2 Codex iter-1 pressure-test surfaces (post-fold-A + 3rd-correction):
1. **Multi-token bounded-context governance at 100-entry scale** — flat
   vs. hierarchical model for related BCs; deterministic B3 tie-breaker
   when both PRs cite same ADR; review cadence at 100-entry scale.
2. **`layer_evidence` audit completeness at 139-crate scale** — per-crate
   `src/`-inspection evidence; A2 heuristic edge cases (struct+fn-body
   crates; cfg-gated items; trivial getters); PROTOCOL-UNKNOWN deferral
   markers blocking-vs-shipping behaviour.

Secondary: 11-check BLOCKER-flip ordering window (24 h target between
Shard 1 merge and the follow-up `severity: BLOCKER` flip PR).

v4 plan: [`docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`](../../docs/plans/rename-plan-v4-clean-arch-2026-05-13.md).

Top 3 expected pressure-test surfaces for Codex iter-1:

- [ ] **Canonical-decision-tree per-crate `src/`-inspection audit (top
  surface)** — v4 §3 provides provisional layer defaults but the
  canonical 12-value enum requires `src/`-inspection per crate via the
  decision tree (§2.2.4). Two sub-surfaces:
  - **(1a) kernel-vs-domain reclassification**: every v3 `*-kernel`
    crate must be `src/`-inspected to decide whether it is PURE types +
    ports (stays `kernel` under v4) OR carries business logic (relayer
    to `domain`). Strongest `kernel`-preservation candidate:
    `oya-platform-data-boundary-kernel` (per `clean-architecture.md §3`
    named-by-identity). Possibly several `oya-foundry-*-kernel`
    crates that are pure check-rule type bundles.
  - **(1b) Per-`*-api` protocol classification**: ~22 crates need
    explicit protocol audit. (a) gRPC candidates (`oya-cloud-observability-api`
    for OTLP; `oya-foundry-rag-api` for streaming retrieval;
    `oya-compute-k8s-api` for k8s watch streams); (b) GraphQL candidates
    (workspace user-facing surfaces — `oya-chat-api`, `oya-drive-api`,
    `oya-meet-api`); (c) multi-protocol crate split-vs-exception
    decision; (d) `worker` candidates for queue-driven surfaces.
  - **(1c) application-vs-app classification**: every v3 `*-app` crate
    must be distinguished — use-case orchestrator (library code) →
    `application`; composition root (deployable binary) → `app`.
    `oya-foundation-app` is the canonical `app` case (row 138).
  The audit answer determines proposed_name for ~22 `-api` rows AND
  may flip layer assignments for any number of `-kernel` and `-app`
  rows. — Blocks ADR-0056 §3 audit close.

- [ ] **Bounded-context registry governance** — v4 §2.4 establishes
  `docs/standards/bounded-contexts.md` as a living document with 90-day
  auto-deprecation. Likely needs Codex iter-1 pressure-test on: (a)
  ownership of bounded-context naming disputes; (b) deterministic vs.
  advisory enforcement of 90-day auto-deprecation; (c) per-entry ADR cite
  requirement vs. prose maintenance. — Blocks ADR-0056 §"Bounded context
  registry as a living document" close.

- [ ] **"Fitness" terminology drop blast radius + check-crate BLOCKER
  chicken-and-egg** — Every v3 fitness crate referenced across ~30 sites
  in `scripts/`, `.github/`, `docs/`. Likely Codex iter-1 pressure-test:
  (a) xtask coverage of every fitness-crate reference (especially the
  load-bearing `oya-foundry-fitness-architecture-conventions-kernel`);
  (b) whether BLOCKER-flip strategy for check crates in Shard 1 step 15
  introduces chicken-and-egg if a check crate's BLOCKER mode would fail
  the Shard 1 merge itself; (c) whether `.omc/fitness-lanes/` directory
  should be renamed `.omc/check-rules/` atomically with Shard 1. — Blocks
  Shard 1 step 15 finalisation.

Plan transitions in Shard 0 commit:
- v3: `status: approved` → `status: Superseded`,
  `superseded_by: docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`,
  banner block prepended; retained in tree as historical record.
- v4: `status: pending approval`, iteration: 1, consensus_loop: v4-iter-1.

Next: Architect iter-1 (Opus) review against v4. Then Codex critic iter-1
(`gpt-5.5-xhigh`) review. Plan stays gated until consensus-locked AND
user execution-approval received.

## rename-plan-v4 iter-2 postfold-A — 2026-05-13 (Codex iter-2 ITERATE-7 folded; D1–D7 closed)

This section REPLACES the iter-1 v4 entries above for current-state
reference (those entries are retained for traceability of the
draft-1 → draft-5 BNF evolution but are no longer the authoritative
current state).

**HONEST-CLAIM CORRECTION (per Codex iter-3 edit 6)**: The iter-2
postfold-A entry below originally claimed "draft-5 2-slot final and
all references to an optional `<thing>` slot are PURGED from prose"
— that claim was **partially accurate** at iter-2 closure. Codex
iter-3 found surviving stale references in (a) the frontmatter
`purpose:` block (still said "2-slot grammar"), (b) §3.1–§3.5 body
rows (still carried `thing?` column data + `rest (provisional)`
cells; only the COLUMN HEADERS had been rewritten, not the per-row
tuples), (c) §3.0 metadata-schema prose comment ("Keeps
`bounded_context`, `thing`, `layer`"), (d) §6 R10 lane row (singular
`oya-check-bounded-context-registry`). What WAS purged at iter-2:
§2.1 BNF `<thing>` slot declaration, §3 column directive prose. What
was NOT purged until iter-3: the four surfaces above. Iter-3 fold
corrects all four — see the new `## rename-plan-v4 iter-3 fold —
2026-05-13` section below for E1–E5 closure cites.

The current authoritative state for v4 is:

**BNF (FINAL, 3-slot)**: `oya-<shared|vertical>-<bc>-<layer>` with
single-token verticals (Option A per ADR-0056 §"Vertical naming
policy"); `shared` is a reserved literal that the verticals registry
refuses; check crates use `oya-check-<rule-name>` flat namespace.
Multi-token verticals BANNED — granularity expressed via multi-token
bounded contexts in slot 3, not via vertical names in slot 2. The
draft-5 "2-slot final" and all references to an optional `<thing>`
slot are PURGED from prose; v4 plan §3.0 + §3 audit + ADR-0056 all
declare 3-slot grammar exclusively.

**Layer enum (FINAL, 12-value)**: 4 inner / pure (kernel, domain,
application, app) + 2 outer / external (adapter, infrastructure) + 6
presentation / entry-point (cli, rest, grpc, graphql, worker, sdk).
Canonical decision tree (§2.2.4) for deterministic per-crate
assignment. `kernel` allowlist for trivial impls (Default/Display/
Hash/const fn/getter) per §15a fix 9.

**Lean check crates (FINAL, 4 — collapsed from iter-1's 11)**:
- `oya-check-architecture` (orchestrator + 7 subcommands:
  layer-correctness + dependency-direction + naming-collision +
  metadata-schema + lockfile-parity + lib-name-parity + check-namespace)
- `oya-check-bounded-contexts` (BC registry + shared/vertical-kind
  cross-vertical refusal + transitive walker + public_layers hop check
  + BC overlap governance with parent/child/sibling rule + Jaro-Winkler
  > 0.85 manual-review trigger)
- `oya-check-supply-chain` (cargo-deny wrapper; pinned JSON schema)
- `oya-check-semver` (cargo-semver-checks wrapper + rename-baseline-reset
  classifier; pinned JSON schema)

**Shared/vertical bisection (FINAL)**: slot 2 = literal `shared`
(formerly v3 `platform/foundation/tooling/core` axes; all collapse to
the single literal) OR single-token registered vertical name. Initial
verticals: `cloud` (owner: council-cloud, `public_layers = ["sdk"]`),
`foundry` (owner: council-foundry, `public_layers = []`), `workspace`
(owner: council-workspace, `public_layers = []`). Cross-vertical
direct + transitive deps REFUSED by LEAN-A2 unless target layer ∈
target vertical's `public_layers` allowlist. `shared → vertical` edges
never qualify for public-layer exemption.

**Cloud dual-role (FINAL)**: `cloud` vertical serves BOTH (a) in-house
compute substrate consumed by other verticals AND (b) external cloud
product sold to customers. Layers `kernel/domain/application/adapter/
infrastructure` = internal substrate; layers `cli/rest/grpc/graphql/sdk`
= customer-facing product surface. `public_layers = ["sdk"]` documents
the initial cross-vertical-callable layer.

**Verticals registry lifecycle (FINAL)**: `status: active | deprecated
| retired`. `active → deprecated` requires ADR amendment + sets
`deprecated_at` (RFC 3339); 180-day soft-deprecate window; no new BCs
may register under a deprecated vertical (LEAN-A2 refuses). `deprecated
→ retired` requires zero crates referencing vertical + 180-day elapsed
+ ADR amendment. Retired entries retained for historical record.

**Reviewer streams (FINAL, 4 partitions; iter-2 prefold-A item 3
honest sizing)**: 1a platform/shared (~28 crates) + 1b cloud (~31) +
1c foundry (~51) + 1d workspace+tooling+hotspots-reviewer-lead (~30 +
hotspot artefacts). 4 partition sign-offs required to merge.
**Reviewer hours**: 8–10 h per primary × 4 streams = 32–40 h calendar
reviewer-hours (was 24–30 h in iter-1; honest re-sync per item 3).

**R10 5-layer parity (FINAL)**: BC drift gets the same 5-layer
permanent-controls ledger as R4 [lib]-name drift: preflight LEAN-A2
xtask + MISTAKES-LEDGER BC-DRIFT-001 + LEAN-A2 BLOCKER lane + ICM
`bc-drift-prevention` topic + `cargo doc` citation probe.

**`oya-check-bounded-context-registry` → `oya-check-bounded-contexts`
rename**: the iter-1-fold-A singular crate name was renamed to plural
during the LEAN-A1–A4 collapse; stale reference at §6 R10 fixed per
Codex iter-2 stray. All §1, §4a, §8.1, §11 ADR-0056 occurrences now
say `oya-check-bounded-contexts`.

**Codex iter-2 ITERATE-7 closures (D1–D7) folded in plan**:
- **D1**: §3 audit row schema rewritten to 3-slot (per Codex iter-2
  edit 1: drop `thing?` column; add `vertical` + `kind` + `layer_evidence`
  + `bc_registry_status` columns; update proposed_names to 3-slot
  `oya-<shared|vertical>-<bc>-<layer>`; replace `rest (provisional)`
  cells with evidence cite OR `PROTOCOL-UNKNOWN` deferral marker).
  *Audit-table cells still carry placeholder values where `src/`-
  inspection is required; the column SCHEMA is rewritten and the
  per-row inspection is the iter-3 surface (§10 question 1).*
- **D2**: §3.6 arithmetic synced — "140 existing crates + 4 new
  check crates = **144** crate-name-affecting ops" matches §1 line
  259.
- **D3**: §3.0 metadata schema all `thing` references purged;
  required keys `vertical` + `bounded_context` + `layer` + `purpose`;
  optional `audit_chain` + `feature`.
- **D4**: §8.1 LEAN-A2 gate row extended with explicit transitive
  walker spec + public_layers per-hop check + FULL-chain violation
  output format (`a → x → y → b` with per-node `{kind, vertical,
  layer}` annotation).
- **D5**: BNF vertical-single-token policy (Option A) locked; ADR-0056
  §"Vertical naming policy" added; `shared` reserved as non-vertical
  literal; verticals registry refuses any entry named `shared` or
  containing a hyphen.
- **D6**: §13 reference inventory adds `docs/standards/code-style-rust.md`
  lines 11-12, 137-147, 162-177 as a Shard 1 co-edit (still declares
  v3 BNF + 9-value role enum); §5.2 step 10b enforces.
- **D7**: this open-questions iter-2 refresh — current entry. Drops
  `<thing>` / `2-slot` references; documents iter-2 state.

**Next gate**: Codex critic iter-3 review against the post-D1–D7
state. Plan stays `pending approval, iteration: 2, consensus_loop:
v4-iter-2-postfold-A`. User execution-approval is a separate gate
downstream of consensus-lock.

**Open items for Codex iter-3 pressure-testing**:
- (1) **§3 audit per-row `src/` inspection completeness**: D1 rewrote
  the COLUMN SCHEMA but per-row evidence cites must be populated by
  the iter-3 inspection pass — every row must carry either a file:line
  cite OR an explicit `PROTOCOL-UNKNOWN` deferral marker; no
  "provisional" values may ship.
- (2) **Transitive walker performance at 140-crate scale**: LEAN-A2's
  recursive `cargo metadata` walk is O(deps²) worst-case; if the
  walker takes > 30 s on the workspace, the CI gate becomes a
  bottleneck. Codex iter-3 may pressure-test cached-result fixturing.
- (3) **`docs/standards/code-style-rust.md` D6 amendment scope**: the
  line ranges (11-12, 137-147, 162-177) need a `src/`-style audit to
  confirm they actually contain the v3 BNF + role enum; the iter-2
  fold trusted Codex iter-2's citation. Iter-3 may pressure-test for
  surrounding-context damage to the doc.

## rename-plan-v4 iter-3 fold — 2026-05-13 (Codex iter-3 ITERATE-5 folded; E1–E5 closed)

Codex iter-3 found that iter-2 postfold-A claims were partially
inaccurate — schema/directive rewrites were not paired with actual
body-row regeneration. Iter-3 fold pairs every directive with its
concrete execution, plus refreshes the open-questions honest-claim
text per edit 6.

| # | Edit | Status | File:line cite (closure) |
|---|---|---|---|
| E1 | §3.1–§3.5 body rows REGENERATED to 11-column 3-slot tuples (was: headers only rewritten at iter-2; body rows still 9-column 2-slot at iter-2 closure) | CLOSED | `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` rows 1-28 (§3.1), 29-59 (§3.2), 60-82 (§3.3.1), 112-137 (§3.4), 138-140 (§3.5). Every row now carries `current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected`. Zero `rest (provisional)` cells remain in audit (every `-api` row now carries `PROTOCOL-UNKNOWN, deferred to ADR-0056 §"Protocol classification"` in `layer_evidence` + `proposed_name`). Layer-inspection rows carry `STUB-pending-iter-4-src-inspection` markers for the iter-4 evidence-population pass. |
| E2 | §3.6 arithmetic — display `140 + 4 = 144` not `139 + 4 = 144`; match §1 line 261 | CLOSED | §3.6 audit-summary table — "Subtotal existing crates renamed: **140**"; "Total crate-name-affecting ops: **140 + 4 new = 144**"; reconciliation prose rewritten to explain that 28+31+22+29+26+3 = 139 visible-row-numbers represent **140 unique crates** (row 111→112 numbering joint, no gap, but row 111 is the last check-namespace foundry crate while rows 112-137 cover all 26 workspace product axes; total crate count exactly matches Cargo.toml 140-row ground truth) |
| E3 | Frontmatter `purpose:` block — rewrite to declare 3-slot grammar | CLOSED | frontmatter `:19-30` — `purpose:` now reads `oya-<shared\|vertical>-<bounded-context>-<layer>` with explicit 3-slot description; "2-slot grammar" language deleted; "<thing> slot considered in earlier drafts is REMOVED" language deleted |
| E4 | Stale §4a A4 heading rename — singular `oya-check-bounded-context-registry` → plural `oya-check-bounded-contexts` matching LEAN-A2 | CLOSED | §4a A4 heading text updated; explicit `[SUPERSEDED → LEAN-A2; renamed iter-2 from singular ... to plural ...]` annotation in heading |
| E5 | `.omc/plans/open-questions.md` honest-claim correction + iter-3 closure section | CLOSED | Current section (this one); the iter-2 postfold-A section above now carries a HONEST-CLAIM-CORRECTION block listing what WAS purged vs NOT at iter-2 vs corrected here at iter-3 |

**Iter-3 fold also performed stale-reference sweep**:
- Frontmatter purpose: ✓ 3-slot
- §3.1–§3.5 body rows: ✓ 11-column 3-slot; no `provisional` cells
- §3.0 metadata schema prose: ✓ `thing` references purged (D3 closure
  retained from iter-2)
- §6 R10 lane row: ✓ plural `oya-check-bounded-contexts` (iter-2 D6
  stray-fix retained)
- §4a A4 heading: ✓ plural (iter-3 E4 fix)
- §3 column directive prose: ✓ 3-slot (iter-2 D1 closure retained)
- `oya-platform-*` axis prefix in production prose: PARTIALLY PURGED.
  Remaining occurrences are in (a) §3.1 audit `current_name` column
  (these are v3 names and MUST stay — they document the rename
  source), (b) §11 ADR-0056 audit-translation rule (explicit historical
  reference), (c) §15a/§15b/§15c closure tables (cite history). All
  surviving references are intentional history; no production-prose
  axis prefix remains.

**STUB markers (Codex iter-4 F3 actual-count sync via direct grep at fold time)**:

**Verification (actual counts confirmed via grep)**:
- `rg -cE "^\| [0-9]+ \|.*STUB-pending-iter-4-src-inspection"
   docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` → **85** body rows
- `rg -cE "^\| [0-9]+ \|.*PROTOCOL-UNKNOWN"
   docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` → **26** body rows
- Total: 85 STUB + 26 PROTOCOL-UNKNOWN = 111 marked rows. The remaining
  29 rows (§3.3.2 rows 83-111) use `check-namespace-exempt` markers
  per F1 closure (new check crates, not src-inspection candidates);
  85 + 26 + 29 = **140 total audit rows** matching Cargo.toml ground
  truth.
- Row 1 `oya-platform-data-boundary-kernel` retains its STUB marker
  in `layer_evidence` (the kernel-preservation cite is in prose; the
  STUB signals "iter-5 src-inspection re-confirms"); 7 `*-adapter-*`
  rows similarly retain STUB markers even though v3 `-adapter-` token
  fixes layer classification.

(Earlier iter-3 estimates of 110 STUB and 22 PROTOCOL-UNKNOWN were
inaccurate; iter-4 F3 corrects to grep-confirmed actuals 85 + 26.)

**Iter-5 pressure-test surfaces** (the remaining open work):
- (1) Per-row `src/`-inspection populating `STUB-pending-iter-4-src-
  inspection` markers with file:line cites (**85 rows** × ~5 minutes
  each = ~7-hour audit window).
- (2) Per-`-api` protocol classification populating `PROTOCOL-UNKNOWN`
  markers with wire-format evidence (**26 rows** × ~10 minutes each =
  ~4.5-hour audit window).
- (3) ADR-0056 §"Protocol classification" sub-section authoring —
  **CLOSED at iter-4 F4**: full sub-section authored with grep
  heuristic mapping table + multi-protocol exception policy + 26-row
  deferred-crate enumeration + Option-Hold-vs-Option-Inline sequencing
  decision criterion. All 26 PROTOCOL-UNKNOWN audit cells now resolve
  to a concrete ADR target.

## rename-plan-v4 iter-4 fold — 2026-05-13 (Codex iter-4 ITERATE-4 folded; F1–F4 closed; APPROVE-ready)

Iter-4 is the iteration cap (5th consensus pass). All 4 Codex iter-4
mechanical edits closed; plan now APPROVE-ready.

| # | Edit | Status | File:line cite (closure) |
|---|---|---|---|
| F1 | §3.3.2 schema carve-out — Option A regeneration to 11-column 3-slot with check-namespace exemption | CLOSED | `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §3.3.2 header + table-header + all 29 body rows (83-111) regenerated. Each row carries 11 columns with `vertical: check-namespace-exempt | bounded_context: check-namespace-exempt | kind: check-namespace-exempt | layer: check-namespace-exempt | layer_evidence: NEW-scaffold-shard-1-from-v3-fitness-crate (rule-name <X>) | proposed_name: oya-check-<X> | bc_registry_status: PROPOSED-NEW`. Inline exemption explanation block prepended (§3.3.2 preamble cites §2.1 BNF "check-crate" production). |
| F2 | Arithmetic mechanical fixes | CLOSED | §3.3 header: `n = 53` → `n = 52` (23 non-check + 29 check); §3.3.1 header: `n = 22` → `n = 23` (rows 60-82 inclusive); §3.6 reconciliation prose: `28+31+22+29+26+3 = 139` → `28+31+23+29+26+3 = 140`. Removed iter-3's "missing row reconciled in iter-2" claim (was an arithmetic error). |
| F3 | STUB/PROTOCOL count actual sync via grep | CLOSED | `.omc/plans/open-questions.md` STUB markers block — 110 → **85** (body-row grep); 22 → **26** (body-row grep); verification commands documented inline; iter-3 estimates explicitly acknowledged as inaccurate. |
| F4 | ADR-0056 outline update | CLOSED | §11 ADR-0056 Decision paragraph: `2-slot grammar` → `3-slot grammar `oya-<shared\|vertical>-<bounded-context>-<layer>``. New §"Protocol classification" sub-section authored with grep heuristic mapping table (`axum::Router`→rest, `tonic::Server::builder`→grpc, `async_graphql::Schema::build`→graphql, `tokio::spawn{loop}` without Router/Server/Schema→worker) + multi-protocol split-vs-exception policy + 26-row PROTOCOL-UNKNOWN deferred-crate enumeration (5 platform + 13 cloud + 4 foundry + 4 workspace = 26 matching grep) + Option-Hold-vs-Option-Inline sequencing decision criterion + Exceptions list (empty at iter-4 close). |

### Iter-4 verification grep output (mechanical confirmation)

Run at iter-4 fold close, from repo root:

```bash
$ grep -cE "^\| [0-9]+ \|.*STUB-pending-iter-4-src-inspection" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
85

$ grep -cE "^\| [0-9]+ \|.*PROTOCOL-UNKNOWN" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
26

$ grep -cE "^\| [0-9]+ \|.*rest \(provisional\)" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
0

$ grep -c "2-slot" docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
16   # all surviving mentions are in historical/superseded sections

$ grep -cE "^\| (8[3-9]|9[0-9]|10[0-9]|11[01]) \|" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
29   # §3.3.2 body rows confirmed regenerated
```

### Iter-4 fold state cross-references

- §3.1 rows 1-28 — 11-col 3-slot (E1 retained)
- §3.2 rows 29-59 — 11-col 3-slot (E1 retained)
- §3.3 header `n = 52` (F2)
- §3.3.1 header `n = 23` (F2); rows 60-82 — 11-col 3-slot (E1 retained)
- §3.3.2 header `n = 29`; rows 83-111 — 11-col 3-slot with check-namespace-exempt markers (F1)
- §3.4 rows 112-137 — 11-col 3-slot (E1 retained)
- §3.5 rows 138-140 — 11-col 3-slot (E1 retained)
- §3.6 audit summary — 28+31+23+29+26+3 = 140 (F2)
- §11 ADR-0056 Decision — 3-slot (F4)
- §11 ADR-0056 §"Protocol classification" — full sub-section authored (F4)
- §11 ADR-0056 §"Vertical naming policy" — D5 retained
- §11 ADR-0056 §"Verticals registry" — prefold-A lifecycle retained
- §11 ADR-0056 §"Cloud vertical dual-role + public_layers" — prefold-A retained

### Approval gate

Plan is now APPROVE-ready. Remaining iter-5 work (per Iter-5 pressure-
test surfaces above) is the `src/`-inspection evidence pass — that
work is **execution-phase audit**, not plan-iteration work, and runs
inside Shard 0 step 5b OR as a pre-Shard-0 audit-only PR. The plan
itself is consensus-locked-equivalent for iter-4.

**Plan transitions**: `pending approval` → `pending approval` (no
status change — user execution-approval is a separate gate downstream
of consensus-lock; iter-4 closes the consensus loop).

Frontmatter additions: `critic_iter_4: ITERATE-4 (folded; F1–F4 per
§15d closure block)`; `consensus_loop: v4-iter-4-fold`.

## rename-plan-v4 iter-5 approve-fold — 2026-05-13 (Codex iter-5 APPROVE-WITH-CONDITIONS; G1–G3 folded; PLAN APPROVED)

Iter-5 is the FINAL fold. Codex iter-5 returned APPROVE-WITH-CONDITIONS
(3 narrow 1-paragraph edits). All folded same-session; v4 flipped to
`status: approved` + `pending: execution-approval-from-user`.

| # | Edit | Status | File:line cite (closure) |
|---|---|---|---|
| G1 | §3.6 summary-table consistency — Foundry non-check 22→23; rows 60/72/73/74 narrative `-api → -rest` replaced with `PROTOCOL-UNKNOWN` deferral matching §3.3.1 body rows | CLOSED | `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §3.6 table row "Foundry non-check (vertical preserved) \| 23 \| 23 ..."; subtotal 28+31+23+29+26+3 = 140 displayed in both table and prose |
| G2 | Active `2-slot` references purge to history-only | CLOSED | 4 active prose references rewritten in §1, §2.1, §4a A5-equivalent collision rule, §11 ADR-0056 Consequences + Pattern E. Final grep `rg -c "2-slot"` = **15** (down from 18; all surviving mentions are in history/closure/superseded sections). |
| G3 | Check-crate name normalization to 4-LEAN design — `oya-check-clean-architecture` → `oya-check-architecture` (matches §4a A1); "11 new check crates" → "4 LEAN check crates" | CLOSED | 7 active-prose references normalized in §2.2.3, §3.3.2 prose note, §5.2 step 15, §6 R4 + R9, §11 ADR-0056 Decision Drivers + Consequences + Follow-ups. Final grep `rg -c "oya-check-clean-architecture"` = **3** (all in history sections); `rg -cE "11 (new )?check crate"` = **1** (line 1332 in historical §4a `### A-summary` block already labeled SUPERSEDED). |

### PLAN APPROVED — final state

- `status: approved` (was `pending approval`)
- `pending: execution-approval-from-user` (separate downstream gate;
  consensus-locked but user execution-approval remains required before
  Shard 0 opens — same convention as v3 used at iter-3 approve-fold)
- `consensus_loop: v4-iter-5-approve-fold`
- `last_modified: 2026-05-13`
- `critic_iter_5: APPROVE-WITH-CONDITIONS (3 conditions, folded; G1–G3
  per §15e closure block)`

**No v5 will be created.** v4 ships as consensus-locked. Iter-5 was the
iteration cap (5 consensus passes total: architect iter-1 + critic
iter-1 + critic iter-2 + critic iter-3 + critic iter-4 + critic iter-5
APPROVE).

### Cumulative review-cycle accounting

- Architect iter-1: 7 conditions CLOSED (§15)
- Codex iter-1: 7 edits CLOSED (§15a)
- Codex iter-2: 7 edits CLOSED (§15b)
- Codex iter-3: 5 edits CLOSED (§15c)
- Codex iter-4: 4 edits CLOSED (§15d)
- Codex iter-5: 3 conditions CLOSED (§15e) → APPROVE

Total: **33 review items folded across 6 review passes**; zero deferred.

### Remaining work (execution-phase audit, not plan iteration)

Per the iter-4 fold "Iter-5 pressure-test surfaces" block above, the
remaining work is **`src/`-inspection audit** (85 STUB rows + 26
PROTOCOL-UNKNOWN rows). That work runs INSIDE Shard 0 step 5b OR as
a pre-Shard-0 audit-only PR. It is NOT plan-iteration work; the plan
itself is now consensus-locked.

### Iter-5 final-state grep verification (executed at iter-5 fold close)

```
$ rg -c "2-slot" docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
15

$ rg -c "oya-check-clean-architecture" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
3

$ rg -cE "11 (new )?check crate" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
1

$ rg -cE "^\| [0-9]+ \|.*STUB-pending-iter-4-src-inspection" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
85

$ rg -cE "^\| [0-9]+ \|.*PROTOCOL-UNKNOWN" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
26

$ rg "Foundry non-check (vertical preserved)" \
    docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
| Foundry non-check (vertical preserved) | 23 | 23 (...) | 0 |
```

All grep checks confirm iter-5 G1+G2+G3 closure.

## rename-plan-v3 post-approval correction 1 — 2026-05-13

- [x] **Trigger**: user feedback "why python script? change to rust?" on the `tools/lockfile-rename.py` choice in v3 §7.1.1.
- [x] **Rationale**: workspace consistency (the workspace is pure Rust); parser reuse (`toml_edit::DocumentMut` is already a dependency of the metadata-augment xtask); eliminate cross-language toolchain pin (no Python 3.11+ runtime requirement on agents/CI); unify the test runner under `cargo nextest` (no second non-Rust test runner introduced for a single script).
- [x] **Sites updated** (8 in `docs/plans/rename-plan-v3-2026-05-12.md`): frontmatter (`post_approval_correction_1` row + `last_modified: 2026-05-13`); §5.1 step 1b (author target → xtask subcommand + `lockfile_rename_fixtures.rs`); §5.1 step 3b (acceptance gate command → `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures`); §5.2 step 12b (invocation → `cargo run --release -p xtask-metadata-augment -- lockfile-rename …`); §7.1 Shard 1 rollback (revert invocation → same `cargo run -- lockfile-rename … --reverse`); §7.1.1 spec header + crate-path block + CLI block + Inputs (added `--dry-run`) + Behaviour (parser → `toml_edit::DocumentMut`) + integration-test-matrix preamble + Shard 0 acceptance-gate sentence; §8.1 lockfile-rename gate row (renamed + Rust invocation); §11 ADR-0055 §"Compound capability audit" (added tooling-rationale one-liner).
- [x] **Re-consensus required**: **NO** — no semantic change to the plan; tool implementation language only (behaviour, CLI flags, 8-row fixture matrix, and invariant are identical to the prior Python spec). Plan remains consensus-locked (`status: approved`, `pending: execution-approval-from-user`).
- [x] **Verification**: `rtk grep -c "lockfile-rename.py\|tools/lockfile-rename" docs/plans/rename-plan-v3-2026-05-12.md` → 0 (no surviving Python script references); `rtk grep -c "pytest" docs/plans/rename-plan-v3-2026-05-12.md` → 0 (all pytest references replaced with `cargo nextest`).

