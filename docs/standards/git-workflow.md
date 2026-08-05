---
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  the cutover-bootstrap window (time-bounded canonical extension with named
  sunset at M01-P08 sign-off), and the revised banned-primitives lane
  semantics (catch *undocumented* invocations only).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-banned-primitives
enforcement_status:
  oya-governance-banned-primitives: existing
  F-FORBIDDEN-PRIMITIVES-CI-GUARD: pending Wave-B webhook receiver (ADR-0116)
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/AGENTS.md
  - docs/standards/claude-code-harness.md
  - docs/standards/agent-instructions-discipline.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Git Workflow

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

This standard codifies **pragmatic git/gh** under the live operating
contract ([`docs/AGENTS.md`](../AGENTS.md)): direct `git` / `gh` invocation is
the forward contribution path (isolated worktree → PR against `dev`).
Merge admission is reviewer APPROVE plus [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md)
`oya-ci-required`. Historical "Directive 12" wording lived under retired
harness plan paths (`.omc/plans/**` provenance only; ADR-0619).

## 1. The default surface

Per [`docs/AGENTS.md`](../AGENTS.md) sanctioned-primitives guidance (the former
[`claude-code-harness.md`](claude-code-harness.md) body is a **retirement
tombstone** only), agent fences default to plain `git` / `gh` plus in-tree
read/build tools:

- plain `git` + `gh` for worktree, commit, push, and PR lifecycle;
- `oya-tooling-agent-read` or equivalent in-tree composed read primitives when present.

External harness locks (grit/icm/OMC claim cycles) are retired (ADR-0116 /
ADR-0619). For everything else, §2 applies.

## 2. When direct `git` / `gh` is justified

Direct invocation is **permitted** by any operator (agent or human) when
**all three** conditions hold:

   intended operation.
2. Inventing a wrapper would be over-engineering — this is a one-shot
   operation (< 5 invocations per 30 days across the repo).
   ```sh
     -i high \
     -k "git,<context>"
   ```

If any of (1), (2), (3) fails, the invocation is **not** permitted and
the lane refuses.

## 3. Cutover-bootstrap window (time-bounded canonical extension)

§8), there is a documented **bootstrap window** during which raw git /
gh is permitted *for the cutover work itself* without per-invocation
rationale:

- Window: from the cutover branch creation through the wave-gate sign-off
  for M01-P08.
  session start:
  ```sh
    -c "M01-P08 cutover bootstrap session" -i critical -k "cutover,bootstrap"
  ```
- After M01-P08 sign-off, the bootstrap window sunsets and the
  per-invocation rule (§2) applies uniformly. No exception clause survives
  the sunset; the bootstrap window is canonical because it is closed-form
  (named milestone sunset + audit-emit on every session).

The cutover bootstrap window is the only multi-invocation extension; it
is canonical because of the sunset clause, not despite it.

## 4. Revised lane semantics

> **Wave-A bootstrap (ADR-0116):** `git` and `gh` are **permitted in agent
> workflow** during the Wave-A bootstrap period while the webhook receiver is
> not yet deployed. This is the documented fallback per
> [ADR-0116](../decisions/ADR-0116-retire-external-agent-coordination-tooling.md)
> §Temporary seam. CI guard `F-FORBIDDEN-PRIMITIVES-CI-GUARD` is **pending
> Wave-B webhook receiver** deployment; until then, the per-invocation

Lane: `oya-governance-banned-primitives` (existing; Wave-A: catches
**undocumented** `git` / `gh` invocations inside agent-instruction
sections; Wave-B: `F-FORBIDDEN-PRIMITIVES-CI-GUARD` will add CI-level
blocking once webhook receiver is deployed per ADR-0116).
Meta-policy: ADR-0133 (chained-enforcement planning contract, pending).

| Pattern | Verdict |
|---|---|
| `git <cmd>` inside fence during cutover-bootstrap window with session-level store | **PASS** |
| `git <cmd>` inside fence during Wave-A bootstrap per ADR-0116 fallback note | **PASS** (with session-level rationale) |
| `git <cmd>` outside any fence (plain prose, human-facing) | **PASS** |
| `gh <cmd>` — same rules as `git` | as above |
| `git --no-verify <cmd>` | **FAIL** unconditionally (per forbidden-operations.json FO-02) |
| `gh pr merge` without `## Code Review` section | **FAIL** (per `guard-pr-merge-review.mjs` hook) |

## 5. Migration-candidate flow

