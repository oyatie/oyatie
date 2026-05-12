# Oyatie — Plugin Authoring Standard

> **Owner:** `axis-saas` (marketplace) + `axis-foundry` (sandbox + signing) + `ops-security`.
> **Companion:** ADR-0036 (Plugin substrate Wasm + trust), ADR-0023 (Foundry sandbox), ADR-0021 (capability registry + MCP gateway).

## 1. What is an Oyatie plugin

A customer-extensible runtime unit that runs in the Oyatie plugin substrate (Wasmtime-sandboxed; capability-gated) and consumes one or more Oyatie surfaces (workflows / data / capabilities). Author personas: ISV building for marketplace; tenant builder customizing their tenant; partner building cross-tenant integration.

## 2. Plugin manifest

Per ADR-0036:

```yaml
# plugins/my-plugin/manifest.yaml
id: <namespace>.<name>
namespace: <vendor-or-tenant-id>
version: 0.1.0
description: |
  One paragraph for marketplace listing + agent discovery.
trust_tier: verified-isv | community | experimental
languages_supported: [rust, ts, python]   # author's source language; runtime is WASM
runtime:
  target: wasm32-wasi-preview2
  capabilities_required:
    - oya.workspace.mail.send         # explicit allowlist; principle-of-least-privilege
    - oya.platform.tenant.read
  data_classes_touched:
    - PII_QUASI_IDENTIFIER
  autonomy_tier_required: T2
permissions:
  - oya.tenant.read
  - oya.workspace.mail.send
  - oya.foundry.capability.invoke
resource_caps:
  cpu_ms_per_invocation: 200
  memory_mb: 64
  network_egress_allowlist: ["api.partner.com", "api.oyatie.com"]
  filesystem_mount: read-only
sunset_policy:
  deprecation_announce: 6_months_before
  end_of_life_after_announce: 6_months
  migration_target: <new-plugin-id-or-null>
signing:
  algorithm: cosign-keyless
  rekor_anchor: required
```

## 3. Sandboxing

- **Wasmtime + WASI Preview 2** runtime per ADR-0023
- Per-tool resource caps enforced at runtime (CPU / memory / time / syscall allowlist)
- Per-tool network egress allowlist
- Per-tool filesystem mount (per-agent worktree, read-only by default)
- Sandbox spawn p99 < 2s cold / < 100ms warm
- Per-spawn audit emission per ADR-0003
- Sandbox escape detection runbook

## 4. Trust tiers

Per ADR-0036:

| Tier | Validation | Marketplace exposure | Customer notification |
|---|---|---|---|
| **verified-isv** | Oyatie-validated; per-vendor security review; per-quarter re-audit | Default | Verified badge |
| **community** | Cosign-signed; manifest-validated; no Oyatie security review | Filterable; warning shown | Community-tier badge |
| **experimental** | Cosign-signed; manifest-validated; no review; per-tenant explicit opt-in required | Hidden by default | Experimental warning |

## 5. Signing + provenance

Per ADR-0039:
- Cosign keyless signing
- Rekor transparency log entry
- SBOM per release (SPDX 2.3 + CycloneDX 1.5)
- Per-tenant verification on install

## 6. Capability invocation from plugin

Per ADR-0021:
- Plugin invokes Foundry capabilities via `oya-foundry-capability` MCP tool
- Per-plugin per-tenant capability allowlist enforced
- Per-capability autonomy ceiling check
- Per-invocation cost attribution to plugin's tenant

## 7. Marketplace economics

- Revenue share: industry-standard 30% (council-decision per [GTM-PLAN.md §4](../GTM-PLAN.md))
- Per-tier pricing (verified-isv / community / experimental)
- Per-tenant opt-in installation
- Per-plugin uninstall + DSR cascade

## 8. Plugin authoring SDK

Per [TOOLCHAIN.md §3](../TOOLCHAIN.md):
- **Rust → WASM** (canonical SDK)
- **TS / AssemblyScript → WASM** (compatibility SDK)
- **Python → WASM** (where Pyodide-class is acceptable)

`oya build plugin new` scaffolds a new plugin per chosen language.

## 9. Anti-patterns

- Permissions request without justification — marketplace review rejects
- Capability invocation outside `capabilities_required` allowlist — runtime denies
- Resource cap exceeded — sandbox kills + audit-emit
- Network egress to non-allowlisted host — sandbox denies
- Filesystem write outside per-agent worktree — sandbox denies
- Self-published as `verified-isv` without Oyatie review — gated; cannot self-promote

## 10. Sources
ADR-0021/0023/0036/0039, [GTM-PLAN.md](../GTM-PLAN.md), [TOOLCHAIN.md](../TOOLCHAIN.md), CLAUDE.md plugin substrate references.
