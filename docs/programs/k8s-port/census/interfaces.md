---
doc_class: Program-Census
doc_status: published
census_id: k8s-port-census-interfaces
subject: implicit-interface-surface
upstream_pin: 756939600b9a7180fc2df6550a4585b638875e67
measured_at: 2026-08-08
authority_tier: 3
---

# Census: the implicit-interface surface of the pinned Kubernetes corpus

This is a measurement record. It ratifies nothing and authorizes nothing. It exists to size the
rule corpus that the port engine must carry for Go interfaces, and to say where the residue is.

Authority is [ADR-0704](../../../decisions/ADR-0704-k8s-port-live-apex.md) (Accepted 2026-08-06),
the live apex for topic `k8s_port`. ADR-0637 and ADR-0638 originated this programme and are listed
in ADR-0704's `supersedes`; their files are no longer present under `docs/decisions/`, so this
census cites the apex rather than the superseded pair.

## Headline

**The trait corpus induced by this corpus is 1,448 – 2,412 distinct method sets, best estimate
2,077.** That is the number of Rust traits, not the 2,832 interface declarations. Declaration
count over-counts by roughly 14% because different packages declare interfaces with identical
method sets.

**Emitting only pairs the program actually uses instead of every structural match is a 17x–60x
prune**: 80,042 name-level structural matches narrow to 22,304 exact-signature matches (a LOWER
bound on structural satisfaction, not a ceiling — see §9), 5,573 of those are between packages that
can even see each other, and the source declares 1,323 outright as `var _ Iface = ...`. Whether the
declared set sits inside the 22,304 was not computed, so these are separate measurements rather
than nested ones. Structural matching is combinatorial; usage is not. The engine must
emit impls from usage.

**The orphan rule is a narrow problem, not a broad one.** Of 1,323 impls Kubernetes declares
explicitly, 6 (0.45%) are foreign-trait-on-foreign-type, the shape Rust forbids. But a small
number of high-fan-out capability probes — `documentable`, `ProtobufMarshaller` — are individually
worth 1,000+ impls each and are exactly the orphan shape. The count is small; the blast radius of
each is not.

## Provenance

| Item | Value |
|---|---|
| Corpus | Kubernetes, Apache-2.0 |
| Pin (peeled commit) | `756939600b9a7180fc2df6550a4585b638875e67` |
| Tag | `v1.36.1` |
| Pin authority | `specs/k8s-port/upstream-pin.json` |
| Checkout used | `$K8S` below |
| Measured | 2026-08-08 |
| Toolchain | `go version go1.26.5 darwin/arm64` |

Pin verified before counting anything:

```sh
K8S=/private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/222702d1-4719-4175-a349-71e41cd88f0d/scratchpad/k8s-corpus
git -C "$K8S" rev-parse HEAD
# 756939600b9a7180fc2df6550a4585b638875e67
```

That value equals `.pin.peeled_commit` in `specs/k8s-port/upstream-pin.json`.

## Denominators

Three populations exist and they are not interchangeable. Every percentage below states which one
it uses.

```sh
find "$K8S" -name '*.go' -not -path '*/.git/*' | wc -l                          # 16941  ALL
find "$K8S" -name '*.go' -not -path '*/.git/*' -not -path '*/vendor/*' | wc -l  # 12587  NO-VENDOR
find "$K8S" -name '*.go' -not -path '*/.git/*' -not -path '*/vendor/*' \
     -not -name '*_test.go' | wc -l                                             # 9573   CORE
```

