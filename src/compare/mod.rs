use std::collections::HashMap;
use postgres::Error;
use crate::db::{Database, Table, Column};

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

    pub fn compare(&mut self, schema: &str) -> Result<bool, Error> {
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
    
    fn compare_tables(&mut self, schema: &str) -> Result<bool, Error> {
        let left_tables = self.left_db.tables(schema)?;
        let right_tables = self.right_db.tables(schema)?;
        
        let mut right_tables_map : HashMap<String, Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();
        
        let mut same = true;
        for left_table in left_tables {
            let right_table = right_tables_map.get(&left_table.table_name);
            
            match right_table {
                None => {
                    same = false;
                    println!("table '{}': removed in right", left_table.table_name);
                },
                Some(rt) => {
                    println!("table '{}':", left_table.table_name);

                    let table_same = self.compare_table(&left_table, rt);
                    let columns_same = self.compare_columns(schema, left_table.table_name.as_str())?;
                    
                    same = same && table_same && columns_same;
                    
                    right_tables_map.remove(&left_table.table_name);
                },
            }
        }
        
        if right_tables_map.len() > 0 {
            same = false;

            for right_table in right_tables_map.values() {
                println!("table '{}': added in right", right_table.table_name);
            }
        }

        Ok(same)
    }
    
    fn compare_table(&mut self, left: &Table, right: &Table) -> bool {
        let mut same = true;
        
        if left.table_type != right.table_type {
            same = false;
            println!("table '{}': type changed from '{}' to '{}'", left.table_name, left.table_type, right.table_type);
        }
        
        if left.is_insertable_into != right.is_insertable_into {
            same = false;
            println!("table '{}': insertable changed from '{}' to '{}'", left.table_name, left.is_insertable_into, right.is_insertable_into);
        }
        
        same
    }

    fn compare_columns(&mut self, schema: &str, table_name: &str) -> Result<bool, Error> {
        let left_columns = self.left_db.columns(schema, table_name)?;
        let right_columns = self.right_db.columns(schema, table_name)?;
        
        let mut right_columns_map : HashMap<String, Column> = right_columns.into_iter().map(|c| (c.column_name.clone(), c)).collect();
        
        let mut same = true;
        for mut left_column in left_columns {
            let right_column = right_columns_map.get_mut(&left_column.column_name);
            
            match right_column {
                None => {
                    same = false;
                    println!("table '{}': column '{}': removed in right", table_name, left_column.column_name);
                },
                Some(rc) => {
                    println!("table '{}': column '{}':", table_name, left_column.column_name);
                    
                    let column_same = self.compare_column(table_name, &mut left_column, rc);
                    same = same && column_same;
                    
                    right_columns_map.remove(&left_column.column_name);
                },
            }
        }
        
        Ok(false)
    }
    
    fn compare_column(&mut self, table_name: &str, left: &mut Column, right: &mut Column) -> bool {
        let mut same = true;

        if left.column_default.clone() != right.column_default.clone() {
            same = false;
            println!("table '{}': column '{}': column_default changed from '{}' to '{}'", table_name, left.column_name, left.column_default.clone().unwrap_or(String::new()), right.column_default.clone().unwrap_or(String::from("")));
        }
        
        if left.is_nullable != right.is_nullable {
            same = false;
            println!("table '{}': column '{}': is_nullable changed from '{}' to '{}'", table_name, left.column_name, left.is_nullable, right.is_nullable);
        }

        if left.data_type != right.data_type {
            same = false;
            println!("table '{}': column '{}': data_type changed from '{}' to '{}'", table_name, left.column_name, left.data_type, right.data_type);
        }
        
        if left.character_maximum_length != right.character_maximum_length {
            same = false;
            println!("table '{}': column '{}': character_maximum_length changed from '{}' to '{}'", table_name, left.column_name, left.data_type, right.data_type);
        }

        if left.numeric_precision != right.numeric_precision {
            same = false;
            println!("table '{}': column '{}': numeric_precision changed from '{}' to '{}'", table_name, left.column_name, left.numeric_precision.clone().map_or(String::new(), |np| np.to_string()), right.numeric_precision.clone().map_or(String::new(), |np| np.to_string()));
        }
        
        if left.numeric_scale != right.numeric_scale {
            same = false;
            println!("table '{}': column '{}': numeric_scale changed from '{}' to '{}'", table_name, left.column_name, left.numeric_scale.clone().map_or(String::new(), |np| np.to_string()), right.numeric_scale.clone().map_or(String::new(), |np| np.to_string()));
        }

        if left.datetime_precision != right.datetime_precision {
            same = false;
            println!("table '{}': column '{}': datetime_precision changed from '{}' to '{}'", table_name, left.column_name, left.datetime_precision.clone().map_or(String::new(), |np| np.to_string()), right.datetime_precision.clone().map_or(String::new(), |np| np.to_string()));
        }
        
        if left.is_identity != right.is_identity {
            same = false;
            println!("table '{}': column '{}': is_identity changed from '{}' to '{}'", table_name, left.column_name, left.is_identity, right.is_identity);
        }

        if left.identity_generation != right.identity_generation {
            same = false;
            println!("table '{}': column '{}': identity_generation changed from '{}' to '{}'", table_name, left.column_name, left.identity_generation.clone().map_or(String::new(), |np| np.to_string()), right.identity_generation.clone().map_or(String::new(), |np| np.to_string()));
        }

        if left.is_generated != right.is_generated {
            same = false;
            println!("table '{}': column '{}': is_generated changed from '{}' to '{}'", table_name, left.column_name, left.is_generated, right.is_generated);
        }

        if left.generation_expression != right.generation_expression {
            same = false;
            println!("table '{}': column '{}': generation_expresion changed from '{}' to '{}'", table_name, left.column_name, left.generation_expression.clone().map_or(String::new(), |np| np.to_string()), right.generation_expression.clone().map_or(String::new(), |np| np.to_string()));
        }

        if left.is_updatable != right.is_updatable {
            same = false;
            println!("table '{}': column '{}': is_updatable changed from '{}' to '{}'", table_name, left.column_name, left.is_updatable, right.is_updatable);
        }

        same
    }
}