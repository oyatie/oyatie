---
id: ADR-0634
title: "Approval attaches to the PRODUCER of a change, not to a reader of its diff: a mechanical auto-approval predicate over declared change classes, an anomalous-residue definition that is the predicate's complement, a digest-pinned expiring approval policy that replaces the second human this repo does not have, and fan-in to the single oya-ci-required context rather than a second bypassable one"
status: Proposed
doc_status: drafted
planning_impact: true
deciders: founder
date: 2026-08-02
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0633, ADR-0515]
amends: []
related: [ADR-0109, ADR-0111, ADR-0124, ADR-0539, ADR-0554, ADR-0562, ADR-0595, ADR-0613, ADR-0523]
milestone: W3
---

# ADR-0634: approval attaches to the producer, not to the reader

## Status

**Proposed — 2026-08-02.** Landed `Proposed`, not `Accepted`, for the reason ADR-0633 states in its
own Status section: a fresh `Accepted` reddens `cloud-ci-cross-artifact-agreement` until the
evidence it claims has propagated. Nothing in this ADR is enforced by its own merge; every decision
below carries the assertion that would enforce it, and that assertion is the follow-up work.

This ADR **generalizes ADR-0633** ("enforcement belongs to the layer that OWNS the fact") from
enforcement to *approval*. ADR-0633's rule is that a fact should be checked by the layer that
produces it. Approval is a fact. This ADR applies the same rule: approval of a mechanically-derived
change belongs to its producer, not to a reader of its output.

---

## Context: what is actually true, measured 2026-08-02

Every row below was produced by the command in its own cell against `jason931225/oyatie` at
`origin/dev` = `890acdaea`. No row is inferred.

### C1 — The live gate on `dev`

| Property | Live value | Command |
|---|---|---|
| Required contexts | `["oya-ci-required"]` — exactly one | `gh api repos/jason931225/oyatie/branches/dev/protection` |
| `required_pull_request_reviews` | **key absent entirely** | same |
| `enforce_admins.enabled` | `false` | same |
| `required_signatures.enabled` | `false` | same |
| `required_linear_history.enabled` | `true` | same |
| Repo rulesets | `[]` — zero, so classic protection is the whole mechanism | `gh api repos/jason931225/oyatie/rulesets` |
| Repo owner account type | `User` (not `Organization`) | `gh api repos/jason931225/oyatie --jq .owner.type` |

**Correction to the premise this ADR was commissioned under.** The premise was that
`require_code_owner_reviews` is *unset*. It is stronger than unset: the entire
`required_pull_request_reviews` object is absent from the live protection payload. There is no
review requirement to weaken, and — because that same object is what carries the pull-request
requirement — no live requirement that a change arrive as a PR at all.

### C2 — Declared protection vs. live protection

`git show origin/dev:.github/branch-protection.yaml` declares, for `dev`:

| Declared | Live | Verdict |
|---|---|---|
| `require_pull_request: true` | no `required_pull_request_reviews` object | DRIFT |
| `require_signed_commits: true` | `required_signatures.enabled: false` | DRIFT |
| `required_approving_reviews: 0` | (n/a — no review object) | **agrees, and is the point** |
| `required_status_checks: [oya-ci-required]` | `["oya-ci-required"]` | agrees |

The declared file **already sets `required_approving_reviews: 0`** and annotates itself
`REVIEW-ADMISSION-GAP-LIVE-BOUNDARY: F-PR5-06 remains open`. The repo's own shadow record says
review admission is target-only. This ADR does not discover that gap; it disputes the *shape* of the
target.

### C3 — CODEOWNERS routes zero

| Fact | Value | Command |
|---|---|---|
| Owner-resolution errors | **111**, every one of kind `Unknown owner` | `gh api repos/jason931225/oyatie/codeowners/errors` |
| Pattern lines in the file | 67 | `git show origin/dev:.github/CODEOWNERS \| grep -c -E '^[^#[:space:]]'` |
| Distinct error kinds | 1 (`Unknown owner`) | same errors endpoint, grouped |

Two independent causes, either of which alone is fatal:

1. Every owner handle is `@teams/*`. `.github/CODEOWNERS` line 2 admits this in its own header —
   *"Team handles are logical owner IDs until the GitHub org/team namespace is provisioned."*
   The namespace was never provisioned; C1 measures `owner.type = User`, and a user account has no
   team namespace to provision one into.
