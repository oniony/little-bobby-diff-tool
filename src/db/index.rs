use sqlx::{Error, FromRow, PgConnection};

const QUERY: &str = r#"SELECT
    schemaname AS table_schema,
    tablename AS table_name,
    indexname AS index_name,
    tablespace AS table_space,
    indexdef AS definition
FROM
    pg_indexes
WHERE
    schemaname = ANY($1)
ORDER BY
    table_schema,
    table_name,
    index_name;"#;

pub async fn indices(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<Index>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Index {
    pub table_schema: String,
    pub table_name: String,
    pub index_name: String,
    pub table_space: Option<String>,
    pub definition: String
}

