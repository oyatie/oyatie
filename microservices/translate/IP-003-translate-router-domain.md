---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-003-translate-router-domain
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-translate-router-domain

## Intent

Pure routing algebra + placeholder/plural preservation logic. Zero I/O. Implements the math of ADR-TRANSLATE-0001:

```
score(candidate, request) = w_fit  · capability_fit_score(candidate, request)
                          - w_cost · normalize(cost_per_1k_chars_usd)
                          - w_lat  · normalize(p99_latency_ms)
                          + w_avail · availability_rolling_15m
                          + w_pref · (prefer_in_house && candidate.vendor == InHouse)
```

Subject to `residency_compliant == true` (HARD; default-deny per ADR-TRANSLATE-0004) and `language_pair_supported == true` and `quality_tier_supported == true`.

Also implements:
- ICU MessageFormat re-parse validation (FR-15).
- CLDR plural-rule mapping per target locale (FR-16).
- Formality + gender policy mapping per locale (FR-17).
- Placeholder allow-list extraction + diff against output.

## ChangeSet boundary

One new Rust crate at `microservices/translate/src/crates/oya-translate-router-domain/`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — depends on kernel + `icu_messageformat`, `icu_plurals`, `icu_locid` |
| `.../src/lib.rs` | create |
| `.../src/scoring.rs` | create — `score()` + weight constants |
| `.../src/residency.rs` | create — residency-constraint enforcement |
| `.../src/placeholders.rs` | create — ICU MessageFormat extraction + diff |
| `.../src/plurals.rs` | create — CLDR plural-rule mapper |
| `.../src/formality.rs` | create — per-locale formality + gender policy |
| `.../src/leverage.rs` | create — TM leverage scoring (minhash-LSH similarity bucketing per ADR-TRANSLATE-0002) |

## Algorithm — Score (Excerpt)

```rust
pub fn score(candidate: &EngineCandidate, req: &TranslationRequest, weights: &Weights) -> f64 {
    let fit  = weights.fit  * candidate.capability_fit_score;
    let cost = weights.cost * normalize_cost(candidate.cost_per_1k_chars_usd);
    let lat  = weights.lat  * normalize_latency(candidate.p99_latency_ms);
    let av   = weights.av   * candidate.availability_rolling_15m;
    let pref = if req.constraints.prefer_in_house && candidate.vendor == Vendor::InHouse {
        weights.pref
    } else { 0.0 };
    fit - cost - lat + av + pref
}

pub fn select(req: &TranslationRequest, candidates: &[EngineCandidate], policy: &TenantPolicy) -> Result<RoutingDecision, RouterError> {
    // 1. Residency filter (HARD; default-deny)
    let residency_filtered: Vec<_> = candidates.iter()
        .filter(|c| policy.residency.permitted_vendors.contains(&c.vendor)
                  && policy.residency.permitted_regions.contains(&c.region))
        .collect();
    if residency_filtered.is_empty() {
        return Err(RouterError::NoResidencyCompliantEngine);
    }

    // 2. Capability filter (language pair, quality tier, content class)
    let cap_filtered: Vec<_> = residency_filtered.iter()
        .filter(|c| c.language_pair_supported && c.eligible)
        .copied()
        .collect();
    if cap_filtered.is_empty() {
        return Err(RouterError::NoCapabilityCompliantEngine);
    }

    // 3. Score + pick top
    let weights = Weights::for_quality_tier(req.quality_tier);
    let best = cap_filtered.into_iter().max_by(|a, b|
        score(a, req, &weights).partial_cmp(&score(b, req, &weights)).unwrap_or(Ordering::Equal)
    ).ok_or(RouterError::Empty)?;

    Ok(RoutingDecision {
        decision_id: gen_decision_id(),
        selected_vendor: best.vendor,
        selected_region: best.region.clone(),
        residency_compliant: true,
        reason: explain(best, req, &weights),
        candidate_set: candidates.to_vec(),
    })
}
```

## Placeholder + Plural Preservation

Per FR-15 + FR-16:
- Extract placeholders from source via `icu_messageformat`; record positions + names + types.
- After MT, re-parse target with `icu_messageformat`; assert same set of placeholder names + types.
- For plural arms (`{count, plural, one {…} other {…}}`), assert target locale's CLDR plural categories are populated (one/few/many/other per locale).
- Per `unicode/cldr` JSON.

## Test Plan

| Test | Verifies |
|---|---|
| `test_residency_filter_default_deny` | non-permitted vendor never appears in decision |
| `test_score_prefers_in_house_when_tied` | `pref` weight flips to in-house on tie |
| `test_quality_tier_premium_weights_quality_over_cost` | weight matrix |
| `test_placeholder_diff_rejects_mismatch` | placeholder rename rejected |
| `test_icu_messageformat_reparse_target` | re-parse confirms target valid |
| `test_cldr_plural_arms_complete_per_locale` | one/few/many/other complete for target locales |
| `test_no_residency_compliant_returns_error` | error not panic |
| `test_canonical_worked_example_KR_KO_EN` | reference per-pack worked example deterministic |
| `proptest_score_is_monotonic_in_fit` | property: increasing fit → non-decreasing score |
| `proptest_residency_filter_is_idempotent` | property |

## Halt Conditions

- Routing function returns a non-residency-compliant candidate (HARD).
- Placeholder mismatch passes through.
- CLDR plural arm dropped from target.

## Next IP

[`IP-004-translate-router-usecase-and-api.md`](IP-004-translate-router-usecase-and-api.md)
