---
id: ADR-0542
title: "Cloud-Intelligence XPROXY External-Proxy Parity Lane: commissioning and governance path"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-10
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-0709]
depends_on: [ADR-0363, ADR-0510, ADR-0515, ADR-0516]
amends: []
related: [ADR-0131, ADR-0132, ADR-0384, ADR-0540, ADR-0541]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: XPROXY parity lane for intelligence

# ADR-0542: Cloud-Intelligence XPROXY External-Proxy Parity Lane — commissioning and governance path

## Status

**Proposed - 2026-06-10 (founder in-session sanction 2026-06-10; ratification pending governance pipeline).**

## Context

The cloud/cloud-intelligence service requires a parity lane that commissions and validates
external-proxy (XPROXY) capability: the ability to proxy requests to external LLM providers
(OpenAI-compatible, Gemini-native, Anthropic subscription) through the owned cloud-intelligence
gateway, with full BNF-canonical crate structure, manifest hygiene, and accounting justification.

PR #644 implements this lane. The pre-merge baseline analysis (FRIC-1781112000 class, session
2026-06-10) identified approximately 90 new accounting debt keys that would be introduced without
proper governance surfaces. The founder sanctioned the lane on 2026-06-10 with the directive:
"start from 644" — authorising the lane to proceed subject to the fixes described in this ADR.

### Authority chain

1. **Founder in-session sanction 2026-06-10** — explicit "start from 644" authorization.
2. **ADR-0363** (VCS/governance substrate) — PR against dev enters the governance pipeline; merge
   requires reviewer-agent APPROVE plus cloud-ci/oya-ci green (ADR-0515 Tide admission).
3. **ADR-0515** — cloud-ci/oya-ci owns merge admission; all accounting gate keys must be zero
   net-new unjustified at merge.
4. **ADR-0510** (transient adapters) — external LLM providers (OpenAI, Gemini, Anthropic
   subscription) are transient, adapter-absorbed infrastructure; owned ports define the interface,
   adapters absorb provider specifics without leaking provider wording into owned contracts.
5. **Owned-stack doctrine** (founder 2026-06-09) — all adapters are Rust-native, trait-shaped for
   the owned-stack destination; provider-specific crates carry the `adapter` BNF role suffix.

## Decision

Commission the XPROXY external-proxy parity lane under `cloud/cloud-intelligence/` with the
following governance constraints:

1. **BNF-canonical crate naming**: all new crates carry a BNF role suffix from the approved
   registry (`kernel|domain|usecase|app|adapter|infrastructure|cli|rest|grpc|graphql|worker|sdk|api`).
   - `oya-cloud-intelligence-worker` (plural corrected; role: `worker` — K8s deployment/controller registry)
   - `oya-cloud-intelligence-ops-infrastructure` (role: `infrastructure` — read-only operational views)
   - `oya-cloud-intelligence-tool-compat-kernel` (role: `kernel` — pure tool compatibility logic)
   - `oya-cloud-intelligence-translation-kernel` (role: `kernel` — pure protocol translation logic)
   - `oya-cloud-intelligence-wire-kernel` (role: `kernel` — pure wire policy/header-filter logic)

2. **Manifest hygiene**: all crates carry `[lints] workspace = true` and `[lib] doctest = false`
   mirroring sibling crate conventions.

3. **Accounting justification**: all files introduced by this lane are justified via this ADR
   (governance surface) and the multispectrum evidence file listed in the Governed Surfaces section
   below. The accounting gate must reach zero net-new unjustified keys at merge.

4. **OWNERS coverage**: `cloud/cloud-intelligence/OWNERS` anchors team ownership (`axis-cloud-platform`)
   for all files under the service, resolving `unowned` accounting keys.

5. **Contracts are owned ports**: `cloud/cloud-intelligence/contracts/` declares OpenAPI/AsyncAPI/proto
   surfaces using owned port names (`owned-secret-provider-port`, `owned-policy-engine-port`) per
   ADR-0510; no direct transient-engine wording (OpenBao, Cedar, Vault) appears in contracts.

## Governed surfaces

The following repo paths are governed by this ADR. The accounting gate validates that each is
justified (this ADR is the justification reference):

