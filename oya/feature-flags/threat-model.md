# Feature Flags Threat Model

## Assets

- Flag definitions, tenant targeting predicates, audit-required evaluation events, and kill-switch controls.

## Threats

- Cross-tenant flag disclosure through read or evaluation APIs.
- Unauthorized flag mutation that enables hidden functionality.
- Cohort inference through evaluation context.
- Audit-required flags evaluated without evidence emission.

## Mitigations

- Cedar tenant policy gates read, write, and evaluation.
- Flag definition changes emit audit-chain events.
- Evaluation context is allowlisted and excludes raw user payloads.
- Kill-switch changes require explicit lifecycle metadata.
