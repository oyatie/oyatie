# `cloud-iam` µservice — IAM Engineer FAQ

20 real questions raised against `cloud-iam` (the µservice that owns Oyatie's identity-and-access surface).

---

**Q1. Does `cloud-iam` replace AWS IAM / GCP IAM / Azure RBAC / Okta?**

It **wraps and translates**. The authority is Cedar; AWS IAM/GCP IAM/Azure RBAC/Okta are downstream surfaces. When a tenant's
Cedar policy needs to govern an AWS API call, `cloud-iam` compiles the relevant Cedar `permit` to AWS IAM JSON and writes it
into the tenant's AWS account. The Cedar source remains authoritative; the IAM JSON carries a `blake3-256` digest pointer.

---

**Q2. Why Cedar instead of OPA?**

Cedar gives in-process evaluation in ≤ 200 µs (paid tenant_class policy), full algorithmic decidability, and a closed-world type system. OPA's
Rego is Turing-complete and harder to certify for high-assurance use; Cedar is the AWS-stewarded choice for ABAC. ADR-0243 binds it.

---

**Q3. What's a `Workload` principal vs a `User` principal?**

`User` is a human (or a delegated identity such as a service account that a human operates). `Workload` is a SPIFFE/SPIRE-bound
identity belonging to a running process (e.g. a pod, a VM, a Lambda). Workloads attest themselves via X.509 SVIDs;
Users authenticate via SAML/OIDC/passkey/TOTP.

---

**Q4. How does federated identity flow end-to-end?**

Okta (or Entra ID, etc.) issues a SAML assertion (or OIDC ID token). `cloud-iam`'s `/saml/v2/acs` (or `/oidc/v1/callback`) endpoint
validates the signature against the cached IdP metadata, applies the JIT provisioning rules (`oya iam idp jit-rules ...`), and
materialises a `User` principal + role assignments in the Cedar entity store. The user then gets a short-lived Oyatie session token.

---

**Q5. Can I issue a long-lived AWS access key?**

No. `cloud-iam` refuses static credentials. To call AWS APIs, your workload assumes an AWS IAM role via STS; `cloud-iam` brokers
the AssumeRole call, returning credentials with a max TTL of 1 h (paid tenant_class) / 4 h (paid tenant_class). The role trust policy must accept only
the `cloud-iam` runner's identity + an external ID scoped to the tenant.

---

**Q6. How does Cedar → AWS IAM translation handle Cedar features AWS IAM doesn't have?**

Mostly via two strategies:
1. **Pre-compute and pin.** Cedar group membership, attribute lookups, and entity hierarchies are evaluated at translation time
   and pinned into the IAM JSON's `Resource` / `Condition` blocks.
2. **Refuse with a clear error.** Cedar policies using closures or complex slice expressions that AWS IAM cannot represent yield
   `TranslationError::UnrepresentableInTarget` at translation time. You then either rewrite the Cedar policy or evaluate it in
   `cloud-iam` directly (i.e. don't push it to AWS IAM).

---

**Q7. Does `cloud-iam` support Okta and Entra ID simultaneously for the same tenant?**

Yes — a single tenant can register N IdPs. The user picks one at login; JIT rules can map identities across IdPs to the same
Cedar `User` (via the `external_id` claim in SAML/OIDC). This is how M&A scenarios work (you migrate Entra users into the
acquirer's Okta over months while both IdPs remain valid).

---

**Q8. What's the per-tenant Cedar entity store schema?**

```sql
CREATE TABLE cedar_entities (
  tenant_id     TEXT NOT NULL,
  entity_uid    TEXT NOT NULL,           -- e.g. 'User::"oyatie.b2b.smb.acme/alice"'
  entity_kind   TEXT NOT NULL,           -- 'User' | 'Role' | 'Application' | ...
  attributes    JSONB NOT NULL,
  parents       JSONB NOT NULL,          -- [entity_uid, ...]
  created_at    TIMESTAMPTZ NOT NULL,
  updated_at    TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, entity_uid)
) PARTITION BY HASH (tenant_id);
```

Indexed on `(tenant_id, entity_kind)` for fast principal-set enumeration.

---

**Q9. How is the entity store kept fresh at Cedar eval time?**

Each `cloud-iam` runner caches the tenant's full entity store (typically ≤ 50 MB for paid tenant_class) in process memory and listens to
`cloud_iam.entity.*` events on the Kafka audit stream for invalidation. The cache propagates within ~80 ms p95 across runners.

---

**Q10. Can Cedar policies reference data from `cloud-data`?**

Yes, via the `context.data_lookup(...)` extension. The lookup is bounded (≤ 2 hops, ≤ 50 entities, ≤ 5 ms p95) and audit-logged.
Use sparingly — most policies should be self-contained on principal + resource attributes.

---

**Q11. How does the cross-tenant bridge (B2B SaaS) work?**

A `CrossTenantBridge` principal (paid tenant_class) represents user A from tenant X interacting with tenant Y's resources. The bridge token
carries both `principal.tenant_id = X` and `context.resource_owner_tenant_id = Y`. Both tenants must have Cedar `permit` clauses
referencing the bridge — neither alone is sufficient.

---

**Q12. What happens if my IdP's metadata signing certificate expires?**

`cloud-iam` polls metadata every 1 h. On the first poll where the certificate is invalid, it raises a `cloud_iam.idp.degraded`
event. Logins continue using the cached metadata (≤ 24 h grace). At 24 h, logins refuse with `IdpMetadataExpired` and the tenant
admin is paged.

---

**Q13. How is the audit chain anchored?**

Every authorisation decision writes a record to `audit-chain` (the BLAKE3-256 chained log). The chain is anchored hourly to the
`audit-chain` Merkle root which is signed by an HSM-bound key at paid tenant_class. Compliance evidence cites the chain root + range.

---

**Q14. Can I do break-glass auth without MFA?**

At demo_trial tenant_class boundary: no (refused). At paid tenant_class: yes, with a `cloud_iam.emergency.break_glass` event + reviewer-agent surveillance.
At paid tenant_class: yes, but every action is ECDSA-signed by the operator's PIV/CAC card, and the session is recorded
(input + output stream) for 7 y per FedRAMP High.

---

**Q15. How does workload identity rotation work?**

SPIFFE SVIDs rotate every 1 h (demo_trial tenant_class) / 30 m (paid tenant_class) / 15 m (paid tenant_class) / 5 m (paid tenant_class). Rotation is automatic via SPIRE agent;
no application change required. Workloads with SVID skew > 2 rotations are quarantined.

---

**Q16. What's the schema for Cedar entity UIDs?**

`<EntityKind>::"<tenant_id>/<local_id>"` — e.g. `User::"oyatie.b2b.smb.acme-software/alice"`,
`Role::"oyatie.b2b.smb.acme-software/engineer"`, `Application::"oyatie.b2b.smb.acme-software/tasks"`. The tenant prefix is
mandatory; `cloud-iam` refuses UIDs without it.

---

**Q17. How do I deprecate a role without orphaning principals?**

```bash
./bin/oya iam role deprecate \
  --tenant <t> \
  --role "Role::\"<t>/old-role\"" \
  --successor "Role::\"<t>/new-role\"" \
  --migration-deadline 2026-09-01
```

`cloud-iam` rewrites Cedar policies referencing `old-role` to also accept `new-role`, emits a `cloud_iam.role.deprecated` event,
and after the deadline rejects new principal-role attachments to `old-role`.

---

**Q18. Can the Cedar policy itself be tenant-scoped?**

Yes — policy storage is tenant-partitioned. A policy in tenant X's policy set cannot reference entities in tenant Y. The
`lean-a3-tenant-trace` lane refuses cross-tenant references at lint time.

---

**Q19. Where does Foundry hook in?**

Foundry pipelines run as `oyatie.foundry.<pipeline-id>` principals per ADR-0247. The Cedar permits for `oyatie.foundry.*` are
narrow and explicit (no `EmergencyBreakGlass`, no `IssueCrossTenantToken`, no `TranslateToAwsIam` unless the pipeline is a
known IaC pipeline). Foundry mutations through `cloud-iam` are audit-chain-anchored alongside human mutations.

---

**Q20. How do I roll back a bad policy push?**

`cloud-iam` keeps the last 256 versions of each policy + the entity store snapshot per version. Rollback:
```bash
./bin/oya iam policy rollback --tenant <t> --policy <name> --to-version <n>
```

Rollback is itself Cedar-gated (`cloud_iam::Action::RollbackPolicy`) and audit-logged. The previous policy returns to authority
within ≤ 80 ms p95.
