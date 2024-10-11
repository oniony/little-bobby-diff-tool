use sqlx::{Error, FromRow, PgConnection};

const QUERY : &str = r#"
SELECT
    table_catalog,
    table_schema,
    table_name,
    table_type,
    self_referencing_column_name,
    reference_generation,
    user_defined_type_catalog,
    user_defined_type_schema,
    user_defined_type_name,
    is_insertable_into,
    is_typed,
    commit_action
FROM
    information_schema.tables
WHERE
    table_schema = ANY($1)
ORDER BY
    table_catalog,
    table_schema,
    table_name;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<Table>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Table {
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub table_type: String,
    pub self_referencing_column_name: Option<String>,
    pub reference_generation: Option<String>,
    pub user_defined_type_catalog: Option<String>,
    pub user_defined_type_schema: Option<String>,
    pub user_defined_type_name: Option<String>,
    pub is_insertable_into: String,
    pub is_typed: String,
    pub commit_action: Option<String>,
}
