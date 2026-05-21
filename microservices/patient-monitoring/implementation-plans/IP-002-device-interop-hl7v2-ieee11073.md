# IP-002 — Device interoperability (HL7v2 + IEEE 11073 + IHE PCD)

**Status**: drafted
**ADR binding**: ADR-MS-001 (canonical sample envelope)
**Bounded contexts**: DeviceInterop
**Owner**: axis-clinical-realtime
**Estimated effort**: 5-6 dev-weeks

## Slice 1: HL7v2 ORU^R01 listener

- Receive MLLP or TCP ORU^R01 messages.
- Parse PCD-1, PCD-3, OBR, OBX segments.
- Canonicalize to internal `VitalSignsSample` envelope.
- ACK with MSA segment per HL7v2 spec.

## Slice 2: IEEE 11073-10101 driver

- Parse Point-of-Care Medical Device frames.
- Map vendor-specific device-types to canonical `parameter_loinc`.

## Slice 3: IEEE 11073-20601 PHD driver

- Parse Personal Health Device frames (wearables + home devices).
- Support Bluetooth GATT bridge mode.

## Slice 4: IHE PCD-DEC + WCM + ACM

- IHE PCD-DEC (Device Enterprise Communication) inbound.
- IHE PCD-WCM (Waveform Content Module) inbound.
- IHE PCD-ACM (Alert Communication Management) inbound.

## Slice 5: Continua PHD bridge

- Parse Continua PHD records per IEEE 11073-10408 + 10417 + 10421.

## Slice 6: Vendor connectors

- Philips CareEvent CMS connector.
- GE Unity DPD connector.
- Mindray eGateway connector.
- Welch Allyn Connex Vitals connector.
- Masimo Patient SafetyNet connector.
- Drager Infinity MEDIBUS connector.
- Edwards HemoSphere VitalView connector.
- BioTelemetry MCOT connector.

## Slice 7: Device registry

- Postgres-16 schema for `device`, `device_metric`, `calibration`, `firmware_revision`.
- FHIR Device + DeviceMetric REST surface.
- Cross-vendor device merge (clinical-engineer view).

## Acceptance criteria

- 8 vendor connectors landed and tested against vendor reference simulators.
- HL7v2 ingest ≥ 10K msgs/sec/cell on Tier-1 OS.
- IEEE 11073 frame ingest ≥ 100K frames/sec/cell.
- Conformance test against IHE Connectathon scenarios passes.

## Dependencies

- IP-001 streaming substrate ready
- healthcare-integration µservice cross-link
