# ADR-MS-003 — VNA federation + legacy migration

`microservice: imaging`
`status: ACCEPTED`
`date: 2026-05-21`
`wave: 15M-G`
`authority: ADR-0132 + user directive 2026-05-21`

## Context

The VNA market is fragmented:

- **GE Enterprise Archive (EA)** — most common legacy install in large US health systems.
- **Philips ISyntax-VNA** — Carestream Vue VNA lineage.
- **Sectra VNA** — strong in Europe + Nordic.
- **Fujifilm Synapse VNA** — strong in Asia-Pacific.
- **Agfa Impax VNA** — strong in Europe.
- **Merge VNA (IBM)** — post-Francisco-Partners spin-out; many install bases looking to migrate.
- **Change Healthcare Stratus** — post-Optum acquisition; migration target.

Cross-VNA interoperability is governed by IHE profiles:

- **XDS-I.b** — Cross-Enterprise Document Sharing for Imaging (point-to-point).
- **XCA-I** — Cross-Community Access for Imaging (federated across communities).

Each legacy vendor has nominal IHE support but real-world deployments frequently rely on proprietary APIs.

## Decision

**The imaging VNA is federation-first.** XDS-I.b + XCA-I are first-class IHE actor implementations. Per-vendor legacy adapters supplement the IHE substrate.

Implementation:

1. The µservice ships XDS-I.b Imaging Document Source + Consumer + Image Display actors.
2. The µservice ships XCA-I Initiating Imaging Gateway + Responding Imaging Gateway actors.
3. Per-legacy-vendor adapters cover GE EA (SOAP + C-MOVE), Philips ISyntax-VNA (REST + C-MOVE), Sectra VNA (XDS-I.b federated), Fujifilm Synapse VNA (C-MOVE), Agfa Impax (REST + C-MOVE), Merge VNA (SOAP + C-MOVE).
4. Migration follows phased dual-write → backfill → read-cutover → decommission per ARCHITECTURE.md §14.
5. Migration validation includes per-instance SOP Instance UID checksum + per-instance pixel SHA-256 + 1% sample-rate full-content verification + audit-chain emission for every migrated study.
6. Cross-VNA queries are federated transparently via the `vnaFederationService.QueryFederated` gRPC.

## Consequences

### Positive

- Real federation, not screen-scraping.
- Tenants can phase migration without big-bang cutover.
- Hospital-to-hospital cross-enterprise sharing works out of the box.
- Audit-chain provides regulatory-grade migration evidence.

### Negative

- Per-vendor adapter maintenance burden.
- Some legacy vendors have undocumented private-tag quirks; vendor-quirks library required.

### Neutral

- XDS-I.b + XCA-I require Consistent Time (NTP <1s skew) and ATNA (audit) implementations; both are universal cross-cutting requirements anyway.

## Alternatives Considered

- **Bridge-only (no federation)**. Rejected: degrades cross-enterprise sharing.
- **Per-vendor migration only (no IHE federation)**. Rejected: locks the µservice into per-vendor adapters with no future-proof.
- **DICOMweb-only federation**. Rejected: IHE XDS-I.b / XCA-I are the regulatory standard for cross-enterprise imaging.

## References

- IHE Radiology Technical Framework Vol 2 (Transactions) + Vol 4 (Cross-Community).
- IHE XDS-I.b Supplement.
- IHE XCA-I Supplement.
- ADR-0253 (HTTP/3 + QUIC default).
- ADR-MS-001 (DICOMweb-first substrate).
