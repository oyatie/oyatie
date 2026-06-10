# ADR Audit — source-33 (coverage backfill)

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-33
- **range:** slice 225–231 of `ls -1 docs/decisions/ADR-*.md | sort`
- **ADRs reviewed (7):** ADR-0273, ADR-0276, ADR-0280, ADR-0284, ADR-0292, ADR-0293, ADR-0294
- **auditor posture:** READ-ONLY. Keystone map (`_map/canonical-posture-and-supersession-map.md`) consumed as baseline. Masterplan authored-vs-generated treated as OPEN; every masterplan binding flagged under both readings. Retired vocabulary (foundry→intelligence/governance, Kafka→Pulsar, Redis→Valkey, tier-system→tenant-class, M0–M3, cell-as-service) treated as dead per the keystone map.

> All 7 are `status: Proposed`. Each carries a `proposed_resolution` (RATIFY/DROP) per the no-unaccounted-proposals rule. None are superseded on disk; none supersede other ADRs. The cluster is internally coherent and high-craftsmanship; the principal corpus-wide problems are (a) **retired-vocabulary leakage** — "Foundry" brand (ADR-0280/0284/0293), "Kafka" substrate (ADR-0293/0294), "Foundry" in the ADR-0293 *title* — and (b) **hyperscaler-scope ambition** (per-tenant DKIM build, bespoke multi-jurisdiction HSM root ceremony) that is sound-in-shape but heavy for a pre-GA platform.

---

### ADR-0273 — Per-tenant DKIM/SPF/DMARC email deliverability

- **decision_atom:** Email deliverability is a per-tenant (not per-cluster) Tier-1 ship-blocker: every sending tenant gets its own dual-algorithm (Ed25519 + RSA-2048) DKIM keys with 90-day automated rotation, per-tenant SPF flattening, staged DMARC progression (none→quarantine→reject), BIMI, ARC, RUA ingestion, warm-up cadence, blocklist monitoring, and inbound auth+spam scoring, all wired through `cloud-secrets`/`cloud-network-dns`/`audit-chain`/`events-bus`.
- **domain:** comms-notify (secondary: crypto-keymgmt)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the per-tenant deliverability doctrine is correct and matches every serious mail operator; ratify, but amend the dead-substrate references (see truth_flag) before/at acceptance.
- **governing:** n/a (not superseded). Amends ADR-0201 (email adapter substrate).
- **truth_flag:** PARTIAL — substance TRUE; two STALE leaks: (1) §D-2 rotation runbook says it is "executable by the Foundry pipeline (per ADR-0116/0145)" — "Foundry" is retired vocabulary (→intelligence/oya-ci per ADR-0335/0347); (2) inbound spam scoring leans on the `intelligence` substrate (ADR-0255) which is correct post-rename. Minor: DMARC RUA `events-bus` could be read as Kafka-era but the ADR is substrate-neutral ("events-bus"), so it survives the Pulsar transition cleanly.
- **in_masterplan:** YES — `ship_blocker_for: [mail]`, `tier: tier-1-lockdown`; gates `mail` GA. Under generated-from-ADRs reading this is a binding deliverable; under masterplan-as-authority it must carry a `masterplan_ref` it currently lacks (8.8% binding problem applies).
- **tensions:** None doctrinal. Operational coupling to ADR-0238 (OpenBao), ADR-0240 (sovereign/air-gap), ADR-0251 (BYOK custody) — all consistent. Watch: "Foundry pipeline" reference will collide with any forge/CI auditor tracking ADR-0363 retirement.
- **hyperscaler_challenge:** ALIGNED — Google/AWS/Azure/Apple all run per-tenant or per-domain DKIM custody, dual-algorithm rollout, DMARC laddering, and FBL ingestion exactly as described; AWS SES and Google Postmaster are the named reference points. The only hyperscaler counterpoint is "buy it" (SES/SendGrid), which the ADR rejects in Alt-D on lock-in/sovereignty grounds consistent with ADR-0173. No argument for archive; argues only for the AMEND (kill "Foundry" string).
- **ai_slop:** No. Dense but every decision is load-bearing and externally checkable (RFC numbers, Gmail/Yahoo/Microsoft 2024-25 bulk rules are real).
- **refinement:** Replace "Foundry pipeline" with the intelligence/oya-ci successor name; confirm `events-bus` resolves to Pulsar (ADR-0377) not standalone Kafka.
- **consensus_needed:** Is per-tenant DKIM custody + BYOK a true GA ship-blocker for the **pilot**, or a post-GA hardening? (6 person-months of substrate eng is asserted as non-optional.)

