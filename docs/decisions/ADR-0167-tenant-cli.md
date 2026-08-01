---
id: ADR-0167
status: Accepted
deciders: council-architecture, council-api-sdk, axis-developer-experience, axis-tenancy
date: 2026-05-18
owner: council-api-sdk
supersedes: []
superseded_by: []
amended_by: [ADR-0632]
related: [ADR-0002, ADR-0011, ADR-0021, ADR-0037, ADR-0121, ADR-0145, ADR-0146, ADR-0632]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

# ADR-0167 — Tenant-facing CLI binary `oya` (separate from internal `oya-dev-cli`)

## Status

Accepted (2026-05-18). Authorizes a separate, narrowly-scoped tenant-facing CLI binary published to tenants as part of the public SDK surface. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C1.

## ADR-0632 product-protocol reconciliation

The tenant CLI **MUST** use the public HTTPS REST surface documented by OpenAPI 3.2.0 and may
consume signed/versioned webhooks or AsyncAPI/CloudEvents notifications through supported public
workflows; SSE and WebSocket remain the public one-way and bidirectional realtime transports. It
**MUST NOT** expose GraphQL, gRPC, gRPC-Web, or Connect. Internal sibling-service RPC remains
internal-only gRPC/proto3 over HTTP/2 and outside the tenant CLI contract.

## Context

Oyatie ships a public HTTPS REST, webhook, event, and realtime API per ADR-0011 (cross-microservice contract registry) and a language SDK roadmap per ADR-0037 (public API stability tiers). Power users — DevOps engineers, automation authors, support engineers, agentic tenants per ADR-0021 — require a command-line workflow that matches the ergonomics of the public clouds:

- **AWS CLI** (`aws s3 cp ...`) — the canonical reference for "every API method exposed as a command".
- **Stripe CLI** (`stripe listen`, `stripe trigger`, `stripe login`) — the canonical reference for OAuth-2.1 login, local webhook tunneling, and event simulation.
- **Anthropic CLI** / `claude` — the canonical reference for an agentic-tenant CLI with structured stdout JSON and pipeable input.
- **GitHub CLI** (`gh pr create ...`) — the canonical reference for high-level workflow commands wrapping multi-step API calls.

Today the only CLI surface in the repo is `oya-dev-cli` (`crates/oya-dev-cli/`, binary `oya`), an INTERNAL contributor tool that runs gate validators, registers µservices, emits evidence bundles, and orchestrates the Foundry pipeline. Mixing tenant-facing commands into `oya-dev-cli` would:

