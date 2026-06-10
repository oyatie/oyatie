# Prelane-0.5 — Per-Migration-Lane Source Inventory

**Generated:** 2026-06-06
**Mode:** READ-ONLY (ls/grep/read + manifest inspection; no builds, no edits)
**Scope:** Six migration source lanes. Records, per lane: source path, first-party crate allowlist (package names), and per-tree DENY-GLOBS (only globs whose dirs are actually present in that tree, plus the canonical universal set).

Canonical DENY-GLOB universe (applied per-tree, intersected with what physically exists):
`_upstream*` / `third-party` / `vendor` / `target` / `buck-out` / `prelude` / `toolchains` / `__pycache__` / `.omc` / `.omx` / `.claude` / `legacy-*` / `talos-reference`

---

## Lane 1 — office

- **Source path:** `/Users/jasonlee/Developer/office`
- **Workspace:** single Cargo workspace (`resolver = "3"`, edition 2024); 19 members (13 crates + 6 apps). Buck2-canonical build, Cargo metadata-only.
- **#first-party crates:** 19
- **First-party crate allowlist (package names):**
  - Library crates (`crates/`): `oyaoffice-kernel`, `oyaoffice-tenant-domain`, `oyaoffice-authz-domain`, `oyaoffice-drive-domain`, `oyaoffice-drive-api-contracts`, `oyaoffice-doc-domain`, `oyaoffice-sheet-domain`, `oyaoffice-slide-domain`, `oyaoffice-format-domain`, `oyaoffice-collab-domain`, `oyaoffice-storage-port`, `oyaoffice-search-port`, `oyaoffice-api-contracts`
  - App crates (`apps/`): `oyaoffice-api`, `oyaoffice-drive-api`, `oyaoffice-drive-worker`, `oyaoffice-collab-gateway`, `oyaoffice-format-worker`, `oyaoffice-web`
- **DENY-GLOBS (present in tree):** `third-party/`, `target/`, `buck-out/`, `toolchains/`, `.omx/`
- **Notes:** `[workspace.metadata.oyaoffice.first_party_layout]` self-declares the same crate+app layout (authoritative). `supply-chain/`, `deploy/`, `specs/`, `docs/` are non-crate dirs (not Rust members).

## Lane 2 — oyago

- **Source path:** `/Users/jasonlee/Developer/oyago`
- **Workspace:** single Cargo workspace (`resolver = "3"`, edition 2024); 3 members. Also contains a Go reference tree (`go/`, `go.mod`) and a Go fixture binary — Go is reference/fixture, not first-party Rust.
- **#first-party crates:** 3
- **First-party crate allowlist (package names):** `oyago-cli`, `oyago-core`, `oyago-runtime`
- **DENY-GLOBS (present in tree):** `target/`, `target/**/buck-out/`, `target/**/toolchains/`, `.omc/`, `.omx/`, plus the Go fixture dirs `go/`, `fixtures/` and prebuilt fixture binaries `oyago-i64lit`, `test_join_temp` (non-Rust artifacts; not crates).
- **Notes:** buck-out/toolchains appear only nested under `target/oyago-test-externalrefs/` (test scaffolding), already covered by the `target/` deny.

## Lane 3 — oyapy

- **Source path:** `/Users/jasonlee/Developer/oyapy`
- **Workspace:** single Cargo workspace (`resolver = "3"`, edition 2024, `unsafe_code = deny`, clippy `all = deny`); 3 members. Contains a Python reference (`python/oyapy_analyzer.py`) — reference, not first-party Rust.
- **#first-party crates:** 3
- **First-party crate allowlist (package names):** `oyapy-cli`, `oyapy-core`, `oyapy-runtime`
- **DENY-GLOBS (present in tree):** `target/`, `.omx/`, plus `python/` and `fixtures/` (non-Rust reference dirs; not crates).

## Lane 4 — claude

- **Source path:** `/Users/jasonlee/Developer/claude`
- **Workspace:** NOT a workspace — single top-level package crate (edition 2024, rust 1.85). Sources live in `src/` directly.
- **#first-party crates:** 1
- **First-party crate allowlist (package names):** `claude-agent-sdk`
- **DENY-GLOBS (present in tree):** `target/`, `.omx/`
- **Notes:** `examples/`, `tests/`, `docs/` are package subdirs of the single crate, not separate members.

## Lane 5 — codex

- **Source path:** `/Users/jasonlee/Developer/codex`
- **Workspace:** NOT a workspace — single package crate located at `sdk/rust/`. Crate package name differs from lib name (lib = `openai_codex_sdk`).
- **#first-party crates:** 1
- **First-party crate allowlist (package names):** `openai-codex-sdk` (lib name `openai_codex_sdk`)
- **DENY-GLOBS (present in tree):** `sdk/rust/target/`, `.omx/`
- **Notes:** Smallest lane. Only Rust artifact is `sdk/rust/`; `sdk/rust/protocol/`, `examples/`, `tests/`, `scripts/` are crate subdirs / non-Rust.

