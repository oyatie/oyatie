---
doc_class: Program-Operations-Journal
doc_status: published
entry_id: W0-F-20260809-os-overlap-archaeology
wave: W0-F
run_id: os-overlap-archaeology-20260809
incident_class: chesterton-fence-archaeology
recorded_at: 2026-08-09
terminal_state: no-op
---

# W0-F-20260809-os-overlap-archaeology

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-09) |
|---|---|---|
| Repository baseline | branch `impl/os-k8s-seam-conformance` @ `8fd606a658130e29fc94075128ab7977c7775d1e`, merge-base `origin/dev` @ `1d31052774ef580553a5ff81014849bb38d6e327` | Authored at `e70cdecb3a44844231229508f291391210d5d423` over merge-base `5e452bd70449b50cc66e63ffb9253adfcd7fc96e`, while `origin/dev` had already advanced. The Land rebase moved the lane onto that tip and rewrote the authoring commit to the sha above — the id in this row is the one reachable from the landed branch, because the original is not. `git diff --stat 5e452bd70 1d3105277 -- os/` is empty, so the rebase changes no `os/` byte and every measurement below stands as taken. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Carried from the seam mapping record. Not re-resolved here; this entry parses no upstream source. |
| Engine | `build/port-engine/*`, v0 — path absent on this tree | Not in force. This entry emits nothing and wires nothing. |
| Neutral rule pack | `specs/port-rules/**`, v0 — path absent on this tree | Not in force. No rule ID is touched. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | None. Every measurement is `git log`, `git grep`, `git ls-tree`, `git diff --no-index` and `wc -l` over tracked Rust and TOML in this tree | Measurement instrument only. No Go was parsed and no Go package layout is asserted. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This entry emits no receipt. |
| Program authority | ADR-0704 (live apex). ADR-0637 and ADR-0638 are archived provenance | Archaeology record only. Authorizes no move, no merge, no deletion. |

## Entry identity

- **Stable entry ID:** `W0-F-20260809-os-overlap-archaeology`.
- **Wave:** W0-F, registry ordinal 5, `completed=false` at the time of writing.
- **Run ID:** `os-overlap-archaeology-20260809`.
- **Incident class:** `chesterton-fence-archaeology`. Not an incident; a deliberate attempt to
  discharge one specific unknown that the seam mapping recorded as unactionable.
- **Recorded at:** 2026-08-09.
- **Question this run was opened to answer:** section 9 of the seam mapping records the three-way
  overlap between `os/core/kubernetes-domain`, `os/core/kubelet-domain` and
  `os/core/k8s-control-domain` as a real finding left unactionable, because no ADR, IP or commit
  message explaining the split had been found and `git log --follow` had not been run. Chesterton's
  Fence was therefore not cleared. This run runs that history and body-diffs the colliding types.

## Scope and inputs

**Subjects.** Three crates, measured with `wc -l` over tracked `*.rs`:

| Crate | Package name | `src/` lines | `tests/` lines | Total |
|---|---|---:|---:|---:|
| `os/core/kubernetes-domain` | `os-kubernetes-domain` | 4,291 | 0 | 4,291 |
| `os/core/kubelet-domain` | `os-kubelet-domain` | 1,719 | 0 | 1,719 |
| `os/core/k8s-control-domain` | `os-k8s-control-domain` | 2,097 | 192 | 2,289 |

**Colliding type names investigated,** as named by the unit: `KubeletConfig`, `KubeletSpec`,
`FileMode`, `InMemoryFileSink`, `RenderedFile`, `RenderedOutput`.

**Consumer edges,** by `git grep -ln <package> -- '*/Cargo.toml' '*/BUCK'` with the crate's own two
files subtracted:

| Crate | External consumers | Who |
|---|---:|---|
| `os-kubernetes-domain` | 1 | `os/harness/difftest-app` |
| `os-kubelet-domain` | 1 | `os/core/k8s-control-domain` |
| `os-k8s-control-domain` | **0** | nobody |

