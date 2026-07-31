# ADR-0510 cutover-trigger measurement

**Measured 2026-07-31** against `origin/dev` @ `953d4a49b` and the live GitHub state of
`jason931225/oyatie`. Read-only analysis; no code or ADR changed.

## Verdict

**Cutover is INDEFINITE.** Not one of the four triggers is within **50x** of firing.

Three triggers are measurable and read **1.2 %–1.9 %** of their thresholds. The fourth is
**not measurable as written** — it names an API surface this repo does not use, pairs it with a
latency metric nobody produces, and offers a qualitative escape hatch that the ADR's own
"Rejected alternatives" section explicitly forbids.

Worse than "far": **two of the four triggers are moving away from firing.** The tracked-file
count has *dropped* 21.7 % since the ADR was authored. On the current trend the working-set
trigger never fires.

There is also no evaluator. ADR-0510 mitigates its own drift risk by promising the triggers are
"monitored as SLO-class metrics" (line 123). Nothing monitors them. 779 `*.openslo.yaml` files
exist in-tree; zero cover any trigger. In the **63 days** since acceptance (2026-06-08), this
report appears to be the first time any of the four numbers has been taken.

**Honest conclusion: git + GitHub is the substrate for the next decade-plus, not the next
quarter.** Effort spent on the bespoke SCM destination (W4) is effort spent against no forcing
function. Effort spent on the *seam* — ADR-0526's `ScmFactsSource` adapter boundary — is cheap
and already done. Everything else belongs elsewhere.

## Scoreboard

| # | Trigger | Threshold | Measured today | % of threshold | Multiple needed to fire | Trend |
|---|---|---|---|---|---|---|
| 1 | Fresh full-clone wall-clock | > 600 s | **10.14 s** | 1.69 % | **59.2x** | flat/down |
| 2 | `.git` size | > 20 GB | **247 MiB** (fresh clone) | 1.21 % | **82.9x** | +3.2 MiB/day → ~17 yr |
| 3 | Working set (tracked files) | > 1,000,000 | **18,894** | 1.89 % | **52.9x** | **−21.7 % since authoring** |
| 4 | Commit-Status fan-out | > 50 writes/s @ p99 > 2 s | **0 commit statuses** | **unmeasurable** | n/a — surface is dead | n/a |

---

## 1. The four triggers, quoted exactly

Source: `docs/decisions/ADR-0510-scm-bespoke-hyperscaler-destination-cutover-trigger.md`
(status **Accepted**, `door: two-way`, `amended_by: [ADR-0518, ADR-0526]`).

Section §3 preamble, line 71:

> The GitHub→bespoke-VCS cutover is **deferred behind explicit numeric thresholds.** The cutover planning IP opens only when **any** of the following crosses its threshold on the production monorepo (measured, not estimated):

The table, lines 73–78:

**line 73:** `| Trigger metric | Threshold (cutover-planning opens when crossed) | Why this is the forcing-function |`

**line 75 — Trigger 1:**
> `| Fresh full-clone wall-clock (cold cache, representative agent runner) | **> 10 min** sustained | Clone latency is the first scale wall git hits; past ~10 min it taxes every cold lane bootstrap. |`

**line 76 — Trigger 2:**
> `| `.git` size | **> 20 GB** | Object-store size at which native partial-clone/sparse-index (ADR-0367/0369-adjacent scale-checkout work) stops being sufficient and server-side virtualization becomes the only lever. |`

**line 77 — Trigger 3:**
> `| Working set (tracked files) | **> 1,000,000 files** | The order where filesystem-level checkout and status operations need server-side virtual-FS / lazy materialization (Piper CitC / EdenFS class). |`

**line 78 — Trigger 4:**
> `| GitHub Commit-Status fan-out throughput | sustained **> 50 status writes/s** at p99 latency **> 2 s**, OR merge-gate status posting becomes the merge-train bottleneck | The server-side fan-out / reverse-dependency-index limit that single-node GitHub cannot scale through. |`

Line 80 governs what firing means:

> Crossing a trigger opens **planning + a build-vs-adopt re-evaluation**, not an automatic build

---

## 2. Trigger 1 — Fresh full-clone wall-clock

**Threshold: > 10 min (600 s) sustained.**

```
$ rm -rf .../scratchpad/fullclone
$ /usr/bin/time -p git clone https://github.com/jason931225/oyatie.git .../scratchpad/fullclone
Cloning into '.../scratchpad/fullclone'...
Updating files: 100% (18894/18894), done.
real 10.14
user 11.42
sys  3.33
```

**Measured: 10.14 s wall-clock.** Threshold is 600 s.

- **1.69 %** of threshold.
- **589.86 s of headroom.**
- Clone time must grow **59.2x** to fire.

For context, git version in use is 2.55.0 — well past the 2.37+ sparse-index/partial-clone floor
ADR-0510 §4 names as the interim bridge. That bridge has not even been *needed* yet; a plain full
clone finishes in ten seconds.

### Measurability: PARTIAL — three undefined terms (defect D4)

The trigger says "cold cache, representative agent runner … sustained". None of the three is
defined:

- **"cold cache"** — whose? The OS page cache on the runner, git's own object cache, or GitHub's
  CDN edge? My measurement was cold on a fresh empty target directory but the source repo is
  warm at GitHub's edge. A genuinely cold-everything clone is a different number.
- **"representative agent runner"** — this measurement is a macOS laptop on a home link. A
  GitHub-hosted `ubuntu-latest` runner in a different region, or the ADR-0515 owned Talos runner,
  gives a different number. Which one is authoritative is unstated. Since the result is network-
  bound at this size, the runner choice dominates the measurement.
- **"sustained"** — over how many samples, at what percentile, over what window? One 10-second
  clone is not "sustained"; neither is any number of them without a stated aggregation rule.

