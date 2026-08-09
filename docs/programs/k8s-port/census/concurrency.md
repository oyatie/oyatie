---
doc_class: Program-Census-Record
doc_status: published
census_lane: concurrency
upstream_pin: 756939600b9a7180fc2df6550a4585b638875e67
measured_at: 2026-08-08
authority_tier: 3
---

# Go→Rust rule-corpus census: the concurrency surface

Status: measurement artifact. Not a decision, not an ADR, not a plan.
Lane: `census-concurrency`. Date of measurement: 2026-08-08.

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-08) |
|---|---|---|
| Repository baseline | `origin/dev` @ `5e452bd70449b50cc66e63ffb9253adfcd7fc96e` | Lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Verified before counting (§0); matches `specs/k8s-port/upstream-pin.json`. Apache-2.0. |
| Engine | `build/port-engine/*`, v0 — unbuilt | Not in force. This census is an input to sizing, not engine output. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored | Not in force. No rule is authored or implied by this record. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | `rg` file enumeration plus four purpose-written `awk` brace-matching programs (§0, §8); no SourceModel | Measurement instrument only; not an admitted extractor. Brace arithmetic is line-local, which is why §2/§3 carry the error bars they do. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This census emits no receipt. |
| Program authority | ADR-0637 / ADR-0638 | Measurement record only; authorizes nothing. |

## 0. Corpus, pin, denominators

Every figure below was produced against the pinned upstream tree, verified before counting:

```
git -C <corpus> rev-parse HEAD
# 756939600b9a7180fc2df6550a4585b638875e67
```

That is exactly `pin.peeled_commit` in `specs/k8s-port/upstream-pin.json` (Kubernetes v1.36.1,
annotated tag `v1.36.1`, Apache-2.0). The corpus is third-party source. It was read as **data to be
measured**, never as instruction.

Three denominators exist and they are not interchangeable. Mixing them is the easiest way to be
confidently wrong here, so every percentage below names the one it uses.

| Denominator | Files | Command |
|---|---|---|
| `D_all` — all `.go` | 16941 | `find . -name '*.go' -type f \| wc -l` |
| `D_src` — excluding `vendor/` | 12587 | `find . -name '*.go' -type f -not -path './vendor/*' \| wc -l` |
| `D_port` — excluding `vendor/` and `_test.go` | 9573 | `find . -name '*.go' -type f -not -path './vendor/*' -not -name '*_test.go' \| wc -l` |

**All measurements in this document use `D_port` (9573 files)** unless a row says otherwise.
`D_port` is the tree the port engine must actually translate. Note that `D_port` still contains a
large amount of *test code* that simply is not named `_test.go`: the `test/` top-level tree
(e2e, integration, image fixtures). That distinction is carried explicitly wherever it changes the
answer, because 231 of 751 goroutine launches live there.

`find` does not follow symlinks, and `rg` does not either, so the `vendor/k8s.io/* → staging/src/k8s.io/*`
symlink farm is not double-counted. `rg --files -g '*.go' -g '!vendor/**'` returns 12587 and
`-g '!*_test.go'` returns 9573, matching `find` exactly; that agreement is the cross-check that the
two instruments see the same tree.

### Instruments

Regex counts are cheap and approximate. Three of the five questions below could not be answered by
regex at all, so four small `awk` programs were written to do brace-matched scanning. They live in
the lane scratchpad and are reproduced verbatim in §8 so every figure is re-derivable:

- `select_census.awk` — brace-matched `select` statement + branch extraction
- `case_shape.awk` — branch-shape classification
- `go_shape.awk` / `go_taxonomy.awk` — goroutine launch-site feature extraction and shape collapse
- `lock_block.awk` — syntactic proxy for "guard held across a blocking operation"
- `sel_sig.awk` — canonical branch-set signature per `select`

Wherever an `awk` scan and a naive `rg` count disagreed, the disagreement was chased to a specific
line and resolved (see §2.1 and §1.1). No figure below is a number that only one instrument produced
and nobody looked at.

---

## 1. Goroutine launch sites

### 1.1 Total — 751, and this one is very nearly exact

```
rg -n --no-heading -g '*.go' -g '!vendor/**' -g '!*_test.go' '^[[:space:]]*go [[:alnum:]_(]' .
```

→ **752 matching lines in 390 files**, of which exactly **1 is a false positive**
(`pkg/volume/util/nestedpendingoperations/nestedpendingoperations.go:21`, the phrase "go routine"
inside a `/* */` package comment; found by `grep -v '('` over the match set — it is the only match
lacking a call paren). **True total: 751.**

This is the rarest thing in this document: a count that is *effectively exact* rather than a bound.
Three checks establish that:

- **No mid-line `go` statements exist.** `rg '[{;][[:space:]]*go [[:alnum:]_(]'` returns **0 matches
  in 0 files**. `gofmt` never emits `if x { go f() }` on one line, so line-anchoring loses nothing.
- **No line carries two `go` statements.** `rg -o '(^|[{;])[[:space:]]*go [[:alnum:]_(]' | wc -l`
  = 752, identical to the line count.
- **Comment lines cannot match.** The anchor is `^\s*go `, so `// go func` and `* go func` are
  structurally excluded; only the one un-prefixed line inside a `/* */` block slipped through, and
  it was found and removed.

Residual false-negative risk: a `go` statement inside a multi-line raw-string literal used as a
codegen template would be counted (false positive), and one written non-gofmt would be missed. Both
were searched for and neither was found. Treat 751 as exact ±1.

For reference, including `_test.go` files the same command yields **1736 sites in 702 files**
(denominator `D_src`). The port programme's own scope decides whether that 985-site test surface is
in or out; it roughly *doubles* the concurrency work.

### 1.2 Where they are

```
sed -E 's|^\./([^/]+)/.*|\1|' go-sites.txt | sort | uniq -c | sort -rn
```

| Top-level tree | Sites | Share of 751 |
|---|---:|---:|
| `staging/` (the published client/apiserver libraries) | 299 | 39.8% |
| `test/` (e2e + integration + fixture images) | 231 | 30.8% |
| `pkg/` | 184 | 24.5% |
| `cmd/` | 32 | 4.3% |
| `plugin/` | 1 | 0.1% |
| `build/`, `third_party/`, `cluster/` (tooling) | 4 | 0.5% |

(`pkg/` is 185 raw and 184 true — the one false positive of §1.1 lives there.)

