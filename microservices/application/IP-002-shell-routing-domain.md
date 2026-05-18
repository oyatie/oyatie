---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-002-shell-routing-domain
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: oya-application-shell-routing-domain

## Intent

Pure route-matching algebra: longest-prefix matching, scope-set
intersection, MFA-requirement combination. Zero I/O. Verified against
property tests.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-shell-routing-domain/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/matcher.rs` | create — longest-prefix matcher with path-parameter capture |
| `.../src/scope.rs` | create — RouteScope set algebra |
| `.../src/mfa.rs` | create — MFA requirement combine |
| `microservices/application/catalog/oya-application-shell-routing-domain.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-application-shell-routing-domain
JUSTIFICATION: microservice=application; bc=shell-routing; layer=domain (ADR-0105 pure-logic)
```

## Code Shape

```rust
pub struct RouteMatcher { trie: PrefixTrie<Route> }

impl RouteMatcher {
    pub fn build(routes: Vec<Route>) -> Self { /* longest-prefix trie */ }
    pub fn match_path(&self, path: &str) -> Option<&Route> { /* O(path_len) */ }
}

pub fn intersect_scopes(required: &[String], principal: &[String]) -> bool {
    required.iter().any(|r| principal.contains(r))
}

pub fn require_mfa(route: MfaFactor, principal: MfaFactor) -> bool {
    use MfaFactor::*;
    match route {
        None => true,
        Totp => matches!(principal, Totp | Webauthn),
        Webauthn => matches!(principal, Webauthn),
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-application-shell-routing-domain --all-features
cargo nextest run -p oya-application-shell-routing-domain --all-features
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-application-shell-routing-domain
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-application-shell-routing-domain
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_longest_prefix_match_property` | proptest over arbitrary route sets |
| `test_scope_intersection_empty_denies` | empty role-set denies |
| `test_mfa_require_lattice` | Webauthn > Totp > None |
| `test_path_parameter_capture` | `/hr/users/:id` captures `id` |

Coverage: 95 % line / 90 % branch.

## Halt Conditions

- Any I/O introduced
- Performance regression (matcher > 100 µs / call)

## Next IP

[`IP-003-shell-routing-usecase.md`](IP-003-shell-routing-usecase.md)
