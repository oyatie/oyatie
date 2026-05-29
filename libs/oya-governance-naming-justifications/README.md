# oya-governance-naming-justifications

Enforces `feedback_naming_justification` for microservice manifests.

## Rule

Every `microservices/*/manifest.{json,yaml,yml,toml}` file must declare top-level `naming_justifications` as one structured, single-line proof. The proof must cite BNF v4 or BNF v4.1 and the `12-layer-enum`.

## Trigger

Run the crate when a microservice manifest is added or changed.

```bash
cargo run --manifest-path crates/oya-governance-naming-justifications/Cargo.toml -- --root . --strict
```

## Compliant Output

```text
feedback_naming_justification: Passed (1 manifests, 0 violations)
OK: every discovered microservice manifest has a valid naming proof.
```

## Violation Output

```text
microservices/mail/manifest.yaml:1: MissingField: missing top-level naming_justifications field
  fix: naming_justifications: "BNF v4.1 service_action_resource=<service>.<bounded_context>.<action>.<resource>; 12-layer-enum=<api|rest|application|usecase|domain|kernel|adapter|worker|sdk|iac|policy|observability>"
```

## How To Fix

Add a single-line field such as:

```yaml
naming_justifications: "BNF v4.1 service_action_resource=mail.notice.deliver.message; 12-layer-enum=api"
```
