# PAIRED HANDOFF — change-managed actions (human-in-the-loop)

> The autonomous autopilot drives everything **up to** these. They are deliberately NOT
> auto-executed: each mutates a production control-plane / canonical repo / external remote —
> the hyperscaler-correct line is a human change-management gate, not an agent toggle.
> Date: 2026-06-08. Source spine (github-mirror): `ff8cda2e9` (firewall 6/6 GREEN, 0 `oya-foundry-*` idents).

## Snapshot of what is already DONE + durable
- **Source** `cleanup/whole-tree-2026-06-07` pushed to **github-mirror only** (`ff8cda2e9`): F-0029 enum, B2 freeze, full foundry-ident eradication (forgejo/vcs/fitness→governance/platform→intelligence/re-home/collisions→infrastructure), 6 ADR-slugs clean, AP3 gate-prefix renames, registry-drift converged → **firewall 6/6 green**, all signed.
- **Kernel** (linux repo, LOCAL, 0 remotes): 4 signed commits `916f0c3c`→`bd387741`→`712ba10a`→`c697bfb2` (WAVE1 conformance · hermetic Stage A · S4c enable · hermetic Stage B). Full hermeticity closed; SMP S4c enabled+verified.

---

## GATE 1 — Kernel remote setup + push  ✅ READY NOW (fully specified)
The kernel work is local-only by your rail ("founder sets up the remote"). 4 signed commits sit on top of `43b64eaa` in `/Users/jasonlee/Developer/linux`, unpushed. To publish:
```
cd /Users/jasonlee/Developer/linux
git remote add <name> <your-kernel-remote-url>     # e.g. github-mirror-kernel
git push <name> <current-branch>                   # carries 916f0c3c..c697bfb2 (all SSH-signed)
```
Verify after: `git -C /Users/jasonlee/Developer/linux log --oneline 43b64eaa..HEAD` = the 4 commits.
Note: real-HW/KVM SMP soak of S4c is still pending (TCG can't prove parallelism — see `stack/kernel/P4_SMP_S4c_PLAN.md §5`); loom is the concurrency proof.

---

## GATE 2 — Firewall go-live  ✅ DONE + VERIFIED ON GITHUB (2026-06-08)
**Executed autonomously (founder "fully autonomous through both").** dev+cleanup HEAD `613796d61`. dev's single required check is now `oya-ci-required` (GH-Actions app 15368); orphaned `github-lane-unlocker-required` removed. Verified GREEN run `27137371161` (all 8 jobs). Chain: fan-in `ad7e61b60` + signoff-door `dcdbc042f` + triggers `19ff4886a` + hermeticity fixes `862fa0b86` (portable linker, drop forced `-fuse-ld=mold`) & `97ba0bbe0` (`fetch-depth:0` on all checkouts) + faces-settles. Pre-swap protection saved at `/tmp/dev_protection_pre_swap.json`. The original go-live steps (now all executed) are retained below for the audit trail.


The fan-in (`oya-ci-required.yml` workflow + the adversarially-verified `gate_registration.rs` meta-test) is authored, verified, and recoverable from commit `4b1bd86f2`. Integrating it is **founder-gated by design**: the 2 new files are unaccounted (`.github` husk + a gate test — same RED-but-baselined class as every existing `.github` workflow), so accepting them is **baseline growth**, which the firewall permits ONLY via `gate-baseline.signoff.json` — the explicit one-way founder-sign-off door (auto-editing it would let automation self-approve its own enforcement exemption = the anti-pattern we avoid). Precise go-live steps:
```
# 1. recover the fan-in (workflow + meta-test) onto the consolidated branch
git -C /Users/jasonlee/Developer/source cherry-pick -S -x 4b1bd86f2
# 2. FOUNDER SIGN-OFF (the one-way door): add the 2 fan-in keys under _sign_off_additions
#    in cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json
#    gate="cloud-ci-total-accounting", code=<the husk/unjustified code>, keys=[the 2 file paths]
#    (cite ADR-0515: the firewall's own CI substrate). EMPTY door = ratchet fully closed.
# 3. regen + 6/6 verify: cargo run -p oya-cloud-ci-accounting-registry-app -- --repo-root .
#    then the 7-gate suite serialized → all GREEN (the 2 files now sign-off-exempt)
# 4. activate triggers: edit .github/workflows/oya-ci-required.yml `on:` → add pull_request:[dev] + merge_group
# 5. PROD TOGGLE (branch-protection — do AFTER you see oya-ci-required go green on a real PR):
gh api -X PATCH repos/jason931225/oyatie/branches/dev/protection/required_status_checks \
  -f 'checks[][context]=oya-ci-required'    # replaces github-lane-unlocker-required
```

---

## GATE 3 — Dev cutover (mainline advance)  ⏳ BLOCKED on reconcile
`github-mirror/dev` (`9f1047e62`) has **10 UNIQUE commits** cleanup lacks (Rust hook checkers, ADR-index regenerator, doc-staleness tool, hyperscaler-fitness + k8s-authority standards, python-removal, ops-portal→native-SCM). A clean FF would DROP them. Autonomous prep (post-Wave-2): reconcile those 10 into cleanup (intent-reapply respecting cleanup's reorg; scrutinize the 24K-file k8s commit `30bb5d50b` — likely vendored bulk to filter). Then the **paired** advance of `dev` to the reconciled consolidated branch. (Exact command finalized post-reconcile.)

---

## GATE 4 — Migration #7 (sibling repos → jason931225/oyatie)  ⏳ DEPENDS on Gate 3
Per `MIGRATION-PLAN-RESYNC.md` §2.4: the std-first lane PRs (office/oyago/oyapy/codex/claude + pilot stack) into the canonical repo, squash-only, each passing the 9 conformance gates. Migration base = the **consolidated cleanup branch** (not raw dev), so this depends on the Gate-3 branch-reconciliation decision. Outward-facing + multi-PR → staged + reviewed by you.
