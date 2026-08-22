---
doc_class: ImpossibleToFailEnvironmentSpec
parent: INDEX.md
status: Accepted
purpose: |
  Top-level architecture spec that binds the failure-mode catalogue,
  quality bar, gap analysis, defense-in-depth layers, and tooling
  roster into a single coherent "impossible to fail" environment for
  oyatie under autonomous agent operation.
owner: council-architecture
date: 2026-05-12
adr_citations:
  - ADR-0053
  - ADR-0055
doc_status: published
---

# Impossible-to-Fail Environment — Master Spec

> The environment is **asymptotically** impossible to fail. Absolute
> zero defect-escape is mathematically unreachable when the system
> includes humans, LLMs, and a non-stationary world. The discipline is
> to design every layer so that each mode requires ≥2 independent
> defenses to fail and so that every escape converts into a mechanical
> prevention before the same shape recurs.
>
> This spec is the normative implementation target of ADR-0055
> (impossible-to-fail environment contract). ADR-0053 governs fitness-lane
> lifecycle for all lanes enumerated here.

## 1. Invariants

Properties that hold regardless of agent / LLM behavior. Each invariant
is enforced at compile time, pre-commit, CI, admission, or runtime —
not at code-review or social agreement.

| ID | Invariant | Enforced at | Mechanism |
|---|---|---|---|
| **I-01** | No agent can land code that panics on a `Result::Err` path in a production crate. | Compile (L0) + CI (L2) | `clippy::unwrap_used`/`expect_used`/`panic`/`todo` = `deny` in workspace `[lints]`. |
| **I-02** | No agent can land code that crosses a data-class boundary. | Pre-commit (L1) + CI (L2) | `pre-commit-data-class.sh` + `governance-data-class` per ADR-0008. |
| **I-03** | No agent can introduce a dependency that was not pre-audited. | Compile (L0) + CI (L2) | `cargo-vet` allowlist + `cargo-deny [bans]`. |
| **I-04** | No agent can ship a binary without Cosign signature + Syft SBOM + SLSA L2+ provenance + Rekor entry. | CI (L2) + admission (L4) | `governance-supply-chain` + Kyverno admission policy. |
| **I-05** | No agent can bypass authn or authz on a regulated capability. | Runtime (L6) | Cedar policy registry — capability not invocable without policy match. |
| **I-06** | No agent can emit PII to logs. | Compile (L0) + pre-commit (L1) | `data_class:` struct annotation + `tracing` field-filter macro. |
| **I-07** | No agent can land a migration without rollback evidence. | CI (L2) | `governance-schema-migration` + per-tenant + per-cell rollback lanes. |
| **I-08** | No agent can hide instructions via Unicode (BiDi controls). | Pre-commit (L1) + CI (L2) | `governance-unicode-discipline`. |
| **I-09** | No agent can orphan a `tokio::spawn`. | Compile (L0) + runtime (L6) | `clippy::disallowed_methods` deny bare `tokio::spawn`; `intelligence-task-supervisor` wrapper. |
| **I-10** | Every Sev-1 / Sev-2 produces a new mechanical prevention before the next merge into `main`. | Governance (L8) | `governance-mistakes-ledger-cite`. |

## 2. Mechanical preventions

Every row in `docs/MISTAKES-LEDGER.md` maps to a lane (existing or new)
that prevents recurrence. The mapping is one-to-many — one MFL row may
shed multiple lanes — but it must be at least one-to-one.

| MFL row | Lane(s) | Status |
|---|---|---|
| MFL-0001 ADR citation drift | `governance-adr-citation` | shipped |
| MFL-0002 brand alias residue | `governance-brand-residue` | shipped |
| MFL-0003 retired-vocab leak | `governance-glossary` | target |
| MFL-0004 Team terminology drift | `governance-glossary` | target |
| MFL-0005 cross-axis contract drift | `governance-blast-radius` | target |
| MFL-0006 external dep without ledger | `governance-build-vs-buy` + `governance-dep-allowlist` (new) | target |
| MFL-0007 AGPL/GPL leak | `governance-license` | target |
| MFL-0008 data-class annotation gap | `governance-data-class` | target |
| MFL-0009 cluster successor authoring | `governance-adr-citation` ext. | shipped |
| MFL-0010 runbook index drift | `governance-runbook-index-resolves` | shipped |
| MFL-0011 brand-rebrand sed | `governance-brand-residue` | shipped |
| MFL-0012 legacy tree reintroduction | `governance-flat-crates` | shipped |
| MFL-0013 OpenAPI 3.2 contract drift | contract-parity tests | shipped |

The 22 new lanes proposed in this work pre-emptively cover the next
~22 MFL rows that the failure-mode catalogue predicts.

