# Sapling / Mononoke as an IMPLEMENTATION of the ADR-0518 destination

Read-only evaluation. Date: 2026-07-31. Branch: `docs/sapling-mononoke-vs-adr-0518-destination`.

Question asked: ADR-0518 is Accepted and one-way — it already *defines* the destination. This is not
"Sapling instead of ADR-0518". It is: **can Sapling/Mononoke implement ADR-0518, or parts of it?**

---

## Verdict

| Question | Verdict |
|---|---|
| **Mononoke as the ADR-0518 server** | **REJECT.** Not a usable hosted substrate outside Meta, and its change identity contradicts ADR-0517 at the root. |
| **Sapling `sl` as a client** | **ADOPT-OPTIONAL — but it buys nothing.** Permissible as local developer ergonomics on the existing GitHub bridge. Discharges zero ADR-0518 requirements and does not replace `oya-stack` (ADR-0369). |
| **Design harvest** | **HARVEST — substantial and cheap.** Four specific patterns worth copying as design, listed in §7. This is where the real value is. |

The two rejections have *independent* causes. Either one alone is sufficient:

1. **Operability.** The maintainers state plainly that there is no supported external workflow, the
   primary production blobstore is a Facebook-internal service, and the released binaries are the
   client only. A client-only tool cannot be an owned hosted substrate.
2. **Identity.** Mononoke's `ChangesetId` is Blake2b over the *whole serialized changeset* — author,
   author_date, committer, message, parent ids, file changes. ADR-0517 / the work-area-content-hash
   contract require the change id to be a pure function of content and explicitly **exclude** author
   names, author timestamps, and moving-HEAD data. These are irreconcilable. Not "hard" —
   irreconcilable, because Mononoke's id *is* the join key for every table in the system.

Honest counterweight, stated up front so the verdict is not read as stronger than it is: **git has the
identical identity defect.** A git commit SHA also covers author, committer, timestamps and parents.
ADR-0526 already solves this by demoting the VCS revision id to *provenance evidence*
(`scm_facts_ref`) rather than the change id. So adopting Sapling would not make the content-addressing
situation worse than today — it would simply not improve it, while costing us a substrate we cannot
operate. **Zero gain on the decisive axis, large cost on the operability axis.** That is what makes
this a clean no rather than a close call.

---

## 1. What ADR-0518 actually requires

From ADR-0518 (Accepted 2026-06-08, door:one-way), ADR-0517, ADR-0520, ADR-0526 and
`specs/bespoke-scm-declare-observe-contract.json` / `specs/work-area-content-hash-contract.json`:

| # | Requirement | Source |
|---|---|---|
| R1 | 10 ordered stages DECLARE→ADMIT→LEASE→ISOLATE→AUTHOR→GATE→ATTEST→INTEGRATE→PROPAGATE→OBSERVE, each a typed envelope with handoff invariants | ADR-0518; declare-observe contract `stage_order` |
| R2 | **Native-only.** No git-overlay. The claim/work/done model is native pipeline stages | ADR-0518 Decision + rejected alternative "standalone grit-style git-overlay tool" |
| R3 | **Leases-not-locks**, sharded, **no single leader** | ADR-0518 Decision; `LeaseRef` forbids `global_lock`, `unbounded_lock`, `leader_only_mutex` |
| R4 | **One content-addressed work-area hash** = SCM change id = buck2/RBE cache key = CD artifact hash, byte-identical | ADR-0517; WAH-ALIAS-001 |
| R5 | The hash **excludes** author names/emails, author timestamps, and current HEAD when it advances without content change | `work-area-content-hash-contract.json` → `excluded_inputs` |
| R6 | Owned, cloud-native, infinite-scale, **transitional-impl-behind-a-stable-interface**, self-hostable end to end | ADR-0520 |
| R7 | Swap-in through the `ScmFactsSource` seam: `tracked_paths()`, `last_touch()`, `revision_author_timestamps()` — one impl swap, zero churn to producer/gates/snapshot | ADR-0526; `ci/facade/scm-facts-snapshot/src/lib.rs:3578` |
| R8 | Work-area identity is the AST-derived `WorkAreaTree` root, not a file-tree hash | ADR-0517; `canonical_frame_inputs.work_area_tree_root` |