## Lane 6 — linux-stack

- **Source path:** `/Users/jasonlee/Developer/linux/stack`
- **Structure:** META-TREE — no top-level Cargo workspace. Contains **3 independent Cargo workspaces** (`kernel/`, `kubernetes/`, `operating-system/`), one set of standalone usermode-test crates (`kernel-usermode-tests/`), and one **Go reference tree** (`talos-reference/`, which is itself a DENY-GLOB target).
- **#first-party crates:** **208 total** (18 kernel + 139 kubernetes + 45 operating-system + 6 kernel-usermode-tests). The 3 Cargo workspaces declare 202 members (kernel 15 + kubernetes 139 + operating-system 44 incl. `difftest`); the kernel tree contains 18 crate manifests on disk (15 listed members + 3 not-in-members nested crates: `arch-x86_64/fsbase-worker-src`, `arch-x86_64/user-fsbase-src`, `arch-x86_64/user-init-src`).

  ### 6a. linux-stack/kernel (Cargo workspace)
  - **#crates:** 18 (manifests on disk)
  - **Allowlist (package names):** `kernel`, `hal`, `arch-aarch64`, `arch-x86_64`, `frame`, `ksync`, `user_layout`, `arch-aarch64-layout-tests` (= `arch-aarch64/tests-host`), `user-smpdemo` (= `arch-aarch64/user-smpdemo-src`), `user-src`→`user-hello-x86_64`, `user-spawn-x86_64`, `user-exec-x86_64`, `user-signal-x86_64`, `user-clock-x86_64`, `user-smpdemo-x86_64`, `fsbase-worker-x86_64`, `user-fsbase-x86_64`, `user-init-x86_64`. (The `user-*-src` / `*-src` dirs are first-party no_std user-payload + host-test crates inside the workspace.)
  - **DENY-GLOBS (present):** `target/`, `.omc/`, `out/`

  ### 6b. linux-stack/kubernetes (Cargo workspace)
  - **#crates:** 139 (all under `crates/`)
  - **Allowlist (package names — dir names = crate names):** `admissionregistration_v1`, `api_equality`, `api_meta`, `api_validation`, `apidiscovery_v2`, `apiserverinternal_v1alpha1`, `apivalidation_path`, `apps_v1`, `authentication_v1`, `authorization_v1`, `autoscaling_v1`, `batch_v1`, `certificates_v1`, `constraints`, `content`, `conversion`, `coordination_v1`, `core_v1_proto`, `cri_api_v1`, `ctrd_api_types`, `ctrd_api_types2`, `ctrd_apparmor`, `ctrd_archive_link`, `ctrd_archive_time`, `ctrd_atomicfile`, `ctrd_blockio`, `ctrd_cap`, `ctrd_cio`, `ctrd_deprecation`, `ctrd_dialer`, `ctrd_display`, `ctrd_epoch`, `ctrd_fifosync`, `ctrd_filters`, `ctrd_gc`, `ctrd_identifiers`, `ctrd_ioutil`, `ctrd_kernelversion`, `ctrd_labels`, `ctrd_namespaces`, `ctrd_netns`, `ctrd_oci_defaults`, `ctrd_oci_defaults_darwin`, `ctrd_oci_defaults_windows`, `ctrd_oom`, `ctrd_progress`, `ctrd_protobuf`, `ctrd_rdt`, `ctrd_reference`, `ctrd_schedcore`, `ctrd_seccomp`, `ctrd_services`, `ctrd_services2`, `ctrd_shim`, `ctrd_shutdown`, `ctrd_snapshotters`, `ctrd_stdio`, `ctrd_sys_oom`, `ctrd_sys_reaper`, `ctrd_sys_socket`, `ctrd_timeout`, `ctrd_tracing`, `ctrd_ttrpcutil`, `cv_common`, `cv_config`, `cv_namespace`, `cv_node`, `cv_pod`, `cv_service`, `cv_storage`, `discovery_v1`, `duration`, `events_v1`, `field_errors`, `field_path`, `fields`, `flowcontrol_v1`, `framer`, `intstr`, `jsonmergepatch`, `labels`, `mergepatch`, `meta_internalversion`, `meta_iv_validation`, `meta_v1`, `meta_v1_proto`, `meta_v1_validation`, `meta_v1beta1`, `mp_discovery`, `mp_gvk`, `mp_labelselector`, `mp_listoptions`, `mp_misc`, `mp_objectmeta`, `mp_options`, `mp_partial`, `mp_time`, `mp_watchevent`, `naming`, `networking_v1`, `node_v1`, `operation`, `pkg_version`, `policy_v1`, `portforward`, `rand`, `rbac_v1`, `recognizer`, `resource`, `resource_v1`, `resourceversion`, `runtime_codec_factory`, `runtime_schema`, `runtime_scheme`, `runtime_serializer_json`, `runtime_serializer_protobuf`, `runtime_serializer_streaming`, `runtime_serializer_versioning`, `runtime_serializer_yaml`, `runtime_types`, `safe`, `scheduling_v1`, `selection`, `sets`, `storage_v1`, `types`, `unstructured`, `util_cache`, `util_errors`, `util_json`, `util_net`, `util_runtime`, `util_uuid`, `util_wait`, `util_yaml`, `validation`, `version`, `waitgroup`, `watch`
  - **DENY-GLOBS (present):** `_upstream/`, `_upstream_containerd/`, `third-party/`, `prelude/`, `prelude/third-party/`, `toolchains/`, `prelude/toolchains/`, `target/`, `buck-out/`, `__pycache__/`, `scripts/__pycache__/`, `.omc/`, `.omx/`

  ### 6c. linux-stack/operating-system (Cargo workspace)
  - **#crates:** 45 (44 declared members on disk incl. `difftest`; 45 Cargo.toml manifests counting the workspace root)
  - **Allowlist (package names — dir names):** `talos-core`, `talos-cosi`, `talos-machine-config`, `talos-machined`, `talos-apid`, `talos-trustd`, `talos-talosctl`, `talos-network`, `talos-block`, `talos-runtime-cri`, `talos-kubernetes`, `talos-cluster`, `talos-meta`, `talos-install`, `talos-time`, `talos-events-logging`, `talos-security`, `talos-init`, `talos-resources`, `talos-config-v1alpha1`, `talos-config-docs`, `talos-controllers`, `talos-kubelet`, `talos-k8s-control`, `talos-kubespan`, `talos-siderolink`, `talos-secrets`, `talos-etcd`, `talos-perf`, `talos-hardware`, `talos-imager`, `talos-extensions`, `talos-board`, `talos-archiver`, `talos-conditions`, `talos-api-proto`, `talos-maintenance`, `talos-dashboard`, `talos-syslogd`, `talos-upgrade`, `talos-cluster-mgmt`, `talos-talosctl-commands`, `talos-platform`, `difftest`
  - **DENY-GLOBS (present):** `target/`, `buck-out/`, `toolchains/`, `.omc/`, `.omx/`, `.claude/`, plus non-crate dirs `boot/`, `platforms/`, `tools/` (not workspace members).

  ### 6d. linux-stack/kernel-usermode-tests (standalone crates, no shared workspace)
  - **#crates:** 6
  - **Allowlist (dir names; package names per-crate):** `init`, `exec`, `hello`, `signal`, `spawn`, `clock`
  - **DENY-GLOBS:** none present locally (no target/omc inside this dir at scan time).

  ### 6e. linux-stack/talos-reference (DENY — Go reference tree, NOT first-party)
  - Go module (`go.mod`, `go.work`); is itself a top-level DENY-GLOB (`talos-reference`). Contains `api/vendor/` (vendor deny). **0 first-party Rust crates.**