## 3. Composability

How the 9 layers interact:

- **Same mode caught at ≥2 layers.** Example: AIS-010 (unwrap) is
  blocked at L0 (compile), L2 (CI), L6 (runtime panic-rate alarm), and
  L7 (auto-rollback).
- **Orthogonal mechanisms.** L0 is the type system. L1 is filesystem
  text. L2 is process-level CI. L3 is workflow tooling. L4 is OS
  capabilities. L5 is traffic routing. L6 is telemetry. L7 is
  controller automation. L8 is human/agent organizational loop. Any
  one mechanism failing does not bypass the rest.
- **Redundancy budget.** Linus discipline rejects ceremony but accepts
  redundancy when (a) the failure mode has high blast radius and (b)
  the redundant defense lives at an independent failure boundary.

## 4. Resilience to agent failure

| Failure | Behavior |
|---|---|
| Agent claim expires | Same TTL mechanism. Other agents see expired claim and re-claim with deduplication. |
| Partial commits | `pre-commit` + CI prevent merging an incoherent diff. Branch protection on `main` blocks bypass. |
| Agent ignores its prompt | Reviewer-agent (Layer 3) disagrees, escalation fires. |
| Agent uses banned primitive | `governance-banned-primitives` + Directive 12 audit row required. |

## 5. Resilience to LLM regression

| Regression | Behavior |
|---|---|
| Model hallucinates more package names | `cargo-vet` + `governance-dep-allowlist` blocks at CI; allowlist is the ground truth. |
| Model regresses on Rust idioms | Workspace `[lints]` + `cargo clippy -D warnings` is invariant to model version. |
| Model regresses on security patterns | Semgrep ruleset + gitleaks/trufflehog are deterministic ground truth. |
| Model emits prompt-injection-laden text | Unicode-discipline lane + `data_class:` annotation. |
| New model has different style preferences | `rustfmt` + `cargo fmt --check` make style invariant. |
| Model "drifts" on convention | Replay-as-eval per ADR-0024 — every quarter, the corpus of MFL rows is replayed and the prevention coverage measured; if a regression appears, the lane is hardened. |

## 6. Measurement formula

The environment **is** impossible-to-fail when:

```
ImpossibleToFailScore(t) = 1 -
  (DefectEscape(t) / TotalChanges(t))
  - α · (MeanTimeToRollback(t) / SLO_TTR)
  - β · (BurnRateExceedances(t) / SLO_BurnBudget)
  - γ · (UncoveredFailureModeClasses(t) / 16)
```

Where:
- `DefectEscape(t)` = number of Sev-1/Sev-2 incidents in time window `t`
  that lacked a mechanical-prevention lane at incident time.
- `MeanTimeToRollback(t)` = production MTTR for the window.
- `BurnRateExceedances(t)` = count of SLO burn-rate alarms.
- `UncoveredFailureModeClasses(t)` = count of the 16 classes from
  the failure-mode catalogue that have **zero** lanes today.
- α = 0.10, β = 0.05, γ = 0.20 (tunable; chosen so γ-term dominates
  early, MTTR-term dominates mid-life, and escape-term dominates
  steady-state).

**Steady-state target**: `ImpossibleToFailScore(t) ≥ 0.99` averaged
over rolling 90 days. Initial target at M01 close: `≥ 0.85`. Initial
target at M04 close: `≥ 0.95`.

Reported quarterly to founder + council-architecture; emitted as
`EVT-IMPOSSIBLE-TO-FAIL-SCORE` audit row.

This formula is the binding contract of ADR-0055.

## 7. Failure budget (the residual)

The environment is **asymptotically** impossible-to-fail. The residual
failure budget is explicit and owned:

| Residual class | Why residual exists | Budget | Owner | Mitigation |
|---|---|---|---|---|
| Novel attack class not in catalogue | Catalogue is a snapshot; new attacks emerge. | ≤ 1 Sev-1 / quarter | ops-security | quarterly threat-model refresh; replay-as-eval per ADR-0024. |
| LLM regression beyond replay coverage | Foundation models change; eval harness is per-snapshot. | ≤ 1 Sev-2 / quarter | axis-foundry | Cross-model diff-vote (`omc ccg`) on high-blast-radius changes. |
| Regulator change without notice | Sovereignty / privacy law shifts. | ≤ 1 Sev-2 / year | regional-packs + ops-compliance | Monthly regional-pack review per MASTERPLAN §8 M01-P12. |
| Hardware failure cascades | Provider outage beyond canary scope. | budgeted via SLO error budget | ops-sre-reliability | Multi-region preparedness (W-Region-Fan-Out future milestone). |
| Human / agent collusion on a banned-primitive bypass | Two agents agree to bypass policy. | should be ≤ 1 / year, audited | council-architecture | Audit-chain ground truth; reviewer-agent veto. |

