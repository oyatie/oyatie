# Pre-lane 0.5 — MERGE-surface diffs (L5 codex-adapter, L6 managed-k8s)

READ-ONLY survey. Evidence-only. No source mutation, no build dry-run.
Date: 2026-06-06. Scope: confirm the MERGE targets exist + characterize the surface deltas
between what is being merged FROM and what would be merged INTO.

Paths:
- L5 source (FROM): `/Users/jasonlee/Developer/codex` (pkg `openai-codex-sdk`, sdk lives at `sdk/rust/`)
- L5 target (INTO): `/Users/jasonlee/Developer/source/cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter`
- L6 source (FROM): `/Users/jasonlee/Developer/linux/stack/kubernetes/crates` (358 Cargo.toml; ~95 k8s + ~44 `ctrd_*` + vendored)
- L6 target (INTO): `/Users/jasonlee/Developer/source/cloud/managed-k8s-{cluster-lifecycle,control-plane-host,sla-observability,tenant-quota}` (+ adjacent `cloud/cloud-k8s`)

---

## (a) L5 — codex-adapter

### Target EXISTS — populated, not a stub
`oya-cloud-intelligence-codex-adapter` is a real crate (NOT empty):
- `Cargo.toml` (43 lines), `BUCK` (42 lines), `src/lib.rs` (**942 LOC**),
  `tests/d3_codex_adapter_integration.rs` (8 `#[tokio::test]` httpmock integration tests).
- Sits beside 7 sibling `oya-cloud-intelligence-*` crates (app, authz-cedar-adapter, eventsink-clickhouse/valkey-adapter, kernel, openbao-adapter, rest).
- Owns ADR-0384 Path B "D3 — Codex provider adapter."

### What the target's surface IS
Public API (`pub use oya_cloud_intelligence_kernel::{AgentId, Provider, SeatId, TenantId}` + own types):
- `CodexAdapter { base_url, http: Arc<reqwest::Client>, cli_version }` — `new(http)`, `with_base_url(http, url)`, `cli_version()`, `refresh_token(refresh_token)`, plus a `/backend-api/codex/responses` streaming proxy.
- `OpenAiApiKeyAdapter` (API-key mode → `/v1/chat/completions`).
- `CodexAdapterError` (`RateLimited{retry_after_secs}`, `RefreshFailed(..)`, ...), `CodexTokens`, `CodexProxyRequest`, `CodexProxyResponse`, `HOP_BY_HOP` (RFC 7230 §6.1 strip list).
- Constants: `CLI_VERSION = "cli/0.27.0"` (hard-coded UA impersonation), `CODEX_DEFAULT_BASE_URL = https://chatgpt.com`, `SESSION_PATH = /api/auth/session`.

Model: **server-side HTTP reverse-proxy** of an OAuth subscription pool. It refreshes a
ChatGPT session cookie → bearer, then proxies/streams bytes upstream to the undocumented
`backend-api/codex/responses` endpoint. Deps: `reqwest` 0.13 (rustls, stream), `tokio`,
`bytes`, `serde`, `tracing`, `oya-cloud-intelligence-kernel`.

### What the source (`openai-codex-sdk`) surface IS
`/Users/jasonlee/Developer/codex/sdk/rust` — pkg `openai-codex-sdk` v0.1.0-beta.1, lib
`openai_codex_sdk`, ~3.9k LOC across 13 modules. Public API (lib.rs re-exports):
- `Codex` / `Thread` / `Turn` / `EventStream` / `RunResult` / `StreamedTurn` — CLI-transport entrypoint.
- App-server JSON-RPC layer: `AppServerClient`, `AppCodex`, `AppThread`, `AppTurn*`, `Notification`, `InitializeResponse`, `CURRENT_APP_SERVER_REQUEST_METHODS`, `CURRENT_UPSTREAM_MAIN_SHA`; async variants behind `async` feature (`AsyncAppCodex`, ...).
- Rich event/item model: `ThreadEvent`, `Turn{Started,Completed,Failed}Event`, `Item*`, `AgentMessageItem`, `CommandExecutionItem`, `McpToolCallItem`, `ReasoningItem`, ...; options (`ApprovalMode`, `SandboxMode`, `ModelReasoningEffort`, `WebSearchMode`); protocol schema exports.
- `CodexError`/`Result`.

