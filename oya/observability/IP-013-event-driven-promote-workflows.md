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

Rewrite promotion pipeline jobs to consume signed `eligibility-changed` events. Retain crons as reconciliation heartbeat only. Decommission FUTURE-marked stubs (`oya-governance-canary-cohort-observability` + `oya-governance-full-rollout-observability` references).

## Concrete File Targets

| Path | Action |
|---|---|
| Jenkins promotion job: dev → staging | update — consume signed `eligibility-changed` events; retain `schedule` as fallback; remove FUTURE notes |
| Jenkins promotion job: staging → production | update — analogous |
| Jenkins/Forgejo required check `oya-governance-promotion-readiness` | already created by IP-012 |

## Code Shape

```groovy
stage('Promote eligible microservice') {
  when {
    expression {
      return env.PROMOTION_EVENT_TYPE == 'eligibility-changed' &&
             env.TARGET_ENV == 'staging' &&
             env.PROMOTION_VERDICT == 'eligible'
    }
  }
  steps {
    sh 'cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --sha "$SOURCE_SHA" --env "$TARGET_ENV"'
    sh '''
set -eu
cargo run -p oya-dev-cli -- promotion advance-release-pointer \
  --microservice "$MICROSERVICE" \
  --env "$TARGET_ENV" \
  --source-sha "$SOURCE_SHA" \
  --event "$PROMOTION_EVENT_ID" \
  --require-signed-source \
  --require-protected-release-ref \
  --append-audit-chain
'''
  }
}
```

## Acceptance Gates

```bash
jenkins build promote-dev-to-staging SOURCE_SHA=<sha> TARGET_ENV=staging
# Verify ref advanced + audit-chain event emitted
```

## Test Plan

| Test | Verifies |
|---|---|
| Promotion job integration | synthetic eligibility-changed event → ref advances |
| Cron heartbeat | reconciles missed events |
| Stub cleanup | stale FUTURE-marked stub references removed (grep returns empty) |

## Halt Conditions

- FUTURE stub references not removed — block; this is the decommission deliverable

## Next IP

[`IP-014-automated-rollback-primitive.md`](IP-014-automated-rollback-primitive.md)

## References

- ADR-0139 §"Layer-B item 15 — Event-driven promote workflows"
- `/specs/agentic-slo-gated-promotion.json` §"promote_workflow_contract"
