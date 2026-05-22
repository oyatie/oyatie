# cloud-network tier scrub remediation notes

Wave: 15J-batch-4 BUCKET-01.

Files modified, with current line counts:

- README.md: 12
- benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md: 101
- coherence-audit-2026-05-20.md: 668
- faqs/network-engineer-faq.md: 175
- feature-parity-matrix-2026-05-20.md: 411
- migration-playbooks/from-aws-vpc-and-istio.md: 166
- onboarding/network-engineer-first-week.md: 181
- performance-benchmark-numbers-2026-05-20.md: 422
- reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md: 199
- runbooks/cross-cell-routing-stall.md: 270
- runbooks/ddos-mitigation-engagement.md: 268
- tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md: 355
- tutorials/provision-vpc-mtls-and-cedar-policy.md: 216

capability-tiers/ dir deleted: Y.

Vocabulary replacement count: roughly 270 source lines matched before scrub.

Design decisions:

- Replaced Bronze/Silver/Gold/Platinum customer ladder language with `demo_trial` and `paid tenant_class`.
- Preserved network capability distinctions as `cell_topology`, deployment context, capacity envelope, or compliance-pack language instead of pricing ladder language.
- Renamed the previous capability-tier delta artifact to a tenant-class adoption delta artifact.

Outstanding follow-ups: none for the vocabulary scrub. Separate implementation work remains for actual OCI Always Free modules and measured network capacity evidence.
