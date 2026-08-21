# Spec: intel-attribution-claim-fanout-cap

## Summary
Extend `plan_attribution` in `intelligence-attribution-kernel` with deterministic per-claim citation fanout control.

## Slice
Pure std-only, no I/O. All changes inside `intelligence-attribution-kernel`.

## New Behavior

### `AttributionDenialKind::ClaimCitationFanoutExceeded`
Added variant. Returned when any claim's deduplicated source_ids count exceeds `max_citations_per_claim`.

### `AttributionRequest::max_citations_per_claim`
New field (`usize`). Valid range: `1..=max_citations`. Validation rejects 0 or values exceeding `max_citations`.

### Duplicate source_id within a claim
If any claim contains repeated `source_id` entries (before dedup), `plan_attribution` returns `Denied` with `InvalidInput`. This surfaces the problem explicitly rather than silently ignoring duplicates.

### Per-claim fanout cap enforcement
After input validation and sensitive-citation check, before rendering, iterate claims. If any claim's deduplicated source_ids count exceeds `max_citations_per_claim`, return:
```
AttributionReport {
    status: Denied,
    denial_kind: Some(ClaimCitationFanoutExceeded),
    evidence_refs: sorted_unique([policy_evidence_ref, "validation:intelligence-attribution-claim-fanout"])
}
```

### Within-claim source ordering
`planned_citations` uses `sorted_unique` on `source_ids` — lexicographic order by `source_id`. Deterministic and stable.

## Acceptance Criteria
- `ClaimCitationFanoutExceeded` variant exists on `AttributionDenialKind`
- Fanout cap enforced; report contains `validation:intelligence-attribution-claim-fanout` evidence
- Duplicate source_ids in one claim → `InvalidInput` denial
- Ordering: within-claim citations follow lexicographic source_id order
- All existing tests pass
- 3+ new `#[cfg(test)]` cases: fanout cap exceeded, dedup detection, ordering stability
