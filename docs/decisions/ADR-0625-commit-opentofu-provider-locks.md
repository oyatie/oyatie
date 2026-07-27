---
id: ADR-0625
title: "Commit OpenTofu provider dependency locks for every deployable root"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-26
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: [ADR-0013]
related_specs: []
milestone: W3
---

# ADR-0625: Commit OpenTofu provider dependency locks for every deployable root

## Status

**Proposed — 2026-07-26.** Two-way door: reverting means deleting the lock files,
after which `tofu init` resumes resolving providers at run time.

## Context

Before this decision, **zero** deployable OpenTofu roots carried a
`.terraform.lock.hcl`. Every `tofu init` therefore resolved provider plugins to
whatever the registry served at that moment.

The gap is not theoretical, and the repository contains its own counter-example:
`infra/cloudflare` declares `cloudflare/cloudflare ~> 4.0` and resolves **4.52.8**,
while `cloud/cloud-iac/tofu/provider-locks/foundation` declares `>= 4.0.0` for the
same provider and resolved **5.19.1**. The same provider, two majors apart,
decided by when someone happened to run `init`.

**A version constraint does not pin. A lock does.**

One root could never have had a lock at all: `infra/cloudflare/.gitignore` listed
`.terraform.lock.hcl`, which defeats the file's entire purpose — a dependency lock
exists to be committed, and ignoring it means every `init` re-resolves and may
select a different provider version than the last apply used.

## Decision

### D1 — Every deployable root commits its lock

Locks are generated with

```
tofu providers lock -platform=linux_amd64 -platform=darwin_arm64
```

which records checksums **without** contacting a backend, reading state, or using
credentials. Both platforms are recorded because CI runs linux and development
runs darwin.

### D2 — The review root is not a deployment lock, and is unchanged

`cloud/cloud-iac/tofu/provider-locks/foundation` documents itself as existing
"only to materialize provider dependency selections and checksums for local
review", with no backend and no plan/apply surface, and it uses deliberately loose
`>=` constraints to survey what is current. It is a provider-signature review
surface. The two are complementary and conflating them would break its purpose.

### D3 — Roots that cannot be locked are recorded, not faked

A root that cannot parse, cannot resolve its providers, or references module
directories that do not exist cannot be locked. Those are recorded as findings —
`F-IAC-ROOTS-DO-NOT-PARSE`, `F-IAC-PROVIDERS-UNRETRIEVABLE`,
`F-IAC-DANGLING-MODULE-REFERENCES`, `F-IAC-OYATIE-PROVIDER-NEVER-PUBLISHED` — and
**not** given a hand-written lock, which would imply the root works.

### D4 — These locks are currently unenforced, and that is stated

No gate verifies them: all three cloud-iac lock gates hardcode the pre-reorg path
`microservices/cloud-iac/`, which no longer exists. The locks will drift until a
gate covers them (`F-IAC-LOCKS-UNENFORCED`). Recording that here is deliberate —
a control believed present but absent is worse than one known missing.

## Justified artifacts

This decision governs, and thereby justifies, the following committed locks. Each
is the dependency lock of the root directory containing it; a lock is meaningless
anywhere other than beside its own root, so these paths relocate with their roots
under the reorg rather than being placed here by choice.

- `infra/cloudflare/.terraform.lock.hcl`
- `infra/cloudflare/OWNERS`
- `oya/community/iac/terraform/.terraform.lock.hcl`
- `oya/emergency/iac/colo/.terraform.lock.hcl`
- `oya/emergency/iac/oci-always-free/.terraform.lock.hcl`
- `oya/emr/iac/colo/.terraform.lock.hcl`
- `oya/emr/iac/guest-on-aws/.terraform.lock.hcl`
- `oya/emr/iac/guest-on-oci/.terraform.lock.hcl`
- `oya/emr/iac/on-prem/.terraform.lock.hcl`
- `oya/emr/iac/oyatie-as-cloud-provider/.terraform.lock.hcl`
- `oya/emr/iac/oyatie-public-cloud/.terraform.lock.hcl`
- `oya/finops-portal/iac/.terraform.lock.hcl`
- `oya/forms/iac/terraform/.terraform.lock.hcl`
- `oya/identity/iac/colo/.terraform.lock.hcl`
- `oya/identity/iac/guest-on-aws/.terraform.lock.hcl`
- `oya/identity/iac/guest-on-oci/.terraform.lock.hcl`
- `oya/identity/iac/oci-guest/always-free/.terraform.lock.hcl`
- `oya/identity/iac/on-prem/.terraform.lock.hcl`
- `oya/identity/iac/oyatie-as-cloud-provider/.terraform.lock.hcl`
- `oya/identity/iac/oyatie-public-cloud/.terraform.lock.hcl`
- `oya/imaging/iac/aws-guest/.terraform.lock.hcl`
- `oya/imaging/iac/colo/.terraform.lock.hcl`
- `oya/imaging/iac/oci-guest/.terraform.lock.hcl`
- `oya/imaging/iac/on-prem/.terraform.lock.hcl`
- `oya/messenger/iac/colo/.terraform.lock.hcl`
- `oya/messenger/iac/guest-on-aws/.terraform.lock.hcl`
- `oya/messenger/iac/guest-on-oci/.terraform.lock.hcl`
- `oya/messenger/iac/on-prem/.terraform.lock.hcl`
- `oya/messenger/iac/oyatie-as-cloud-provider/.terraform.lock.hcl`
- `oya/messenger/iac/oyatie-public-cloud/.terraform.lock.hcl`
- `oya/observability/iac/terraform/.terraform.lock.hcl`
- `oya/pharmacy/iac/oci-guest/always-free/.terraform.lock.hcl`
- `oya/recordings/iac/terraform/.terraform.lock.hcl`

## Consequences

Provider selection becomes reproducible and reviewable per root. Refreshing a lock
becomes a deliberate, reviewed change rather than an accident of timing — which is
precisely the property that was missing.

The locks pin what the registry serves today. That is the point: they make the
selection reproducible, not eternally correct.

## Alternatives considered

- **Extend the foundation review root to cover all roots** — rejected by D2: it
  has no apply surface and uses loose constraints by design.
- **Add `.terraform.lock.hcl` to the accounting `path_excludes`** — rejected. The
  locks are meaningful supply-chain artifacts; excluding them from accounting to
  avoid justifying them would hide exactly what should be recorded.
