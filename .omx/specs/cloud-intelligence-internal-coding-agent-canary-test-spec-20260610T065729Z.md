# Test spec: internal coding-agent workflow + scheduled parity/drift canaries

## Required failing tests before implementation
1. `contracts/tests/xproxy_contract_parity.rs`
   - Assert OpenAPI exposes read-only admin endpoints for agent runtimes, agent schedules, and parity canaries.
   - Assert OpenAPI schemas include agent runtime/schedule/delegation/workflow/canary status DTOs.
   - Assert AsyncAPI includes redacted agent workflow/canary status messages.
   - Assert proto includes matching DTO/message names.
   - Assert no endpoint/schema/comment introduces CLI/TUI/local-panel control surfaces.

2. `oya-cloud-intelligence-rest`
   - Assert `GET /admin/v1/agent-runtimes`, `/admin/v1/agent-schedules`, `/admin/v1/parity/canaries` return admin-gated, read-only, redacted resources.
   - Assert responses contain policy/secret/workspace references, not raw prompt/body/token/secret values.
   - Assert status DTOs expose Retry-After/failure-mode/safety escalation fields where applicable.

3. `oya-cloud-intelligence-workers`
   - Assert internal coding-agent workflow resources compose from `AgentRuntimeProfile`, `AgentSchedule`, `AgentDelegationPolicy`, guardrail profile, evidence retention profile, and redaction profile.
   - Assert scheduled canary plan emits parity/drift probes, compatibility canaries, status targets, and never writes raw prompt/secret data.
   - Assert worker ownership covers runtime controller, scheduler worker, guardrail workers, evidence retention, and drift parity CronJob.

4. Parity/security scans
   - Assert draft target rows link to the new tests.
   - Assert the forbidden external project name only appears in provenance/source metadata.
   - Assert no internal identifier contains CLI/TUI/local-panel workflow language.

## Minimal passing production code
- Contract additions only for stable DTO/status surfaces.
- REST handlers may return deterministic status snapshots backed by existing app state until controller persistence is added, but must be shaped as production DTOs and admin-gated.
- Worker primitives should be typed constructors/validators, not comments or placeholder docs.
- K8s manifest additions only when tests need real resource/status/RBAC coverage.

## Regression evidence to collect
- Targeted tests showing initial red, then green.
- `cargo test` or Buck equivalent for changed cloud-intelligence crates/tests.
- `cargo metadata` or workspace type/build check if feasible.
- `oya-gate` / `oya-verify` output if available in the worktree.

## Scan semantics update after Critic review
The no-CLI/TUI/local-control-plane tests must distinguish **positive shipped surfaces** from **negative guard evidence**.

Allowed in tests/metadata when explicitly negative or rejected:
- boolean guard fields such as `installs_cli_or_tui_surface: false` or `embeds_local_cron: false`;
- rejected-pattern catalog entries such as `local-cli-smoke-surface` and `local-tui-test-surface`;
- invalid fixtures used to prove local paths are rejected, such as `/tmp/local-workspace`.

Forbidden in new public/control-plane implementation surfaces:
- API paths/operationIds/schema names/resource names/event names that present local CLI/TUI/panel/cron/workspace as a supported workflow;
- runtime code that shells out to local model-agent CLIs or mounts local host paths as the control plane;
- product-layer task orchestration embedded in cloud-intelligence resources.

Add tests that scan new/changed public contract and manifest sections for forbidden positive surfaces while allowing the explicit negative guard list above.
