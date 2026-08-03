# Strangler-move playbook (capability-first reorg, ADR-0562)

Leader-state coordination memory (NOT a repo governance surface). The proven, repeatable
per-capability move cycle. Founder directive: **"Auto-drive, leaf-first"** = run moves
SERIALLY (one PR at a time), cleanest leaf substrate first, no per-move pause for approval.

## Why serial (do NOT parallelize moves)
Each move mutates SHARED MUTABLE STATE that would collide across concurrent moves:
- `specs/capability-registry.json` (absorbs_current_dirs)
- `ci/facade/module-membership/capability-membership-policy.json` (scan_roots + allowed_top_level_dirs)
- `ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-baseline.json` (crate_root_globs + unclassified_roots)

> **PATH CORRECTION 2026-07-31.** The two gate policies above were cited at
> `cloud/cloud-ci/gates/oya-cloud-ci-{capability-membership,tier-dependency-acyclicity}-app/…`
> until this edit. Those paths **no longer exist** — the gate fleet moved to `ci/facade/<gate>/`.
> Verified with `git cat-file -e origin/dev:<path>`: both old paths MISSING, both new paths EXIST.
> An agent following steps (c)/(d) literally would have created a file at the OLD path and
> silently reintroduced the deprecated `cloud/cloud-ci/gates/` shape while the real gate kept
> reading the real policy — a move that passes locally and enforces nothing.
> This playbook is `.omc` leader-state, not a repo governance surface, so **no gate covers it**
> and nothing detected the rot for three weeks. That is the staleness-needs-a-detector class.
- the two frozen lint baselines (membership 62-unmapped; acyclicity 12-violation)
One move at a time. Start the next only AFTER the prior move's post-merge dev push-tier verify is GREEN.

## Burn-down ledger (update after each move)
- Membership unmapped baseline: 62 (frozen at b458e7cf2). messaging move: eventing crates were already-mapped (not in baseline) → 0 burn-down, 0 regression.
- Acyclicity violation baseline: 12 (frozen at b458e7cf2). messaging move: 0 burn-down, 0 regression.
- Lints flip from advisory-baseline → blocking when each baseline reaches 0.

