use sqlx::PgConnection;

pub(crate) async fn use_catalog_path(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL search_path = pg_catalog, pg_temp")
        .execute(connection)
        .await?;
    Ok(())
}