1. Leak internal gate names (e.g. `gate validate active-artifact-contract`) into the tenant surface — a confidentiality leak about Oyatie's own engineering controls.
2. Force tenants to depend on every internal crate (oya-dev-cli's dependency closure spans 200+ crates today).
3. Couple tenant-CLI semver to internal-CLI semver (ADR-0037 Tier-A vs Tier-D incompatibility).
4. Violate the hyperscaler-reference shape — AWS, Stripe, GitHub all ship a tenant CLI distinct from any internal tooling.

We need a tenant-facing binary that is narrowly scoped to the public API surface, builds against the public SDK only, and ships through the same Tier-A stability promises as the public API.

## Decision

Oyatie introduces a SECOND CLI binary, also named `oya` from the tenant's perspective, distributed as the crate `oya-tenant-cli` and packaged under the name `oya` in tenant-facing artifact channels (Homebrew tap, apt repo, container image `ghcr.io/oyatie/oya:<semver>`, MSI installer).

The two binaries are kept distinct via:

- **Repo layout**: `crates/oya-tenant-cli/` (this ADR) vs `crates/oya-dev-cli/` (internal).
- **Distribution**: `oya-tenant-cli` is published to tenant artifact channels; `oya-dev-cli` is internal-only (workspace-local `cargo run -p oya-dev-cli -- ...`). The workspace bin target is `oya-tenant` to avoid colliding with the internal `oya` binary; packaging aliases it to `oya` only in tenant channels.
- **Dependency closure**: `oya-tenant-cli` depends ONLY on the public SDK crates (`oya-shared-public-sdk-*` family) and a thin command dispatcher. It does NOT depend on any `oya-check-*`, `oya-foundry-*`, or `oya-dev-cli` crate.
- **Semver tier**: `oya-tenant-cli` follows Tier-A (per ADR-0037) — breaking changes require an ADR + 18-month sunset window. `oya-dev-cli` follows Tier-D (internal, change at will).

### Command surface (v0.1)

The v0.1 surface mirrors the public API tier-A contracts (per ADR-0011 contract registry):

| Command group | Purpose | Backing µservice |
|---|---|---|
| `oya auth login` | OAuth 2.1 device-code flow per ADR-0002 | tenancy |
| `oya auth logout` | revoke local token | tenancy |
| `oya auth whoami` | show current tenant + principal | tenancy |
| `oya workflow run <flow-id>` | invoke a workflow per ADR-0035 | workflow (Workflow Studio) |
| `oya workflow status <run-id>` | poll run status | workflow |
| `oya messenger send --to <id> --body <text>` | send a message | messenger |
| `oya messenger search --query <q>` | search messages | messenger |
| `oya tasks create --title <t>` | create a task | tasks |
| `oya tasks list` | list tasks | tasks |
| `oya foundry capability invoke <cap-id> --args <json>` | invoke a Foundry capability per ADR-0021 | foundry |
| `oya foundry capability list` | enumerate available capabilities | foundry |
| `oya ontology entity get <urn>` | fetch an Ontology entity | ontology |
| `oya audit chain query --since <ts>` | query the tenant's audit-chain seals per ADR-0003 | audit-chain |
| `oya version` | print build version + API version compatibility | (local) |
| `oya completion <shell>` | emit shell-completion scripts | (local) |

Output contract:
- Default human-readable table output to TTY.
- `--output json` emits one canonical JSON object per command (Stripe-CLI parity).
- `--output ndjson` for streaming list commands (AWS-CLI `--no-paginate` parity).
- Exit code `0` success, `1` user error, `2` server error, `3` auth error (Anthropic-CLI parity).

### Authentication

OAuth 2.1 device-code grant (RFC 8628) per the IETF "OAuth 2.1 for Browserless and Input-Constrained Devices" specification, the same flow used by `gh auth login` and `stripe login`. Token storage:

- macOS: Keychain Services.
- Linux: `secret-service` D-Bus (libsecret) — fallback to encrypted file under `$XDG_DATA_HOME/oya/credentials` when running in CI / containers.
- Windows: Credential Manager.

Refresh tokens rotated per OAuth-2.1 best practice (single-use refresh tokens).

### Transport

`oya-tenant-cli` calls the public REST surface via the public SDK (`oya-shared-public-sdk-http`). No direct gRPC from the tenant binary at v0.1 — gRPC is reserved for sibling-µservice traffic per ADR-0145.

## Alternatives considered

### A. Web-only tenant surface (no CLI)
- Pros: zero binary distribution burden; one rendering tier (the SPA).
- Cons: power users, support engineers, CI/CD pipelines, and agentic tenants (per ADR-0021) cannot script the platform. Every hyperscaler reference (AWS, GCP, Azure, Stripe, GitHub, Vercel, Cloudflare, Fly.io, Anthropic) ships a CLI; tenants will use a community-written shim otherwise.
- **Rejected**: violates hyperscaler-bar parity (audit Row C1); blocks the agentic-tenant use case from ADR-0021.

### B. HTTP-only public API (curl-the-API)
- Pros: zero new code; tenants compose their own commands with curl + jq.
- Cons: every tenant reimplements OAuth-2.1 device-code flow, retry, JSON parsing, pagination, and shell-completion. Engineering cost is borne by the tenant, not Oyatie — visible in support load. Stripe/AWS/GCP explicitly publish CLIs to reduce this load.
- **Rejected**: hides ergonomic cost in tenant codebases instead of paying it once in Oyatie's CLI.

### C. Language SDK only (no CLI)
- Pros: well-typed APIs in Python / TypeScript / Rust / Go — power users automate from their language of choice.
- Cons: cross-cutting CLI commands (e.g. `oya workflow run`) are not language-bound — a CI job in `.github/workflows/*.yml` shell-execs the CLI; a support engineer types it interactively; an LLM agent emits it as a shell snippet. Language SDKs do not cover the ad-hoc + shell-script + agentic-LLM modalities.
- **Rejected**: covers ~60% of the use cases; misses the shell-script + interactive-support modalities entirely.

### D. Extend `oya-dev-cli` with tenant-facing commands
- Pros: one binary, one repo location, one release pipeline.
- Cons: as enumerated in Context — leaks internal gate names; forces 200+ internal-crate dependency closure on tenants; couples Tier-A tenant semver to Tier-D internal semver; tenants ship with a binary that exposes `gate validate <internal-name>` commands.
- **Rejected**: confidentiality leak + dependency-closure explosion + semver-tier collision.

### E. Tenant CLI written in TypeScript on Node.js
- Pros: faster iteration; large ecosystem of CLI libraries (oclif, commander.js); same language as the SPA.
- Cons: Node.js runtime adds 40MB to the tenant install; node-keytar for credential storage is unmaintained; mismatched tooling vs the Rust-first on-prem tooling authority (ADR-0120); CI ergonomics suffer (slow cold-start vs a Rust static binary). Stripe-CLI is Go; AWS-CLI v2 is Python (with bundled interpreter); both ship as static binaries to avoid runtime fragility — Rust gives us a smaller static binary than either.
- **Rejected**: violates ADR-0120 Rust-first tooling authority; ships a heavier runtime than the AWS/Stripe references it parities.

## Consequences

### Positive

1. **Hyperscaler-parity** — tenants get an `oya` CLI that mirrors `aws`, `stripe`, `gh`, `claude` ergonomics. Audit Row C1 closed.
2. **Tier-A semver isolation** — tenant CLI breaking changes cost an ADR + 18-month sunset window. Internal gate-CLI iterates at Tier-D pace without dragging tenant compatibility.
3. **Minimal dependency closure** — `oya-tenant-cli` depends on the public SDK only (≤20 crates), reducing CVE exposure surface and supply-chain attack surface (per ADR-0039).
4. **Static binary distribution** — single Rust binary per platform. No runtime install on the tenant side; aligns with Stripe-CLI / GitHub-CLI distribution shape.
5. **Agentic-tenant compatible** — `--output json` and exit-code contract make the CLI a deterministic surface for ADR-0021 agentic tenants and for LLM-emitted shell snippets.

### Negative

1. **Second binary to maintain** — the council-api-sdk owns release cadence, distribution channels, and the per-platform installer matrix (Homebrew tap, apt repo, MSI, container).
2. **Public-SDK crate coupling** — `oya-shared-public-sdk-*` family must exist and be Tier-A semver-protected. Filed as an upstream prerequisite for any v1.0 CLI release.
3. **Auth state coupling to OS credential store** — three OS-specific code paths (Keychain / libsecret / Credential Manager). Tested via the SDK's portable credential-store trait; per-OS adapters validated in CI matrix.
4. **Public-API surface area policed by the CLI** — every new public API method must consider whether to expose a CLI command. The council-api-sdk owns this gate at PR-review time.

### Operational

1. `crates/oya-tenant-cli/` lives in the workspace as a sibling of `crates/oya-dev-cli/`. Binary name in tenant channels is `oya`; the workspace target is `oya-tenant`, and `cargo run -p oya-tenant-cli -- ...` uses that default target.
2. Tenant-CLI release cadence: monthly minor (additive) + on-demand patch. Major bump requires an ADR per ADR-0037.
3. Distribution channels (M01-foundation slice):
   - `brew tap oyatie/oya && brew install oya` (macOS, Linux Homebrew).
   - `apt install oya` from a signed Oyatie repo (Linux).
   - `winget install Oyatie.Oya` (Windows).
   - `ghcr.io/oyatie/oya:<semver>` (container image, distroless-nonroot per ADR-0146).
4. Shell completion: `oya completion bash|zsh|fish|powershell` emits scripts (clap-complete pattern).
5. Telemetry: opt-in via `oya telemetry enable`; disabled by default per privacy-by-default (ADR-0008 data-use boundary).

### Error model

Tenant-CLI errors map to the public-API error envelope per ADR-0011. The CLI surfaces:

- `oya: error: <human message> (code=<api-error-code>, request_id=<id>)` on stderr.
- Exit code per the contract in the Decision section.
- `--output json` emits `{"error": {...}, "request_id": "<id>"}` instead of the success payload.
- Network errors retried per the SDK's standard policy (3 retries, exponential backoff with jitter 1s/2s/4s). Idempotent commands (`GET`, `list`) auto-retry; mutating commands (`POST`, `create`, `delete`) DO NOT auto-retry unless the SDK confirms idempotency-key receipt.

### Perf + size budgets

- Binary size ≤25 MB stripped (compared to `gh` ~30 MB, `stripe` ~25 MB, `aws` v2 ~40 MB).
- Cold-start to first-byte-of-output ≤80ms p99 on a 2020 baseline laptop (compared to `aws` v2 ~250ms with bundled Python interpreter; we target Rust-static-binary class).
- Memory ≤50 MB RSS for any single command (TUI commands excluded).
- Per-command request budget ≤3 backend calls for nominal-success paths (audited via `--debug` mode dumping the trace tree).

### Versioning and compatibility matrix

Tenant CLI version `oya-tenant-cli vX.Y.Z` corresponds to public-API `vN` per a compatibility matrix published in the public docs. The CLI emits a deprecation warning when the connected backend's API version exceeds the CLI's compiled compatibility range; tenants are nudged via the warning to upgrade. The compatibility window is N-2 backend minors per ADR-0037.

### Audit-chain integration

Every state-changing CLI command emits a tenant-side audit hint (`X-Oya-Cli-Version`, `X-Oya-Cli-Invocation-Id`) that the backend µservice stamps into its audit-chain seal per ADR-0003. Tenants can query their own audit chain via `oya audit chain query --since <ts>` and correlate CLI invocations to seals end-to-end.

### Migration / rollout plan

1. M01 slice: skeleton crate lands (this ADR companion); `oya auth login`, `oya version`, `oya completion` only.
2. M01.5: `oya workflow run`, `oya tasks create`, `oya messenger send` (Tier-A backends already live).
3. M02: `oya foundry capability invoke`, `oya ontology entity get`.
4. M03: `oya audit chain query`, `oya status` (depends on ADR-0168 status-page live).
5. v1.0 cut at M03 + 30 days bake.

### Localization

CLI message catalog supports the regional packs per ADR-0010. Messages externalized via `fluent-rs` localization; en-US is canonical base + ko-KR pack #1 per the canonical-base-localization memory. Backend-emitted error messages localized server-side; CLI surfaces them unchanged.

### Configuration precedence

Where multiple sources specify the same setting (e.g. API endpoint, output format), precedence is:

1. Command-line flag (highest).
2. Environment variable (`OYA_API_ENDPOINT`, `OYA_OUTPUT`, etc.).
3. Per-project config file `./.oya/config.toml` (when present and the CWD is inside a tenant project).
4. Per-user config file `$XDG_CONFIG_HOME/oya/config.toml`.
5. Hardcoded default (lowest).

This mirrors the AWS CLI / GitHub CLI precedence and lets per-project pinning override per-user defaults.

## References

- AWS CLI v2 — https://docs.aws.amazon.com/cli/latest/userguide/ — canonical reference for "every API method as a command".
- Stripe CLI — https://stripe.com/docs/stripe-cli — OAuth-2.1 login (`stripe login`), event simulation (`stripe trigger`), local webhook tunneling (`stripe listen`).
- GitHub CLI (`gh`) — https://cli.github.com/manual/ — high-level workflow commands wrapping multi-step API calls; OAuth device-code flow.
- Anthropic Claude CLI — agentic-tenant CLI shape; structured JSON output contract.
- Vercel CLI — `vercel` — project + deployment-centric CLI; single Rust-friendly static binary distribution.
- Fly.io `flyctl` — Go-based static binary; OAuth device-code login.
- RFC 8628 — OAuth 2.0 Device Authorization Grant — https://datatracker.ietf.org/doc/html/rfc8628
- IETF OAuth 2.1 draft — https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1
- ADR-0002 — tenant + identity kernel (auth substrate this CLI uses).
- ADR-0011 — cross-microservice contract registry (the CLI invokes Tier-A contracts from this registry).
- ADR-0021 — Foundry capability registry + MCP gateway (agentic-tenant invocation path).
- ADR-0037 — public API stability tiers (CLI is Tier-A).
- ADR-0120 — Rust-first on-prem tooling authority.
- ADR-0146 — distroless non-root container base image (CLI container image base).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C1 closes here.
