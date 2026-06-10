# Prelane-0.5 / 10 — k8s + containerd 139-crate SPLIT + cloud-k8s relationship

LANE G4. READ-ONLY audit. Evidence collected 2026-06-06 against live filesystem +
`stack/kubernetes/Cargo.toml` workspace manifest.

## TL;DR (counts)

| Bucket | Count | Definition |
|---|---|---|
| **k8s-MERGE** | **95** | non-`ctrd_*` crates = k8s apimachinery / api / serializer / `cv_*` core-v1 split |
| **containerd-CREATE** (`ctrd_*`) | **44** | crates prefixed `ctrd_` (containerd Go-pkg ports) |
| **vendored-exclude** | **0** | no vendored crate lives inside the 139 workspace members |
| **TOTAL workspace members** | **139** | reconciles exactly: 44 + 95 = 139 |

All three counts triangulated identically from (a) `ls crates/`, (b) the `[workspace].members`
list in `stack/kubernetes/Cargo.toml` (139 unique `"crates/<name>"` entries), and
(c) prefix greps. Every one of the 139 dirs has a `Cargo.toml` (no orphan/non-crate dirs).

## containerd-CREATE — the 44 `ctrd_*` crates

These are the Go→Rust ports of containerd packages. They form the containerd-merge target:

```
ctrd_api_types  ctrd_api_types2  ctrd_apparmor  ctrd_archive_link  ctrd_archive_time
ctrd_atomicfile ctrd_blockio     ctrd_cap       ctrd_cio           ctrd_deprecation
ctrd_dialer     ctrd_display     ctrd_epoch     ctrd_fifosync      ctrd_filters
ctrd_gc         ctrd_identifiers ctrd_ioutil    ctrd_kernelversion ctrd_labels
ctrd_namespaces ctrd_netns       ctrd_oci_defaults ctrd_oci_defaults_darwin
ctrd_oci_defaults_windows        ctrd_oom       ctrd_progress      ctrd_protobuf
ctrd_rdt        ctrd_reference   ctrd_schedcore ctrd_seccomp       ctrd_services
ctrd_services2  ctrd_shim        ctrd_shutdown  ctrd_snapshotters  ctrd_stdio
ctrd_sys_oom    ctrd_sys_reaper  ctrd_sys_socket ctrd_timeout      ctrd_tracing
ctrd_ttrpcutil
```

Notes:
- `*2` siblings (`ctrd_api_types2`, `ctrd_services2`) are extension crates that
  `path`-depend on their base crate (e.g. `ctrd_api_types2 → ctrd_api_types`).
- `*_darwin` / `*_windows` are per-OS variants of `ctrd_oci_defaults` (kept as
  separate crates, not feature-gated). All count as containerd-CREATE.
- Upstream reference source vendored separately at `stack/kubernetes/_upstream_containerd/`
  (NOT a workspace member → not counted).

## k8s-MERGE — the 95 non-`ctrd_*` crates

Ports of `k8s.io/apimachinery`, `k8s.io/api`, and supporting util/serializer packages.
Includes the **7 `cv_*` crates** which are the `core/v1` proto split
(`cv_common cv_config cv_namespace cv_node cv_pod cv_service cv_storage`, re-aggregated
by `core_v1_proto`) and the typed apiGroup crates (`apps_v1`, `batch_v1`, `rbac_v1`,
`networking_v1`, `storage_v1`, `meta_v1*`, `runtime_*`, `util_*`, etc.). `cri_api_v1`
(CRI gRPC types) sits on the k8s side. Full set = the 139 members minus the 44 `ctrd_*`.

- Upstream k8s reference source vendored separately at `stack/kubernetes/_upstream/`
  (NOT a workspace member → not counted).
- `stack/kubernetes/third-party/` holds vetted external-crate vendoring per
  `ALLOWED_CRATES.md` (std-only-by-default policy) — also not part of the 139.

## cloud-k8s relationship — VERDICT REQUIRED FROM FOUNDER

