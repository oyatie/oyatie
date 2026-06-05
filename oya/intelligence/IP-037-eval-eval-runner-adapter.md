---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: foundry
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-037-eval-eval-runner-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
co_owners: [axis-data]
date: 2026-05-18
related_adrs: [ADR-0131, ADR-0064, ADR-0145]
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness, oya-governance-promotion-readiness]
depends_on: [IP-035, IP-036]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-037 — `oya-foundry-eval-eval-runner-adapter`

## Goal

Author the protocol-neutral kernel-port implementations for the eval-runner adapter layer of the Foundry eval-harness substrate. Three primary ports are realized:

1. **Filesystem eval-set reader** — reads YAML/JSON eval-set definitions from `microservices/intelligence/eval-sets/` and emits typed `EvalSet` aggregates to the use-case layer.
2. **Provider-route resolver** — queries `foundry-providers` µservice (via Workflow event topology) to resolve a logical provider id (`anthropic.claude-opus-4-7`) to a concrete route (endpoint + auth + rate-limit token).
3. **Foundry-evidence client** — emits per-eval-run evidence records to the `foundry-evidence` µservice (audit-chain-sealed evaluation outcomes).

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/Cargo.toml` | create | ~50 LoC |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/src/lib.rs` | create | ~120 LoC; pub surface, error types, trait impls |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/src/eval_set_reader.rs` | create | ~220 LoC; filesystem walker + serde-yaml + schema validator + bounded read budget |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/src/route_resolver.rs` | create | ~240 LoC; Workflow-event client; ratelimit token negotiation; tenant-scope guard |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/src/evidence_emitter.rs` | create | ~180 LoC; foundry-evidence client + per-run audit-chain seal |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/src/tenant_guard.rs` | create | ~80 LoC; rejects cross-tenant eval-set reads |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/tests/eval_set_reader_test.rs` | create | ~200 LoC; 4 tests |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/tests/route_resolver_test.rs` | create | ~200 LoC; 4 tests against fake `foundry-providers` |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/tests/evidence_emitter_test.rs` | create | ~160 LoC; 3 tests against fake evidence sink |
| `microservices/intelligence/src/crates/oya-foundry-eval-eval-runner-adapter/tests/tenant_guard_test.rs` | create | ~120 LoC; 3 isolation tests |
| `microservices/intelligence/catalog/oya-foundry-eval-eval-runner-adapter.yaml` | create | ~60 LoC; catalog entry per ADR-0131 catalog substrate |
| `microservices/intelligence/runbooks/eval-runner-adapter-debug.md` | create | ~100 LoC operator playbook |

## Code shape

`src/eval_set_reader.rs` (excerpt):

```rust
#[async_trait]
impl EvalSetReader for FilesystemEvalSetReader {
    async fn read(&self, id: EvalSetId, claims: TenantClaims)
        -> Result<EvalSet, EvalSetReadError>
    {
        self.tenant_guard.verify_read(&claims, &id)?;
        let path = self.resolve_path(&id)?;
        let raw = tokio::fs::read_to_string(&path).await
            .map_err(|e| EvalSetReadError::Io { path: path.clone(), source: e })?;
        if raw.len() > self.config.max_eval_set_bytes {
            return Err(EvalSetReadError::TooLarge { path, size: raw.len() });
        }
        let parsed: EvalSet = serde_yaml::from_str(&raw)
            .map_err(EvalSetReadError::Parse)?;
        self.schema_validator.validate(&parsed)?;
        Ok(parsed)
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `eval_set_reader_parses_valid_yaml_fixture` | eval_set_reader_test.rs | Real fixture parses cleanly |
| `eval_set_reader_rejects_oversize_file` | eval_set_reader_test.rs | File > max budget → error |
| `eval_set_reader_rejects_schema_mismatch` | eval_set_reader_test.rs | Missing required field → error |
| `eval_set_reader_io_error_propagated` | eval_set_reader_test.rs | Missing file → typed I/O error |
| `route_resolver_resolves_known_provider` | route_resolver_test.rs | `anthropic.claude-opus-4-7` → concrete route |
| `route_resolver_returns_error_on_unknown_provider` | route_resolver_test.rs | Unknown id → `ProviderNotFound` |
| `route_resolver_respects_ratelimit_token` | route_resolver_test.rs | Rate-limited → backoff respected |
| `route_resolver_cross_tenant_rejected` | route_resolver_test.rs | Mismatched tenant claim → `Unauthorized` |
| `evidence_emitter_writes_run_record` | evidence_emitter_test.rs | Record written to fake sink with correct schema |
| `evidence_emitter_seals_via_audit_chain` | evidence_emitter_test.rs | Each emit results in one audit-chain seal |
| `evidence_emitter_failure_returns_typed_error` | evidence_emitter_test.rs | Sink down → typed error; no silent drop |
| `tenant_guard_cross_tenant_read_rejected` | tenant_guard_test.rs | Tenant A → Tenant B eval-set: rejected |
| `tenant_guard_audits_rejected_attempts` | tenant_guard_test.rs | Reject emits audit-chain event |
| `tenant_guard_clean_access_passes` | tenant_guard_test.rs | Same-tenant read passes |

Minimum 8 required; 14 specified.

## Evidence to emit

- `evidence/microservices/intelligence/eval-runner-adapter-coverage-{date}.json` — line + branch coverage report (target ≥ 85% line)
- `evidence/microservices/intelligence/eval-runner-adapter-isolation-{date}.json` — tenant-guard scan
- Audit-chain seal: `oya audit-chain seal --kind eval-runner-adapter-build --window 30d`
- Metrics: `oya_foundry_eval_runner_adapter_read_latency_ms_bucket`, `oya_foundry_eval_runner_adapter_route_resolve_latency_ms_bucket`, `oya_foundry_eval_runner_adapter_evidence_emit_total`, `oya_foundry_eval_runner_adapter_tenant_reject_total`

## Rollback procedure

1. Revert ChangeSet for the adapter crate + catalog entry + runbook.
2. Eval-harness use-case layer (built on this adapter) blocks new eval runs gracefully (returns `AdapterUnavailable`); existing run records preserved.
3. Banner in foundry dashboard: "Eval-harness adapter rollback — see ops".
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-035 — eval-harness kernel (defines port traits).
- IP-036 — eval-harness domain (defines `EvalSet` + `Route` aggregates).
- `foundry-providers` µservice — supplies route resolution.
- `foundry-evidence` µservice — consumes evidence emission.
- ADR-0131 — per-µservice flat layout (catalog substrate).
- ADR-0145 — cell isolation canonical (tenant guard).

## Acceptance gates

```bash
cargo nextest run -p oya-foundry-eval-eval-runner-adapter
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --crate oya-foundry-eval-eval-runner-adapter
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --crate oya-foundry-eval-eval-runner-adapter
buck2 build //:quality-lane-registry-authority-check # lane=oya-governance-promotion-readiness --microservice foundry
```

## Halt conditions

- Line coverage < 85%: STOP, file coverage IP.
- Any cross-tenant test fails: STOP — security-critical.
- Layer-correctness lane fails (kernel→domain→adapter dependency direction violated): STOP.

## Exit criteria

1. All 14 tests green.
2. ≥ 85% line coverage emitted to evidence.
3. `lean-a1`, `layer-correctness`, `oya-governance-promotion-readiness` lanes green.
4. Catalog entry registered.
5. Runbook published.

## Next IP

[`IP-038-eval-eval-runner-adapter-s3.md`](IP-038-eval-eval-runner-adapter-s3.md)

## References

- ADR-0131 — per-µservice flat layout (catalog substrate).
- ADR-0064 — canonical base + localization overlay.
- ADR-0145 — cell isolation canonical.
- microservices/intelligence/PHASE-01-EVAL-HARNESS-SUBSTRATE.md.
- `foundry-providers` µservice contract.
- `foundry-evidence` µservice contract.
- Workflow event topology spec.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Evals, LangSmith, Google Vertex AI Evaluation, and Databricks Mosaic AI evals.
- Gap closure: this IP closes deterministic replay, baseline comparison, and eval evidence emission in the Foundry product boundary.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
