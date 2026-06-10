# Losslessness Verification — synthesis vs per-chunk audit artifacts

**Verifier lane:** independent. Trust-nothing; every row checked against the PRIMARY source (the per-chunk artifact on disk) and confirmed present-or-consciously-folded in the synthesis (`00-MASTER-REGISTER.md`, `01-ADR-DISPOSITION-TABLE.md`). Citations are `file:line` / grep evidence.

**Question:** did the synthesis drop anything worth preserving from the 54 audit artifacts — specifically every (a) `consensus_needed: yes` founder question, (b) tension/contradiction, (c) `truth_flag` of WRONG/GARBAGE/STALE — across a 13-artifact sample spanning the full ADR range, plus the 5 cross-tension + 2 hyperscaler digest top findings?

**Sample (13 of 54, spanning ADR-0001 → ADR-0514 + LINUX 0001–0026):**
`source-1` (0001–0007), `source-5` (DCIM/workflow band), `source-10` (0066–0092), `source-15` (0121–0130), `source-20` (0159–0165), `source-25` (0194–0200), `source-30` (0239–0245), `source-37` (0316–0322), `source-44` (0365–0371), `source-49` (0482/0506–0511), `source-50` (0512–0514), `linux-1` (L-0001–0007), `linux-4` (L-0022–0026).

Verdict legend: ✓ PRESENT (explicit) · ◐ FOLDED (consciously rolled up, traceable) · ✗ DROPPED (in artifact, absent from synthesis).

---

## A. `consensus_needed: yes` founder questions

| # | Artifact source (item) | In synthesis? | Evidence |
|---|---|---|---|
| 1 | src-1 ADR-0001 — "is the **six-substrate count** canonical & frozen, or drifted (agent-runtime still a substrate post-foundry-absorption; cap-registry survive 0011)?" | ◐ DROPPED-as-founder-Q / partial | 0001 row exists `01:14` (AMEND, naming 0335/0347/0362) but the *count-frozen ruling* is **not** a founder decision. `grep -rniE 'six.substrate\|substrate.{0,3}count\|count.{0,12}(canonical\|frozen)' synthesis/` → **zero** hits in 00. Folded indirectly into 0335 foundry-split coverage. **LOSS (soft).** |
| 2 | src-1 ADR-0002 — "does `oya-identity-kernel` **own** the IdP or **front** Zitadel (0187)?" | ✓ | C-4 `00:79-83`; founder-decision #5 `00:203`; exec `00:31`. |
| 3 | src-1 ADR-0007 — "Cedar **adopted** engine vs LINUX **owned** Cedar-compatible policy (linux-0021)?" | ✓ | C-5 `00:85-89`; founder-decision #6 `00:206`. |
| 4 | src-5 ADR-0032 — "DCIM in-house day-0 vs adopt OSS until owning a DC; is no-custom-silicon still right?" | ✓ | §4 `00:147,153`; §7.9 `00:207`; row `01:44` (AMEND Phase-2 re-seq). |
| 5 | src-5 ADR-0035 — "own bespoke FSM+DAG workflow engine day-0 vs adopt Temporal-class behind a port?" | ✓ | C-12 `00:111-112`; §4 `00:154`; §7.9 `00:207`. |
| 6 | src-10 ADR-0067 — "ops.oyatie.com = ~20-BC owned console vs adopt best-of-breed OSS behind thin shell?" | ◐ | row `01:78` flagged `FOUNDER-CALL (own-everything)`; rolls into §4 own-day-0-vs-own-when-proven axis + §7.8. Question preserved as a disposition flag, not verbatim. |
| 7 | src-10 ADR-0069 — "does the 9-capability artifact contract bind ADRs→masterplan (masterplan-authority) or is it subordinate to ADRs-generate-masterplan?" | ✓ | §2 PIVOTAL FORK `00:38-45`; exec #1 `00:28`; founder-decision #1 `00:199`. |
| 8 | src-10 ADR-0067 — "canonical write-gate belongs to `governance` or `intelligence` (0335 split)?" | ◐ DROPPED-as-founder-Q | narrow sub-question. No explicit "write-gate home" item in 00 (`grep 'write.gate'` → 0 hits). Folded into 0335 foundry→intelligence/governance split (pervasive). **LOSS (minor).** |
| 9 | src-15 ADR-0124 — forge fault-line / "what replaces the dead 0113/0124 merge-gate?" | ✓ | C-1 `00:63-67` ("What replaces the dead ADR-0113/0124 merge-gate?" verbatim); row `01:115` ARCHIVE→0363/0513. |
| 10 | src-20 ADR-0160 — "Flagger (0160) or Argo-Rollouts (§3/0040) canonical progressive-delivery?" | ✓ | C-7 `00:96-97`; founder-decision #10 `00:208`. |
| 11 | src-20 ADR-0163 — "rename 'environment tiers' post-0329 tier-retirement?" | ✓ | §7.12 `00:210` ("rename ADR-0163 'environment tiers'→'environment stages'"). |
| 12 | src-20 ADR-0164 — "is sovereign/air-gapped a committed masterplan FD or optional per-pack capability?" | ◐ | row `01:152` AMEND with explicit "**FD-bind?**" annotation. Question preserved as disposition note, not a top-level founder decision. |

