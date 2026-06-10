# 10 — Directive-without-edge: Accepted ADRs made STALE by a later Accepted ADR or a founder consolidation-ruling, carrying NO supersession/amendment edge

> **Lane:** find the broader class of which flat-crates is one instance — an Accepted ADR that a LATER Accepted ADR (or a founder consolidation-ruling in `decision-record-oyatie-canon.md`) has rendered stale, but where NO `superseded_by` / `amended_by` / `supersedes` / `amends` edge records the relationship, so a fresh reader following the ADR graph still lands on the stale directive as live canon.
> **Method:** READ-ONLY. Every claim below carries `path:line` + a verbatim snippet from the real file. The synthesis rulings live at `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`; the ADRs live at `/Users/jasonlee/Developer/source/docs/decisions/`. The machine-readable index was NOT trusted; only the ADR files were scanned.
> **Scope honesty (NO SILENT CAPS):** I verified the specific candidate pairs named in the directive plus the directly-adjacent ADRs reachable from the same rulings (forge/CI cluster, identity, foundry, eventing, progressive-delivery). I did NOT exhaustively re-scan all ~346 Accepted ADRs for every possible later-ruling-staleness; the registers `00-MASTER-REGISTER.md` / `01-ADR-DISPOSITION-TABLE.md` are the broad-coverage instruments. Candidates I checked and EXCLUDED (with reason) are listed at the end so the cap is explicit.

---

## CONFIRMED edgeless-stale pairs (Accepted ADR + later Accepted-ADR/ruling + missing edge)

### P1 — ADR-0160 Flagger  ⟂  D10 ruling "Argo Rollouts; Supersede Flagger" + ADR-0515 Argo-Rollouts (Accepted) — **NO EDGE**

- **Stale ADR:** `ADR-0160` — **status: Accepted**, edgeless.
  - `ADR-0160-progressive-delivery-flagger.md:3` — `status: Accepted`
  - `ADR-0160-progressive-delivery-flagger.md:7-8` — `supersedes: []` / `superseded_by: []`
  - The decision picks Flagger over Argo Rollouts explicitly: `ADR-0160-progressive-delivery-flagger.md:42` — "Oyatie adopts **Flagger 1.x** as the canonical progressive-delivery controller…"; `:62` — "### Why Flagger over Argo Rollouts".
