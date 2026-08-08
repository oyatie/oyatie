---
doc_class: Program-Census-Record
doc_status: published
entry_id: census-ownership-escape-20260808
census_lane: ownership-escape
run_id: go-rust-rule-corpus-census-20260808
recorded_at: 2026-08-08
measurement_basis: go1.26.5-escape-analysis-plus-go-ast
---
# Census: ownership and escape surface of the pinned Kubernetes corpus

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-08) |
|---|---|---|
| Repository baseline | `origin/dev` @ `5e452bd70449b50cc66e63ffb9253adfcd7fc96e` | Lane base; verified with `git rev-parse HEAD` after `checkout -B census/go-ownership-retry origin/dev`. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Verified against the working corpus before counting (see §1). Apache-2.0. |
| Engine | `build/port-engine/*`, v0 — unbuilt | Not in force. This census is an input to sizing, not engine output. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored | Not in force. No rule is authored or implied by this record. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Ad-hoc `go build -gcflags=-m` plus `go/parser`; no SourceModel | Measurement instrument only; not an admitted extractor. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This census emits no receipt. |
| Program authority | [ADR-0637](../../../adr-archive/ADR-0637-owned-deterministic-go-to-rust-port-engine.md) / [ADR-0638](../../../adr-archive/ADR-0638-mechanically-maintained-kubernetes-rust-port.md) | Measurement record only; authorizes nothing. |

## Non-claims

This record measures. It does not author a rule, admit a snapshot, approve a wave, or assert
that any translation is achievable. Every number below is a measurement of upstream Go source
at the pin, not a statement about the port engine. The upstream corpus was treated strictly as
data to be counted; no instruction, comment, or directive found in it was acted on.

---

## 1. Provenance and denominators

The corpus was already cloned and pin-verified; it was not re-cloned. Verification first:

```sh
CORPUS=/private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/222702d1-4719-4175-a349-71e41cd88f0d/scratchpad/k8s-corpus
git -C "$CORPUS" rev-parse HEAD
# 756939600b9a7180fc2df6550a4585b638875e67   -> matches specs/k8s-port/upstream-pin.json
```

Three denominators exist and are **not** interchangeable. Every percentage below names which
one it uses.

| Denominator | Count | Command |
|---|---:|---|
| D1 — all `.go` files | 16,941 | `find "$CORPUS" -name '*.go' -type f \| wc -l` |
| D2 — excluding `vendor/` | 12,587 | `find "$CORPUS" -name '*.go' -type f -not -path '*/vendor/*' \| wc -l` |
| D3 — excluding `vendor/` and `_test.go` | 9,573 | `find "$CORPUS" -name '*.go' -type f -not -path '*/vendor/*' -not -name '*_test.go' \| wc -l` |

All three reproduce the shape stated in the lane brief exactly. The AST instruments below parse
**9,523** files, which is D3 minus the 50 `testdata/` fixtures they deliberately skip
(`find "$CORPUS" -path '*/testdata/*' -name '*.go' -not -path '*/vendor/*' -not -name '*_test.go' | wc -l` → 50).
9,523 + 50 = 9,573; the reconciliation is exact, so D3 is the AST denominator with a named
50-file exclusion, not an approximation.

One further split governs the interpretation of nearly every finding:

| Class | Count | Share of D3 | Command |
|---|---:|---:|---|
| Machine-generated (`// Code generated … DO NOT EDIT.`) | 3,326 | 34.7% | `grep -rl 'Code generated .* DO NOT EDIT' --include='*.go' "$CORPUS" --exclude-dir=vendor --exclude='*_test.go' \| wc -l` |
| Hand-written | 6,247 | 65.3% | D3 minus the above |

**Roughly one in three non-test source files in this corpus is machine-generated.** Generated
files are produced by a small number of generators, so they are internally near-homogeneous:
they contribute enormous *occurrence* counts against a tiny number of *shapes*. Reporting any
figure below without this split produces a number that is arithmetically true and
programme-sizing-useless. Every headline figure is therefore reported split.

---

## 2. Meta-finding: what running this census actually cost

This is reported because it is evidence for the separate open question of whether a Go
toolchain is tolerable inside the port pipeline. The lane's experience *is* the data.

**It worked, first time, with nothing broken.** That is the honest headline and it was not the
expected outcome.

| Fact | Measured value |
|---|---|
| Toolchain | `go1.26.5 darwin/arm64`, already present at `/opt/homebrew/bin/go` (`go version`) |
| Install cost | Zero — no install was performed or needed |
| Whole-corpus build, main module | exit 0, **53 s** wall |
| Whole-corpus build, all dependencies | exit 0, **53 s** wall |
| Diagnostic output, main module | 626,178 lines / 66 MB |
| Diagnostic output, all dependencies | 1,618,013 lines / 176 MB |
| Build cache produced | 4.5 GB and 4.6 GB — **one cache per `-gcflags` configuration** |
| Network access required | None — the corpus vendors its dependencies |

Commands, runnable as written:

```sh
cd "$CORPUS"
export GOFLAGS=-mod=vendor
GOCACHE=/tmp/census/gocache  go build -gcflags=-m     ./...  2>/tmp/census/escape.stderr
GOCACHE=/tmp/census/gocache2 go build -gcflags=all=-m ./...  2>/tmp/census/escape-all.stderr
```

