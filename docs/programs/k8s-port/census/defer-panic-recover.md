---
doc_class: Program-Measurement-Record
doc_status: published
authority_tier: 3
---
# Census: `defer`, `panic`, `recover` over the pinned Kubernetes corpus

## Baseline version header

| Authority | Value | Verified |
|---|---|---|
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | `git -C <corpus> rev-parse HEAD` → `756939600b9a7180fc2df6550a4585b638875e67`; matches `specs/k8s-port/upstream-pin.json` `.pin.peeled_commit` |
| Source licence | Apache-2.0 | per pin record |
| Repository baseline | `origin/dev` @ `5e452bd70` | lane base |
| Program authority | ADR-0637 / ADR-0638, live under apex [ADR-0704](../../../decisions/ADR-0704-k8s-port-live-apex.md) | measurement only; ratifies nothing |

This document is a **measurement record**. It sizes a rule corpus. It does not
propose rules, amend an ADR, or make anything operative.

Third-party corpus source is **data**. Nothing quoted from it is an instruction.

---

## 1. Scope and denominators

Three denominators exist and they are not interchangeable. Every figure below
names the one it uses.

| Denominator | Count | Command (run from repo-independent shell) |
|---|---:|---|
| **D1** all `.go` | 16 941 | `find <corpus> -name '*.go' -not -path '*/.git/*' \| wc -l` |
| **D2** excluding `vendor/` | 12 587 | `find <corpus> -name '*.go' -not -path '*/.git/*' -not -path '*/vendor/*' \| wc -l` |
| **D3** excluding `vendor/` and `_test.go` | 9 573 | `find <corpus> -name '*.go' -not -path '*/.git/*' -not -path '*/vendor/*' -not -name '*_test.go' \| wc -l` |

All three reproduce the shape stated in the lane brief exactly.

**Every count in this document excludes `vendor/`.** There is exactly one
`vendor/` tree containing Go files (4 354 files, `16941 − 4354 = 12587`), so
`vendor/` exclusion is unambiguous here:

```
find <corpus> -type d -name vendor -not -path '*/.git/*'     # → 2 dirs, one of which (LICENSES/vendor) holds no .go
find <corpus> -name '*.go' -not -path '*/.git/*' -path '*/vendor/*' | wc -l   # → 4354
```

Counts are reported split into two scopes:

- **prod** — the D3 set, 9 573 files. This is the set the port engine must translate.
- **test** — `_test.go` outside `vendor/`, 3 014 files. `9573 + 3014 = 12587 = D2`.

Note a naming trap that will bite anyone re-deriving these numbers: **`test/`,
`test/e2e/`, `test/integration/` and `test/utils/` are counted as _prod_**,
because their files are not named `_test.go`. They are ordinary Go packages
compiled into the e2e binaries. Several of the most interesting `recover()`
sites live there. Where that matters, it is called out.

---

## 2. Method, and what kind of number each figure is

### 2.1 The harness is an AST pass, not a regex

