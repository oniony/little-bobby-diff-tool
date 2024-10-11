use sqlx::{Error, FromRow, PgConnection};
use crate::db::privilege::Privilege;

const QUERY: &str = r#"
SELECT
    grantor,
    grantee,
    table_catalog,
    table_schema,
    table_name,
    column_name,
    privilege_type,
    is_grantable
FROM
    information_schema.column_privileges
WHERE
    table_schema = ANY($1)
ORDER BY
    table_catalog,
    table_schema,
    table_name,
    column_name,
    grantor,
    grantee,
    privilege_type;"#;

pub async fn column_privileges(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<ColumnPrivilege>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct ColumnPrivilege {
    pub grantor: String,
    pub grantee: String,
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

impl Privilege for &ColumnPrivilege {
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