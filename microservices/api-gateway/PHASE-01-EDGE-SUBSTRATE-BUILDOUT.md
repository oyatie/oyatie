# PHASE-01 — api-gateway Edge Substrate Buildout

**Status:** In flight (Wave-3-A)
**Authority:** ADR-0157 + ADR-0182 + ADR-0253 + ADR-0297 (in flight).
**Hyperscaler precedent:** Cloudflare Edge + Envoy data-plane + Apigee management-plane.

---

## A — Why this phase

The api-gateway is the Tier-0 edge µservice — the front door to all of oyatie. Before any product can ship, this substrate must be hyperscaler-grade. The 2026-05-20 corpus snapshot lists api-gateway as **below floor (16 artifacts)** under `docs/standards/documentation-rigor.md`. Phase-01 brings it to ≥110 artifacts, full PR-143 shape, ECH + PQC + abuse-defence wired, ready to gate all Wave-3 µservices.

PHASE-01 is the substrate-build phase. PHASE-02 (future) will be the multi-region scale-out + cross-cell mesh. PHASE-03 (future) is the partner tenant_class + private-link integration.

---

## B — What "done" looks like

A request arrives. Every hop in §B of `ARCHITECTURE.md` is covered by a runbook, a Cedar fragment, an IaC manifest, a contract, a dashboard, an SLO, and an audit-event-class. The intern-buildability test (per `docs/standards/documentation-rigor.md` §1) passes: an intern reading `PRD.md` + `ARCHITECTURE.md` can stand up a Tier-0 edge cell from cold.

Acceptance criteria:

1. **Artifact count ≥110.** PR-143 shape match.
2. **All 28 rows** of the §3.2.1 ADR-adherence matrix answered in `ARCHITECTURE.md` + `compliance.md`.
3. **Six-hops graph traversability** (per §3.1) from `docs/README.md` reaches every primitive.
4. **Zero TBD/FIXME/TODO** in canonical bodies.
5. **HTTP/3 default** verified by `iac/envoy-config.yaml` listener block.
6. **TLS 1.3 strict** verified by `iac/cert-manager.yaml` issuer block.
7. **ECH advertised** verified by `iac/ech-config.yaml` HTTPS RR block.
8. **PQC hybrid offered** verified by `iac/pqc-cert.yaml` cert chain.
9. **Bot-management** verified by `iac/edge-waf.yaml` rule set.
10. **CI lane `oya-governance-microservice-doc-suite`** green for `api-gateway`.

---

## C — Deliverables (PHASE-01 scope)

### C-1. Strategic docs

- `PRD.md` — extended from 117 lines to ≥1500 lines (per §2 PRD rigor); ≥40 stories, ≥6 UX flows, B2C+B2B personas, compliance map.
- `PHASE-01-EDGE-SUBSTRATE-BUILDOUT.md` — this doc.
- `threat-model.md` — STRIDE + LINDDUN; per §1.1 / §3.2.3.
- `dpia.md` — DPIA per GDPR Article 35 for edge processing.
- `ARCHITECTURE.md` — Tier-0 edge layer-by-layer trace.

### C-2. Architecture + ops docs

- `README.md`, `CHANGELOG.md`
- `capacity-model.md`, `cost-budget.md` (extend), `failure-modes.md` (extend)
- `multi-region.md`, `incident-response.md`, `backfill-replay.md`, `compliance.md`
- `competitor-parity-matrix.md`, `sdk-plan.md`

### C-3. Cedar policies (≥6)

- `policy/route-authorization.cedar`
- `policy/rate-limit.cedar`
- `policy/abuse-defence.cedar` ← per §3.2.3
- `policy/tls-policy.cedar`
- `policy/sov-cloud-overlay.cedar`
- `policy/tenant-scope.cedar` (extend existing)
- `policy/auditor-scope.cedar`
- `policy/ci-scope.cedar`
- `policy/public-read.cedar`
- `policy/data-residency.md` (extend)

### C-4. Runbooks (≥6)

- `runbooks/ddos-mitigation.md`
- `runbooks/rate-limit-saturation.md`
- `runbooks/tls-cert-rotation.md`
- `runbooks/bot-storm.md`
- `runbooks/edge-cache-poisoning.md`
- `runbooks/blue-green-rollback.md`
- `runbooks/h3-fallback-verification.md`
- `runbooks/ech-key-rotation.md`
- `runbooks/pqc-cert-rotation.md`
- `runbooks/circuit-breaker-engaged.md`
- `runbooks/cell-evac.md`
- `runbooks/edge-admission-regression.md` (extend existing)

