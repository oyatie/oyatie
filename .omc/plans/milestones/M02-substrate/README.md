---
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M02-substrate
parent: ../../../docs/MASTERPLAN.md
status: Proposed
entry_gate: 'M01-foundation Complete; commit 55058f6+8f86d4d+d942b3d on main.

  cargo check --workspace --all-features exits 0.

  4 LEAN architecture lanes (lean-a1..a4) at BLOCKER severity.

  '
exit_gate: "- All 22 M02 phases (P01-P22) Complete with grit done on each\n- All 14+\
  \ CI fitness lanes flipped from --report-only to BLOCKER at P22:\n  lean-a1..a5\
  \ + statelessness + shardability + perf-budget + benchmark +\n  architecture sub-commands\
  \ canonical-base-neutrality + cross-pack-refusal\n- oya-check-doc-coverage --workspace\
  \ exits 0 (no violations)\n- Application B2B shell deployed to OCI ARM64 Stage 0\
  \ cell\n- Sibling-team smoke test: external team scaffolds + ships a new \xB5service\n\
  \  end-to-end using grit claim \u2192 work \u2192 grit done, no build-team help\n\
  - 9 architecture planes assessed L4+ per Bominal ADR-0223..0231 inheritance\n- ICM\
  \ milestone-complete row emitted\n"
owner_team: council-architecture
bominal_adrs_inherited:
- ADR-0009
- ADR-0011
- ADR-0018
- ADR-0028
- ADR-0107
- ADR-0111
- ADR-0116
- ADR-0117
- ADR-0120
- ADR-0132
- ADR-0140
- ADR-0190
- ADR-0223
- ADR-0224
- ADR-0232
oyatie_adrs_cited:
- ADR-0056
- ADR-0058
- ADR-0059
- ADR-0062
- ADR-0063
- ADR-0064
purpose: Auto-backfilled purpose for README.md
---
# M02-substrate — Foundry engine + Cloud-Tenancy + Ontology + Workflow + Application + 16 substrate µservices ready

## Intent

Substrate completion milestone: every always-on substrate µservice (Foundry, Ontology, Identity, Audit-chain, Eventing, Secrets, Observability, KMS, Search, Vector, Data-Boundary, Finance-library, Capability-registry, Records, Workflow, Tenancy, Policy) ships with full doc suite + working canonical-base + CI lanes operational. Application B2B shell deploys to OCI ARM64 Stage 0. After M02 exits, sibling teams can scaffold + ship any new µservice with zero build-team help.

## Phase index (Wave-organized)

See `.omc/plans/M01-M03-parallelization-manifest.md` for the full DAG. Summary:

| Wave | Phases | Theme |
|---|---|---|
| Wave-A (P01-P11) | foundry-engine-consolidation / ontology / identity / audit-chain / eventing / secrets / observability / kms / search / vector / finance-library | Substrate kernel + adapter scaffolds; ports in kernel; impls in adapter |
| Wave-B (P12-P16) | workflow-engine / tenancy / policy / data-boundary / records | Cross-µservice adapters + boundaries; LEAN-A2 must stay green |
| Wave-C (P17-P19) | capability-registry / cloud-tenancy / application | Application B2B shell + product-enablement console |
| Wave-D (P20) | ci-lanes-operational | 5 new fitness lanes + 9 architecture-check sub-commands |
| Wave-E (P21-P22) | architecture-planes-green / m02-exit-gate | Plane verification L4-L5; flip all 14+ lanes from --report-only to BLOCKER; sibling-team smoke test |

## Parallelization

22 phases, 14-node critical path (per the manifest). Wave-A can run 11-wide; Wave-D runs in parallel with Wave-B+C (independent crate paths); Wave-E is serial gate.

## Unblocks

All M03+ milestones. M03-first-tenant cannot begin product authoring (HR, Payroll, Accounting, Connect Pro) until M02 substrate is green.

## References

- `docs/MASTERPLAN.md` §4 M02
- `.omc/plans/M01-M03-parallelization-manifest.md` (dispatch DAG + grit symbol-lock pre-flight)
- ADR-0056, ADR-0058, ADR-0059, ADR-0062, ADR-0063, ADR-0064
- Bominal ADRs 0009/0011/0018/0028/0107/0111/0116/0117/0120/0132/0140/0190/0223/0224-0231/0232
