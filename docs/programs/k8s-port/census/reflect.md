---
doc_class: Program-Census-Record
doc_status: published
census_lane: reflect
upstream_pin: 756939600b9a7180fc2df6550a4585b638875e67
measured_at: 2026-08-08
authority_tier: 3
---

# Go→Rust rule-corpus census: the reflection surface

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-08) |
|---|---|---|
| Repository baseline | `origin/dev` @ `5e452bd70449b50cc66e63ffb9253adfcd7fc96e` | Lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Verified before counting; matches `specs/k8s-port/upstream-pin.json`. Apache-2.0. |
| Engine | `build/port-engine/*`, v0 — unbuilt | Not in force. This census is an input to sizing, not engine output. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored | Not in force. No rule is authored or implied by this record. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | `find`/`grep`/`comm` set arithmetic over generated-marker headers and `reflect` imports; no Go parser | Measurement instrument only; not an admitted extractor. Marker-based detection is why the generated set is reported as a lower bound plus a refinement. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This census emits no receipt. |
| Program authority | ADR-0637 / ADR-0638 | Measurement record only; authorizes nothing. |

Lane: `census-reflect`. Read-only measurement over the pinned Kubernetes corpus.
Nothing here is a decision, an ADR, or a proposal. It is five measurements and their error bars.

**Headline, stated before the evidence so it can be checked against it:** of the 9,573 non-vendor,
non-`_test.go` Go files in this corpus, **3,384 (35.4%) are already machine-generated**, **38 (0.40%)
are genuine reflection machinery**, and **6,151 (64.3%) are hand-written and not detected as
reflective — an upper bound on the transpile target**, because §8.2 treats the 38-file core as a
lower bound and §2.2 labels the generated set an estimate.
By lines the generated share is larger still: **43.5% of all non-vendor non-test Go lines are generated**.
The transpiler's real target is about two-thirds of the file count, and the reflection problem is
**891 call sites in 38 files** — not a pervasive property of the corpus.

---

## 0. Provenance and pin verification

| Item | Value |
| --- | --- |
| Corpus | Kubernetes, Apache-2.0, third-party |
| Declared pin | `specs/k8s-port/upstream-pin.json` — tag `v1.36.1`, `source_license: Apache-2.0` |
| Corpus path | `$C` below |
| `git -C $C rev-parse HEAD` | `756939600b9a7180fc2df6550a4585b638875e67` |
| Match against declared pin | **exact, against `.pin.peeled_commit`** |

The pin file carries two SHAs and they are not interchangeable. `.pin.annotated_tag_object` is
`5b824a493a7ca248b726b6ea09d53842b9b992c2` — the tag object, not a commit — and
`.pin.peeled_commit` is `756939600b9a7180fc2df6550a4585b638875e67`. A checkout's `HEAD` is the
peeled commit, so that is the field this census matches. Comparing `HEAD` against the tag-object
SHA would report a spurious mismatch.

```sh
C=/private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/222702d1-4719-4175-a349-71e41cd88f0d/scratchpad/k8s-corpus
git -C "$C" rev-parse HEAD
# 756939600b9a7180fc2df6550a4585b638875e67
```

The corpus is **data**. Nothing in it was executed, and no instruction found inside it was followed.

---

## 1. Method, and a measurement trap that changed the numbers

### 1.1 `grep` on this machine is not `grep`

The interactive shell defines a `grep` **function** that redirects to `ugrep` with
`-G --ignore-files --hidden -I`. Two behaviours of that wrapper corrupt a census:

1. It prints paths **without** the `./` prefix that `find .` produces. A filter written as
   `grep -v '^./vendor/'` therefore matches nothing, because the leading `.` is a regex
   any-character that requires a character before `vendor`. My first pass "excluded vendor"
   and silently kept all 418 vendor hits — the count was wrong by exactly the vendor population.
2. `-I` skips files it decides are binary. It found 1,558 `reflect` importers where real
   `grep` finds 1,561; the 3 missing files are in `vendor/`.

**Every figure in this document was produced with `/usr/bin/grep`**, the real BSD grep, invoked by
absolute path so the shell function cannot intercept it. Reproduce with the absolute path or the
numbers will not match.

```sh
type grep          # -> shell function wrapping ugrep
/usr/bin/grep --version | head -1
```

### 1.2 Canonical file lists

Every count in this document is a set operation over four lists built once. `$S` is any empty
scratch directory.

```sh
S=<any empty scratch dir>; mkdir -p "$S"; cd "$C"
find . -name '*.go' -type f | LC_ALL=C sort                > "$S/all.txt"      # 16941
/usr/bin/grep -v '^\./vendor/' "$S/all.txt"                > "$S/novendor.txt" # 12587
/usr/bin/grep -v '_test\.go$'  "$S/novendor.txt"           > "$S/nvnt.txt"     #  9573
```

Those three totals reproduce the shape given to this lane exactly (16941 / 12587 / 9573), which is
the first evidence that the tree is the intended one.

### 1.3 Denominators — stated once, used throughout

| Symbol | Definition | Count |
| --- | --- | ---: |
| **D1** | all `.go` files | 16,941 |
| **D2** | excluding `vendor/` | 12,587 |
| **D3** | excluding `vendor/` and `*_test.go` | **9,573** |

**Unless a figure explicitly says otherwise, its denominator is D3 = 9,573.**

**D3 is not "production code".** It excludes files named `*_test.go` but retains the entire `./test/`
tree — e2e and integration harness code that happens not to use the `_test.go` suffix:

```sh
/usr/bin/grep -c '^\./test/' "$S/nvnt.txt"      # 805
/usr/bin/grep -c '^\./staging/' "$S/nvnt.txt"   # 5986
/usr/bin/grep -c '^\./pkg/' "$S/nvnt.txt"       # 2272
```

So D3 carries 805 files of test infrastructure. Where that materially changes a conclusion I give
the figure both ways. It is also why D3 counts, not D2 counts, are the honest basis for sizing a
port: `_test.go` files are regenerated by writing Rust tests, not transpiled.

### 1.4 Syntactic vs semantic

Everything below is **syntactic** — it reads text, not types. There is no Go type-checker in this
measurement. Section 8 states exactly which questions that leaves unanswered and what it would cost
to answer them.

One exception worth naming: import counts here are **exact, not approximate**, because Go's import
syntax is closed and gofmt-normalised. Section 3.1 shows two independent parsers agreeing to the file.

---

## 2. Question 5 first: how much of this corpus is already generated

This is answered first because it resizes every other number.

### 2.1 The canonical marker

Go defines the generated-file marker at `go help generate`: a line matching
`^// Code generated .* DO NOT EDIT\.$` appearing **before** the package clause. Parsed exactly:

```sh
cd "$C"
find . -name '*.go' -type f -print0 | xargs -0 awk '
  FNR==1 { done=0 }
  done { next }
  /^package / { done=1; next }
  /^\/\/ Code generated .* DO NOT EDIT\.$/ { print FILENAME; done=1 }
' | LC_ALL=C sort -u > "$S/gen-awk.txt"
wc -l < "$S/gen-awk.txt"                                             # 3776  (D1)
LC_ALL=C comm -12 "$S/gen-awk.txt" "$S/novendor.txt" > "$S/gen-nv.txt"
wc -l < "$S/gen-nv.txt"                                              # 3360  (D2)
LC_ALL=C comm -12 "$S/gen-awk.txt" "$S/nvnt.txt"     > "$S/gen-nvnt.txt"
wc -l < "$S/gen-nvnt.txt"                                            # 3326  (D3)
```

