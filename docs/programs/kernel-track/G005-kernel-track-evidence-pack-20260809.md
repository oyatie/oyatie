---
doc_class: Program-Evidence-Pack
doc_status: published
entry_id: G005-kernel-track-evidence-pack-20260809
program: kernel-track
run_id: g005-kernel-evidence-20260809
recorded_at: 2026-08-09
terminal_state: measured-not-ruled
decision_status: founder-reserved
---

# G005 — kernel track evidence pack

## What this document is, and what it is not

This document **gathers and presents measurements**. It does **not** rule on the kernel
track, and nothing in it should be read as a decision. The choice between an OSDK-based
greenfield kernel and adopting Asterinas is founder-reserved. No ADR is authored here and
nothing is marked Accepted.

Every figure carries an explicit confidence label:

| Label | Meaning |
|---|---|
| **EXACT** | Directly read off a committed artifact or an upstream API response. Reproducible by re-running the stated command. |
| **LOWER BOUND** | The true value is at least this. Named systematic under-counts are listed with the figure. |
| **UPPER BOUND** | The true value is at most this. Named systematic over-counts are listed with the figure. |
| **COULD NOT DETERMINE** | Not measurable with the evidence available here. What it would take is stated. |

§7 states, for each of the two options, what evidence would make it the **wrong** choice.
That section exists because a comparison that cannot be argued against is advocacy, not
evidence. Read it before acting on anything above it.

## Baseline

| Axis | Value |
|---|---|
| Repository baseline | `origin/dev` @ `885794461` |
| Lane branch | `impl/g005-kernel-evidence` |
| Measured on | 2026-08-09 |
| Upstream sources | `asterinas/asterinas`, `asterinas/kata-containers`, `asterinas/vostd`, `verus-lang/verus` — read via the GitHub API on 2026-08-09 |

**Trust boundary.** All upstream repository content quoted below is third-party DATA. It was
read for measurement only. Nothing in it was executed and nothing in it was treated as an
instruction.

---

## 1. Measurement 1 — the Tier-1 syscall census

The stated threshold: roughly **60–80 distinct syscalls** means an OSDK greenfield kernel is
credible; **200+** means adopt Asterinas.

### 1.1 Which workloads are Tier 1, and why

Tier 1 is not a judgement made in this document. It is a declared, machine-checkable field.
Every µservice manifest declares `pod_runtime_tier`, and the live corpus declares:

```
git grep -h -A1 '"pod_runtime_tier"' origin/dev -- '*/manifest.json' \
  | grep -o '"pod_runtime_tier": *[0-9]' | sort | uniq -c
```

| Tier | Manifests | Meaning |
|---:|---:|---|
| 0 | 4 | tenant-customer untrusted code |
| 1 | **24** | substrate touching tenant data plane |
| 2 | 45 | first-party application |
| 3 | 5 | edge / perf-critical |
| | **78 total** | |

**EXACT.** The Tier-1 set taken for this measurement is those **24** services, listed in §1.2.
This is a stronger basis than the ADR prose, which names a *floor* of seven surfaces; the live
declarations have since grown that floor to 24.

### 1.2 The decisive obstacle: most Tier-1 workloads have no executable

Before any tracing question arises, there is a prior one — does the workload exist?

```
for d in <the 24 tier-1 dirs>; do
  find "$d" -name Cargo.toml -not -path '*/target/*' | wc -l
  find "$d" -name main.rs    -not -path '*/target/*' | wc -l
done
```

| Tier-1 service | Cargo.toml | `main.rs` |
|---|---:|---:|
| oya/intelligence | 78 | 5 |
| tenancy | 22 | 2 |
| audit | 18 | 0 |
| k8s | 18 | 3 |
| secrets | 10 | 1 |
| network | 8 | 0 |
| storage | 8 | 0 |
| compliance | 7 | 0 |
| iac | 5 | 1 |
| observability | 5 | 0 |
| flags | 2 | 1 |
| comms/comms-email, comms/messenger, data/cloud-data, data/ontology, gateway/connector, iam/cloud-iam, iam/consent-graph, iam/identity, intelligence/detection, oya/governance, oya/payments, secrets/kms, storage/imaging | **0** | **0** |

**EXACT.** Of the 24 declared Tier-1 services, **13 have zero Rust crates**, and only **6 have
any binary entry point at all** (13 `main.rs` files in total).

Against the seven surfaces the ADR names as the Tier-1 *floor*, the position is worse:

| ADR-named Tier-1 floor surface | Directory | Executable? |
|---|---|---|
| cloud-iam | `iam/cloud-iam` | **no crates at all** |
| cloud-kms | `secrets/kms` | **no crates at all** |
| messenger MLS keys | `comms/messenger` | **no crates at all** |
| payments | `oya/payments` | **no crates at all** |
| audit-chain | `audit` | 18 crates, **library only, no binary** |
| cloud-secrets | `secrets` | yes — `secrets/facade/kms-operator-app` |
| intelligence transport | `oya/intelligence` | yes — 5 binaries |

**EXACT: 5 of the 7 ADR-named Tier-1 floor surfaces have no executable.**

This is the single most important finding in this document, and it governs how every other
number here should be read. **The syscall surface "our Tier-1 workloads actually require"
cannot be measured today, because most of those workloads do not exist yet.** It is a
forward-looking property of code not yet written, not a present property of a running system.
A figure presented as a measurement of it would be a projection wearing a measurement's
clothes.

### 1.3 What *is* measurable: the framekernel's demand-driven boot set

The repository already contains an owned framekernel with a machine-generated syscall census:
`cloud/cloud-kernel/specs/syscall-coverage-matrix.json`, generated from the live dispatch
tables.

```
python3 -c "import json,collections; m=json.load(open('cloud/cloud-kernel/specs/syscall-coverage-matrix.json'));
print({a: collections.Counter(r['status'] for r in m['matrix'][a]['rows']) for a in m['matrix']})"
```

| Arch | implemented | stub | enosys | total | declared `differential_verified` |
|---|---:|---:|---:|---:|---:|
| aarch64 | 61 | 12 | 245 | 318 | 49 |
| x86_64 | 67 | 12 | 296 | 375 | 55 |

**EXACT (as a reading of the committed artifact).** The aarch64 demand set is
**61 + 12 = 73 distinct syscalls**, and that set boots talos-init plus `svc` to clean
power-off. 73 sits inside the stated 60–80 band — but read §1.4 before drawing anything
from that, because a boot set is not a service set.

The matrix itself separates two demand classes, which is exactly the init-versus-runtime
distinction the census question asks for:

| Demand source | Count (both arches) |
|---|---:|
| `talos-init+svc M1/M2 boot trace` | 61 (aarch64) / 67 (x86_64) |
| `libc-init / process-info surface` — "every glibc/musl binary issues these during init" | 12 |

**EXACT.** The 12-syscall libc-init group is *init-only* by the artifact's own description.
The remaining 61 are boot-sequencer syscalls. **No hot-path classification exists** for any
row, because no service workload has been traced — see §1.6.

### 1.4 A caution about the `differential_verified` figure

The matrix declares `differential_verified: 49` on aarch64. The only committed golden trace is:

```
$ wc -l cloud/cloud-kernel/tests/golden/svc-aarch64.trace
       6
$ grep -o 'syscall [0-9]*' cloud/cloud-kernel/tests/golden/svc-aarch64.trace | sort -u
syscall 172
syscall 173
syscall 220
```

**EXACT: the committed golden trace is 6 lines covering 3 distinct syscall numbers**
(aarch64 asm-generic 172/173/220 = `getpid`/`getppid`/`clone`).

This does not prove the 49 figure is wrong — the differential runs may be reproduced by
tooling rather than by a committed artifact. It does mean **the 49 is not substantiated by
committed evidence in this repository**, and a reader should not treat it as verified without
asking the kernel lane where those traces live. Flagged rather than resolved: chasing it was
out of scope for this pack.

### 1.5 Static lower bound for one Tier-1 Rust service

Since the workloads cannot be traced, the surface was derived statically from what a Tier-1
Rust service *links*. Method: intersect syscall-wrapper symbols referenced in dependency
source with the canonical Linux syscall name table from the coverage matrix, excluding crates
whose purpose is to *declare* the whole table (`libc`, `rustix`, `linux-raw-sys`, the seccomp
family) — a mention there is not evidence any workload issues the call.

The method is stated in full here rather than shipped as a script: no tool is committed for it,
because it produced a bound rather than a number and should not acquire the standing that a
committed generator implies.

| Source | Distinct syscalls | Valid on the aarch64 ABI |
|---|---:|---:|
| Framekernel demand set (implemented + stub) | 73 | 73 |
| Core async-runtime / TLS / net crates (`tokio`, `mio`, `socket2`, `rustls`, `hyper`, `getrandom`, `polling`, …) | 40 | 34 |
| Rust `std` (unix paths, non-excluded platforms) | 75 | 60 |
| **Union** | | **107** |

**LOWER BOUND: 107 distinct syscalls** to run one networked Tier-1 Rust service on aarch64.

The 34 beyond the current framekernel set:

