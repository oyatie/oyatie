# Dependency-Automation Ban — Sunset Plan (rev 1)

**Status: PENDING APPROVAL** — planning artifact, deliberate mode (supply chain + governance record). Date: 2026-07-27.

---

## Verified baseline (each checked against `origin/dev` this session)

| Fact | Evidence |
|---|---|
| Ban is **file-presence only**, 13 paths, **hardcoded in Rust** | `ci/facade/dependency-automation/src/lib.rs:34-45` — an array, not policy data |
| Dependabot has a **fileless mode**; Renovate does not | `dependabot_security_updates` is a repo setting (now enabled); every Renovate config path is on the ban list |
| The owned actuator **was never built** | `git ls-tree origin/dev` — zero matches for `bump-bot` / `bump_bot` / `dependency-bump` |
| `oya-deps.toml` declares `engine = "owned-rust-bump-bot"` | `oya-deps.toml:13` — names the absent actuator as the authority |
| `audit_policy = "cargo-vet"` has no config | no `supply-chain/config.toml` on `origin/dev` |
| `advisory_policy = "cargo-deny"` is superseded | `supply-chain-audit` purpose: *"owned pure-Rust replacement for the reverted #974 shell cargo-audit/deny"* |
| ADR-0535 is `status: Accepted`, **`door: one-way`**, **no sunset clause** | frontmatter + grep for sunset/expire/temporary/revisit returns only tenant-API EOL language |
| **Measured cost:** 1,556 locked crates; `third-party/BUCK` changed **3× in 6 months**, most recently to *add* Leptos, not to bump | `git log --since="6 months ago" -- third-party/BUCK`; `Cargo.lock` name count |

### Two claims I made earlier and retract
- **"Built on a transport ADR-0363 retired."** False. `scm-facts` is a **live** VCS-agnostic seam (ADR-0535:115, ADR-0526) and `ci/facade/scm-facts-snapshot` is an active matrix gate. ADR-0363 retired the changeset **state machine** (ADR-0110); I conflated the two.
- **"There is no third slot Renovate fits into."** Reasoned from the ban as if it were permanent architecture. It was a forcing function — which the ADR fails to record.

---

## The actual defect

The founder's intent was a **temporary forcing function**: ban the external bots so an owned mechanism gets built. The ADR records `door: one-way`, `Accepted`, and **no expiry**. So:

1. The bot was never built.
2. The ban stayed, and reads as architecture to every subsequent reader — including me, an hour ago, confidently.
3. Nothing measures the gap, because nothing knows a gap was intended.

**This is the gate-defensibility problem in one instance:** a control that cannot state its own retirement condition will outlive its purpose silently. Fixing only the ban repeats the defect; fixing only the record changes nothing today.

---

## RALPLAN-DR

### Principles
1. **A constraint must record its own expiry.** A forcing function with no sunset becomes architecture by default.
2. **Redundant checks are welcome; dependencies are not.** Third-party tools may run — nothing may *rely* on them (founder, 2026-07-27).
3. **Amendability is part of the control.** A ban list that needs a recompile to change is a ratchet, not a policy.
4. **Cost must be measured, not asserted.** Three plan revisions died this session on unmeasured premises.

### Drivers
1. 1,556 dependencies with effectively zero routine version maintenance for 6 months.
2. Security is **already covered** — `supply-chain-audit` blocks known-vulnerable crates from a vendored RustSec mirror, fail-closed. So the gap is **freshness only**, which bounds the severity.
3. The owned actuator is unbuilt and unscheduled; the freeze forbids building it now.
4. The founder has ruled config files acceptable.

### Options
**A — Build the owned bump-bot.** What ADR-0535 intended. *Rejected:* unbounded machinery under a standing freeze, and it is what has already failed to happen for months.
**B — Lift the ban, ship nothing else.** *Rejected:* repeats the original defect — a change with no recorded rationale, which the next reader misreads.
**C — Record the sunset only.** *Rejected:* honest but delivers nothing; deps stay unmaintained.
**D — Readmit Dependabot for version updates AND record the sunset + retirement condition. SELECTED.**

---

## Scope — 3 changes, one PR

### D1 — Move the ban list from code to policy data
`dependency-automation/src/lib.rs:34-45` is a hardcoded array. Move it to a policy JSON alongside the crate. This is the repo's own policy-as-data doctrine, and it is what makes the remaining ban amendable without a Rust edit.
**Exit:** gate tests green; the list lives in JSON; `gate-self-conformance` still passes.

### D2 — Readmit **both** Dependabot and Renovate as redundant checks
*(Revised by founder directive 2026-07-27: "dependabot and renovate are redundant checks that we want with our owned. we want owned method that is cloud native, and in rust.")*

Delete the ban list. Both tools are admitted as **redundant checks**, permanently — not as a stopgap. The prior rev kept Renovate banned on a "no fileless mode" argument; with the ban lifted, its config file is simply allowed and that asymmetry is moot.

The rest of `dependency-automation` survives intact — `oya-deps.toml` closed-schema validation, Rust pin alignment across toolchain/MSRV/container/Buck2, and `third-party/BUCK` overlay validation. **Only the ban list goes.**

Ship both configs deliberately conservative — grouped, low `open-pull-requests-limit`, monthly — because 1,556 crates against a 38-min pipeline could otherwise flood the queue. Two bots on the same dependency set will propose overlapping bumps; that is the definition of a redundant check, and the cost is duplicate PRs, not conflict.
**Exit:** gate green with both configs present; each bot opens ≤ its configured limit; `supply-chain-audit` untouched.