What this means for the pipeline question, stated as observation rather than recommendation:

- Kubernetes v1.36.1 **builds clean on darwin/arm64** with `-mod=vendor` and no network. The
  platform risk anticipated in the brief did not materialize. Exit code 0 on both runs.
- 53 s for a whole-corpus escape analysis is cheap enough to run per-commit.
- The cost is **disk, not time**. `GOCACHE` is keyed on compiler flags, so an escape-analysis
  build cannot reuse an ordinary build cache: budget ~4.6 GB per distinct flag set, and expect
  a cold rebuild whenever the flags change.
- The output is a 176 MB unstructured text stream that must be parsed with regexes. It is a
  diagnostic channel, not an API, and it is not covered by Go's compatibility promise. A
  pipeline depending on it is depending on an unstable surface.

**What broke:** nothing in the toolchain. Four things broke in *my own* measurement pipeline —
two in the instruments, caught before the numbers below were produced, and two in the *prose*,
caught only by independent review of the first published revision. All four are recorded in §8,
because they are the kind of defect this census exists to avoid shipping, and because the fact
that the prose errors survived my own checking and needed an outside reader is itself a finding
about how this work should be reviewed.

---

## 3. Coverage — measured, not extrapolated

`-gcflags=-m` applies only to packages named on the command line; `-gcflags=all=-m` applies to
dependencies too. The main-module-only run reached no `staging/` or `vendor/` code at all, which
would have excluded most of the corpus's API machinery. **All findings below therefore use the
`all=-m` run.**

| Coverage measure | Value | Basis |
|---|---:|---|
| Packages analysed carrying `k8s.io/…` | 2,417 | `grep '^# ' escape-all.stderr \| sort -u \| grep -c 'k8s.io'` |
| Corpus package directories | 2,889 | `find . -name '*.go' -not -path './vendor/*' -not -name '*_test.go' -not -path '*/testdata/*' \| xargs -n1 dirname \| sort -u \| wc -l` |
| **Package coverage** | **83.7%** | 2,417 / 2,889 |
| Corpus files carrying ≥1 diagnostic | 6,939 | unique paths in the corpus-filtered diagnostic set |
| **File coverage (lower bound)** | **72.9%** | 6,939 / 9,523 |

File coverage is a **lower bound and understates true coverage**: a file that compiled fine but
contained nothing the compiler had cause to report emits no line, and is indistinguishable here
from a file that was never compiled. Package coverage (83.7%) is the better figure.

The uncovered remainder is explained, not unexplained. Both builds exited 0, so no package
failed. The gap is:

1. **Platform-excluded files.** Building on darwin/arm64 excludes Linux- and Windows-guarded
   files by build constraint: 153 files carry a `linux` constraint and 130 carry a `windows`
   one, plus 70 `*_linux.go` and 62 `*_windows.go` by filename convention. For a corpus whose
   production target is Linux, this is a **material** gap concentrated in exactly the code most
   likely to be ownership-hostile — kubelet, container runtime, cgroup and networking paths.
   **A Linux run is required before these numbers are used to size Linux-specific work.**
2. Test-only packages, excluded by `go build`.
3. Files with no reportable construct.

Item 1 is the largest known threat to validity in this record.

---

## 4. Finding 1 — the escape ratio

### 4.1 A correction to the framing, stated first

The brief describes the escape ratio as "the single most important number… values that do not
escape map to move/borrow mechanically, and only the escaping remainder needs Arc/Arc<Mutex>."
The first half is right. **The second half does not follow, and the distinction matters more
than the ratio.**

Go's escape analysis answers *"must this value outlive its stack frame?"*. Rust's ownership
question is *"how many owners does this value have?"*. These are different questions:

| Go verdict | Correct Rust consequence |
|---|---|
| does not escape | stack value, or a borrow `&T` / `&mut T` |
| escapes to heap, single owner | `Box<T>`, or simply an owned `T` that is moved |
| escapes to heap, **aliased**, read-only | `Arc<T>` |
| escapes to heap, **aliased**, mutated | `Arc<Mutex<T>>` |

Escape analysis distinguishes row 1 from rows 2–4. **It cannot distinguish row 2 from rows 3–4,
because it never asks about aliasing.** Heap allocation is not shared ownership. Treating
"escapes to heap" as "needs `Arc`" is precisely the naive translation the brief warns about, and
it would be wrong for the large majority of escaping values, which have exactly one owner.

So the escape ratio sizes **stack-vs-heap**, and §5 addresses sharing separately and far less
confidently.

### 4.2 Parameters — the borrow question

This is the ratio that maps most directly onto Rust, because a Go parameter that does not escape
is a parameter that can become `&T` with no lifetime gymnastics and no refcount.

All counts are **unique source positions** (`file:line:col`), corpus-only (stdlib and `vendor/`
removed).

| Verdict | Positions | Share | Rust consequence |
|---|---:|---:|---|
| `x does not escape` | 96,823 | 49.3% | `&T` — direct borrow |
| `leaking param: x to result …` | 11,391 | 5.8% | `fn f<'a>(x: &'a T) -> &'a U` — lifetime-parametric borrow, still no refcount |
| `leaking param content: x` | 38,009 | 19.3% | contents outlive the call — owned field, or borrow of the contents |
| `leaking param: x` | 50,289 | 25.6% | genuinely escapes — owned `T`, `Box`, or shared |
| **Sum** | **196,512** | 100% | |

