# governance µservice

CI-fitness substrate. Bundles all ~50 `oya-check-*` crates per ADR-0131 §"governance" + ADR-0132. Implements the 6-axis industry-best-practice + hyperscaler-grade conformance program per ADR-0133.

## Status

- M01-foundation; per-microservice flat layout (ADR-0131) — **scaffolded** 2026-05-17.
- ~50 historical `oya-check-*` crates: **migration in progress** under IP-001..IP-015 (Tier-A first 10 in IP-002..IP-011).

## Owner

`axis-foundry` (primary). Sibling reviewers: `council-architecture`, `ops-security`, `ops-sre-reliability`.

## Quick start

```bash
# Run the full ~50-lane fitness suite locally against current working tree
cargo run -p oya-dev-cli -- gate run --all

# Run a single lane
cargo run -p oya-dev-cli -- gate validate <lane-name> --microservice <ms>

# Regenerate aggregation indices
cargo run -p oya-dev-cli -- aggregation-index regenerate

# Replay evidence for a PR
cargo run -p oya-dev-cli -- governance evidence replay --pr <N>
```

## Folder map

```
PRD.md                                            # this µservice's PRD
PHASE-01-CI-FITNESS-CONSOLIDATION.md              # first phase
IP-001..IP-015                                    # implementation plans (M01 scope)
README.md                                         # this file
decisions/                                        # service-scoped ADRs
contracts/{openapi,asyncapi,proto}/               # API contracts
specs/                                            # µservice-scoped specs
catalog/                                          # per-crate catalog rows
runbooks/                                         # 6 operational runbooks
threat-model.md                                   # STRIDE + LINDDUN
dpia.md                                           # DPIA
cost-budget.md                                    # FinOps posture
failure-modes.md                                  # 12+ failure modes
capacity-model.md                                 # per-cell capacity
compliance.md                                     # meta-compliance
multi-region.md                                   # cross-pack story
incident-response.md                              # IR playbook
policy/*.cedar                                    # Cedar fragments + policy docs
dashboards/*.json                                 # Grafana dashboards
backfill-replay.md                                # replay against new policy
sdk-plan.md                                       # client SDK roadmap
competitor-parity-matrix.md                       # vs SonarQube + GHAS + Snyk + ...
capabilities/                                     # 3 Foundry capabilities
iac/{helm,kustomize,terraform}/                   # Layer-A IaC
src/crates/                                       # ~50 oya-check-* + 36 umbrella crates
tests/{integration,e2e,perf,load}/                # cross-crate tests
evidence/multispectrum/                           # per-ChangeSet evidence
audit/                                            # per-axis audit overlays
slos/                                             # OpenSLO manifests (self-observability)
```

## SLO

Availability target 99.95% monthly for the per-PR gate decision path. See `slos/lane-runtime-availability.openslo.yaml`.

## Industry baselines tracked

SLSA, NIST SSDF (SP 800-218), OWASP ASVS v4, Google SRE Workbook, AWS Well-Architected Framework, Azure Well-Architected Framework, CNCF cloud-native maturity, OpenSSF Best Practices, Stripe API design, conventional-commits 1.0.0, Diátaxis. Pins at `/specs/industry-best-practice-conformance.json`.

## References

- ADR-0131 (per-microservice flat layout) — folder authority.
- ADR-0132 (no-grouping forward-policy) — bundle decision.
- ADR-0133 (industry-best-practice conformance) — 6-axis program.
- `docs/standards/agentic-dev-team-optimization.md` — axis-5 + axis-6 reference.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
