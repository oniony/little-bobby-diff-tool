pub(crate) mod report;

use std::collections::HashMap;
use std::fmt::{Display};
use postgres::Error;
use crate::compare::report::{Report, ReportEntry, Thing};
use crate::compare::report::ReportEntry::{Addition, Change, Match, Removal};
use crate::compare::report::Thing::{Column, Constraint, Property, Routine, Schema, Sequence, Table, View};
use crate::db;
use crate::db::{Database};
use crate::string::{EqualIgnoreWhitespace};

pub struct Comparer {
    left_db: Database,
    right_db: Database,
    ignore_whitespace: bool,
    ignore_column_ordinal: bool,
}

impl Comparer {
    pub fn new(left_db: Database, right_db: Database, ignore_whitespace: bool, ignore_column_ordinal: bool) -> Comparer {
        Comparer{
            left_db,
            right_db,
            ignore_whitespace,
            ignore_column_ordinal,
        }
    }

    pub fn compare(&mut self, schema: &str) -> Result<Report, Error> {
        let mut report = Report::new();
        
        let mut schema_entries = self.compare_schema(schema)?;
        report.entries.append(&mut schema_entries);
        
        let mut table_entries = self.compare_tables(schema)?;
        report.entries.append(&mut table_entries);
        
        let mut view_entries = self.compare_views(schema)?;
        report.entries.append(&mut view_entries);

        let mut routine_entries = self.compare_routines(schema)?;
        report.entries.append(&mut routine_entries);
        
        let mut sequence_entries = self.compare_sequences(schema)?;
        report.entries.append(&mut sequence_entries);
        
        Ok(report)
    }

    fn compare_schema(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_schemas = self.left_db.schemas()?;
        let right_schemas = self.right_db.schemas()?;

        let left_schema = left_schemas.iter().find(|s| s.schema_name == schema_name);
        let right_schema = right_schemas.iter().find(|s| s.schema_name == schema_name);
        
        let mut entries = Vec::new();
        
        if left_schema.is_none() {
            entries.push(Addition { path: vec![], thing: Schema(String::from(schema_name)) });
        }
        if right_schema.is_none() {
            entries.push(Removal { path: vec![], thing: Schema(String::from(schema_name)) });
        }

        entries.push(self.compare_property(vec![Schema(String::from(schema_name))], "schema_owner", &left_schema.unwrap().schema_owner, &right_schema.unwrap().schema_owner));
        
        Ok(entries)
    }

