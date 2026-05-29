# Tutorial — Provision a zone, enable DNSSEC, configure geo-routing + health-checks, query via DoH/3 + DoQ

Goal: end-to-end tenant zone with DNSSEC chain-of-trust, geo-routed apex with multi-region health-checks, query via every
supported transport. Loopback `cloud-network-dns` cell.

Pre-reqs:
- Loopback DNS cell: `make dev-cell.up CELL=dns-loopback-1 PROFILE=cloud-network-dns-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`
- `dig`, `curl --http3`, `kdig` (Knot DNS utilities) on PATH.

## Step 1 — provision the zone

```bash
./bin/oya dns zone create \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --soa-mname ns1.oyatie.dns.net \
  --soa-rname hostmaster.acme-software.io \
  --ttl-default 60 \
  --refresh 3600 --retry 600 --expire 604800 --minimum 60
```

Expected:
```
zone_id            : zone-2026-05-20-...
ns_records         : ns1.oyatie.dns.net, ns2.oyatie.dns.net, ns3.oyatie.dns.net, ns4.oyatie.dns.net
delegation_status  : not-yet-delegated (update registrar to point at our NS)
audit_chain_event  : ce-2026-05-20T10:01:33Z-…
```

## Step 2 — add core records

```bash
./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name @ --type CAA --value '0 issue "letsencrypt.org;account=acme-software"' --ttl 86400

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name @ --type CAA --value '0 issue "digicert.com;account=acme-software-ev"' --ttl 86400

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name @ --type CAA --value '0 iodef "mailto:security@acme-software.io"' --ttl 86400

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name www --type CNAME --value acme-software.io --ttl 60

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name @ --type MX --value "10 mail.acme-software.io" --ttl 3600

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name @ --type TXT --value '"v=spf1 mx -all"' --ttl 3600

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name _dmarc --type TXT --value '"v=DMARC1; p=quarantine; rua=mailto:dmarc-reports@acme-software.io"' --ttl 3600
```

## Step 3 — enable DNSSEC (NSEC3 + ECDSA-P256)

```bash
./bin/oya dns dnssec enable \
  --tenant oyatie.b2b.smb.acme-software \
  --zone acme-software.io \
  --algorithm ECDSAP256SHA256 \
  --nsec-mode nsec3 \
  --nsec3-salt-length 8 \
  --nsec3-iterations 10 \
  --ksk-rotation-cadence 180d \
  --zsk-rotation-cadence 30d
```

Expected:
```
dnssec_state   : signed
ksk_keytag     : 19273  (alg 13; software-keys on paid; HSM-bound at paid)
zsk_keytag     : 47102
nsec3_params   : 1 0 10 4a3b2c1d8e7f6a5b
ds_record_for_registrar:
  acme-software.io. IN DS 19273 13 2 7c8d9e0f1a2b3c4d...
```

Publish the DS at your registrar (manual step; we don't auto-publish to the registrar's API yet).

Verify:
```bash
dig @loopback.dns.oyatie.local +dnssec acme-software.io SOA
```

Look for `flags: qr aa rd ra ad`. The `ad` bit confirms DNSSEC validated.

## Step 4 — geo-routed apex with health-checks

Add three regional A records:
```bash
./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name api --type A --value 203.0.113.42 \
  --routing-policy geo --geo-continent NA --ttl 60 --set-identifier "api-na-primary"

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name api --type A --value 198.51.100.42 \
  --routing-policy geo --geo-continent EU --ttl 60 --set-identifier "api-eu-primary"

./bin/oya dns record create --tenant oyatie.b2b.smb.acme-software --zone acme-software.io \
  --name api --type A --value 192.0.2.42 \
  --routing-policy geo --geo-continent AS --ttl 60 --set-identifier "api-apac-primary"
```

Create three health-checks:
```bash
for region in na eu apac; do
  case $region in
    na)   ip=203.0.113.42;;
    eu)   ip=198.51.100.42;;
    apac) ip=192.0.2.42;;
  esac
  ./bin/oya dns health-check create \
    --tenant oyatie.b2b.smb.acme-software \
    --name api-$region-health \
    --target https://$ip/healthz --expected-status 200 --expected-body-contains '"ok":true' \
    --interval 10s --failure-threshold 3 --success-threshold 1 \
    --attach-to-record-set api-$region-primary
done
```

## Step 5 — induce a failure + observe failover

```bash
./bin/oya dns health-check simulate-failure \
  --tenant oyatie.b2b.smb.acme-software \
  --health-check api-na-health \
  --duration 120s
```

Query from a NA-emulated resolver:
```bash
./bin/oya dns query-from \
  --resolver-location NA --resolver-ip 8.8.8.8 \
  --name api.acme-software.io --type A
```

Expected: failover within 30 s; returns 198.51.100.42 (EU) until NA is healthy again.

```bash
./bin/oya dns health-check status \
  --tenant oyatie.b2b.smb.acme-software \
  --health-check api-na-health
```

Expected:
```
target           : https://203.0.113.42/healthz
state            : unhealthy (since 2026-05-20T11:14:33Z, 8 consecutive failures)
last_failure_code: connection_refused
attached_records : api.acme-software.io A (set: api-na-primary; status: removed-from-rotation)
```

## Step 6 — query via every transport

**Plain DNS (UDP/53):**
```bash
dig @loopback.dns.oyatie.local acme-software.io A
```

**DoH (HTTPS/443) — HTTP/2:**
```bash
curl -s -H "Accept: application/dns-message" \
  "https://loopback.dns.oyatie.local/dns-query?name=acme-software.io&type=A"
```

**DoH/3 (HTTP/3):**
```bash
curl --http3-only -s -H "Accept: application/dns-message" \
  "https://loopback.dns.oyatie.local/dns-query?name=acme-software.io&type=A"
```

**DoT (TCP/853):**
```bash
kdig -d @loopback.dns.oyatie.local +tls-ca acme-software.io A
```

**DoQ (UDP/853 via QUIC):**
```bash
./bin/oya dns query-doq \
  --resolver loopback.dns.oyatie.local:853 \
  --name acme-software.io --type A
```

**ODoH (oblivious DoH):**
```bash
./bin/oya dns query-odoh \
  --proxy https://odoh-proxy.loopback.oyatie.local \
  --target https://loopback.dns.oyatie.local/dns-query \
  --name acme-software.io --type A
```

## Step 7 — query log

```bash
./bin/oya dns query-log query \
  --tenant oyatie.b2b.smb.acme-software \
  --since "30m ago" \
  --group-by transport,rcode
```

Expected (truncated):
```
transport     rcode    count
udp           NOERROR  142
udp           NXDOMAIN 8
doh-h2        NOERROR  56
doh-h3        NOERROR  78
dot           NOERROR  22
doq           NOERROR  19
odoh          NOERROR  14
```

## What you just demonstrated

- Multi-record-type zone with CAA, MX, TXT, SPF/DMARC, CNAME, A.
- DNSSEC chain-of-trust with NSEC3 + ECDSA-P256.
- Continent-level geo-routing with three regional answer sets.
- 10-second-interval health-checks with body-content matching + auto failover.
- Six transports (UDP, DoH-H2, DoH-H3, DoT, DoQ, ODoH) all serving the same authoritative data.
- Per-query observability via the query log.