**This table is a sum, not a partition, and the 100% is therefore nominal.** The four position
sets overlap: a single `file:line:col` can carry more than one verdict (typically a parameter
reported both as leaking to a result and as leaking its content). Measured directly:

| Set | Positions | Command |
|---|---:|---|
| Sum of the four rows | 196,512 | `wc -l` over the four `.pos` files |
| **Union of the four rows** | **194,727** | `cat param_{noescape,to_result,content,leak}.pos \| sort -u \| wc -l` |
| Double-counted | 1,785 (0.9%) | 196,512 − 194,727 |
| …of which within the three leak flavours | 1,776 | 99,689 − 97,913 (`cat param_{to_result,content,leak}.pos \| sort -u \| wc -l` → 97,913) |
| Union of the two borrow-compatible rows | 108,213 | `cat param_{noescape,to_result}.pos \| sort -u \| wc -l` |

> **55.1% of analysed parameters (108,214 of 196,512) are borrow-compatible with no reference
> counting whatsoever** — 49.3% as plain `&T`, a further 5.8% as lifetime-parametric borrows.
> On the deduplicated union basis the same figure is **55.6%** (108,213 / 194,727). The 0.5 pp
> difference is immaterial to every conclusion drawn from it; both are reported so a reader is
> not left to assume a partition that does not exist. Every other percentage in this section is
> stated on the sum basis.

Splitting `leaking param` by flavour is what produces this. Collapsing all three leak flavours
into one bucket — the obvious reading of the raw output — would report **50.7% leaking**
(99,689 / 196,512; 50.3% on the union basis, 97,913 / 194,727), against a borrowable 49.3%.
Splitting them moves borrowability from 49.3% to 55.1%, because the lifetime-parametric 5.8% is
in fact borrowable. Commands:

```sh
grep -E ' leaking param: [^ ]+ to result ' corpus.txt | cut -d' ' -f1 | sort -u | wc -l   # 11391
grep -E ' leaking param content'           corpus.txt | cut -d' ' -f1 | sort -u | wc -l   # 38009
grep -E ' leaking param: [^ ]+$'           corpus.txt | cut -d' ' -f1 | sort -u | wc -l   # 50289
grep -E ' [A-Za-z_][A-Za-z0-9_]* does not escape$' corpus.txt \
  | grep -vE ' (append|make|new|len|cap|copy|delete) does not escape$' \
  | cut -d' ' -f1 | sort -u | wc -l                                                       # 96823
```

The builtin exclusion is load-bearing: `append` is a bare identifier and would otherwise be
miscounted as a parameter. This was an actual defect in an earlier pass (§8).

**Generated code borrows better than hand-written code, and the gap is large:**

| | Total | Generated | Hand-written |
|---|---:|---:|---:|
| Borrow-compatible params | 108,214 | 55,486 | 52,728 |
| All params | 196,512 | 86,832 | 109,680 |
| **Borrow rate** | **55.1%** | **63.9%** | **48.1%** |

Sizing consequence: the hand-written 48.1% is the number that should drive planning. Generated
code will be regenerated by an owned generator, not translated, so its favourable 63.9% is
largely irrelevant to the rule corpus.

### 4.3 Allocations — the stack-vs-heap question

| Verdict | Positions |
|---|---:|
| Allocation escapes to heap | 206,271 |
| Allocation does not escape | 9,063 |
| Local moved to heap | 12,563 |
| `append` sites | 15,570 (12,838 escaping) |

Taken at face value that is **95.8% escaping**, which reads as catastrophic. It is not, and the
face value is misleading, because 45.0% of the "escaping allocations" — 24.8% pattern-matched
plus a residue bucket classified by assumption, see §6.1 — are not allocations in any
sense a Rust port would reproduce (§6.1). Excluding interface-boxing artifacts:

| | Escaping | Non-escaping | Escape rate |
|---|---:|---:|---:|
| All reported sites | 206,271 | 9,063 | 95.8% |
| Real allocations only | 113,369 | 7,751 | **93.6%** |

Either way the answer is the same and it is unwelcome: **Kubernetes allocates on the heap almost
everywhere.** This is not a measurement artifact — it is what the program is. Kubernetes is an
API-object-graph system whose working set is `*v1.Pod`-shaped records threaded through
informers, caches and queues. Values genuinely do outlive their frames.

The mitigating fact is §5: heap-allocated is not shared, and `Box<T>` costs nothing that Go's
heap allocation did not already cost.

### 4.4 Directionality of these bounds

This asymmetry is essential to using the numbers correctly:

- **`does not escape` is effectively exact and is a safe LOWER bound on borrowability.** The
  compiler proved the value does not outlive the frame. If it says stack, it is stack.
- **`escapes to heap` is an UPPER bound on genuine heap need.** Escape analysis is conservative
  and flow-insensitive across function boundaries; it heap-promotes whenever it cannot prove
  otherwise. Some fraction of these values would not require heap allocation under Rust's more
  precise, whole-function borrow reasoning.

Therefore: 55.1% borrow-compatible parameters is a **floor**, not a ceiling. The true figure is
higher by an amount this method cannot measure.

---

## 5. Finding 2 — mutable vs immutable sharing (`Arc<Mutex<T>>` vs `Arc<T>`)

