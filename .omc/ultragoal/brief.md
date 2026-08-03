# Ultragoal: FD-001 Enterprise SaaS — vertical core + unified shell + full cloud substrate, dogfooded

**Authored:** 2026-06-09 · **Base:** `dev` @ `8acec8920` · **Mode:** aggregate /goal, parallel lanes after contract lock

## Mission (founder, 2026-06-09)
Ship production-ready, industry-leading, trailblazing Enterprise SaaS: the FIRST VERTICAL MODULE (FD-001 tenancy+RBAC core — the masterplan/ADR-0217 mandated first deliverable) + the UNIFIED FRONTEND SHELL hosting modules + EVERY required cloud substrate at FULL extent (IdP, Cedar policy engine, KMS, persistence, observability, audit, metering/billing, messaging, network/DNS) to support AND dogfood the product. Never MVP/demo/good-enough/defer. Every delivery friction is a pipeline-product failure → becomes an enforced gate/automation.

## Founder directives (binding, recorded in session memory)
1. Root `goal.json` is stale — authority order: HANDOFF.md → /specs/masterplan.json (FD-001) + master-plan-sequencing → ADR-0516..0535 fabric canon.
2. Authorization = **RBAC + ABAC + PBAC** full spectrum (Cedar natively models all three). Read every "Tenant RBAC" shorthand as full-spectrum.
3. **Proven patterns, Rust reimplementation** — every decision cites its hyperscaler precedent; no invented architecture where proven practice exists.
4. **No mockups/prototypes** — production-grade running systems only; rename/productionize the Leptos "prototype" crate.
5. **Parallelize** — API-first contract lock, then independent worktree lanes (masterplan `parallel_lanes_after_contract_lock` doctrine).
6. **ALL CLI retired** — authority = cloud-ci gates/required contexts; operations = console + API; no new CLI surfaces ever.
7. **Cloud-native, Kubernetes-native operation — written in Rust, whole stack owned**: CRDs + operators/reconcilers + GitOps for everything, zero imperative ops; AND the stack itself is owned Rust end-to-end — kuberos-kernel (`cloud-kernel`) → Talos-like OS (`cloud-os`) → bespoke Rust Kubernetes substrate (`cloud-k8s`) → Rust cloud services → Rust oyatie products. Upstream k8s/containerd/Talos = transitional impls behind stable interfaces, cutover-gated (ADR-0510), never terminal.
8. Consolidate the 6 `consolidate/*` snapshot branches per HANDOFF §4 founder map (office-pilot conformance pattern).

