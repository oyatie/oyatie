---
doc_class: ReferenceImplementation
title: Translate with TM hit, QE gate, and engine fallback via `oya-translate-router-sdk`
microservice: translate
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-translate
related_adrs: [ADR-0056, ADR-0105, ADR-0131, ADR-0244, ADR-0255]
related_artifacts:
  - microservices/translate/contracts/openapi/translate.yaml
  - microservices/translate/contracts/openapi/translate-stream.yaml
  - microservices/translate/contracts/proto/translate.proto
  - microservices/translate/sdk-plan.md
  - microservices/translate/IP-005-translation-memory-stack.md
  - microservices/translate/IP-007-quality-estimation-stack.md
  - microservices/translate/IP-012-engine-adapter-foundry-runtime.md
doc_status: published
---

# Reference implementation — Translate a UI string with TM lookup, QE gate, and engine fallback via `oya-translate-router-sdk`

Runnable Rust program that submits a UI-string translation, hits the translation-memory cache, falls back to the primary
engine on TM miss, runs the QE (quality-estimation) gate, fails over to the backup engine on QE refusal, refuses with a
Cedar-evidenced denial when the tenant lacks the BYOK credential for the requested engine, and finally walks the cost
attribution and audit-chain anchor. Mirrors the displacement targets called out in the competitor-parity matrix: a tenant
who already uses Google Cloud Translation, AWS Translate, DeepL, or Lokalise should be able to swap the SDK without
losing TM, QE, or per-engine cost telemetry.

## `Cargo.toml`

```toml
[package]
name = "translate-router-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-translate-router-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::Result;
use oya_translate_router_sdk::{
    ContentClass, EngineChoice, EnginePreference, PackId, ProviderCredentialMode,
    QualityEstimationGate, RouteOutcome, RouterClient, RouterConfig, Tenant, TranslateError,
    TranslateRequest, TranslateResponse, TranslationMemoryHit,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    // 1. Connect — tenant pack drives default engine ordering, residency, and minor-protection refusals
    let cfg = RouterConfig::builder()
        .endpoint("https://translate.kr.oyatie.dev/v1".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .pack(PackId::Kr) // tenant pack — drives PIPA + RR PIIPP minor-protection defaults
        .service_account_credentials_path("/etc/oya/translate/sa-creds.json")
        .request_timeout(Duration::from_secs(4))
        .translation_deadline(Duration::from_millis(900)) // UI-string SLO target
        .build()?;

    let client = RouterClient::connect(cfg).await?;
    info!("connected to translate router");

    // 2. Build a request — content_class drives router heuristics:
    //    UiString  ⇒ TM lookup first, then NMT engine, QE threshold 0.85
    //    Marketing ⇒ NMT engine, then LLM-rewrite, QE threshold 0.92
    //    Legal     ⇒ TM strict-match, refusal if QE < 0.95 (forces human-in-the-loop)
    let req = TranslateRequest::builder()
        .source_lang("en")
        .target_lang("ko")
        .text("Save changes")
        .content_class(ContentClass::UiString)
        .glossary_id("acme-software-ui-glossary-v18")
        .preferred_engines(vec![
            EnginePreference::primary(EngineChoice::OyaNmtKr),
            EnginePreference::backup(EngineChoice::DeepLProV2),
            EnginePreference::escalate(EngineChoice::OyaLlmReviewer),
        ])
        .provider_credential_mode(ProviderCredentialMode::PlatformDefault) // ADR-0255 §D-4 default
        .quality_estimation_gate(QualityEstimationGate::Threshold(0.85))
        .audit_chain_emission(true)
        .build()?;

    // 3. Submit — the SDK returns a `RouteOutcome` describing every decision the router made
    let outcome = match client.translate(req.clone(), trace.child()).await {
        Ok(outcome) => outcome,
        Err(TranslateError::ProviderCredentialMissing { engine, mode }) => {
            // BYOK is the tenant's responsibility when mode = ByokRequiredByPack
            warn!(
                %engine, ?mode,
                "engine refused — tenant pack requires BYOK credential not present"
            );
            return Ok(());
        }
        Err(TranslateError::CedarRefused { decision_id, principal, action }) => {
            warn!(
                %decision_id, %principal, %action,
                "translation refused by Cedar (likely minor-protection or residency)"
            );
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // 4. Walk the outcome
    match outcome {
        RouteOutcome::TmHit { hit, response } => {
            info!(
                tm_id = %hit.tm_id(),
                tm_match_quality = hit.match_quality(),
                glossary_overrides = hit.glossary_overrides(),
                cost_usd = response.cost_usd(),
                latency_ms = response.latency_ms(),
                engine = %response.engine(),
                "TM hit — no engine call"
            );
            log_response(&response);
            assert!(hit.match_quality() >= 0.95, "TM hits below 0.95 must escalate");
        }
        RouteOutcome::EnginePrimaryAccepted { qe_score, response } => {
            info!(
                qe_score,
                engine = %response.engine(),
                cost_usd = response.cost_usd(),
                latency_ms = response.latency_ms(),
                "primary engine accepted by QE gate"
            );
            log_response(&response);
        }
        RouteOutcome::EnginePrimaryRefused {
            primary_engine,
            primary_qe_score,
            backup_response,
        } => {
            warn!(
                %primary_engine,
                primary_qe_score,
                backup_engine = %backup_response.engine(),
                backup_qe_score = backup_response.qe_score().unwrap_or_default(),
                "primary refused by QE; backup engine succeeded"
            );
            log_response(&backup_response);
        }
        RouteOutcome::EscalatedToHuman { evidence_packet } => {
            warn!(
                evidence_uri = %evidence_packet.uri(),
                qe_history = ?evidence_packet.qe_history(),
                "router escalated to human-in-the-loop"
            );
        }
        RouteOutcome::Refused { reason, audit_chain_event_id } => {
            warn!(%reason, %audit_chain_event_id, "router refused");
        }
    }

    // 5. Confirm the audit-chain anchor (BLAKE3 root) — required for SOX/PIPA evidence packets
    let last_audit_anchor = client.last_audit_anchor(trace.child()).await?;
    info!(
        anchor_root = %last_audit_anchor.root(),
        shard = %last_audit_anchor.shard(),
        signed_by = %last_audit_anchor.signed_by(),
        "audit-chain anchor captured for compliance evidence"
    );

    // 6. Pull the per-translation cost-attribution row — feeds finops-portal
    let cost = client.last_cost_attribution(trace.child()).await?;
    info!(
        usd_total = cost.usd_total(),
        usd_engine = cost.usd_engine(),
        usd_glossary = cost.usd_glossary(),
        usd_qe = cost.usd_qe(),
        cost_component = %cost.component(),
        "cost attribution captured"
    );

    Ok(())
}

fn log_response(response: &TranslateResponse) {
    info!(
        translated_text = %response.translated_text(),
        engine = %response.engine(),
        engine_version = %response.engine_version(),
        target_lang = %response.target_lang(),
        cost_usd = response.cost_usd(),
        latency_ms = response.latency_ms(),
        cedar_decision_id = %response.cedar_decision_id(),
        audit_chain_event_id = %response.audit_chain_event_id(),
        "translation response"
    );
}
```

