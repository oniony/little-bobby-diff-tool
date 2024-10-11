use sqlx::{Error, FromRow, PgConnection};

const QUERY: &str = r#"
SELECT
    schema_name,
    schema_owner,
    default_character_set_catalog,
    default_character_set_schema,
    default_character_set_name,
    sql_path
FROM
    information_schema.schemata
WHERE
    schema_name = ANY($1)
ORDER BY
    schema_name;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<Schema>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Schema {
    pub schema_name: String,
    pub schema_owner: String,
    pub default_character_set_catalog: Option<String>,
    pub default_character_set_schema: Option<String>,
    pub default_character_set_name: Option<String>,
    pub sql_path: Option<String>,
}

