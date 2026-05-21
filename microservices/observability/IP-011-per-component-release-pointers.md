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
acceptance_lanes: [oya-governance-protection-context-match]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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


## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-011-per-component-release-pointers.md` matched `multi-region`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Next IP

[`IP-012-oya-vcs-promotion-readiness-lane.md`](IP-012-oya-vcs-promotion-readiness-lane.md)

## References

- `/specs/agentic-slo-gated-promotion.json` §"release_pointer_convention"
- ADR-0139 §"Layer-B item 11"
