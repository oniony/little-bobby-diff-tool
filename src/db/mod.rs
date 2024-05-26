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
        let row = self.connection.query_one("SELECT catalog_name \
                                             FROM information_schema.information_schema_catalog_name;",
                                            &[])?;

        let catalog_name : String = row.get(0);
        
        Ok(catalog_name)
    }
    
    pub fn tables(&mut self, schema: &str) -> Result<Vec<Table>, Error> {
        let mut tables = Vec::new();
        
        let rows = self.connection.query("SELECT table_name, table_type, is_insertable_into \
                                          FROM information_schema.tables \
                                          WHERE table_schema = $1;",
                                         &[&schema]);

        for row in rows? {
            let table_name : String = row.get(0);
//            let table_type : &str = row.get(1);
//            let is_insertable_into : bool = row.get(2);
            
            let table = Table {
                name: table_name,
            };
            
            tables.push(table.clone());
        }
        
        Ok(tables)
    }
}

#[derive(Clone)]
pub struct Table {
   pub name: String 
}