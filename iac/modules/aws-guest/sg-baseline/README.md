# aws-guest/sg-baseline

Canonical ADR-0339 OpenTofu fixture module for the AWS-guest security-group baseline.

This first IAC-001 slice is deliberately metadata-only: it emits a normalized deny-by-default ingress and explicit egress allow-list policy shape for consuming wrappers, plus tenant/cell/compliance labels. It does not create AWS security groups, call provider APIs, configure credentials, run OpenTofu plan/apply, or claim production cloud provisioning.

## Version

`v0.1.0`, release identifier `iac/modules/aws-guest/sg-baseline/v0.1.0`.

## Required inputs

- `tenant_id`
- `tenant_class` (`demo_trial` or `paid`)
- `cell_id`
- `service_name`
- `allowed_egress_cidrs`
- `allowed_egress_tcp_ports`

## Outputs

- `module_identity`
- `security_group_baseline`
- `required_labels`
