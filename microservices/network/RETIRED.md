---
doc_class: RetiredMicroserviceMarker
microservice: network
status: Retired
retired_on: 2026-05-21
retirement_wave: Wave 15K
successor: microservices/community/
retirement_protocol: ADR-0138
---

# RETIRED: network µservice

`network` is retired as a standalone µservice as of 2026-05-21.

Reason: the path name implied networking infrastructure, but the source corpus
defined a LinkedIn-class professional product. Per the 2026-05-21 directive and
ADR-0132 single-concern discipline, that professional content now belongs to
`microservices/community/`.

Successor:

- `microservices/community/PRD.md`
- `microservices/community/ARCHITECTURE.md`
- `microservices/community/manifest.json`
- `microservices/community/competitor-parity-matrix.md`
- `microservices/community/REMEDIATION-NOTES-2026-05-21.md`

Migrated responsibilities:

- Resume / profile aggregates, profile verification, profile export
- Connections graph and connection request lifecycle
- InMail-equivalent professional outreach
- Endorsements and recommendations
- Jobs, applications, recruiter-stub, and ATS handoff
- Skill assessments
- Pages and events

Not migrated:

- LinkedIn-style engagement-optimized text feed
- For-You-style algorithmic attention feed
- Sponsored post promotion
- Influencer monetization via followers
- VPC Lattice / Cross-Cloud Network / Azure Virtual WAN counterpart claims

Infrastructure networking belongs to `microservices/cloud-network/`, not this
retired product path.
