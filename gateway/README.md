# api-gateway

Tier-0 edge µservice. North-south request admission for every product surface of oyatie.

**Status:** Wave-3-A buildout (PHASE-01).
**Owner:** axis-network + ops-security.
**Tier:** Substrate / Tier-0 cell (per ADR-0248).

## What this is

The api-gateway is the front door. Every external HTTP request to oyatie hits this µservice first. It is responsible for:

1. TLS 1.3 termination (with ECH + PQC hybrid where supported).
2. HTTP/3 + QUIC negotiation (with h3 → h2 → h1.1 fallback).
3. Anti-bot + anti-spoof + anti-scrape defence-in-depth (per `docs/standards/documentation-rigor.md` §3.2.3).
4. Per-tenant rate-limiting.
5. Cedar-gated route admission.
6. Authentication handoff to identity µservice.
7. Request canonicalisation + body-size enforcement.
8. mTLS-SPIFFE handoff to upstream µservices.
9. Response mediation (security headers, observability trailers).
10. Blue-green + canary routing.
11. Circuit-breaker per upstream.
12. Audit emission per ADR-0263.

It DOES NOT authenticate (handoff to identity), DOES NOT host the policy-engine (caller-side library per ADR-0246+amendment), DOES NOT call Intelligence on the hot path.

## Where to read

| Audience | Start here |
|---|---|
| Intern, cold-start | `PRD.md` → `ARCHITECTURE.md` § A (entry point) |
| Architect | `ARCHITECTURE.md` §B (layer trace) → `threat-model.md` |
| SRE | `runbooks/` index → `failure-modes.md` |
| Compliance | `compliance.md` → `dpia.md` |
| Product manager | `PRD.md` → `competitor-parity-matrix.md` |
| Capacity planner | `capacity-model.md` → `cost-budget.md` |

## What's where

```
microservices/api-gateway/
├── PRD.md                          Product requirements (≥40 stories)
├── ARCHITECTURE.md                 Layer-by-layer trace
├── PHASE-01-EDGE-SUBSTRATE-BUILDOUT.md
├── README.md                       This file
├── CHANGELOG.md
├── threat-model.md                 STRIDE + LINDDUN
├── dpia.md                         GDPR Art. 35 DPIA
├── compliance.md                   Pack overlay roster + ADR-adherence matrix
├── capacity-model.md               Little's Law / queue theory
├── cost-budget.md                  $/M-request frontier
├── failure-modes.md                Failure-mode tree
├── multi-region.md                 Cross-region behaviour
├── incident-response.md            On-call playbook
├── backfill-replay.md              Audit replay model
├── competitor-parity-matrix.md     vs Cloudflare/AWS API Gateway/Apigee/Kong
├── sdk-plan.md                     Client SDK surfaces
├── operational-boundaries.md       What we will/will not do
├── manifest.json                   Artifact roster + ADR roster
├── scorecards/overrides.json       AWS WA / Google SRE / CIS / SLSA scorecards
├── policy/                         Cedar v4 fragments
├── runbooks/                       On-call procedures
├── contracts/                      OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3
├── capabilities/                   v2 capability records
├── dashboards/                     Grafana JSON + cross-reference docs
├── slos/                           OpenSLO v1 manifests
├── catalog/                        Per-crate-per-layer records
├── iac/                            K8s + Envoy + Cloudflare + cert-manager
└── IP-NNN-*.md                     Single-PR-sized atomic deliverables
```

## CI lanes that gate this µservice

- `governance-microservice-doc-set` — artifact-count floor.
- `governance-doc-rigor` — per-doc-class density signals.
- `governance-doc-graph-6hops` — six-hops graph traversability.
- `governance-cross-consistency` — field naming + audit-class consistency.
- `governance-abuse-defence` — anti-bot + anti-spoof + anti-scrape coverage.
- `governance-tls-floor` — TLS 1.3 enforcement.
- `governance-pqc-readiness` — PQC hybrid offered.
- `governance-ech-readiness` — ECH config advertised.

## ADR roster

ADR-0157 (api-gateway tier), ADR-0182 (north-south vs east-west separation), ADR-0183 (policy-engine separation), ADR-0242 (oyatie-is-a-tenant), ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping), ADR-0245 (substrate vs product), ADR-0246+amendment (policy library-first), ADR-0248 (cellular architecture), ADR-0253 (HTTP/3 + TLS strict + ECH + PQC), ADR-0254 (deployment shape), ADR-0263 (observability emission), ADR-0273 (mail anti-spoof not applicable; cross-ref only), ADR-0284 (platform-owner name indirection), ADR-0294 (Cedar fragment soak), ADR-0295 (SPIFFE workload identity + kill-switch), ADR-0296 (library-first credential sidecar), ADR-0297 (abuse-defence baseline — in flight).

## References

- `docs/standards/documentation-rigor.md` — the buildability bar.
- `microservices/observability/` — shape exemplar.
- `docs/AGENTS.md` — operating contract.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
