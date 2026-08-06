---
id: ADR-0036
status: Proposed
doc_status: published
---

# ADR-0036: Plugin substrate — Wasmtime + WASI Preview 2 with capability-gated context, Cosign signing, trust tiers, marketplace economics

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0011, ADR-0029, ADR-0033, ADR-0034, ADR-0037, ADR-0038, ADR-0039

---

## Context

Plugins are how third parties extend the ecosystem without forking it: a tenant-supplied analyzer in Workspace Sheets, a vendor-supplied EMR adapter in Vertical-Healthcare, an ISV-supplied custom retrieval plugin in Search, a partner-supplied document parser in Drive. Without a structured plugin substrate, every plugin becomes a security incident in waiting (RCE on host) or a stability incident in waiting (a misbehaving plugin tanks the host process).

The pack-of-19 foundation ADRs decided plugins are a first-class concern but did not pin the runtime, the capability gating, the trust tiers, the signing chain, or the marketplace economics. This ADR pins all five so that a plugin author has a single SDK to target, a tenant admin has a single trust knob to turn, and a regulator has a single signing chain to audit.

---

## Decision

We adopt **Wasmtime + WASI Preview 2** as the canonical plugin runtime; **capability-gated `PluginContext`** as the only API surface plugins see; **Cosign keyless signing + Rekor transparency log** as the signing chain; **three trust tiers** (verified-isv / community / experimental); a **per-plugin per-tenant resource cap** model; and a **marketplace economics** spec (revenue share + payout cadence).

### Runtime: Wasmtime + WASI Preview 2

```rust
// crates/oya-intelligence-plugin-runtime-kernel
pub struct PluginRuntime {
    pub engine: wasmtime::Engine,
    pub linker: wasmtime::component::Linker<PluginContext>,
    pub store_factory: StoreFactory,
}

pub struct PluginContext {
    pub plugin_id: PluginId,
    pub tenant_id: TenantId,
    pub capabilities: PluginCapabilitySet,  // ADR-0011 capability binding
    pub autonomy_tier: PersonaTier,         // ADR-0007
    pub resource_caps: ResourceCaps,
    pub audit: AuditEmitter,                // ADR-0003
}
```

- **Wasmtime.** Apache-2; Bytecode Alliance; mature; the only credible WASM runtime in Rust.
- **WASI Preview 2.** Component Model + interface types; lets us define plugin interfaces in WIT (WebAssembly Interface Types).
- **No syscalls outside `PluginContext`.** Plugins cannot make network calls, file access, env reads, or process spawns directly; every external interaction goes through the context.

### Capability-gated `PluginContext`

Every plugin declares its required capabilities in its manifest. At load time, the host validates that:

1. The capability is registered in the capability registry (ADR-0011).
2. The plugin's trust tier (below) is allowed to request this capability.
3. The tenant has granted the plugin this capability.

The plugin sees only the capabilities it has been granted. Cedar policy (ADR-0007) gates each capability call.

### Cosign keyless signing + Rekor transparency log

Every plugin artifact is signed via Cosign keyless (Sigstore OIDC). The signature is recorded in the Rekor transparency log; the host verifies signature + transparency-log inclusion before loading. Unsigned plugins cannot load in production tenants (experimental tier exception below).

### Three trust tiers

