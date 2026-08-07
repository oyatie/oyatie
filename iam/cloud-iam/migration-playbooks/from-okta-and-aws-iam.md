# Migration playbook — Okta Workforce + AWS IAM → Oyatie `cloud-iam`

Audience: an IAM/identity team running Okta Workforce as the primary IdP and AWS IAM (with or without IAM Identity Center) for
cloud authorisation. Goal: migrate identity + authorisation to `cloud-iam` with zero seat loss and no user-visible re-enrollment.

## Phase 0 — Inventory (Day 0…5)

### From Okta

1. Export users + groups:
   ```bash
   okta-cli users list --filter 'status eq "ACTIVE"' --format json > okta-users.json
   okta-cli groups list --format json > okta-groups.json
   ```
2. Export SAML/OIDC apps:
   ```bash
   okta-cli apps list --filter 'signOnMode eq "SAML_2_0" or signOnMode eq "OPENID_CONNECT"' --format json > okta-apps.json
   ```
3. Export Okta Policies (Sign-On, MFA, Password):
   ```bash
   okta-cli policies list --type ACCESS_POLICY > okta-policies.json
   ```

### From AWS IAM

1. Export users + groups + roles:
   ```bash
   aws iam list-users > aws-users.json
   aws iam list-groups > aws-groups.json
   aws iam list-roles > aws-roles.json
   ```
2. For each role, export the trust policy + attached policies:
   ```bash
   jq -r '.Roles[].RoleName' aws-roles.json | while read r; do
     aws iam get-role --role-name "$r" > "role-$r.json"
     aws iam list-attached-role-policies --role-name "$r" > "role-attached-$r.json"
     aws iam list-role-policies --role-name "$r" > "role-inline-$r.json"
   done
   ```
3. Note SCPs at the Organization root (if any).

## Phase 1 — IdP registration in cloud-iam (Day 5…7)

Register Okta as a federated IdP:
```bash
./bin/oya iam idp register \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --kind saml-2.0 \
  --name okta-workforce-acme \
  --metadata-url https://acme.okta.com/app/exk.../sso/saml/metadata \
  --jit-rules-file jit-rules-acme.yaml
```

`jit-rules-acme.yaml` maps Okta groups → Cedar roles. Generate it from `okta-groups.json`:
```bash
./bin/oya iam migrate okta-groups-to-jit-rules \
  --input okta-groups.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output jit-rules-acme.yaml
```

## Phase 2 — Cedar policy authoring (Day 7…21)

For each Okta App Policy + AWS IAM role, author a Cedar policy. Use the translator in reverse-discovery mode:
```bash
./bin/oya iam translate \
  --source-format aws-iam-json \
  --source-file role-attached-OrgAdmin.json \
  --target cedar \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output policies/acme/translated/org-admin.cedar
```

This is **lossy** — the translator emits a `_translation_note` listing IAM features that didn't carry over (resource wildcards
beyond Cedar's slice expressions, IAM Condition operators with no Cedar equivalent). Manual rewrite to clean Cedar is mandatory
before pushing to production.

Lint:
```bash
./bin/oya policy lint --tenant oyatie.b2b.midmarket.acme-corp policies/acme/
```

## Phase 3 — Dual-IdP shadow phase (Day 21…42)

Keep Okta live as the user-facing IdP, but route every login through `cloud-iam`'s SAML proxy. Configure Okta to post the SAML
assertion to `cloud-iam` (instead of directly to the SP). `cloud-iam` then re-issues a downstream assertion and forwards to the SP.
This lets you:
1. Materialise principals in `cloud-iam` (silent JIT) without changing the end-user flow.
2. Run **shadow authorisation** — for every user action, evaluate Cedar in `cloud-iam` AND let the existing system make the
   real decision; compare for divergence.
3. Catch missing or mis-translated policies before hard cut-over.

Run divergence check daily:
```bash
./bin/oya iam migrate divergence-report \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --since "24h ago" \
  --threshold 0.01   # 1% divergence triggers alert
```

## Phase 4 — Cedar → AWS IAM translation push (Day 42…56)

For policies that govern AWS API calls, translate Cedar back to AWS IAM JSON and push to the tenant's AWS account:
```bash
./bin/oya iam translate \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --source-policy policies/acme/aws-org-admin.cedar \
  --target aws-iam-json \
  --output translated/aws-org-admin.iam.json

./bin/oya iam push aws \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --aws-account 123456789012 \
  --policy-file translated/aws-org-admin.iam.json \
  --role-name OyatieOrgAdmin
```

The AWS IAM policy now carries the `_oya_cedar_digest` annotation pointing back to Cedar.

## Phase 5 — Hard cut-over (Day 56…70)

1. Switch Okta apps from "post to SP" to "post to `cloud-iam`" (irreversible-ish — see rollback below).
2. Disable AWS IAM users (delete static keys); switch every AWS call to assume `cloud-iam`-brokered STS.
3. Decommission Okta Policies that overlap Cedar's authority — keep Okta Sign-On + MFA policies (Okta still owns the front door),
   but retire authz-style Okta Policies.

## Phase 6 — Decommission overlaps (Day 70+)

After 30 d clean run on `cloud-iam`:
- Delete unused AWS IAM users (`aws iam delete-user`).
- Delete legacy AWS IAM roles that have been re-issued under `cloud-iam`.
- Archive Okta Policies that `cloud-iam` now owns.
- (Optional) move Okta to "IdP-only" tenant_class policy license to save cost.

## Rollback strategy

Within the 30-day window:
1. Re-enable AWS IAM users + roles (they still exist; just delete the deny-all overlay).
2. Switch Okta apps back to "post to SP".
3. Disable `cloud-iam` Cedar policy push to AWS IAM.

After 30 d: rollback requires reconstituting the AWS IAM roles from the version archive `cloud-iam` keeps (last 256 versions).
Plan on 8-16 h for a paid tenant_class tenant with 5,000 roles.

## What you gain

- 40-130× lower authz latency (190 µs vs 7-25 ms).
- 16 % TCO reduction vs AWS IdC + IAM + Okta combo at mid-market scale.
- Cedar as single source of truth — IAM JSON drift-detected.
- BLAKE3 audit chain (tamper-evident); 7y retention at paid tenant_class.
- Reviewer-agent gating on every policy push.
- Per-tenant compliance pack overlays.
- SPIFFE/SPIRE workload identity unified with human identity.

## What you give up

- Okta admin console UX maturity (Oyatie admin is `workflow-studio`-based).
- Okta marketplace IdP catalog breadth (600 vs 7,000+).
- AWS IAM JSON as authoritative — Cedar is authority; IAM is a downstream artefact.
- Mobile SDK breadth (Swift/Kotlin only at v1; Auth0 has Flutter/RN).
