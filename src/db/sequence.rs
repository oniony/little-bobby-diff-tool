use sqlx::{Error, FromRow, PgConnection};

const QUERY : &str = r#"
SELECT
    sequence_catalog,
    sequence_schema,
    sequence_name,
    data_type,
    numeric_precision,
    numeric_precision_radix,
    numeric_scale,
    start_value,
    minimum_value,
    maximum_value,
    increment,
    cycle_option
FROM
    information_schema.sequences
WHERE
    sequence_schema = ANY($1)
ORDER BY
    sequence_catalog,
    sequence_schema,
    sequence_name;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<Sequence>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Sequence {
    pub sequence_catalog: String,
    pub sequence_schema: String,
    pub sequence_name: String,
    pub data_type: String,
    pub numeric_precision: i32,
    pub numeric_precision_radix: i32,
    pub numeric_scale: i32,
    pub start_value: String,
    pub minimum_value: String,
    pub maximum_value: String,
    pub increment: String,
    pub cycle_option: String,
}

