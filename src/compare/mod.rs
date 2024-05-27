use std::collections::HashMap;
use std::fmt::Display;
use postgres::Error;
use crate::db::{Database, Table, Column, View, Routine, TableConstraint};

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
        let same = 
            self.compare_schema(schema)? & 
            self.compare_tables(schema)? & 
            self.compare_views(schema)? & 
            self.compare_routines(schema)?;
        
        Ok(same)
    }

    fn compare_schema(&mut self, schema_name: &str) -> Result<bool, Error> {
        let left_schemas = self.left_db.schemas()?;
        let right_schemas = self.right_db.schemas()?;

        let left_schema = left_schemas.iter().find(|s| s.schema_name == schema_name);
        let right_schema = right_schemas.iter().find(|s| s.schema_name == schema_name);

        if left_schema.is_none() {
            println!("schema '{}': missing in left", schema_name);
            return Ok(false)
        }
        
        if right_schema.is_none() {
            println!("schema '{}': missing in right", schema_name);
            return Ok(false)
        }
        
        let same = self.compare_schema_property(schema_name, "schema_owner", &left_schema.unwrap().schema_owner, &right_schema.unwrap().schema_owner);
        
        Ok(same)
    }

    fn compare_tables(&mut self, schema_name: &str) -> Result<bool, Error> {
        let left_tables = self.left_db.tables(schema_name)?;
        let right_tables = self.right_db.tables(schema_name)?;
        
        let mut right_tables_map : HashMap<String, Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();
        let mut same = true;
        
        for left_table in left_tables {
            let right_table = right_tables_map.get(&left_table.table_name);
            
            match right_table {
                None => {
                    println!("schema: '{}': table '{}': removed", schema_name, left_table.table_name);
                    same = false;
                },
                Some(rt) => {
                    let table_same = self.compare_table(schema_name, &left_table, rt);
                    let columns_same = self.compare_table_columns(schema_name, left_table.table_name.as_str())?;
                    let constraints_same = self.compare_table_constraints(schema_name, left_table.table_name.as_str())?;
                    
                    same = same & table_same & columns_same & constraints_same;
                    
                    right_tables_map.remove(&left_table.table_name);
                },
            }
        }
        
        if right_tables_map.len() > 0 {
            for right_table in right_tables_map.values() {
                println!("schema: '{}': table '{}': added", schema_name, right_table.table_name);
            }
            
            same = false;
        }

        Ok(same)
    }

    fn compare_views(&mut self, schema_name: &str) -> Result<bool, Error> {
        let left_views = self.left_db.views(schema_name)?;
        let right_views = self.right_db.views(schema_name)?;

        let right_views_map : HashMap<String, View> = right_views.into_iter().map(|t| (t.view_name.clone(), t)).collect();
        let mut same = true;

        for left_view in left_views {
            let right_view = right_views_map.get(&left_view.view_name);

            match right_view {
                None => {
                    continue
                },
                Some(rv) => {
                    let view_same = self.compare_view(schema_name, &left_view, rv);
                    
                    same = same & view_same;
                }
            }
        }

        Ok(same)
    }
    
    fn compare_routines(&mut self, schema_name: &str) -> Result<bool, Error> {
        let left_routines = self.left_db.routines(schema_name)?;
        let right_routines = self.right_db.routines(schema_name)?;

        let mut right_routines_map : HashMap<String, Routine> = right_routines.into_iter().map(|t| (t.routine_name.clone(), t)).collect();
        let mut same = true;

        for left_routine in left_routines {
            let right_routine = right_routines_map.get(&left_routine.routine_name);

            match right_routine {
                None => {
                    println!("schema: '{}': routine '{}': removed", schema_name, left_routine.routine_name);
                    same = false;
                },
                Some(rr) => {
                    let routine_same = self.compare_routine(schema_name, &left_routine, rr);
                    
                    same = same & routine_same;
                    
                    right_routines_map.remove(&left_routine.routine_name);
                }
            }
        }

        if right_routines_map.len() > 0 {
            for right_routine in right_routines_map.values() {
                println!("schema: '{}': routine '{}': added", schema_name, right_routine.routine_name);
            }

            same = false;
        }
        
        Ok(same)
    }
    
    fn compare_table(&mut self, schema_name: &str, left: &Table, right: &Table) -> bool {
        let same = 
            self.compare_entity_property(schema_name, "table", &left.table_name, "table_type", &left.table_type, &right.table_type) &
            self.compare_entity_property(schema_name, "table", &left.table_name, "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into);

        same
    }

    fn compare_view(&mut self, schema_name: &str, left: &View, right: &View) -> bool {
        let same = 
            self.compare_entity_option_property(schema_name, "view", &left.view_name, "view_definition", &left.view_definition, &right.view_definition) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "check_option", &left.check_option, &right.check_option) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "is_updatable", &left.is_updatable, &right.is_updatable) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "is_trigger_updatable", &left.is_trigger_updatable, &right.is_trigger_updatable) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "is_trigger_deletable", &left.is_trigger_deletable, &right.is_trigger_deletable) &
            self.compare_entity_property(schema_name, "view", &left.view_name, "is_trigger_insertable_into", &left.is_trigger_insertable_into, &right.is_trigger_insertable_into);

        same
    }

    fn compare_routine(&mut self, schema_name: &str, left: &Routine, right: &Routine) -> bool {
        let same =
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "routine_type", &left.routine_type, &right.routine_type) &
            self.compare_entity_option_property(schema_name, "routine", &left.routine_name, "routine_type", &left.data_type, &right.data_type) &
            self.compare_entity_option_property(schema_name, "routine", &left.routine_name, "type_udt_name", &left.type_udt_name, &right.type_udt_name) &
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "routine_body", &left.routine_body, &right.routine_body) &
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "routine_definition", &left.routine_definition, &right.routine_definition) &
            self.compare_entity_option_property(schema_name, "routine", &left.routine_name, "external_name", &left.external_name, &right.external_name) &
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "external_language", &left.external_language, &right.external_language) &
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "is_deterministic", &left.is_deterministic, &right.is_deterministic) &
            self.compare_entity_option_property(schema_name, "routine", &left.routine_name, "is_null_call", &left.is_null_call, &right.is_null_call) &
            self.compare_entity_property(schema_name, "routine", &left.routine_name, "security_type", &left.security_type, &right.security_type);

        same
    }
    
    fn compare_table_columns(&mut self, schema_name: &str, table_name: &str) -> Result<bool, Error> {
        let left_columns = self.left_db.columns(schema_name, table_name)?;
        let right_columns = self.right_db.columns(schema_name, table_name)?;
        
        let mut right_columns_map : HashMap<String, Column> = right_columns.into_iter().map(|c| (c.column_name.clone(), c)).collect();
        
        let mut same = true;
        for mut left_column in left_columns {
            let right_column = right_columns_map.get_mut(&left_column.column_name);
            
            match right_column {
                None => {
                    same = false;
                    println!("schema: '{}': table '{}': column '{}': removed", schema_name, table_name, left_column.column_name);
                },
                Some(rc) => {
                    let column_same = self.compare_table_column(schema_name, table_name, &mut left_column, rc);
                    same = same & column_same;
                    
                    right_columns_map.remove(&left_column.column_name);
                },
            }
        }

        if right_columns_map.len() > 0 {
            for right_column in right_columns_map.values() {
                println!("schema: '{}': table '{}': column '{}': added", schema_name, table_name, right_column.column_name);
            }

            same = false;
        }

        Ok(same)
    }

    fn compare_table_constraints(&mut self, schema_name: &str, table_name: &str) -> Result<bool, Error> {
        let left_constraints = self.left_db.table_constraints(schema_name, table_name)?;
        let right_constraints = self.right_db.table_constraints(schema_name, table_name)?;

        let mut right_constraint_map : HashMap<String, TableConstraint> = right_constraints.into_iter().map(|c| (c.constraint_name.clone(), c)).collect();

        let mut same = true;
        for mut left_constraint in left_constraints {
            let right_constraint = right_constraint_map.get_mut(&left_constraint.constraint_name);

            match right_constraint {
                None => {
                    same = false;
                    println!("schema: '{}': table '{}': constraint '{}': removed", schema_name, table_name, left_constraint.constraint_name);
                },
                Some(rc) => {
                    let constraint_same = self.compare_table_constraint(schema_name, table_name, &mut left_constraint, rc);
                    same = same & constraint_same;

                    right_constraint_map.remove(&left_constraint.constraint_name);
                },
            }
        }

        if right_constraint_map.len() > 0 {
            for right_constraint in right_constraint_map.values() {
                println!("schema: '{}': table '{}': constraint '{}': added", schema_name, table_name, right_constraint.constraint_name);
            }

            same = false;
        }

        Ok(same)
    }
    
    fn compare_table_column(&mut self, schema_name: &str, table_name: &str, left: &mut Column, right: &mut Column) -> bool {
        let same =
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "column_default", &left.column_default, &right.column_default) &
            self.compare_column_property(schema_name, table_name, &left.column_name, "is_nullable", &left.is_nullable, &right.is_nullable) &
            self.compare_column_property(schema_name, table_name, &left.column_name, "data_type", &left.data_type, &right.data_type) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "character_maximum_length", &left.character_maximum_length, &right.character_maximum_length) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "numeric_precision", &left.numeric_precision, &right.numeric_precision) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "numeric_scale", &left.numeric_scale, &right.numeric_scale) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "datetime_precision", &left.datetime_precision, &right.datetime_precision) &
            self.compare_column_property(schema_name, table_name, &left.column_name, "is_identity", &left.is_identity, &right.is_identity) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "identity_generation", &left.identity_generation, &right.identity_generation) &
            self.compare_column_property(schema_name, table_name, &left.column_name, "is_generated", &left.is_generated, &right.is_generated) &
            self.compare_column_option_property(schema_name, table_name, &left.column_name, "generation_expression", &left.generation_expression, &right.generation_expression) &
            self.compare_column_property(schema_name, table_name, &left.column_name, "is_updatable", &left.is_updatable, &right.is_updatable);

        same
    }

    fn compare_table_constraint(&mut self, schema_name: &str, table_name: &str, left: &mut TableConstraint, right: &mut TableConstraint) -> bool {
        let same =
            self.compare_table_constraint_property(schema_name, table_name, &left.constraint_name, "constraint_type", &left.constraint_type, &right.constraint_type) &
            self.compare_table_constraint_property(schema_name, table_name, &left.constraint_name, "is_deferrable", &left.is_deferrable, &right.is_deferrable) &
            self.compare_table_constraint_property(schema_name, table_name, &left.constraint_name, "initially_deferred", &left.initially_deferred, &right.initially_deferred) &
            self.compare_table_constraint_option_property(schema_name, table_name, &left.constraint_name, "nulls_distinct", &left.nulls_distinct, &right.nulls_distinct);

        same
    }

    //TODO combine these comparison functions

    fn compare_schema_property<T>(&mut self, schema_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("schema: '{}': property '{}': changed from '{}' to '{}'", schema_name, property_name, left_value, right_value);
        }

        same
    }

    fn compare_entity_property<T>(&mut self, schema_name: &str, entity_type: &str, entity_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("schema: '{}': {} '{}': property '{}': changed from '{}' to '{}'", schema_name, entity_type, entity_name, property_name, left_value, right_value);
        }

        same
    }

    fn compare_entity_option_property<T>(&mut self, schema_name: &str, entity_type: &str, entity_name: &str, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            let l = left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            let r = right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            println!("schema: '{}': {} '{}': property '{}': changed from '{}' to '{}'", schema_name, entity_type, entity_name, property_name, l, r);
        }

        same
    }

    fn compare_column_option_property<T>(&mut self, schema_name: &str, table_name: &str, column_name: &str, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            let l = left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            let r = right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            println!("schema: '{}': table '{}': column '{}': property '{}': changed from '{}' to '{}'", schema_name, table_name, column_name, property_name, l, r);
        }

        same
    }

    fn compare_column_property<T>(&mut self, schema_name: &str, table_name: &str, column_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("schema: '{}': table '{}': column '{}': property '{}': changed from '{}' to '{}'", schema_name, table_name, column_name, property_name, left_value, right_value);
        }

        same
    }

    fn compare_table_constraint_property<T>(&mut self, schema_name: &str, table_name: &str, constraint_name: &str, property_name: &str, left_value: T, right_value: T) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            println!("schema: '{}': table '{}': constraint '{}': property '{}': changed from '{}' to '{}'", schema_name, table_name, constraint_name, property_name, left_value, right_value);
        }

        same
    }
    
    fn compare_table_constraint_option_property<T>(&mut self, schema_name: &str, table_name: &str, constraint_name: &str, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> bool where T: PartialEq, T: Display {
        let same = left_value == right_value;

        if !same {
            let l = left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            let r = right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
            println!("schema: '{}': table '{}': constraint '{}': property '{}': changed from '{}' to '{}'", schema_name, table_name, constraint_name, property_name, l, r);
        }

        same
    }
}