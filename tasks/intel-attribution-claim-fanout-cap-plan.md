# Plan: intel-attribution-claim-fanout-cap

## Crate
`intelligence-attribution-kernel`

## Objective
Extend `plan_attribution` with deterministic per-claim citation fanout control:
- Stable within-claim source ordering
- `source_id` dedup within a claim (deny on duplicates, not silent drop)
- Max-citations-per-claim guard via new `AttributionDenialKind::ClaimCitationFanoutExceeded`

## Steps

1. Add `AttributionDenialKind::ClaimCitationFanoutExceeded` variant to enum
2. Add `max_citations_per_claim` field to `AttributionRequest`
3. Add per-claim fanout validation in `invalid_input_reasons` for duplicate source_ids in a claim
4. Add per-claim fanout cap check in `plan_attribution` after sensitive-citation check
5. Ensure `planned_citations` stable within-claim ordering (already uses `sorted_unique`)
6. Add 3+ new tests

## Acceptance
- `ClaimCitationFanoutExceeded` denial variant present
- Per-claim fanout cap enforced; returns Denied with `policy_evidence_ref` + `validation:intelligence-attribution-claim-fanout`
- Within-claim ordering deterministic
- Existing tests remain green
- 3+ new `#[cfg(test)]` cases
