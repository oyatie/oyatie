# G039 diagnosis — PR #1558 three red gates

**Diagnosed:** 2026-08-05  
**Base:** `origin/dev@a4a5ace5fcba343ee979f7f1d4fa885ca41b9ff0`  
**PR head:** `54b22d0c6470d8008012542eb37d0ff32b72e1b5`  
**Worktree:** `/Users/jasonlee/Developer/oyatie-g039-1558-diag` (branch `agent/g039-1558-diag-20260805`)  
**CI evidence:** run `30977703798` (exact-head failures), logs extracted under goal scratch  

**Hard stop:** Keep draft; do not merge current shape.

---

## Diff under diagnosis

| Path | Change |
|------|--------|
| `infra/arc/tests/ci_workspace_capacity.rs` | +88 (test) |
| `infra/gitops/local-path-storage.yaml` | +177 (**new YAML**) |
| `infra/talos/qemu-cilium.patch.yaml` | +8 (**new YAML**) |

---

## Root cause synthesis (three lanes, two causes)

All three failed CI jobs share **two independent registration failures** on the **new YAML files only** (not the Rust test file under `infra/arc/`, which already has `infra/arc/OWNERS`).

### Cause A — ADR-0555 accounting registration (firewall / total-accounting)

**Symptom (firewall log):**

```text
firewall_is_green_on_the_live_corpus_with_the_baseline ... FAILED
GO-LIVE: ... codes FAIL: ["unjustified", "unowned", "unreachable"]
[cloud-ci-total-accounting] unjustified regressions
  {"infra/gitops/local-path-storage.yaml", "infra/talos/qemu-cilium.patch.yaml"}
[cloud-ci-total-accounting] unowned regressions
  {same two paths}
[cloud-ci-total-accounting] unreachable regressions
  {same two paths}
```

**Meaning:**

| Code | Gate demand |
|------|-------------|
| **unowned** | Add OWNERS (or directory OWNERS) for the artifact |
| **unreachable** | Place under a workspace-member / buck package that reaches the path |
| **unjustified** | Registry must justify the path once reached (derived from reachability) |

Gate-registration meta-tests **passed** (23 ok) — this is **not** a missing gate crate; it is **new file debt vs baseline**.

### Cause B — corpus-index-coverage unpackaged YAML ceiling

**Symptom (buck2 log):**

```text
corpus_index_unpackaged_regression
detail: "450 YAML files belong to no buck2 package, above the frozen ceiling of 448.
         New YAML must land inside a buck2 package so it is a build-graph input."
the_frozen_ceilings_equal_todays_counts: left=450 right=448
```

**Meaning:** The two new YAML files are **outside every Buck2 package**, increasing unpackaged YAML from **448 → 450**. Policy freezes `baseline_unpackaged_yaml_files: 448` (shrink-only). New YAML must be **in-graph** (package + extraction face), not merely present on disk.

Related tests also fail when live evaluate fails:  
`live_corpus_is_within_the_frozen_ceiling`, `the_frozen_ceilings_equal_todays_counts`, plus dependent fixtures `an_attribution_collapse_fails_the_live_policy`, `a_new_indexed_package_passes_the_ratchet`.

### Why affected-set is red (second-order)

Affected-set reports:

```text
Fail: root//ci/facade/baseline-ratchet:ci-baseline-ratchet-gate
Fail: root//ci/facade/corpus-index-coverage:ci-corpus-index-coverage-gate
affected-set: test-health — head test failures=2, baseline failures=0, regressions=2
```

So **affected-set is not a third independent product bug**; it **re-runs** firewall/baseline-ratchet + corpus-index on the affected set and fails because Causes A and B already fail.

| CI job | Independent? | Root |
|--------|--------------|------|
| cloud-ci-firewall | Yes | Cause A (unowned/unreachable/unjustified on 2 YAMLs) |
| buck2 | Yes | Cause B (+ re-runs firewall among affected tests) |
| gate · affected-set | Mostly second-order | A + B via test-health regressions |

---

## What is *not* the problem

- Not `oya-ci-required` fan-in wiring of new gate crates (registration tests green)
- Not secret leaks in the three files (handoff secret scan clean)
- Not the arc capacity test file itself for ownership (covered by `infra/arc/OWNERS`) — though it still must pass unit tests
- Not activation of live CAS/RE/cluster (PR stays declarative-only)

---

## Fix direction for G003 (do not implement until assigned)

1. **OWNERS**  
   - Add or extend OWNERS for `infra/gitops/` and `infra/talos/` (or nearest directory that matches ADR-0555 patterns used by sibling patches).  
   - Mirror style of `infra/talos/local/patches/OWNERS`, `infra/arc/OWNERS`.

2. **Reachability / Buck package**  
   - Ensure both YAML paths are **owned by a Buck package** so they are not “unpackaged”.  
   - Prefer pulling them into an existing `infra/**` package with a `corpus-yaml-facts` extraction face rather than raising the unpackaged ceiling (ceiling raise is anti-northstar).

3. **Do not** only bump `baseline_unpackaged_yaml_files` to 450 without packaging — that pays debt in the wrong direction unless explicitly waived by policy owners.

4. Re-run locally (on this worktree after fixes):

   ```bash
   # representative — exact targets may vary
   buck2 test //ci/facade/corpus-index-coverage:ci-corpus-index-coverage-gate
   buck2 test //ci/facade/baseline-ratchet:ci-baseline-ratchet-gate
   # or the firewall package target used in CI
   ```

5. Keep PR **draft** until exact-head `oya-ci-required` green + independent review.

---

## Evidence pointers

| Artifact | Location |
|----------|----------|
| CI run | https://github.com/jason931225/oyatie/actions/runs/30977703798 |
| Extracted logs | goal scratch `extract-{firewall,affected,buck2}.txt` |
| Policy | `ci/facade/corpus-index-coverage/corpus-index-coverage-policy.json` |
| Handoff inherit | `.grok/programs/cas-fabric/INHERIT.md` |

---

## Status

| Item | State |
|------|--------|
| G002 diagnose | **Complete** (this report) |
| G003 fix | Pending — not started |
| Merge | **Hard STOP** |
| Ultragoal aggregate | Still active/checkpointing — not completed |
