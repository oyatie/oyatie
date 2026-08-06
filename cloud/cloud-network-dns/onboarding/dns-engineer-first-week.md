# DNS Engineer — First Week on `cloud-network-dns`

Audience: a DNS/SRE engineer with Route 53 + NS1 + Cloudflare + BIND/PowerDNS experience joining the
`oya-cloud-network-dns-*` lane. Goal: by Friday EOD you can provision a tenant zone, enable DNSSEC, configure geo-routing,
set up health-checks, and walk a DoH/DoQ query.

## Day 1 — read before touching

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — anycast per cell.
- `docs/decisions/ADR-0253-http3-quic-default-protocol.md` — DoH/3 + DoQ default.
- RFCs 4033/4034/4035 (DNSSEC), RFC 8484 (DoH), RFC 7858 (DoT), RFC 9250 (DoQ), RFC 9230 (ODoH).
- `microservices/cloud-network-dns/retired tenant_class adoption artifact` — the tenant_class model.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-dns-week1 .worktrees/$USER-dns-week1
cd .worktrees/$USER-dns-week1
```

## Day 2 — bring up a loopback DNS cell

```bash
make dev-cell.up CELL=dns-loopback-1 PROFILE=cloud-network-dns-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
```

Create your first tenant zone:
```bash
./bin/oya dns zone create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --soa-mname ns1.oyatie.dns.net \
  --soa-rname hostmaster.acme-software.io \
  --ttl-default 300
```

Expected:
```
zone_id           : zone-2026-05-20-...
zone              : acme-software.io
soa.serial        : 2026052001
ns_records        : ns1.oyatie.dns.net, ns2.oyatie.dns.net, ns3.oyatie.dns.net, ns4.oyatie.dns.net
delegation_status : not-yet-delegated (update registrar)
audit_chain_event : ce-2026-05-20T10:01:33Z-…
```

Add records:
```bash
./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name @ --type A --value 203.0.113.42 --ttl 60

./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name www --type CNAME --value acme-software.io --ttl 60

./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name @ --type MX --value "10 mail.acme-software.io" --ttl 3600
```

Test:
```bash
dig @loopback.dns.oyatie.local acme-software.io
```

You should get `203.0.113.42` back with NOERROR.

## Day 3 — DNSSEC

Enable DNSSEC on the zone:
```bash
./bin/oya dns dnssec enable \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --algorithm ECDSAP256SHA256 \
  --nsec-mode nsec3 \
  --nsec3-salt-length 8 \
  --nsec3-iterations 10
```

Expected:
```
zone           : acme-software.io
dnssec_state   : signed
ksk_keytag     : 12345 (algorithm 13 ECDSAP256SHA256, hsm-bound: no -- paid software-keys)
zsk_keytag     : 54321 (algorithm 13, rotation 30 d)
nsec_mode      : NSEC3 (salt: 4a3b2c1d, iterations: 10)
ds_record_for_registrar:
  acme-software.io. IN DS 12345 13 2 8e7f6d5c4b3a2918...
```

You manually publish the DS record at your domain registrar (we don't auto-publish; that's a registrar-API integration on the roadmap).

Verify the signed chain:
```bash
dig @loopback.dns.oyatie.local +dnssec acme-software.io
```

Expected: `flags: ad` set in the response header (Authentic Data); RRSIG records visible.

## Day 4 — geo-routing + health-checks

Add a geo-routed A record set: `203.0.113.42` for North America, `198.51.100.42` for Europe, `192.0.2.42` for APAC:
```bash
./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name api --type A --value 203.0.113.42 \
  --routing-policy geo --geo-continent NA --ttl 60 \
  --set-identifier "api-na-primary"

./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name api --type A --value 198.51.100.42 \
  --routing-policy geo --geo-continent EU --ttl 60 \
  --set-identifier "api-eu-primary"

./bin/oya dns record create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --name api --type A --value 192.0.2.42 \
  --routing-policy geo --geo-continent AS --ttl 60 \
  --set-identifier "api-apac-primary"
```

Attach health-checks:
```bash
./bin/oya dns health-check create \
  --tenant oyatie.b2b.smb.acme-software \
  --name api-na-health \
  --target https://203.0.113.42/healthz \
  --interval 30s --failure-threshold 3 --success-threshold 1 \
  --attach-to-record-set "api-na-primary"
```

Simulate a failure (dev profile mock):
```bash
./bin/oya dns health-check simulate-failure \
  --tenant oyatie.b2b.smb.acme-software \
  --health-check api-na-health \
  --duration 120s
```

Query as if you're a NA client:
```bash
./bin/oya dns query-from \
  --resolver-location NA \
  --name api.acme-software.io --type A
```

Expected: `198.51.100.42` (failover to EU because NA target is unhealthy).

## Day 5 — DoH/3 + DoQ

Test DoH over HTTP/3:
```bash
curl --http3-only \
     -H "Accept: application/dns-message" \
     "https://loopback.dns.oyatie.local/dns-query?dns=$(echo -n 'acme-software.io. IN A' | base64url | tr -d =)"
```

Test DoQ (DNS over QUIC):
```bash
./bin/oya dns query-doq \
  --resolver loopback.dns.oyatie.local:853 \
  --name acme-software.io --type A
```

Test ODoH (oblivious DoH — the resolver doesn't see your IP):
```bash
./bin/oya dns query-odoh \
  --proxy https://odoh-proxy.oyatie.local \
  --target https://loopback.dns.oyatie.local/dns-query \
  --name acme-software.io --type A
```

## What "done with week 1" means

- [ ] You can recite the four tenant_classes and which routing primitives each unlocks.
- [ ] You created a zone + records and resolved them.
- [ ] You enabled DNSSEC with NSEC3 and verified the AD bit.
- [ ] You configured geo-routing + a health-check + saw failover happen.
- [ ] You queried via DoH/3, DoT, and DoQ — and you understand the privacy properties of ODoH.
- [ ] You read ADR-0248 + ADR-0253 + RFCs 4033/4034/4035/8484/9250/9230.

## Rookie traps

1. **Forgetting to publish the DS record.** DNSSEC isn't operational without a DS at the parent zone (registrar). A `dig +dnssec`
   on a non-DS-published zone returns AD bit OFF and SERVFAIL from validating resolvers.
2. **Stale TTLs during failover.** Long TTLs (e.g. 3600 s) defeat health-check failover. For health-checked records, set TTL ≤ 60 s.
3. **Wildcard records + DNSSEC.** Wildcard records with NSEC3 require careful zone-walking protection (opt-out vs opt-in).
   Default is opt-out (matches BIND).
4. **CAA records.** Forgetting a CAA record allows any CA to issue certs for your domain; always create CAA for production.
5. **Mixing latency + geo policies.** Latency and geo are different routing modes; combining them requires explicit weight rules.
6. **Bypassing DoH/DoT.** Sensitive workloads should refuse plain port-53; paid enforces; paid/paid should opt-in via
   `cloud_network_dns::Action::RequireEncryptedTransport`.