```
accept accept4 chdir chroot connect epoll_create1 epoll_ctl fchmod fchown fdatasync
flock fsync getpeername getsockname getsockopt linkat listen pidfd_open
pidfd_send_signal pipe2 preadv pwritev readv recvmsg sendfile sendmsg setgid
setgroups setsockopt setuid shutdown socketpair utimensat waitid
```

And six syscalls currently present only as **stubs** (fixed/no-op returns) must acquire real
semantics: `fcntl`, `fstat`, `futex`, `getcwd`, `ioctl`, `mprotect`. **A stubbed `futex` that
returns 0 cannot carry a multi-threaded tokio runtime, and a no-op `munmap` leaks address
space in a long-running server** — these are not cosmetic gaps.

**Why 107 is a lower bound, not an estimate.** Four named systematic under-counts:

1. **871 of 1530 locked crates had no cached source** (57%) and were not scanned. EXACT count.
2. **libc-internal syscalls are invisible to source scanning.** glibc/musl `malloc` reaching
   `brk`/`mmap`, the DNS resolver reaching `openat`/`socket`/`sendmmsg`, NSS — none appear as
   `libc::x(` in crate source.
3. **The actual service code does not exist** (§1.2). Postgres drivers, object storage, KMS/HSM
   paths, OpenTelemetry exporters and the business logic itself all add surface.
4. **Only two call syntaxes were matched** (`libc::name(`, `syscall!(name(`). Other indirect
   forms are missed. This under-count is demonstrable: the first pass matched only `libc::name(`
   and returned 33 syscalls with **no `epoll` at all**, because `mio` calls it through a
   `syscall!` macro. Widening the pattern moved the figure 33 → 49 and surfaced the whole epoll
   family. There is no reason to believe the second pattern set is complete either.

### 1.6 Answer against the stated threshold

| Quantity | Value | Label |
|---|---|---|
| Distinct syscalls, our Tier-1 workloads, measured from those workloads | — | **COULD NOT DETERMINE** |
| Distinct syscalls, one networked Tier-1 Rust service, aarch64 | ≥ 107 | **LOWER BOUND** |
| Distinct syscalls, framekernel talos-init + svc boot set, aarch64 | 73 | **EXACT** |
| Split: invoked-at-all vs hot-path vs init-only | init-only group = 12; **hot-path split unavailable** | **COULD NOT DETERMINE** |

Against the threshold: **the lower bound of 107 already exceeds the 60–80 band** at which an
OSDK greenfield kernel was called credible, and the bound is loose in the *upward* direction
only — every named error term adds syscalls. Whether it reaches 200+ is **not determined**
here. The honest summary is that the evidence places the requirement **above the "greenfield
is comfortable" band and short of demonstrating the "adopt Asterinas" band** — that is, in the
ambiguous middle, which is the least convenient answer and the one the evidence supports.

The hot-path versus invoked-at-all distinction matters more than the headline count and is the
part this pack could not supply. Implementation cost is not uniform per syscall: an init-only
`uname` is a constant return, while a hot-path `epoll_wait` or `futex` is a scheduler-coupled
subsystem. **A count without that split systematically flatters whichever option is being
argued for.**

### 1.7 What it would take to get a defensible number

This is the honest "we cannot do this here" result, with its price:

1. **Build the Tier-1 services.** 5 of 7 ADR-named floor surfaces have no executable. Nothing
   downstream is measurable until they do. This is the long pole and it is not a kernel task.
2. **Trace on Linux.** The measurement host here is `aarch64-apple-darwin`; Linux syscall
   tracing is not possible on it. Requires `strace -f -c -U name`, `perf trace`, or a seccomp
   `SCMP_ACT_LOG` profile, on the laptop Talos arm64 cluster.
3. **Trace under representative load, not at startup.** Startup traces produce the init set and
   miss the steady-state set. Requires the service under its own load profile.
4. **Separate the three classes** by counting invocations, not just presence: init-only
   (count stops rising after startup), hot-path (count scales with request rate),
   cold (invoked but rare).
5. **Union across all 24 declared Tier-1 services**, not one representative, because the
   kernel must carry the union.

Estimated as a work item: this is a runtime-tracing exercise gated on service implementation,
not a research question. Until step 1 lands, any Tier-1 syscall census is a projection.

---

## 2. Measurement 2 — VMM validation against the Cloud Hypervisor mandate

### 2.1 The governance position is live, but not where expected

The task names ADR-0338 as a ratified ADR mandating Cloud Hypervisor for Tier 0/1 pods. On
`origin/dev` the actual position is more layered, and the difference matters:

```
git show origin/dev:docs/adr-archive/ADR-0338-pod-runtime-tier-0-to-3.md | grep -E '^(status|superseded_by):'
status: Superseded
superseded_by: [ADR-0701]
```