2. The path patterns are pre-reorg. `crates/oya-platform-*`, `crates/oya-cloud-*`, `crates/oya-saas-*`
   and eleven siblings address a `crates/` root that ADR-0562's capability-first reorg emptied. Even
   a resolvable handle on those lines would route nothing.

### C4 — Reviews, measured across every recent PR

GraphQL over the 25 most recently created PRs (`#1489`–`#1515`), fields `reviews.totalCount`,
`reviewRequests.totalCount`, `latestOpinionatedReviews.totalCount`:

| Metric | Value across all 25 PRs |
|---|---|
| Reviews submitted | 0 |
| Review requests | 0 |
| Latest opinionated reviews | 0 |

Not "few". Zero, uniformly, with no exception in the window.

### C5 — The single required context is not observed green at merge

GraphQL over the 30 most recent MERGED PRs, correlating each PR's `mergedAt` against the
`completedAt`/`conclusion` of the `oya-ci-required` check-run on its own head commit:

| State of `oya-ci-required` on the PR head | Count | Share |
|---|---:|---:|
| `NONE` — never reported on that head at all | 17 | 56.7% |
| `CANCELLED` | 6 | 20.0% |
| `FAILURE` | 4 | 13.3% |
| `SUCCESS`, but concluded **after** `mergedAt` | 1 | 3.3% |
| `SUCCESS`, concluded **before** `mergedAt` | **2** | **6.7%** |

Only `#1506` (green `04:47:47Z`, merged `04:48:01Z`) and `#1502` (green `02:47:43Z`, merged
`02:47:45Z`) merged with the one required context observed green. `#1505` shows `SUCCESS` but
concluded `05:22:05Z`, 34m14s **after** it merged at `04:47:51Z` — a `SUCCESS` that a naive
present-tense rollup query would have miscounted, which is why this table joins on timestamps and
not on conclusion alone.

Confirmed by a second, differently-shaped probe. For `#1514` (`NONE` row), the combined-status
endpoint — a different API surface from check-runs — returns
`{"state":"pending","n":0}` for head `2d803c3`
(`gh api repos/jason931225/oyatie/commits/2d803c328cecad237d5e9160be44c9e1e0fd1983/status`).
Zero statuses of any kind ever posted; merged `06:40:25Z` regardless.

### C6 — `#1507`, measured precisely

The premise stated `#1507` merged "with THREE required contexts in FAILURE". The mechanism is
different, and worse. Head SHA `b848881`, `mergedBy: jason931225`, `mergedAt: 2026-08-02T04:48:14Z`
(`gh api repos/jason931225/oyatie/commits/b84888127040f2e4382db25cbd252f481838091a/check-runs`):

| Check | Conclusion | Completed | Relative to the `04:48:14Z` merge |
|---|---|---|---|
| `cloud-ci-firewall (baseline ratchet + gate-registration meta-test)` | FAILURE | `04:42:50Z` | **5m24s before** — red at the merge instant |
| `buck2 (hermetic build + affected gate tests)` | FAILURE | `04:59:08Z` | 10m54s after — still running at merge |
| `gate · affected-set (ADR-0554, binding workspace coverage)` | FAILURE | `05:07:34Z` | 19m20s after — still running at merge |
| `oya-ci-required` | FAILURE | `05:21:59Z` | **33m45s after** — the required context had not reported |

**Correction.** At the merge instant exactly *one* check had concluded red; the other three had not
concluded at all. The single required context was **pending**, and the merge did not wait for it.
Three-reds-and-merged-anyway would be a policy violation. What actually happened is that policy was
never consulted: `enforce_admins: false` (C1) means the repo owner's merge is not evaluated against
required contexts in any state — pending, red, or absent. C5 shows this is the normal path
(17 `NONE` rows), not an incident.

### C7 — The reviewer producer that exists, and the two that do not

| Artifact | State | Evidence |
|---|---|---|
| `oya/intelligence/.../pr-review-dispatcher-app` | Scaffold that **approves on zero input** | Its own doc comment: *"until then, the dispatcher reads zero files and emits an APPROVE verdict tagged `subagent_runtime_pending = true`"* |
| `.github/workflows/pr-review.yml` — the workflow that dispatcher's doc comment says invokes it | **Does not exist** | `git ls-tree -r --name-only origin/dev .github/workflows/` returns exactly four entries: `OWNERS`, `cache-integrity-canary.yml`, `docs-graph-drift.yml`, `oya-ci-required.yml` |
| `oya-ci-controller-kernel::ReviewAdmissionProducer` | **Real, fail-closed, 20 integration tests** | `oya/ci-controller/crates/oya-ci-controller-github-adapter/tests/review_admission.rs` |
| That producer's wiring into CI | **None** | `oya-ci-required.yml:12-14` states it is *"deliberately NOT wired here"*; no workflow invokes it |