`$S/gen-nvnt.txt` — the 3,326 marker-bearing D3 files — is referenced by every later section.

The awk state machine is used rather than a bare `grep` because the marker must precede the package
clause; a `grep` would also match the phrase quoted inside a generator's own source. The `done=1`
on `^package ` enforces the positional rule.

### 2.2 Generated files that lack the marker

The marker is a **lower bound**. Searching for any generation phrase and subtracting the canonical
set exposes the gap:

```sh
/usr/bin/grep -rlE 'DO NOT EDIT|[Aa]uto-?generated by|generated by' --include='*.go' . \
  | LC_ALL=C sort -u > "$S/loose-gen-all.txt"
LC_ALL=C comm -12 "$S/loose-gen-all.txt" "$S/nvnt.txt" > "$S/loose-gen-nvnt.txt"
wc -l < "$S/loose-gen-nvnt.txt"                                              # 3427
LC_ALL=C comm -23 "$S/loose-gen-nvnt.txt" "$S/gen-nvnt.txt" | wc -l          #  101
LC_ALL=C comm -23 "$S/loose-gen-nvnt.txt" "$S/gen-nvnt.txt" | sed 's|.*/||' \
  | sort | uniq -c | sort -rn | head -3
#   58 types_swagger_doc_generated.go
#   11 types.go
#    3 args.go
```

Of those 101, **58 are `types_swagger_doc_generated.go`** — genuinely generated (by
`genswaggertypedocs` via `hack/update-codegen.sh`), emitting per-field doc maps, but carrying a
prose header instead of the canonical marker. I inspected one and confirmed the header is
descriptive prose, not the marker. The remaining ~43 are hand-written files whose comments merely
*mention* generated code; I confirmed one by name, `test/e2e/apimachinery/generated_clientset.go`,
which is a hand-written e2e test *of* the generated clientset — a filename false positive, correctly
excluded.

**Refined generated set (D3): 3,326 + 58 = 3,384 files.** This is `$S/gen-nvnt-refined.txt`, used by
every later section:

```sh
/usr/bin/grep 'types_swagger_doc_generated\.go$' "$S/nvnt.txt" | LC_ALL=C sort -u > "$S/swagger.txt"
wc -l < "$S/swagger.txt"                                                        #   58
LC_ALL=C sort -u "$S/gen-nvnt.txt" "$S/swagger.txt" > "$S/gen-nvnt-refined.txt"
wc -l < "$S/gen-nvnt-refined.txt"                                               # 3384
```

| Basis | What it counts | D3 count | % of D3 |
| --- | --- | ---: | ---: |
| Canonical marker only | files bearing the canonical marker | 3,326 | 34.75% |
| **Best estimate** | canonical marker ∪ swagger-doc | **3,384** | **35.35%** |
| Marker-bearing candidate set | any generation phrase (≈43 known false positives) | 3,427 | 35.80% |

**These are marker-derived proxies, not bounds on the generated population.** An earlier draft
labelled the third row an upper bound and concluded "the bound is tight … this number is
reliable"; both are withdrawn. "Any generation phrase" can only ever bound files that *contain* a
generation phrase — a generated file carrying no such phrase matches nothing and is invisible to
every row of this table. **The demonstration is in this very section:** the canonical marker missed
58 `types_swagger_doc_generated.go` files, and they were recovered by *name*, not by any phrase
search. A method already caught missing 58 unmarked generated files cannot then be declared a tight
bound on the generated population.

The closure is bounded and does not need prose grep at all: the generated-file population is
enumerable from the corpus's own generator manifest — the generator list driven by
`hack/update-codegen.sh`, cross-checked against `make verify-generated`, which by construction
names every file the build regenerates whether or not it carries a marker. Until that is run,
**every generated/hand-written split downstream of this section is marker-derived** — the 6,247
hand-written figure in §2.x, the 251 hand-written `reflect` importers of §5, and the 113-of-129
`unsafe.Pointer` split in §7 all inherit the label, and it is stated here once rather than repeated
at each derivation.

### 2.3 By lines, not files — generated code is bigger than average

```sh
tr '\n' '\0' < "$S/nvnt.txt"             | xargs -0 cat | wc -l   # 2137190
tr '\n' '\0' < "$S/gen-nvnt-refined.txt" | xargs -0 cat | wc -l   #  929168
```

**929,168 of 2,137,190 lines (43.48%) of the non-vendor non-test corpus are generated.** Generated
files average 275 lines against a corpus average of 223. Any sizing done on file counts
*understates* the generated share by 8 percentage points.

### 2.4 Which generators, and what that implies

```sh
tr '\n' '\0' < "$S/gen-nvnt.txt" | xargs -0 \
  /usr/bin/grep -hoE '^// Code generated by [^.]*\.' | sort | uniq -c | sort -rn
```

| Generator | Files | Share of 3,326 | What it consumes |
| --- | ---: | ---: | --- |
| `applyconfiguration-gen` | 968 | 29.1% | API type schema |
| `client-gen` | 804 | 24.2% | API type schema |
| `informer-gen` | 298 | 9.0% | API type schema |
| `deepcopy-gen` | 286 | 8.6% | API type schema |
| `lister-gen` | 233 | 7.0% | API type schema |
| `validation-gen` | 166 | 5.0% | API type schema + tags |
| `conversion-gen` | 152 | 4.6% | paired API type schemas |
| `defaulter-gen` | 113 | 3.4% | API type schema + tags |
| `openapi-gen` | 110 | 3.3% | API type schema |
| `protoc-gen-gogo` | 77 | 2.3% | `.proto` schema |
| `prerelease-lifecycle-gen` | 59 | 1.8% | API type schema + tags |
| `protoc-gen-go` | 19 | 0.6% | `.proto` schema |
| `protoc-gen-go-grpc` | 18 | 0.5% | `.proto` schema |
| `register-gen` | 13 | 0.4% | API type schema |
| `mockery` | 10 | 0.3% | Go interfaces |
| **Total** | **3,326** | 100% | — |

The 15 counts sum to exactly 3,326, which confirms every marker-bearing file was attributed and none
double-counted.

**Every one of these 15 is schema-driven.** Fourteen consume the API type definitions or `.proto`
files; the fifteenth (`mockery`) consumes Go interface declarations. Not one consumes hand-written
algorithmic logic. In Rust each is a derive macro or a codegen pass over the same schema — they are
**re-emitted, never transpiled**. The rule corpus needs zero rules for 35.4% of the files and 43.5%
of the lines.

The concentration is also favourable: the top 5 generators produce 2,589 of 3,326 files (77.8%), and
all five are clientset/informer/lister/applyconfiguration/deepcopy — i.e. **the client-side API
surface, which is exactly the part a Rust implementation would generate from the same OpenAPI schema
anyway.**

---

## 3. Question 1: how much of the corpus imports `reflect`

