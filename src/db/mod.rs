pub mod thing;

use postgres::{Client, NoTls, Error};
use crate::db::thing::{Column, Constraint, Privilege, Routine, Schema, Sequence, Table, Trigger, View};

pub struct Database {
    connection: Client
}

impl Database {
    pub fn connect(url: &str) -> Result<Database, Error> {
        let client = Client::connect(url, NoTls)?;

        Ok(Database {
            connection: client
        })
    }

    pub fn columns(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<Column>, Error> {
        let mut columns = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    column_name,
    ordinal_position,
    column_default,
    is_nullable,
    data_type,
    character_maximum_length,
    numeric_precision,
    numeric_scale,
    datetime_precision,
    is_identity,
    identity_generation,
    is_generated,
    generation_expression,
    is_updatable
FROM
    information_schema.columns
WHERE
    table_schema = $1 AND
    table_name = $2
ORDER BY
    table_name,
    column_name;"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let column_name: String = row.get(0);
            let ordinal_position: i32 = row.get(1);
            let column_default: Option<String> = row.get(2);
            let is_nullable: String = row.get(3);
            let data_type: String = row.get(4);
            let character_maximum_length: Option<i32> = row.get(5);
            let numeric_precision: Option<i32> = row.get(6);
            let numeric_scale: Option<i32> = row.get(7);
            let datetime_precision: Option<i32> = row.get(8);
            let is_identity: String = row.get(9);
            let identity_generation: Option<String> = row.get(10);
            let is_generated: String = row.get(11);
            let generation_expression: Option<String> = row.get(12);
            let is_updatable: String = row.get(13);

            let column = Column {
                column_name,
                ordinal_position,
                column_default,
                is_nullable,
                data_type,
                character_maximum_length,
                numeric_precision,
                numeric_scale,
                datetime_precision,
                is_identity,
                identity_generation,
                is_generated,
                generation_expression,
                is_updatable,
            };

            columns.push(column.clone());
        }

        Ok(columns)
    }

    pub fn column_privileges(&mut self, schema_name: &str, table_name: &str, column_name: &str) -> Result<Vec<Privilege>, Error> {
        let mut column_privileges = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    grantor,
    grantee,
    privilege_type,
    is_grantable
FROM
    information_schema.column_privileges
WHERE
    table_schema = $1 AND
    table_name = $2 AND
    column_name = $3 AND
    grantor != grantee
ORDER BY
    table_name,
    column_name,
    privilege_type,
    grantor,
    grantee;"#,
                                         &[&schema_name, &table_name, &column_name])?;

        for row in rows {
            let grantor: String = row.get(0);
            let grantee: String = row.get(1);
            let privilege_type: String = row.get(2);
            let is_grantable: String = row.get(3);

            let column_privilege = Privilege {
                grantor,
                grantee,
                privilege_type,
                is_grantable,
            };

            column_privileges.push(column_privilege.clone());
        }

