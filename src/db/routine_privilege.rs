use sqlx::{Error, FromRow, PgConnection};
use crate::db::privilege::Privilege;

const QUERY: &str = r#"
SELECT
    rp.grantor,
    rp.grantee,
    rp.routine_catalog,
    rp.routine_schema,
    rp.routine_name || '(' || COALESCE((
        SELECT string_agg(COALESCE(p.parameter_name, '$' || p.ordinal_position) || ' ' || p.parameter_mode || ' ' || p.udt_schema || '.' || p.udt_name, ', ' order by p.ordinal_position)
        FROM information_schema.parameters p
        WHERE p.specific_name = rp.specific_name
        GROUP BY p.specific_name
    ), '') || ')' signature,
    rp.privilege_type,
    rp.is_grantable
FROM
    information_schema.routine_privileges rp
WHERE
    rp.routine_schema = ANY($1) AND
    rp.grantor != rp.grantee
ORDER BY
    signature,
    privilege_type;"#;

pub async fn routine_privileges(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<RoutinePrivilege>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct RoutinePrivilege {
    pub grantor: String,
    pub grantee: String,
    pub routine_catalog: String,
    pub routine_schema: String,
    pub signature: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

impl Privilege for &RoutinePrivilege {
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