**EXACT.** ADR-0338 is **archived and Superseded**. Its successor is ADR-0701 (*Live monorepo
capability layout, faces, and reorg doctrine*), status **Accepted**, dated 2026-08-06, which
bulk-supersedes 63 ADRs.

ADR-0701 carries the pod-runtime decision statement forward verbatim, so **the mandate is
live** — Tier 0 and Tier 1 run Kata Containers + Cloud Hypervisor, RuntimeClass
`kata-cloud-hypervisor`, handler `kata-clh`. The conflict is real; it is simply owned by
ADR-0701 rather than ADR-0338.

**Worth flagging to the founder separately:** a *layout and reorg doctrine* ADR is now the home
of the pod-runtime-tier and VMM decision, along with 62 other absorbed decisions. That is a
governance smell independent of the kernel question — it makes the live VMM mandate hard to
find and easy to change by accident. Not this lane's call, but it is what the evidence showed.

### 2.2 Which VMM the Asterinas guest kernel is actually validated under

Upstream evidence, not inference. The Asterinas Kata integration lives in
`asterinas/kata-containers`, and its repo-owned runtime config is unambiguous:

```
gh api repos/asterinas/kata-containers/contents/tools/kata/config/kata-10-container.toml \
  --jq '.content' | base64 -d
```

```toml
[hypervisor.qemu]
enable_debug = true
disable_nesting_checks = true
default_memory = 4096
shared_fs = "virtio-fs"
virtio_fs_daemon = "/opt/kata/libexec/virtiofsd"
```

Corroborated by the config-selection helper, which resolves `configuration-qemu.toml` and
searches only for QEMU binaries:

```
gh api repos/asterinas/kata-containers/contents/tools/kata/kata_config.sh --jq '.content' | base64 -d
```
```
/opt/kata/share/defaults/kata-containers/configuration-qemu.toml
kata_select_qemu_binary_path() {
  local qemu_candidates=(
    /opt/kata/bin/qemu-system-x86_64
    /usr/local/qemu/bin/qemu-system-x86_64
    /usr/bin/qemu-system-x86_64
    ...
```

And by the release workflow, which builds with `BOOT_METHOD=qemu-direct` and emits
`aster-kernel-osdk-bin.qemu_elf` under `ARCHITECTURE: amd64`.

**EXACT: the Asterinas guest kernel is validated under QEMU, on amd64.**

This is worth stating plainly because it contradicts *both* prior readings:

- It is **not Cloud Hypervisor** — so the conflict with the live ADR-0701 mandate is real.
- It is **not Dragonball** either, which the task anticipated. Dragonball appears in the
  upstream Kata README as Kata's optional built-in VMM, but the Asterinas integration does not
  configure it. **The recorded expectation was wrong in a way that made the conflict look
  smaller than it is**: a Dragonball conflict is a swap between two modern Rust VMMs, whereas
  the actual position is that Asterinas is validated only under QEMU.

### 2.3 Cost of each side of the conflict

Presented as costs, not as a recommendation.

**Side A — keep Cloud Hypervisor, port Asterinas to it.**

| Cost | Detail |
|---|---|
| Boot protocol | Asterinas builds `qemu-direct`; Cloud Hypervisor uses PVH/linux boot. New boot path, new OSDK boot method. |
| Device model | virtio-fs daemon path, vsock, virtio-net wiring all currently expressed against QEMU. |
| Validation | Zero upstream CI coverage under Cloud Hypervisor. Every guest-kernel bump becomes our regression risk, not upstream's. |
| Ongoing | The port is not a one-off; it must be re-validated against each upstream Asterinas release. |
| Confidence | **COULD NOT DETERMINE** in engineering days — no comparable port is available to size against. |

**Side B — amend the mandate to permit QEMU for the Asterinas RuntimeClass.**

| Cost | Detail |
|---|---|
| Governance | ADR-0701 is Accepted and bulk-carries 63 decisions; amending it touches a very wide surface. |
| Security posture | ADR-0338's own rationale for Cloud Hypervisor is a smaller attack surface than QEMU. Adopting QEMU for the *most isolated* tiers inverts the reason the tier exists. |
| Density | The ADR already prices Kata at 30–40% pod-density loss and sizes `kata-pool` at 1.5×. QEMU is heavier than Cloud Hypervisor; that factor would need re-deriving. **COULD NOT DETERMINE** by how much. |
| Blast radius | Tier 0 and Tier 1 are precisely the tenant-untrusted and tenant-data-plane surfaces. This is the worst place to widen the hypervisor attack surface. |

**Neither side is costless, and the asymmetry is worth naming:** Side A is an engineering cost
we can bound by trying it; Side B is a security-posture change to the two most sensitive tiers,
which is much harder to reverse once shipped.

