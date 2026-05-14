---
doc_class: Template
purpose: Engineering design-doc template (Google-style) for non-trivial changes; forces explicit trade-offs before code is written and provides a durable record for future-you.
---

# Design Doc Template

## How to use

Open a design doc **after** the PR/FAQ has approval-in-principle, and
**before** you start writing code beyond a throwaway prototype. The
target audience is a teammate joining the project three months from now,
not the people in today's review.

Sections marked `(required)` cannot be empty when the design doc enters
review.

---

## 1. Metadata `(required)`

- **Title**:
- **Author(s) + accountable owner**:
- **Status**: `draft` | `in-review` | `approved` | `superseded-by <link>`
- **Last updated**:
- **Linked PR/FAQ**:
- **Reviewers (eng + adjacent disciplines)**:

## 2. Context `(required)`

What is the problem? What does the world look like today, and what is
broken or missing? Link to the PR/FAQ for customer-facing framing; this
section explains the engineering reality.

Length budget: ≤ 1 page. If it's longer, link to background docs.

## 3. Goals `(required)`

A short, ordered list of what success looks like. Each goal should be
measurable (latency budget, error rate, lines-of-code-deleted, etc.).

## 4. Non-goals `(required)`

What this design explicitly does **not** address. Phrase as "we are
choosing not to solve X here because Y." Listing non-goals prevents the
proposal from being judged against the wrong yardstick.

## 5. Constraints

External constraints (regulatory, contractual, dependency LTS, security
review boundaries) that any acceptable design must respect.

## 6. Considered alternatives `(required)`

For each non-trivial decision, list at least two real alternatives. For
each, state:
- The shape of the solution in 2–4 sentences.
- Why we rejected it (concretely).

A design doc with only one option is not finished. A reviewer should
finish this section with no surprises about the chosen direction.

## 7. Proposed design `(required)`

The chosen approach. Use enough diagrams + pseudocode that a reviewer
can build a mental model without reading the implementation.

Sub-sections to cover:
- Data model + invariants.
- Public API / interface surface.
- Failure modes + recovery.
- Observability (what gets logged, what gets metricked, what pages).
- Security posture (threats considered + mitigations).
- Compatibility (backwards/forwards; migration plan if any).

## 8. Rollout plan `(required)`

How this lands. Default cadence:
1. Behind a feature flag, off by default.
2. Internal cohort (specific users / tenants).
3. Gradual ramp with concrete rollback trigger.
4. GA (no flag).

State the rollback trigger explicitly. "Page rate > X on dashboard Y
during ramp ⇒ revert to flag=off."

## 9. Test plan `(required)`

What level of testing satisfies done-criteria for this design. Be
explicit about anything not covered by the default unit + integration
suite (load tests, chaos, end-to-end with real upstreams, etc.).

## 10. Open questions

Unresolved decisions with owner + deadline. Anything here at review time
blocks "approved" status.

## 11. References

Linked ADRs, prior art, vendor docs, related design docs. Cite specific
sections, not just root URLs.

---

## Review checklist (reviewer adds initials)

- [ ] Goals + non-goals are measurable.
- [ ] At least two alternatives considered for each major decision.
- [ ] Failure modes + observability + security all addressed.
- [ ] Rollout plan names the rollback trigger.
- [ ] Test plan covers more than "the existing CI passes."
