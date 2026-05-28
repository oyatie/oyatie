---
doc_class: Changelog
template_id: TPL-CHANGELOG
microservice: intelligence
status: Accepted
date: 2026-05-20
owner_team: axis-intelligence
doc_status: published
---

# Changelog — intelligence µservice

All material design and contract changes for the `intelligence` µservice are recorded here per
ADR-0131 (per-microservice flat layout) and ADR-0255 §"Versioning and changelog discipline".

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and SemVer 2.0.

## [Unreleased]

### Added
- Full doc-set buildout to match ADR-0255 two-layer scope.
- 8 bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`,
  `brand-ux-surface`, `credential-resolver`, `audit-tap`.
- 25 implementation plans (`IP-001..IP-025`).
- 18+ catalog records covering BC × layer combinations.
- 8 Cedar policy fragments — dispatch authorization, provider routing, refusal baseline,
  provider-credential BYOK gating (ADR-0255 §D-4), EU AI Act Annex III high-risk refusal, abuse defence, auditor scope, CI scope.
- 9 runbooks — provider outages (Anthropic / OpenAI / Google), rate-limit saturation,
  sidecar credential-handle expired, prompt-injection detected, refusal false-positive cascade,
  audit-row forgery detected, provider-credential BYOK rotation cascade (ADR-0255 §D-4).
- Contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, provider-adapter trait spec.
- Capabilities: 6 capability records (dispatch, routing, guardrails, eval, attribution, audit-tap).
- Dashboards: overview, provider-latency heatmap, refusal-rate by pack, FinOps cost attribution,
  prompt-injection detection, provider-credential BYOK vs platform-default mix (ADR-0255 §D-4).
- 5 OpenSLO manifests — dispatch availability + latency, first-token latency,
  streaming throughput, audit emission success.
- Phase specs: PHASE-01 (two-layer MVP), PHASE-02 (consumer brand surface).
- Threat model + DPIA (high-risk EU AI Act Art. 35 + KR PIPA Art. 33 assessment).
- IaC scaffolding (K8s manifests, Helm values, Terraform module, OpenBao policy, network policy).

### Changed
- `manifest.json` extended from 2 → 8 bounded contexts; substrate-tier declaration; substrate
  dependencies (cloud-secrets / policy-engine / observability / audit-chain / cell); EU AI Act risk
  classification fields; per-provider routing surface.

### Removed
- `embeddings` and `fine-tuning` scope per ADR-0255 §D — these now live in separate µservices.

### Deprecated
- The legacy `assist-draft` + `context-aware-retrieval` BC names are retained as adapter-shape
  shims under `model-routing` and `attribution` respectively for backward compatibility; new code
  should use the canonical BC names.

## [0.1.0] — 2026-05-18

### Added
- Initial scaffold: PRD + operational-boundaries + threat-model + capabilities (assist-draft,
  context-aware-retrieval) + SLOs (latency, refusal-correctness) + IP-001 + manifest.json.

## References

- ADR-0255 — Intelligence as two-layer AI Substrate.
- ADR-0255 amendment — Library-first network-opt-in clarification.
- ADR-0263 — Audit-tap.
- ADR-0296 — Sidecar credential-handle.
- ADR-0131 — Per-microservice flat layout.