### 3.1 Count, by two independent parsers

Method A — the import block parsed as a block:

```sh
cd "$C"
find . -name '*.go' -type f -print0 | xargs -0 awk '
  FNR==1 { inimp=0 }
  /^import \(/ { inimp=1; next }
  inimp && /^\)/ { inimp=0; next }
  inimp && /"reflect"$/ { print FILENAME; inimp=0; next }
  /^import ([A-Za-z0-9_]+ )?"reflect"$/ { print FILENAME }
' | LC_ALL=C sort -u > "$S/refl-awk.txt"
wc -l < "$S/refl-awk.txt"     # 1561
```

Method B — a line-anchored regex, no block tracking:

```sh
/usr/bin/grep -rlE '^[[:space:]]*(_[[:space:]]+|[A-Za-z0-9_]+[[:space:]]+)?"reflect"$' \
  --include='*.go' . | LC_ALL=C sort > "$S/refl-all.txt"
wc -l < "$S/refl-all.txt"     # 1561
LC_ALL=C comm -3 "$S/refl-awk.txt" "$S/refl-all.txt"    # (no output — the sets are identical)
```

The two methods **agree file-for-file**. `comm -3` prints nothing.

| Denominator | Importers | Fraction |
| --- | ---: | ---: |
| D1 = 16,941 | 1,561 | 9.21% |
| D2 = 12,587 | 1,140 | 9.06% |
| **D3 = 9,573** | **355** | **3.71%** |

The D2→D3 collapse is the first real signal: **785 of the 1,140 non-vendor importers are `_test.go`
files (68.9%)**. Most Go code that touches `reflect` in this corpus is test code, and test code is
rewritten, not transpiled.

### 3.2 Is this count exact?

**Exact, with three named caveats** — unusually strong for a regex-based census, because Go's import
syntax is closed:

- *False positives:* a line consisting of exactly `"reflect"` outside an import block. Method A
  eliminates these structurally (it requires the line to be inside `import ( … )`). The two methods
  agreeing at 1,561 shows Method B had none either — gofmt forces a trailing comma on multi-line
  composite literal elements, so a bare `"reflect"` line cannot occur in a slice or map literal.
- *False negatives (import side):* an aliased import `r "reflect"` is caught (both patterns allow an
  alias); a dot-import `. "reflect"` is **not** caught by either. Searched explicitly — there are none:

  ```sh
  /usr/bin/grep -rlE '^[[:space:]]*\.[[:space:]]+"reflect"$' --include='*.go' . | wc -l   # 0
  ```
- *Build tags:* files excluded by `//go:build` constraints for any given GOOS/GOARCH still count.
  This is deliberate — a port must handle every platform variant in the tree, not one build.

### 3.3 The reverse check, and what it caught

Files that *use* a `reflect.` selector should be a subset of files that import it:

```sh
/usr/bin/grep -rlE '\breflect\.[A-Za-z]' --include='*.go' . | LC_ALL=C sort > "$S/uses-all.txt"
wc -l < "$S/uses-all.txt"                                            # 1588
LC_ALL=C comm -13 "$S/uses-all.txt" "$S/refl-awk.txt" | wc -l        #    0  (imports with no use)
LC_ALL=C comm -23 "$S/uses-all.txt" "$S/refl-awk.txt" > "$S/uses-not-import.txt"
wc -l < "$S/uses-not-import.txt"                                     #   27  (use with no import)
```

Zero files import `reflect` without using it — no blank or unused imports to explain.

The 27 files that appear to use `reflect.` without importing it were inspected individually.
**None of the 27 calls stdlib `reflect`** — the import-based count loses nothing. Sorting them by
whether the match is even Go code:

```sh
# which of the 27 have a reflect. match outside a line-comment and outside a bare map-key string
tr '\n' '\0' < "$S/uses-not-import.txt" | xargs -0 /usr/bin/grep -HnE '\breflect\.[A-Za-z]' \
  | /usr/bin/grep -vE ':[0-9]+:[[:space:]]*(//|\*|/\*)' | /usr/bin/grep -vE '"reflect":' \
  | cut -d: -f1 | sort -u
# 5 files
```

**26 of the 27 are pure textual matches**, in three kinds — none is a call:

- **Line comments** (the majority): `// We don't use reflect.DeepEqual here because …`.
- **Block-comment and doc text** that the line-comment filter above does not catch, e.g.
  `vendor/github.com/davecgh/go-spew/spew/doc.go:113`, which sits inside the package doc block.
- **String literals and compiler directives**: `//go:linkname … reflect.mapiterinit` in
  `vendor/github.com/modern-go/reflect2`; `strings.Contains(fn, "reflect.Value")` in ginkgo;
  `"reflect": {` as a map key in `x/tools/internal/stdlib/manifest.go`; and
  `informerType := reflect.TypeOf(obj)` at
  `staging/src/k8s.io/code-generator/cmd/informer-gen/generators/factory.go:304`, which is **template
  text inside the generator's output string**, not code the generator runs.

**Exactly 1 of the 27 is real executing code**, and it resolves to something other than the stdlib:
`staging/src/k8s.io/apimachinery/pkg/conversion/deep_equal.go` uses `reflect.Equalities`, where
`reflect` is the import alias for `k8s.io/apimachinery/third_party/forked/golang/reflect` — a
**forked copy of reflect logic**. Confirmed:
`/usr/bin/grep -n 'forked/golang/reflect' <that file>` → line 20.

The `factory.go` case is a structural warning for the port engine: **a naive scan will attribute
reflection to generator source files that merely contain it as string data.** Both cases are
correctly excluded from the import-based counts, which is why the import-based count is the one used
throughout.

Consequence: the usage regex `\breflect\.[A-Za-z]` has a measured false-positive rate that makes it
unsuitable as a primary instrument. It is used below only *within* files already known to import
reflect, where the residual error is confined to comments.

---

## 4. Question 2: is reflection concentrated, or diffuse?

The expectation put to this lane was apimachinery, `runtime.Scheme`, conversion, and the dynamic
client. **The answer depends entirely on the granularity, and the naive answer is the wrong one.**

### 4.1 At file granularity it is diffuse — this refutes the naive form of the hypothesis

```sh
LC_ALL=C comm -12 "$S/refl-awk.txt" "$S/nvnt.txt" > "$S/refl-nvnt.txt"
wc -l < "$S/refl-nvnt.txt"                                       # 355
sed 's|/[^/]*$||' "$S/refl-nvnt.txt" | sort -u | wc -l           # 286
sed 's|/[^/]*$||' "$S/refl-nvnt.txt" | sort | uniq -c | sort -rn | head -5
#   9 ./staging/src/k8s.io/apimachinery/pkg/runtime
#   7 ./staging/src/k8s.io/cli-runtime/pkg/printers
#   7 ./staging/src/k8s.io/apiserver/pkg/cel
#   4 ./test/e2e/apimachinery
#   4 ./staging/src/k8s.io/code-generator/cmd/go-to-protobuf/protobuf
```

**355 files spread over 286 distinct directories**, the densest holding 9. That is a long tail, not a
cluster. If "imports reflect" were the boundary, the boundary would be worthless.

### 4.2 But 104 of the 355 are generated, and 213 more barely touch reflect

