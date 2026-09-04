//! Lag must be measured against the log, not against a snapshot of it.
//!
//! `sync_status` read its head from `TenantState.entries`, the boot mirror.
//! That vector is assigned once in `compose` and never appended to, so the
//! head was frozen while `applied_ordinal` only grew: the saturating
//! subtraction was identically zero for the life of the process. Every
//! consumer inherited it — `/readyz` could not detect a projection behind its
//! log, and `foundry_projection_lag` was a gauge that could not move, which
//! is why the freshness objective was deleted rather than shipped over it.
//!
//! A second writer appending to the same durable log is the cheapest way to
//! DRIVE the signal here, but it is not the argument that the signal matters:
//! `AppState` declares SQLite single-writer. The in-contract breach is that
//! `append_with_receipt` commits before `apply_sealed` runs, so a panic
//! between them leaves this process permanently one behind for its lifetime,
//! with no second writer anywhere.

mod facade_support;
mod failing_log;
mod out_of_band;
use facade_support as support;

use axum::http::StatusCode;
use support::{Fixture, Session, scrape, value_of};

#[tokio::test]
async fn a_projection_behind_its_log_reports_the_lag() {
    let fixture = Fixture::new("lag-behind");
    let state = fixture.state();

    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_out_of_band");

    let tenant = state.tenants.get("ten_acme").expect("the served tenant");
    let status = tenant
        .lock()
        .await
        .sync_status("ten_acme")
        .expect("the head is readable");

    assert_eq!(
        status.lag, 1,
        "one entry is durable and unfolded, so the lag is one: {status:?}"
    );
}

#[tokio::test]
async fn a_process_behind_its_log_is_not_ready() {
    let fixture = Fixture::new("lag-readiness");
    let state = fixture.state();
    assert!(
        foundry_ontology_app::observation::observe(&state).is_caught_up(),
        "a fresh process is caught up"
    );

    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_unready");

    assert!(
        !foundry_ontology_app::observation::observe(&state).is_caught_up(),
        "a projection that has not folded a durable entry is not ready"
    );
}

/// The EXPOSITION must move, not just the accessor. A gauge whose value a
/// constant would satisfy is the state this signal was in before it had a
/// durable head, and the reason its objective was deleted.
#[tokio::test]
async fn the_exported_gauge_reports_the_real_lag() {
    let fixture = Fixture::new("lag-exported");
    let session = Session::from_state(fixture.state());
    assert_eq!(
        value_of(&scrape(&session).await, "foundry_projection_lag"),
        0,
        "a fresh process is caught up"
    );

    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_exported");

    assert_eq!(
        value_of(&scrape(&session).await, "foundry_projection_lag"),
        1,
        "the durable entry this process has not folded must show in the exposition"
    );
}

/// A tenant that could not be sampled must be COUNTED, not scored as zero.
///
/// Without this the two totals are indistinguishable from healthy: a lag of
/// zero because every tenant is caught up, and a lag of zero because no
/// tenant could be read, render identically.
#[tokio::test]
async fn a_tenant_that_cannot_be_sampled_is_counted_as_unknown() {
    let fixture = Fixture::new("lag-unknown");
    let session = Session::from_state(failing_log::state_with_a_failing_log(
        &fixture.config(),
        "the head is unreadable",
    ));

    let body = scrape(&session).await;
    assert_eq!(
        value_of(&body, "foundry_projection_lag_unknown"),
        1,
        "the one served tenant could not be sampled: {body}"
    );
    assert_eq!(
        value_of(&body, "foundry_projection_lag"),
        0,
        "an unsampled tenant contributes nothing to the total, which is why the \
         unknown gauge exists to qualify it: {body}"
    );
}

/// The totals are SUMS across tenants, and `unknown` counts tenants rather
/// than being a flag.
///
/// Two tenants is the smallest fixture that can tell `unknown = 1` from
/// `unknown += 1`, which is this test's subject: `unknown` counts tenants, so
/// equal load separates assignment from accumulation. The TOTALS need more
/// than that — equal lag cannot tell a sum of entries from a count of
/// tenants — which is why the two tests below load their tenants unequally.
#[tokio::test]
async fn the_totals_are_sums_over_every_served_tenant() {
    let fixture = Fixture::new("lag-two-tenants");
    let mut config = fixture.config();
    config.tenants = vec!["ten_acme".into(), "ten_second".into()];

    let state = failing_log::state_with_a_failing_log_from(config, "both heads are gone");
    let seen = foundry_ontology_app::observation::observe(&state);

    assert_eq!(
        seen.unreadable, 2,
        "each unreadable tenant counts, so two tenants are two unknowns: {seen:?}"
    );
    assert!(
        !foundry_ontology_app::observation::observe(&state).is_caught_up(),
        "no tenant could be read, so nothing supports a claim of readiness"
    );
}

