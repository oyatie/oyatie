---
doc_status: drafted
---
# ADR-XXXX — <title>

- Status: Proposed | Accepted | Superseded
- Phase: M02-P03
- Date: YYYY-MM-DD
- Authors: <agent-id or human>
- Supersedes: -
- Superseded-by: -

## Context

What forces are in play (technical, organizational, regulatory)? Cite the
governing plan (`ralplan-*-YYYY-MM-DD`) and parent phase INDEX. Reference any
existing ADRs that bound the decision.

## Decision

The concrete decision in active voice. Single, unambiguous sentence in the
opening; supporting paragraphs may follow. Identify the principle the decision
upholds (one of the 12-layer enum, the BNF v4.1 crate name contract, or a named
clean-arch invariant).

## Consequences

- Positive consequences (what is now possible / clearer / safer).
- Negative consequences (added surface, successor-IP work, risk).
- Operational implications (runbooks, dashboards, alerts, on-call).

## Linus good-taste row

| # | Question | Answer |
|---|---|---|
| 1 | What special case is eliminated? | <answer> |
| 2 | Where was the special case before this ADR? | <pointer> |
| 3 | What does the data structure look like after? | <one-line> |
| 4 | Could this be done by simplifying instead of adding? | <yes/no + why> |

## Citation contract

This ADR is citation-eligible by `grit-claim-intent-gate` (regex
`ADR-[0-9]{4}`). The number is allocated at PR time from
`docs/decisions/INDEX.md`.