The zero is a negative claim, so it carries a positive control: the same pattern
`git grep -ln os-k8s-control-domain` over the whole tree returns six paths (`Cargo.lock`,
`ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json`, `cloud/cloud-os/manifest.json`,
`docs/programs/k8s-port/seam/os-k8s-seam-mapping.md`, and the crate's own `BUCK` and `Cargo.toml`),
so the pattern demonstrably matches. Restricted to manifests it matches only the crate itself.

## Judgment

### J-1. The split has no recorded rationale, and now demonstrably cannot have one in this repository

`git log --follow` on a directory does not follow renames — Git's `--follow` is file-scoped — so run
on the three crate directories it terminates at `24b917f5d` for all three and shows only `A` lines.
Run on each `src/lib.rs` it walks the whole chain. All three chains are identical in shape:

| Step | Commit | Date | `kubernetes-domain` | `kubelet-domain` | `k8s-control-domain` |
|---|---|---|---|---|---|
| Birth | `3786d41e8` | 2026-06-05 | `A stack/kuberos/talos-kubernetes/src/lib.rs` | `A stack/kuberos/talos-kubelet/src/lib.rs` | `A stack/kuberos/talos-k8s-control/src/lib.rs` |
| Reorg | `2adcbc2ef` | 2026-06-07 | `R100` → `stack/operating-system/…` | `R100` | `R100` |
| Consolidation | `2fadcaec1` / `aee2297fe` | 2026-06-09 | `A cloud/cloud-os/crates/cloud-os-kubernetes-domain/…` | `A …-kubelet-domain/…` | `A …-k8s-control-domain/…` |
| Homing | `24b917f5d` | 2026-07-26 | `R100` → `os/core/kubernetes-domain/…` | `R097` | `R099` |

Every step after birth is a pure rename: `R100`, `R100`, `R100`, then `R100`/`R097`/`R099`. **No
commit in the history of any of the three ever changed the boundary between them.** The split was
born whole and has been carried mechanically ever since.

Birth commit `3786d41e8` is titled *"Checkpoint: M1+M2+P4(x2APIC/GICv3)+SMP verified; paused at P5;
establish parallel lanes"*. It adds 22,260 files and states in its own body: *"First checkpoint of
the cloud-native Rust monorepo (previously 0 commits, fully untracked)."* It introduces 43
`stack/kuberos/talos-*` crate directories at once. Its message is entirely about kernel milestones
and lane charters; it says nothing about how the Kubernetes surface was divided.

**This is the decisive archaeological fact.** The pre-checkpoint tree was untracked, so the reasoning
that produced the three-way split was never committed anywhere and is not recoverable from this
repository. The two later commits that touch these crates are explicit that they made no such
decision: `2fadcaec1` describes slice 2 as *"the 25 pure (no external deps, lib-only) crates that
depend only on the foundational kernel crate"* and `aee2297fe` describes slice 3 as *"the 7 crates
whose internal deps all resolve within slices 1-2"*. `kubelet-domain` and the other two landed in
different slices **because of their dependency counts**, for stacked-PR reviewability — not because
anyone judged them to be different domains. Both commits say the rename `talos-<name>` →
`cloud-os-<name>-domain` was applied *"per the BNF 13-suffix + oya- prefix + manifest-hygiene
gate rules"*, i.e. mechanically.

**Verdict on the fence: the fence has no builder on record.** That is a strictly different state
from "unexplained". An unexplained fence might have a reason someone forgot to write down; this one
was erected in a bulk import of an untracked tree, and the repository can prove that no subsequent
author ever revisited it. Chesterton's Fence is *not* thereby cleared for removal — see J-4 — but the
specific unknown section 9 recorded is now closed: **searching further for a rationale commit is
futile and should not be repeated.**

### J-2. `rendered.rs` is a verbatim fork, and it has diverged

`os/core/kubernetes-domain/src/rendered.rs` (230 lines) and
`os/core/k8s-control-domain/src/rendered.rs` (241 lines) define the same four types —
`FileMode`, `RenderedFile`, `RenderedOutput`, `InMemoryFileSink` — plus the same `FileSink` trait.
`git diff --no-index` reports 38 insertions and 27 deletions. Body-level, the delta is:

| Delta | Direction | Consequence |
|---|---|---|
| `K8sError::Render` → `ControlError::Render` | rename only | Different error enums; the two crates cannot exchange a `Result` without conversion. |
| `FileMode::EXEC` (`0o755`) | present in `kubernetes-domain`, **absent** in `k8s-control-domain` | Dead either way: `git grep FileMode::EXEC -- os/` matches nothing, while the positive control `git grep -cn FileMode::SECRET -- os/` matches five files. The fork dropped a constant nobody used. |
| `RenderedFile::as_str` | **only** in `k8s-control-domain` | Capability the other copy lacks. |
| `RenderedOutput::get` | **only** in `k8s-control-domain` | Capability the other copy lacks. |
| `#[must_use]` on 9 accessors | **only** in `k8s-control-domain` | Lint posture only. |
| test `overwrite_counts_as_write_not_path` | **only** in `kubernetes-domain` | A regression test asserting `count()` counts paths while `write_count()` counts writes. The fork **deleted a regression test** and kept the code it guards. |

Read as a whole this is one file copied and then edited on one side. The only thing the fork removed
that was not dead is a test. Nothing here reflects a domain distinction; the two copies model the
same filesystem boundary for the same host paths.

### J-3. `KubeletConfig` / `KubeletSpec` are two independent ports of the *same* upstream controller, and they do not agree

Both crates state their upstream provenance in a module doc comment, and they name the same thing:

- `os/core/kubernetes-domain/src/kubelet.rs:3` — *"Mirrors Talos
  `internal/app/machined/pkg/controllers/k8s` kubelet controllers (`KubeletConfigController` /
  `KubeletSpecController`)"*.
- `os/core/kubelet-domain/src/spec.rs:3` — *"Mirrors
  `internal/app/machined/pkg/controllers/k8s.KubeletSpecController`"*.

They are not the same code. `KubeletConfig` has 7 fields in `kubernetes-domain` and 9 in
`kubelet-domain`, with `extra_args` as `Vec<(String, String)>` versus `BTreeMap<String, String>`
(the latter de-duplicates, the former does not). `kubelet-domain` additionally models
`cgroup_driver`, `extra_mounts`, `credential_providers`, `skip_node_registration` and
`default_runtime_seccomp_enabled`; `kubernetes-domain` additionally models `node_name`,
`taints: Vec<String>` and `static_pod_path: Option<String>`. `KubeletSpec` has 3 fields versus 6.

The divergence is **behavioural, not cosmetic**. Both types exist to render the kubelet's argv, and
the two renderers emit different flag sets for the same node:

| Flag | `kubernetes-domain::KubeletSpec::render` | `kubelet-domain::KubeletSpec::render` |
|---|:--:|:--:|
| `--hostname-override` | yes | yes |
| `--kubeconfig` | yes | yes |
| `--config` | yes | yes |
| `--cert-dir` | yes | yes |
| `--container-runtime-endpoint` | yes | yes |
| `--pod-manifest-path` | conditional on `static_pod_path` | unconditional |
| `--register-node=false` | yes | yes |
| `--node-ip` | **no** | yes |
| `--bootstrap-kubeconfig` | **no** | yes |
| `--cgroup-driver` | **no** | yes |
| `--register-with-taints` | **no** | yes (taints are a `Vec<String>` field never rendered on the other side) |

`kubernetes-domain`'s renderer also drops the taints it stores: `KubeletConfig::from_node_config`
pushes `node-role.kubernetes.io/control-plane:NoSchedule` into `taints` on a control-plane node, and
`KubeletSpec::render` never reads that field. A control-plane node rendered through that path
registers **untainted**. Whether that is reachable in production depends on wiring this entry does
not assert — see the coverage gap in J-4 — but as written the two ports disagree about whether a
control-plane node is tainted, which node IP it advertises, and which cgroup driver it uses.

The signature difference is the reason the drift is invisible: `kubernetes-domain` renders from a
`&KubeletConfig` alone, while `kubelet-domain` renders from
`(&KubeletConfig, &Nodename, &[NodeAddress], &[NodeTaint])`. The second knows about node identity and
addressing; the first does not, and cannot.

### J-4. There is no equivalence oracle over any of this

ADR-0638 D5 charters the Talos corpus by naming its **landed** `os/harness/difftest-app` vectors.
Measured on this tree:

- `os/harness/difftest-app/Cargo.toml:19` depends on `os-kubernetes-domain` and on neither of the
  other two. `kubelet-domain` and `k8s-control-domain` have **no** differential coverage at all.
- Within `differential.rs`, `grep -c Kubelet` returns 3, and all three hits are
  `SecretK8sCert::ApiServerKubeletClient` — a certificate name. `grep -n "fn .*kubelet"` returns
  nothing.

So **zero differential vectors exercise `KubeletConfig` or `KubeletSpec` in any of the three
crates.** The one crate with an oracle has an oracle that does not touch the overlapping surface, and
the crate with zero consumers (`k8s-control-domain`, 2,289 lines) also has zero vectors.

This is why the fence stays up even though J-1 shows nobody put it there deliberately. INV-9 of the
seam mapping requires a body-level equivalence proof plus the `git log --follow` output in the commit
message before any deletion. The `git log --follow` half is discharged by J-1. The equivalence half
**fails**: J-2 and J-3 are the proof that the copies are *not* equivalent, and J-4 shows there is no
harness that could tell us which behaviour is the correct one. Deleting or merging any of the three
today would be choosing between two argv renderings with no evidence about which matches upstream
Talos.

### J-5. Incidental finding — `KubeletConfig` is duplicated four times, not twice

The unit named two colliding `KubeletConfig` definitions. There are four:

| Definition | What it models |
|---|---|
| `os/core/config-v1alpha1-domain/src/kubelet.rs:33` | The `machine.kubelet` sub-tree of the Talos v1alpha1 machine config. 9 fields. |
| `os/core/machine-config-domain/src/machine.rs:62` | The same `machine.kubelet` sub-tree. 3 fields — `image`, `extra_args`, `cluster_dns` — a strict subset of the previous, with a byte-identical `extra_arg` accessor. |
| `os/core/kubelet-domain/src/config.rs:180` | The *derived* kubelet config (J-3). |
| `os/core/kubernetes-domain/src/kubelet.rs:22` | The *derived* kubelet config (J-3). |

The first two are Talos v1alpha1 **schema**, genuinely Talos-specific and inside what the program
charters as the Talos corpus; they are a distinct duplication from the one this run was opened on,
and the shorter one looks like an abandoned partial copy. The second two are the derived controller
output. So one upstream pipeline — v1alpha1 schema → derived config → rendered spec — is modelled
twice at every stage across four crates. This is recorded, not actioned; it is outside this unit.

### What is genuinely Talos and what is chartered Kubernetes, for these three crates

Stated because the goal asks for it explicitly. Nothing below authorises a change.

| Surface | Classification | Basis |
|---|---|---|
| v1alpha1 `machine.kubelet` schema (`config-v1alpha1-domain`, `machine-config-domain`) | **Talos** | Talos's own config format. No upstream Kubernetes API group. |
| `KubeletConfig` / `KubeletSpec` derivation and argv rendering | **Talos** | Ports `internal/app/machined/pkg/controllers/k8s`, a Talos-owned controller. The *flags* are kubelet's, but the derivation policy — which flags Talos owns, which it refuses to let users override via `PROTECTED_ARGS`, how cluster DNS is derived from the service CIDR — is Talos policy, not upstream Kubernetes API surface. |
| `rendered.rs` `FileSink` / `RenderedFile` / `FileMode` | **Talos** | A host-filesystem write boundary. Not an upstream API type. |
| Anything in these crates emitting an upstream `apiVersion` for a `k8s.io` API group | **Chartered Kubernetes** | Already ledgered by the seam mapping's 16-site table; this run adds no site and removes none. |

The three crates in this entry are therefore Talos surface with a duplication problem, not
hand-written upstream Kubernetes surface. The seam does not fix them. Consolidating them is ordinary
`os/` maintenance, gated on an oracle that does not yet exist.

## Change disposition

**No code was changed. No crate was deleted, merged, moved, renamed or rewired.** This unit is
RECORD-ONLY by construction and the deliverable is this finding.

**No rule change.** `specs/port-rules/**` and `specs/k8s-port/rules/**` do not exist on this tree, so
there is no rule ID to touch. Reason recorded explicitly rather than omitted, per the entry schema.

Files this run writes, and why each one:

| File | Why |
|---|---|
| `docs/programs/k8s-port/operations/W0-F-20260809-os-overlap-archaeology.md` | This entry. |
| `docs/programs/k8s-port/wave-registry.rdoc` | Its `operations_entries=` field for W0-F, so the entry is reachable from the registry rather than only from the directory listing. |
| `governance/check/adr-citation-closure/adr-citation-closure-policy.json` | `measured.files_scanned` re-freeze forced by adding one tracked `.md`. See the Gate result. |

## Gate result

**Red gate 1 — `governance/check/adr-citation-closure`, `files_scanned` equality pin.** Expected, not
a defect. That census is pinned by *equality*, and it is asserted before any finding count, so any
tracked file with a scanned extension moves it and the gate fails until the ceiling follows in the
same change. Root cause is the design of the ratchet, which is deliberate: it is what distinguishes a
genuine add from a narrowed scan. Repaired by re-deriving the observed value from the live tree with
the new file staged and writing that number back — **not** by adding one to the frozen value, because
the seam mapping records two other lanes concurrently setting this field to different values, so
arithmetic on the previous number would silently adopt whichever lane happened to land first. The
ratchet stated the number itself: *"files_scanned: observed 16522, frozen 16521"*. `16522` was written
back as text keyed by name, and `citation_lines`, `adr_citation_dangling_path` and every other ceiling
were left untouched and stayed green — which is the cross-check that this entry adds a file and no
citation debt.

`citation_lines` is unchanged. This entry cites ADR ids as **bare text only** and writes no
`docs/decisions/ADR-…` path, and `scan_line` routes a bare id to `context` rather than `cited`; a
non-authority-surface line with an empty `cited` is skipped before it can become a `CitationLine`.
This entry is not in `authority_surfaces`.

**Red gate 2 — none.** `ci/facade/lifecycle-status` `stage_not_declared` is held at its frozen 1,921
by declaring `doc_status: published` in this entry's front matter — `published` is a declared stage in
`specs/lifecycle-configs/doc-status-lifecycle.json`. The baseline is shrink-only debt and was **not**
raised.

**`ci/facade/k8s-program-docs` — green, with one observation.** `R-DOC-BASELINE-HEADER-MISSING`
requires a `## Baseline version header` section containing `Repository baseline`, one of
`Kubernetes upstream` / `Upstream Kubernetes pin`, and all six axis tokens; this entry carries them.

The observation is a **latent gap in that gate, recorded rather than fixed**: `load_wave_registry`
only reads `operations_entries` for rows whose `completed=true`. Every row in the registry is
`completed=false` today, so the `operations_entries` value this run writes is parsed for field
validity and then **never resolved to a file**. A typo'd, missing or empty journal path on an
incomplete wave is not caught. The linkage becomes load-bearing only when W0-F is marked complete.
Not repaired here: this unit does not own `ci/facade/` — U3 does — and marking a wave complete to
force the check would be a false completion claim.

**The gap is measured, not inferred.** Replacing this entry's registry value with
`operations_entries=THIS-FILE-DOES-NOT-EXIST.md` and re-running
`buck2 test //ci/facade/k8s-program-docs:ci-k8s-program-docs-gate` yields
`Tests finished: Pass 1. Fail 0.` — a journal path resolving to no file passes.

**Non-vacuity of the two green results above** — each was proved by mutating this entry and observing
the gate turn red, then restoring it:

| Mutation to this file | Gate | Observed |
|---|---|---|
| Delete `` `toolchain_digest` `` from the Baseline version header | `ci-k8s-program-docs-gate` | `Fail 1` — `live_k8s_program_document_corpus_is_green_and_nonempty` panics. |
| Delete `doc_status: published` from the front matter | `ci-lifecycle-status-gate` | `Fail 1` — `doc-status-lifecycle Observed { artifacts: 2702, violations: {"stage_not_declared": 1922, …} }`, `baseline_regression: doc-status-lifecycle/stage_not_declared grew 1921 -> 1922`. |
| (none — the census moved on its own) | `check-adr-citation-closure-gate` | `files_scanned: observed 16522, frozen 16521`. |

The middle row is the one worth keeping: it proves this entry is inside the 2,702-document
`doc-status` corpus, so its green is a declared stage rather than a document the lane cannot see.

## Reproduction

Every command below is run from the repository root of the branch tree. All are read-only.

```
git log --follow --oneline --name-status --find-renames -- os/core/kubelet-domain/src/lib.rs
git log --follow --oneline --name-status --find-renames -- os/core/kubernetes-domain/src/lib.rs
git log --follow --oneline --name-status --find-renames -- os/core/k8s-control-domain/src/lib.rs
git log --follow --oneline --name-status --find-renames --all -- stack/operating-system/talos-kubelet/src/lib.rs
git log -1 --format=%B 3786d41e8
git log -1 --format=%B 2fadcaec1
git log -1 --format=%B aee2297fe
git show --name-only --format= 3786d41e8 | wc -l

git diff --no-index -U2 os/core/kubernetes-domain/src/rendered.rs os/core/k8s-control-domain/src/rendered.rs
grep -oE '\-\-[a-z-]+=' os/core/kubernetes-domain/src/kubelet.rs | sort -u
grep -oE '\-\-[a-z-]+=' os/core/kubelet-domain/src/spec.rs | sort -u

git grep -ln os-k8s-control-domain -- '*/Cargo.toml' '*/BUCK'
git grep -ln os-kubernetes-domain  -- '*/Cargo.toml' '*/BUCK'
git grep -ln os-kubelet-domain     -- '*/Cargo.toml' '*/BUCK'
git grep -c Kubelet -- os/harness/difftest-app/tests/differential.rs
git grep -n FileMode::EXEC -- os/
git grep -cn FileMode::SECRET -- os/
```

**Instrument caveat, recorded because it changed a result.** The first attempt at the type search was

```
git grep -nE '(pub )?(struct|enum|trait|type) KubeletConfig\b' -- os/
```

which returned **nothing at all**. `git grep -E` is POSIX ERE, which has no `\b` atom, so the pattern
silently matched zero files and would have been reported as "no collision exists". The working form
drops `\b` and uses an explicit terminator class, and every negative claim in this entry is paired
with a positive control that proves the pattern can match. This is the same trap the seam mapping
records as T-1/T-5 in a different guise.

**Resources.** Negligible: no build, no test binary, no network. Peak working set is one
`git show --name-only` over a 22,260-file commit. No runner, cluster or external input is required.
The only external reference is the upstream Kubernetes pin, which is carried from the mapping record
and not re-resolved.

## Review

- **Reviewer role:** none at time of writing. This entry is authored by the U4 implementation lane
  and has had no separate reviewer pass. Authoring and review are separate passes by doctrine, and the
  review pass belongs to the lane's Land phase.
- **Verdict:** unknown — not yet reviewed.
- **Review evidence reference:** none exists yet. Recorded as `unknown` explicitly rather than
  omitted, per the entry schema.
- **Resolved findings:** none to resolve; this run repairs nothing.
- **Deferred findings, each with its owner:**
  - The four-way `KubeletConfig` duplication (J-5) — outside this unit's scope.
  - `os-k8s-control-domain`: 2,289 lines, zero consumers, zero differential vectors. Disposition is
    ordinary `os/` maintenance, not seam work, and is gated on J-4.
  - The `kubernetes-domain` control-plane taint that is stored and never rendered (J-3). Whether it is
    reachable was **not** determined; determining it needs the caller trace this run did not do.
  - The `k8s-program-docs` registry-linkage gap on incomplete waves (Gate result).

## Terminal state

`no-op`. Nothing was fixed because this unit fixes nothing by construction; the durable evidence is
this entry plus the reproducers above. The blocker on acting is stated in J-4: no equivalence oracle
exists over the overlapping surface, and INV-9's body-level equivalence half fails rather than being
unattempted.

## Graduation links

- Seam mapping record: `docs/programs/k8s-port/seam/os-k8s-seam-mapping.md`, section 9 (the finding
  this run was opened against) and INV-9 (the deletion precondition this run does not satisfy).
- Program authority: ADR-0704, live apex. ADR-0637 and ADR-0638 are archived provenance; the D5 clause
  naming the landed difftest vectors is the one J-4 measures against.
- Prescription: **none authored.** A prescription would encode a repeatable remedy, and this run
  produced no remedy — it produced the evidence that the remedy is currently unknowable. Recorded here
  so the absence is a judgment rather than an omission.
- Doctrine: none touched.
- Rule change: none. See the Change disposition.
