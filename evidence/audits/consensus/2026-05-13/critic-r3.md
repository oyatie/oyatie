# Critic r3 — Wave 5 v13

## Verdict
ITERATE

## Session
n/a

## Fixes (only if ITERATE/REJECT)
1. §3 line 47 / §8 lines 385-388: Required §3 "Viable Options" is absent; alternatives are buried in ADR notes and are one-line rejections, so fair-options review cannot pass. Add a §3 Viable Options table with Option α/β/γ/δ, strengths, costs, risks, and which principles each satisfies.
2. §8 line 383 / §6(c) line 308 / §6(e) line 352: Artifact/lane accounting is inconsistent: line 383 says 48 artifacts + 5 lanes, while v13 elsewhere says 49 artifacts + 7 lanes; line 308 gates unsplit `lean-a-statelessness`/`lean-a-shardability` instead of smoke/nightly lanes. Normalize all gate text to the v13 49-artifact / 7-lane split and name exactly which 5 lanes are blocking vs 2 nightly.
3. §5 line 237: Cedar red-team row still says "ALL 11 NEW FRAGMENTS" even though Wave 5 owns 12 fragments and 13 probes. Replace with a 12-fragment / 13-probe map, including separate vote and execute probes for `ops-multi-party-approval.cedar`.
4. §5 line 242: Multi-party approval probe asserts `resource.approval_count >= required_approvals`, but §4 lines 194-196 define Rule 2 as `resource.state == Approved && execute_operation_id != null`. Align the test fixture with the actual Cedar rule or change the rule; current PASS criteria can test the wrong predicate.
5. §12 lines 494-541 and 569: Section premise and capability rows still use ✅ delivered-style statuses for prerequisite-gated capabilities, contradicting line 578's "PLANNED ... NOT yet implemented in HEAD." Relabel the row status vocabulary to "planned/prerequisite-gated", "partial", "not delivered", or add a column separating planned capability from delivered state.
6. §14 lines 669-671: Consensus stop condition says the live gap list is reduced to ≤5, but §14.1 enumerates 20 entries, including many live gaps. Either reduce §14.1 to ≤5 with named blocker/owner dispositions, or mark S3 not achieved.
7. §14 lines 685-687: Stale duplicate v11 text remains and restates a stronger "CURRENT STATE at Wave 5 close is hyperscaler-mature" claim. Delete it; it directly violates honest-introspection and stale-hashtext/stale-version cleanup.
8. §11 line 472: V14 says HEAD workflow contract is `contracts/workflow.openapi.yaml`, but `git ls-files -- contracts/workflow.openapi.yaml` returned no tracked path. Either add the tracked contract path before relying on it, or state that it exists only in the worktree/not HEAD and keep it prerequisite-gated without "HEAD" wording.

## Quality-Criteria Check
- Principle↔option consistency:    FAIL — Principle 8 requires CI-proven smoke/nightly lanes, but §8 line 383 and §6(c) line 308 still describe 5 unsplit lanes; no §3 options table shows an option satisfying all principles.
- Fair alternatives in §3:         FAIL — §3 is BC inventory, not viable options; alternatives only appear as terse rejections in §8 lines 385-388.
- Risk-mitigation clarity (§4):    PASS — All 8 scenarios name prevention and recovery lanes; scenarios 1-8 have concrete lanes/tests/audit rows, though downstream test text has inconsistencies listed above.
- Acceptance-criteria testability: FAIL — §5 line 242 tests a stale multi-party predicate; §5 line 243 uses stale unsplit lane names and PR-time/deep-load wording inconsistent with §4.
- Verification concreteness:       FAIL — Many gates name commands/fixtures, but cited workflow/test artifacts are absent from HEAD and some gates are not concrete tracked files (`.github/workflows/ci-governance-lanes.yml`, k6 script, Cedar probes).

## User-Mandated-Rule Check
- (i)   honest-claims:           FAIL — §12 still uses ✅ delivered-style row statuses for prerequisite-gated capabilities, despite §12.9's planned-only reframe.
- (ii)  Linus-grade:             FAIL — Public contract regression gate is not enforceable yet: `registry/quality/lanes.yaml` shows `lean-a10-regression` as `status: planned`, and cited OpenAPI contract paths are not tracked in HEAD.
- (iii) verified-claims:         FAIL — `git ls-files` verified only `.omc/plans/ralplan-docs-portal-2026-05-13.md`, `.omc/plans/ralplan-ops-portal-2026-05-13.md`, `docs/CONSTITUTION.md`, `docs/DOC-CATALOG.md`, `docs/MASTERPLAN.md`, and `registry/quality/lanes.yaml` from the checked set. Missing from HEAD: current Wave 5 plan, Wave 2-4 companion plans, `contracts/workflow.openapi.yaml`, `contracts/ops-workspace-shell.openapi.yaml`, tenant/user/deployments/docs OpenAPI contracts, all checked Cedar fragments, `.github/workflows/ci-governance-lanes.yml`, and all checked Wave 5 verification tests/docs/crates.
- (iv)  honest-introspection:    FAIL — §14 has v13-aware entries, but line 669 claims consensus-stage stop can require ≤5 live gaps while §14.1 lists 20, and lines 685-687 retain stale v11 text.

## Notes
`icm recall-context` failed with `failed to open database`; this review used the plan plus current `git ls-files` evidence.
