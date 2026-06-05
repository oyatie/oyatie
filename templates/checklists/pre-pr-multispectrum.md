---
template_class: PrePrChecklist
status: Accepted
date: 2026-05-14
authority: docs/standards/multispectrum-review.md
machine_mirror: /specs/multispectrum-review.json
purpose: |
  Runnable gate every PR/changeset/session MUST pass before claiming "done" or
  opening a PR. Acts as the iterative review loop: find issues per facet →
  fix → re-review → repeat until all required facets green per change_class.
---

# Pre-PR Multispectrum Checklist

> **Discipline rule:** No claim of "complete", no PR open, no PR-ready label
> until every required facet for the declared `change_class` is GREEN.
> A facet is GREEN when:
> - `deep` rigor: at least one finding (or explicit `null_finding_reason`)
>   AND every trigger question answered in evidence.
> - `scan` rigor: `considered: true` recorded.
> - `skip` rigor: `considered: false` AND `not_applicable_reason` recorded.

## Step 1 — declare change_class

Mark exactly ONE row. The machine spec enumerates seven classes;
the rigor matrix is read from `/specs/multispectrum-review.json#change_classes`.

- [ ] `CC-1_kernel_public_api` — touches public API of a kernel crate
- [ ] `CC-2_adapter_or_infrastructure` — adapter or infrastructure crate
- [ ] `CC-3_application_or_domain` — use-case or business logic
- [ ] `CC-4_refactor_pure` — rename/move only, no semantic change
- [ ] `CC-5_doc_only` — documentation, ADR, standard
- [ ] `CC-6_generated_or_vendored` — generated code or vendored upstream
- [ ] `CC-7_test_or_fixture` — test, bench, fixture only

## Step 2 — capture git context

```
git_sha:          <40-char SHA of HEAD at time of evidence>
freshness_unix:   <integer epoch seconds, current time>
session_id:       <session UUID or agent id>
multispectrum_spec_version: 1.0.0
```

## Step 3 — facet review

For each facet, answer ALL trigger questions per the rigor required by your
`change_class`. Trigger questions from `/specs/multispectrum-review.json#facets`.

### F1 Linus critique

- [ ] Trigger questions answered.
- Findings (real or `null_finding_reason: "..."`):
  - …
- FixupTasks emitted (with priority + reasoning):
  - …

### F2 Hyperscaler critique

- [ ] Trigger questions answered.
- Findings:
  - …
- FixupTasks:
  - …

### F3 Adversarial critique

- [ ] **Failing fixture added** (required for new lane sub-check or new
  kernel public API): yes / no / not_applicable_reason.
- [ ] Failing fixture ID + diagnostic asserted: `<fixture_id>` →
  diagnostic: "<…>".
- [ ] Passing fixture ID: `<fixture_id>`.
- [ ] Meta-test asserts both: yes.

### F4 Ergonomic critique

- [ ] Trigger questions answered.
- New-contributor minutes-of-cognitive-load estimate: `<integer>`.
  (Must be ≤ 15 for `CC-1`; ≤ 30 for `CC-2`.)
- Findings:
  - …

### F5 Quality critique

- [ ] Trigger questions answered.
- Findings (code-shape issues independent of stated goal):
  - …
- FixupTasks:
  - …

### F6 Better-alternative critique

- [ ] **At least one alternative named** (mandatory; `alternatives_named` is
  `minItems: 1`).
- Alternatives considered:
  1. …
  2. …
- Selected rationale: "…"
- Rejected reasons (one per non-selected):
  - …

### F7 Security review

- [ ] OWASP class reviewed: `<dos|injection|broken_access|sensitive_data|…|none_applicable>`
- [ ] Severity max: `<none|low|medium|high|critical>`
- [ ] Secrets touched: yes / no.
- [ ] Tenant data touched: yes / no.
- [ ] Audit-chain emission required: yes / no.
- Findings (each: severity + summary + recommended_action + fixuptask_id?):
  - …

## Step 4 — iterative-fix-loop

While ANY required facet is RED (depth not met, mandatory artifact missing,
or fixture absent):

1. Pick the RED facet with highest severity (F7 critical > F7 high > F1 > others).
2. Apply the minimum-correct fix.
3. Re-run the relevant verification (Buck2 test, lane fixture, security scan).
4. Re-evaluate the facet. Update evidence.
5. Loop.

When all required facets are GREEN AND mandatory artifacts present,
proceed to Step 5.

## Step 5 — mandatory artifacts per change_class

Mark each as present (with path + lane evidence).

For `CC-1_kernel_public_api`:
- [ ] ADR cite: `docs/decisions/ADR-<NNNN>-<slug>.md`
- [ ] Failing fixture: `crates/<lane>/tests/fixtures/<sub-check>/failing/...`

For `CC-2_adapter_or_infrastructure`:
- [ ] Request-body limit declaration: `<path>:<line>` (if HTTP boundary)
- [ ] Timeout policy declaration: `<path>:<line>` (if I/O boundary)

For `CC-4_refactor_pure`:
- [ ] Before/after symbol-set parity: Buck2 query output, Rust symbol scan, or reviewer-attached diff result attached.

For `CC-6_generated_or_vendored`:
- [ ] Regeneration source pinned (git_sha + tool version)
- [ ] Supply-chain scan result attached.

For `CC-7_test_or_fixture`:
- [ ] Fixture pair (passing + failing) for new lane sub-checks.

## Step 6 — emit evidence

Write the JSON evidence to:

```
/evidence/multispectrum/<change_id>-<unix-ts>.json
```

Schema: `/specs/multispectrum-review.json#evidence_schema`.

## Step 7 — invoke lane

```
buck2 build //:repo-hygiene-automation-check
```

Refuses to exit 0 unless:
- evidence file exists at the expected path
- JSON validates against the evidence_schema
- rigor matrix satisfied for declared change_class
- mandatory artifacts present
- fixture-pair-coverage check passes for any new lane sub-check

## Step 8 — declare done

Only after the lane exits 0:

```
gh pr ready <pr-number>  # marks the isolated PR lane ready after verification
```

The pre-PR checklist is the **gate**. Skipping it is a process violation
recorded as F-MULTISPECTRUM-VIOLATION-<n> with severity.

## Decision-log row (Linus good-taste)

Special cases eliminated by this checklist:

- "I reviewed it" without saying *how* — replaced by seven named facets,
  each with trigger questions and evidence schema.
- Happy-path-only tests — F3 mandates a failing fixture.
- Silent deferrals — F6 mandates named alternatives; deferrals require
  bounded FixupTasks.
- Cosmetic refactor masquerading as fix — F1 + F5 catch.
- Security gaps slipping through — F7 mandatory OWASP class review.

## References

- `docs/standards/multispectrum-review.md` (human authority)
- `/specs/multispectrum-review.json` (machine truth)
- `/specs/iterative-fix-loop.json` (loop protocol)
- ADR-0092 (first ADR citing this checklist)
- ADR-0069 (active-artifact-contract evidence bundle)
