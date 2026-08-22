# Tutorial — Issue, rotate, and revoke a dynamic Postgres credential

Goal: walk a full dynamic-credential lifecycle. Issue a 15-minute Postgres user, use it to query, rotate it, observe overlap,
and revoke. End-to-end on a loopback cell with a real containerized Postgres.

Pre-reqs:
- Loopback cell: `make dev-cell.up CELL=secrets-loopback-1 PROFILE=cloud-secrets-dev`
- Postgres container started by the dev-cell profile (port 5439 on localhost)
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`
- A `secrets/db-admin/postgres-prod-primary` admin credential pre-loaded (cell startup script does this)

## Step 1 — register the backend

`cloud-secrets` needs to know how to talk to Postgres. Register the backend:
```bash
./bin/oya secrets backend register \
  --tenant oyatie.b2b.smb.acme-software \
  --kind postgres \
  --name postgres-prod-primary \
  --connection-uri "postgres://oyatie_admin@localhost:5439/acme_prod?sslmode=require" \
  --admin-secret secrets/db-admin/postgres-prod-primary
```

Expected:
```
backend_id: be-2026-05-20-postgres-prod-primary
status   : Registered
```

## Step 2 — author a role template

The role template describes what kind of dynamic user to mint:
```bash
./bin/oya secrets role create \
  --tenant oyatie.b2b.smb.acme-software \
  --backend postgres-prod-primary \
  --name app-readonly \
  --statements 'CREATE USER "{{name}}" WITH PASSWORD '"'"'{{password}}'"'"' VALID UNTIL '"'"'{{expiration}}'"'"';
                GRANT CONNECT ON DATABASE acme_prod TO "{{name}}";
                GRANT USAGE ON SCHEMA public TO "{{name}}";
                GRANT SELECT ON ALL TABLES IN SCHEMA public TO "{{name}}";' \
  --revoke-statements 'REVOKE ALL PRIVILEGES ON DATABASE acme_prod FROM "{{name}}";
                       DROP USER IF EXISTS "{{name}}";' \
  --default-ttl 15m --max-ttl 1h
```

Expected:
```
role_id: ro-2026-05-20-app-readonly
status : Active
```

## Step 3 — issue a dynamic credential

```bash
./bin/oya secrets dynamic-issue \
  --tenant oyatie.b2b.smb.acme-software \
  --role app-readonly \
  --ttl 15m
```

Expected:
```json
{ "lease_id": "le-2026-05-20T08:31:00.214Z-fa3c…",
  "user": "dyn_4f8c2b7a9e1d",
  "password": "k7m@2vQ9pX!nL6tR3wY8eF5sB0",
  "expires_at": "2026-05-20T08:46:00.214Z",
  "fencing_token": "ft-blake3-256:0a2b…" }
```

> The password appears once. The CLI marks the line `--SENSITIVE--` and your shell history hook redacts it.

## Step 4 — use the credential to query Postgres

```bash
psql "postgres://dyn_4f8c2b7a9e1d:k7m@2vQ9pX!nL6tR3wY8eF5sB0@localhost:5439/acme_prod?sslmode=require" \
  -c "SELECT count(*) FROM orders WHERE created_at > now() - interval '24 hours';"
```

You should get a row count. If you try to INSERT:
```sql
INSERT INTO orders DEFAULT VALUES;
-- ERROR:  permission denied for table orders
```

## Step 5 — rotate the underlying admin secret

The admin secret backs all dynamic issuance. Rotate it:
```bash
./bin/oya secrets rotate \
  --tenant oyatie.b2b.smb.acme-software \
  --name db-admin/postgres-prod-primary
```

Expected:
```
old_version: 1 -- still readable for 30m (overlap window)
new_version: 2 -- active for issuance
audit_event: ce-2026-05-20T08:34:11.318Z-…
```

During the overlap window, both versions are accepted by Postgres for admin operations; new dynamic users from this point use the
new admin credential. Pre-existing dynamic users (your `dyn_4f8c2b7a9e1d`) are unaffected — they have their own passwords.

## Step 6 — observe lease expiry

Wait until `expires_at`. After expiry, attempting the query:
```bash
psql "postgres://dyn_4f8c2b7a9e1d:…@localhost:5439/acme_prod"
# FATAL:  password authentication failed
```

`cloud-secrets` ran the revoke statements at TTL expiry; the user is gone.

## Step 7 — explicit revoke before expiry

Issue another lease:
```bash
LEASE=$(./bin/oya secrets dynamic-issue --tenant oyatie.b2b.smb.acme-software --role app-readonly --ttl 1h --json | jq -r .lease_id)
```

Revoke immediately:
```bash
./bin/oya secrets revoke --tenant oyatie.b2b.smb.acme-software --lease-id $LEASE
```

Expected:
```
lease_id      : le-…
status        : Revoked
revoke_latency: 0.5 s (Postgres DROP USER)
audit_event   : ce-2026-…
```

## Step 8 — verify audit chain

```bash
./bin/oya audit query \
  --tenant oyatie.b2b.smb.acme-software \
  --resource "secrets/dynamic/postgres-prod-primary/app-readonly" \
  --window 1h
```

You should see issuance events, the rotation event, the expiry event, the second issuance, and the revocation event — all
chain-linked by BLAKE3-256 hashes.

## What you proved

- Dynamic credentials mint real Postgres users on demand and clean up at expiry.
- Admin credential rotation is non-disruptive (overlap window).
- Explicit revoke beats TTL when needed (e.g. compromised credential).
- The audit chain captures every lifecycle event linkably.
