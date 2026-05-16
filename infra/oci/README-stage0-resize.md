# Stage-0 instance A2 → A1 resize

Background: `VM.Standard.A1.Flex` (Ampere Altra, Always Free) capacity is
exhausted at launch in `ap-chuncheon-1`. Workaround per user direction:

1. **Bootstrap** the instance on `VM.Standard.A2.Flex` (AmpereOne, paid,
   has capacity).
2. **Resize** to `VM.Standard.A1.Flex` Always Free once the instance is
   `RUNNING`.

OpenTofu does the bootstrap. The resize is done out-of-band with `oci`
CLI; `compute.tf` declares `lifecycle.ignore_changes = [shape, shape_config]`
so future plans don't try to revert.

## Resize procedure

```bash
INSTANCE_ID=$(tofu output -raw stage0_instance_id)

# 1. Stop the instance (soft stop preferred).
oci compute instance action --instance-id "$INSTANCE_ID" --action SOFTSTOP --wait-for-state STOPPED

# 2. Change shape to A1.Flex with Always Free profile (1 OCPU / 6 GB).
oci compute instance update --instance-id "$INSTANCE_ID" \
  --shape VM.Standard.A1.Flex \
  --shape-config '{"ocpus":1,"memoryInGBs":6}'

# 3. Start the instance.
oci compute instance action --instance-id "$INSTANCE_ID" --action START --wait-for-state RUNNING
```

If the start fails with "Out of host capacity" again, leave the instance on
A2 (it will continue accruing charges; budget-wise undesirable) and retry
the resize later. Always Free A1 capacity in `ap-chuncheon-1` opens
intermittently.

## Cost note

While the instance runs on A2.Flex (between bootstrap and successful resize),
it bills at the AmpereOne flex rate (roughly USD$0.01–0.02 per OCPU-hour
plus memory and storage). The bootstrap window should be measured in
minutes if Always Free capacity is currently open. Document any extended
A2 dwell in `evidence/oci-readiness/stage0-resize-log.md`.
