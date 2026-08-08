# Migration playbook — HashiCorp Vault → Oyatie `cloud-secrets`

Audience: a security engineer with a running Vault cluster (any edition, any HA topology) who wants to migrate to `cloud-secrets`
without secret leaks, downtime, or compliance gaps.

> Phase budget: 21 days for an SMB paid tenant, 60 days for paid, 120 days for paid with regulated packs and encryption-key BYOK ceremony (ADR-0251 §D-10).

## Phase 0 — Inventory (Day 0…5)

1. Enumerate mounts:
   ```bash
   vault secrets list -format=json > vault-mounts.json
   ```
2. For each mount, classify:
   - **kv** (static secrets) → map to `cloud-secrets` static
   - **database** (dynamic Postgres/MySQL/Mongo/etc) → map to `cloud-secrets` dynamic backend
   - **pki** → map to `kms` (not `cloud-secrets`)
   - **transit** → map to `kms` (not `cloud-secrets`)
   - **aws / azure / gcp** (dynamic cloud IAM) → map to `cloud-secrets` `IamRoleAssumption`
   - **ssh** → map to `cloud-secrets` `SshUserKey` + `SshHostKey`
3. Enumerate policies:
   ```bash
   vault policy list | while read p; do
     vault policy read -format=json "$p" > "policy-$p.json"
   done
   ```
4. Enumerate auth methods (AppRole, JWT, Kubernetes, AWS IAM, etc) — each becomes a Cedar principal mapping.

## Phase 1 — Tenant provisioning (Day 5…7)

```bash
./bin/oya tenant create \
  --id oyatie.b2b.smb.acme-software.vault-migration \
  --tenant-class paid \
  --region us-east-2 \
  --pack-set "soc2-type-ii-v2017"
```

If paid tenant_class with encryption-key BYOK is required (ADR-0251 §D-10), schedule the encryption-key BYOK ceremony 5-7 calendar days out:
```bash
./bin/oya secrets byok-schedule \
  --tenant oyatie.b2b.smb.acme-software.vault-migration \
  --hsm-profile aws-cloudhsm \
  --participants "alice@oyatie,bob@oyatie,carol@oyatie,dan@acme,erin@acme,frank@acme" \
  --ceremony-date 2026-06-01
```

## Phase 2 — Policy translation (Day 7…14)

Vault policies translate to Cedar policies. Example:

Vault HCL:
```hcl
path "kv/data/prod/postgres-primary" {
  capabilities = ["read"]
}
path "kv/data/prod/*" {
  capabilities = ["list"]
}
```

Cedar equivalent (`policies/acme/postgres.cedar`):
```cedar
permit (
  principal in oyatie.b2b.smb.acme-software.vault-migration::Group::"prod-app",
  action == cloud_secrets::Action::Read,
  resource == cloud_secrets::Secret::"kv/prod/postgres-primary"
);

permit (
  principal in oyatie.b2b.smb.acme-software.vault-migration::Group::"prod-app",
  action == cloud_secrets::Action::List,
  resource
)
when {
  resource.path.starts_with("kv/prod/")
};
```

Lint:
```bash
./bin/oya policy lint --tenant oyatie.b2b.smb.acme-software.vault-migration policies/
```

## Phase 3 — Auth method translation (Day 14…21)

| Vault auth | `cloud-secrets` mapping |
| --- | --- |
| AppRole | `cloud_secrets::Principal::Workload` with role-id + secret-id stored in `kms` |
| Kubernetes service account JWT | `cloud_secrets::Principal::K8sServiceAccount` (uses SA JWT projection) |
| AWS IAM auth | `cloud_secrets::Principal::AwsIam` (uses STS GetCallerIdentity) |
| JWT/OIDC | `cloud_secrets::Principal::Oidc` (binds to identity µservice OIDC) |
| Token auth | maps to short-lived bearer leased from `identity` |

Each mapping requires a Cedar permit linking the principal to the secrets it should access.

## Phase 4 — Static secret migration (Day 21…35)

```bash
./bin/oya secrets migrate import \
  --source-format hashicorp-vault \
  --source-url https://vault.acme.internal:8200 \
  --source-token "$VAULT_TOKEN" \
  --target-tenant oyatie.b2b.smb.acme-software.vault-migration \
  --mount-filter "kv/" \
  --dry-run
```

Review the dry-run output, then:
```bash
./bin/oya secrets migrate import ... --confirm
```

The migrator preserves Vault version history (Vault KV v2 supports versioning; the migrator maps versions 1:1).

## Phase 5 — Dynamic backend migration (Day 35…50)

Database backends require parallel setup — `cloud-secrets` must register the same backends with admin credentials. Then point your
workloads at both Vault and `cloud-secrets` simultaneously for a 1-week dual-issuance window:
```bash
./bin/oya secrets backend register \
  --tenant oyatie.b2b.smb.acme-software.vault-migration \
  --kind postgres \
  --name postgres-prod-primary \
  --connection-uri "postgres://oyatie_admin@db.acme.internal:5432/acme?sslmode=require" \
  --admin-secret secrets/db-admin/postgres-prod-primary
```

After 1 week clean dual-issuance, remove the Vault database backend.

## Phase 6 — Workload cut-over (Day 50…56)

Update each workload's secret-resolution path:

Before (Vault):
```bash
export DATABASE_URL=$(vault kv get -field=connection_url kv/prod/postgres-primary)
```

After (`cloud-secrets`):
```bash
export DATABASE_URL=$(oya secrets get --tenant oyatie.b2b.smb.acme-software.vault-migration \
                                       --name kv/prod/postgres-primary --field connection_url)
```

For Kubernetes, swap the Vault Agent sidecar for the `cloud-secrets` CSI driver:
```yaml
volumes:
  - name: secrets
    csi:
      driver: secrets.oyatie.io
      readOnly: true
      volumeAttributes:
        tenant: oyatie.b2b.smb.acme-software.vault-migration
        secrets: "kv/prod/postgres-primary,kv/prod/valkey-primary"
```

## Phase 7 — Vault decommission (Day 56+)

After 30 d clean run, decommission Vault:
1. Revoke all remaining tokens: `vault token revoke -mode=path auth/`
2. Seal Vault: `vault operator seal`
3. Snapshot Vault data for compliance retention (typically 7 y): `vault operator raft snapshot save`
4. Tear down Vault infrastructure.

## Rollback

Within the 30-day window:
1. Reverse the workload cut-over (point back at Vault).
2. Vault is still running (sealed but not destroyed); unseal: `vault operator unseal`.
3. Refresh any rotated secrets in Vault (since `cloud-secrets` rotated them during dual-run).

After Vault decommission: rollback requires re-deploying Vault from snapshot and re-migrating secrets back via export. Plan on 1-2
days for an SMB tenant.

## What you gain

- 2-3x lower p95 latency on reads.
- Per-tenant Cedar + pack overlays vs identity-policy-only.
- encryption-key BYOK ceremony + HSM-backed KEK at paid tenant_class (ADR-0251 §D-10).
- BLAKE3 audit chain (verifiable) vs append-only audit devices.
- HTTP/3 default.
- No Vault Enterprise license cost.

## What you give up

- 50+ Vault secret engines (vs 16 first-party + extension).
- Vault Agent local rendering (`cloud-secrets` CSI is the K8s answer; non-K8s workloads use the SDK).
- The "Vault is everywhere" mental model (`cloud-secrets` is Oyatie-tenant-scoped).