R3, R4, R5 and R8 are the load-bearing ones. R1/R2 are shape.

---

## 2. THE CRUX — is Mononoke self-hostable? No.

I looked for the strongest possible *positive* evidence — build docs, published images, k8s manifests,
third-party production reports — and applied a differently-shaped probe to each negative before
recording it.

### 2.1 The maintainers say so, in writing

`facebook/sapling` README, on both Mononoke and EdenFS:

> "While it is used in production within Meta it is not yet supported for external usage. OSS builds
> in GitHub Actions are available for unsupported experimentation."

Mononoke's own README ("Caveat Emptor"):

> "Mononoke is still in development. … The version that we provide on GitHub omits some functions.
> This is because the code is exported verbatim from an internal repository at Facebook, and not all
> of the scaffolding from our internal repository can be easily extracted."

Named omissions: Thrift API serving, MySQL failover, CacheLib.

### 2.2 A Meta maintainer answering the direct question

`facebook/sapling` issue #812, "Add docs for building and running the oss build of Mononoke".
`markbt` (core maintainer), 2024-01-15:

> "Unfortunately we don't currently have a supported workflow for running Mononoke externally. As a
> scalable server, Mononoke still has some strong dependencies on internal Meta infrastructure in
> order to provide that scalability. Many of these can be stubbed out for testing purposes, but the
> stubs are not necessarily fully featured. **For now, the code is mostly available for information
> only.**"

Same repo, issue #922, `mzr` (Meta) 2024-08-23:

> "The builds of Mononoke aren't as well supported as builds of Sapling at this time. I don't recall
> we tried full setup of EdenFS, Mononoke and Sapling together outside of the internal use at Meta"

`ahornby` (Meta) 2024-08-28, on getting it up:

> "you'll need to build your own binaries … I think mononoke is **quite likely to start** if you
> manage to decipher the mysql and s3 bucket configs"

"Quite likely to start" is not a substrate posture.

### 2.3 The issue is still open, 2.5 years later

Verified via the GitHub API, this session:

```
$ gh api repos/facebook/sapling/issues/812 --jq '{state, created_at, updated_at, comments}'
{"comments":3,"created_at":"2024-01-12T02:22:24Z","state":"open","updated_at":"2026-04-22T09:24:28Z"}
```

The last comment, `ericdog`, **2026-04-22**:

> "Hey Matthew 👋 I'm taking a look too, do you have an update of the situation ? Or is there still a
> missing component somewhere that prevents us to have a working mononoke server ?"

Two independent people, two and a half years apart, still cannot start the server. Nobody answered.
The original reporter's failure was a hard panic:

> `PANIC: not implemented: This is implemented only for fbcode_build!`

…in the config store. `markbt` identified the missing component as **Configerator**, Meta's internal
distributed configuration service.

### 2.4 No server binaries are published

Verified via the GitHub API, this session — the three most recent releases:

```
2026-05-22  sapling-…-linux-arm64.tar.xz, …-linux-x64.tar.xz, …-windows-x64.zip, …arm64_sequoia.bottle.tar.gz
2026-03-18  (same four client artifacts)
2025-05-21  (same four client artifacts)
```

Every published asset is the `sl` **client**. Zero server artifacts, ever. Issue #922 ("Mononoke
binaries aren't actually published") was closed in 2024 without ever publishing them. No official
container image, no Helm chart, no k8s manifest exists in the repo or anywhere I could find.

### 2.5 The in-repo docs that *do* exist are internal docs

There *is* now a documentation set at `eden/mononoke/docs/` — 27 files including
`local-dev-testing.md`. I checked it specifically because "no docs" is exactly the kind of negative
that decays. It does not change the answer:

- `local-dev-testing.md` requires a Meta **devserver**, uses `mononoke-bootstrap` scripts, and links
  to `https://www.internalfb.com/wiki/Source_Control/Mononoke/Development/Running_Locally_for_Development`
  for the parts it does not cover. It is Meta's internal onboarding doc exported verbatim — a
  *local ephemeral test* recipe, not an external deployment guide.
- `1.3-architecture-overview.md` names **Manifoldblob** as the "Primary production backend
  (Facebook-internal)", plus Configerator, Memcache and CacheLib.
- `3.1-servers-and-services.md` marks **Bookmark Service**, **Diff Service** and **Load Limiter
  Service** as Facebook-internal (they live under `facebook/` subdirs in-tree).

So the production storage backend, the config plane, the cache tier, the bookmark service and the
load limiter are all internal. What is open is the *code shape*, not a runnable system.

### 2.6 Nobody outside Meta runs it

I found **zero** third-party production reports, zero hosted offerings, zero community deployment
guides. Corroborating negative from a differently-shaped probe: Uber's **GitFarm** paper (arXiv
2604.11977, 2026) describes a 40+ GB Go monorepo with ~15-minute clones — comfortably past ADR-0510's
`.git > 20 GB` numeric trigger — and Uber built a gRPC service layer over *plain git* rather than
adopt Mononoke, noting these systems are "proprietary, tightly integrated with internal tooling, and
not designed to expose Git-compatible interfaces for general-purpose automation workloads."

A company that crossed our own cutover trigger looked at this option and did not take it.

### 2.7 Plainly stated

**Mononoke is a client-only project from the outside.** A client-only tool cannot be an owned hosted
substrate. Against R6 (ADR-0520: owned, self-hostable, transitional-impl-*behind*-a-stable-interface)
this fails before any capability comparison starts. ADR-0510's 2026-05-29 finding — "Mononoke,
Meta-internal, explicitly not supported for external use" — **re-verified 2026-07-31 and still
accurate**, now with the maintainer quote and the still-open issue to back it.

---

## 3. Capability map against ADR-0518

| Req | Sapling/Mononoke | Verdict |
|---|---|---|
| **R1** 10 typed stages | Mononoke covers roughly AUTHOR (client), ISOLATE (EdenFS), INTEGRATE (Land Service / pushrebase). DECLARE, ADMIT, LEASE, GATE, ATTEST, PROPAGATE, OBSERVE have no counterpart — at Meta those are separate systems (Phabricator/Diff, SandcastleCI, Conveyor). | **~3 of 10.** Partial. |
| **R2** native-only, no git overlay | Mononoke is genuinely native (Bonsai is the canonical model; git and hg are *derived* via `bonsai_git_mapping` / `bonsai_hg_mapping`). This is the one requirement Mononoke satisfies **elegantly** — and it is a pattern worth stealing (§7.1). | **PASS (design).** |
| **R3** leases-not-locks, sharded, no single leader | `1.3-architecture-overview.md`: "Operations requiring mutual exclusion (e.g., landing to the same bookmark) are coordinated by **routing through a single service instance**." `3.1-servers-and-services.md`: the Land Service "queues landing requests per bookmark and **processes them serially**." That is a per-bookmark serialization point, i.e. a lock with a queue in front of it — the `LeaseRef` contract explicitly forbids `leader_only_mutex`. | **FAIL as written** — see the honest caveat below. |
| **R4/R5** content-only change id | Blake2b over the full Thrift-serialized changeset incl. author, dates, message, parents. | **FAIL. Decisive.** §4. |
| **R6** ownable/self-hostable | §2. | **FAIL. Decisive.** |
| **R7** `ScmFactsSource` impl | A `SaplingCliScmFactsSource` shelling `sl` is trivially writable — and is a lateral move from `GitCliScmFactsSource`, still a CLI shell-out, so it also fails ADR-0523's zero-shell posture. Zero gain. | **Possible, pointless.** |
| **R8** AST-derived work-area identity | Mononoke's granularity is file/tree. No AST layer, no sub-file node identity. This is the `WorkAreaTree` job (ADR-0517/0520) and no VCS supplies it. | **N/A — out of scope for any VCS.** |