---

### ADR-0276 — Backup + Portability Format (GDPR Article 20)

- **decision_atom:** The canonical tenant data-export/portability format is JSON-LD 1.1 bundled in tar.gz with a JSON-Schema manifest, per-µservice schemas resolved by URI, Ed25519 (tenant) + Sigstore cosign (oyatie) dual signatures, Merkle-proofed audit-chain export, full+incremental modes, cross-tenant restore, and a 5+ year (target 10) re-import compatibility commitment — satisfying GDPR Art. 20 and parallel rights (KR-PIPA 35-2, CCPA, LGPD, PIPEDA, POPIA).
- **domain:** compliance-residency (secondary: api-contracts — the format is a Tier-1 public contract)
- **current_status:** Proposed
- **disposition:** KEEP (light AMEND of one stale ref)
- **proposed_resolution:** RATIFY — format choice is correct, defensible, and reversibility-discounted; ratify as the Tier-1 portability contract.
- **governing:** n/a. Does not amend/supersede other ADRs.
- **truth_flag:** TRUE (PARTIAL on naming) — §D-4 in-scope contributor list names `foundry-self-modification-records` as a µservice contributor "for oyatie-tenant exports only." "Foundry" is retired; this should read `intelligence-self-modification-records` per ADR-0335/0255. Substance is otherwise correct. References ADR-0220 with "successor ADR-0255" annotation already in-text — good hygiene.
- **in_masterplan:** YES — `keystone_bundle: 2026-05-20-tier-1-lockdown`, `enforcement_status: advisory-until-portability-substrate-lands`, eight `oya gate validate` lanes declared. Binding deliverable under both readings; masterplan_ref absent (binding-gap applies).
- **tensions:** Mild with the LINUX own-DB fault-line (§5.1 of map): the export format is deliberately DBMS-agnostic (rejects SQL-dump Alt-C precisely because it locks to Postgres), which is *consistent* with LINUX's "eliminate Postgres" posture and with source's best-of-breed posture — so this ADR is neutral-to-helpful across the fault-line. Cross-tenant restore + principal-mapping is a large new surface (self-acknowledged).
- **hyperscaler_challenge:** ALIGNED — Google Takeout and Apple Data+Privacy are the explicit decade-stable precedents; JSON-LD + dual-signature + Merkle audit export is more rigorous than what most hyperscalers ship (they would likely *not* build Sigstore-cosign + Rekor transparency into a takeout bundle — that is a stronger posture than the reference, not a weaker one). No archive argument.
- **ai_slop:** No. Long but each alternative (protobuf, RDF, SCIM, ActivityPub, SQL-dump, streaming) is genuinely weighed and rejected on Article 20 criteria.
- **refinement:** Rename `foundry-self-modification-records` → `intelligence-self-modification-records`.
- **consensus_needed:** Is cross-tenant restore (D-10: B2C→B2B, agency-handoff, M&A) in scope for the format ADR, or should it split to its own ADR? It triples the import-side complexity.

---

### ADR-0280 — Substrate-of-Substrate Dependency Doctrine

