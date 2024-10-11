use sqlx::{Error, FromRow, PgConnection};
use crate::db::privilege::Privilege;

const QUERY: &str = r#"
SELECT
    grantor,
    grantee,
    table_catalog,
    table_schema,
    table_name,
    privilege_type,
    is_grantable,
    with_hierarchy
FROM
    information_schema.table_privileges
WHERE
    table_schema = ANY($1)
ORDER BY
    table_catalog,
    table_schema,
    table_name,
    grantor,
    grantee,
    privilege_type;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<TablePrivilege>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct TablePrivilege {
    pub grantor: String,
    pub grantee: String,
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub privilege_type: String,
    pub is_grantable: String,
    pub with_hierarchy: String,
}

impl Privilege for &TablePrivilege {
    fn grantor(&self) -> &str {
        &self.grantor
    }

    fn grantee(&self) -> &str {
        &self.grantee
    }
    
    fn privilege_type(&self) -> &str {
        &self.privilege_type
    }
}