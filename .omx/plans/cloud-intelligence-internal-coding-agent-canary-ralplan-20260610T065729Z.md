# Ralplan: cloud-intelligence internal coding-agent workflow + scheduled parity/drift canaries

## Execution unit
Separate next PR, stacked on foundation PR #644 only while #644 is unmerged. Target remains `dev`; do not modify or force-update `dev`.

## Current foundation evidence
Foundation branch/worktree: `/Users/jasonlee/oyatie-worktrees/cloud-intelligence-xproxy-20260610` at `04c5c4f6e26c5f04b915d2d12b1fae963088d836`.
`origin/dev` is `b049a777e`; foundation head is not on `origin/dev`, so this PR must either stack on foundation or be rebased/recreated after #644 merges.

Foundation already provides:
- `AgentRuntimeProfileSpec`, `AgentScheduleSpec`, `AgentDelegationPolicySpec`, guardrail/evidence/redaction specs in `oya-cloud-intelligence-workers`.
- Safety primitives in `oya-cloud-intelligence-kernel::safety` for critical block, second-pass review, manual escalation, redacted evidence handles, and reversible token policy.
- Gemini adapter plus translation boundary and route advisor profiles.
- K8s manifest entries for agent runtime/schedule/guardrail/evidence/manual-review resources.
- Draft target rows for routing, auth, drift, observability, wire compatibility, safety.

## Gap this PR closes
Make the internal coding-agent workflow and scheduled parity/drift canaries observable and contract-first end-to-end at the cloud-intelligence layer, without becoming product-layer orchestration:

1. **Contracts**
   - Add OpenAPI admin/read-only endpoints for cloud-intelligence agent runtime catalog/status and scheduled canary status, e.g. `/admin/v1/agent-runtimes`, `/admin/v1/agent-schedules`, `/admin/v1/parity/canaries`.
   - Add schemas for `AgentRuntimeProfile`, `AgentSchedule`, `AgentDelegationPolicy`, `AgentWorkflowStatus`, `ParityCanaryPlan`, `ParityCanaryStatus` if missing.
   - Add AsyncAPI messages for redacted agent-run requested/completed/safety-gated/parity-drift canary status where not already present.
   - Add proto DTOs for the same status surfaces if the proto contract has parity tests.

2. **REST/admin status surface**
   - Implement read-only REST handlers returning deterministic in-memory/status DTOs for the new admin endpoints.
   - Enforce admin authz gate pattern already used by existing admin routes.
   - Ensure payloads contain resource refs and redacted status only: no raw prompts, completions, tokens, secrets, workspace paths, or local CLI/TUI concepts.

3. **Worker/control-plane primitives**
   - Add typed workflow/canary plan constructors if current worker specs are insufficient to model: internal coding agent run request, scheduled parity/drift canary, safety-gated second pass, and manual escalation handoff.
   - Keep runtime execution externalized to controllers/workers; no embedded local process/CLI/TUI workflow.

4. **Parity map / target rows**
   - Add or update draft target rows for first-class agent runtime resources and scheduled canary workflow, with linked target tests.
   - Preserve all 50 `XPROXY-*` baseline rows and existing provenance-only naming constraints.

5. **Kubernetes manifest**
   - Ensure CRD/config snippets and worker ownership exist for `AgentRuntimeProfile`, `AgentSchedule`, guardrail/evidence profiles, and parity canary CronJob/status resources.
   - Add missing RBAC/status-subresource markers only if tests require them.

## Non-goals
- No Oyatie CLI, TUI, local panel, local cron, local hook installation, or local workspace path as a control plane.
- No product-specific coding-agent prompts or tenant workflow implementation; cloud-intelligence exposes reusable primitives/statuses.
- No raw provider credential, prompt, completion, token, or secret logging/storage by default.
- No direct dependency from kernel to REST/workers or adapters to workers.
- No direct Cedar/OpenBao integration in kernel/contracts; use owned policy-engine/secret-provider/KMS handles through adapters.

## Implementation stride
Deliverable stride: implement all tests and production code for this one PR. If #644 remains unmerged, work from a new stacked worktree branch based on foundation head and merge latest `origin/dev` into the branch; before final, report that the branch is stacked and may need rebasing after #644 merges.