**Honest caveat on R3.** Mononoke's per-bookmark serial land is the *battle-tested* design; ADR-0518's
"sharded, no single leader" is the *unproven* one. Mononoke's README states the system is "meant to
scale up to accepting thousands of commits every hour across millions of files" — a stated design
target, not a measured figure I could verify, but it is the target Meta runs this design against.
If R3 ever proves to be the expensive requirement, Mononoke's Land Service is the evidence that
per-target serialization with a queue in front is sufficient at a scale well beyond ours — and that
the serialization point can be moved off the frontend so servers don't "compete to move the same
bookmark". Record that as a finding against R3, not as a point against Mononoke.

---

## 4. Content-addressing — the decisive technical question

**Question:** can Sapling accept an externally-computed content id as the change id?

**Answer: no.** Its id is history-derived, in both the client and the server, by construction.

### 4.1 Server — Mononoke `ChangesetId`

Source, `eden/mononoke/mononoke_types/src/bonsai_changeset.rs`:

```rust
fn changeset_id(&self) -> ChangesetId {
    let thrift = self.clone().into_thrift();
    let data = compact_protocol::serialize(&thrift);
    let mut context = ChangesetIdContext::new();
    context.update(&data);
    context.finish()
}
```

`BonsaiChangesetMut` fields fed into that serialization:

```
parents, author, author_date, committer, committer_date, message,
hg_extra, git_extra_headers, file_changes, is_snapshot,
git_tree_hash, git_annotated_tag, subtree_changes
```

Independently confirmed by Mononoke's own `docs/2.1-bonsai-data-model.md`:

> "A Bonsai changeset is serialized (using Thrift compact protocol) and hashed with Blake2b. The
> resulting hash becomes the changeset identifier (`ChangesetId`). This identifier depends on: All
> metadata fields (author, dates, message, etc.), Parent changeset hashes, All file changes (paths,
> content identifiers, types), Extra fields."

Line up against `work-area-content-hash-contract.json` `excluded_inputs`:

| Contract says EXCLUDE | Mononoke `ChangesetId` |
|---|---|
| author names, author emails | **included** (`author`, `committer`) |
| author timestamps | **included** (`author_date`, `committer_date`) |
| current HEAD when it advances without content change | **included** transitively (`parents`) |

Three direct violations. Mononoke calls its model "content-addressed", and it genuinely is — but it is
content-addressed over the *changeset record*, which is a history node. ADR-0517 needs content
addressing over the *work area*, which is a content frame. **Same words, different objects.** That
distinction is the whole answer.

### 4.2 Can the wa-hash be injected?

Mechanically you could write `wa1:sha256:…` into `hg_extra` or `git_extra_headers` (Gerrit's
`Change-Id` trailer, essentially). Rejected:

- **WAH-ALIAS-001** requires `scm_change_id` to be the *byte-identical* string. An extra field is a
  carried label, not the id.