        Ok(column_privileges)
    }

    pub fn routines(&mut self, schema_name: &str) -> Result<Vec<Routine>, Error> {
        let mut routines = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    r.routine_name || '(' || COALESCE((
	    SELECT string_agg(COALESCE(p.parameter_name, '$' || p.ordinal_position) || ' ' || p.parameter_mode || ' ' || p.udt_schema || '.' || p.udt_name, ', ' order by p.ordinal_position)
        FROM information_schema.parameters p
        WHERE p.specific_name = r.specific_name
        GROUP BY p.specific_name
    ), '') || ')' signature,
    r.routine_type,
    r.data_type,
    r.type_udt_name,
    r.routine_body,
    r.routine_definition,
    r.external_name,
    r.external_language,
    r.is_deterministic,
    r.is_null_call,
    r.security_type
FROM
    information_schema.routines r
WHERE
    r.routine_schema = $1
ORDER BY
    signature;"#,
                                         &[&schema_name])?;

        for row in rows {
            let signature: String = row.get(0);
            let routine_type: Option<String> = row.get(1);
            let data_type: Option<String> = row.get(2);
            let type_udt_name: Option<String> = row.get(3);
            let routine_body: String = row.get(4);
            let routine_definition: Option<String> = row.get(5);
            let external_name: Option<String> = row.get(6);
            let external_language: String = row.get(7);
            let is_deterministic: String = row.get(8);
            let is_null_call: Option<String> = row.get(9);
            let security_type: String = row.get(10);

            let routine = Routine {
                signature,
                routine_type,
                data_type,
                type_udt_name,
                routine_body,
                routine_definition,
                external_name,
                external_language,
                is_deterministic,
                is_null_call,
                security_type,
            };

            routines.push(routine);
        }

        Ok(routines)
    }

    pub fn routine_privileges(&mut self, schema_name: &str, routine_signature: &str) -> Result<Vec<Privilege>, Error> {
        let mut routine_privileges = Vec::new();

        let rows = self.connection.query(r#"
WITH schema_routines AS (
    SELECT
        rp.grantor,
        rp.grantee,
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
        rp.routine_schema = $1 AND
        rp.grantor != rp.grantee
)
SELECT
    grantor,
    grantee,
    privilege_type,
    is_grantable
FROM
    schema_routines
WHERE
    signature = $2
ORDER BY
    signature,
    privilege_type;"#,
                                         &[&schema_name, &routine_signature])?;

        for row in rows {
            let grantor: String = row.get(0);
            let grantee: String = row.get(1);
            let privilege_type: String = row.get(2);
            let is_grantable: String = row.get(3);

            let routine_privilege = Privilege {
                grantor,
                grantee,
                privilege_type,
                is_grantable,
            };

            routine_privileges.push(routine_privilege);
        }

        Ok(routine_privileges)
    }

    pub fn schema(&mut self, schema_name: &str) -> Result<Option<Schema>, Error> {
        let row = self.connection.query_opt(r#"
SELECT
    schema_owner
FROM
    information_schema.schemata
WHERE
    schema_name = $1
ORDER BY
    schema_name;"#,
                                            &[&schema_name])?;
        
        match row {
            None => Ok(None),
            Some(row) => {
                let schema_owner = row.get(0);

                let schema = Schema {
                    schema_name: String::from(schema_name),
                    schema_owner,
                };

                Ok(Some(schema))
            }
        }
    }

    pub fn sequences(&mut self, schema_name: &str) -> Result<Vec<Sequence>, Error> {
        let mut sequences = Vec::new();

        let rows = self.connection.query(r#"
SELECT
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
    sequence_schema = $1
ORDER BY
    sequence_name;"#,
                                         &[&schema_name])?;

        for row in rows {
            let sequence_name: String = row.get(0);
            let data_type: String = row.get(1);
            let numeric_precision: i32 = row.get(2);
            let numeric_precision_radix: i32 = row.get(3);
            let numeric_scale: i32 = row.get(4);
            let start_value: String = row.get(5);
            let minimum_value: String = row.get(6);
            let maximum_value: String = row.get(7);
            let increment: String = row.get(8);
            let cycle_option: String = row.get(9);

            let sequence = Sequence {
                sequence_name,
                data_type,
                numeric_precision,
                numeric_precision_radix,
                numeric_scale,
                start_value,
                minimum_value,
                maximum_value,
                increment,
                cycle_option,
            };

            sequences.push(sequence);
        }

        Ok(sequences)
    }

    pub fn tables(&mut self, schema_name: &str) -> Result<Vec<Table>, Error> {
        let mut tables = Vec::new();
        
        let rows = self.connection.query(r#"
SELECT
    table_name,
    table_type,
    is_insertable_into
FROM
    information_schema.tables
WHERE
    table_schema = $1
ORDER BY
    table_name;"#,
                                         &[&schema_name])?;

        for row in rows {
            let table_name : String = row.get(0);
            let table_type : String = row.get(1);
            let is_insertable_into : String = row.get(2);
            
            let table = Table {
                table_name,
                table_type,
                is_insertable_into,
            };
            
            tables.push(table);
        }
        
        Ok(tables)
    }

    pub fn table_constraints(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<Constraint>, Error> {
        let mut table_constraints = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    constraint_name,
    constraint_type,
    is_deferrable,
    initially_deferred,
    nulls_distinct
FROM
    information_schema.table_constraints
WHERE
    table_schema = $1 AND
    table_name = $2 AND
    constraint_type != 'CHECK'
ORDER BY
    table_name,
    constraint_name;"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let constraint_name: String = row.get(0);
            let constraint_type: String = row.get(1);
            let is_deferrable: String = row.get(2);
            let initially_deferred: String = row.get(3);
            let nulls_distinct: Option<String> = row.get(4);

            let table_constraint = Constraint {
                constraint_name,
                constraint_type,
                is_deferrable,
                initially_deferred,
                nulls_distinct,
            };

            table_constraints.push(table_constraint.clone());
        }

        Ok(table_constraints)
    }

    pub fn table_privileges(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<Privilege>, Error> {
        let mut table_privileges = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    grantor,
    grantee,
    privilege_type,
    is_grantable
FROM
    information_schema.table_privileges
WHERE
    table_schema = $1 AND
    table_name = $2 AND
    grantor != grantee
ORDER BY
    privilege_type,
    grantor,
    grantee;"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let grantor: String = row.get(0);
            let grantee: String = row.get(1);
            let privilege_type: String = row.get(2);
            let is_grantable: String = row.get(3);

            let table_privilege = Privilege {
                grantor,
                grantee,
                privilege_type,
                is_grantable,
            };

            table_privileges.push(table_privilege.clone());
        }

        Ok(table_privileges)
    }

    pub fn triggers(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<Trigger>, Error> {
        let mut triggers = Vec::new();

        let rows = self.connection.query(r#"
SELECT
    trigger_name,
    event_manipulation,
    action_order,
    action_condition,
    action_statement,
    action_orientation,
    action_timing,
    action_reference_old_table,
    action_reference_new_table
FROM
    information_schema.triggers
WHERE
    trigger_schema = $1 AND
    event_object_schema = $1 AND
    event_object_table = $2
ORDER BY
    trigger_name, event_manipulation;"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let trigger_name: String = row.get(0);
            let event_manipulation: String = row.get(1);
            let action_order: i32 = row.get(2);
            let action_condition: Option<String> = row.get(3);
            let action_statement: String = row.get(4);
            let action_orientation: String = row.get(5);
            let action_timing: String = row.get(6);
            let action_reference_old_table: Option<String> = row.get(7);
            let action_reference_new_table: Option<String> = row.get(8);

            let trigger = Trigger {
                trigger_name,
                event_manipulation,
                action_order,
                action_condition,
                action_statement,
                action_orientation,
                action_timing,
                action_reference_old_table,
                action_reference_new_table,
            };

            triggers.push(trigger);
        }

        Ok(triggers)
    }

    pub fn views(&mut self, schema_name: &str) -> Result<Vec<View>, Error> {
        let mut views = Vec::new();

        let rows = self.connection.query(r#"
SELECT
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
    table_schema = $1
ORDER BY
    table_name;"#,
                                         &[&schema_name])?;

        for row in rows {
            let view_name: String = row.get(0);
            let view_definition: Option<String> = row.get(1);
            let check_option: String = row.get(2);
            let is_updatable: String = row.get(3);
            let is_insertable_into: String = row.get(4);
            let is_trigger_updatable: String = row.get(5);
            let is_trigger_deletable: String = row.get(6);
            let is_trigger_insertable_into: String = row.get(7);

            let view = View {
                view_name,
                view_definition,
                check_option,
                is_updatable,
                is_insertable_into,
                is_trigger_updatable,
                is_trigger_deletable,
                is_trigger_insertable_into,
            };

            views.push(view);
        }

        Ok(views)
    }
}
