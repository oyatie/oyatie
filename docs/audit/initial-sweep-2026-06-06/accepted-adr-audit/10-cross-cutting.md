# 10 — CROSS-CUTTING / SYSTEMIC AUDIT of the Accepted ADR corpus

> **Lane:** CROSS-CUTTING. NOT per-ADR. Hunts systemic issues across the whole Accepted corpus: duplicate/overlap decisions, tool/vendor inconsistency vs the consolidation rulings, and layering/blast-radius/domain-cohesion violations.
> **Scope read:** 171 Accepted ADRs (`status: Accepted` / `accepted` / `Accepted (amendment)`), enumerated from a live front-matter scan of `/Users/jasonlee/Developer/source/docs/decisions/` (machine index `docs/machine-readable/decisions.json` NOT used — declared drifted).
> **Canon:** `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`.
> **Method:** every finding cites `path:line` + verbatim snippet on BOTH sides. No assertion from memory. Proposed/Superseded ADRs are flagged but not counted as live Accepted-vs-Accepted contradictions (that is the per-ADR lanes' job); where a Proposed ADR is load-bearing context it is marked **[non-Accepted, context]**.

---

## COVERAGE LEDGER (no silent caps)

- **Status scan basis:** `grep -iE "^status:[[:space:]]*accepted"` over all `*.md` in `decisions/` → **171 Accepted** files (`/tmp/accepted_files.txt`). Full corpus = 351 `.md`; non-Accepted = 92 `Proposed` + 34 `proposed` + 20 `Superseded` + 1 `superseded` + 1 `deprecated` + 1 conditional-Proposed.
- **Fully read (decision + front-matter + relevant body):** ADR-0044, 0117, 0136, 0137, 0138, 0143, 0148, 0157, 0160, 0166, 0182, 0187, 0191, 0195, 0208, 0335, 0338, 0363, 0374, 0377, 0379, 0380, 0476, 0515; plus front-matter/status of the full Cedar / CI / data-tier / foundry / identity / admission clusters (0007, 0022, 0035, 0091, 0097, 0099, 0102, 0179, 0183, 0192, 0193, 0194, 0196, 0243, 0246, 0421-phantom, 0397-phantom, 0150).
- **Corpus-wide grep sweeps (all 171 Accepted files):** `kafka` (14 files hit), `forgejo` (15 files hit), `istio` (29 files hit) — counts below are from these sweeps.
- **NOT individually deep-read (cross-cutting lane does not re-audit per-ADR):** the ~140 Accepted ADRs outside the five clusters. Their tool/vendor declarations were caught only if surfaced by the three corpus-wide greps above. **Caveat:** a duplicate decision that uses NONE of {kafka, forgejo, istio} and sits outside the five named clusters could be missed by this lane; the per-domain lanes own that.

---

## SEVERITY SUMMARY

| # | Systemic issue | Severity | Type |
|---|---|---|---|
| X1 | CI/CD cluster: 2 live Accepted Forgejo+Jenkins ADRs (0374, 0380) NOT superseded by the unifying oya-ci ADR-0515 | **CRITICAL** | duplicate/overlap + vendor-inconsistency |
| X2 | Identity: 0187 (Zitadel) and 0476 (oya-identity, supersedes phantom 0421=Keycloak) both Accepted-canonical, no supersession edge; two different vendored bridges for one slot | **CRITICAL** | duplicate/overlap + dangling-ref |
| X3 | Foundry: 6 Accepted ADRs treat Foundry as a live µservice while 0335/0363 (Accepted) retire it; 0363's "eradicated" claim is false | **CRITICAL** | duplicate/overlap + false-claim |
| X4 | Admission engine: 0379 (Accepted) makes Kubewarden default, but 0338/0117/0148 (Accepted) still hard-wire Kyverno as "canonical admission gate" | **HIGH** | duplicate/overlap + stale-ref |
| X5 | Eventing: 0377 (Accepted) retires Kafka for Pulsar, but 0195 (Accepted) keeps "ClickHouse MV + Kafka Engine" as the default stream substrate + title; 0377 cites phantom ADR-0397 | **HIGH** | vendor-inconsistency + dangling-ref |
| X6 | Forgejo persists across 15 Accepted ADRs after D2/D-FORGE-CLARIFY DROPPED Forgejo entirely (GitHub-interim → bespoke Sapling SCM) | **HIGH** | vendor-inconsistency (consolidation ruling) |
| X7 | Progressive delivery: 0160 (Accepted) adopts Flagger; canon D10 supersedes with Argo Rollouts; 0515 (Accepted) re-decides CD as Argo-CD/Rollouts-behind-port — 0160 not superseded by 0515 | **HIGH** | duplicate/overlap + vendor-inconsistency |
| X8 | Gateway version drift: 0157 pins Envoy Gateway **1.1** / Envoy **1.30**; 0182 pins Envoy Gateway **1.8.0** — two Accepted ADRs, same substrate, different versions | **MED** | vendor/version-inconsistency |
| X9 | Phantom-anchor 0150: 0148 (Accepted) cites "ADR-0150 — Kubernetes policy engine separation" but 0150 IS cursor-pagination; the real policy-engine-separation ADR (0183) is now Superseded | **MED** | dangling-ref / layering |
| X10 | 0044 (Proposed) duplicates the Accepted 0148 mesh decision (Istio-Ambient+Envoy vs Cilium+Istio-Ambient) — un-reconciled overlap | **MED** | duplicate/overlap [non-Accepted, context] |
| X11 | Layering/EventBus drift inside 0515: names "NATS JetStream / Kafka" for the CI EventBus port, contradicting the Pulsar canon (D-EVENT/0377) it post-dates | **MED** | layering / vendor-inconsistency |

---

## X1 — CI/CD CLUSTER: live Forgejo+Jenkins Accepted ADRs not superseded by the unifying oya-ci ADR (CRITICAL)

The single largest blast-radius systemic issue. **ADR-0515** (the unified Rust-native CI/CD product) was authored to "resolve the entire CI/CD ADR cluster" and supersedes seven ADRs — **but it does NOT supersede the two Accepted ADRs that mandate the Forgejo→Jenkins pipeline (0374, 0380).** They remain live, equal-authority, non-superseded Accepted decisions that contradict 0515 on forge, CI engine, and gate runner.

**0515 supersession set (excludes 0374/0380):**
- `ADR-0515-oya-ci-cd-unified-rust-native-cicd.md:9` — `supersedes: [ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511, ADR-0513, ADR-0514]`
- `ADR-0515-...:13` — `related: [ADR-0111, ADR-0116, ADR-0181, ADR-0247, ADR-0363, ADR-0366, ADR-0367, ADR-0369, ADR-0374, ADR-0392, ADR-0408, ADR-0131, ADR-0512]` → **0374 is merely `related`, not superseded; 0380 is absent entirely.**

**The live contradicting Accepted ADRs:**
- `ADR-0374-ci-webhook-gateway-forgejo-jenkins.md:9` — `superseded_by: []` (nothing supersedes it)
- `ADR-0374-...:24-27` — *"Define the CI webhook gateway — the missing trigger that turns a Forgejo … (git + Jenkins + self-hosted Forgejo) — so PRs against dev are gated by REAL automated checks (Jenkins posts the required Forgejo commit statuses)"*
- `ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment.md:9` — `superseded_by: []`
- `ADR-0380-...:55` (title) — *"CI-loop closure on Talos: Jenkins farm re-establishment + Forgejo gating"*
- `ADR-0380-...:29-30` — the gated pipeline *"runs `oya gate run-all` … and posts Forgejo commit-status (success/failure) via the forgejo-ci-token credential."*

**vs the unifying decision:**
- `ADR-0515-...:65` — *"Build `oya-ci` + `oya-cd`: a bespoke, Rust-native, cloud-native (kube-rs on Talos) CI/CD product that reimplements the patterns of Prow + Tekton + Argo Workflows + Argo CD + Argo Rollouts in Rust"*
- `ADR-0515-...:39` — *"It supersedes ADR-0124/0349/0359/0361/0511/0513/0514 … the physical Jenkins/Argo scaffold … remain operative-but-unratified as an explicit bridge until cutover"*

**Canon position:** D3/D-CICD ratifies the unified oya-ci; D-SEQUENCE explicitly says *"ARCHIVE/DROP 0349+0361 … 0511 Argo … only oya-ci 0513 is ratified canon; the physical Jenkins scaffold … stay OPERATIVE as an explicitly-UNRATIFIED de-facto bridge"* (`decision-record-oyatie-canon.md:78`). The canon treats Jenkins as **unratified bridge** — but in the live Accepted corpus 0374 and 0380 are **ratified canon** (status Accepted, not superseded). **Systemic gap:** 0515's supersession set is incomplete; the bridge-vs-ratified distinction the canon relies on is NOT expressed in the ADR front-matter graph. Per build-first-cutover-later the correct edge is `0374.superseded_by:[0515]` / `0380.superseded_by:[0515]` marked "superseded-on-cutover," matching how 0349/0359/0361 are handled.

---

## X2 — IDENTITY: two Accepted canonical IdP ADRs, no supersession edge, conflicting vendored bridges, phantom supersede target (CRITICAL)

Two Accepted ADRs each declare THE canonical human-identity substrate, name DIFFERENT vendored Phase-1 bridges, and are linked by NO supersession edge. The later one supersedes a non-existent ADR.

- `ADR-0187-canonical-oidc-idp-zitadel-primary.md:3` — `status: Accepted`; `:8` — `superseded_by: []`
- `ADR-0187-...:21` — *"Names Zitadel v2.55+ … as the canonical Identity Provider (IdP) … and the authoritative origin of OIDC ID-tokens, SAML assertions, SCIM 2.0 endpoints, and WebAuthn/Passkey credentials across the oyatie fleet."*
- `ADR-0187-...:150` — its own endgame is a bespoke `oya-identity-server` (Phase 2) fronting Zitadel as the Phase-1 bridge.

**vs:**
- `ADR-0476-oya-identity-bespoke-human-identity.md:4` — `status: Accepted`; `:9` — `supersedes: [ADR-0421]`; `:10` — `superseded_by: []`
- `ADR-0476-...:18` — *"Supersedes ADR-0421 (Keycloak)."*
- `ADR-0476-...:36-37` — *"`microservices/oya-identity/`. Keycloak (ADR-0421) is the Phase-1 bridge; oya-identity is the canonical long-term target."*
- `ADR-0476-...:103` — alternatives table demotes **Zitadel**: *"Go-based; newer; smaller federation adoption; same Go-stack objection"* — i.e. 0476 rejects the very vendor 0187 made canonical.

**Dangling reference:** `ADR-0421` (Keycloak) **does not exist** in `decisions/` (verified: glob `ADR-0421*` → no matches). So 0476's only supersession edge points at a phantom, while the ADR it actually conflicts with (0187, Zitadel) is left live and un-superseded.

**Net:** THREE incompatible identity narratives in the canon corpus — 0187 (Zitadel→in-house), 0476 (Keycloak→oya-identity), and phantom 0421 — with two of them carrying `status: Accepted`.

**Canon position:** D5 (`decision-record-oyatie-canon.md:31`): *"0476 `supersedes:[0187]` (fix the 0421 mis-number); 0187 status → superseded-as-endpoint/bridge-retained; resolve the hard contradiction (C-4)."* The canon has already diagnosed this exactly; the systemic point is that BOTH remain Accepted in source today and the supersede edge points at a number that was never assigned.

---

## X3 — FOUNDRY: live µservice ADRs coexist with the retirement ADRs; "eradicated" claim is false (CRITICAL)

The Accepted corpus simultaneously (a) retires the Foundry µservice and (b) keeps six Accepted ADRs that operate Foundry as a live, bounded-context µservice with write-gates, release pointers, and settings templates. None of the six is superseded.

**Retirement side (Accepted):**
- `ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md:3` — `status: Accepted`; title `:60` — *"foundry µservice retired; absorbed by intelligence; Hermes terminology dropped"*; it only **amends** (not supersedes) 0136/0138 (`:36-40`).
- `ADR-0363-...:35` — *"The Foundry name was eradicated (ADR-0362 + the #181–#184 cutover): the former `oya-foundry-*` crates were renamed … `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell."*

**Live-µservice side (Accepted, all `supersedes/superseded_by: []`):**
- `ADR-0136-foundry-as-single-microservice.md:3,7,8` — `status: Accepted`, `supersedes: []`, `superseded_by: []`; title `:31` *"Foundry as a single µservice (with internal bounded contexts)"*
- `ADR-0137-foundry-bounded-contexts.md:3,7,8` — Accepted, no supersede edges; *"Foundry bounded contexts"*
- `ADR-0138-foundry-six-path-deprecation.md:3,7,8` — Accepted, no supersede edges
- `ADR-0143-foundry-per-bc-release-pointer.md:3,7,8` — Accepted, no supersede edges; *"Foundry per-BC release pointer"*
- `ADR-0091-foundry-write-gate-foundations.md:9` — *"Status: Accepted"*; *"Foundry write-gate foundations (Phase 05 contract)"*
- `ADR-0102-foundry-settings-template-canonical-rendering.md:4` — `status: Accepted`; *"Foundry Settings Template Canonical Rendering"*
- (also `ADR-0097` Accepted — `oya-foundry-account-adapter-*` rename rule)

**False-claim (self-contradicting within Accepted corpus):** 0363's *"The Foundry name was eradicated"* (`:35`) is contradicted by the live `microservices/foundry/` shell it admits in the same sentence, and by the canon's verified residue: `decision-record-oyatie-canon.md:204` — *"Amend ADR-0363 to fix BOTH its false 'The Foundry name was eradicated' claim AND its now-stale 3-way … ~4,746 tracked files mention `foundry`."*

**Internal 0363 incoherence:** 0363 title (`:20`) says *"Retire bespoke agentic-VCS; Foundry→Intelligence"* yet `:35` says the agentic-VCS crates were **renamed** to `oya-vcs-*` (20 crates) — retired AND renamed-to-a-live-namespace at once.

---

## X4 — ADMISSION ENGINE: Kubewarden-default (0379) vs Kyverno hard-wired across 0338/0117/0148 (HIGH)

0379 (Accepted) flips the default admission engine from Kyverno to Kubewarden and supersedes 0183 (correctly: 0183 is now `Superseded`). But three OTHER Accepted ADRs still hard-wire **Kyverno** as the canonical admission gate and were never updated.

- `ADR-0379-kubewarden-default-admission-substrate.md:18-22` — *"Make Kubewarden the DEFAULT Kubernetes admission/policy substrate, with Kyverno retained as a first-class adapter — superseding ADR-0183's choice of Kyverno as the default admission engine … only the admission engine changes."*; title `:30` — *"supersedes ADR-0183"* ✓

**vs still-Kyverno Accepted ADRs:**
- `ADR-0338-...:229-233` — *"A.7 Named pressure: Kyverno admission is the canonical gate per ADR-0183 … This ADR's D-5 Kyverno policy (`enforce-pod-runtime-tier`) is the canonical admission gate."* (also `:112, :132, :244, :270, :278` all Kyverno; cites the now-Superseded 0183 as live authority)
- `ADR-0117-repo-hygiene-...:13,25-27` — *"consolidate kyverno admission"* … *"the established admission-policy root at `infra/kyverno/` … Maintaining two parallel admission roots … "* — consolidates everything under a **Kyverno** root, never reconciled to Kubewarden.
- `ADR-0148-service-mesh-cilium-ambient-layered.md:257` — *"ADR-0150 — Kubernetes policy engine separation (Cedar app authz vs Kyverno admission)."* (double-defective: wrong engine AND phantom anchor — see X9.)

**Systemic point:** 0379 changed the engine but only superseded ONE consumer (0183). The admission-engine decision now lives inconsistently in 4 Accepted ADRs (Kubewarden in 1, Kyverno in 3) — the cohesion gate would not have caught this because none carry a supersede edge to each other. (Note ADR-0034 taxonomy entry also tags 0338 as "Kyverno→Kubewarden" pending — the drift is known but unfixed in the live corpus.)

---

## X5 — EVENTING: Pulsar-retires-Kafka (0377) vs Kafka-Engine-default (0195); phantom ADR-0397 (HIGH)

0377 (Accepted) retires Kafka in favor of Pulsar, but its own provenance cites a non-existent ADR, and a sibling Accepted ADR (0195) keeps "Kafka Engine" as the canonical default stream-processing substrate in its title and prose.

- `ADR-0377-kafka-to-pulsar-via-kop.md:4` — title *"Migrate Kafka to Pulsar via KoP wire-compat"*; `:9` — `supersedes: [ADR-0005]`; `:30` — *"Retire standalone Kafka as a cluster substrate."*
- `ADR-0377-...:22` — *"ADR-0397 (this session) then confirmed Pulsar 4.x + Oxia as the canonical event-bus, superseding any competing choice."* → **ADR-0397 does not exist** in `decisions/` (verified: glob `ADR-0397*` → no matches). Dangling provenance in an Accepted ADR.

**vs Kafka-Engine still canonical (Accepted):**
- `ADR-0195-stream-processing-tier.md:15` (title) — *"Stream processing tier: ClickHouse Materialized Views + Kafka Engine default; Apache Flink 2.2 escalation …"*
- `ADR-0195-...:19` — *"ClickHouse Materialized Views + Kafka Engine are the default for the overwhelming majority of stream workloads"*
- `ADR-0195-...:67` (header) — *"### Default: ClickHouse Materialized Views + Kafka Engine"*

**Partial reconcile (the inconsistency is real but mitigated):** 0195's body DOES route the source to Pulsar — `:69-71`: *"Events land in the log-broker substrate (Apache Pulsar 4.2.x …). ClickHouse `Kafka` engine connects to Pulsar's Kafka-protocol endpoint as a consumer."* So the *substrate* is Pulsar, but the *title + default-tier naming + 15 Kafka mentions* still brand the decision around "Kafka Engine." Canon D-EVENT (`decision-record-oyatie-canon.md:147`): *"PULSAR is the canonical eventing/streaming bridge. Keep the Kafka→Pulsar ruling; the NATS-JetStream + Redpanda framing … is STALE."* **Kafka residue spans 14 Accepted ADRs** (0377:23 hits, 0195:15, 0166:7, 0091:4, 0193:2, 0192:2, 0169:2, 0062:2, 0515:1, 0350:1, 0122:1, 0060:1, 0059:1, 0015:1) — most are wire-compat/historical references, but 0195 is the one where Kafka still names a *default substrate*.

---

## X6 — FORGEJO PERSISTS ACROSS 15 ACCEPTED ADRs after the consolidation DROPPED it (HIGH)

Canon D2 + D-FORGE-CLARIFY rule that **Forgejo is DROPPED entirely** — not a bridge — with GitHub as the only interim forge and a bespoke Sapling-inspired SCM as the endpoint:
- `decision-record-oyatie-canon.md:153` (D2) — *"GitHub `jason931225/oyatie` is the canonical forge for now … Forgejo is dropped as the transitional."*
- `decision-record-oyatie-canon.md:206-207` (D-FORGE-CLARIFY) — *"Forgejo is DROPPED entirely — NOT a bridge adapter … FIX = a systematic Forgejo-eradication sweep … sweep all 59 tracked Forgejo refs → GitHub-interim."*

**But Forgejo is named in 15 Accepted ADRs** (corpus grep, hit-count | file):
`31 ADR-0380`, `20 ADR-0374`, `19 ADR-0363`, `12 ADR-0378`, `12 ADR-0369`, `7 ADR-0391`, `5 ADR-0367`, `3 ADR-0370`, `2 ADR-0515`, `1 ADR-0482`, `1 ADR-0476`, `1 ADR-0375`, `1 ADR-0371`, `1 ADR-0366`, `1 ADR-0365`.

Representative both-sides cites:
- `ADR-0380-...:55` (title) — *"Jenkins farm re-establishment + **Forgejo** gating"*
- `ADR-0515-...:76` — *"**Forgejo-native** webhook → CloudEvent → one `Run`"* and `:96` — *"`cloud/cloud-scm` — the bespoke VCS destination (**Forgejo** transitory → bespoke; the `ForgeAdapter` seam)."*

**Systemic point:** even the NEWEST Accepted ADR (0515, dated 2026-06-06) carries the stale Forgejo framing the canon explicitly flags as a *"procedure failure — I propagated stale Forgejo framing into ADR-0515 without reconciling against D2"* (`decision-record-oyatie-canon.md:207`). The vendor inconsistency is corpus-wide and self-replicating, not isolated.

---

## X7 — PROGRESSIVE DELIVERY: Flagger (0160) vs Argo-Rollouts (canon D10 / 0515) (HIGH)

Three Accepted/canon layers disagree on the progressive-delivery controller:

- `ADR-0160-progressive-delivery-flagger.md:3` — `status: Accepted`; `:8` — `superseded_by: []`; `:15` (title) — *"Progressive Delivery via Flagger 1.x"*; `:42` — *"Oyatie adopts Flagger 1.x as the canonical progressive-delivery controller"*; `:62` (header) — *"### Why Flagger over Argo Rollouts"* (it explicitly REJECTS Argo Rollouts at `:81-85`).
- **Canon D10** (`decision-record-oyatie-canon.md:62`) — *"Ruling: **Argo Rollouts + Chaos Mesh** as vendored bridges behind ports … **Supersede Flagger (0160)**; reconcile 0040/0165."*
- `ADR-0515-...:80` (the CD face) — *"`Argo CD` (declarative GitOps) + `Argo Rollouts` (canary / blue-green / analysis) … REUSE-behind-`DeliveryPlane`"* — i.e. 0515 re-decides progressive delivery as Argo-Rollouts-behind-port.

**Systemic point:** 0160 is Accepted, un-superseded, and actively argues AGAINST the tool the canon (D10) and 0515 now adopt. Two Accepted decisions (0160, 0515) pick **opposite** progressive-delivery controllers (Flagger vs Argo Rollouts) for the same concern, with no supersession edge between them. (Note 0160 also depends on ArgoCD per "ADR-0121" — a forward/cross reference whose target was not validated in this lane.)

---

## X8 — GATEWAY VERSION DRIFT: Envoy Gateway 1.1/1.30 (0157) vs 1.8.0 (0182) (MED)

Two Accepted ADRs decide the SAME north-south gateway substrate but pin different versions — a tool/version inconsistency between co-equal Accepted ADRs (0182 even self-declares as the "architectural authority" that 0157 "picks the implementation" for, so they should agree exactly).

- `ADR-0157-api-gateway-tier.md:10` — `architectural_authority: ADR-0182 (gateway-vs-mesh separation principle; this ADR picks the implementation)`
- `ADR-0157-...:61-62` — *"Data plane: **Envoy 1.30 LTS** … Control plane: **Envoy Gateway 1.1** (Kubernetes Gateway API)."*; `:137` — *"Helm chart … ships with **Envoy Gateway 1.1**."*

**vs:**
- `ADR-0182-...:42` (header) — *"### North-south (public → cluster): **Envoy Gateway 1.8.0**"*; *"The canonical north-south substrate is **Envoy Gateway 1.8.0**"*.

Same substrate, same author-cohort, one names 1.1, the other 1.8.0. (The gateway-vs-mesh *separation* is otherwise coherent — this is a version-canon drift, not an architectural overlap.)

---

## X9 — PHANTOM-ANCHOR 0150: policy-engine-separation cited at a cursor-pagination ADR (MED)

- `ADR-0148-service-mesh-cilium-ambient-layered.md:257` — *"ADR-0150 — Kubernetes policy engine separation (Cedar app authz vs Kyverno admission)."*
- `ADR-0150-cursor-pagination-canonical.md:1` — *"# ADR-0150: Cursor Pagination Canonical"* → 0150 is **cursor pagination**, not policy-engine separation. The real policy-engine-separation ADR is **0183** (now `Superseded` by 0379).

This is the same phantom-0150 anchor the canon flags for the Cedar engine (D6 `:34`, D11(c) `:56`). Here it surfaces in the mesh ADR's reference list, compounding X4 (it points readers to a non-existent separation rule AND names the superseded engine).

---

## X10 — 0044 (Proposed) duplicates the Accepted 0148 mesh decision (MED) [non-Accepted, context]

Flagged because it is a textbook duplicate-decision, even though 0044 is not Accepted:
- `ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md:3` — `status: proposed`; `:28` — *"We adopt **Istio Ambient mode** as the canonical east-west service mesh; **Envoy** (gateway-class) as the canonical north-south edge gateway"*
- `ADR-0148-...` (Accepted) — *"Service-mesh canonical: Cilium L3/L4 + Istio Ambient L7 (layered globally; zero overlap)"*

Both decide the canonical mesh; 0148 layers Cilium under Istio-Ambient, 0044 has Istio-Ambient as the whole east-west mesh + Envoy edge. **No supersession edge** (0044 `Superseded-by: -`, 0148 does not mention 0044 anywhere — verified grep). The mesh decision exists in two ADRs with different shapes; the Proposed one should be explicitly superseded-by-0148 or dropped so it cannot be mistaken for live canon (the per-Proposed lane owns the disposition; noted here as a systemic duplicate pattern). The downstream 0160 (X7) was authored citing "Istio (ADR-0148)" as if Istio were the whole mesh — evidence the 0044/0148 ambiguity already leaked into a third ADR.

---

## X11 — LAYERING/VENDOR DRIFT INSIDE 0515: NATS/Kafka EventBus port contradicts the Pulsar canon (MED)

The newest CI/CD ADR names a stream substrate for its own EventBus port that contradicts the eventing canon it post-dates:
- `ADR-0515-...:78` (Face C) — *"`EventBus` (**NATS JetStream / Kafka**), `Scheduler` (K8s as one impl)"*
- vs D-EVENT (`decision-record-oyatie-canon.md:147`) — *"PULSAR is the canonical eventing/streaming bridge … the NATS-JetStream + Redpanda framing … is STALE."*

A second-order layering smell: oya-ci (a `cloud/` product) defining its own broker choice (NATS/Kafka) rather than consuming the canonical eventing substrate (Pulsar) is a blast-radius/cohesion concern — the CI/CD bounded context is re-deciding a data-tier concern that the eventing domain owns. (Distinct from X5: X5 is about 0195's default tier; X11 is about 0515 re-introducing NATS/Kafka by name into an Accepted ADR after the Pulsar ruling.)

---

## SYSTEMIC PATTERNS (the meta-findings)

1. **Incomplete supersession sets are the dominant failure mode.** X1 (0515 misses 0374/0380), X2 (0476 misses 0187, hits phantom 0421), X4 (0379 misses 0338/0117/0148), X7 (0515 vs 0160 no edge). A new "unifying" ADR is authored, supersedes SOME of the cluster, and leaves co-equal Accepted siblings live. The ADR supersession graph is the SSOT for "what is canon," and it is systematically under-populated — exactly the drift D-DOCTRINE (`:177-184`) predicts when there is no enforced cohesion gate.

2. **Dangling supersede/provenance edges point at phantom IDs.** 0476→0421 (X2), 0377→0397 (X5), 0148→0150-as-policy (X9). Three Accepted ADRs cite IDs that either don't exist or hold a different decision. No no-dangling-ref invariant is enforced (canon D11(c)/D13 demand one).

3. **Vendor rulings from the consolidation have NOT propagated into the Accepted corpus.** Forgejo→GitHub (X6, 15 files incl. the newest ADR-0515), Kafka→Pulsar (X5, 14 files), Flagger→Argo-Rollouts (X7), Kyverno→Kubewarden (X4). The canon rulings exist only in `decision-record-oyatie-canon.md`; the live ADRs still carry the superseded vendor names. This is the precise "drift = faulty process + enforcement" the charter (`:166, :178`) names.

4. **Bridge-vs-ratified is encoded in prose, not in the graph.** The canon's build-first-cutover-later doctrine (`:26`) wants Jenkins/Forgejo/Flagger marked "superseded-on-cutover (pending build+proof)" — but the front-matter has no such state, so these read as fully-ratified Accepted canon (X1, X6, X7). The status enum lacks a "superseded-on-cutover / bridge-unratified" value, forcing the distinction into unenforceable prose.

---

## RETURNED DIGEST (systemic contradictions + anti-patterns, cited)

- **CRITICAL X1 — CI/CD:** ADR-0515 `supersedes:[0124,0349,0359,0361,0511,0513,0514]` (`ADR-0515:9`) excludes the live Forgejo+Jenkins ADRs 0374 (`superseded_by:[]`, `ADR-0374:9,24-27`) and 0380 (`superseded_by:[]`, `ADR-0380:9,29-30,55`) → two Accepted ADRs mandate Forgejo+Jenkins CI against 0515's bespoke-Rust oya-ci (`ADR-0515:65`).
- **CRITICAL X2 — Identity:** 0187 Zitadel-canonical (`ADR-0187:21`, `superseded_by:[]`) vs 0476 oya-identity-canonical (`ADR-0476:18,36-37`) which `supersedes:[ADR-0421]` (`:9`) — ADR-0421 is **phantom** (no file); no edge links 0187↔0476; 0476 rejects Zitadel (`:103`).
- **CRITICAL X3 — Foundry:** 0335 retires foundry (`ADR-0335:60`, only amends 0136/0138) + 0363 claims it "eradicated" (`ADR-0363:35`, false per its own 597-file shell admission) while 0136/0137/0138/0143/0091/0102/0097 stay Accepted with empty supersede edges.
- **HIGH X4 — Admission:** 0379 Kubewarden-default supersedes 0183 (`ADR-0379:18-22,30`) but 0338 (`:229-233`), 0117 (`:25-27`), 0148 (`:257`) still hard-wire Kyverno citing the now-Superseded 0183.
- **HIGH X5 — Eventing:** 0377 retires Kafka (`ADR-0377:4,9,30`) citing phantom ADR-0397 (`:22`, no file) while 0195 keeps "Kafka Engine" as default-tier name (`ADR-0195:15,19,67`; substrate reconciled to Pulsar at `:69-71`).
- **HIGH X6 — Forgejo:** dropped by canon D2/D-FORGE-CLARIFY (`canon:153,206-207`) but named in 15 Accepted ADRs incl. the newest 0515 (`ADR-0515:76,96`) and 0380 title (`:55`).
- **HIGH X7 — Progressive delivery:** 0160 adopts Flagger and rejects Argo Rollouts (`ADR-0160:42,62,81-85`, `superseded_by:[]`) vs canon D10 "supersede Flagger 0160 → Argo Rollouts" (`canon:62`) and 0515 CD-face Argo-Rollouts (`ADR-0515:80`) — no supersede edge.
- **MED X8 — Gateway version:** 0157 Envoy Gateway 1.1 / Envoy 1.30 (`ADR-0157:61-62,137`) vs 0182 Envoy Gateway 1.8.0 (`ADR-0182:42`) for the same substrate (0157 declares 0182 its authority, `:10`).
- **MED X9 — Phantom 0150:** 0148 cites "ADR-0150 policy-engine separation" (`ADR-0148:257`) but 0150 is cursor-pagination (`ADR-0150:1`); real ADR is 0183 (Superseded).
- **MED X10 — Mesh dup:** 0044 (Proposed) Istio-Ambient+Envoy mesh (`ADR-0044:28`) duplicates Accepted 0148 Cilium+Istio-Ambient, no supersede edge either way.
- **MED X11 — EventBus drift:** 0515 names "NATS JetStream / Kafka" EventBus (`ADR-0515:78`) vs Pulsar canon (`canon:147`).
- **META:** incomplete supersession sets (X1,X2,X4,X7) + dangling phantom edges (0421/0397/0150-as-policy) + un-propagated vendor rulings (Forgejo/Kafka/Flagger/Kyverno) + no graph-level "bridge-unratified / superseded-on-cutover" status = the structural drift the charter (`canon:166,177-184`) targets.
