export const meta = {
  name: 'amendment-prewave0-gates',
  description: 'Pre-Wave-0 gating units for the approved amendment plan (Option 1). All READ-ONLY on source / linux-artifact-only: (A.6) bidirectional ruling-provenance verifier pass; (A.0-1) census-reconcile the rest-of-docs register 731/105 -> 831/43; (A.0-2) PRODUCE the cluster-level consolidation-design-set (provisional, pending founder freeze). NO source mutation, no auth needed. Output gates the founder A.0-2 sign-off + credential handoff.',
  phases: [
    { title: 'Provenance', detail: 'A.6 bidirectional ruling-provenance pass (read-only)' },
    { title: 'Reconcile', detail: 'A.0-1 census-reconcile the linux audit register' },
    { title: 'DesignSet', detail: 'A.0-2 produce the provisional consolidation-design-set' },
  ],
}

const WS = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06'
const PLAN = WS + '/AMENDMENT-PLAN.md'
const DR = WS + '/synthesis/decision-record-oyatie-canon.md'
const SRC = '/Users/jasonlee/Developer/source/docs/decisions'
const OUT = WS + '/_execution'
const RULE = 'You are an INDEPENDENT verifier/producer for the approved amendment plan. READ-ONLY on source (never edit any source/ file). Trust nothing as-is; ground every claim in primary sources (cite file:line / grep). Write ONLY your one named output artifact under ' + OUT + ' (and, for A.0-1 only, the named linux audit register).'

phase('Provenance')
const a6 = await agent(
  RULE + '\n§A.6 BIDIRECTIONAL RULING-PROVENANCE PASS (the plan\'s own gate before any door:one-way mutation). Read ' + PLAN + ' (esp. §A.5 table + §A.6) and the SSOT ' + DR + '. ' +
  'DIRECTION 1 (no RULED-inflation): for EVERY `[RULED]` tag / "the decision record rules" assertion in the plan, confirm a citable SSOT line exists (grep the SSOT — do not infer from "overlap"). Flag any RULED tag with no citable line. Confirm `grep double-work` over the SSOT = 0 and no fabricated quote remains. ' +
  'DIRECTION 2 (no RULED-deflation): for EVERY SSOT D-decision that names a concrete amendment ACTION (D11(a)-(d), D12, D13, D14, D15, D-EVENT, D-META, D-SAFETY, D-KR, D-DEPTH, D-RECOVER), confirm the plan carries a matching `[RULED:<line>]` tag; flag any ruled action left DERIVED/untagged. Specifically re-confirm L2-foundry-rename-sweep-membership is RULED `D11(d)+:52`. ' +
  'WRITE ' + OUT + '/A6-provenance-verdict.md (per-tag table both directions + any re-review triggers). RETURN: a terse verdict line "A6: GREEN" or "A6: RE-REVIEW (<list>)" + the evidence summary.',
  { label: 'exec:a6-provenance', phase: 'Provenance', model: 'opus' }
)

phase('Reconcile')
const a01 = await agent(
  RULE + '\nA.0-1 CENSUS-RECONCILE (linux audit artifact only — you MAY edit this one linux file, NOT source). The register ' + WS + '/docs-sweep/00-REST-OF-DOCS-REGISTER.md carries stale foundry-census figures (731 files / 105 Palantir carve-out) that contradict the SSOT census-of-record (' + DR + ' — total non-ADR foundry = 831, Palantir-Foundry carve-out = 43). VERIFY the SSOT figure yourself (grep the SSOT for 831/43 + re-grep source/docs for `foundry` non-ADR count to confirm ~831). Then EDIT 00-REST-OF-DOCS-REGISTER.md: correct the 731 file-count, the 105 carve-out figure, and the `foundry | 731` table row to 831/43, with a dated correction note pointing at the SSOT census line. Do NOT touch any other figure. WRITE a short note ' + OUT + '/A01-census-reconcile.md (what changed + the verified counts). RETURN "A0-1: DONE (831/43)" + the grep evidence.',
  { label: 'exec:a01-census', phase: 'Reconcile' }
)

phase('DesignSet')
const a02 = await agent(
  RULE + '\nA.0-2 PRODUCE THE CONSOLIDATION-DESIGN-SET (PROVISIONAL — pending founder door:one-way freeze; READ-ONLY on source). Build the cluster-level table {old Accepted source ADR -> which clean ADR-0000+ doc it is INTENDED to fold into / be archived by}, at the DESIGN-INTENT granularity the SSOT rules (D3/D5/D6/D7 cluster resolutions, the disposition table ' + WS + '/synthesis/01-ADR-DISPOSITION-TABLE.md, the Proposed ledger ' + WS + '/synthesis/03-PROPOSED-RESOLUTION-LEDGER.md). This is the partition that tells Wave-0 which files are provisionally throwaway-by-re-foundation (skip the foundry rename) vs live-and-amended (rename in place). Cover the re-foundation clusters: oya-ci (reshape 0513 + supersede/relate 0511/0124, phase 0369/0367/0366), identity (0476/0187), policy (Cedar+PARC, phantom-0150), isolation, autonomy (0007/0022), data-tier (+0005 Pulsar), masterplan-wiring (0364/0365). For each: list the old ADR ids that fold in, the target ADR-0000+ cluster, and whether each old ADR is ARCHIVED-by-refoundation vs AMENDED-in-place. Mark clearly this is the PROVISIONAL design-freeze candidate (Wave-1 L1.1-CONS-CONFIRM confirms the file-level fold; delta = re-review). WRITE ' + OUT + '/A02-consolidation-design-set.md (the table + a "STATUS: provisional — pending founder door:one-way freeze" header + a completeness note: every foundry-sense re-foundation candidate classified). RETURN the design-set summary (cluster -> folded-ids counts) + any ambiguous classifications needing founder input.',
  { label: 'exec:a02-designset', phase: 'DesignSet', model: 'opus' }
)

return {
  a6_verdict: a6,
  a01: a01,
  a02_summary: a02,
  artifacts: [OUT + '/A6-provenance-verdict.md', OUT + '/A01-census-reconcile.md', OUT + '/A02-consolidation-design-set.md'],
}
