---
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M02-substrate
parent: ../../../docs/MASTERPLAN.md
status: Proposed
entry_gate: |
  M01-foundation Complete; commit 55058f6+8f86d4d+d942b3d on main.
  cargo check --workspace --all-features exits 0.
  4 LEAN architecture lanes (lean-a1..a4) at BLOCKER severity.
exit_gate: |
  - All 22 M02 phases (P01-P22) Complete with grit done on each
  - All 14+ CI fitness lanes flipped from --report-only to BLOCKER at P22:
    lean-a1..a5 + statelessness + shardability + perf-budget + benchmark +
    architecture sub-commands canonical-base-neutrality + cross-pack-refusal
  - oya-check-doc-coverage --workspace exits 0 (no violations)
  - Application B2B shell deployed to OCI ARM64 Stage 0 cell
  - Sibling-team smoke test: external team scaffolds + ships a new µservice
    end-to-end using grit claim → work → grit done, no build-team help
  - 9 architecture planes assessed L4+ per Bominal ADR-0223..0231 inheritance
  - ICM milestone-complete row emitted
owner_team: council-architecture
bominal_adrs_inherited:
  - ADR-0009  # cell architecture
  - ADR-0011  # isolation tests mandatory
  - ADR-0018  # tenancy posture
  - ADR-0028  # audit chain Merkle/Ed25519
  - ADR-0107  # capability registry / agent gateway
  - ADR-0111  # KMS + envelope encryption
  - ADR-0116  # outbox → Kafka KRaft
  - ADR-0117  # OCI A1 → OKE staged scaling
  - ADR-0120  # platform finance library
  - ADR-0132  # Cedar policy + pillars
  - ADR-0140  # regional-pack pattern
  - ADR-0190  # versioned regulatory corpus.lock
  - ADR-0223  # Proof Ladder L0..L7
  - ADR-0224  # 9 architecture planes (referenced individually 0224..0231)
  - ADR-0232  # Wave integration framework
oyatie_adrs_cited:
  - ADR-0056  # BNF v4.1
  - ADR-0058  # flat µservice catalog
  - ADR-0059  # Workflow + Ontology adapter layer
  - ADR-0062  # quality/performance/scalability bar
  - ADR-0063  # documentation suite coverage (LEAN-A5)
  - ADR-0064  # canonical base + localization seams/adapters/packs
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
