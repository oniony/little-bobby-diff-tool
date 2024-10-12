use sqlx::{Error, FromRow, PgConnection};

const QUERY: &str = r#"
SELECT
    table_catalog,
    table_schema,
    table_name,
    view_definition,
    check_option,
    is_updatable,
    is_insertable_into,
    is_trigger_updatable,
    is_trigger_deletable,
    is_trigger_insertable_into
FROM
    information_schema.views
WHERE
    table_schema = ANY($1)
ORDER BY
    table_name;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<View>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct View {
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub view_definition: Option<String>,
    pub check_option: String,
    pub is_updatable: String,
    pub is_insertable_into: String,
    pub is_trigger_updatable: String,
    pub is_trigger_deletable: String,
    pub is_trigger_insertable_into: String,
}
