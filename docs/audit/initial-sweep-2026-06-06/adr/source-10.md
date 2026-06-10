# ADR Audit — SOURCE chunk 10

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 10
- **Slice range (lines 64–70 of sorted ADR listing):** ADR-0066, ADR-0067, ADR-0069, ADR-0083, ADR-0090, ADR-0091, ADR-0092
- **ADRs actually reviewed (7):** 0066, 0067, 0069, 0083, 0090, 0091, 0092
- **Auditor posture:** READ-ONLY. Trust the superseding ADR over stale front-matter. Treat `foundry`/`tier`/`M0–M3`/`Jenkins`/`Kafka`/`Redis`/`grit`/`rtk`/`icm`/`ICM`/`Citus` as retired-vocab per keystone map §2.

---

### ADR-0066 — Live code-introspection: docs portal reflects realtime project state (endpoints / dep graph / dead-code)

- **decision_atom:** The docs portal is a realtime, fully machine-generated control-plane view: every structured fact (endpoint inventory, crate dep graph, dead-code) is produced by canonical extractors over code+telemetry+markdown (zero hand-authored tables), with on-commit/on-PR/live-daemon refresh modes and four CI lanes (doc-consistency, endpoint-coverage, dead-code-zero-tolerance as BLOCKER).
- **current_status:** accepted / published.
- **disposition:** SUPERSEDE/MERGE — explicitly subsumed by ADR-0067 §6 ("ADR-0065 + ADR-0066 are subsumed… the `docs` µservice is retired → renamed to `ops.docs` BC"). Content semantics carry forward; the parent-µservice naming is governed by ADR-0067.
- **governing:** ADR-0067 (ops console reframes the `docs` portal as the `docs` BC of `ops`).
- **truth_flag:** PARTIAL — the extractor-as-canonical-source thesis is TRUE and durable; the surrounding scaffolding is heavily retired-vocab-laced (GitHub Actions as CI source, `grit`/`rtk`/`icm` extractors, `oya-docs-*` crate prefix, Bominal cross-refs) so the doc as written is STALE in its mechanics.
- **in_masterplan:** PARTIAL — owns no planning front-matter (`doc_class`/`authority_tier`/`masterplan_ref` absent; only `id/status/doc_status`); the ops-portal capability is reflected in MASTERPLAN only via its successor ADR-0067. Fails the 8.8%→100% binding goal.
- **tensions:**
  - vs **ADR-0067** — direct supersession/rename (`docs`→`ops`, `oya-docs-*`→`oya-ops-docs-*`). 0066 still says `oya-docs-*` and "`docs` µservice" throughout — stale.
  - vs **keystone §2 retired-vocab** — extractor table cites `GitHub Actions API`/`gh api` (CI is now Argo Workflows per ADR-0511), `grit status --json` + `icm export` (grit/icm retired per ADR-0116), `oya-foundry-*`-grade observability ("Bominal Foundry-grade", §Neutral) — all retired.
  - vs **owner `axis-foundry`** — "foundry" axis/team naming is retired-brand-adjacent (ADR-0335/0347; founder: cloud-intelligence is the valid name).
  - vs **planning-ssot-consolidation.md** — ADR uses `M02-P20/P21/P22` milestone IDs and `lean-aN` ADR-number-adjacent lane names; the consolidation doc FORBIDS milestone-keyed / number-keyed gate names going forward.
- **hyperscaler_challenge:** Aligned (questionable on scope). Google (Cloud Asset Inventory + live introspection) and AWS (Config + Resource Explorer) absolutely build single-pane realtime inventory — the pattern is sound. Questionable: a `dead-code-zero-tolerance` lane as a **BLOCKER from day 1** across the whole repo is more aggressive than any hyperscaler runs (they quarantine/report, rarely hard-block CI on unreferenced files). Argues for AMEND (downgrade the absolutism), not archive.
- **ai_slop:** Mild. "≤2s p99 (incremental)" precision on an unbuilt daemon is fabricated-precision; the §Neutral "Bominal Foundry-grade observability" is brand-residue filler. ~18-page exhaustive table enumeration (20 portal pages, 8 interactive primitives) is over-spec for a not-yet-built surface — but it is genuine design, not hallucination.
- **refinement:** Fold into ADR-0067 as the `docs` BC appendix; rewrite extractor table to drop GitHub-Actions/grit/icm rows and point at Argo Workflows + the live agent-coordination ledger; rename `oya-docs-*`→`oya-ops-docs-*`; add planning front-matter so it binds to masterplan; soften dead-code BLOCKER to report-then-flip.
- **consensus_needed:** no — supersession is already self-declared by ADR-0067; only the masterplan-binding question is shared across the chunk.

