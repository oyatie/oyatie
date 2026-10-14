use sqlx::Connection;

use super::{create_database, drop_database};
use crate::support::{SETUP_URL_ENV, pool, require_enabled};

#[tokio::test]
#[ignore = "live postgres"]
async fn live_cleanup_waits_for_session_exit_without_forcing_deletion() {
    require_enabled();
    let setup = pool(&std::env::var(SETUP_URL_ENV).expect("setup URL is required")).await;
    let name = "compute_claim_cleanup";
    let database = create_database(&setup, name).await;
    let mut retained = database
        .acquire()
        .await
        .expect("retain an owned disposable session")
        .detach();
    let backend: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut retained)
        .await
        .expect("identify retained disposable session");
    database.close().await;
    let cleanup_setup = setup.clone();
    let failure = tokio::spawn(async move { drop_database(&cleanup_setup, name).await })
        .await
        .expect_err("an outstanding session must refuse deletion");
    assert!(failure.is_panic());
    let panic = failure.into_panic();
    let message = panic
        .downcast_ref::<String>()
        .expect("cleanup deadline reports its failure");
    let unchanged: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = $1) AND EXISTS (SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid = $2 AND datname = $1)",
    )
    .bind(name)
    .bind(backend)
    .fetch_one(&setup)
    .await
    .expect("database and retained session survive refused cleanup");
    retained
        .close()
        .await
        .expect("close the owned retained session");
    drop_database(&setup, name).await;
    assert!(
        unchanged,
        "cleanup must not delete the database or terminate its session"
    );
    assert!(
        message.contains("disposable claim-test sessions did not exit before the cleanup deadline")
    );
    let removed: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = $1)",
    )
    .bind(name)
    .fetch_one(&setup)
    .await
    .expect("completed cleanup removes the disposable database");
    assert!(removed);
    setup.close().await;
}