**Finding: nearly a third of all goroutine launches are in `test/`.** They are not `_test.go`, so
they survive the `D_port` filter, but they are e2e scaffolding, not the control plane. The
*production* concurrency surface (`staging` + `pkg` + `cmd` + `plugin`) is **516 launch sites, not
751** — a 31% reduction in the thing that has to be right.

### 1.3 Literal vs named

| Form | Count | Share of 751 |
|---|---:|---:|
| `go func(...) { ... }(...)` closure literal | 351 | 46.7% |
| `go someCall(...)` named call | 400 | 53.3% |

The named half is the important structural fact, because **the shape of a named launch lives in the
callee, not at the launch site**. See §1.6.

### 1.4 The named half is dominated by one library

```
grep -v ':[[:space:]]*go func' go-sites.txt \
  | sed -E 's/^.*:[0-9]+:[[:space:]]*go //' | sed -E 's/\(.*$//' | sort | uniq -c | sort -rn
```

| Callee | Sites |
|---|---:|
| `wait.Until` | 54 |
| `wait.UntilWithContext` | 36 |
| `wait.Forever` | 9 |
| `wait.PollImmediateUntil` | 4 |
| `wait.JitterUntilWithContext` | 3 |
| `wait.JitterUntil` | 2 |
| **`wait.*` subtotal** | **108** |
| `waitStreamReply` (one e2e helper) | 18 |
| `controller.Run` | 18 |
| `cletest.createAndRunFakeController` | 11 |
| everything else (**205 distinct callees**) | 245 |

**108 of 400 named launches (27%) are one library function family.** `go wait.Until(f, period, stopCh)`
*is* the Kubernetes background loop. A single hand-written Rust equivalent of `k8s.io/apimachinery/pkg/util/wait`
collapses 108 launch sites into 108 calls to one already-correct primitive — no per-site rule needed.

The whole `wait` package surface, measured:

```
rg -o --no-filename -g '*.go' -g '!vendor/**' -g '!*_test.go' '\bwait\.[A-Z][A-Za-z]*' .
```

→ **39 distinct symbols, 1182 call sites.** Top five: `PollUntilContextTimeout` 366,
`PollImmediate` 117, `UntilWithContext` 107, `Until` 77, `ForeverTestTimeout` 74.

This is the single highest-leverage observation in the census. §7 returns to it.

### 1.5 Pattern taxonomy — 8 shapes, top 3 cover 64%

Shapes were derived by brace-matching each closure body and extracting a feature vector
(`go_shape.awk`), then collapsing feature vectors to named shapes with an explicit rule set
(`go_taxonomy.awk`). This is *not* eyeballing: the classification is a deterministic function of
the source, so it is reproducible and its errors are inspectable.

`go_shape.awk` extracted **745** of the 751 sites. The 6 missing are `go` statements nested inside
another `go func` literal's body (the extractor does not re-enter an open closure). Percentages below
use 745.

| Shape | Sites | Share | Cum. | What it is |
|---|---:|---:|---:|---|
| **S7** fire-and-forget named call | 173 | 23.2% | 23.2% | `go x.Method(args)` where the shape is in the callee |
| **S1** background loop | 165 | 22.1% | 45.4% | periodic work + shutdown channel/ctx (`wait.Until`, `for { select { tick, done } }`) |
| **S2** long-lived service task | 141 | 18.9% | 64.3% | `go c.Run(ctx)`, `go srv.Serve(l)`, unbounded `for` body |
| **S3** fan-out / worker pool | 74 | 9.9% | 74.2% | launch inside a `for`, joined by `WaitGroup` or reporting on a channel |
| **S9** fire-and-forget closure | 67 | 9.0% | 83.2% | closure, no join, no loop, no channel |
| **S8** consumer / drain | 54 | 7.2% | 90.5% | awaits one channel then acts (signal handlers, shutdown bridges) |
| **S4** joined parallel sub-task | 37 | 5.0% | 95.4% | `WaitGroup`-joined, not in a loop (errgroup-shaped) |
| **S5** request/response bridge | 34 | 4.6% | 100.0% | runs work, returns result or error on a channel |

**Top 3 cover 64.3%. Top 5 cover 83.2%. There are only 8 shapes and no tail beyond them.**

That is the finding the programme should be sized on: the launch-site vocabulary is *small and
closed*, not long-tailed. The candidate vocabulary in the brief maps onto it as follows — worker pool
and fan-out/fan-in are the same measured shape (S3, 74) because in this corpus fan-in is always a
result channel or a `WaitGroup`, never a distinct construct; "one-goroutine-per-request" does not
appear as a distinct shape at all (the apiserver gets that from `net/http`, which owns the goroutine,
not from a `go` statement in this corpus); `errgroup` proper is absent — the corpus uses `sync.WaitGroup`
and the k8s-local `wait.Group` instead; "singleton init" is `sync.Once` (§5), not a goroutine shape.
Pipeline stages exist but are S5/S8, not a separate shape.

### 1.6 Honest limit of the taxonomy: 23% is not syntactically classifiable

S7 (173 sites, 23.2%) is **not a shape**. It is the bucket of named launches whose callee is not
recognisable from the call site. Its method-name residue is a grab-bag:
`waitStreamReply` 18, `createAndRunFakeController` 11, `RunWithContext` 9, `Informer` 5,
`receive` 3, `loop` 3, `SyncLoop` 2, `Start` 2, `startReflector` 2, `Monitor` 2, … .

Several of those are obviously S1 or S2 (`SyncLoop`, `startReflector`, `Monitor`, `Start`) but
proving it requires resolving the callee — that is **semantic, and needs type/call-graph
information this census does not have**. To close it you need a Go type-checked call graph
(`go/packages` + `golang.org/x/tools/go/callgraph`), then re-run the same feature extractor over
each callee body. Until that is done, the taxonomy is **77% syntactically resolved, 23% deferred**.

### 1.7 Taxonomy accuracy — measured, not asserted

A stratified sample of 27 sites (3 per shape: the 1st, 7th and 19th of each shape in file order)
was read in source. **3 of 27 initial labels were wrong (11%)**:

- `pkg/kubelet/kubelet.go:1919` — labelled S9, actually S1 (`wait.JitterUntil` inside the closure with
  no lexical `for`). Rule fixed: a `ticker`/`wait.*` feature now implies S1 without requiring `for`.
- `pkg/controller/job/job_controller.go:1930` — labelled S5, actually S3 (launch inside a `for` with a
  result channel). Rule fixed: `inloop` + result-channel now implies S3.
