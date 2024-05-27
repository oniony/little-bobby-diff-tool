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
    
    pub fn schema(&mut self, schema_name: &str) -> Result<Schema, Error> {
        let row = self.connection.query_one(r#"
SELECT schema_owner
FROM information_schema.schemata
WHERE schema_name = $1;"#,
                                            &[&schema_name])?;
        
        let schema_owner = row.get(0);
        
        let schema = Schema {
            schema_name: String::from(schema_name),
            schema_owner,
        };
        
        Ok(schema)
    }

    pub fn tables(&mut self, schema_name: &str) -> Result<Vec<Table>, Error> {
        let mut tables = Vec::new();
        
        let rows = self.connection.query(r#"
SELECT table_name,
       table_type,
       is_insertable_into
FROM information_schema.tables
WHERE table_schema = $1;"#,
                                         &[&schema_name]);

        for row in rows? {
            let table_name : String = row.get(0);
            let table_type : String = row.get(1);
            let is_insertable_into : String = row.get(2);
            
            let table = Table {
                table_name,
                table_type,
                is_insertable_into,
            };
            
            tables.push(table.clone());
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
                                         &[&schema_name]);

        for row in rows? {
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

            views.push(view.clone());
        }

        Ok(views)
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
                                         &[&schema_name, &table_name]);

        for row in rows? {
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