pub(crate) mod report;

use std::collections::HashMap;
use std::fmt::{Display};
use postgres::Error;
use crate::compare::report::{Report, ReportEntry, Thing};
use crate::compare::report::ReportEntry::{Addition, Change, Match, Removal};
use crate::compare::report::Thing::{Column, TableConstraint, Property, Routine, Schema, Sequence, Table, Trigger, View, ColumnPrivilege, TablePrivilege, RoutinePrivilege};
use crate::{db};
use crate::db::{Database};
use crate::string::{EqualIgnoreWhitespace};

pub struct Comparer {
    left_db: Database,
    right_db: Database,
    ignore_whitespace: bool,
    ignore_column_ordinal: bool,
    ignore_privileges: bool,
}

impl Comparer {
    pub fn new(left_db: Database, right_db: Database, ignore_whitespace: bool, ignore_column_ordinal: bool, ignore_privileges: bool) -> Comparer {
        Comparer{
            left_db,
            right_db,
            ignore_whitespace,
            ignore_column_ordinal,
            ignore_privileges,
        }
    }

    pub fn compare(&mut self, schema: &str) -> Result<Report, Error> {
        let mut report = Report::new();

        report.entries.append(&mut self.compare_schema(schema)?);
        report.entries.append(&mut self.compare_columns(schema)?);
        if !self.ignore_privileges { report.entries.append(&mut self.compare_column_privileges(schema)?); }
        report.entries.append(&mut self.compare_table_constraints(schema)?);
        report.entries.append(&mut self.compare_routines(schema)?);
        if !self.ignore_privileges { report.entries.append(&mut self.compare_routine_privileges(schema)?); }
        report.entries.append(&mut self.compare_sequences(schema)?);
        report.entries.append(&mut self.compare_tables(schema)?);
        if !self.ignore_privileges { report.entries.append(&mut self.compare_table_privileges(schema)?); }
        report.entries.append(&mut self.compare_triggers(schema)?);
        report.entries.append(&mut self.compare_views(schema)?);

        Ok(report)
    }