- `pkg/kubelet/eviction/eviction_manager.go:202` — `go notifier.Start(ctx)`, labelled S7; it is really S2.
  **Not fixed**, because fixing it means matching on `Start$` and that is exactly the callee-resolution
  problem of §1.6. It stands as an instance of the 23% residue, not as a classifier bug.

After the two rule fixes the sampled error rate is **1 in 27 (≈4%)**, and that one error is the
known S7/S2 boundary. The numbers in §1.5 are post-fix. Sample size 27 gives a wide interval —
read "≈4%, plausibly 0–15%" — but the *direction* of the residual error is known and one-sided:
it under-counts S1 and S2 by pushing them into S7.

### 1.8 One sub-shape worth naming separately

```
awk -F'\t' 'NR==FNR && $1=="GO"{f[$2":"$3]=$5; next} {k=$3":"$4; if ($2=="S1-background-loop" && f[k] ~ /inloop/) c++} END{print c}' go-shapes.txt go-taxonomy.txt
```

→ **24 of the 165 S1 sites are inside a `for` loop** — i.e. `for range workers { go wait.UntilWithContext(ctx, c.runWorker, time.Second) }`.
That is the canonical controller `Run(workers int)` pattern: N identical workqueue consumers. It is a
worker pool built out of background loops, and it is the single most-repeated concurrency structure
in the control plane. It deserves its own port rule even though the classifier folds it into S1.

---

## 2. `select` statements

### 2.1 Count: 425 — and the naive count is wrong

```
rg --files -g '*.go' -g '!vendor/**' -g '!*_test.go' . | xargs awk -f select_census.awk
```

→ **425 `select` statements, 885 branches.**

A naive `rg -c 'select[[:space:]]*\{'` gives **426**. The single difference was chased with `comm`:
`test/integration/scheduler_perf/util.go:553`, a `select` inside a commented-out block. The
brace-matched scan is correct and the naive count is not. This is a small thing, but it is the
reason the rest of §2 is trustworthy: the instrument was checked against a second instrument and the
discrepancy was resolved to a line, not waved away.

The scanner was additionally validated by hand against
`staging/src/k8s.io/client-go/tools/cache/reflector.go` (7 selects, branch counts and `default`
presence confirmed by reading) and against every select it reported with <2 branches (13 of them —
all real: 11 are `select {}` block-forever, 2 are genuine single-case selects at
`pkg/kubelet/nodeshutdown/nodeshutdown_manager_linux.go:250` and
`staging/src/k8s.io/kubectl/pkg/drain/drain.go:368`).

Known limitation: literal/comment stripping is line-local, so a `{` or `}` inside a *multi-line* raw
string literal can skew brace depth. No instance was found (the 425/426 reconciliation would have
surfaced it), but the bound is: **425 is exact to the extent that no multi-line raw string in the
corpus contains unbalanced braces.**

### 2.2 Branches per select

```
grep '^SEL' select-census.txt | awk -F'\t' '{print $4}' | sort -n | uniq -c
```

| Branches | Selects | Share | With `default` |
|---:|---:|---:|---:|
| 0 (`select {}`, block forever) | 11 | 2.6% | 0 |
| 1 | 2 | 0.5% | 0 |
| 2 | 367 | 86.4% | 132 |
| 3 | 35 | 8.2% | 3 |
| 4 | 9 | 2.1% | 0 |
| 8 | 1 | 0.2% | 0 |

**86% of selects have exactly two branches.** Mean 2.08. The maximum in the entire corpus is 8.
There is no wide-select problem to solve.

**135 of 425 selects (31.8%) have a `default`.** Of the 2-branch selects, 132/367 (36%) do — i.e.
"`default` present" and "2 branches" are strongly correlated, because the dominant `default` idiom is
non-blocking try-send / try-recv, which needs exactly one real branch.

### 2.3 Branch shapes — 885 branches, 5 kinds

```
awk -f case_shape.awk select-census.txt | awk -F'\t' '{print $2}' | sort | uniq -c | sort -rn
```

| Branch kind | Count | Share of 885 |
|---|---:|---:|
| `case <-ch:` receive, value discarded | 519 | 58.6% |
| `default:` | 135 | 15.3% |
| `case v := <-ch:` receive with binding | 117 | 13.2% |
| `case ch <- v:` **send** | 63 | 7.1% |
| `case v, ok := <-ch:` receive with closed-flag | 51 | 5.8% |

The classifier leaves **zero** branches in an "other" bucket, which is itself the check that the
five kinds are exhaustive for this corpus.

Top receive subjects (of the 519 discarding receives): `ctx.Done()` 104, `stopCh` 34,
`time.After(wait.ForeverTestTimeout)` 21, `ticker.C` 17, `tCtx.Done()` 14, `timeoutChannel` 8,
`done` 8, `stop` 7, then a tail of ≈200 distinct one- and two-occurrence channel names.

### 2.4 The distribution that actually sizes the rule corpus

Occurrences do not size a rule corpus; **branch-set signatures** do. Each select was reduced to the
canonical set of its branch roles — `cancel` (a `Done()`/`stopCh`/`done`/`quit` receive), `timer`
(a `.C`, `.C()` or `time.After(...)` receive), `recv` (any other receive), `send`, `default`:

```
awk -f sel_sig.awk select-census.txt | sort | uniq -c | sort -rn
```

| Signature | Selects | Share | Cumulative |
|---|---:|---:|---:|
| `recv+timer` | 76 | 17.9% | 17.9% |
| `cancel+timer` | 55 | 12.9% | 30.8% |
| `cancel+recv` | 54 | 12.7% | 43.5% |
| `cancel+default` | 54 | 12.7% | 56.2% |
| `default+recv` | 39 | 9.2% | 65.4% |
| `default+send` | 37 | 8.7% | 74.1% |
| `recv` | 34 | 8.0% | 82.1% |
| `cancel+recv+timer` | 21 | 4.9% | 87.1% |
| `cancel+send` | 19 | 4.5% | 91.5% |
| `cancel` | 11 | 2.6% | 94.1% |
| `(empty select{})` | 11 | 2.6% | 96.7% |
| `recv+send` | 5 | 1.2% | 97.9% |
| `timer` | 3 | 0.7% | 98.6% |
| `default+timer` | 3 | 0.7% | 99.3% |
| `send+timer` | 1 | 0.2% | 99.5% |
| `cancel+default+send` | 1 | 0.2% | 99.8% |
| `cancel+default+recv` | 1 | 0.2% | 100.0% |

**17 signatures cover all 425 selects. Top 3 cover 43.5%, top 6 cover 74.1%, top 10 cover 94.1%.**