- Mononoke's `ChangesetId` remains the primary key of `bonsai_git_mapping`, `bonsai_hg_mapping`, the
  commit-graph index, bookmarks, derived-data keys and every blobstore key
  (`changeset.blake2.{hash}`). You would run two identifier spaces plus a mapping table — precisely
  the divergent-identifier drift ADR-0517 exists to prevent (its rejected alternative: "separate
  per-consumer parsers — drift across consumers, no shared identity, no single hash").
- Writing the wa-hash into extras changes the `ChangesetId`, so the two ids can never coincide even
  by accident.

### 4.3 Client — `sl`

No better, and for the same reason. `sl` operates in two modes
(`sapling-scm.com/docs/git/git_support_modes/`): `.git` mode produces **git commit SHAs**
(author + committer + timestamps + parents), `.sl` mode produces **Mercurial-style changeset hashes**
(also author + date + parents). Sapling's mutation tracking records predecessor links across amends,
but that is a lineage edge, not a content id. There is no knob to supply your own.

### 4.4 The honest reframe

This is not a Sapling defect — it is a property of every parent-chained VCS, git included. The
content-addressed-change-id requirement (R4/R5) is **not satisfiable by adopting any existing VCS**;
it is satisfiable only by computing the hash *outside* the VCS from a canonical content frame, which
is exactly what `work-area-content-hash-contract.json` specifies and what ADR-0526's `scm_facts_ref`
demotion already accommodates.

Consequence for the destination: R4/R5 is an argument for the **thin owned layer**, not against
Sapling specifically. The layer that computes the wa-hash cannot be borrowed from anyone. Adjacent
data point: Epic Games' **Lore** (`epicgames.github.io/lore`) does key its immutable store on BLAKE3
and identify a revision by "the hash of its serialized state" — closer in spirit, and worth a separate
30-minute look — but a Lore revision is still a hash-chained snapshot, so it too is history-derived.

---

## 5. Client vs server — different questions, different answers

They separate cleanly, and the split matters.

**Server (Mononoke): reject.** §2 and §4.1. Not operable, wrong identity model, GPL-2.0 (see §6).

**Client (`sl`): permissible, valueless for governance.** `sl` is the actively supported component,
builds on Linux/macOS/Windows, published binaries as recently as 2026-05-22, and is git-compatible —
you can `sl clone` a GitHub repo and get stacked commits + ISL today. But:

- It emits git commits to the same GitHub bridge, so it changes **nothing** in the pipeline —
  no stage, no envelope, no gate, no evidence bundle.
- It does not replace `oya-stack` (ADR-0369): both produce GitHub PR chains, and `oya-stack` is
  already the owned wedge.
- Sapling's own docs warn that mixing `sl` and `git` commands in a `.git` repo "might not work in all
  cases" — a real friction for agents that shell git today.
- **The `.sl` mode features that would actually be interesting — the Sapling network protocol, lazy
  commit graph, EdenFS virtual working copy — all require Mononoke.** They are gated behind exactly
  the component we cannot run. The client's scale story is a *server* story.

Net: an individual developer may use `sl` locally if they like it. It is a personal-preference tool
here, not a platform decision, and it should not appear in any ADR-0518 lane.

---

## 6. Licensing

`facebook/sapling` is **GPL-2.0** (verified: `gh api repos/facebook/sapling --jq .license.spdx_id` →
`GPL-2.0`; Mononoke source headers carry "GNU General Public License version 2"). Against the
transient-stack selection bar (MIT/Apache behind an owned destination port), GPL-2.0 rules out
lifting code into the owned Rust stack. **Reading the design is free; copying the code is not.**
This is a further independent reason the verdict can only be "harvest ideas", never "vendor it".

---

## 7. What to harvest — this is where the value is

Four patterns. All are design-level, all are cheap, none require running anything.

### 7.1 Bonsai as the canonical model, everything else *derived* — HIGH value

Mononoke's canonical change record is Bonsai. Git and Mercurial representations are **derived data**
with bidirectional mapping tables (`bonsai_git_mapping`, `bonsai_hg_mapping`), computed lazily by a
Derived Data Service.

This is a direct, ready-made shape for the ADR-0526 `ScmFactsSource` seam and the eventual cutover.
The canonical record is ours (the work-area content frame); git is one *derived* projection with a
mapping table; the cutover becomes "stop deriving git" rather than "migrate". Meta ran the
git↔hg↔bonsai triple simultaneously in production — that is the parallel-run bridge discipline
ADR-0482/ADR-0520 asks for, with an existence proof.

### 7.2 Two-tier storage split: immutable content-addressed blobstore + mutable SQL metadata — HIGH value

`docs/1.3-architecture-overview.md`: immutable blobstore (file content, changesets, derived data —
keyed `<type>.blake2.<hash>`) and a separate mutable SQL metadata DB (bookmarks, mappings,
commit-graph index). Multiplexed blobstore: "Mononoke writes to multiple independent blobstores
simultaneously. If one backend is down, reads succeed from others." (Further detail lives in
`docs/2.4-storage-architecture.md`, not read for this evaluation.)