- **decision_atom:** The Tier-1 substrate dependency graph is declared as a single canonical machine-readable acyclic DAG (`/specs/substrate-dependency-dag.json`) over ten core substrates (cell→identity→tenancy→policy-engine→cloud-secrets→audit-chain→observability→ontology→intelligence→workflow-engine), from which bootstrap order (Kahn topological sort), unidirectional failure-cascade rules, deterministic Markov SLO composition, per-edge Cedar permits, and build-time client-crate checks are all *derived* and CI-enforced (Tarjan acyclicity = BLOCKER, no exception path).
- **domain:** orchestration-scheduling (secondary: governance-process — it is a doctrine + CI-lane regime)
- **current_status:** Proposed
- **disposition:** KEEP (AMEND for Foundry/Kafka naming + DAG drift hooks)
- **proposed_resolution:** RATIFY — DAG-as-SSOT for substrate dependencies is exactly right and is the cleanest "derive-don't-author" precedent in the corpus (bootstrap order derived from DAG, not hand-authored). Ratify.
- **governing:** n/a. Amends ADR-0245 (hardens prose DAG into machine spec), ADR-0246 (locks policy-engine DAG position), ADR-0145 (constrains direct-gRPC liberty with substrate-dependency check).
- **truth_flag:** PARTIAL — doctrine TRUE; two leaks: (1) `tier_subtype: "substrate-meta"` for **Foundry** (R-11) and `axis-foundry` owner — "Foundry" retired (→intelligence-meta per ADR-0335); the meta-substrate node should be renamed. (2) Edge rationales reference "Valkey" correctly (good — Redis already retired here), so storage vocab is current. The §D-6 SLO worked example is internally consistent.
- **in_masterplan:** YES — 10 `oya gate validate` lanes + `enforced_by`; declares `/specs/substrate-dependency-dag.json` as "the canonical authority" with "every derived surface derives from this artifact." NOTE the meta-tension: this ADR asserts a *spec file* (not the ADR) is canonical authority for its domain — which leans toward the **masterplan-as-authority / specs-bind** reading, the opposite of the generated-from-ADRs reading. Flag under both: under generated-from-ADRs the DAG spec is a *deliverable* of this ADR; under specs-authority the DAG spec *is* the authority and the ADR is provenance. **This ADR is itself evidence for the open founder question.**
- **tensions:** Internal-coherent but forward-declares ADR-0258 (substrate API versioning) as a hard co-dependency ("partial acceptance rejected"); if ADR-0258 isn't real yet, this ADR's version-compatibility edges are unanchored. Cross-cutting with ADR-0176 (brown-out) which it consumes.
- **hyperscaler_challenge:** ALIGNED — Borg strata, Bezos API mandate, Hamilton LISA-2007 "dependency graph must be acyclic," Stripe service tiers, and K8s control-plane stratum are cited accurately and the pattern is genuinely universal. Google/AWS/Azure absolutely make this decision. No archive argument; if anything hyperscalers would push the DAG even harder (build-time import-graph enforcement).
- **ai_slop:** No. The Foote+Yoder "Big Ball of Mud" framing and the five-company precedent table are real and correctly applied.
- **refinement:** Rename the `substrate-meta`/Foundry node to the intelligence-meta successor; confirm ADR-0258 exists or downgrade the "partial acceptance rejected" coupling.
- **consensus_needed:** Founder question this ADR forces: does the **DAG spec** or the **ADR** hold authority for substrate dependencies? This is the authored-vs-generated question in miniature — answer it here and it generalizes.

---

### ADR-0284 — Platform-Owner-Name Indirection

