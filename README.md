# Oyatie

Oyatie is an owned cloud—compute, storage, identity, network, data, and
delivery—and the first-party applications that consume it like any other
tenant. The implementation is a Rust Cargo workspace; the toolchain is pinned
in [`rust-toolchain.toml`](rust-toolchain.toml), and the default branch is
`dev`.

This README is the repository's human overview and data surface, not agent
instruction authority. Agents start with root [`AGENTS.md`](AGENTS.md); root
[`CLAUDE.md`](CLAUDE.md) is a compatibility delta for that harness.

## Build and verify

```sh
git clone git@github.com:oyatie/oyatie.git
cd oyatie
cargo fmt --all --check
cargo nextest run --locked --workspace --profile ci
```

`cargo clippy --workspace --all-targets -- -D warnings` provides local lint
feedback. `buck2 build //...` checks the local hermetic graph but is not merge
evidence. Protected delivery uses a PR against `dev` and the required
`presubmit` context.

## Repository and current facts

Each top-level capability owns one cloud engine; `app/<product>/` owns a
first-party product. Their current facts are the native Rust and tests, port and
protobuf contracts, Cedar policy, reconciliation state, SLO-controller inputs,
build declarations, and `OWNERS` consumed at an immutable revision. Any human
view is an untracked projection of those native facts. Historical lookup is a
separate explicit opt-in and does not become current truth.

## Participation and security

Authorized contributors follow [contribution expectations](.github/CONTRIBUTING.md)
and the [Code of Conduct](.github/CODE_OF_CONDUCT.md). Report vulnerabilities
privately through the route in the [security policy](.github/SECURITY.md); do
not open a public issue for a vulnerability.

This repository is proprietary. Use and distribution are governed by the
[`LICENSE`](LICENSE).