Split the 355 against the generated set:

```sh
LC_ALL=C comm -12 "$S/refl-nvnt.txt" "$S/gen-nvnt-refined.txt" > "$S/refl-gen.txt"
LC_ALL=C comm -23 "$S/refl-nvnt.txt" "$S/gen-nvnt-refined.txt" > "$S/refl-hand.txt"
wc -l < "$S/refl-gen.txt"     # 104 generated
wc -l < "$S/refl-hand.txt"    # 251 hand-written
```

The 104 generated ones come from exactly three generators:

```sh
cd "$C"; tr '\n' '\0' < "$S/refl-gen.txt" | xargs -0 \
  /usr/bin/grep -hoE '^// Code generated by [^.]*\.' | sort | uniq -c | sort -rn
#   74 // Code generated by protoc-gen-gogo.
#   19 // Code generated by protoc-gen-go.
#   11 // Code generated by informer-gen.
```

74 + 19 + 11 = 104. Protobuf codegen accounts for 93 of them — gogo emits `reflect` in its generated
`String()` and equality helpers. **All 104 disappear the moment protobuf and informers are generated
for Rust instead of transpiled.** Note this is the same `informer-gen` whose *source* was flagged as
a template false positive in §3.3 — here it is the *output* that genuinely imports reflect, which is
the correct and opposite finding.

### 4.3 The real boundary: two orthogonal reflection idioms

"Uses reflect" conflates two different programs. I measured both.

**Axis A — generic container walking.** A file that switches on `reflect.Kind` across several kinds
is doing structural recursion over arbitrary values. Per-file symbol sets:

```sh
cd "$C"
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -HoE '\breflect\.[A-Za-z_][A-Za-z0-9_]*' \
  | sort -u | awk -F: '{split($2,a,"."); m[$1]=m[$1]" "a[2]} END{for(f in m) print f m[f]}' \
  | LC_ALL=C sort > "$S/file-syms.txt"
awk '{n=0; for(i=2;i<=NF;i++) if($i ~ /^(Slice|Map|Struct|Array|Pointer|Ptr|Interface|Chan|Func)$/) n++; print n}' \
  "$S/file-syms.txt" | sort -n | uniq -c
```

| Distinct `Kind` constants used | Files |
| ---: | ---: |
| 0 | 206 |
| 1 | 11 |
| 2 | 7 |
| 3 | 4 |
| 4 | 9 |
| 5 | 6 |
| 6 | 4 |
| 7 | 2 |
| 8 | 2 |

206 + 11 + 7 + 4 + 9 + 6 + 4 + 2 + 2 = 251. There is a clean cliff: **224 files (89.2%) use two or
fewer Kind constants; 27 files (10.8%) use three or more.** The threshold is not delicate — moving it
between 3 and 4 moves the count from 27 to 23.

```sh
awk '{n=0; for(i=2;i<=NF;i++) if($i ~ /^(Slice|Map|Struct|Array|Pointer|Ptr|Interface|Chan|Func)$/) n++; if(n>=3) print $1}' \
  "$S/file-syms.txt" > "$S/walkers.txt"
wc -l < "$S/walkers.txt"    # 27
```

**Axis B — type-keyed registries.** A `map[reflect.Type]…` is a runtime type registry: `runtime.Scheme`
is the archetype. This idiom uses almost no `Kind` constants, so Axis A misses it entirely — I found
this by checking whether `scheme.go` had landed in the 27, and it had not:

```sh
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -lE 'map\[reflect\.Type\]' \
  > "$S/typekeyed.txt"
wc -l < "$S/typekeyed.txt"    # 16
/usr/bin/grep -coE '\breflect\.[A-Za-z]+' staging/src/k8s.io/apimachinery/pkg/runtime/scheme.go   # 38
```

`runtime/scheme.go` has 38 reflect sites and 0 Kind-switch structure. **Had I reported only Axis A, I
would have omitted the single most important reflection site in Kubernetes.** This is recorded
because the same blind spot will recur in any port tooling that measures reflection by Kind switches.

**The union is the reflective core:**

```sh
LC_ALL=C sort -u "$S/walkers.txt" "$S/typekeyed.txt" > "$S/reflective-core.txt"
wc -l < "$S/reflective-core.txt"                                    # 38
LC_ALL=C comm -12 "$S/walkers.txt" "$S/typekeyed.txt" | wc -l       #  5 (in both)
```

27 + 16 − 5 = 38.

### 4.4 At *that* granularity the hypothesis is confirmed

```sh
sed -e 's|^\./staging/src/\(k8s.io/[^/]*\)/.*|STAGING \1|' -e 's|^\./\([^/]*\)/.*|TREE \1|' \
  "$S/reflective-core.txt" | sort | uniq -c | sort -rn
```

| Module | Core files | Share of 38 |
| --- | ---: | ---: |
| `k8s.io/apimachinery` | 13 | 34.2% |
| `k8s.io/client-go` | 6 | 15.8% |
| `k8s.io/kubectl` | 4 | 10.5% |
| `k8s.io/apiserver` | 4 | 10.5% |
| `k8s.io/code-generator` | 3 | 7.9% |
| `k8s.io/component-base` | 2 | 5.3% |
| `k8s.io/apiextensions-apiserver` | 2 | 5.3% |
| `./test/` | 2 | 5.3% |
| `k8s.io/cli-runtime` | 1 | 2.6% |
| `./pkg/` | 1 | 2.6% |

**apimachinery alone holds 34%; the four API-machinery modules (apimachinery, client-go, apiserver,
apiextensions-apiserver) hold 25 of 38 = 65.8%.**

**Verdict on Question 2:** *diffuse by import, concentrated by substance.* The expectation
(apimachinery, Scheme, conversion, dynamic/unstructured access) is **confirmed** — but only once
generated files and shallow uses are removed. Reported at the import level it looks diffuse and the
scope cut would have been missed. The boundary is clean, and it is worth more than the count, as
predicted.

Note the dynamic client specifically: it appears via `apimachinery/pkg/runtime/converter.go`
(typed ↔ `map[string]interface{}`) and `apis/meta/v1/unstructured/*`, which is the unstructured
machinery the dynamic client is built on. So that part of the expectation holds through the
unstructured converter rather than through a file named "dynamic".

---

## 5. Question 3: what the reflection is used *for*

### 5.1 Shape distribution — the rule corpus is sized by shapes, not sites

Across the 251 hand-written reflect importers there are **1,487 executing `reflect.X` sites, 49
distinct symbols, and 73 distinct per-file symbol-set shapes**. An earlier draft reported 1,511
sites from an unfiltered text match; that figure counted comment text as reflection, and it is
corrected here by re-running the extraction through the comment filter §3.3 already uses:

