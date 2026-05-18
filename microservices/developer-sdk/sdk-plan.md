---
doc_class: SdkPlan
title: "SDK family roadmap"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# SDK family roadmap


## GA-targeted SDK families

| Family | Stack | Phase | Distribution |
|---|---|---|---|
| TypeScript / JavaScript | Node 22 LTS + browser ESM | Phase 3 GA | In-house npm registry |
| Rust | stable + WASM | Phase 3 GA | In-house cargo registry |
| Swift | iOS 17+ / macOS 14+ (SPM) | Phase 3 GA | In-house SPM index |
| Kotlin | JVM + Android (Gradle) | Phase 3 GA | In-house Maven |
| C# | .NET 8 LTS (NuGet) | Phase 3 GA | In-house BaGet |
| Python | 3.12+ (PyPI) | Phase 3 GA | In-house pypiserver |

## Codegen invariants

- Deterministic: 2 runs on identical input → byte-identical output.
- Versioned: each spec version maps to a unique SDK package version.
- Tested: every generated package has a smoke-test in its target stack CI.
- Documented: TechDocs generated alongside SDK.

## Post-GA roadmap

- Go SDK (Phase 4).
- Ruby SDK (Phase 4).
- Elixir SDK (Phase 5; lower priority).
- gRPC Web SDK (Phase 5).

