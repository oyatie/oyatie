# Oyatie IaC module catalog

Authority: ADR-0339 B2.014/B2.029. This first catalog slice records only the IAC-001 fixture module and does not claim the full Wave 15Q module library has landed.

## aws-guest/sg-baseline

- Context: `aws-guest`
- Primitive: `sg-baseline`
- Version: `v0.1.0`
- Release identifier: `cloud-iac/modules/aws-guest/sg-baseline/v0.1.0`
- Source path: `cloud/cloud-iac/modules/aws-guest/sg-baseline`
- Wrapper fixture: `cloud/cloud-billing/iac/aws-guest/main.tf`
- Cosign attestation metadata: `cloud/cloud-iac/modules/aws-guest/sg-baseline/cosign-attestation.json`
- Cosign attestation digest: `sha256:7c24d764bd9f70fd24769e87f10a4386df023a76e09ef828a19d5fd717c73762`
- Required provider: `registry.opentofu.org/hashicorp/aws >= 5.0.0`
- Tenant class validation: `demo_trial` or `paid`

Inputs: `tenant_id`, `tenant_class`, `cell_id`, `service_name`, `compliance_pack`, `allowed_egress_cidrs`, `allowed_egress_tcp_ports`, `cosign_attestation_digest`.

Outputs: `module_identity`, `security_group_baseline`, `required_labels`, `cosign_attestation_digest`.

Non-claims: this fixture emits a normalized deny-by-default ingress and explicit egress allow-list policy shape only. It does not create AWS security groups, execute cosign signing, generate SLSA/VSA evidence, run OpenTofu plan/apply, call AWS provider APIs, or provision cloud resources.