```sh
# -Hn so the line prefix exists, then §3.3's whole-line-comment filter, then extract symbols
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -HnE '\breflect\.[A-Za-z_][A-Za-z0-9_]*' \
  | /usr/bin/grep -vE ':[0-9]+:[[:space:]]*(//|\*|/\*)' > "$S/keep.txt"
wc -l < "$S/keep.txt"                                                          # 1183 source lines
cut -d: -f3- "$S/keep.txt" | /usr/bin/grep -oE '\breflect\.[A-Za-z_][A-Za-z0-9_]*' \
  | sort | uniq -c | sort -rn > "$S/occ-f.txt"
awk '{s+=$1} END{print s}' "$S/occ-f.txt"    # 1487   (unfiltered: 1511)
wc -l < "$S/occ-f.txt"                       #   49   (unfiltered:   49)
# per-file symbol sets from the kept lines -> "$S/fs-f.txt"
awk '{ $1=""; sub(/^ /,""); print }' "$S/fs-f.txt" | sort -u | wc -l   # 73   (unfiltered: 73)
awk '{ $1=""; sub(/^ /,""); print }' "$S/file-syms.txt" | sort | uniq -c | sort -rn | head -8
```

**Why the filter is needed, and how much it moves.** The original extraction was a raw text match
with no comment stripping, and §3.3 of this same document is the in-document demonstration that the
class is real: over files that do *not* import `reflect`, the identical kind of match found that
**exactly 1 of 27 was real executing code** — the other 26 were line comments (`// We don't use
reflect.DeepEqual here because …`), block-comment doc text, `//go:linkname` directives, string
literals (`strings.Contains(fn, "reflect.Value")`), a map key, and generator template text.
Importing `reflect` does not stop a file commenting about `reflect`, so the import filter cannot
remove this. The filter reused here is §3.3's own rather than a new one.

**Measured effect: 24 of 1,511 sites (1.6%) were whole-line comment text.** §3.3's 26-of-27 rate is
over non-importers and does not transfer — inside actual importers the contamination is small, as
the re-run confirms. Neither the 49 symbols nor the 73 shapes moves, and no file drops out of the
251, so the shape distribution below is unaffected. **1,487 remains an upper bound on executing
selector sites**: this filter removes whole-line comments only, not trailing comments, string
literals or `//go:linkname` directives, all three of which §3.3 found in the wild.

| Rank | Shape (the complete set of `reflect.*` symbols the file uses) | Files | Cum. % of 251 |
| ---: | --- | ---: | ---: |
| 1 | `DeepEqual` | 115 | 45.8% |
| 2 | `TypeOf` | 17 | 52.6% |
| 3 | `ValueOf` | 14 | 58.2% |
| 4 | `Type` | 10 | 62.2% |
| 5 | `Type TypeOf` | 9 | 65.7% |
| 6 | `StructTag` | 8 | 68.9% |
| 7 | `Indirect ValueOf` | 5 | 70.9% |
| 8 | `DeepEqual TypeOf` | 5 | 72.9% |

**The top shape covers 45.8%; the top 5 cover 65.7%; the top 8 cover 72.9%. The remaining 65 shapes
share 68 files** — a classic long tail where the head is trivial and the tail is the work.

### 5.2 The head of the distribution needs no reflection in Rust at all

The top 8 shapes are not "reflection" in any sense Rust would recognise. Each is a Go idiom that
exists only because Go lacks a language feature Rust has:

| Shape | Files | What it actually is | Note |
| --- | ---: | --- | --- |
| `DeepEqual` only | 115 | structural equality | NOT mechanically a derived `==`. Go's `DeepEqual` distinguishes a nil slice or map from an empty non-nil one, and has defined behaviour for function values and other shapes called out in section 8; a derived structural equality does not reproduce those. Whether each site can use plain equality depends on the types it compares, which is not measured here. |
| `TypeOf` only | 17 | dynamic type name in an error/log string | These names are OBSERVABLE — they appear in operator-facing diagnostics. Any rewrite must preserve the name a reader would see, which is a behavioural constraint, not a formatting detail. |
| `ValueOf` only | 14 | mostly the typed-nil-interface check | `Option<T>` — the bug class does not exist | vanishes |
| `Type` / `Type TypeOf` | 19 | a type used as a map key | a generic param or an enum discriminant | trivial |
| `StructTag` only | 8 | reading `json:`/`protobuf:` struct tags | `#[serde(rename=…)]` | trivial |
| `Indirect ValueOf` | 5 | dereference-through-pointer | auto-deref | vanishes |

Evidence for the `TypeOf`-only claim — all occurrences in those files are diagnostic:

```sh
# the 17 files whose ONLY reflect symbol is TypeOf, taken from the shape table above
awk 'NF==2 && $2=="TypeOf" {print $1}' "$S/file-syms.txt" > "$S/shape-typeof.txt"
wc -l < "$S/shape-typeof.txt"    # 17
cd "$C"; tr '\n' '\0' < "$S/shape-typeof.txt" | xargs -0 \
  /usr/bin/grep -hoE '.{0,60}reflect\.TypeOf.{0,30}'
```

produces lines of the form `fmt.Errorf("unable to decode %s into %v", gvk, reflect.TypeOf(into))`,
`klog.V(2).Infof("Refreshing cache for provider: %v", reflect.TypeOf(d.Provider).String())`,
`fmt.Errorf("unknown type: %v", reflect.TypeOf(typedVal))`. A mechanical proxy confirms the direction
but understates it, because Go wraps long `Errorf` calls across lines:

```sh
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -hB2 -E '\breflect\.TypeOf\b' \
  | /usr/bin/grep -cE '(fmt\.(Errorf|Sprintf|Sprint|Printf)|klog\.|Infof|Errorf|errors\.New|HandleError)'
# 29   -- LOWER BOUND on the 163 total TypeOf sites; a 2-line window misses wider call wrappings
```

I report 29 as a lower bound rather than inflating it, and rest the `TypeOf`-only claim on having
read all the extracted occurrences from those 17 files instead.

Evidence for the `ValueOf`-only claim — the dominant idiom is Go's typed-nil-interface wart:

```sh
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -hE 'reflect\.ValueOf' \
  | /usr/bin/grep -cE 'IsNil\(\)|IsZero\(\)'
# 10
```

producing `if client != nil && !reflect.ValueOf(client).IsNil() {`,
`if podsGetter == nil || reflect.ValueOf(podsGetter).IsNil() {`. This is the check for "a non-nil
interface holding a nil pointer". **In Rust the condition is unrepresentable**; these sites are
deleted, not translated.

Evidence for the `StructTag` claim — 6 of the 8 files are inside the code generators themselves:

```sh
tr '\n' '\0' < "$S/refl-hand.txt" | xargs -0 /usr/bin/grep -lE 'reflect\.StructTag' \
  | /usr/bin/grep -cE 'code-generator|gengo|go-to-protobuf'
# 6   (of 8 total)
```

with bodies like `tag := reflect.StructTag(m.Tags).Get("json")` — the generator parsing tag strings
it read from source at **build** time. That is already compile-time metaprogramming wearing a
runtime type's clothing, and it maps onto a Rust proc-macro directly.

### 5.3 The 38-file core, categorised

The core is small enough to classify **exhaustively rather than by sampling** — I read the exported
functions and the reflect sites of all 38. Categories are the ones requested, plus those the corpus
actually contains.