Model: **client-side CLI JSONL transport** — spawns `codex exec --experimental-json`, writes
the prompt to stdin, parses JSONL events from stdout (mirrors the TS SDK). Requires the Codex
CLI on `PATH` or a runtime path override; does NOT bundle binaries. Deps: `serde`,
`serde_json`, `tempfile`, optional `tokio` (no reqwest, no HTTP client).

### Surface DELTA (the merge gap)
These are **two different abstractions for two different jobs** — this is NOT a drop-in
vendor-merge:
1. **Transport axis:** target = outbound HTTP proxy (reqwest → chatgpt.com / api.openai.com);
   source = local child-process CLI spawner (stdin/stdout JSONL). No shared transport code.
2. **Direction/role:** target is server-side (multi-tenant credential pool, seat/tenant aware
   via kernel `AgentId/SeatId/TenantId`, hop-by-hop stripping); source is a single-user
   embedding SDK (approval modes, sandbox, MCP tool calls).
3. **Type overlap is near-zero:** target's `Codex*` types are wire/proxy DTOs; source's
   `Codex/Thread/Turn/ThreadEvent/*Item` are an agent-conversation domain model. No name
   collisions that imply shared types beyond the word "Codex."
4. **Dep divergence:** target pulls `reqwest 0.13` + `oya-cloud-intelligence-kernel`; source
   pulls `tempfile` + optional `tokio` and has NO HTTP client. BUCK already flags
   `# UNRESOLVED: httpmock ^0.7`.
5. **No existing wiring:** `grep` finds **zero** references to `openai-codex-sdk` /
   `openai_codex_sdk` / `codex/sdk` anywhere in `cloud-intelligence`. The app crate references
   `codex_auth_mode` / `codex_oauth_status` (compliance flags) — it consumes the *adapter*, not the SDK.

**Verdict for founder:** L5 is **complement, not overlay**. The codex-adapter (proxy/pool) and
`openai-codex-sdk` (CLI embedding) solve disjoint problems. A "merge" here is really a
*decision*: (i) keep both as separate concerns (adapter = production data-plane proxy, SDK =
optional local-CLI embedding), or (ii) vendor `openai-codex-sdk` as a NEW sibling crate
(e.g. `oya-cloud-intelligence-codex-sdk`/`oya-codex-sdk`) rather than folding it into the
existing 942-LOC proxy adapter. The hard-coded `cli/0.27.0` impersonation + reverse-engineered
endpoint in the adapter are independent of the SDK's CLI-on-PATH contract. Recommend NOT
collapsing them into one crate.

---

## (b) L6 — managed-k8s (4 services)

### All 4 targets EXIST — populated layered crate sets
None are empty. None are a single root cargo workspace (no top-level
`managed-k8s-*/Cargo.toml`); each is a service dir with `crates/`, `contracts/`, `cedar/`,
`capabilities/`, `manifest.json`, PRD + threat-model + SLO docs. Per-service crate inventory:

| Service | Crates (layers per manifest) |
|---|---|
| **cluster-lifecycle** | `-kernel` (1005 LOC), `-api` (289), `-app` (lib+main, 368) — layers: api/app/kernel |
| **control-plane-host** | `-kernel`, `-api`, `-adapter-capi`, `-adapter-inmemory`, `-app` — layers: kernel/api/adapter/app |
| **sla-observability** | `-kernel`, `-api`, `-adapter-inmemory`, `-app` — layers: adapter/api/app/kernel |
| **tenant-quota** | `-kernel`, `-api`, `-adapter-cedar`, `-adapter-inmemory`, `-app` — layers: adapter/api/app/kernel |

