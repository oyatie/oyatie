---
doc_class: Checklist
checklist_id: CHK-REL
status: pending approval
purpose: |
  Milestone-level release gate. Walked at every wave-gate boundary per `docs/ROADMAP.md §2`. Trace to `.omc/plans/MASTERPLAN.md §13 Definition of done`.
lift_target: oyatie/templates/checklists/release-readiness.md
enforcing_fitness_lane: governance-release-readiness
owner_team: ops-sre-reliability + council-architecture
related:
  - docs/RELEASE-MANAGEMENT.md
  - docs/ROADMAP.md
  - .omc/plans/MASTERPLAN.md
  - /templates/checklists/per-phase-completion-checklist.md
---

# Release Readiness Checklist (wave-gate)

> Walk **all** rows before flipping a wave-gate to `passed`. Each row names a lane / command / advisory.

## Scope + governance

- [ ] **RR1** Milestone INDEX `status: merged`. *Lane:* `governance-plan-hierarchy`.
- [ ] **RR2** Every phase under the milestone walked through `per-phase-completion-checklist.md`. *(transitive)*
- [ ] **RR3** Wave-gate row in `docs/ROADMAP.md §2` has every acceptance criterion checked. *Lane:* `governance-release-readiness`.
- [ ] **RR4** `docs/PRD.md §4 success metrics` for the wave-gate met (if applicable). *(advisory; founder + council review)*

## Engineering bar (hyperscaler-grade)

- [ ] **RR5** Trunk-based-development discipline preserved: no long-lived feature branches >7 days on the milestone tree. *(advisory)*
- [ ] **RR6** Feature-flag rail green for wave: every milestone-shipped surface has a kill-switch flag. *Lane:* `governance-feature-flag-discipline`.
- [ ] **RR7** Canary rollout drilled in staging for at least one milestone-critical surface. *Command:* `oya ops canary dry-run <surface>`.
- [ ] **RR8** Automated SLO-burn-rate rollback drilled. *Command:* `oya ops rollback dry-run <surface>`.
- [ ] **RR9** Distroless image discipline: every binary in the milestone passes image-size budget. *Lane:* `governance-image-discipline`.
- [ ] **RR10** LTS-dependency discipline: every direct dep tracks current LTS or has ADR-tracked exception. *Lane:* `governance-lts-dependency`.

## Supply-chain (SLSA L2+)

- [ ] **RR11** Every shipped binary has Cosign keyless OIDC signature. *Lane:* `governance-supply-chain`.
- [ ] **RR12** Every shipped binary has Syft / CycloneDX SBOM artifact. *Lane:* `governance-supply-chain`.
- [ ] **RR13** SLSA L2+ provenance attestation present + Rekor log index recorded. *Lane:* `governance-supply-chain`.
- [ ] **RR14** Dependency/advisory posture green via `cloud-ci-supply-chain-audit` (owned RustSec advisory scan over vendored mirror; `cargo-vet` retired until maintained inputs). *Lane:* `cloud-ci-supply-chain-audit`. *Evidence:* `presubmit` supply-chain packet or `buck2 test //ci/facade/supply-chain-audit:ci-supply-chain-audit-gate`.

## SRE / observability

- [ ] **RR15** SLOs defined for every milestone-shipped surface (per `docs/SLO-CATALOG.md`). *Lane:* `cloud-ci-slo-coverage`.
- [ ] **RR16** 4 golden signals (latency, traffic, errors, saturation) dashboards live. *(advisory; observability team)*
- [ ] **RR17** Runbooks live for every alert resolving to a runbook URL. *Lane:* `governance-runbook-index-resolves`.
- [ ] **RR18** On-call rotation (primary + secondary) staffed for milestone owner-axis. *(advisory)*
- [ ] **RR19** Burn-rate alerts (1x / 3x / 14x) configured. *(advisory)*

## Security + privacy + compliance

- [ ] **RR20** Threat model authored per `docs/templates/threat-model-template.md` for new surfaces. *(advisory)*
- [ ] **RR21** Privacy review: `data_class` allowlists honored end-to-end. *Lane:* `governance-data-class`.
- [ ] **RR22** Audit-chain emission verified end-to-end for every cross-axis flow. *Lane:* `governance-audit-emission`.
- [ ] **RR23** Regulator notification matrix dry-run executed (per region). *(advisory)*
- [ ] **RR24** DSR cascade dry-run completed. *Lane:* `governance-dsr`.

## Capability / intelligence automation

- [ ] **RR25** Every capability/intelligence automation surface shipping in this wave has: record + eval-set ≥ min-pass-rate + autonomy tier + audit topic + Cosign. *Lane:* `governance-capability-publish`.
- [ ] **RR26** Capability deprecation announcements (if any) follow `12 months announce + 6 months EOL`. *Lane:* `governance-capability-sunset`.

## Communications

- [ ] **RR27** Release notes drafted (customer-facing), or release-governance / no-user-facing-change no-op recorded. Release Please is required only when a live repo config/workflow exists.
- [ ] **RR28** Trust-portal page prepared (for surfaces with regulatory disclosure).
- [ ] **RR29** Internal Slack `#oyatie-masterplan-status` notified.
- [ ] **RR30** Founder + Council-Architecture sign-off recorded.

## Post-gate

- [ ] **RR31** `EVT-WAVE-GATE-PASSED` emitted with milestone ID + gate name + sign-off list. *Lane:* `governance-audit-emission`.
- [ ] **RR32** `docs/CHANGELOG.md` "wave-gate-passed" row appended.
- [ ] **RR33** Next milestone phases unblocked: their `gates_on:` rows mark this milestone `merged`.
- [ ] **RR34** Release-scoped rollout, rollback, observability, user-story, and
  release-note evidence is attached to the release record when applicable;
  ADR-0716 requires no generic packet for every merged PR.

If any row is unchecked, the wave-gate is not passed. Loop back; do not declare release readiness.