---

## 3. Measurement 3 — arm64 status, re-verified today

The 2026-08-03 record states: arm64 is not on Asterinas main; a maintainer `arm-v4` branch is
active; OSDK does not port architectures. Re-verified 2026-08-09 (six days later).

### 3.1 No arm64 branch in the upstream repository

```
gh api repos/asterinas/asterinas/branches --paginate --jq '.[].name' | grep -iE 'arm|aarch'
```
→ no matches. Sanity-checked: the same listing returns `main`, so the pattern can match.

**EXACT: there is no `arm-v4` or other arm branch in `asterinas/asterinas`.** The recorded
"maintainer arm-v4 branch" is not in the org repository. The live arm64 work is a **pull
request from a personal fork**.

### 3.2 The arm64 pull request

```
gh api repos/asterinas/asterinas/pulls/3270 \
  --jq '{number,title,state,created_at,updated_at,commits,additions,changed_files,head:.head.label}'
```

| Field | Value |
|---|---|
| PR | **#3270**, "Add the initial Arm64 support" |
| Head | `wanywhn:arm64-support` — a **personal fork**, not an org branch |
| State | **open**, not draft |
| Opened | 2026-05-24 |
| Last activity | **2026-06-17** |
| Staleness | **53 days** as of 2026-08-09 |
| Size | 29 commits, 80 files, +4475 / −49 |
| Formal reviews | **zero** (`/pulls/3270/reviews` returns empty) |

**EXACT, all fields.**

### 3.3 Why it is stalled

The comment thread carries the reason directly. The contributor (quoted by the maintainer):

> I currently work at a fairly busy company, and my work involves ARM. Therefore, I am unable
> to commit significant long-term effort to maintenance at this point.

Maintainer `lrh2000`, 2026-06-17 — the most recent activity on the PR:

> OK. It's good to know this, and thanks for confirming it. This PR is still very helpful in
> this case, as it will make it much easier for me to complete my previous efforts. I will try
> it out soon.

There is also an unresolved design disagreement in-thread over whether the GICv3 interrupt
controller driver should be implemented in-tree or imported as a third-party crate.

**Assessment (EXACT as to facts, interpretive as to outlook):** arm64 support is an unmerged,
unreviewed, 53-day-stale PR from a contributor who has explicitly stated they cannot maintain
it, with an open design disagreement, and a maintainer intention stated but not yet acted on.
**No landing date is derivable.** Anyone needing a date should treat this as unbounded.

### 3.4 A correction to the recorded position on OSDK

The record states "OSDK does not port architectures". This is right in spirit but wrong as
stated, and the distinction matters:

```
gh api repos/asterinas/asterinas/contents/osdk/src/arch.rs --jq '.content' | base64 -d
```
```rust
pub enum Arch {
    #[serde(rename = "aarch64")]  Aarch64,
    #[serde(rename = "riscv64")]  RiscV64,
    #[serde(rename = "x86_64")]   X86_64,
    #[serde(rename = "loongarch64")] LoongArch64,
}
```

**EXACT: OSDK does enumerate `Aarch64` as a target architecture.** OSDK is a build tool — it
selects a target triple. What is missing is the **OSTD arch backend** for aarch64 (interrupt
controller, MMU, timer, context switch), which is the content of the unmerged PR #3270.

The practical consequence is unchanged and should be stated in the corrected form:
**OSDK being able to target aarch64 does not give you an aarch64 kernel.** Selecting OSDK for
a greenfield kernel means writing that arch backend ourselves — which is also precisely what
`cloud/cloud-kernel` has already done, since its aarch64 matrix is populated and its
`rust-toolchain.toml` pins `aarch64-unknown-none-softfloat`.

### 3.5 Why this weighs disproportionately

The laptop Talos cluster is a **permanent arm64 CI substrate**. Consequences, stated as facts:

- The Asterinas Kata bundle is released **amd64 only** (§5). There is no artifact to run there.
- The Asterinas guest kernel has **no arm64 support on main** to build from.
- `cloud/cloud-kernel` already carries a populated **aarch64** dispatch table (61 implemented,
  12 stub) and pins aarch64 bare-metal targets.

**On the arm64 axis specifically, the owned framekernel is ahead of upstream Asterinas today.**
That is one axis of several and is not by itself a ruling.

---

## 4. Measurement 4 — Verus / vostd, priced

Priced, not evaluated. Whether the price is worth paying is not this document's call.

### 4.1 Licences — the recorded position was conflated

```
gh api repos/verus-lang/verus --jq '{full_name,license:.license.spdx_id,pushed_at}'
gh api repos/asterinas/vostd  --jq '{full_name,license:.license.spdx_id,pushed_at,description}'
```