**Note on src-25 / src-37 / src-44 / src-49 / src-50 / linux-1 / linux-4 consensus:** src-25, src-30, src-37, src-44, src-49, src-50, linux-1, linux-4 carried **0** `consensus_needed: yes` (verified by grep). Their load-bearing founder calls (identity/crypto cluster, forge, CI-destination, data-tier) surface via the cross-tension digests and are captured in C-1..C-6 (see §C below) — no per-chunk consensus item lost there.

---

## B. `truth_flag` = WRONG / GARBAGE / STALE

| # | Artifact source (item) | In synthesis? | Evidence |
|---|---|---|---|
| 1 | src-1 ADR-0005 — Kafka broker **STALE/WRONG** (Pulsar canonical); patterns survive | ✓ | row `01:18` (ARCHIVE broker/SUPERSEDE; →0377); C-9 supersession; exec `00:25` (Kafka→Pulsar). |
| 2 | src-1 ADR-0006 — **GARBAGE**: "Ontology renamed to Ontology" ×2 (destroyed Object-Graph rename) | ✓ | exec §1.2(b) `00:29`; §7.11(b) `00:209`; row `01:19` ("fix 'Ontology renamed to Ontology' ×2"). Founder-flagged garbage class fully surfaced. |
| 3 | src-2 (via src-1 cross-ref) — `KCminimum-shippable-tier` corruption of **KCMVP** | ✓ | exec §1.2(a) `00:29`; §7.11(a) `00:209` (8 files/31 occ). |
| 4 | src-5 ADR — STALE foundry-owner / VerticalId / undated W+N wrapping | ◐ | foundry-rename is the corpus-dominant AMEND driver; exec `00:25`, §7.11(d). Folded into bulk foundry→intelligence/governance wave, not per-line. |
| 5 | src-10 ADR-0067-cluster — WRONG refs + retired vocab (`rtk`/`grit`/`ICM`/`axis-foundry`) | ◐ | grit/icm cluster ARCHIVE `00:25` (0052/0053/0054/0103); foundry-rename wave §7.11(d). Folded. |
| 6 | src-15 ADR-0121 — **STALE**: kubeadm/containerd/Istio stack abandoned (Talos reversed) | ✓ | row `01:112` (`superseded`, ARCHIVE→0375, STALE, "rejects Talos — reversed"). |
| 7 | src-15 ADR-0124 — **STALE/WRONG-now**: merge-queue on dead Foundry-VCS/GitHub substrate; ~13 crates slated for deletion | ✓ | row `01:115` (`STALE/WRONG-now`, ARCHIVE salvage→Tide, →0363/0513, misaligned). |
| 8 | src-30 ADR-0239 — **STALE**: Foundry-internal-only/`audience` model killed by 0242/0244; still `Accepted`, no `superseded_by` | ✓ | row `01:217` (STALE, ARCHIVE "audience killed; absorbed by 0335", misaligned). |
| 9 | src-30 ADR-0244 — **STALE/WRONG sub-decision**: free-text `tier CHECK` column contradicts 0329 tenant-class | ✓ | row `01:222` ("tier-column→tenant-class"); vocabulary overload §7.12 `00:210`. |
| 10 | src-37 ADR-0316 — superseded-by-0329 (capability-tier vocab retired) | ✓ | row `01:266` ARCHIVE/DROP→0329; exec "4 true DROPs" `00:25`. |
| 11 | src-49 / src-50 — branch-locality (0476–0482 supersede Phase-1 predecessors on `origin/dev`) | ✓ | §8 residual `00:218`; coverage note `00:13`. |
| 12 | linux-4 ADR-0023 — security claim "unproven-pending-measurement" (honest TRUE, *not* WRONG) | ✓ (correctly NOT a defect) | ADR-0018 H2 honest-moonshot model `00:162`; assume-breach ALIGNED `00:162`. Artifact explicitly says not-WRONG; synthesis agrees. |

