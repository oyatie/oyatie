# Spec: G011 item 1 — glob workspace membership + Cargo.lock merge driver

Status: ACTIVE (lane spec + team brief) · Story: G011 · Frictions retired: FRIC-007 (Cargo.toml member-list conflict class), FRIC-021 (partial: lockfile merge), FRIC-1781062100-G06 (partial: lock-refresh path)
Authority: ADR-0538 (authored in PR-1) · Verified facts as of dev @ b049a777e, 2026-06-10.

## Objective

Kill the merge-conflict class created by 813 EXPLICIT workspace members in root `Cargo.toml`: every new-crate PR edits the shared members array + `Cargo.lock`, so concurrent lanes conflict 100% by construction (founder hit it on #656). After this lands:

- A new-crate PR touches **zero shared manifest lines** (glob membership).
- `Cargo.lock` concurrent disjoint additions merge structurally (Rust merge driver), not textually.
- A fail-closed gate prevents regression to explicit member paths and orphan crates.

## Verified ground truth (do NOT re-derive; checked 2026-06-10)

- 813 members, exactly 6 shapes: `oya/*/crates/*` (427), `libs/*` (184), `cloud/*/crates/*` (149), `tools/*` (21), `oya/office/*` (19, direct children), `cloud/cloud-ci/gates/*` (13).
- Root `[workspace]` has **no `exclude` today**.
- `cloud/cloud-kernel/` is **its own workspace** (own Cargo.toml `[workspace]` line 10, own Cargo.lock + rust-toolchain); its `crates/*` (7 manifests) must be excluded from root globs.
- `tools/` has 7 non-crate dirs (anchor-sweep, buck, buck2, completions, governance, hooks, opensk-vendored) — must be excluded (cargo errors on glob matches without manifests).
- `cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/` exists with no Cargo.toml — residue; investigate (tracked? delete or exclude).
- `libs/*` (184) and `oya/office/*` are clean: every dir has a manifest.
- Glob-breakable parsers (textual members-array readers), ALL must migrate in PR-1:
  1. `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:1189` `read_cargo_member_prefixes` — member strings become path prefixes for cargo-members reachability; `"libs/*/"` never prefix-matches → silent reachability collapse.
  2. `oya/developer-sdk/crates/oya-dev-cli/src/workspace_manifest.rs` — textual members extraction (errors on empty).
  3. `oya/developer-sdk/crates/oya-dev-cli/src/workspace_topology_gate.rs` — R4 "every members entry resolves to an existing dir with a Cargo.toml" is false for glob entries.
  4. `tools/oya-xtask-metadata-augment-app/src/metadata.rs` — iterates `workspace.members` as literal paths.
  5. `libs/oya-check-dependency-seam/src/lib.rs:152` `read_workspace_members` — verify + migrate if textual.
  - `collect_cargo_prefix` (accounting-registry main.rs:484) is glob-SAFE (walks tracked `**/Cargo.toml`, never reads the members array) — this is the canonical glob-safe enumeration pattern to reuse.
- Gate architecture (mirror `oya-cloud-ci-manifest-hygiene-app`): producer `oya-cloud-ci-accounting-registry-app` does ALL I/O and emits face rows; gate crate is a PURE policy over rows (zero file access); registration = ONE matrix line in `.github/workflows/oya-ci-required.yml` (~line 93). Gate runs as `cargo test --locked -p <crate>` in matrix + buck2 lane runs `buck2 test //cloud/cloud-ci/...`.
- Generated faces (`*.generated.json`): NEVER hand-edit; CI `producer-regen`/materialize scripts own them. New face ⇒ producer code emits it; firewall/control-plane baselines must be justified via ADR citation, not baseline hand-bumps (check `oya-cloud-ci-firewall-app` baseline mechanism during build).
- Next free ADR number: **0538**.
- No git-config bootstrap exists; merge-driver activation is per-clone opt-in. CI never merges with the driver (squash-merge via GitHub) — the driver is local merge-train/rebase convenience, zero merge authority (complies with cli_surface_policy: local bridge only).

## Deliverables

### PR-1 (atomic): glob membership + parser migrations + coverage gate + ADR-0538
1. Root `Cargo.toml`: `members = ["libs/*","tools/*","oya/*/crates/*","oya/office/*","cloud/*/crates/*","cloud/cloud-ci/gates/*"]` + `exclude = [cloud/cloud-kernel(+/crates/* if needed), 7 tools dirs, residue dir disposition]`.
   **Equivalence proof (hard acceptance):** `cargo metadata --format-version 1 --no-deps` workspace member id set BEFORE == AFTER, byte-identical sorted diff, recorded in PR body. `Cargo.lock` refreshed only via sanctioned `cargo metadata >/dev/null`.
2. Migrate the 5 parsers to glob-aware enumeration. Shared approach: expand member patterns against the filesystem/tracked paths (reuse `path_glob_matches` / `collect_cargo_prefix` pattern); honor `exclude`. TDD: each parser gets a failing test with a glob+exclude manifest fixture FIRST.
3. New single-concern gate `cloud/cloud-ci/gates/oya-cloud-ci-workspace-glob-coverage-app` (ADR-0132: flat, single-concern):
   - Producer emits face rows: `{member_entry, is_glob}` per members entry + `{crate_dir, covered, excluded}` per tracked first-party crate-manifest dir (root + third-party + sub-workspaces skipped).
   - Violations (born-blocking): `workspace_member_explicit_path` (non-glob entry — the ratchet), `crate_dir_not_covered` (orphan crate: manifest dir matched by no glob and not excluded).
   - One matrix line + BUCK target + tests (green fixture, RED fixture per violation code).
4. `docs/decisions/ADR-0538-*.md` per template + INDEX/required registries (run the doc gates locally to find required registrations).

### PR-2 (independent lane): Cargo.lock structural merge driver
1. `tools/oya-cargo-lock-merge-driver-app`: Rust binary, git merge-driver protocol (`%O %A %B` = base/ours/theirs paths; result written to %A; exit 0 merged / 1 conflict).
   - Parse all three as TOML `[[package]]` sets keyed by (name) → (version, source, checksum, dependencies, full entry).
   - Compute ours-delta and theirs-delta vs base (added/removed/changed). Disjoint or identical deltas ⇒ apply both to base, emit canonical sorted lockfile preserving header comment + `version = N` field, byte-stable.
   - Same-key divergent deltas ⇒ exit 1 (git records conflict, %A left as ours) — never guess versions.
   - `unwrap/expect/panic`-free production code (ADR-0083 Tier-3), `#![forbid(unsafe_code)]`.
2. `.gitattributes`: `Cargo.lock merge=cargo-lock` (mirrors the existing documented `evidence/audit-chain.jsonl merge=union` block — copy its comment discipline: what/why/trade-off).
3. Activation doc (in the crate README + ADR): `git config merge.cargo-lock.name ... / .driver "<buck2-built binary> %O %A %B"`; unregistered clones get default git behavior (safe degradation).
4. TDD with fixture lockfiles: disjoint-add (green), same-package-version-divergence (conflict), removal-vs-edit (conflict), idempotence (ours==theirs), header/version preservation.

## Commands (canonical verification preamble — AMENDMENT 3)

- Build: `buck2 build <affected targets>` · Test: `buck2 test <affected targets>` (BUCK + reindeer wiring = part of done; cargo supplementary only).
- Lock refresh (sanctioned, the ONLY cargo write path): `cargo metadata >/dev/null`.
- Gate dry-run: `cargo test --locked -p <gate-crate>` (mirrors the CI matrix leg).
- Known pre-existing local RED (NOT yours): firewall / slo-coverage / registry-drift / generated-artifact-control-plane buck2 gate tests fail on stale local faces (FRIC-009). Compare against dev baseline before attributing.

## Structure / style

- Gate crate mirrors `oya-cloud-ci-manifest-hygiene-app` exactly: `src/lib.rs` pure policy + `GATE_ID` + `VIOLATION_CODES`, `tests/`, `BUCK`, doc-comment contract block. Naming: `oya-` prefix mandatory (cargo-prefix gate enforces).
- Workspace manifest hygiene applies to new crates: version/rust-version/lints workspace-inherited, `publish = false`, license, `doctest = false` when `[lib]` (manifest-hygiene gate enforces).
- Commits: SSH-signed, conventional style matching recent dev history.

## Boundaries

- Always: isolated worktree per lane (NEVER touch the main checkout — FRIC-022/-019); PR to dev; oya-ci-required green on rebased head; adversarial review APPROVE before merge; buck2-first verification.
- Ask founder first: anything touching merge authority, branch protection, required contexts, or new CLI surfaces beyond the merge driver (which is sanctioned local-bridge tooling here).
- Never: hand-edit `*.generated.json`; hand-edit Cargo.lock (driver fixtures are test data, exempt); `cargo build/check/test/fmt` as verification authority; recreate explicit member paths; weaken an existing gate to make this pass; run omc orphan-cleanup (destructive).

## Success criteria

1. `cargo metadata` member set provably identical pre/post conversion (diff in PR body).
2. All 5 parser call-sites glob-correct with new tests; no gate behavior change on dev tree (reachability set identical — assert in test).
3. Coverage gate: green on converted tree; RED fixtures prove both violation codes fire; explicit-path regression is impossible-to-ship.
4. Merge driver: all fixture cases pass; binary built by buck2; .gitattributes documented; no CI dependency on driver registration.
5. oya-ci-required 16/16 green on both PR heads (rebased per merge train); adversarial review APPROVE in code, not narration.
6. Friction ledger rows FRIC-007/021/1781062100 statuses updated with PR links; checkpoint + INDEX refreshed.

## Open questions (resolve during build, escalate only if blocked)

- Firewall baseline: how a NEW gate-id gets justified (read oya-cloud-ci-firewall-app contract before wiring; FRIC-009 showed +1 unjustified key = RED).
- dev-cli fd001_manifest_workspace_alignment_cli test semantics under globs — align test intent (alignment still provable via expansion).

## AMENDMENT A (2026-06-10, supersedes conflicting text above) — PR-1 lane is RESUMED, not greenfield

The worktree `/Users/jasonlee/oyatie-worktrees/g11-glob-members` (branch `agent/g11-glob-members`, base = dev tip b049a777e) holds STAGED, uncommitted prior-session work. EXTEND it; do not redo or revert it. Verified staged state:

- **Glob design (adopt this, not the 6-pattern draft above):** `members = ["libs/oya-*", "cloud/*/crates/oya-*", "cloud/cloud-ci/gates/*", "oya/*/crates/oya-*", "oya/office/oya-*", "tools/oya-*"]` — crate leaf narrowed to `oya-*` (cargo-prefix gate guarantees first-party naming), so non-crate sibling dirs (tools/buck, completions, vendored trees) can never break resolution and need NO exclude. `exclude = ["cloud/cloud-kernel" (separate no_std workspace), "cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app" (buck2-only gate, no Cargo.toml BY DESIGN — not residue)]`.
- **Canonical resolver exists:** `libs/oya-workspace-members-kernel` (`resolve_member_dirs(repo_root)`, 4 unit tests) — ALL parser migrations call this kernel; never re-implement glob expansion (reuse, not re-derive). Friction id is **FRIC-1781069288** (founder manually resolved #656's lock conflict; structural elimination ordered).
- **Already migrated (staged):** accounting-registry `read_cargo_member_prefixes` → kernel; `libs/oya-check-dependency-seam`; BUCK/Cargo.toml wiring for both; regenerated faces staged (accounting-registry/scm-facts/gate-baseline/decision-crosswalk — produced by the producer, NOT hand-edited; verify freshness by re-running the producer, never edit).
- **Remaining for PR-1:** (a) migrate `oya-dev-cli` `workspace_manifest.rs` + `workspace_topology_gate.rs` (R4 phantom-members must validate via kernel expansion) + `tools/oya-xtask-metadata-augment-app/src/metadata.rs` to the kernel; (b) member-set equivalence proof vs dev (cargo metadata before/after, diff in PR body); (c) `oya-cloud-ci-workspace-glob-coverage-app` gate per Deliverable 3 (violation codes: explicit-path regression + orphan crate dir not covered/excluded) + matrix line; (d) ADR-0538; (e) buck2 build/test affected + kernel crate; (f) signed commit(s), push, PR.
- PR-2 lane is unchanged (greenfield in `/Users/jasonlee/oyatie-worktrees/g011-lock-merge-driver`, branch `agent/g011-lock-merge-driver`). Structural 3-way TOML merge is the chosen design (hermetic, no cargo spawn); FRIC-1781069288's "runs cargo metadata" wording is a permitted alternative, not a requirement.
