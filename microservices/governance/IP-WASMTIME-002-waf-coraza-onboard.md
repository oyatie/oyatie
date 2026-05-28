---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: governance
impl_plan_id: IP-WASMTIME-002-waf-coraza-onboard
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: oya-governance
co_owners: [axis-security, axis-platform-edge]
date: 2026-05-18
related_adrs: [ADR-0200, ADR-0182, ADR-0064]
acceptance_lanes: [waf-correctness, perf-edge-p99, oya-governance-promotion-readiness]
depends_on: [IP-WASMTIME-001]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-WASMTIME-002 — Coraza WAF filter onboarding

## Goal

Compile Coraza (Apache-2.0 OSS WAF engine) to component-model bytecode and load it via the Envoy WASM filter substrate (IP-WASMTIME-001) so all north-south traffic is inspected against OWASP CRS 4.x rules. Per-pack overlays select which rules are active (EU pack enables GDPR-leak rules; US-healthcare enables PHI-leakage rules) without rebuilding the bytecode. Per ADR-0200 + ADR-0182, WAF is the canonical north-south defense layer of the governance µservice.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/governance/src/waf/coraza/build/Cargo.toml` | create | ~40 LoC; build orchestration shim |
| `microservices/governance/src/waf/coraza/build/build.sh` | create | ~80 LoC; `tinygo` compile pipeline producing `coraza-envoy-filter.wasm` component-model bytecode |
| `microservices/governance/src/waf/coraza/build/Dockerfile.builder` | create | ~60 LoC; reproducible build image (pinned go + tinygo + wabt) |
| `microservices/governance/src/waf/coraza/rules/owasp-crs-4.x.tar.gz` | vendor (pinned) | OWASP Core Rule Set 4.x snapshot + SHA256 manifest |
| `microservices/governance/src/waf/coraza/overlays/eu-gdpr-rules.conf` | create | ~80 LoC; EU pack: GDPR-leak rule set |
| `microservices/governance/src/waf/coraza/overlays/us-healthcare-phi-rules.conf` | create | ~100 LoC; US-healthcare pack: PHI-leakage rules |
| `microservices/governance/src/waf/coraza/overlays/global-default.conf` | create | ~60 LoC; baseline (OWASP CRS only) |
| `microservices/governance/iac/envoy-filter-coraza.yaml` | create | ~120 LoC; EnvoyFilter resource attaching the WASM filter to the ingress gateway |
| `microservices/governance/tests/waf_correctness.rs` | create | ~260 LoC; 6 black-box tests via Envoy fixture |
| `microservices/governance/tests/waf_perf.rs` | create | ~120 LoC; latency-overhead bench |
| `microservices/governance/runbooks/waf-rule-disable.md` | create | ~100 LoC; per-rule selective-disable playbook |
| `microservices/governance/decisions/ADR-0200.md` | append §"Coraza onboard landed" | +6 LoC |

## Code shape

`overlays/us-healthcare-phi-rules.conf` (excerpt):

```text
# US-healthcare pack overlay — PHI leakage detection
# Pack: us-healthcare (per ADR-0117)
# Enabled rules:
SecRule RESPONSE_BODY "@rx \b(\d{3}-\d{2}-\d{4})\b" \
  "id:9100,phase:4,deny,status:451,t:none,\
   msg:'SSN pattern detected in response body',\
   tag:'OYA/PACK/US-HEALTHCARE/PHI'"
SecRule RESPONSE_BODY "@rx \b(MRN[:#]?\s*\d{6,10})\b" \
  "id:9101,phase:4,deny,status:451,t:none,\
   msg:'Medical Record Number leak detected',\
   tag:'OYA/PACK/US-HEALTHCARE/PHI'"
SecRule RESPONSE_HEADERS:Content-Disposition "@rx attachment;filename=.*\.(dicom|hl7)" \
  "id:9102,phase:3,deny,status:451,t:none,\
   msg:'Raw clinical artifact leak',\
   tag:'OYA/PACK/US-HEALTHCARE/PHI'"
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `waf_blocks_known_sqli_payload` | tests/waf_correctness.rs | `' OR 1=1 --` → 403 + log entry |
| `waf_blocks_known_xss_payload` | tests/waf_correctness.rs | `<script>alert(1)</script>` → 403 + log entry |
| `waf_passes_legitimate_request_unmodified` | tests/waf_correctness.rs | Normal request → 200; body byte-identical |
| `waf_eu_pack_blocks_gdpr_leak` | tests/waf_correctness.rs | EU pack: response containing EU SSN → 451 |
| `waf_us_healthcare_blocks_phi_leak` | tests/waf_correctness.rs | US-healthcare: response containing SSN pattern → 451 |
| `waf_overlay_disabled_rule_passes_through` | tests/waf_correctness.rs | Overlay disables rule 9100 → SSN passes |
| `waf_added_p99_latency_le_5ms` | tests/waf_perf.rs | p99 added latency ≤ 5ms over 10k req bench |
| `waf_per_rule_hit_metric_emitted` | tests/waf_correctness.rs | Each rule hit increments `coraza_rule_hit_total{rule_id}` |

Minimum 4 required; 8 specified.

## Evidence to emit

- `evidence/microservices/governance/waf-correctness-{date}.json` — pass/fail per test
- `evidence/microservices/governance/waf-perf-overhead-{date}.json` — latency histogram (p50/p95/p99)
- Audit-chain seal: `oya audit-chain seal --kind waf-correctness --ms governance --window 30d`
- Metrics: `coraza_rule_hit_total{rule_id,pack}`, `oya_governance_waf_added_latency_ms_bucket`, `oya_governance_waf_request_blocked_total{rule_id}`

## Rollback procedure

1. Revert ChangeSet for the WASM build + overlays + EnvoyFilter manifest.
2. `kubectl delete envoyfilter waf-coraza -n istio-system` → traffic flows without WAF inspection.
3. Banner displayed in governance dashboard: "WAF disabled — north-south inspection paused".
4. Per-rule selective-disable runbook is the lighter-weight alternative; full rollback only on bytecode regression.
5. Emit rollback evidence JSON.

## Blocking dependencies

- IP-WASMTIME-001 — Envoy WASM filter substrate.
- ADR-0200 — WAF canonical.
- ADR-0182 — Envoy as ingress data plane.
- OWASP CRS 4.x — pinned + checksum.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate waf-correctness --target coraza
cargo run -p oya-dev-cli -- gate validate perf-edge-p99 --component waf-coraza
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice governance
cargo test -p oya-governance-waf-coraza --tests
```

## Halt conditions

- Sanity SQLi/XSS not blocked: STOP, WAF non-functional.
- p99 added latency > 5ms: STOP, perf regression.
- Legitimate-request modification (body byte mismatch): STOP, correctness regression.

## Exit criteria

1. All 8 tests green.
2. `waf-correctness`, `perf-edge-p99`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. EnvoyFilter live in dev cluster; WAF hit-rate visible in governance dashboard.
5. Runbook published.
6. ADR-0200 status updated.

## Next IP

[`IP-WASMTIME-003-regulatory-response-shaper.md`](IP-WASMTIME-003-regulatory-response-shaper.md)

## References

- ADR-0200 — WAF canonical.
- ADR-0182 — Envoy ingress data plane.
- ADR-0117 — residency + per-pack overlays.
- Coraza WAF upstream — `https://coraza.io/`.
- OWASP CRS 4.x — `https://coreruleset.org/`.
- Envoy WASM extension — `https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/wasm_filter`.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
