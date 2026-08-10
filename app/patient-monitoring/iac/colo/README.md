# iac/colo — patient-monitoring

Deployment context: **colo**. The customer (typically an EU hospital network or KR
sovereign tenant) hosts the µservice in a colocation facility they own or lease.

Tier mapping: Tier-1 or Tier-2 depending on tenant size.

## Modules

Same module set as `on-prem/` but with `provider = "colo"` and:

- `module.patient_monitoring_colo_bgp_anycast` — BGP anycast for cell front-doors
- `module.patient_monitoring_colo_kms_hsm` — dedicated HSM for tenant CMK
- `module.patient_monitoring_colo_cross_connect` — cross-connect to oyatie POP
- `module.patient_monitoring_colo_lights_out_remote_admin` — JIT remote ops

## Variables

- `tenant_id`
- `colo_facility_id` (e.g., equinix-fr5, sftt-yokohama-1)
- `cell_count`
- `compliance_packs` (typically HIPAA + EU MDR + GDPR or HIPAA + KR PIPA + KR Medical Law + KR MFDS)
- `sovereign_overlay` (true for EU sovereign / KR sovereign / DoD)

## Apply

```bash
tofu init -backend-config=colo-backend.hcl
tofu plan -var-file=tenant-colo.tfvars
tofu apply -var-file=tenant-colo.tfvars
```

## Notes

- Per global memory: EU + KR hospital tenants often prefer colo over public-cloud
  due to PHI residency posture.
- Cross-connect to oyatie POP enables stream-platform fan-out + ML model registry
  + audit-chain mirror.