### 5.1 I could not determine this, and the reason is structural

**Escape analysis contains no aliasing information and no mutability information.** It cannot
distinguish a heap value with one owner from a heap value with ten, and it says nothing about
whether anyone writes through those references. The question as posed — of the escaping values,
how many are shared mutably — is **not answerable from the data the brief directs me to
collect**, and no amount of care with the `-m` output will make it answerable.

Answering it properly requires whole-program alias/points-to analysis over a typed IR: Go's
`golang.org/x/tools/go/pointer` or an equivalent built on `go/types` and SSA. That is a
substantially larger instrument than this census, and it does not scale trivially to a corpus
this size. **Recording that as the requirement is the honest output here.** I have not
substituted a syntactic proxy and called it the answer.

What follows are **bounds**, explicitly labelled, that constrain the answer without giving it.

### 5.2 Lower bound — author-declared shared-mutable state

A struct with an embedded `sync.Mutex`/`RWMutex`/`sync.Map` is one whose author declared
concurrent mutation. These are the near-certain `Arc<Mutex<T>>` cases.

| Measure | All code | Hand-written only |
|---|---:|---:|
| Named struct types declared | 10,821 | 8,263 |
| …with a sync-primitive field | **448 (4.1%)** | **437 (5.3%)** |

That the two counts are nearly identical (448 vs 437) is itself informative: **generated code is
essentially free of concurrency primitives**; all but 11 of the corpus's locked types are
hand-written.

### 5.3 Upper bound — types with any mutating method

