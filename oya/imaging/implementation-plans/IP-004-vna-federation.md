# IP-004 — VNA federation (XDS-I.b + XCA-I + legacy vendor adapters)

`scope: oya-imaging-vna-federation-app + per-vendor adapter crates`
`wave_target: 16-imaging-substrate`
`adr_binding: ADR-MS-003`

## Objective

Stand up IHE XDS-I.b + XCA-I federation + per-legacy-vendor migration adapters.

## Scope

1. XDS-I.b Imaging Document Source + Imaging Document Consumer + Image Display actors.
2. XCA-I Initiating Imaging Gateway + Responding Imaging Gateway.
3. Per-legacy-vendor adapter crates: GE EA, Philips ISyntax-VNA, Sectra VNA, Fujifilm Synapse VNA, Agfa Impax, Merge VNA.
4. Migration validation: SOP Instance UID checksum + pixel SHA-256 + 1% sample-rate full-content verification + audit-chain emission.
5. Phased migration workflow per ARCHITECTURE.md §14: dual-write → backfill → read-cutover → decommission.

## Acceptance criteria

- IHE Connectathon NA + Europe pass for XDS-I.b + XCA-I.
- Per-vendor adapter test against vendor-sandbox or recorded-API fixture.
- Migration validation accuracy ≥99.999% (no false-positive missing checksums).
- Audit-chain emission for every migrated study.

## Dependencies

- IP-001, IP-002, IP-003.

## Risks

- Vendor-API drift (GE EA SOAP versioning).
- IHE Connectathon scheduling.

## Estimated effort

- 8–12 person-weeks per Wave 17-imaging-pacs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-004-vna-federation.md:16` - 4. Migration validation: SOP Instance UID checksum + pixel SHA-256 + 1% sample-rate full-content verification + audit-chain emission.; `microservices/imaging/implementation-plans/IP-004-vna-federation.md:24` - - Audit-chain emission for every migrated study..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/imaging/implementation-plans/IP-004-vna-federation.md:22` - - Per-vendor adapter test against vendor-sandbox or recorded-API fixture..
