---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P22-m02-exit-gate
status: Proposed
acceptance_lanes: []
entry_gate: |
  All Wave-A (P01–P11), Wave-B (P12–P16), Wave-C (P17–P19), Wave-D (P20) and
  Wave-E/P21 complete; plane-verification-M02.md and wave-integration-report-M02.md
  produced and accepted; all 14 CI lanes exit 0 in --report-only mode; oya gate
  validate planes --all exit 0; grit done on all P21 symbols; ICM phase-handoff
  rows emitted for all preceding phases.
exit_gate: |
  All 14 CI fitness lanes flipped from --report-only to BLOCKER in
  .github/workflows/ci-fitness-lanes.yml; Application B2B shell (oya-application-shell-app)
  deploys without error to OCI ARM64 Always Free (Stage 0); sibling team can scaffold
  and ship any product vertical through grit claim/work/done with zero build-team
  intervention; cargo check/nextest/deny all exit 0 across full workspace; oya gate
  validate planes --all exit 0; oya gate validate wave-integration --milestone M02 exit 0;
  grit done on all P22 symbols; ICM phase-complete row emitted.
depends_on:
  - milestone: M02
    phase: P21-architecture-planes-green
    reason: "Exit gate requires all 9 architecture planes verified at L4-L5 before flipping lanes to BLOCKER or claiming M02 complete."
owner_team: council-architecture
---

# P22-m02-exit-gate: M02 Substrate Exit — Flip Lanes to BLOCKER + Deploy App Shell + Validate Sibling Onboarding

## Purpose

P22 is the terminal phase of M02-substrate. It has three coordinated goals:

1. **Flip all 14 CI fitness lanes from `--report-only` to BLOCKER** — any violation in a post-P22 PR
   causes CI to fail hard. This enforces the hyperscaler-grade quality bar going forward.

2. **Verify Application B2B shell is deployable** — `oya-application-shell-app` starts, authenticates
   via the SSO hub, resolves products from TenantProductRegistry, and renders the shell without error
   on OCI ARM64 Stage 0.

3. **Validate sibling-team self-sufficiency** — a sibling team (medical, hr, payroll, etc.) can pick up
   a product vertical, follow the grit claim/work/done protocol, and ship a minimal feature end-to-end
   without any build-team or Foundry-team help. This is the acid test that M02 substrate is actually
   usable by consumers.

M02 is declared complete only when all three goals are achieved and ICM phase-complete is emitted.

---

## Scope

### In-scope

| Deliverable | Description |
|---|---|
| `.github/workflows/ci-fitness-lanes.yml` | Flip all 14 lanes: remove `--report-only` flags; add `exit-on-violation: true` |
| `docs/architecture/m02-exit-checklist.md` | Per-gate checklist artifact: each gate checked + evidence reference |
| `docs/runbooks/sibling-team-onboarding.md` | Step-by-step runbook: grit claim → scaffold product vertical → ship → grit done |
| `oya gate validate planes --all` | Gate command confirming all 9 planes pass with BLOCKER lanes active |
| `grit done` on all P22 symbols | Release grit session for all M02 symbols |

### Out-of-scope

