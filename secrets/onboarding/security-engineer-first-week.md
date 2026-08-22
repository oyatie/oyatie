# Security Engineer — First Week on `cloud-secrets`

Audience: a security/cryptography engineer with KMS + HSM experience (Vault / AWS KMS / Azure Key Vault / Akeyless) joining the
`cloud-secrets-*` lane.

## Day 1 — required reading

- `docs/adr-archive/ADR-0219-no-code-first-ux-with-optional-ai-assist.md` — binding scope.
- `docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md` §D-4 — provider-credential BYOK.
- `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md` — pack overlays for HIPAA/GDPR/SOC2/PCI/EU-AI-Act.
- NIST SP 800-57 Part 1 Rev 5 — key management lifecycle (you'll need this for rotation reasoning).
- RFC 9180 — HPKE; cloud-secrets uses HPKE for cross-region replication of secret envelopes.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-secrets-week1 .worktrees/$USER-secrets-week1
```

## Day 2 — walk a real lifecycle

```bash
make dev-cell.up CELL=secrets-loopback-1 PROFILE=cloud-secrets-dev
make dev-tenant.create T=oyatie.community.dev-sample TENANT_CLASS=demo_trial
```

Create + read + rotate + lease in one session:
```bash
./bin/oya secrets put --tenant oyatie.community.dev-sample --name db-password --value "p@ssw0rd-initial"
./bin/oya secrets get --tenant oyatie.community.dev-sample --name db-password --lease 1h
./bin/oya secrets rotate --tenant oyatie.community.dev-sample --name db-password --new-value "p@ssw0rd-2026-05-20"
./bin/oya secrets get --tenant oyatie.community.dev-sample --name db-password --version 1   # old version still readable
./bin/oya secrets get --tenant oyatie.community.dev-sample --name db-password --version 2   # new
```

Walk the audit chain:
```bash
./bin/oya audit query --tenant oyatie.community.dev-sample --resource secrets/db-password
```

## Day 3 — first-party types

Read `crates/cloud-secrets-domain/src/secret_kind.rs`. The first-party types are:
1. `DbPassword` (Postgres, MySQL, MariaDB, CockroachDB, Spanner)
2. `OauthRefreshToken` (Google, GitHub, GitLab, Microsoft Entra, Okta, Auth0)
3. `X509Certificate` (chain + private key)
4. `SshHostKey`
5. `SshUserKey`
6. `JwtSigningKey` (Ed25519, ES256, RS256)
7. `MlsPackageKey` (RFC 9420 leaf node init + signature keys)
8. `LlmProviderApiKey` (OpenAI, Anthropic, Google AI, Mistral, Together AI, Replicate, OpenRouter, Bedrock, Vertex)
9. `S3CompatibleAccessKey` (AWS S3, GCS HMAC, R2, B2, MinIO)
10. `WebhookHmacKey`
11. `EncryptionAtRestDek`
12. `EncryptionAtRestKek`
13. `IamRoleAssumption` (AWS, GCP, Azure)
14. `BlobAccountKey` (Azure Blob)
15. `KafkaSaslCredential`
16. `ValkeyAuthString`
17. Custom-typed extension via `secret_kind = "custom:<name>"` (no first-party rotation logic)

Each first-party type has a dedicated rotation handler under `crates/cloud-secrets-rotators-*`.

## Day 4 — author a rotator

Pick a starter type from `microservices/cloud-secrets/backlog/starter-rotators.md`. Implement under
`crates/cloud-secrets-rotator-<name>/`:

```rust
use cloud_secrets_rotator::prelude::*;

#[derive(Rotator)]
#[rotator(
    secret_kind = "DbPassword/Postgres",
    rotation_cadence_default = "30d",
    rotation_cadence_max = "90d",
    overlap_window = "2h"
)]
pub struct PostgresPasswordRotator;

impl PostgresPasswordRotator {
    async fn rotate(&self, ctx: RotateCtx) -> RotateResult {
        let new_pw = ctx.generate_password(32);
        ctx.exec_admin_sql(
            "ALTER USER $user PASSWORD $new_pw",
            &[("user", &ctx.metadata.user), ("new_pw", &new_pw)],
        ).await?;
        Ok(RotateResult::ok(new_pw))
    }
}
```

Add hermetic tests against a containerized Postgres. Sign and ship through Foundry.

## Day 5 — encryption-key BYOK ceremony rehearsal (ADR-0251 §D-10)

Run the key-ceremony simulation in dev:
```bash
./bin/oya secrets byok-rehearsal \
  --tenant oyatie.b2b.smb.acme-software \
  --hsm-profile aws-cloudhsm-mock \
  --participants "alice@oyatie,bob@acme,carol@oyatie"
```

The simulator walks the 4-quorum ceremony (2-of-3 Oyatie + 2-of-3 customer), prints the choreography, and produces a signed
attestation. Read the ceremony script `microservices/cloud-secrets/runbooks/byok-ceremony.md`.

## Done with week 1

- [ ] Walked secret lifecycle (put/get/rotate/lease) end-to-end.
- [ ] Recited the 16 first-party secret types and named the corresponding rotator crate.
- [ ] Authored, signed, and merged a rotator through Foundry.
- [ ] Ran the encryption-key BYOK ceremony rehearsal (ADR-0251 §D-10) and read the runbook.
- [ ] Read ADR-0219 + ADR-0251 + ADR-0255 §D-4 + RFC 9180.

## Rookie traps

1. **Logging cleartext.** Any log line with a value field that could contain secret material trips `lean-a4-secret-cleartext` CI.
2. **Skipping the overlap window.** Rotation without ≥ 2 × lease TTL overlap causes downstream auth failures.
3. **Using software keys at paid tenant_class.** Cedar refuses; you must use HSM-backed KEK.
4. **Bypassing the rotator framework.** "Just one quick rotation script" leaks into audit and breaks compliance evidence.