No WRONG/GARBAGE/STALE truth-flag in the sample was lost. The two founder-flagged corruption classes (Ontology-self-rename, KCMVP) are surfaced prominently in §1.2 and §7.11.

---

## C. Cross-tension (5) + hyperscaler (2) digest top findings → in 00?

| Digest | Top finding(s) | In 00? | Evidence |
|---|---|---|---|
| ci-cd-forge-build | T-1 Argo (0511) vs oya-ci Prow (0513); T-2 webhook GitHub(0387) vs Forgejo(0374) same svc | ✓ | C-2 `00:69-72`; C-1 `00:63-67`. |
| data-storage-identity-crypto | T-1 data-tier; T-2 time/clock TrueTime; T-3 identity triple (0187/0394/0476); T-4 vector/OLAP absorb | ✓ | C-3 `00:74-77`; C-13 `00:114-115`; C-4 `00:79-83`; data-tier-boundary #4 `00:202`. |
| isolation-kernel-os-mesh | T-1 framekernel vs Talos+Kata; T-2 ADR-0023 number-collision; T-3 Rust-Talos own vs adopt | ✓ | C-6 `00:91-95` + #7 `00:205`; collision §1.7 `00:34`/§7.13 `00:211`. |
| naming-brand-vocabulary-scope | T-1 foundry leak; T-2 KCMVP corruption; T-3 tier 5-way overload; T-4 masterplan fork | ✓ | foundry §7.11(d); KCMVP §1.2/§7.11(a); tier overload §7.12 `00:210`; fork §2 `00:38-45`. |
| policy-authz-autonomy-governance | T-1 own-engine vs Cedar; T-2 phantom 0150-cedar; **T-3 autonomy-ceiling authored 3× — which T1–T4 semantics canonical (0007 advisory vs 0022 execution) + lives in intelligence or governance?** | ✓ T-1/T-2 · **✗ T-3** | C-5 `00:85-89`; phantom-0150 §1.3 `00:30` + C-5 `00:88` + §7.11(c). **T-3 NOT surfaced:** `grep -rniE 'advisory.centric\|execution.centric\|autonomy.{0,20}(diverg\|three\|semantics canonical)' synthesis/` → 0 hits; 0007/0022 rows (`01:20,34`) only note "dedupe vs 0002"/"foundry→governance". Only the *vocabulary-overload* facet survives (§7.12). **LOSS.** |
| HS build-vs-buy-scope | BVB-02 Postgres; BVB-03 0015 k8s-rewrite MISALIGNED; BVB-04 L7 BuildKit; BVB-05 Rust-Talos+kernel; BVB-06 0035; BVB-07 0032; BVB-08 IdP | ✓ | §4 `00:148-162` (0015 "most hubristic" `00:154`; L7 defer `00:155`; 0032/0035 `00:153-154`; Postgres factual-correction `00:151`; node-OS/kernel honest-moonshot `00:162`). |
| HS specific-tech-choices | A1 Jenkins MISALIGNED; A4 oya-ci trigger; B3 GitHub-substrate MISALIGNED; C2 owned-policy QUESTIONABLE; C3 Kubewarden | ✓ | §4 MISALIGNED list `00:141-142` (Jenkins, GitHub-substrate); C-2 trigger `00:71`; C-5 owned-policy `00:85-89`; Kubewarden ALIGNED `00:162`. |

