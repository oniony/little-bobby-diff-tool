use std::collections::HashMap;
use std::fmt::Display;
use postgres::Error;
use crate::db::{Database, Table, Column, View};

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

        same = same & self.compare_catalog_name()?;
        same = same & self.compare_tables(schema)?;
        same = same & self.compare_views(schema)?;
        
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
                    println!("table '{}': removed", left_table.table_name);
                },
                Some(rt) => {
                    println!("table '{}':", left_table.table_name);

                    let table_same = self.compare_table(&left_table, rt);
                    let columns_same = self.compare_columns(schema, left_table.table_name.as_str())?;
                    
                    same = same & table_same && columns_same;
                    
                    right_tables_map.remove(&left_table.table_name);
                },
            }
        }
        
        if right_tables_map.len() > 0 {
            same = false;

            for right_table in right_tables_map.values() {
                println!("table '{}': added", right_table.table_name);
            }
        }

        Ok(same)
    }

    fn compare_views(&mut self, schema: &str) -> Result<bool, Error> {
        let left_views = self.left_db.views(schema)?;
        let right_views = self.right_db.views(schema)?;

        let right_views_map : HashMap<String, View> = right_views.into_iter().map(|t| (t.view_name.clone(), t)).collect();
        let mut same = true;

        for left_view in left_views {
            let right_view = right_views_map.get(&left_view.view_name);

            match right_view {
                None => {
                    continue
                },
                Some(rv) => {
                    println!("view '{}':", left_view.view_name);

                    let view_same = self.compare_view(&left_view, rv);

                    same = same & view_same;
                }
            }
        }

        Ok(same)
    }
    
    fn compare_table(&mut self, left: &Table, right: &Table) -> bool {
        let mut same = true;
        
        same = same & self.compare_table_property(&left.table_name, "table_type", &left.table_type, &right.table_type);
        same = same & self.compare_table_property(&left.table_name, "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into);

        same
    }

    fn compare_view(&mut self, left: &View, right: &View) -> bool {
        let mut same = true;

        same = same & self.compare_view_option_property(&left.view_name, "table_type", &left.view_definition, &right.view_definition);
        same = same & self.compare_view_property(&left.view_name, "check_option", &left.check_option, &right.check_option);
        same = same & self.compare_view_property(&left.view_name, "is_updatable", &left.is_updatable, &right.is_updatable);
        same = same & self.compare_view_property(&left.view_name, "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into);
        same = same & self.compare_view_property(&left.view_name, "is_trigger_updatable", &left.is_trigger_updatable, &right.is_trigger_updatable);
        same = same & self.compare_view_property(&left.view_name, "is_trigger_deletable", &left.is_trigger_deletable, &right.is_trigger_deletable);
        same = same & self.compare_view_property(&left.view_name, "is_trigger_insertable_into", &left.is_trigger_insertable_into, &right.is_trigger_insertable_into);

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
                    println!("table '{}': column '{}': removed", table_name, left_column.column_name);
                },
                Some(rc) => {
                    println!("table '{}': column '{}':", table_name, left_column.column_name);
                    
                    let column_same = self.compare_column(table_name, &mut left_column, rc);
                    same = same & column_same;
                    
                    right_columns_map.remove(&left_column.column_name);
                },
            }
        }
        
        Ok(same)
    }
    
    fn compare_column(&mut self, table_name: &str, left: &mut Column, right: &mut Column) -> bool {
        let mut same = true;

        same = same & self.compare_column_option_property(table_name, &left.column_name, "column_default", &left.column_default, &right.column_default);
        same = same & self.compare_column_property(table_name, &left.column_name, "is_nullable", &left.is_nullable, &right.is_nullable);
        same = same & self.compare_column_property(table_name, &left.column_name, "data_type", &left.data_type, &right.data_type);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "character_maximum_length", &left.character_maximum_length, &right.character_maximum_length);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "numeric_precision", &left.numeric_precision, &right.numeric_precision);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "numeric_scale", &left.numeric_scale, &right.numeric_scale);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "datetime_precision", &left.datetime_precision, &right.datetime_precision);
        same = same & self.compare_column_property(table_name, &left.column_name, "is_identity", &left.is_identity, &right.is_identity);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "identity_generation", &left.identity_generation, &right.identity_generation);
        same = same & self.compare_column_property(table_name, &left.column_name, "is_generated", &left.is_generated, &right.is_generated);
        same = same & self.compare_column_option_property(table_name, &left.column_name, "generation_expression", &left.generation_expression, &right.generation_expression);
        same = same & self.compare_column_property(table_name, &left.column_name, "is_updatable", &left.is_updatable, &right.is_updatable);

        same
    }

    fn compare_table_property<T>(&mut self, table_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("table '{}': property '{}': changed from '{}' to '{}'", table_name, property_name, left_value, right_value);
        }

        same
    }

    fn compare_view_property<T>(&mut self, table_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("table '{}': property '{}': changed from '{}' to '{}'", table_name, property_name, left_value, right_value);
        }

        same
    }

    fn compare_view_option_property<T>(&mut self, view_name: &str, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            let l = left_value.as_ref().map_or(String::from("none"), |v| v.to_string());
            let r = right_value.as_ref().map_or(String::from("none"), |v| v.to_string());
            println!("view '{}': property '{}': changed from '{}' to '{}'", view_name, property_name, l, r);
        }

        same
    }


    fn compare_column_option_property<T>(&mut self, table_name: &str, column_name: &str, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            let l = left_value.as_ref().map_or(String::from("none"), |v| v.to_string());
            let r = right_value.as_ref().map_or(String::from("none"), |v| v.to_string());
            println!("table '{}': column '{}': property '{}': changed from '{}' to '{}'", table_name, column_name, property_name, l, r);
        }

        same
    }

    fn compare_column_property<T>(&mut self, table_name: &str, column_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("table '{}': column '{}': property '{}': changed from '{}' to '{}'", table_name, column_name, property_name, left_value, right_value);
        }

        same
    }
}