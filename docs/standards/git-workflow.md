---
purpose: Pragmatic git/gh workflow per MASTERPLAN Directive 12. Defines the grit-first default, when direct git / gh is justified, the icm rationale-logging contract, the cutover-bootstrap window (time-bounded canonical extension with named sunset).
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
  Pragmatic git/gh workflow per MASTERPLAN Directive 12. Defines the grit-first
  default, when direct git / gh is justified, the icm rationale-logging contract,
  the cutover-bootstrap window (time-bounded canonical extension with named
  sunset at M-CC-P01 sign-off), and the revised banned-primitives lane
  semantics (catch *undocumented* invocations only).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
enforced_by: oya-foundry-fitness-banned-primitives
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

This standard codifies [`.omc/plans/MASTERPLAN.md`](../../.omc/plans/MASTERPLAN.md)
**Directive 12 — Pragmatic git/gh**: direct `git` / `gh` invocation is
**permitted** when the documented rationale is logged and no sanctioned
primitive exists. This standard names the rationale flow, the migration
trigger, and the lane semantics.

## 1. The default surface

Per [`claude-code-harness.md`](claude-code-harness.md) §1, agent fences
default to the **sanctioned-primitive triad**:

- [`grit`](https://github.com/rtk-ai/grit) — claim, work, done, merge-queue.
- [`icm`](https://github.com/rtk-ai/icm) — persistent memory across sessions.
- `oya-tooling-agent-read` — in-tree composed read primitives.

The triad covers ≥ 80% of agent-side git / gh use cases. For everything
else, §2 applies.

## 2. When direct `git` / `gh` is justified

Direct invocation is **permitted** by any operator (agent or human) when
**all three** conditions hold:

1. No grit / icm / `oya-tooling-agent-read` primitive exists for the
   intended operation.
2. Inventing a wrapper would be over-engineering — this is a one-shot
   operation (< 5 invocations per 30 days across the repo).
3. The operator logs a rationale to icm **BEFORE** the invocation:
   ```sh
   icm store -t direct-tool-invocations \
     -c "<one-line rationale: what op, why grit doesn't cover it>" \
     -i high \
     -k "git,<context>"
   ```

If any of (1), (2), (3) fails, the invocation is **not** permitted and
the lane refuses.

## 3. Cutover-bootstrap window (time-bounded canonical extension)

During the grit/icm agentic-pipeline cutover (M-CC-P01 per MASTERPLAN
§8), there is a documented **bootstrap window** during which raw git /
gh is permitted *for the cutover work itself* without per-invocation
rationale:

- Window: from the cutover branch creation through the wave-gate sign-off
  for M-CC-P01.
- Operators MUST instead log a **once-per-session** rationale to icm at
  session start:
  ```sh
  icm store -t direct-tool-invocations \
    -c "M-CC-P01 cutover bootstrap session" -i critical -k "cutover,bootstrap"
  ```
- After M-CC-P01 sign-off, the bootstrap window sunsets and the
  per-invocation rule (§2) applies uniformly. No exception clause survives
  the sunset; the bootstrap window is canonical because it is closed-form
  (named milestone sunset + audit-emit on every session).

The cutover bootstrap window is the only multi-invocation extension; it
is canonical because of the sunset clause, not despite it.

## 4. Revised lane semantics

Lane: `oya-foundry-fitness-banned-primitives` (revised). It catches
**undocumented** `git` / `gh` invocations inside agent-instruction
sections, **not all invocations**.

| Pattern | Verdict |
|---|---|
| `git <cmd>` inside `<!-- agent-instructions -->` fence, no prior icm-store of the same session | **FAIL** |
| `git <cmd>` inside fence, with matching icm-store row | **PASS** |
| `git <cmd>` inside fence during cutover-bootstrap window with session-level store | **PASS** |
| `git <cmd>` outside any fence (plain prose, human-facing) | **PASS** (advisory only) |
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
`oya-foundry-fitness-direct-tool-rationale` queries icm and flags
patterns that exceed the threshold.

## 6. Pragmatic gh patterns

Several common `gh` patterns currently lack a sanctioned wrapper. Each
is a candidate for `oya-tooling-agent-read` once usage justifies:

| Pattern | Current path | Migration target |
|---|---|---|
| `gh pr view <num>` (read PR body) | direct `gh` + icm-store | `oya-tooling-agent-read pr view <num>` |
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
5. PR shape follows [`docs/AGENTS.md`](../AGENTS.md) §PR shape (5 H2
   sections, `## Code Review` at merge time).

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

1. **`git <cmd>` inside a fence without an icm-store rationale.** Lane
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

- [`.omc/plans/MASTERPLAN.md`](../../.omc/plans/MASTERPLAN.md) §2 Directive 12.
- [`decision-principles.json`](../../specs/decision-principles.json) + [`forbidden-operations.json`](../../specs/forbidden-operations.json).
- [`docs/AGENTS.md`](../AGENTS.md) §Boundaries + §PR shape.
- [`docs/standards/claude-code-harness.md`](claude-code-harness.md).
- [rtk-ai/grit](https://github.com/rtk-ai/grit), [rtk-ai/icm](https://github.com/rtk-ai/icm).
- [Conventional Commits](https://www.conventionalcommits.org/) (advisory; not adopted verbatim).
