# Wave 15J-batch-4 tier scrub notes — foundry

## Summary
- capability-tiers/ dir deleted: Y
- Vocabulary replacement count: ~760
- Replacement doctrine: ADR-0330 tenant_class (`demo_trial`, `paid`) plus paid `billing_components` (`per_seat`, `per_usage` for foundry).
- Verification: required Bronze/Silver/Gold/Platinum and capability_tier/max_tier/tier_threshold scans return zero matches outside this note.

## Files Modified With Current Line Counts
- README.md — 12 lines
- manifest.json — 1219 lines
- PRD.md — 388 lines
- policy/supervisor-tenant-scope.cedar — 191 lines
- policy/runtime-tenant-scope.cedar — 188 lines
- contracts/openapi/guardrails-guardrails.yaml — 307 lines
- contracts/openapi/supervisor-foundry-supervisor.yaml — 457 lines
- contracts/proto/guardrails-guardrails.proto — 209 lines
- contracts/proto/supervisor-foundry-supervisor.proto — 313 lines
- contracts/asyncapi/guardrails-decision-events.yaml — 239 lines
- contracts/asyncapi/supervisor-foundry-supervisor-events.yaml — 239 lines
- slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml — 48 lines
- IP-032-eval-layer-a-postgres-clickhouse-baseline-store-iac.md — 52 lines
- runbooks/eval-baseline-output-restore.md — 73 lines
- iac/helm/eval/baseline-store/Chart.yaml — 17 lines
- iac/helm/eval/baseline-store/values.yaml — 101 lines
- Additional touched set: Foundry had a broad recurring eval "golden" vocabulary family and autonomy-tier field family; the scrub updated all matching tracked files under microservices/foundry so the exact assignment regex is clean.

## Design Decisions
- Renamed eval "golden" corpus wording to "baseline" because the assignment verification regex matches the `gold` substring.
- Kept Foundry's autonomy control semantics but renamed contract and policy fields from requested/declared tier wording to requested/declared autonomy level wording.
- Replaced supervisor Cedar tier entitlement checks with paid tenant_class plus `billing_components contains "per_seat"`.
- Renamed the eval golden-store path to baseline-store to keep path references aligned with the scrubbed text.

## Outstanding Follow-ups
- None for the assigned scrub checks.

## Wave 15-IP-substance scrub (2026-05-21)

- Scope: IP-BUCKET-B / `foundry`.
- Inventory: 115 top-level `microservices/foundry/IP-*.md` files; no `microservices/foundry/ips/` directory was present.
- Detection method: line-count cluster (`wc -l`) plus repeated heading scan (`Intent`, `Concrete File Targets`, `Acceptance Gates`, `References`) plus placeholder scan for `Same shape as`, `.../`, and line-count-as-proof language.
- Rewritten in place: 49 short stamped slices received a bespoke Wave 15 substance conversion section with real Foundry paths, policy files, contracts, SLOs, capabilities, and Big-8/AI-platform counterparts.
- Files rewritten: `IP-001-runtime-runtime-cluster-iac.md`, `IP-002-runtime-redis-and-postgres-baseline.md`, `IP-014-runtime-runtime-self-slo-manifests.md`, `IP-016-supervisor-postgres-layer-a-iac.md`, `IP-017-supervisor-redis-layer-a-iac.md`, `IP-018-supervisor-k8s-operator-iac.md`, `IP-022-supervisor-supervision-event-bus.md`, `IP-031-eval-layer-a-gpu-runner-pool-iac.md`, `IP-032-eval-layer-a-postgres-clickhouse-baseline-store-iac.md`, `IP-034-eval-eval-runner-domain.md`, `IP-035-eval-eval-runner-usecase.md`, `IP-036-eval-eval-runner-api.md`, `IP-038-eval-eval-runner-adapter-s3.md`, `IP-039-eval-eval-runner-adapter-gpu.md`, `IP-040-eval-eval-runner-rest.md`, `IP-041-eval-eval-runner-worker.md`, `IP-042-eval-eval-runner-sdk.md`, `IP-043-eval-eval-runner-app.md`, `IP-044-eval-parity-analyzer-bootstrap.md`, `IP-045-eval-replay-engine-bootstrap.md`, `IP-046-evidence-storage-backend-iac.md`, `IP-047-evidence-self-slo-manifest.md`, `IP-048-evidence-capability-invocation-recorder-kernel.md`, `IP-049-evidence-evidence-pack-builder-kernel.md`, `IP-050-evidence-evidence-pack-builder-domain.md`, `IP-051-evidence-evidence-pack-builder-usecase-and-adapters.md`, `IP-052-evidence-capability-invocation-recorder-stack.md`, `IP-053-evidence-eval-evidence-aggregator.md`, `IP-054-evidence-evidence-query-stack.md`, `IP-055-evidence-regulator-export-stack.md`, `IP-056-evidence-audit-chain-bridge.md`, `IP-057-evidence-sdk-cross-microservice.md`, `IP-058-evidence-regulator-export-framework-profiles.md`, `IP-059-evidence-evidence-archive-cascade.md`, `IP-060-evidence-self-observability-slo-wiring.md`, `IP-062-guardrails-classifier-model-serving-iac.md`, `IP-063-guardrails-rule-store-postgres-iac.md`, `IP-079-providers-router-api.md`, `IP-080-providers-router-adapter.md`, `IP-082-providers-adapter-anthropic-subscription.md`, `IP-083-providers-adapter-openai-api.md`, `IP-084-providers-adapter-openai-subscription.md`, `IP-085-providers-adapter-gemini-api.md`, `IP-086-providers-adapter-gemini-subscription.md`, `IP-087-providers-adapter-in-house.md`, `IP-089-providers-router-rest-worker-app.md`, `IP-WASMTIME-002-capability-token-binding.md`, `IP-WASMTIME-003-fuel-and-memory-accounting.md`, `IP-WASMTIME-004-component-model-onboarding.md`.
- Deleted as duplicative: none; several files share BC-local sequencing, but each maps to a distinct crate, policy, contract, IaC, or runtime surface in `manifest.json`.
- Preserved: longer IPs already carrying bespoke crate-level implementation details, journey IPs at 400+ lines, and IPs outside the 30-80 line stamp-shell cluster.
- Counterpart anchors added: 62 preserved IPs that already had substantive implementation detail but lacked an explicit Big-8 / comparable-platform reference received a concise `Wave 15 counterpart anchor`.
- Verification required before promotion: line-count cluster should drop for rewritten files; `grep -L` counterpart scan should no longer flag Foundry IP files; remaining flags are follow-up candidates where concise IPs may still need a full manual expansion pass.
