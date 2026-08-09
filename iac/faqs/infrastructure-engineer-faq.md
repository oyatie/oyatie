# `cloud-iac` µservice — Infrastructure Engineer FAQ

20 real questions raised against `cloud-iac` (the µservice that owns Oyatie's declarative-infrastructure surface).

---

**Q1. Does `cloud-iac` replace Terraform?**

No — it **wraps and gates** Terraform (and Pulumi and Crossplane). The plan engine still calls Terraform 1.10 / Pulumi 3.135 /
Crossplane 1.18 under the hood, but every plan-apply flows through a Cedar permit, a tenant-scoped state backend, and an audit-chain
entry. You never call `terraform apply` directly in production.

---

**Q2. Why are modules signed?**

Because ADR-0247 + ADR-0250 require build-ahead-of-certification: every infrastructure mutation must be attributable to a signed,
attested supply chain. Cosign + Sigstore is the chosen primitive (matches what Kubernetes ecosystem standardised on). The
`lean-a8-module-attestation` CI lane refuses unsigned modules.

---

**Q3. What's the difference between `Apply` and `RemediateDrift`?**

`Apply` mutates the world to match the declared state from your module inputs. `RemediateDrift` mutates the world to match the
last-applied state (i.e. closes a drift gap that came from out-of-band changes). They're different Cedar actions because they
have different governance — `Apply` is a user-initiated change; `RemediateDrift` is auto-initiated and gates differently per tenant_class.

---

**Q4. Why three plan engines (Terraform + Pulumi + Crossplane)?**

Different tools dominate different domains:
- **Terraform** for cloud-account-scoped resources (AWS, GCP, Azure, Cloudflare).
- **Pulumi** for resources where typed language ergonomics matter (multi-region orchestration, complex conditionals).
- **Crossplane** for Kubernetes-native resources whose lifecycle is best owned by the cluster (managed by composition + claim).

Tenants pick the engine per module. The catalogue contains all three.

---

**Q5. Can I use OpenTofu instead of Terraform?**

Only for paid tenants with a governance-pre-cleared fork. The fork must publish reproducible builds + pass `lean-a8-module-attestation`.

---

**Q6. How does state isolation work between tenants?**

Each paid tenant has its own CockroachDB schema; demo_trial tenants use a row-scoped namespace in a shared cluster. State rows are
encrypted at rest with the tenant's KMS key. Cedar enforces that one tenant's principal cannot read another tenant's state row.

---

**Q7. What happens if a plan-apply takes longer than the SLO?**

The runner emits a `cloud_iac.apply.slo_breached` event to `observability`, the on-call dashboard alerts at the tenant's SLO
breach budget, and the plan can be canceled via `./bin/oya iac cancel --plan-id <id>`. Cancel is itself Cedar-gated.

---

**Q8. How does cross-account access work?**

Paid tenants can declare remote accounts they own:
```yaml
remote_accounts:
  - provider: aws
    account_id: "123456789012"
    role_arn: arn:aws:iam::123456789012:role/OyatieAssume
```
The role trust policy must permit assumption only from Oyatie's `cloud-iac` runner role + a specific external ID. The runner
assumes per-plan and disposes credentials after apply.

---

**Q9. Can I run a one-off ad-hoc `terraform apply` from my laptop?**

Not for production. For dev-cell, yes — there's a `dev-permissive` Cedar mode. Production plans must come through the `cloud-iac`
runner.

---

**Q10. How is plan output reviewed?**

For non-empty diffs, the plan output (JSON) is posted to a reviewer-agent thread. The reviewer-agent (a Cedar-gated principal)
applies the multispectrum review v2.4.0 facets and emits APPROVE / BLOCK. Human approval is required for paid `Destroy`
actions and paid dedicated-mode non-empty plans.

---

**Q11. What's the drift detection cadence?**

demo_trial: every 24 h. paid baseline: every 4 h. paid governed: every 1 h. paid dedicated mode: continuous
(event-stream from provider CloudTrail / Azure Monitor / GCP Audit Logs). Cadence is set by Cedar policy, not by runtime config.

---

**Q12. How are provider credentials managed?**

They're not managed by `cloud-iac`. They live in `cloud-secrets` and are short-lived (≤ 1 h) tokens fetched per-plan. Rotation
cadence (paid baseline 30 d, paid governed 14 d, paid dedicated mode 7 d) is enforced by `cloud-secrets`.

---

**Q13. What happens when a provider API breaks?**

The runner falls back to the previous known-good provider plugin version (pinned in the module signature). The fallback writes
a `cloud_iac.provider.degraded` audit event. If both fail, the plan returns `ProviderUnavailable` and the on-call rotation gets paged.

---

**Q14. Can I import existing cloud resources?**

Yes — `cloud_iac::Action::ImportExistingResource` permits the import flow for paid tenants with the required governance claim. The import is a special plan that produces no diff
and records the resource in tenant state. demo_trial and paid baseline tenants must clean-room re-create resources.

---

**Q15. How is module versioning handled?**

Modules are semver. `oya-iac-modules-<set>-<version>` is the catalogue identifier. Tenants pin module versions in their input file;
the catalogue's CI ensures backward-compat within minor versions and emits an ADR for any breaking change.

---

**Q16. What's the difference between a "module" and a "module set"?**

A module is a single declarative unit (e.g. `cloudflare_zone`). A module set is a catalogue of modules curated for tenant_class
(e.g. `oya-iac-modules-demo-trial-v1` contains 45 modules). Tenants subscribe to a set; sets are themselves versioned.

---

**Q17. How do I add a new provider?**

Three steps:
1. Vendor the provider plugin under `vendor/iac-providers/<provider>-<version>/`.
2. Add a Cedar action `cloud_iac::Action::Provider<Name>::Plan|Apply|Destroy`.
3. Author at least one starter module that uses the provider and ship it to the appropriate module set.

Step 1 requires governance approval because new providers expand the attack surface.

---

**Q18. Can the same plan target multiple providers?**

Yes — that's a multi-provider module. Pulumi and Crossplane both support it natively; Terraform requires explicit provider blocks
per-resource. The runner orchestrates the combined plan and records per-resource provider in the state row.

---

**Q19. Where does Foundry hook in?**

Foundry is a tenant of `cloud-iac` per ADR-0247. When a Foundry pipeline needs to mutate infra (e.g. provision a new cell),
it calls `cloud-iac` with the `oyatie.foundry.*` principal. The Cedar permits for `oyatie.foundry.*` are deliberately narrow:
no `Destroy`, no `EmergencyApply`, no `ImportExistingResource`.

---

**Q20. How do I roll back a bad apply?**

If the apply succeeded but produced bad outcomes: `./bin/oya iac rollback --apply-id <id>` reverses the last apply by computing
the inverse plan. Rollback is Cedar-gated and itself audit-logged. If the apply failed mid-flight: the runner auto-rollbacks within
the same plan transaction (Terraform state lock + provider rollback hooks).