### C-5. Contracts (≥3)

- `contracts/api-gateway.openapi.yaml` (extend; routing-control-plane API, OpenAPI 3.2.0).
- `contracts/api-gateway.asyncapi.yaml` (extend; AsyncAPI 3.1.0 routing-events).
- `contracts/api_gateway.proto` (extend; proto3 gRPC management plane).
- `contracts/metric-naming-convention.md`.

### C-6. Capabilities (≥3)

- `capabilities/north-south-request-admission.yaml` (extend existing).
- `capabilities/edge-cedar-eval.yaml`.
- `capabilities/canary-route-shift.yaml`.

### C-7. Dashboards (≥3)

- `dashboards/edge-overview.json` + `.md`
- `dashboards/rate-limit-hits.json` + `.md`
- `dashboards/tls-health.json` + `.md`
- `dashboards/bot-score-distribution.json` + `.md`

### C-8. SLOs (≥4)

- `slos/edge-availability.openslo.yaml`
- `slos/edge-latency-p50.openslo.yaml`
- `slos/edge-latency-p95.openslo.yaml`
- `slos/edge-latency-p99.openslo.yaml`
- `slos/tls-handshake-success.openslo.yaml`
- `slos/h3-negotiation-rate.openslo.yaml`

### C-9. IPs (≥15)

- IP-001 (existing) → extend.
- IP-002..IP-016 — domain/kernel/usecase/adapter/rest/grpc/worker × routing/rate-limit/auth-handoff/abuse-defence layers.

### C-10. Catalog records (≥11)

- One per crate per BC×layer: `oya-api-gateway-routing-{domain,kernel,usecase,adapter,api,rest,worker,sdk}`, `oya-api-gateway-rate-limit-{...}`, `oya-api-gateway-abuse-defence-{...}`.

### C-11. IaC (≥8)

- `iac/k8s-deployment.yaml`
- `iac/envoy-config.yaml`
- `iac/cloudflare-config.yaml`
- `iac/cert-manager.yaml`
- `iac/ech-config.yaml`
- `iac/pqc-cert.yaml`
- `iac/edge-waf.yaml`
- `iac/spire-trust-bundle.yaml`
- `iac/csp-defaults.yaml`
- `iac/cert-manager-hsts-preload.yaml`

### C-12. Manifest + scorecard + audit-findings

- `manifest.json` (extend with all the new artifacts).
- `scorecards/overrides.json`.
- `AUDIT-FINDINGS-2026-05-18.json` (existing) + `AUDIT-FINDINGS-2026-05-20.json` (this wave).

---

## D — Sequencing

Day 0–1: Strategic docs + ARCHITECTURE.
Day 1–2: Cedar policies + IaC manifests.
Day 2–3: Runbooks + contracts + capabilities.
Day 3–4: SLOs + dashboards.
Day 4–5: IPs + catalog records.
Day 5: Manifest sync + scorecard + audit-findings.
Day 5 EOD: Six-hops graph walk green; CI lane `oya-governance-microservice-doc-suite` green.

---

## E — Risks

| Risk | Mitigation |
|---|---|
| ECH config rotation breaks legacy clients | Graceful degradation — ECH-disabled clients fall through to standard TLS 1.3. |
| PQC handshake adds latency | X25519MLKEM768 measured at +0.2ms p50 over X25519; budget absorbed. |
| Bot-score model false positives lock out legitimate users | Friendly-crawler always-pass list + CAPTCHA-on-suspicion (not CAPTCHA-by-default) + human-rights review of model weights. |
| Cedar fragment soak (≥60s) delays emergency edge changes | Emergency rollback path: `runbooks/edge-admission-regression.md` provides per-cell bypass with elevated audit trail. |
| QUIC-blocked corporate networks lose h3 → h2 fallback | Verified via `runbooks/h3-fallback-verification.md`; monitored via SLO `h3-negotiation-rate`. |

---

## F — References

- `microservices/api-gateway/PRD.md`
- `microservices/api-gateway/ARCHITECTURE.md`
- `docs/standards/documentation-rigor.md` §1.1 / §1.2 / §3.2.1 / §3.2.3
- ADR-0157, ADR-0182, ADR-0253, ADR-0297 (in flight).
- Wave-3-A buildout coordination.
