use sqlx::{Error, FromRow, PgConnection};

pub const QUERY: &str = r#"
SELECT
    p.specific_catalog,
    p.specific_schema,
	p.specific_name,
	p.ordinal_position,
	p.parameter_mode,
	p.is_result,
	p.as_locator,
	p.parameter_name,
	p.data_type,
	p.character_maximum_length,
	p.character_octet_length,
	p.character_set_catalog,
	p.character_set_schema,
	p.character_set_name,
	p.collation_catalog,
	p.collation_schema,
	p.collation_name,
	p.numeric_precision,
	p.numeric_precision_radix,
	p.numeric_scale,
	p.datetime_precision,
	p.interval_type,
	p.interval_precision,
	p.udt_catalog,
	p.udt_schema,
	p.udt_name,
	p.scope_catalog,
	p.scope_schema,
	p.scope_name,
	p.maximum_cardinality,
	p.dtd_identifier,
	p.parameter_default
FROM information_schema.parameters p
WHERE
    p.specific_schema = ANY($1);"#;

pub async fn routine_parameters(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<RoutineParameter>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct RoutineParameter {
    pub specific_catalog: String,
    pub specific_schema: String,
    pub specific_name: String,
    pub ordinal_position: i32,
    pub parameter_mode: String,
    pub is_result: Option<String>,
    pub as_locator: Option<String>,
    pub parameter_name: Option<String>,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub character_octet_length: Option<i32>,
    pub character_set_catalog: Option<String>,
    pub character_set_schema: Option<String>,
    pub character_set_name: Option<String>,
    pub collation_catalog: Option<String>,
    pub collation_schema: Option<String>,
    pub collation_name: Option<String>,
    pub numeric_precision: Option<i32>,
    pub numeric_precision_radix: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub datetime_precision: Option<i32>,
    pub interval_type: Option<String>,
    pub interval_precision: Option<i32>,
    pub udt_catalog: String,
    pub udt_schema: String,
    pub udt_name: String,
    pub scope_catalog: Option<String>,
    pub scope_schema: Option<String>,
    pub scope_name: Option<String>,
    pub maximum_cardinality: Option<i32>,
    pub dtd_identifier: String,
    pub parameter_default: Option<String>,
}

