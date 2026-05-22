---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-015-hg-translate-gate-registration
status: pending
execution_unit: ChangeSet
owner: axis-translate + gtm + ops-iac
acceptance_lanes: [oya-governance-hyperscaler-gate-registration, sdk-codegen, branch-protection-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-TRANSLATE hyperscaler-gate registration + SDK scaffold

## Intent

Final wiring IP. Three concerns:

1. Register `HG-TRANSLATE` hyperscaler-gate in `/specs/hyperscaler-gates.json`. Per ADR-0133, every µservice declares an HG-### gate that captures the (performance + audit + residency + competitor-parity) bar it commits to.
2. Scaffold the Rust + TypeScript + Python SDK clients per `sdk-plan.md`. Code-generated from OpenAPI + AsyncAPI + proto.
3. Add `oya-translate-credential-isolation` + `oya-translate-data-residency-correctness` + `oya-translate-eu-ai-act-disclosure` lanes to `.github/branch-protection.yaml` as BLOCKER lanes on `dev` + `staging`.

## ChangeSet boundary

Mixed: spec + SDK crate + repo-wide config.

## File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | update — add HG-TRANSLATE block |
| `.github/branch-protection.yaml` | update — add 3 BLOCKER lanes |
| `microservices/translate/src/crates/oya-translate-router-sdk/Cargo.toml` | create |
| `microservices/translate/src/crates/oya-translate-router-sdk/src/lib.rs` | create — client surface |
| `microservices/translate/src/crates/oya-translate-router-sdk/src/client.rs` | create |
| `sdk/ts/@oyatie/translate-client/package.json` | create — TS scaffold |
| `sdk/ts/@oyatie/translate-client/src/index.ts` | create |
| `sdk/python/oyatie_translate_client/pyproject.toml` | create — Python scaffold |
| `sdk/python/oyatie_translate_client/__init__.py` | create |
| `developer-docs/translate/README.md` | create — quickstart |
| `developer-docs/translate/eu-ai-act-disclosure-consumption.md` | create |

## HG-TRANSLATE Gate (Excerpt)

```json
{
  "HG-TRANSLATE": {
    "microservice": "translate",
    "status": "pending",
    "bar": {
      "performance": {
        "translation_request_p95_ms_le_500_chars_inhouse": 250,
        "batch_translate_p95_ms_100_seg": 1500,
        "language_detection_p99_ms_le_4kb": 50,
        "tm_leverage_p99_ms": 80,
        "qe_p99_ms": 200,
        "real_time_caption_p99_ms_per_chunk": 400,
        "document_translate_p95_s_10page_docx": 8,
        "bulk_translate_p95_s_10k_segment_xliff": 60,
        "router_decision_p99_ms": 5
      },
      "availability": {
        "translate_request_monthly_pct": 99.95,
        "data_residency_correctness_monthly_pct": 100
      },
      "audit": {
        "translation_completed_emit_ratio": 1.0,
        "eu_ai_act_disclosure_emit_ratio_for_eu_pack": 1.0,
        "ed25519_envelope_present_on_every_event": true
      },
      "residency": {
        "default_deny": true,
        "per_pack_engine_whitelist_enforced": true,
        "cross_region_inference_event_count": 0
      },
      "competitor_parity": {
        "feature_matrix_review_quarterly": true,
        "wmt_eval_pass_per_pair": true
      },
      "isolation": {
        "credential_bytes_in_source_or_logs": 0,
        "cross_tenant_tm_match_count": 0,
        "document_sandbox_seccomp_violation_count": 0
      }
    },
    "blocker_lanes": [
      "oya-translate-credential-isolation",
      "oya-translate-data-residency-correctness",
      "oya-translate-eu-ai-act-disclosure"
    ],
    "evidence_emitted_to": "/specs/hyperscaler-gates.json#registry",
    "related_adrs": ["ADR-0135", "ADR-0131", "ADR-TRANSLATE-0001", "ADR-TRANSLATE-0003", "ADR-TRANSLATE-0004", "ADR-TRANSLATE-0005", "ADR-TRANSLATE-0006"]
  }
}
```

## Branch-Protection Lanes

```yaml
required_status_checks:
  contexts:
    # ...existing lanes...
    - "oya-translate-credential-isolation"          # BLOCKER
    - "oya-translate-data-residency-correctness"    # BLOCKER
    - "oya-translate-eu-ai-act-disclosure"          # BLOCKER
```

## SDK Scaffold (Rust client surface excerpt)

```rust
pub struct TranslateClient { /* ... */ }

impl TranslateClient {
    pub fn new(base_url: &str) -> Self { ... }
    pub fn with_oidc_token(self, token: String) -> Self { ... }
    pub fn with_tenant(self, tenant_id: &str) -> Self { ... }
    pub fn with_pack(self, pack: PackId) -> Self { ... }

    pub async fn translate(&self, req: TranslateRequest) -> Result<TranslateResponse> { ... }
    pub async fn translate_batch(&self, req: BatchTranslateRequest) -> Result<BatchTranslateResponse> { ... }
    pub async fn detect_language(&self, text: &str) -> Result<LanguageDetection> { ... }
    pub async fn lookup_tm(&self, ...) -> Result<Option<LeverageMatch>> { ... }
    pub async fn estimate_quality(&self, ...) -> Result<QualityScore> { ... }
    pub async fn submit_bulk_job(&self, ...) -> Result<JobId> { ... }
    pub async fn get_bulk_job(&self, id: &JobId) -> Result<JobState> { ... }
    pub async fn translate_document(&self, ...) -> Result<DocumentResult> { ... }
    pub async fn open_caption_stream(&self, ...) -> Result<CaptionStream> { ... }
    pub async fn import_tbx(&self, ...) -> Result<()> { ... }
    pub async fn export_tbx(&self, ...) -> Result<Vec<u8>> { ... }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_hg_translate_gate_schema_valid` | spec block parses |
| `test_branch_protection_includes_new_blockers` | YAML present |
| `test_sdk_rust_compiles` | scaffold compiles |
| `test_sdk_ts_typecheck` | TS scaffold typechecks |
| `test_sdk_python_imports` | Python scaffold imports |
| `tests/e2e/sdk_rust_quickstart.rs` | end-to-end through SDK |
| `tests/e2e/sdk_ts_quickstart.spec.ts` | end-to-end |

## Halt Conditions

- HG-TRANSLATE gate omitted.
- Any of three BLOCKER lanes missing from branch-protection.
- SDK published before HG-TRANSLATE green.

## Phase Exit

When this IP merges + verification passes, Phase P01-translate-platform exits and the µservice becomes promotable per ADR-0139.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/translate/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `1800s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `EU-AI-ACT-2024-HIGH-RISK` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=1800`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/translate/IP-015-hg-translate-gate-registration.md:56` - "language_detection_p99_ms_le_4kb": 50,; `microservices/translate/IP-015-hg-translate-gate-registration.md:57` - "tm_leverage_p99_ms": 80,.

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/translate/IP-015-hg-translate-gate-registration.md:85` - "document_sandbox_seccomp_violation_count": 0.
