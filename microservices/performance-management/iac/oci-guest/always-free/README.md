# performance-management — OCI Always Free demo_trial deployment

Demo-tier deployment of `performance-management` exploiting the OCI Always Free
allotment per `feedback_oci_always_free_maximization_2026_05_20`.

## Resources used

- 2x Ampere A1 ARM Flex VMs (2 OCPU + 12 GB each = 4 OCPU + 24 GB total).
- 1x Autonomous DB (Always Free 20 GB).
- Standard subnet + LB allowance.

## Limits

- Tenant class is locked to `demo_trial`.
- Settlement is suppressed (no DealSet billing).
- Synthetic data only; production PII is forbidden.

## Provisioning

```bash
tofu init
tofu apply -var="tenant_id=<uuid>" -var="compartment_ocid=ocid1.compartment..." \
           -var="subnet_ocid=ocid1.subnet..." \
           -var="ubuntu_arm_image_ocid=ocid1.image..." \
           -var="adb_admin_password=<gen-strong>"
```
