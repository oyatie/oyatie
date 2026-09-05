use std::collections::BTreeSet;

use sqlx::{PgConnection, Row};

use crate::error::PgK8sLifecycleSchemaError;
use crate::schema_contract::{
    EXPECTED_COLUMNS, EXPECTED_CONSTRAINTS, EXPECTED_GRANTS, EXPECTED_INDEXES, EXPECTED_POLICIES,
};
use crate::{RUNTIME_ROLE, SCHEMA_NAME};

const EXPECTED_RELATIONS: &[&str] = &[
    "clusters|r|p|true|true|false|heap|true|true",
    "operations|r|p|true|true|false|heap|true|true",
    "schema_migrations|r|p|false|false|false|heap|true|true",
];

pub(crate) async fn attest_schema(
    connection: &mut PgConnection,
) -> Result<(), PgK8sLifecycleSchemaError> {
    crate::role_database_claim::attest_role_database_claim(connection, false).await?;
    attest_runtime_role(connection).await?;
    attest_namespace(connection).await?;
    attest_ownership(connection).await?;
    crate::expression_dependencies::attest_expression_dependencies(connection).await?;
    attest_set(
        connection,
        columns_sql(),
        EXPECTED_COLUMNS,
        PgK8sLifecycleSchemaError::ColumnContract,
    )
    .await?;
    attest_set(
        connection,
        constraints_sql(),
        EXPECTED_CONSTRAINTS,
        PgK8sLifecycleSchemaError::ConstraintContract,
    )
    .await?;
    attest_set(
        connection,
        indexes_sql(),
        EXPECTED_INDEXES,
        PgK8sLifecycleSchemaError::IndexContract,
    )
    .await?;
    attest_set(
        connection,
        policies_sql(),
        EXPECTED_POLICIES,
        PgK8sLifecycleSchemaError::PolicyContract,
    )
    .await?;
    attest_set(
        connection,
        grants_sql(),
        EXPECTED_GRANTS,
        PgK8sLifecycleSchemaError::GrantContract,
    )
    .await
}

async fn attest_runtime_role(
    connection: &mut PgConnection,
) -> Result<(), PgK8sLifecycleSchemaError> {
    let row = sqlx::query(
        "SELECT rolcanlogin, rolsuper, rolinherit, rolcreatedb, rolcreaterole, rolreplication, rolbypassrls, rolconnlimit, rolvaliduntil IS NULL AS no_expiry, rolconfig IS NULL AS no_config FROM pg_roles WHERE rolname = $1",
    )
    .bind(RUNTIME_ROLE)
    .fetch_optional(&mut *connection)
    .await?;
    let Some(row) = row else {
        return Err(PgK8sLifecycleSchemaError::RuntimeRoleContract);
    };
    let flags_match = !row.try_get::<bool, _>("rolcanlogin")?
        && !row.try_get::<bool, _>("rolsuper")?
        && row.try_get::<bool, _>("rolinherit")?
        && !row.try_get::<bool, _>("rolcreatedb")?
        && !row.try_get::<bool, _>("rolcreaterole")?
        && !row.try_get::<bool, _>("rolreplication")?
        && !row.try_get::<bool, _>("rolbypassrls")?
        && row.try_get::<i32, _>("rolconnlimit")? == -1
        && row.try_get::<bool, _>("no_expiry")?
        && row.try_get::<bool, _>("no_config")?;
    let authority: bool = sqlx::query_scalar(
        "SELECT has_schema_privilege($1, $2, 'CREATE') OR NOT has_schema_privilege($1, $2, 'USAGE') OR EXISTS (SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid = m.member WHERE r.rolname = $1)",
    )
    .bind(RUNTIME_ROLE)
    .bind(SCHEMA_NAME)
    .fetch_one(&mut *connection)
    .await?;
    (flags_match && !authority)
        .then_some(())
        .ok_or(PgK8sLifecycleSchemaError::RuntimeRoleContract)
}

async fn attest_namespace(connection: &mut PgConnection) -> Result<(), PgK8sLifecycleSchemaError> {
    let relations: BTreeSet<String> = sqlx::query_scalar(
        "SELECT concat(c.relname, '|', c.relkind::text, '|', c.relpersistence::text, '|', c.relrowsecurity::text, '|', c.relforcerowsecurity::text, '|', c.relispartition::text, '|', am.amname, '|', (c.reltablespace = 0)::text, '|', (c.reloptions IS NULL)::text) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace LEFT JOIN pg_am am ON am.oid = c.relam WHERE n.nspname = $1 AND c.relkind IN ('r', 'p', 'v', 'm', 'S', 'f')",
    )
    .bind(SCHEMA_NAME)
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .collect();
    let unexpected_objects: i64 = sqlx::query_scalar(
        "SELECT ((SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = $1) + (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1 AND NOT t.tgisinternal) + (SELECT count(*) FROM pg_rewrite r JOIN pg_class c ON c.oid = r.ev_class JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1 AND r.rulename <> '_RETURN'))::bigint",
    )
    .bind(SCHEMA_NAME)
    .fetch_one(&mut *connection)
    .await?;
    let inheritance_edges: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_inherits inheritance JOIN pg_class relation ON relation.oid IN (inheritance.inhrelid, inheritance.inhparent) JOIN pg_namespace n ON n.oid = relation.relnamespace WHERE n.nspname = $1",
    ).bind(SCHEMA_NAME).fetch_one(&mut *connection).await?;
    (relations == strings(EXPECTED_RELATIONS) && unexpected_objects == 0 && inheritance_edges == 0)
        .then_some(())
        .ok_or(PgK8sLifecycleSchemaError::NamespaceContract)
}

