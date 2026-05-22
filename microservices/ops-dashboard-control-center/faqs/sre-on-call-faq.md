---
doc_class: FAQ
microservice: ops-dashboard-control-center
persona: sre-on-call + release-manager + compliance-operator
date: 2026-05-20
doc_status: published
---

# SRE / Release / Compliance Operator FAQ

## Why is every mutating API in ODCC required to carry an idempotency key?

Per AC-01 in `PRD.md`. The operator surface is the highest-blast-radius surface in the substrate: a single command can roll back a service across 1000 cells, export a 100 GB evidence pack, or place a deployment hold that blocks a feature freeze. Network retries, browser duplicate-submit, copy-paste-twice into a CLI, mobile-app duplicate-fire — all are common operator failure modes. Without the idempotency key, a deployment-rollback emitted twice in 50 ms produces two distinct operator-actions in the audit chain, each citing a different evidence pack, and the second one might roll back PAST the intended target. The key (default `uuidgen` per command) gives a server-side dedup window of 24 h. Repeated submissions of the same key return the same response without re-executing the command.

## Why are `deployment.approve`, `deployment.hold`, and `deployment.rollback` three separate commands instead of one parameterised command?

Per AC-02 in `PRD.md`. Each command type has distinct rationale fields, distinct step-up auth tier requirements, distinct Cedar gates, distinct audit-chain emission schemas, distinct retention. Approve has rationale = "why I think this is safe to promote"; hold has rationale = "why I'm pausing"; rollback has rationale = "why I'm reverting + what evidence I have". Conflating them into `deployment.action` would force a single rationale schema + single Cedar gate + single audit shape — the forensic clarity gets compromised. The pattern is borrowed from Stripe API design (per their 2023 engineering blog on idempotent-write design) + Linear's mutation-naming convention.

## Why is step-up authentication required for some actions but not all?

Per IP-008. Step-up auth costs ~ 800 ms p99 + an operator interaction (Yubikey tap or Touch ID). For low-risk reads (view dashboards, scroll cluster panels), step-up would be over-friction. For high-risk writes (rollback, evidence-export, cross-tenant peek, sovereign-cell action), step-up is mandatory. The tier classification:

- **Tier 0 (no step-up)**: dashboard reads, panel scrolls, incident-view, log-query within own tenant.
- **Tier 1 (step-up within session)**: incident-declare, deployment-approve, evidence-pack-export, cross-tenant-act (within compliance pack).
- **Tier 2 (step-up + 5-min freshness)**: deployment-rollback, deployment-hold, tenant-quarantine, cell-evacuate.
- **Tier 3 (2-person step-up + freshness)**: cross-tenant-peek-investigation, cross-pack-act, plaintext-PII-disclosure, sovereign-cell-direct-action, evidence-pack-disclose-to-non-tenant-party.

## What is the difference between "operator action" and "system action" in the audit chain?

Per `IP-009-audit-emission-integration.md`. Every audit event has `actor_type`:

