# CI Portfolio Rebalance — Consensus Plan (rev 3)

**Status: PENDING APPROVAL** — planning artifact, deliberate mode. Date: 2026-07-27.
rev 2 was REJECTED by the Critic. rev 3 is **two items**. That is the honest size of this work.

---

## Errors in rev 2, each verified against `origin/dev`

| rev-2 claim | truth |
|---|---|
| "no matrix gate calls git — the blocking audit is already recorded in-tree" (cited workflow `:144-146`) | **The comment is false.** `ci/facade/automation-language-policy/src/lib.rs:902-921` runs `Command::new("git")` for `merge-base <base_ref> HEAD` and `show <rev>:<path>`. Needs full history **and** a live `origin/dev` ref. `scm-facts-snapshot` is a second such leg. rev 2 named only the second, as "the candidate", singular. |
| "matrix membership is a hard invariant (`gate_registration.rs:681`)" | **False.** `gate_registration.rs:713-715` accepts **three** forms: `-p <crate>`, matrix leg, **or** a dedicated buck target. Used to close an alternative it does not close. |
| "`oya-governance-supply-chain` … **does not exist**" | **False.** `libs/oya-governance-supply-chain-kernel/{BUCK,Cargo.toml,src/lib.rs}` exists. It is **unwired to any lane** — which is a *stronger* finding, stated wrongly. |
| "The sanctioned form is already in `.github/CONTRIBUTING.md:80`" | **Self-contradictory.** Line 80 is `cargo clippy --workspace --all-features --all-targets -- -D warnings` — it *includes* `--all-targets`, the one flag rev 2 called "the whole trick" to omit. |
| S3 is "production-exact by construction" | **Under-counts.** `Cargo.toml` `exclude` lists `cloud/cloud-kernel` and `kernel` — separate workspaces holding the rung-0 kernel. `--workspace` cannot reach them. |
| deliberate mode satisfied | **Test plan was dropped** between rev 1 and rev 2. Mandatory section, missing. |

The pattern is the one this plan twice claimed to be guarding against: **rev 2 inherited the Architect-verified premises and reasoned onward. Every claim the Architect did not touch was wrong.** Trusting a *code comment* as "an audit recorded in-tree" is the sharpest instance.

---

## Scope — 2 items

### S1 — Enable secret scanning + push protection *(founder action, no code)* — UNCHANGED
The only item to survive all three revisions untouched. Closes CICD-SEC-6 from literal zero (`secret_scanning`, `secret_scanning_push_protection`, `dependabot_security_updates` all report `disabled`; no `.github/dependabot.yml` on `origin/dev`).
**Exit:** `gh api repos/... --jq .security_and_analysis` reports `enabled` ×3.
**Expected:** a nonzero historical backlog. Normal, not failure. Triage separately; rotate-or-dismiss-with-reason, never bulk-ignore.
**SHIPPED 2026-07-27, partially.** `secret_scanning`, `secret_scanning_push_protection`, `dependabot_security_updates` all verified `enabled`. `secret_scanning_non_provider_patterns` and `secret_scanning_validity_checks` were requested twice and **silently refused** (API returns 200, field stays `disabled`) — paid Secret Protection features, unavailable on this repo's plan. 0 alerts at enable time; treat as provisional until backfill completes.

**Keep both, as REDUNDANT CHECKS.** Founder doctrine (2026-07-27): third-party tools are fine — *depending* on them is the anti-pattern. That is why ADR-0535 bans `dependabot.yml`/Renovate **config** (which would make it the mechanism) while the security-updates **setting** (a redundant signal) is fine. Same for GitHub secret scanning: enable it, keep it, do not build on it.

The layering as it actually stands:

| | owned primary | redundant check |
|---|---|---|
| dependency vulns | `oya-deps.toml` + `supply-chain-audit` (vendored RustSec mirror, fail-closed) | Dependabot security updates |
| credentials | **none yet** | GitHub secret scanning |

The credential row's empty cell is a backlog item, not an argument against enabling the check. Note the redundant check is also **blind to this repo's credential shapes** — partner patterns cover AWS/GitHub/Stripe formats; the generic detector that would catch OCI Vault refs, Talos configs and Postgres DSNs is the paid half we cannot enable, and the known `/tmp` incident is in that blind half. So the owned primary, when built, cannot be modelled on what GitHub does here.

**Also known-weak:** hand-set, with no liveness detector — if someone disables it, nothing REDs.

### S3 — Measure the clippy blast radius *(one command, corrected)*
```
cargo clippy --workspace --all-features -- \
  --force-warn clippy::unwrap_used \
  --force-warn clippy::expect_used \
  --force-warn clippy::panic
```
- **Omit `--all-targets`** — that is what excludes `#[cfg(test)] mod tests` bodies from compilation, so the count reflects production only.
- **`--force-warn`, not `-W`** — the workspace currently sets these three to `"allow"` (`Cargo.toml:940-942`) and 17 in-source `#![allow(...)]` sites exist. `-W` can be overridden by both and **silently return 0**, which for a one-integer deliverable is the worst possible failure.
- **Coverage caveat, stated not hidden:** `--workspace` excludes `kernel/` and `cloud/cloud-kernel` (`Cargo.toml` `exclude`). The integer is the root workspace only; the rung-0 kernel needs a second run in its own workspace.
- **Blocker:** `cargo` is hook-blocked *in this agent session* — a property of my tooling, not the repo. Needs the founder or an unblocked lane.