All 5 cross-tension + 2 hyperscaler digests' top findings are represented **except policy cross-tension T-3** (autonomy-ceiling semantic divergence + ownership home), which is the single substantive content loss.

---

## LOSSES (present in an artifact, absent-as-such from synthesis)

1. **[MODERATE] Autonomy-ceiling semantic-divergence founder question.** *Source:* `cross-tension/policy-authz-autonomy-governance.md` T-3 (`:51-63`), echoing src-1 ADR-0007 + ADR-0022. The digest raises a real **DECISION-NEEDED-FROM-FOUNDER**: *which T1–T4 semantics are canonical (ADR-0007 advisory-centric vs ADR-0022 execution-centric), and does the autonomy ceiling live in `intelligence` (post-0335) or `governance`/policy-engine?* Synthesis 00 carries **no** corresponding founder question or contradiction entry; 0007/0022 disposition rows note only mechanical "dedupe vs 0002" / "foundry→governance," and §7.12 captures only the *vocabulary*-overload of "tier," not the *semantic-authority* or *ownership-home* fork. This is a substantive founder call that is currently folded out of the gate.

2. **[SOFT] Six-substrate-count "is it frozen?" founder ruling.** *Source:* src-1 ADR-0001 `consensus_needed: yes` (`adr/source-1.md:28`). The synthesis keeps the 0001 AMEND row but does **not** ask the founder to rule whether the substrate **count** (six) is canonical/frozen vs drifted (agent-runtime-as-substrate post-foundry-absorption; cap-registry survival under 0011). Arguably folded into 0335 coverage, but the organizing-invariant ruling the artifact explicitly flagged as "founder ruling needed" is not posed.

3. **[MINOR] Write-gate ownership home (governance vs intelligence).** *Source:* src-10 ADR-0067 `consensus_needed: yes (narrow)` (`adr/source-10.md:126`). Narrow mechanical sub-question; folded into the pervasive 0335 foundry-split coverage but not posed explicitly.

**Folded-not-lost (conscious roll-ups, traceable — NOT counted as losses):** ops-console own-vs-assemble (0067 → §4 own-day-0 axis / row FOUNDER-CALL); sovereign/air-gapped FD-bind (0164 → row "FD-bind?" note); all foundry/grit/icm retired-vocab STALE lines (→ bulk-rename wave §7.11(d)); src-25 tech-substrate KEEPs (→ §4 CONFIRMED-ALIGNED list). These are defensible compressions, each still reachable.

---

## Coverage sanity

- Disposition-table per-ADR coverage independently spot-checked: every sampled ADR id (0001–0007, 0032, 0035, 0066–0092 subset, 0121–0130 subset, 0159–0165, 0194–0200, 0239–0245, 0316–0322, 0365–0371, 0482/0506–0511, 0512–0514, L-0001–0007, L-0022–0026) has a row with decision_atom + truth + disposition. Zero missing in sample.
- Truth-flag fidelity is high: STALE→STALE, STALE/WRONG-now→STALE/WRONG-now reproduced verbatim (0121, 0124, 0239, 0244).

**Bottom line:** synthesis is high-fidelity and near-lossless. Of ~40 sampled (consensus + tension + truth-flag + digest) items, **3 are lost as distinct founder-facing items** (1 moderate: autonomy-ceiling semantics/ownership; 1 soft: six-substrate-count freeze; 1 minor: write-gate home). All three are *founder questions folded into adjacent coverage* rather than dropped data — but each is a ruling the founder gate currently would not surface. Recommend adding the autonomy-ceiling fork to §3/§7 before the consensus gate.