    fn compare_columns(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_columns = self.left_db.columns(schema_name)?;
        let right_columns = self.right_db.columns(schema_name)?;

        let mut right_columns_map : HashMap<(String, String), db::thing::Column> = right_columns.into_iter().map(|c| ((c.table_name.clone(), c.column_name.clone()), c)).collect();
        let mut entries = Vec::new();

        for mut left_column in left_columns {
            let right_column = right_columns_map.get_mut(&(left_column.table_name.clone(), left_column.column_name.clone()));

            match right_column {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(String::from(&left_column.table_name.clone()))], thing: Column(left_column.column_name.clone()) });
                },
                Some(rc) => {
                    entries.append(&mut self.compare_column(schema_name, &left_column.table_name.clone(), &mut left_column, rc));
                    right_columns_map.remove(&(left_column.table_name.clone(), left_column.column_name.clone()));
                },
            }
        }

        if right_columns_map.len() > 0 {
            let mut added_columns : Vec<&db::thing::Column> = right_columns_map.values().collect();
            added_columns.sort_unstable_by_key(|c| (&c.table_name, &c.column_name));
            
            for right_column in added_columns {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(right_column.table_name.clone())], thing: Column(right_column.column_name.clone()) });
            }
        }

        Ok(entries)
    }
    
    fn compare_column_privileges(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_column_privileges = self.left_db.column_privileges(schema_name)?;
        let right_column_privileges = self.right_db.column_privileges(schema_name)?;

        let mut right_column_privileges_map : HashMap<(String, String, String, String, String), db::thing::ColumnPrivilege> = right_column_privileges.into_iter().map(|c| ((c.table_name.clone(), c.column_name.clone(), c.privilege_type.clone(), c.grantor.clone(), c.grantee.clone()), c)).collect();
        let mut entries = Vec::new();

        for left_column_privilege in left_column_privileges {
            let right_column_privilege = right_column_privileges_map.get_mut(&(left_column_privilege.table_name.clone(), left_column_privilege.column_name.clone(), left_column_privilege.privilege_type.clone(), left_column_privilege.grantor.clone(), left_column_privilege.grantee.clone()));

            match right_column_privilege {
                None => entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(String::from(left_column_privilege.table_name.clone())), Column(String::from(left_column_privilege.column_name.clone()))], thing: ColumnPrivilege(left_column_privilege.privilege_type, left_column_privilege.grantor.clone(), left_column_privilege.grantee.clone()) }),
                Some(..) => _ = right_column_privileges_map.remove(&(left_column_privilege.table_name, left_column_privilege.column_name.clone(), left_column_privilege.privilege_type.clone(), left_column_privilege.grantor.clone(), left_column_privilege.grantee.clone())),
            }
        }

        if right_column_privileges_map.len() > 0 {
            let mut added_column_privileges : Vec<&db::thing::ColumnPrivilege> = right_column_privileges_map.values().collect();
            added_column_privileges.sort_unstable_by_key(|cp| (&cp.table_name, &cp.column_name, &cp.privilege_type, &cp.grantor, &cp.grantee));
            
            for right_column_privilege in added_column_privileges {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(right_column_privilege.table_name.clone()), Column(right_column_privilege.column_name.clone())], thing: ColumnPrivilege(right_column_privilege.privilege_type.clone(), right_column_privilege.grantor.clone(), right_column_privilege.grantee.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_routines(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_routines = self.left_db.routines(schema_name)?;
        let right_routines = self.right_db.routines(schema_name)?;

        let mut right_routines_map : HashMap<String, db::thing::Routine> = right_routines.into_iter().map(|t| (t.signature.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_routine in left_routines {
            let right_routine = right_routines_map.get(&left_routine.signature);

            match right_routine {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name))], thing: Routine(left_routine.signature) });
                },
                Some(rr) => {
                    let mut routine_entries = self.compare_routine(schema_name, &left_routine, rr);
                    entries.append(&mut routine_entries);

                    right_routines_map.remove(&left_routine.signature);
                }
            }
        }

        if right_routines_map.len() > 0 {
            let mut added_routines : Vec<&db::thing::Routine> = right_routines_map.values().collect();
            added_routines.sort_unstable_by_key(|r| &r.signature);
            
            for right_routine in added_routines {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Routine(right_routine.signature.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_routine_privileges(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_routine_privileges = self.left_db.routine_privileges(schema_name)?;
        let right_routine_privileges = self.right_db.routine_privileges(schema_name)?;

        let mut right_routine_privileges_map : HashMap<(String, String, String, String), db::thing::RoutinePrivilege> = right_routine_privileges.into_iter().map(|c| ((c.signature.clone(), c.privilege_type.clone(), c.grantor.clone(), c.grantee.clone()), c)).collect();
        let mut entries = Vec::new();

        for left_routine_privilege in left_routine_privileges {
            let right_routine_privilege = right_routine_privileges_map.get_mut(&(left_routine_privilege.signature.clone(), left_routine_privilege.privilege_type.clone(), left_routine_privilege.grantor.clone(), left_routine_privilege.grantee.clone()));

            match right_routine_privilege {
                None => entries.push(Removal { path: vec![Schema(String::from(schema_name)), Routine(String::from(left_routine_privilege.signature.clone()))], thing: RoutinePrivilege(left_routine_privilege.privilege_type.clone(), left_routine_privilege.grantor.clone(), left_routine_privilege.grantee.clone()) }),
                Some(..) => _ = right_routine_privileges_map.remove(&(left_routine_privilege.signature, left_routine_privilege.privilege_type.clone(), left_routine_privilege.grantor.clone(), left_routine_privilege.grantee.clone())),
            }
        }

        if right_routine_privileges_map.len() > 0 {
            let mut added_routine_privileges : Vec<&db::thing::RoutinePrivilege> = right_routine_privileges_map.values().collect();
            added_routine_privileges.sort_unstable_by_key(|rp| (&rp.signature, &rp.privilege_type, &rp.grantor, &rp.grantee));

            for right_routine_privilege in added_routine_privileges {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Routine(right_routine_privilege.signature.clone())], thing: RoutinePrivilege(right_routine_privilege.privilege_type.clone(), right_routine_privilege.grantor.clone(), right_routine_privilege.grantee.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_schema(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_schemas = self.left_db.schemas()?;
        let right_schemas = self.right_db.schemas()?;

        let left_schema = left_schemas.iter().find(|s| s.schema_name == schema_name);
        let right_schema = right_schemas.iter().find(|s| s.schema_name == schema_name);
        
        let mut entries = Vec::new();
        
        if left_schema.is_none() && right_schema.is_none() {
            // nothing to compare
        } else {
            if left_schema.is_none() {
                entries.push(Addition { path: vec![], thing: Schema(String::from(schema_name)) });
            } else if right_schema.is_none() {
                entries.push(Removal { path: vec![], thing: Schema(String::from(schema_name)) });
            } else {
                entries.push(self.compare_property(vec![Schema(String::from(schema_name))], "schema_owner", &left_schema.unwrap().schema_owner, &right_schema.unwrap().schema_owner));
            }
        }

        Ok(entries)
    }

    fn compare_sequences(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_sequences = self.left_db.sequences(schema_name)?;
        let right_sequences = self.right_db.sequences(schema_name)?;

        let mut right_sequences_map : HashMap<String, db::thing::Sequence> = right_sequences.into_iter().map(|t| (t.sequence_name.clone(), t)).collect();
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
            let mut added_sequences : Vec<&db::thing::Sequence> = right_sequences_map.values().collect();
            added_sequences.sort_unstable_by_key(|s| &s.sequence_name);

            for right_sequence in added_sequences {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Sequence(right_sequence.sequence_name.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_tables(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_tables = self.left_db.tables(schema_name)?;
        let right_tables = self.right_db.tables(schema_name)?;
        
        let mut right_tables_map : HashMap<String, db::thing::Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();
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

                    right_tables_map.remove(&left_table.table_name);
                },
            }
        }
        
        if right_tables_map.len() > 0 {
            let mut added_tables : Vec<&db::thing::Table> = right_tables_map.values().collect();
            added_tables.sort_unstable_by_key(|t| &t.table_name);
            
            for right_table in added_tables {
                entries.push(Addition { path: vec![Schema(String::from(schema_name))], thing: Table(right_table.table_name.clone()) });
            }
        }
    
        Ok(entries)
    }

    fn compare_table_constraints(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_table_constraints = self.left_db.table_constraints(schema_name)?;
        let right_table_constraints = self.right_db.table_constraints(schema_name)?;

        let mut right_table_constraints_map : HashMap<(String, String), db::thing::TableConstraint> = right_table_constraints.into_iter().map(|t| ((t.table_name.clone(), t.constraint_name.clone()), t)).collect();
        let mut entries = Vec::new();

        for left_table_constraint in left_table_constraints {
            let right_table_constraint = right_table_constraints_map.get(&(left_table_constraint.table_name.clone(), left_table_constraint.constraint_name.clone()));

            match right_table_constraint {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(left_table_constraint.table_name.clone())], thing: TableConstraint(left_table_constraint.constraint_name) });
                },
                Some(rtc) => {
                    let mut table_constraint_entries = self.compare_table_constraint(schema_name, &left_table_constraint, rtc);
                    entries.append(&mut table_constraint_entries);

                    right_table_constraints_map.remove(&(left_table_constraint.table_name.clone(), left_table_constraint.constraint_name.clone()));
                },
            }
        }

        if right_table_constraints_map.len() > 0 {
            let mut added_table_constraints : Vec<&db::thing::TableConstraint> = right_table_constraints_map.values().collect();
            added_table_constraints.sort_unstable_by_key(|tc| (&tc.table_name, &tc.constraint_name));
            
            for right_table_constraint in added_table_constraints {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(right_table_constraint.table_name.clone())], thing: TableConstraint(right_table_constraint.constraint_name.clone()) });
            }
        }

        Ok(entries)
    }
    
    fn compare_table_privileges(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_table_privileges = self.left_db.table_privileges(schema_name)?;
        let right_table_privileges = self.right_db.table_privileges(schema_name)?;

        let mut right_table_privileges_map : HashMap<(String, String, String, String), db::thing::TablePrivilege> = right_table_privileges.into_iter().map(|c| ((c.table_name.clone(), c.privilege_type.clone(), c.grantor.clone(), c.grantee.clone()), c)).collect();
        let mut entries = Vec::new();

        for left_table_privilege in left_table_privileges {
            let right_table_privilege = right_table_privileges_map.get_mut(&(left_table_privilege.table_name.clone(), left_table_privilege.privilege_type.clone(), left_table_privilege.grantor.clone(), left_table_privilege.grantee.clone()));

            match right_table_privilege {
                None => entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(String::from(left_table_privilege.table_name.clone()))], thing: TablePrivilege(left_table_privilege.privilege_type.clone(), left_table_privilege.grantor.clone(), left_table_privilege.grantee.clone()) }),
                Some(..) => _ = right_table_privileges_map.remove(&(left_table_privilege.table_name.clone(), left_table_privilege.privilege_type.clone(), left_table_privilege.grantor.clone(), left_table_privilege.grantee.clone())),
            }
        }

        if right_table_privileges_map.len() > 0 {
            let mut added_table_privileges : Vec<&db::thing::TablePrivilege> = right_table_privileges_map.values().collect();
            added_table_privileges.sort_unstable_by_key(|tp| (&tp.table_name, &tp.privilege_type, &tp.grantor, &tp.grantee));

            for right_table_privilege in added_table_privileges {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(right_table_privilege.table_name.clone())], thing: ColumnPrivilege(right_table_privilege.privilege_type.clone(), right_table_privilege.grantor.clone(), right_table_privilege.grantee.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_triggers(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_triggers = self.left_db.triggers(schema_name)?;
        let right_triggers = self.right_db.triggers(schema_name)?;

        let mut right_triggers_map : HashMap<(String, String), db::thing::Trigger> = right_triggers.into_iter().map(|t| ((t.trigger_name.clone(), t.event_manipulation.clone()), t)).collect();
        let mut entries = Vec::new();

        for left_trigger in left_triggers {
            let right_trigger = right_triggers_map.get(&(left_trigger.trigger_name.clone(), left_trigger.event_manipulation.clone()));

            match right_trigger {
                None => {
                    entries.push(Removal { path: vec![Schema(String::from(schema_name)), Table(left_trigger.event_object_table.clone())], thing: Trigger(left_trigger.trigger_name, left_trigger.event_manipulation) });
                },
                Some(rt) => {
                    let mut trigger_entries = self.compare_trigger(schema_name, &left_trigger, rt);
                    entries.append(&mut trigger_entries);

                    right_triggers_map.remove(&(left_trigger.trigger_name.clone(), left_trigger.event_manipulation.clone()));
                },
            }
        }

        if right_triggers_map.len() > 0 {
            let mut added_triggers : Vec<&db::thing::Trigger> = right_triggers_map.values().collect();
            added_triggers.sort_unstable_by_key(|t| (&t.event_object_table, &t.trigger_name, &t.event_manipulation));
            
            for right_trigger in added_triggers {
                entries.push(Addition { path: vec![Schema(String::from(schema_name)), Table(right_trigger.event_object_table.clone())], thing: Trigger(right_trigger.trigger_name.clone(), right_trigger.event_manipulation.clone()) });
            }
        }

        Ok(entries)
    }

    fn compare_views(&mut self, schema_name: &str) -> Result<Vec<ReportEntry>, Error> {
        let left_views = self.left_db.views(schema_name)?;
        let right_views = self.right_db.views(schema_name)?;

        let right_views_map: HashMap<String, db::thing::View> = right_views.into_iter().map(|t| (t.view_name.clone(), t)).collect();
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

        // added already detected by table comparison

        Ok(entries)
    }

    fn compare_column(&mut self, schema_name: &str, table_name: &str, left: &mut db::thing::Column, right: &mut db::thing::Column) -> Vec<ReportEntry> {
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

    fn compare_routine(&mut self, schema_name: &str, left: &db::thing::Routine, right: &db::thing::Routine) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Routine(left.signature.clone())];

        entries.push(self.compare_option_property(path(), "routine_type", &left.routine_type, &right.routine_type));
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

    fn compare_sequence(&mut self, schema_name: &str, left: &db::thing::Sequence, right: &db::thing::Sequence) -> Vec<ReportEntry> {
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

    fn compare_table(&mut self, schema_name: &str, left: &db::thing::Table, right: &db::thing::Table) -> Result<Vec<ReportEntry>, Error> {
        let mut entries = Vec::new();

        //TODO work out how to better clone the path
        let path = || vec![Schema(String::from(schema_name)), Table(left.table_name.clone())];

        entries.push(self.compare_property(path(), "table_type", &left.table_type, &right.table_type));
        entries.push(self.compare_property(path(), "is_insertable_into", &left.is_insertable_into, &right.is_insertable_into));

        Ok(entries)
    }

    fn compare_table_constraint(&mut self, schema_name: &str, left: &db::thing::TableConstraint, right: &db::thing::TableConstraint) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Table(left.table_name.clone()), TableConstraint(left.constraint_name.clone())];

        entries.push(self.compare_property(path(), "constraint_type", &left.constraint_type, &right.constraint_type));
        entries.push(self.compare_property(path(), "is_deferrable", &left.is_deferrable, &right.is_deferrable));
        entries.push(self.compare_property(path(), "initially_deferred", &left.initially_deferred, &right.initially_deferred));
        entries.push(self.compare_option_property(path(), "nulls_distinct", &left.nulls_distinct, &right.nulls_distinct));

        entries
    }

    fn compare_trigger(&mut self, schema_name: &str, left: &db::thing::Trigger, right: &db::thing::Trigger) -> Vec<ReportEntry> {
        let mut entries = Vec::new();

        //TODO work out how to better clone this
        let path = || vec![Schema(String::from(schema_name)), Table(left.event_object_table.clone()), Trigger(left.trigger_name.clone(), left.event_manipulation.clone())];

        entries.push(self.compare_property(path(), "event_object_schema", &left.event_object_schema, &right.event_object_schema));
        entries.push(self.compare_property(path(), "event_object_table", &left.event_object_table, &right.event_object_table));
        entries.push(self.compare_property(path(), "action_order", &left.action_order, &right.action_order));
        entries.push(self.compare_option_property(path(), "action_condition", &left.action_condition, &right.action_condition));
        entries.push(self.compare_property(path(), "action_statement", &left.action_statement, &right.action_statement));
        entries.push(self.compare_property(path(), "action_orientation", &left.action_orientation, &right.action_orientation));
        entries.push(self.compare_property(path(), "action_timing", &left.action_timing, &right.action_timing));
        entries.push(self.compare_option_property(path(), "action_reference_old_table", &left.action_reference_old_table, &right.action_reference_old_table));
        entries.push(self.compare_option_property(path(), "action_reference_new_table", &left.action_reference_new_table, &right.action_reference_new_table));

        entries
    }

    fn compare_view(&mut self, schema_name: &str, left: &db::thing::View, right: &db::thing::View) -> Vec<ReportEntry> {
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

    fn compare_property<T>(&mut self, mut path: Vec<Thing>, property_name: &str, left_value: T, right_value: T) -> ReportEntry
        where T: PartialEq, T: Display {
        path.push(Property(String::from(property_name)));

        if left_value == right_value {
            Match { path, left_value: left_value.to_string(), right_value: right_value.to_string() }
        } else {
            Change { path, left_value: left_value.to_string(), right_value: right_value.to_string() }
        }
    }

    fn compare_option_property<T>(&mut self, path: Vec<Thing>, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> ReportEntry
        where T: PartialEq, T: Display {
        self.compare_option_property_impl(path, property_name, left_value, right_value, &|l: &T, r: &T| l == r)
    }

    fn compare_option_property_ignore_whitespace<T>(&mut self, path: Vec<Thing>, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> ReportEntry
        where T: PartialEq, T: Display {
        self.compare_option_property_impl(path, property_name, left_value, right_value, &|l: &T, r: &T| l.to_string().as_str().eq_ignore_whitespace(r.to_string().as_str()))
    }

    fn compare_option_property_impl<T>(&mut self, mut path: Vec<Thing>, property_name: &str, left_value: &Option<T>, right_value: &Option<T>, compare: &dyn Fn(&T, &T) -> bool) -> ReportEntry
        where T: PartialEq, T: Display {
        path.push(Property(String::from(property_name)));
        
        if left_value.is_none() && right_value.is_none() {
            return Match { path, left_value: String::from("<none>"), right_value: String::from("<none>") };
        }
        
        if left_value.is_none() || right_value.is_none() {
            return Change { path, left_value: left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string()), right_value: right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string()) };
        }
        
        let left = left_value.as_ref().unwrap();
        let right = right_value.as_ref().unwrap();

        if compare(left, right) {
            Match { path, left_value: left.to_string(), right_value: right.to_string() }
        } else {
            Change { path, left_value: left.to_string(), right_value: right.to_string() }
        }
    }
}