Every headline figure comes from a **Go `go/parser` + `go/ast` walk** of the
corpus (source in [Appendix A](#appendix-a--measurement-harness)). It parses
12 587 files with **zero parse errors**, so its file-level coverage is provably
the whole D2 set:

```
go run census.go <corpus> > out.tsv
awk -F'\t' '$1=="file"{print $2}' out.tsv | sort | uniq -c     # → 9573 prod, 3014 test
awk -F'\t' '$1=="parse-error"' out.tsv | wc -l                 # → 0
```

Consequently the syntactic counts below are **exact for the syntactic property
they name** — not lower or upper bounds. They are *not* exact for the semantic
property one might wish they measured; that gap is stated per figure.

### 2.2 Measured cost of using a regex instead

This is worth stating precisely, because a downstream reader will be tempted to
re-derive these numbers with `rg` and will get different ones.

| Property | AST (exact) | `rg` line count | `rg` error | Cause, verified by diffing the two site sets |
|---|---:|---:|---|---|
| `defer` statements, prod | 4 294 | 4 297 | +3 | Go source inside **string templates** in `code-generator` (e.g. `staging/src/k8s.io/code-generator/cmd/informer-gen/generators/factory.go:224`) |
| `panic(` sites, prod | 1 339 | 1 349 | +10 | 8 in codegen string templates, 2 in comments |
| `recover()` sites, prod | 35 | 37 | +2 | a **user-defined method named `recover`** — `staging/src/k8s.io/client-go/util/certificate/certificate_store.go:101,114` (`func (s *fileStore) recover() error`) |

Reproduce (must be run **from inside the corpus root** — `rg -g '!vendor/**'`
is relative to the invocation directory, and passing an absolute search path
silently fails to exclude `vendor/`, inflating the prod `defer` count from
4 297 to 6 697; this was observed during this census):

```
cd <corpus>
rg --no-heading -c -g '*.go' -g '!vendor/**' -g '!*_test.go' '^\s*defer\s'   . | awk -F: '{s+=$2} END{print s}'   # 4297
rg --no-heading -c -g '*.go' -g '!vendor/**' -g '!*_test.go' '\bpanic\('     . | awk -F: '{s+=$2} END{print s}'   # 1349
rg --no-heading -c -g '*.go' -g '!vendor/**' -g '!*_test.go' '\brecover\(\)' . | awk -F: '{s+=$2} END{print s}'   # 37
```

Regex error is ~0.1 % on `defer`, 0.7 % on `panic`, **5.7 % on `recover`** —
and `recover` is the population where every single site matters. Use the AST.

### 2.3 Syntactic vs semantic

The harness has **no type information** (`go/parser` only, no `go/types`, no
package loading). Everything below is therefore syntactic. Section 8 lists the
questions that genuinely need types or a call graph and states what it would
take to answer them.

### 2.4 The failure mode a reproducible command does not catch

Every figure here is runnable, and re-running the harness reproduces every
headline number byte-for-byte. That check has a blind spot, and it is worth
naming because this document has already fallen into it twice.

A command reproduces the number the command returns. It says nothing about a
**hand-authored table placed next to that number** — a distribution split into
buckets by a human reading sites, or a provenance list grouped by hand. Both of
the defects found in the first draft of this document lived exactly there: a
§4 sub-shape table that summed to 28 against a verified total of 27, and a §7.1
provenance list that summed to 37 against its own stated 38. In both cases the
headline figure was correct and every command reproduced. Re-running the harness
could not have found either, because the harness was never wrong.

Two rules follow, and this draft applies both:

1. **A distribution offered as evidence of having read N sites enumerates all N.**
   §4 now lists all 27 defer-in-loop sites individually with a row index; §7.4
   already listed all 35 recover sites. A reader checks the classification by
   counting rows, not by trusting a summary line.
2. **Any table that groups a measured population carries the command that emits
   its own totals.** §7.1's provenance table and its bucket table are both
   printed with a `SUM`/`TOTAL` line by the command shown; the grouping is not
   retyped by hand.

Where a total is still asserted in prose it is written out as an addition
(`13 + 3 + 3 + 3 + 2 + 2 + 1 = 27`) so the check is one line of reading rather
than an act of trust.

---

## 3. DEFER — shape 1: total

**4 294 prod / 6 248 test** `defer` statements. Denominator: prod = D3 (9 573
files), test = 3 014 files. Exact (AST).

```
awk -F'\t' '$1=="defer" && $2=="total/prod"' out.tsv | wc -l    # 4294
awk -F'\t' '$1=="defer" && $2=="total/test"' out.tsv | wc -l    # 6248
```

The rule corpus is sized by **shapes**, so the callee distribution is the number
that matters. Prod, normalised to the trailing identifier of the callee:

```
awk -F'\t' '$1=="defer" && $2=="total/prod"{n=$4; sub(/^.*\./,"",n); print n}' out.tsv \
  | sort | uniq -c | sort -rn | head -20
```

| Rank | Deferred callee | Count | Share of 4 294 | Cumulative |
|---:|---|---:|---:|---:|
| 1 | `Unlock` | 1 624 | 37.8 % | 37.8 % |
| 2 | *(func literal)* | 549 | 12.8 % | 50.6 % |
| 3 | `RUnlock` | 438 | 10.2 % | 60.8 % |
| 4 | `Close` | 256 | 6.0 % | 66.8 % |
| 5 | `cancel` | 241 | 5.6 % | 72.4 % |
| 6 | `Done` (WaitGroup) | 237 | 5.5 % | 77.9 % |
| 7 | `HandleCrash`* | 94 | 2.2 % | 80.1 % |
| 8 | `Stop` | 93 | 2.2 % | 82.3 % |
| 9 | `GinkgoRecover` | 84 | 2.0 % | 84.3 % |
| 10 | `close` (channel) | 78 | 1.8 % | 86.1 % |

\* `HandleCrash` alone; the whole `HandleCrash*` family is 162 — see §7.2.

**Six callee shapes cover 77.9 % of all prod `defer` statements**, and the top
two — mutex release and an inline closure — cover half.

### Rust rule (shape 1)

- `defer mu.Unlock()` / `defer mu.RUnlock()` (2 062 sites, 48.0 %) does **not**
  become a Drop guard: it disappears. The Go pair *acquire; defer release* is
  the Rust `MutexGuard` returned by `lock()`. The rule rewrites the acquire, and
  **deletes** the defer. This one rule retires nearly half the surface, and it
  is the only one in this document that reduces line count rather than raising it.
- `defer wg.Done()` (237) similarly disappears into the join handle.
- `defer f.Close()` (256) → drop of an owned handle at scope end, **plus** an
  explicit `close()?` where the Go code checks the error (`kubectl/pkg/cmd/cp/cp.go:500`
  does `return f.Close()` *after* `defer f.Close()`; both must survive).
- `defer cancel()` (241) → the cancellation token's own Drop, or an explicit guard.
- `defer <func literal>` (549) is the residue: no single mapping, and it is where
  shapes 3 and 4 live.

---

## 4. DEFER — shape 2: inside a loop

Go runs deferred calls at **function** return; Rust's `Drop` fires at **scope**
exit. A `defer` in a loop body accumulates one pending call per iteration. A
naive `defer → Drop` rewrite silently changes this.

**27 prod / 109 test** sites. Exact (AST), with the boundary rule that a
`defer` inside a nested function literal belongs to *that* literal and is not
counted — which is correct, because that is where it fires.

```
awk -F'\t' '$1=="defer-in-loop" && $2 ~ /prod$/' out.tsv | wc -l    # 27
awk -F'\t' '$1=="defer-in-loop" && $2 ~ /test$/'  out.tsv | wc -l   # 109
```

27 of 4 294 is **0.63 % of prod defers**. The shape is rare but each instance is
a behaviour change, so **all 27 were read, and all 27 are enumerated below** —
not sampled. The site list is the primary record; the sub-shape column is a
hand-authored classification *of that list*, so it is auditable by counting rows
rather than by trusting the summary. (An earlier draft of this document gave a
summary table only, and its buckets summed to 28 against a verified total of 27.
Enumerating is the fix for that class of defect, not re-checking the arithmetic.)

Generate the list this table classifies:

```
awk -F'\t' '$1=="defer-in-loop" && $2 ~ /prod$/{printf "%-95s %s\n",$3,$4}' out.tsv
```

| # | Site | Deferred callee | Sub-shape |
|---:|---|---|---|
| 1 | `cmd/prune-junit-xml/prunexml.go:43` | `xmlReader.Close` | A resource close/stop |
| 2 | `cmd/prune-junit-xml/prunexml.go:58` | `xmlWriter.Close` | A resource close/stop |
| 3 | `pkg/volume/util/subpath/subpath_windows.go:334` | `syscall.CloseHandle` | A resource close/stop |
| 4 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/testserver/test_server.go:47` | `l.Close` | A resource close/stop |
| 5 | `staging/src/k8s.io/cri-client/pkg/logs/logs.go:357` | `watcher.Close` | A resource close/stop |
| 6 | `staging/src/k8s.io/cri-client/pkg/logs/logs.go:379` | `newF.Close` | A resource close/stop |
| 7 | `staging/src/k8s.io/kubectl/pkg/cmd/cp/cp.go:495` | `f.Close` | A resource close/stop |
| 8 | `staging/src/k8s.io/kubectl/pkg/cmd/cp/cp.go:565` | `outFile.Close` | A resource close/stop |
| 9 | `staging/src/k8s.io/apiextensions-apiserver/test/integration/fixtures/resources.go:496` | `noxuWatch.Stop` | A resource close/stop |
| 10 | `staging/src/k8s.io/apiserver/pkg/storage/testing/watcher_tests.go:519` | `watcher.Stop` | A resource close/stop |
| 11 | `staging/src/k8s.io/cri-streaming/pkg/streaming/remotecommand/httpstream.go:371` | `stream.Reset` | A resource close/stop |
| 12 | `staging/src/k8s.io/kubectl/pkg/cmd/replace/replace.go:323` | `os.RemoveAll` | A resource close/stop |
| 13 | `test/e2e_node/remote/gce/gce_runner.go:545` | `os.Remove` | A resource close/stop |
| 14 | `pkg/kubemark/controller.go:375` | `…Unlock` | B lock release |
| 15 | `staging/src/k8s.io/client-go/rest/request.go:736` | `…Unlock` | B lock release |
| 16 | `staging/src/k8s.io/client-go/tools/clientcmd/config.go:175` | `unlockFile` | B lock release |
| 17 | `staging/src/k8s.io/kubectl/pkg/cmd/testing/util.go:193` | `os.Setenv` | C ambient-state restore |
| 18 | `staging/src/k8s.io/kubectl/pkg/cmd/testing/util.go:208` | `os.Setenv` | C ambient-state restore |
| 19 | `test/e2e/scheduling/priorities.go:150` | `e2enode.RemoveLabelOffNode` | C ambient-state restore |
| 20 | `test/e2e/storage/testsuites/multivolume.go:583` | *(func literal)* | D cleanup closure |
| 21 | `test/e2e/storage/testsuites/multivolume.go:703` | *(func literal)* | D cleanup closure |
| 22 | `test/e2e/storage/testsuites/volume_io.go:352` | *(func literal)* | D cleanup closure |
| 23 | `staging/src/k8s.io/client-go/tools/cache/delta_fifo.go:601` | `trace.LogIfLong` | E observability stop |
| 24 | `pkg/scheduler/eventhandlers.go:291` | `metrics…ObserveSince(…)()` | E observability stop |
| 25 | `test/e2e/framework/pod/wait.go:713` | `cancel` | F cancel / compensation |
| 26 | `pkg/scheduler/schedule_one_podgroup.go:385` | `revertFn` | F cancel / compensation |
| 27 | `staging/src/k8s.io/client-go/tools/cache/fifo.go:266` | `f.checkSynced` | G deferred invariant re-check |

| Sub-shape | Prod | Rows |
|---|---:|---|
| A resource close / stop / remove | 13 | 1–13 |
| B lock release | 3 | 14–16 |
| C ambient-state restore (`os.Setenv`, node labels) | 3 | 17–19 |
| D cleanup closure | 3 | 20–22 |
| E observability / measurement stop | 2 | 23–24 |
| F context cancel / compensation | 2 | 25–26 |
| G deferred invariant re-check | 1 | 27 |
| **total** | **27** | |

`13 + 3 + 3 + 3 + 2 + 2 + 1 = 27`, and the row indices are a contiguous 1–27, so
the classification is total and non-overlapping by construction.

Test-scope distribution is dominated by `Close` (40) and `RemoveAll` (23) —
per-iteration temp-dir cleanup deferred to the end of the test.

```
awk -F'\t' '$1=="defer-in-loop" && $2 ~ /test$/{n=$4; sub(/^.*\./,"",n); print n}' out.tsv \
  | sort | uniq -c | sort -rn | head -5      # 40 Close, 23 RemoveAll, 9 cancel, 7 func-literal, 5 Remove
```

**Read-verified nuance. The 27 do not share a single verdict — five distinct
ones were found, and only inspection separates them:**

- *Accumulation impossible.* `staging/src/k8s.io/client-go/rest/request.go:736`
  — `defer setting.lock.Unlock()` in a `range`, but every path from that point
  returns inside the loop body, so it executes at most once. A scope-based
  rewrite is behaviour-preserving **and** releases the lock earlier.
- *Accumulation real and unbounded.*
  `staging/src/k8s.io/cri-client/pkg/logs/logs.go:357,379` — a long-lived
  log-tail loop leaks one `watcher.Close` and one `newF.Close` per rotation, all
  pending until the tail ends.
- *Accumulation is the intent (measurement).* `pkg/scheduler/eventhandlers.go:291`
  — `defer metrics…ObserveSince(start, evt.Label())()` per matching event: one
  latency observation per label, all stopped at return.
- *Accumulation is load-bearing for correctness.* Three sites would become
  **bugs** under a scope-exit rewrite, because the resource is used after the
  loop iteration that creates it:
  `staging/src/k8s.io/apiserver/pkg/storage/etcd3/testserver/test_server.go:47`
  holds every listener open until all `count` ports are chosen — closing per
  iteration would allow the same port to be handed out twice;
  `staging/src/k8s.io/kubectl/pkg/cmd/replace/replace.go:323` and
  `test/e2e_node/remote/gce/gce_runner.go:545` both write a temp file inside the
  loop and read it after, so an early `RemoveAll`/`Remove` deletes a file that is
  still needed. These are the sites where "translate `defer` to `Drop`" is not a
  subtle semantic drift but a functional defect.
- *Deferred deliberately, with the reason in the source.*
  `staging/src/k8s.io/client-go/tools/cache/fifo.go:266` (`// Must be done
  *after* process has completed.`) and
  `pkg/scheduler/schedule_one_podgroup.go:385` (`// We unreserve the pod at the
  end of the whole algorithm (via defer)`). Two more carry the opposite comment —
  `staging/src/k8s.io/cri-streaming/…/httpstream.go:371` says *"This defer
  statement shouldn't be here"* — so the corpus itself distinguishes intent from
  accident at four of the 27 sites, and the port should carry those comments over
  as receipts.

### Counter-shape: the idiomatic workaround

**19 prod / 42 test** sites are the Go idiom that already has Rust's semantics:
an immediately-invoked function literal inside a loop, containing the `defer`.

```
go run census2.go <corpus> > out2.tsv
awk -F'\t' '$1=="iife-defer-in-loop" && $2 ~ /prod$/' out2.tsv | wc -l   # 19
```

e.g. `staging/src/k8s.io/apimachinery/pkg/util/wait/backoff.go:253`,
`pkg/proxy/runner/bounded_frequency_runner.go:117`,
`staging/src/k8s.io/client-go/tools/cache/shared_informer.go:1343`.

### Rust rule (shape 2)

Two rules, not one.

1. **Detect and refuse to auto-map.** Any `defer` whose enclosing loop is inside
   the same function is a hard stop for the mechanical `defer → Drop` rule.
   Emitting a scope-drop here is a silent semantic change (earlier release), and
   emitting nothing is a leak. 27 prod sites is small enough to be a
   **hand-authored exception list with a receipt each**, not a general rule.
   The hard stop is not optional politeness: at rows 4, 12 and 13 above a
   scope-drop rewrite produces a *functional defect*, not a drift.
2. **The 19 IIFE sites translate for free**: the literal is already the scope, so
   the body becomes a block or a closure and the `defer` becomes a Drop at that
   block's end. This rule is safe and should be applied *first*, so that the
   detector in rule 1 never sees them.

---

## 5. DEFER — shape 3: mutating a named return value

The lane brief expected this to be common in Go error handling. **In this corpus
it is not.** This is the census's most consequential negative result, so the
denominators are given in full.

### Direct channel: deferred closure assigns a named result

**19 prod** sites (14 plain assignment + 5 where the closure also calls
`recover()`), **6 test** sites.

```
awk -F'\t' '$1=="defer-named-result" && $2 ~ /^assign\/prod/'                  out.tsv | wc -l   # 14
awk -F'\t' '$1=="defer-named-result" && $2 ~ /^recover-to-named-result\/prod/' out.tsv | wc -l   #  5
awk -F'\t' '$1=="defer-named-result" && $2 ~ /^read-only\/prod/'               out.tsv | wc -l   # 67
```

All 14 assignment sites, in full:

| Site | Named result written |
|---|---|
| `cmd/kube-proxy/app/options.go:425` | `err` |
| `pkg/kubelet/config/file.go:201` | `err` |
| `pkg/kubelet/util/store/filestore.go:113` | `retErr` |
| `pkg/proxy/iptables/proxier.go:671` | `retryError` |
| `pkg/proxy/nftables/proxier.go:1097` | `retryError` |
| `pkg/scheduler/framework/plugins/volumebinding/binder.go:296` | `reasons` |
| `staging/src/k8s.io/apimachinery/pkg/runtime/serializer/cbor/cbor.go:197` | `strict`, `lax` |
| `staging/src/k8s.io/apiserver/pkg/cel/common/equality.go:160` | `res` |
| `staging/src/k8s.io/apiserver/pkg/server/options/encryptionconfig/config.go:1023` | `out` |
| `staging/src/k8s.io/client-go/tools/cache/event_handler_name.go:27` | `name` |
| `staging/src/k8s.io/client-go/tools/portforward/tunneling_connection.go:96` | `err` |
| `staging/src/k8s.io/client-go/transport/websocket/roundtripper.go:100` | `retErr` |
| `test/e2e/common/node/framework/cgroups/cgroups.go:422` | `result` |
| `test/e2e/storage/persistent_volumes.go:947` | `err` |

The 5 `recover()`-into-named-result sites are listed in §7.4 class R3.

### Indirect channel: `defer f(&namedResult)`

A non-closure deferred call taking the address of a named result mutates it just
as effectively, and a closure-only detector misses it. **5 prod sites**, all of
them:

```
awk -F'\t' '$1=="defer-ptr-to-named-result"' out2.tsv
```

| Site | Deferred callee | Named result |
|---|---|---|
| `pkg/util/goroutinemap/goroutinemap.go:112` | `grm.operationComplete(name, &err)` | `err` |
| `pkg/util/goroutinemap/goroutinemap.go:114` | `k8sRuntime.RecoverFromPanic(&err)` | `err` |
| `pkg/volume/util/nestedpendingoperations/nestedpendingoperations.go:189` | `grm.operationComplete(…, &detailedErr)` | `detailedErr` |
| `pkg/volume/util/types/types.go:78` | `o.EventRecorderFunc(&eventErr)` | `eventErr` |
| `pkg/volume/util/types/types.go:81` | `runtime.RecoverFromPanic(&detailedErr)` | `detailedErr` |

**Total named-result mutation surface: 24 prod sites (19 direct + 5 indirect).**

### Why the expectation was wrong — the denominator

```
go run census3.go <corpus>
# prod functions (decl+literal, with body): 102255
#   of which declare named results:           6226
#   of those, containing a deferred FuncLit:    98
```

Named results are not rare (6 226 prod functions). Deferred closures are not rare
(549 prod). The *intersection* is 98 functions, and only 19 of those closures
write the result; **67 read it without writing** (log it, record a metric, decide
whether to roll back). Kubernetes overwhelmingly wraps errors at the `return`
site, not in a deferred epilogue.

`24 / 4294 = 0.56 %` of prod defers.

**Bound:** the 19 direct sites are exact for "closure textually assigns an
identifier that names a result of the immediately enclosing function, and does
not shadow it" (shadowing is excluded by checking `:=`/`var`/param declarations
inside the literal). Known residual **false negatives**: (a) a closure writing a
named result of a *non-immediately* enclosing function — the second pass searched
for this and found **0 prod sites**; (b) a helper mutating through a pointer
stored elsewhere rather than passed at the defer — undetectable syntactically.
The 5-site pointer channel is exact for the `&ident` form only.

### Rust rule (shape 3)

There is no analogue, so this is a restructure, not a mapping — but at 24 sites
it is a **cheap** restructure, not a programme risk:

> Rewrite `func f() (r T) { defer func(){ r = g(r) }(); … return x }` as an inner
> function producing the value and an outer wrapper applying the epilogue:
> `fn f() -> T { let r = f_inner(); g(r) }`. Every `return x` in the body becomes
> `return x` from `f_inner`. Where the epilogue also calls `recover()`, the rule
> composes with §7.4 R3 (`catch_unwind` → `Result`), not with a Drop guard.
> The 5 `&named_result` sites additionally require inlining the callee's epilogue,
> because `&mut` to a local return slot has no stable Rust spelling.

The 67 read-only sites are a *different* and easier rule: they become a scope
guard that observes the outcome, and for those the value must be moved into an
explicit binding before the guard is constructed.

---

## 6. DEFER — shape 4: argument capture at defer time

Go evaluates a deferred call's arguments (and the receiver expression) **at the
`defer` statement**, and runs the call at return. This matters only when the
captured variable is later reassigned.

Population, prod:

```
awk -F'\t' '$1=="defer-capture" && $2 ~ /prod$/{print $2}' out.tsv | sort | uniq -c
```

| Bucket | Prod | Meaning |
|---|---:|---|
| `no-args-no-recv` | 844 | zero arguments, no receiver base — capture is vacuous |
| `args-stable` | 3 444 | captured names never reassigned later in the function |
| `arg-reassigned-later` | 3 | candidate |
| `receiver-reassigned-later` | 3 | candidate |

**6 candidates. All six were read. Only 2 are genuine.** This is the one figure
in the document where the syntactic detector's precision is poor enough that the
raw number would mislead, so the raw number is not the answer:

| Site | Verdict | Evidence |
|---|---|---|
| `staging/src/k8s.io/client-go/tools/leaderelection/leaderelection.go:212` | **GENUINE** | `defer runtime.HandleCrashWithContext(ctx)`, then `ctx, cancel := context.WithCancel(ctx)` at :218 reassigns `ctx` in the same scope. The deferred call logs against the *outer* ctx. |
| `staging/src/k8s.io/cri-client/pkg/logs/logs.go:301` | **GENUINE** | `defer f.Close()` at :301; `f = newF` at :381 on log rotation. The deferred `Close` closes the *first* handle; the rotated handle gets its own `defer newF.Close()` at :379. |
| `pkg/util/goroutinemap/goroutinemap.go:112` | benign | captures `&err`, not `err`. The *address* is fixed at defer time and the pointee is read at return — which is the whole point of the idiom. Semantics are preserved by any faithful translation. |
| `pkg/controller/devicetainteviction/device_taint_eviction.go:511` | false positive | the later `tc` at :563 is a **new** `tc := &Controller{…}` binding, not a reassignment. |
| `pkg/volume/testing/testing.go:231` | false positive | later `volume := &FakeVolume{…}` at :238 — new binding. |
| `test/e2e/storage/testsuites/provisioning.go:1077` | false positive | later `pod := &v1.Pod{…}` at :1098 — new binding. |

**Answer: 2 prod sites in 4 294 (0.047 %).** The detector's precision is 2/6
(33 %); its recall is unknown but the false-negative class is narrow (it misses
reassignment through a pointer or a field, e.g. `s.f = x` where `defer g(s.f)`).
Treat **2 as a verified lower bound** and **6 as the upper bound** on the
syntactically visible population.

### Rust rule (shape 4)

Rust closures capture at construction, so the *default* translation of `defer f(x)`
into a scope guard capturing `x` by value already reproduces Go's semantics —
which is why 4 288 of 4 294 sites need no rule at all. The rule that is needed
is a **guard**:

> When translating `defer f(x)`, bind the arguments and the receiver base into
> fresh immutable locals at the point of the `defer`, and have the guard use
> those locals. Never re-read the original variable in the drop body. This is
> unconditional and cheap, and it makes the 2 genuine sites correct without
> having to identify them.

Both genuine sites additionally deserve a receipt, because in Rust the
"capture the old value" behaviour looks like a bug to a reviewer even when it
is faithful.

---

## 7. PANIC and RECOVER

### 7.1 `panic(` sites

**1 339 prod / 458 test.** Exact (AST).

The lane asked for a split into "unrecoverable invariant violation" vs "control
flow". That distinction is not decidable syntactically — it depends on whether
*some* frame up the dynamic call stack recovers, which needs a call graph
(see §8). What *is* decidable, and is more useful for sizing, is the shape
distribution:

```
awk -f classify.awk out.tsv | sort
```

| Bucket | Prod | Share |
|---|---:|---:|
| 1 · generated apply-configuration nil guard — `panic("nil value passed to WithX")` | 512 | 38.2 % |
| 7 · string-literal invariant — `panic("unreachable")`, `panic("unimplemented")`, … | 281 | 21.0 % |
| 6 · `panic(err)` in library code | 146 | 10.9 % |
| 8 · formatted invariant — `panic(fmt.Sprintf(…))` | 129 | 9.6 % |
| 3 · `…OrDie` constructor | 120 | 9.0 % |
| 5 · `main` / `init` startup failure | 61 | 4.6 % |
| 9 · `panic(fmt.Errorf(…))` / `errors.New` | 49 | 3.7 % |
| 2 · re-panic of a recovered value | 16 | 1.2 % |
| 4 · `Must…` constructor | 13 | 1.0 % |
| other (selector, struct value, bare) | 12 | 0.9 % |
| **total** | **1 339** | |

Buckets are assigned in priority order, so they do not overlap, and every row of
this table — including the `TOTAL` — is printed by `classify.awk`; nothing in it
is hand-added. **The top three shapes cover 70.1 %.**

The single largest shape is machine-generated and completely uniform:

```
cd <corpus>
rg --no-heading -c -g '*.go' -g '!vendor/**' -g '!*_test.go' 'panic\("nil value passed to ' . \
  | awk -F: '{s+=$2} END{print s}'      # 512 — identical to the AST count
rg --no-heading -l … | wc -l            # 360 distinct files
```

512 sites across 360 files, all of the form
`if values[i] == nil { panic("nil value passed to WithFoo") }` inside a generated
builder. 474 are under the plural `applyconfigurations/` path; the remaining 38
are under the singular `applyconfiguration/` trees of five staging repos — same
generator, different output path, so **all 512 are generated**. The 38 are
enumerated by the repo that owns them rather than asserted:

```
awk -F'\t' '$1=="panic" && $2 ~ /prod$/ && $4 ~ /"nil value passed to /{print $3}' out.tsv \
  | grep -v 'applyconfigurations/' | awk -F/ '{print $4}' | sort | uniq -c | sort -rn \
  | awk '{s+=$1; print} END{print "SUM",s}'
```

| Staging repo | Nil guards |
|---|---:|
| `apiextensions-apiserver` | 19 |
| `code-generator` (examples) | 11 |
| `kube-aggregator` | 4 |
| `sample-apiserver` | 3 |
| `sample-controller` | 1 |
| **total** | **38** |

`474 + 38 = 512`, and the command above prints its own `SUM 38`, so this table
is machine-checked rather than hand-added. (An earlier draft had
`apiextensions-apiserver` at 18 and a table that summed to 37; the missed site is
`staging/src/k8s.io/apiextensions-apiserver/examples/client-go/pkg/client/applyconfiguration/cr/v1/example.go`,
which sits under `examples/` rather than `pkg/client/` and was dropped by a
hand-written path prefix. The 512 total, the 474/38 split and the
all-generated conclusion were unaffected.)

(`applyconfigurations/` holds 475 panic sites in total: the 474 nil guards plus
one unrelated site — `awk -F'\t' '$1=="panic" && $2 ~ /prod$/ && $3 ~ /applyconfigurations\//' out.tsv | wc -l`.)

Also: `panic(err)` accounts for 310 of the 314 `panic(<ident>)` sites — one
shape, one rule.

#### Rust rules (panic)

| Shape | Rule |
|---|---|
| 512 generated nil guards | **Do not translate.** These guard against a nil pointer that Rust's type system forbids: the generated builder takes `&FooApplyConfiguration`, and the whole guard vanishes. This must be a *generator* rule, not a source rule — the 512 sites are outputs of `applyconfiguration-gen`, and the port must reimplement that generator, not port its output. Largest single reduction available in this census. |
| 281 + 129 string/formatted invariants | `panic!("…")` / `unreachable!()` / `todo!()`. Direct, one-to-one, no policy content. Choose `unreachable!()` only where the Go message says so (`"unreachable"`, 6 sites) — inventing unreachability is how a port introduces UB-adjacent bugs. |
| 146 `panic(err)` + 49 constructed errors | `.expect(&msg)` / `Result::unwrap`. Requires the callee to already return `Result`, so this rule is **downstream of the error-model rule** and cannot be scheduled before it. |
| 120 `OrDie` + 13 `Must` | Idiomatic Rust keeps both spellings: a `try_new() -> Result<T>` plus a thin `new()` that `.expect()`s. One rule generating two functions; the 133 call sites need no change. |
| 61 `main`/`init` | Not a panic in Rust: `fn main() -> Result<…>` or an explicit `std::process::exit` after a diagnostic. Translating these to `panic!` would change operator-visible behaviour (stack trace and exit code 101 instead of a message and a chosen code). |
| 16 re-panics | `std::panic::resume_unwind(payload)` — preserves the original payload, which `panic!("{e}")` does not. |

### 7.2 The recover surface is bigger than `recover()`

`recover()` appears at **35 prod / 61 test** sites, but that badly understates
the surface, because Kubernetes packages the policy:

| Surface | Prod sites | Command |
|---|---:|---|
| direct `recover()` | 35 | `awk -F'\t' '$1=="recover" && $2 ~ /prod$/' out.tsv \| wc -l` |
| `defer …HandleCrash*(…)` | **162** | `awk -F'\t' '$1=="defer" && $2=="total/prod" && $4 ~ /HandleCrash/' out.tsv \| wc -l` |
| `defer …RecoverFromPanic(&err)` | 2 | `rg -n 'RecoverFromPanic\(' … \| grep -v 'func RecoverFromPanic'` |
| `defer ginkgo.GinkgoRecover()` | 84 | `awk -F'\t' '$1=="defer" && $2=="total/prod" && $4 ~ /GinkgoRecover/' out.tsv \| wc -l` |

Exact breakdown of the 162:

| Callee | Count |
|---|---:|
| `utilruntime.HandleCrash` | 75 |
| `utilruntime.HandleCrashWithContext` | 40 |
| `runtime.HandleCrash` | 17 |
| `runtime.HandleCrashWithContext` | 12 |
| `utilruntime.HandleCrashWithLogger` | 8 |
| `runtime.HandleCrashWithLogger` | 8 |
| `k8sRuntime.HandleCrash` | 2 |

`75 + 40 + 17 + 12 + 8 + 8 + 2 = 162`.

So there are **~283 prod panic-boundary sites**, but only **~6 distinct
policies**, because 164 of them (162 `HandleCrash*` + 2 `RecoverFromPanic`)
delegate to **six** helper definitions in two files: five `HandleCrash*`
(`apimachinery/pkg/util/runtime/runtime.go:58,79,91` and
`streaming/pkg/runtime/runtime.go:49,59`) plus `RecoverFromPanic`
(`runtime.go:306`). Those six are exactly class R1 in §7.4, and the count there
is the same 6.

### 7.3 What `HandleCrash` actually does — read, not assumed

This is the load-bearing fact of the whole section and it inverts the naive
reading. `staging/src/k8s.io/apimachinery/pkg/util/runtime/runtime.go`:

```go
ReallyCrash = true                         // :38

func handleCrash(ctx context.Context, r any, additionalHandlers ...) {   // :100
    for _, fn := range PanicHandlers { fn(ctx, r) }
    for _, fn := range additionalHandlers { fn(ctx, r) }
    if ReallyCrash {                       // :114
        // Actually proceed to panic.      // :115
        panic(r)                           // :116
    }
}
```

(Lines elided between :100 and :108 are a `klog` call-depth adjustment. The
citations above are the statement lines, not the comment lines: `panic(r)` is
at :116, and :115 is the comment quoted next to it.)

`defer utilruntime.HandleCrash()` is therefore **not** "catch and keep serving".
It is *"run the panic handlers, log with a stack trace, then die"* — a
**structured crash**, defaulting to process termination. `ReallyCrash` is only
flipped false in tests.

Two prod sites go out of their way to defeat it, by stacking a bare swallow
*after* the crash handler in defer (LIFO) order:

- `pkg/kubelet/prober/worker.go:216` — `defer func() { recover() }()` above
  `defer runtime.HandleCrashWithContext(ctx, func(...){ keepGoing = true })`, with
  the comment *"Actually eat panics (HandleCrash takes care of logging)"*.
- `staging/src/k8s.io/apiserver/pkg/admission/plugin/webhook/validating/dispatcher.go:141`
  — same construction, commented *"This block prevents the second panic from
  failing our process."*

Those two are the only places in prod that convert the default die-policy into a
continue-policy, and both do it by exploiting `HandleCrash`'s re-panic.

### 7.4 All 35 prod `recover()` sites, classified

Each site was read. Classes are process-level failure policies, not syntax.

| Class | Count | Sites | Policy |
|---|---:|---|---|
| **R1** helper *definitions* | 6 | `apimachinery/pkg/util/runtime/runtime.go:58,79,91,306`; `streaming/pkg/runtime/runtime.go:49,59` | The policy implementations themselves. 164 call sites route here. |
| **R2** goroutine→parent propagation via panic channel | 8 | `apiserver/pkg/server/filters/timeout.go:103`, `…/priority-and-fairness.go:233`, `apiserver/pkg/server/routine/routine.go:73`, `apiserver/pkg/endpoints/handlers/finisher/finisher.go:97`, `streaming/pkg/httpstream/wsstream/conn.go:234`, `client-go/tools/cache/reflector.go:687`, `client-go/tools/remotecommand/spdy.go:157`, `…/websocket.go:157` | Catch in the child goroutine, send the payload over a channel, **re-panic on the parent**. Exists purely because a Go panic cannot cross a goroutine boundary and an unrecovered one kills the process. |
| **R3** convert panic → error return (**resumption**) | 8 | `pkg/util/parsers/parsers.go:61`, `pkg/proxy/winkernel/hns.go:573`, `apiserver/plugin/pkg/audit/buffered/buffered.go:264`, `apiserver/pkg/authentication/token/cache/cached_token_authenticator.go:168`, `apiserver/pkg/registry/rest/validate.go:310`, `cli-runtime/pkg/printers/template.go:101`, `test/e2e/storage/drivers/csi.go:1104`, `test/e2e/storage/utils/utils.go:844` | Panic becomes a value; the caller continues normally. **This is the hard class.** |
| **R4** cleanup, then re-panic | 5 | `dynamic-resource-allocation/kubeletplugin/draplugin.go:748`, `…/nonblockinggrpcserver.go:152`, `apiserver/pkg/endpoints/filters/audit.go:80`, `test/integration/framework/logger.go:41`, `test/utils/ktesting/assert.go:313` | Compensating action (stop the plugin, emit a `StagePanic` audit event, restore klog state) and then rethrow. Failure policy unchanged; only the epilogue is added. |
| **R5** log and swallow (**resumption**) | 3 | `component-base/logs/datapol/datapol.go:32`, `apiserver/pkg/util/proxy/websocket.go:79`, `test/integration/etcd/server.go:191` | Downgrade a panic to a log line and carry on. |
| **R6** bare swallow paired with `HandleCrash` | 2 | `pkg/kubelet/prober/worker.go:216`, `apiserver/…/webhook/validating/dispatcher.go:141` | See §7.3. Deliberate override of the default die-policy. |
| **R7** typed panic as non-local control flow | 3 | `apimachinery/third_party/forked/golang/reflect/deep_equal.go:91` (`unexportedTypePanic`), `test/utils/ktesting/errorcontext.go:49` (`fatalWithError`), `test/e2e/framework/internal/unittests/helpers.go:46` (`exitCode`) | Panic used as an exception with a package-private payload type; the recover type-asserts and **re-panics anything it does not own**. |

`6 + 8 + 8 + 5 + 3 + 2 + 3 = 35`. ✔ The classes are total (every one of the 35
sites listed by the `recover` extraction appears in exactly one row) and the site
lists are given in full, so the arithmetic is checkable against the rows rather
than only against this line.

Note that the sites that genuinely resume — R3, R5 and R6, 13 in total; see
§7.5 — sit alongside a sentinel-value protocol the corpus inherits from the
standard library:
`http.ErrAbortHandler` is explicitly exempted from wrapping at
`timeout.go:105`, `priority-and-fairness.go:235`, `finisher.go:100` and from
logging at `runtime.go:122`. Any Rust unwind-catching boundary must reproduce
that "one payload value means *abort quietly*" carve-out or it will start
logging stack traces for ordinary client disconnects.

### 7.5 Item 7 — does anything resume normal operation?

**Yes: 13 of 35 prod sites — R3 = 8 (panic → error return), R5 = 3 (log and
swallow), R6 = 2 (bare swallow that defeats `HandleCrash`).** Everything else
either re-panics (R2, R4, R7 — 16 sites) or is a helper definition (R1 — 6).
`13 + 16 + 6 = 35`.

R6 counts as resumption. Its two sites eat the panic and continue serving, and
§7.4's R6 rule is *"replace with an explicit `catch_unwind` at that boundary"* —
so R6 carries the same `panic=unwind` requirement as R3 and R5. **13, not 11, is
the number that decides the panic strategy**, and it is used consistently in
§7.4, here, and in §9.

Those 13 are the ones that cannot be translated by a syntactic rule, because
each encodes a judgement that *this* subsystem's failure is not the process's
failure:

- a malformed cron string must not kill the kubelet (`parsers.go`);
- a panicking token authenticator must return HTTP 500, not terminate the
  apiserver (`cached_token_authenticator.go` — comment: *"We're leaving the
  request handling stack so we need to handle crashes ourselves"*);
- a panicking *declarative* validator must be downgraded to a metric increment
  when it is not authoritative and to a validation error when it is
  (`validate.go:310`, branching on `shouldFail`);
- a send to a closed audit buffer becomes `"audit backend shut down"`
  (`buffered.go`).

### Rust rules (recover)

| Class | Rule |
|---|---|
| R1 (6 definitions) → 164 call sites | Reimplement `HandleCrash*` **once** as a panic hook (`std::panic::set_hook`) that runs the registered handlers and logs, and leave the abort to the runtime. Because `ReallyCrash` is true, the 162 `defer HandleCrash()` sites become **nothing at all** — the hook is global. This is the largest single deletion in the panic surface, and it depends entirely on §7.3 being read rather than assumed. |
| R2 (8) | `std::thread::spawn` + `JoinHandle::join()` already returns `Err(payload)` on panic; the panic channel disappears and the parent does `resume_unwind(payload)`. Direct, and *simpler* than the Go original. Requires the spawned work to be `UnwindSafe`-clean. |
| R3 (8) — hard | `std::panic::catch_unwind` returning `Result`. **Each site is a policy decision, not a mapping**, and each carries two preconditions the engine cannot discharge alone: the crate must not be built with `panic=abort` (or the boundary must be re-expressed as a process/thread boundary), and the caught closure must satisfy `UnwindSafe` — which for `cached_token_authenticator.go:168` and `buffered.go:264` means the mutated state (`record`, `evIndex`) must be moved behind `AssertUnwindSafe` with an argued justification, not a blanket wrapper. Budget: 8 hand-authored ports with individual receipts. |
| R4 (5) | A Drop guard that performs the compensation, plus normal unwinding. Drop already runs during unwind, so the re-panic is implicit — this class becomes *simpler* in Rust, provided the guard does not itself panic (double-panic aborts). |
| R5 (3) | `catch_unwind` + log. Same preconditions as R3. Two of the three are diagnostics-only and are candidates for deletion rather than translation. |
| R6 (2) | Do **not** translate structurally. These exist only to cancel `HandleCrash`'s re-panic; under the R1 hook design there is nothing to cancel. Replace with an explicit `catch_unwind` at that boundary and a receipt recording that the die-policy was intentionally overridden upstream. |
| R7 (3) | These are not panics, they are exceptions with a private payload type. Translate the *protocol*, not the panic: a private error enum returned as `Result` through the call chain. The re-panic-if-not-mine branch becomes a type-level impossibility. |

**Programme-level consequence.** The choice of `panic=unwind` vs `panic=abort`
for the ported crates is decided by exactly **13 prod sites** (R3 = 8 + R5 = 3 +
R6 = 2 — the resuming set of §7.5). If the port instead re-expresses those 13 as
process or thread boundaries, the whole corpus can be built `panic=abort` and
`catch_unwind` never appears. That is a tractable, countable decision — and it is
a decision, which is why it belongs to the programme and not to the engine.

---

## 8. What this census could NOT determine

Stated plainly, with the cost of answering each:

1. **Whether a given `panic()` is "recoverable control flow" or a genuine
   invariant violation.** Needs a whole-program call graph: is there a frame
   between the panic and `main` that recovers? Cost: `go/packages` type-checked
   load of the corpus plus a call-graph pass (RTA or CHA from `golang.org/x/tools`),
   which is a substantially larger harness than the one used here. The
   *syntactic* answer (§7.1) is a shape distribution and is not a substitute.
   The three known control-flow protocols (R7) were found by reading the recover
   sites *backwards*, which is a sound method precisely because there are only 35.
2. **Whether a deferred call can itself panic.** Determines whether the Rust Drop
   guard risks a double-panic abort. Needs types and a call graph.
3. **Whether the value passed to `panic(err)` is an `error`.** 310 sites are
   `panic(err)` by name; the type is assumed, not measured. Only affects the
   choice between `.expect()` and `resume_unwind`.
4. **`UnwindSafe`-ness of the 8 R3 closures.** A Rust property, not a Go one;
   it can only be settled by attempting the port.
5. **Recall of the shape-4 detector.** Its false-positive rate is measured (4 of
   6, all verified by reading); its false-negative rate is not, because
   establishing it would require checking all 3 444 `args-stable` sites for
   reassignment through pointers and fields. The reported figure of 2 is a
   verified lower bound.
6. **`vendor/` is entirely unmeasured** — 4 354 files, roughly 2 400 additional
   `defer` statements by line count. If vendored dependencies are in port scope,
   this census must be re-run with the exclusion lifted; nothing here can be
   extrapolated to them.

---

## 9. Summary — rule-corpus sizing

Prod scope (D3, 9 573 files). "Shapes" is the number of distinct rules the port
engine needs, which is the number that sizes the programme.

| Surface | Sites | Shapes | Engine cost |
|---|---:|---:|---|
| `defer` total | 4 294 | 6 shapes cover 77.9 % | Low. `Unlock`/`RUnlock` (2 062) and `Done` (237) **delete**; they do not translate. |
| `defer` in loop | 27 | 7 sub-shapes | Low volume, high care. 19 IIFE counter-shape sites translate free; the 27 need a hard-stop detector and per-site receipts — 3 of them become functional defects under a naive scope-drop. |
| `defer` mutating named result | 24 (19 + 5) | 2 channels | Restructure, no analogue — but 0.56 % of defers, **not** the common case the brief anticipated. |
| `defer` argument capture that matters | 2 verified (6 syntactic) | 1 | One unconditional rule (bind args into locals at the defer) makes all 4 294 correct. |
| `panic(` | 1 339 | top 3 = 70.1 % | 512 vanish with the type system (generator rule); ~400 map one-to-one; ~200 wait on the error model. |
| `recover()` + packaged | 283 (35 + 162 + 2 + 84) | **7 policy classes** | 164 collapse into one panic hook. **13 sites decide `panic=unwind` vs `panic=abort` for the whole port.** |

The headline for programme sizing: on this surface the corpus is far more
uniform than its size suggests. `defer` is 78 % six callee shapes, `panic` is
70 % three shapes, and the entire 283-site panic-boundary surface reduces to
**seven** distinct failure policies — of which exactly **one** (R3, 8 sites) is
genuinely hard.

---

## Appendix A — measurement harness

Throwaway measurement provenance, reproduced here so every figure above is
runnable as written. It is **not** a repository artifact and must not be
committed as code: the port engine's own Go front end supersedes it. Save each
block to a file of the given name in an empty directory and run
`go run <file>.go <corpus-root>`. Requires only the Go standard library.

<details>
<summary><code>census.go</code> — defer shapes 1–4, panic classification, recover sites</summary>

```go
// census.go — syntactic (AST, no type info) census of defer / panic / recover
// over a pinned Kubernetes checkout.
//
//	go run census.go <corpus-root> > out.tsv
//
// Emits one TSV record per finding:  KIND \t CLASS \t FILE:LINE \t DETAIL
// Counting is done by the caller (sort | uniq -c), so every number in the
// report is traceable to concrete sites.
package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
)

var root string

func rel(fset *token.FileSet, p token.Pos) string {
	pos := fset.Position(p)
	r, _ := filepath.Rel(root, pos.Filename)
	return fmt.Sprintf("%s:%d", r, pos.Line)
}

func emit(kind, class, loc, detail string) {
	detail = strings.ReplaceAll(detail, "\t", " ")
	detail = strings.ReplaceAll(detail, "\n", " ")
	if len(detail) > 160 {
		detail = detail[:160]
	}
	fmt.Printf("%s\t%s\t%s\t%s\n", kind, class, loc, detail)
}

var srcCache = map[string][]byte{}

func exprStr(fset *token.FileSet, e ast.Expr) string {
	if e == nil {
		return ""
	}
	s := fset.Position(e.Pos()).Offset
	f := fset.Position(e.End()).Offset
	if src, ok := srcCache[fset.Position(e.Pos()).Filename]; ok && s >= 0 && f <= len(src) && s < f {
		return string(src[s:f])
	}
	return fmt.Sprintf("%T", e)
}

// callName renders the callee in a normalised form: "pkg.Fn", "recv.Method",
// "Fn", or "func-literal".
func callName(fun ast.Expr) string {
	switch f := fun.(type) {
	case *ast.Ident:
		return f.Name
	case *ast.SelectorExpr:
		if x, ok := f.X.(*ast.Ident); ok {
			return x.Name + "." + f.Sel.Name
		}
		return "?." + f.Sel.Name
	case *ast.FuncLit:
		return "func-literal"
	case *ast.IndexExpr:
		return callName(f.X)
	case *ast.IndexListExpr:
		return callName(f.X)
	case *ast.ParenExpr:
		return callName(f.X)
	}
	return "other"
}

func namedResults(ft *ast.FuncType) map[string]bool {
	out := map[string]bool{}
	if ft == nil || ft.Results == nil {
		return out
	}
	for _, f := range ft.Results.List {
		for _, n := range f.Names {
			if n.Name != "_" {
				out[n.Name] = true
			}
		}
	}
	return out
}

// assignedIdents: identifiers used as assignment targets, ++/-- targets, or
// operands of & (address-taken), anywhere in n.
func assignedIdents(n ast.Node) map[string]bool {
	out := map[string]bool{}
	ast.Inspect(n, func(x ast.Node) bool {
		switch s := x.(type) {
		case *ast.AssignStmt:
			for _, l := range s.Lhs {
				if id, ok := l.(*ast.Ident); ok {
					out[id.Name] = true
				}
			}
		case *ast.IncDecStmt:
			if id, ok := s.X.(*ast.Ident); ok {
				out[id.Name] = true
			}
		case *ast.UnaryExpr:
			if s.Op == token.AND {
				if id, ok := s.X.(*ast.Ident); ok {
					out[id.Name] = true
				}
			}
		}
		return true
	})
	return out
}

// declaredIdents: identifiers introduced by :=, var, or a param/result name
// inside n. Used to exclude shadowing of a named result.
func declaredIdents(n ast.Node) map[string]bool {
	out := map[string]bool{}
	ast.Inspect(n, func(x ast.Node) bool {
		switch s := x.(type) {
		case *ast.AssignStmt:
			if s.Tok == token.DEFINE {
				for _, l := range s.Lhs {
					if id, ok := l.(*ast.Ident); ok {
						out[id.Name] = true
					}
				}
			}
		case *ast.ValueSpec:
			for _, id := range s.Names {
				out[id.Name] = true
			}
		case *ast.Field:
			for _, id := range s.Names {
				out[id.Name] = true
			}
		}
		return true
	})
	return out
}

// rootIdent returns the leftmost identifier of an expression (x for x.y.z[0]).
func rootIdent(e ast.Expr) string {
	for {
		switch v := e.(type) {
		case *ast.Ident:
			return v.Name
		case *ast.SelectorExpr:
			e = v.X
		case *ast.IndexExpr:
			e = v.X
		case *ast.CallExpr:
			return ""
		case *ast.ParenExpr:
			e = v.X
		case *ast.StarExpr:
			e = v.X
		case *ast.UnaryExpr:
			e = v.X
		default:
			return ""
		}
	}
}

type fnCtx struct {
	fset    *token.FileSet
	body    *ast.BlockStmt
	ftype   *ast.FuncType
	name    string
	file    string
	isTest  bool
	results map[string]bool
}

// walk tracks loop depth and stops at nested FuncLit boundaries: a defer inside
// a FuncLit belongs to that FuncLit, because that is where it fires.
func (c *fnCtx) walk(n ast.Node, loopDepth int, inDeferredLit bool) {
	switch s := n.(type) {
	case nil:
		return
	case *ast.FuncLit:
		return
	case *ast.ForStmt:
		c.walkChildren(s.Init, loopDepth, inDeferredLit)
		c.walkChildren(s.Post, loopDepth, inDeferredLit)
		c.walkChildren(s.Body, loopDepth+1, inDeferredLit)
		return
	case *ast.RangeStmt:
		c.walkChildren(s.Body, loopDepth+1, inDeferredLit)
		return
	case *ast.DeferStmt:
		c.onDefer(s, loopDepth)
		return
	}
	c.walkChildren(n, loopDepth, inDeferredLit)
}

func (c *fnCtx) walkChildren(n ast.Node, loopDepth int, inDeferredLit bool) {
	if n == nil {
		return
	}
	ast.Inspect(n, func(x ast.Node) bool {
		if x == nil || x == n {
			return x == n
		}
		switch x.(type) {
		case *ast.FuncLit, *ast.ForStmt, *ast.RangeStmt, *ast.DeferStmt:
			c.walk(x, loopDepth, inDeferredLit)
			return false
		}
		return true
	})
}

func (c *fnCtx) onDefer(d *ast.DeferStmt, loopDepth int) {
	loc := rel(c.fset, d.Pos())
	scope := "prod"
	if c.isTest {
		scope = "test"
	}
	call := d.Call

	emit("defer", "total/"+scope, loc, callName(call.Fun))

	if loopDepth > 0 {
		emit("defer-in-loop", "loop/"+scope, loc, callName(call.Fun))
	}

	if lit, ok := call.Fun.(*ast.FuncLit); ok {
		if len(c.results) > 0 {
			shadow := declaredIdents(lit)
			assigned := assignedIdents(lit.Body)
			hit := []string{}
			for name := range c.results {
				if assigned[name] && !shadow[name] {
					hit = append(hit, name)
				}
			}
			if len(hit) > 0 {
				hasRecover := false
				ast.Inspect(lit.Body, func(x ast.Node) bool {
					if ce, ok := x.(*ast.CallExpr); ok && callName(ce.Fun) == "recover" {
						hasRecover = true
					}
					return true
				})
				cls := "assign"
				if hasRecover {
					cls = "recover-to-named-result"
				}
				emit("defer-named-result", cls+"/"+scope, loc, strings.Join(hit, ","))
			} else {
				reads := false
				ast.Inspect(lit.Body, func(x ast.Node) bool {
					if id, ok := x.(*ast.Ident); ok && c.results[id.Name] {
						reads = true
					}
					return true
				})
				if reads {
					emit("defer-named-result", "read-only/"+scope, loc, "")
				}
			}
		}
		emit("defer-shape", "func-literal/"+scope, loc, "")
	} else {
		emit("defer-shape", "call/"+scope, loc, callName(call.Fun))
	}

	// Argument capture: args and the receiver base are evaluated at defer time.
	captured := map[string]bool{}
	for _, a := range call.Args {
		if r := rootIdent(a); r != "" {
			captured[r] = true
		}
	}
	if sel, ok := call.Fun.(*ast.SelectorExpr); ok {
		if r := rootIdent(sel.X); r != "" {
			captured["recv:"+r] = true
		}
	}
	if len(call.Args) == 0 && len(captured) == 0 {
		emit("defer-capture", "no-args-no-recv/"+scope, loc, "")
		return
	}
	later := map[string]bool{}
	ast.Inspect(c.body, func(x ast.Node) bool {
		if x == nil {
			return false
		}
		if x.Pos() <= d.End() {
			return x.End() > d.End()
		}
		switch s := x.(type) {
		case *ast.AssignStmt:
			for _, l := range s.Lhs {
				if id, ok := l.(*ast.Ident); ok {
					later[id.Name] = true
				}
			}
		case *ast.IncDecStmt:
			if id, ok := s.X.(*ast.Ident); ok {
				later[id.Name] = true
			}
		case *ast.UnaryExpr:
			if s.Op == token.AND {
				if id, ok := s.X.(*ast.Ident); ok {
					later[id.Name] = true
				}
			}
		}
		return true
	})
	matters := []string{}
	for k := range captured {
		name := strings.TrimPrefix(k, "recv:")
		if later[name] {
			matters = append(matters, k)
		}
	}
	if len(matters) > 0 {
		cls := "arg-reassigned-later"
		allRecv := true
		for _, m := range matters {
			if !strings.HasPrefix(m, "recv:") {
				allRecv = false
			}
		}
		if allRecv {
			cls = "receiver-reassigned-later"
		}
		emit("defer-capture", cls+"/"+scope, loc, strings.Join(matters, ","))
	} else {
		emit("defer-capture", "args-stable/"+scope, loc, "")
	}
}

func panicClass(fset *token.FileSet, call *ast.CallExpr, enclosing string, recoveredVars map[string]bool) (string, string) {
	if len(call.Args) != 1 {
		return "no-arg", ""
	}
	a := call.Args[0]
	switch v := a.(type) {
	case *ast.BasicLit:
		return "string-literal", exprStr(fset, a)
	case *ast.Ident:
		if recoveredVars[v.Name] {
			return "repanic-recovered", v.Name
		}
		return "ident", v.Name
	case *ast.CallExpr:
		n := callName(v.Fun)
		switch n {
		case "fmt.Sprintf", "fmt.Sprint", "fmt.Sprintln":
			return "fmt-message", n
		case "fmt.Errorf", "errors.New":
			return "error-value", n
		}
		return "call:" + n, n
	case *ast.SelectorExpr:
		if recoveredVars[rootIdent(v)] {
			return "repanic-recovered", exprStr(fset, a)
		}
		return "selector", exprStr(fset, a)
	case *ast.BinaryExpr:
		return "string-concat", exprStr(fset, a)
	case *ast.CompositeLit:
		return "struct-value", exprStr(fset, a)
	}
	return "other", exprStr(fset, a)
}

func main() {
	root = os.Args[1]
	fset := token.NewFileSet()

	filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			base := info.Name()
			if base == ".git" || base == "vendor" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(path, ".go") {
			return nil
		}
		src, err := os.ReadFile(path)
		if err != nil {
			return nil
		}
		srcCache[path] = src
		f, err := parser.ParseFile(fset, path, src, parser.ParseComments)
		if err != nil {
			r, _ := filepath.Rel(root, path)
			emit("parse-error", "fail", r+":0", err.Error())
			return nil
		}
		isTest := strings.HasSuffix(path, "_test.go")
		scope := "prod"
		if isTest {
			scope = "test"
		}
		rp, _ := filepath.Rel(root, path)
		emit("file", scope, rp+":0", "")

		analyzeFn := func(name string, ft *ast.FuncType, body *ast.BlockStmt) {
			if body == nil {
				return
			}
			c := &fnCtx{fset: fset, body: body, ftype: ft, name: name, file: rp,
				isTest: isTest, results: namedResults(ft)}
			c.walkChildren(body, 0, false)
		}
		var curFn string
		ast.Inspect(f, func(n ast.Node) bool {
			switch d := n.(type) {
			case *ast.FuncDecl:
				curFn = d.Name.Name
				analyzeFn(d.Name.Name, d.Type, d.Body)
			case *ast.FuncLit:
				analyzeFn(curFn+".func", d.Type, d.Body)
			}
			return true
		})

		recoveredVars := map[string]bool{}
		ast.Inspect(f, func(n ast.Node) bool {
			as, ok := n.(*ast.AssignStmt)
			if !ok {
				return true
			}
			for i, r := range as.Rhs {
				if ce, ok := r.(*ast.CallExpr); ok && callName(ce.Fun) == "recover" {
					if i < len(as.Lhs) {
						if id, ok := as.Lhs[i].(*ast.Ident); ok {
							recoveredVars[id.Name] = true
						}
					}
				}
			}
			return true
		})

		type span struct {
			name       string
			lo, hi     token.Pos
			results    map[string]bool
			isFuncLit  bool
			deferredAt token.Pos
		}
		var spans []span
		deferredLits := map[*ast.FuncLit]bool{}
		ast.Inspect(f, func(n ast.Node) bool {
			if ds, ok := n.(*ast.DeferStmt); ok {
				if lit, ok := ds.Call.Fun.(*ast.FuncLit); ok {
					deferredLits[lit] = true
				}
			}
			return true
		})
		var topName string
		ast.Inspect(f, func(n ast.Node) bool {
			switch d := n.(type) {
			case *ast.FuncDecl:
				topName = d.Name.Name
				if d.Body != nil {
					spans = append(spans, span{d.Name.Name, d.Body.Pos(), d.Body.End(), namedResults(d.Type), false, 0})
				}
			case *ast.FuncLit:
				dl := token.Pos(0)
				if deferredLits[d] {
					dl = d.Pos()
				}
				spans = append(spans, span{topName + ".func", d.Body.Pos(), d.Body.End(), namedResults(d.Type), true, dl})
			}
			return true
		})
		innermost := func(p token.Pos) span {
			best := span{name: "?"}
			bestw := token.Pos(1 << 40)
			for _, s := range spans {
				if s.lo <= p && p <= s.hi && s.hi-s.lo < bestw {
					best, bestw = s, s.hi-s.lo
				}
			}
			return best
		}

		ast.Inspect(f, func(n ast.Node) bool {
			ce, ok := n.(*ast.CallExpr)
			if !ok {
				return true
			}
			name := callName(ce.Fun)
			loc := rel(fset, ce.Pos())
			switch name {
			case "recover":
				s := innermost(ce.Pos())
				cls := "bare"
				if s.isFuncLit && s.deferredAt != 0 {
					cls = "in-deferred-literal"
				} else if s.isFuncLit {
					cls = "in-non-deferred-literal"
				} else {
					cls = "in-named-func"
				}
				res := "no-named-result"
				if len(s.results) > 0 {
					res = "has-named-result"
				}
				emit("recover", cls+"/"+scope, loc, s.name+"|"+res+"|"+rp)
			case "panic":
				cl, det := panicClass(fset, ce, "", recoveredVars)
				s := innermost(ce.Pos())
				emit("panic", cl+"/"+scope, loc, s.name+"|"+det)
			case "runtime.HandleCrash", "runtime.HandleCrashWithContext", "runtime.HandleCrashWithLogger",
				"utilruntime.HandleCrash", "utilruntime.HandleCrashWithContext", "utilruntime.HandleCrashWithLogger":
				emit("handlecrash", name+"/"+scope, loc, rp)
			}
			return true
		})
		return nil
	})
}
```

Known limitation, stated because a figure depends on it: the `recover` record's
`has-named-result` field reports the results of the *innermost* function
(usually the deferred literal itself, which has none), not of the enclosing
function. `census2.go`'s `recover2` record computes the enclosing-chain answer
correctly and is the one used in §5. The two agree on the total (35 prod).

</details>

<details>
<summary><code>census2.go</code> — IIFE counter-shape, <code>&amp;namedResult</code> channel, enclosing-scope recover</summary>

```go
// census2.go — second pass: shapes census.go does not cover.
//   go run census2.go <corpus-root> > out2.tsv
package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
)

