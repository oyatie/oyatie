---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-011-per-component-release-pointers
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [oya-foundry-fitness-protection-context-match]
---

# IP-011: Per-component release pointers

## Intent

Introduce `release/<microservice>/<environment>` ref naming + pattern-based GitHub branch-protection per `/specs/agentic-slo-gated-promotion.json` §"release_pointer_convention". `slo-engine-adapter` (Git refs store) writes to these refs via signed PATCH.

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | update — pattern rules for `release/*/staging` and `release/*/production`; PAT scope |
| `microservices/observability/src/crates/oya-observability-slo-engine-adapter/src/git_refs_store.rs` | update — implements `ReleasePointerStore` via GitHub API (Octocrab) |
| `microservices/observability/tests/integration/release_pointer.rs` | create |

## Code Shape

```rust
// adapter/src/git_refs_store.rs
#[async_trait]
impl ReleasePointerStore for OctocrabGitRefsStore {
    async fn read(&self, ms: &str, env: Environment) -> Result<ReleasePointer, RepositoryError> {
        let ref_name = format!("release/{ms}/{env}");
        let ref_obj = self.octocrab.repos(&self.owner, &self.repo).get_ref(&Reference::Branch(ref_name)).await?;
        Ok(ReleasePointer { microservice: ms.into(), environment: env, current_sha: ref_obj.object.sha, ..Default::default() })
    }
    async fn advance(&self, ms: &str, env: Environment, sha: &Sha) -> Result<(), RepositoryError> {
        let ref_name = format!("release/{ms}/{env}");
        self.octocrab.repos(&self.owner, &self.repo)
            .update_ref(&Reference::Branch(ref_name), &sha.to_string(), false /* not forced */)
            .await?;
        Ok(())
    }
}
```

## branch-protection.yaml diff

(Per PHASE-01 §"branch-protection.yaml diff preview")

## Acceptance Gates

```bash
cargo nextest run -p oya-observability-slo-engine-adapter --test release_pointer
cargo run -p oya-dev-cli -- gate validate protection-context-match
# GitHub-side validation:
gh api repos/jason931225/oyatie/branches?protected=true | jq '.[].name'
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_read_release_pointer` | reads existing ref |
| `test_advance_release_pointer_signed` | advance PATCH; verify signature |
| `test_advance_force_push_refused` | branch-protection rejects force push |
| `integration_full_promotion_cycle` | dev SHA → staging ref advance → production ref advance |

## Halt Conditions

- GitHub branch-protection rejects pattern rule — fall back to per-µservice explicit rules; document in `multi-region.md`
- WORKFLOW_PAT scope-creep — refuse; PAT must be tightly bounded

## Next IP

[`IP-012-oya-vcs-promotion-readiness-lane.md`](IP-012-oya-vcs-promotion-readiness-lane.md)

## References

- `/specs/agentic-slo-gated-promotion.json` §"release_pointer_convention"
- ADR-0130 §"Layer-B item 11"