The tail is *not* long. This is the headline: the `select` rule corpus is bounded at roughly
17 shapes, of which 10 carry 94% of the mass, and 5 of the remaining 7 occur ≤3 times each and can
be handled by hand.

Caveat on this table: the `cancel` vs `recv` split is heuristic — it keys on channel *names*
(`ctx.Done()`, `*stopCh`, `done`, `stop`, `quit`, `*.stopped`). A shutdown channel with an unusual
name is classified `recv`, which inflates `recv+timer` (the largest row) and deflates `cancel+timer`.
The error is one-directional and does not change the shape count, only the split between rows 1 and 2.

### 2.5 Reactor loops

```
rg -c --multiline -g '*.go' -g '!vendor/**' -g '!*_test.go' 'for[ \t]*\{[ \t]*\n[ \t]*select[ \t]*\{' .   # → 107
rg -c --multiline -g '*.go' -g '!vendor/**' -g '!*_test.go' 'for[^\n{]*\{[ \t]*\n([ \t]*\n)?[ \t]*select[ \t]*\{' .  # → 154
```

**154 of 425 selects (36.2%) sit immediately inside a `for` loop.** These are the reactor loops:
`for { select { work, shutdown } }`. This is a lower bound — it only catches a select on the line
immediately following the `for`; a select separated by statements is missed.

---

## 3. Cancellation safety

This is the question the brief calls the deepest risk, and the honest answer is more interesting
than "N branches are unsafe".

### 3.1 What the corpus does *not* contain

A Go `select` branch's communication operation is always a bare channel send or receive. It is never
a composite operation. Go's grammar does not permit `case <-doTheWholeRequest():` to mean "run a
multi-step operation"; it means "evaluate this expression once to obtain a channel, then wait on it".
The *body* of a branch runs only after the branch is chosen, and is never interrupted.

Consequence: **the upstream corpus contains essentially no intrinsically cancellation-unsafe select
branch.** The hazard the brief describes is real, but it is created by the *translation*, not
inherited from the source. A port that fuses a Go branch *body* into a `tokio::select!` arm — the
obvious way to write "run this until shutdown" — manufactures the unsafety. That is a **rule-design
constraint on the port engine**, not a count to be extracted from Kubernetes.

Stated as a rule the engine must obey: a Go `select` branch translates to a `tokio::select!` arm
whose future is *only* the channel operation; the branch body must run *after* the `select!`
completes, in the arm's handler, never inside the awaited expression. Follow that and 2.3's 885
branches are cancellation-safe by construction.

### 3.2 What is genuinely at risk, mechanically counted

Two things survive that argument.

**(a) Send branches — 63 (7.1% of 885), in 63 distinct selects (14.8% of 425).**

`tokio::sync::mpsc::Sender::send(v)` *is* documented cancellation-safe in the narrow sense
(if the future is dropped, the message was not sent). But `v` is **moved into the future**, and
dropping the future drops `v`. Go's `case ch <- v:`, when not selected, leaves `v` intact in the
caller's frame. So the naive arm-per-case translation either fails to compile (use-after-move — a
*good* failure) or silently drops a value the Go code still owned.

Splitting them by whether the select has a `default` is decisive:

| | Selects | Verdict |
|---|---:|---|
| send + `default` | 38 | `try_send` — no future, no cancellation, **mechanical** |
| send, no `default` (blocking send racing another branch) | 25 | **restructure**: `reserve()` then `send`, or re-materialise the value |

The 25 are listed in full in §9. They are not scattered: they are concentrated almost entirely in
**watch-event delivery** — `apimachinery/pkg/watch/mux.go`, `streamwatcher.go`,
`apiserver/pkg/storage/cacher/cache_watcher.go`, `apiserver/pkg/storage/etcd3/watcher.go`,
`client-go/tools/cache/shared_informer.go`, `client-go/tools/watch/retrywatcher.go`,
`dynamic-resource-allocation/.../watcher.go`. A dropped value here is a **lost watch event**, which
in a Kubernetes control plane is a silent correctness failure, not a performance blip.

One of them, read in source to confirm the semantics
(`apiserver/pkg/storage/cacher/cache_watcher.go:214`), is explicitly commented
`// OK, block sending, but only until timer fires.` and returns `true`/`false` to tell the caller
whether delivery happened. The Go code *depends* on the not-taken-send semantics. These 25 are the
highest-value hand-review targets in the whole concurrency surface.

**(b) Branch expressions that are calls — 307 of 687 receive branches (44.7%).**

```
awk -F'\t' '($2 ~ /^recv/) && $3 ~ /\(/' case-shapes.txt | wc -l   # → 307
awk -F'\t' '($2 ~ /^recv/) && $3 ~ /Done\(\)$/' case-shapes.txt | wc -l   # → 152
```

152 of those are `*.Done()` — pure accessors returning a cached channel, zero risk. The remaining 155
are dominated by `w.ResultChan()` 17+8+2, `time.After(...)` ~45 across distinct durations,
`timer.C()`/`t.C()`/`ticker.C()` ~10, `r.clock.After(...)` 2, `pf.streamConn.CloseChan()` 2. All are
accessors or timer constructors. **No branch expression in the corpus was found to be a call with
externally-visible side effects.** Both Go and `tokio::select!` evaluate every branch expression once
per select entry, so the semantics match; this category turned out empty, which is a useful negative.

### 3.3 Sampled estimate of the true restructure rate

Mechanical counting cannot see idioms. So: a deterministic sample of **25 selects** (every 17th of
the 425, sorted by file then line — reproducible, not cherry-picked) was read in source and each
judged as *mechanical arm-per-case* or *restructure*.

Result: **22 mechanical, 3 restructure — 12%.** The three:

1. `apiserver/pkg/registry/generic/registry/decorated_watcher.go:66` — blocking send racing `ctx.Done()`
   (an instance of §3.2a).
2. `apiserver/pkg/storage/cacher/ready.go:186` — `select { case <-r.waitCh: default: close(r.waitCh) }`,
   i.e. *select used as a "is this channel already closed?" test*, under a held mutex. This is not a
   select in Rust at all; it is a `Notify` or an `AtomicBool` + `oneshot`. Idiom rewrite, not translation.
3. `test/integration/scheduler_perf/executor.go:587` — blocking send racing `tCtx.Done()` (§3.2a again).