A type is counted as mutating if it has ≥1 pointer-receiver method that assigns through the
receiver (`r.f = …`, `r.f++`, `r.m[k] = v`). This is a **lower bound on mutation** (it misses
mutation via free functions taking `*T`, and mutation of a field's pointee) but serves as an
**upper bound on `Arc<Mutex>` candidates**, since most mutable types have a single owner and map
to `&mut T`, not to a lock.

| Measure | All code | Hand-written only |
|---|---:|---:|
| Types with ≥1 mutating method | 3,631 (33.6%) | **1,152 (13.9%)** |
| Types with methods, none mutating | 4,965 | 3,915 |
| Types with no methods at all | 2,225 | 3,196 |
| Free functions taking `*T` and mutating it | 4,260 | 948 |

**2,479 of the 3,631 mutating types — 68.3% — are generated.** They are overwhelmingly
apply-configuration builders (`WithFoo(v) *FooApplyConfiguration`). That shape maps to an
owned-`self` Rust builder and never needs a lock. Removing them cuts the upper bound from 33.6%
to 13.9%.

### 5.4 The concurrency surface is small

| Measure | All code | Hand-written only |
|---|---:|---:|
| `go` statements | 751 | 751 |
| Channel send statements | 375 | 375 |
| Function literals | 19,324 | 10,277 |

751 goroutine launches across 9,523 files. Sharing across threads — the only thing that forces
`Arc` over `Rc` or plain ownership — originates at a strikingly small number of sites.

### 5.5 What the bounds jointly say

Assembling the bounds, with the uncertainty kept explicit:

> `Arc<Mutex<T>>` is required for **at least 437** hand-written types (declared locks) and for
> **at most 1,152** (all hand-written types with any mutating method) — that is, between **5.3%
> and 13.9%** of hand-written struct types. Not for the 93.6% of allocation sites that escape to
> the heap.

The gap between those two bounds is where the real answer lies, and closing it needs the alias
analysis of §5.1.

The programme-relevant conclusion: **the failure mode the brief names — `Arc<Mutex>` everywhere,
with its performance tax and deadlock surface — would arise from mistaking the escape ratio for
a sharing ratio.** A translation keyed on escape verdicts would emit on the order of 10⁵
`Arc<Mutex>` sites. The measured need is bounded by ~10³ types. That is **two orders of
magnitude** of avoidable damage, and avoiding it is a rule-design decision, not a code-generation
detail.

---

## 6. Finding 3 — distribution and shapes (rule-corpus sizing)

Rules are sized by **shapes**, not occurrences. This section is the most directly useful output.

### 6.1 Escaping-allocation shapes

**Read the coverage claim carefully.** The classes below are an **authored taxonomy**, not a
discovered one: `census/shapes.sh` is a hand-written `awk` cascade of 14 patterns terminating in
a bare `else` that sweeps everything unmatched into a residue class. **Its 100% coverage is true
by construction, not by measurement.** Fourteen of the fifteen rows are genuine measured shapes;
row 2 is the residue and is analysed separately below. Corpus-only, deduplicated by position.

| Rank | Shape class | Sites | Share | Cum. |
|---:|---|---:|---:|---:|
| 1 | `&T{…}` — address of composite literal | 52,960 | 25.67% | 25.7% |
| 2 | **residue — unmatched subject expression** (the `else` default; assumed to be a named expression boxed into `interface{}`) | 41,655 | 20.19% | 45.9% |
| 3 | string literal boxed into `interface{}` | 24,003 | 11.64% | 57.5% |
| 4 | `[]T{…}` slice literal | 16,856 | 8.17% | 65.7% |
| 5 | synthesised variadic `...` slice | 14,721 | 7.14% | 72.8% |
| 6 | closure (`func` literal) | 13,404 | 6.50% | 79.3% |
| 7 | call result boxed into `interface{}` | 11,858 | 5.75% | 85.1% |
| 8 | `new(T)` | 7,149 | 3.47% | 88.5% |
| 9 | `map[K]V{…}` literal | 6,624 | 3.21% | 91.7% |
| 10 | value composite literal | 6,172 | 2.99% | 94.7% |
| 11 | `make([]T, …)` | 4,860 | 2.36% | 97.1% |
| 12 | result temporary `~rN` | 2,251 | 1.09% | 98.2% |
| 13 | `make(map[K]V, …)` | 1,889 | 0.92% | 99.1% |
| 14 | `make(…)` other | 1,204 | 0.58% | 99.7% |
| 15 | numeric literal boxed | 665 | 0.32% | 100.0% |
| | **Total** | **206,271** | | |

**Top 3 classes cover 57.5%; top 5 cover 72.8%; all 15 cover 100% by construction.** Fourteen
measured classes cover **79.81%** of escaping sites (206,271 − 41,655 = 164,616). The remaining
20.19% is the residue, and **the residue is not closed:**

```sh
# reproduce the residue bucket: everything the 14 patterns do not match
awk '{s=$0; if (s ~ /^&/ || s ~ /^new\(/ || s ~ /^make\(/ || s ~ /^\[\]/ || s ~ /^map\[/ \
   || s=="func literal" || s=="... argument" || s ~ /^"/ || s ~ /^-?[0-9]/ || s ~ /^~r/ \
   || s ~ /\{\.\.\.\}$/ || s ~ /\{\}$/ || s ~ /\(/) next; print}' final/esc.subj.u > O_bucket.txt
wc -l           < O_bucket.txt   # 41655 sites
sort -u O_bucket.txt | wc -l     # 10125 distinct subject expressions
grep -cE '^[A-Za-z_][A-Za-z0-9_]*$' O_bucket.txt                  # 22679 bare identifiers
grep -E  '^[A-Za-z_][A-Za-z0-9_]*$' O_bucket.txt | sort -u | wc -l #  3427 distinct
grep -cE '\.'                       O_bucket.txt                  # 18094 selector chains
grep -E  '\.'                       O_bucket.txt | sort -u | wc -l #  6190 distinct
```

| Residue composition | Sites | Distinct subjects |
|---|---:|---:|
| Bare identifiers (`wireType`, `key`, `ns`) | 22,679 | 3,427 |
| Field-selector chains (`pod.ObjectMeta.Name`) | 18,094 | 6,190 |
| Other (index expressions, method values, unary) | 882 | 508 |
| **Total residue** | **41,655** | **10,125** |

> **Correction to the obvious reading: this dimension is NOT closed at 15 shapes.** Fourteen
> shapes are measured and closed over 79.81% of escaping sites. The remaining 20.19% is a
> classifier default containing 10,125 distinct subject expressions, and nothing here
> establishes how many *rule* shapes that resolves to. Sizing the rule corpus at "15" would
> understate it by an unknown amount concentrated in exactly the fifth of sites this instrument
> did not classify. Closing it needs a typed AST pass over the subject expressions, not a
> larger regex cascade — the residue is dominated by bare identifiers whose shape is determined
> by their *type*, which this instrument does not have.

The homogeneity of the residue's head is nonetheless informative: `wireType` (4,418 sites),
`wire` and `fieldNum` (1,254 each) are protobuf-unmarshaller locals — 6,926 sites, 16.6% of the
residue, from three identifiers in generated code.

**The critical split within this table.** Classes 2, 3, 5, 7 and 15 — 92,902 sites, **45.0% of
all escaping allocations** — are not ownership decisions at all. They are Go boxing arguments
into `interface{}` for `fmt.Errorf`, `klog`, and friends, plus compiler-synthesised variadic
slices. In Rust the equivalent code (`format_args!`, `&dyn Display`, a slice of borrows)
allocates nothing and owns nothing.

**This 45.0% inherits the residue's uncertainty**, because class 2 (41,655 sites) is the residue
bucket, classified as boxing by assumption rather than by pattern. Two spot-checks against the
source support the assumption and neither proves it across all 10,125 distinct subjects:

```sh
grep -m1 ': wireType escapes to heap$' final/corpus.txt
#   staging/src/k8s.io/apimachinery/pkg/util/intstr/generated.pb.go:282:55
#   -> return fmt.Errorf("proto: wrong wireType = %d for field Type", wireType)
grep -m1 ': pod\.ObjectMeta\.Name escapes to heap$' final/corpus.txt
#   staging/src/k8s.io/kubectl/pkg/cmd/util/podcmd/podcmd.go:59:86
#   -> return nil, fmt.Errorf("pod %s/%s does not have any containers", pod.Namespace, pod.Name)
```

The floor that does not depend on the assumption is classes 3, 5, 7 and 15 alone: **51,247
sites, 24.8%**, all pattern-matched. So the boxing share is **24.8% (measured) to 45.0%
(measured plus the residue assumption)**.

> **A port rule that treats "escapes to heap" as an ownership signal would be wrong on at least
> 24.8% and probably ~45% of escaping sites.** These must be recognised and discarded before any
> ownership rule fires. The conclusion is unchanged across the whole range.

Non-escaping allocations are differently shaped, which corroborates the split — slice literals
lead at 31.4%, and the same five boxing classes fall to **14.5%** (vs 45.0%). Full table, all
twelve classes present (three of the fifteen classes have zero non-escaping sites):

| Shape class | Sites | Share |
|---|---:|---:|
| `[]T{…}` slice literal | 2,842 | 31.36% |
| `&T{…}` composite literal address | 2,277 | 25.12% |
| *call result boxed* (boxing) | 603 | 6.65% |
| `map[K]V{…}` literal | 568 | 6.27% |
| `make([]T, …)` | 565 | 6.23% |
| *residue — unmatched subject* (boxing, by assumption) | 456 | 5.03% |
| value composite literal | 410 | 4.52% |
| `make(…)` other | 406 | 4.48% |
| `make(map[K]V, …)` | 344 | 3.80% |
| `new(T)` | 269 | 2.97% |
| *string literal boxed* (boxing) | 253 | 2.79% |
| result temporary `~rN` | 70 | 0.77% |
| closure / synth variadic / numeric literal boxed | 0 | 0.00% |
| **Total** | **9,063** | 100% |

Boxing subtotal: 603 + 456 + 253 + 0 + 0 = **1,312 of 9,063 = 14.5%**. This is the same quantity
§4.3 encodes as 9,063 − 7,751 "real" non-escaping allocations, so the two sections agree.
Reproduce the whole table with `sh census/shapes.sh`.

### 6.2 Ownership pressure is uniform across subsystems

Borrowable-to-leaking parameter ratio by top-level area:

| Area | `does not escape` | `leaking param` | Ratio |
|---|---:|---:|---:|
| `staging/` | 42,653 | 20,978 | 2.03 |
| `pkg/` | 33,201 | 16,215 | 2.05 |
| `test/` | 16,045 | 10,877 | 1.48 |
| `cmd/` | 3,222 | 1,747 | 1.84 |
| `plugin/` | 1,589 | 388 | 4.09 |

`staging/` and `pkg/` — the two areas that are the actual system — are statistically
indistinguishable (2.03 vs 2.05). **There is no easy subsystem to carve off and port first, and
no ownership-hostile subsystem to quarantine.** Sequencing must be driven by something other
than ownership difficulty, because ownership difficulty is uniform.

### 6.3 Pointer-shaped syntax — labelled proxy

**These are SYNTACTIC PROXIES for aliasing pressure, not measurements of aliasing.** A `*T`
field in Go means "optional, or shared, or large, or mutable, or all four" — the syntax does not
say which. Counted with `go/parser`, so the counts are AST-exact even though their
interpretation is not.

Struct fields (39,507 fields across 12,088 struct type nodes, D3 minus testdata):

| Field shape | Count | Share |
|---|---:|---:|
| plain value | 24,308 | 61.5% |
| **pointer `*T`** | **9,190** | **23.3%** |
| slice of values | 3,605 | 9.1% |
| map, non-pointer value | 1,056 | 2.7% |
| func | 536 | 1.4% |
| slice of pointers | 339 | 0.9% |
| map to pointer | 205 | 0.5% |
| channel | 192 | 0.5% |
| interface | 76 | 0.2% |

Pointer-bearing fields total 9,734 (24.6%). But the pointee distribution reframes that number:

| Pointee | Count |
|---|---:|
| `string` | 1,258 |
| `int32` | 541 |
| `bool` | 492 |
| `int64` | 215 |
| `int`, `float64`, `uint64` | 89 |
| **subtotal: pointers to scalars** | **2,595 (28.2% of all pointer fields)** |

> **28.2% of pointer-typed fields point at a scalar.** These are not aliasing at all — they are
> Go's only idiom for an optional field, and they map to `Option<T>` with zero ownership
> consequence. The genuine aliasing-pressure figure is nearer 17% of fields than 24.6%.

The remaining pointee mass is dominated by generated machinery: `mock.Call` (161),
`gentype.ClientWithListAndApply` (158+158), `genericregistry.Store` (120), and
`*ApplyConfiguration` types.

Functions (82,914 declarations):

| Measure | Count | Share of declarations |
|---|---:|---:|
| Return ≥1 pointer | 18,410 | 22.2% |
| Take ≥1 pointer parameter | 24,501 | 29.6% |
| Pointer receivers | 44,076 | (86.5% of 50,957 methods) |
| Value receivers | 6,881 | (13.5% of methods) |

The 86.5% pointer-receiver figure is the **weakest proxy in this record** and should not be
cited as aliasing evidence: Go style recommends pointer receivers for consistency once any
method needs one, so it reflects convention, not sharing.

Return shapes among the 18,410 pointer-returning functions concentrate hard:

| Return shape | Count | Share |
|---|---:|---:|
| `*T` alone | 15,013 | 81.6% |
| `(*T, error)` | 2,791 | 15.2% |
| **top 2 shapes** | **17,804** | **96.7%** |
| remaining 18 shapes | 606 | 3.3% |

**96.7% of pointer-returning functions are one of two shapes**, both the constructor/factory
idiom. A returned `*T` from a constructor is an *owned* value in Rust — `fn new() -> T` or
`-> Result<T, E>`. No `Arc`, no lifetime. This is the single most mechanically translatable
shape in the corpus and it covers ~18k functions.

---

## 7. Finding 4 — `unsafe.Pointer`

Values with no safe Rust mapping. The occurrence count is alarming and the shape count is not.

| Measure | Count | Command |
|---|---:|---|
| `unsafe.Pointer` occurrences (D3) | 4,264 | `grep -rho 'unsafe\.Pointer' --include='*.go' . --exclude-dir=vendor --exclude='*_test.go' \| wc -l` |
| Files containing it (D3) | 129 | same with `-rl \| wc -l` |
| …of which generated | 113 | `comm -12` against the generated-file list |
| Occurrences in `zz_generated.conversion.go` | 4,195 (**98.4%**) | `grep -rho 'unsafe\.Pointer' --include='zz_generated.conversion.go' . --exclude-dir=vendor \| wc -l` |
| Occurrences in hand-written files | **67** | 4,264 − 4,195 − 2 (in other `zz_generated.*`) |
| `unsafe.Slice`/`String`/`Add` | 88 | AST instrument |
| `import "unsafe"` occurrences | 154 | AST instrument |

**4,264 occurrences resolve to three shapes.**

*Shape 1 — generated layout-identical reinterpretation (4,195 occurrences, 1 shape).* Every one
is the conversion-generator's zero-copy cast between structurally identical API versions:

```go
out.Items    = *(*[]apps.ControllerRevision)(unsafe.Pointer(&in.Items))
out.Selector =  (*metav1.LabelSelector)(unsafe.Pointer(in.Selector))
```

This is generator output. It is not translated; it is regenerated by an owned generator that can
emit a safe field-wise copy, or a checked transmute, at its own discretion. **98.4% of the
corpus's `unsafe.Pointer` surface is a generator design decision, not a porting problem.**

*Shape 2 — hand-written layout reinterpretation (32 occurrences, 6 files).* Identical shape to
Shape 1, written by hand:

```
22  staging/src/k8s.io/apiserver/pkg/apis/apidiscovery/v2/conversion.go
 3  staging/src/k8s.io/dynamic-resource-allocation/api/v1beta1/conversion.go
 2  staging/src/k8s.io/controller-manager/config/v1/conversion.go
 2  staging/src/k8s.io/apimachinery/pkg/apis/meta/v1beta1/conversion.go
 2  pkg/apis/resource/v1beta1/conversion.go
 1  staging/src/k8s.io/apiextensions-apiserver/pkg/apis/apiextensions/v1/conversion.go
```

*Shape 3 — syscall FFI (35 occurrences, 10 files).* Windows/Darwin/FreeBSD platform bindings:

```
10  pkg/kubelet/winstats/cpu_topology.go
 5  pkg/kubelet/network/dns/dns_windows.go
 5  test/images/agnhost/dns/dns_windows.go
 4  pkg/volume/util/fs/fs_windows.go
 4  pkg/kubelet/winstats/winstats.go
 2  pkg/windows/service/service.go
 2  pkg/kubelet/winstats/perfcounter_nodestats_windows.go
 1  cmd/kubelet/app/init_windows.go
 1  pkg/kubelet/util/boottime_util_darwin.go
 1  pkg/kubelet/util/boottime_util_freebsd.go
```

Shape 3 has an exact Rust counterpart — `unsafe extern` FFI — so it is not an unmapped
construct; it is unsafe in Rust too, and bounded to 35 sites. **The genuinely
translation-hostile residue is Shape 2: 32 occurrences in 6 files.**

**Caveat, and it matters:** this is measured on darwin/arm64's file set by `grep`, which is
platform-blind, so the *counts* include Linux/Windows files — but §3 notes those files were
never compiled, so any `unsafe.Pointer` there is uncovered by the escape analysis. The grep
counts are complete for D3; the escape analysis is not. A Linux run may reveal additional
hand-written `unsafe` in cgroup/netlink paths.

---

## 8. Threats to validity, and four defects found in this census's own pipeline

Recorded because an unstated method defect is how a confidently wrong number ships.

**Defect 1 — vendor leakage and path-prefix mismatch (found, fixed, numbers re-derived).** The
Go compiler emits some paths with a leading `./` and others without. The first filtering pass
excluded `vendor/` but not `./vendor/`, admitting **8,004 third-party diagnostic lines** into the
"corpus" set, and the generated-vs-hand-written join silently failed for all 75,129 `./`-prefixed
lines, classifying every one as hand-written. Fixed by normalising `sed 's|^\./||'` **before**
filtering. Every number in this record is post-fix.

**Defect 2 — `append` counted as a parameter (found, fixed).** `append` is a bare identifier, so
`append does not escape` matched the bare-identifier pattern used to detect parameter verdicts,
inflating the borrowable-parameter count. Fixed by excluding Go builtins. The §4.2 figures are
post-fix.

**Defect 3 — three prose figures wrong in the first published revision (found by independent
review, corrected here).** All three were prose errors contradicted by tables in the same
document; no underlying measurement changed and no conclusion changed.

| Where | Published | Correct | How it was caught |
|---|---|---|---|
| §6.1 | non-escaping boxing share "3.5%" | **14.5%** (1,312 / 9,063) | Contradicted by the table beneath it (one boxing class alone at 6.65%) and by §4.3's 9,063 − 7,751 |
| §4.2 | collapsed leak flavours "39.4%" | **50.7%** (99,689 / 196,512) | Not derivable from the document's own table by any route; the figure had no provenance |
| §4.2 | four-row table presented as a 100% partition | **a sum**; union is 194,727, overlap 1,785 | Re-derived from the four `.pos` files |

The mechanism behind the first two is worth naming: a table was computed correctly and a
sentence about it was written from memory. **Every prose figure in this revision was re-derived
from the artefact it summarises**, and the commands are inline so the next reader can do the
same rather than trusting the sentence.

**Defect 4 — a taxonomy's coverage reported as a measurement (found by independent review,
corrected here).** §6.1 claimed a "closed set of 15 shapes" with 100% coverage and §9 graded it
"Measured, closed set". The classifier is a hand-authored `awk` cascade ending in a bare `else`,
so 100% coverage was true by construction. The default bucket is the *second-largest class*
(20.19%, 10,125 distinct subjects). Corrected in §6.1 and §9; this was the most
consequential of the four, because it was promoted to a decision-changing finding and it
mis-sized the thing the census exists to size.

