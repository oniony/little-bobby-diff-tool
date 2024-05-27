use postgres::{Client, NoTls, Error};

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
FROM information_schema.schemata;"#,
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
WHERE table_schema = $1;"#,
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
WHERE table_schema = $1;"#,
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
SELECT routine_name,
       routine_type,
       data_type,
       type_udt_name,
       routine_body,
       routine_definition,
       external_name,
       external_language,
       is_deterministic,
       is_null_call,
       security_type
FROM information_schema.routines
WHERE routine_schema = $1;"#,
                                         &[&schema_name])?;

        for row in rows {
            let routine_name: String = row.get(0);
            let routine_type: String = row.get(1);
            let data_type: Option<String> = row.get(2);
            let type_udt_name: Option<String> = row.get(3);
            let routine_body: String = row.get(4);
            let routine_definition: String = row.get(5);
            let external_name: Option<String> = row.get(6);
            let external_language: String = row.get(7);
            let is_deterministic: String = row.get(8);
            let is_null_call: Option<String> = row.get(9);
            let security_type: String = row.get(10);

            let routine = Routine {
                routine_name,
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
      table_name = $2;"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let column_name: String = row.get(0);
            let column_default: Option<String> = row.get(1);
            let is_nullable: String = row.get(2);
            let data_type: String = row.get(3);
            let character_maximum_length: Option<i32> = row.get(4);
            let numeric_precision: Option<i32> = row.get(5);
            let numeric_scale: Option<i32> = row.get(6);
            let datetime_precision: Option<i32> = row.get(7);
            let is_identity: String = row.get(8);
            let identity_generation: Option<String> = row.get(9);
            let is_generated: String = row.get(10);
            let generation_expression: Option<String> = row.get(11);
            let is_updatable: String = row.get(12);

            let column = Column {
                column_name,
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

    pub fn table_constraints(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<TableConstraint>, Error> {
        let mut table_constraints = Vec::new();

        //NOTE CHECK constraints are currently filtered out as they have unpredictable names

        let rows = self.connection.query(r#"
SELECT constraint_name,
       constraint_type,
       is_deferrable,
       initially_deferred,
       nulls_distinct
FROM information_schema.table_constraints
WHERE table_schema = $1 AND
      table_name = $2 AND
      constraint_type != 'CHECK';"#,
                                         &[&schema_name, &table_name])?;

        for row in rows {
            let constraint_name: String = row.get(0);
            let constraint_type: String = row.get(1);
            let is_deferrable: String = row.get(2);
            let initially_deferred: String = row.get(3);
            let nulls_distinct: Option<String> = row.get(4);

            let table_constraint = TableConstraint {
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
}

#[derive(Clone, PartialEq)]
pub struct Schema {
    pub schema_name: String,
    pub schema_owner: String,
}

#[derive(Clone, PartialEq)]
pub struct Table {
   pub table_name: String,
   pub table_type: String,
   pub is_insertable_into: String,
}

#[derive(Clone, PartialEq)]
pub struct Column {
    pub column_name: String,
    pub column_default: Option<String>,
    pub is_nullable: String,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub datetime_precision: Option<i32>,
    pub is_identity: String,
    pub identity_generation: Option<String>,
    pub is_generated: String,
    pub generation_expression: Option<String>,
    pub is_updatable: String,
}

#[derive(Clone, PartialEq)]
pub struct View {
    pub view_name: String,
    pub view_definition: Option<String>,
    pub check_option: String,
    pub is_updatable: String,
    pub is_insertable_into: String,
    pub is_trigger_updatable: String,
    pub is_trigger_deletable: String,
    pub is_trigger_insertable_into: String,
}

#[derive(Clone, PartialEq)]
pub struct Routine {
    pub routine_name: String,
    pub routine_type: String,
    pub data_type: Option<String>,
    pub type_udt_name: Option<String>,
    pub routine_body: String,
    pub routine_definition: String,
    pub external_name: Option<String>,
    pub external_language: String,
    pub is_deterministic: String,
    pub is_null_call: Option<String>,
    pub security_type: String,
}

#[derive(Clone, PartialEq)]
pub struct TableConstraint {
    pub constraint_name: String,
    pub constraint_type: String,
    pub is_deferrable: String,
    pub initially_deferred: String,
    pub nulls_distinct: Option<String>,
}
