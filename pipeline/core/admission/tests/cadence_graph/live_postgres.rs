use pipeline_admission::{LIVE_POSTGRES_CRATES, LIVE_POSTGRES_JOBS, live_postgres_cells_ok};

const WORKFLOW_CALL_BODY: &str = r#"    inputs:
      run_backbone:
        description: Qualify the backbone database cell
        required: false
        default: true
        type: boolean
      run_compute_lifecycle:
        description: Qualify the Compute lifecycle database cell
        required: false
        default: true
        type: boolean"#;

const BACKBONE_LIVE_STEP: &str = r#"      - name: Live Postgres tests (live_* only)
        env:
          OYATIE_BACKBONE_LIVE_POSTGRES: "1"
          OYATIE_BACKBONE_POSTGRES_URL: ${{ env.PG_ADMIN_URL }}
          OYATIE_BACKBONE_POSTGRES_APP_URL: ${{ env.PG_APP_URL }}
        run: |
          set -euo pipefail
          test "${OYATIE_BACKBONE_LIVE_POSTGRES}" = "1"
          test -n "${OYATIE_BACKBONE_POSTGRES_URL}"
          test -n "${OYATIE_BACKBONE_POSTGRES_APP_URL}"
          cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p tenancy-tenant-lifecycle-store-postgres
          cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p identity-scim-store-postgres
          cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p iam-identity-service
          cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p tenancy-tenant-lifecycle-app"#;

const COMPUTE_LIVE_STEP: &str = r#"      - name: Compute lifecycle live Postgres tests
        env:
          OYATIE_BACKBONE_LIVE_POSTGRES: "1"
          OYATIE_BACKBONE_POSTGRES_URL: ${{ env.PG_ADMIN_URL }}
          OYATIE_BACKBONE_POSTGRES_APP_URL: ${{ env.PG_COMPUTE_APP_URL }}
        run: |
          set -euo pipefail
          test "${OYATIE_BACKBONE_LIVE_POSTGRES}" = "1"
          test -n "${OYATIE_BACKBONE_POSTGRES_URL}"
          test -n "${OYATIE_BACKBONE_POSTGRES_APP_URL}"
          cargo nextest run --locked --profile live --run-ignored only --no-tests=error \
            -p compute-k8s-lifecycle-repository-postgres"#;

const VERDICT_BODY: &str = r##"    name: live Postgres cell verdict
    if: always()
    needs: [live-postgres, compute-lifecycle-postgres]
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - name: Require every selected cell
        env:
          RUN_BACKBONE: ${{ inputs.run_backbone }}
          RUN_COMPUTE_LIFECYCLE: ${{ inputs.run_compute_lifecycle }}
        run: |
          cell() {
            if [ "$1" = "true" ]; then [ "$2" = "success" ]; else [ "$2" = "skipped" ] || [ "$2" = "success" ]; fi
          }
          requested() { [ "${RUN_BACKBONE}" = "true" ] || [ "${RUN_COMPUTE_LIFECYCLE}" = "true" ]; }
          requested && cell "${RUN_BACKBONE}" "${{ needs.live-postgres.result }}" && cell "${RUN_COMPUTE_LIFECYCLE}" "${{ needs.compute-lifecycle-postgres.result }}""##;

const PRESUBMIT_CALLER: &str = r#"    needs: [layout, change-gates]
    if: needs.layout.result == 'success' && !(needs.change-gates.outputs.backbone_postgres == 'false' && needs.change-gates.outputs.compute_lifecycle_postgres == 'false')
    uses: oyatie/oyatie/.github/workflows/live-postgres.yml@dev
    with:
      run_backbone: ${{ needs.change-gates.outputs.backbone_postgres == 'true' }}
      run_compute_lifecycle: ${{ needs.change-gates.outputs.compute_lifecycle_postgres == 'true' }}"#;

