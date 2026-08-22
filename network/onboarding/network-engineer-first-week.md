# Network Engineer — First Week on `cloud-network`

Audience: a network/SRE engineer with AWS VPC + Cilium + Envoy + BGP experience joining the `cloud-network-*` lane.
Goal: by Friday EOD you can provision a tenant VPC, configure mTLS termination, establish a hybrid BGP peer, and walk a flow-log
investigation.

## Day 1 — read before touching

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — cells are the unit of blast radius; network is per-cell.
- `docs/adr-archive/ADR-0253-network-topology-edge-service-mesh.md` — HTTP/3 mandatory default.
- `docs/adr-archive/ADR-0254-deployment-model-spectrum.md` — K8s + Cloud Hypervisor; Cilium is the CNI.
- `microservices/cloud-network/retired tenant_class adoption artifact` — the tenant_class model.
- Cilium v1.18 docs + RFC 9000 (QUIC) + RFC 9114 (HTTP/3) skim.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-network-week1 .worktrees/$USER-network-week1
cd .worktrees/$USER-network-week1
```

## Day 2 — bring up a loopback network cell

```bash
make dev-cell.up CELL=network-loopback-1 PROFILE=cloud-network-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
```

Provision a tenant VPC:
```bash
./bin/oya network vpc create \
  --tenant oyatie.b2b.smb.acme-software \
  --region us-east-2 \
  --cidr-suggest auto \
  --availability-zones 3
```

Expected output:
```
vpc_id           : vpc-2026-05-20-...
provider         : loopback (Cilium-only dev cell)
cidr             : 10.214.0.0/22
availability_zones: ["loop-az-a", "loop-az-b", "loop-az-c"]
subnets:
  - loop-az-a public  10.214.0.0/26  (16 IPs)
  - loop-az-a private 10.214.0.64/24 (256 IPs)
  - loop-az-b public  10.214.1.0/26
  - loop-az-b private 10.214.1.64/24
  - loop-az-c public  10.214.2.0/26
  - loop-az-c private 10.214.2.64/24
audit_chain_event: ce-2026-05-20T10:01:33Z-…
```

## Day 3 — deploy a service + mTLS termination

Deploy a tiny test service (in the cell's K8s):
```bash
./bin/oya network service deploy \
  --tenant oyatie.b2b.smb.acme-software \
  --service-name webapp \
  --image ghcr.io/samples/echo:0.4.1 \
  --replicas 2 \
  --port 8080
```

Expose it through an Envoy ingress with mTLS:
```bash
./bin/oya network ingress create \
  --tenant oyatie.b2b.smb.acme-software \
  --service webapp \
  --hostname acme-webapp.loopback.oyatie.local \
  --tls-mode mtls \
  --client-ca-source spiffe://oyatie.b2b.smb.acme-software/clients
```

Test:
```bash
curl --cert /etc/oya/svid/cert.pem --key /etc/oya/svid/key.pem \
  --cacert /etc/oya/spiffe-trust-bundle.pem \
  --http3-only \
  https://acme-webapp.loopback.oyatie.local/
```

Expected: HTTP/3 200, with response body confirming mTLS client certificate validated against the tenant SPIFFE trust bundle.
A request without the client cert returns `403 ssl_client_cert_required`.

## Day 4 — hybrid BGP peer

paid allows ≤ 1 hybrid connection. Simulate a Direct peer (the dev cell ships a quagga peer for testing):
```bash
./bin/oya network hybrid-connection create \
  --tenant oyatie.b2b.smb.acme-software \
  --name acme-prod-dx \
  --type direct-connect-simulated \
  --bgp-asn 65000 \
  --bgp-customer-asn 65500 \
  --customer-side-cidr 192.168.0.0/16 \
  --vlan 100
```

Establish peering:
```bash
./bin/oya network bgp peer-up --tenant oyatie.b2b.smb.acme-software --name acme-prod-dx
./bin/oya network bgp routes --tenant oyatie.b2b.smb.acme-software --name acme-prod-dx
```

Expected:
```
bgp_state         : Established
local_asn         : 65000
remote_asn        : 65500
prefixes_received : 12  (192.168.0.0/16 + subnets)
prefixes_advertised: 7  (10.214.0.0/22 + subnets)
session_uptime    : 4 min 23 s
```

## Day 5 — Cedar network policy + flow-log investigation

Author a Cedar policy that allows only the `webapp` service to talk to `cloud-data` on port 5432:
`policies/acme-software/webapp-to-data.cedar`:
```cedar
permit (
  principal in Workload::"oyatie.b2b.smb.acme-software/webapp",
  action == cloud_network::Action::EstablishFlow,
  resource == Service::"oyatie.b2b.smb.acme-software/cloud-data"
)
when {
  context.flow.destination_port == 5432 &&
  context.flow.protocol == "tcp" &&
  context.session.spiffe_svid_valid == true
};
```

Push:
```bash
./bin/oya network policy push --tenant oyatie.b2b.smb.acme-software --policy-file policies/acme-software/webapp-to-data.cedar
```

Generate some traffic + query flow logs:
```bash
./bin/oya network flow-log query \
  --tenant oyatie.b2b.smb.acme-software \
  --service webapp \
  --since "10m ago"
```

Expected (truncated):
```
flow_id       src_pod                 dst                          port  proto  bytes  verdict
fl-001        webapp-7d4f-abc1        cloud-data.svc               5432  tcp    2.4 MB allow
fl-002        webapp-7d4f-abc1        cloud-data.svc               5432  tcp    1.8 MB allow
fl-003        webapp-7d4f-abc2        random-tenant-b.example.io   443   tcp    0      deny (cross-tenant; no permit)
```

The `deny` entry is the canonical "tenant scope check at packet level" signal — read the Cedar policy log via:
```bash
./bin/oya network policy-decision-log query \
  --tenant oyatie.b2b.smb.acme-software \
  --since "10m ago"
```

## What "done with week 1" means

- [ ] You can recite the four tenant_classes and which network primitives each unlocks.
- [ ] You provisioned a tenant VPC + deployed a service + configured Envoy mTLS.
- [ ] You established a BGP peer in the simulator.
- [ ] You wrote and pushed a Cilium-Cedar network policy.
- [ ] You walked a flow-log investigation that shows a cross-tenant deny.
- [ ] You read ADR-0248 + ADR-0253 + ADR-0254 + Cilium 1.18 docs.

## Rookie traps

1. **Forgetting HTTP/3 ALPN.** Custom Envoy configs that put `h2` before `h3` regress to HTTP/2; the linter catches it but
   reviewer-agent doesn't always. Always validate ALPN order.
2. **Skipping SPIFFE SVID rotation.** Services with stale SVIDs (> 2 rotation periods) get quarantined; this is a feature, not a bug.
3. **Cross-tenant flows by accident.** A misconfigured Cilium policy can permit cross-tenant; the `lean-a3-tenant-trace` lane
   blocks at policy lint time, but always verify with a flow-log spot check.
4. **BGP route leak.** Without RPKI on the peer side (paid doesn't require it; paid tenant_class does), a misconfigured peer can advertise
   bogon routes. Always enable per-peer prefix-limit + bogon-filter.
5. **Manual NAT instance scaling.** NAT instances at paid are 1× active + 1× standby; if you scale to 2× active you break
   stateful flow tracking. Use HA cluster (paid tenant_class) for active-active NAT.
6. **Bypassing the service mesh.** Direct pod-to-pod IP traffic bypasses Cilium L7 policy; always go through service names.