Confidence: n=25, 3 hits. Point estimate 12%; a Wilson 95% interval is roughly **4%–30%**, i.e.
**17–128 of 425 selects, best estimate ≈51**. That interval is wide and I am not going to pretend
otherwise. It brackets the mechanical figure (25 blocking-send selects + 5 closed-check-idiom
selects = 30) comfortably, and the two methods agreeing to within a factor of two is the main reason
to believe either. **If a single number is needed for planning, use 30–55 selects requiring hand
restructure, with 25 of them already enumerated by name in §9.**

The `close`-check idiom was counted mechanically as a cross-check:
`rg --multiline 'select \{\n[^\n]*case <-[^\n]*:\n[^\n]*\n?[ \t]*default:\n[^\n]*close\('` → **5**.

---

## 4. Mutexes held across blocking operations

### 4.1 The honest framing first

The real question — "how many `MutexGuard`s would be held across an `.await`?" — **cannot be answered
by this census, and cannot be answered syntactically at all.** It requires the async-colouring
fixpoint over the whole call graph: which Go functions become `async fn` in Rust, which depends on
which of their callees do, transitively, seeded by the I/O leaves. That is a whole-program semantic
analysis needing type resolution. This census does not have it.

What was measured instead is a **syntactic lower-bound proxy**, and it is labelled as such:
*a lock is in effect (unreleased `Lock()`/`RLock()`, or a `defer Unlock()` registered in an enclosing
scope) and a line in that scope performs a visible channel op, `select`, `.Wait()` or `time.Sleep()`.*

**Error direction is one-sided and known: this UNDER-counts, probably by a large factor.** It sees
only *visible* blocking operations. It cannot see `mu.Lock(); defer mu.Unlock(); client.Get(ctx, ...)`
— an HTTP call under a lock, which is the case that most reliably becomes a held-guard-across-await.
Over-counting is possible but bounded: lines inside a `go func` launched under the lock are excluded
explicitly (0 of the 103 hits fell in that category), and scope tracking resets `deferred` state when
brace depth leaves the scope that registered the `defer`, so the closure-under-lock false positive is
handled.

### 4.2 The proxy result

```
rg --files -g '*.go' -g '!vendor/**' -g '!*_test.go' . | xargs awk -f lock_block.awk
```

**2273 lock acquisition sites. 103 flagged lines**, none inside a launched goroutine.
Collapsing the 29 `case` lines that belong to an already-flagged `select`:

| Class | Distinct sites | Note |
|---|---:|---|
| `select` under a held lock | 26 | each is a full restructure |
| `.Wait()` under a held lock | 27 | see split below |
| channel receive under a held lock | 14 | |
| channel send under a held lock | 6 | |
| `time.Sleep` under a held lock | 1 | |
| **total distinct** | **74** | |

The `.Wait()` class splits cleanly and the split matters:

- **19 are `cond.Wait()`** — `sync.Cond`, where holding the lock is *required and correct* Go.
  These map to `Condvar::wait(guard)` (sync) or a `Notify` + re-lock loop (async); they are a
  **single well-understood rule**, applied 19 times, not 19 problems. Sites include
  `client-go/tools/cache/{delta_fifo,the_real_fifo,fifo,heap,shared_informer}.go`,
  `client-go/util/workqueue/queue.go` (×2), `apiserver/.../watch_cache.go`,
  `pkg/util/goroutinemap` (×2), `pkg/scheduler/util/assumecache`.
- **8 are `WaitGroup.Wait()` or similar under a lock** — genuine hazards, guaranteed deadlock if the
  goroutines being awaited need the same lock.

Two flagged sites were read in full to confirm the proxy is not producing noise:

- `pkg/controller/garbagecollector/graph_builder.go:305` — `gb.monitorLock.Lock(); defer …Unlock();`
  then `<-gb.informersStarted`. **True positive.** An unbounded wait on a channel with a mutex held.
- `apimachinery/pkg/watch/mux.go:120–134` (`Broadcaster.blockQueue`) — `m.incomingBlock.Lock();
  defer …Unlock();` then a `select` on `m.stopped`, then an **unbuffered send** `m.incoming <- Event{…}`,
  then `wg.Wait()`. **True positive, and the worst single site found.** Its own comment calls it
  "this terrible hack". In Rust this is a compile error with `tokio::Mutex` and a deadlock with
  `std::Mutex`. It is not a translation; it is a redesign.

So: **74 sites is a floor, and it is a floor established on evidence.** The true count once
async-colouring is known will be materially higher. The right next measurement is: build the Go call
graph, mark the I/O leaves, propagate `async`, and re-run this same scan asking "is any *call* in the
guard's live range async-coloured?" That is a bounded, mechanical follow-up — it just is not this census.

---

## 5. Sync primitives, context, and channels

All figures: `D_port` (non-vendor, non-`_test.go`).
Command form: `rg -o --no-filename -g '*.go' -g '!vendor/**' -g '!*_test.go' '<pattern>' . | wc -l`.

### 5.1 sync primitives

| Primitive | Declarations | Call sites |
|---|---:|---:|
| `sync.Mutex` | 368 | 2273 `Lock()`/`RLock()` total (§4) |
| `sync.RWMutex` | 229 | (included above) |
| `sync.Once` | 241 | 141 `once.Do(` |
| `sync.WaitGroup` | 218 | 178 `.Add(1)`/`.Add(len…)`, 265 `.Wait()` |
| `sync.Map` | 30 | |
| `sync.Cond` / `sync.NewCond` | 31 | 19 `cond.Wait()` under lock (§4.2) |
| `sync.Pool` | 18 | |
| `atomic.Value` / `Bool` / `Pointer` / `IntN` / `UintN` | 47 / 45 / 35 / 27+28+18+5 / 7+3 | |
| `runtime.HandleCrash` | 38 | |
| `wait.Group` (k8s-local) | 4 | |
| `errgroup.*` | **2** | `errgroup.WithContext` ×1, `errgroup.Group{}` ×1 |

Note these are *type-mention* counts (declarations, embedded fields, composite literals), not
distinct variables — a lower bound on instances and an upper bound on distinct types. `atomic.AddInt`
etc. are truncated at the width suffix by the regex (`atomic\.[A-Za-z]+` stops before `64`), so the
`atomic.*` rows aggregate `Int32`/`Int64` together; that is intentional, since the port rule is the
same for both.