### Evidence
`/Users/jasonlee/Developer/source/cloud/cloud-k8s` is **NOT** a Rust crate workspace and
does **NOT** mirror the k8s/containerd port:
- It has **no `crates/` content** (dir empty) and **no `Cargo.toml` / Cargo workspace** anywhere.
- It is a **docs/spec/governance service**: `PRD.md`, `ARCHITECTURE.md` (94 KB),
  `compliance.md` (125 KB), ADR-linked README, `manifest.json`, IP-* implementation
  plans, journeys, threat-models, SLOs, runbooks, contracts (OpenAPI/AsyncAPI/proto scaffolds).
- Its `manifest.json` declares bounded_context **`cloud-compute`** with crates named
  `oya-cloud-compute-{domain,functions-api,k8s-api,vm-api,adapter-aws,adapter-oci}` —
  i.e. a DDD provider-neutral cloud-compute control surface. The manifest explicitly
  states it "does not implement live Kubernetes bootstrap, EKS, OKE, ... CNI, REST
  server, SDK" — it is a metadata/invariants foundation, not a kubelet/apiserver port.
- The 4 `cloud/managed-k8s-*` services (`cluster-lifecycle`, `control-plane-host`,
  `sla-observability`, `tenant-quota`) DO have `crates/`, but they are **`oya-managed-k8s-*`
  DDD layer crates** (`-kernel`, `-app`, `-api`, `-adapter-cedar`, `-adapter-inmemory`) —
  managed-service control planes, with **no naming or path overlap** with the 139
  upstream-port crates under `stack/kubernetes/crates`.

### Conclusion
`cloud-k8s` and the `managed-k8s-*` services are a **different layer** (cloud control-plane /
governance over Kubernetes) than `stack/kubernetes` (a Go→Rust *reimplementation* of
upstream k8s + containerd). They consume Kubernetes; they are not a 6th copy of it.

### Founder verdict-options for the cloud-k8s relationship
1. **Out-of-scope of the 139-crate split (RECOMMENDED).** Treat `cloud-k8s` +
   `managed-k8s-*` as separate cloud-layer microservices. The k8s/containerd split lane
   stays confined to `stack/kubernetes/crates`. cloud-k8s relationship captured docs-only.
2. **Docs-only merge-target.** Record a cross-link (ADR / masterplan reachability edge)
   so cloud-k8s's k8s-API surface (`oya-cloud-compute-k8s-api`) is documented as the
   *consumer* of the merged k8s crates — no code merge, just a dependency-direction note.
3. **6th merge target.** Reject unless founder asserts cloud-k8s should be folded into
   the upstream-port workspace. Evidence is AGAINST this: different naming scheme
   (`oya-*` DDD vs upstream-package ports), no Cargo workspace, explicitly non-runtime
   scope. Would conflate "reimplement Linux/k8s" with "operate managed k8s".

**Open question for founder:** Confirm option (1) [out-of-scope] vs (2) [docs-only
cross-link]. (3) appears contradicted by evidence and is listed only for completeness.

## Files referenced (absolute)
- `/Users/jasonlee/Developer/linux/stack/kubernetes/Cargo.toml` (139 members)
- `/Users/jasonlee/Developer/linux/stack/kubernetes/crates/` (139 dirs, all with Cargo.toml)
- `/Users/jasonlee/Developer/linux/stack/kubernetes/_upstream/`, `_upstream_containerd/`, `third-party/` (vendored, excluded)
- `/Users/jasonlee/Developer/linux/stack/kubernetes/ALLOWED_CRATES.md`
- `/Users/jasonlee/Developer/source/cloud/cloud-k8s/` (docs service, no crates/, no Cargo.toml)
- `/Users/jasonlee/Developer/source/cloud/cloud-k8s/manifest.json` (bounded_context `cloud-compute`)
- `/Users/jasonlee/Developer/source/cloud/managed-k8s-{cluster-lifecycle,control-plane-host,sla-observability,tenant-quota}/crates/` (`oya-managed-k8s-*` DDD crates)