**Unless stated otherwise, every figure in this document is over CORE: 9,573 non-vendor,
non-test `.go` files.** CORE is the right default for the port programme because vendored
third-party code is a separate sourcing decision and test files are a separate wave. Both other
populations are reported alongside in [§10](#10-scope-sensitivity) so the numbers can be re-based
without re-deriving them.

Note the vendor delta is smaller than it looks: `vendor/k8s.io/*` are symlinks into `staging/src/`,
which `find` does not follow, so the 4,354-file vendor population is genuine third-party code and
is not double-counted against staging.

## Method

Counts came from a purpose-written Go AST walker (`go/parser`, `go/printer`), not from regex.
Full source is in [Appendix A](#appendix-a-the-measurement-tool); it is the command.

```sh
# every figure below, by section tag:
go run census-interfaces.go "$K8S" core       # CORE     (9573 files)  <- default
go run census-interfaces.go "$K8S" coretest   # NO-VENDOR (12587 files)
go run census-interfaces.go "$K8S" all        # ALL      (16941 files)
go run census-interfaces.go "$K8S" json       # raw records
```

The tool parses all 16,941 files in every mode and filters at report time, so the whole-tree index
is available for resolving embedded interfaces into vendored packages even when reporting on CORE.

### What is exact and what is not

| Class | Status | Why |
|---|---|---|
| Declaration counts, method-set sizes, embed counts | **Exact** for top-level declarations | AST, not regex. Residue enumerated below. |
| `interface{}` / `any` occurrences and their syntactic role | **Exact** | AST type-expression walk with an explicit role label at each position. |
| Type assertions, type switches, case counts | **Exact** | AST node counts. Validated against grep, §5. |
| Inline interface literals | **Exact** | AST. |
| Distinct method sets after dedup | **Estimate inside a proven bracket** | Signatures are compared as written, not as resolved types. See §2. |
| Structural satisfaction (which type implements what) | **Bracketed, not exact** | Needs `go/types`. See §8. |
| "Used pairs" | **Not determined** | Needs whole-program assignment analysis. See §11. |

### Known residue in the declaration count

The walker counts top-level `type X interface` declarations. It does **not** count interfaces
declared inside function bodies. There are exactly 9 of those in NO-VENDOR, 7 in CORE, and they
are enumerated rather than estimated:

```sh
rg -n -g '!**/vendor/**' -g '*.go' '^[ \t]+type [A-Za-z_].* interface' "$K8S"
```

| Site | Shape |
|---|---|
| `pkg/controlplane/apiserver/aggregator.go:158` | `controller` — 2 methods |
| `staging/src/k8s.io/client-go/transport/transport.go:362` | `canceler` — 1 method |
| `staging/src/k8s.io/client-go/discovery/cached/disk/round_tripper.go:57` | `canceler` — 1 method |
| `staging/src/k8s.io/cli-runtime/pkg/genericclioptions/command_headers.go:94` | `canceler` — 1 method |
| `staging/src/k8s.io/apimachinery/pkg/util/net/http.go:253` | `closeIdler` — 1 method |
| `cmd/kubeadm/app/util/apiclient/dryrun.go:296` | `actionWithNameAndNamespace` — 1 embed + 2 methods |
| `cmd/kubeadm/app/util/apiclient/dryrun.go:303` | `actionWithObject` — 1 embed + 1 method |
| `pkg/controller/resourceclaim/controller_test.go:1185` | `object` (test only) |
| `test/e2e/framework/config/config_test.go:276` | `boolFlag` (test only) |

All 9 are ad-hoc capability probes declared immediately before a type switch or assertion — the
same shape as the inline interfaces in §3, and they should be handled by the same rule. **Corrected
CORE declaration total: 2,832 + 7 = 2,839.** The rest of this document uses 2,832 because that is
what the reproducible command emits; the 7 do not move any distribution.

### Cross-validation

The walker was validated two ways before its output was trusted.

1. **Hand-counted fixture.** A 36-line Go file with a known answer (6 named interfaces, 3 inline,
   5 empty-interface uses, 2 assertions, 1 type switch) reproduced exactly.
2. **Independent grep, with every discrepancy resolved to a named cause.** Not "close enough" —
   each difference was opened and read.

Type switches:

```sh
rg --no-heading -g '!**/vendor/**' -g '*.go' -c '\.\(type\)' "$K8S" | awk -F: '{s+=$NF} END {print s}'
# 369      vs AST 368 (NO-VENDOR)
```

The single difference is `test/utils/ktesting/assert.go:264`, a commented-out line
(`// switch arg := arg.(type) {`). AST 368 is correct; grep has 1 false positive, 0 false
negatives.

Interface declarations:

```sh
rg --no-heading -g '!**/vendor/**' -g '*.go' -c '^type [A-Za-z_].* interface' "$K8S" | awk -F: '{s+=$NF} END {print s}'
# 2904     vs AST 2880 (NO-VENDOR)
```

All 24 grep-only sites were inspected. Every one is a regex false positive of two kinds:
`type ProcessFunc func(obj interface{}, ...) error` (a func type whose *parameter* mentions
`interface`), and Go source held inside code-generator raw-string templates such as
`staging/src/k8s.io/code-generator/cmd/client-gen/generators/generator_for_clientset.go:104`.
There were **zero AST-only sites**: the AST count has no false positives, and its only false
negatives are the 9 function-local declarations above. Parse errors across all 16,941 files: **0**.

---

## 1. Interface declarations and method-set size

`go run census-interfaces.go "$K8S" core`, tags `S2`, `S3`, `S5`.

| Measure | CORE | Note |
|---|---|---|
| Interface declarations (top-level) | **2,832** | exact; +7 function-local (§Method) |
| — exported | 2,623 (92.6%) | |
| — generic constraint-like (`~T`, unions, `comparable`) | 10 | not traits; these are Rust bounds |
| — named empty (`type X interface{}`) | 410 (14.5%) | marker types; no method set at all |
| — type aliases | 1 | |
| — declarations with type parameters | 30 | |
| **Trait candidates** (method-bearing, non-constraint) | **2,412** | the population everything below uses |

That is 0.30 interface declarations per CORE file. The 410 named-empty interfaces deserve
separate attention: they are `type Object interface{}`-style markers that carry no methods, so
they are not traits at all. In Rust they are either a marker trait with a blanket impl, an enum, or
`Box<dyn Any>` — a rule-corpus decision distinct from every other interface in this census, and one
that touches 14.5% of declarations.

### Own explicit methods per trait candidate (exact)

Own methods only — embedded interfaces excluded, counted separately in §6.

| Methods | Interfaces | Share | Cumulative |
|---:|---:|---:|---:|
| 0 (embed-only) | 50 | 2.1% | 2.1% |
| 1 | 931 | 38.6% | 40.7% |
| 2 | 722 | 29.9% | 70.6% |
| 3 | 188 | 7.8% | 78.4% |
| 4 | 98 | 4.1% | 82.5% |
| 5 | 61 | 2.5% | 85.0% |
| 6–8 | 90 | 3.7% | 88.7% |
| 9 | 106 | 4.4% | 93.1% |
| 10–14 | 131 | 5.4% | 98.5% |
| 15–21 | 27 | 1.1% | 99.7% |
| 22+ | 8 | 0.3% | 100% |

Total 2,412; total own methods 7,453.

### Full method set after resolving embedded interfaces (exact where resolvable)

| Methods | Interfaces | Share |
|---:|---:|---:|
| 0 (all embeds unresolvable) | 1 | <0.1% |
| 1 | 761 | 31.6% |
| 2 | 757 | 31.4% |
| 3 | 230 | 9.5% |
| 4–5 | 202 | 8.4% |
| 6–9 | 234 | 9.7% |
| 10–14 | 167 | 6.9% |
| 15–29 | 43 | 1.8% |
| 30+ | 17 | 0.7% |

Total expanded methods: 9,070 (up from 7,453 own — embedding adds 22%).

**63.0% of traits have ≤2 methods after expansion; 9.4% have ≥10.** The mode is a one-method
interface. This is the single most encouraging number in the census: the trait corpus is dominated
by tiny capability traits that map to Rust almost mechanically.

The tail is where the cost sits, and it is concentrated and nameable:

| Interface | Methods |
|---|---:|
| `pkg/scheduler/framework.Framework` | 61 |
| `test/integration/apiserver/discovery.testClient` | 59 |
| `client-go/kubernetes.Interface` | 54 |
| `pkg/kubelet/volumemanager/cache.ActualStateOfWorld` | 47 |
| `cri-api/.../v1.RuntimeServiceServer` | 36 |
| `pkg/kubelet/cm.ContainerManager` | 35 |
| `cri-api/.../v1.RuntimeServiceClient` | 35 |

17 interfaces have ≥30 methods and 60 have ≥15. These are god-interfaces and they are a
hand-porting decision, not a rule: a 61-method trait with one implementation plus test fakes is
better served by a concrete struct than by a trait. That judgment is per-interface and there are
60 of them — a bounded, one-time cost, not a rule-corpus cost.

## 2. Distinct method sets — the trait-corpus size

`S6`. **This is the number that sizes the programme.**

| Basis | CORE | Meaning |
|---|---:|---|
| Trait-candidate declarations | 2,412 | **true upper bound** — dedup can only reduce |
| Distinct full signature sets | **2,077** | best estimate |
| Distinct method-*name* sets | **1,448** | **true lower bound** |
| Signature sets shared by ≥2 declarations | 192 | |

The two outer numbers are proven bounds, and the reasoning is worth stating because the middle
number is not a bound:

- **2,412 is a true upper bound.** Deduplication is a quotient; it never increases the count.
- **1,448 is a true lower bound.** Two interfaces whose method *names* differ cannot possibly be
  the same trait, whatever their signatures resolve to. So the count of distinct name sets can
  never exceed the count of distinct traits.
- **2,077 is an estimate, not a bound, and it errs in both directions.** Signatures are compared as
  *written*. A **false split** inflates it: the same type spelled `v1.Pod` in one package and
  `corev1.Pod` in another produces two keys for one trait. A **false merge** deflates it: two
  packages that each declare a local `Options` type yield an identical `Get() Options` key for
  genuinely different traits. Resolving this needs `go/types`; nothing cheaper closes it.

### The gap between 1,448 and 2,077 is generics, and it is measurable

The 629-set gap is not noise. It is almost entirely Kubernetes' generated client surface, where
one shape is instantiated per API resource:

| Method-name set | Declarations |
|---|---:|
| `Get;List` | 175 |
| `Informer;Lister` | 173 |
| `Apply;Create;Delete;DeleteCollection;Get;List;Patch;Update;Watch` | 78 |
| `Apply;ApplyStatus;Create;Delete;DeleteCollection;Get;List;Patch;Update;UpdateStatus;Watch` | 59 |
| `InformerFor;InformerName;Start` | 11 |

The top two name-sets alone cover 348 declarations — 14.4% of all trait candidates — and the top
five cover 496 (20.6%). These are `PodInterface`, `NodeInterface`, `ServiceInterface` … differing
only in the resource type they return. In Go they are 496 separate interfaces. In Rust they are a
handful of generic traits parameterised over the resource type.

**So the bracket is not "uncertainty", it is a design choice with a price tag:**

- Emit non-generic traits → ~2,077 traits.
- Emit generic traits where the corpus instantiates one shape per resource → ~1,448 traits,
  and roughly 500 fewer hand-maintained surfaces.

The generic path is worth ~30% of the trait corpus. It should be decided before the rule pack is
authored, because retro-fitting generics onto emitted traits is a rewrite, not a refactor.

## 3. Inline and anonymous interfaces

`S7`. These are the ones not enumerable from declarations, and the stated worry is that they make
the trait set induced by usage rather than declaration.

**In this corpus that worry is unrealised.** CORE has **46 inline interface literals** — one per
208 files, and 1.6% of the 2,832 declarations — and they reduce to **7 distinct shapes**:

| Shape | Sites |
|---|---:|
| `testEmbeddedByValue()` | 21 |
| `Cleanup(func()); mock.TestingT` | 20 |
| `Unwrap() []error` | 1 |
| `rest.Creater; rest.NamespaceScopedStrategy; rest.Scoper; rest.Storage` | 1 |
| `GetName() string` | 1 |
| `Deadline() (time.Time, bool)` | 1 |
| `Parallel()` | 1 |

By role: 25 are type-assertion targets (`x.(interface{ Foo() })`), 20 are function parameters, 1 is
a var declaration. By size: 25 have one element, 20 have two, 1 has four.

Adding tests raises this to 67 sites; including vendor, 152. **Even at the widest scope this is a
rounding error against a 2,077-trait corpus.** Inline interfaces need a rule — anonymous trait
synthesis with structural dedup against the named corpus — but they do not need a budget. Seven
shapes can be inspected by hand.

## 4. `interface{}` and `any`

`S8`. **10,212 occurrences in CORE** — 1.07 per file, exact.

| Form | Count | Share |
|---|---:|---:|
| `interface{}` | 9,456 | 92.6% |
| `any` | 756 | 7.4% |

Kubernetes has largely not migrated to `any`; the engine must treat the two as one input. By
syntactic role — this is the breakdown that matters, because the role decides the Rust target:

| Role | Count | Share of 10,212 | Plausible Rust target |
|---|---:|---:|---|
| function parameter | 5,905 | 57.8% | generic `T`, `impl Trait`, or `&dyn Any` |
| map value (`map[K]interface{}`) | 2,288 | 22.4% | `serde_json::Value` / owned dynamic enum |
| slice element (`[]interface{}`) | 998 | 9.8% | same as map value |
| function result | 455 | 4.5% | `Box<dyn Any>` — the hard direction |
| variadic parameter | 219 | 2.1% | format/log varargs; macro target |
| type-parameter constraint (`[T any]`) | 138 | 1.4% | unconstrained generic — free |
| struct field | 75 | 0.7% | needs a real type decision |
| local var declaration | 59 | 0.6% | |
| map key | 31 | 0.3% | |
| type argument | 24 | 0.2% | |
| explicit conversion | 12 | 0.1% | |
| channel element | 5 | <0.1% | |
| pointer element | 3 | <0.1% | |

Three readings matter for sizing:

1. **Over 80% of `interface{}` is parameters plus container elements** (5,905 + 2,288 + 998 =
   9,191, 90.0%). Parameters are the benign case: most are logging, formatting, and
   `DeepCopy`-style plumbing where a generic parameter or a trait bound is a faithful translation.
2. **The 3,286 container occurrences (map value + slice element, 32.2%) are one problem, not
   3,286.** `map[string]interface{}` and `[]interface{}` in this corpus are overwhelmingly decoded
   JSON/YAML — `unstructured.Unstructured` and friends. One rule that recognises the
   dynamic-document shape and targets a single owned value type retires roughly a third of the
   whole `interface{}` surface. It is the highest-leverage single rule the census found.
3. **The residue is 455 results plus 75 struct fields.** Returning `interface{}` is where Rust has
   no clean target, because the caller cannot recover the type without a downcast. 530 sites is
   small enough to enumerate and triage by hand.

`interface{}` used as a type-parameter constraint (138) is free — `[T any]` is an unconstrained
Rust generic.

**Expected false positives:** an `any` that has been shadowed by a local type or variable
declaration would be miscounted. Go permits this; the corpus is not expected to contain it, and the
count is a strict AST type-position walk, so a field or variable *named* `any` is not counted. The
error is bounded by however many shadowing declarations exist, which is plausibly zero and
certainly not material against 10,212.

## 5. Type assertions and type switches

`S9`, `S10`. Both exact; the type-switch count is the one validated line-by-line against grep
above.

### Type assertions — 12,290 in CORE

| Split | Count | Share |
|---|---:|---:|
| comma-ok (`v, ok := x.(T)`) | 2,565 | 20.9% |
| single-value (panics on failure) | 9,725 | **79.1%** |

| Target kind | Count | Share |
|---|---:|---:|
| pointer (`*T`) | 9,983 | 81.2% |
| selector (`pkg.T`) | 850 | 6.9% |
| bare ident (`T`) | 818 | 6.7% |
| other (slices, maps, generics) | 614 | 5.0% |
| inline interface literal | 25 | 0.2% |

Distinct target types: **3,731**. Distribution is long-tailed — the top target `string` is 220
sites (1.8%), and the top 25 targets cover 1,501 (12.2%). There is no small set of shapes here;
the rule must be general.

The 79.1% single-value figure is a translation hazard worth stating plainly: those 9,725 sites are
`panic`-on-mismatch in Go. A mechanical translation to Rust's `downcast_ref().unwrap()` preserves
semantics exactly, including the panic. That is faithful but it exports a Go runtime-safety
property into a Rust codebase where it will read as a bug. Whether the engine preserves the panic
or forces a `Result` is a doctrine decision affecting 9,725 sites; it is not a per-site judgment
and should be ruled once.

### Type switches — 282 in CORE

916 case types across 282 switches, mean 3.2. 198 (70.2%) have a `default`; only 10 (3.5%) have an
explicit `nil` case.

| Case types | Switches | Share |
|---:|---:|---:|
| 1 | 82 | 29.1% |
| 2 | 107 | 37.9% |
| 3 | 32 | 11.3% |
| 4–5 | 22 | 7.8% |
| 6–10 | 26 | 9.2% |
| 11+ | 13 | 4.6% |

**67.0% of type switches have ≤2 cases.** A 1–2 case type switch is not really a switch; it is a
downcast with a fallback, and it translates to `if let` on a downcast. The genuine
match-on-a-closed-set cases — where a Rust enum is the right target — are the 13 switches with 11+
cases, topping out at 28. That is a hand-portable population.

Distinct case types: 427. The most frequent are `map[string]interface{}` (28), `[]interface{}`
(27), `string` (27), `int64` (21), `float64` (20) — i.e. the top of the type-switch distribution is
the *same* dynamic-document problem as §4, reached from a different direction. The next tier
(`*appsv1.Deployment` 15, `*corev1.Pod` 14, `*appsv1.ReplicaSet` 12) is API-version dispatch, which
is a closed set per switch and a clean enum target.

## 6. Embedded interfaces

`S4`, `S5`. 769 of 2,412 trait candidates (31.9%) embed at least one interface; 960 embed elements
total.

| Embeds | Interfaces |
|---:|---:|
| 0 | 1,643 |
| 1 | 671 |
| 2 | 59 |
| 3 | 13 |
| 4 | 14 |
| 5 | 6 |
| 6 | 5 |
| 16 | 1 |

Embedding is shallow: 87.3% of embedding interfaces embed exactly one. Rust supertraits
(`trait A: B`) map this directly.

Embed resolution succeeded for **935 of 960 elements (97.4%)**, transitively, with a cycle guard.
The 25 unresolved are enumerated, not estimated:

| Unresolved target | Count | Why |
|---|---:|---|
| `net/http` | 13 | stdlib — outside the corpus |
| `io` | 5 | stdlib |
| `context`, `flag`, `net` | 3 | stdlib |
| `<local:error>` | 1 | builtin |
| `<local:*T>`, `<local:VolumeHost>`, `<unknown-alias:klog>` | 3 | generic type-param embed; dot-import |

**Consequence for §2:** interfaces embedding stdlib types have method sets that are 1–3 methods
short of their true size, so their dedup keys are slightly under-specified. 25 sites out of 2,412
interfaces; it does not move the bracket. It does mean the port needs a decision on stdlib trait
mapping (`io.Reader` → `std::io::Read`, `fmt.Stringer` → `Display`) as a *precondition* of the
rule pack, since those traits are foreign in the strictest sense — they belong to neither crate.

## 7. The orphan rule

Rust forbids `impl ForeignTrait for ForeignType`. Go has no such restriction, so a faithful
translation can be illegal. The question is how often.

### Declared impls — exact, and reassuring

Kubernetes writes 1,323 explicit satisfaction assertions in CORE (`var _ Iface = &T{}`), covering
1,316 distinct pairs and 516 distinct interfaces. These are the impls the authors *intended*:

| Quadrant | Count | Share | Legal in Rust? |
|---|---:|---:|---|
| foreign trait + local type | 919 | 69.5% | **yes** — the common case is fine |
| local trait + local type | 390 | 29.5% | **yes** |
| local trait + foreign type | 8 | 0.6% | **yes** |
| **foreign trait + foreign type** | **6** | **0.45%** | **no — needs a newtype or relocation** |

The overwhelming majority (99.5%) put the concrete type in the emitting crate, which is precisely
the case Rust permits. **The orphan rule blocks 6 of 1,323 declared impls.** Six newtypes is not a
programme risk.

The most-asserted interfaces show the shape: `admission.ValidationInterface` (30),
`rest.ShortNamesProvider` (28), `rest.CategoriesProvider` (20), `admission.MutationInterface` (18),
`fmt.Stringer` (18), `volume.VolumePlugin` (17). All foreign traits implemented on local plugin
types — legal.

### Undeclared impls — where the real exposure is

The declared set is not the whole picture, because Go does not require declaration. §8 finds 22,304
exact-signature structural matches, of which **16,731 (75.0%) are between a type whose package does
not import the interface's package at all**. Most of those are coincidental (§8), but not all, and
the exceptions are large. The clearest case, read from source rather than inferred:

```go
// staging/src/k8s.io/apiserver/pkg/endpoints/installer.go:175
type documentable interface {
    SwaggerDoc() map[string]string
}
```

`SwaggerDoc()` is generated onto every API type in `k8s.io/api`. The apiserver probes for it with a
type assertion. **1,128 types match, the match is genuine and used, and `k8s.io/api` does not
import `k8s.io/apiserver`** — nor could it, the dependency runs the other way. In Rust the trait
lives in the apiserver crate and the types live in the api crate, and neither crate may host the
impl. This one interface is an orphan violation worth 1,128 impls.

The same shape recurs: `runtime.ProtobufMarshaller` and five sibling probes at 1,264 matches each,
`metrics.resettable` at 1,588, `endpoints.documentable` at 1,128.

**So the orphan finding is two-sided and must be reported as such:** by count it is negligible
(0.45% of declared impls); by blast radius it is concentrated in perhaps a dozen high-fan-out
capability probes where a single unresolved case costs 1,000+ impls. The mitigation is
architectural, not per-site — sink the trait to a crate below both (`apimachinery`-level), or
generate the impls in the defining crate — and it must be decided before those crates are emitted,
because it determines crate layering.

## 8. Structural matches: the combinatorial risk, measured

`S11`. **This is bracketed, not exact.** Deciding whether a Go type satisfies an interface requires
resolved types; the walker compares signatures as written. Two computations bracket the truth:

- **Upper bound — method *names* only** (ignore signatures entirely). Cannot miss a real
  satisfaction; admits types whose method names coincide but whose signatures differ.
- **Lower bound — exact canonical signature strings must match**. Cannot admit a signature
  mismatch; misses real satisfactions where the same type is spelled differently across packages
  (`v1.Pod` vs `corev1.Pod`).

CORE has **9,017 concrete named types with methods**.

| Basis | Pairs |
|---|---:|
| Name-only match (upper bound) | **80,042** |
| Exact-signature match (lower bound) | **22,304** |
| — same package | 1,978 |
| — different package, type's package imports the interface's package | 3,595 |
| — different package, no import either way | 16,731 |

The distribution is where the story is, not the total. Of 2,411 interfaces with a resolvable method
set, **764 (31.7%) have zero exact-signature matches** and **708 (29.4%) have exactly one**. Over
60% of interfaces are implemented by at most one type. Meanwhile a handful attract thousands:

| Interface | Definition | Exact matches |
|---|---|---:|
| `kubectl/pkg/cmd/cp.pathSpec` | `String() string` | 1,807 |
| `component-base/metrics.resettable` | `Reset()` | 1,588 |
| `.../flowcontrol/metrics.resettable` | `Reset()` | 1,588 |
| `runtime.ProtobufMarshaller` + 5 siblings | marshal/size probes | 1,264–1,268 |
| `apiserver/pkg/endpoints.documentable` | `SwaggerDoc() map[string]string` | 1,128 |

Read from source, not inferred:

```go
// staging/src/k8s.io/kubectl/pkg/cmd/cp/filespec.go:31
type pathSpec interface {
    String() string
}
```

`pathSpec` is an unexported interface in `kubectl cp` with two intended implementors. It is
structurally matched by 1,807 types, because 1,807 types in Kubernetes have a `String() string`
method. **This is the combinatorial explosion, and it is entirely an artefact of emitting impls
for structural matches.** Nine interfaces account for 12,571 of the 22,304 exact matches (56.4%);
they are all 1-method probes over ubiquitous method names.

**An engine that emits `impl` for every structural match produces a corpus dominated by impls that
no Go program could ever have used.** Not slow-but-correct: wrong, and unbounded — the count grows
as the product of trait count and type count.

## 9. Used pairs, and the prune

The deliverable asks how many impls are needed if only *used* pairs are emitted. The honest answer
has one gap in it.

| Basis | Pairs | Status |
|---|---:|---|
| Name-only structural matches | 80,042 | upper bound, exact computation |
| Exact-signature structural matches | 22,304 | **lower bound on structural satisfaction** — NOT an upper bound on used pairs (see note) |
| …restricted to pairs whose packages can see each other | 5,573 | **not a bound** — see below |
| Declared `var _ Iface = T{}` assertions | 1,323 | **strict lower bound on used pairs** |
| Interface-typed downcast sites | 789, over 229 interfaces | exact; distinct probe surface |

**The prune is 17x from exact-signature matches to declared impls (22,304 → 1,323), and 60x from
the name-only ceiling.** That is the difference between bounded and combinatorial.

**Why 22,304 is not the ceiling.** §8's own method makes the exact-signature pass a LOWER bound on
structural satisfaction: it requires canonical signature strings to match, so it misses every real
satisfaction where the same type is spelled differently across packages (`v1.Pod` vs `corev1.Pod`).
A really-used pair can therefore sit outside the 22,304, which means structural satisfaction
brackets to [22,304, 80,042] and used pairs cannot be capped at 22,304. Nor is the declared set
shown to be a subset of it — that containment was never computed.

**What I could not determine: the true used-pair count.** It sits somewhere in [1,323, 80,042]. I
will not narrow it further than the evidence allows, and the reason the obvious tightening fails is
worth recording:

The 5,573 figure — pairs where the type's package is the interface's package or imports it — looks
like a tight upper bound and is not one. `documentable` (§7) is the counterexample: 1,128 genuinely
used impls where the type's package does not import the interface's package, brought together by a
third package that imports both. Any bound that assumes a direct import edge is wrong by at least
that much. 5,573 is a useful *subset* — the pairs that are trivially legal in Rust — not a ceiling.

Closing this needs `go/types` plus a whole-program walk of every site where a concrete value is
assigned, passed, returned, or stored into an interface-typed position. That is a real analysis:
`go/packages` load of the full corpus with type information, then an assignability pass over
`types.Info`. It is buildable — the corpus vendors its dependencies so it type-checks offline — and
it is the single measurement that would most improve this census. It was out of scope for a
read-only pass; it is not out of reach.

Until then, the defensible planning figures are the two anchors. **The declared-impl count of 1,323
is the better anchor of the two**, because it is what Kubernetes' own authors thought was worth
writing down, and the engine can extract exactly that set with no type information at all.

### The probe surface is small and enumerable

789 type assertions in CORE target a resolvable named interface — the duck-typed downcasts that
*force* a real impl in Rust — spread over just **229 distinct interfaces**. 486 (61.6%) target an
interface in another package. Of the 229 probe targets, **140 (61.1%) have ≤2 methods**:

| Probe target methods | Interfaces |
|---:|---:|
| 0–1 | 88 |
| 2 | 52 |
| 3–5 | 45 |
| 6–10 | 26 |
| 11+ | 18 |

A further ~100 assertions target stdlib interfaces (`context.Context` 63, `http.Hijacker` 11,
`error` 7, `http.Flusher` 6, …), which the tool cannot resolve because stdlib is outside the corpus;
that sample is a hand-picked list and therefore a lower bound on the stdlib class. Only 60
type-switch *cases* target an interface.

**229 interfaces is the enumerable duck-typing surface of Kubernetes.** It is small enough to
enumerate, triage, and rule individually. That, not the 2,077-trait corpus, is the population that
actually needs whole-program reasoning.

## 10. Scope sensitivity

Every headline re-based on the other two denominators, so downstream work never has to re-derive
them.

| Measure | CORE (9,573) | NO-VENDOR (12,587) | ALL (16,941) |
|---|---:|---:|---:|
| Interface declarations | 2,832 | 2,880 | 4,166 |
| Trait candidates | 2,412 | 2,456 | 3,728 |
| Distinct signature sets | 2,077 | 2,108 | 3,272 |
| Distinct name-only sets | 1,448 | 1,471 | 2,484 |
| Inline interfaces | 46 | 67 | 152 |
| `interface{}` / `any` | 10,212 | 15,408 | 22,303 |
| Type assertions | 12,290 | 14,835 | 20,083 |
| Type switches | 282 | 368 | 895 |
| Concrete types with methods | 9,017 | 10,301 | 16,012 |
| Exact-signature structural pairs | 22,304 | 24,963 | 63,119 |
| Declared impl assertions | 1,323 | 1,533 | 1,885 |
| Interface-typed downcast sites | 789 | 1,287 | 2,686 |
| — distinct interfaces probed | 229 | 264 | 556 |

Two observations. Adding tests costs +48 interface declarations (+1.7%) but +5,196 `interface{}`
sites (+50.9%) and +2,545 assertions (+20.7%) — **tests are dynamic-typing-heavy out of all
proportion to their interface surface**, and a test-porting wave should budget for §4 and §5 rules,
not §1 and §2 rules. Vendored code adds 1,286 declarations (+44.7% over NO-VENDOR), so a decision
to port rather than re-source vendored dependencies raises the trait corpus by roughly half.

## 11. What I could not determine

Stated plainly, because an honest gap is more useful than a substituted proxy.

1. **The true used-pair count.** Bracketed to [1,323, 80,042]; see §9. Needs `go/types` plus a
   whole-program assignability walk. This is the highest-value follow-up.
2. **Exact structural satisfaction.** Bracketed to [22,304, 80,042]. Same blocker. The brackets are
   3.6x apart, which is wide, but both ends are computed rather than guessed and the *shape* of the
   distribution (§8) is robust to which end is right.
3. **True distinct method sets.** Bracketed to [1,448, 2,412] with 2,077 as the estimate. The
   bounds are proven; the estimate is not. Same blocker.
4. **Whether a high-fan-out structural match is real or coincidental.** `documentable`'s 1,128 are
   real; `pathSpec`'s 1,807 are almost certainly not. I did not attempt to separate these
   automatically, and I do not think a syntactic method can. Roughly a dozen interfaces need this
   call and it should be made by reading them.
5. **Stdlib interface satisfaction.** The corpus excludes Go's standard library, so embeds of and
   assertions to `io.Reader`, `fmt.Stringer`, `error`, `context.Context` are visible as sites (25
   embeds, ~100 assertions) but their method sets are not resolvable here. Mapping these to Rust
   std traits is a prerequisite of the rule pack, not an output of this census.
6. **Shadowed `any`.** A locally-declared type or variable named `any` would corrupt the §4 count.
   Not checked; expected zero; bounded and immaterial.

## 12. Consequences for rule-corpus sizing

Read off the measurements, not asserted.

1. **Size the trait corpus at ~2,077 traits, or ~1,448 with generics.** Not 2,832. Decide the
   generics question first — it is worth ~630 traits and ~500 hand-maintained surfaces (§2).
2. **Emit impls from usage, never from structural matching.** The prune is 17x–60x (§9), and the
   unpruned count grows as traits × types (§8). This is the single most important structural
   ruling in this census.
3. **Bootstrap the used set from the 1,323 declared assertions.** They are extractable with no type
   information, they are what the authors intended, and they cover 516 interfaces.
4. **The duck-typing surface needing whole-program reasoning is 229 interfaces, not 2,077** (§9).
   61% of them have ≤2 methods. This is a tractable, enumerable population.
5. **One rule for `map[string]interface{}` / `[]interface{}` retires ~32.2% of the entire
   `interface{}` surface** (3,286 of 10,212 sites, §4). Highest single-rule leverage found.
6. **Rule the single-value type assertion once, globally.** 9,725 sites (79.1%) panic on mismatch
   in Go (§5). Preserve-the-panic and force-a-`Result` are both defensible; per-site judgment is
   not.
7. **The orphan rule needs a crate-layering decision, not 22,304 per-site decisions** (§7). Six
   declared impls are blocked; the exposure is concentrated in ~12 high-fan-out probes where the
   trait must sink below both crates.
8. **60 god-interfaces (≥15 methods) and 410 named-empty marker interfaces are hand
   decisions, not rules** (§1). Budget them as one-time work; do not attempt to rule them.

---

## Appendix A: the measurement tool

Reproducing every figure above requires this program, which is not committed to the repository —
this census's declared write scope is a single file. Save it as `census-interfaces.go` in an empty
directory with `go mod init census`, then run the commands in [§Method](#method).

`sha256(main.go) = 4263b2243cbd23c1623f0229df8499671f6939ccf1abf6fcf3564850d503abbd`

Design notes worth knowing before trusting its output:

- It parses with `parser.SkipObjectResolution` — syntax only, no type checking. Every limitation in
  §11 follows from this.
- `canonMethod` renders a method as `Name(paramTypes)(resultTypes)` with parameter *names* stripped
  and whitespace normalised, so `Read(p []byte) (int, error)` and `Read(b []byte) (n int, e error)`
  produce the same key. Type names are **not** resolved — this is the false-split/false-merge
  source in §2.
- Embedded interfaces resolve through each file's own import map, against an index built from all
  16,941 files, with a cycle guard. Unresolved targets are reported, never silently dropped.
- `interface{}` and `any` are recorded by an explicit type-expression walk that tags each position
  with a role; a variable or field merely *named* `any` is not counted.
- The `all`/`coretest`/`core` scopes filter at report time, not at parse time.

<details>
<summary>census-interfaces.go (full source)</summary>

```go
// census-interfaces.go — AST-level measurement of the Go implicit-interface
// surface over a pinned Kubernetes checkout. Read-only.
//
//   go run census-interfaces.go <corpus-root> core|coretest|all|json
//
// Emits tab-separated records: SECTION \t LABEL \t VALUE
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/printer"
	"go/token"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

var fset = token.NewFileSet()

func typeStr(e ast.Expr) string {
	var b bytes.Buffer
	if err := printer.Fprint(&b, fset, e); err != nil {
		return "?"
	}
	return strings.Join(strings.Fields(b.String()), " ")
}

type IfaceDecl struct {
	File       string   `json:"file"`
	Pkg        string   `json:"pkg"`
	Name       string   `json:"name"`
	Exported   bool     `json:"exported"`
	IsTest     bool     `json:"is_test"`
	IsAlias    bool     `json:"is_alias"`
	Generic    bool     `json:"generic"`
	Constraint bool     `json:"constraint"`
	Methods    []string `json:"methods"`
	Embeds     []string `json:"embeds"`
	Line       int      `json:"line"`
}

type InlineIface struct {
	File    string   `json:"file"`
	Pkg     string   `json:"pkg"`
	Line    int      `json:"line"`
	IsTest  bool     `json:"is_test"`
	Role    string   `json:"role"`
	Methods []string `json:"methods"`
	Embeds  []string `json:"embeds"`
	Empty   bool     `json:"empty"`
}

type EmptyIfaceUse struct {
	File   string `json:"file"`
	Line   int    `json:"line"`
	IsTest bool   `json:"is_test"`
	Role   string `json:"role"`
	Form   string `json:"form"`
}

type Assertion struct {
	File       string `json:"file"`
	Line       int    `json:"line"`
	IsTest     bool   `json:"is_test"`
	Target     string `json:"target"`
	TargetKind string `json:"target_kind"`
	CommaOk    bool   `json:"comma_ok"`
}

type TypeSwitch struct {
	File       string   `json:"file"`
	Line       int      `json:"line"`
	IsTest     bool     `json:"is_test"`
	NumCases   int      `json:"num_cases"`
	NumTypes   int      `json:"num_types"`
	HasNil     bool     `json:"has_nil"`
	HasDefault bool     `json:"has_default"`
	Types      []string `json:"types"`
}

type ConcreteMethod struct {
	File     string `json:"file"`
	Pkg      string `json:"pkg"`
	Recv     string `json:"recv"`
	Ptr      bool   `json:"ptr"`
	Method   string `json:"method"`
	Canon    string `json:"canon"`
	IsTest   bool   `json:"is_test"`
	Exported bool   `json:"exported"`
}

type ImplAssert struct {
	File   string `json:"file"`
	Pkg    string `json:"pkg"`
	Line   int    `json:"line"`
	IsTest bool   `json:"is_test"`
	Iface  string `json:"iface"`
	Value  string `json:"value"`
}

type Out struct {
	Files        int                          `json:"files_parsed"`
	ParseErrors  []string                     `json:"parse_errors"`
	Ifaces       []IfaceDecl                  `json:"ifaces"`
	Inline       []InlineIface                `json:"inline"`
	Empty        []EmptyIfaceUse              `json:"empty_uses"`
	Assertions   []Assertion                  `json:"assertions"`
	TypeSwitches []TypeSwitch                 `json:"type_switches"`
	Imports      map[string]map[string]string `json:"imports"`
	Methods      []ConcreteMethod             `json:"methods"`
	ImplAsserts  []ImplAssert                 `json:"impl_asserts"`
}

var out Out
var root string

func importPathOfDir(dir string) string {
	rel, err := filepath.Rel(root, dir)
	if err != nil {
		return dir
	}
	rel = filepath.ToSlash(rel)
	if strings.HasPrefix(rel, "staging/src/") {
		return strings.TrimPrefix(rel, "staging/src/")
	}
	if strings.HasPrefix(rel, "vendor/") {
		return strings.TrimPrefix(rel, "vendor/")
	}
	if rel == "." {
		return "k8s.io/kubernetes"
	}
	return "k8s.io/kubernetes/" + rel
}

type fileCtx struct {
	path   string
	pkg    string
	isTest bool
}

func isEmptyIfaceExpr(e ast.Expr) (bool, string) {
	switch t := e.(type) {
	case *ast.Ident:
		if t.Name == "any" {
			return true, "any"
		}
	case *ast.InterfaceType:
		if t.Methods == nil || len(t.Methods.List) == 0 {
			return true, "interface{}"
		}
	}
	return false, ""
}

func (fc *fileCtx) classifyType(e ast.Expr, role string) {
	if e == nil {
		return
	}
	if ok, form := isEmptyIfaceExpr(e); ok {
		out.Empty = append(out.Empty, EmptyIfaceUse{
			File: fc.path, Line: fset.Position(e.Pos()).Line, IsTest: fc.isTest,
			Role: role, Form: form,
		})
	}
	switch t := e.(type) {
	case *ast.InterfaceType:
		methods, embeds, _ := splitIface(t)
		if len(methods) > 0 || len(embeds) > 0 {
			out.Inline = append(out.Inline, InlineIface{
				File: fc.path, Pkg: fc.pkg, Line: fset.Position(t.Pos()).Line,
				IsTest: fc.isTest, Role: role, Methods: methods, Embeds: embeds,
			})
		}
		if t.Methods != nil {
			for _, f := range t.Methods.List {
				if ft, ok := f.Type.(*ast.FuncType); ok {
					fc.classifyFuncType(ft, "inline-iface-method")
				}
			}
		}
	case *ast.StarExpr:
		fc.classifyType(t.X, "pointer-elem")
	case *ast.ArrayType:
		fc.classifyType(t.Elt, "slice-elem")
	case *ast.MapType:
		fc.classifyType(t.Key, "map-key")
		fc.classifyType(t.Value, "map-value")
	case *ast.ChanType:
		fc.classifyType(t.Value, "chan-elem")
	case *ast.Ellipsis:
		fc.classifyType(t.Elt, "variadic-param")
	case *ast.FuncType:
		fc.classifyFuncType(t, "func-type")
	case *ast.StructType:
		if t.Fields != nil {
			for _, f := range t.Fields.List {
				fc.classifyType(f.Type, "struct-field")
			}
		}
	case *ast.IndexExpr:
		fc.classifyType(t.Index, "type-arg")
	case *ast.IndexListExpr:
		for _, ix := range t.Indices {
			fc.classifyType(ix, "type-arg")
		}
	case *ast.ParenExpr:
		fc.classifyType(t.X, role)
	case *ast.BinaryExpr:
		fc.classifyType(t.X, "constraint-union")
		fc.classifyType(t.Y, "constraint-union")
	case *ast.UnaryExpr:
		fc.classifyType(t.X, "constraint-tilde")
	}
}

func (fc *fileCtx) classifyFuncType(ft *ast.FuncType, ctx string) {
	if ft.TypeParams != nil {
		for _, f := range ft.TypeParams.List {
			fc.classifyType(f.Type, "type-param-constraint")
		}
	}
	if ft.Params != nil {
		for _, f := range ft.Params.List {
			fc.classifyType(f.Type, "func-param")
		}
	}
	if ft.Results != nil {
		for _, f := range ft.Results.List {
			fc.classifyType(f.Type, "func-result")
		}
	}
}

func splitIface(t *ast.InterfaceType) (methods []string, embeds []string, constraint bool) {
	if t.Methods == nil {
		return
	}
	for _, f := range t.Methods.List {
		if len(f.Names) > 0 {
			ft, ok := f.Type.(*ast.FuncType)
			if !ok {
				continue
			}
			methods = append(methods, canonMethod(f.Names[0].Name, ft))
			continue
		}
		embeds = append(embeds, typeStr(f.Type))
		switch e := f.Type.(type) {
		case *ast.BinaryExpr:
			constraint = true
		case *ast.UnaryExpr:
			constraint = true
		case *ast.Ident:
			switch e.Name {
			case "comparable", "int", "int8", "int16", "int32", "int64",
				"uint", "uint8", "uint16", "uint32", "uint64", "uintptr",
				"float32", "float64", "complex64", "complex128",
				"string", "bool", "byte", "rune":
				constraint = true
			}
		}
	}
	sort.Strings(methods)
	return
}

func canonMethod(name string, ft *ast.FuncType) string {
	render := func(fl *ast.FieldList) string {
		if fl == nil {
			return ""
		}
		var parts []string
		for _, f := range fl.List {
			n := 1
			if len(f.Names) > 0 {
				n = len(f.Names)
			}
			ts := typeStr(f.Type)
			for i := 0; i < n; i++ {
				parts = append(parts, ts)
			}
		}
		return strings.Join(parts, ",")
	}
	return fmt.Sprintf("%s(%s)(%s)", name, render(ft.Params), render(ft.Results))
}

func recvBase(e ast.Expr) (string, bool) {
	ptr := false
	for {
		switch t := e.(type) {
		case *ast.StarExpr:
			ptr = true
			e = t.X
		case *ast.ParenExpr:
			e = t.X
		case *ast.IndexExpr:
			e = t.X
		case *ast.IndexListExpr:
			e = t.X
		case *ast.Ident:
			return t.Name, ptr
		default:
			return "", ptr
		}
	}
}

func (fc *fileCtx) walkFile(f *ast.File) {
	imps := map[string]string{}
	for _, im := range f.Imports {
		p := strings.Trim(im.Path.Value, `"`)
		name := p[strings.LastIndex(p, "/")+1:]
		if im.Name != nil {
			name = im.Name.Name
		}
		imps[name] = p
	}
	if len(imps) > 0 {
		out.Imports[fc.path] = imps
	}
	fc.walkDecls(f)
}

func (fc *fileCtx) walkDecls(f *ast.File) {
	for _, d := range f.Decls {
		switch decl := d.(type) {
		case *ast.GenDecl:
			if decl.Tok == token.TYPE {
				for _, s := range decl.Specs {
					ts := s.(*ast.TypeSpec)
					if it, ok := ts.Type.(*ast.InterfaceType); ok {
						methods, embeds, constraint := splitIface(it)
						out.Ifaces = append(out.Ifaces, IfaceDecl{
							File: fc.path, Pkg: fc.pkg, Name: ts.Name.Name,
							Exported: ts.Name.IsExported(), IsTest: fc.isTest,
							IsAlias:    ts.Assign.IsValid(),
							Generic:    ts.TypeParams != nil,
							Constraint: constraint,
							Methods:    methods, Embeds: embeds,
							Line: fset.Position(ts.Pos()).Line,
						})
						if ts.TypeParams != nil {
							for _, tp := range ts.TypeParams.List {
								fc.classifyType(tp.Type, "type-param-constraint")
							}
						}
						if it.Methods != nil {
							for _, fl := range it.Methods.List {
								if ft, ok := fl.Type.(*ast.FuncType); ok {
									fc.classifyFuncType(ft, "iface-method")
								}
							}
						}
						continue
					}
					fc.classifyType(ts.Type, "type-decl-rhs")
					if ts.TypeParams != nil {
						for _, tp := range ts.TypeParams.List {
							fc.classifyType(tp.Type, "type-param-constraint")
						}
					}
				}
			}
			if decl.Tok == token.VAR || decl.Tok == token.CONST {
				for _, s := range decl.Specs {
					vs := s.(*ast.ValueSpec)
					if vs.Type != nil {
						fc.classifyType(vs.Type, "var-decl")
					}
					if decl.Tok == token.VAR && vs.Type != nil && len(vs.Names) == 1 &&
						vs.Names[0].Name == "_" && len(vs.Values) == 1 {
						switch vs.Type.(type) {
						case *ast.Ident, *ast.SelectorExpr, *ast.IndexExpr, *ast.IndexListExpr:
							out.ImplAsserts = append(out.ImplAsserts, ImplAssert{
								File: fc.path, Pkg: fc.pkg,
								Line:   fset.Position(vs.Pos()).Line,
								IsTest: fc.isTest,
								Iface:  typeStr(vs.Type), Value: typeStr(vs.Values[0]),
							})
						}
					}
				}
			}
		case *ast.FuncDecl:
			if decl.Recv != nil && len(decl.Recv.List) > 0 {
				for _, f := range decl.Recv.List {
					fc.classifyType(f.Type, "method-receiver")
				}
				base, ptr := recvBase(decl.Recv.List[0].Type)
				if base != "" {
					out.Methods = append(out.Methods, ConcreteMethod{
						File: fc.path,
						Pkg:  fc.pkg, Recv: base, Ptr: ptr, Method: decl.Name.Name,
						Canon: canonMethod(decl.Name.Name, decl.Type), IsTest: fc.isTest,
						Exported: decl.Name.IsExported(),
					})
				}
			}
			fc.classifyFuncType(decl.Type, "func-decl")
		}
	}
	fc.walkBodies(f)
}

func (fc *fileCtx) walkBodies(f *ast.File) {
	commaOk := map[ast.Node]bool{}
	ast.Inspect(f, func(n ast.Node) bool {
		if s, ok := n.(*ast.AssignStmt); ok {
			if len(s.Lhs) == 2 && len(s.Rhs) == 1 {
				if ta, ok := s.Rhs[0].(*ast.TypeAssertExpr); ok {
					commaOk[ta] = true
				}
			}
		}
		return true
	})
	ast.Inspect(f, func(n ast.Node) bool {
		switch s := n.(type) {
		case *ast.TypeAssertExpr:
			if s.Type == nil {
				return true // part of a type switch; counted there
			}
			kind := "other"
			switch s.Type.(type) {
			case *ast.Ident:
				kind = "ident"
			case *ast.SelectorExpr:
				kind = "selector"
			case *ast.StarExpr:
				kind = "pointer"
			case *ast.InterfaceType:
				kind = "iface-literal"
			}
			out.Assertions = append(out.Assertions, Assertion{
				File: fc.path, Line: fset.Position(s.Pos()).Line, IsTest: fc.isTest,
				Target: typeStr(s.Type), TargetKind: kind, CommaOk: commaOk[s],
			})
			fc.classifyType(s.Type, "type-assertion")
		case *ast.TypeSwitchStmt:
			ts := TypeSwitch{File: fc.path, Line: fset.Position(s.Pos()).Line, IsTest: fc.isTest}
			if s.Body != nil {
				for _, c := range s.Body.List {
					cc, ok := c.(*ast.CaseClause)
					if !ok {
						continue
					}
					ts.NumCases++
					if cc.List == nil {
						ts.HasDefault = true
						continue
					}
					for _, e := range cc.List {
						if id, ok := e.(*ast.Ident); ok && id.Name == "nil" {
							ts.HasNil = true
						}
						ts.NumTypes++
						ts.Types = append(ts.Types, typeStr(e))
						fc.classifyType(e, "type-switch-case")
					}
				}
			}
			out.TypeSwitches = append(out.TypeSwitches, ts)
		case *ast.DeclStmt:
			if gd, ok := s.Decl.(*ast.GenDecl); ok && (gd.Tok == token.VAR || gd.Tok == token.CONST) {
				for _, sp := range gd.Specs {
					if vs, ok := sp.(*ast.ValueSpec); ok && vs.Type != nil {
						fc.classifyType(vs.Type, "local-var-decl")
					}
				}
			}
		case *ast.CompositeLit:
			if s.Type != nil {
				fc.classifyType(s.Type, "composite-lit-type")
			}
		case *ast.FuncLit:
			fc.classifyFuncType(s.Type, "func-lit")
		case *ast.CallExpr:
			if _, ok := s.Fun.(*ast.InterfaceType); ok {
				fc.classifyType(s.Fun, "conversion")
			}
		}
		return true
	})
}

// ---------- analysis ----------

func isVendor(file string) bool { return strings.HasPrefix(file, "vendor/") }

func inScope(file string, isTest bool, scope string) bool {
	switch scope {
	case "core":
		return !isVendor(file) && !isTest
	case "coretest":
		return !isVendor(file)
	default:
		return true
	}
}

type key struct{ pkg, name string }

func hist(w io.Writer, section string, counts map[int]int) {
	var ks []int
	total, sites := 0, 0
	for k, v := range counts {
		ks = append(ks, k)
		total += v
		sites += k * v
	}
	sort.Ints(ks)
	for _, k := range ks {
		fmt.Fprintf(w, "%s\t%d\t%d\n", section, k, counts[k])
	}
	fmt.Fprintf(w, "%s.TOTAL\t-\t%d\n", section, total)
	fmt.Fprintf(w, "%s.WEIGHTED\t-\t%d\n", section, sites)
}

func topN(w io.Writer, section string, counts map[string]int, n int) {
	type kv struct {
		k string
		v int
	}
	var xs []kv
	total := 0
	for k, v := range counts {
		xs = append(xs, kv{k, v})
		total += v
	}
	sort.Slice(xs, func(i, j int) bool {
		if xs[i].v != xs[j].v {
			return xs[i].v > xs[j].v
		}
		return xs[i].k < xs[j].k
	})
	fmt.Fprintf(w, "%s.DISTINCT\t-\t%d\n", section, len(xs))
	fmt.Fprintf(w, "%s.TOTAL\t-\t%d\n", section, total)
	cum := 0
	for i, x := range xs {
		if i >= n {
			break
		}
		cum += x.v
		fmt.Fprintf(w, "%s\t%s\t%d\t%d\n", section, x.k, x.v, cum)
	}
}

func resolveEmbeds(idx map[key]*IfaceDecl, id *IfaceDecl, seen map[key]bool, unresolved *map[string]int) []string {
	k := key{id.Pkg, id.Name}
	if seen[k] {
		return nil
	}
	seen[k] = true
	set := map[string]bool{}
	for _, m := range id.Methods {
		set[m] = true
	}
	for _, e := range id.Embeds {
		base := e
		if i := strings.Index(base, "["); i > 0 {
			base = base[:i]
		}
		base = strings.TrimSpace(base)
		if strings.ContainsAny(base, "|~ ") {
			continue // constraint element, not a method-set embed
		}
		var target *IfaceDecl
		if dot := strings.LastIndex(base, "."); dot > 0 {
			alias, name := base[:dot], base[dot+1:]
			if imps, ok := out.Imports[id.File]; ok {
				if path, ok := imps[alias]; ok {
					target = idx[key{path, name}]
					if target == nil {
						(*unresolved)[path]++
					}
				} else {
					(*unresolved)["<unknown-alias:"+alias+">"]++
				}
			} else {
				(*unresolved)["<no-imports>"]++
			}
		} else {
			target = idx[key{id.Pkg, base}]
			if target == nil {
				(*unresolved)["<local:"+base+">"]++
			}
		}
		if target != nil {
			for _, m := range resolveEmbeds(idx, target, seen, unresolved) {
				set[m] = true
			}
		}
	}
	var res []string
	for m := range set {
		res = append(res, m)
	}
	sort.Strings(res)
	return res
}

func methodNames(canon []string) []string {
	var ns []string
	for _, c := range canon {
		if i := strings.Index(c, "("); i > 0 {
			ns = append(ns, c[:i])
		}
	}
	sort.Strings(ns)
	return ns
}

func valueTypeToken(v string) string {
	s := v
	if i := strings.Index(s, "{"); i >= 0 {
		s = s[:i]
	}
	if i := strings.Index(s, "("); i >= 0 && strings.HasPrefix(strings.TrimSpace(s), "(") {
		if j := strings.Index(s, ")"); j > i {
			s = s[i+1 : j]
		}
	}
	s = strings.TrimPrefix(strings.TrimSpace(s), "new(")
	s = strings.TrimSuffix(s, ")")
	s = strings.TrimLeft(strings.TrimSpace(s), "&*(")
	s = strings.TrimRight(strings.TrimSpace(s), ")")
	if i := strings.IndexAny(s, "[ "); i > 0 {
		s = s[:i]
	}
	return strings.TrimSpace(s)
}

func report(w io.Writer, scope string) {
	p := func(section, label string, v int) {
		fmt.Fprintf(w, "%s\t%s\t%d\n", section, label, v)
	}

	idx := map[key]*IfaceDecl{}
	for i := range out.Ifaces {
		id := &out.Ifaces[i]
		if _, dup := idx[key{id.Pkg, id.Name}]; !dup {
			idx[key{id.Pkg, id.Name}] = id
		}
	}

	p("S1.files_parsed", "all", out.Files)
	p("S1.parse_errors", "all", len(out.ParseErrors))
	p("S1.scope", scope, 0)

	var scoped []*IfaceDecl
	for i := range out.Ifaces {
		id := &out.Ifaces[i]
		if inScope(id.File, id.IsTest, scope) {
			scoped = append(scoped, id)
		}
	}
	nConstraint, nEmpty, nAlias, nGeneric, nExported, nEmbedOnly := 0, 0, 0, 0, 0, 0
	for _, id := range scoped {
		if id.Constraint {
			nConstraint++
		}
		if len(id.Methods) == 0 && len(id.Embeds) == 0 && !id.Constraint {
			nEmpty++
		}
		if id.IsAlias {
			nAlias++
		}
		if id.Generic {
			nGeneric++
		}
		if id.Exported {
			nExported++
		}
		if len(id.Methods) == 0 && len(id.Embeds) > 0 {
			nEmbedOnly++
		}
	}
	p("S2.decls", "total", len(scoped))
	p("S2.decls", "exported", nExported)
	p("S2.decls", "constraint_like", nConstraint)
	p("S2.decls", "named_empty", nEmpty)
	p("S2.decls", "alias", nAlias)
	p("S2.decls", "generic_decl", nGeneric)
	p("S2.decls", "embed_only_no_own_methods", nEmbedOnly)

	var traits []*IfaceDecl
	for _, id := range scoped {
		if id.Constraint || (len(id.Methods) == 0 && len(id.Embeds) == 0) {
			continue
		}
		traits = append(traits, id)
	}
	p("S2.decls", "trait_candidates", len(traits))

	own := map[int]int{}
	for _, id := range traits {
		own[len(id.Methods)]++
	}
	hist(w, "S3.own_method_count", own)

	embedCount := map[int]int{}
	nWithEmbeds := 0
	unresolved := map[string]int{}
	for _, id := range traits {
		embedCount[len(id.Embeds)]++
		if len(id.Embeds) > 0 {
			nWithEmbeds++
		}
	}
	hist(w, "S4.embed_count", embedCount)
	p("S4.embedding", "ifaces_with_embeds", nWithEmbeds)

	full := map[*IfaceDecl][]string{}
	expanded := map[int]int{}
	for _, id := range traits {
		ms := resolveEmbeds(idx, id, map[key]bool{}, &unresolved)
		full[id] = ms
		expanded[len(ms)]++
	}
	hist(w, "S5.expanded_method_count", expanded)
	topN(w, "S5.unresolved_embed_target", unresolved, 25)

	strict := map[string]int{}
	loose := map[string]int{}
	for _, id := range traits {
		ms := full[id]
		if len(ms) == 0 {
			continue
		}
		strict[strings.Join(ms, ";")]++
		loose[strings.Join(methodNames(ms), ";")]++
	}
	p("S6.dedup", "sets_considered", len(traits))
	p("S6.dedup", "distinct_strict_signature_sets", len(strict))
	p("S6.dedup", "distinct_name_only_sets", len(loose))
	collapsed := 0
	for _, v := range strict {
		if v > 1 {
			collapsed++
		}
	}
	p("S6.dedup", "strict_sets_shared_by_2plus_decls", collapsed)
	topN(w, "S6.top_shared_name_only_sets", loose, 15)

	roles := map[string]int{}
	inlineSizes := map[int]int{}
	nInline := 0
	for _, il := range out.Inline {
		if !inScope(il.File, il.IsTest, scope) {
			continue
		}
		nInline++
		roles[il.Role]++
		inlineSizes[len(il.Methods)+len(il.Embeds)]++
	}
	p("S7.inline", "total", nInline)
	topN(w, "S7.inline_role", roles, 20)
	hist(w, "S7.inline_size", inlineSizes)

	eroles := map[string]int{}
	eforms := map[string]int{}
	nEmptyUse := 0
	for _, e := range out.Empty {
		if !inScope(e.File, e.IsTest, scope) {
			continue
		}
		nEmptyUse++
		eroles[e.Role]++
		eforms[e.Form]++
	}
	p("S8.empty_iface", "total", nEmptyUse)
	topN(w, "S8.empty_role", eroles, 30)
	topN(w, "S8.empty_form", eforms, 5)

	kinds := map[string]int{}
	targets := map[string]int{}
	commaOk, single := 0, 0
	nAssert := 0
	for _, a := range out.Assertions {
		if !inScope(a.File, a.IsTest, scope) {
			continue
		}
		nAssert++
		kinds[a.TargetKind]++
		targets[a.Target]++
		if a.CommaOk {
			commaOk++
		} else {
			single++
		}
	}
	p("S9.assertions", "total", nAssert)
	p("S9.assertions", "comma_ok", commaOk)
	p("S9.assertions", "single_value_panicking", single)
	topN(w, "S9.assert_target_kind", kinds, 10)
	topN(w, "S9.assert_target", targets, 25)

	caseHist := map[int]int{}
	swTypes := map[string]int{}
	nSw, nSwTypes, nNil, nDefault := 0, 0, 0, 0
	for _, ts := range out.TypeSwitches {
		if !inScope(ts.File, ts.IsTest, scope) {
			continue
		}
		nSw++
		caseHist[ts.NumTypes]++
		nSwTypes += ts.NumTypes
		if ts.HasNil {
			nNil++
		}
		if ts.HasDefault {
			nDefault++
		}
		for _, t := range ts.Types {
			swTypes[t]++
		}
	}
	p("S10.type_switch", "total", nSw)
	p("S10.type_switch", "total_case_types", nSwTypes)
	p("S10.type_switch", "with_nil_case", nNil)
	p("S10.type_switch", "with_default", nDefault)
	hist(w, "S10.cases_per_switch", caseHist)
	topN(w, "S10.case_type", swTypes, 20)

	type conc struct {
		names map[string]bool
		canon map[string]bool
		file  string
	}
	types := map[key]*conc{}
	for _, m := range out.Methods {
		if !inScope(m.File, m.IsTest, scope) {
			continue
		}
		k := key{m.Pkg, m.Recv}
		c := types[k]
		if c == nil {
			c = &conc{names: map[string]bool{}, canon: map[string]bool{}, file: m.File}
			types[k] = c
		}
		c.names[m.Method] = true
		c.canon[m.Canon] = true
	}
	p("S11.concrete_types_with_methods", "total", len(types))

	pkgImports := map[string]map[string]bool{}
	pkgOfFile := map[string]string{}
	for i := range out.Ifaces {
		pkgOfFile[out.Ifaces[i].File] = out.Ifaces[i].Pkg
	}
	for _, m := range out.Methods {
		pkgOfFile[m.File] = m.Pkg
	}
	for f, imps := range out.Imports {
		pk := pkgOfFile[f]
		if pk == "" {
			continue
		}
		s := pkgImports[pk]
		if s == nil {
			s = map[string]bool{}
			pkgImports[pk] = s
		}
		for _, path := range imps {
			s[path] = true
		}
	}

	byName := map[string][]key{}
	for k, c := range types {
		for n := range c.names {
			byName[n] = append(byName[n], k)
		}
	}

	upper, lower, orphanUpper, samePkg := 0, 0, 0, 0
	lowerSamePkg, lowerImports, lowerOrphan := 0, 0, 0
	matchHist := map[int]int{}
	matchHistExact := map[int]int{}
	exactPerIface := map[string]int{}
	sizePerIface := map[string]int{}
	for _, id := range traits {
		ms := full[id]
		if len(ms) == 0 {
			continue
		}
		names := methodNames(ms)
		var cand []key
		best := -1
		for _, n := range names {
			l := len(byName[n])
			if best < 0 || l < best {
				best, cand = l, byName[n]
			}
		}
		perIface, perIfaceExact := 0, 0
		for _, ck := range cand {
			c := types[ck]
			okNames := true
			for _, n := range names {
				if !c.names[n] {
					okNames = false
					break
				}
			}
			if !okNames {
				continue
			}
			upper++
			perIface++
			okCanon := true
			for _, m := range ms {
				if !c.canon[m] {
					okCanon = false
					break
				}
			}
			if okCanon {
				lower++
				perIfaceExact++
				switch {
				case ck.pkg == id.Pkg:
					lowerSamePkg++
				case pkgImports[ck.pkg][id.Pkg]:
					lowerImports++
				default:
					lowerOrphan++
				}
			}
			if ck.pkg == id.Pkg {
				samePkg++
			} else if !pkgImports[ck.pkg][id.Pkg] {
				orphanUpper++
			}
		}
		matchHist[perIface]++
		matchHistExact[perIfaceExact]++
		label := id.Pkg + "." + id.Name
		exactPerIface[label] = perIfaceExact
		sizePerIface[label] = len(ms)
	}
	p("S11.structural_match", "upper_bound_name_only_pairs", upper)
	p("S11.structural_match", "lower_bound_exact_signature_pairs", lower)
	p("S11.structural_match", "pairs_same_package", samePkg)
	p("S11.structural_match", "pairs_type_pkg_does_not_import_iface_pkg", orphanUpper)
	topN(w, "S11.top_iface_by_exact_matches", exactPerIface, 20)
	topN(w, "S11.top_iface_by_method_count", sizePerIface, 20)
	p("S11.exact_match", "same_package", lowerSamePkg)
	p("S11.exact_match", "cross_pkg_type_imports_iface_pkg", lowerImports)
	p("S11.exact_match", "cross_pkg_no_import_orphan_risk", lowerOrphan)
	hist(w, "S11.matches_per_iface", matchHist)
	hist(w, "S11.exact_matches_per_iface", matchHistExact)

	lookupIface := func(file, target string) *IfaceDecl {
		pk := pkgOfFile[file]
		t := strings.TrimSpace(target)
		if i := strings.Index(t, "["); i > 0 {
			t = t[:i]
		}
		if strings.ContainsAny(t, "*[]{}() ") {
			return nil
		}
		if dot := strings.LastIndex(t, "."); dot > 0 {
			alias, name := t[:dot], t[dot+1:]
			if imps, ok := out.Imports[file]; ok {
				if path, ok := imps[alias]; ok {
					return idx[key{path, name}]
				}
			}
			return nil
		}
		return idx[key{pk, t}]
	}
	nIfaceAssert, nIfaceAssertForeign := 0, 0
	ifaceAssertTargets := map[string]int{}
	probeSize := map[int]int{}
	probeSeen := map[string]bool{}
	for _, a := range out.Assertions {
		if !inScope(a.File, a.IsTest, scope) {
			continue
		}
		if id := lookupIface(a.File, a.Target); id != nil {
			nIfaceAssert++
			ifaceAssertTargets[id.Pkg+"."+id.Name]++
			if id.Pkg != pkgOfFile[a.File] {
				nIfaceAssertForeign++
			}
			lbl := id.Pkg + "." + id.Name
			if !probeSeen[lbl] {
				probeSeen[lbl] = true
				ms, ok := full[id]
				if !ok {
					ms = resolveEmbeds(idx, id, map[key]bool{}, &unresolved)
				}
				probeSize[len(ms)]++
			}
		}
	}
	nIfaceCase := 0
	for _, ts := range out.TypeSwitches {
		if !inScope(ts.File, ts.IsTest, scope) {
			continue
		}
		for _, t := range ts.Types {
			if lookupIface(ts.File, t) != nil {
				nIfaceCase++
			}
		}
	}
	p("S13.iface_typed_assertions", "total", nIfaceAssert)
	p("S13.iface_typed_assertions", "target_iface_in_other_package", nIfaceAssertForeign)
	p("S13.iface_typed_switch_cases", "total", nIfaceCase)
	hist(w, "S13.probe_target_method_count", probeSize)
	topN(w, "S13.iface_assert_target", ifaceAssertTargets, 20)

	nIA, crossPkg := 0, 0
	iaIface := map[string]int{}
	quad := map[string]int{}
	iaPairs := map[string]bool{}
	for _, ia := range out.ImplAsserts {
		if !inScope(ia.File, ia.IsTest, scope) {
			continue
		}
		nIA++
		iaIface[ia.Iface]++
		traitForeign := strings.Contains(ia.Iface, ".")
		if traitForeign {
			crossPkg++
		}
		typeForeign := strings.Contains(valueTypeToken(ia.Value), ".")
		lbl := "trait_local"
		if traitForeign {
			lbl = "trait_foreign"
		}
		if typeForeign {
			lbl += "+type_foreign"
		} else {
			lbl += "+type_local"
		}
		quad[lbl]++
		iaPairs[ia.Pkg+"|"+ia.Iface+"|"+valueTypeToken(ia.Value)] = true
	}
	p("S12.impl_assertions", "total", nIA)
	p("S12.impl_assertions", "iface_qualified_foreign_package", crossPkg)
	p("S12.impl_assertions", "iface_local_package", nIA-crossPkg)
	p("S12.impl_assertions", "distinct_pairs", len(iaPairs))
	topN(w, "S12.orphan_quadrant", quad, 6)
	topN(w, "S12.assert_iface", iaIface, 20)
}

func main() {
	root = os.Args[1]
	mode := os.Args[2] // json | core | coretest | all
	out.Imports = map[string]map[string]string{}
	var files []string
	err := filepath.Walk(root, func(p string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			if info.Name() == ".git" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(p, ".go") {
			return nil
		}
		files = append(files, p)
		return nil
	})
	if err != nil {
		panic(err)
	}
	for _, p := range files {
		f, err := parser.ParseFile(fset, p, nil, parser.SkipObjectResolution)
		if err != nil {
			out.ParseErrors = append(out.ParseErrors, fmt.Sprintf("%s: %v", p, err))
			continue
		}
		rel, _ := filepath.Rel(root, p)
		fc := &fileCtx{
			path:   filepath.ToSlash(rel),
			pkg:    importPathOfDir(filepath.Dir(p)),
			isTest: strings.HasSuffix(p, "_test.go"),
		}
		out.Files++
		fc.walkFile(f)
	}
	if mode == "json" {
		json.NewEncoder(os.Stdout).Encode(&out)
		return
	}
	report(os.Stdout, mode)
}
```

</details>

### Appendix A.1: fixture self-check

The tool was validated against this hand-counted file before its corpus output was used. Expected:
6 named interfaces, 3 inline, 5 empty-interface uses, 2 assertions, 1 type switch. Observed:
identical.

```go
package pkg

import "io"

type Reader interface{ Read(p []byte) (int, error) }
type Writer interface{ Write(p []byte) (n int, err error) }
type RW interface {
	Reader
	io.Writer
}
type Empty interface{}
type Num interface{ ~int | ~float64 }
type Big interface {
	A()
	B(x any) error
	C() (interface{ Close() error }, error)
}

func f(cb interface{ Foo() }, v any, m map[string]interface{}) {
	var w io.Writer
	if r, ok := w.(Reader); ok {
		_ = r
	}
	_ = w.(io.Closer)
	switch v.(type) {
	case int, string:
	case nil:
	default:
	}
}

type S struct {
	F any
	G interface{}
	H interface{ Bar() int }
}
```