    fn compare_tables(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_tables = self.left_db.tables(schema_name)?;
        let right_tables = self.right_db.tables(schema_name)?;
        
        let mut right_tables_map : HashMap<String, db::Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_table in left_tables {
            let right_table = right_tables_map.get(&left_table.table_name);
            
            match right_table {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name))], thing: Table(left_table.table_name) });
                },
                Some(rt) => {
                    let mut table_entries = self.compare_table(schema_name, &left_table, rt)?;
                    entries.append(&mut table_entries);

                    let mut column_entries = self.compare_table_columns(schema_name, left_table.table_name.as_str())?;
                    entries.append(&mut column_entries);
                    
                    let mut constraint_entries = self.compare_table_constraints(schema_name, left_table.table_name.as_str())?;
                    entries.append(&mut constraint_entries);
                    
                    right_tables_map.remove(&left_table.table_name);
                },
            }
        }
        
        if right_tables_map.len() > 0 {
            for right_table in right_tables_map.values() {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Table(right_table.table_name.clone()) });
            }
        }
    
        Ok(entries)
    }

    fn compare_views(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_views = self.left_db.views(schema_name)?;
        let right_views = self.right_db.views(schema_name)?;

        let right_views_map: HashMap<String, db::View> = right_views.into_iter().map(|t| (t.view_name.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_view in left_views {
            let right_view = right_views_map.get(&left_view.view_name);

            match right_view {
                None => {
                    // already detected by table comparison
                    continue
                },
                Some(rv) => {
                    let mut view_entries = self.compare_view(schema_name, &left_view, rv);
                    entries.append(&mut view_entries);
                }
            }
        }

        Ok(entries)
    }

    fn compare_routines(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_routines = self.left_db.routines(schema_name)?;
        let right_routines = self.right_db.routines(schema_name)?;

        let mut right_routines_map : HashMap<String, db::Routine> = right_routines.into_iter().map(|t| (t.routine_name.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_routine in left_routines {
            let right_routine = right_routines_map.get(&left_routine.routine_name);

            match right_routine {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name))], thing: Routine(left_routine.routine_name) });
                },
                Some(rr) => {
                    let mut routine_entries = self.compare_routine(schema_name, &left_routine, rr);
                    entries.append(&mut routine_entries);

                    right_routines_map.remove(&left_routine.routine_name);
                }
            }
        }

        if right_routines_map.len() > 0 {
            for right_routine in right_routines_map.values() {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Routine(right_routine.routine_name.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_sequences(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_sequences = self.left_db.sequences(schema_name)?;
        let right_sequences = self.right_db.sequences(schema_name)?;
    
        let mut right_sequences_map : HashMap<String, db::Sequence> = right_sequences.into_iter().map(|t| (t.sequence_name.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_sequence in left_sequences {
            let right_sequence = right_sequences_map.get(&left_sequence.sequence_name);
    
            match right_sequence {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name))], thing: Sequence(left_sequence.sequence_name) });
                },
                Some(rs) => {
                    let mut sequence_entries = self.compare_sequence(schema_name, &left_sequence, rs);
                    entries.append(&mut sequence_entries);

                    right_sequences_map.remove(&left_sequence.sequence_name);
                }
            }
        }
    
        if right_sequences_map.len() > 0 {
            for right_sequence in right_sequences_map.values() {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Sequence(right_sequence.sequence_name.clone()) });
            }
        }
        
        Ok(entries)
    }
    
    fn compare_table(&mut self, schema_name: &str, left: &db::Table, right: &db::Table) -> Result<Vec<ReportEntry>, Error> {
        let mut entries = Vec::new();
        
        //TODO work out how to better clone the path
        let path = || vec![Schema(String::from(schema_name)), Table(left.table_name.clone())];
        
        entries.push(self.compare_property(path(), "table_type", &left.table_type, &right.table_type));
        entries.push(self.compare_property(path(), "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into));
        
        Ok(entries)
    }

    fn compare_view(&mut self, schema_name: &str, left: &db::View, right: &db::View) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), View(left.view_name.clone())];
        
        entries.push(self.compare_option_property(path(), "view_definition", &left.view_definition, &right.view_definition));
        entries.push(self.compare_property(path(), "check_option", &left.check_option, &right.check_option));
        entries.push(self.compare_property(path(), "is_updatable", &left.is_updatable, &right.is_updatable));
        entries.push(self.compare_property(path(), "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into));
        entries.push(self.compare_property(path(), "is_trigger_updatable", &left.is_trigger_updatable, &right.is_trigger_updatable));
        entries.push(self.compare_property(path(), "is_trigger_deletable", &left.is_trigger_deletable, &right.is_trigger_deletable));
        entries.push(self.compare_property(path(), "is_trigger_insertable_into", &left.is_trigger_insertable_into, &right.is_trigger_insertable_into));
    
        entries
    }

    fn compare_routine(&mut self, schema_name: &str, left: &db::Routine, right: &db::Routine) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Routine(left.routine_name.clone())];
        
        entries.push(self.compare_property(path(), "routine_type", &left.routine_type, &right.routine_type));
        entries.push(self.compare_option_property(path(), "data_type", &left.data_type, &right.data_type));
        entries.push(self.compare_option_property(path(), "type_udt_name", &left.type_udt_name, &right.type_udt_name));
        entries.push(self.compare_property(path(), "routine_body", &left.routine_body, &right.routine_body));
        
        entries.push(
            if self.ignore_whitespace {
                self.compare_option_property_ignore_whitespace(path(), "routine_definition", &left.routine_definition, &right.routine_definition)
            } else {
                self.compare_option_property(path(), "routine_definition", &left.routine_definition, &right.routine_definition)
            }
        );
        
        entries.push(self.compare_option_property(path(), "external_name", &left.external_name, &right.external_name));
        entries.push(self.compare_property(path(), "external_language", &left.external_language, &right.external_language));
        entries.push(self.compare_property(path(), "is_deterministic", &left.is_deterministic, &right.is_deterministic));
        entries.push(self.compare_option_property(path(), "is_null_call", &left.is_null_call, &right.is_null_call));
        entries.push(self.compare_property(path(), "security_type", &left.security_type, &right.security_type));
        
        entries
    }

    fn compare_sequence(&mut self, schema_name: &str, left: &db::Sequence, right: &db::Sequence) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Sequence(left.sequence_name.clone())];

        entries.push(self.compare_property(path(), "data_type", &left.data_type, &right.data_type));
        entries.push(self.compare_property(path(), "numeric_precision", &left.numeric_precision, &right.numeric_precision));
        entries.push(self.compare_property(path(), "numeric_precision_radix", &left.numeric_precision_radix, &right.numeric_precision_radix));
        entries.push(self.compare_property(path(), "numeric_scale", &left.numeric_scale, &right.numeric_scale));
        entries.push(self.compare_property(path(), "start_value", &left.start_value, &right.start_value));
        entries.push(self.compare_property(path(), "minimum_value", &left.minimum_value, &right.minimum_value));
        entries.push(self.compare_property(path(), "maximum_value", &left.maximum_value, &right.maximum_value));
        entries.push(self.compare_property(path(), "increment", &left.increment, &right.increment));
        entries.push(self.compare_property(path(), "cycle_option", &left.cycle_option, &right.cycle_option));

        entries
    }

    fn compare_table_columns(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_columns = self.left_db.columns(schema_name, table_name)?;
        let right_columns = self.right_db.columns(schema_name, table_name)?;
        
        let mut right_columns_map : HashMap<String, db::Column> = right_columns.into_iter().map(|c| (c.column_name.clone(), c)).collect();
        let mut entries = Vec::new();
        
        for mut left_column in left_columns {
            let right_column = right_columns_map.get_mut(&left_column.column_name);
            
            match right_column {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(String::from(table_name))], thing: Column(left_column.column_name) });
                },
                Some(rc) => {
                    let mut column_entries = self.compare_table_column(schema_name, table_name, &mut left_column, rc);
                    entries.append(&mut column_entries);

                    right_columns_map.remove(&left_column.column_name);
                },
            }
        }
    
        if right_columns_map.len() > 0 {
            for right_column in right_columns_map.values() {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(String::from(table_name))], thing: Column(right_column.column_name.clone()) });
            }
        }
    
        Ok(entries)
    }

    fn compare_table_constraints(&mut self, schema_name: &str, table_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_constraints = self.left_db.table_constraints(schema_name, table_name)?;
        let right_constraints = self.right_db.table_constraints(schema_name, table_name)?;
    
        let mut right_constraint_map : HashMap<String, db::TableConstraint> = right_constraints.into_iter().map(|c| (c.constraint_name.clone(), c)).collect();
        let mut entries = Vec::new();
        
        for mut left_constraint in left_constraints {
            let right_constraint = right_constraint_map.get_mut(&left_constraint.constraint_name);
    
            match right_constraint {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(String::from(table_name))], thing: Constraint(left_constraint.constraint_name) });
                },
                Some(rc) => {
                    let mut constraint_entries = self.compare_table_constraint(schema_name, table_name, &mut left_constraint, rc);
                    entries.append(&mut constraint_entries);

                    right_constraint_map.remove(&left_constraint.constraint_name);
                },
            }
        }
    
        if right_constraint_map.len() > 0 {
            for right_constraint in right_constraint_map.values() {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(String::from(table_name))], thing: Constraint(right_constraint.constraint_name.clone()) });
            }
        }
    
        Ok(entries)
    }
    
    fn compare_table_column(&mut self, schema_name: &str, table_name: &str, left: &mut db::Column, right: &mut db::Column) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Table(String::from(table_name)), Column(left.column_name.clone())];
        
        if !self.ignore_column_ordinal {
            entries.push(self.compare_property(path(), "ordinal_position", &left.ordinal_position, &right.ordinal_position));
        }
        
        entries.push(self.compare_option_property(path(), "column_default", &left.column_default, &right.column_default));
        entries.push(self.compare_property(path(), "is_nullable", &left.is_nullable, &right.is_nullable));
        entries.push(self.compare_property(path(), "data_type", &left.data_type, &right.data_type));
        entries.push(self.compare_option_property(path(), "character_maximum_length", &left.character_maximum_length, &right.character_maximum_length));
        entries.push(self.compare_option_property(path(), "numeric_precision", &left.numeric_precision, &right.numeric_precision));
        entries.push(self.compare_option_property(path(), "numeric_scale", &left.numeric_scale, &right.numeric_scale));
        entries.push(self.compare_option_property(path(), "datetime_precision", &left.datetime_precision, &right.datetime_precision));
        entries.push(self.compare_property(path(), "is_identity", &left.is_identity, &right.is_identity));
        entries.push(self.compare_option_property(path(), "identity_generation", &left.identity_generation, &right.identity_generation));
        entries.push(self.compare_property(path(), "is_generated", &left.is_generated, &right.is_generated));
        entries.push(self.compare_option_property(path(), "generation_expression", &left.generation_expression, &right.generation_expression));
        entries.push(self.compare_property(path(), "is_updatable", &left.is_updatable, &right.is_updatable));
        
        entries
    }

    fn compare_table_constraint(&mut self, schema_name: &str, table_name: &str, left: &mut db::TableConstraint, right: &mut db::TableConstraint) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Table(String::from(table_name)), Constraint(left.constraint_name.clone())];

        entries.push(self.compare_property(path(), "constraint_type", &left.constraint_type, &right.constraint_type));
        entries.push(self.compare_property(path(), "is_deferrable", &left.is_deferrable, &right.is_deferrable));
        entries.push(self.compare_property(path(), "initially_deferred", &left.initially_deferred, &right.initially_deferred));
        entries.push(self.compare_option_property(path(), "nulls_distinct", &left.nulls_distinct, &right.nulls_distinct));
    
        entries
    }

    fn compare_property<T>(&mut self, mut path: Vec<Thing>, property_name: &str, left_value: T, right_value: T) -> ReportEntry
        where T: PartialEq, T: Display {
        path.push(Property(String::from(property_name)));
        
        match left_value == right_value {
            true => Match { path, left_value: left_value.to_string(), right_value: right_value.to_string() },
            false => Change { path, left_value: left_value.to_string(), right_value: right_value.to_string() },
        }
    }
    
    fn compare_option_property<T>(&mut self, mut path: Vec<Thing>, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> ReportEntry
        where T: PartialEq, T: Display {
        path.push(Property(String::from(property_name)));

        let l = left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
        let r = right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
        
        match left_value == right_value {
            true => Match { path, left_value: l, right_value: r },
            false => Change { path, left_value: l, right_value: r },
        }
    }

    fn compare_option_property_ignore_whitespace<T>(&mut self, mut path: Vec<Thing>, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> ReportEntry
        where T: PartialEq, T: Display {
        path.push(Property(String::from(property_name)));
        
        let l = left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());
        let r = right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string());

        match l.as_str().eq_ignore_whitespace(r.as_str()) {
            true => Match { path, left_value: l, right_value: r },
            false => Change { path, left_value: l, right_value: r },
        }
    }
}