| Category | Files | Representative sites | Rust answer |
| --- | ---: | --- | --- |
| **Test / fuzz infrastructure** | 11 | `apitesting/fuzzer/valuefuzz.go`, `apitesting/roundtrip/construct.go`, `apitesting/naming/naming.go`, `deepcopy-gen/output_tests/*`, `apiextensions/fuzzer/fuzzer.go`, `test/utils/format/format.go`, `validation-gen/testscheme` | `proptest`/`arbitrary` derive — rewritten, not ported |
| **Type registration / GVK mapping** | 6 | `runtime/scheme.go`, `endpoints/installer.go`, `tools/cache/shared_informer.go`, `leaderelection/leasecandidate.go`, `healthz.go`, `printers/tablegenerator.go` | a registry keyed by `TypeId` or a generated GVK enum; the mapping is static and knowable at compile time |
| **JSONPath / template evaluation** | 4 | `client-go/util/jsonpath`, `cli-runtime/printers/jsonpath.go`, forked `template/exec.go`, `kubectl/cmd/get/sorter.go` | genuine dynamic evaluation over `serde_json::Value` — this is an interpreter, and it stays one |
| **Config tree navigation** | 4 | `clientcmd/merge.go`, `clientcmd/api/helpers.go`, `kubectl/cmd/config/set.go`, `navigation_step_parser.go` | derive-driven field-path access, or an explicit generated path enum |
| **Unstructured ↔ typed conversion** | 3 | `runtime/converter.go`, `serializer/cbor/raw.go`, `forked/golang/json/fields.go` | `serde` — this *is* what serde does |
| **CEL value bridging** | 3 | `apiserver/pkg/cel/value.go`, `cel/common/valuesreflect.go`, `apiextensions/schema/cel/validation.go` | a `TryFrom` bridge into the CEL value enum, generated per type |
| **Struct-tag-driven field logic** | 3 | `conversion/queryparams/convert.go`, `strategicpatch/meta.go`, `logs/datapol/datapol.go` | `serde` attributes plus a derive for the patch/datapolicy metadata |
| **Object / list accessors** | 2 | `api/meta/help.go` (`IsListType`, `EachListItem`, `ExtractList`), `api/meta/meta.go` (`Accessor`) | a trait — `ObjectMeta` access is an interface, not a reflection problem |
| **Deep equality (forked)** | 2 | `third_party/forked/golang/reflect/deep_equal.go`, `conversion/deep_equal.go` | `#[derive(PartialEq)]` with semantic overrides |
| **Total** | **38** | | |

11 + 6 + 4 + 4 + 3 + 3 + 3 + 2 + 2 = 38.

**Nine categories. The top three (test/fuzz 11, GVK registration 6, jsonpath/template 4) cover 21 of
38 = 55.3%.** Excluding test infrastructure, the **production reflective core is 27 files across
8 categories**.

Two categorisations were checked rather than assumed, because the path name would have misled:

- `serializer/cbor/raw.go` — I expected serialization glue; it is
  `map[reflect.Type]func(reflect.Value) error` transcode dispatch, i.e. simultaneously Axis A and
  Axis B. Placed under unstructured conversion.
- `logs/datapol/datapol.go` — path suggests logging; it walks structs collecting `datapolicy` struct
  tags to find sensitive fields. Placed under struct-tag-driven, not logging.

### 5.4 Sizing the core honestly

```sh
tr '\n' '\0' < "$S/reflective-core.txt" | xargs -0 cat | wc -l                                 # 20158
tr '\n' '\0' < "$S/reflective-core.txt" | xargs -0 /usr/bin/grep -HnE '\breflect\.[A-Za-z_][A-Za-z0-9_]*' \
  | /usr/bin/grep -vE ':[0-9]+:[[:space:]]*(//|\*|/\*)' | cut -d: -f3- \
  | /usr/bin/grep -oE '\breflect\.[A-Za-z_][A-Za-z0-9_]*' | wc -l                              # 891 (unfiltered 899)
```

**The 20,158-line figure is a bad metric and I am not using it as the answer.** File-level LOC
over-attributes badly: the largest core file, `kubectl/pkg/describe/describe.go`, is 5,428 lines and
contains **20** reflect sites — 27% of the core's lines for 2% of its reflection. It qualifies only
because it holds one `map[reflect.Type]` dispatch table.

The defensible size is the site count, filtered as in §5.1:

| Population | Files | `reflect.X` sites (filtered) | (unfiltered) |
| --- | ---: | ---: | ---: |
| Reflective core | 38 | **891** | 899 |
| Shallow reflect users | 213 | 596 | 612 |
| — of which `DeepEqual` | — | 226 | 226 |
| **Hand-written total** | **251** | **1,487** | 1,511 |

891 + 596 = 1,487, matching §5.1 independently. The unfiltered column is kept because the earlier
revision published it and because the reconciliation holds on both bases — which checks the
partition, not the filtering. Both columns are upper bounds on executing sites for the residual
reason §5.1 gives.

**The entire reflection problem in hand-written, non-vendor, non-`_test.go` Kubernetes is 891 call
sites in 38 files.** The other 596 sites in 213 files are equality checks, error strings and nil
tests that require no reflection facility in Rust whatsoever.

---

## 6. Question 4: the `unsafe` package

Measured with the same import-block parser, substituting `"unsafe"`:

```sh
cd "$C"
find . -name '*.go' -type f -print0 | xargs -0 awk '
  FNR==1 { inimp=0 }
  /^import \(/ { inimp=1; next }
  inimp && /^\)/ { inimp=0; next }
  inimp && /"unsafe"$/ { print FILENAME; inimp=0; next }
  /^import ([A-Za-z0-9_]+ )?"unsafe"$/ { print FILENAME }
' | LC_ALL=C sort -u > "$S/unsafe-all.txt"

wc -l < "$S/unsafe-all.txt"                                                    # 441 (D1)
LC_ALL=C comm -12 "$S/unsafe-all.txt" "$S/novendor.txt" > "$S/unsafe-nv.txt"   # 157 (D2)
LC_ALL=C comm -12 "$S/unsafe-all.txt" "$S/nvnt.txt"     > "$S/unsafe-nvnt.txt" # 154 (D3)
LC_ALL=C comm -12 "$S/unsafe-nvnt.txt" "$S/gen-nvnt-refined.txt" > "$S/unsafe-gen.txt"  # 132
LC_ALL=C comm -23 "$S/unsafe-nvnt.txt" "$S/gen-nvnt-refined.txt" > "$S/unsafe-hand.txt" #  22
```

| Population | Files | % of its denominator |
| --- | ---: | ---: |
| D1, all `.go` | 441 | 2.60% |
| D2, non-vendor | 157 | 1.25% |
| **D3, non-vendor non-test** | **154** | **1.61%** |
| — generated | 132 | 85.7% of the 154 |
| — **hand-written** | **22** | **0.23% of D3** |

**85.7% of `unsafe` in this corpus is generated**, from two generators only:

```sh
tr '\n' '\0' < "$S/unsafe-gen.txt" | xargs -0 /usr/bin/grep -hoE '^// Code generated by [^.]*\.' \
  | sort | uniq -c | sort -rn
#  113 // Code generated by conversion-gen.
#   19 // Code generated by protoc-gen-go.
```

`conversion-gen` emits the layout-punning conversion between structurally identical API types across
versions:

```go
// staging/src/k8s.io/apimachinery/pkg/apis/meta/v1beta1/conversion.go
out.Items = *(*[]v1.PartialObjectMetadata)(unsafe.Pointer(&in.Items))
```