- **decision_atom:** The platform-owner tenant slug (`oyatie`) is sourced from a single named constant `PLATFORM_OWNER_TENANT_SLUG` (declared once in `/specs/platform-constants.json`, generated into `oya-shared-platform-constants-kernel`), with brand display name separated from slug (Apple-Intelligence pattern, i18n'd via Fluent), a reserved-namespace root family, a 7-year backward-compat alias window, and a full rebrand migration ceremony — so a future rebrand collapses from a multi-week portfolio sed into "change one constant + rebuild + re-sign once."
- **domain:** governance-process (secondary: docs-ssot-masterplan — it is a SSOT-indirection doctrine)
- **current_status:** Proposed
- **disposition:** KEEP
- **proposed_resolution:** RATIFY — cheap, correct, one-time hardening with unbounded downside if skipped; ratify.
- **governing:** n/a. Amends ADR-0242 (sources its hardcoded `oyatie` literal from the constant).
- **truth_flag:** TRUE — and notably this ADR is itself part of the **same retired-vocabulary problem it is structurally adjacent to**: it enumerates the literal blast radius including `oyatie.foundry.*` Cedar fragments, `oyatie.foundry.ci-agent`, audit streams `oyatie.foundry`, etc. Those "foundry" sub-scopes are retired-brand strings (ADR-0335/0347/0363). This ADR indirects the *tenant slug* but the `foundry` sub-scope literals it lists are independently stale. The slug-indirection decision itself is correct and current.
- **in_masterplan:** PARTIAL — 3 `oya gate validate` lanes + `enforcement_status: advisory-until-constant-crate-lands`; `keystone_position: tier-1-lockdown`. Carries planning impact (CI lanes) but lighter masterplan binding than the others.
- **tensions:** Brushes the Forge fault-line (§5.4): D-6 rebrand ceremony lists DNS/cert/registry renames including `dsar@oyatie.com` and CT-log entries — fine, but the "external integration update" stage assumes GitHub/registry org control which intersects the founder's GitHub-migration directive vs Forgejo canon. Not a conflict, just a touchpoint. The `foundry` sub-scope literals are the real drift.
- **hyperscaler_challenge:** ALIGNED — Facebook→Meta, Google→Alphabet, Twitter→X, Slack/GitHub slug-retention-post-acquisition are cited correctly; the Apple-Intelligence brand-vs-identifier split is a real and apt pattern. Compile-time-constant-not-config (twelve-factor §III reasoning) is exactly how a hyperscaler would do it. Strong align; no archive.
- **ai_slop:** No. The 16-item "catastrophe without indirection" list and the 5 rejected alternatives (env var / config file / DB row / Makefile / chosen spec+crate) are substantive and correctly reasoned.
- **refinement:** Note (do not edit) that the `oyatie.foundry.*` sub-scope literals enumerated here are retired-brand and will themselves need the intelligence/governance rename — i.e., this indirection ADR should be amended to also indirect or rename the retired `foundry` sub-scope, otherwise it hardcodes a dead brand into the constant's documented blast radius.
- **consensus_needed:** Should the platform-owner slug indirection ALSO cover the retired `foundry` sub-scope rename in one stroke, since both are "stable internal identifier" changes?

---

### ADR-0292 — Minor User Doctrine (COPPA + KOSA + EU Age Verification)

- **decision_atom:** No B2C surface ships without the Minor User Doctrine: a single signed compliance pack (`MINOR-USER-2024`) binding per-jurisdiction age thresholds, age-assurance methods + per-jurisdiction providers, verifiable parental consent, maximal-restriction defaults, age-down protection, age-of-majority migration, algorithm transparency, marketplace/payments restrictions, mandatory audit emission (`minor_policy_decision_v1`), and per-µservice minor-aware UX bindings — centrally specified at the identity/policy/consent/audit layer and consumed uniformly, with `blocker-before-any-b2c-tenant-onboarding` enforcement.
- **domain:** compliance-residency (secondary: authz-policy — it is a Cedar pack + policy-engine doctrine)
- **current_status:** Proposed
- **disposition:** KEEP
- **proposed_resolution:** RATIFY — minor-user is the canonical non-retrofittable Tier-1 lockdown with statutory penalty exposure; ratify as a B2C precondition.
- **governing:** n/a. Amends ADR-0007 (minor persona tier), ADR-0099 (MINOR_PII data class), ADR-0218 (tenant override surface), ADR-0251 (registers the pack).
- **truth_flag:** TRUE — the jurisdiction matrix, statutory ceilings (COPPA USD 50,120/child 2024-adjusted, GDPR Art. 8, KOSA 2024 Senate 91-3, UK AADC, KR Youth Protection 2024, JP/AU/CA/BR/IN/AE/KSA) and the named enforcement actions (TikTok, YouTube $170M, Epic $275M, Meta €405M) are real and current. One minor leak: §D-3 consent records and §D-13 hot store reference the ADR-0005 outbox ("append-only outbox per ADR-0005") — ADR-0005 is the retired Kafka eventing ADR (superseded by ADR-0377→Pulsar); the *outbox pattern* survives the retirement (map §1.1 notes "outbox pattern survives, Kafka retired") so this is TRUE-but-cite-drift, not wrong.
- **in_masterplan:** YES — `keystone_bundle: 2026-05-20-b2c-tier-1-lockdown` (3-of-9), 12 `oya gate validate` lanes, `enforcement_status: blocker-before-any-b2c-tenant-onboarding`. Hard binding under both readings.
- **tensions:** Internal only. Notes its own dependency on `age-assurance` being promoted to peer substrate (ADR-0246) and on the pack registry shipping (ADR-0251) — both gating. No cross-side LINUX tension (LINUX has no B2C/minor surface).
- **hyperscaler_challenge:** ALIGNED — Apple Family Sharing/Ask-to-Buy, Google Family Link/YouTube-Kids, Microsoft Family Safety, Meta Teen Accounts, TikTok/Roblox/Discord/Snap are all cited accurately, and the "central minor flag at identity, consume uniformly, most-restrictive default, parent-mediated loosening" pattern is exactly what mature B2C platforms do post-settlement. Google/AWS(Amazon Alexa $25M)/Microsoft($20M) made this decision *after* being fined for not making it — strongest possible align. No archive.
- **ai_slop:** No. This is one of the more genuinely defensible ADRs in the corpus; the cost-of-failure asymmetry is the correct framing.
- **refinement:** Update the ADR-0005-outbox citation to the Pulsar-era successor (ADR-0377/0397) while preserving the outbox-pattern semantics.
- **consensus_needed:** None doctrinal. Scope question only: is the full 9-ADR B2C lockdown bundle a *pilot* concern or deferred until B2C is actually on the roadmap? (LINUX pilot is kernel/substrate, not B2C — this may be a SOURCE-only binding.)

---

### ADR-0293 — Foundry Meta-Trust-Root for Self-Modification Witness

- **decision_atom:** Autonomous self-modification cannot be authorized by any single operational key: a new offline-HSM `meta-trust-root` principal (5-of-9 Shamir across ≥3 jurisdictions, FIPS 140-3 L3, yearly rotation, 1-of-9 duress-revocation kill-switch) issues an independent witness signature that the Cedar self-modification permit now requires *in conjunction with* the baseline-signed-workflow predicate — closing the F5-247-01 circular-trust exploit where a compromised workflow-publisher key alone could deploy a backdoored substrate image.
- **domain:** crypto-keymgmt (secondary: security-supplychain / agentic-platform — it gates autonomous self-modification)
- **current_status:** Proposed
- **disposition:** AMEND (substance KEEP; **title + brand + Kafka must change**)
- **proposed_resolution:** RATIFY the *mechanism*, DROP-and-reissue the *name* — i.e., ratify the meta-trust-root witness + Shamir-expansion + duress kill-switch, but the ADR title and principal names are built on the retired "Foundry" brand and must be renamed before acceptance. This is the only ADR in the chunk whose *title* carries dead vocabulary ("Foundry Meta-Trust-Root").
- **governing:** n/a (closes findings, doesn't supersede). `requires_amendment_to` ADR-0247, ADR-0243, ADR-0246 (the self-modification + Cedar gate + policy-engine ADRs it hardens).
- **truth_flag:** PARTIAL / STALE-NAMING — the cryptographic decision is TRUE and excellent. But: (1) **title** = "Foundry Meta-Trust-Root"; (2) principals `oyatie.foundry.meta-trust-root`, `oyatie.foundry.meta-trust-root-attestor`, `oyatie.foundry.workflow-publisher`, `oyatie.foundry.adr-drafter` — all carry the retired `foundry` sub-scope (→intelligence per ADR-0335/0347/0363, which explicitly retired agentic-VCS/foundry naming); (3) §D-3.1 step 4 publishes the CeremonyRequest to a **"Kafka topic"** (`meta-trust-root-ceremony-queue`) and §D-7 references Kafka — Kafka is retired (→Pulsar+Oxia per ADR-0377). So the *shape* is TRUE, the *vocabulary* is STALE on three axes (brand, sub-scope, substrate).
- **in_masterplan:** YES — `keystone_bundle: 2026-05-20-foundational-doctrine`, `keystone_position: promotion-gate-fix-1-of-4`, blocks ADR-0247/0243/0246 promotion to Accepted; 5 `oya gate validate` lanes. Hard binding — it is a *promotion gate* for three other keystone ADRs, so it carries unusually high planning leverage.
- **tensions:** Directly tied to the Forge/self-modification fault-line via the retired-Foundry brand: ADR-0363 retired the agentic-VCS "foundry" and folded it into intelligence. This ADR, authored 2026-05-20, predates or ignores that retirement and re-cements `foundry` into the deepest trust-anchor names. **Sharpest naming-drift in the chunk.** Also references ADR-0294 (soak) and forthcoming ADR-0295 (bootstrap kill-switch) as composable — those are real cross-refs.
- **hyperscaler_challenge:** QUESTIONABLE (on scope, not shape) — the *pattern* (offline root, M-of-N Shamir, multi-jurisdiction, witnessed ceremony) is exactly ICANN KSK / Mozilla CA / AWS KMS / GCP CAS / DigiCert, all cited accurately. BUT no hyperscaler stands up a 17-participant, $120-180K, FIPS-140-3-L3, multi-jurisdiction root-key ceremony to gate *its own internal CI self-modification* — they gate *customer-facing root CAs*. Google/AWS/Azure would protect autonomous-deploy with hardware-backed signing + 2-person review + provenance (SLSA/Sigstore), not a DNSSEC-grade ceremony. The decision is *over-engineered for a pre-GA self-modification loop*. This argues for AMEND (right shape, right-size the ceremony to the actual blast radius and timeline) rather than archive.
- **ai_slop:** No (not slop), but **over-scoped**. Every precedent is real; the concern is proportionality, not fabrication. The autonomous-masterplan throughput cap it self-imposes (≤6 self-mods/day/workflow-class at 4h ceremony latency) is a real cost it honestly books.
- **refinement:** (1) Rename title + all `*.foundry.*` principals to the intelligence/governance successor per ADR-0335/0347; (2) replace "Kafka topic" with the Pulsar/Oxia successor per ADR-0377; (3) right-size the ceremony to match an early-stage self-modification blast radius (consider HSM + 2-of-3 + provenance as the v1, escalating to 5-of-9 multi-jurisdiction at the autonomy-tier where it is warranted).
- **consensus_needed:** Does a *pre-GA, single-founder* platform need an ICANN-grade multi-jurisdiction root ceremony to gate its own autonomous self-modification, or is that the right doctrine deferred to a later autonomy tier? (Founder question: shape-now vs build-now.)

---

### ADR-0294 — Cedar Fragment Soak + Anomaly-Rollback

- **decision_atom:** Every Cedar policy fragment must pass through a new mandatory `Soaking` lifecycle stage (≥60s shadow-mode evaluation alongside the prior fragment) during which a per-cell soak-detector watches permit/denial-rate, P99 latency, and unique-resource count against a 7-day rolling EWMA baseline and auto-revokes (within ≤500ms, via a separation-of-duty anomaly-revoker) any fragment whose behavior diverges >3σ on ≥2 sustained signals — closing the F5-243-01 hot-reload TOCTOU where a compromised pack-owner key could open a 3-second wide permit and exfiltrate before reverting.
- **domain:** authz-policy (secondary: security-supplychain)
- **current_status:** Proposed
- **disposition:** KEEP (light AMEND for Kafka)
- **proposed_resolution:** RATIFY — the soak + anomaly-rollback closes a real CRITICAL finding with a well-precedented canary pattern; ratify, with the Kafka→Pulsar substrate fix.
- **governing:** n/a. `requires_amendment_to` ADR-0243 (adds Soaking stage to fragment lifecycle), ADR-0246 (soak-window admission invariant).
- **truth_flag:** PARTIAL — decision TRUE and tightly reasoned. Leak: the detector/revoker pipeline is described over **Kafka topics** (`cedar-fragment-soak-anomaly`, `cedar-fragment-registry-revocations`, `cedar-fragment-registry-activations`, and §D-2 "Cell-local Kafka broker") — Kafka retired →Pulsar+Oxia (ADR-0377). Storage refs to "per-cell Valkey hot-cache" are CURRENT (good). So: substance TRUE, eventing substrate STALE.
- **in_masterplan:** YES — `keystone_bundle: 2026-05-20-foundational-doctrine`, `keystone_position: promotion-gate-fix-2-of-4`, blocks ADR-0243 promotion; 4 `oya gate validate` lanes + SQL CHECK constraint + migration schedule. Hard binding — promotion gate for ADR-0243.
- **tensions:** None doctrinal. Explicitly complementary to ADR-0293 (stacks: self-mod fragments need both witness AND soak) and to ADR-0295 (bootstrap kill-switch). Reverse-dependency registry to ADR-0297/0311/0313/0319 is coherent forward-pointing. The 60s soak vs emergency-forbid latency is honestly tensioned and resolved (60s floor even for emergencies).
- **hyperscaler_challenge:** ALIGNED — AWS IAM Access Analyzer reachability, Google SRE canarying (ch.16), Cloudflare Workers progressive deploy, Netflix Kayenta 3σ canary, Azure Front Door auto-rollback are cited accurately and the "policy change is a config release, canary it" framing is exactly right. Google/AWS/Azure make this decision. Welford/West online-variance + NIST SP 800-94 multi-signal correlation are correctly applied. No archive; this is among the best-engineered ADRs in the chunk.
- **ai_slop:** No. The statistical model (EWMA + EWMSD + warm-threshold + cold-baseline carve-out + false-positive budget) is real and the Rust snippet is coherent.
- **refinement:** Replace all Kafka topic/broker references with the Pulsar/Oxia successor per ADR-0377; otherwise accept as-is.
- **consensus_needed:** None — the only open item is the Kafka→Pulsar substrate substitution, which is a mechanical retired-vocab fix, not a design question.

---

## Chunk notes

**Cross-cutting findings for this 7-ADR slice (225–231):**

1. **Retired-vocabulary leakage is the dominant defect, not design error.** Six of seven ADRs are substantively TRUE and hyperscaler-ALIGNED; their problems are stale strings the keystone map already flags as dead:
   - **"Foundry" brand** → intelligence/governance (ADR-0335/0347/0363): leaks in ADR-0273 ("Foundry pipeline"), ADR-0276 (`foundry-self-modification-records`), ADR-0280 (`substrate-meta`/`axis-foundry` node, R-11), ADR-0284 (`oyatie.foundry.*` sub-scope literals enumerated as blast radius), and **ADR-0293 (in the title and the deepest trust-anchor principal names)** — the worst case.
   - **"Kafka" substrate** → Pulsar+Oxia (ADR-0377): leaks in ADR-0293 (ceremony queue) and ADR-0294 (entire detector/revoker pipeline + "cell-local Kafka broker"). ADR-0292's ADR-0005-outbox citation is the milder "outbox-pattern-survives, Kafka-ADR-retired" cite-drift.
   - **Current vocab confirmed:** "Valkey" (not Redis) appears correctly in ADR-0280 and ADR-0294 — storage rename has propagated; eventing rename has not.

2. **No unaccounted Proposed ADRs:** all 7 are Proposed; all 7 resolve to **RATIFY** on substance (ADR-0293 = RATIFY-mechanism / rename-required). Zero DROPs — none are garbage, redundant, or obsolete. The whole slice is keep-worthy decision content.

3. **ADR-0280 is a live witness to the open founder question (map §4).** It declares a *spec file* (`/specs/substrate-dependency-dag.json`) as "the canonical authority" from which the ADR-text and bootstrap order *derive*. That is the **specs-authority / masterplan-binds** posture, not the **generated-from-ADRs** posture. If the founder resolves authored-vs-generated, ADR-0280's "DAG-as-SSOT, derive-everything-else" model is the cleanest existing template for the generated direction — recommend surfacing it to the founder as a worked example either way.

4. **Hyperscaler scope-challenge clusters on the two security-ceremony ADRs.** ADR-0293 (multi-jurisdiction HSM root ceremony to gate internal self-mod) is the one QUESTIONABLE-on-scope verdict: right *shape*, over-sized *blast-radius* for a pre-GA single-founder platform. Everything else (per-tenant DKIM, JSON-LD portability, substrate DAG, slug indirection, minor-user doctrine, fragment soak) is decision-for-decision what Google/AWS/Azure actually do. The founder-level question is uniform across the chunk: **shape-now vs build-now** — these are correct doctrines whose *timing/sizing* (not correctness) is the only thing to challenge.

5. **Masterplan binding gap (map §4, 8.8% binding):** all 7 carry `enforced_by` gate lanes and keystone positions (planning impact = YES) but none carries an explicit `masterplan_ref` frontmatter. Under masterplan-as-authority they are part of the 91% unbound; under generated-from-ADRs they are themselves the authored source and the gates are their derived deliverables. Flagged under both readings per instruction; not resolved here.

6. **LINUX-pilot relevance:** ADR-0292 (minor/B2C), ADR-0273 (mail) are SOURCE-product concerns with no LINUX-pilot counterpart. ADR-0280 (substrate DAG), ADR-0284 (slug indirection), ADR-0293/0294 (policy-engine trust + soak) are substrate/governance doctrines that *would* apply to any merged platform but collide with nothing in LINUX ADR-0001..0026 directly. On merge, none of these 7 supersede a LINUX ADR; standard renumber-on-merge applies (map §6.4).
