# Migration playbook — AWS App Runner → Oyatie `application`

Audience: a platform engineer who runs one or more services on AWS App Runner (any size) and wants to migrate to the
Oyatie `application` substrate without a customer-visible regression.

Source assumptions:
- App Runner service URL `https://<svc>.<region>.awsapprunner.com`
- IAM role attached for ECR/S3 access
- VPC connector for private downstreams
- Existing client traffic over HTTPS (HTTP/2)

Target: a tenant + tier on `application` with the same observable behavior, plus Cedar permits + pack overlays + audit-chain.

> Phase budget: 14 calendar days for a SMB tenant_class paid tenant, 35 days for a tenant_class paid tenant, 60 days for compliance_pack-bound paid with regulated packs.

## Phase 0 — Inventory (Day 0…2)

1. Export App Runner config:
   ```bash
   aws apprunner describe-service --service-arn $SVC_ARN > snapshot.json
   ```
   Keep `snapshot.json` as your source of truth.
2. Enumerate routes:
   ```bash
   aws apprunner list-operations --service-arn $SVC_ARN
   ```
   For each route, write down its intent kind in Oyatie terms (likely a `application::Intent::*` variant or a candidate for a new one
   if no existing variant fits — see FAQ Q3 for the ADR amendment workflow).
3. Capture inbound concurrency profile from CloudWatch:
   - `RequestCount` per minute (p95)
   - `ActiveInstances` (max in last 30 d)
   - `InstanceMemoryUsage` (p95)

## Phase 1 — Tenant provisioning (Day 2…4)

1. File a tenant creation ticket:
   ```bash
   ./bin/oya tenant create \
     --id oyatie.b2b.smb.<your-org>.app-runner-migration \
     --tier tenant_class paid \
     --region us-east-2 \
     --pack-set "" \
     --provider-credential-mode platform_default
   ```
   For tenant_class paid tenants add `--tier tenant_class paid` and the relevant packs (e.g. `--pack-set "soc2-type-ii-v2017,gdpr-eu-v2018"`).
2. Wait for the tenancy reviewer-agent to APPROVE; ticket auto-creates the cell admittances.

## Phase 2 — Cedar permit authoring (Day 4…7)

For each App Runner route, write the corresponding Cedar permit:
```cedar
permit (
  principal in oyatie.b2b.smb.your-org.app-runner-migration::User,
  action == application::Action::Dispatch,
  resource == application::Intent::CreateWorkspace
);
```

If a route has IAM-conditional access (e.g. only certain customer accounts), translate that to a Cedar `when` clause:
```cedar
permit ( ... )
when {
  resource.requestor in oyatie.b2b.smb.your-org.app-runner-migration::Group::"early-access"
};
```

Validate with the policy linter:
```bash
./bin/oya policy lint --tenant oyatie.b2b.smb.your-org.app-runner-migration policies/
```

## Phase 3 — Dual-run (Day 7…21)

1. Stand up the `application` cell:
   ```bash
   ./bin/oya tenant admit \
     --tenant oyatie.b2b.smb.your-org.app-runner-migration \
     --cell tenant_class paid-us-east-2-a \
     --cell tenant_class paid-us-east-2-b
   ```
2. Configure a 5 % traffic shadow from App Runner to `application` via a Cloudflare Worker that copies inbound requests:
   ```js
   // worker.js
   addEventListener('fetch', e => {
     const req = e.request;
     e.waitUntil(fetch('https://gateway.oyatie.io/v1/dispatch', {
       method: req.method,
       headers: copyHeaders(req, ['x-oyatie-tenant: oyatie.b2b.smb.your-org.app-runner-migration']),
       body: req.body
     }));
     e.respondWith(fetch(req)); // still hit App Runner
   });
   ```
3. Compare outcomes daily:
   ```bash
   ./bin/oya migrate compare \
     --source app-runner \
     --target application \
     --tenant oyatie.b2b.smb.your-org.app-runner-migration \
     --window 24h
   ```
   Goal: 99.95 % outcome parity, p95 latency parity within 15 ms.

## Phase 4 — Cut-over (Day 21…28)

1. Flip Cloudflare to 100 % to `application`.
2. Keep App Runner running cold for 7 days as rollback.
3. Watch `application.dispatch.outcome` SLO dashboard.
4. After 7 days clean, decommission App Runner:
   ```bash
   aws apprunner delete-service --service-arn $SVC_ARN
   ```

## Phase 5 — Hardening (Day 28+)

- Tighten Cedar policies (remove any catch-all `permit (principal, action, resource)`).
- Enable per-pack overlays if regulatory packs apply.
- Move to BYOK provider credentials if you need vendor lock-out (`feedback_byok_everywhere_credentials.md`).
- Subscribe the on-call rotation to `application.degrade.mode` alerts.

## Rollback

Within the 7-day dual-run, flip Cloudflare back to App Runner:
```bash
./bin/oya migrate rollback \
  --tenant oyatie.b2b.smb.your-org.app-runner-migration \
  --to source
```

After App Runner is decommissioned, rollback requires standing up a fresh App Runner service from the `snapshot.json` you saved in Phase 0.
Estimated rollback time: 2 h (snapshot replay) + 1 h (DNS propagation).

## What you gain by migrating

- Cold-start: 6.2 s → 0 ms (warm pool by default).
- Multi-tenant primitive: from "do it yourself" to first-class header + audit chain.
- ABAC: from IAM round-trip (5-15 ms) to in-process Cedar (200 µs).
- Compliance packs: from "buy SOC2 tooling" to overlay flip.
- Cost: ~12 % lower TCO at mid-market scale (see benchmarks).

## What you give up

- "Run any container in 5 min" — `application` requires you to be a tenant on Oyatie.
- Per-second billing granularity (monthly tier billing today; per-second add-on roadmap).
- Direct CloudWatch metric ingestion (use `observability` µservice and the OTLP bridge).
