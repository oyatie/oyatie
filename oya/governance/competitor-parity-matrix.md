---
doc_class: CompetitorParityMatrix
title: Competitor Parity Matrix
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + council-architecture
deciders: axis-foundry, council-architecture
related_adrs: [ADR-0123, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/governance/PRD.md
review_cadence: quarterly + per major competitor release
doc_status: published
---

# Competitor Parity Matrix: governance µservice

## Purpose

Per ADR-0123 (hyperscaler-maturity-claim-gate) + `feedback_quality_performance_scalability_bar.md`: every claim of parity or superiority must cite specific competitor + specific dimension. This matrix is the citation source for marketing + sales artifacts and is enforced by `oya-check-hyperscaler-maturity-claims` lane.

## Competitor set

| Competitor | Category | Primary surface |
|---|---|---|
| SonarQube + SonarCloud | Code-quality + security gate | rule-pack-driven static analysis; per-PR gate |
| GitHub Advanced Security (CodeQL + Dependabot + Secret Scanning) | Built-in GitHub PR gate | code scanning + dependency review + secret scanning |
| Snyk | Vulnerability + license + IaC + container | per-PR gate; SBOM; remediation suggestions |
| Polyspace (MathWorks) | Formal-methods static analysis | rule-pack; per-PR; primarily safety-critical (auto, aero) |
| CodeClimate | Maintainability + test-coverage | per-PR; dashboard; tech-debt scoring |
| Trivy (Aqua Security) | Container + IaC vulnerability + SBOM | CI integration; not primarily per-PR |
| Open Policy Agent + Conftest | Policy-as-code on configs | Rego; per-PR; IaC + K8s |
| Backstage TechDocs | Per-service doc system | doc-coverage; runbook-index; readme-coverage |
| Renovate (Mend) | Dependency-recency policy | auto-PRs; pin updates; security advisories |

## Parity Matrix (✓ = parity; △ = partial; ✗ = no parity; ★ = oyatie-unique advantage)

| Dimension | SonarQube | GHAS | Snyk | Polyspace | CodeClimate | Trivy | OPA | Backstage | Renovate | **oyatie governance** |
|---|---|---|---|---|---|---|---|---|---|---|
| Per-PR gate (block on BLOCKER) | ✓ | ✓ | ✓ | ✓ | ✓ | △ | △ | ✗ | △ | ✓ |
| Per-PR gate (BLOCKER + WARN + INFO severity) | ✓ | △ | ✓ | ✓ | △ | △ | △ | ✗ | ✗ | ✓ |
| Rule-pack versioning + ADR | △ | △ | △ | ✓ | △ | △ | △ | ✗ | ✗ | ★ |
| Per-µservice flat-layout enforcement | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | △ | ✗ | ★ |
| Industry-baseline citation in every Finding | ✗ | △ | △ | ✗ | ✗ | △ | ✗ | ✗ | ✗ | ★ |
| Quarterly baseline-pin refresh (auto-PR) | ✗ | ✗ | ✓ (CVE only) | ✗ | ✗ | ✓ (CVE only) | ✗ | ✗ | ✓ | ★ (6-axis, not just CVE) |
| 6-axis continuous conformance (pipeline + directory + naming + standards + practices + policies) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | △ | ✗ | ★ |
| Ed25519-signed Findings | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Audit-chain Merkle seal | ✗ | △ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |
| Replayable evidence (7y retention; object-lock) | △ | △ | △ | ✓ | △ | ✗ | ✗ | ✗ | ✗ | ★ |
| Aggregation-index source-of-truth (refuses hand-edits) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | △ | ✗ | ★ |
| Self-application (gate gates itself) | ✗ | △ | ✗ | ✗ | ✗ | ✗ | △ | ✗ | ✗ | ★ |
| Agentic-dev-team optimization (8 principles per ADR-0133) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |
| SLSA Build + Source L3 enforcement | ✗ | △ | ✓ | ✗ | ✗ | △ | △ | ✗ | △ | ✓ |
| NIST SSDF SP 800-218 mapping | ✗ | △ | △ | ✗ | ✗ | △ | △ | ✗ | △ | ✓ |
| OWASP ASVS rule pack | △ | ✓ | ✓ | ✗ | △ | △ | △ | ✗ | ✗ | ✓ |
| CIS Benchmarks (Kubernetes / Docker / Linux) | △ | ✗ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✓ |
| OpenSLO authoring + per-µservice SLO coverage gate | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | △ | ✗ | ✓ |
| Cedar policy fragments (ABAC) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | △ (OPA Rego) | ✗ | ✗ | ✓ |
| Multi-region + DR pair (per-pack residency) | ✓ (SonarCloud) | ✓ (GitHub.com) | ✓ (Snyk Cloud) | △ | △ | ✗ | ✗ | ✗ | ✗ | ✓ |
| HIPAA-eligible pack | ✗ | ✓ (Enterprise Cloud) | ✓ (Snyk Cloud-HC) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| KR-PIPA pack | ✗ | △ | △ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Tenant-self-service Finding query | ✓ (SaaS) | ✓ (Org settings) | ✓ | △ | ✓ | ✗ | ✗ | △ | ✗ | ✓ |
| External-auditor JIT scope (≤1h TTL) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |
| FinOps line items per lane | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |
| Open Source / self-hostable | ✓ (CE) | ✗ (vendor) | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ (oyatie) |
| Polyglot (Rust/TS/Python/Go SDKs) | △ | △ | ✓ | △ | △ | ✗ | △ | △ | ✗ | ✓ (per `sdk-plan.md`) |
| Lane bypass with cryptographic record + break-glass | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |

## Per-Competitor Strategic Posture

### SonarQube + SonarCloud

- **Their strength**: mature rule-pack ecosystem (50+ languages); large user base; well-known.
- **Our advantage**: 6-axis program (vs. code-quality only); audit-chain seal; per-µservice flat-layout enforcement; agentic-dev-team optimization.
- **Where we lag**: rule-pack breadth (we cover Rust-heavy stack first; language breadth follows).
- **Source**: `docs.sonarsource.com/sonarqube/`.

### GitHub Advanced Security

- **Their strength**: native GitHub integration; CodeQL is best-in-class for code scanning; built-in.
- **Our advantage**: per-µservice flat-layout enforcement; industry-baseline citation; aggregation-index source-of-truth; 6-axis program; per-pack residency.
- **Where we lag**: code-scanning depth (we use Trivy + custom; CodeQL is more mature for vuln-class detection).
- **Source**: `docs.github.com/en/code-security/`.

### Snyk

- **Their strength**: vulnerability database; supply-chain; SaaS UX.
- **Our advantage**: 6-axis program; per-µservice flat-layout; audit-chain seal; agentic-dev-team; quarterly baseline refresh (not just CVE).
- **Where we lag**: vulnerability database depth (we rely on `cargo audit` + `cargo deny` + Trivy; Snyk's database is more curated).
- **Source**: `docs.snyk.io`.

### Polyspace

- **Their strength**: formal methods for safety-critical (auto, aero, medical).
- **Our advantage**: 6-axis; tenant residency; cloud-native deploy.
- **Where we don't compete**: safety-critical formal methods (different category; we don't intend to compete in DO-178C / ISO 26262 / IEC 62304 spaces at M01).
- **Source**: `mathworks.com/products/polyspace.html`.

### CodeClimate

- **Their strength**: maintainability scoring; tech-debt dashboards.
- **Our advantage**: 6-axis + industry-baseline citation + audit-chain seal + replayability.
- **Where we lag**: tech-debt scoring (we don't have a maturity metric on tech-debt; consider adding ADR successor-IP).
- **Source**: `docs.codeclimate.com`.

### Trivy (Aqua Security)

- **Their strength**: container vuln scanning; IaC; SBOM.
- **Our advantage**: native PR gate (Trivy is CI-step not PR-gate); 6-axis program; per-µservice; agentic-dev-team.
- **Where we use it**: oyatie governance's `oya-check-supply-chain` lane uses Trivy under the hood for container scanning.
- **Source**: `aquasecurity.github.io/trivy/`.

### Open Policy Agent + Conftest

- **Their strength**: policy-as-code; Rego language; broad adoption.
- **Our advantage**: Cedar (more rigorous ABAC); per-µservice flat-layout; 6-axis; audit-chain.
- **Where we use it / coexist**: Cedar fragments serve same function for governance; OPA viable for tenant-customizable policy (future ADR if surfaced).
- **Source**: `openpolicyagent.org`.

### Backstage TechDocs

- **Their strength**: per-service doc system; Spotify-developed; large ecosystem.
- **Our advantage**: per-µservice flat-layout enforced at PR-time (Backstage is descriptive; we are prescriptive); 6-axis includes doc-coverage + runbook-index + readme-coverage gates.
- **Where we coexist**: oyatie can render Backstage-compatible TechDocs from per-µservice sources.
- **Source**: `backstage.io/docs/features/techdocs/`.

### Renovate

- **Their strength**: dependency-recency auto-PRs; CVE coverage.
- **Our advantage**: 6-axis (not just deps); industry-baseline pin refresh (not just deps).
- **Where we use it**: oyatie governance's `oya-check-vendor-recency` lane integrates Renovate-style PRs.
- **Source**: `docs.renovatebot.com`.

## Net-net summary

oyatie governance carries **13 ★-rated unique advantages** in the matrix above. Strategic positioning:

- **Vs. SonarQube/CodeClimate** (code-quality category): we win on 6-axis breadth + audit-chain seal + per-µservice flat-layout + agentic-dev-team.
- **Vs. Snyk/Trivy/GHAS** (vulnerability/supply-chain category): we cover their domain via integrated lanes + add 6-axis breadth on top.
- **Vs. OPA/Cedar** (policy-as-code category): we use Cedar; we're at parity on ABAC but win on 6-axis breadth + per-µservice.
- **Vs. Backstage/Renovate** (doc + deps category): we cover via lanes; we win on enforced source-of-truth + 6-axis.

## Refresh cadence

This matrix is reviewed quarterly per `runbooks/industry-baseline-refresh.md` Step 4 + on every major competitor release announcement.

## References

- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0133 (6-axis program).
- `microservices/governance/PRD.md` §"Competitive Benchmark".
- `microservices/observability/competitor-parity-matrix.md` (shape reference).
- Per-competitor URLs cited inline above.