## Run it

```bash
cargo run --release
```

Expected output (TM-hit path, trimmed):
```
INFO  connected to translate router
INFO  TM hit — no engine call tm_id=tm-acme-ui-… tm_match_quality=0.99 glossary_overrides=1 cost_usd=0.0000031 latency_ms=11 engine=tm
INFO  translation response translated_text="변경 사항 저장" engine=tm engine_version=tm-router-0.42.0 target_lang=ko cost_usd=0.0000031 latency_ms=11
INFO  audit-chain anchor captured for compliance evidence anchor_root=blake3-256:… shard=translate.shard-7 signed_by=audit-chain-key-12
INFO  cost attribution captured usd_total=0.0000031 usd_engine=0 usd_glossary=0.0000031 usd_qe=0 cost_component=tm-only
```

Expected output (TM-miss → primary-engine path, trimmed):
```
INFO  primary engine accepted by QE gate qe_score=0.91 engine=oya-nmt-kr cost_usd=0.000019 latency_ms=312
INFO  translation response translated_text="변경 사항 저장" engine=oya-nmt-kr engine_version=oya-nmt-kr-2026.05 …
INFO  cost attribution captured usd_total=0.000019 usd_engine=0.000018 usd_glossary=0.0000003 usd_qe=0.0000007 cost_component=engine-primary
```

Expected output (primary-refused → backup-accepted path, trimmed):
```
WARN  primary refused by QE; backup engine succeeded primary_engine=oya-nmt-kr primary_qe_score=0.74 backup_engine=deepl-pro-v2 backup_qe_score=0.93
INFO  translation response translated_text="변경 사항 저장" engine=deepl-pro-v2 engine_version=deepl-pro-v2-2026-05 …
INFO  cost attribution captured usd_total=0.000071 usd_engine=0.000069 usd_glossary=0.0000003 usd_qe=0.0000014 cost_component=engine-backup
```

Latency targets (per `microservices/translate/slos/translate-router.openslo.yaml`):

| Path | p50 | p99 | SLO |
|---|---|---|---|
| TM hit | 11 ms | 38 ms | < 50 ms |
| Engine primary (NMT) | 280 ms | 720 ms | < 900 ms |
| Engine backup (DeepL) | 410 ms | 1.1 s | < 1.4 s |
| Escalation to human | n/a (async) | n/a | resolution ≤ 4 h |

## SDK correctness guarantees

1. `translate(...)` is **content-class strict**: passing `ContentClass::Legal` cannot bypass the QE 0.95 threshold from
   the client side — the server refuses any override below the pack-floor and returns
   `TranslateError::QeThresholdBelowPackFloor`.
2. `TM hits ≥ 0.95` are returned without an engine call and without engine cost; lower TM hits are demoted to engine
   suggestions, not direct responses.