---

### ADR-0067 — `ops.oyatie.com` canonical hyperscaler-grade operations console

- **decision_atom:** All operational surfaces (docs, dashboards, DB/schema browser, tech-stack, architecture, health/SLO, tenant-mgmt, user-mgmt, observability, deployments, capacity, finops, on-call, incident, audit-view, CI-runs — ~20 bounded contexts) consolidate into a single Cedar-policy-gated operations console at `ops.oyatie.com`, sharing one Leptos stack and one tenant-scoped manifest; the `docs` µservice is renamed to the `docs` BC of the parent `ops` µservice.
- **current_status:** accepted / published (carries `sunset_topic: adr-0067-ops-console-protected-contracts`, `sunset_milestone: doctrine-not-time-bounded`).
- **disposition:** AMEND — sound, load-bearing, and still canonical (it is the *governing* ADR for 0065/0066), but drifted on retired vocab and on the CI/forge posture. Keep the decision; reconcile the references.
- **governing:** n/a (this is itself a governing ADR for ADR-0065/0066).
- **truth_flag:** PARTIAL — single-pane ops-console thesis is TRUE; specific BC source-bindings are STALE (GH Actions runs, `grit`/`ICM`-browser BCs, Citus/Postgres "ADR-0117" inheritance, PagerDuty-equivalent, OKE).
- **in_masterplan:** PARTIAL — has `sunset_*` fields but NO `doc_class`/`authority_tier`/`masterplan_ref` front-matter; ADR-0067 explicitly says "MASTERPLAN §2.1 catalog update: `docs` line replaced with `ops`" — i.e. it *asserts* a masterplan edit but the binding is one-directional and unverified. Partial.
- **tensions:**
  - vs **keystone §2** — `icm-browser` + `grit-status` BCs are first-class surfaces for tools that are RETIRED (ADR-0116). `ci-runs` BC is bound to "GH Actions API"; CI is Argo Workflows (ADR-0511). "Foundry-grade observability", `internal-foundry` Cedar role, owner `axis-foundry` — retired brand (founder: cloud-intelligence/governance).
  - vs **Bominal ADR-* citations** — ~12 "Bominal ADR-NNNN" cross-refs (0009/0020/0028/0049/0107/0117/0123/0132/0209/0224–0231). These are an external/inherited corpus the keystone map never indexes; unverifiable provenance inside this repo (possible fabricated-precision or a legitimately separate inherited series — flag, do not assume).
  - vs **ADR-0090 §Amendment** — `tech-stack` BC and the Leptos stack assume hyper backbone; consistent, but the §4 Cedar roles reference "Bominal ADR-0132 pillars" rather than the in-repo Cedar canon (ADR-0243/0246) — naming drift.
  - vs **forge fault-line** — `ci-runs` surface design hard-codes GitHub-Actions semantics; conflicts with both the Argo destination and the Forgejo/GitHub forge contest (keystone §5).
- **hyperscaler_challenge:** Aligned. AWS Console / GCP Cloud Console / Azure Portal ARE exactly this — one console, role-gated, fleet-wide + per-tenant. The single-pane + Cedar-per-tenant-scope + zero-cross-tenant-leak bar is precisely what a hyperscaler ships. Verdict aligned; the only questionable bit is the "0 external tools (no Datadog/Grafana/PagerDuty)" build-everything stance — hyperscalers do build these, but it is a multi-year cost; argues mildly for staging, not archive.
- **ai_slop:** Low-moderate. The AWS-Console-+-Datadog-+-Grafana-+-PagerDuty-+-Linear-+-Workday-+-GitHub-Insights-+-Palantir-Foundry comparison stack is rhetorical inflation. "~100–140 crates over time" and per-ms p99 targets on 20 unbuilt BCs are fabricated-precision. Otherwise a coherent, deliberate architecture doc.
- **refinement:** Strip retired-vocab BCs/roles (icm/grit → "agent-coordination ledger" surface; `internal-foundry` → `internal-intelligence`/`internal-governance`; GH-Actions ci-runs → Argo Workflows); resolve the Bominal-ADR citation provenance (either import that series into the index or convert to in-repo ADR cites); add proper masterplan front-matter; reconcile Cedar role refs to ADR-0243/0246/0379.
- **consensus_needed:** yes — **"Is `ops.oyatie.com` a single ~20-BC owned console (build Datadog/Grafana/PagerDuty/audit-view ourselves), or do we adopt best-of-breed OSS (Grafana/Loki/Tempo per ADR-0383) as data sources behind a thin owned shell?"** This is the same own-vs-assemble fault-line (keystone §5) at the ops-surface layer and is load-bearing for ~100+ crates.

