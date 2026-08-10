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
| Intern, cold-start | `README.md` → `manifest.json` → `contracts/` |
| Architect | `manifest.json` → `decisions/` → `policy/` |
| SRE | `runbooks/` → `observability/slos/` → `dashboards/` |
| Compliance | `dpia/` → `scorecards/` |
| Connector / adapters | `connector/` → `adapters/` |
| Drain / deferred cites | `REORG-DRAIN.md` |

## What's where

```
gateway/
├── README.md                       This file
├── manifest.json                   Capability-root + edge artifact roster
├── REORG-DRAIN.md                  Deferred microservices/ cites + next gaps
├── adapters/                       gateway-*-connector workspace crates (10)
├── capabilities/                   v2 capability records
├── catalog/                        Forward-declared oya-api-gateway-* rows
├── cedar/                          Cedar fragments (edge)
├── connector/                      Absorbed connector service tree
├── contracts/                      OpenAPI + AsyncAPI + proto3 (verified)
├── dashboards/                     Grafana JSON + cross-reference docs
├── decisions/                      Service-local ADRs
├── dpia/                           DPIA materials
├── iac/                            K8s + cert-manager + ECH/PQC config
├── IPs/                            Landed IP dossiers (ADR-0339, Wave-15-ZD)
├── observability/slos/             OpenSLO v1 manifests
├── policy/                         Cedar v4 fragments
├── runbooks/                       On-call procedures
└── scorecards/                     Scorecard overrides
```

## CI lanes that gate this µservice

- `oya-governance-microservice-doc-set` — artifact-count floor.
- `oya-governance-doc-rigor` — per-doc-class density signals.
- `oya-governance-doc-graph-6hops` — six-hops graph traversability.
- `oya-governance-cross-consistency` — field naming + audit-class consistency.
- `oya-governance-abuse-defence` — anti-bot + anti-spoof + anti-scrape coverage.
- `oya-governance-tls-floor` — TLS 1.3 enforcement.
- `oya-governance-pqc-readiness` — PQC hybrid offered.
- `oya-governance-ech-readiness` — ECH config advertised.

## ADR roster

ADR-0157 (api-gateway tier), ADR-0182 (north-south vs east-west separation), ADR-0183 (policy-engine separation), ADR-0242 (oyatie-is-a-tenant), ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping), ADR-0245 (substrate vs product), ADR-0246+amendment (policy library-first), ADR-0248 (cellular architecture), ADR-0253 (HTTP/3 + TLS strict + ECH + PQC), ADR-0254 (deployment shape), ADR-0263 (observability emission), ADR-0273 (mail anti-spoof not applicable; cross-ref only), ADR-0284 (platform-owner name indirection), ADR-0294 (Cedar fragment soak), ADR-0295 (SPIFFE workload identity + kill-switch), ADR-0296 (library-first credential sidecar), ADR-0297 (abuse-defence baseline — in flight).

## References

- `docs/standards/documentation-rigor.md` — the buildability bar.
- `microservices/observability/` — shape exemplar.
- `docs/AGENTS.md` — operating contract.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