**An independent cross-check that passed.** The generated/hand-written split was computed twice
by different methods — `grep -c -F -f` (substring matching, risks over-counting) and an `awk`
exact field-1 comparison. All seven categories agreed to the digit, so no substring
contamination occurred.

Remaining threats, unresolved:

| Threat | Effect | Direction |
|---|---|---|
| **darwin/arm64 build** excludes ≥153 Linux-guarded files | kubelet/runtime/cgroup/netlink paths unmeasured | **Unknown; likely understates difficulty**, since these are the pointer-heaviest paths |
| Position deduplication keeps one verdict per `file:line:col` **within** a verdict class | 357,265 raw → 292,974 unique, an 18% collapse; a site whose verdict differs per generic instantiation is attributed to one | Unknown, believed small |
| Verdict classes **overlap across** classes; the §4.2 table is a sum, not a partition | 1,785 positions (0.9%) carry two verdicts; union 194,727 vs sum 196,512. Borrowability is 55.6% on the union basis vs 55.1% on the sum basis | Sum basis **understates** shares of the total by ≤0.9% |
| The §6.1 shape taxonomy is hand-authored with an `else` default | 20.19% of escaping sites land in a residue holding 10,125 distinct subjects; 100% coverage is by construction, not measurement | **Understates the rule-corpus size** by an unknown amount |
| `-m` output is an unstable diagnostic surface | Regex parsing may silently miss a format not present in this corpus | Understates counts |
| Escape analysis is conservative | Over-reports escaping | **Escape figures are upper bounds; borrow figures are lower bounds** |
| Mutation detection is depth-1 | Misses mutation via helper functions | Understates mutating types |
| File coverage counts only files with ≥1 diagnostic | 72.9% is a floor, not the true coverage | Understates coverage |

