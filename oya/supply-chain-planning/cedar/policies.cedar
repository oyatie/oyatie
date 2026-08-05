// Cedar policy fragments for supply-chain-planning.
//
// Preview/provenance-only aggregate note: the canonical PRD at
// `specs/microservices/supply-chain-planning.json` is a metadata-only preview.
// This inventory may record authorization intent and historical doctrine, but it
// must not activate runtime enforcement, CI/CD merge authority, GitOps
// deployment, cross-tenant migration, cloud readiness, runtime audit-chain
// emission, or autonomous sharding execution.
//
// Current source-of-truth amendments:
// - ADR-0363 retires `oya vcs` / `oya git`; coordination uses plain git plus a
//   protected PR, and legacy `oya verify` / `oya gate` output is local-feedback
//   evidence only, never protected-branch merge authority.
// - ADR-0515 makes GitHub Actions plus the single protected `oya-ci-required`
//   context the live CI authority. Jenkins, Prow, and legacy `oya` CLI CI
//   wording are superseded provenance. ArgoCD is a CD bridge/reference surface
//   only where separately authorized, not activation by this preview inventory.
//
// Historical provenance retained by reference only:
// - ADR-0346 local-verifier mirror doctrine.
// - ADR-0348 sharding-automation doctrine.
// - ADR-0349 Jenkins/Argo self-hosted CI/CD bridge doctrine.
//
// This aggregate file intentionally authors no permit/forbid statements. Target
// authorization intent for bounded supply-chain-planning surfaces lives in
// `policy/*.cedar`; those files are preview policy metadata and are not wired to
// runtime enforcement by this aggregate.