**What would make it measurable:** name the runner class (e.g. "the ADR-0515 `oya-ci-required`
runner image"), name the aggregation ("p95 over ≥20 clones in a rolling 7-day window"), and emit
it from the existing CI lane, which already performs a checkout on every run and could time it
for free.

---

## 3. Trigger 2 — `.git` size

**Threshold: > 20 GB.**

```
$ du -sh .../scratchpad/fullclone/.git
247M	.../scratchpad/fullclone/.git

$ gh api repos/jason931225/oyatie --jq '{size_kb:.size, created:.created_at}'
{"created":"2026-05-12T11:28:21Z","size_kb":233205}
```

**Measured: 247 MiB** (fresh full clone, single pack). GitHub's own server-side accounting agrees:
233,205 KB = **227.7 MiB**.

- **1.21 %** of threshold (247 MiB / 20,480 MiB).
- **20,233 MiB of headroom.**
- `.git` must grow **82.9x** to fire.

### Growth rate and time-to-fire

```
$ git rev-list -1 --before=2026-07-01 origin/dev
aa8956e1cbc898029a53ca0f323225bf1523aed4

$ git rev-list --objects origin/dev ^aa8956e1cbc898029a53ca0f323225bf1523aed4 \
    | cut -d' ' -f1 | git cat-file --batch-check='%(objectsize)' \
    | awk '/^[0-9]+$/{s+=$1; n++} END {printf "objects=%d raw_bytes=%.0f MiB=%.1f\n", n, s, s/1048576}'
objects=5167 raw_bytes=100691935 MiB=96.0
```

96.0 MiB of *raw, uncompressed* new object bytes in 30 days = **3.2 MiB/day**. That is a
deliberately conservative upper bound: packed and deltified, the real on-disk growth is a
fraction of it (the entire 1,682-commit history packs to 227 MiB).

At 3.2 MiB/day: **(20,480 − 247) / 3.2 = 6,323 days ≈ 17.3 years to fire.**

Cross-check against lifetime average: 227.7 MiB over the repo's 80-day life = 2.84 MiB/day →
**19.6 years**. Both estimates land in the same place.

### Measurability: AMBIGUOUS — a 4.3x spread (defect D5)

"`.git` size" does not name *which* `.git`, and the candidates differ by more than 4x:

```
$ du -sh /Users/jasonlee/Developer/oyatie/.git
975M	/Users/jasonlee/Developer/oyatie/.git

$ git count-objects -vH
count: 1644
size: 13.65 MiB
in-pack: 242253
packs: 5
size-pack: 428.22 MiB
garbage: 2
size-garbage: 167.00 MiB
```

Four defensible readings of the same repo on the same day:

| Reading | Value |
|---|---|
| Local working checkout, un-gc'd, 5 packs + 167 MiB garbage + worktree metadata | **975 MiB** |
| Local `size-pack` | **428 MiB** |
| Fresh full clone (server-packed, single pack) | **247 MiB** |
| GitHub API `repo.size` | **228 MiB** |

The 975 MiB local reading is 4.3x the 228 MiB server reading and reflects *local repo hygiene*,
not object-store growth. A trigger evaluated against the wrong one fires 4.3x early. This also
explains the ADR's own stale "~482M `.git`" ground truth (lines 55, 142) — that was almost
certainly an un-gc'd local `du`, not the object store.

**What would make it measurable:** name the source explicitly — "GitHub REST `repos/{repo}.size`"
is the only reading that is single-valued, server-authoritative, and free to poll.

---

## 4. Trigger 3 — Working set (tracked files)

**Threshold: > 1,000,000 files.**

```
$ git ls-tree -r --name-only origin/dev | wc -l
   18894
```

**Measured: 18,894 tracked files.**

- **1.89 %** of threshold.
- **981,106 files short.**
- Working set must grow **52.9x** to fire.

### The trend is NEGATIVE

```
$ git rev-list -1 --before=2026-05-30 origin/dev      # ADR-0510 authored 2026-05-29
d5ab0f80adc3d0085c11792047d3ed831d613baf
$ git ls-tree -r --name-only d5ab0f80adc3d0085c11792047d3ed831d613baf | wc -l
   24123

$ git rev-list -1 --before=2026-06-09 origin/dev      # ADR-0510 accepted 2026-06-08
812a83f6d2af16dd40b4a874c04ac94945a6659c
$ git ls-tree -r --name-only 812a83f6d2af16dd40b4a874c04ac94945a6659c | wc -l
   17790
```

| Date | Event | Tracked files |
|---|---|---|
| 2026-05-29 | ADR-0510 authored | 24,123 |
| 2026-06-08 | ADR-0510 accepted | 17,790 |
| 2026-07-31 | today | 18,894 |

**Net change since authoring: −5,229 files (−21.7 %).** The reorg, the markdown-retirement
policy, and the generated-artifact de-commit work are all pushing this number *down*. On the
current trend this trigger does not fire at any horizon.

Even under the repo's most aggressive historical growth — 0 → 24,123 files in the 17 days between
repo creation (2026-05-12) and the ADR (2026-05-29), i.e. 1,419 files/day — reaching 1,000,000
would take 691 days. That rate has since reversed sign.

### Measurability: CLEAN

This is the only trigger that is unambiguously measurable as written. One command, one number,
no undefined terms. It is also, by the ADR's own §5 verdict ("the trigger that most plausibly
forces cutover … not file count"), the *least* likely to fire.

### Note: the ADR's ground truth is stale

ADR-0510 cites "657 workspace members, ~23,164 git-tracked files, ~482M `.git`" (lines 55, 110,
142). Measured at the ADR's own authoring commit, the file count was 24,123, not 23,164. The
"657 workspace members" figure is no longer reproducible at all: `Cargo.toml` membership became
glob-based under ADR-0538, so there is no member array to count. The nearest live equivalent:

```
$ git ls-tree -r --name-only origin/dev | grep -c '/Cargo.toml$'
900
```

900 tracked crate manifests. None of this changes the verdict, but a trigger table whose
baseline figures cannot be reproduced is a trigger table nobody has re-run.

---

## 5. Trigger 4 — GitHub Commit-Status fan-out throughput

**Threshold: sustained > 50 status writes/s at p99 latency > 2 s, OR merge-gate status posting
becomes the merge-train bottleneck.**

### This trigger is NOT MEASURABLE AS WRITTEN. Three independent reasons.

#### D1 (Critical) — the named API surface is dead. Value is structurally pinned at zero.

The trigger measures the **GitHub Commit-Status API**. This repo writes to it zero times.

```
$ gh api repos/jason931225/oyatie/commits/502abb95131c2cbb6102c62836ac071fc57bba24/status \
    --jq '{state:.state, total:.total_count, contexts:[.statuses[].context]}'
{"contexts":[],"state":"pending","total":0}
```

Sampled across five recent merged-PR head SHAs (#1463, #1462, #1460, #1458, #1455) —
`total_count` is **0** on every one. Confirmed at the branch-protection layer:

```
$ gh api repos/jason931225/oyatie/branches/dev/protection \
    --jq '{required_checks:.required_status_checks.contexts, checks:.required_status_checks.checks}'
{"checks":[{"app_id":15368,"context":"oya-ci-required"}],"required_checks":["oya-ci-required"]}
```

`app_id: 15368` is GitHub Actions. The single ADR-0515 required context `oya-ci-required` is a
**Check Run**, not a Commit Status. 100 % of this repo's merge signal rides the Check Runs API.

A metric whose measured value is 0 and whose producer does not exist can never cross 50/s. As
written, **this trigger can never fire** — which is precisely the failure mode ADR-0510's own
Rejected-alternatives section was trying to prevent ("that is how a decided destination silently
disappears", line 112).

#### D2 (High) — the p99-latency conjunct has no producer.

The threshold is a conjunction: > 50 writes/s **at p99 latency > 2 s**. GitHub exposes no
per-write latency metric to API clients. The only way to obtain it is for the poster to time its
own API calls and emit a histogram. Nothing does:

```
$ git grep -rlniE "clone.time|clone_wall|status_writes|fan.out.throughput|cutover_trigger" \
    -- specs cloud/cloud-observability ci
specs/bespoke-scm-declare-observe-contract.json
```

That single hit is prose (`"git_and_github_remain_bridge_until_cutover_triggers": true`), not a
metric. Zero instrumentation exists for status-write latency anywhere in the repo.

#### D3 (High) — the second disjunct is qualitative, and self-contradictory with the ADR.

> "OR merge-gate status posting becomes the merge-train bottleneck"

There is no number in that clause. ADR-0510 line 112 rejects exactly this:

> **"Qualitative ("revisit later") cutover with no number"** — rejected: that is how a decided destination silently disappears. The trigger must be numeric and measured (§3).

The ADR rejects qualitative triggers in §Rejected-alternatives and then ships one in §3.

It is also unevaluable on two counts. First, **there is no merge train**:

```
$ gh api graphql -f query='{repository(owner:"jason931225",name:"oyatie"){mergeQueue(branch:"dev"){id}}}'
{"data":{"repository":{"mergeQueue":null}}}
```

`mergeQueue` is `null` on `dev`. ADR-0111's speculative merge-train is still deferred. A clause
gated on "the merge-train bottleneck" cannot be evaluated when no merge train exists.

Second, even if one existed, status posting is nowhere near the bottleneck — gate *execution* is,
by three orders of magnitude. On PR #1463's head SHA the `oya-ci-required` check-run was posted
in **2 seconds** (`started 09:48:07 → completed 09:48:09`) while the gate matrix it summarises ran
**09:35:42 → 09:44:13**. Across the whole fleet:

```
$ gh api "repos/jason931225/oyatie/actions/runs?per_page=100&created=>=2026-07-26&status=completed" \
    --paginate --jq '.workflow_runs[] | select(.name=="oya-ci-required")
       | ((.updated_at|fromdate)-(.run_started_at|fromdate))' \
  | sort -n | awk '{a[NR]=$1} END {print "n="NR, "min="a[1], "median="a[int(NR/2)+1], "max="a[NR]}'
n=261 min=10 median=1289 max=5244
```

261 runs, **median 1,289 s (21.5 min)**, **max 5,244 s (87 min)**. Status posting is ~0.15 % of
the median lane. The merge-lane bottleneck is buck2 + gate execution — a compute problem the
bespoke SCM does not solve.

### Measured anyway, using the Check Runs surface as the honest proxy

If the trigger is read charitably as "CI status-signal fan-out, whatever API carries it", it is
still nowhere close. Peak activity in the measured window was the hour `2026-07-26T09`
(31 merges that day — the busiest day in the last 100 PRs):

```
$ gh api "repos/jason931225/oyatie/actions/runs?per_page=100&created=2026-07-26" --paginate \
    --jq '.workflow_runs[] | .run_started_at' | cut -c1-13 | sort | uniq -c | sort -rn | head -3
  40 2026-07-26T09
  40 2026-07-26T07
  39 2026-07-26T08

$ xargs -I{} -P8 gh api "repos/jason931225/oyatie/actions/runs/{}/jobs?per_page=100" \
    --jq '.total_count' < peakruns.txt | sort -n | uniq -c
   2 0
  17 1
   2 11
  19 53
```

Peak hour: 40 workflow runs totalling **1,046 jobs** (2×0 + 17×1 + 2×11 + 19×53). One job = one
check-run; each check-run costs roughly 3 state writes (queued → in_progress → completed):

- ~3,138 status writes / 3,600 s = **0.87 writes/s sustained** — **1.7 % of the 50/s threshold,
  57x headroom.**
- Absurd worst case, collapsing all 1,046 check-run creations into a single 60-second burst:
  **17.4 writes/s** — still under 50, and not "sustained" by any reading.

### What would make Trigger 4 measurable

1. **Retarget the surface.** Replace "GitHub Commit-Status fan-out" with "CI status-signal
   writes (Check Runs + Commit Statuses combined)", so the metric tracks whatever API actually
   carries merge signal rather than one that has been dead since ADR-0515.
2. **Produce the latency.** Have the status/check-run poster time its own API calls and emit a
   p99 histogram, or delete the latency conjunct. A conjunction with an unproduced term is a
   condition that cannot evaluate to true.
3. **Numerify or delete the merge-train clause.** If the intent is "merge throughput is capped by
   forge-side status handling", express it as a number (e.g. "status-posting wall-clock > 20 % of
   median merge-lane wall-clock"). Today that ratio is **0.15 %**. As prose it is unevaluable and
   it contradicts line 112.

---

## 6. D6 (Critical, systemic) — nobody evaluates any of this

ADR-0510 line 123 names the mitigation for its own drift risk:

> Maintaining the destination intent without building it risks drift; mitigated by (a) **the numeric triggers being monitored as SLO-class metrics** and (b) every layered capability … being designed host-portable.

Half (b) shipped — the ADR-0526 `ScmFactsSource` seam and the ADR-0547 kernel-purity gate are
real. **Half (a) was never built.**

```
$ git ls-files | grep -c "openslo.yaml"
779
$ git grep -lniE "clone|scm|vcs" -- "*.openslo.yaml"
data/observability/slos/data-warehouse/zero-copy-clone-latency.openslo.yaml
oya/supply-chain-planning/slos/...
```

779 SLO definitions in-tree; the only "clone" hit is a data-warehouse zero-copy SLO, unrelated.
No SLO, no gate, no scheduled job, no dashboard reads clone wall-clock, `.git` size, tracked-file
count, or status-write throughput.

Every in-tree reference to ADR-0510 uses it as a *transient-tech marker* — the kernel-purity
denylist (`ci/facade/core-dependency-isolation/kernel-purity-policy.json`, 20 entries all tagged
`"cutover": "ADR-0510"`), Helm labels (`oyatie.com/adr-0510: transient-...`), masterplan prose.
Not one reads a trigger value.

This is the defect that matters most. An unmeasured trigger is indistinguishable from no trigger.
ADR-0510 was written specifically to avoid "revisit later" — and then landed with no evaluator, so
in practice it *is* "revisit later" with a table attached. 63 days accepted, 0 measurements.

**Cheapest fix:** the four numbers cost seconds to compute (this whole report is four `git`
commands and four `gh` calls). A weekly scheduled job that writes them to a JSON face, with the
gate failing only if a value crosses threshold, converts the ADR from aspiration to instrument.
Trigger 3 is already a one-liner; Trigger 1 could be timed for free by the checkout step the CI
lane already runs.

---

## 7. What the destination concretely IS, and when it is scheduled

### ADR-0518 — the destination (Accepted 2026-06-08, `door: one-way`, `milestone: W4`)

`docs/decisions/ADR-0518-bespoke-scm-ast-work-area-change-pipeline.md` defines ADR-0510's
deferred destination concretely (line 43–51):

> The bespoke SCM destination is a **10-stage hyperscaler change pipeline**:
> DECLARE → ADMIT → LEASE → ISOLATE(virtual) → AUTHOR → GATE(buck2 + AST gates + auto-remediate) → ATTEST → INTEGRATE → PROPAGATE(CD) → OBSERVE.
> It is Sapling / Mononoke / EdenFS / CommitCloud-inspired, owned in Rust, and **native-only**. … Concurrency is **leases-not-locks**, sharded, with no single leader. Work-area identity is the content-addressed AST hash (ADR-0517).

Its own Alternatives-considered section (line 69) rejects building it now:

> **Building it now** — rejected: deferred to W4, cutover-gated per ADR-0510's numeric triggers.

Current state is a metadata-only contract projection at
`specs/bespoke-scm-declare-observe-contract.json`, whose line 286 restates the gate:
`"W4 native cutover remains blocked unless ADR-0510 numeric triggers are separately green"`.

### ADR-0521 — the schedule (Accepted 2026-06-08, `door: one-way`, `milestone: W1`)

`docs/decisions/ADR-0521-staged-w0-w6-fabric-roadmap.md` places it at W4 of a W0–W6 roadmap
(lines 41–56):

- **W0 (DONE)** — hermetic buck2, firewall live on `dev`, JSON SSOT stores.
- **W1 (NEXT)** — convergence, dev-advance, interface locking. **This is where the repo is today.**
- **W2 (high priority)** — owned AST parser behind `WorkAreaTree`, AST gates, auto-remediation fleet.
- **W3** — de-oyatie config, gate/plugin SDK, `oya new`, buck2 RBE/NativeLink.
- **W4 (cutover-gated)** — "the bespoke SCM work-area change pipeline + virtual materialization + Mononoke-essence Rust server + cloud-backed ChangeUnits + segmented changelog + leases."
- W5, W6 — far-term.

Line 71: *"W4 is explicitly GATED by ADR-0510's numeric cutover triggers."*

**W4 sits three full waves out AND behind a gate that reads 1.2–1.9 % on every measurable
trigger.** There is no date attached to W4 anywhere. Line 66 rejects reordering it forward:
*"Build-substrates-first — rejected: no forcing function, fails the honest-cost test."*

---

## 8. Assessment

**Is cutover near, far, or indefinite? — INDEFINITE.**

Not "far" in the sense of a distant date. There is no date. There is a gate, the gate reads
~1.5 % across the board, and one of its four conditions is structurally incapable of evaluating
to true.

**Is any trigger already met? — NO. None. Not close.**

The nearest any trigger comes to firing is Trigger 2 at 1.21 %, and its own growth curve puts it
**17 years out**. The smallest multiple required to fire any trigger is **52.9x** (Trigger 3),
and that one is trending backwards.

**Is the cutover getting nearer? — On two of four axes, no. It is receding.**

- Tracked files: **−21.7 %** since the ADR was authored.
- Repo pressure generally: the reorg, markdown-retirement, and generated-artifact de-commit
  programs are all net-deletion programs.
- Clone time and `.git` size grow slowly and are the same physical quantity; both project to
  ~17–20 years.
- Status fan-out at 0.87 writes/s peak-hour has ~57x headroom, and the actual merge-lane
  bottleneck (median 21.5 min, max 87 min of gate execution) is a **compute** problem the
  bespoke SCM does not address at all.

**What this means for effort allocation.**

ADR-0510 was correct about the *decision* (record the destination, gate on numbers) and correct
about the *near term* (KEEP GitHub, line 105). What it got wrong is calibration and
instrumentation:

1. **The thresholds are so distant they are decorative.** 50x–83x headroom on a repo whose file
   count is shrinking is not a trigger, it is a footnote. The ADR itself anticipates this — line
   80 says the numbers "are re-ratified (not silently changed) in the cutover-planning IP when it
   opens" — but a threshold that cannot plausibly be crossed inside a decade never opens that IP,
   so it never gets re-ratified either. The re-ratification path is dead code.
2. **Trigger 4 — the one the ADR itself calls "the load-bearing destination capability" and "the
   trigger that most plausibly forces cutover" (line 101, line 105) — is the one that cannot
   fire.** It points at an API this repo abandoned. The single most important cutover condition
   in the document is inert. That is the finding to act on.
3. **Nothing measures anything (D6).** Fixing #2 without fixing D6 just moves an unread number.

Concretely: **git + GitHub remains the SCM substrate for the foreseeable future — a decade-plus
on the measured curves, not a horizon worth planning against.** W4 is correctly deferred and
should stay deferred. The right investment is the *seam*, not the destination — ADR-0526's
`ScmFactsSource` adapter boundary, which already reduces the cutover to a single impl-swap, is
the whole hedge and it is already paid for.

The one piece of work ADR-0510 justifies today is small and cheap: make the four triggers
**measurable and measured** — retarget Trigger 4 at the Check Runs surface (or delete it as
written), produce or drop its latency conjunct, numerify or drop its merge-train clause, define
Trigger 1's runner/aggregation and Trigger 2's `.git` source, and emit all four weekly. That is
a day of work and it converts a decorative table into an instrument. Anything larger aimed at the
bespoke SCM is effort against a forcing function that, on today's numbers, does not exist.

---

## Appendix — measurement provenance

| Item | Value | Source |
|---|---|---|
| Base commit | `953d4a49b` | `git log --oneline -1 origin/dev` |
| Repo | `jason931225/oyatie` (private) | `gh api repos/jason931225/oyatie` |
| Repo created | 2026-05-12T11:28:21Z | same |
| Total commits | 1,682 | `git rev-list --count origin/dev` |
| Tracked files | 18,894 | `git ls-tree -r --name-only origin/dev \| wc -l` |
| Tracked crate manifests | 900 | `git ls-tree -r --name-only origin/dev \| grep -c '/Cargo.toml$'` |
| git version | 2.55.0 | `git --version` |
| Fresh clone wall-clock | 10.14 s | `/usr/bin/time -p git clone …` |
| Fresh clone `.git` | 247 MiB | `du -sh …/fullclone/.git` |
| GitHub `repo.size` | 233,205 KB | `gh api repos/… --jq .size` |
| 30-day raw object growth | 96.0 MiB / 5,167 objects | `git rev-list --objects … \| git cat-file --batch-check` |
| Commit statuses (5 head SHAs) | 0, 0, 0, 0, 0 | `gh api repos/…/commits/{sha}/status` |
| Required context | `oya-ci-required`, app_id 15368 | `gh api repos/…/branches/dev/protection` |
| Merge queue on `dev` | `null` | `gh api graphql … mergeQueue(branch:"dev")` |
| Peak hour workflow runs | 40 (`2026-07-26T09`) | `gh api repos/…/actions/runs?created=2026-07-26` |
| Peak hour jobs | 1,046 | `gh api repos/…/actions/runs/{id}/jobs` × 40 |
| `oya-ci-required` durations | n=261, median 1,289 s, max 5,244 s | `gh api repos/…/actions/runs?created=>=2026-07-26` |
| SLO files in-tree | 779 | `git ls-files \| grep -c openslo.yaml` |
| SLOs covering a trigger | 0 | `git grep -lniE … -- "*.openslo.yaml"` |