/// The lag total ACCUMULATES across tenants.
///
/// One tenant cannot tell `total += x` from `total = x`. Two tenants with
/// EQUAL lag cannot tell a sum of entries from a count of tenants, nor from
/// an accumulation clamped to one per tenant. Unequal lag tells all three
/// apart, which is what the help text's "entries … summed over served
/// tenants" actually claims.
#[tokio::test]
async fn the_lag_total_accumulates_across_tenants() {
    let fixture = Fixture::new("lag-two-lagging");
    let mut config = fixture.config();
    config.tenants = vec!["ten_acme".into(), "ten_second".into()];
    let state = foundry_ontology_app::compose(&config).expect("boots");

    // ASYMMETRIC on purpose. Two tenants one behind each cannot tell a sum of
    // ENTRIES from a count of affected TENANTS — both give two — and the help
    // text promises entries. Two behind plus one behind gives three, which no
    // per-tenant count and no clamped accumulation can produce.
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_acme_1");
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_acme_2");
    out_of_band::append_for(&fixture.action_log_path(), "ten_second", "idem_second");

    let seen = foundry_ontology_app::observation::observe(&state);
    assert_eq!(
        seen.lag, 3,
        "two entries behind on one tenant and one on the other sums to three: {seen:?}"
    );
    assert_eq!(
        (seen.unreadable, seen.contended),
        (0, 0),
        "both tenants were readable and neither was busy: {seen:?}"
    );
}

/// The poison total accumulates across tenants too.
///
/// Seeded BEFORE boot, so the fold consumes them and refuses them: that is a
/// poison, where the same bytes appended after boot would have been lag.
/// Unequal counts per tenant, so a clamped or per-tenant accumulation cannot
/// produce the total either.
#[tokio::test]
async fn the_poison_total_accumulates_across_tenants() {
    let fixture = Fixture::new("poison-two-tenants");
    let mut config = fixture.config();
    config.tenants = vec!["ten_acme".into(), "ten_second".into()];

    // Asymmetric for the same reason as the lag total above.
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_poison_acme_1");
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_poison_acme_2");
    out_of_band::append_for(
        &fixture.action_log_path(),
        "ten_second",
        "idem_poison_second",
    );

    let state = foundry_ontology_app::compose(&config).expect("boots over poisoned logs");
    let seen = foundry_ontology_app::observation::observe(&state);

    assert_eq!(
        seen.poisoned, 3,
        "two refused entries on one tenant and one on the other sums to three: {seen:?}"
    );
    assert_eq!(
        seen.lag, 0,
        "a poisoned entry advances the fold, so it is consumed, not pending: {seen:?}"
    );
}

/// EVERY gauge comes from one observation, asserted exactly.
///
/// With an always-failing log every pass agrees, so one pass and three are
/// indistinguishable; a head that fails once separates them. The assertion is
/// exact rather than a disjunction over acceptable pairs, for two reasons a
/// disjunction got wrong. An alternative arm admitting `unknown 0` green-lit
/// a revert that merely consumed the first failure elsewhere — a warm-up read
/// before the passes — because that arm is unreachable under one read per
/// tenant per scrape and absorbed the world where it is not. And a pair
/// covering only lag and unknown left `foundry_poisoned_entries` free to walk
/// out of the shared observation, while its own help text on the wire claims
/// the unknown gauge qualifies it.
///
/// The fixture seeds one poisoned entry so the third element is a real zero:
/// a tenant that could not be read contributes nothing to ANY total, and a
/// zero that no tenant could have contributed to proves nothing.
#[tokio::test]
async fn the_gauges_are_one_observation_not_several() {
    let fixture = Fixture::new("lag-torn-snapshot");
    // Seeded before boot, so this tenant genuinely carries a poison. Without
    // it the poisoned total is zero under every implementation and the third
    // element below would be vacuous.
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_torn_poison");
    // The seed must ACTUALLY poison, or the third element below is vacuous:
    // every implementation renders zero when there is no poison to report,
    // and the element stops discriminating without anything going red.
    let baseline = foundry_ontology_app::compose(&fixture.config()).expect("boots");
    assert_eq!(
        foundry_ontology_app::observation::observe(&baseline).poisoned,
        1,
        "the fixture must actually poison, or this test proves less than it says"
    );
    // Released before the double is installed. Harmless to hold while the
    // path is read-only, but stating the intent beats relying on it.
    drop(baseline);
    let session = Session::from_state(failing_log::state_with_a_transiently_failing_head(
        &fixture.config(),
        7,
    ));

    let body = scrape(&session).await;

    assert_eq!(
        (
            value_of(&body, "foundry_projection_lag"),
            value_of(&body, "foundry_projection_lag_unknown"),
            value_of(&body, "foundry_poisoned_entries"),
        ),
        (0, 1, 0),
        "the one tenant could not be read on this scrape, so it contributes to \
         no total and is counted once as unknown; any other triple means the \
         gauges did not come from one observation of one tenant\n{body}"
    );
}