| Project | Licence | Note |
|---|---|---|
| `verus-lang/verus` | **MIT** | The verifier toolchain. **EXACT.** |
| `asterinas/vostd` | **MPL-2.0** | "Ongoing formal verification for Asterinas OSTD". **EXACT.** |

**The recorded "MPL-2.0" applied to Verus is wrong**; MPL-2.0 is vostd's licence, and Verus
itself is MIT. This is a correction in the *permissive* direction: MIT is the easier of the two
to consume, and vostd's MPL-2.0 is file-level copyleft, which matters only for modified vostd
files, not for code that merely depends on it.

Neither licence is a blocker. Both are already-accepted classes in this repository.

### 4.2 The real cost is the toolchain, and it is a third one, not a second

```
gh api repos/verus-lang/verus/contents/rust-toolchain.toml --jq '.content' | base64 -d
```
```toml
[toolchain]
channel = "1.97.1"
components = [ "rustc", "rust-std", "cargo", "rustfmt", "rustc-dev", "llvm-tools" ]
```

Against what this repository already pins:

| Pin | Channel | Components |
|---|---|---|
| Repository root `rust-toolchain.toml` | **1.97.1** | `rustfmt`, `clippy` (profile `minimal`) |
| `cloud/cloud-kernel/rust-toolchain.toml` | **nightly-2026-02-28** | `rust-src`, `rustfmt`, `clippy`, `llvm-tools` + bare-metal targets |
| Verus | **1.97.1** | `rustc`, `rust-std`, `cargo`, `rustfmt`, **`rustc-dev`**, `llvm-tools` |

**EXACT, and the most useful single fact here: Verus pins the same channel the repository root
already pins — 1.97.1.** There is no channel divergence today. Two consequences, in opposite
directions:

- **Cheaper than expected now.** No third channel download; the delta is components, not channel.
- **Not durable.** The alignment is coincidence, not contract. Verus moves its pin on Verus's
  schedule. The repository already runs a second toolchain for `cloud-kernel`
  (`nightly-2026-02-28`), so adopting Verus makes this a **third** toolchain the moment the pins
  diverge — and it will.

Priced components and their consequences:

| Cost axis | Measurement | Label |
|---|---|---|
| Extra rustup components | `rustc-dev` + `rust-std` + `llvm-tools` beyond the root `minimal` profile. `rustc-dev` ships the compiler's own crates and is one of the largest components in the distribution. | **EXACT** as to which components; **COULD NOT DETERMINE** as to MB/build-minutes — not measured, would need a clean-image `rustup component add` timing on the CI runner image. |
| Z3 solver | Verus requires a Z3 binary alongside the toolchain — a non-Rust dependency in a repository whose doctrine is owned-Rust. | **EXACT** that it is required; version pin not measured. |
| Verification wall-clock | **COULD NOT DETERMINE.** No verification run was performed. Verus proof checking is SMT-bound and typically dominates compile time on verified crates. Would require running vostd's own suite to size. | **COULD NOT DETERMINE** |
| Release cadence | Verus ships **rolling daily releases**: `release/rolling/0.2026.08.09.92f466f` (2026-08-09), `release/0.2026.08.02.b677dd5`, `release/0.2026.07.27.31579f0`. | **EXACT** |
| Who maintains the pin | No owner exists today. The repository has no Verus pin and no lane that would own one. This would be a new standing ownership obligation. | **EXACT** (absence verified: no `verus` reference in any `rust-toolchain.toml`) |
| What breaks when it lags | A daily-rolling upstream against a repository that pins by exact version means the pin goes stale continuously. When Verus's channel moves off 1.97.1, verified crates stop building until someone bumps and re-verifies. Proof breakage is not the same as compile breakage: **a lagged pin can leave proofs silently unchecked rather than loudly failing**, unless CI asserts that verification actually ran. | Interpretive, grounded in the EXACT cadence above |

### 4.3 Adopting Asterinas does not deliver the verification

```
gh api 'search/code?q=repo:asterinas/asterinas+verus' --jq '.total_count'  → 0
gh api 'search/code?q=repo:asterinas/asterinas+vostd' --jq '.total_count'  → 0
```

Sanity-checked so this negative is not a silent search failure:
`...+ostd` → **598**, `...+filename:Cargo.toml` → **108**. The index is live; the zeros are real.

**EXACT: `asterinas/asterinas` main references neither Verus nor vostd.**

vostd is a **separate, out-of-tree, explicitly "ongoing"** verification effort (52 stars, last
push 2026-08-08 — active, but small and independent). It verifies OSTD, the framekernel
substrate — not the Asterinas kernel as shipped.

