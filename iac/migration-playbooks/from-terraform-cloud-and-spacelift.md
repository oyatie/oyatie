# Migration playbook — Terraform Cloud or Spacelift → Oyatie `cloud-iac`

Audience: an infra engineer running ≥ 1 workspace on Terraform Cloud (any tier) or Spacelift (any tier) who wants to migrate to
`cloud-iac` without state loss or downtime windows.

## Phase 0 — Inventory (Day 0…3)

### From Terraform Cloud
1. Export workspace list:
   ```bash
   curl -H "Authorization: Bearer $TFC_TOKEN" \
        https://app.terraform.io/api/v2/organizations/$ORG/workspaces > tfc-workspaces.json
   ```
2. For each workspace export current state:
   ```bash
   jq -r '.data[] | .id' tfc-workspaces.json | while read id; do
     curl -H "Authorization: Bearer $TFC_TOKEN" \
          "https://app.terraform.io/api/v2/workspaces/$id/current-state-version" \
       -o "state-$id.json"
   done
   ```
3. Note any Sentinel policies attached — you'll translate these to Cedar.

### From Spacelift
1. Export stacks:
   ```bash
   spacectl stack list --output json > spacelift-stacks.json
   ```
2. Export state per-stack via Spacelift API; mark whether the stack uses a private worker pool (`worker_pool_id != null`).

## Phase 1 — Module catalogue mapping (Day 3…7)

For each workspace/stack, map its Terraform modules to the `cloud-iac` catalogue:

| Source module | Closest `cloud-iac` module | Action |
| --- | --- | --- |
| `terraform-aws-modules/vpc/aws` v5+ | `aws-vpc-canonical` in `iac-modules-paid-per-usage-v1` | Direct map; update input names |
| `terraform-aws-modules/eks/aws` v20+ | `aws-eks-managed` in `iac-modules-paid-governed-v1` | Map; paid-only |
| Custom in-house module | n/a | Author as a new module + sign + submit through `GitOps change-bundle claim` |

For each module not in the catalogue, file a backlog entry under `microservices/cloud-iac/backlog/starter-modules.md`.

## Phase 2 — Sentinel → Cedar translation (Day 7…14)

Sentinel policies translate roughly 1:1 to Cedar `forbid` clauses. Example:

Sentinel:
```hcl
main = rule {
  all tfplan.resource_changes as _, rc {
    rc.type is not "aws_security_group_rule" or
    rc.change.after.cidr_blocks not contains "0.0.0.0/0"
  }
}
```

Cedar equivalent (`policies/forbid-public-sg.cedar`):
```cedar
forbid (
  principal,
  action == cloud_iac::Action::Apply,
  resource
)
when {
  resource.kind == "aws_security_group_rule" &&
  resource.attributes.cidr_blocks.contains("0.0.0.0/0")
};
```

Lint:
```bash
./bin/oya policy lint --tenant <tenant> policies/
```

## Phase 3 — State import (Day 14…21)

For each workspace, run the state migrator:
```bash
./bin/oya iac migrate import \
  --source-format terraform-cloud \
  --source-state state-<id>.json \
  --target-tenant oyatie.b2b.smb.<org> \
  --target-module-set iac-modules-paid-per-usage-v1
```

The migrator produces a `migration-plan.json` summarising how each resource will be re-homed. Review before confirming:
```bash
./bin/oya iac migrate confirm --migration-id <id>
```

This does not mutate cloud resources; only imports them into `cloud-iac` tenant state.

## Phase 4 — Dual-run (Day 21…35)

Keep both systems live but flag the migrated tenant as **read-only on the source side**:
- Terraform Cloud: lock workspaces (`curl -XPOST .../actions/lock`).
- Spacelift: enable "manual confirmation only" + revoke runner auto-trigger.

In `cloud-iac`, run plans with `--dry-run` to verify they produce the expected diff:
```bash
./bin/oya iac plan --tenant <t> --inputs <i> --dry-run
```

A correctly-imported tenant produces empty diffs on dry-run.

## Phase 5 — Cut-over (Day 35…42)

1. Run a final apply on the source side (if any pending changes).
2. Re-import state to `cloud-iac`.
3. Verify empty diff.
4. Flip your CI/CD to write to `cloud-iac` only.
5. Keep source workspaces locked for 30 d as rollback.

## Phase 6 — Decommission source (Day 42+)

After 30 d clean run on `cloud-iac`:
- Terraform Cloud: delete workspaces (`curl -XDELETE ...`).
- Spacelift: archive stacks.
- Cancel paid plans.

## Rollback strategy

Within the 30-day window:
1. Unlock source workspaces.
2. Re-point CI/CD to source.
3. Run a `terraform refresh` on source to pick up any out-of-band changes from `cloud-iac` operations.
4. Disable `cloud-iac` runner for that tenant.

After source decommission: rollback requires state re-export from `cloud-iac` and re-import to a fresh source workspace. Plan
on 4-8 h for a paid governed tenant.

## What you gain

- 18 % cost reduction at mid-market scale vs Terraform Cloud Plus.
- Multi-engine (TF + Pulumi + Crossplane) under one Cedar policy surface.
- Reviewer-agent built-in (multispectrum v2.4.0).
- Per-pack regulatory overlays (SOC2/GDPR/HIPAA/PCI/EU-AI-Act).
- BLAKE3 audit-chain instead of append-only logs.
- HTTP/3 (QUIC) runner protocol.

## What you give up

- Vendor-native VCS UI (Oyatie surfaces this through `workflow-studio` instead).
- Sentinel / OPA policy ecosystem (must rewrite to Cedar).
- Provider plugin freshness (Oyatie re-vendors monthly; vendors update within hours).
- Self-service public signup (you must provision tenant_class).