That is a zero-copy reinterpretation of two types the generator knows to be layout-identical. In
Rust this is a generated `From` impl over identical field sets — **safe, and generated from the same
schema.** All 113 leave the `unsafe` surface with the generator.

**They leave it at a cost this record priced at zero, and should not have.** The Go form
reinterprets a slice header: `O(1)`, no allocation, and the two versions share backing storage. A
safe Rust `From` over `Vec<A>` → `Vec<B>` cannot do that — it copies element-wise, so the rewrite
is `O(n)` where the original is `O(1)`. `conversion-gen` emits exactly the API version-conversion
paths that every apiserver request crosses, so this is a real cost on a hot path, not a
micro-optimisation. The safety conclusion is unchanged and unconditional — a safe `From` always
compiles and is always memory-safe, and nothing in `conversion-gen`'s contract makes the aliasing
observable, since these are one-way conversions whose input is discarded. What is conditional is
the **performance**. The programme has to say which it intends: give the two API versions a single
Rust representation, so no conversion is emitted at all; or accept an `O(n)` copy per conversion.
That choice is not made here and is recorded as open in §8.

The 22 hand-written sites, by symbol:

```sh
tr '\n' '\0' < "$S/unsafe-hand.txt" | xargs -0 /usr/bin/grep -hoE '\bunsafe\.[A-Za-z]+' \
  | sort | uniq -c | sort -rn
#   69 unsafe.Pointer     9 unsafe.Sizeof     8 unsafe.SliceData
#    4 unsafe.StringData  4 unsafe.String     2 unsafe.Slice
```

They fall into three clusters, all of which shrink rather than grow the programme:

1. **Platform syscall interop — 11 files.** `pkg/kubelet/winstats/*`, `pkg/windows/service`,
   `dns_windows.go`, `boottime_util_darwin.go`, `boottime_util_freebsd.go`,
   `cmd/kubelet/app/init_windows.go`, `test/images/agnhost/dns/dns_windows.go`. These are Win32/BSD
   struct marshalling. Rust replaces them with `windows-sys`/`libc` bindings — **not transpiled, and
   `unsafe` in Rust too**. This is the one cluster that stays unsafe, and it is 11 files.
2. **Hand-written layout-punning conversions — 6 files.** `pkg/apis/resource/v1beta1/conversion.go`,
   `apiextensions/v1/conversion.go`, `meta/v1beta1/conversion.go`, `apidiscovery/v2/conversion.go`,
   `controller-manager/config/v1/conversion.go`, `dynamic-resource-allocation/api/v1beta1/conversion.go`.
   Same idiom as the generated 113, written by hand. Same Rust answer, and the same unpriced `O(n)`
   copy stated above: safe `From` impls.
3. **Zero-copy `[]byte` ↔ `string` — 5 files.** The cache and envelope files under `apiserver/pkg`:
   `return unsafe.String(unsafe.SliceData(b), len(b))`. An earlier draft called this
   `std::str::from_utf8` and is WITHDRAWN: **Go strings hold arbitrary bytes and `unsafe.String`
   performs no UTF-8 validation, whereas `from_utf8` REJECTS invalid UTF-8.** For cache and
   envelope payloads, which are not guaranteed text, that turns a successful zero-copy conversion
   into an error path. The byte-vs-text distinction is the whole content of these 5 sites, and it
   is why they are listed rather than mapped.

**Verdict:** `unsafe` is a 22-file hand-written surface (0.23% of D3), of which only the 11
platform-interop files remain unsafe in Rust. It is not a programme risk.

---

## 7. Deliverable: the three numbers

Denominator **D3 = 9,573** non-vendor, non-`_test.go` `.go` files; **2,137,190** lines.
The three sets are disjoint and exhaustive by construction: the reflective core is a subset of the
hand-written files, which is D3 minus the generated files.

| | Population | Files | % of D3 | Lines | % of lines |
| --- | --- | ---: | ---: | ---: | ---: |
| **(a)** | **Already generated** — regenerate from schema in Rust, never transpile | **3,384** | **35.35%** | 929,168 | **43.48%** |
| **(b)** | **Reflection-heavy core** — schema-codegen and interpreter territory | **38** | **0.40%** | (891 sites) | — |
| **(c)** | **Hand-written, NOT DETECTED as reflective by the two idioms of §4.3** — the transpile target, an upper bound | **6,151** | **64.25%** | ≈1,208,022 | ≈56.5% |

3,384 + 38 + 6,151 = 9,573.

For (c) the line figure is given as the hand-written remainder (2,137,190 − 929,168 = 1,208,022)
because the core's own line count is not meaningfully attributable (§5.4); subtracting it either way
moves the percentage by 0.9 points.

### Refinements that matter for sizing

- **(c) still contains 801 files of e2e/integration harness** under `./test/` that are not
  `_test.go`. Those are rewritten as Rust tests, not transpiled. **Production hand-written
  not-detected-as-reflective control logic is 6,151 − 801 = 5,350 files, 55.9% of D3** — an upper
  bound, inheriting (c)'s (§9).

  ```sh
  LC_ALL=C comm -23 "$S/nvnt.txt" "$S/gen-nvnt-refined.txt" > "$S/hand-nvnt.txt"
  wc -l < "$S/hand-nvnt.txt"                                              # 6189
  /usr/bin/grep -c '^\./test/' "$S/hand-nvnt.txt"                         #  803
  LC_ALL=C comm -23 "$S/hand-nvnt.txt" "$S/reflective-core.txt" > "$S/target-c.txt"
  wc -l < "$S/target-c.txt"                                               # 6151
  /usr/bin/grep -c  '^\./test/' "$S/target-c.txt"                         #  801
  /usr/bin/grep -vc '^\./test/' "$S/target-c.txt"                         # 5350
  ```

  The 803 → 801 step is real, not rounding: 2 of the 38 reflective-core files live under `./test/`
  and are already counted in (b), so they must not be subtracted from (c) twice.
- **(b) contains 11 test/fuzz files.** The **production reflective core is 27 files** (0.28% of D3).
- **`unsafe` cuts across all three:** 132 generated (inside (a)), 22 hand-written, of which only 11
  platform-interop files remain unsafe in Rust.

### What this does to the programme

**It shrinks it, as anticipated, and by more than the file count suggests.**

1. **A third of the corpus needs zero transpile rules.** 3,384 files and 929,168 lines are the
   deterministic output of 15 schema-driven generators. The Rust programme needs 15 generators (most
   of them derive macros), not 3,384 translations. The top 5 generators — applyconfiguration, client,
   informer, lister, deepcopy — produce 77.8% of that and are precisely the client-side surface a
   Rust client would generate from OpenAPI anyway.
2. **Reflection is not a transpile problem; it is 38 files.** 891 call sites. Of the nine categories,
   six map onto `serde`, derive macros, traits, or generated registries. **One does not: JSONPath and
   Go-template evaluation (4 files) is genuine dynamic interpretation over a dynamic value tree, and
   it stays an interpreter in Rust** — that is the residue, and it is 4 files.
