---
id: ADR-0133
status: Superseded
deciders: council-architecture, council-engineering, axis-foundry, axis-observability, ops-sre-reliability, ops-security
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0056, ADR-0105, ADR-0106, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0135]
related_specs: [/specs/industry-best-practice-conformance.json, /specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
session_context:
  authored: 2026-05-17
  parallel_session_caveat: "ADR numbers 0125-0129 claimed by parallel session work. This ADR takes 0133 as the next available after my session's 0130/0131/0132."
bominal_source: "override — Bominal does not maintain a continuous industry-best-practice conformance program; oyatie originates this as a hyperscaler-grade discipline."
purpose: Establish a continuous, 6-axis industry-best-practice + hyperscaler-grade conformance program optimised for oyatie's fully-agentic development team. Every artifact in the repo (pipeline, directory, naming, standards, practices, policies) is audited against named industry references; a new BLOCKER CI lane refuses regression; quarterly refresh re-validates against current industry state.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0133: Industry-best-practice + hyperscaler-grade conformance program

## Status

Accepted — 2026-05-17.

## Context

User directive 2026-05-17: *"make sure all our pipeline, directories, naming, standards, practices, policies follow industry best practices, and hyperscaler grade. The only difference is that our dev team is fully agentic. optimization is key."*

oyatie has already adopted several industry-leading baselines piecemeal (per ADR-0030 Argo Rollouts; ADR-0041 GitOps trunk-based; ADR-0117 cloud-native infrastructure; ADR-0123 hyperscaler-maturity-claim-gate; ADR-0114 canary observability rollback; ADR-0139 agentic SLO-gated promotion; ADR-0131 per-microservice flat layout; ADR-0132 no-grouping forward-policy). What is missing is a **continuous, multi-axis audit program** that catches drift the moment an artifact deviates from industry baseline and that explicitly optimises for oyatie's fully-agentic developer (rather than the human-developer assumption embedded in most industry artifacts).

The audit must distinguish:
- **Conformance**: artifact matches an industry-published baseline (SOC 2 + ISO 27001 + SLSA + Google SRE + AWS Well-Architected + Microsoft Azure Well-Architected Framework + CNCF cloud-native maturity + OWASP ASVS + Google styleguide + conventional-commits + OpenAPI + AsyncAPI + OpenSLO + OpenTelemetry + Stripe API design + Linear + Vercel deploy patterns + etc.).
- **Optimisation**: artifact is shaped for fully-agentic consumption (machine-readable, structured-action, parallel-safe ChangeSet claim semantics, fail-closed defaults, smallest-actionable, audit-chain-sealed, idempotent, no-blanket-sed). This is oyatie specific because most industry artifacts assume a human-on-keyboard dev team.

Both dimensions are mandatory. An artifact that conforms to SOC 2 + ISO 27001 but blocks parallel agent execution is **not hyperscaler-grade for oyatie**.

This ADR is the override layer over any per-axis ad-hoc audit. Per-axis findings + remediation IPs feed back into this program.

## Decision

Adopt the 6-axis continuous industry-best-practice + hyperscaler-grade conformance program. Each axis carries:
- **Industry baseline** (named primary sources)
- **Audit cadence** (quarterly minimum; on-change for new µservices)
- **Findings schema** (per `/specs/industry-best-practice-conformance.json`)
- **Enforcement lane** (`oya-governance-industry-best-practice-conformance`; BLOCKER on dev)
- **Agentic-optimization overlay** (per `docs/standards/agentic-dev-team-optimization.md`)
- **Remediation IP series** (filed as `IP-M01-AUDIT-<axis>-<NNN>.md` under `microservices/governance/`)

### The 6 Axes

#### Axis 1 — Pipeline conformance

Every CI lane (existing `oya-governance-*` + new `oya-governance-*` + `oya-vcs-*`) audited against the canonical industry pipeline-stage taxonomy:

```text
build → test → security-scan → SBOM → sign → deploy → verify → promote
```

Industry baselines:
- **SLSA Level 3** — `slsa.dev/spec/v1.0/levels` (build provenance, source provenance, isolation).
- **NIST SSDF (SP 800-218)** — secure software development framework.
- **Google Cloud Build canonical pipeline** — `cloud.google.com/build/docs/build-config-overview`.
- **AWS CodePipeline patterns** — `docs.aws.amazon.com/codepipeline/latest/userguide/`.
- **GitHub Actions Hardening Guide** — `docs.github.com/en/actions/security-guides/`.
- **CNCF cloud-native CI/CD maturity model** — `landscape.cncf.io`.
- **OpenSSF Best Practices Badge** — `openssf.org/badges/`.

Audit output: map every oyatie lane to its canonical industry stage; surface gaps (stages without a covering lane) and overlaps (multiple lanes covering one stage redundantly).

#### Axis 2 — Directory conformance

Walk the entire repo against ADR-0131 per-microservice flat layout + the canonical AWS/Google/Microsoft/Oracle/Stripe per-service folder shape. Flag every artifact not yet under `microservices/<ms>/` (per ADR-0131 `migration_ip_enumeration`).

Industry baselines:
- **AWS service-team folder template** (smithy-models + src + docs + runbooks + integ-tests).
- **Google `google3/<area>/<service>/{server,client,docs,BUILD}`**.
- **Microsoft Engineering Playbook** — `microsoft/code-with-engineering-playbook`.
- **Oracle OCI service-team template**.
- **Stripe monorepo per-service layout** (`<service>/{lib,test,api,docs}`).

Audit output: per-µservice migration progress against `ADR-0131 §"Migration DAG"`; new finding for any µservice not yet under `microservices/<ms>/`.

#### Axis 3 — Naming conformance

Audit BNF v4.1 (ADR-0056) + 13-layer enum (ADR-0105) + `application → usecase` rename (ADR-0106) + no-grouping policy (ADR-0132) against every crate, ADR, spec, registry entry.

Industry baselines:
- **Rust API guidelines** — `rust-lang.github.io/api-guidelines/`.
- **Domain-Driven Design** (Eric Evans 2003; Vaughn Vernon "Implementing DDD" 2013) — bounded-context naming.
- **conventional-commits 1.0.0** — `conventionalcommits.org`.
- **CNCF crate-naming conventions for cloud-native** — varies by sub-foundation.

Audit output: per-crate naming-conformance status; per-ADR slug-conformance; per-spec `_meta` block conformance.

#### Axis 4 — Standards conformance

Every `docs/standards/*.md` audited against its industry equivalent:

| Standard | Industry source |
|---|---|
| code-style | Google styleguide; Rust `rustfmt` defaults + ADR-0056 enforcement |
| commit-message | conventional-commits 1.0.0 + signed-commits per branch-protection |
| api-design | Microsoft REST API guidelines; Stripe API design conventions; Google AIP |
| schema-migration | Liquibase/Flyway patterns; Bytebase migration guide |
| error-handling | Rust `Result<T,E>` conventions; Google AIP-193 |
| logging-tracing | OpenTelemetry semconv (LTS pinned per docs/standards/observability-slo.md) |
| testing | Google "Testing on the Toilet"; Kent Beck TDD; property-based testing (proptest / Hypothesis) |
| security-review | OWASP ASVS v4; SLSA; NIST SSDF; CIS Benchmarks |
| privacy-review | ICO + CNIL DPIA methodology; GDPR + KR PIPA + HIPAA + DPDPA + LGPD per pack |
| code-review | Google Code Review Developer Guide; Microsoft Code Review Practices |
| release | trunk-based development (Paul Hammant); Google Borg deploy; Argo Rollouts |
| on-call | Google SRE Workbook ch. 11; PagerDuty Incident Response |
| incident-severity | Google SRE Workbook ch. 7; Atlassian Incident Management |
| doc-style | Diátaxis framework — `diataxis.fr` |
| brand-voice | per regional pack; KR + global |
| migration-playbook | Stripe migration patterns; Shopify shop-rebuild patterns |
| observability-slo | Google SRE Workbook chs. 4-5; OpenSLO v1.0 |

Audit output: per-standard delta against named industry reference; flag any `docs/standards/<topic>.md` lacking explicit industry citation.

#### Axis 5 — Practices conformance (agentic-dev-team optimised)

Audit oyatie's repo-wide development practices against the canonical agentic-dev-team principles in `docs/standards/agentic-dev-team-optimization.md`:

1. **ChangeSet semantic claim** (not line-hunk; per durable-goal-spec).
2. **Parallel-safe operations** (per `dispatching-parallel-agents` skill + ADR-0110 ChangeSet boundary).
3. **Idempotent operations** (every CLI subcommand re-runnable; every migration replayable).
4. **Audit-chain seals on every state transition** (Ed25519 + Merkle per Bominal ADR-0028).
5. **Fail-closed on every gate** (default-deny per ADR-0139 + ADR-0140 (retired per ADR-0145)).
6. **Smallest-actionable artifact format** (per durable user preference; no repeated memory dumps).
7. **No-blanket-sed** (per ADR-0131 migration-tooling; structured ast-grep / Cargo workspace operations).
8. **No-deeper-hole rule** (Oya façade over external framework when adding-cheap; per durable preference).

Each practice is verified against existing artifacts; new artifacts must satisfy at scaffold time.

#### Axis 6 — Policies conformance

Every Cedar fragment under `microservices/<ms>/policy/*.cedar` audited against:
- **Principle of least privilege** (default-deny + explicit permits).
- **Defence-in-depth** (multiple layers; redundant permit + redundant forbid).
- **Tenant isolation** (per `microservices/<ms>/policy/tenant-isolation.md`; X-Scope-OrgID enforcement).
- **Per-pack regulatory overlay** (per `docs/standards/observability-slo.md` §"Compliance frameworks").
- **OPA / Cedar / AWS IAM industry policy-as-code patterns**.

## Output Artifacts (per execution wave)

| Artifact | Path | Cadence |
|---|---|---|
| Audit findings (machine-readable) | `/specs/industry-best-practice-conformance.json` | This ADR's session (foundation) + quarterly refresh |
| ADR (this document) | `docs/decisions/ADR-0133-industry-best-practice-conformance-program.md` | one-time foundation + supersession via ADR-#### successor-IP |
| Cross-cutting standard | `docs/standards/agentic-dev-team-optimization.md` | foundation + annual refresh |
| BLOCKER CI lane | `microservices/governance/src/crates/oya-check-industry-best-practice-conformance/` (or co-located under existing governance crate) | new on dev |
| Per-axis remediation IPs | `microservices/governance/IP-M01-AUDIT-<axis>-<NNN>.md` (or per-µservice when scope is µservice-local) | as findings surface |
| Per-µservice audit overlay | `microservices/<ms>/audit/industry-best-practice-conformance.md` | per-µservice (Slice-D-equivalent) |
| Quarterly refresh report | `evidence/audits/industry-best-practice-conformance/<quarter>.json` | quarterly |

## Rejected Alternatives

- **Trust the existing CI lanes are enough.** Rejected: existing `oya-governance-*` lanes were not designed against a comprehensive 6-axis baseline; this audit surfaces what they miss.
- **One-off audit (no continuous program).** Rejected: industry baselines evolve (SLSA v1 → v2; OpenTelemetry semconv monthly; LTS lines move). Continuous discipline required.
- **Per-axis audit owned by different teams without a unifying ADR.** Rejected: leads to inconsistent severity and remediation cadence.
- **Run audit only on new artifacts (not legacy).** Rejected: legacy gaps are the largest surface; continuous = legacy + new.

## Consequences

### Positive

- Every oyatie artifact has a named industry baseline; sales claims at the `oya-governance-hyperscaler-maturity-claims` lane (per ADR-0123 HG-OBS gate) become source-bounded by this program.
- Drift detected at PR time, not at audit time.
- Agentic-dev-team optimisation is codified, not implicit.
- External auditors (SOC 2 Type 2 / ISO 27001 / etc.) read the same audit-findings spec as the internal lane reads, simplifying audit preparation.

### Negative

- 6-axis audit produces a large finding set on first run (many legacy artifacts predate this ADR). Triage budget required.
- Industry baselines update; the program absorbs the maintenance cost of tracking them.
- BLOCKER lane on dev creates initial PR friction until legacy violations close.

### Operational

- **New CI lane**: `oya-governance-industry-best-practice-conformance` (BLOCKER on `dev`). Implemented under `microservices/governance/` (existing bundle per ADR-0132 governance umbrella). Lane reads `/specs/industry-best-practice-conformance.json` + asserts no new artifact lands without a matching audit row.
- **First-run amnesty**: existing legacy violations are recorded as `severity: legacy-grandfathered` + remediation owner + target-close-date 1y; lane refuses NEW violations matching the same pattern.
- **Per-axis remediation IPs**: filed under `microservices/governance/IP-M01-AUDIT-<axis>-<NNN>.md`; each IP closes one axis-finding and is bundleable per ADR-0110 ChangeSet contract.
- **Quarterly refresh**: `oya dev industry-best-practice-refresh` CLI subcommand fetches current industry baselines + diffs against pinned baselines + opens successor-IP ADR if baseline moved materially.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none |
| `cross-product-refusal` (LEAN-A2) | Not affected | none |
| `oya-governance-industry-best-practice-conformance` (NEW) | BLOCKER on dev | refuses unaudited new artifacts |
| `oya-governance-quarterly-refresh-cadence` (NEW, lower-priority) | CRON | fires quarterly; opens PRs for baseline updates |

## Verification

- `cargo run -p oya-dev-cli -- gate validate industry-best-practice-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0; this ADR registered.
- Quarterly: `cargo run -p oya-dev-cli -- industry-best-practice-refresh --quarter <YYYY-Q>` — exits 0 + opens refresh PR.

## References

- `/specs/industry-best-practice-conformance.json` (machine-readable audit findings schema)
- `docs/standards/agentic-dev-team-optimization.md` (axis-5 + axis-6 enforcement reference)
- ADR-0123 (hyperscaler-maturity-claim-gate)
- ADR-0139 (agentic SLO-gated promotion)
- ADR-0131 (per-microservice flat layout)
- ADR-0132 (no-grouping forward-policy)
- SLSA — `slsa.dev`
- NIST SSDF SP 800-218 — `csrc.nist.gov`
- Google SRE Workbook — `sre.google/workbook/`
- AWS Well-Architected Framework — `aws.amazon.com/architecture/well-architected/`
- Azure Well-Architected Framework — `learn.microsoft.com/azure/well-architected/`
- Diátaxis — `diataxis.fr`
- conventional-commits 1.0.0 — `conventionalcommits.org`
- OWASP ASVS — `owasp.org/www-project-application-security-verification-standard/`
- OpenSSF Best Practices — `openssf.org/badges/`
- Microsoft Engineering Playbook — `microsoft.github.io/code-with-engineering-playbook/`
- `feedback_quality_performance_scalability_bar.md`
- `feedback_no_silent_regression.md`
- `feedback_clean_architecture_requirements.md`
