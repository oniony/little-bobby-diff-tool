use sqlx::{Error, FromRow, PgConnection};

pub const QUERY: &str = r#"
SELECT
    r.routine_catalog,
    r.routine_schema,
    r.routine_name || '(' || COALESCE((
	    SELECT string_agg(COALESCE(p.parameter_name, '$' || p.ordinal_position) || ' ' || p.parameter_mode || ' ' || p.udt_schema || '.' || p.udt_name, ', ' order by p.ordinal_position)
        FROM information_schema.parameters p
        WHERE p.specific_name = r.specific_name
        GROUP BY p.specific_name
    ), '') || ')' signature,
    r.routine_type,
    r.module_catalog,
    r.module_schema,
    r.module_name,
    r.udt_catalog,
    r.udt_schema,
    r.udt_name,
    r.data_type,
    r.character_maximum_length,
    r.character_octet_length,
    r.character_set_catalog,
    r.character_set_schema,
    r.character_set_name,
    r.collation_catalog,
    r.collation_schema,
    r.collation_name,
    r.numeric_precision,
    r.numeric_precision_radix,
    r.numeric_scale,
    r.datetime_precision,
    r.interval_type,
    r.interval_precision,
    r.type_udt_catalog,
    r.type_udt_schema,
    r.type_udt_name,
    r.maximum_cardinality,
    r.dtd_identifier,
    r.routine_body,
    r.routine_definition,
    r.external_name,
    r.external_language,
    r.parameter_style,
    r.is_deterministic,
    r.sql_data_access,
    r.is_null_call,
    r.sql_path,
    r.schema_level_routine,
    r.max_dynamic_result_sets,
    r.is_user_defined_cast,
    r.is_implicitly_invocable,
    r.security_type,
    r.is_udt_dependent
FROM
    information_schema.routines r
WHERE
    r.routine_schema = ANY($1)
ORDER BY
    signature;"#;

pub async fn routines(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<Routine>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Routine {
    pub routine_catalog: String,
    pub routine_schema: String,
    pub signature: String,
    pub routine_type: Option<String>,
    pub module_catalog: Option<String>,
    pub module_schema: Option<String>,
    pub module_name: Option<String>,
    pub udt_catalog: Option<String>,
    pub udt_schema: Option<String>,
    pub udt_name: Option<String>,
    pub data_type: Option<String>,
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
    pub type_udt_catalog: Option<String>,
    pub type_udt_schema: Option<String>,
    pub type_udt_name: Option<String>,
    pub maximum_cardinality: Option<i32>,
    pub dtd_identifier: Option<String>,
    pub routine_body: String,
    pub routine_definition: Option<String>,
    pub external_name: Option<String>,
    pub external_language: String,
    pub parameter_style: String,
    pub is_deterministic: String,
    pub sql_data_access: String,
    pub is_null_call: Option<String>,
    pub sql_path: Option<String>,
    pub schema_level_routine: String,
    pub max_dynamic_result_sets: Option<i32>,
    pub is_user_defined_cast: Option<String>,
    pub is_implicitly_invocable: Option<String>,
    pub security_type: String,
    pub is_udt_dependent: String,
}