## Strangler MODE: crate-first incremental (DECIDED, #736-review-confirmed)
A move relocates CRATES only (the codemod is a crate-mover). Non-crate artifacts (docs/slos/contracts/GitOps-manifests/tofu) stay in the old dir and are homed in a tracked PHASE-2 (task #62). This is SAFE+correct: #736 review proved RED-set delta = 0-new/24-removed — left-behind artifacts were ALREADY orphaned (owner:null/RED) at merge-base, so a crate-first move strictly IMPROVES state, never regresses. A capability is "crate-homed" per move, "fully homed" after phase-2. Cross-tree refs (e.g. moved app test include_str! to old-dir data) work under buck (cargo is hook-blocked repo-wide). De-brand residue in moved crates ([[bin]] names, OYA_* constants, stale crate_root/mapped_srcs literals) is DEFERRED to the ADR-0532/0533 de-brand profile lane + codemod hardening task #63 — NOT fixed per-move (gate-green, non-corrupting). Founder directive: SERIAL, one capability per PR, no batching ("leaf-first", "serially without pausing per move").

## Leaf order (cleanest-first; AVOID violation sources early)
Violation sources to defer (in the 12-violation baseline): cloud-kms, cloud-network(residency), cloud-billing/saas-bench, oya-intelligence, oya-community, oya-application.
Clean leaves by size: messaging=2 ✅DONE(#735) → iac=5 ✅DONE(#736) → tenancy(~17) → audit(~18) → larger. NEXT: re-scout the smallest remaining clean leaf in-worktree off the new dev tip before each move.
Confirm per-move via in-worktree scout (registry entry + crate dirs + not-a-violation-source).

## Per-move cycle (each step proven on #735)

### 1. Worktree
`git -C <canonical> worktree add /Users/jasonlee/oyatie-worktrees/p<N>-<cap> -b agent/p<N>-<cap> origin/dev`
(worktree add fetches; never mutates the canonical working tree/HEAD. NEVER git-mutate canonical.)

### 2. Executor brief (model=opus, background). Generalize the #735 brief:
- Home <cap>: move `<old crate dirs>` → `<cap>/{core,ports,adapters,facade}/<...>` via `tools/oya-reorg-codemod-app` (NOT hand-moved). cargo pkg renames = path-tail (e.g. `iac-core-domain`). Rewrite ALL dependents mechanically.
- HARD GATE (#54 pattern): codemod full-tree buck2 dry-run → cargo_ok=true AND buck_ok=true AND clean=true; FAIL-CLOSED if buck_ok null.
- Repo-contract interactions to resolve at the strangler/registry layer (NOT hand-patching moved crates):
  (a) workspace members are GLOB-only (ADR-0538) — replace codemod literal members with `<cap>/*/*` (verify the glob matches every moved crate dir, over-matches nothing).
  (b) registry: `<cap>.absorbs_current_dirs` old-dir → `<cap>`.
  (c) membership policy: add `<cap>` to scan_roots + allowed_top_level_dirs.
  (d) acyclicity policy: add `<cap>/*/*` to crate_root_globs + `<cap>` to unclassified_roots.
  (e) born-accounting (ADR-0555): new paths need OWNERS + ADR-0562 §10.x justification + reachability-registry seed.
- VERIFY: buck2 build+test moved crates + dependents; full gate suite vs merge-base via `infra/ci/materialize-cloud-ci-generated-faces.sh .` then `buck2 test //cloud/cloud-ci/...` (firewall merge-base ratchet is AUTHORITATIVE — running individual gates is a false-green class); membership/acyclicity 0 regression; grep-clean of old tokens; cargo metadata --locked clean; Cargo.lock synced.
- Conflict-marker sweep before commit. Commit trailer: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. Push (NOT merge). Worktree only.

### 3. Independent review (fresh-context opus, background, pinned to PR head)
Adversarial (DISPROVE). Probes: move complete + grep-clean of old tokens ANYWHERE; hard-gate buck_ok genuinely true (not null-as-pass); full gate suite green vs merge-base (firewall GO + lints 0-regression); the (a)-(e) contract interactions each resolved; born-accounting covers every new path; codemod touched only declared scope; reversibility.
HYGIENE (task #56): reviewers MUST use `gh pr diff`/`gh api` (read-only) or a DETACHED throwaway worktree (`worktree add --detach /tmp/...` then `worktree remove`); NEVER git-checkout/stash in canonical.
**If the move bundles ANY codemod source change, review that delta SEPARATELY** (it drives all future moves; #735 caught a silent BUCK-corruption bug this way). Bound: 3 doubt cycles, then escalate.

### 4. Governance merge (only when review APPROVE + oya-ci-required green)
- Content-assert: re-verify origin/dev tip == PR base (no rebase when based on current tip) AND head == reviewed SHA.
- `gh pr merge <N> --squash --subject "..." --body "..."`.
- Signature check: `gh api repos/jason931225/oyatie/commits/<merge_sha> --jq .commit.verification` → verified=true (web-flow GPG).
- Append row to `.omc/ultragoal/dispatch-ledger.jsonl` {pr,merge_commit,ci,review,signature,notes}.
- Post-merge dev push-tier verify: poll the push-event `check-substrates` + `oya-ci-required` runs on the new dev tip until GREEN (false-green discipline — push-tier can differ from PR-tier; ref gate-baseline-pr-push-asymmetry memory). Only then start the next leaf.

## Codemod (tools/oya-reorg-codemod-app) — proven correct as of #735
buck.rs hardened across 3 review cycles: rust_test name/crate rewrite; B1 -bin-sibling clobber fixed via
longest-crate-prefix + field-key anchoring + stanza-head boundary. KNOWN latent (task #61, NOT strangler-relevant
today): hyphen-free single-token cargo name skips snake exact pass — all real destinations are multi-segment.

## Rename-aware baseline engine — LIVE as of #737 (ADR-0563, task #64). REQUIRED move-protocol step.
Path-keyed FROZEN CI baselines (brand-residue forbidden_*, total-accounting, target-parity, tier-dep edges) used to
read a RELOCATED already-accepted entry as NEW debt (blocked move-3). Now the scm-facts emitter RELABELS the frozen
merge-base baseline old->new from the codemod's committed move-plan->manifest, guarded per-(gate,code) by
P1(old is frozen key)+P2(old absent from candidate)+P3(new present)+P4(new content-occurrence ⊆ old via census SSOT),
fail-closed+injective; firewall stays byte-unchanged. No manual signoff door.
**THEREFORE every move PR MUST commit exactly ONE move-plan at `specs/reorg/<capability>-move-plan.json`** (the
codemod's MovePlan, the (old_dir->new_dir) bijection). The manifest regenerates from it (registry-drift byte-bound);
without a committed plan the manifest is empty and NO relabel happens (a move that relocates a baselined entry would
then false-RED). #65: hard-error on >1 committed plan (currently silent first-wins). This also de-risks the
violation-source capability moves (kms/network/billing/intelligence/community/application): their baselined acyclicity
edges relabel on move instead of false-REDing.
**EXTENDED #745 (ADR-0563 §C2): per-FILE total-accounting relabel.** The original relabel handled total-accounting via
crate-DIR pairs only; its codes `unjustified`/`unowned`/`unreachable` are keyed per-FILE. `unowned` re-derives via OWNERS
and `unreachable` via the reachability-registry, but `unjustified` has NO re-derivation seed — so relocating
accepted-`unjustified` files (first hit by marketplace's dev-cli, 151 files) false-RED'd until #745 added
`relabel_existence_only_file_gate` (relabels per-FILE via the manifest file_pairs; P1+P2-exact+P3-exact+injective; NO
content guard — sound because `unjustified` is registry-row-derived keyed by path, not content; anti-laundering is
load-bearing on the move-manifest registry-drift binding = mirror-suffix pairs from wholesale git-mv, byte-bound). 3-skeptic
adversarially verified non-false-green. LESSON: any move relocating files that carry accepted per-FILE total-accounting
debt (esp. `unjustified`) now relabels cleanly — no per-move action, the engine handles it.

## EXTRA contract surfaces (conditional — add when the move's crates trigger them)
- **(b) self-slug in absorbs_current_dirs (flags #744 lesson):** the capability's `absorbs_current_dirs` MUST include the
  capability's OWN top-level slug (e.g. `flags`) in addition to the old source dirs, or the membership gate REDs
  `MEM-NEW-UNMAPPED-CRATE <cap>/...`. Pre-seeded registry entries may omit it — always verify.
- **include-site crates (`include_str!`/`include_bytes!`; marketplace dev-cli #743/#745 lesson):** if any moved crate has
  embedded-asset include sites, TWO extra frozen surfaces need the move (NOT covered by the rename-aware engine — same
  manual-per-move class as the membership/acyclicity baselines): (1) `embedded-asset-hermeticity-policy.json` scan_roots
  += `<cap>` (the include sites leave the scanned corpus → site-floor RED otherwise); (2)
  `embedded-asset-hermeticity-baseline.json` skip-set keys RELABEL old->new. Follow-up candidate: fold the hermeticity
  baseline into the rename-aware engine too (after marketplace lands; don't parallelize — it touches scm-facts-emitter).

## NO FACE MATERIALIZATION IN A MOVE (revised 2026-07-09 — supersedes the old MATERIALIZE-LAST step)
A move is a PURE STRUCTURAL rename+rewire. DO NOT run a face materializer and DO NOT commit any `*.generated.json`
face. The old "materialize-last + commit the faces-only diff" step CAUSED a corruption incident: a local materializer
was mis-invoked with partial inputs and committed a corrupt masterplan projection (adr_count 106->64) that nearly
landed. Every pure-derivation face is now DE-COMMITTED and materialized by CI on demand:
- accounting-registry / decision-crosswalk / enforcement-inventory / enforcement-liveness / ttl-policy (ADR-0595)
- scm-facts.generated.json (ADR-0597)
- gate-baseline.generated.json is a FROZEN reference read via `git show <merge-base>:...` — never re-materialized in a PR (ADR-0596)
- masterplan.generated.json + product-graph.html (ADR-0613, PR #1222)
The ONLY committed face a move updates is `specs/reorg/move-manifest.generated.json` (RETAINED committed per ADR-0563 —
the reviewed move-bijection). The codemod regenerates it DETERMINISTICALLY as part of `apply`/`manifest`; it is NOT a
separately-invoked materializer and cannot be mis-invoked the way the projection materializer was. VERIFY: `registry-drift`
`committed_move_manifest_equals_regenerated` matches (the codemod is the sole generator; a hand-forged row is RED). Rely on
CI freshness (regenerate-twice determinism canary) for the de-committed faces — do NOT try to reproduce it locally.
NOTE: the CI tree relocated in #1216 (cloud/cloud-ci/... -> ci/facade/...); the firewall ratchet target is now
`buck2 test //ci/facade/...` and the materializer is `//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`.
The invariant: after a move, `git status` shows ONLY renames/rewires + move-manifest + registry/catalog + Cargo.lock —
ZERO `*.generated.json` projection/accounting faces.

## Open productization tasks from the messaging move
#58 gate name-correctness enforcement (target-parity is blind to rust_test name correctness);
#59 OWNERS husk-misclassification (accounting husk-detector must KEEP OWNERS markers);
#60 registry-graph drift (architecture-map/registry-store stale crate keys — codemod-owned vs reconciled?);
#61 codemod single-token round-trip edge.
