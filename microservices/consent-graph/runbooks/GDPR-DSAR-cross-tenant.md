# Runbook: GDPR-DSAR-cross-tenant (DSAR cascade across all agreements)

- Severity: P2 routine (P1 if 30d clock approaches; P0 if past 30d)
- Trigger: data subject submits DSAR (right to know, right to erasure, right to portability,
  right to restriction) involving cross-tenant data.
- Authority: GDPR Art. 15-22; KR PIPA §35; equivalent provisions; ADR-0214 §F-AGR-* + dpia.md §8.

## Scope

This runbook covers the consent-graph-specific portion of DSAR fulfillment. Each µservice that holds
subject data has its own DSAR runbook; consent-graph's role is to enumerate cross-tenant grants
affecting the subject and cascade the action.

## Step 1 — Receive + validate request (≤24h)

1. DSAR intake via privacy-portal (out-of-band µservice).
2. Privacy officer validates identity + categorizes request type.
3. Open ticket `dsar-<id>`; bind to `subject_principal_id` + jurisdiction.

## Step 2 — Enumerate affected agreements (≤24h)

`oya consent-graph dsar enumerate --subject <subject_id> --jurisdiction <pack>`:
- Queries `consent_graph_agreements` for active agreements where
  `terms.predicate` resolves to the subject OR
  `agreement.subject_principal_id == <subject>` (B2C case).
- Output: list of agreement_ids + grantor + grantee + sharing mode + projection topics.

## Step 3 — Right-to-know (Art. 15)

For each affected agreement:
1. Compile evidence packet: agreement scope + terms + sovereignty + bilateral chain entries within
   request window.
2. Emit to privacy-portal via authenticated channel.

Time bound: 30d (GDPR) / 10d (KR PIPA).

## Step 4 — Right-to-erasure (Art. 17)

For each affected agreement:
1. Mark agreement for cascade tombstone:
   `oya consent-graph dsar tombstone --agreement <id> --subject <subject>`.
2. Emit `oya.consent-graph.dsar-cascade-tombstone` event to projection topic.
3. Grantee-side ontology projection cache subscribers tombstone rows matching subject.
4. Grantor-side ontology row tombstoned (out-of-band; ontology DSAR runbook).
5. Bilateral audit-chain entry on both sides recording the tombstone.

Time bound: 30d (GDPR) / 7d (consent-graph target).

## Step 5 — Right-to-portability (Art. 20)

For each B2C agreement:
1. Export subject's data + scope + terms + revocation history.
2. Format: structured JSON + human-readable PDF per regulator preference.
3. Deliver via privacy-portal download.

Time bound: 30d.

## Step 6 — Right-to-restriction (Art. 18)

1. Suspend all affected agreements:
   `oya consent-graph agreement suspend <id> --reason DsarRestrictionRequest`.
2. Subject's data still stored but no further projection emission.
3. Resume on request fulfillment.

## Step 7 — Bilateral notification

For each affected (grantor, grantee) pair:
1. Notify grantee via partner-directory channel with DSAR action summary (NOT the subject's identity
   unless legally required).
2. Confirm grantee acknowledges within 7d.

## Step 8 — Audit evidence + closure (≤30d total)

1. Generate evidence/dsar-cascade-<id>.json containing:
   - DSAR request metadata
   - List of affected agreements
   - Cascade events emitted
   - Grantee acknowledgments
   - Tombstone confirmations
   - Bilateral chain entry references
2. Sealed in audit-chain.
3. Privacy officer closes ticket.

## Step 9 — Reporting

- Quarterly: aggregate DSAR cascade metrics (count, mean cascade time, success rate).
- Annually: privacy review.

## Verification

- Tombstone propagation E2E test: synthetic DSAR → confirm tombstone on grantee within 7d.
- Audit-chain query confirms cascade event sealed bilaterally.

## Cross-references

- dpia.md §8 right-to-erasure
- compliance.md per-pack DSAR clauses
- runbooks/data-residency-enforcement.md (sovereignty-aware cascade)