This maps **exactly** onto ADR-0520's declared substrate pair — bespoke distributed-SQL DB (metadata)
+ bespoke infinite-scale object-store (content). It is independent evidence that ADR-0520 drew the
line in the right place, and the multiplexed-blobstore availability trick is worth copying into the
`object-store-kernel` interface.

### 7.3 Land Service — move serialization off the frontend — MEDIUM value

Frontends do not pushrebase locally; they forward to a Land Service that queues per bookmark and
processes serially, "preventing wasted work from failed pushrebases where servers compete to move the
same bookmark". Directly relevant to the ADR-0518 INTEGRATE stage and to the "near-zero wasted work"
driver. It is also the honest counter-evidence to R3's "no single leader" (§3).

### 7.4 Stateless frontend tiers, independently scaled — MEDIUM value

Mononoke Server / Git Server / LFS Server / SCS Server are stateless and horizontally scalable;
Land Service, Derived Data Service, Bookmark Service are separate internal microservices. "Each tier
can be scaled independently." A clean decomposition template for the DECLARE→OBSERVE services, and
consistent with ADR-0131/ADR-0132 single-concern flat services.

---

## 8. The Piper lesson, applied

The framing question was: Piper is **not** a bespoke storage engine — it is a *thin VCS layer over
standard infrastructure*. The CACM paper reports Piper as implemented on top of standard Google
infrastructure — originally Bigtable, now Spanner — distributed over 10 Google data centres and
relying on Paxos for consistency across replicas, serving 1B files / 86 TB / 35M commits, with CitC
as a cloud-based storage backend plus a Linux-only FUSE filesystem. (cacm.acm.org and dl.acm.org both
returned HTTP 403 to automated fetch this session; the wording above is a paraphrase corroborated
across two independent secondary summaries, not a verbatim quote from the primary.) So "build owned
SCM" plausibly means a thin layer over owned primitives.

Assessed in that light, the conclusion sharpens rather than softens:

- **Piper is the closer precedent, and ADR-0520 already has its shape.** Piper : Spanner ::
  ADR-0518 fabric : (owned distributed-SQL DB + owned object store). ADR-0520 names exactly that pair.
  The Piper lesson **validates ADR-0520's layering**, and says the SCM layer should be thin.
- **Mononoke is the *opposite* of the Piper lesson.** It is a fat, multi-service system with its own
  derived-data engine, multiplexed blobstore, land service and bookmark service, bolted to
  Manifold/Configerator/CacheLib. That fatness is precisely why it does not travel.
- Corroboration from a third direction: Uber's GitFarm (2026) — thin gRPC service layer over plain
  git, over a 40+ GB monorepo. Thin layers travel; fat platforms do not.
- **Therefore the harvest in §7.1–7.2 is the *right* harvest**: it is exactly the thin/thick seam —
  take the canonical-record + derived-projections model and the two-tier storage split (thin,
  portable, design-only), leave the internal-infra-coupled service fleet.

The one thing nobody can hand us is R4/R5 — the content-only work-area hash. That is the genuinely
novel bit of ADR-0517, no existing VCS has it, and it must be computed above the VCS regardless of
which VCS sits below. Which is what ADR-0526 already arranged.

---

## 9. Recommendation

1. **Do not open a Mononoke adoption lane.** Verdict: reject-with-reasons (§2 operability, §4.1
   identity, §6 licensing — three independent sufficient causes).
2. **No ADR change is needed.** ADR-0510's rejected alternative already reads "EdenFS/Mononoke
   (externally unsupported)". This document is the re-verification, dated 2026-07-31, with the
   maintainer quote and the still-open issue. Attach it as evidence; it costs nothing and closes the
   question durably.
3. **`sl` is a personal tool, not a platform decision.** No lane, no gate, no ADR-0518 role.
4. **Do open a design-harvest note** for §7.1 (canonical record + derived projections) and §7.2
   (two-tier storage split) against the `ScmFactsSource` / `object-store-kernel` seams. Both are
   free — they are design reading, not dependencies, and §7.1 in particular is the cheapest available
   de-risking of the ADR-0510 cutover.
