---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-005-slo-engine-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-005: oya-observability-slo-engine-usecase

## Intent

Per ADR-0106 rename `application` → `usecase` for new crates. Orchestrators: read OpenSLO manifests via `SloTargetRepository`; query Prometheus via `PrometheusClient`; compute verdicts via domain; emit verdicts via `EligibilityVerdictEmitter`. Pure orchestration over kernel ports; no protocol or backend logic.

## ChangeSet boundary

One new Rust crate; consumes kernel + domain. Mock-port integration tests prove orchestration flow without infrastructure.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-usecase/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/evaluate_use_case.rs` | create |
| `.../src/rollback_use_case.rs` | create |
| `.../src/backfill_use_case.rs` | create — per `backfill-replay.md` contract |
| `Cargo.toml` (workspace) | update |
| `microservices/observability/catalog/oya-observability-slo-engine-usecase.yaml` | create |

## Crate Naming

```
NAME: oya-observability-slo-engine-usecase
JUSTIFICATION:
- microservice = observability; bc-tokens = slo-engine
- layer = usecase per ADR-0106 (replaces legacy 'application' for new code)
- exemptions: none
```

## Code Shape

```rust
// src/evaluate_use_case.rs
use oya_observability_slo_engine_kernel::*;
use oya_observability_slo_engine_domain as dom;

pub struct EvaluateUseCase<R, P, E> {
    repo: R,
    prom: P,
    emitter: E,
}

impl<R: SloTargetRepository, P: PrometheusClient, E: EligibilityVerdictEmitter> EvaluateUseCase<R, P, E> {
    pub async fn run(&self, ms: &str, sha: &Sha, env: Environment) -> Result<EligibilityVerdict, KernelError> {
        let targets = self.repo.load_for_microservice(ms).await?;
        let tenant = MimirTenant::reserved("oya-ci");
        let mut worst = Verdict::Eligible;
        let mut snapshot = BurnRateSnapshot::default();
        for t in &targets {
            let s = self.collect_snapshot(&t, &tenant).await?;
            let v = dom::classify(&s);  // returns Verdict per burn-rate thresholds
            if v.priority() > worst.priority() { worst = v; }
            snapshot = s;
        }
        let verdict = EligibilityVerdict {
            microservice: ms.into(),
            source_sha: sha.to_string(),
            target_env: env,
            verdict: worst,
            burn_rate_snapshot: snapshot,
            evaluated_at: chrono::Utc::now(),
            ..Default::default()
        };
        self.emitter.emit(&verdict).await?;
        Ok(verdict)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-usecase --all-features
cargo nextest run -p oya-observability-slo-engine-usecase --all-features
cargo clippy -p oya-observability-slo-engine-usecase -- -D warnings
```

## Test Plan

Per PHASE-01 usecase class: 1 test per use case (happy + 2 sad paths) + ≥3 against mocked ports. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_evaluate_happy_path` | mocked ports return clean signal ⇒ Eligible |
| `test_evaluate_burn_breach` | mocked ports return fast-burn ⇒ Held |
| `test_evaluate_no_manifest` | repo returns empty ⇒ Rejected |
| `test_rollback_happy_path` | reads prior pointer + advances back |
| `test_backfill_window_bounded` | per `backfill-replay.md` contract |

## Halt Conditions

- Any direct backend call (Mimir HTTP, file I/O) — refactor to adapter
- Verdict-priority inversion — fail fast

## Next IP

[`IP-006-slo-engine-adapter.md`](IP-006-slo-engine-adapter.md)

## References

- ADR-0106 (application→usecase rename)
- `backfill-replay.md`