```
cloud/cloud-intelligence/contracts/BUCK
cloud/cloud-intelligence/contracts/tests/transient_adapter_boundary.rs
cloud/cloud-intelligence/contracts/tests/xproxy_contract_parity.rs
intelligence/adapters/gemini-adapter/BUCK
intelligence/adapters/gemini-adapter/Cargo.toml
intelligence/adapters/gemini-adapter/src/lib.rs
intelligence/adapters/gemini-adapter/tests/gemini_adapter.rs
intelligence/core/kernel/capability-parity/external-proxy-reference-20260610.json
intelligence/core/kernel/capability-parity/external-proxy-reference-draft-targets-20260610.json
intelligence/core/kernel/src/model_routing.rs
intelligence/core/kernel/src/safety.rs
intelligence/core/kernel/src/xproxy_parity.rs
intelligence/core/kernel/tests/cloud_intelligence_safety_guardrails.rs
intelligence/core/kernel/tests/xproxy_capability_parity.rs
intelligence/core/kernel/tests/xproxy_model_routing_matrix.rs
intelligence/core/kernel/tests/xproxy_pool_headroom_stickiness.rs
intelligence/adapters/ops-infrastructure/BUCK
intelligence/adapters/ops-infrastructure/Cargo.toml
intelligence/adapters/ops-infrastructure/src/lib.rs
intelligence/adapters/ops-infrastructure/tests/xproxy_ops_readonly.rs
intelligence/adapters/rest/tests/d8_secret_provider_envelope_encryption.rs
intelligence/core/tool-compat-kernel/BUCK
intelligence/core/tool-compat-kernel/Cargo.toml
intelligence/core/tool-compat-kernel/src/lib.rs
intelligence/core/tool-compat-kernel/tests/xproxy_tool_compat.rs
intelligence/core/translation-kernel/BUCK
intelligence/core/translation-kernel/Cargo.toml
intelligence/core/translation-kernel/src/lib.rs
intelligence/core/translation-kernel/tests/xproxy_translation_fixtures.rs
intelligence/core/wire-kernel/BUCK
intelligence/core/wire-kernel/Cargo.toml
intelligence/core/wire-kernel/src/lib.rs
intelligence/core/wire-kernel/tests/xproxy_wire_policy.rs
intelligence/facade/worker/BUCK
intelligence/facade/worker/Cargo.toml
intelligence/facade/worker/src/lib.rs
intelligence/facade/worker/tests/xproxy_worker_ownership.rs
evidence/multispectrum/cloud-intelligence-xproxy-20260610-1781062794.json
evidence/multispectrum/cloud-intelligence-canary-status-salvage-20260612-1781239694.json
```

### Addendum 2026-06-12 — #663 status-surface salvage continuation

The XPROXY lane ran as two parallel branches (#644 merged; #663 went stale) sharing the
emission ID `cloud-intelligence-xproxy-20260610` and the same evidence path with divergent
content — an evidence-artifact collision (friction ledger row FRIC-1781300000). The
genuinely-unmerged subset of #663 (agent-runtime/agent-schedule/parity-canary read-only
status surfaces across OpenAPI/proto/AsyncAPI, kernel `RedactedSeatStatus`, REST admin read
routes, and worker status-ownership types) is re-derived from current dev as a single-concern
salvage PR under the fresh emission ID above
(`evidence/multispectrum/cloud-intelligence-canary-status-salvage-20260612-1781239694.json`,
change_id `cloud-intelligence-canary-status-salvage-20260612`). That evidence artifact is a
governed surface of this lane and this ADR is its justification reference; all source files it
touches were already governed by this ADR's list above. Per-lane unique emission IDs close the
collision class.

### Addendum 2026-06-26 — PR #895 overage-guard continuation

These additive `intelligence/core/kernel` surfaces are governed by the same XPROXY parity lane and ADR-0384 OAuth subscription-pool kernel boundary:

```
intelligence/core/kernel/src/overage_guard.rs
intelligence/core/kernel/tests/proptest_overage_guard.rs
```

### Addendum 2026-06-26 — PR #896 session-pinning continuation

These additive `intelligence/core/kernel` surfaces are governed by the same XPROXY parity lane and ADR-0384 OAuth subscription-pool kernel boundary:

```
intelligence/core/kernel/src/session.rs
intelligence/core/kernel/tests/proptest_session_pinning.rs
intelligence/core/kernel/tests/session_pinning.rs
```

### Addendum 2026-06-26 — PR #897 cost-tracking continuation

These additive `intelligence/core/kernel` surfaces are governed by the same XPROXY parity lane and ADR-0384 OAuth subscription-pool kernel boundary:

```
intelligence/core/kernel/src/cost.rs
intelligence/core/kernel/tests/cost_pricebook.rs
```
## Consequences

**Positive:**
- Zero net-new accounting debt keys at merge: all introduced surfaces are governed by this ADR.
- BNF-canonical crate names enable the `cloud-ci-bnf-layer-suffix` gate to clear without
  baseline entries.
- Manifest hygiene (`[lints] workspace = true`, `doctest = false`) clears `cloud-ci-manifest-hygiene`.
- OWNERS coverage clears `unowned` keys for the entire cloud-intelligence service tree.
- The XPROXY lane demonstrates that external-proxy adapters can be commissioned without laundring
  debt through baseline gates.

**Negative / constraints:**
- The gemini-adapter crate introduces a non-trivial async/HTTP dependency (reqwest, futures) that
  must be kept adapter-isolated per ADR-0510; owned kernel crates must not take this dependency.
- Protocol translation logic (`translation-kernel`) must remain pure (no I/O, no external deps
  beyond serde) to stay `kernel`-classified; adding HTTP/async moves it to `adapter` class.

## Alternatives considered

- **Inline all XPROXY logic into the existing kernel crate**: rejected — violates ADR-0132
  single-concern constraint and makes the owned-port / adapter boundary invisible.
- **Leave non-canonical crate names**: rejected — the `cloud-ci-bnf-layer-suffix` gate would
  block or launder debt on each new violation; canonical naming is a born-blocking requirement.