## Decision basis (16-domain × 5-company research, 2 workflows, source-grounded)
- **IdP:** single-homed write CP + cell-replicated authn DP; offline credential verification (cell-local JWKS, never sync introspection); Oracle-style identity domains w/ primordial operator domain + sealed offline FIDO2 break-glass; passkeys v1; CAEP-style event revocation + Cedar issue-time cutoff. (AWS IAM/Entra/OCI 5/5 convergence.)
- **Authz:** Cedar **embedded in-process PDP** (4–11µs p99) in every service; central policy-store CP compiles/signs/pushes **content-addressed policy bundles** via the delivery fabric; tenant = Cedar namespace; `forbid` reserved for the structural tenant-isolation invariant; retire the hand-rolled `oya-policy-cedar-*` evaluator (ADR-0243 violation — two decision algorithms must never coexist).
- **Cells:** cluster-per-cell (Talos k8s), zero shared state, caps published in TPS/tenants/GB, ≤70% tested max; thinnest Rust router serving last-known-good content-addressed route tables.
- **Shell:** ONE platform-owned production Leptos shell (ADR-0393) owning all chrome + sole token brokerage; **build-time composition from buck2 monorepo** (no iframes — Google retired them; no module federation); design system as merge gate; the console is the **replacement operator surface for all retired CLIs**.
- **Control plane:** uniform resource-provider contract (shared Rust contract-test crate gating CI before service #2 diverges — ARM/AIP/CloudControl lesson); AIP-151 operation ledger; client-UUID idempotency; K8s-native actuation via reconcilers.
- **Observability:** OpenSLO files = single codegen source → multiwindow multi-burn-rate alerts → **automatic rollback triggers**; one wide-event per unit of work; static-threshold paging CI-rejected.
- **Delivery fabric:** presubmit latency SLO (~10–15min, ≥95% predictive) + exhaustive postsubmit w/ auto-bisect/auto-revert; Tide pessimistic merge queue first; code review = last human gate; shadow→warn→enforce for every new gate.
- **KMS:** AWS domain model (per-cell sealing roots → versioned per-tenant KEKs as wrapped tokens → per-object DEKs); one-way door in the type system (KEK plaintext only in mlock'd zeroize enclave process); rotation = key versions, never re-encryption; static stability (bounded-TTL DEK cache + bucket keys: reads never need live KMS); per-tenant KEK → quorum crypto-shred offboarding. Transitional custody = OpenBao behind owned interface (ADR-0510).
- **Persistence:** owned `oya-data` Rust SQL interface; PROVEN CRDB/TiKV-class transitional impl (ADR-0510 cutover-gated to W5 bespoke multi-Raft leader-per-range + HLC ClockSource trait + Pebble-class LSM); RLS tenant isolation; transactional outbox; CDC.
- **Storage (CAS):** four planes; metadata in oya-data (Tectonic keyspaces); strong read-after-write from first commit; BLAKE3 content addressing (dedup within tenant-KEK boundary); Object-Lock compliance-mode semantics at launch (audit/WORM sink).
- **Compute:** one shared fleet (Borg/Twine; per-team clusters rejected w/ 20–30% tax evidence); Cedar-enforced isolation ladder (first-party = hardened runc; tenant-influenced = Firecracker microVM); Talos zero-SSH validated.
- **Messaging:** Pulsar VALIDATED launch-primary behind thin owned Rust client; queue/stream/bus as 3 single-concern surfaces over ONE substrate; at-least-once + outbox = effectively-once; per-key ordering only.
- **Metering/Billing:** pipeline not query (at-least-once → idempotent dedup `(tenant,resource,dimension,usage_hour)` → hourly rating → monthly invoice); FOCUS 1.2 internal schema day one (+tenant_id/cell_id); versioned immutable price book; append-only identified line items; restatement-then-freeze close; double-entry subledger (debits=credits transactional invariant); 6h lateness window w/ explicit rejection; KR-VAT native.
- **Audit:** CloudEvents envelope + GCP-AuditLog-shaped payload as one libs/ crate emitted from tower middleware; admin stream always-on, **no kill switch (CI lint)**; audit-chain → CloudTrail-grade signed digest chain anchored in CAS WORM; verification = gate app/console surface (NOT a CLI per directive 6).
- **Network/DNS:** W0-critical DNS to Route-53 doctrine (shuffle-shard-of-4, serves from signed snapshots, runs with control plane dead, minimum-answer floor invariant); Katran-class Rust L4 (aya eBPF/XDP) + GFE-class Rust L7; config-compiler/dataplane split per network service.
- **Gateway/SSOT:** Smithy ARCHITECTURE in Rust (typed model + traits + emitters → OpenAPI/proto3/GraphQL/axum/tonic/clients as content-addressed outputs; OpenAPI emitted, never authored); gateway as Cedar PEP; two-stage rate limiting; one Check/Report substrate for quotas AND metering.

## Dogfood bootstrap order (circular-dependency-free, 10 steps)
0. Root-of-trust ceremony (Shamir M-of-N offline, dual-control safes) → 1. KMS unseal (OpenBao+PKCS#11; KMS storage = own local Raft group, NOT oya-data — breaks KMS↔DB cycle) → 2. Secrets + workload identity (CA mints SPIFFE mTLS certs at pod admission; fetch-fail = deploy-fail; zero static secrets) → 3. IdP (human/agent principals; primordial domain) → 4. Cedar PDP (embedded; all authz from here) → 5. Network/DNS (bootstrap from hand-signed seed snapshot; CAS later) → 6. Persistence (envelope-encrypted; separate single-Raft bootstrap metastore) → 7. CAS (metadata seeded from static-config instance; recursion-break ADR; DNS switches to CAS snapshots) → 8. Messaging (outbox relay; two loss classes) → 9. Audit/compliance (digest chain anchored in CAS WORM; logging-mode-first) → 10. Commercial/edge (metering/billing/gateway; internal chargeback for every service from this step). Hard rule as buck2 dep lint: no Tier-N service links a live client of Tier>N.

## Governance (every story)
Isolated worktree branch off `dev` → PR → single required context `oya-ci-required` green → review threads resolved → squash-merge. SSH-signed commits. NO `*.generated.json` add/modify in any diff (materializer script + diff-policy gate). No new CLI surfaces. Every service: slos/*.openslo.yaml before promotion (ADR-0130, live slo-coverage gate); K8s-native operational shape (CRD/operator/GitOps); clean architecture + API-first contracts before handlers; multispectrum evidence. Shadow→warn→enforce for new gates. Each story cites its hyperscaler precedent in its ADR/PR.

## Flagged founder decision points (carried, non-blocking)
- Cell boundary GTM posture (cell-within-region vs region-is-cell vs OCI realm) — research says GTM choice, not engineering.
- Audit retention posture (fixed ≥400d vs customer-configurable to 10y).
- Pricing/metering dimensions + price-sheet structure for the first module (needed before invoice rating goes live).
- Numeric W5 cutover trigger table per substrate (ADR-0510 format) — oya-data/CAS/SCM.
- HSM procurement timing (software-FIPS + OpenBao seal-wrap until hardware custody?).
- Firecracker adopt-vs-reimplement confirmation (research: adopt the proven Rust artifact behind a bespoke runtime shim).
- Policy-Zones logging-mode at FD-001 launch vs enforce for messenger/mail personal/professional split under KR/EU posture.
- oyago/oyapy transpiler destination path (HANDOFF §7.1).

## Story lane map
G01 serialized contract-lock → G02..G09 + G12 parallel worktree lanes → G10 integration fan-in → G11 continuous ratchet → G13 final gate.

## AMENDMENT 2026-06-09 (founder /goal condition, applies to G011 + G013)
- Every friction/failure/troubleshoot is appended to .omc/ultragoal/friction-ledger.jsonl BEFORE its workaround is applied; G011 converts entries into enforcement (shadow→warn→enforce).
- Terminal bar: impossible to ship anti-scalable / anti-production / anti-best-practice patterns — enforcement and automation, never convention.
- G011 acceptance gains a UNIVERSALITY proof: the conformance ratchet (gate crates + oya-ci.toml schema + oya-ci-config) must be de-oyatie'd config-data-first and demonstrably run against a non-oyatie fixture repo (hermetic, universal, canonical pipeline usable by any project/repo/enterprise — W3 de-oyatie + gate SDK direction, HANDOFF guardrail 8).

## AMENDMENT 2 — 2026-06-09 (founder design doctrine, applies to ALL lanes G02–G09 + SSOT work)
Ports/interfaces are shaped by the OWNED destination stack (bespoke Rust kuberos/cloud-os/cloud-k8s, oya-data multi-Raft,
CAS, KMS domain model, queue/stream/bus semantics); adapters absorb ALL impedance to transient infra (OpenBao, CRDB-class,
Pulsar, upstream k8s). Review question for every port: "would this trait change at W5 cutover?" If yes — redesigned around
the transient stack; reject. Also: intelligence SDK adapters destination corrected to cloud/cloud-intelligence/ (NOT
oya/intelligence/), overriding HANDOFF §4.

## AMENDMENT 3 — 2026-06-09 (founder: buck2-first verification, applies to ALL lanes + workflows)
Every workflow/subagent/teammate prompt that builds or tests Rust must EXPLICITLY instruct buck2-first:
`buck2 build` + `buck2 test` on affected targets (hermetic fabric substrate; content-addressed). Cargo is
supplementary feedback only (CI still runs the cargo gate matrix). New crates: BUCK targets + reindeer
regeneration are part of definition-of-done. G011 enforcement: keep the hermetic buck2 lane required and
extend toward cargo↔buck2 target-parity fail-closed.

## AMENDMENT 4 — 2026-06-09 (founder: staleness = process failure; automation maximalism)
G011 acceptance gains the staleness gate family (mechanical, repo-agnostic): adr-citation-liveness,
referenced-path-exists, authority-claim-parity (docs vs branch-protection config), hook-text-lint
(no retired-surface suggestions in agent-facing guidance), plus enforcement-liveness (every hook
wired, every gate registered — FRIC-012). Anything done manually twice in any lane becomes an
automation or a logged exception (masterplan automation-ratchet rules now bind orchestration too).
W2 AST doc-tracking subsumes the staleness gates long-term.

## AMENDMENT 5 — 2026-06-09 (founder: enforcement layering)
Hierarchy: (1) canonical/universal = cloud-ci required contexts + branch protection — binds every actor;
(2) structural = violations impossible by construction (buck2 hermeticity, type-system one-way doors,
closed schemas, RLS); (3) hooks = last-stop safety net for agent lanes ONLY, never load-bearing.
G011 acceptance test per rule: "does it still hold with hooks globally disabled?" Any hook-only rule
is an open friction until its gate or structural fix lands.

## AMENDMENT 6 — 2026-06-09 (founder: Rust hooks/scripts/tools)
All hooks, scripts, and tools are Rust (HANDOFF §6.2 zero-shell + Rust-owned-stack). Existing tools/hooks/*.sh
and infra/ci/*.sh are transitional: G011 ships Rust hook binaries (same stdin JSON contract, one subcommand per
hook, buck2-built, unit-tested) and repoints .codex/hooks.json + .claude/settings.json at them. New automation
never ships as shell; irreducible glue requires an exception-ledger entry. Hook binaries are runtime-invoked
enforcement, not operator CLIs — consistent with all-CLI retirement.

## AMENDMENT 6b — 2026-06-09 (founder: Rust hooks delivered cloud-natively)
The Rust hook/tool successors follow the cloud-native/K8s-native doctrine: policy logic lives once in Rust
crates whose canonical PEP is the cloud-ci pipeline / K8s-native controllers (admission, reconcilers);
local agent hooks are thin clients of the SAME crates, delivered as content-addressed buck2/OCI artifacts
pulled through the fabric (never hand-synced binaries). One policy source, two enforcement points
(gate = canonical, hook = safety net per AMENDMENT 5).

## AMENDMENT 6c — 2026-06-09 (founder: 0-to-minimal shell)
Target = ZERO shell scripts; tiny irreducible-glue allowlist only. Baseline 48 tracked .sh
(26 scripts/, 12 tools/, 9 infra/, 1 oya/). G011: no-new-shell gate (fail closed on .sh additions
outside the allowlist), shrink baseline via Rust/buck2 replacements, exception-ledger row per survivor.

## AMENDMENT 7 — 2026-06-09 (founder: testing standards — unit alone inadequate)
Every lane's DoD includes the test ladder for its tier: unit+property-based → contract tests (resource-provider
harness from G001) → integration vs REAL substrates (containerized Postgres-RLS, real cedar-policy, Pulsar) →
E2E dogfood on clean bring-up → RED/GREEN fixture pair per gate/enforcement artifact → load/perf vs published
budgets (p99 ≤50ms read / ≤200ms write) → failure injection (KMS down, cell partition, policy-store down —
static-stability DEMONSTRATED, not claimed). Acceptance evidence names which rungs ran; unit-green alone never
satisfies a story. Aligns cloud-production-quality-kits-target (7 harness gates) + multispectrum fixture-pair-coverage.

## AMENDMENT 8 — 2026-06-09 (founder: complete Rust purity; cargo only for release images)
ZERO non-Rust languages (python/shell/perl/ruby/js/ts tooling all out). Baseline 78 tracked non-Rust files
(14 .py, 9 .mjs, 6 .ts, 1 .js, 48 .sh) → ratchet to ~0 via G011-family lanes; irreducible survivors need
exception-ledger rows. buck2 everywhere for dev/test/CI/hooks/tooling. SINGLE cargo exception: release-image
building (Dockerfile.distroless pattern) with maximum optimization — cargo build --release --locked,
lto="fat", codegen-units=1, panic=abort, strip, opt-level=3, target-cpu tuning, cargo-auditable SBOM,
SOURCE_DATE_EPOCH reproducibility, multi-arch amd64+arm64, distroless/scratch, PGO/BOLT later stage.
Enforcement: no-new-non-rust gate (canonical) + no-cargo hook (safety net) allowlisting only the
release-image context.

## AMENDMENT 9 — 2026-06-09 (founder: every-file authority chain in registry)
Not just hooks/scripts — EVERY tracked file records its authority chain (authoring ADR/spec + purpose) in
the registry. Implementation: extend the total-accounting registry rows (per-file ownership/justification
already enforced by the live cloud-ci-total-accounting gate) with authoring_adrs, reconciled with
artifact-capabilities-registry. ADR-retirement ratchet: when an ADR flips Superseded/Retired, a gate
queries files chaining to it and FAILS CLOSED until each is dispositioned (update-to-successor or dispose)
— the retirement packet is the query result. docs/audit/ provenance archives exempt. Baseline+block-NEW
rollout like every ratchet (existing files backfilled incrementally; new files require the chain at birth).

## AMENDMENT 10 — 2026-06-09 (founder: two-outcome ADR retirement; AST-derived chains)
ADR retirement has exactly two outcomes: SUPERSEDED (successor named; chained files re-point and survive)
or REMOVED (chained files disposed). Gate fails closed until every chained file took one path.
The file→ADR edges are AST-DERIVED, not hand-authored: W2 owned AST parser (content-addressed node identity,
WorkAreaTree) extracts citations as typed nodes (markdown/rustdoc/JSON-path); the authority-chain registry
face is a generated projection materialized by CI (generated-faces discipline applies — never hand-edited).
Transitional bridge until W2: grep-derived projection behind the same registry interface (ADR-0510).
Staleness gates go AST-native at W2. This raises W2 (owned AST parser) priority — it is now load-bearing
for governance, not just dev tooling.

## AMENDMENT 11 — 2026-06-09 (founder: tree-sitter transitional; owned runtime = north-star, not urgent)
W2 AST substrate ruling frame: ADOPT tree-sitter (proven at scale: GitHub/Zed/Neovim/Semgrep; grammar
ecosystem serves the universal-pipeline requirement) as TRANSITIONAL impl behind the WorkAreaTree port.
Owned bespoke-Rust runtime stays the NORTH-STAR destination, scheduled in the roadmap (scoped W-stage)
but NOT urgent — no near-term capacity on parser reimplementation. Build OUR layers on top from day one
(content-addressed node identity, doc-tracking/staleness gates, AST-derived ADR citation edges per
AMENDMENT 10) — port + layers survive cutover unchanged. C-core purity exception ledger row
(aws-lc-rs precedent). Cutover = ADR-0510 numeric triggers (from wf_265ffc1e-fcf prescription), not calendar.

## AMENDMENT 12 — 2026-06-09 (founder FINAL on W2 AST: bespoke rowan-style is the default — supersedes AMENDMENT 11)
Default = bespoke rowan-style owned parser (already started + scoped). Tree-sitter NOT adopted; evaluation
workflow results = reference research only (algorithm/query-language input; grammars as possible test oracles).
No C-core purity exception needed. First grammar targets: Rust + markdown + JSON/TOML/YAML (governance-load-
bearing for AST-derived citation edges); customer-language breadth follows. WorkAreaTree port shape unchanged.

## AMENDMENT 12b — 2026-06-09 (founder: one AST core, reused by transpilers)
The bespoke rowan-style core is ONE substrate: pipeline gates/doc-tracking/citation-edges + oyago/oyapy
transpiler frontends (Go/Python grammars = frontends on the same CST core) + future customer-language
breadth. Build once, reuse. Tasks #9/#10 destination: parsing layer co-locates with the owned AST core;
sibling snapshots preserved as grammar/frontend input material.

## AMENDMENT 12c — 2026-06-09 (founder: bespoke core = tree-sitter feature-parity SUPERSET where it matters)
The rowan-style owned core must meet-or-exceed tree-sitter on the capabilities that matter to us:
QUERY/pattern language (S-expression-class queries driving gates/lints/auto-fix — explicitly named by
founder), incremental re-parse, error-tolerant parsing, published performance baselines — PLUS our
superset features: content-addressed node identity, lossless CST, AST-derived citation edges, transpiler
frontends (oyago/oyapy), markdown+specs as first-class grammars. The tree-sitter evaluation workflow
(wf_265ffc1e-fcf) output = the parity checklist + benchmarks to beat, not an adoption decision.

## AMENDMENT 13 — 2026-06-10 (founder: consensus quorum must scale to extremes)
Standard Kubernetes quorum (etcd-class: ALL cluster state + watches through ONE Raft group) is REJECTED for
the owned stack — it does not scale to extremes (~8GB/single-leader/5k-node caps). Compliant shape: within a
cell, control-plane state shards across MANY small consensus groups (multi-Raft leader-per-range on oya-data
— Spanner/DynamoDB/CRDB/TiKV/ZippyDB 5/5 precedent; or Physalia-class micro-quorum colonies); across cells,
NO global quorum (independent consensus domains + eventually-consistent static-stable thin global layer).
ADR-0537 Tier-0 per-cell bootstrap metastore (5-9 nodes, bounded) stands as the bootstrap exception.
cloud-k8s (owned substrate) designs its state plane on oya-data ranges from day one; upstream etcd is
adapter-absorbed transitional (ADR-0510) and never shapes the port. M-of-N key/human ceremonies unaffected.
Fold into ADR-0537 at founder sign-off; W5 cutover criteria gain the multi-group state-plane proof.

## WIND-DOWN 2026-06-10 (founder: let them finish, start no new work)
Substrate team enters clean wind-down: workers complete ONLY their current in-flight PR + rebase-to-green,
then idle — no new sub-slices/scope/lanes. Leader merges the train as PRs go CLEAN; launches no new review
scope or fix agents beyond verifying in-flight work. Stopping point = all open substrate PRs (G02-G09)
either merged or left as reviewed-and-pushed for a future session. Remaining backlog (G011 ratchet from the
friction ledger, oyago/oyapy W2-AST fold-in, ADR-0536/0537 sign-off, XPROXY #644 routing) deferred to a
future session by founder direction.

## AMENDMENT 14 — 2026-06-10 (founder: Fable dispatch + ultraqa review agents)
Execution teammates AND review agents dispatch on **Fable (Claude)**, not codex; codex-exec lanes in flight at directive time ran to completion only, and no new codex dispatches are permitted. Review agents are fresh-context Fable subagents that (1) load the meta-skills `/using-superpowers` + `/using-agent-skills` first, (2) run `/oh-my-claudecode:ultraqa` as the QA cycling harness, and (3) apply `RUBRIC-torvalds-review.md` with the Torvalds lens (verify intent AND execution, in-code findings only) and the hyperscaler lens (precedent cited or divergence justified) as mandatory axes. Rationale rows: FRIC-1781110000 (silent compaction death), FRIC-1781111000 (stdin-blocked dispatch), FRIC-1781113000 (codex-env buck2 daemon failures). Encoded in: TEAMMATE-PREAMBLE.md, RUBRIC header, RESUME-PROMPT, CHECKPOINT §9, INDEX dispatch section, memory dispatch-via-fable.

## AMENDMENT 14a — 2026-06-10 (founder: codex as supplementary review lens)
Clarifies AMENDMENT 14: `codex exec` MAY still be run at the leader's discretion as an ADDITIONAL review lens — for critical/high-risk changes, for consensus (Fable + codex dual-model verdicts), or for extra insight. Primary execution and primary review remain Fable. Codex never becomes a worker-lane dispatch path again; its role is supplementary opinion, with the Fable review remaining the verdict of record.

## AMENDMENT 14b — 2026-06-10 (founder: /team orchestration with mixed fleets)
Utilize `/team` (OMC team orchestration) where appropriate — large parallel fan-outs, mass-batch campaigns, multi-lane sweeps. Teams MAY mix codex and Fable workers. Hard caps: **Fable ≤ 4, codex ≤ 8, total ≤ 12**, arranged in a **3×4 layout**. Codex workers in a team ride the `/team` runtime itself — the LEADER never dispatches worker lanes as raw `codex exec` (founder clarification same day). Teammates MAY run `codex exec` within their own sessions (nested supplementary review, consensus, insight — the AMENDMENT 14a lens applies one level down too). Fable remains primary for single-lane dispatch and for review verdicts of record (AMENDMENT 14/14a unchanged for those); the team runtime is the scale-out path when one lane per worker is the right shape. Leader still owns the merge train and independent review of every PR.
