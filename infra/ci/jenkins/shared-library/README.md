# oya-jenkins-shared — Buck2 bridge lane (ADR-0361/0408/0513)

The shared library is a Jenkins bridge for the target `oya-ci-required` context.
Buck2 is the only build/test execution authority in this lane. Cargo contexts and
`oya-verify` are forbidden as branch-protection or Phase-0 exit authority.

## Entry point: `oyaCiLane(Map cfg)`

Mandatory stages:

| Stage | Tool | Authority note |
|---|---|---|
| Buck2 authority policy | `buck2 build //:buck2-authority-policy-check` | Static no-Cargo regression gate |
| Buck2 affected build/test | `infra/ci/buck2-affected-gate.sh` | Builds/tests affected Buck2 target closure |
| Buck2 governance bridge smoke | `buck2 uquery //oya/developer-sdk/crates/oya-dev-cli:oya` | Proves oya binary is Buck2-addressable; not merge authority by itself |
| Supply-chain scans | Syft, Trivy, osv-scanner | Advisory/hardening as configured by lane |
| Sign/provenance | cosign + in-toto/SLSA | Trunk-only artifact evidence |

Jenkins can post `oya-ci-required` while the cloud-ci/oya-ci producer is being cut
over, but the Phase-0 exit target is a trusted cloud-ci/oya-ci producer posting the
same context from controller/trunk-sourced gate definitions.

## Layout

- `vars/oyaCiLane.groovy` — Buck2 bridge lane definition.
- `examples/microservice-lane.Jenkinsfile` — thin per-service form.
- Root `Jenkinsfile` — repo-wide governance gate + lane fan-out.

## Forbidden in active CI lanes

- Cargo build/test/check/fmt/clippy/nextest/bench/deny/cyclonedx/install commands.
- Legacy required contexts named after Cargo lanes.
- Treating `oya verify` or `oya gate` output as protected-branch authority.

The narrow Cargo exception is documented in `specs/buck2-authority-policy.json`:
production release image/binary optimization may use Cargo release profiles for
binary size/codegen/allocator evidence, but that evidence cannot satisfy CI merge
authority.