**Exit:** two integers (root workspace; kernel workspaces), recorded on an issue with the exact command.

### DELETED — S2 and S2′
- **S2** (add buck-out restore to the matrix): net **+175s/leg × 42**. Refuted on measurement. The `−33s` credit was itself wrong — a warm cache makes that step *faster*, it does not remove it, so the true cost is worse than stated.
- **S2′** (shallow the matrix checkout): its enabling premise is a false comment. Two legs call git; excluding both leaves an unmeasured fraction of 753s traded against a false-green on the two legs that police history. **If anything survives it is a measurement — "what does shallow actually save on one safe leg?" — not a PR.**

### RESTATED — S4 *(no ruling requested)*
`oya-governance-supply-chain` is **unwired, not absent**. Its claim surface is wider than rev 2 found: `docs/advanced-cicd/branch-pipeline/branch-protection-rules.md:60,96,130` cites it as a **required status context**; `ci-policy-per-branch.md:42` as a **BLOCKER on every PR**; `docs/checklists/done-definition-checklist.md:90` and `per-implementation-plan-checklist.md:37` as a **done-definition lane**.
So: a real kernel crate, wired to nothing, cited across the governance corpus as a blocking gate. **Inventory the full claim surface first; no ruling is requested on a binary that has now been mis-sized twice.**

### Explicitly out
Any new gate crate. Rebalancing the 27 governance gates. E2E/coverage/SAST. Collapsing the matrix — rejected **solely** on the Windows leg (`//libs/oya-workspace-members-kernel:...-cargo-differential` on `windows-latest`, unreachable from `//ci/...`), which stands unaided. The `gate_registration` argument is withdrawn.
**Gate defensibility** (founder directive, 2026-07-27) is deliberately **not** folded in — see below.

---

## Pre-mortem

**P1 — S1's alert backlog gets the control switched back off.** The likely failure of S1 is social, not technical.
*Mitigation:* backlog triage is a separate issue with its own owner; the control is never evaluated on alert count.

**P2 — S3 is run with `-W` instead of `--force-warn` and returns 0, and the 0 is believed.** A false zero would "prove" the denies can be restored freely and would be acted on.
*Mitigation:* the exit requires the exact command recorded alongside the integer. A reported 0 must be re-run with one lint flipped to `--deny` on a known-violating file as a positive control before it is trusted.

**P3 — S3 has no owner and is never run**, and the blast radius keeps being cited as a reason for things.
*Mitigation:* if unrun by the next CI review, record "not measured" and stop citing it as justification for anything.

---

## Test plan *(deliberate mode — restored)*

| Layer | What |
|---|---|
| Unit | none — neither item adds logic |
| Integration | S1: a canary test-secret push must be **rejected** by push protection (this is the only real behavioural test in the plan) |
| E2E | n/a — no user-facing surface changes |
| Observability | S1: `gh api .../security_and_analysis` reports `enabled` ×3 and stays enabled at next review |
| Positive control | S3: re-run with one lint at `--deny` against a known-violating file; a 0 that survives that is trustworthy, a 0 that does not is a flag-ordering bug |
| Regression | none — no code ships in rev 3 |

---

## ADR

**Decision.** Enable secret scanning + push protection. Run one corrected clippy command to get the blast-radius integers. Ship no code.

**Drivers.** Credential hygiene is the only category at literal zero, with an incident precedent. The clippy measurement is one command and its absence is currently used to justify inaction. Every efficiency item proposed across three revisions was refuted on measurement.

**Alternatives considered.** *rev 1 Option C* — refuted: its central item was +7,350 job-sec. *rev 2* — refuted: its efficiency item rested on a false code comment, and three supporting citations were wrong. *Collapse the matrix* — rejected on the Windows leg alone.

**Why chosen.** It is what survived. Two items, one of which is a settings toggle and one a single command; no code, no machinery, no new gate.

**Consequences.** SLSA stays at Level 0. Clippy stays off pending S3. The 63%-governance portfolio shape and the 1,385s of matrix compilation are **documented and left alone** — this plan does not improve pipeline efficiency at all, and says so rather than pretending otherwise.

**Follow-ups.** #1427, #1431, #1432. Gate defensibility is a separate and larger decision (below).

---

## Deferred: gate defensibility *(founder directive 2026-07-27, NOT scoped here)*

The standard asked for — *should it exist · should it exist this way · what is the hyperscaler pattern/anti-pattern* — is sound and unaddressed by anything in the pipeline today. `gate-self-conformance` emits seven codes and **all seven police construction** (hermetic, policy-as-data, registered, well-formed); **none asks whether a gate should exist, whether anyone acts on it, or when it retires.**

Reference pattern ([Tricorder](https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/43322.pdf)): a false positive is *any report the reader did not want to see*; analyzers above ~10% FP are **disabled**; a "NOT USEFUL" signal drives continuous measurement; the preferred report **ships a fix**.

Our anti-pattern: an admission bar with **no eviction bar**. 43 of 49 gates opt out of providing a fix via `no_autofix_reason` — the inverse of the reference default.

**Why it is not in rev 3:** enforcing it is new machinery under a standing freeze, and authoring 43 justification records big-bang is precisely the scope explosion that killed the first loop. It needs its own decision, not a line item appended to a two-item plan.
