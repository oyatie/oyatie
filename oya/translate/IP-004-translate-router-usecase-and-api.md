---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-004-translate-router-usecase-and-api
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-translate-router-usecase + oya-translate-router-api

## Intent

`usecase` composes router (domain) + TM leverage + termbase enforcement + engine invoker + QE + audit emission into one orchestration. `api` defines the protocol-neutral typed contracts used by REST/gRPC.

## ChangeSet boundary

Two new Rust crates:
- `microservices/translate/src/crates/oya-translate-router-usecase/`
- `microservices/translate/src/crates/oya-translate-router-api/`

## File Targets

| Path | Action |
|---|---|
| `oya-translate-router-usecase/src/lib.rs` | create |
| `oya-translate-router-usecase/src/translate_uc.rs` | create — primary use-case |
| `oya-translate-router-usecase/src/decide_uc.rs` | create — dry-run decision |
| `oya-translate-router-usecase/src/batch_translate_uc.rs` | create |
| `oya-translate-router-api/src/lib.rs` | create |
| `oya-translate-router-api/src/dto.rs` | create — `TranslateRequestDto`, `TranslateResponseDto`, etc. |

## Use-case Orchestration (Excerpt)

```rust
pub struct TranslateUseCase<R, T, B, Q, I, P, E>
where R: EngineRouter, T: TmLeverageQuery, B: TermbaseQuery,
      Q: QualityEstimator, I: TranslateInvoker, P: TenantPolicyRepository,
      E: EventEmitter
{
    pub router: R,
    pub tm: T,
    pub termbase: B,
    pub qe: Q,
    pub invoker: I,
    pub policy_repo: P,
    pub emitter: E,
}

impl<R, T, B, Q, I, P, E> TranslateUseCase<R, T, B, Q, I, P, E> {
    pub async fn translate(&self, req: TranslationRequest) -> Result<TranslationResult, RouterError> {
        // 1. Load tenant policy (residency pack + engine whitelist + cost ceiling)
        let policy = self.policy_repo.load(&req.tenant_id, &req.pack).await?;

        // 2. TM leverage opt-in
        if req.use_tm {
            if let Some(lm) = self.tm.lookup(&req.tenant_id, req.project_id.as_deref(),
                &req.source_lang, &req.target_lang, &req.text).await?
            {
                if matches!(lm.match_kind, MatchKind::Exact100 | MatchKind::Ice) {
                    let result = build_tm_result(&req, &lm);
                    self.emitter.emit_translation_completed(&result).await?;
                    return Ok(result);
                }
                // Fuzzy match attached for engine prompt context
            }
        }

        // 3. Termbase enforcement constraints
        let terms = self.termbase.enforce(&req.tenant_id, req.project_id.as_deref(),
            &req.source_lang, &req.target_lang, &req.text).await?;

        // 4. Router decide (residency-bound; default-deny)
        let decision = self.router.decide(&req).await?;
        self.emitter.emit_engine_routed(&decision).await?;
        if !decision.residency_compliant {
            return Err(RouterError::ResidencyViolation);
        }

        // 5. Invoke selected engine (via adapter)
        let mut result = self.invoker.translate(&req).await?;

        // 6. Placeholder + plural validation (domain crate)
        domain::placeholders::validate_preserved(&req.text, &result.translated_text)?;

        // 7. Termbase post-check (re-prompt or annotate violations)
        if !terms.is_empty() {
            domain::termbase::enforce_post(&mut result, &terms)?;
        }

        // 8. QE sampling per ADR-TRANSLATE-0003 (configured per content_class)
        if should_sample_qe(&req) {
            let qe = self.qe.score(&req.text, &result.translated_text,
                &req.source_lang, &req.target_lang, req.content_class).await?;
            result.qe_score = Some(qe);
            self.emitter.emit_quality_estimated(&req, &result).await?;
        }

        // 9. EU AI Act Art. 50 + Art. 13 disclosure when jurisdiction = EU
        if req.pack.starts_with("eu") || policy.eu_ai_act_disclosure_required() {
            self.emitter.emit_eu_ai_act_disclosure(&req, &result).await?;
        }

        // 10. Final TranslationCompleted seal (Ed25519 envelope already in result)
        self.emitter.emit_translation_completed(&result).await?;

        Ok(result)
    }
}
```

## API DTO Surface (Excerpt)

```rust
// dto.rs
#[derive(Serialize, Deserialize)]
pub struct TranslateRequestDto {
    pub source_lang: String,
    pub target_lang: String,
    pub text: String,
    pub content_class: String,    // serialized ContentClass enum
    pub quality_tier: String,
    pub project_id: Option<String>,
    pub use_tm: bool,
    pub constraints: Option<RoutingConstraintsDto>,
}

#[derive(Serialize, Deserialize)]
pub struct TranslateResponseDto {
    pub decision_id: String,
    pub translated_text: String,
    pub engine: String,
    pub model_id: String,
    pub region: String,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub request_hash: String,
    pub response_hash: String,
    pub envelope_signature: String,
    pub evidence_ref: String,
    pub leverage_match: Option<LeverageMatchDto>,
    pub qe_score: Option<QualityScoreDto>,
    pub residency_compliant: bool,
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_tm_exact_short_circuits_engine_call` | exact/ICE bypasses engine |
| `test_residency_violation_aborts_before_invoke` | invoker not called |
| `test_engine_routed_event_emitted_on_every_decide` | event count = decide count |
| `test_eu_pack_emits_eu_ai_act_disclosure` | event present when pack=eu |
| `test_qe_sampled_per_content_class_policy` | QE called per sampling rule |
| `test_dto_serde_roundtrip` | DTO ↔ entity stable |
| `test_batch_fan_out_concurrency_cap` | ≤ 16 concurrent per job |

## Halt Conditions

- Any use-case path skips residency check.
- Any disclosure event missed for EU pack.
- Use-case introduces I/O directly (must go through ports).

## Next IP

[`IP-005-translation-memory-stack.md`](IP-005-translation-memory-stack.md)

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/translate/IP-004-translate-router-usecase-and-api.md:19` - `usecase` composes router (domain) + TM leverage + termbase enforcement + engine invoker + QE + audit emission into one orchestration. `api` defines the protocol-neutr...; `microservices/translate/IP-004-translate-router-usecase-and-api.md:57` - // 1. Load tenant policy (residency pack + engine whitelist + cost ceiling).