3. `provider_credential_mode` defaults to `PlatformDefault`; a pack that sets
   `provider_credential_mode = byok_required_by_pack` overrides the client value and forces BYOK refusals when
   credentials are absent — per ADR-0255 §D-4.
4. `RouteOutcome::EnginePrimaryRefused` carries the primary engine's QE score for evidence — callers cannot suppress
   this in production builds (`#![deny(unused_must_use)]` enforces consumption).
5. Every accepted response carries `cedar_decision_id`, `audit_chain_event_id`, and a child `TraceContext`; the SDK
   refuses to ack the response until the audit-chain anchor is observed downstream.
6. `last_cost_attribution(...)` is **per-trace**, not per-process — concurrent translations do not race on cost rows.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_translate_router_sdk::testkit::Hermetic`, which provides:

- An in-process TM store seeded from `tests/fixtures/acme-ui-tm.jsonl`.
- Stubbed engine adapters: `OyaNmtKr` (returns deterministic translations), `DeepLProV2` (returns deterministic
  translations + an injectable QE score so the test can force the `EnginePrimaryRefused` branch).
- A stubbed Cedar evaluator preloaded with the KR pack policies.
- A stubbed audit-chain shard.

Tests finish in ≤ 25 s and do not require any external translation engine, real Cedar, or real audit-chain shard.

## Error budget

- `TranslateError::ProviderCredentialMissing { engine, mode }` — the tenant pack requires BYOK and no credential is
  bound for the requested engine. Bind via `oya cloud-iam credential put --engine deepl-pro-v2 --tenant …` and retry.
- `TranslateError::QeThresholdBelowPackFloor` — caller requested a QE threshold below the pack floor (e.g. 0.80 on a KR
  pack that requires 0.92 for Marketing content). Raise the threshold or change the content class.
- `TranslateError::CedarRefused` — most commonly minor-protection (KR PIIPP §17 or US-COPPA pack) or residency
  (cross-region request to a tenant whose pack pins KR). Inspect `decision_id` against the audit chain.
- `TranslateError::EngineCapacityShed` — the primary engine is shed-loading. The SDK has already retried the backup;
  surface to the operator and back off; file `translate.slo.engine_capacity_shed`.
- `TranslateError::AuditChainAnchorTimeout` — the audit-chain anchor did not appear within the deadline. The translation
  succeeded but the evidence packet is incomplete; do **not** commit the result to a regulated workflow until the
  anchor is observed (poll via `client.last_audit_anchor(...)`).

## Pack overlay behaviour

When `PackId::Kr` is set:

- Minor-protection: requests for `target_lang = "ko"` and `content_class = ContentClass::Chat` whose tenant flags a
  subject under 14 are refused by Cedar; `decision_id` chains to the PIPA / RR PIIPP evidence packet.
- Residency: requests are pinned to KR shards; cross-region fallback is refused unless the pack overlay declares an
  emergency-services-bypass override per ADR-0103.
- Engine ordering: `EngineChoice::OyaNmtKr` is forced into the primary slot; the SDK rejects requests that demote it.

When `PackId::EuGdpr` is set:

- `OyaNmtEu` is forced into the primary slot.
- Engines whose data-processing addendum is not in the GDPR DPA registry (e.g. an unvetted upstream model) are removed
  from `preferred_engines` at the SDK boundary before any network call.

When `PackId::UsCopPa` is set:

- `content_class = ContentClass::Chat` is forced through the human-review escalation path regardless of QE score for
  any tenant whose Cedar attribute set includes `minor_subject_present = true`.

## Migration parity callouts

- **From Google Cloud Translation v3**: `glossary_id` is a 1:1 carry-over; the `model` parameter maps to
  `EngineChoice::OyaNmt*` or `EngineChoice::GoogleNmtViaByok` (BYOK required); regional endpoints map to oyatie
  cell-aware endpoints (KR cell → `translate.kr.oyatie.dev`).
- **From AWS Translate**: `TerminologyData` ⇒ `glossary_id`; `ParallelData` ⇒ TM stack (see
  `microservices/translate/IP-005-translation-memory-stack.md`); `Settings.Formality` ⇒ `ContentClass` heuristic.
- **From DeepL Pro API**: DeepL remains available as `EngineChoice::DeepLProV2` (BYOK or platform-default depending on
  pack); the DeepL `formality` parameter maps to `ContentClass::{Marketing|UiString|Legal}` heuristics; the DeepL
  `glossary_id` parameter maps 1:1.
- **From Lokalise / Crowdin TM**: TM import via `oya translate tm import --tmx tm.tmx`; glossary import via
  `oya translate glossary import --tbx glossary.tbx`. See `microservices/translate/migration-playbooks/` for full
  vendor mappings.
- **From Microsoft Translator (Azure)**: `Custom Translator` projects map to `glossary_id` + tenant-scoped fine-tune
  ledger; `Profanity Handling` maps to the Cedar minor-protection policy set.

See the migration playbooks under `microservices/translate/migration-playbooks/` for vendor-by-vendor field-level
mapping and parity validation scripts.