5. **Record §3's R3 caveat.** Mononoke's per-bookmark serial Land Service is production evidence that
   the "no single leader" constraint may be more expensive than it is worth at INTEGRATE. That is a
   question for the ADR-0518 implementation lane, not a reason to touch the Accepted ADR now.

---

## Sources

Primary, all re-verified 2026-07-31.

- [facebook/sapling README](https://github.com/facebook/sapling) — Mononoke/EdenFS "not yet supported for external usage"; GPL-2.0; 7.0k stars; last push 2026-07-31.
- [eden/mononoke/README.md](https://github.com/facebook/sapling/blob/main/eden/mononoke/README.md) — "Caveat Emptor"; omitted Thrift APIs / MySQL failover / CacheLib.
- [Issue #812 — Add docs for building and running the oss build of Mononoke](https://github.com/facebook/sapling/issues/812) — `markbt` "no supported workflow for running Mononoke externally … mostly available for information only"; `fbcode_build` panic; Configerator; **still open**, last activity 2026-04-22.
- [Issue #922 — Mononoke binaries aren't actually published](https://github.com/facebook/sapling/issues/922) — `mzr`, `ahornby`.
- [facebook/sapling releases](https://github.com/facebook/sapling/releases) — client-only assets, latest 2026-05-22 (via GitHub API).
- `eden/mononoke/mononoke_types/src/bonsai_changeset.rs` — `changeset_id()`, `BonsaiChangesetMut` fields.
- `eden/mononoke/mononoke_types/src/typed_hash.rs` — Blake2, `changeset.blake2.{hash}` keys, GPL-2 header.
- `eden/mononoke/docs/` — `local-dev-testing.md` (devserver + internalfb.com wiki), `1.3-architecture-overview.md` (Manifoldblob "Facebook-internal"; single-instance mutual exclusion), `2.1-bonsai-data-model.md` (id derivation), `3.1-servers-and-services.md` (Land Service; internal services).
- [Sapling — Git support modes](https://sapling-scm.com/docs/git/git_support_modes/) — `.git` vs `.sl`; EdenFS + Sapling protocol require Mononoke.
- [Sapling — Scale overview](https://sapling-scm.com/docs/scale/overview/) — client/server split, lazy commit graph.
- [Why Google Stores Billions of Lines of Code in a Single Repository (CACM 2016)](https://cacm.acm.org/research/why-google-stores-billions-of-lines-of-code-in-a-single-repository/) — Piper on Bigtable→Spanner, 10 DCs, Paxos; CitC FUSE; 1B files / 86 TB / 35M commits.
- [GitFarm: Git as a Service for Large-Scale Monorepos (Uber, arXiv 2604.11977)](https://arxiv.org/html/2604.11977v1) — 40+ GB monorepo, thin gRPC layer over plain git; on Piper/Mononoke: "proprietary, tightly integrated with internal tooling".
- [Epic Games — Lore system design](https://epicgames.github.io/lore/explanation/system-design/) — BLAKE3 content-addressed store; revision = "hash of its serialized state".
- [Branching in a Sapling Monorepo (Meta, 2025-10-16)](https://engineering.fb.com/2025/10/16/developer-tools/branching-in-a-sapling-monorepo/) — `sl subtree copy`; directory branches appear as linear commits.

In-repo: `docs/decisions/ADR-0510`, `ADR-0517`, `ADR-0518`, `ADR-0520`, `ADR-0526`;
`specs/work-area-content-hash-contract.json`; `specs/bespoke-scm-declare-observe-contract.json`;
`ci/facade/scm-facts-snapshot/src/lib.rs:3578` (`ScmFactsSource`).

Negative results recorded honestly: no third-party Mononoke production report, no official container
image, no Helm chart, no k8s manifest, no hosted offering found. Each negative was probed a second
time with a differently-shaped query (issue search, release-asset enumeration, in-repo docs listing,
adjacent-company paper) before being recorded. Note for future searchers: `sapling.ai` is an unrelated
writing-assistant product and is not evidence of anything here.
