---
doc_class: Program-Mapping-Record
doc_status: published
seam_lane: os-k8s
upstream_pin: 756939600b9a7180fc2df6550a4585b638875e67
measured_at: 2026-08-09
authority_tier: 3
---

# os/ ↔ k8s/ seam mapping — the contract every unit of this lane is checked against

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-09) |
|---|---|---|
| Repository baseline | authored against `origin/dev` @ `5e452bd70449b50cc66e63ffb9253adfcd7fc96e`; **rebased at Land onto `origin/dev` @ `1d31052774ef580553a5ff81014849bb38d6e327`** | Lane base for branch `impl/os-k8s-seam-conformance`. `git diff --stat 5e452bd70 1d3105277 -- os/` is empty — the new base touches no `os/` path — so every `os/` measurement below survives the rebase unchanged. Section 8 names the one number that did move. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Read from `specs/k8s-port/upstream-pin.json` and **re-resolved against upstream**: `git ls-remote --tags https://github.com/kubernetes/kubernetes.git 'refs/tags/v1.36.1' 'refs/tags/v1.36.1^{}'` returns annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2` and this peeled commit, matching the pin file. |
| Engine | `build/port-engine/*`, v0 — `build/port-engine/core/port-engine-kernel` IS on the base tree (7 tracked files); it registers no producer | Not in force. A W0 skeleton with no front end, no rule data and no renderer: it emits no output and owns no path. |
| Neutral rule pack | `specs/port-rules/**`, v0 — the path does not exist on the base tree | Not in force. No rule is authored or implied here. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | None. Repository measurements are `git grep` / `git ls-tree` over the base tree; the section 3.2 module column is `git ls-tree` / `git grep` over a blob-less fetch of the pin | Measurement instrument only; not an admitted extractor. No Go was parsed — only paths and top-level `type`/`const` declarations were read. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This document emits no receipt. |
| Program authority | ADR-0704 (live apex). ADR-0637 and ADR-0638 are archived provenance | Mapping record only; authorizes no runtime edge and no ADR change. |

## 0. What this document is

It is the mapping every unit of branch `impl/os-k8s-seam-conformance` is checked against. It fixes
the naming, the invariants and the definition of done **before** any implementation exists, so that
units written in parallel converge. It is not an ADR, not a plan approval, and it authorizes nothing
that ADR-0704 does not already authorize.

It is **not** a decision to move a crate, delete a crate, or wire a dependency. Sections 3 and 4
decide *what a thing would be named and where it would live if it were created*; section 5 forbids
creating the runtime edge today, and section 9 records why.

## 1. Authority — cite the apex, keep the members as provenance

Live law for this program is **ADR-0704**. ADR-0637 and ADR-0638 are `status: Superseded`, live under
`docs/adr-archive/`, and carry a non-authority banner. ADR-0704 preserves the ADR-638 D1 destination
rule verbatim.

The clauses this lane is bound by, stated once so no unit has to re-derive them:

| Ref | Clause, as it binds this lane |
|---|---|
| ADR-638 D1 (live via ADR-0704) | Generated Kubernetes Rust output SHALL live under `k8s/`. `os/`, `cloud/cloud-k8s` and the managed-Kubernetes facades consume it only through approved `k8s/ports/**` seams and MUST NOT become alternate homes for generated upstream code. |
| ADR-637 D2 (live via ADR-0704) | Output in a registered regenerable region is generated: emitted by the registered producer, never hand-edited. A red result is an engine/rule/policy defect, never repaired by editing emitted Rust. |
| ADR-638 D5 (live via ADR-0704) | W0 does not exit until the engine ports a bounded **Talos** second corpus and passes its **already-landed** `os/harness/difftest-app` vectors. The word *landed* charters the existing hand-written Talos corpus as the proof surface. |
| ADR-638 D6 (live via ADR-0704) | The `os/` shared-types direction is deferred to W0-G. **No runtime edge is authorized** before W0-G topology ratification. |

**Citation rule for every unit:** write ADR ids as bare text (`ADR-0704`, `ADR-638 D1`). Never write a
id as a path under `docs/decisions/` — that directory holds only ADR-0700..0709, so any other id
written that way is a broken link and scores `adr_citation_dangling_path`, which is pinned by
equality (see trap T-5).

## 2. Corrected premises — what survives of the two findings

The lane brief carried two findings. Both are directionally real and both were mis-stated. The
corrected form is what units implement.

**FINDING 1 — topology confirmed, provenance wrong.** `os/` has 41 crates; eight are
Kubernetes-facing (`cluster-domain`, `cluster-mgmt-domain`, `etcd-domain`, `k8s-control-domain`,
`kubelet-domain`, `kubernetes-domain`, `kubespan-domain`, `runtime-cri-domain`); zero depend on any
`k8s/` crate. But every one of the eight cites `siderolabs/talos` internal paths in its module doc and
**none** cites a `k8s.io/kubernetes` source path. They are a hand-written **Talos** port, and ADR-638
D5 charters exactly that corpus, naming its landed difftest vectors as the engine's proof obligation.
`os/` hand-writing Talos is therefore chartered, not the rejected alternative. The `~123 lines` figure
for `os/core/kubelet-domain` is `src/lib.rs` alone; the crate is 1,719 source lines across six files.

**What is genuinely Talos and MUST NOT be deleted or relocated by this lane:** cluster
discovery/affiliates/membership, the KubeSpan WireGuard mesh, etcd-on-Talos lifecycle
(backup/member/snapshot), `talosctl`-class cluster provisioning and bootstrap, the containerd/CRI
integration (`runtime-cri-domain`, 14,210 lines, five consumers), machined controllers, and the
`v1alpha1` machine-config document surface. None of it exists in the Kubernetes corpus.

**What is chartered Kubernetes and is the real defect:** not whole crates — **16 sites in 7 files
across 3 crates** where `os/` hand-writes upstream **Kubernetes API** wire surface. Upstream Talos
does not string-template these; `internal/app/machined/pkg/controllers/k8s/kubelet_spec.go` imports
`k8s.io/kubelet/config/v1beta1`, `k8s.io/api/core/v1` and `k8s.io/apimachinery/pkg/runtime` and
marshals typed structs. The Rust port severed that import edge and replaced it with hand-rolled
string builders, one of which documents itself as *"not a full YAML serializer"*. That is the
hand-written Kubernetes surface, it is lossy, and no divergence-ledger row covers it.

**FINDING 2 — line counts exact, framing wrong.** `k8s/ports/` holds four managed-Kubernetes product
APIs totalling 1,162 lines, exactly as stated. They are not squatting. `specs/capability-registry.json`
is `"closed": true`, charters capability `k8s` as *"Owned Kubernetes control plane (core) +
managed-k8s product (facade)"*, lists the four `cloud/managed-k8s-*` dirs in `absorbs_current_dirs`,
and charters the `ports` face as *"Capability traits; the stable seam"* — which is what three of the
four crates are (`ControlPlaneProvisioning`, `SlaObservabilityPort`, `QuotaDecisionPort`/`QuotaAdminPort`).
`k8s/ports/**` in ADR-0704 is a **glob**; generated seam leaves sit beside the incumbents without a
name collision. See section 6 for the disposition and why a MOVE is refused.

## 3. The mapping — every recurring pattern and exactly what it becomes

### 3.1 Emit home: upstream package path decides the face, mechanically

The predicate is the SourceModel package path, not a human judgement:

| Upstream package path at the pin | Becomes | Rationale |
|---|---|---|
| `staging/src/k8s.io/<module>/<rest>` | `k8s/ports/upstream-<dashed>` | Upstream itself declares `staging/src/k8s.io/*` the externally consumable set, published read-only with PRs to the published repos refused — the same no-hand-edit contract ADR-637 D2 asserts. Externally consumable upstream ⇒ the seam face. |
| any other first-party path (`pkg/**`, `cmd/**`, `plugin/**`) | `k8s/core/upstream-<dashed>` | Not externally consumable upstream ⇒ engine face, no external dependents. `core` is chartered *"the engine we RUN (substrate face)"*. |
| `vendor/**`, generated inputs | neither — `EXCLUDE` per `specs/k8s-port/scope.json` | Already ruled by the landed W0-A scope rules; this lane does not restate them. |

`<dashed>` is derived by a dumb, total function — generated names are never typed by hand, so length
does not matter and a special case would:

1. strip the leading `staging/src/k8s.io/` (staging) or nothing (non-staging);
2. drop any `pkg/apis/`, `pkg/` or `tools/` segment — Go layout artifacts, not identity;
3. replace every remaining `/` with `-`;
4. lowercase; no other transformation.

Crate directory `k8s/<face>/upstream-<dashed>`, package name `k8s-upstream-<dashed>`, one to one.

**Why `upstream-` and not a new directory.** The root `Cargo.toml` members are the four depth-3 layer
globs `*/core/*`, `*/ports/*`, `*/adapters/*`, `*/facade/*`. `k8s/ports/upstream-api-core-v1` matches
`*/ports/*` and needs **zero** root-manifest edit. A novel root (`build/…`, `k8s/generated/…`) matches
no glob, REDs `cloud-ci-workspace-glob-coverage` with `crate_dir_not_covered`, and would need an
ADR-authorized members-line exception this lane has no authority to write. The `upstream-` prefix is
what makes a generated leaf visually and mechanically separable from a hand-authored product port
inside the same glob — which is the whole reason the incumbents do not need to move.

### 3.2 The seam demands — the 16 sites, and what each one becomes

Two measurement sources, kept separate on purpose. Columns 1–3 and 5 are measured on the **base
tree** — reproducer, with its negative control, in section 8. Column 4 is measured on the **upstream
pin** — reproducer, with two `MISSING` controls, immediately below the table.

| # | Site | Emitted upstream group | Upstream module at the pin (**measured**) | Becomes |
|---|---|---|---|---|
| 1 | `os/core/k8s-control-domain/src/admission.rs:96` | `pod-security.admission.config.k8s.io/v1`, `PodSecurityConfiguration` | `k8s.io/pod-security-admission/admission/api/v1` | consume `k8s/ports/upstream-pod-security-admission-admission-api-v1` |
| 2 | `os/core/k8s-control-domain/src/admission.rs:173` | `apiserver.config.k8s.io/v1`, `AdmissionConfiguration` | `k8s.io/apiserver/pkg/apis/apiserver/v1` | consume `k8s/ports/upstream-apiserver-apiserver-v1` |
| 3 | `os/core/k8s-control-domain/src/admission.rs:234` | `audit.k8s.io/v1`, `Policy` | `k8s.io/apiserver/pkg/apis/audit/v1` | consume `k8s/ports/upstream-apiserver-audit-v1` |
| 4 | `os/core/k8s-control-domain/src/admission.rs:338` | `apiserver.config.k8s.io/v1`, **`EncryptionConfiguration`** — a different kind from #2 | as #2 — `types_encryption.go` sits in that same module at this pin; the historical separate home `k8s.io/apiserver/pkg/apis/config/v1` does **not** exist at `v1.36.1` (0 files) | as #2 |
| 5 | `os/core/k8s-control-domain/src/manifest_controller.rs:200` | core/v1 `ConfigMap` | `k8s.io/api/core/v1` | **test fixture** (inside `mod tests`, which opens at line 196) — becomes a fixture over the seam type, not a production call site |
| 6 | `os/core/k8s-control-domain/src/static_pod_controller.rs:128` | core/v1 `Pod` | `k8s.io/api/core/v1` | consume `k8s/ports/upstream-api-core-v1` |
| 7 | `os/core/kubelet-domain/src/spec.rs:120` | `kubelet.config.k8s.io/v1beta1`, `KubeletConfiguration` | `k8s.io/kubelet/config/v1beta1` | consume `k8s/ports/upstream-kubelet-config-v1beta1` |
| 8 | `os/core/kubernetes-domain/src/kubeconfig.rs:117` | `v1` `Config` (kubeconfig; clientcmd's own scheme, **not** the core API group) | `k8s.io/client-go/tools/clientcmd/api/v1` — **corrected**, see below | consume `k8s/ports/upstream-client-go-clientcmd-api-v1` (unchanged) |
| 9 | `os/core/kubernetes-domain/src/static_pod.rs:137` | core/v1 `Pod` | `k8s.io/api/core/v1` | as #6 — **second independent Pod renderer**, see T-7 |
| 10–11 | `os/core/kubernetes-domain/src/templates.rs:21,34` | `rbac.authorization.k8s.io/v1`, `ClusterRoleBinding` ×2 | `k8s.io/api/rbac/v1` | consume `k8s/ports/upstream-api-rbac-v1` |
| 12–13 | `os/core/kubernetes-domain/src/templates.rs:56,93` | `apps/v1`, `DaemonSet` and `Deployment` | `k8s.io/api/apps/v1` | consume `k8s/ports/upstream-api-apps-v1` |
| 14–16 | `os/core/kubernetes-domain/src/templates.rs:115,129,148` | core/v1 `Service`, `ConfigMap`, `Namespace` | `k8s.io/api/core/v1` | as #6 |

**The module column is measured at the pin**, peeled commit
`756939600b9a7180fc2df6550a4585b638875e67` (`v1.36.1`). Each of the eight distinct modules was
confirmed to exist there, and the kind each `os/` site emits was confirmed to be declared inside it.
This is a *path-and-declaration* confirmation and nothing more: no package enumeration has run
(`specs/k8s-port/scope.json` `enumeration_state.state = "pending_source_model_manifest"`), so no row
claims a scope disposition, a package boundary, an emitted crate, or field-level fidelity.

Reproducer — a blob-less fetch is enough, and it checks the whole column rather than a sample:

```
git init k8s-pin && git -C k8s-pin remote add origin https://github.com/kubernetes/kubernetes.git
git -C k8s-pin fetch --depth 1 --filter=blob:none origin 756939600b9a7180fc2df6550a4585b638875e67
for m in api/core/v1 api/rbac/v1 api/apps/v1 kubelet/config/v1beta1 \
         apiserver/pkg/apis/apiserver/v1 apiserver/pkg/apis/audit/v1 \
         pod-security-admission/admission/api/v1 client-go/tools/clientcmd/api/v1 \
         client-go/clientcmd/api/v1 apiserver/pkg/apis/config/v1 ; do
  n=$(git -C k8s-pin ls-tree -r --name-only FETCH_HEAD -- "staging/src/k8s.io/$m/" | wc -l | tr -d ' ')
  [ "$n" -gt 0 ] && echo "OK      $n	$m" || echo "MISSING	$m"
done
```

The eight rows of the column print `OK` with 20, 9, 9, 6, 9, 9, 8 and 9 files. The last two entries
are the controls and both print `MISSING`: `client-go/clientcmd/api/v1` is the pre-correction row 8,
and `apiserver/pkg/apis/config/v1` is the module row 4 was *suspected* of needing. Group identity was
read the same way — the `GroupName` const in each module's `register.go` equals column 3's group —
except kubeconfig, which declares no `GroupName` and instead carries
`SchemeGroupVersion = schema.GroupVersion{Group: "", Version: "v1"}` in `register.go` and
`type Config struct` in `types.go`.

**Row 8 was wrong and is corrected: the module is `k8s.io/client-go/tools/clientcmd/api/v1`.** The
`tools/` segment was missing. The *Becomes* cell did not change, and that is the instructive part —
section 3.1 step 2 drops any `tools/` segment, so the wrong source path derives the *right* crate
name. A correct derived name is therefore no evidence at all about the path it came from (T-11).

**Row 4 was suspected wrong and is not.** `EncryptionConfiguration` shares the group
`apiserver.config.k8s.io/v1` with `AdmissionConfiguration` but is a different kind, and in earlier
minors it lived in a separate package. At this pin it does not:
`staging/src/k8s.io/apiserver/pkg/apis/apiserver/v1/types_encryption.go:70` declares it and
`staging/src/k8s.io/apiserver/pkg/apis/config/v1/` has zero files. "as #2" holds — but it holds
*because it was measured at this pin*, not because the group matched, and it is pin-dependent.

**Direction evidence, recorded not ruled.** Sixteen concrete demands `os → k8s`; zero demands
`k8s → os`; zero dependency edges in either direction today. This is the measured dependency evidence
ADR-638 D6 deferred to W0-G. This document records it; it does not rule it (section 9).

### 3.3 The seam contract `os/` consumes

Four clauses, all mechanically checkable, all stated as the contract a future generated crate must
satisfy — none is wired today:

- **C1 — leaves.** A `k8s/ports/upstream-*` crate has **no** `path =` dependency on any first-party
  crate. It may depend only on other `k8s/ports/upstream-*` crates and on transient-infra crates the
  rule pack declares. The incumbent `k8s/ports/cluster-lifecycle-api` depends *inward* on
  `k8s/core/cluster-lifecycle-kernel` and on two sibling ports; generated seams must not copy that
  shape (trap T-6).
- **C2 — no re-declaration.** `os/` must not define a type a seam exports. Consuming the seam is a
  *faithful port of an import edge Talos already has*, so it needs no divergence row. The present
  string-templating is the divergence.
- **C3 — the pin lives with the application.** A seam crate never pins the Kubernetes version; the
  binary that composes it does. Precedent: `k8s-openapi` requires exactly one `v1_*` feature and
  forbids library crates from enabling one. Mechanism cited, crate **not** adopted — ADR-637 rejected
  `kube-rs` for introducing another upstream clock and the same reasoning covers its siblings.
- **C4 — no hand edit.** Once a path is a registered regenerable region, a red result is repaired in
  the rules, never in the emitted Rust (ADR-637 D2).

## 4. Naming, module and ownership conventions

| Thing | Convention | Note |
|---|---|---|
| Generated seam crate | dir `k8s/ports/upstream-<dashed>`, package `k8s-upstream-<dashed>` | Section 3.1. Matches `*/ports/*`; zero root-manifest edit. |
| Generated engine crate | dir `k8s/core/upstream-<dashed>`, package `k8s-upstream-<dashed>` | Same name function, different face. |
| Hand-authored product port | unchanged: `k8s/ports/<concern>-api` | The four incumbents keep their names. No `oya-` prefix, no `managed-k8s-` infix in a crate name. |
| Talos-port crate in `os/` | unchanged: `os/core/<domain>-domain`, package `os-<domain>-domain` | This lane renames nothing in `os/`. |
| Divergence row id | `DVG-<SCOPE>-<SUBJECT>`, screaming-kebab | Matches the five landed seeds (`DVG-CEDAR-AUTHORIZATION-SEAM`, …). |
| Doc under `docs/programs/k8s-port/` | frontmatter with `doc_status:` **and** a `## Baseline version header` section | Both are gates, not style (T-4, T-9). |
| Ownership of a generated crate | `OWNERS` names the k8s-port program owner, not the consuming lane | A consumer never owns emitted output it cannot edit. |
| Commit target | every unit commits **directly** to `impl/os-k8s-seam-conformance` | No per-unit branch, no per-unit PR. One PR at Land. |

## 5. Invariants — true after every unit, checkable on that unit alone

A reviewer holding one diff can check all ten. `INV-1`..`INV-3` and `INV-4` are commands; the rest are
diff-readable.

| Id | Invariant | Check |
|---|---|---|
| INV-1 | No `os/` crate depends on any `k8s/` crate. | `ci-k8s-program-docs` parses every `os/**/Cargo.toml` for package names starting with `k8s-` under dependency tables **and** Cargo `[patch]` / `[patch.crates-io]` / `[patch."…"]` override tables (inline `{ path }`, quoted keys such as `"k8s-foo" = …`, named `[dependencies.k8s-*]` / `[dependencies."k8s-*"]`, `k8s-*.workspace = true`, and `package = "k8s-…"` renames) and every `os/**/{BUCK,BUCK.v2}` for `//k8s/` target edges (`BUCK.v2` shadows `BUCK`). Only those manifest/buildfile basenames are UTF-8-decoded. Finding code `R-DOC-CROSS-SEAM-DEPENDENCY`. |
| INV-2 | No `k8s/` crate depends on any `os/` crate. | Same gate over `k8s/**/Cargo.toml` for package names starting with `os-`, and over `k8s/**/{BUCK,BUCK.v2}` for `//os/` target edges. |
| INV-3 | The count of upstream-Kubernetes `apiVersion:` emit sites in `os/` equals the frozen census. **Frozen at 16.** | Section 8 reproducer. Frozen at equality: growth is the chartered defect, and a unit that retires a site re-freezes the number in the same commit — an unrecorded shrink is red too, because banked headroom lets the site come back. |
| INV-4 | Every `k8s/ports/upstream-*` crate is a dependency leaf (C1). | `git grep -nE '\{ *path *=' -- 'k8s/ports/upstream-*/Cargo.toml'` matches only other `upstream-` paths. Vacuously true until the first such crate exists. |
| INV-5 | No file under a registered regenerable region is hand-edited. | Vacuously true until `specs/k8s-port/regenerable-regions.json` exists and is non-empty. Once it does, the diff must touch no listed path. |
| INV-6 | A unit re-freezes a governed census **only when its own diff moved it**, in the same commit, as a text edit keyed by name. A unit that adds or removes no tracked governed file touches no ceiling. | The assertion itself instructs "re-freeze it in the SAME change", and the corpus axes assert **before** the finding axes — so a stale corpus number silently disables the finding ratchet for every later unit. That is why this is not deferred to Land like other bookkeeping. |
| INV-7 | Every added `.md` under `docs/` declares `doc_status:`; every one under `docs/programs/k8s-port/` also carries `## Baseline version header` with all six axis tokens. | Read the diff's frontmatter and header. |
| INV-8 | ADR ids appear as bare text, never as a path under `docs/decisions/`. | `git grep -n 'decisions/ADR' <changed files>` is empty. |
| INV-9 | No `os/` crate, module or public type is deleted without (a) `git log --follow` on the path in the commit message and (b) a body-level equivalence proof, not a name match. | Read the commit message. See T-7. |
| INV-10 | `os/harness/difftest-app` vectors are byte-identical, unless the same commit adds the divergence-ledger row that authorizes the change. | `git diff --stat -- os/harness/difftest-app/vectors/` empty, or a ledger row in the same diff. |