- **Superseding ruling:** `decision-record-oyatie-canon.md:62` (D10) — "**Ruling: Argo Rollouts + Chaos Mesh** as vendored bridges behind ports … Supersede Flagger (0160); reconcile 0040/0165."
- **Superseding Accepted ADR (the actual canonical adopter of Argo Rollouts):** `ADR-0515` — **status: Accepted** (`ADR-0515-oya-ci-cd-unified-rust-native-cicd.md:3`), which adopts Argo Rollouts for the CD / progressive-delivery face: `ADR-0515-…:80` — "**Argo CD / Argo Rollouts = REUSE-behind-`DeliveryPlane`** in MVP (reimplemented in Rust only when proven…)"; `:83` — "the **ratchet** … makes Argo-CD/Rollouts a REUSE-behind-`DeliveryPlane` bridge at MVP".
- **Contradiction:** ADR-0160 names **Flagger** the single canonical progressive-delivery controller and rejects Argo Rollouts; D10 + ADR-0515 make **Argo Rollouts** the canonical (bridge) controller and explicitly order Flagger superseded. Direct opposite tool picks.
- **Edge present?** **NO.** ADR-0160 `superseded_by: []` (`:8`). ADR-0515's supersedes list does NOT include 0160: `ADR-0515-…:` `supersedes: [ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511, ADR-0513, ADR-0514]` (frontmatter). Repo-wide there is no ADR whose `supersedes`/`amends` names 0160 (grep returned empty). A reader following the graph still finds Flagger as live canon.
- **Note (founder's own door):** D10 marks this **door: two-way** ("reversible tool picks") — so it is correctly NOT a one-way refoundation case, but the *edge is still owed* (the ruling literally says "Supersede Flagger (0160)").

---

### P2 — ADR-0187 "Zitadel PRIMARY"  ⟂  D5 ruling "Zitadel demoted to bridge; 0187 superseded-as-endpoint by 0476" — **NO EDGE; and the ruling's asserted edge does NOT exist on 0476**

- **Stale ADR:** `ADR-0187` — **status: Accepted**, edgeless.
  - `ADR-0187-canonical-oidc-idp-zitadel-primary.md:3` — `status: Accepted`
  - `ADR-0187-…:7-8` — `supersedes: []` / `superseded_by: []`
  - Title + decision name Zitadel the canonical/primary IdP: `ADR-0187-…:17` — "# ADR-0187 — Canonical OIDC IdP: Zitadel primary…"; `:37` — "**Zitadel v2.55+ (Apache-2.0) is the canonical IdP** … the single issuer of OIDC ID-tokens…". Zitadel is framed as a Phase-0→Phase-2 *in-house-replaceable* substrate (`:142-150`), but with Zitadel itself as the current canonical endpoint, not a demoted bridge under a bespoke owner.
- **Superseding ruling:** `decision-record-oyatie-canon.md:31` (D5) — "oya-identity (bespoke Rust, 0476 founder-locked) = owned endpoint; **Zitadel = the vendored OIDC bridge** (0187 demoted from canonical → Phase-1 bridge, superseded-as-endpoint by 0476). … **Amendments:** 0476 `supersedes:[0187]` (fix the 0421 mis-number); 0187 status → superseded-as-endpoint/bridge-retained".
- **Superseding Accepted ADR (the owned identity endpoint):** `ADR-0476` — **status: Accepted** (`ADR-0476-oya-identity-bespoke-human-identity.md:4`), the bespoke human-identity endpoint.
- **Contradiction:** ADR-0187 holds Zitadel as *canonical / primary IdP*; D5 demotes it to a *vendored bridge* behind the owned oya-identity (ADR-0476) endpoint. The frontmatter status + title still read "primary / canonical".
- **Edge present?** **NO — doubly.**
  1. ADR-0187 `superseded_by: []` (`:8`); repo-wide grep for any `supersedes/amends/…` naming 0187 returns **empty** — nothing points at 0187.
  2. The edge D5 says SHOULD exist on 0476 does **not** exist: `ADR-0476-…:9` — `supersedes: [ADR-0421]` (NOT `[ADR-0187]`). The "0421 mis-number" the ruling flags is still un-fixed; 0476 supersedes 0421, never 0187. So even the intended forward edge is absent.
- **Severity:** highest — D5 calls this "the hard contradiction (C-4)"; a fresh reader following the identity graph lands on "Zitadel is canonical" with zero pointer to the oya-identity endpoint or the demotion.

---

### P3 — ADR-0374 Forgejo→Jenkins webhook gateway  ⟂  D2 / D-FORGE-CLARIFY (Forgejo DROPPED) + D-CICD/ADR-0515 (oya-ci replaces Jenkins) — **NO EDGE**

- **Stale ADR:** `ADR-0374` — **status: Accepted**, edgeless.
  - `ADR-0374-ci-webhook-gateway-forgejo-jenkins.md:6` — `status: Accepted`
  - `ADR-0374-…:8-9` — `supersedes: []` / `superseded_by: []`
  - Substrate is Forgejo + Jenkins: `:55-56` — "The change-coordination substrate is **git + Jenkins + self-hosted Forgejo** (ADR-0363)"; `:188` — "**Decision (2026-05-26, founder): Jenkins-as-orchestrator.**"
- **Superseding rulings:**
  - `decision-record-oyatie-canon.md:153-154` (D2) — "Forgejo is **dropped** as the transitional. **Amendments authorized:** supersede/amend the Forgejo ADRs (0363/**0374**/0377-forge/0387)…"
  - `decision-record-oyatie-canon.md:207` (D-FORGE-CLARIFY) — "**Forgejo is DROPPED entirely** — NOT a bridge adapter. **GitHub is the ONLY interim forge** … FIX = a **systematic Forgejo-eradication sweep** … amend ADR-0510 + ADR-0363 + ADR-0515 + the `oya-ci-controller-forgejo-adapter` framing…"
  - `decision-record-oyatie-canon.md:169` (D-CICD) + ADR-0515 (Accepted) make oya-ci the bespoke-Rust CI/CD that replaces Jenkins.
- **Contradiction:** ADR-0374 builds and depends on a Forgejo webhook receiver dispatching to Jenkins; both halves are killed — Forgejo dropped entirely (D2/D-FORGE-CLARIFY), Jenkins superseded by oya-ci (D-CICD/0515).
- **Edge present?** **NO.** ADR-0374 `superseded_by: []` (`:9`). The only ADRs that reference 0374 do so under non-supersession edges: ADR-0513 lists it under `relates:` (`ADR-0513-…:14`, and `:25` "**retains ADR-0374's webhook gateway as the `hook`**") and ADR-0514 under `related:` (`ADR-0514-…:15`); both 0513 and 0514 are themselves **Superseded** (`ADR-0513-…:3` `status: Superseded` / `superseded_by: [ADR-0515]`; `ADR-0514-…:8` `superseded_by: [ADR-0515]`). The successor 0515 does **not** list 0374 in `supersedes` (it is only in 0515's `related`). So no live edge records 0374's staleness.

---

### P4 — ADR-0380 Jenkins-farm-on-Talos + Forgejo gating  ⟂  same D2 / D-FORGE-CLARIFY + D-CICD/ADR-0515 — **NO EDGE (and the one edge the source promises has NOT landed)**

- **Stale ADR:** `ADR-0380` — **status: Accepted (amendment)**, edgeless.
  - `ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment.md:3` — `status: Accepted (amendment)`
  - `ADR-0380-…:8-9` — `supersedes: []` / `superseded_by: []`
  - Re-establishes the Jenkins CI farm + Forgejo gating: `:55` — "# ADR-0380 — CI-loop closure on Talos: Jenkins farm re-establishment + Forgejo gating"; D1-D5 install Jenkins plugins, mint a `forgejo-ci-token`, register the Forgejo webhook (`:32-39`).
- **Superseding rulings:** identical to P3 — D2 `decision-record-oyatie-canon.md:153` (Forgejo dropped, supersede the Forgejo ADRs), D-FORGE-CLARIFY `:207` (Forgejo-eradication sweep), D-CICD `:169` + ADR-0515 (oya-ci replaces Jenkins).
- **Contradiction:** ADR-0380 builds the Jenkins-on-Talos farm and the Forgejo webhook/commit-status loop — exactly the two substrates the rulings retire.
- **Edge present?** **NO — and notably the source itself acknowledges the missing edge.** ADR-0380 `superseded_by: []` (`:9`). ADR-0513 (Superseded) states the supersession was deferred and never landed: `ADR-0513-…:22-23` — "**Phased replacement of ADR-0380's Jenkins+Groovy gate path** … **the formal supersession of ADR-0380 lands at the Phase-1 cutover (when the Jenkins gate path is deleted)**; Jenkins remains a hardened BRIDGE meanwhile." Since 0513 is itself superseded by 0515 and 0515 does NOT carry 0380 in its `supersedes` list, the promised "Phase-1 cutover" edge is now orphaned — no live ADR records 0380 as stale.

---

### P5 — ADR-0335 foundry-retired-absorbed-by-intelligence  ⟂  D-INTEL FINAL "re-home the engine DOWN into cloud/cloud-intelligence" — **NO EDGE**

- **Stale ADR:** `ADR-0335` — **status: Accepted**, edgeless on the staleness axis.
  - `ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md:3` — `status: Accepted`
  - `ADR-0335-…` frontmatter has `supersedes:` (4 foundry docs) + `amends:` (0136/0138/0220/0239/0247/0255) but **no `superseded_by` / `amended_by`** — nothing points back to flag it stale.
  - Decision homes the AI substrate in `intelligence` (the oya-side service): `:60` — "# ADR-0335: foundry µservice retired; absorbed by intelligence…"; `:158` — "D-4. `microservices/intelligence/` is the canonical AI substrate µservice"; `:514` — "C-20. The only approved AI substrate kernel surface is under the intelligence workspace."
- **Superseding ruling:** `decision-record-oyatie-canon.md:90` (D-INTEL FINAL, founder 2026-06-06) — "**RE-HOME the 96k-LOC engine DOWN from `oya/intelligence` into `cloud/cloud-intelligence`** ('cloud owns it fully') — **overrides ADR-0389's port-not-relocation lean**". I.e. the canonical AI-substrate home moves from `intelligence` (ADR-0335's target) to `cloud/cloud-intelligence`.
- **Contradiction:** ADR-0335 makes `intelligence` the single canonical AI-substrate owner; D-INTEL FINAL re-homes that engine into `cloud/cloud-intelligence` and recasts `oya-intelligence` as a thin per-tenant servicing layer (`decision-record-oyatie-canon.md:91`).
- **Edge present?** **NO.** ADR-0335 has no `superseded_by`/`amended_by`. The ADRs that reference 0335 (ADR-0338, ADR-0340, ADR-0347) cite it only as a downstream `related`/dependency consequence of the retirement, none as a superseding edge for the re-home. No live ADR records the cloud-intelligence re-home against 0335.
- **Caveat (honest):** D-INTEL FINAL is explicitly "ratchet-sequenced … NOT Wave-0" and uses build-first-cutover-later (`decision-record-oyatie-canon.md:98-99`); the founder's own note (`:99`) says Wave-0 keeps the CURRENT `intelligence` home and the re-home is "a SEPARATE later migration." So the *missing edge* here is a "superseded-on-cutover (pending build)" marker per the D-META ratchet rule (`decision-record-oyatie-canon.md:26`), not an immediate archival — but ADR-0335 currently carries NO such marker at all, so a reader sees "intelligence is the canonical AI home" as unqualified live canon.

---

## SECONDARY — same class, but the stale ADR is Proposed (NOT Accepted) → outside the strict "Accepted-stale" lane; recorded so the cap is explicit

These are the same directive-without-edge shape (a ruling renders them stale, no edge), but their `status` is Proposed, so they fall outside this lane's "Accepted ADR" gate. Flagged for the Forgejo/foundry eradication sweep regardless.

- **ADR-0387** (Forgejo→Jenkins→GitHub commit-status bridge) — `ADR-0387-ci-webhook-gateway-forgejo-to-jenkins-commit-status.md:3` `status: Proposed`; `:8-9` `supersedes: []`/`superseded_by: []`. Named in D2 (`decision-record-oyatie-canon.md:153`) for supersession; no edge.
- **ADR-0377-forgejo-board-git-ref-cas-fallback** — `:3` `status: Proposed (conditional…)`; `:8-9` edgeless. Forgejo-substrate; killed by D2/D-FORGE-CLARIFY; no edge. (Also a DUP id with ADR-0377-kafka-to-pulsar — separate id-collision issue tracked under D13/D11.)
- **ADR-0347** (foundry-fitness→governance bulk-rename) — `ADR-0347-…:3` `status: Proposed`; edgeless. D11(d) (`decision-record-oyatie-canon.md:56`) flags "0347 Proposed→Accept". Not stale-without-edge so much as un-ratified.
- **ADR-0040** (progressive-delivery-canary…) — `status: proposed`; a D10 reconcile target (`:62`), not Accepted.
- **ADR-0510** (SCM cutover trigger) — `:4` `status: Proposed`; `amends: [ADR-0363]`; its title still says "Forgejo transitory" (`:3`) which D-FORGE-CLARIFY (`:207`) flags for amendment, but it is Proposed + already carries an `amends` edge.

---

## CHECKED-and-EXCLUDED candidates from the directive (why each is NOT a confirmed "Accepted-stale-without-edge" instance)

- **"ADR-0010 Argo-Rollouts" (directive candidate):** **MIS-CITATION in the directive.** `ADR-0010-regional-pack-architecture.md:7` is "# ADR-0010: **Regional pack architecture**…" with `status: proposed` — it has nothing to do with Argo Rollouts. The D10 ruling text (`decision-record-oyatie-canon.md:62`) names only "Flagger (0160)" + "reconcile 0040/0165", NOT an "ADR-0010 Argo-Rollouts". The Argo-Rollouts adopter is ADR-0515 (Accepted) / ADR-0511 (Superseded). The real Flagger-vs-Argo pair is **P1 above (0160)**, not 0010.
- **ADR-0363** (retire-agentic-VCS; Foundry→Intelligence; Forgejo substrate): Accepted (`:3`) and its Forgejo-canonical core IS contradicted by D2/D-FORGE-CLARIFY (Forgejo dropped) — BUT it is NOT edgeless: `ADR-0363-…:10` `amended_by: [ADR-0510, ADR-0513]`, plus inline "Amended by ADR-0513 / platform-readiness" (`:26`). So it carries supersession-class edges (even if those amenders are themselves stale/superseded). It is a *stale-via-stale-chain* case, not a *no-edge* case — excluded from the strict lane but noted: the D-FORGE-CLARIFY (`:207`) and D-FOUNDRY-CLARIFY (`:204`) sweeps still need to fix its false "Foundry name was eradicated"/Forgejo-canonical text.
- **ADR-0195** (stream-processing tier; title "ClickHouse MV + **Kafka Engine** default")  vs ADR-0377-kafka-to-pulsar: **NOT a clean contradiction.** Despite the title saying "Kafka Engine", ADR-0195's body already sources events from Pulsar: `ADR-0195-stream-processing-tier.md:71` — "Events land in the log-broker substrate (**Apache Pulsar 4.2.x**; supports Kafka wire protocol via Pulsar's Kafka-on-Pulsar proxy)"; `:72` — "ClickHouse `Kafka` engine connects to **Pulsar's Kafka-protocol endpoint**". The "Kafka Engine" in the title is ClickHouse's *Kafka table engine* (a client) pointed at Pulsar's KoP wire endpoint, NOT Kafka-as-broker. ADR-0377-kafka-to-pulsar even cites 0195 as the ADR that "introduced Apache Pulsar with KoP" (`ADR-0377-kafka-to-pulsar-via-kop.md:22`, `:102`) and supersedes **ADR-0005** (the real Kafka-broker ADR), not 0195. So 0195 is consistent with D-EVENT/Pulsar (`decision-record-oyatie-canon.md:147`); the title is at most a cosmetic-clarity nit, not a stale directive. Excluded.
- **ADR-0040 / ADR-0165:** D10 says "reconcile 0040/0165" — 0040 is `proposed` (excluded as non-Accepted); 0165 (Chaos Mesh) is Accepted (`ADR-0165-…` `status: Accepted`) and D10 ADOPTS Chaos Mesh (consistent, not stale). Excluded.

---

## Summary table — confirmed Accepted-stale-without-edge

| # | Stale ADR (status) | Superseding ADR / ruling | Contradiction | Edge present? |
|---|---|---|---|---|
| P1 | ADR-0160 Flagger (Accepted) | D10 ruling (`canon:62`) + ADR-0515 Argo-Rollouts (Accepted, `0515:80,83`) | Flagger named canonical & rejects Argo Rollouts vs Argo Rollouts is canonical bridge; ruling says "Supersede Flagger" | **NO** (`0160:8` empty; not in 0515 supersedes) |
| P2 | ADR-0187 Zitadel-primary (Accepted) | D5 ruling (`canon:31`) + ADR-0476 oya-identity (Accepted) | Zitadel canonical/primary vs Zitadel demoted to vendored bridge under oya-identity endpoint | **NO** (`0187:8` empty; **and** `0476:9` supersedes 0421 not 0187 — the promised edge never landed) |
| P3 | ADR-0374 Forgejo→Jenkins gateway (Accepted) | D2 (`canon:153`) + D-FORGE-CLARIFY (`canon:207`) + D-CICD/ADR-0515 | Forgejo+Jenkins substrate vs Forgejo dropped + Jenkins→oya-ci | **NO** (`0374:9` empty; only `related` in Superseded 0513/0514; not in 0515 supersedes) |
| P4 | ADR-0380 Jenkins-farm-Talos+Forgejo (Accepted) | D2 (`canon:153`) + D-FORGE-CLARIFY (`canon:207`) + D-CICD/ADR-0515 | Jenkins farm + Forgejo gating vs both retired | **NO** — source even says edge "lands at Phase-1 cutover" (`0513:22-23`) but it never landed (0513 superseded; 0380 not in 0515 supersedes) |
| P5 | ADR-0335 foundry→intelligence (Accepted) | D-INTEL FINAL ruling (`canon:90`) | `intelligence` is canonical AI home vs engine re-homed to `cloud/cloud-intelligence` | **NO** (no `superseded_by`/`amended_by`; should carry a "superseded-on-cutover" ratchet marker per `canon:26`) |
