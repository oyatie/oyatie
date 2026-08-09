# Tutorial — Provision a VPC, configure mTLS ingress, write a Cedar L7 policy, walk flow logs

Goal: end-to-end multi-AZ tenant VPC, deploy two services, lock down east-west traffic with a Cedar policy that requires SPIFFE
SVIDs + port 5432, verify allow + deny via flow logs. Loopback `cloud-network` cell, no real cloud account needed.

Pre-reqs:
- Loopback network cell: `make dev-cell.up CELL=network-loopback-1 PROFILE=cloud-network-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`
- `kubectl` + `curl --http3` (curl 8.x with QUIC) on PATH.

## Step 1 — provision a VPC

```bash
./bin/oya network vpc create \
  --tenant oyatie.b2b.smb.acme-software \
  --region loopback-us-east-2 \
  --cidr 10.214.0.0/22 \
  --availability-zones 3 \
  --enable-ipv6 true
```

Expected:
```
vpc_id           : vpc-2026-05-20-...
cidr             : 10.214.0.0/22 + 2400:...::/56
availability_zones: ["loop-az-a","loop-az-b","loop-az-c"]
subnets:
  - public  loop-az-a 10.214.0.0/26
  - private loop-az-a 10.214.0.64/24
  - public  loop-az-b 10.214.1.0/26
  - private loop-az-b 10.214.1.64/24
  - public  loop-az-c 10.214.2.0/26
  - private loop-az-c 10.214.2.64/24
audit_chain_event: ce-2026-05-20T10:01:33Z-…
```

## Step 2 — deploy two services

`webapp.yaml`:
```yaml
apiVersion: oya.network/v1
kind: TenantService
metadata:
  name: webapp
  tenant: oyatie.b2b.smb.acme-software
spec:
  image: ghcr.io/oya-samples/echo:0.4.1
  replicas: 2
  ports:
    - name: http
      port: 8080
  spiffe_id: spiffe://oyatie.b2b.smb.acme-software/webapp
```

`pgproxy.yaml`:
```yaml
apiVersion: oya.network/v1
kind: TenantService
metadata:
  name: pgproxy
  tenant: oyatie.b2b.smb.acme-software
spec:
  image: ghcr.io/oya-samples/pgproxy:0.2.0
  replicas: 1
  ports:
    - name: postgres
      port: 5432
  spiffe_id: spiffe://oyatie.b2b.smb.acme-software/pgproxy
```

Apply:
```bash
./bin/oya network service apply --tenant oyatie.b2b.smb.acme-software --file webapp.yaml
./bin/oya network service apply --tenant oyatie.b2b.smb.acme-software --file pgproxy.yaml
```

## Step 3 — public ingress with mTLS for `webapp`

```bash
./bin/oya network ingress create \
  --tenant oyatie.b2b.smb.acme-software \
  --service webapp \
  --hostname acme-webapp.loopback.oyatie.local \
  --tls-mode mtls \
  --client-ca-source spiffe://oyatie.b2b.smb.acme-software/clients \
  --alpn-default h3
```

Hit it (the loopback dev cell mints a test client SVID and trust bundle for you):
```bash
SVID_CERT=$(./bin/oya network test-svid --tenant oyatie.b2b.smb.acme-software --client-id alice-laptop)

curl --cert $SVID_CERT/cert.pem --key $SVID_CERT/key.pem \
     --cacert $SVID_CERT/trust-bundle.pem \
     --http3-only \
     https://acme-webapp.loopback.oyatie.local/
```

Expected:
```
HTTP/3 200
content-type: text/plain
x-oyatie-tenant-id: oyatie.b2b.smb.acme-software
x-oyatie-cell-id: network-loopback-1
x-oyatie-svid-pod: spiffe://oyatie.b2b.smb.acme-software/webapp

echo: hello from webapp-7d4f-abc1
```

Without the client cert:
```bash
curl --http3-only https://acme-webapp.loopback.oyatie.local/
```
Expected: `HTTP/3 403 ssl_client_cert_required`.

## Step 4 — author a Cedar L7 network policy