Each residual class has a named owner and a budget. If the budget is
exhausted, the owner triggers the postmortem layer (L8) and a new lane
is authored to convert the residual into a mechanical prevention.

## 8. Bootstrapping order

To go from current state (22 lanes, 11 of 42 modes mechanically
prevented) to the impossible-to-fail steady state:

1. **M01 (Foundation)** — ship BLOCKER tools: workspace `[lints]`
   no-unwrap, `cargo-vet`, Cosign + Syft + SLSA L2, gitleaks +
   trufflehog, Semgrep, unicode-discipline, cargo-mutants, cargo-fuzz,
   loom. Closes 18 of 22 open modes.
2. **M02 (Foundry-Preview)** — HIGH tools: Kani, MIRI, insta,
   cargo-semver-checks, cargo-machete + cargo-udeps, reviewer-agent,
   Falco, cargo-hakari, chaos-mesh, OPA/Cedar coverage. Closes 3 of 4
   remaining open modes.
3. **M03** — MED tools: kube-linter + kubescape, RepoMap,
   diff-vote, cargo-auditable, dependency-review-action,
   cargo-binstall, perf-budget hardening. Closes the final mode +
   hardens the residual.
4. **M04+** — LOW tools: commitlint, cocogitto, release-please, ruff /
   mypy if Python lands. Pure hygiene.

## 9. Operational contract

- **Lane authoring**: per new MFL row, a lane PR is opened within 24h.
- **Lane validation**: replay-as-eval against the original failure
  trace must pass before the lane merges.
- **Lane deprecation**: never; lanes are append-only. If a lane
  becomes redundant under a stronger lane, it stays as belt+suspenders.
- **Lane bypass**: requires founder + council-architecture explicit
  approval, audit-chain emit `EVT-LANE-BYPASS`, time-limited (≤ 24h),
  and a mandatory follow-up MFL row.

## 10. Why this is "final shape"

Per [MASTERPLAN Directive 3 (final shape)](../../plans/MASTERPLAN.md),
there is no prototype-shortcut path. The architecture above is the end state.
Iteration happens only within each layer (lane authoring per new
failure mode); the **layering itself** does not iterate. The number of
layers is fixed at 9 because the failure-mode classes saturate against
them — no observed failure shape from the AWS S3 2017
([summary](https://aws.amazon.com/message/41926/)), Cloudflare
2025 ([postmortem](https://blog.cloudflare.com/18-november-2025-outage/)),
or USENIX 2025 package-hallucination corpus needs a 10th layer.

## Sources

- [AWS S3 Feb 2017 outage summary](https://aws.amazon.com/message/41926/)
- [Gremlin — After the AWS S3 2017 retrospective](https://www.gremlin.com/blog/the-2017-amazon-s-3-outage)
- [Cloudflare 18-Nov-2025 outage postmortem](https://blog.cloudflare.com/18-november-2025-outage/)
- [USENIX 2025 — Package Hallucinations](https://www.usenix.org/system/files/conference/usenixsecurity25/sec25cycle1-prepub-742-spracklen.pdf)
- [Google SRE — Postmortem Culture](https://sre.google/sre-book/postmortem-culture/)
- [Google SRE workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/)
- [Google SRE PRR / launch checklist](https://sre.google/sre-book/launch-checklist/)
- [AWS Operational Readiness Review](https://docs.aws.amazon.com/wellarchitected/latest/operational-readiness-reviews/the-orr-tool.html)
- [Oracle OCI Well-Architected](https://blogs.oracle.com/cloud-infrastructure/oci-wellarchitected-framework)
- [Sigstore Cosign attestation](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [SLSA L3 build provenance — OneUptime 2026](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)
- [Mozilla cargo-vet](https://mozilla.github.io/cargo-vet/)
- [Pillar Security — Rules File Backdoor](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)
- [Semgrep AI-powered detection](https://semgrep.dev/blog/2025/ai-powered-detection-with-semgrep/)
- [Chaos Mesh](https://chaos-mesh.org/)
- [Falco docs](https://falco.org/docs/)
- [Argo Rollouts analysis](https://argo-rollouts.readthedocs.io/en/stable/features/analysis/)
- [hyperscaler-best-practices-2026-05-12.md](../../specs/hyperscaler-best-practices-2026-05-12.md)
- [MISTAKES-LEDGER.md](../../../docs/MISTAKES-LEDGER.md)
- [MASTERPLAN.md](../../plans/MASTERPLAN.md)
