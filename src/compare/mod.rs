pub(crate) mod report;

use std::collections::HashMap;
use std::fmt::{Display};
use postgres::Error;
use SchemaComparison::{SchemaAdded, SchemaMissing};
use crate::compare::report::{PropertyComparison, SchemaComparison, TableColumnReport, TableConstraintReport, SchemaReport, PropertyReport, RoutineReport, PrivilegeReport, SequenceReport, TableReport, ViewReport, TableTriggerReport};
use crate::{db};
use crate::compare::report::PrivilegeComparison::{PrivilegeAdded, PrivilegeMaintained, PrivilegeRemoved};
use crate::compare::report::PropertyComparison::{PropertyChanged, PropertyUnchanged};
use crate::compare::report::RoutineComparison::{RoutineAdded, RoutineMaintained, RoutineRemoved};
use crate::compare::report::SchemaComparison::{SchemaMaintained, SchemaRemoved};
use crate::compare::report::SequenceComparison::{SequenceAdded, SequenceMaintained, SequenceRemoved};
use crate::compare::report::TableColumnComparison::{ColumnAdded, ColumnMaintained, ColumnRemoved};
use crate::compare::report::TableComparison::{TableAdded, TableMaintained, TableRemoved};
use crate::compare::report::TableConstraintComparison::{ConstraintAdded, ConstraintMaintained, ConstraintRemoved};
use crate::compare::report::TableTriggerComparison::{TriggerAdded, TriggerMaintained, TriggerRemoved};
use crate::compare::report::ViewComparison::ViewMaintained;
use crate::db::{Database};
use crate::db::thing::{Privilege, Schema};
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
        Comparer {
            left_db,
            right_db,
            ignore_whitespace,
            ignore_column_ordinal,
            ignore_privileges,
        }
    }

    pub fn compare(&mut self, schemas: Vec<String>) -> Result<SchemaReport, Error> {
        let left_schemas = self.left_db.schemas()?;
        let right_schemas = self.right_db.schemas()?;

        let mut entries = Vec::new();
        
        for schema in schemas {
            let left_schema = left_schemas.iter().find(|s| s.schema_name == schema);
            let right_schema = right_schemas.iter().find(|s| s.schema_name == schema);

            let entry: SchemaComparison;

            if left_schema.is_none() && right_schema.is_none() {
                entry = SchemaMissing { schema_name: String::from(schema) };
            } else {
                if left_schema.is_none() {
                    entry = SchemaAdded { schema_name: String::from(schema) };
                } else if right_schema.is_none() {
                    entry = SchemaRemoved { schema_name: String::from(schema) };
                } else {
                    let properties = self.compare_schema_properties(&left_schema.unwrap(), &right_schema.unwrap());

                    let routines = self.compare_routines(&schema)?;
                    let sequences = self.compare_sequences(&schema)?;
                    let tables = self.compare_tables(&schema)?;
                    let views = self.compare_views(&schema)?;

                    entry = SchemaMaintained { schema_name: String::from(schema), properties, routines, sequences, tables, views };
                }
            }
            
            entries.push(entry)
        }

        Ok(SchemaReport { entries })
    }

    fn compare_schema_properties(&self, left_schema: &Schema, right_schema: &Schema) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_property("schema_owner", &left_schema.schema_owner, &right_schema.schema_owner)
            ]
        }
    }

    fn compare_routines(&mut self, schema_name: &str) -> Result<RoutineReport, Error> {
        let left_routines = self.left_db.routines(schema_name)?;
        let right_routines = self.right_db.routines(schema_name)?;

        let mut right_routines_map: HashMap<String, db::thing::Routine> = right_routines.into_iter().map(|t| (t.signature.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_routine in left_routines {
            let right_routine = right_routines_map.get(&left_routine.signature);

            match right_routine {
                None => {
                    entries.push(RoutineRemoved { routine_signature: left_routine.signature });
                },
                Some(rr) => {
                    let properties = self.compare_routine_properties(&left_routine, rr);
                    let privileges = self.compare_routine_privileges(schema_name, &left_routine.signature)?;

                    entries.push(RoutineMaintained { routine_signature: left_routine.signature, properties, privileges });

                    right_routines_map.remove(&rr.signature.clone());
                }
            }
        }

        if right_routines_map.len() > 0 {
            let mut added_routines: Vec<&db::thing::Routine> = right_routines_map.values().collect();
            added_routines.sort_unstable_by_key(|r| &r.signature);

            for right_routine in added_routines {
                entries.push(RoutineAdded { routine_signature: right_routine.signature.clone() });
            }
        }

        Ok(RoutineReport { entries })
    }

    fn compare_routine_properties(&self, left: &db::thing::Routine, right: &db::thing::Routine) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_option_property("routine_type", &left.routine_type, &right.routine_type),
                self.compare_option_property("data_type", &left.data_type, &right.data_type),
                self.compare_option_property("type_udt_name", &left.type_udt_name, &right.type_udt_name),
                self.compare_property("routine_body", &left.routine_body, &right.routine_body),
                if self.ignore_whitespace {
                    self.compare_option_property_ignore_whitespace("routine_definition", &left.routine_definition, &right.routine_definition)
                } else {
                    self.compare_option_property("routine_definition", &left.routine_definition, &right.routine_definition)
                },
                self.compare_option_property("external_name", &left.external_name, &right.external_name),
                self.compare_property("external_language", &left.external_language, &right.external_language),
                self.compare_property("is_deterministic", &left.is_deterministic, &right.is_deterministic),
                self.compare_option_property("is_null_call", &left.is_null_call, &right.is_null_call),
                self.compare_property("security_type", &left.security_type, &right.security_type),
            ]
        }
    }

    fn compare_routine_privileges(&mut self, schema_name: &str, routine_signature: &str) -> Result<PrivilegeReport, Error> {
        if self.ignore_privileges {
            return Ok(PrivilegeReport { entries: vec![] })
        }

        let left_routine_privileges = self.left_db.routine_privileges(schema_name, routine_signature)?;
        let right_routine_privileges = self.right_db.routine_privileges(schema_name, routine_signature)?;

        Ok(self.compare_privileges(left_routine_privileges, right_routine_privileges))
    }

    fn compare_privileges(&self, left_privileges: Vec<Privilege>, right_privileges: Vec<Privilege>) -> PrivilegeReport {
        let mut right_privileges_map: HashMap<(String, String, String), Privilege> = right_privileges.into_iter().map(|c| ((c.privilege_type.clone(), c.grantor.clone(), c.grantee.clone()), c)).collect();
        let mut entries = Vec::new();

        for left_privilege in left_privileges {
            let key = &(left_privilege.privilege_type.clone(), left_privilege.grantor.clone(), left_privilege.grantee.clone());
            let right_privilege = right_privileges_map.get_mut(&key);

            match right_privilege {
                None => entries.push(PrivilegeRemoved { privilege_name: left_privilege.privilege_type.clone(), grantor: left_privilege.grantor.clone(), grantee: left_privilege.grantee.clone() }),
                Some(..) => {
                    entries.push(PrivilegeMaintained { privilege_name: left_privilege.privilege_type.clone(), grantor: left_privilege.grantor.clone(), grantee: left_privilege.grantee.clone() });

                    _ = right_privileges_map.remove(&key);
                },
            }
        }

        if right_privileges_map.len() > 0 {
            let mut added_privileges: Vec<&Privilege> = right_privileges_map.values().collect();
            added_privileges.sort_unstable_by_key(|rp| (&rp.privilege_type, &rp.grantor, &rp.grantee));

            for added_privilege in added_privileges {
                entries.push(PrivilegeAdded { privilege_name: added_privilege.privilege_type.clone(), grantor: added_privilege.grantor.clone(), grantee: added_privilege.grantee.clone() });
            }
        }

        PrivilegeReport { entries }
    }

    fn compare_sequences(&mut self, schema_name: &str) -> Result<SequenceReport, Error> {
        let left_sequences = self.left_db.sequences(schema_name)?;
        let right_sequences = self.right_db.sequences(schema_name)?;

        let mut right_sequences_map: HashMap<String, db::thing::Sequence> = right_sequences.into_iter().map(|t| (t.sequence_name.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_sequence in left_sequences {
            let right_sequence = right_sequences_map.get(&left_sequence.sequence_name);

            match right_sequence {
                None => {
                    entries.push(SequenceRemoved { sequence_name: left_sequence.sequence_name });
                },
                Some(rs) => {
                    let properties = self.compare_sequence_properties(&left_sequence, rs);

                    entries.push(SequenceMaintained { sequence_name: left_sequence.sequence_name, properties });

                    right_sequences_map.remove(&rs.sequence_name.clone());
                }
            }
        }

        if right_sequences_map.len() > 0 {
            let mut added_sequences: Vec<&db::thing::Sequence> = right_sequences_map.values().collect();
            added_sequences.sort_unstable_by_key(|s| &s.sequence_name);

            for right_sequence in added_sequences {
                entries.push(SequenceAdded { sequence_name: right_sequence.sequence_name.clone() });
            }
        }

        Ok(SequenceReport { entries })
    }

    fn compare_sequence_properties(&self, left: &db::thing::Sequence, right: &db::thing::Sequence) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_property("data_type", &left.data_type, &right.data_type),
                self.compare_property("numeric_precision", &left.numeric_precision, &right.numeric_precision),
                self.compare_property("numeric_precision_radix", &left.numeric_precision_radix, &right.numeric_precision_radix),
                self.compare_property("numeric_scale", &left.numeric_scale, &right.numeric_scale),
                self.compare_property("start_value", &left.start_value, &right.start_value),
                self.compare_property("minimum_value", &left.minimum_value, &right.minimum_value),
                self.compare_property("maximum_value", &left.maximum_value, &right.maximum_value),
                self.compare_property("increment", &left.increment, &right.increment),
                self.compare_property("cycle_option", &left.cycle_option, &right.cycle_option),
            ]
        }
    }

    fn compare_tables(&mut self, schema_name: &str) -> Result<TableReport, Error> {
        let left_tables = self.left_db.tables(schema_name)?;
        let right_tables = self.right_db.tables(schema_name)?;

        let mut right_tables_map: HashMap<String, db::thing::Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();
        let mut entries = Vec::new();

        for left_table in left_tables {
            let right_table = right_tables_map.get(&left_table.table_name);

            match right_table {
                None => {
                    entries.push(TableRemoved { table_name: left_table.table_name });
                },
                Some(rt) => {
                    let properties = self.compare_table_properties(&left_table, rt);
                    let columns = self.compare_table_columns(schema_name, &rt.table_name)?;
                    let privileges = self.compare_table_privileges(schema_name, &rt.table_name)?;
                    let constraints = self.compare_table_constraints(schema_name, &rt.table_name)?;
                    let triggers = self.compare_table_triggers(schema_name, &rt.table_name)?;

                    entries.push(TableMaintained { table_name: left_table.table_name, properties, columns, privileges, constraints, triggers });

                    right_tables_map.remove(&rt.table_name.clone());
                },
            }
        }

        if right_tables_map.len() > 0 {
            let mut added_tables: Vec<&db::thing::Table> = right_tables_map.values().collect();
            added_tables.sort_unstable_by_key(|t| &t.table_name);

            for right_table in added_tables {
                entries.push(TableAdded { table_name: right_table.table_name.clone() });
            }
        }

        Ok(TableReport { entries })
    }

    fn compare_table_properties(&self, left: &db::thing::Table, right: &db::thing::Table) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_property("table_type", &left.table_type, &right.table_type),
                self.compare_property("is_insertable_into", &left.is_insertable_into, &right.is_insertable_into),
            ]
        }
    }

    fn compare_table_privileges(&mut self, schema_name: &str, table_name: &str) -> Result<PrivilegeReport, Error> {
        if self.ignore_privileges {
            return Ok(PrivilegeReport { entries: vec![] })
        }

        let left_table_privileges = self.left_db.table_privileges(schema_name, table_name)?;
        let right_table_privileges = self.right_db.table_privileges(schema_name, table_name)?;
    
        Ok(self.compare_privileges(left_table_privileges, right_table_privileges))
    }

    fn compare_table_columns(&mut self, schema_name: &str, table_name: &str) -> Result<TableColumnReport, Error> {
        let left_columns = self.left_db.columns(schema_name, table_name)?;
        let right_columns = self.right_db.columns(schema_name, table_name)?;
    
        let mut right_columns_map : HashMap<String, db::thing::Column> = right_columns.into_iter().map(|c| (c.column_name.clone(), c)).collect();
        let mut entries = Vec::new();
    
        for left_column in left_columns {
            let key = left_column.column_name.clone();
            let right_column = right_columns_map.get_mut(&key);
    
            match right_column {
                None => {
                    entries.push(ColumnRemoved { column_name: left_column.column_name.clone() });
                },
                Some(rc) => {
                    let properties = self.compare_table_column_properties(&left_column, rc);
                    let privileges = self.compare_table_column_privileges(schema_name, table_name, &rc.column_name)?;

                    entries.push(ColumnMaintained { column_name: rc.column_name.clone(), properties, privileges });
                    
                    right_columns_map.remove(&key);
                },
            }
        }
    
        if right_columns_map.len() > 0 {
            let mut added_columns : Vec<&db::thing::Column> = right_columns_map.values().collect();
            added_columns.sort_unstable_by_key(|c| &c.column_name);
            
            for right_column in added_columns {
                entries.push(ColumnAdded { column_name: right_column.column_name.clone() });
            }
        }
    
        Ok(TableColumnReport { entries })
    }

    fn compare_table_column_properties(&self, left: &db::thing::Column, right: &db::thing::Column) -> PropertyReport {
        let mut properties = vec![
            self.compare_option_property("column_default", &left.column_default, &right.column_default),
            self.compare_property("is_nullable", &left.is_nullable, &right.is_nullable),
            self.compare_property("data_type", &left.data_type, &right.data_type),
            self.compare_option_property("character_maximum_length", &left.character_maximum_length, &right.character_maximum_length),
            self.compare_option_property("numeric_precision", &left.numeric_precision, &right.numeric_precision),
            self.compare_option_property("numeric_scale", &left.numeric_scale, &right.numeric_scale),
            self.compare_option_property("datetime_precision", &left.datetime_precision, &right.datetime_precision),
            self.compare_property("is_identity", &left.is_identity, &right.is_identity),
            self.compare_option_property("identity_generation", &left.identity_generation, &right.identity_generation),
            self.compare_property("is_generated", &left.is_generated, &right.is_generated),
            self.compare_option_property("generation_expression", &left.generation_expression, &right.generation_expression),
            self.compare_property("is_updatable", &left.is_updatable, &right.is_updatable),
        ];
    
        if !self.ignore_column_ordinal {
            properties.push(self.compare_property("ordinal_position", &left.ordinal_position, &right.ordinal_position));
        }
        
        PropertyReport { entries: properties }
    }
    
    fn compare_table_column_privileges(&mut self, schema_name: &str, table_name: &str, column_name: &str) -> Result<PrivilegeReport, Error> {
        if self.ignore_privileges {
            return Ok(PrivilegeReport { entries: vec![] })
        }

        let left_column_privileges = self.left_db.column_privileges(schema_name, table_name, column_name)?;
        let right_column_privileges = self.right_db.column_privileges(schema_name, table_name, column_name)?;

        Ok(self.compare_privileges(left_column_privileges, right_column_privileges))
    }
    
    fn compare_table_constraints(&mut self, schema_name: &str, table_name: &str) -> Result<TableConstraintReport, Error> {
        let left_table_constraints = self.left_db.table_constraints(schema_name, table_name)?;
        let right_table_constraints = self.right_db.table_constraints(schema_name, table_name)?;
    
        let mut right_table_constraints_map : HashMap<String, db::thing::TableConstraint> = right_table_constraints.into_iter().map(|t| (t.constraint_name.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_table_constraint in left_table_constraints {
            let key = left_table_constraint.constraint_name.clone();
            let right_table_constraint = right_table_constraints_map.get(&key);
    
            match right_table_constraint {
                None => {
                    entries.push(ConstraintRemoved { constraint_name: left_table_constraint.constraint_name });
                },
                Some(rtc) => {
                    let properties = self.compare_table_constraint(&left_table_constraint, rtc);
                    
                    entries.push(ConstraintMaintained { constraint_name: rtc.constraint_name.clone(), properties });
    
                    right_table_constraints_map.remove(&key);
                },
            }
        }
    
        if right_table_constraints_map.len() > 0 {
            let mut added_table_constraints : Vec<&db::thing::TableConstraint> = right_table_constraints_map.values().collect();
            added_table_constraints.sort_unstable_by_key(|tc| &tc.constraint_name);
            
            for right_table_constraint in added_table_constraints {
                entries.push(ConstraintAdded { constraint_name: right_table_constraint.constraint_name.clone() });
            }
        }
    
        Ok(TableConstraintReport { entries })
    }
    
    fn compare_table_constraint(&mut self, left: &db::thing::TableConstraint, right: &db::thing::TableConstraint) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_property("constraint_type", &left.constraint_type, &right.constraint_type),
                self.compare_property("is_deferrable", &left.is_deferrable, &right.is_deferrable),
                self.compare_property("initially_deferred", &left.initially_deferred, &right.initially_deferred),
                self.compare_option_property("nulls_distinct", &left.nulls_distinct, &right.nulls_distinct),
            ]
        }
    }

    fn compare_table_triggers(&mut self, schema_name: &str, table_name: &str) -> Result<TableTriggerReport, Error> {
        let left_triggers = self.left_db.triggers(schema_name, table_name)?;
        let right_triggers = self.right_db.triggers(schema_name, table_name)?;
    
        let mut right_triggers_map : HashMap<(String, String), db::thing::Trigger> = right_triggers.into_iter().map(|t| ((t.trigger_name.clone(), t.event_manipulation.clone()), t)).collect();
        let mut entries = Vec::new();
    
        for left_trigger in left_triggers {
            let key = &(left_trigger.trigger_name.clone(), left_trigger.event_manipulation.clone());
            let right_trigger = right_triggers_map.get(&key);
    
            match right_trigger {
                None => {
                    entries.push(TriggerRemoved { trigger_name: left_trigger.trigger_name, event_manipulation: left_trigger.event_manipulation });
                },
                Some(rt) => {
                    let properties = self.compare_trigger_properties(&left_trigger, rt);
                    
                    entries.push(TriggerMaintained { trigger_name: rt.trigger_name.clone(), event_manipulation: rt.event_manipulation.clone(), properties });
    
                    right_triggers_map.remove(key);
                },
            }
        }
    
        if right_triggers_map.len() > 0 {
            let mut added_triggers : Vec<&db::thing::Trigger> = right_triggers_map.values().collect();
            added_triggers.sort_unstable_by_key(|t| (&t.trigger_name, &t.event_manipulation));
            
            for right_trigger in added_triggers {
                entries.push(TriggerAdded { trigger_name: right_trigger.trigger_name.clone(), event_manipulation: right_trigger.event_manipulation.clone() });
            }
        }
    
        Ok(TableTriggerReport { entries })
    }
    
    fn compare_trigger_properties(&mut self, left: &db::thing::Trigger, right: &db::thing::Trigger) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_property("action_order", &left.action_order, &right.action_order),
                self.compare_option_property("action_condition", &left.action_condition, &right.action_condition),
                self.compare_property("action_statement", &left.action_statement, &right.action_statement),
                self.compare_property("action_orientation", &left.action_orientation, &right.action_orientation),
                self.compare_property("action_timing", &left.action_timing, &right.action_timing),
                self.compare_option_property("action_reference_old_table", &left.action_reference_old_table, &right.action_reference_old_table),
                self.compare_option_property("action_reference_new_table", &left.action_reference_new_table, &right.action_reference_new_table),
            ]
        }
    }

    fn compare_views(&mut self, schema_name: &str) -> Result<ViewReport, Error> {
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
                    let properties = self.compare_view_properties(&left_view, rv);
                    
                    entries.push(ViewMaintained { view_name: rv.view_name.clone(), properties });
                }
            }
        }
    
        // added already detected by table comparison
    
        Ok(ViewReport { entries })
    }
    
    fn compare_view_properties(&mut self, left: &db::thing::View, right: &db::thing::View) -> PropertyReport {
        PropertyReport {
            entries: vec![
                self.compare_option_property("view_definition", &left.view_definition, &right.view_definition),
                self.compare_property("check_option", &left.check_option, &right.check_option),
                self.compare_property("is_updatable", &left.is_updatable, &right.is_updatable),
                self.compare_property("is_insertable_into", &left.is_insertable_into, &right.is_insertable_into),
                self.compare_property("is_trigger_updatable", &left.is_trigger_updatable, &right.is_trigger_updatable),
                self.compare_property("is_trigger_deletable", &left.is_trigger_deletable, &right.is_trigger_deletable),
                self.compare_property("is_trigger_insertable_into", &left.is_trigger_insertable_into, &right.is_trigger_insertable_into),
            ]
        }
    }

    fn compare_property<T>(&self, property_name: &str, left_value: T, right_value: T) -> PropertyComparison
        where T: PartialEq, T: Display {
        if left_value == right_value {
            PropertyUnchanged { property_name: String::from(property_name), value: left_value.to_string() }
        } else {
            PropertyChanged { property_name: String::from(property_name), left_value: left_value.to_string(), right_value: right_value.to_string() }
        }
    }

    fn compare_option_property<T>(&self, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> PropertyComparison
        where T: PartialEq, T: Display {
        self.compare_option_property_impl(property_name, left_value, right_value, &|l: &T, r: &T| l == r)
    }

    fn compare_option_property_ignore_whitespace<T>(&self, property_name: &str, left_value: &Option<T>, right_value: &Option<T>) -> PropertyComparison
        where T: PartialEq, T: Display {
        self.compare_option_property_impl(property_name, left_value, right_value, &|l: &T, r: &T| l.to_string().as_str().eq_ignore_whitespace(r.to_string().as_str()))
    }

    fn compare_option_property_impl<T>(&self, property_name: &str, left_value: &Option<T>, right_value: &Option<T>, compare: &dyn Fn(&T, &T) -> bool) -> PropertyComparison
        where T: PartialEq, T: Display {
        if left_value.is_none() && right_value.is_none() {
            return PropertyUnchanged { property_name: String::from(property_name), value: String::from("<none>") }
        }
        
        if left_value.is_none() || right_value.is_none() {
            return PropertyChanged { property_name: String::from(property_name), left_value: left_value.as_ref().map_or(String::from("<none>"), |v| v.to_string()), right_value: right_value.as_ref().map_or(String::from("<none>"), |v| v.to_string()) };
        }
        
        let left = left_value.as_ref().unwrap();
        let right = right_value.as_ref().unwrap();
    
        if compare(left, right) {
            PropertyUnchanged { property_name: String::from(property_name), value: left.to_string() }
        } else {
            PropertyChanged { property_name: String::from(property_name), left_value: left.to_string(), right_value: right.to_string() }
        }
    }
}