`policies/acme-software/webapp-to-pgproxy.cedar`:
```cedar
permit (
  principal in Workload::"oyatie.b2b.smb.acme-software/webapp",
  action == cloud_network::Action::EstablishFlow,
  resource == Service::"oyatie.b2b.smb.acme-software/pgproxy"
)
when {
  context.flow.destination_port == 5432 &&
  context.flow.protocol == "tcp" &&
  context.session.spiffe_svid_valid == true &&
  context.session.svid_pod_in_namespace == "oyatie.b2b.smb.acme-software"
};
```

Lint + push:
```bash
./bin/oya policy lint --tenant oyatie.b2b.smb.acme-software policies/acme-software/webapp-to-pgproxy.cedar
./bin/oya network policy push --tenant oyatie.b2b.smb.acme-software --policy-file policies/acme-software/webapp-to-pgproxy.cedar
```

## Step 5 — generate traffic + verify

Trigger some webapp → pgproxy traffic (the echo container probes downstream on /probe-db):
```bash
for i in 1 2 3 4 5; do
  curl --cert $SVID_CERT/cert.pem --key $SVID_CERT/key.pem \
       --cacert $SVID_CERT/trust-bundle.pem \
       --http3-only \
       https://acme-webapp.loopback.oyatie.local/probe-db
done
```

Try a forbidden flow (webapp → external port 5432 — pretend we're trying to exfiltrate):
```bash
curl --cert $SVID_CERT/cert.pem --key $SVID_CERT/key.pem \
     --cacert $SVID_CERT/trust-bundle.pem \
     --http3-only \
     "https://acme-webapp.loopback.oyatie.local/probe-db?host=ext-pg.evil.example.com"
```

The webapp container will try to connect; Cilium L7 enforces the Cedar policy and drops the flow.

## Step 6 — query flow logs

```bash
./bin/oya network flow-log query \
  --tenant oyatie.b2b.smb.acme-software \
  --service webapp \
  --since "10m ago"
```

Expected (truncated):
```
flow_id  src                     dst                                  port  proto bytes verdict
fl-001   webapp-7d4f-abc1        pgproxy-7c3a-def1                    5432  tcp   18KB  allow
fl-002   webapp-7d4f-abc1        pgproxy-7c3a-def1                    5432  tcp   16KB  allow
fl-003   webapp-7d4f-abc2        pgproxy-7c3a-def1                    5432  tcp   17KB  allow
...
fl-099   webapp-7d4f-abc1        ext-pg.evil.example.com:5432         5432  tcp   0     deny
```

The `deny` flows are the canonical signal — read the Cedar decision log:
```bash
./bin/oya network policy-decision-log query \
  --tenant oyatie.b2b.smb.acme-software \
  --verdict deny \
  --since "10m ago"
```

Expected:
```
ts                          flow_id  principal                                       reason
2026-05-20T10:09:11.214Z    fl-099   Workload::"acme-software/webapp"                no permit matched: destination not in Service set for tenant
```

## Step 7 — examine the BGP flow (paid hybrid)

Establish + verify the hybrid loopback BGP peer (optional but illuminating):
```bash
./bin/oya network hybrid-connection create \
  --tenant oyatie.b2b.smb.acme-software \
  --name acme-prod-dx \
  --type direct-connect-simulated \
  --bgp-asn 65000 --bgp-customer-asn 65500 \
  --customer-side-cidr 192.168.0.0/16 --vlan 100

./bin/oya network bgp peer-up --tenant oyatie.b2b.smb.acme-software --name acme-prod-dx
./bin/oya network bgp routes --tenant oyatie.b2b.smb.acme-software --name acme-prod-dx
```

## What you just demonstrated

- Multi-AZ tenant VPC with deterministic CIDR and IPv6 dual-stack.
- mTLS ingress with HTTP/3-default ALPN, SPIFFE client cert verification.
- Two-service tenant deployment with SPIFFE SVID identities.
- Cedar L7 policy translated into Cilium policy in the data path.
- Allow + deny verdicts at packet rate, with full Cedar decision log.
- Optional simulated BGP peering for hybrid connectivity.
