# Oyatie

Oyatie is a cloud (compute, storage, identity, network, data, pipeline) and the first-party apps that use that cloud the same way any other tenant would.

This tree is a Cargo workspace. Rust is pinned in `rust-toolchain.toml` (1.98.0). The default branch is `dev`. The repository is proprietary; see [`LICENSE`](LICENSE).

## Build

```sh
git clone git@github.com:oyatie/oyatie.git
cd oyatie
cargo nextest run --locked --workspace --profile ci
```

Format: `cargo fmt --all --check`. Local hermetic graph (not merge evidence): `buck2 build //...`.

## Tree

| Path | What |
| --- | --- |
| `<capability>/` | One cloud engine. Faces: `core/`, `ports/`, `adapters/`, `facade/`. |
| `app/<product>/` | A first-party product. |
| `AGENTS.md` | How an agent works in this tree. |
| [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md) | How a human contributes. |
| [`.github/SECURITY.md`](.github/SECURITY.md) | Vulnerability reports. |

Law for a path is `ADR.md`, `PRD.md`, `SPEC.md`, `PLAN.md` in that owner directory.

## License

See [`LICENSE`](LICENSE).
