# IP-012 — Peer review (RadPeer-style + blinded)

`scope: oya-imaging-peer-review-app + ACR RadPeer adapter`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0243 (Cedar) + `policies/peer-reviewer-can-read-blind.cedar``

## Objective

Stand up the peer-review workflow: 5% random sample + targeted high-risk subsets, blinded view of primary read, ACR RadPeer 3-point scoring, ACR submission.

## Scope

1. Sample selection algorithm (random + targeted).
2. Blinded view (Cedar policy `peer-reviewer-can-read-blind.cedar` enforces).
3. Discordance scoring per ACR RadPeer 3-point scale.
4. Aggregate per-radiologist + per-tenant scoring.
5. ACR submission adapter.

## Acceptance criteria

- Cedar policy test: peer reviewer cannot see primary report until peer review submitted.
- Sample-selection statistical test: 5% random sample is correctly random per-radiologist.
- Aggregate per-radiologist RadPeer score correctness.

## Dependencies

- IP-009.

## Risks

- ACR RadPeer API integration; mitigate with batch submission fallback.

## Estimated effort

- 4–6 person-weeks.