This materially changes how the verification argument should be read in either direction:
**"adopt Asterinas and get formal verification" is not supported by the evidence.** Adopting
Asterinas gets you an unverified kernel plus the *option* to track a separate MPL-2.0
verification project that is not yet integrated upstream. That option is equally available
without adopting Asterinas.

---

## 5. Measurement 5 — integration, not invention

The claim under test: `asterinas/kata-containers` already builds a Kata bundle with Asterinas
as the guest kernel. **Verified true, with one blocking qualification.**

### 5.1 The repository

```
gh api repos/asterinas/kata-containers \
  --jq '{default_branch,pushed_at,fork,parent:.parent.full_name,license:.license.spdx_id,archived}'
```

| Field | Value | Label |
|---|---|---|
| Default branch | `asterinas` (the **only** branch) | **EXACT** |
| Licence | Apache-2.0 | **EXACT** |
| Last push | **2026-08-09** — today | **EXACT** |
| Archived | no | **EXACT** |
| `fork` / `parent` | **`false` / `null`** | **EXACT** |

The `fork: false` is a real finding and not a triviality: this is **not a tracked GitHub fork
of `kata-containers/kata-containers`**. It is a hard import. Upstream Kata changes do not flow
in through the fork relationship; rebasing onto new upstream Kata releases is a manual
maintenance act by the Asterinas org, and its cadence is not visible from the fork graph.

### 5.2 It really does release consumable artifacts

```
gh api 'repos/asterinas/kata-containers/releases?per_page=3' \
  --jq '.[] | "\(.tag_name) | \(.published_at) | \([.assets[].name]|join(", "))"'
```

| Tag | Published | Assets |
|---|---|---|
| `3.28.0-20260809-asterinas` | 2026-08-09 | `kata-static-3.28.0-asterinas-amd64.tar.zst`, `.SHA256SUMS`, `.manifest.json`, `.summary.md` |
| `3.28.0-20260808-asterinas` | 2026-08-08 | same shape |
| `3.28.0-20260807-asterinas` | 2026-08-07 | same shape |

**EXACT: daily releases, Kata 3.28.0, a static tarball with SHA256SUMS and a manifest.** CI runs
a nightly matrix (`guest_kernel: [linux, asterinas]`), so the Asterinas guest kernel is tested
against a Linux control on every run — genuinely good engineering hygiene, and real evidence
the integration works rather than merely existing.

### 5.3 What consuming it would take

From the repo-owned tooling (`tools/kata/kata_env.sh install`, `run_kata.sh`), the mechanism is
small and well-defined:

1. Pull `kata-static-<ver>-asterinas-amd64.tar.zst`, verify against `.SHA256SUMS`.
2. Unpack to `/opt/kata` (shim `containerd-shim-kata-v2`, `virtiofsd`, guest kernel
   `aster-kernel-osdk-bin.qemu_elf`).
3. Install `configuration-qemu.toml` with `[hypervisor.qemu]` and the virtio-fs daemon path.
4. Register a containerd runtime handler and a Kubernetes `RuntimeClass`.
5. Point Tier 0/1 pods at it.

**This is genuinely integration rather than invention, and it does materially change the cost
side of the comparison** — that part of the premise holds. But two blockers sit on top of it:

- **amd64 only.** Every release asset, the workflow `ARCHITECTURE: amd64`, and all seven QEMU
  binary candidate paths (`qemu-system-x86_64`) are x86_64. **There is no artifact for the
  arm64 Talos substrate**, and per §3 there is no upstream arm64 kernel to build one from.
- **QEMU, not Cloud Hypervisor** (§2). Consuming the bundle as-published means running Tier 0/1
  under QEMU, against the live ADR-0701 mandate.

Consuming the bundle unchanged is cheap. Consuming it **on our substrate and under our mandate**
requires the arm64 port (§3, unbounded) and the Cloud Hypervisor port (§2.3, unsized). Those two
costs, not the bundle, are where the integration argument actually has to be settled.

---

## 6. What this pack did not cover

Stated so absence is not mistaken for a negative finding:

1. **No runtime tracing of any workload.** Measurement host is `aarch64-apple-darwin`; Linux
   syscall tracing is not possible on it (§1.7).
2. **No build or boot of Asterinas.** No performance, density, or cold-start figure is offered
   for either option. All comparative performance claims here are quoted from the ADR, not
   re-measured.
3. **No Verus verification run.** Verification wall-clock is COULD NOT DETERMINE (§4.2).
4. **Cloud Hypervisor + Asterinas was not attempted.** The port cost in §2.3 is unsized.
5. **871 of 1530 locked crates were not scanned** for §1.5 (no cached source).
6. **The `differential_verified: 49` claim was flagged, not chased** (§1.4).
7. **No cost-per-engineer-day figures.** Every "cost" here is a described obligation, not a
   priced one.
