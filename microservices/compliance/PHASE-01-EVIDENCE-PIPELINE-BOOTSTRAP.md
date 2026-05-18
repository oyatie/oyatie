---
microservice: compliance
doc: PhaseSpec
phase: PHASE-01-EVIDENCE-PIPELINE-BOOTSTRAP
status: Drafting
authority_tier: 2
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0131, ADR-0145, ADR-0170, ADR-0181, ADR-0183, ADR-0209]
---

# PHASE-01 — Evidence Pipeline Bootstrap

## Phase intent

Bootstrap the compliance µservice from greenfield to SOC 2 Type II + GDPR DSAR continuous-evidence readiness. Land the kernel + domain + use-case + REST API + Backstage auditor plugin in 4-6 weeks of agentic-pipeline execution.

Exit criterion: per-framework coverage gate at 100% required artifacts for at least one µservice in the fleet (identity, observability, or workflow-studio). Auditor portal renders.

## IP rollout (sequenced)

1. **IP-001** — evidence-collector-bootstrap. Lands kernel coverage matrix wiring + collector trait + in-memory test impl.
2. **IP-002** — soc2-control-mapping. Maps Trust Services Criteria → required artifact kinds → per-µservice emission cadence.
3. **IP-003** — gdpr-dsar-automation-pipeline. DSAR request lifecycle + Ontology cascade + 5-day SLA scheduler.
4. **IP-004** — hipaa-min-necessary-log-substrate. Minimum-necessary access log emitter + BAA inventory store.
5. **IP-005** — audit-chain-seal-coverage. Seal-hex validation + cosign keyless OIDC verify path.
6. **IP-006** — evidence-storage-seaweedfs. SeaweedFS filer binding + per-framework bucket.
7. **IP-007** — auditor-readonly-portal. Backstage plugin + Cedar per-engagement role binding.
8. **IP-008** — pii-scrubber. DSAR export redaction (k-anonymity + format-preserving encryption where appropriate).
9. **IP-009** — retention-tier-policy. Hot / warm / cold + cold-archive cosign re-seal.
10. **IP-010** — attestation-aggregator. Cross-µservice attestation rollup per framework.
11. **IP-011** — cross-microservice-evidence-fan-in. Outbox-pattern subscriber per ADR-0153.
12. **IP-012** — evidence-replay. Re-emit historical evidence into freshly-restored audit chain.
13. **IP-013** — audit-anomaly-detection. Seal-chain anomaly detector → Sev-1 paging.
14. **IP-014** — manual-evidence-upload-flow. Pen-test + BAA inventory + manual artifact upload UI.
15. **IP-015** — regulatory-pack-evidence-overlay. Per-pack uplift (e.g., KR financial regulation overlay; UAE/SA pack overlay).

## Architecture surface

```
Phase 1 layout (flat, per ADR-0131):

microservices/compliance/
  src/                         # Rust binary entry point
  iac/helm/evidence-collector/ # Kubernetes deployment
  capabilities/                # Cedar capability fragments
  catalog/                     # Backstage catalog records
  contracts/                   # OpenAPI + AsyncAPI
  dashboards/                  # Grafana JSON
  decisions/                   # µservice-local ADRs
  evidence/                    # SLO + acceptance evidence
  policy/                      # Cedar policy
  runbooks/                    # ops runbooks
  scorecards/                  # per-framework scorecards
  slos/                        # OpenSLO authoring
  specs/                       # OpenAPI / AsyncAPI
  tests/                       # integration tests
  clients/                     # auditor-portal Backstage plugin
```

## Acceptance gates

| Gate | Owner |
|---|---|
| Build + test green (cargo build / cargo test) | axis-compliance |
| OpenAPI parity (oya-check-openapi-rest-route-parity) | axis-compliance |
| Cedar fragment coverage (oya-check-cedar-fragment-coverage) | axis-security |
| Audit-chain seal coverage (oya-check-audit-chain-seal-coverage) | axis-security |
| Compliance evidence coverage (oya-check-compliance-evidence-coverage) | axis-compliance |
| Cross-tenant isolation integration test | axis-security + axis-compliance |
| Backstage auditor plugin renders against test fixture | axis-compliance |

## Out-of-scope (Phase 1)

- PCI-DSS payments substrate (deferred until `microservices/payments/`).
- Auto-discovery of external sub-processors for BAA inventory (manual upload only).
- Drata / Vanta data migration wizard.

## References

- ADR-0209 — compliance evidence automation.
- PRD.md — product requirements.
- threat-model.md — security threat model.
- compliance.md — regulatory framework mapping.
