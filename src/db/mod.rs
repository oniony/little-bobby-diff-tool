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
    
    pub fn catalog_name(&mut self) -> String {
        let row = self.connection.query_one("SELECT catalog_name FROM information_schema.information_schema_catalog_name;", &[]).unwrap();
        let catalog_name : Option<String> = row.get(0);
        
        catalog_name.unwrap()
    }
}