---

### ADR-0069 — Active machine-readable artifact contract (9-capability declaration + knowledge-graph substrate + registry control plane)

- **decision_atom:** Every machine-readable artifact (JSON/TOML/YAML/Cedar/SQL/OpenAPI/Cargo.toml) must declare 9 capabilities (Enforcement, Verification, Validation, Autogen, Selfheal, Selfupdate, Selfmaintain, Telemetry, Provenance) with honest `operational|planned|blocked-by-foundation|not-applicable` status backed by resolvable evidence, governed by a central capability registry + reusable-building-blocks (DRY) registry + a typed knowledge-graph catalog, enforced by a validator crate and CI lane.
- **current_status:** accepted / published.
- **disposition:** AMEND — the contract is conceptually strong and masterplan-relevant (it IS a drift-prevention/SSOT-binding mechanism), but it has two concrete broken references and is self-admittedly plan-stage-only (validator not integrated, lane `planned`).
- **governing:** n/a directly; deeply entangled with planning-ssot-drift-prevention.md (the `masterplan_ref` binding gate is a sibling mechanism).
- **truth_flag:** PARTIAL — the 9-capability/knowledge-graph design is TRUE-and-durable; status claims are honestly hedged to `planned`; but the doc contains WRONG references (see tensions) and retired-vocab (`rtk git commit`, `grit claim`/`grit done`, `ICM`/`scaffold-locks-oyatie` fallback, owner `axis-foundry`).
- **in_masterplan:** PARTIAL — owns no planning front-matter itself, yet it *designs the very binding system* the masterplan needs. This is the most masterplan-adjacent ADR in the chunk and ironically does not carry the `masterplan_ref` it would mandate for other artifacts.
- **tensions:**
  - **BROKEN REF (WRONG):** References ADR-0088 twice — front-matter `Related: ADR-0088 (foundry microservice scaffolding)` and References `ADR-0088-microservice-foundry.md (ADR scaffolding pattern precedent)`. **No ADR-0088 exists on disk** (verified) — fabricated-precision / dangling citation, AND the two titles given for it differ. Also cites `ADR-0067-ops-oyatie-com-portal-foundation.md` (References) but the real filename is `…-hyperscaler-operations-console.md` — broken path.
  - vs **planning-ssot-consolidation.md / -drift-prevention.md** — strong alignment in spirit (mechanical drift prevention, provenance, autogen) but the OPEN founder question (ADRs-generate-masterplan vs masterplan-is-authority) is unaddressed here; this ADR's "registry control plane" leans toward generated/derived artifacts. Flag under both readings.
  - vs **keystone §2** — `grit`/`rtk`/`ICM` as the state-transition + commit protocol throughout the Decision and Migration sections — retired (ADR-0116). Owner `axis-foundry` retired-brand.
  - vs **own-vs-assemble** — §Negative concedes the knowledge-graph needs a real graph store (Neo4j/Memgraph/Postgres-CTE) "before it scales past ~10k artifacts"; consistent with source's "best-of-breed now" posture, but no governing ADR is named for that substrate choice.