## Planned changed areas
- `cloud/cloud-intelligence/contracts/cloud-intelligence.openapi.yaml`
- `cloud/cloud-intelligence/contracts/cloud-intelligence.asyncapi.yaml`
- `cloud/cloud-intelligence/contracts/cloud-intelligence.proto`
- `cloud/cloud-intelligence/contracts/tests/xproxy_contract_parity.rs`
- `cloud/cloud-intelligence/crates/oya-cloud-intelligence-rest/src/lib.rs`
- `cloud/cloud-intelligence/crates/oya-cloud-intelligence-rest/tests/...` or module tests
- `cloud/cloud-intelligence/crates/oya-cloud-intelligence-workers/src/lib.rs`
- `cloud/cloud-intelligence/crates/oya-cloud-intelligence-workers/tests/xproxy_worker_ownership.rs`
- `cloud/cloud-intelligence/crates/oya-cloud-intelligence-kernel/capability-parity/external-proxy-reference-draft-targets-20260610.json`
- `cloud/cloud-intelligence/k8s/cloud-intelligence.yaml` if CRD/status/RBAC coverage is missing

## Verification plan
1. Add failing tests first for contract endpoints/schemas/messages, REST read-only status, worker/canary primitives, parity target linkage, no CLI/TUI terms, no forbidden external project-name identifiers, and no raw payload/secret status.
2. Confirm targeted failures.
3. Implement minimal production code.
4. Run targeted Rust tests for contracts/rest/workers/kernel parity.
5. Run broader Buck/Cargo checks available for cloud-intelligence.
6. Run governance bridge checks (`oya-gate` / `oya-verify`) where available and cloud-ci once pushed/PR opened.

## Handoff acceptance criteria
- New admin/status endpoints are contract-first and tested.
- Internal coding agent workflow is represented as cloud-intelligence resources/statuses, not product workflow code.
- Scheduled parity/drift canary is represented as a controller/CronJob/status resource with tests.
- Safety/guardrail/manual review/evidence handles remain code-backed and redacted.
- Provider translation/routing remains adapter-owned for Codex/Claude/Gemini and routing-only advisors.
- No CLI/TUI/local workflow implementation is introduced.
- External reference project name remains only in provenance/source metadata.

## RALPLAN-DR principles, drivers, and options

### Principles
1. **Cloud-intelligence primitive, not product workflow** — cloud-intelligence owns reusable resource/status/guardrail/canary primitives; product layers compose them into tenant-specific workflows.
2. **Cloud-native control plane only** — Kubernetes resources, controllers/workers, contract-first APIs, events, and dashboard/read-only status surfaces are valid. Local control surfaces are not.
3. **Adapter-owned translation and transient integrations** — Codex/OpenAI-compatible, Claude/Anthropic, Gemini, routing-advisor models, policy-engine ports, KMS/secret-provider handles, and evidence sealing remain behind adapters/ports.
4. **Redacted by default** — status APIs/events/tests must traffic in refs, hashes, policy names, result states, Retry-After/failure hints, and sealed evidence handles; never raw prompts, completions, tokens, secrets, or unrestricted workspace contents.
5. **Executable TDD, not comments** — every new target must be backed by a failing test first, then minimal production code.

### Drivers
- User explicitly chose a separate next PR after foundation.
- User wants internal coding-agent path plus scheduled parity/drift canaries as the first workflow after foundation.
- Existing foundation already contains many worker/safety/resource specs; duplicating them would increase risk.
- PR #644 is not yet on `dev`, so branch strategy must be safe/stacked without rewriting `dev`.
- CI and scans must avoid false positives from existing negative guard identifiers/fixtures.

### Options considered
- **Option A — Expand PR #644**: rejected because user chose separate next PR and prior consensus says one workflow per PR.
- **Option B — Add product-layer coding workflow now**: rejected because pure dogfood separates cloud-intelligence primitives from product-specific orchestration.
- **Option C — Contract/admin/status/canary resource PR (selected)**: add missing read-only contract/status surfaces and typed worker plan primitives around existing foundation resources.
- **Option D — Wait for #644 merge before any planning**: rejected for planning/execution preparation; final PR may still need rebase after #644 merges.

### Principle-option consistency
Selected Option C is the smallest slice that satisfies the user’s dogfood goal while preserving boundaries: it exposes cloud-intelligence primitives/statuses for tenants and Oyatie to use, leaves provider/security integrations in adapters/ports, and avoids creating a local or product-layer workflow.

### Scan semantics clarification
CLI/TUI/local-control-plane bans apply to positive/public implementation targets: API paths, schemas, resource names, worker names, event names, docs describing shipped control surfaces, and new runtime behavior. Negative test fixtures, defensive field names, and rejected-pattern metadata that explicitly prove those surfaces are absent are allowed when clearly scoped as guards (for example `installs_cli_or_tui_surface: false`, `embeds_local_cron: false`, rejected pattern `local-cli-smoke-surface`, or invalid fixture `/tmp/local-workspace`).
