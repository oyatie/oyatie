# Oyatie IaC module catalog

Authority: ADR-0339 B2.014/B2.029. This first catalog slice records only the IAC-001 fixture module and does not claim the full Wave 15Q module library has landed.

## aws-guest/sg-baseline

- Context: `aws-guest`
- Primitive: `sg-baseline`
- Version: `v0.1.0`
- Release identifier: `iac/modules/aws-guest/sg-baseline/v0.1.0`
- Source path: `iac/modules/aws-guest/sg-baseline`
- Wrapper fixture: `cloud/cloud-billing/iac/aws-guest/main.tofu`
- Required providers: none (metadata-only module; pure locals, no `aws_*` resources)
- Tenant class validation: `demo_trial` or `paid`
- Egress guard: `allowed_egress_cidrs` rejects the open internet (`0.0.0.0/0` and `::/0`)

Inputs: `tenant_id`, `tenant_class`, `cell_id`, `service_name`, `compliance_pack`, `allowed_egress_cidrs`, `allowed_egress_tcp_ports`.

Outputs: `module_identity`, `security_group_baseline`, `required_labels`.

Non-claims: this fixture emits a normalized deny-by-default ingress and explicit egress allow-list policy shape only. It does not create AWS security groups, run OpenTofu plan/apply, call AWS provider APIs, or provision cloud resources.
