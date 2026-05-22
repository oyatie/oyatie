# Infrastructure Engineer — First Week on `cloud-iac`

Audience: an infra/platform engineer with Terraform + Pulumi + Kubernetes experience joining the `oya-cloud-iac-*` lane.
Goal: by Friday EOD you can author + apply a new module, detect drift, and run a multi-provider plan via Foundry.

## Day 1 — read before touching

- `docs/decisions/ADR-0218-cloud-iac-canonical-declarative-infra.md` — binding definition.
- `docs/decisions/ADR-0247-self-modification.md` — `cloud-iac` is a substrate that Foundry uses to mutate itself; understand the
  recursion (Foundry is a tenant of `cloud-iac`).
- `docs/decisions/ADR-0250-build-ahead-of-certification.md` — every module must be authored to compliance-certified shape day one.
- ADR-0329 + ADR-0330 + ADR-0331 — retired legacy vocabulary and the tenant_class replacement.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-iac-week1 .worktrees/$USER-iac-week1
cd .worktrees/$USER-iac-week1
```

## Day 2 — walk a real apply end-to-end

Start a loopback cloud-iac cell:
```bash
make dev-cell.up CELL=iac-loopback-1 PROFILE=cloud-iac-dev
```

Run the canonical sample plan:
```bash
./bin/oya iac plan \
  --tenant oyatie.community.dev-sample \
  --module-set oya-iac-modules-demo-trial-v1 \
  --inputs samples/demo-trial/minimal.yaml
```

Expected: structured plan output with `resources_to_add: 7, resources_to_change: 0, resources_to_destroy: 0` and a Cedar permit log
showing 7 individual `cloud_iac::Action::Plan` permits.

Apply (still loopback):
```bash
./bin/oya iac apply --plan-id $(jq -r .plan_id last-plan.json)
```

## Day 3 — author your first module

Pick an unallocated starter module from `microservices/cloud-iac/backlog/starter-modules.md`. Author under
`crates/oya-iac-modules-demo-trial/src/<name>/mod.rs`:

```rust
use oya_iac_module::prelude::*;

#[derive(Module)]
#[module(
    name = "demo-trial-cloudflare-zone",
    version = "0.1.0",
    providers = ["cloudflare 4.x"],
    cedar_action = "cloud_iac::Action::Plan"
)]
pub struct CloudflareZoneModule {
    #[input(required)]
    pub zone_name: String,
    #[input(default = "free")]
    pub plan: String,
    #[output]
    pub zone_id: String,
}
```

Add the corresponding Cedar permit in `policies/demo-trial/cloudflare-zone.cedar`. Add a substance test in
`crates/oya-iac-modules-demo-trial/tests/cloudflare_zone.rs`. Run:
```bash
cargo test -p oya-iac-modules-demo-trial --features dev-cell
```

## Day 4 — claim and ship the module

```bash
./bin/oya vcs claim \
  --agent infra-eng-$USER \
  --intent cloud-iac-module-cloudflare-zone \
  crates/oya-iac-modules-demo-trial microservices/cloud-iac
```

Implement + verify + done; open PR:
```bash
gh pr create --base dev --title "cloud-iac: demo-trial cloudflare-zone module"
```

The Foundry admission gate enforces `lean-a8-module-attestation` (cosign signature), `lean-a5-doc-coverage` (the README on the
module), and `lean-a3-tenant-trace` (every action carries `tenant_id`).

## Day 5 — drift detection and remediation drill

Manually induce drift in your loopback cloud account (mock provider):
```bash
./bin/oya iac mock provider-mutate \
  --resource cloudflare_zone.example_com \
  --field plan \
  --new-value "pro"
```

Run drift detection:
```bash
./bin/oya iac drift --tenant oyatie.community.dev-sample
```

Expected output: 1 drifted resource, with the diff and a remediation plan. Apply remediation:
```bash
./bin/oya iac remediate-drift --tenant oyatie.community.dev-sample --resource cloudflare_zone.example_com
```

The remediation is a fresh `cloud_iac::Action::RemediateDrift` Cedar permit — different from `Apply`.

## What "done with week 1" means

- [ ] You can recite `tenant_class` eligibility and the paid billing components emitted by `cloud-iac`.
- [ ] You authored, signed, and merged a module through Foundry.
- [ ] You walked a drift detection + remediation cycle end-to-end.
- [ ] You read ADR-0218 + ADR-0247 + ADR-0250.
- [ ] You filed at least one `oya vcs note` against a gap you found in `demo_trial`.

## Rookie traps

1. **Hand-rolling Terraform.** All Terraform must come through the `cloud-iac` module surface. Hand-written `.tf` files in repo
   trip the `lean-a8-module-attestation` lane.
2. **Mixing tenant-class gates.** A `demo_trial` tenant cannot use paid-only module gates even on dev. Cedar refuses.
3. **Forgetting drift policy.** A module that doesn't declare `#[drift_policy(...)]` defaults to `manual-only` — meaning automated
   remediation is disabled.
4. **Skipping cosign.** Unsigned modules fail at plan time with `ModuleAttestationMissing`.
