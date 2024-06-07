pub(crate) mod thing;

use postgres::{Client, NoTls, Error};
use crate::db::thing::{Column, Routine, Schema, Sequence, Table, TableConstraint, Trigger, View};

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
    
    pub fn schemas(&mut self) -> Result<Vec<Schema>, Error> {
        let mut schemas = Vec::new();

        let rows = self.connection.query(r#"
SELECT schema_name, schema_owner
FROM information_schema.schemata
ORDER BY schema_name;"#,
                                            &[])?;
        
        for row in rows {
            let schema_name = row.get(0);
            let schema_owner = row.get(1);

            let schema = Schema {
                schema_name,
                schema_owner,
            };

            schemas.push(schema);
        }
        
        Ok(schemas)
    }

    pub fn tables(&mut self, schema_name: &str) -> Result<Vec<Table>, Error> {
        let mut tables = Vec::new();
        
        let rows = self.connection.query(r#"
SELECT table_name,
       table_type,
       is_insertable_into
FROM information_schema.tables
WHERE table_schema = $1
ORDER BY table_name;"#,
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
    
    pub fn views(&mut self, schema_name: &str) -> Result<Vec<View>, Error> {
        let mut views = Vec::new();

        let rows = self.connection.query(r#"
SELECT table_name,
       view_definition,
       check_option,
       is_updatable,
       is_insertable_into,
       is_trigger_updatable,
       is_trigger_deletable,
       is_trigger_insertable_into
FROM information_schema.views
WHERE table_schema = $1
ORDER BY table_name;"#,
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
    
    pub fn routines(&mut self, schema_name: &str) -> Result<Vec<Routine>, Error> {
        let mut routines = Vec::new();

        let rows = self.connection.query(r#"
SELECT r.routine_name || '(' || COALESCE((
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
FROM information_schema.routines r
WHERE r.routine_schema = $1
ORDER BY signature;"#,
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

    pub fn columns(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<Column>, Error> {
        let mut columns = Vec::new();

        let rows = self.connection.query(r#"
SELECT column_name,
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
FROM information_schema.columns
WHERE table_schema = $1 AND
      table_name = $2
ORDER BY table_name, column_name;"#,
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

    pub fn table_constraints(&mut self, schema_name: &str) -> Result<Vec<TableConstraint>, Error> {
        let mut table_constraints = Vec::new();

        let rows = self.connection.query(r#"
SELECT constraint_name,
       table_name,
       constraint_type,
       is_deferrable,
       initially_deferred,
       nulls_distinct
FROM information_schema.table_constraints
WHERE table_schema = $1 AND
      constraint_type != 'CHECK'
ORDER BY constraint_name;"#,
                                         &[&schema_name])?;

        for row in rows {
            let constraint_name: String = row.get(0);
            let table_name: String = row.get(1);
            let constraint_type: String = row.get(2);
            let is_deferrable: String = row.get(3);
            let initially_deferred: String = row.get(4);
            let nulls_distinct: Option<String> = row.get(5);

            let table_constraint = TableConstraint {
                constraint_name,
                table_name,
                constraint_type,
                is_deferrable,
                initially_deferred,
                nulls_distinct,
            };

            table_constraints.push(table_constraint.clone());
        }

        Ok(table_constraints)
    }

    pub fn sequences(&mut self, schema_name: &str) -> Result<Vec<Sequence>, Error> {
        let mut sequences = Vec::new();

        let rows = self.connection.query(r#"
SELECT sequence_name,
       data_type,
       numeric_precision,
       numeric_precision_radix,
       numeric_scale,
       start_value,
       minimum_value,
       maximum_value,
       increment,
       cycle_option
FROM information_schema.sequences
WHERE sequence_schema = $1
ORDER BY sequence_name;"#,
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

    pub fn triggers(&mut self, schema_name: &str) -> Result<Vec<Trigger>, Error> {
        let mut triggers = Vec::new();

        let rows = self.connection.query(r#"
SELECT trigger_name,
       event_manipulation,
       event_object_schema,
       event_object_table,
       action_order,
       action_condition,
       action_statement,
       action_orientation,
       action_timing,
       action_reference_old_table,
       action_reference_new_table
FROM information_schema.triggers
WHERE trigger_schema = $1
ORDER BY trigger_name;"#,
                                         &[&schema_name])?;

        for row in rows {
            let trigger_name: String = row.get(0);
            let event_manipulation: String = row.get(1);
            let event_object_schema: String = row.get(2);
            let event_object_table: String = row.get(3);
            let action_order: i32 = row.get(4);
            let action_condition: Option<String> = row.get(5);
            let action_statement: String = row.get(6);
            let action_orientation: String = row.get(7);
            let action_timing: String = row.get(8);
            let action_reference_old_table: Option<String> = row.get(9);
            let action_reference_new_table: Option<String> = row.get(10);

            let sequence = Trigger {
                trigger_name,
                event_manipulation,
                event_object_schema,
                event_object_table,
                action_order,
                action_condition,
                action_statement,
                action_orientation,
                action_timing,
                action_reference_old_table,
                action_reference_new_table,
            };

            triggers.push(sequence);
        }

        Ok(triggers)
    }
}