var root string

func rel(fset *token.FileSet, p token.Pos) string {
	pos := fset.Position(p)
	r, _ := filepath.Rel(root, pos.Filename)
	return fmt.Sprintf("%s:%d", r, pos.Line)
}

func emit(kind, class, loc, detail string) {
	detail = strings.ReplaceAll(detail, "\t", " ")
	detail = strings.ReplaceAll(detail, "\n", " ")
	if len(detail) > 200 {
		detail = detail[:200]
	}
	fmt.Printf("%s\t%s\t%s\t%s\n", kind, class, loc, detail)
}

func callName(fun ast.Expr) string {
	switch f := fun.(type) {
	case *ast.Ident:
		return f.Name
	case *ast.SelectorExpr:
		if x, ok := f.X.(*ast.Ident); ok {
			return x.Name + "." + f.Sel.Name
		}
		return "?." + f.Sel.Name
	case *ast.FuncLit:
		return "func-literal"
	case *ast.IndexExpr:
		return callName(f.X)
	case *ast.IndexListExpr:
		return callName(f.X)
	case *ast.ParenExpr:
		return callName(f.X)
	}
	return "other"
}

func namedResults(ft *ast.FuncType) map[string]bool {
	out := map[string]bool{}
	if ft == nil || ft.Results == nil {
		return out
	}
	for _, f := range ft.Results.List {
		for _, n := range f.Names {
			if n.Name != "_" {
				out[n.Name] = true
			}
		}
	}
	return out
}