Repeat invocations (≥ 5 of the same shape in 30 days) auto-emit a
migration-candidate row in
[`docs/MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md):

```
| MFL-NNNN | Direct `git <pattern>` invoked N times in 30d | tooling-gap | mechanical | extend `oya-tooling-agent-read` w/ <op> | <date> |
```

The owning team (default: `axis-foundry`) opens an issue to grow the
sanctioned-primitive surface. The lane
patterns that exceed the threshold.

## 6. Pragmatic gh patterns

Several common `gh` patterns currently lack a sanctioned wrapper. Each
is a candidate for `oya-tooling-agent-read` once usage justifies:

| Pattern | Current path | Migration target |
|---|---|---|
| `gh pr checks <num>` | direct `gh` | `oya-tooling-agent-read pr checks <num>` |
| `gh api repos/.../pulls/<n>/comments` | direct `gh` | `oya-tooling-agent-read pr comments <n>` |
| `gh run list` | direct `gh` | `oya-tooling-agent-read run list` |
| `gh issue view <num>` | direct `gh` | `oya-tooling-agent-read issue view <num>` |

When a pattern crosses the 5-per-30d threshold, the migration row
is auto-created.

## 7. Forbidden operations

These are **never** permitted, regardless of rationale:

| Operation | Why forbidden |
|---|---|
| `git push --force` to `main` | forbidden-operations.json FO-03 |
| `git reset --hard` on someone else's work | forbidden-operations.json FO-03 |
| `git --no-verify` (skip hooks) | forbidden-operations.json FO-02 |
| `gh pr merge` without `## Code Review` | merge-gate hook |
| `git config user.email` mutation | CLAUDE.md / AGENTS.md user-machine guard |
| Editing `~/.claude/` or `~/.codex/` from a project session | user-machine boundary |
| Force-push that destroys an in-flight reviewer-agent verdict | merge integrity |
| Cross-repo destructive `gh` ops (delete, transfer) | scope guard |

## 8. Commit and PR conventions

While the commit-message standard lives in a separate file (deferred
per INDEX.md §Out-of-scope), the **floor** rules below apply:

1. Commits MUST be signed (gpg or sigstore-keyless via gitsign).
2. Commit subjects ≤ 72 characters; imperative present tense.
3. Co-author trailers honored for paired work or agent-collaborative
   work (use the harness's documented co-author trailer convention).
4. Branch names: `<type>/<short-slug>-<issue-num>` (e.g.,
   `feat/foundry-rag-tenant-gate-1245`).
5. PR shape follows [`docs/AGENTS.md`](../AGENTS.md) §PR shape (5 traceability
   H2 sections plus automated `## Code Review` verdict for merge-ready PRs).

## 9. Risky actions — confirmation contract

Per [`docs/AGENTS.md`](../AGENTS.md) §Boundaries, risky actions MUST be
confirmed with the user **before execution** unless the user has
pre-authorized the scope:

- `git push --force` (any branch).
- `git reset --hard`.
- Package downgrade in `Cargo.lock` or `package-lock.json`.
- Migration to shared infra.
- Sending external messages (Slack, email, regulator portal).

The default agent stance is **decline**; require explicit user authorization
scoped to the action.

## 10. Anti-patterns

   refuses.
2. **A "convenience" alias that wraps `git`** without exposing the
   rationale field. Just call `git` and log the rationale.
3. **Re-using a stale rationale row** for a different operation type.
   One rationale per operation class per session.
4. **Cargo-culting `git push --force-with-lease`** to dodge the lane.
   Both `--force` and `--force-with-lease` are caught.
5. **Bypassing the merge-gate hook** via `gh pr merge --admin` or web UI.
   Admin merges require an ADR-tracked extension (named principal +
   audit-emit on every invocation).

## 11. Sources scanned

- [`docs/AGENTS.md`](../AGENTS.md) §Boundaries + §PR shape; [`docs/MASTERPLAN.md`](../MASTERPLAN.md) projection + [`/specs/masterplan.json`](../../specs/masterplan.json).
- [`decision-principles.json`](../../specs/decision-principles.json) + [`forbidden-operations.json`](../../specs/forbidden-operations.json).
- [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md), [ADR-0619](../decisions/ADR-0619-zero-live-context-retirement-of-external-agent-harness-brand.md).
- [`docs/standards/claude-code-harness.md`](claude-code-harness.md) (retirement tombstone only).
- [Conventional Commits](https://www.conventionalcommits.org/) (advisory; not adopted verbatim).
