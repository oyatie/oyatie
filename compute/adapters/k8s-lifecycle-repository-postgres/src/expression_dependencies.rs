use sqlx::PgConnection;

use crate::{SCHEMA_NAME, error::PgK8sLifecycleSchemaError};

const NONCATALOG_DEPENDENCIES: &str = r#"
WITH governed_relations AS (
    SELECT c.oid FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = $1
), expressions AS (
    SELECT 'pg_catalog.pg_attrdef'::regclass AS catalog, d.oid
    FROM pg_attrdef d JOIN governed_relations r ON r.oid = d.adrelid
    UNION ALL
    SELECT 'pg_catalog.pg_constraint'::regclass, c.oid
    FROM pg_constraint c JOIN governed_relations r ON r.oid = c.conrelid
    UNION ALL
    SELECT 'pg_catalog.pg_policy'::regclass, p.oid
    FROM pg_policy p JOIN governed_relations r ON r.oid = p.polrelid
)
SELECT EXISTS (
    SELECT 1 FROM expressions expression
    JOIN pg_depend dependency
      ON dependency.classid = expression.catalog AND dependency.objid = expression.oid
    WHERE dependency.refclassid IN (
        'pg_catalog.pg_proc'::regclass, 'pg_catalog.pg_operator'::regclass,
        'pg_catalog.pg_type'::regclass, 'pg_catalog.pg_collation'::regclass
    ) AND (CASE dependency.refclassid
        WHEN 'pg_catalog.pg_proc'::regclass THEN
            (SELECT pronamespace FROM pg_proc WHERE oid = dependency.refobjid)
        WHEN 'pg_catalog.pg_operator'::regclass THEN
            (SELECT oprnamespace FROM pg_operator WHERE oid = dependency.refobjid)
        WHEN 'pg_catalog.pg_type'::regclass THEN
            (SELECT typnamespace FROM pg_type WHERE oid = dependency.refobjid)
        WHEN 'pg_catalog.pg_collation'::regclass THEN
            (SELECT collnamespace FROM pg_collation WHERE oid = dependency.refobjid)
    END) IS DISTINCT FROM 'pg_catalog'::regnamespace
)
"#;

pub(crate) async fn attest_expression_dependencies(
    connection: &mut PgConnection,
) -> Result<(), PgK8sLifecycleSchemaError> {
    let unexpected: bool = sqlx::query_scalar(NONCATALOG_DEPENDENCIES)
        .bind(SCHEMA_NAME)
        .fetch_one(connection)
        .await?;
    (!unexpected)
        .then_some(())
        .ok_or(PgK8sLifecycleSchemaError::ExpressionDependencyContract)
}
