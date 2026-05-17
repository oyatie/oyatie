---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P07
title: Regional Pack Onboarding (KR + one of JP/US/EU)
status: complete
purpose: Onboard ≥ 2 regional packs per W-Cloud-Preview gate; KR-Seoul mandatory, second pack council-elected (JP-Tokyo / US-Northern-Virginia / EU-Frankfurt).
execution_variant: merge-into-existing-crates
decided_at: 2026-05-17
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 backports PackOnboardingPhase + PackInstallStatus + RegionalRolloutGate into existing oya-regional-pack-domain::pack_onboarding_phase instead of scaffolding new onboarding crates. Honors no-over-scaffolding rule."
---

# M03-P07 — Regional Pack Onboarding

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.3 and [`../../../../../docs/DESIGN.md`](../../../../../docs/DESIGN.md) §12 regional pack architecture.

## Acceptance
- KR pack: PIPA + KISA + MFDS + FSC + KCC + NIS + CSAP + K-ISMS-P + KCMVP residency contract on file.
- Second pack: JP (APPI + ISMAP) or US (HIPAA + HITECH + SOX + CCPA-CPRA + StateAGs + FedRAMP) or EU (GDPR + DORA + EU-AI-Act + GAIA-X) full residency contract on file.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | KR pack onboarding (PIPA / CSAP / K-ISMS-P / KCMVP seam) | complete | [`IP-001-kr-pack.md`](IP-001-kr-pack.md) |
| IP-002 | Second pack onboarding (JP/US/EU — council-elected) | complete | [`IP-002-second-pack.md`](IP-002-second-pack.md) |

## Estimated parallelism
2 agents; one per pack.

## Symbols-touched
`regional-packs/kr/`, `regional-packs/<elected>/`, `crates/oya-platform-regional-pack-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P07 complete: KR + <second-pack> onboarded with residency contracts" -i critical -k "M03,P07,regional-pack,kr,complete"
```
