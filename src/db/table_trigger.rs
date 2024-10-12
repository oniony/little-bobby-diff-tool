use sqlx::{Error, FromRow, PgConnection};

const QUERY : &str = r#"
SELECT
    trigger_catalog,
    trigger_schema,
    trigger_name,
    event_manipulation,
    event_object_catalog,
    event_object_schema,
    event_object_table,
    action_order,
    action_condition,
    action_statement,
    action_orientation,
    action_timing,
    action_reference_old_table,
    action_reference_new_table,
    action_reference_old_row,
    action_reference_new_row
FROM
    information_schema.triggers
WHERE
    trigger_schema = ANY($1)
ORDER BY
    event_object_catalog,
    event_object_schema,
    event_object_table,
    trigger_name,
    event_manipulation;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<TableTrigger>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct TableTrigger {
    pub trigger_catalog: String,
    pub trigger_schema: String,
    pub trigger_name: String,
    pub event_manipulation: String,
    pub event_object_catalog: String,
    pub event_object_schema: String,
    pub event_object_table: String,
    pub action_order: i32,
    pub action_condition: Option<String>,
    pub action_statement: String,
    pub action_orientation: String,
    pub action_timing: String,
    pub action_reference_old_table: Option<String>,
    pub action_reference_new_table: Option<String>,
}

