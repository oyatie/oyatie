---
ip_id: IP-022
microservice: comms-email
bounded_context: template-rendering
layer: usecase
related_adrs: [ADR-0201, ADR-0064]
---

# IP-022 — MJML and Liquid template rendering use case

## Goal

Unify the older IP-006 MJML compiler and IP-007 Liquid substitution plans into the
service-local rendering use case that prepares `OutboundMessageRequest.template_id`,
`locale`, and `vars` for the send pipeline.

## Service anchors

- REST contract: `microservices/comms-email/contracts/openapi.yaml` requires
  `template_id`, `locale`, and `vars` on `OutboundMessageRequest`.
- gRPC contract: `microservices/comms-email/contracts/comms_email.proto` mirrors those
  fields on `SendMessageRequest`.
- Decision: `microservices/comms-email/decisions/SVC-ADR-005-mjml-liquid-canonical.md`
  chooses MJML via `mrml`, Liquid via `liquid`, and MJML-before-Liquid ordering.
- Counterpart send policy: `microservices/comms-email/policy/comms-email-send.cedar`.

## Use-case behavior

1. Resolve template id and locale through tenant/pack/canonical fallback.
2. Compile MJML to HTML using the stack chosen in SVC-ADR-005.
3. Validate the variable contract before substitution.
4. Apply Liquid substitution without filesystem or shell access.
5. Produce HTML and text alternatives for IP-001/IP-002/IP-003/IP-004 provider adapters.

## Counterpart refs

- IP-006 remains the detailed MJML compiler predecessor.
- IP-007 remains the detailed Liquid substitution predecessor.
- IP-001 through IP-004 consume the rendered message body in provider-specific adapters.
- IP-020 may flag template/content changes during reputation incidents.

## Acceptance

- `template_id`, `locale`, and `vars` remain contract-compatible with both OpenAPI and proto.
- Rendering is deterministic for identical template, locale, pack, and variable input.
- Liquid cannot perform filesystem access or shell-out.
- Template failures reject before provider send and before audit claims provider acceptance.
