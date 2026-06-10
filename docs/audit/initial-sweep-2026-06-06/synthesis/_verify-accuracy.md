# _verify-accuracy — ACCURACY CHECK of the ADR-DISPOSITION-TABLE vs the REAL ADRs

**Verifier lane.** Independent. Trust nothing as-is. Verified against PRIMARY SOURCES — the real
`*.md` ADR files on disk — never against the synthesis's own claims.

- **Audited doc:** `synthesis/01-ADR-DISPOSITION-TABLE.md`
- **Primary sources:** `~/Developer/source/docs/decisions/ADR-*.md` (350 files on disk) and
  `~/Developer/linux/docs/decisions/ADR-*.md` (26 files on disk)
- **Question:** does each table row tell the truth about `{status, disposition, governing, decision_atom, truth_flag}` of the REAL ADR?
- **Sample:** 18 ADRs (14 source + 4 linux), deliberately weighted to the high-stakes/governing/phantom-ref/garble cases named in the task. Status field independently re-checked across all 18 in one consolidated grep pass.

## Verdict table

Legend: VERDICT = ok / wrong. "ok*" = faithful with a minor caveat noted, not a defect.

| id | table-says (status / disp / govern / truth) | file-says (evidence) | VERDICT | evidence (file:line) |
|---|---|---|---|---|
| s0005 | proposed(retired-in-fact) / ARCHIVE broker, SUPERSEDE patterns survive / gov `0377-kafka-to-pulsar` / PARTIAL | front-matter `status: proposed`; **0377 `supersedes: [ADR-0005]`** and §"ADR-0005 is superseded-in-part" — substrate clause superseded, outbox/at-least-once semantics carry forward | **ok** | 0005:3; 0377-kafka:9,57 |
| s0006 | accepted / KEEP+AMEND (fix "Ontology renamed to Ontology" ×2; vector→Milvus) / naming 0055/0122 / PARTIAL | `status: accepted`; the **"Ontology" renamed to "Ontology"** garble is REAL — header L11 + Context L22 (self-identical rename) | **ok** | 0006:3,11,22 |
| s0007 | proposed / AMEND (Cedar sole RBAC/ABAC + T1–T4; dedupe vs 0002) / reaffirmed 0243/0246 / TRUE / hyperscaler aligned (AWS authored Cedar) | `status: proposed`; "Cedar as the **sole** authorization policy engine for RBAC/ABAC", persona tiers T1–T4, AWS-authored Cedar | **ok** | 0007:3,20,28 |
| s0150 | Accepted / KEEP (do NOT archive; re-key map 0150≠policy-engine) / self / TRUE; atom "cursor carries scope_hash (NOT Cedar — mislabeled)" | header `- Status: Accepted`; opaque cursor MANDATORY, offset BANNED; **zero Cedar content**; `scope_hash` appears only in REJECTED-alternatives framing (keyset-without-scope_hash rejected) | **ok\*** | 0150:3,21-22,52 |
| s0183 | superseded / ARCHIVE (separation principle survives) / gov `0379 (Kubewarden)` / STALE | front-matter `status: Superseded`, `superseded_by: [ADR-0379]`; body §Status still says "Accepted (2026-05-18)" → confirms STALE internal text | **ok** | 0183:3,8,18 |
| s0187 | accepted / SUPERSEDE/AMEND (Zitadel→bridge; mark superseded_by 0476) / C-4; 0476 / TRUE (stale 0183 ref) | `status: Accepted`, `superseded_by: []` (NOT yet superseded — consistent with "mark…" = recommendation); references now-superseded 0183 | **ok** | 0187:3,8,25 |
| s0243 | Proposed / AMEND+RATIFY (phantom 0150-cedar; Kafka→Pulsar) / C-5; extends 0150(phantom) / TRUE | `status: Proposed`; amends/extends **`ADR-0150-cedar-policy-engine.md` which does NOT exist** (real 0150 = cursor-pagination) → phantom CONFIRMED; "23 policy-class decisions"; still cites Kafka (L453/462/567) | **ok** | 0243:3,17,28,141; (no 0150-cedar file) |
| s0335 | accepted / KEEP (top-tier governing; amends 0136/0138/0220/0239/0247/0255; drop Hermes; oyatie.foundry.* principal survives) / self / TRUE | `status: Accepted`; **amends list EXACT match** 0136/0138/0220/0239/0247/0255; Hermes dropped; "`oyatie.foundry.*` … principal namespace persists, the µservice does not" | **ok** | 0335:3,20-26,107,225-226,228 |
| s0364 | Accepted / KEEP (settles FORK→B; audit's own mandate) / self; depends_on 0363 / TRUE | `status: Accepted`, `depends_on: [ADR-0363]`; "`oya gen masterplan` reads accepted `planning_impact:true` ADRs" = masterplan-is-generated | **ok** | 0364:3,13,68 |
| s0365 | Accepted / KEEP (settles FORK→B) / self / TRUE | `status: Accepted`, `depends_on: [ADR-0364]`, `door: one-way`; every planning_impact ADR = pipeline output, auto-propagates regenerating masterplan | **ok** | 0365:3,12-13 |
| s0476 | Accepted / AMEND (phantom 0421/parents; add supersedes 0187; Cedar mis-cite 0083) / supersedes 0421(absent); NOT 0187 yet / PARTIAL | `status: Accepted`, `supersedes: [ADR-0421]`; **0421 ABSENT on disk** (confirmed); **ADR-0083 = rust-error-handling, NOT Cedar** → "Cedar (ADR-0083)" is a real mis-cite; Zitadel only a rejected alt (no 0187 supersession yet) | **ok** | 0476:3,9,47,103; (no 0421 file); 0083 title |
| s0511 | Proposed / AMEND→RATIFY-or-archive (reconcile w/0513) / C-2; supersedes 0359 / PARTIAL / FOUNDER-CALL | `status: Proposed`, `supersedes: [ADR-0359]`; Argo Workflows=destination, Jenkins transitory; 0359 was only Proposed | **ok** | 0511:4,11,29,57 |
| s0513 | Accepted / KEEP (founder-locked; governs CI; reconcile 0511; folds 0111/0116) / C-2; forward-governs 0380 / TRUE | `status: Accepted` (founder-locked); phased replacement of **ADR-0380**'s Jenkins+Groovy gate (forward-governs 0380); **folds 0111 + 0116** | **ok** | 0513:3,21-23,64,80 |
| L-0001 | Accepted / AMEND (finish "eliminate"→"retain" scrub L36; dead spec ref) / C-3 / PARTIAL; atom "Postgres+Citus RETAINED (not eliminated)" | `status: Accepted`; **line 36 still reads "to eliminate external DB dependencies"** (unscrubbed) while L38/L115/L136 say it does NOT eliminate Postgres+Citus (retained OLTP) → half-scrubbed = PARTIAL; line cite is EXACT | **ok** | 0001:3,36,38,115,136 |
| L-0018 | accepted-w-reservations / KEEP+cross-ref (consensus=FALSE; research-gated; honest moonshot; H0/H1 only, H2 uncommitted) / C-6 / PARTIAL | `status: accepted-with-reservations`; review_note "loop reached **consensus = FALSE**"; literal "we are the host" staged to optional uncommitted H2; commit H0/H1 | **ok** | 0018:3,14,21-24 |
| L-0021 | accepted / KEEP+cross-ref (own engine behind Cedar contract; own-vs-reuse trigger) / C-5 / TRUE; atom "typed Cedar-superset, compile-to-Rust, T1–T4, cedar-policy oracle" | `status: accepted`; "typed authorization policy language that compiles to native Rust", Cedar PARC, autonomy-tier T1–T4, `cedar-policy` vendored oracle | **ok** | 0021:3,17,21 |
| L-0023 | accepted / KEEP+cross-ref (vs source 0338 runc-for-first-party default) / C-6 / TRUE; atom "microVM secure-by-default for ALL incl first-party; supersedes provisional native-default" | `status: accepted`; "strength by blast-radius, **not authorship**"; supersedes provisional native-default-for-first-party | **ok** | 0023:3,15,19,21 |

## Summary count

- **Rows sampled:** 18 (14 source + 4 linux)
- **ok:** 18 / 18 (one of which, s0150, carries a minor non-defect caveat → "ok*")
- **wrong:** 0 / 18
- **Status-field independent recheck (all 18):** every `status:` front-matter value matches the table (incl. the bold/parenthetical/header-only renderings: 0150 header `- Status:`, 0018 `accepted-with-reservations`→`accepted-w-reservations`, 0005 `proposed`→`proposed (retired-in-fact)`). **0 status mismatches.**

## Worst errors found

**None rising to "wrong."** The disposition table is accurate on every sampled row, including the
hardest cases. The high-stakes claims that could have been fabricated all check out against the
real files:

- **Phantom-reference claims are TRUE, not invented.** `ADR-0150-cedar-policy-engine.md`
  (referenced by 0243's `amends`/`related`/`extends`) does NOT exist — the real ADR-0150 is
  cursor-pagination. `ADR-0421` (0476's `supersedes`) does NOT exist on disk. The table flags both
  precisely.
- **The "Cedar mis-cite 0083" claim is TRUE.** 0476 body cites "Cedar (ADR-0083)", but ADR-0083 is
  `rust-error-handling-tier-decision`, not Cedar. Real defect, correctly flagged.
- **The "Ontology renamed to Ontology" garble (0006) is REAL** (self-identical rename, ×2: header
  L11 + body L22).
- **L-0001's signature claim is EXACT.** Table: "finish 'eliminate'→'retain' scrub **L36**". Line 36
  literally still says "to eliminate external DB dependencies" while L38 says the opposite — a real
  half-applied scrub, and the cited line number is correct to the line.
- **Governing pointers verified against front-matter `supersedes`/`superseded_by`:** 0005←0377,
  0183→0379, 0187/0476 pair, 0359←0511, 0335 amends-list (6 ids, exact). All faithful.

### Caveats (non-defects, logged for completeness)

1. **s0150 decision_atom — mild overstatement, not wrong.** The atom states the canonical cursor
   "carries scope_hash". In the file, `scope_hash` appears only inside the REJECTED-alternatives
   block ("Keyset pagination without `scope_hash` — REJECTED"); the Decision body (L21-33) does not
   restate that the adopted cursor carries it. It is a reasonable inference, and the load-bearing
   part of the row — "**NOT Cedar — mislabeled**", "re-key the map: 0150≠policy-engine", KEEP — is
   fully correct (zero Cedar content in 0150; the 0243 phantom proves the map mis-key). Net: ok*.

2. **s0187 / s0476 "supersede" are recommendations, not on-disk state.** 0187 `superseded_by: []`
   and 0476 does not yet list 0187 — both correctly rendered by the table as proposed actions
   ("mark superseded_by 0476" / "add supersedes 0187"), not as already-applied facts. Honest.

3. **s0377 is rendered as a governing-pointer for 0005, not as its own status row.** There are two
   on-disk `ADR-0377-*` files (`kafka-to-pulsar-via-kop`, `forgejo-board-git-ref-cas-fallback`);
   the table's row-count note (line ~402) explicitly accounts for "the duplicate ADR-0377" and the
   "0377-forge variant row". Consistent with disk.

## Bottom line

On a 18-ADR sample stress-weighted toward the table's riskiest claims (phantom refs, mis-cites,
internal garbles, governing-supersession chains, both repos, all four truth_flags), the
**ADR-DISPOSITION-TABLE is faithful to the primary sources: 18/18 ok, 0 wrong.** Status fields,
dispositions, governing pointers, decision atoms, and truth_flags all hold up to file:line
evidence. The verifier could not find a single row that is wrong, overstated to the point of
falsehood, or mislabeled. The one logged caveat (s0150 scope_hash) is a soft inference inside an
otherwise-correct row.
