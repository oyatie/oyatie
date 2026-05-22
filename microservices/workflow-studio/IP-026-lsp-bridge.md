---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-026-lsp-bridge
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-shared, axis-security]
date: 2026-05-18
related_adrs: [ADR-0205, ADR-0208]
acceptance_lanes: [lsp-bridge-correctness, lsp-tenant-isolation, oya-vcs-promotion-readiness]
depends_on: [IP-025]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-026 — LSP bridge (codemirror-languageserver wiring for in-product editors)

## Goal

Wire `codemirror-languageserver` to a per-tenant LSP server pool so Workflow Studio's in-product editors (custom-code step body, SQL scratch, JSON, Cedar) get completion, diagnostics, hover, and rename. WebSocket transport per ADR-0208 (already authorized for collab + presence; reused here). Per-tenant isolation via cell-local LSP pods scheduled by the cell µservice. Language servers pinned: `typescript-language-server` (TS/JS), `rust-analyzer` (Rust), `pylsp` (Python), `sqls` (SQL), `cedar-lsp` (Cedar, in-house thin wrapper around the Cedar core).

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/workflow-studio/src/lsp_bridge/Cargo.toml` | create | ~40 LoC |
| `microservices/workflow-studio/src/lsp_bridge/src/lib.rs` | create | ~220 LoC; multiplexes LSP requests over per-tenant WebSocket |
| `microservices/workflow-studio/src/lsp_bridge/src/pool.rs` | create | ~180 LoC; cell-local LSP pod pool; min/max replicas; warm pool of 2 |
| `microservices/workflow-studio/src/lsp_bridge/src/tenant_guard.rs` | create | ~80 LoC; rejects mismatched tenant_id sessions |
| `microservices/workflow-studio/src/lsp_bridge/src/audit.rs` | create | ~60 LoC; per-request seal to audit-chain |
| `clients/web-sveltekit/lib/editor/cm6/lsp-client.ts` | create | ~140 LoC; `codemirror-languageserver` client wired to the bridge |
| `microservices/workflow-studio/iac/lsp-server-pool.yaml` | create | ~120 LoC; Kubernetes manifests for the pod pool (per-tenant cohort namespaces) |
| `microservices/workflow-studio/tests/lsp_bridge_correctness.rs` | create | ~260 LoC; 6 tests |
| `microservices/workflow-studio/tests/lsp_isolation.rs` | create | ~140 LoC; 3 isolation tests |
| `microservices/workflow-studio/runbooks/lsp-server-debug.md` | create | ~120 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0205.md` | append §"LSP bridge shipped" | +6 LoC |

## Code shape

`src/lsp_bridge/src/lib.rs` (excerpt):

```rust
pub struct LspBridge {
    pool: Arc<LspPool>,
    tenant_guard: TenantGuard,
}

impl LspBridge {
    pub async fn handle(&self, session: SessionToken, msg: LspMessage)
        -> Result<LspMessage, LspBridgeError>
    {
        let claims = self.tenant_guard.verify(session)?;
        let pod = self.pool.lease(claims.tenant_id, msg.language).await?;
        let response = pod.send(msg.with_tenant_scope(claims.tenant_id)).await?;
        audit::seal_lsp_request(&claims, &msg, &response)?;
        Ok(response)
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `lsp_bridge_typescript_completion_under_300ms` | tests/lsp_bridge_correctness.rs | TS completion p95 ≤ 300ms |
| `lsp_bridge_rust_analyzer_hover_returns_signature` | tests/lsp_bridge_correctness.rs | Hover returns non-empty signature |
| `lsp_bridge_pylsp_diagnostic_on_undefined_var` | tests/lsp_bridge_correctness.rs | Diagnostic emitted for undefined name |
| `lsp_bridge_websocket_reconnect_resumes_session` | tests/lsp_bridge_correctness.rs | Reconnect after 5s outage resumes via session token |
| `lsp_bridge_diagnostic_published_within_500ms` | tests/lsp_bridge_correctness.rs | Edit-to-diagnostic round-trip p95 ≤ 500ms |
| `lsp_bridge_warm_pool_keeps_two_replicas` | tests/lsp_bridge_correctness.rs | Pool always has ≥ 2 warm pods per (tenant, language) |
| `lsp_bridge_cross_tenant_rejected` | tests/lsp_isolation.rs | Tenant A's session cannot lease Tenant B's pod |
| `lsp_bridge_pod_label_scoped_to_tenant` | tests/lsp_isolation.rs | Pod selector includes `tenant=<id>` label |
| `lsp_bridge_audit_chain_seal_per_request` | tests/lsp_isolation.rs | Every request emits one audit-chain ledger entry |

Minimum 5 required; 9 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/lsp-bridge-correctness-{date}.json`
- `evidence/microservices/workflow-studio/lsp-bridge-isolation-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind lsp-bridge --ms workflow-studio --window 30d` (rollup; per-request seals already happen in `audit.rs`)
- Metrics: `oya_workflow_studio_lsp_completion_latency_ms_bucket{language}`, `oya_workflow_studio_lsp_pool_active_pods{tenant,language}`, `oya_workflow_studio_lsp_reject_total{reason}`

## Rollback procedure

1. Revert ChangeSet for `lsp_bridge` crate + `lsp-client.ts`.
2. Flip feature flag `workflow_studio_lsp=disabled` → editor falls back to syntax highlight only (no completion / diagnostics; banner displayed).
3. Drain LSP pod pool via `kubectl delete -f iac/lsp-server-pool.yaml`.
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-025 — CodeMirror adapter (consumer).
- cell µservice — provides per-tenant cohort namespaces.
- ADR-0205 — code editor canonical.
- ADR-0208 — WebSocket transport.
- audit-chain µservice — ledger sealing.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate lsp-bridge-correctness --crate oya-workflow-studio-lsp-bridge
cargo run -p oya-dev-cli -- gate validate lsp-tenant-isolation --crate oya-workflow-studio-lsp-bridge
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
cargo test -p oya-workflow-studio-lsp-bridge --tests
```

## Halt conditions

- Cross-tenant test fails: STOP, security-critical.
- p95 completion latency > 500ms: STOP, file regression IP.
- Audit-chain seal failure rate > 0.01%: STOP, governance-critical.

## Exit criteria

1. All 9 tests green.
2. `lsp-bridge-correctness`, `lsp-tenant-isolation`, `oya-vcs-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Pod pool manifests validated in dev cluster.
5. Runbook published.
6. ADR-0205 LSP section updated.

## Next IP

[`IP-027-cedar-grammar-impl.md`](IP-027-cedar-grammar-impl.md)

## References

- ADR-0205 — code editor.
- ADR-0208 — WebSocket transport.
- `codemirror-languageserver` — `https://github.com/FurqanSoftware/codemirror-languageserver`.
- LSP 3.17 spec — `https://microsoft.github.io/language-server-protocol/`.
- typescript-language-server, rust-analyzer, pylsp, sqls upstream READMEs.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-026-lsp-bridge.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