Total: **17 crates** across the 4 services. All follow the same hexagonal shape
(kernel = pure domain, api = port, adapter-* = driven adapters, app = lib+main wiring).

### What the targets' surface IS (representative: cluster-lifecycle-kernel)
Pure-domain Rust, `serde`-only deps (no k8s client libs):
- `DesiredTier`, `ClusterResourceRequest`, `LifecycleRequest`, `NodePoolAction`/`NodePoolOpRequest`,
  `ClusterLifecycleState` (state machine + `IllegalClusterTransition`),
  `validate_dedicated_readiness(..)`, `evaluate_drain_admission(..) -> DrainAdmission`,
  `LifecycleValidationError`.
- Domain framing per manifests: tenant-quota provides a `QuotaDecision` PORT that
  cluster-lifecycle calls *before* control-plane-host provisioning (fail-closed); control-plane-host
  = Kamaji/Talos/CAPI tenant control-plane provisioning (ADR-0376); sla-observability consumes the
  control-plane status seam. These are **product/SaaS control-plane services**, deliberately
  port-and-adapter so that live Kubernetes/Prometheus/CAPI integration is *deferred behind ports*.

### What the source (linux k8s) surface IS
`/Users/jasonlee/Developer/linux/stack/kubernetes/crates` — **358 Cargo.toml** total, comprising
the Go→Rust port of upstream Kubernetes + containerd machinery. Crate names are
**apimachinery/apiserver-flavored**, not product-flavored:
- k8s API/group crates: `apps_v1`, `batch_v1`, `core_v1_proto`, `authentication_v1`,
  `authorization_v1`, `autoscaling_v1`, `certificates_v1`, `coordination_v1`, `networking_v1`,
  `scheduling_v1`, `storage_v1`, `admissionregistration_v1`, `apidiscovery_v2`,
  `apiserverinternal_v1alpha1`, `meta_v1`, `cri_api_v1`, ...
- machinery: `api_equality`, `api_validation`, `api_meta`, `apivalidation_path`, `conversion`,
  `jsonmergepatch`, `mp_*` (merge-patch), `runtime_*` (codec/serializer), `resourceversion`,
  `field_errors`, `labels`, `constraints`, `operation`, `safe`, `util_wait`/`util_runtime`, ...
- containerd: ~44 `ctrd_*` crates (`ctrd_shim`, `ctrd_seccomp`, `ctrd_snapshotters`, `ctrd_cio`,
  `ctrd_netns`, `ctrd_oom`, ...) — these are the L7 container-runtime lane, NOT L6.
- `_upstream/` and `_upstream_containerd/` vendored trees (excluded).

Model: **low-level apimachinery substrate** (typed API objects, codecs, validation, merge-patch)
— the building blocks of a Kubernetes-API-compatible control plane.

### Surface DELTA (the merge gap)
1. **Abstraction-level mismatch is the whole story.** L6 source = ~95 fine-grained
   apimachinery/API-group crates (typed objects + codecs + validation). L6 target = 17
   coarse-grained product control-plane crates (lifecycle state machines, quota ports, SLA
   summaries, CAPI/Kamaji provisioning). The source is the *substrate the target's adapters would
   consume*, not a 1:1 replacement for any target crate.
2. **Naming systems differ:** source = `apps_v1`/`core_v1_proto`/`api_validation` (upstream-shaped);
   target = `oya-managed-k8s-<service>-<layer>` (product/hexagonal). No shared crate names.
3. **No existing target↔source dependency:** the managed-k8s kernels depend only on `serde`
   (cluster-lifecycle-kernel = `serde` only). They intentionally keep Kubernetes integration
   behind ports/adapters; none currently `path = "../linux/.../apps_v1"` or similar.
