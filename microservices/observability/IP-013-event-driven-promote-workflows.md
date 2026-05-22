---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-013-event-driven-promote-workflows
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [oya-governance-protection-context-match]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Event-driven promote workflows

## Intent

Rewrite `.github/workflows/promote-dev-to-staging.yml` and `.github/workflows/promote-staging-to-production.yml` to consume `repository_dispatch` event `eligibility-changed`. Retain crons as reconciliation heartbeat only. Decommission FUTURE-marked stubs (`oya-governance-canary-cohort-observability` + `oya-governance-full-rollout-observability` references).

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/workflows/promote-dev-to-staging.yml` | update — `on.repository_dispatch.types: [eligibility-changed]`; retain `schedule` as fallback; remove FUTURE notes |
| `.github/workflows/promote-staging-to-production.yml` | update — analogous |
| `.github/workflows/oya-vcs-promotion-readiness.yml` | already created by IP-012 |

## Code Shape

```yaml
# promote-dev-to-staging.yml (excerpt of new shape)
name: promote-dev-to-staging

on:
  repository_dispatch:
    types: [eligibility-changed]
  workflow_dispatch:
  schedule:
    - cron: '*/30 * * * *'  # reconciliation heartbeat

permissions:
  contents: write

jobs:
  promote-per-microservice:
    if: ${{ github.event.action == 'eligibility-changed' && github.event.client_payload.target_env == 'staging' && github.event.client_payload.verdict == 'eligible' }}
    runs-on: ubuntu-latest
    steps:
      - name: Verify promotion readiness lane green
        run: cargo run -p oya-dev-cli -- gate validate vcs-promotion-readiness --sha ${{ github.event.client_payload.source_sha }} --env staging
      - name: Fast-forward release pointer
        run: |
          MS="${{ github.event.client_payload.microservice }}"
          SHA="${{ github.event.client_payload.source_sha }}"
          gh api -X PATCH repos/${{ github.repository }}/git/refs/heads/release/${MS}/staging -F sha="$SHA" -F force=false
```

## Acceptance Gates

```bash
gh workflow run promote-dev-to-staging.yml --field client_payload='...' --ref dev
# Verify ref advanced + audit-chain event emitted
```

## Test Plan

| Test | Verifies |
|---|---|
| Workflow integration: synthetic eligibility-changed event → ref advances |
| Cron heartbeat reconciles missed events |
| Stale FUTURE-marked stub references removed (grep returns empty) |

## Halt Conditions

- FUTURE stub references not removed — block; this is the decommission deliverable

## Next IP

[`IP-014-automated-rollback-primitive.md`](IP-014-automated-rollback-primitive.md)

## References

- ADR-0139 §"Layer-B item 15 — Event-driven promote workflows"
- `/specs/agentic-slo-gated-promotion.json` §"promote_workflow_contract"