`errgroup` is worth flagging because the brief lists it as candidate vocabulary. It exists in the
corpus but is **vestigial: 2 call sites total**, at
`pkg/controlplane/controller/leaderelection/leaderelection_controller.go:270` and
`staging/src/k8s.io/cli-runtime/pkg/resource/visitor.go:211`
(`rg -n 'errgroup' -g '*.go' -g '!vendor/**'` → 4 lines: 2 imports, 2 uses).
`sync.WaitGroup` (218) and the k8s-local `wait.Group` (4) do that job instead. Sizing note: this
figure was initially recorded as **0** because a `| head -60` truncated the frequency table below
the count-2 rows. It was caught by re-running the pattern on its own. Any figure in this document
that came from a truncated listing would fail the same way; the ones that did not come from a
dedicated single-pattern command are §5.1's `sync.*` and `atomic.*` rows, all of which sit far above
any cut.

### 5.2 `context.Context` propagation

| Measure | Count |
|---|---:|
| parameter positions `ctx context.Context` | 12521 |
| parameter positions `<any ident> context.Context` | 13070 |
| total `^func` declarations | 83029 |
| `context.TODO()` sites | 699 |
| `context.Background()` sites | 668 |
| `<-chan struct{}` parameters (the pre-context shutdown idiom) | 340 |

**≈15.7% of function declarations take a `context.Context`** (13070/83029). That denominator is
inflated by generated code (deepcopy, clientset accessors, conversion functions) which by construction
takes no context, so the *hand-written* proportion is higher.

The number that actually matters for the port is **1367 `context.TODO()` + `context.Background()`
sites**. Each is a point where context propagation is *broken* — a new root is minted instead of a
parent being threaded. In Rust these become either a fresh `CancellationToken` (losing the parent
link, faithfully reproducing the Go bug) or a compile-time hole the port engine must surface. This is
a **propagation-depth measurement by its complement**: rather than measuring how deep ctx threads
(which needs a call graph), measure how often it *stops*. 1367 stops against 13070 threaded parameters
is roughly one break per ten hops.

The 340 `<-chan struct{}` parameters are the legacy `stopCh` convention, mid-migration to `ctx`. Both
conventions are live in the corpus simultaneously, so the port needs rules for both **and** for the
bridging helper `wait.ContextForChannel` (55 sites).

### 5.3 Channel declarations by direction

| Direction | Count | Command |
|---|---:|---|
| send-only `chan<-` | 126 | `rg -o 'chan<-'` |
| receive-only `<-chan` | 453 | `rg -o '<-chan\b'` |
| bidirectional `chan T` (all positions) | 658 | `rg --pcre2 -o '(?<!<-)\bchan\s+[A-Za-z_*\[(]'` |
| — of which inside `make(` | 400 | `rg -o 'make\(chan\b\|make\(<-chan\b\|make\(chan<-'` |
| — bidirectional in declaration positions (not `make`) | 260 | `rg --pcre2 -o '(?<!<-)(?<!make\()\bchan\s+[A-Za-z_*\[(]'` |
| `make(chan struct{})` (pure signal channels) | 179 | |
| buffered `make(chan T, N)` | 161 | |
| `close(ch)` calls | 265 | |

Read the split as: **400 channels are created; 579 direction-restricted type positions
(126 send-only + 453 receive-only) carry them across API boundaries.** Direction annotations
outnumber creations, which is the good case for a port — the intended direction is usually declared,
so a Rust `Sender<T>`/`Receiver<T>` split is derivable from the source rather than inferred.