const PRESUBMIT_FANIN_STEP: &str = r#"      - name: Fan-in verdict
        env:
          BACKBONE_POSTGRES: ${{ needs.change-gates.outputs.backbone_postgres }}
          COMPUTE_LIFECYCLE_POSTGRES: ${{ needs.change-gates.outputs.compute_lifecycle_postgres }}
          REINDEER: ${{ needs.change-gates.outputs.reindeer }}
          EVENT: ${{ github.event_name }}
        run: |
          req() { [ "$1" = "success" ]; }
          gate() { [ "$1" = "true" ] || [ "$1" = "false" ]; }
          pg() {
            if [ "${BACKBONE_POSTGRES}" = "true" ] || [ "${COMPUTE_LIFECYCLE_POSTGRES}" = "true" ]; then [ "$1" = "success" ]; else [ "$1" = "skipped" ] || [ "$1" = "success" ]; fi
          }
          reindeer() {
            if [ "${REINDEER}" = "true" ]; then [ "$1" = "success" ]; else [ "$1" = "skipped" ] || [ "$1" = "success" ]; fi
          }
          occ() {
            if [ "${EVENT}" = "pull_request" ]; then [ "$1" = "success" ]; else [ "$1" = "skipped" ] || [ "$1" = "success" ]; fi
          }
          gate "${BACKBONE_POSTGRES}" && gate "${COMPUTE_LIFECYCLE_POSTGRES}" && gate "${REINDEER}" && req "${{ needs.layout.result }}" && occ "${{ needs.occupancy.result }}" && req "${{ needs.lint.result }}" && req "${{ needs.clippy.result }}" && req "${{ needs.test.result }}" && req "${{ needs.deny.result }}" && req "${{ needs.change-gates.result }}" && occ "${{ needs.commit-signing.result }}" && reindeer "${{ needs.reindeer-source-qualification.result }}" && pg "${{ needs.live-postgres.result }}" && req "${{ needs.build-cache-qualification.result }}" && exit 0 || exit 1"#;

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    text.split_once(start)?
        .1
        .split_once(end)
        .map(|(body, _)| body)
}

fn job_body<'a>(yaml: &'a str, id: &str, next: Option<&str>) -> Option<&'a str> {
    let start = format!("\n  {id}:\n");
    let rest = yaml.split_once(&start)?.1;
    match next {
        Some(next) => rest
            .split_once(&format!("\n  {next}:\n"))
            .map(|(body, _)| body),
        None => Some(rest),
    }
}

fn job_conditions(body: &str) -> Vec<&str> {
    body.lines()
        .filter(|line| line.starts_with("    if:"))
        .collect()
}

fn ensure(ok: bool, message: &'static str) -> Result<(), &'static str> {
    ok.then_some(()).ok_or(message)
}

fn validate_cell_contracts(yaml: &str) -> Result<(), &'static str> {
    let Some(backbone) = job_body(yaml, "live-postgres", Some("compute-lifecycle-postgres")) else {
        return Err("backbone job");
    };
    let Some(compute) = job_body(
        yaml,
        "compute-lifecycle-postgres",
        Some("live-postgres-verdict"),
    ) else {
        return Err("Compute job");
    };
    let Some(verdict) = job_body(yaml, "live-postgres-verdict", None) else {
        return Err("verdict job");
    };
    let call = between(yaml, "\n  workflow_call:\n", "\npermissions:\n");

    ensure(super::job_ids(yaml) == LIVE_POSTGRES_JOBS, "job set")?;
    ensure(
        call.is_some_and(|body| body.trim_end() == WORKFLOW_CALL_BODY),
        "workflow inputs",
    )?;
    ensure(yaml.matches("image: postgres:16").count() == 2, "cells")?;
    ensure(!yaml.contains("continue-on-error:"), "continue on error")?;
    ensure(
        job_conditions(backbone) == ["    if: inputs.run_backbone"],
        "backbone predicate",
    )?;
    ensure(
        job_conditions(compute) == ["    if: inputs.run_compute_lifecycle"],
        "Compute predicate",
    )?;
    ensure(
        backbone.trim_end().ends_with(BACKBONE_LIVE_STEP),
        "backbone live step",
    )?;
    ensure(
        compute.trim_end().ends_with(COMPUTE_LIVE_STEP),
        "Compute live step",
    )?;
    ensure(verdict.trim_end() == VERDICT_BODY, "verdict")
}

fn closed_cell_contracts(yaml: &str) -> bool {
    validate_cell_contracts(yaml).is_ok()
}

fn mutate_step(yaml: &str, step: &str, from: &str, to: &str) -> String {
    assert!(step.contains(from), "mutation fixture");
    assert!(yaml.contains(step), "live step fixture");
    yaml.replacen(step, &step.replacen(from, to, 1), 1)
}