## 6. Disposition of the four managed-Kubernetes product APIs: **KEEP IN PLACE. NO MOVE.**

`k8s/ports/{cluster-lifecycle,control-plane-host,sla-observability,tenant-quota}-api` stay where they
are, under their current names.

- They match the `ports` face charter the closed capability registry writes, and three of the four
  define real capability traits. The registry is `"closed": true` and its `k8s` row already absorbs the
  four `cloud/managed-k8s-*` dirs — the placement is a recorded founder ruling, not drift.
- `k8s/ports/**` is a glob. `upstream-*` leaves land beside them with no collision, so nothing is
  blocked by leaving them alone.
- A move buys **zero enforcement**. The boundary-partition obligation classifies leaf by leaf; the
  enforcement surface is the region registry, not the path prefix.
- A move costs real co-move debt: four catalog rows (the crate-catalog-coverage policy makes a moved
  crate's row a same-change obligation), `registry/dependency-rationales.json`,
  `registry/stores/registry-store.json`, six `k8s/observability/slos/**` files, per-crate `BUCK`,
  `Cargo.lock`, the `cloud/managed-k8s-*/manifest.json` docs, and a whole-graph buck2. Moving to
  `cloud/managed-k8s-*/` additionally breaks all four depth-3 layer globs.

**The one cheap thing they actually need** is a classification field, not a new address: an
`origin: first-party` row per leaf in the regions registry, so the boundary partition can call them
classified without moving a byte.

**Two open defects recorded, not fixed here:** `registry/stores/registry-store.json` carries the key
`managed-k8s-tenant-quota-api` against a crate actually named `k8s-tenant-quota-api` — a retired
prefix plus an infix the crate does not have; and the `k8s` charter reserves `core/` for the owned
control plane while all four `k8s/core/*` crates are product kernels. Both are out of this lane's
scope and neither is a reason to move anything.

## 7. The cheapest correction available today, and what is deferred

The engine registers no producer. `build/port-engine/core/port-engine-kernel` IS on the base tree —
7 tracked files, 924 lines — but it is an ADR-0637 D4 W0 seam skeleton that declares itself to have
"no source-language front end (no parser, no grammar, no tree-sitter)", no rule DATA and "no corpus
knowledge of any kind", so it emits nothing and owns no path. `specs/port-rules/**` genuinely does
not exist, and `specs/k8s-port/scope.json` reports no package enumeration has run. Presence without
production is why `"regions": []` is still correct. Creating empty
`k8s/ports/upstream-*` crates now is scaffolding for an emitter that cannot run. **Do not create them.**

What is worth doing while the duplication is small — each is one data file, zero crates:

1. `specs/k8s-port/regenerable-regions.json` — declares `"regions": []`, classifies existing `k8s/**`
   and `os/**` leaves `origin: first-party`, and encodes the section 3.1 staging predicate. This is
   what makes a zero-scan result honest instead of vacuous.
2. Two divergence-ledger rows: `DVG-OS-HANDROLLED-K8S-API-SERIALIZERS` (the 16 sites, owner, expiry
   at seam landing, enumerated expected-red ids) and `DVG-OS-DUPLICATE-STATIC-POD-RENDERER` (the two
   independent Pod renderers). The budget is exactly two new rows per wave; the five seeds are exempt.
   **Adding a third row in this wave is rejected by the ledger's own growth policy** — if a unit finds
   a third divergence, it records it in the operations journal and waits for the next wave. That
   rejection has a reader: `ci-k8s-program-docs-gate` loads `baseline_seed_count` and
   `max_new_rows_per_wave` FROM the ledger and reds `R-DOC-K8S-PORT-DIVERGENCE-GROWTH-BUDGET-EXCEEDED`
   past their sum, so the sentence is enforced rather than asserted.
3. The INV-3 equality-frozen ratchet, census 16.

Deferred, with the reason: no seam crate, no `os → k8s` dependency (ADR-638 D6 and the closed
19-tuple substrate DAG), no crate move, no crate deletion.

**Per unit, only when that unit's own diff moved the number** (INV-6): the `adr-citation-closure`
census re-freeze — `files_scanned`, `citation_lines`, and any finding count that genuinely moved —
edited **as text keyed by name**, never round-tripped through JSON, which reformats the whole file.

**Batched to a single Land-phase commit, never per unit:**

- the rebase re-freeze — whatever `dev` landed meanwhile will restate the census, and the ratchet will
  name the observed number at that point (see the ordering note in section 8);
- any other frozen-ceiling re-anchor not caused by a unit's own diff;
- `Cargo.lock` regeneration, if any crate is ever added;
- the operations-journal entry for the wave gate.

## 8. Measured baselines — frozen here, re-derived by every unit

All measured on `origin/dev` @ `5e452bd70`, the tree this document was authored against. The lane was
rebased at Land onto `origin/dev` @ `1d3105277`, and restacked again onto `origin/dev` @ `8857944`
after PR #1621 merged. Every number in this section was re-run at the newest
base; only the `files_scanned` row moved, and that row states the number the gate **observed**, never
an arithmetic one.

**Upstream-Kubernetes emit sites in `os/`: 16** (15 production + 1 test fixture at
`manifest_controller.rs:200`, whose `mod tests` opens at line 196).

```
P='apiVersion: (v1(\\n|"|$)|[A-Za-z0-9.-]*\.k8s\.io/|(apps|batch|autoscaling|policy|extensions)/)'
git grep -hE "$P" -- 'os/**/*.rs' | grep -cvE '^[[:space:]]*//'       # => 16
git grep -cE "$P" -- 'os/core/block-domain/src/controller.rs'         # => exit 1 (negative control)
```

The discriminator is the API **group shape**, never an enumeration of the groups that happened to
exist at census time: a value is upstream Kubernetes when it carries a `<group>/<version>` segment
whose group ends in `.k8s.io` or is one of the five suffix-less upstream groups (`apps`, `batch`,
`autoscaling`, `policy`, `extensions`). A closed allowlist would have been blind to a first
`batch/v1` or `networking.k8s.io/v1` — exactly the growth this invariant exists to catch.

The `git grep` line above is a **reproducer, not the definition**. It enumerates spellings, so it
reads only an unquoted value after `apiVersion: `. The `grep -v` stage is not decoration: the
gate skips any line whose first non-whitespace characters are `//`, because a comment naming an
upstream `apiVersion` is documentation, not emission, and counting it would red the census with a
diagnostic ("consume the seam instead of hand-writing it") that is wholly wrong for someone who
wrote a comment. Without that stage the reproducer and the gate would disagree the first time such
a comment lands. The gate normalizes the value before classifying
it — a quoted key (`"apiVersion":`), a quoted value (`"apps/v1"`, `\"apps/v1\"`, `'apps/v1'`) and
extra spaces all count, which is what stops the census being silently under-inclusive as soon as a
site is written in any of those spellings. On this tree the two agree at 16 with zero per-line
differences across all 373 `os/**/*.rs`, so the widening moved no number; it removed a way for a
future site to hide. Where they can disagree, the gate is authoritative.

The census has a companion red that no reproducer can express: a line whose `apiVersion` value is
BUILT AT RUNTIME (`writeln!(out, "apiVersion: {}", group)`) emits an API group that is not in the
source, so it is unclassifiable by any read of the file — a tokenizer does not help. Counted as
absent, such a site is exactly how a new hand-written upstream serializer lands while this census
stays frozen at 16 and the gate passes. The gate therefore fails closed on it
(`R-DOC-OS-DYNAMIC-APIVERSION-UNCLASSIFIABLE`): make the value static, or consume it through the
`k8s/ports` seam. Zero such sites exist on this tree.

The negative control is the point: `os/core/block-domain/src/controller.rs` carries 16 `apiVersion:`
lines of its own and matches none of them (trap T-1). The exclusion is structural, not incidental:
all 168 `v1alpha1` occurrences in `os/**/*.rs` are **bare**, carrying no group segment, while every
slashed value in the tree is an upstream group — so a slash-shaped predicate cannot swallow Talos
surface.

| Quantity | Value | Note |
|---|---|---|
| `os/` crates | 41 | 40 under `os/core/`, 1 under `os/harness/`. |
| Kubernetes-facing `os/` crates | 8 | All eight cite `siderolabs/talos` paths; none cites `k8s.io/kubernetes`. |
| `os/` → `k8s/` dependency edges | 0 | INV-1, with positive control. |
| `k8s/` → `os/` dependency edges | 0 | INV-2, with positive control. |
| `k8s/ports/` incumbent lines | 1,162 | 289 + 386 + 236 + 251. |
| `divergence-ledger.json` rows | 5 seeds | Growth budget 2 per wave after the seeds. |
| `adr-citation-closure` `files_scanned` | 16,524 → **16,527** | Equality-pinned, and **observed rather than computed**: `check-adr-citation-closure-gate` asserted `observed 16527, frozen 16524` on the restacked tree and the row takes the number it stated. The base rose 16,518 → 16,524 across two later dev tips (PR #1621's port-engine skeleton and its journal), and this lane still adds exactly three files carrying a scanned extension: this document, `docs/programs/k8s-port/operations/W0-F-20260809-os-overlap-archaeology.md`, and `specs/k8s-port/regenerable-regions.json`. `wave-registry.rdoc` has no scanned extension and `divergence-ledger.json` already existed, so neither is in the delta. The tracked-add delta (3) and the observed census delta (+3) agree, which is what distinguishes an ADD from a narrowed scan. |
| `adr-citation-closure` `citation_lines` | 8,896, unchanged | Only a line carrying an ADR **path** enters this census; a bare id in prose costs nothing. The draft of this document scored +2 purely from the two malformed path placeholders in T-5, and repairing them returned it to the base value. |
| `adr-citation-closure` `adr_citation_dangling_path` | 2,002, unchanged | Held after repairing the three findings the draft introduced (T-5). Its being *unchanged* is the proof this document adds no citation debt. |
| `lifecycle-status` `stage_not_declared` | 1,921, unchanged | Shrink-only. Declared `doc_status:` in frontmatter rather than raising it (T-4). |

Two of these numbers were racing other lanes when this section was written: an in-flight census lane
set `files_scanned` to 16,524 and `citation_lines` to 8,898, and a separate PR set `files_scanned` to
16,518. **Do not pre-compute the Land value.** That instruction paid twice. At Land the second had
merged and the first had not, so the arithmetic that looked right when this was drafted would have
been wrong by six. Then PR #1621 merged and moved the base again, 16,518 → 16,524, and the restack
onto that tip moved this lane's own value 16,521 → 16,527 — a delta the branch never computed and
took from the ratchet's own `observed 16527, frozen 16524`. Each time the equality ratchet stated the
observed number and the row above took it. Whichever lane lands next pays the same one-line
re-freeze. That is the ratchet working, not a defect.

## 9. What this lane does not decide

- The Q7 direction (`os/` shared types below both consumers, versus `k8s` consuming an OS-owned
  HostOps seam) is reserved to W0-G by ADR-638 D6. Section 3.2 supplies the measured evidence W0-G
  was told to wait for and stops there.
- Adding `k8s` or `k8s.bootstrap` to `specs/substrate-dependency-dag.json` — a closed exact-19-tuple
  contract that fails closed on any addition. Tracked as W0-C topology coverage.
- Whether the three-way overlap inside `os/` (`kubernetes-domain` 4,291 lines, `kubelet-domain` 1,719,
  `k8s-control-domain` 2,097+192) should collapse. Only `kubernetes-domain` is consumed by
  `difftest-app` and covered by Go-oracle vectors; the other two have no consumer outside their own
  pair. That is a real finding — and it is **not** actionable here, because no ADR, IP or commit
  message explaining the split has been found and `git log --follow` has not been run. Chesterton's
  Fence is not cleared. See INV-9 and T-7.

## 10. Traps — where the obvious translation is subtly wrong

**T-1 — `apiVersion:` is not the discriminator.** Talos's own machine-config documents emit
`apiVersion: v1alpha1` in `block-domain`, `controllers-domain`, `config-docs-domain`,
`extensions-domain` and more — dozens of hits that are *correct Talos surface*, not Kubernetes
surface. A ratchet keyed on the token `apiVersion` would count them, would be wrong by ~40, and would
red on legitimate Talos work. **The discriminator is the API group.** Use the section 8 pattern and
run its negative control.

**T-2 — POSIX ERE has no word-boundary atom.** `git grep -E "\bapiVersion"` matches nothing and exits
1, which reads exactly like a clean negative. Every negative claim in this lane must be paired with a
positive control proving the pattern can match something. Both INV-1 and INV-2 carry one.

**T-3 — a `mod tests` boundary is not a stable classifier.** Site 5 is inside a test module today;
counting "production sites only" needs a parser, and a parser is a bigger surface than the thing it
guards. INV-3 counts **all 16** and records the composition in prose. Do not build the parser.

**T-4 — the sibling gates have opposite right answers.** `adr-citation-closure` is an *equality-pinned
census*: a genuine file addition legitimately moves it and it must be re-frozen. `lifecycle-status`
`stage_not_declared` is a *shrink-only ratchet over debt*: the right answer is to declare
`doc_status:` frontmatter, never to raise the baseline. Getting these backwards buys a green by adding
permanent debt to the exact number the ratchet exists to drive down.

**T-5 — a valid ADR link can still be a finding, and the id parser is greedier than it looks.**
`docs/decisions/` holds only ADR-0700..0709. A citation of any other id written as a path under that
directory is a dangling path against an equality-pinned count; and a line that cites a live apex while
every resolvable id on that line closes onto a *different* apex is a mismatch. Bare ids avoid both.

This trap bit this very document, and the bite is the instructive part. The scanner takes **any** run
of digits after `ADR-` and normalizes short forms, so a *placeholder* — `ADR-06xx` — parses as
`ADR-0006`, and a *regex written inside a check command* — `ADR-0[0-6]` — parses as `ADR-0000`. Both
sat immediately after `decisions/`, so both were read as real path citations of ids that do not
exist: three new `adr_citation_dangling_path` findings, `2002 → 2005`, from a document whose entire
subject is not doing that. The self-check that missed it looked for `docs/decisions/ADR-[0-9]{4}` and
found nothing, because neither placeholder has four digits. **Grep for `decisions/ADR` with no digit
class at all.**

Two corollaries worth keeping. The corpus axes assert *before* the finding axes, so a stale
`files_scanned` hides this whole class until the corpus is re-frozen — which is the ordering argument
behind INV-6. And a bare id in prose is **not** a citation line: only a line carrying an ADR path
enters `citation_lines`, so the bare-id convention costs nothing on either census.

**T-6 — the incumbent ports are not the shape to copy.** `k8s/ports/cluster-lifecycle-api` depends
inward on `k8s/core/cluster-lifecycle-kernel` and on two sibling ports, and the repo's own catalog
records it `role: adapter` with an explicit non-claim. It is a use-case orchestrator that happens to
live in `ports/`. A generated seam crate that imitates it violates C1.

**T-7 — a name match is not a semantic match.** `KubeletConfig` and `KubeletSpec` exist in both
`os-kubernetes-domain` and `os-kubelet-domain`; `FileMode`, `InMemoryFileSink`, `RenderedFile` and
`RenderedOutput` exist in both `os-kubernetes-domain` and `os-k8s-control-domain`. The bodies have
**not** been diffed. Deleting the "duplicate" on the strength of the name is how a lane ships a
silent behaviour change into a corpus whose only oracle covers ~40 vector rows. INV-9 requires the
body-level proof.

**T-8 — replacing a lossy serializer changes bytes, and changed bytes are a divergence.**
`kubelet-domain::spec::render_config_yaml` documents itself as *"not a full YAML serializer"* and emits
five fields of `KubeletConfiguration`. A faithful typed emitter emits more, in a different order.
That will move `os/harness/difftest-app` vectors. A moved vector is **not** a fix to be re-baselined;
it is a divergence that needs a ledger row, and the ledger admits two new rows per wave. INV-10.

**T-9 — the classic release-build traps, in the form they take here.** Go has no debug-only assert. A
Go `if err != nil { return err }` that becomes `debug_assert!` is a check that *disappears in release*
— the port must return an error. Conversely, Go's `int32(x)` truncates silently where Rust's
`x.try_into().unwrap()` **panics**: "improving" a cast during a port changes a silent wrap into a
production crash. `as` truncates like Go; `try_into().unwrap()` does not. Go's `int` is 64-bit on our
targets and `usize` is not, on every target. A port is faithful or it is a divergence; there is no
third category and no silent upgrade.

**T-10 — a MOVE is never just a move.** If any unit relocates a crate: the catalog row moves in the
**same** change, the registry rows move with it, `k8s/observability/slos/**` co-moves, the per-crate
`BUCK` and `Cargo.lock` follow, and the evidence owed is a **whole-graph** buck2 build, not a
package-scoped one. Section 6 refuses the move that prompted this lane precisely so no unit has to pay
this. Note also that `os/` crates appear in **no** `registry/catalog/` row (41 crates, zero rows, all
41 frozen as shrink-only `uncatalogued` debt) — so an `os/` move has no catalog obligation *and* no
catalog coverage, which is worse, not better.

**T-11 — neither the derived crate name nor the API group can confirm an upstream module path.** The
section 3.1 `<dashed>` function is deliberately **lossy**: it drops `pkg/apis/`, `pkg/` and `tools/`
segments. So `k8s.io/client-go/clientcmd/api/v1` — a path that does not exist at the pin — and
`k8s.io/client-go/tools/clientcmd/api/v1` — the one that does — derive the *identical* crate name
`upstream-client-go-clientcmd-api-v1`. A plausible-looking generated name is zero evidence about its
source path, and that is exactly how the original row 8 read as fine. The API group is no better a
witness in the other direction: `apiserver.config.k8s.io/v1` carries at least two kinds whose home
package has moved between minors (row 4). **Confirm the module by `ls-tree` at the pin, and confirm
the kind by its `type` declaration inside that module.** Both are cheap; a blob-less `--depth 1`
fetch of the pin costs under a second.

## 11. Definition of done for one unit

A reviewer holding only the diff applies this. All eleven must hold; any "N/A" must be visibly N/A
from the diff.

1. **Scope.** The diff advances exactly one row of section 3.2, one item of section 7, or one
   invariant of section 5 — and the commit message says which by id.
2. **Charter.** It does not add hand-written upstream-Kubernetes surface. INV-3 count is shown in the
   commit message and equals 16.
3. **No unauthorized edge.** INV-1 and INV-2 hold, each shown with its positive control.
4. **No unauthorized structure.** No crate created, moved or deleted. If one is, T-10 and INV-9 are
   both satisfied *in this commit* and a whole-graph buck2 result is quoted with its `Commands:` line.
5. **Talos preserved.** No deletion of cluster/KubeSpan/etcd/CRI/machined/`v1alpha1` logic. Section 2
   names what is protected.
6. **Vectors.** INV-10: `os/harness/difftest-app/vectors/` unchanged, or a divergence row in the same
   diff and within the two-row wave budget.
7. **Governed corpora.** Any added `.md`/`.json`/`.yaml`/`.toml`/`.rs` carries what INV-7 requires; if
   it moved a census, the same diff re-freezes it as a keyed text edit and the commit message
   attributes the delta to the exact files (INV-6). A unit that added no governed file touches no
   ceiling at all.
8. **Citations.** INV-8: bare ADR ids; the live apex ADR-0704 is what is cited for binding force.
9. **Evidence, not assertion.** Every claim in the commit message that a thing holds is accompanied by
   the command and its literal output. Every negative claim carries a positive control (T-2).
10. **One runnable check.** Non-trivial logic leaves behind the smallest thing that fails if the logic
    breaks. A data-file-only unit satisfies this with the gate run that reads it.
11. **Landing.** Units of this lane commit onto the lane branch `impl/os-k8s-seam-conformance` rather
    than spawning their own branches; the lane lands as a single pull request against `dev` through
    the governance pipeline in `CLAUDE.md` / `docs/AGENTS.md`, which is the authority for admission
    and merge. This item restates no admission rule and confers no exemption from one. Hygiene within
    the shared tree is what is local here: named files only — never `git add -A`, never `git
    stash`/`reset`/`clean`.

## 12. Non-claims

This document does not claim that the engine exists, that a seam crate exists, that any dependency
edge is authorized, that the four product APIs were reviewed for product correctness, or that the
`os/` three-way overlap is safe to collapse. No buck2 build, test or gate run backs any number here;
every repository measurement is `git grep` / `git ls-tree` over `origin/dev` @ `5e452bd70`, and each
carries its reproducer.

Section 3.2's module column is the one measurement taken **outside** this repository: `git ls-tree`
and `git grep` over a blob-less fetch of the upstream pin. It confirms that each module path exists
at the pin and that the kind each site emits is declared inside it. It does **not** claim a
SourceModel enumeration has run, that these modules are the complete set the seam needs, that any of
them resolves to `PORT` under `specs/k8s-port/scope.json`, or that the fields the `os/` string
builders emit match the upstream struct fields — that last is a per-field comparison this document
did not make and which T-8 says will move difftest vectors when someone does.
