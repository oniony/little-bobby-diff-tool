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
    
    pub fn catalog_name(&mut self) -> Result<String, Error> {
        let row = self.connection.query_one(r#"
SELECT catalog_name
FROM information_schema.information_schema_catalog_name;"#,
                                            &[])?;

        let catalog_name : String = row.get(0);
        
        Ok(catalog_name)
    }
    
    pub fn tables(&mut self, schema: &str) -> Result<Vec<Table>, Error> {
        let mut tables = Vec::new();
        
        let rows = self.connection.query(r#"
SELECT table_name,
       table_type,
       is_insertable_into
FROM information_schema.tables
WHERE table_schema = $1;"#,
                                         &[&schema]);

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
    
    pub fn columns(&mut self, schema: &str, table_name: &str) -> Result<Vec<Column>, Error> {
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
                                         &[&schema, &table_name]);

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