4. **The real L6 question is the missing 6th surface (`cloud/cloud-k8s`).** `cloud/cloud-k8s`
   EXISTS but is **docs/design-only — it has NO `crates/` directory (0 Cargo.toml)**: it holds
   ARCH/ARCHITECTURE/PRD, IP-001..015 + IP-CLUSTERAPI-001..003 + journey IPs, manifest.json,
   cedar/contracts/policy. ADR-0015/0016 in linux name the orchestration crates
   `oya-cloud-k8s-{apimachinery,apiserver,scheduler,controller-manager,client,...}` with
   **canonical home `cloud/managed-k8s-control-plane-host/`**. So the ~95 apimachinery crates do
   NOT map onto the 4 *product* services 1:1 — they are slated to become a NEW
   `oya-cloud-k8s-*` crate family (apiserver/apimachinery/scheduler/...) that lands under (or
   beside) `managed-k8s-control-plane-host`, with `cloud/cloud-k8s` currently the design SSOT
   (no code) for that family.

**Verdict for founder:** L6 is a **layering merge, not a crate-for-crate overlay**. The 4
`managed-k8s-*` services are the *product control-plane* tier (admission, quota, lifecycle,
SLA, CAPI/Kamaji provisioning) and are already implemented at the domain layer. The linux
~95-crate k8s port is the *apimachinery/apiserver substrate* tier. These compose vertically:
substrate underneath, product control-plane on top. The decision the founder must make is the
**home of the substrate crate family**: per ADR-0015/0016 it is `oya-cloud-k8s-*` under
`managed-k8s-control-plane-host/` — which means L6's true merge target for the bulk of the 95
crates is **control-plane-host (becoming the apimachinery/apiserver home)**, NOT split evenly
across all 4 services. `cloud/cloud-k8s` is the existing docs-only SSOT for this family and is
the natural place that gap is already documented (see sibling artifact `10-k8s-split.md`,
G4).

---

## Summary table — what exists to merge INTO

| Lane | Target dir | Exists? | Crates / LOC | Source (FROM) | Pkg/lib | Merge shape |
|---|---|---|---|---|---|---|
| **L5** | `cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter` | YES | 1 crate, 942 LOC lib + 8 integ tests | `/Developer/codex/sdk/rust` | `openai-codex-sdk` / `openai_codex_sdk` (~3.9k LOC, 13 mods) | **Complement, not overlay** — proxy/pool adapter vs CLI-embedding SDK; ~0 type/dep overlap; recommend vendor SDK as a NEW sibling crate, do not fold into adapter |
| **L6** | 4× `cloud/managed-k8s-*` (+ docs-only `cloud/cloud-k8s`) | YES (4 services, 17 crates) | cluster-lifecycle (3: k:1005/a:289/app:368), control-plane-host (5), sla-observability (4), tenant-quota (5) | `/linux/stack/kubernetes/crates` (358 toml: ~95 k8s + ~44 ctrd + vendored) | upstream-shaped (`apps_v1`, `core_v1_proto`, `api_validation`, ...) | **Vertical layering, not 1:1** — product control-plane (target) sits ON the apimachinery substrate (source); bulk of 95 crates → NEW `oya-cloud-k8s-*` family under `control-plane-host` per ADR-0015/0016; `cloud-k8s` = design SSOT (no code) |

### Open decisions surfaced (for founder)
- **L5-D1:** Keep codex-adapter (proxy) and `openai-codex-sdk` (CLI) as separate concerns? Recommend YES — vendor SDK as new `oya-cloud-intelligence-codex-sdk` rather than merging into the 942-LOC adapter.
- **L6-D1:** Confirm the ~95 k8s crates land as `oya-cloud-k8s-*` under `managed-k8s-control-plane-host` (per ADR-0015/0016), NOT spread across all 4 product services.
- **L6-D2:** Resolve `cloud/cloud-k8s` role — currently docs/design-only (0 crates). Is it the 6th merge target (gets the substrate crates), or stays docs-SSOT while crates land under control-plane-host? (cross-ref G4 / `10-k8s-split.md`.)