- `operator`: a human or service-account operator submitted a command via ODCC.
- `system`: an automated workflow (deployment-pipeline-bot, drift-reconciler-bot) emitted via the workflow-engine.
- `tenant`: a tenant-side principal (this is the most common, but ODCC doesn't surface tenant actions; it surfaces operator + system).

Operator actions always carry: principal-ID, step-up freshness proof, idempotency-key, rationale text. System actions carry: workflow-run-ID + step-name + automated-trigger reason. The audit chain stores both in the same Merkle tree; query filters separate them.

## Why does ODCC require evidence packs to be signed by HSM (paid tenant_class) instead of software keys?

Per ADR-0251 § Per-pack compliance + FIPS 140-2 Level 3. Evidence packs are the artefact that auditors (SOC2 Type II, ISO 27001, FedRAMP 3PAO, KR-PIPA-ISMS-P) inspect to verify the operator-action chain. If the signing key is software-resident, an attacker with root on a single ODCC node can forge an evidence pack with a different content but valid signature. The HSM partition key NEVER leaves the HSM; the HSM signs the content hash + returns the signature; the bare key material is non-exportable. FIPS 140-2 Level 3 also requires physical tamper-evidence + identity-based authentication on the HSM operator. demo_trial tier uses software keys because preview cells don't go through audit — that's a documented downgrade.

## What is the `polygon zkEVM` anchor in evidence packs at paid tier?

Per ADR-0252 + ADR-0316. After HSM signing, the evidence pack's `sha256` content hash is committed to two ledgers:

1. **AWS QLDB**: a private, immutable, append-only ledger maintained by AWS. Auditable via QLDB API; provides cryptographic proof that the content hash was committed at time T.
2. **Polygon zkEVM L2 (public Ethereum L2)**: a public on-chain anchor. The content hash is committed to a smart contract `oyatie-odcc-evidence-anchor` on the L2. Anyone (including the auditee) can verify the anchor exists at block N.

Two anchors are belt-and-suspenders: QLDB is faster + has rich query, L2 is public + tamper-evident across the entire Ethereum ecosystem. For sovereign packs (KR-PIPA, FedRAMP High), the L2 anchor can be skipped if the sovereign requires on-shore-only data (the QLDB is in the sovereign region; the L2 is global).

## How does the KR-PIPA escalation overlay route incidents?

Per `IP-007-localization-escalation-runbooks.md`. When a P1 incident is declared on a pack-kr-pipa cell:

1. The incident-commander rotation defaults to `oncall-sre-kr-resident-rotation` (a KR-resident SRE on PagerDuty's Korean schedule).
2. If KR-resident SRE is unavailable within 15 min, the cross-region failover commander is `oncall-sre-ap-region-rotation` (Asia-Pacific operators).
3. The KR-PIPA Ombudsman is notified via API call to the Korean Communications Commission's reporting endpoint within 24 h (statutory per PIPA Art. 39-4 for breaches affecting ≥ 1000 data subjects).
4. The Korean-language runbook is auto-pulled (`runbooks/kr-pipa-<incident-class>-ko.md`); English-language fallback `<incident-class>.md` is available but the operator-action log records the language switch.
5. Audit-chain entries include `pack_id=pack-kr-pipa` + `kr_kic_breach_notification_clock_start=<timestamp>` + `kr_kic_breach_notification_deadline=<timestamp + 72h>`.
6. The 72-hour PIPA breach-notification clock starts; ODCC's deadline alarm fires at T+60 h if the breach-notification has not been confirmed.

Same shape applies to: EU-GDPR (72-h GDPR Art. 33 clock; lead supervisory authority is the German BfDI), US-CA-CCPA (HHS OCR for HIPAA via pack-us-healthcare), Brazil-LGPD (ANPD via pack-br-lgpd), India-DPDPA (Data Protection Board via pack-in-dpdpa).

## What is a "2-person step-up" and when does it fire?

Per IP-008 + AC-01. For Tier-3 actions (cross-tenant-peek-investigation, cross-pack-act, plaintext-PII-disclosure, sovereign-cell-direct-action), the operator + a council-architecture lead (or compliance officer, depending on action class) must each independently complete WebAuthn step-up within a 5-minute window. Both must sign the same command body (the command hash + their WebAuthn signature). Cedar gate evaluates with both principals; if either fails, the command is rejected.

Use case examples:

- A forensic investigator queries 6-month historical operator-actions across tenants → must pair with compliance-officer.
- A deployment-rollback affecting a sovereign cell (pack-fedramp-high) → must pair with council-architecture lead.
- An evidence-pack export disclosing tenant-A data to non-tenant-A counsel during litigation → must pair with general-counsel-delegate principal.

## How does ODCC enforce that an operator's tenant-posture read returns only authorized tenants?

Per AC-03. The Cedar gate `odcc::tenant::view-posture` evaluates the operator-principal's authorized tenant scope. Operators have tenant-scope declarations in their identity (per `oya identity operator show <operator-id>`): some operators are tenant-bounded (e.g., `release-mgr-acme-team` can see only `drill-acme` + `drill-acme-staging`); some are cross-tenant (e.g., `oncall-sre-syd-rotation` can see all tenants in syd cells); some are global (rare, audit-flagged on every read). When the operator queries `oya odcc tenant posture list`, the response is filtered server-side by Cedar to the principal's scope. Cross-tenant reads emit a per-tenant audit-chain seal indicating the read happened, even if the value was just a count.

## How does ODCC interact with the audit-chain µservice?

Per `IP-009-audit-emission-integration.md`. ODCC does NOT maintain its own audit log; it emits to the audit-chain µservice. Every operator action:

1. ODCC backend formats the action as `AuditEvent{actor, resource, action, decision, justification, evidence_refs, idempotency_key, ...}`.
2. ODCC submits the event to `audit-chain` via the `oya-audit-chain-client` SDK (synchronous; ODCC blocks on seal-ack).
3. `audit-chain` Ed25519-signs the event + adds to the Merkle tree leaf + returns seal reference.
4. ODCC records the seal reference + returns the command response to the operator.

If audit-chain is unreachable, ODCC fails the operator action (it is a hard dependency). This is intentional: a deployment-rollback without audit emission is a compliance hole, not a graceful degradation.

## What happens during cross-region operator handoff (follow-the-sun)?

Per paid tier. Operator-actions submitted in us-east are HLC-timestamped (per ADR-0252) + cross-region replicated to eu-west + ap-south within 100 ms p99. When the follow-the-sun handoff happens at 17:00 PT, the eu-west operator opens ODCC + sees:

- Active incidents declared by us-east operators (with full audit-chain visibility).
- Pending deployment approvals queued during us-east hours.
- Outstanding ADR-promotion-triage items.
- Tenant-posture deltas since their last shift.

For an eu-west operator to act on a us-east tenant, Cedar gate `odcc::cross-region::act` evaluates. Most tenants permit this (default); some sovereign tenants (pack-us-fedramp) explicitly forbid cross-region operator action → the eu-west operator is denied + sees a "cross-region action forbidden for this pack" message.

## Why doesn't ODCC expose a "super-admin bypass" mode for emergency?

Per ADR-0243 + project Memory `feedback_cedar_as_universal_gate.md`. There is no code path that bypasses Cedar; even break-glass actions are Cedar-evaluated against a break-glass principal (`odcc::breakglass::emergency-operator`). The break-glass principal:

- Requires 2-person step-up Tier 3.
- Carries a 60-minute time-bound credential.
- Emits an additional `breakglass_invoked` audit-chain seal.
- Triggers a PagerDuty incident to compliance-officer-rotation immediately.
- Is rate-limited (max 3 break-glass uses per pack per 30 days; ADR amendment required if more).

If the substrate is in an unrecoverable state where Cedar itself is down, ODCC fails closed (no operator actions accepted). This is intentional per `feedback_no_silent_regression.md`.

## What is the ADR-promotion-triage panel?

Per `IP-013-adr-promotion-triage-panel.md`. Council-architecture leads use this panel to:

- See proposed ADRs in `proposed` state (Wave-3-G doctrine cluster, etc.).
- Read multispectrum-review verdicts (F1..F9 + M1..M2 + A1..A7 facets).
- Approve / reject promotion to `accepted` or `superseded`.
- Trigger consensus debate via subagent spawn (per `feedback_consensus_debate_spectrum_lens_subagents.md`).

ODCC integrates with the docs lane: when an ADR is promoted, the next IP/journey/dossier wave gets the new ADR available; the lane gate enforces that no µservice code lands without an ADR.

## How does ODCC display per-tenant FinOps data without leaking cost details across tenants?

Per `IP-014-finops-portal-integration.md`. ODCC embeds a read-only FinOps panel from finops-portal. The Cedar gate `finops::tenant::view-cost` evaluates whether the operator can see the tenant. Cross-tenant cost rollups (e.g., "total cost across all my tenants") are computed server-side by finops-portal + only delivered if Cedar permits; a cross-tenant total that includes any tenant the operator can't see is server-rejected (no partial-data leakage). This pattern matches AC-03 + ADR-0244 tenant-scoping primitive.