---

## 9. Summary of the sizing-relevant numbers

| # | Question | Answer | Confidence |
|---|---|---|---|
| 1 | Parameters that borrow with no refcount | **55.1%** (48.1% hand-written) | Measured; a **lower bound** |
| 2 | Allocation sites escaping to heap | **93.6%** real / 95.8% raw | Measured; an **upper bound** |
| 3 | Escaping sites that are *not* ownership decisions | **24.8%–45.0%** (interface boxing) | Lower end measured; upper end assumes the §6.1 residue is boxing |
| 4 | Distinct escaping-allocation shapes | **14 measured shapes covering 79.81%**; a residue of 20.19% holding 10,125 distinct subjects | **Authored taxonomy with a residue bucket — NOT a closed set.** 100% coverage is by construction |
| 5 | Types needing `Arc<Mutex<T>>` | **5.3%–13.9%** of hand-written types | **Bounded, not determined** — needs alias analysis |
| 6 | Types needing `Arc<T>` vs `Box<T>` | **Not determined** | Requires whole-program points-to analysis |
| 7 | Pointer fields that are really `Option<scalar>` | **28.2%** of pointer fields | Measured (AST-exact) |
| 8 | Pointer-returning functions in 2 shapes | **96.7%** of 18,410 | Measured (AST-exact) |
| 9 | `unsafe.Pointer` with no safe mapping | **3 shapes**; hand-written residue **32 occurrences / 6 files** | Measured |
| 10 | Package coverage of the escape analysis | **83.7%**, Linux-only paths excluded | Measured |
| 11 | Go toolchain cost | 53 s, 4.6 GB cache, **zero install, nothing broke** | Measured |

The two findings that should change a decision:

1. **Sharing is not escape.** Sizing `Arc`/`Mutex` from the escape ratio over-provisions by
   roughly two orders of magnitude. The measured shared-mutable surface is 437–1,152 types, not
   ~200,000 sites.
2. **Much of the residue is shaped, but it is not closed, and the corpus is generated-heavy.**
   14 measured allocation shapes cover 79.81% of escaping sites; 3 `unsafe` shapes and 2
   pointer-return shapes are genuinely closed (both enumerated from the full population, not
   from a default bucket); and 34.7% of files are machine output that should be regenerated
   rather than translated. The rule corpus is sized by the hand-written 65.3%, where the borrow
   rate is 48.1%, not 55.1%. **Do not size the allocation-rule corpus at 15** — that figure is
   an artefact of an authored classifier, and the unclassified fifth is where the sizing risk
   lives.

**Not determined, and requiring a separate instrument:** the alias/points-to analysis of §5.1;
a typed AST pass to close the §6.1 residue's 10,125 distinct subject expressions; and a
Linux-hosted re-run to cover the ≥153 platform-excluded files of §3.