### D3 — Amend ADR-0535: record the intent, and that the checks are **permanent**
Record: the ban was a forcing function; the owned bump-bot was never built; **Dependabot and Renovate are readmitted as permanent redundant checks**; `oya-deps.toml` + `supply-chain-audit` remain the authority.

**Correction to this plan's own rev 1:** the retirement condition is *not* "remove the bots when the bot ships." They stay. What changes when the owned actuator ships is that they go from **sole/transitional** to **redundant**.

**And this is the repo's existing pattern, not a new exception** *(founder, 2026-07-27: "it can be transitional until we have our owned way ready")*. Dependabot/Renovate are **transitional adapters behind a named owned destination** — the same construction already applied to:
- GitHub Actions — `oya-deps.toml:16` `github_actions = "adapter-only"`; ADR-0535:96 *"only the transitional runner adapter, while the policy source of truth is…"*
- git — ADR-0526 *"git is transitional"* behind the scm-facts VCS-agnostic seam
- Talos / upstream k8s — ADR-0510, transitional behind stable interfaces

So rev 1's "accepted violation" framing was wrong. There is no violation: the reliance rule forbids *undeclared, permanent* dependence, and the sanctioned form is a declared transitional adapter with the owned destination named. That is what D3 records.

**What must be greppable** is the pair: `transitional_until = owned-rust-bump-bot`, and the fact that the destination is currently **unbuilt**. Declaring the adapter without declaring the destination's absence is how the last one calcified.

ADR-0535 is `door: one-way`, so this is an amending ADR, not an edit. Land `Proposed`.
**Exit:** amending ADR merged; both projections updated; the load-bearing-until-owned statement is machine-greppable.

### D4 — Record the owned bump-bot as the destination *(no build)*
Cloud-native, Rust, per `oya-deps.toml:13` `engine = "owned-rust-bump-bot"`. Not built here — the freeze stands — but it moves from unspoken assumption to a tracked item with an owner, because D3's honesty depends on someone eventually closing it.
**Exit:** a filed issue naming the actuator, its interface (`scm-facts` ChangeSets per ADR-0526), and what "shipped" means.

### Explicitly out
Building the bump-bot in this PR. Touching `supply-chain-audit`. Reconciling `oya-deps.toml`'s stale `cargo-vet`/`cargo-deny` fields — recorded as a separate finding; they name tools absent or superseded, but fixing them is not required to lift the ban.

---

## Pre-mortem

**P1 — Dependabot floods the merge queue.** 1,556 crates; even grouped, the first run can open a large batch against a pipeline at 38 min wall and a 10 GB cache already over budget.
*Mitigation:* land with `open-pull-requests-limit: 3` and a monthly schedule, grouped by ecosystem. Raise only after observing one cycle. **If the first cycle disrupts throughput, revert the config — the gate change and the ADR stand on their own.**

**P2 — Readmitting Dependabot is read as "Dependabot is now the mechanism."** Exactly the reliance the founder banned.
*Mitigation:* D3 states the authority explicitly, and D2 admits it for *version* updates only while `supply-chain-audit` keeps security. The retirement condition is written down, so the next reader inherits the intent this time.

**P3 — D1 changes gate behaviour while moving the list.** A data migration that silently alters the ban set would be invisible in review.
*Mitigation:* D1 lands as a pure move — byte-identical path set, asserted by a test that pins the 13 entries — and D2 removes exactly two of them in a separate commit within the PR, so the diff shows the policy change on its own.

---

## Test plan (deliberate mode)

| Layer | What |
|---|---|
| Unit | ban-list loader: malformed policy fails closed; empty list is rejected (an empty ban set is a vacuous pass) |
| Contract | RED fixture — a `renovate.json` still REDs after D1/D2; GREEN fixture — `.github/dependabot.yml` passes |
| Migration | D1 asserts the moved list is set-equal to the 13 hardcoded paths before D2 removes any |
| Live-corpus | `dependency-automation` green on the candidate tree with the new `dependabot.yml` present |
| Integration | full `oya-ci-required` green on the PR |
| Observability | after one Dependabot cycle: count PRs opened vs `open-pull-requests-limit`; record whether queue throughput changed |
| Regression | `supply-chain-audit` untouched — assert its policy and baseline are byte-identical |

---

## ADR

**Decision.** Move the ban list to policy data; readmit `.github/dependabot.yml` for version updates only; amend ADR-0535 to record the forcing-function intent, the unbuilt actuator, and an explicit retirement condition.

**Drivers.** 1,556 dependencies with ~zero routine maintenance in 6 months. Security already covered by an owned, fail-closed gate, so the gap is freshness only. The owned actuator is unbuilt and unschedulable under the freeze. The founder has ruled config files acceptable.

**Alternatives considered.** *Build the bump-bot* — unbounded machinery under a freeze, and already the thing that did not happen. *Lift the ban silently* — repeats the defect that caused this. *Record the sunset only* — honest, delivers nothing.

**Why chosen.** It is the only option that both unblocks maintenance today and leaves a record that prevents the same silent calcification. The retirement condition means this decision can be undone deliberately rather than forgotten.

**Consequences.** A third-party bot opens PRs in a repo that had none. `oya-deps.toml` remains partly aspirational (its `engine` names an absent actuator) — documented, not fixed. Renovate stays banned. If the bump-bot ever ships, D2 is reverted by its own stated condition.

**Follow-ups.** `oya-deps.toml` stale fields (`cargo-vet` absent, `cargo-deny` superseded). Gate defensibility as a general standard — this plan applies it to exactly one gate; #1431, #1432 remain open.