- **hyperscaler_challenge:** Aligned. This is explicitly modeled on AWS Config + AWS Resource Explorer + GCP Cloud Asset Inventory/Asset Graph + K8s CRD+admission — the citation is apt and a hyperscaler would absolutely build a typed resource+relationship inventory with policy-as-code admission. Verdict aligned; the 9-capability-per-artifact heaviness is the only questionable bit (mitigated in-doc by the `artifact_profile` defaults). Argues for keep-and-amend.
- **ai_slop:** Moderate. The dangling ADR-0088 (with two conflicting titles) is genuine fabricated-precision. The blow-by-blow "Linus-style findings closed in commit b0798b0 / 5880ce0" with per-finding numbers (#1–#10) reads as process-theater embedded in a decision record — verbose, low decision-value, and references commit SHAs an auditor can't resolve. Internal-contradiction risk in the `consumer_count_*` field-rename saga.
- **refinement:** Delete/repoint the ADR-0088 references (likely meant ADR-0091 foundry-write-gate or a never-authored scaffolding ADR — founder must confirm); fix the ADR-0067 filename; strip grit/rtk/ICM protocol prose to a one-line "agent-coordination ledger" pointer; move the per-finding commit log to an evidence appendix; name the eventual graph-store ADR; and — critically — make THIS contract emit the `masterplan_ref` binding the drift-prevention doc requires.
- **consensus_needed:** yes — **"Does the 9-capability artifact contract become the mechanism that binds ADRs/specs into masterplan.json (i.e. masterplan-is-authority, artifacts bind in), or is it subordinate to an ADRs-generate-masterplan model?"** Same open founder question (keystone §4), and this ADR is where it gets mechanized.

---

### ADR-0083 — Rust error-handling tier decision (thiserror at libraries / anyhow at binaries / no panics in library code)

- **decision_atom:** Library crates MUST return matchable `thiserror` enums (no `anyhow`/`eyre`/`Box<dyn Error>` in public APIs, no `.unwrap()`/`.expect()` outside tests); binary crates MAY use `anyhow`/`eyre` only at `main()`/top frames; test code is unrestricted; a `silent-failure-hunter` reviewer + clippy `deny(unwrap_used/expect_used/panic)` enforce it mechanically.
- **current_status:** Accepted / published. **Best front-matter in the chunk** — full `doc_class: DecisionRecord`, `authority_tier: 2`, `length_cap`, `canonical_authority: docs/CONSTITUTION.md`, `supersedes/superseded_by`, `related_adrs`, `companion_docs`, structured `purpose`.
- **disposition:** KEEP — current, correct, well-formed, non-conflicting; this is the model ADR for the whole chunk. Backfill-ready as-is.
- **governing:** n/a (no supersession; pairs with ADR-0037 stability tiers, ADR-0056 layer enum).
- **truth_flag:** TRUE. The decision is industry-standard Rust practice; the amendment (`append_classifications` 0.1.0→0.2.0 breaking change to restore Tier-1 conformance) is concrete, evidenced, and correct.
- **in_masterplan:** YES (effectively) — carries proper planning/authority front-matter (`authority_tier: 2`, `canonical_authority`), `companion_docs`, and a decision log; this is exactly the binding shape the drift-prevention gate wants. The lane `oya-governance-error-boundary` uses the CORRECT post-rename governance prefix (not `oya-foundry-*`) — clean.
- **tensions:**
  - Minor naming: title uses "**tier**" ("error-handling tier", "Tier 1/2/3"). This is the *correct* generic English sense, NOT the retired tenant `tier-system` (ADR-0329) — no real conflict, but a lint scanning for "tier" will false-positive. Worth a one-line disambiguation note.
  - `sunset_topic: adr-0083-infallible-audit-signature` / `sunset_milestone: adr-0083-merge-historical-2026-05-15` — the milestone value is a same-day historical marker, slightly odd but harmless.
  - Cleanly consistent with ADR-0092/0091 (both rely on typed `*Error` enums: `WriteGateError`, `HyperRuntimeError`, `AuditChainError`).
- **hyperscaler_challenge:** Aligned. Google's Rust guidance, AWS (Rust SDK uses typed errors), and the broader ecosystem all converge on thiserror-at-libs / anyhow-at-bins / no-panic-in-libs. A hyperscaler would make exactly this call. No amend pressure.
- **ai_slop:** None of substance. The amendment is dense but every claim is concrete (specific file:line `oya-audit-chain-domain/src/lib.rs:343`, specific variants, specific call-site counts). The `feedback_no_*` doc references are slightly jargon-heavy but load-bearing.
- **refinement:** Add a one-line note disambiguating "error-handling tier" from the retired tenant "tier-system" (pre-empt lint false-positives). Otherwise none — use this ADR's front-matter as the template for re-stamping 0066/0067/0069/0090/0091/0092.
- **consensus_needed:** no.

---

### ADR-0090 — Hyper canonical HTTP backbone (+ 2026-05-29 strategic hyper/axum split amendment)

- **decision_atom:** hyper 1.x (+ tokio, hyper-util, http-body-util, bytes) is the canonical low-level HTTP backbone and the *preferred default*; axum is a sanctioned, recorded exception for control-plane/CRUD-heavy services (built on the same hyper+tower backbone), with all other HTTP frameworks (actix/poem/warp/rocket/salvo/ntex) forbidden — enforced by `oya-check-http-stack` reading `specs/http-stack-policy.json`.
- **current_status:** accepted / published; `Amended 2026-05-29`.
- **disposition:** KEEP (as amended). The amendment already reconciled the original absolutism; current text is coherent and current. Minor AMEND only to add planning front-matter.
- **governing:** n/a; relates forward to ADR-0509 (canonical service layout prescribes `rest/` axum handlers).
- **truth_flag:** TRUE. The amendment honestly records the founder's 2026-05-29 reversal of "axum banned" → "strategic split", lists the 7 on-disk axum crates, and preserves the core thesis. Self-aware and accurate.
- **in_masterplan:** PARTIAL — only `id/status/doc_status` front-matter; no `authority_tier`/`masterplan_ref`. The policy IS machine-bound (`specs/http-stack-policy.json` + `oya-check-http-stack` gate) which is excellent, but the ADR itself isn't masterplan-stamped.
- **tensions:**
  - vs **LINUX ADR-0019 universal-port-ratchet** (cross-side) — same "own when proven, vendor now" philosophy ("build our libraries as we build", own router/middleware/sse kernels). The original "hyper everywhere, build it all ourselves" was the sharpest own-everything stance; the amendment softened it toward best-of-breed-where-it-pays — which actually moves SOURCE *closer* to its own staged-ownership ratchet and *away* from LINUX's day-0-ownership ambition. Worth noting as evidence the own-vs-assemble trigger threshold is the real axis.
  - vs **ADR-0092** — fully consistent (0092 isolates hyper to a single adapter crate; 0090 §layering says "Layer 5 is the only crate importing hyper"). Good coupling.
  - Internal pre/post-amendment tension is resolved in-doc; original "axum banned" lines remain in the body above the amendment, which could confuse a naive reader (the amendment supersedes them but the doc doesn't strike them).
- **hyperscaler_challenge:** Aligned (with nuance). The amended "prefer low-level hyper for data-plane/proxies/hot-paths, axum for CRUD control-plane, ban the rest" is a *more* hyperscaler-realistic posture than the original. Google/AWS do own their core HTTP/proxy layers (Envoy, gRPC-core, s2n) while using ergonomic frameworks for control-plane CRUD. The *original* "axum banned everywhere, hand-roll everything" would have been misaligned (NIH); the amendment fixes exactly that. Verdict aligned post-amendment; pre-amendment would have argued archive.
- **ai_slop:** Low. Original drivers are slightly repetitive ("support everything ourselves" stated 3 ways). The amendment is precise and evidence-bearing (names the 7 crates). No fabrication.
- **refinement:** Strike or clearly mark the superseded "axum banned" body lines so the doc reads consistently after the amendment; add `authority_tier` + `masterplan_ref` front-matter (the gate already exists, so binding is cheap). Consider migrating `oya-ci-webhook-gateway-app` off axum as the doc itself flags.
- **consensus_needed:** no — the founder already ruled (2026-05-29 amendment).

---

### ADR-0091 — Foundry write-gate foundations (Phase 05 contract)

- **decision_atom:** A single canonical write-gate state machine (`Proposed → Reviewed → Approved → Executed`, any non-terminal → `Rejected`, default-deny, four-eyes separation-of-duties across proposer/reviewer/approver/executor) governs every mutation across all transports (REST/GraphQL/SSE/WS/gRPC/Webhook/Kafka), so no single principal carries a write end-to-end.
- **current_status:** accepted / published; owner `council-foundry`.
- **disposition:** AMEND — the *decision* (four-eyes default-deny write-gate) is sound and durable, but the doc is saturated with RETIRED vocabulary: owner `council-foundry`, crate `oya-foundry-write-gate-kernel`, "Foundry needs…", plus a Kafka write-transport (retired → Pulsar) and `M02-P0x` milestone IDs (retired → Wave names). Naming/ref reconciliation required before it is masterplan-ready.
- **governing:** Brand governed by **ADR-0335** (foundry µsvc retired → absorbed by intelligence) + **ADR-0347** (`oya-foundry-*` → `oya-governance-*` rename). Eventing governed by **ADR-0377-kafka-to-pulsar** (Kafka retired).
- **truth_flag:** PARTIAL — the write-gate state-machine decision is TRUE; the foundry/Kafka/M02 wrapping is STALE.
- **in_masterplan:** NO — only `id/status/doc_status`; no authority/masterplan front-matter; the entire artifact (`oya-foundry-write-gate-kernel`) would need renaming before it could bind. Not currently reflected.
- **tensions:**
  - vs **keystone §2 (foundry RETIRED)** — owner `council-foundry`, crate prefix `oya-foundry-*`, and the word "Foundry" as the owning subsystem are all retired-brand. Per ADR-0347 this crate should be `oya-governance-write-gate-kernel` (write-gate is a CI/governance concern, not consumer-intelligence).
  - vs **ADR-0377-kafka-to-pulsar** — "M02-P05 introduces first write transports (gRPC / Webhook / Kafka)" and Follow-up IP-003 "bind Kafka producer to WriteGate" — Kafka is retired; should be Pulsar (KoP wire-compat).
  - vs **planning-ssot-consolidation.md** — `M02-P04/P05`, `IP-001/002/003` milestone+IP-keyed identifiers are the FORBIDDEN naming antipattern going forward.
  - vs **ADR-0083** — consistent (uses typed `WriteGateError::{Denied,Terminal,SamePrincipal}` — proper Tier-1 enum).
  - vs **ADR-0067 §4 audience tiers / §5.5 four-eyes** — overlapping four-eyes/separation-of-duties semantics; potential DRY consolidation (one write-gate kernel vs ops-portal's own approval flows).
- **hyperscaler_challenge:** Aligned. Four-eyes / separation-of-duties / default-deny on every mutation is textbook (AWS IAM approval workflows, GCP Binary Authorization, change-approval in all three). A hyperscaler absolutely builds one canonical mutation-gating state machine rather than per-transport gates. Verdict aligned on substance; argues for amend (rename), not archive — the decision survives the foundry-brand retirement, it just changes owner to governance.
- **ai_slop:** Low. Tight, concrete ADR. The only issue is retired-vocab, not fabrication. "Stripe/AWS-style four-eyes" is apt, not inflated.
- **refinement:** Rename `oya-foundry-write-gate-kernel` → `oya-governance-write-gate-kernel`; reassign owner `council-foundry` → `council-governance` (or intelligence per the split); replace Kafka transport with Pulsar; convert M02/IP identifiers to function-named/Wave-named ids; add authority front-matter. Consider merging its four-eyes semantics with ADR-0067 §5.5 to avoid two approval models.
- **consensus_needed:** yes (narrow) — **"Does the canonical write-gate belong to `governance` (CI/gates) or to `intelligence` (the foundry-absorbing subsystem), and is it one kernel shared with the ops-console approval flow (ADR-0067 §5.5)?"** — needed because ADR-0335 split foundry into two homes and this gate could land in either.

---

### ADR-0092 — Workspace dependency-seam policy

- **decision_atom:** hyper-family deps (`hyper`/`hyper-util`/`http-body-util`/`bytes`) are isolated to exactly one adapter crate (`oya-http-runtime-hyper-adapter`) with std-only (`Vec<u8>`) kernels above it; the canonical 12-layer enum (ADR-0056) is the single layer source-of-truth (derived from crate-name suffix, not metadata); workspace deps are tracked by a flat 11-row `dependency-rationales.json` overlay (NOT a state machine) enforced by `oya-check-dependency-seam`; plus a bundle of security hardening (body-cap/413, path-traversal, SSE/header injection, telemetry label safety).
- **current_status:** accepted / published.
- **disposition:** KEEP (with minor AMEND for front-matter). Decision is correct, well-reasoned (rejects speculative state-machine complexity with explicit, reversible re-evaluation triggers T1–T6), mechanically verified, and self-aware. Strong ADR.
- **governing:** n/a; amends an IP (M01-P13-IP-002) and pairs with ADR-0056/0090/0093/0094/0095.
- **truth_flag:** TRUE. The seam audit is reproducible (shell snippet returns exactly one crate); the YAGNI-now/reversible-later framing is honest and disciplined.
- **in_masterplan:** PARTIAL — only `id/status/doc_status`; the policy is machine-bound (`/registry/dependency-rationales.json` + lane) but the ADR lacks authority/masterplan front-matter.
- **tensions:**
  - vs **own-vs-assemble (keystone §5)** — §"Why the original IP may have been right" #1/#8 explicitly invokes "autonomous masterplan execution" and "hyperscalers build control planes early"; the ADR's judgment ("at 11 deps with zero removals YAGNI wins; revisit at 30 deps / 3 in-flight removals") is a clean articulation of the *trigger-threshold* disagreement that the keystone map says is the real axis. This is the most thoughtful own-vs-assemble reasoning in the chunk.
  - vs **Cedar canon (ADR-0243/0246/0379)** — §"What this ADR does NOT relax" mandates trigger predicates be expressed in **Cedar** (`oya-policy-cedar-*`), NOT a new DSL — correctly aligned with the universal-Cedar-gate posture. Good.
  - vs **keystone §2 / planning-ssot** — uses `M01-P06`, `M01-P13-IP-002`, `lean-aN`-style identifiers (retired/forbidden naming) and a `[[feedback-bominal-inheritance-precedence]]` wiki-link to the unindexed Bominal corpus (#5: "this ADR does not check Bominal; if inheritance is real, ADR-0092 owes Bominal a citation amendment" — an openly acknowledged dangling provenance).
  - vs **ADR-0093/0094/0095** — tightly coupled (renames, Handler trait, TenantSlug); the §Negative wording "4 ADRs… numbered 0091-0094 originally, but 0091 was already taken, so we renumbered" documents a real numbering collision that produced this 0092–0095 run — useful provenance, confirms ADR-0091 (foundry-write-gate) squatted the originally-intended number.
- **hyperscaler_challenge:** Aligned. Isolating a foundational dep behind one seam crate is exactly how hyperscalers manage blast-radius (Google's single-version-policy + one wrapper per external lib; AWS's vendored-and-wrapped deps). The explicit "don't build the control plane until scale warrants it, but enumerate the reversal triggers" is *better* engineering judgment than most hyperscaler ADRs show. Verdict aligned; no amend pressure on substance.
- **ai_slop:** Low-moderate. The 8-point "Why the original IP may have been right" + 6-row trigger table + "What this ADR does NOT relax" is thorough to the point of over-documentation — but it is genuine, decision-relevant reasoning (preserving a reversible decision's rationale), not filler. The `[[feedback-bominal-inheritance-precedence]]` link is an unresolved dangling ref (honestly flagged).
- **refinement:** Convert milestone/IP-keyed identifiers to function-named ids; resolve or drop the Bominal-inheritance wiki-link (decide whether that corpus is in-scope); add authority/masterplan front-matter; the 8-point reversal rationale could move to an appendix to keep the Decision section crisp.
- **consensus_needed:** no — the decision is sound and explicitly reversible with named triggers; no founder ruling needed unless the Bominal-corpus question is in-scope.

---

## Chunk notes for synthesis

**1. Two clean KEEPs, the rest need reference hygiene.** ADR-0083 (error-handling tiers) and ADR-0092 (dependency seam) are the strongest ADRs in the chunk — correct decisions, honest framing, mechanically verified, reversible-with-triggers. ADR-0083's front-matter (`doc_class`/`authority_tier`/`canonical_authority`/`companion_docs`/`related_adrs`) is the **template the other 5 ADRs in this chunk should be re-stamped against** to hit the masterplan-binding goal.

**2. Front-matter binding is the dominant systemic gap.** 5 of 7 (0066/0067/0069/0090/0091/0092 — all but 0083) carry only `id/status/doc_status` and NO `authority_tier`/`masterplan_ref`/`doc_class`. This directly instantiates the keystone §4 finding (8.8% ADR binding). Ironically ADR-0069 *designs* the very binding/provenance contract it fails to carry, and ADR-0090/0092 are machine-bound via `specs/*.json` gates yet still lack the ADR-level stamp.

**3. Retired-vocab cluster (the biggest amend driver).** The `docs`/`ops`/`foundry` lineage is shot through with retired terms:
   - **foundry brand:** ADR-0091 (`council-foundry`, `oya-foundry-write-gate-kernel`, "Foundry"), ADR-0066/0067/0069 (owner `axis-foundry`, "Foundry-grade observability", `internal-foundry` Cedar role). Governed by ADR-0335 (retire→intelligence) + ADR-0347 (`oya-foundry-*`→`oya-governance-*`). Founder ruling: cloud-intelligence/governance are the valid names.
   - **grit/rtk/icm/ICM:** ADR-0066 extractors, ADR-0067 BCs (`icm-browser`, `grit-status`), ADR-0069 commit/transition protocol. Retired by ADR-0116.
   - **GitHub Actions as CI:** ADR-0066 extractor + ADR-0067 `ci-runs` BC. CI is now Argo Workflows (ADR-0511) — touches the forge fault-line (keystone §5).
   - **Kafka write transport:** ADR-0091 §contract + Follow-up IP-003. Retired → Pulsar (ADR-0377-kafka-to-pulsar).
   - **M0x/M02-P0x/IP-00x milestone+IP-keyed identifiers + `lean-aN` lane names:** all of 0066/0067/0091/0092. FORBIDDEN going forward per planning-ssot-consolidation.md.

**4. The ADR-0065→0066→0067 supersession spine.** ADR-0066 is SUPERSEDE/MERGE into ADR-0067 (self-declared: "0065+0066 subsumed, `docs` µsvc retired→`ops.docs` BC"). On merge/backfill, the masterplan should carry ONE ops-console decision (0067-governed) with 0066's extractor-thesis folded in as the `docs` BC, not three parallel portal ADRs. ADR-0067 is the survivor and the masterplan-ready atom.

**5. Broken/dangling references found (call them out for the synthesis index):**
   - **ADR-0069 → ADR-0088: WRONG.** No ADR-0088 exists on disk (verified). Cited twice with two *different* titles ("foundry microservice scaffolding" vs "microservice-foundry"). Fabricated-precision; founder must say what was intended (possibly ADR-0091, possibly a never-authored scaffolding ADR).
   - **ADR-0069 → `ADR-0067-ops-oyatie-com-portal-foundation.md`: WRONG filename.** Real file is `…-hyperscaler-operations-console.md`.
   - **ADR-0067 → ~12 "Bominal ADR-NNNN" cites** and **ADR-0092 → `[[feedback-bominal-inheritance-precedence]]`**: unindexed external "Bominal" corpus. Provenance unresolved repo-internally — either an inherited series to import into the index, or fabricated-precision. **Cross-chunk action:** the synthesis should decide whether "Bominal ADRs" are in-scope; if not, every Bominal cite across the corpus is a dangling ref.
   - **ADR-0092 self-documents a real numbering collision** ("0091 was already taken, so we renumbered to 0092-0095") — confirms ADR-0091 squatted the number the seam-policy run wanted; benign but explains the 0091-vs-0092 lineage.

**6. own-vs-assemble fault-line shows up three times** (keystone §5), and SOURCE consistently lands on the *amended/staged* side: ADR-0090's amendment walks back "build all HTTP ourselves, ban axum" to "prefer low-level, axum where it pays"; ADR-0092 explicitly defers the dep control-plane until scale triggers (T1–T6); ADR-0067 is the one outlier pushing maximal own-everything (build Datadog/Grafana/PagerDuty/audit-view in-house). The two `consensus_needed=yes` items (0067 own-vs-assemble ops surfaces; 0069/0067 masterplan-authority mechanism) are the load-bearing founder questions from this chunk.

**7. Cross-chunk coupling to flag:** This 0090/0091/0092/0093/0094/0095 run is a single HTTP-backbone work-package; 0093/0094/0095 sit in the *next* chunk(s) and should be audited as one cluster with 0090/0092 (hyper seam + LatencyBudgetReporter + Handler trait + TenantSlug). ADR-0083's typed-error policy underpins all of them (`WriteGateError`, `HyperRuntimeError`, `AuditChainError`) — it is the quiet keystone for the chunk's correctness story.