type frame struct {
	ft      *ast.FuncType
	name    string
	results map[string]bool
}

func main() {
	root = os.Args[1]
	fset := token.NewFileSet()

	filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			if info.Name() == ".git" || info.Name() == "vendor" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(path, ".go") {
			return nil
		}
		src, _ := os.ReadFile(path)
		f, err := parser.ParseFile(fset, path, src, 0)
		if err != nil {
			return nil
		}
		scope := "prod"
		if strings.HasSuffix(path, "_test.go") {
			scope = "test"
		}
		rp, _ := filepath.Rel(root, path)

		var stack []frame
		var loopDepth []int

		var visit func(n ast.Node)
		pushFn := func(name string, ft *ast.FuncType) {
			stack = append(stack, frame{ft, name, namedResults(ft)})
			loopDepth = append(loopDepth, 0)
		}
		popFn := func() {
			stack = stack[:len(stack)-1]
			loopDepth = loopDepth[:len(loopDepth)-1]
		}
		curDepth := func() int {
			if len(loopDepth) == 0 {
				return 0
			}
			return loopDepth[len(loopDepth)-1]
		}
		bump := func(d int) {
			if len(loopDepth) > 0 {
				loopDepth[len(loopDepth)-1] += d
			}
		}
		enclosingResults := func() map[string]bool {
			out := map[string]bool{}
			for _, fr := range stack {
				for k := range fr.results {
					out[k] = true
				}
			}
			return out
		}
		outerResultsOnly := func() map[string]bool {
			out := map[string]bool{}
			for i := 0; i < len(stack)-1; i++ {
				for k := range stack[i].results {
					out[k] = true
				}
			}
			return out
		}

		visit = func(n ast.Node) {
			if n == nil {
				return
			}
			switch s := n.(type) {
			case *ast.FuncDecl:
				pushFn(s.Name.Name, s.Type)
				if s.Body != nil {
					for _, st := range s.Body.List {
						visit(st)
					}
				}
				popFn()
				return
			case *ast.FuncLit:
				pushFn("func-lit", s.Type)
				for _, st := range s.Body.List {
					visit(st)
				}
				popFn()
				return
			case *ast.ForStmt:
				bump(1)
				visit(s.Body)
				bump(-1)
				return
			case *ast.RangeStmt:
				bump(1)
				visit(s.Body)
				bump(-1)
				return
			case *ast.DeferStmt:
				loc := rel(fset, s.Pos())
				call := s.Call
				if _, isLit := call.Fun.(*ast.FuncLit); !isLit {
					res := enclosingResults()
					for _, a := range call.Args {
						if u, ok := a.(*ast.UnaryExpr); ok && u.Op == token.AND {
							if id, ok := u.X.(*ast.Ident); ok && res[id.Name] {
								emit("defer-ptr-to-named-result", callName(call.Fun)+"/"+scope, loc, id.Name)
							}
						}
					}
				} else {
					lit := call.Fun.(*ast.FuncLit)
					outer := outerResultsOnly()
					if len(outer) > 0 {
						ast.Inspect(lit.Body, func(x ast.Node) bool {
							if as, ok := x.(*ast.AssignStmt); ok && as.Tok != token.DEFINE {
								for _, l := range as.Lhs {
									if id, ok := l.(*ast.Ident); ok && outer[id.Name] {
										emit("defer-outer-named-result", "assign/"+scope, loc, id.Name)
									}
								}
							}
							return true
						})
					}
				}
				ast.Inspect(call, func(x ast.Node) bool {
					if fl, ok := x.(*ast.FuncLit); ok && fl != call.Fun {
						visit(fl)
						return false
					}
					return true
				})
				if fl, ok := call.Fun.(*ast.FuncLit); ok {
					pushFn("deferred-lit", fl.Type)
					for _, st := range fl.Body.List {
						visit(st)
					}
					popFn()
				}
				return
			case *ast.ExprStmt:
				if ce, ok := s.X.(*ast.CallExpr); ok {
					if fl, ok := ce.Fun.(*ast.FuncLit); ok {
						hasDefer := false
						ast.Inspect(fl.Body, func(x ast.Node) bool {
							if _, ok := x.(*ast.DeferStmt); ok {
								hasDefer = true
							}
							return true
						})
						if hasDefer && curDepth() > 0 {
							emit("iife-defer-in-loop", "workaround/"+scope, rel(fset, s.Pos()), rp)
						}
					}
				}
			}
			ast.Inspect(n, func(x ast.Node) bool {
				if x == nil || x == n {
					return x == n
				}
				switch x.(type) {
				case *ast.FuncDecl, *ast.FuncLit, *ast.ForStmt, *ast.RangeStmt, *ast.DeferStmt, *ast.ExprStmt:
					visit(x)
					return false
				}
				return true
			})
		}

		for _, d := range f.Decls {
			visit(d)
		}

		var st2 []frame
		var rvisit func(n ast.Node)
		rvisit = func(n ast.Node) {
			if n == nil {
				return
			}
			switch s := n.(type) {
			case *ast.FuncDecl:
				st2 = append(st2, frame{s.Type, s.Name.Name, namedResults(s.Type)})
				if s.Body != nil {
					rvisit(s.Body)
				}
				st2 = st2[:len(st2)-1]
				return
			case *ast.FuncLit:
				st2 = append(st2, frame{s.Type, "func-lit", namedResults(s.Type)})
				rvisit(s.Body)
				st2 = st2[:len(st2)-1]
				return
			case *ast.CallExpr:
				if callName(s.Fun) == "recover" {
					names := []string{}
					has := false
					for _, fr := range st2 {
						if len(fr.results) > 0 {
							has = true
						}
						names = append(names, fr.name)
					}
					cls := "enclosing-no-named-result"
					if has {
						cls = "enclosing-has-named-result"
					}
					emit("recover2", cls+"/"+scope, rel(fset, s.Pos()), rp+"|"+strings.Join(names, ">"))
				}
			}
			ast.Inspect(n, func(x ast.Node) bool {
				if x == nil || x == n {
					return x == n
				}
				switch x.(type) {
				case *ast.FuncDecl, *ast.FuncLit, *ast.CallExpr:
					rvisit(x)
					return false
				}
				return true
			})
		}
		for _, d := range f.Decls {
			rvisit(d)
		}
		return nil
	})
}
```

</details>

<details>
<summary><code>census3.go</code> — named-result denominator, and <code>classify.awk</code></summary>

```go
// census3.go — denominator: how many functions declare named results, and how
// many of those contain at least one deferred func literal.
//   go run census3.go <corpus-root>
package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
)