async fn attest_ownership(connection: &mut PgConnection) -> Result<(), PgK8sLifecycleSchemaError> {
    let matches: Option<bool> = sqlx::query_scalar(
        "SELECT count(*) = 3 AND bool_and(c.relowner = n.nspowner) AND NOT pg_has_role($2, n.nspowner, 'MEMBER') FROM pg_namespace n JOIN pg_class c ON c.relnamespace = n.oid AND c.relkind = 'r' WHERE n.nspname = $1 GROUP BY n.nspowner",
    )
    .bind(SCHEMA_NAME)
    .bind(RUNTIME_ROLE)
    .fetch_optional(connection)
    .await?;
    matches
        .unwrap_or(false)
        .then_some(())
        .ok_or(PgK8sLifecycleSchemaError::OwnershipContract)
}

async fn attest_set(
    connection: &mut PgConnection,
    sql: &'static str,
    expected: &[&str],
    error: PgK8sLifecycleSchemaError,
) -> Result<(), PgK8sLifecycleSchemaError> {
    let actual: BTreeSet<String> = sqlx::query_scalar(sql)
        .bind(SCHEMA_NAME)
        .fetch_all(connection)
        .await?
        .into_iter()
        .collect();
    (actual == strings(expected)).then_some(()).ok_or(error)
}

fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn columns_sql() -> &'static str {
    r#"SELECT concat(c.relname, '|', a.attnum::text, '|', a.attname, '|',
        format('%I.%I', type_namespace.nspname, column_type.typname), '|', a.atttypmod::text, '|',
        a.attnotnull::text, '|', a.attidentity::text, '|', a.attgenerated::text, '|',
        COALESCE(pg_get_expr(d.adbin, d.adrelid, true), ''), '|',
        CASE WHEN a.attcollation = 0 THEN '' ELSE COALESCE((
            SELECT format('%I.%I', cn.nspname, coll.collname)
            FROM pg_collation coll JOIN pg_namespace cn ON cn.oid = coll.collnamespace
            WHERE coll.oid = a.attcollation
        ), '<missing collation>') END)
    FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    JOIN pg_type column_type ON column_type.oid = a.atttypid
    JOIN pg_namespace type_namespace ON type_namespace.oid = column_type.typnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
    WHERE n.nspname = $1 AND c.relkind = 'r' AND a.attnum > 0 AND NOT a.attisdropped"#
}

fn constraints_sql() -> &'static str {
    "SELECT concat(c.relname, '|', p.conname, '|', p.contype::text, '|', p.convalidated::text, '|', p.condeferrable::text, '|', p.condeferred::text, '|', pg_get_constraintdef(p.oid, true)) FROM pg_constraint p JOIN pg_class c ON c.oid = p.conrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1"
}

fn indexes_sql() -> &'static str {
    "SELECT concat(table_rel.relname, '|', index_rel.relname, '|', i.indisunique::text, '|', i.indisprimary::text, '|', i.indisvalid::text, '|', i.indisready::text, '|', i.indislive::text, '|', (index_rel.reltablespace = 0)::text, '|', pg_get_indexdef(i.indexrelid, 0, true)) FROM pg_index i JOIN pg_class table_rel ON table_rel.oid = i.indrelid JOIN pg_class index_rel ON index_rel.oid = i.indexrelid JOIN pg_namespace n ON n.oid = table_rel.relnamespace WHERE n.nspname = $1"
}

fn policies_sql() -> &'static str {
    "SELECT concat(tablename, '|', policyname, '|', permissive, '|', roles::text, '|', cmd, '|', COALESCE(qual, ''), '|', COALESCE(with_check, '')) FROM pg_policies WHERE schemaname = $1"
}

fn grants_sql() -> &'static str {
    "SELECT concat('table|', c.relname, '|', COALESCE(grantee.rolname, 'PUBLIC'), '|', acl.privilege_type, '|', acl.is_grantable::text) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) acl LEFT JOIN pg_roles grantee ON grantee.oid = acl.grantee WHERE n.nspname = $1 AND c.relkind = 'r' AND acl.grantee <> c.relowner UNION ALL SELECT concat('column|', c.relname, '|', a.attname, '|', COALESCE(grantee.rolname, 'PUBLIC'), '|', acl.privilege_type, '|', acl.is_grantable::text) FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid JOIN pg_namespace n ON n.oid = c.relnamespace CROSS JOIN LATERAL aclexplode(a.attacl) acl LEFT JOIN pg_roles grantee ON grantee.oid = acl.grantee WHERE n.nspname = $1 AND c.relkind = 'r' AND a.attnum > 0 AND NOT a.attisdropped AND acl.grantee <> c.relowner UNION ALL SELECT concat('schema|', COALESCE(grantee.rolname, 'PUBLIC'), '|', acl.privilege_type, '|', acl.is_grantable::text) FROM pg_namespace n CROSS JOIN LATERAL aclexplode(n.nspacl) acl LEFT JOIN pg_roles grantee ON grantee.oid = acl.grantee WHERE n.nspname = $1 AND acl.grantee <> n.nspowner"
}