3. **The 213 shallow reflect users are a non-problem.** 596 sites, 226 of them `reflect.DeepEqual`.
   Every one is a Go workaround for a missing language feature — no generics-era equality derive, no
   `Option`, no compile-time type names. Rust deletes the need rather than translating the code.
4. **`unsafe` is 22 hand-written files and shrinks to 11.**
5. **The honest negative:** 6,151 files of hand-written control logic remain, and nothing in this
   census makes *them* smaller. This census bounds the reflection and codegen problem; it says
   nothing about the difficulty of the two-thirds that is ordinary Go. Sizing that is a different
   census.

---

## 8. What I could not determine

Stated plainly, because a wrong number here is expensive.

1. **Whether an individual `reflect` site is semantically necessary.** Deciding that
   `reflect.DeepEqual(a, b)` can become `a == b` requires knowing whether the types contain maps,
   func values, or NaN floats — Go's `DeepEqual` and Rust's `PartialEq` differ on all three. That is
   a **type-checked** question. My §5.2 claim that the 115 `DeepEqual`-only files are trivial is a
   claim about the *shape*, not a proof about each site.
   *To answer:* run `go/packages` in `LoadAllSyntax` mode over the corpus and inspect the static type
   of each `DeepEqual` argument. Blocked here — see item 7.
2. **Whether the 38-file core is complete.** It is the union of two idioms I chose (Kind-switching,
   `map[reflect.Type]`). A third idiom would add files. I found the second only by noticing
   `scheme.go` was missing from the first (§4.3), which is direct evidence that **one such blind spot
   existed and was caught; I cannot prove a third does not.** Treat 38 as a **lower bound**.
   Candidate misses: `reflect.MakeFunc` dispatch, `reflect.Value.MethodByName`, embedded-field
   traversal without Kind switches.
3. **Whether the 113 generated conversions can keep their zero-copy property.** §6 cluster 2 shows
   the safe Rust `From` is unconditional on safety and `O(n)` on cost, against the Go form's `O(1)`
   slice-header reinterpretation, on the API version-conversion path every apiserver request
   crosses. Whether the port gives the two API versions a single Rust representation — which
   removes the conversion rather than pricing it — is a programme decision, not a measurement, and
   it is not made here.
4. **The true generated-file population.** §2.2's three rows are marker-derived: they can only see
   files containing a generation phrase, and that method was already caught missing 58 unmarked
   `types_swagger_doc_generated.go` files in this corpus. Every generated/hand-written split in this
   record inherits the label.
   *To answer:* enumerate from the corpus's own generator manifest — the generator list driven by
   `hack/update-codegen.sh`, cross-checked against `make verify-generated` — rather than from any
   phrase search.
5. **Reflection reached through wrappers.** Code calling
   `apimachinery/third_party/forked/golang/reflect` or `k8s.io/apimachinery/pkg/api/equality`
   is reflecting without importing `reflect`. My counts attribute reflection to the wrapper, not the
   caller — correct for sizing the *implementation*, wrong for sizing the *blast radius*. §3.3 shows
   at least one such wrapper exists and is aliased to the name `reflect`.
6. **Line-level attribution of reflection.** I can count files and sites, not "lines of reflective
   logic". §5.4 shows why the file-LOC proxy fails by an order of magnitude on `describe.go`. The
   891-site count is offered instead; it is a count of call sites, not of the code around them.
7. **Exact package-level import facts from the Go toolchain.** Go 1.26.5 is installed and I attempted
   `go list -mod=vendor ./...` to get authoritative per-package imports. It fails on this checkout:

   ```
   go: inconsistent vendoring in <corpus>:
     k8s.io/api: is replaced in go.mod, but not marked as replaced in vendor/modules.txt
   ```

   Kubernetes symlinks `vendor/k8s.io/*` to `staging/src/k8s.io/*`, and this clone did not
   materialise those symlinks — `find ./vendor -maxdepth 3 -type l | wc -l` returns `0`, and
   `ls vendor/k8s.io` shows only 5 real directories (`gengo`, `klog`, `kube-openapi`,
   `system-validators`, `utils`) where the module graph expects the full staging set. Every
   type-aware question above is blocked on this same cause.
   *To unblock:* re-create the staging symlinks, or run `go list` per staging module. Cost: hours,
   not weeks — and it would convert items 1, 2 and 5 from bounds into exact answers. **I recommend it
   before anyone commits budget against these numbers.**
8. **Build-tag variance.** Counts include files excluded on every platform. For `unsafe` this
   inflates the hand-written count — 11 of 22 are Windows/darwin/BSD-only, so a Linux-only build sees
   roughly half. Deliberate: a port must cover the tree, not one build.

---

## 9. Bounds summary

| Figure | Value | Kind | Why |
| --- | ---: | --- | --- |
| Corpus files (D1/D2/D3) | 16,941 / 12,587 / 9,573 | exact | `find`, matches the shape given to this lane |
| `reflect` importers (D3) | 355 | exact | two independent parsers agree file-for-file (§3.1) |
| — generated | 104 | **exact relative to the marker-derived generated set (§2.2); not exact as generated vs hand-written** | set intersection against `gen-nvnt-refined.txt`, which §2.2 labels an estimate; an unmarked generated `reflect` importer moves a file between this row and the next |
| — hand-written | 251 | **exact relative to the marker-derived generated set (§2.2); not exact as generated vs hand-written** | set difference against `gen-nvnt-refined.txt`, which §2.2 labels an estimate; an unmarked generated `reflect` importer moves a file between this row and the one above |
| Generated files (D3) | 3,384 | **marker-derived estimate; not a bound** (§2.2) | every row of the §2.2 table can only see files bearing a generation phrase; the +58 swagger recovery in that section is the demonstration that unmarked generated files exist and are invisible to it |
| Generated lines (D3) | 929,168 | exact given the file set | `cat | wc -l` over the set |
| Reflective core | 38 | **lower bound** | union of two chosen idioms; a third would add (§8.2) |
| `reflect.X` sites, core | 891 (was 899) | **upper bound on executing sites** | §3.3's comment filter applied (§5.1); residual over-count from trailing comments, string literals and `//go:linkname`, which that filter does not catch |
| `reflect.X` sites, hand-written | 1,487 (was 1,511) | **upper bound on executing sites** | same filter; 891 + 596 reconciles independently, which checks the partition, not the filtering |
| Distinct symbol-set shapes | 73 | exact for the extraction, unchanged by filtering | no symbol and no file is comment-only, so the shape distribution does not move |
| `unsafe` importers (D3) | 154 | exact | same parser as `reflect`, same guarantees |
| — hand-written | 22 | **exact relative to the marker-derived generated set (§2.2); not exact as generated vs hand-written** | set difference against `gen-nvnt-refined.txt`, which §2.2 labels an estimate; an unmarked generated `unsafe` importer moves a file out of this row |
| Transpile target (c) | 6,151 files | **derived upper bound** | D3 − generated − core; the core is a **lower bound** (§8.2) and the generated set an estimate (§2.2), so this complement is an upper bound relative to the *detected* sets, not a measured non-reflective population |

Every count in this document was produced by running the command shown, on the tree at
`756939600b9a7180fc2df6550a4585b638875e67`, with `/usr/bin/grep`. None was inferred from reading
source.