| Tier | Loading allowed in | Capability scope | Signing requirement |
|---|---|---|---|
| **Verified ISV** | Production tenants by default | Full capability surface (subject to per-capability tenant grant) | Cosign + ISV identity verification + per-release security review |
| **Community** | Production tenants by tenant opt-in | Restricted (no `*.write` capabilities outside the plugin's own tenant data) | Cosign keyless |
| **Experimental** | Development/preview tenants only | Restricted + sandboxed (no PII / PHI / financial data classes) | Cosign keyless OR explicit tenant override |

### Plugin manifest schema

```yaml
# plugin.yaml
plugin_id: "com.example.sheets-analyzer"
version: "1.4.2"
trust_tier_requested: "verified-isv"
capabilities:
  - "workspace.sheets.read"
  - "workspace.sheets.write"
  - "workflow.workspace.notify"
data_classes_required:
  - "TenantContent"
data_classes_forbidden:
  - "PHI"
  - "PCI"
resource_caps:
  cpu_ms_per_invocation: 5000
  memory_mb: 256
  net_kb_per_invocation: 1024
host_compatibility: ">=1.0.0,<2.0.0"
signature: "<cosign>"
```

### Per-plugin per-tenant resource caps

The host enforces:

- CPU time per invocation (default 5s; per-capability override).
- Memory ceiling per instance (default 256MB; per-capability override).
- Network bandwidth per invocation (default 1MB).
- Per-tenant aggregate quota (per-day) per plugin.

Cap exhaustion = invocation aborted, audit-chained event emitted.

### Plugin marketplace economics

- **Revenue share.** Plugin author receives 80% of plugin revenue (tenant pays plugin price); platform retains 20%. Per-plugin pricing model: free / one-time / subscription / usage.
- **Payout cadence.** Monthly; T+30 days; per-author KYC + tax form mandatory.
- **Refund policy.** 14-day full refund for new tenant installs; per-plugin SLA-driven refund for outages.
- **Per-region tax handling.** KR 부가가치세 (VAT) auto-collected per per-pack tax-invoice format (ADR-0028 billing).
- **Per-plugin transparency report.** Monthly: install count, revenue, refund rate, SLA compliance.

### Plugin authoring SDK

| Language | Crate / package |
|---|---|
| **Rust** | `oya-plugin-sdk-rust` (canonical; native WIT bindings) |
| **TypeScript** | `oya-plugin-sdk-ts` (compiles to WASM via Javy / wasm-tools) |
| **Python** | `oya-plugin-sdk-py` (Pyodide WASM build) |
| **Go** | `oya-plugin-sdk-go` (TinyGo WASM build) |

Each SDK ships:
- WIT-derived bindings.
- Capability-call helpers.
- Local development sandbox (`oya plugin dev`).
- Test harness with capability mocks.

### Per-plugin sandboxed dev environment

`oya plugin dev` runs the plugin against a per-author sandboxed Workspace + Vertical tenant; all data classes are `Synthetic`; no real PII can land in dev. CI lane verifies plugin manifest + signature + capability declarations.

### License-tier check for plugin deps

Per ADR (Product License Policy), every plugin dep is scanned at submit-time:

- Forbidden licenses (SSPL, AGPL outside legal isolation, BUSL outside legal isolation, BSL): reject.
- Permitted licenses (Apache-2, MIT, BSD, MPL-2): allow.
- Conditional licenses (LGPL, GPL): require legal isolation analysis per License Policy ADR.

A plugin that depends on a forbidden-license library cannot be submitted to the marketplace.

### Anti-scope

Plugins do not get raw network access, raw filesystem access, or raw process spawning — even at verified-isv tier. Plugins do not own their own audit chain (ADR-0003), do not own their own identity surface (ADR-0002), do not register capabilities (only consume per ADR-0011 governance).

---

## Consequences

### Positive

- Single plugin substrate across all axes: a Workspace plugin, a Vertical plugin, a Search plugin all use the same SDK, same capability model, same signing chain.
- Capability-gated context means plugin security review reduces to "what capabilities does the plugin request, and is the tier allowed to request them?"
- Cosign + Rekor transparency log gives regulators a single audit chain for plugin provenance.
- Trust tiers give tenants a single dial for risk tolerance.
- Marketplace economics give ISVs a credible business model without bespoke commercial terms per ISV.

### Negative

- WASI Preview 2 is recent; some plugin author ergonomics (notably async + complex types) are still maturing.
- Per-plugin resource cap enforcement adds host overhead.
- Per-language SDK maintenance (Rust + TS + Python + Go) is a real recurring cost.
- KR tax handling for international plugin authors is a per-author tax-residency complication.

### Operational

- Per-plugin SLO: load latency, invocation latency, cap-exhaustion rate, signature-verification success.
- Per-plugin install/uninstall audit-chained.
- Per-marketplace fraud detection (e.g. install-count manipulation) reviewed weekly.
- Per-tier promotion review (community → verified-isv) quarterly.
- Plugin sunset cascade per ADR-0038 (deprecation: 6m for stable, 12m for verified-isv).

---

## Alternatives considered

### Alternative A — Lua / JavaScript plugin runtime (no WASM)

- **Pros:** lower author barrier; large existing ecosystems.
- **Cons:** weaker isolation; per-language sandbox effort each time; cannot apply uniform capability gate.
- **Rejected because:** WASM gives uniform isolation across all languages.

### Alternative B — Wasmer instead of Wasmtime

- **Pros:** broadly comparable runtime.
- **Cons:** Wasmer's enterprise license posture has shifted; Wasmtime is BCL Apache-2 only.
- **Rejected because:** license-clean is a hard requirement.

### Alternative C — Native plugin (.so / .dll loaded into host process)

- **Pros:** zero overhead; full-featured.
- **Cons:** zero isolation; one bad plugin crashes the host; security review per plugin is per-binary.
- **Rejected because:** explicitly the failure mode this ADR exists to prevent.

### Alternative D — Container-based plugin (one container per invocation)

- **Pros:** strong isolation.
- **Cons:** container start-time is too high for per-call invocation; container as plugin substrate is fine for batch but wrong for in-line plugin call.
- **Rejected because:** doesn't fit the per-call latency budget.

---

## Open questions

1. **Q1.** Day-1 verified-ISV bar — security review by internal team or third-party? Default: internal; consider third-party at W+24. → owner: `foundry`.
2. **Q2.** Per-plugin signing key rotation cadence? Default: per-release (Cosign keyless rotates per OIDC session, no manual rotation needed). → ADR-0043.
3. **Q3.** Plugin marketplace at GA (Workspace + Vertical + Search), or staged? Default: Workspace at GA; Vertical + Search at W+12. → ADR-0029.
4. **Q4.** Per-plugin observability — opt-in OpenTelemetry hooks or default-on? Default: default-on per ADR-0042 stack; per-plugin export to plugin author with tenant consent. → ADR-0042.
5. **Q5.** Plugin marketplace currency support — KRW + USD only at GA, or full multi-currency? Default: KRW + USD at GA; JPY + EUR + others at W+12. → owner: `foundry`.

---

## References

- `docs/PRD.md` §7 (plugin substrate)
- `docs/DESIGN.md` §4 (plugin runtime), §11 (cross-microservice contracts)
- WebAssembly Component Model + WASI Preview 2 spec; WIT (WebAssembly Interface Types)
- Sigstore Cosign + Rekor specs; SLSA framework
- KR 「부가가치세법」 (VAT for plugin marketplace); 「전자상거래법」
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0029 (workspace), ADR-0033 (vertical pack), ADR-0034 (per-vertical override), ADR-0037 (API stability), ADR-0038 (DSR cascade), ADR-0039 (supply chain), ADR-0042 (observability), ADR-0043 (secrets + KMS)