- **linux-stack DENY-GLOBS (whole-tree, present):** `_upstream*` (kubernetes), `_upstream_containerd` (kubernetes), `third-party` (kubernetes, prelude/), `vendor` (talos-reference/api), `target` (all 3 workspaces), `buck-out` (kubernetes, operating-system), `prelude` (kubernetes), `toolchains` (kubernetes, operating-system, prelude/), `__pycache__` (kubernetes), `.omc` (root + 3 workspaces + _upstream_containerd), `.omx` (talos-reference, kubernetes, operating-system), `.claude` (operating-system), `talos-reference` (entire subtree).

---

## RETURN — per-repo summary

| Lane | Source path | #first-party crates | DENY-GLOBS (present in tree) |
|---|---|---|---|
| office | /Users/jasonlee/Developer/office | 19 (13 crates + 6 apps) | third-party/, target/, buck-out/, toolchains/, .omx/ |
| oyago | /Users/jasonlee/Developer/oyago | 3 | target/ (covers nested buck-out+toolchains), .omc/, .omx/; +Go reference go/,fixtures/ |
| oyapy | /Users/jasonlee/Developer/oyapy | 3 | target/, .omx/; +Python reference python/, fixtures/ |
| claude | /Users/jasonlee/Developer/claude | 1 (claude-agent-sdk) | target/, .omx/ |
| codex | /Users/jasonlee/Developer/codex | 1 (openai-codex-sdk @ sdk/rust) | sdk/rust/target/, .omx/ |
| linux-stack | /Users/jasonlee/Developer/linux/stack | 208 (kernel 18 + kubernetes 139 + operating-system 45 + kernel-usermode-tests 6); talos-reference Go=0 | _upstream*, _upstream_containerd, third-party, vendor, target, buck-out, prelude, toolchains, __pycache__, .omc, .omx, .claude, talos-reference |

**Totals across all lanes:** 235 first-party Rust crates (19 + 3 + 3 + 1 + 1 + 208).