- Implementing any new product vertical — only validating that a sibling team CAN do so
- M03 Citus deployment verification — deferred to M03
- mTLS Istio between services — deferred to M03 per ADR-0117 §1
- Active-active cell failover testing — deferred to M04

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-flip-lanes-to-blocker.md`](IP-001-flip-lanes-to-blocker.md) | Flip all 14 CI lanes from --report-only to BLOCKER in GitHub Actions workflow | pending | `council-foundry` |
| [`IP-002-app-shell-deployment-gate.md`](IP-002-app-shell-deployment-gate.md) | Verify oya-application-shell-app deploys to OCI ARM64 Stage 0; OCI ARM64 smoke test | pending | `council-architecture` |
| [`IP-003-sibling-onboarding-validation.md`](IP-003-sibling-onboarding-validation.md) | Produce sibling-team-onboarding.md runbook; validate end-to-end with a tracer product feature | pending | `council-architecture` |
| [`IP-004-m02-exit-checklist.md`](IP-004-m02-exit-checklist.md) | Produce m02-exit-checklist.md evidence artifact; emit final ICM phase-complete | pending | `council-architecture` |

---

## Acceptance Gates

```bash
# 1. All lanes pass as BLOCKER (no --report-only); doc-coverage + canonical-base-neutrality
#    + cross-pack-refusal flip from --report-only to BLOCKER here per ADR-0063 / ADR-0064.
cargo run -p oya-check-statelessness -- --workspace             # exit 0; 0 BLOCKER violations
cargo run -p oya-check-shardability -- --migrations-dir migrations/  # exit 0; 0 BLOCKER violations
cargo run -p oya-check-documentation -- --workspace --blocker    # exit 0 (LEAN-A5; ADR-0063; --blocker post-P22)
cargo run -p oya-check-architecture -- dependency-direction --workspace        # exit 0
cargo run -p oya-check-architecture -- layer-correctness --workspace           # exit 0
cargo run -p oya-check-architecture -- lib-name-parity --workspace             # exit 0
cargo run -p oya-check-architecture -- port-location --workspace               # exit 0
cargo run -p oya-check-architecture -- cross-product-refusal --workspace       # exit 0
cargo run -p oya-check-architecture -- composition-root-only --workspace       # exit 0
cargo run -p oya-check-architecture -- sdk-kernel-only --workspace             # exit 0
cargo run -p oya-check-architecture -- canonical-base-neutrality --workspace   # exit 0 (ADR-0064 §8)
cargo run -p oya-check-architecture -- cross-pack-refusal --workspace          # exit 0 (ADR-0064 §7)
cargo run -p oya-check-perf-budget -- --workspace                              # exit 0
cargo run -p oya-check-benchmark -- --workspace                                # exit 0

# 2. Full workspace compile + test + deny
cargo check --workspace --all-features               # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0

# 3. Plane + wave gates
oya gate validate planes --all                        # exit 0; all 9 planes L4+
oya gate validate wave-integration --milestone M02    # exit 0; no cycles

# 4. App shell deployment (Stage 0)
# Runs on OCI ARM64 VM.Standard.A1.Flex; single-node OKE namespace
kubectl apply -f deployments/stage0/oya-application-shell.yaml
kubectl rollout status deployment/oya-application-shell -n oyatie-stage0  # ready
curl -f https://stage0.oyatie.internal/health                              # HTTP 200

# 5. Sibling team grit smoke test
grit claim --agent sibling-test --intent "tracer feature smoke test" --ttl 1800 \
  tracer/src/main.rs::tracer_feature
# ... sibling implements minimal feature ...
grit done --agent sibling-test                        # exit 0; no merge conflict
```

---

## Clean Architecture Compliance

This phase modifies only:
- GitHub Actions workflow YAML (`.github/workflows/ci-fitness-lanes.yml`) — flip flags
- Documentation files (`docs/architecture/`, `docs/runbooks/`) — evidence + runbook

No new Rust crates. No new schemas. The phase is purely operational + evidence.

### CI lanes enforced as BLOCKER after this phase

All 14 lanes (same list as P20/P21 acceptance gates), now without `--report-only`.

### New BCs registered in this phase

None.

---

## Grit Claim Symbols

```
.github/workflows/ci-fitness-lanes.yml::fitness-lanes-blocker
docs/architecture/m02-exit-checklist.md::M02ExitChecklist
docs/runbooks/sibling-team-onboarding.md::SiblingOnboardingRunbook
```

TTL: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P22-m02-exit-gate started; flipping 14 lanes to BLOCKER; verifying app shell deploy; sibling team onboarding validation; M02 final gate" \
  -i high \
  -k "M02,P22,phase-start,exit-gate"

icm store \
  -t context-oyatie \
  -c "Phase P22-m02-exit-gate complete; M02-substrate milestone complete; all 14 lanes BLOCKER; app shell deployed Stage 0; sibling onboarding validated; next: M03" \
  -i high \
  -k "M02,P22,phase-complete,milestone-complete,M03"
```

---

## References

- Phase specs: P20-ci-lanes-operational/phase-spec.md (lanes list), P21-architecture-planes-green/phase-spec.md (plane gates)
- Bominal ADRs: ADR-0223 (Proof Ladder), ADR-0224..ADR-0231 (9 planes), ADR-0232 (wave integration)
- oyatie ADRs: ADR-0056 v4.1 (BNF + check-namespace), ADR-0062 (quality/perf bar), ADR-0117 (OCI stages)
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`, `feedback_autonomous_implementation_artifacts.md`, `feedback_grit_claim_work_done.md`
