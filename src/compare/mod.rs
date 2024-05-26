use std::collections::HashMap;
use postgres::Error;
use crate::db::{Database, Table};

pub struct Comparer {
    left_db: Database,
    right_db: Database,
}

impl Comparer {
    pub fn new(left_db: Database, right_db: Database) -> Comparer {
        Comparer{
            left_db,
            right_db,
        }
    }

    pub fn compare(&mut self, schema: String) -> Result<bool, Error> {
        let mut same = true;

        same = same && self.compare_catalog_name()?;
        same = same && self.compare_tables(schema)?;
        
        Ok(same)
    }

    fn compare_catalog_name(&mut self) -> Result<bool, Error> {
        let left_name = self.left_db.catalog_name()?;
        let right_name = self.right_db.catalog_name()?;

        Ok(left_name == right_name)
    }
    
    fn compare_tables(&mut self, schema: String) -> Result<bool, Error> {
        let left_tables = self.left_db.tables(schema.as_str())?;
        let right_tables = self.right_db.tables(schema.as_str())?;
        
        let right_tables_map : HashMap<String, Table> = right_tables.into_iter().map(|t| (t.name.clone(), t)).collect();
        //TODO turn right tables into map
        
        //TODO for each table in left, find in right, report status
        //TODO for each extra table in right, report as missing in left
        for left_table in left_tables {
            let right_table = right_tables_map.get(&left_table.name);
            println!("left table: {}, right table: {}", left_table.name, right_table.map_or("not found", |t| &t.name))
        }

        Ok(false)
    }
}