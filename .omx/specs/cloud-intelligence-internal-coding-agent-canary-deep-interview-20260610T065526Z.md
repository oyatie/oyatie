# Deep-interview handoff: cloud-intelligence internal coding agent + parity/drift canaries

## Completion unit
Separate next PR after the green/open foundation PR #644, implemented on an isolated worktree/branch and targeting `dev` without modifying `dev` directly.

## User decision captured
- OMX question answer: `separate_next_pr`.
- Prior user consensus in this thread: foundations first, then one workflow end-to-end in each PR until coverage is complete.
- Next workflow selected from prior user direction: internal coding agent path with scheduled parity/drift canaries.

## Intent and outcome
Provide cloud-intelligence layer primitives that Oyatie can dogfood for internal coding-agent work while keeping product-layer workflow ownership separate. The cloud-intelligence layer must expose reusable, tenant-safe capabilities, guardrails, harnesses, policy signals, sandbox boundaries, event/status surfaces, and scheduled parity/drift canary execution without creating a CLI/TUI control plane.

## Scope for the next PR
- First-class cloud-intelligence resources/contracts for an internal coding-agent workflow and scheduled parity/drift canaries.
- Adapter-owned model/provider translation and routing for the current provider family scope: OpenAI Codex, Anthropic Claude/Codex-compatible surfaces, and Google Gemini, with routing-model support as a routed adapter concern rather than product logic.
- Guardrail primitives for fraud/fault/unsafe/anomaly/hostile-pattern detection, second-pass agentic review, manual escalation, redacted evidence handles, tenant policy signals, and no raw prompt/body/token logging by default.
- Kubernetes/cloud-native control-plane shape: CRD/config resources, workers/controllers/jobs, contract-first APIs/events/status surfaces.

## Non-goals
- Do not expand PR #644 directly unless the branch strategy requires a temporary stacked base; keep this as a separate next completion unit.
- Do not introduce Oyatie CLI/TUI/local panel workflows.
- Do not put product-specific tenant workflow orchestration in cloud-intelligence; expose primitives and policies that products/tenants can use.
- Do not use the external source project name outside provenance/source metadata.
- Do not couple cloud-intelligence directly to Cedar/OpenBao implementations; use owned policy/KMS/secret-handle abstractions through adapters.

## Decision boundaries
- Safe default branch strategy while #644 is not merged: create a stacked isolated branch/worktree from the foundation head and merge/sync latest `origin/dev` into that branch only; never rewrite or push `dev`.
- If #644 merges before/during work, rebase or recreate the next branch from updated `origin/dev` before final PR evidence.
- Defer irreversible production deployment/merge actions to governance gates and reviewer approval.

## Readiness rationale
Ambiguity is below the standard deep-interview threshold because the PR unit, next workflow, non-goals, branch safety, and verification expectations are explicit. Remaining design choices belong in `$ralplan` consensus review rather than additional user interview.