So the premise's "the only reviewer producer in-tree self-documents as a scaffold that emits APPROVE
on zero input" is correct about that crate and **understates the deadness** — the crate is not
merely a permissive scaffold, it is never invoked, because its invoking workflow was never written.
The premise also misses that a *second*, genuinely rigorous producer exists and is likewise unwired.

### C8 — The PR-body approval requirement was retired eight days ago

The premise states that repo policy binds approval to a `## Code Review` section in the PR body.
That was true and is now **stale**. `oya-ci-required.yml:678-689` records the removal verbatim:

> PR-body admission was removed here (founder directive 2026-07-26: "stop the entire pr body checks
> — that is just unnecessary"). It validated that a PR description contained nine string literals […]
> It inspected no code and could not fail on a defect: an author who wrote the right headings passed
> regardless of what the diff did.

`docs/AGENTS.md` was not updated and still asserts the requirement in six places — lines 138, 144,
161, 228, 236, 249, 258, including *"no signature, no merge"* (:138) and *"The PR body's
`## Code Review` section MUST contain the agent name, the verdict…"* (:161).

The corrected statement of the defect is therefore sharper than the commissioning premise. It is not
"a gate that checks prose instead of code". It is: **the operating contract mandates a review
artifact that no mechanism produces, no mechanism consumes, and no mechanism checks.**

### C9 — Change volume, so the pattern is sized to real work

40 most recently merged PRs (`changedFiles`): sum 965, max 360, min 1, two above 100.

| PR | Files | Diff | Title |
|---|---:|---|---|
| `#1498` | 360 | `+2492/-2103` | `refactor(governance): home the dep-lint authority's 56 leaf check kernels into governance/check` |
| `#1507` | 259 | `+5650/-973` | `feat(protocol): establish the owned product transport contract` |
| `#1483` | 53 | `+1042/-1465` | `fix(ci): close the gate-registration scope gap and retire 11 dark tools/ entries` |

`#1498` is this repo's LSC archetype: one mechanical rehome across 360 files. It is also the honest
counter-example — `+2492/-2103` is not a pure rename, so under D2 below it would **not** clear the
auto-approval predicate as authored. That is the intended behaviour and the reason the predicate is
written as a property of the diff rather than of the commit message.

---

## The three defects, named

| # | Defect | Derived from |
|---|---|---|
| **F1** | The approval obligation is stated in prose — `docs/AGENTS.md` (×7) and root `CLAUDE.md` `required_workflow.completion_gate: reviewer-agent APPROVE plus cloud-ci green before auto-merge` — with no producer, no consumer, and no checker. | C7, C8 |
| **F2** | The routing table that would say *who* approves resolves to nothing: 111/111 unknown owners over 67 patterns that address a directory tree the reorg deleted. | C3 |
| **F3** | The one thing that *is* wired — `oya-ci-required` — is bypassed on 28 of the last 30 merges, so any new obligation attached beside it inherits the same bypass. | C5, C6 |

**Why "just enable required reviews" is the wrong repair, stated as a derivation rather than an
opinion.** Turning on `required_pull_request_reviews` with the model as currently specified composes
three measured facts into a worse outcome than the status quo:

1. The eligible approver set would resolve through CODEOWNERS → 111/111 unknown owner (C3) → **no
   one is eligible**, so either every PR blocks forever or the setting is configured to accept any
   user.
2. "Any user" in a repo with `owner.type: User` and exactly one human (C1, C4) means
   self-approval — which the repo's own rigorous producer explicitly refuses:
   `author_cannot_satisfy_review_admission_and_failure_is_posted`
   (`tests/review_admission.rs:434`) asserts
   `Err(KernelError::InvalidInput(msg)) if msg.contains("distinct")`.
3. The only automated producer that *could* fill the seat approves on zero input (C7).

Composing (1)–(3): the reachable configuration is one that emits APPROVE unconditionally. That is
strictly worse than no gate, because a gate that always passes manufactures the *evidence* of review
while providing none — and every downstream consumer (release governance, audit chain, the
post-merge product gate) would then be reading a signal that means nothing. `enforce_admins: false`
(C1) means it would additionally be bypassable, so it would not even always-pass honestly.

---

## The pattern (stated independently of any implementation)

Named practice: **Google's Large-Scale Change model** — Winters, Manshreck & Wright,
*Software Engineering at Google* (O'Reilly, 2020), Ch. 22 "Large-Scale Changes", and the approval
decomposition in Ch. 9 "Code Review". The elements this ADR adopts:

| Element of the practice | What it means | What this ADR takes from it |
|---|---|---|
| Review the **tool**, not the output | An LSC's correctness argument is made once, about the generator, by a human who reads the generator. | D1 |
| **Global approvers** | A small trusted set may approve mechanically-generated shards repo-wide, bypassing per-directory owners, because per-directory review of generated output adds no information. | D4 |
| **Shard + mechanical predicate** | Each shard is approved because it matches a declared pattern, not because someone read it. | D2 |
| **Residue escalation** | Whatever the pattern does not cleanly cover goes to a human as a normal change. | D3 |

**The pattern in one sentence, implementation-free:** *approval is a claim about the process that
produced a change; when the process is mechanical and its output is re-derivable, the claim is
discharged by re-derivation, and human attention is spent only on the part that is not re-derivable.*

The transitional stack (GitHub protection, GitHub Actions, `gh`) can be replaced wholesale — per
`cli_surface_policy` and the ADR-0363 / owned-SCM northstar — without touching the sentence above.
Everything from D2 down is written as predicates over a *diff* and a *declared manifest*, both of
which survive a substrate swap; nothing below depends on GitHub's review object model.

---

## Decisions

### D1 — Approval attaches to the producer of a change, not to a reader of its diff

*Obligation:* ADR-0633 — enforcement belongs to the layer that owns the fact. A producer owns the
fact "this output is what my inputs imply". A diff reader does not.
*Corollary:* the human-reviewable unit for machine-derived change is the **producer and its
predicate**, reviewed once, not the N outputs, reviewed N times.

**Acceptance test.** The approval record must name a producer identity and a policy digest, and must
be refused when either is absent. This assertion already exists and passes today; D1 adopts it as
the contract shape rather than inventing one:

```
// oya/ci-controller/crates/oya-ci-controller-github-adapter/tests/review_admission.rs:372
fn policy_receipt_and_producer_identity_fail_closed_when_incomplete_or_invalid()
// and :224
fn unchanged_digest_cannot_authorize_a_tampered_reviewer_allowlist()
```

Gate id when wired: `oya-pr-review` → fanned into `oya-ci-required` (see D6).

### D2 — The auto-approvable classes, and the mechanical predicate for each

A change is **machine-derived** iff every changed path is covered by exactly one row below and that
row's predicate holds. The predicate is evaluated over the merge-base diff of the candidate tree —
never over the commit message, the branch name, the PR title, or any prose.

| Class | Mechanical predicate (evaluated, not described) | Gate that already computes it | Anchor |
|---|---|---|---|
| **G — generated face regeneration** | Every path ∈ `registry/generated-artifact-control-plane.json` with a declared producer; re-running that producer over the candidate tree yields a byte-identical file | `registry-drift (materialized == regenerated)`, `producer-regen (accounting-registry)` | ADR-0595, ADR-0613 |
| **L — lock / freshness** | `Cargo.lock` + declared generated faces are fresh w.r.t. their sources | `freshness (lock + generated faces, ADR-0539)` | ADR-0539 |
| **M — pure capability move** | Over the repo's own canonical argv `merge_base_diff_args()` (`affected-target-set/src/lib.rs:434` — `diff --name-status -z --find-renames -l0`), every record's status is exactly `R100`, i.e. rename with zero content delta; **and** every destination is legal under the placement policies | `gate · affected-set (ADR-0554)` via `StructuralKind::Rename`; `ci/facade/port-placement`, `ci/facade/module-membership` | ADR-0554, ADR-0562 |
| **D — declared doc lifecycle transition** | The only frontmatter delta is `status:` / `doc_status:`, and `(from, to)` ∈ the `transitions[]` array of the matching `specs/lifecycle-configs/*.json` | `cloud-ci-lifecycle-status` | ADR-0109 |
| **B — baseline shrink** | Every changed count in a declared frozen baseline is strictly less than its merge-base value; no key added | `cloud-ci-firewall (baseline ratchet + gate-registration meta-test)` | ADR-0515 |

**Why these five and no others.** Each row is admissible precisely because a gate *already* re-derives
it — the class carries no new enforcement, only a new *consequence* for an existing green. A sixth
class would require a sixth re-derivation gate first; D7 makes that ordering a rule.

**Acceptance test.** The classifier is total and fail-closed. Written as the assertion, in the
`Decision`-enum idiom `ci/facade/affected-target-set/src/lib.rs:662` already uses:

```rust
/// Fail-closed classification of a candidate diff into auto-approvable classes.
/// Mirrors `Decision::RefuseEmptySelection` (affected-target-set lib.rs:675): a NON-EMPTY diff
/// that classifies to nothing is a BUG in the classifier, never a pass.
#[test]
fn every_changed_path_must_land_in_exactly_one_class_or_become_residue() {
    let diff = merge_base_diff(&fixture("mixed-regen-and-handwritten"));
    let c = classify(&diff, &manifest(), &lifecycle_configs(), &placement_policy());

    // 1. Totality: classification partitions the diff. No path is silently dropped.
    let covered: BTreeSet<&str> = c.machine_derived.iter().chain(c.residue.iter()).collect();
    assert_eq!(covered, diff.paths(), "classifier lost paths: not a partition");

    // 2. Disjointness: a path is machine-derived XOR residue, never both.
    assert!(c.machine_derived.is_disjoint(&c.residue));

    // 3. A non-empty diff that produced an empty partition is a refusal, not an approval.
    if !diff.is_empty() && covered.is_empty() {
        assert!(matches!(c.verdict, Verdict::Refuse), "empty partition over a non-empty diff must refuse");
    }

    // 4. R99 is NOT R100. A rename carrying ANY content delta is residue, by the byte.
    assert!(c.residue.contains("governance/check/dep_lint/src/lib.rs"),
        "a renamed file with a content delta must not clear class M");
}
```

Predicate (4) is what `#1498` (C9) fails: 360 files renamed but `+2492/-2103` of content moved with
them, so under class M its content delta is residue. **This is the behaviour we want**, and it
implies a doctrine consequence recorded as D5.

### D3 — Anomalous residue is the predicate's complement, computed, never estimated

**Residue = every changed path that D2's classifier did not place in a class, plus every path whose
class predicate was evaluated and did not hold.** Residue is not a judgement call and not a
severity; it is a set difference.

| Residue source | Example | Routed to |
|---|---|---|
| Unclassified path | a hand-written `.rs` change | human |
| Class predicate evaluated false | `R99` rename, non-identical regen, non-declared lifecycle transition | human |
| Class-M destination illegal | move lands outside the declared capability stratum | human, blocking (ADR-0562/ADR-0633) |
| Graph-invisible path | `Decision::RefuseUnowned` (`affected-target-set/src/lib.rs:665`) — owner-required file with no owning target | **refuse, not human**: a full run would not compile it, so review of it would be theatre |

**Acceptance test.**

```rust
#[test]
fn non_empty_residue_forbids_machine_approval() {
    let c = classify(&diff_with_one_handwritten_file(), /* … */);
    assert_eq!(c.residue.len(), 1);
    assert!(!matches!(c.verdict, Verdict::MachineApproved),
        "machine approval requires residue == ∅; residue was {:?}", c.residue);
}

#[test]
fn residue_ratio_is_reported_even_when_zero() {
    // A residue metric that is absent when empty cannot be distinguished from a metric that
    // was never computed — the absence-of-a-proxy trap. It is always emitted.
    let c = classify(&pure_regen_diff(), /* … */);
    assert_eq!(c.residue.len(), 0);
    assert_eq!(c.report().get("residue_paths"), Some(&json!(0)));
    assert_eq!(c.report().get("residue_ratio"), Some(&json!(0.0)));
}
```

### D4 — Global approval in a one-person repo: the approver is a digest-pinned expiring policy, not a second human

**The honest constraint, measured.** `owner.type: User` (C1). One human. 0 reviews across 25 PRs
(C4). The repo's own admission kernel refuses author-as-reviewer by construction
(`tests/review_admission.rs:434`). **Therefore any model whose eligible-approver set must contain a
human other than the author is unimplementable here.** Saying otherwise would be the same category
of error as the current `docs/AGENTS.md`: an obligation with no possible producer.

What replaces the second human is **separation in time instead of separation in person** — which is
what Google's global-approver seat actually buys. A global approver at Google does not re-read the
shard; they attest that *this generator, producing this pattern, is authorized*. That attestation is
transferable to an artifact.

| Role | Filled by | Its independence property |
|---|---|---|
| **Approver of the *predicate*** | The single human, once, in a separate PR from any change the predicate later approves | Separation in **time** and in **artifact**: the predicate is reviewed before the changes it will approve exist |
| **Approver of a *change*** | `ReviewAdmissionProducer { github_app_id, workload_identity }` — a machine principal, not a GitHub user | Separation in **person**: the producer is not the author, so `author_cannot_satisfy_review_admission` is satisfiable rather than structurally unsatisfiable |
| **Approver of *residue*** | The single human, contemporaneously | **None. This is self-review.** Recorded as such — see the ceiling below |

The policy artifact already has the right shape in-tree; D4 adopts it rather than designing one.
`ReviewAdmissionPolicy` (`oya-ci-controller-kernel`) carries exactly the fields a global-approver
attestation needs:

```
ReviewAdmissionPolicy {
    policy_ref:          "repo://review-policy/machine-derived-classes",
    version:             "<ISO date>",
    sha256_digest:       <canonical_sha256 of the policy, self-binding>,
    issuer:              <the attesting principal>,
    effective_at_unix_s: <not before>,
    expires_at_unix_s:   <MUST be set: an attestation that never expires is a permanent excuse>,
    revoked:             false,
    eligible_reviewers:  [ <machine principal for classes G/L/M/D/B> ],
}
```

**Acceptance test** — the two properties that make a policy an attestation rather than a config file
both already have passing assertions; D4's contribution is *requiring* them for this use:

```
// tampering the allowlist without re-deriving the digest must not authorize
review_admission.rs:224  unchanged_digest_cannot_authorize_a_tampered_reviewer_allowlist()
// an incomplete or invalid receipt fails closed rather than degrading to permissive
review_admission.rs:372  policy_receipt_and_producer_identity_fail_closed_when_incomplete_or_invalid()
```

plus one new assertion this ADR adds, because expiry is the whole mechanism by which
separation-in-time stays real:

```rust
#[test]
fn an_expired_global_approval_policy_cannot_approve_anything() {
    let mut p = machine_class_policy();
    p.expires_at_unix_s = EVALUATED_AT - 1;
    assert!(matches!(produce(&p, EVALUATED_AT), Err(KernelError::InvalidInput(m)) if m.contains("expire")),
        "an unexpiring predicate attestation is indistinguishable from no attestation");
}
```

**The ceiling, named because it is real.** Residue in this repo is self-reviewed. That is a genuine
weakness and no configuration removes it while the repo has one human. What the model *does* buy is
that self-review is bounded to residue, is measured (D3's `residue_ratio` is always emitted), and
shrinks as classes are added — so the weakness has a size, a trend, and an upgrade path. The upgrade
path is a second principal, human or a genuinely independent reviewing agent; when one exists, only
`eligible_reviewers` changes. Nothing above it does.

**CODEOWNERS.** 111/111 unknown owners over 67 patterns naming a deleted directory tree (C3). It
routes zero and, per D2, per-directory ownership is not the routing mechanism for machine-derived
change anyway. **Decision: delete `.github/CODEOWNERS`.** A registry that resolves to nothing is not
neutral — it is a claim of governance that a reader believes.

**Acceptance test:**

```
# born-blocking, not advisory: the file is either absent or fully resolvable, never decorative
test 0 -eq "$(gh api repos/jason931225/oyatie/codeowners/errors --jq '.errors | length')"
```

That assertion passes on deletion (the endpoint reports zero errors for an absent file) and passes
again if a future org migration makes every handle resolvable. It fails on today's tree. It never
passes on a partially-dead file.

### D5 — An LSC must be *authored* as a pure move so the predicate can fire; content changes ride a separate commit

*Derived from* the D2 class-M predicate meeting the C9 measurement: `#1498` renamed 360 files and
carried `+2492/-2103` in the same change, so no byte-level predicate can distinguish its mechanical
part from its hand-written part after the fact.

The rule: a capability move lands as `R100` renames only. Import-path rewrites, module wiring, and
any content edit land as a **separate commit in the same PR**, where they are residue and get read.
This costs the author nothing (the codemod already produces both halves) and converts a 360-file
unreviewable change into a 360-file auto-approved change plus a small readable one.

**Acceptance test:**

```rust
#[test]
fn a_move_commit_contains_only_r100_renames() {
    // Reuses the repo's canonical argv rather than inventing a second, divergent one — the
    // 100% requirement lives in the ASSERTION, so it cannot drift from the diff the
    // affected-set gate already computes. (affected-target-set/src/lib.rs:434)
    for commit in pr_commits().iter().filter(|c| c.subject.starts_with("move:")) {
        let statuses = run_git(&merge_base_diff_args(&commit.parent, &commit.sha)).statuses();
        let impure: Vec<_> = statuses.iter().filter(|s| *s != "R100").collect();
        assert!(impure.is_empty(),
            "commit {} labelled `move:` carries content deltas: {impure:?}", commit.sha);
    }
}
```

### D6 — Review admission fans IN to `oya-ci-required`; it never becomes a second required context

*Obligation, cited from the repo's own shadow record.* `.github/branch-protection.yaml` already
states it: *"oya-pr-review is intentionally ABSENT from required checks. Once controller deployment,
trusted credentials, fan-in wiring, and live API readback are complete, it must feed this one context
rather than become a competing protected merge authority."* ADR-0515 D-series makes
`oya-ci-required` the single canonical context.

*Reinforced by measurement.* C5 shows the one existing required context is not observed green on
28/30 merges. A second required context would not add a second gate; it would add a second thing to
not wait for.

**Acceptance test** — two assertions, because the contract has two halves:

```
# half 1: the protected-context set stays a singleton, forever
test '["oya-ci-required"]' = "$(gh api repos/jason931225/oyatie/branches/dev/protection \
  --jq -c '.required_status_checks.contexts')"
```

```rust
// half 2: and that singleton is RED whenever review admission is red — otherwise half 1
// is satisfied by simply not fanning anything in.
#[test]
fn oya_ci_required_is_red_when_review_admission_is_red() {
    let rollup = fan_in(&[Lane::new("oya-pr-review", Conclusion::Failure), Lane::all_green()]);
    assert_eq!(rollup.conclusion, Conclusion::Failure);
    assert!(rollup.reasons.contains(&"oya-pr-review"),
        "a fan-in that drops a red lane's identity cannot be debugged from the required context");
}
```

### D7 — A class may not be added to D2 before the gate that re-derives it is green on trunk

*Obligation:* ADR-0633's promotion rule — a check joins a gate only once it is demonstrated not to
false-positive. Applied to approval: a class whose re-derivation gate is absent, advisory, or
known-broken would auto-approve on an unproven predicate, which is C7's failure mode wearing a
different name.

**Acceptance test** — this is a meta-test over the class table itself, in the idiom
`ci/facade/baseline-ratchet/tests/gate_registration.rs` already uses:

```rust
#[test]
fn every_auto_approvable_class_names_a_live_required_gate() {
    for class in AUTO_APPROVABLE_CLASSES {
        let gate = class.rederivation_gate_id;
        assert!(registered_gates().contains(gate),
            "class {} auto-approves on gate `{}` that is not registered", class.id, gate);
        assert!(!known_broken_lanes().contains(gate),
            "class {} auto-approves on `{}`, which is listed known-broken and enforces nothing",
            class.id, gate);
        assert!(required_context_fan_in().contains(gate),
            "class {} auto-approves on `{}`, which does not feed oya-ci-required", class.id, gate);
    }
}
```

The `known_broken_lanes()` arm is not hypothetical: `ci/facade/lifecycle-status/lifecycle-status-policy.json`
lists seven lifecycle lanes that cannot observe a live corpus. Class D depends on
`doc-status-lifecycle`, which is **not** in that list (it observes 2667 docs), so class D is
admissible — but a class built on, say, `capability-status-lifecycle` would be auto-approving on a
lane whose own policy file records that its glob *"matches ZERO artifacts, so the lane evaluates
perfectly clean while observing nothing."*

### D8 — Close the bypass, or every decision above is decorative

*Derived from C1 + C6.* `enforce_admins: false` means the sole human's merge is evaluated against no
required context in any state. D1–D7 all terminate in `oya-ci-required`; if that context is advisory
for the only account that merges, the entire model is advisory.

**Acceptance test** — the config stanza and its readback assertion:

```
gh api -X PUT repos/jason931225/oyatie/branches/dev/protection \
  -H 'Accept: application/vnd.github+json' \
  --input - <<'JSON'
{ "required_status_checks": { "strict": true, "contexts": ["oya-ci-required"] },
  "enforce_admins": true,
  "required_pull_request_reviews": { "required_approving_reviews": 0,
                                     "require_code_owner_reviews": false,
                                     "dismiss_stale_reviews": true },
  "required_linear_history": true, "allow_force_pushes": false, "allow_deletions": false,
  "restrictions": null }
JSON

# readback — the PUT is not the evidence; the readback is
test true = "$(gh api repos/jason931225/oyatie/branches/dev/protection --jq '.enforce_admins.enabled')"
test true = "$(gh api repos/jason931225/oyatie/branches/dev/protection --jq '.required_status_checks.strict')"
```

Three notes on that stanza, each load-bearing:

- `required_approving_reviews: 0` is **kept at zero deliberately**, and this is the crux. The
  `required_pull_request_reviews` object is present only to restore the *pull-request requirement*
  (C1: its absence is why direct pushes are live) and `dismiss_stale_reviews`. Setting it above zero
  would demand the human-approver seat that C4 and `review_admission.rs:434` prove cannot be filled.
  Approval enters through `oya-pr-review`→`oya-ci-required`, per D6 — not through GitHub's review
  object.
- `strict: true` is currently `false` (C1). Without it, a green computed against a stale base merges
  into a trunk it was never evaluated against — the class of failure C6's post-merge red is an
  instance of.
- `require_code_owner_reviews: false` is explicit rather than omitted, so D4's CODEOWNERS deletion
  cannot silently change merge semantics.

---

## Consequences

| | |
|---|---|
| **Gained** | Machine-derived change (classes G/L/M/D/B) merges on a re-derivation proof instead of on nothing. Human attention concentrates on a computed, always-reported residue set. `#1498`-shaped 360-file changes stop being unreviewable. |
| **Cost** | D5 makes moves land as two commits. D7 makes each new class wait for its gate. D8 makes the sole human's merges wait for CI — measurably, that is the 28/30 merges in C5 that currently do not. |
| **Risk accepted** | Residue is self-reviewed while the repo has one human (D4). Bounded, measured, trending; not eliminated. |
| **Risk introduced** | A wrong predicate auto-approves silently. Mitigated by D7 (gate must be live and required) and by D2's totality/disjointness assertions, which turn a classifier bug into a refusal instead of a pass. |
| **Repudiated** | `docs/AGENTS.md`'s `## Code Review` PR-body obligation (lines 138, 144, 161, 228, 236, 249, 258) — already dead in CI since 2026-07-26 (C8), and now doctrinally superseded rather than merely unenforced. Its removal is a follow-up PR, not this one. |

---

## Alternatives rejected

| Alternative | Why rejected | Evidence |
|---|---|---|
| Enable `required_approving_reviews: 1` | Reachable configuration is unconditional self-approval or permanent block | C1, C3, C4, `review_admission.rs:434` |
| Fix CODEOWNERS instead of deleting it | Requires an org namespace a `User` account does not have, *and* re-pointing 67 patterns at a tree ADR-0562 deleted — after which it still routes to one person | C3 |
| Wire the existing `pr-review-dispatcher` scaffold | It approves on zero input, and its invoking workflow does not exist | C7 |
| Make `oya-pr-review` a second required context | C5 shows required contexts are already not waited for; a second one is a second bypass, and `.github/branch-protection.yaml` forbids it in its own text | C5, C6, D6 |
| Keep the PR-body `## Code Review` section as the approval record | Already retired by founder directive 2026-07-26; it inspected no code and could not fail on a defect | C8 |
| Do nothing | Status quo is F1+F2+F3: an obligation with no producer, a routing table resolving to zero, and a required context bypassed on 28/30 merges | C3, C5, C7, C8 |

---

## Open

| Ref | Question | Blocks |
|---|---|---|
| F-PR5-06 | Controller deployment + trusted credentials for `ReviewAdmissionProducer` | D1, D6 |
| — | Which principal issues the D4 policy attestation, and what expiry period | D4 |
| — | Whether `strict: true` (D8) is affordable given measured CI wall-clock; if not, the merge-queue projection of ADR-0111 / ADR-0124 is the alternative to serialized `strict` | D8 |
| — | Sixth class candidates, each blocked on its own re-derivation gate per D7 | D2 |
