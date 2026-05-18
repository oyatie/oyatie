---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-002-router-domain
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

# IP-002: oya-foundry-providers-router-domain

## Intent

The routing algebra: capability-fit × cost × latency × residency × health weighting. Pure functions; no I/O; tested against canonical worked examples.

## ChangeSet boundary

One new crate `microservices/foundry/src/crates/oya-foundry-providers-router-domain/`. Depends only on `oya-foundry-providers-router-kernel`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/algebra.rs` | create — routing weighting algebra |
| `.../src/capability_fit.rs` | create — match capability-profile vs vendor capability catalog |
| `.../src/residency.rs` | create — per-pack permitted-vendor matrix evaluation |
| `.../src/health_scoring.rs` | create — convert ProviderHealthSnapshot to score 0..1 |
| `.../src/cost_normalization.rs` | create — convert cost-per-1K-tokens to per-call cost prediction |

## Crate Naming

```
NAME: oya-foundry-providers-router-domain
JUSTIFICATION:
- microservice = foundry-providers
- bc-tokens = router
- layer = domain (ADR-0105 13-value; business rules; pure)
- exemptions claimed: none
```

## Algorithm Skeleton

```rust
pub fn score_candidate(
    candidate: &ProviderCandidate,
    request: &RoutingRequest,
    health: &ProviderHealthSnapshot,
) -> Score {
    let fit = capability_fit::score(&request.capability_profile, &candidate.capability_catalog);
    if fit < 0.5 { return Score::INELIGIBLE; }

    let residency_ok = residency::permitted(&request.pack, candidate.vendor, candidate.transport, &candidate.region);
    if !residency_ok { return Score::INELIGIBLE; }

    let cost_score = cost_normalization::score(candidate, request);  // lower cost ⇒ higher score
    let latency_score = latency_score(candidate.p99_latency_ms, request.constraints.latency_ceiling_p99_ms);
    let health_score = health_scoring::score(health);

    let weighted = 0.4 * fit + 0.25 * cost_score + 0.2 * latency_score + 0.15 * health_score;
    Score(weighted)
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_capability_fit_exact_match` | fit = 1.0 for exact-match |
| `test_capability_fit_partial` | fit ∈ (0, 1) for partial |
| `test_residency_pack_kr_anthropic_kr_ok` | per `policy/data-residency.md` |
| `test_residency_pack_kr_openai_us_deny` | deny per `policy/data-residency.md` |
| `test_residency_pack_eu_openai_eu_ok` | post-SCC permitted |
| `test_residency_pack_us_healthcare_subscription_deny` | per `policy/data-residency.md` |
| `test_health_score_demoted_zero` | demoted vendor scores 0 |
| `test_cost_score_decreasing_in_cost` | monotonic |
| `test_router_decision_canonical_example` | worked example from PRD |
| `test_in_house_prefer_when_capability_fit_and_cost` | ADR-0026 rule |

## Acceptance Gates

```bash
cargo check -p oya-foundry-providers-router-domain --all-features
cargo build -p oya-foundry-providers-router-domain --all-features
cargo clippy -p oya-foundry-providers-router-domain --all-features -- -D warnings
cargo nextest run -p oya-foundry-providers-router-domain --all-features
cargo deny check
cargo doc -p oya-foundry-providers-router-domain --no-deps
```

## Next IP

[`IP-003-router-usecase.md`](IP-003-router-usecase.md)
