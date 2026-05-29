---
doc_class: PolicySpec
title: Secret Isolation Contract
microservice: cloud-secrets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-cloud-secrets
deciders: ops-security, council-architecture, axis-cloud-secrets
related_adrs: [ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/threat-model.md (T-S-01, T-S-03, T-E-01, T-E-03)
  - microservices/cloud-secrets/policy/data-residency.md
  - microservices/cloud-secrets/policy/tenant-scope.cedar
  - microservices/cloud-secrets/policy/ci-scope.cedar
  - microservices/cloud-secrets/policy/auditor-scope.cedar
  - microservices/cloud-secrets/policy/public-read.cedar
review_cadence: quarterly
doc_status: published
---

# Secret Isolation Contract (cloud-secrets µservice)

## Purpose

Define the invariants that bound the **blast radius of any single secret** in the cloud-secrets substrate. The contract has three load-bearing parts:

1. **Per-tenant namespace isolation** — every tenant owns an OpenBao namespace; cross-tenant reads are forbidden by both OpenBao policy and Cedar.
2. **Per-µservice scope isolation** — within a tenant namespace, every consumer µservice has a scope policy bounding it to `secret/<tenant>/<microservice>/*`; cross-µservice reads are forbidden.
3. **SecretReference contract** — the only sanctioned consumption form is `${openbao:secret/<path>}`; raw secret emission anywhere in the repo, chat, or checkpoint is a LEAN-A11 BLOCKER.

## Invariants

### TI-01 — Per-tenant namespace

Every tenant has exactly one OpenBao namespace at `<pack>/<tenant_namespace_path>` where `tenant_namespace_path = sha256(tenant_id + per_pack_salt)[..16]`. The namespace is provisioned by `per-tenant-namespace-controller` on `TenantRegistered`.

**Enforcement**:
- OpenBao namespace policy uses templated paths: `secret/data/{{identity.entity.aliases.<auth>.name}}/<microservice>/*` — the `{{identity}}` template ensures cross-tenant paths are unreachable.
- Cedar policy at `policy/tenant-scope.cedar` defence-in-depth.
- Audit-emit `cross_tenant_read_attempt` on any violation.

### TI-02 — Per-µservice scope

Within a tenant namespace, every consumer µservice has a scope of `secret/<tenant>/<microservice>/*`. The scope is provisioned by `per-tenant-namespace-controller` on `MicroserviceRegistered` for each `(tenant, microservice)` pair.

**Enforcement**:
- OpenBao policy at `policy/openbao/per-microservice-scope.hcl` (templated).
- LEAN-A12 lane `oya-check-openbao-policy-scope` refuses any policy granting `>scope`.
- Audit-emit `cross_microservice_read_attempt`.

### TI-03 — SecretReference is the law

Every secret consumed by any oyatie µservice MUST be referenced via the form:

```
${openbao:secret/<path>}
```

Where `<path>` is a tenant-scoped + microservice-scoped path of the form:

```
<tenant_namespace_path>/<microservice>/<secret_name>[@version]
```

Or, for shared substrate secrets that are not tenant-scoped:

```
shared/<microservice>/<secret_name>[@version]
```

Resolved by the SecretReference SDK at runtime; never written as a raw value to:
- the git repo (any file under any path);
- chat transcripts (agent dialogue logs);
- `.omc/state/` checkpoints (agent session state);
- µservice logs, metrics, traces (consumer-side logging is `Secret<T>`-wrapped);
- error messages (errors return opaque codes);
- environment variables in plain text (consumers receive references, not values; values resolved in-process).

**Enforcement**:
- **LEAN-A11 `oya-check-raw-secret-emission` lane (BLOCKER)**: gitleaks + tartufo + oyatie custom regexes scan every PR diff + commit history. Patterns include but are not limited to:
  - Generic high-entropy strings (Shannon entropy > 4.5 per character over ≥20 chars)
  - AWS: `AKIA[0-9A-Z]{16}`, `[0-9a-zA-Z/+]{40}`
  - Stripe: `sk_(live|test)_[0-9a-zA-Z]{24,}`, `pk_(live|test)_[0-9a-zA-Z]{24,}`
  - GitHub: `ghp_[0-9a-zA-Z]{36}`, `ghs_[0-9a-zA-Z]{36}`, `github_pat_[0-9a-zA-Z_]{82}`
  - Google: `AIza[0-9A-Za-z\-_]{35}`, `-----BEGIN PRIVATE KEY-----`
  - OpenBao: `hvb\.[a-zA-Z0-9_-]{20,}`, `hvs\.[a-zA-Z0-9_-]{20,}`
  - Generic PKCS#8/RSA/EC private keys: `-----BEGIN (RSA |EC |DSA |OPENSSH |)?PRIVATE KEY-----`
  - JWT-shape: `eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+`
  - OAuth client secrets matching common provider shapes (Slack, Discord, Twilio, SendGrid, etc.)
  - oyatie-specific: any string of length > 30 matching `[A-Za-z0-9+/=]{30,}` not whitelisted as `INTERNAL_ONLY`
  - HSM PKCS#11 PIN-shaped strings
- **`oya-check-secret-newtype-leak` LEAN lane**: scans Rust code for `format!("{:?}", secret)`, `.to_string()` on `Secret<T>`, `println!("{}")` patterns; refuses commit.
- **Pre-receive Git hook** on `oya-vcs`-managed branches (defence-in-depth).
- **Quarterly retroactive scan** via `oya-cloud-secrets-secret-leak-scanner` cron over full git history + `.omc/state/sessions/*` directories.
- **Reviewer-agent vigilance**: pr-review skill checks for any SecretReference URI alteration that broadens scope.

### TI-04 — Cache TTL ceiling

In-process SDK cache TTL MUST NOT exceed 60s. The TTL ensures revocation propagates within the cache invalidation window.

**Enforcement**: SDK code path uses a constant `MAX_CACHE_TTL: Duration = Duration::from_secs(60)`; saturating clamp on every cache-write; unit test validates clamp.

### TI-05 — Revocation push must propagate

On `SecretRevoked`, OpenBao emits a revocation push to every consumer; consumer SDK flushes cache for the affected secret path within ≤1s. Consumers honour revocation by re-resolving on next read, which fails with `RevokedError` until the underlying secret is rotated.

**Enforcement**: chaos-drill `revoke-cascade-drill` runs monthly per `runbooks/` schedule; SLA p99 ≤5s end-to-end for 100 consumers.

### TI-06 — Default-deny

Every Cedar policy in `microservices/cloud-secrets/policy/*.cedar` begins with an unconditional `forbid (principal, action, resource);`; permits are explicit and scoped. No catch-all permit exists.

### TI-07 — No SECRET-class data via tenant reads

Even with valid OIDC + correct tenant_id, tenant operators MUST NOT receive raw `SECRET`-class data via the REST surface (admin endpoints). Resolution happens in-process via SDK only. The `auditor-scope.cedar` policy enforces.

### TI-08 — HSM-resident KEK

KEK material MUST reside in an HSM partition (FIPS 140-3 Level 3 minimum). No software-only KEK in production packs (pack-kr, pack-eu, pack-us-healthcare, pack-ksa, pack-ae); sandbox/test packs may use software-only KEK explicitly flagged in IaC values.

**Enforcement**: openbao-operator refuses to bring up a cluster with `auto_unseal.type != "pkcs11"` in regulated pack overlays.

### TI-09 — Per-pack residency

Secrets are stored in pack-pinned OpenBao + HSM. Cross-pack replication forbidden. See `policy/data-residency.md`.

### TI-10 — 4-eye break-glass

Privileged admin operations (cluster unseal recovery, KEK rotation, namespace seal) require 2 ops-security approvers via OpenBao Sentinel policy `4_eye_approval`. JIT elevation expires in 1h. Audit-emit every break-glass approval + use.

### TI-11 — encryption-key BYOK acceptance (ADR-0251 §D-10)

Tenant-supplied encryption-key BYOK material (ADR-0251 §D-10) MUST be wrapped under the per-pack KEK-of-KEKs before storage. encryption-key BYOK upload requires OIDC + MFA + JIT token; signed receipt issued to tenant; `KekAttested` audit event recorded.

## Anti-Patterns (forbidden)

| Anti-pattern | Why forbidden | Remediation |
|---|---|---|
| Raw secret value in any YAML / env / config file | Repo / image leak; violates user directive | Replace with `${openbao:secret/...}` reference; LEAN-A11 BLOCKS |
| Logging resolved secret value | Logs are shipped to Loki + retained | Use `Secret<T>` newtype; consumer must use `with_secret(|s| ...)` callback |
| Forwarding a secret across µservices via Workflow event | Secret in event payload = secret in audit-chain + workflow-engine = leak surface | Each consumer resolves its own reference |
| Caching beyond 60s | Revocation won't propagate in time | Saturating clamp at SDK |
| Storing a secret in a database table (any µservice) | DB leak surface | Reference via SecretReference; resolve at runtime |
| Echoing secret in error message | Error logs are noisy + retained | Errors return opaque codes |
| Storing a secret in `.omc/state/` (agent checkpoint) | Checkpoint persistence == leak | Agent context never contains raw secret; only reference |
| Storing a secret in chat (agent dialogue) | Chat transcripts retained + exportable | Agent treats secret references symbolically; never resolves |
| Importing `oya-cloud-secrets-*` from a different product | Cross-product import violation | Consume via SDK only |
| HSM bypass for "test in production" | Test in stage; PR-review enforces | Stage cluster has its own HSM partition |

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate lean-a11 --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate lean-a12 --microservice cloud-secrets   # policy-scope
cargo run -p oya-dev-cli -- gate validate secret-newtype-leak --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate cedar-deny-by-default --policy microservices/cloud-secrets/policy/
```

Monthly chaos drills:
- `revoke-cascade-drill` — verify p99 ≤5s end-to-end for 100 consumers.
- `cross-tenant-read-attempt-drill` — verify Cedar + OpenBao both refuse.
- `cross-microservice-read-attempt-drill` — verify scope refuses.
- `raw-secret-emission-drill` — seed a known-fake credential into a PR; verify LEAN-A11 blocks.

## References

- `microservices/cloud-secrets/threat-model.md` (mitigations matrix)
- `microservices/cloud-secrets/policy/data-residency.md`
- `microservices/cloud-secrets/policy/{tenant-scope,ci-scope,auditor-scope,public-read}.cedar`
- ADR-0028, ADR-0131
- OpenBao best-practices (informing OpenBao migration)
- NIST SP 800-57 Part 1 (Key Management — General)