func named(ft *ast.FuncType) bool {
	if ft == nil || ft.Results == nil {
		return false
	}
	for _, f := range ft.Results.List {
		for _, n := range f.Names {
			if n.Name != "_" {
				return true
			}
		}
	}
	return false
}

func main() {
	root := os.Args[1]
	fset := token.NewFileSet()
	var funcs, namedFuncs, namedWithDeferLit int
	filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			if info.Name() == ".git" || info.Name() == "vendor" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(path, ".go") || strings.HasSuffix(path, "_test.go") {
			return nil
		}
		src, _ := os.ReadFile(path)
		f, err := parser.ParseFile(fset, path, src, 0)
		if err != nil {
			return nil
		}
		check := func(ft *ast.FuncType, body *ast.BlockStmt) {
			if body == nil {
				return
			}
			funcs++
			if !named(ft) {
				return
			}
			namedFuncs++
			found := false
			ast.Inspect(body, func(x ast.Node) bool {
				if ds, ok := x.(*ast.DeferStmt); ok {
					if _, ok := ds.Call.Fun.(*ast.FuncLit); ok {
						found = true
					}
				}
				return true
			})
			if found {
				namedWithDeferLit++
			}
		}
		ast.Inspect(f, func(n ast.Node) bool {
			switch d := n.(type) {
			case *ast.FuncDecl:
				check(d.Type, d.Body)
			case *ast.FuncLit:
				check(d.Type, d.Body)
			}
			return true
		})
		return nil
	})
	fmt.Printf("prod functions (decl+literal, with body): %d\n", funcs)
	fmt.Printf("  of which declare named results:         %d\n", namedFuncs)
	fmt.Printf("  of those, containing a deferred FuncLit: %d\n", namedWithDeferLit)
}
```

`classify.awk`, which produces the §7.1 table from `out.tsv`
(`awk -f classify.awk out.tsv | sort`):

```awk
BEGIN { FS = "\t" }
$1 == "panic" && $2 ~ /prod$/ {
  split($4, a, "|"); fn = a[1]; arg = a[2]; cls = $2; sub(/\/prod$/, "", cls)
  if (arg ~ /^"nil value passed to /)                 b = "1-generated-builder-nil-guard"
  else if (cls == "repanic-recovered")                b = "2-repanic-recovered"
  else if (fn ~ /OrDie/)                              b = "3-OrDie-constructor"
  else if (fn ~ /^Must/ || fn ~ /Must[A-Z]/)          b = "4-Must-constructor"
  else if (fn == "main" || fn == "init" || fn ~ /^main\.func/ || fn ~ /^init\.func/) b = "5-main-or-init"
  else if (cls == "ident" && arg == "err")            b = "6-panic-err-in-library"
  else if (cls == "string-literal")                   b = "7-string-invariant"
  else if (cls == "fmt-message")                      b = "8-formatted-invariant"
  else if (cls == "error-value")                      b = "9-constructed-error"
  else                                                b = "Z-other:" cls
  n[b]++; total++
}
END {
  for (k in n) printf "%-34s %5d  %5.1f%%\n", k, n[k], 100*n[k]/total
  printf "%-34s %5d\n", "TOTAL", total
}
```

</details>