8. **`riscv64` and Intel TDX were not examined**, though both appear in Asterinas's tier list.

---

## 7. What would make each option the WRONG choice

This section is the point of the document. Each option is argued **against**, including against
the reading the sections above might suggest.

### 7.1 Evidence that would make "OSDK greenfield kernel" the wrong choice

1. **A Tier-1 trace materially above ~150 distinct syscalls, with a large hot-path fraction.**
   §1.5's lower bound is 107 with four named under-counts. If the real number lands near 200+
   — especially if `epoll`/`futex`/`mm` dominate the hot path — the greenfield premise fails on
   its own stated threshold. **This is the single measurement most likely to overturn the
   greenfield case, and it is precisely the one this pack could not make.**
2. **A Cloud Hypervisor port of Asterinas turning out cheap.** §2.3 could not size it. If it is
   weeks rather than quarters, the ADR-0701 conflict largely dissolves and the integration
   option gets much stronger.
3. **PR #3270 landing.** If arm64 merges to main, §3 — currently the strongest single argument
   against Asterinas — evaporates, and with it the "we are ahead on arm64" position in §3.5.
4. **LTP conformance proving unreachable.** The framekernel's own spec sets the bar at passing
   the Linux Test Project suite and running arbitrary unmodified musl/glibc static binaries.
   If that bar proves out of reach at our staffing, the 61-implemented starting point is a
   local maximum, not a head start.
5. **The stub-to-real cost being underestimated.** Six syscalls are stubs that must become real
   (§1.5). If `futex` and `mprotect` alone turn out to be scheduler- and MMU-coupled
   multi-month subsystems, the "73 already done" figure was never comparable to "73 needed".
6. **Sustained single-lane staffing.** A greenfield kernel that only one lane can work on has
   the same maintainer-bus-factor problem that §3.3 identifies as fatal for upstream's arm64
   branch. That criticism cuts both ways and should be applied to ourselves.

### 7.2 Evidence that would make "adopt Asterinas" the wrong choice

1. **arm64 not landing on a bounded timeline.** §3 shows an unreviewed, 53-day-stale PR from a
   contributor who has said they cannot maintain it. On a permanent arm64 CI substrate, adopting
   an amd64-only kernel means either the substrate or the kernel choice has to give.
2. **The QEMU dependency proving structural rather than incidental.** If Asterinas's device
   model, boot path and virtio wiring are coupled to QEMU deeply enough that Cloud Hypervisor
   support is a rewrite, §2.3 Side A is unbounded and Side B is a security-posture change to the
   two most sensitive tiers.
3. **A Tier-1 trace landing at or below ~80 distinct syscalls.** Symmetrically to §7.1.1: if the
   real workload need is genuinely small, adopting a general-purpose Linux-ABI kernel imports an
   enormous surface for no benefit, and the greenfield case is straightforwardly correct.
4. **Verification not arriving.** §4.3 shows Asterinas main references neither Verus nor vostd.
   If the verification story is a significant part of the adoption case, that case is currently
   unsupported and would need vostd to actually integrate upstream.
5. **The hard-import maintenance model degrading.** `asterinas/kata-containers` is not a tracked
   fork (§5.1). If upstream Kata rebases lag, we inherit an increasingly divergent Kata whose
   security fixes arrive on someone else's schedule.
6. **MPL-2.0 file-level copyleft colliding with modification needs.** Asterinas is MPL-2.0. If
   adoption requires extensive in-file modification — which arm64 and Cloud Hypervisor support
   both imply — those modified files carry MPL obligations. Not a blocker, but it is a real
   constraint on an owned-stack doctrine and was not weighed here.

### 7.3 Evidence that would make the framing itself wrong

Both options assume the kernel is the thing to decide next. §1.2 found that **5 of 7 ADR-named
Tier-1 surfaces have no executable**. If that is the binding constraint, then the syscall census
is unmeasurable, the kernel requirement is unknowable, and **a kernel decision taken now would
be taken on a projection either way**. Deferring until Tier-1 services exist is a third option
this document is obliged to name, though it is equally not this lane's to choose.

---

## 8. Reproduction

Every figure above is reproducible from `origin/dev` @ `885794461` plus GitHub API reads on
2026-08-09. In-repo commands are inline in each section. Upstream reads use `gh api` with the
exact endpoints shown. The static syscall extraction method is described in §1.5 and its two
regex patterns and exclusion list are stated there; its known error terms are enumerated rather
than corrected, because correcting them does not produce a defensible number — only tracing
real workloads does (§1.7).