#[test]
fn reusable_workflow_has_two_closed_cells_and_a_verdict() {
    let y = super::read(".github/workflows/live-postgres.yml");
    validate_cell_contracts(&y).unwrap_or_else(|error| panic!("{error}"));
    assert!(!live_postgres_cells_ok("skipped", "skipped", false, false));
}

#[test]
fn cell_contract_rejects_fail_open_routing_and_verdict_mutations() {
    let y = super::read(".github/workflows/live-postgres.yml");
    for (from, to) in [
        (
            "    if: inputs.run_backbone",
            "    if: inputs.run_backbone || true",
        ),
        (
            "    if: inputs.run_compute_lifecycle",
            "    if: inputs.run_compute_lifecycle || true",
        ),
        (
            "    if: inputs.run_backbone",
            "    if: inputs.run_compute_lifecycle",
        ),
        ("    steps:\n", "    continue-on-error: true\n    steps:\n"),
        ("default: true", "default: false"),
        ("requested && cell", "true || cell"),
    ] {
        assert!(!closed_cell_contracts(&y.replacen(from, to, 1)), "{from}");
    }
    let compute_default = y.replacen(
        "      run_compute_lifecycle:\n        description: Qualify the Compute lifecycle database cell\n        required: false\n        default: true",
        "      run_compute_lifecycle:\n        description: Qualify the Compute lifecycle database cell\n        required: false\n        default: false",
        1,
    );
    assert!(!closed_cell_contracts(&compute_default));
}

#[test]
fn each_cell_rejects_zero_test_and_cross_cell_package_mutations() {
    let y = super::read(".github/workflows/live-postgres.yml");
    for step in [BACKBONE_LIVE_STEP, COMPUTE_LIVE_STEP] {
        for token in ["--profile live", "--run-ignored only", "--no-tests=error"] {
            let mutation = mutate_step(&y, step, token, "--quiet");
            assert!(!closed_cell_contracts(&mutation), "{token}");
        }
    }
    let missing_backbone = mutate_step(
        &y,
        BACKBONE_LIVE_STEP,
        "          cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p identity-scim-store-postgres\n",
        "",
    );
    assert!(!closed_cell_contracts(&missing_backbone));
    let swapped_compute = mutate_step(
        &y,
        COMPUTE_LIVE_STEP,
        "compute-k8s-lifecycle-repository-postgres",
        "tenancy-tenant-lifecycle-store-postgres",
    );
    assert!(!closed_cell_contracts(&swapped_compute));
}

#[test]
fn protected_and_postsubmit_callers_select_cells_explicitly() {
    let presubmit = super::read(".github/workflows/presubmit.yml");
    let caller = between(
        &presubmit,
        "\n  live-postgres:\n",
        "\n  build-cache-qualification:\n",
    )
    .expect("presubmit live-Postgres caller");
    assert_eq!(caller.trim_end(), PRESUBMIT_CALLER);
    for output in ["backbone_postgres", "compute_lifecycle_postgres"] {
        assert!(presubmit.contains(&format!(
            "      {output}: ${{{{ steps.g.outputs.{output} }}}}"
        )));
    }
    let fan_in = job_body(&presubmit, "presubmit", None).expect("presubmit fan-in");
    assert!(fan_in.trim_end().ends_with(PRESUBMIT_FANIN_STEP));

    let postsubmit = super::read(".github/workflows/postsubmit.yml");
    let caller = between(&postsubmit, "\n  live-postgres:\n", "\n  postsubmit:\n")
        .expect("postsubmit live-Postgres caller");
    assert_eq!(
        caller.trim_end(),
        "    uses: ./.github/workflows/live-postgres.yml\n    with:\n      run_backbone: true\n      run_compute_lifecycle: true"
    );
}

#[test]
fn live_postgres_crate_inventory_is_exact() {
    assert_eq!(
        LIVE_POSTGRES_CRATES,
        [
            "compute-k8s-lifecycle-repository-postgres",
            "tenancy-tenant-lifecycle-store-postgres",
            "identity-scim-store-postgres",
            "iam-identity-service",
            "tenancy-tenant-lifecycle-app",
        ]
    );
    let t = super::read(".config/nextest.toml");
    assert!(t.contains("[profile.live]"));
    assert!(t.contains("test(/^live_/)"));
}