**179 of 400 created channels (44.8%) are `chan struct{}`** — pure signals, not data. Those do not
map to `mpsc` at all; they map to `CancellationToken`, `Notify`, or `oneshot`. Combined with the 265
`close(ch)` calls (Go's broadcast-close idiom, which `mpsc` has no equivalent for), **close to half of
all channel traffic in this corpus is signalling, not data transport**, and needs a different Rust
primitive entirely.

**239 of 400 created channels (59.8%) are unbuffered** (400 − 161 buffered). Unbuffered Go channels
are rendezvous points; `tokio::mpsc::channel(0)` does not exist (capacity must be ≥1), so unbuffered
rendezvous semantics need `oneshot` per message or an explicit ack channel. That is a distinct rule
and it applies to the majority of channels.

---

## 6. What this corpus needs: estimated rule count

Sizing derives from **shapes**, not occurrences. Ranges, with the derivation attached, because a
single number here would be false precision.

| Rule family | Rules | Derived from |
|---|---:|---|
| Goroutine launch shapes | **10–14** | 8 measured shapes (§1.5) + the N-identical-worker sub-shape (§1.8) + join/no-join and ctx/stopCh variants on S1/S3/S4 |
| `select` translation | **17–22** | 17 measured branch-set signatures (§2.4); +3–5 for arity variants above 2 branches and for nested selects |
| `select` idiom rewrites (not translations) | **3–5** | `select{}`-forever (11 sites), closed-check (5), blocking-send-with-cancel (25), non-blocking try-send/try-recv (135 `default` sites collapse to 2 rules) |
| Channel construction & lifecycle | **8–12** | bidirectional/send-only/recv-only × buffered/unbuffered × `struct{}`-signal, plus `close()` broadcast, plus range-over-channel |
| Sync primitives | **12–16** | Mutex, RWMutex, Once, WaitGroup, Cond, Map, Pool, and 5–6 `atomic.*` families (§5.1) |
| Context / cancellation | **6–9** | `ctx` threading, `WithCancel`/`WithTimeout`/`WithDeadline`/`WithValue`, `TODO`/`Background` root-minting, `stopCh`↔`ctx` bridging |
| **Mechanical subtotal** | **56–78** | |

Plus, and separately:

- **≈5 hand-ported runtime libraries, not rules.** `k8s.io/apimachinery/pkg/util/wait`
  (39 symbols, 1182 call sites), `client-go/util/workqueue` (typed queues + 5 rate limiters,
  ≈330 mentions), `client-go/tools/cache` (reflector/informer/DeltaFIFO), `apimachinery/pkg/watch`
  (Broadcaster/StreamWatcher/FakeWatcher), `apiserver/pkg/storage/cacher`. **These five carry a
  disproportionate share of the concurrency surface** — `wait.*` alone accounts for 27% of named
  goroutine launches and **65% of the S1 shape** (§1.4's 108 `wait.*` named launches out of §1.5's
  165 S1 sites = 65.5%; the classifier ladder quoted in §8.3 routes every `wait.*` launch to S1).
  Porting them by hand, once, correctly, is worth more
  than any rule in the table above, and it *shrinks the rule corpus* rather than adding to it.

- **A residue of 100–160 sites requiring human restructure**, not translation:
  74 lock-across-blocking sites (§4.2, a floor — this total ALREADY CONTAINS the 8
  `WaitGroup.Wait()`-under-lock sites broken out beneath §4.2's table, so they are not a separate
  addend), 25 blocking-send selects (§3.2a, enumerated in §9), 5 closed-check idioms, and the
  sampled estimate's slack (§3.3). These are enumerable and mostly already enumerated, which is the point: they can be
  scheduled as a finite, named work list rather than discovered during a multi-year port.

**Read the whole table as: the concurrency surface is roughly 60–80 mechanical rules, five hand-ported
libraries, and ~150 named exceptions.** It is not open-ended. The single largest uncertainty in that
statement is not any of the counts — it is §1.6 (23% of launch sites unclassified) and §4.1 (the
async-colouring fixpoint), both of which are closed by the same follow-up: a type-checked Go call graph.

---

## 7. Things this census could not determine

Stated plainly, because an honest "could not determine" is the point:

1. **How many `MutexGuard`s cross an `.await`.** Needs async-colouring over a type-resolved call
   graph. §4 gives a measured floor of 74 and names its error direction (under-count). It does not
   give the answer.
2. **The shape of 173 of 745 goroutine launches (23.2%).** The shape is in the callee body. Needs
   `go/packages` + `callgraph`. §1.6.
3. **Actual `context.Context` propagation *depth*** — the mean/max number of hops a context is
   threaded through. §5.2 measures the complement (1367 propagation breaks) because depth needs a call
   graph. The complement is arguably the more actionable metric, but it is not the metric asked for
   and is not presented as if it were.
4. **Whether any select branch body mutates shared state in a way that a mis-designed port would
   half-apply.** §3.1 argues from Go's grammar that branch bodies cannot be interrupted, so the
   hazard is a property of the port's design rather than of the source. That is an argument, not a
   measurement, and it is flagged as such.
5. **Goroutine *lifetime* and leak surface** — how many launched goroutines have no shutdown path.
   Not attempted. It needs escape/lifetime analysis and would be a separate census.

---

## 8. Instruments (verbatim, for re-derivation)

The five `awk` programs are reproduced here rather than referenced, because the lane scratchpad is
ephemeral and every figure above depends on them. Each is run as
`rg --files -g '*.go' -g '!vendor/**' -g '!*_test.go' . | xargs awk -f <program>` from the corpus root,
except `case_shape.awk` and `sel_sig.awk` which consume `select_census.awk`'s output.

### 8.1 `select_census.awk`

```awk
function strip(l) {
  gsub(/`[^`]*`/, "``", l); gsub(/"([^"\\]|\\.)*"/, "\"\"", l);
  gsub(/'([^'\\]|\\.)*'/, "''", l); sub(/\/\/.*$/, "", l);
  gsub(/\/\*[^*]*\*\//, "", l); return l;
}
function braces(l,   i, c, d) {
  d = 0;
  for (i = 1; i <= length(l); i++) { c = substr(l, i, 1); if (c == "{") d++; else if (c == "}") d--; }
  return d;
}
FNR == 1 { depth = 0; sp = 0 }
{
  s = strip($0);
  if (sp > 0 && depth == seldepth[sp]) {
    if (s ~ /^[ \t]*case[ \t(]/ || s ~ /^[ \t]*case<-/) {
      nb[sp]++; printf "CASE\t%s\t%d\t%d\t%s\n", FILENAME, FNR, selline[sp], s;
    } else if (s ~ /^[ \t]*default[ \t]*:/) {
      nb[sp]++; hd[sp] = 1;
      printf "CASE\t%s\t%d\t%d\t%s\n", FILENAME, FNR, selline[sp], "default:";
    }
  }
  isSel = (s ~ /(^|[^[:alnum:]_.])select[ \t]*{/);
  d = braces(s); depth += d;
  if (isSel) { sp++; selline[sp] = FNR; nb[sp] = 0; hd[sp] = 0; seldepth[sp] = depth }
  while (sp > 0 && depth < seldepth[sp]) {
    printf "SEL\t%s\t%d\t%d\t%d\n", FILENAME, selline[sp], nb[sp], hd[sp]; sp--;
  }
}
```

### 8.2 `case_shape.awk`

```awk
BEGIN { FS = "\t" }
$1 != "CASE" { next }
{
  t = $0; sub(/^([^\t]*\t){4}/, "", t); gsub(/^[ \t]+/, "", t);
  if (t ~ /^default/) { print "SHAPE\tdefault\t-\t" $2 "\t" $3; next }
  sub(/^case[ \t]*/, "", t); sub(/:[ \t]*$/, "", t);
  if (t ~ /^<-/) { kind = "recv-discard"; subj = substr(t, 3) }
  else if (t ~ /:=[ \t]*<-/) { kind = "recv-assign"; subj = t; sub(/^.*:=[ \t]*<-[ \t]*/, "", subj) }
  else if (t ~ /=[ \t]*<-/)  { kind = "recv-assign"; subj = t; sub(/^.*=[ \t]*<-[ \t]*/, "", subj) }
  else if (t ~ /<-/)         { kind = "send"; subj = t; sub(/[ \t]*<-.*$/, "", subj) }
  else                       { kind = "other"; subj = t }
  if (kind == "recv-assign" && t ~ /,[ \t]*(ok|more|open)[a-zA-Z]*[ \t]*(:?)=/) kind = "recv-assign-ok";
  gsub(/^[ \t]+|[ \t]+$/, "", subj);
  print "SHAPE\t" kind "\t" subj "\t" $2 "\t" $3;
}
```

### 8.3 `go_shape.awk` (feature extraction) and `go_taxonomy.awk` (collapse)

`go_shape.awk` brace-matches each `go func` closure body and emits a feature vector over
`{inloop, wgadd, wgdone, bodyloop, bodyselect, ticker, bodysend, bodyrecv, ctxdone, crash, errch}`;
named launches emit the callee instead. `go_taxonomy.awk` collapses vectors to shapes with this
priority ladder (the two `// fixed` lines are the §1.7 corrections):

```awk
if (kind == "named") {
  if (callee ~ /^wait\.(Until|UntilWithContext|Forever|Jitter|Poll)/)        n = "S1-background-loop";
  else if (callee ~ /\.?[Rr]un$|\.?[Ss]erve$|\.?[Rr]unLoop$|RunReloadLoop$/) n = "S2-long-lived-service-task";
  else                                                                       n = "S7-fire-and-forget-named-call";
}
else if (has_loop && (has_sel || has_tick || has_ctx)) n = "S1-background-loop";
else if (has_tick)                                     n = "S1-background-loop";        // fixed, §1.7
else if (has_loop)                                     n = "S2-long-lived-service-task";
else if (has_inloop && (has_wg || has_send))           n = "S3-fanout-worker-pool";     // fixed, §1.7
else if (has_wg)                                       n = "S4-joined-parallel-subtask";
else if (has_send && has_recv)                         n = "S5-request-response-bridge";
else if (has_send)                                     n = "S6-producer-into-channel";
else if (has_recv)                                     n = "S8-consumer-drain";
else                                                   n = "S9-fire-and-forget-closure";
```

(S6 is empty after the §1.7 fix — every producer-into-channel launch also reads a channel or sits in
a loop — which is why §1.5 lists 8 shapes, not 9.)

### 8.4 `lock_block.awk`

Tracks, per top-level `func`, whether a `Lock()`/`RLock()` is unreleased or a `defer …Unlock()` is in
scope, resetting both when brace depth leaves the registering scope; excludes lines inside a `go func`
opened under the lock; flags lines matching channel-recv / channel-send / `select {` / `.Wait()` /
`time.Sleep(`. Full source in the lane scratchpad; the classification predicates are:

```awk
if (s ~ /(^|[^-[:alnum:]_])<-[ \t]*[[:alnum:]_(]/ && s !~ /<-chan/)                    cls = "chan-recv";
else if (s ~ /[[:alnum:]_)\]][ \t]*<-[ \t]/ && s !~ /chan<-/ && s !~ /<-chan/)         cls = "chan-send";
else if (s ~ /(^|[^[:alnum:]_])select[ \t]*{/)                                        cls = "select";
else if (s ~ /\.Wait\(\)/)                                                            cls = "wait";
else if (s ~ /time\.Sleep\(/)                                                         cls = "sleep";
```

### 8.5 `sel_sig.awk`

Joins `SEL` and `CASE` rows by `file:selectline` and reduces each select to a sorted set over
`{cancel, timer, recv, send, default}`. `cancel` keys on `*.Done()` / `*stopCh` / `done` / `stop` /
`quit` / `*.stopped`; `timer` on `time.After(` / `.C` / `.C()` / `After(` / `timeout`. Both are
name-based heuristics — see the caveat under §2.4.

---

## 9. Appendix: the 25 blocking-send selects

Enumerated in full because these are the highest-risk, smallest, most actionable set in the census.
Each is a `case ch <- v:` racing another branch with **no `default`** — a restructure, not a
translation, and (for the watch paths) a silent-data-loss risk if translated naively.

```
awk -F'\t' '$1=="SEL"{hd[$2":"$3]=$5}
            $1=="CASE"{k=$2":"$4; t=$0; sub(/^([^\t]*\t){4}/,"",t); gsub(/^[ \t]+/,"",t);
              if (t ~ /^case/ && t !~ /^case[ \t]*<-/ && t ~ /<-/ && t !~ /=[ \t]*<-/) snd[k]=$2":"$3"\t"t }
            END{ for (k in snd) if (!hd[k]) print snd[k] }' select-census.txt | sort
```

| # | Site | Branch |
|---:|---|---|
| 1 | `pkg/controller/tainteviction/taint_eviction.go:322` | `case tc.nodeUpdateChannels[hash] <- nodeUpdate:` |
| 2 | `pkg/controller/tainteviction/taint_eviction.go:343` | `case tc.podUpdateChannels[hash] <- podUpdate:` |
| 3 | `staging/src/k8s.io/apimachinery/pkg/watch/mux.go:292` | `case w.result <- event:` |
| 4 | `staging/src/k8s.io/apimachinery/pkg/watch/streamwatcher.go:127` | `case sw.result <- Event{` |
| 5 | `staging/src/k8s.io/apimachinery/pkg/watch/streamwatcher.go:139` | `case sw.result <- Event{` |
| 6 | `staging/src/k8s.io/apiserver/pkg/registry/generic/registry/decorated_watcher.go:67` | `case d.resultCh <- send:` |
| 7 | `staging/src/k8s.io/apiserver/pkg/registry/generic/registry/store.go:1337` | `case toProcess <- newItems[i]:` |
| 8 | `staging/src/k8s.io/apiserver/pkg/storage/cacher/cache_watcher.go:214` | `case c.input <- event:` |
| 9 | `staging/src/k8s.io/apiserver/pkg/storage/cacher/cache_watcher.go:430` | `case c.result <- *watchEvent:` |
| 10 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/watcher.go:518` | `case p.processingQueue <- processingResponse:` |
| 11 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/watcher.go:526` | `case response <- &processingResult{…}:` |
| 12 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/watcher.go:668` | `case wc.resultChan <- *errResult:` |
| 13 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/watcher.go:684` | `case wc.resultChan <- *event:` |
| 14 | `staging/src/k8s.io/apiserver/pkg/storage/etcd3/watcher.go:696` | `case wc.incomingEventChan <- e:` |
| 15 | `staging/src/k8s.io/apiserver/pkg/util/proxy/websocket.go:105` | `case channel <- size:` |
| 16 | `staging/src/k8s.io/client-go/tools/cache/shared_informer.go:1305` | `case nextCh <- notification:` |
| 17 | `staging/src/k8s.io/client-go/tools/pager/pager.go:231` | `case chunkC <- chunk:` |
| 18 | `staging/src/k8s.io/client-go/tools/watch/informerwatcher.go:77` | `case e.out <- event:` |
| 19 | `staging/src/k8s.io/client-go/tools/watch/retrywatcher.go:103` | `case rw.resultChan <- event:` |
| 20 | `staging/src/k8s.io/client-go/util/certificate/certificate_manager.go:485` | `case templateChanged <- struct{}{}:` |
| 21 | `staging/src/k8s.io/client-go/util/workqueue/delaying_queue.go:266` | `case q.waitingForAddCh <- &waitFor[T]{…}:` |
| 22 | `staging/src/k8s.io/cri-streaming/pkg/streaming/remotecommand/httpstream.go:419` | `case channel <- size:` |
| 23 | `staging/src/k8s.io/dynamic-resource-allocation/client/generic.go:361` | `case w.resultChan <- e:` |
| 24 | `staging/src/k8s.io/dynamic-resource-allocation/resourceslice/watcher.go:67` | `case w.result <- event:` |
| 25 | `test/integration/scheduler_perf/executor.go:589` | `case scheduledPods <- newPod:` |

24 of 25 are in production code (`pkg/` or `staging/`); **16 of 25 are watch-event delivery**
(rows 3–6, 8–14, 16, 18, 19, 23, 24). Row 21 (`workqueue/delaying_queue.go`) is work-item delivery,
which fails the same way.
