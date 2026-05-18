# PHASE-01: consent-graph substrate (cross-tenant real-time visibility kernel)

- Phase ID: M01-PHASE-CG-01
- Authority: ADR-0214 + ADR-0131 + ADR-0130
- Status: Drafted → target Active on PR #143 merge to `dev`
- Owner: axis-consent-graph
- Date: 2026-05-18

---

## 1. Goal

Land the consent-graph µservice substrate so that any other µservice can route a cross-tenant data
flow through it on day-1 after merge. Substrate completeness = all 6 bounded contexts buildable, all
9 SLO manifests authored, all 4 Cedar policy stubs in place, all 8 runbooks drafted, manifest authored,
contracts authored, iac/helm renderable, evidence emitted.

This phase does **not** yet claim hyperscaler-maturity-GA. ADR-0123 maturity audit runs in PHASE-02.

## 2. Scope

In:
- 6 bounded contexts × 8–9 layers = 51 crates (scaffold only; deep impl per IPs).
- 15 implementation plans (IP-001..IP-015), each ≥150 substantive lines.
- 9 SLO manifests, 4 Cedar policies, 8 runbooks, 3 contracts (OpenAPI / AsyncAPI / Proto).
- 16 catalog entries (one per crate-with-public-surface).
- 3 capabilities (consent.grant T3, consent.project.subscribe T2, consent.enforce T0).
- 3 dashboards (grant-funnel, projection-freshness, revocation-fan-out).
- 4 framework scorecards.
- 5 pack overlays (kr, eu, us, us-healthcare, jp).
- 5 service-local ADRs (ADR-SVC-CG-001..005).
- 14 supporting docs (threat-model, compliance, dpia, multi-region, capacity-model, cost-budget,
  failure-modes, incident-response, backfill-replay, sdk-plan, competitor-parity, data-residency,
  partnership-onboarding, break-glass).
- 5 Ontology cross-tenant projection IPs under `microservices/ontology/IP-CT-001..005.md`.

Out:
- Deep crate impl beyond the kernel scaffolds emitted by `oya gen crate`.
- Workflow Studio cross-tenant trigger node (separate PR).
- Capability-tier T3 promotion (PHASE-02).
- Partner SDKs in TS/Python (Rust SDK only in PHASE-01; TS/Python tracked in IP-014 follow-up).

## 3. Sequencing of IPs

```
   IP-001 agreement-kernel ──┐
                             ├──► IP-002 agreement-domain ──► IP-003 agreement-usecase+adapter+...
                             │
   IP-004 enforcement-kernel ┘
                             ├──► IP-005 enforcement-domain (Cedar bindings) ──► IP-006 enforcement-usecase+adapter
                             │
   IP-007 revocation-kernel+worker ──► IP-008 revocation-fan-out (Pulsar)
   IP-009 projection-gateway-kernel ──► IP-010 mint+ACL ──► IP-011 scope-narrowing+aggregate
   IP-012 audit-bridge bilateral emitter ──► IP-013 cross-pointer integrity
   IP-014 partner-directory handshake+trust-anchor
   IP-015 self-observability SLO wiring
```

Parallelism: IP-001/IP-004/IP-007/IP-009/IP-012/IP-014 may proceed in parallel (independent kernels).

## 4. Acceptance criteria

| ID | Criterion | Evidence file |
|----|-----------|---------------|
| AC-1 | All 51 crates listed in manifest.json | manifest.json |
| AC-2 | `cargo build` clean on workspace path (parent-wiring done by separate PR) | evidence/consent-graph-batch-report.json |
| AC-3 | All 15 IPs present and ≥150 substantive (non-blank, non-bullet-noise) lines | IP-001..IP-015.md |
| AC-4 | All 9 SLO manifests valid OpenSLO YAML | slos/*.openslo.yaml |
| AC-5 | All 4 Cedar policies parse with `cedar-policy` crate | policy/*.cedar |
| AC-6 | All 8 runbooks have Severity, Trigger, Steps, Verification, Audit-evidence sections | runbooks/*.md |
| AC-7 | OpenAPI, AsyncAPI, Proto contracts validated against schemas | contracts/* |
| AC-8 | iac/helm chart renders with `helm template` and 0 errors | iac/helm/consent-graph/ |
| AC-9 | 5 pack overlays each have residency + retention + DSAR config | iac/kustomize/overlays/*/ |
| AC-10 | 5 service ADRs accepted | decisions/ADR-SVC-CG-*.md |
| AC-11 | Ontology cross-tenant projection extension lands 5 IPs | microservices/ontology/IP-CT-001..005.md |
| AC-12 | Threat-model covers all 7 attack vectors enumerated in ADR-0214 §7 | threat-model.md |
| AC-13 | Bilateral chain link integrity test passes | tests/ (per IP-013) |
| AC-14 | Sovereignty zero-violation test passes | tests/ (per IP-009) |
| AC-15 | Parent-wiring TODO emitted (workspace Cargo.toml + MICROSERVICES const) | evidence/parent-wiring-todo-consent-graph-batch.json |

## 5. Risks

- **R-1 Cedar policy author cognitive load**: partners writing Cedar will mis-scope.
  Mitigation: 5 templates + lint + dry-run evaluation API.
- **R-2 Pulsar cross-region ACL complexity**: cross-tenant cross-region Pulsar token issuance is novel.
  Mitigation: IP-CT-002 narrowly covers it; Pulsar OAuth2 + JWT issuer per tenant.
- **R-3 Revocation latency drift**: under load, propagation may exceed 1s.
  Mitigation: separate Pulsar partition with `messaging.priority=high`; SLO burn-rate page at 0.5s p99
  warning before 1s breach.
- **R-4 Bilateral chain divergence**: if grantor or grantee misses an audit event, chains diverge.
  Mitigation: IP-013 cross-pointer integrity check + nightly reconciliation worker + alert.
- **R-5 Sovereignty enforcement bypass**: bug in projection-gateway could route data to wrong region.
  Mitigation: IP-CT-005 zero-copy contract test + region-pinning unit test + production canary.
- **R-6 Agreement template misuse**: starter template used unchanged in production allows over-broad scope.
  Mitigation: templates ship in `Drafted` state requiring data-steward review before `Offered`; CI
  warns on unchanged template fingerprint.
- **R-7 Cedar compilation latency at peak**: 100K agreements compiling concurrently.
  Mitigation: compile at agreement-acceptance time, cache in `enforcement-adapter` keyed by
  `agreement_id`; pre-warm cache on µservice cold start.

## 6. Out-of-band coordination

- Parent-wiring (workspace Cargo.toml + MICROSERVICES const) emitted to TODO; not landed in this PR.
- `oya-dev-cli gate maturity-claim` run is a PHASE-02 concern.
- Ontology team must approve IP-CT-001..005 PR-comment-thread; coordination via `axis-ontology`.

## 7. Definition of done

This phase is Complete iff:
- All 15 ACs above pass.
- Evidence files emitted: `evidence/consent-graph-batch-report.json` and
  `evidence/parent-wiring-todo-consent-graph-batch.json`.
- ADR-0214 status flipped from Proposed → Accepted by reviewer-agent.
- Ontology team approval recorded on IP-CT-001..005.
- No Codex P2 unresolved on the PR (per Codex bulk-resolve anti-pattern, P2s are real defects to fix).

## 8. Sequencing into PHASE-02 (out-of-scope here)

- Hyperscaler maturity claim audit (ADR-0123).
- TS + Python SDKs.
- Workflow Studio cross-tenant trigger node.
- Marketplace discovery (separate µservice).
- ADR review of Cedar policy version pinning.
