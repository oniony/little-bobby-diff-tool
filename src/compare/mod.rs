use std::collections::HashMap;
use std::fmt::Display;
use itertools::Itertools;
use sqlx::Error;

use crate::compare::report::index::IndexComparison;
use crate::compare::report::privilege::PrivilegeComparison;
use crate::compare::report::privilege::PrivilegeComparison::{PrivilegeAdded, PrivilegeMaintained, PrivilegeRemoved};
use crate::compare::report::property::PropertyComparison;
use crate::compare::report::property::PropertyComparison::{PropertyChanged, PropertyUnchanged};
use crate::compare::report::Report;
use crate::compare::report::schema::SchemaComparison;
use crate::compare::report::schema::SchemaComparison::{SchemaAdded, SchemaMaintained, SchemaMissing, SchemaRemoved};
use crate::compare::report::sequence::SequenceComparison;
use crate::compare::report::sequence::SequenceComparison::{SequenceAdded, SequenceMaintained, SequenceRemoved};
use crate::compare::report::table::TableComparison;
use crate::compare::report::table::TableComparison::{TableAdded, TableMaintained, TableRemoved};
use crate::compare::report::column::ColumnComparison;
use crate::compare::report::column::ColumnComparison::{ColumnAdded, ColumnMaintained, ColumnRemoved};
use crate::compare::report::index::IndexComparison::{IndexAdded, IndexMaintained, IndexRemoved};
use crate::compare::report::routine::RoutineComparison;
use crate::compare::report::routine::RoutineComparison::{RoutineAdded, RoutineMaintained, RoutineRemoved};
use crate::compare::report::table_constraint::TableConstraintComparison;
use crate::compare::report::table_constraint::TableConstraintComparison::{ConstraintAdded, ConstraintMaintained, ConstraintRemoved};
use crate::compare::report::table_trigger::TableTriggerComparison;
use crate::compare::report::table_trigger::TableTriggerComparison::{TriggerAdded, TriggerMaintained, TriggerRemoved};
use crate::compare::report::view::ViewComparison;
use crate::compare::report::view::ViewComparison::ViewMaintained;
use crate::db::Database;
use crate::db::schema::Schema;
use crate::db::table::Table;
use crate::db::column::Column;
use crate::db::column_privilege::ColumnPrivilege;
use crate::db::index::Index;
use crate::db::privilege::Privilege;
use crate::db::routine::Routine;
use crate::db::routine_privilege::RoutinePrivilege;
use crate::db::table_constraint::TableConstraint;
use crate::db::table_privilege::TablePrivilege;
use crate::db::table_trigger::TableTrigger;
use crate::db::sequence::Sequence;
use crate::db::view::View;
use crate::string::EqualIgnoreWhitespace;

pub mod report;

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

    pub async fn compare(&mut self, schemas: Vec<String>) -> Result<Report<SchemaComparison>, Error> {
        let mut entries = Vec::new();

        let left_columns = self.left_db.columns(&schemas[..]).await?;
        let right_columns = self.right_db.columns(&schemas[..]).await?;
        let left_column_privileges = self.left_db.column_privileges(&schemas[..]).await?;
        let right_column_privileges = self.right_db.column_privileges(&schemas[..]).await?;
        let left_routines = self.left_db.routines(&schemas[..]).await?;
        let right_routines = self.right_db.routines(&schemas[..]).await?;
        let left_routine_privileges = self.left_db.routine_privileges(&schemas[..]).await?;
        let right_routine_privileges = self.right_db.routine_privileges(&schemas[..]).await?;
        let left_schemas = self.left_db.schemas(&schemas[..]).await?;
        let right_schemas = self.right_db.schemas(&schemas[..]).await?;
        let left_sequences = self.left_db.sequences(&schemas[..]).await?;
        let right_sequences = self.right_db.sequences(&schemas[..]).await?;
        let left_tables = self.left_db.tables(&schemas[..]).await?;
        let right_tables = self.right_db.tables(&schemas[..]).await?;
        let left_table_constraints = self.left_db.table_constraints(&schemas[..]).await?;
        let right_table_constraints = self.right_db.table_constraints(&schemas[..]).await?;
        let left_indices = self.left_db.indices(&schemas[..]).await?;
        let right_indices = self.right_db.indices(&schemas[..]).await?;
        let left_table_privileges = self.left_db.table_privileges(&schemas[..]).await?;
        let right_table_privileges = self.right_db.table_privileges(&schemas[..]).await?;
        let left_table_triggers = self.left_db.table_triggers(&schemas[..]).await?;
        let right_table_triggers = self.right_db.table_triggers(&schemas[..]).await?;
        let left_views = self.left_db.views(&schemas[..]).await?;
        let right_views = self.right_db.views(&schemas[..]).await?;
        
        for schema in schemas {
            let left_schema = left_schemas.iter().filter(|s| s.schema_name == schema).at_most_one().map_err(|_| Error::RowNotFound )?;
            let right_schema = right_schemas.iter().filter(|s| s.schema_name == schema).at_most_one().map_err(|_| Error::RowNotFound)?;

            if left_schema.is_none() && right_schema.is_none() {
                entries.push(SchemaMissing { schema_name: String::from(schema) });
            } else {
                if left_schema.is_none() {
                    entries.push(SchemaAdded { schema_name: String::from(schema) });
                } else if right_schema.is_none() {
                    entries.push(SchemaRemoved { schema_name: String::from(schema) });
                } else {
                    let properties = self.compare_schema_properties(&left_schema.unwrap(), &right_schema.unwrap());

                    let left_schema_routines = left_routines.iter().filter(|r| r.routine_schema == schema).collect();
                    let right_schema_routines = right_routines.iter().filter(|r| r.routine_schema == schema).collect();
                    let left_schema_routine_privileges = left_routine_privileges.iter().filter(|p| p.routine_schema == schema).collect();
                    let right_schema_routine_privileges = right_routine_privileges.iter().filter(|p| p.routine_schema == schema).collect();
                    let routines = self.compare_routines(left_schema_routines, right_schema_routines, left_schema_routine_privileges, right_schema_routine_privileges)?;

                    let left_schema_sequences = left_sequences.iter().filter(|s| s.sequence_schema == schema).collect();
                    let right_schema_sequences = right_sequences.iter().filter(|s| s.sequence_schema == schema).collect();
                    let sequences = self.compare_sequences(left_schema_sequences, right_schema_sequences)?;

                    let left_schema_tables = left_tables.iter().filter(|t| t.table_schema == schema).collect();
                    let right_schema_tables = right_tables.iter().filter(|t| t.table_schema == schema).collect();
                    let left_schema_columns = left_columns.iter().filter(|c| c.table_schema == schema).collect();
                    let right_schema_columns = right_columns.iter().filter(|c| c.table_schema == schema).collect();
                    let left_schema_indices = left_indices.iter().filter(|i| i.table_schema == schema).collect();
                    let right_schema_indices = right_indices.iter().filter(|i| i.table_schema == schema).collect();
                    let left_schema_table_privileges = left_table_privileges.iter().filter(|p| p.table_schema == schema).collect();
                    let right_schema_table_privileges = right_table_privileges.iter().filter(|p| p.table_schema == schema).collect();
                    let left_schema_table_constraints = left_table_constraints.iter().filter(|c| c.table_schema == schema).collect();
                    let right_schema_table_constraints = right_table_constraints.iter().filter(|c| c.table_schema == schema).collect();
                    let left_schema_table_triggers = left_table_triggers.iter().filter(|t| t.event_object_schema == schema).collect();
                    let right_schema_table_triggers = right_table_triggers.iter().filter(|t| t.event_object_schema == schema).collect();
                    let left_schema_column_privileges = left_column_privileges.iter().filter(|p| p.table_schema == schema).collect();
                    let right_schema_column_privileges = right_column_privileges.iter().filter(|p| p.table_schema == schema).collect();
                    let tables = self.compare_tables(
                        left_schema_tables,
                        right_schema_tables,
                        left_schema_columns,
                        right_schema_columns,
                        left_schema_column_privileges,
                        right_schema_column_privileges,
                        left_schema_indices,
                        right_schema_indices,
                        left_schema_table_privileges,
                        right_schema_table_privileges,
                        left_schema_table_constraints,
                        right_schema_table_constraints,
                        left_schema_table_triggers,
                        right_schema_table_triggers,
                    )?;

                    let left_schema_views = left_views.iter().filter(|t| t.table_schema == schema).collect();
                    let right_schema_views = right_views.iter().filter(|t| t.table_schema == schema).collect();
                    let views = self.compare_views(left_schema_views, right_schema_views)?;

                    entries.push(SchemaMaintained { schema_name: String::from(schema), properties, routines, sequences, tables, views });
                }
            }
        }

        Ok(Report { entries })
    }

    fn compare_schema_properties(&self, left: &Schema, right: &Schema) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("schema_owner", left, right, |s| &s.schema_owner),
                self.compare_option_property("default_character_set_catalog", left, right, |s| &s.default_character_set_catalog),
                self.compare_option_property("default_character_set_schema", left, right, |s| &s.default_character_set_schema),
                self.compare_option_property("default_character_set_name", left, right, |s| &s.default_character_set_name),
            ]
        }
    }

    fn compare_routines(&mut self,
                        left_routines: Vec<&Routine>,
                        right_routines: Vec<&Routine>,
                        left_routine_privileges: Vec<&RoutinePrivilege>,
                        right_routine_privileges: Vec<&RoutinePrivilege>,
    ) -> Result<Report<RoutineComparison>, Error> {
        let mut right_routines_map: HashMap<String, &Routine> = right_routines.into_iter().map(|t| (t.signature.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_routine in left_routines {
            let right_routine = right_routines_map.get(&left_routine.signature);
    
            match right_routine {
                None => {
                    entries.push(RoutineRemoved { routine_signature: left_routine.signature.clone() });
                },
                Some(rr) => {
                    let properties = self.compare_routine_properties(&left_routine, rr);
                    
                    let left_routine_routine_privileges: Vec<&RoutinePrivilege> = left_routine_privileges.iter().filter(|p| p.signature == left_routine.signature).cloned().collect();
                    let right_routine_routine_privileges: Vec<&RoutinePrivilege> = right_routine_privileges.iter().filter(|p| p.signature == rr.signature).cloned().collect();
                    let privileges = self.compare_routine_privileges(left_routine_routine_privileges, right_routine_routine_privileges)?;
    
                    entries.push(RoutineMaintained { routine_signature: left_routine.signature.clone(), properties, privileges });
    
                    right_routines_map.remove(&rr.signature.clone());
                }
            }
        }
    
        if right_routines_map.len() > 0 {
            let mut added_routines: Vec<&&Routine> = right_routines_map.values().collect();
            added_routines.sort_unstable_by_key(|r| &r.signature);
    
            for right_routine in added_routines {
                entries.push(RoutineAdded { routine_signature: right_routine.signature.clone() });
            }
        }
    
        Ok(Report { entries })
    }

    fn compare_routine_properties(&self, left: &Routine, right: &Routine) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_option_property("routine_type", left, right, |p| &p.routine_type),
                self.compare_option_property("module_catalog", left, right, |p| &p.module_catalog),
                self.compare_option_property("module_schema", left, right, |p| &p.module_schema),
                self.compare_option_property("module_name", left, right, |p| &p.module_name),
                self.compare_option_property("udt_catalog", left, right, |p| &p.udt_catalog),
                self.compare_option_property("udt_schema", left, right, |p| &p.udt_schema),
                self.compare_option_property("udt_name", left, right, |p| &p.udt_name),
                self.compare_option_property("data_type", left, right, |p| &p.data_type),
                self.compare_option_property("character_maximum_length", left, right, |p| &p.character_maximum_length),
                self.compare_option_property("character_octet_length", left, right, |p| &p.character_octet_length),
                self.compare_option_property("character_set_catalog", left, right, |p| &p.character_set_catalog),
                self.compare_option_property("character_set_schema", left, right, |p| &p.character_set_schema),
                self.compare_option_property("character_set_name", left, right, |p| &p.character_set_name),
                self.compare_option_property("collation_catalog", left, right, |p| &p.collation_catalog),
                self.compare_option_property("collation_schema", left, right, |p| &p.collation_schema),
                self.compare_option_property("collation_name", left, right, |p| &p.collation_name),
                self.compare_option_property("numeric_precision", left, right, |p| &p.numeric_precision),
                self.compare_option_property("numeric_precision_radix", left, right, |p| &p.numeric_precision_radix),
                self.compare_option_property("numeric_scale", left, right, |p| &p.numeric_scale),
                self.compare_option_property("datetime_precision", left, right, |p| &p.datetime_precision),
                self.compare_option_property("interval_type", left, right, |p| &p.interval_type),
                self.compare_option_property("interval_precision", left, right, |p| &p.interval_precision),
                self.compare_option_property("type_udt_catalog", left, right, |p| &p.type_udt_catalog),
                self.compare_option_property("type_udt_schema", left, right, |p| &p.type_udt_schema),
                self.compare_option_property("type_udt_name", left, right, |p| &p.type_udt_name),
                self.compare_option_property("maximum_cardinality", left, right, |p| &p.maximum_cardinality),
                self.compare_option_property("dtd_identifier", left, right, |p| &p.dtd_identifier),
                self.compare_property("routine_body", left, right, |p| &p.routine_body),
                if self.ignore_whitespace {
                    self.compare_option_property_ignore_whitespace("routine_definition", left, right, |p| &p.routine_definition)
                } else {
                    self.compare_option_property("routine_definition", left, right, |p| &p.routine_definition)
                },
                self.compare_option_property("external_name", left, right, |p| &p.external_name),
                self.compare_property("external_language", left, right, |p| &p.external_language),
                self.compare_property("parameter_style", left, right, |p| &p.parameter_style),
                self.compare_property("is_deterministic", left, right, |p| &p.is_deterministic),
                self.compare_property("sql_data_access", left, right, |p| &p.sql_data_access),
                self.compare_option_property("is_null_call", left, right, |p| &p.is_null_call),
                self.compare_option_property("sql_path", left, right, |p| &p.sql_path),
                self.compare_property("schema_level_routine", left, right, |p| &p.schema_level_routine),
                self.compare_option_property("max_dynamic_result_sets", left, right, |p| &p.max_dynamic_result_sets),
                self.compare_option_property("is_user_defined_cast", left, right, |p| &p.is_user_defined_cast),
                self.compare_option_property("is_implicitly_invocable", left, right, |p| &p.is_implicitly_invocable),
                self.compare_property("security_type", left, right, |p| &p.security_type),
            ]
        }
    }

    fn compare_routine_privileges(&mut self, left_routine_privileges: Vec<&RoutinePrivilege>, right_routine_privileges: Vec<&RoutinePrivilege>) -> Result<Report<PrivilegeComparison>, Error> {
        if self.ignore_privileges {
            return Ok(Report { entries: vec![] })
        }
    
        Ok(self.compare_privileges(&left_routine_privileges, &right_routine_privileges))
    }

    fn compare_privileges<P>(&self, left_privileges: &Vec<P>, right_privileges: &Vec<P>) -> Report<PrivilegeComparison> 
        where P : Privilege
    {
        let mut right_privileges_map: HashMap<(&str, &str, &str), &P> = right_privileges.into_iter().map(|c| ((c.privilege_type(), c.grantor(), c.grantee()), c)).collect();
        let mut entries = Vec::new();
    
        for left_privilege in left_privileges {
            let key = &(left_privilege.privilege_type(), left_privilege.grantor(), left_privilege.grantee());
            let right_privilege = right_privileges_map.get(&key);
    
            match right_privilege {
                None => entries.push(PrivilegeRemoved { privilege_name: left_privilege.privilege_type().to_string(), grantor: left_privilege.grantor().to_string(), grantee: left_privilege.grantee().to_string() }),
                Some(..) => {
                    entries.push(PrivilegeMaintained { privilege_name: left_privilege.privilege_type().to_string(), grantor: left_privilege.grantor().to_string(), grantee: left_privilege.grantee().to_string() });
    
                    _ = right_privileges_map.remove(&key);
                },
            }
        }
    
        if right_privileges_map.len() > 0 {
            let mut added_privileges: Vec<&&P> = right_privileges_map.values().collect();
            added_privileges.sort_unstable_by_key(|p| (p.privilege_type(), p.grantor(), p.grantee()));
    
            for added_privilege in added_privileges {
                entries.push(PrivilegeAdded { privilege_name: added_privilege.privilege_type().to_string(), grantor: added_privilege.grantor().to_string(), grantee: added_privilege.grantee().to_string() });
            }
        }
    
        Report { entries }
    }

    fn compare_sequences(&mut self, left_sequences: Vec<&Sequence>, right_sequences: Vec<&Sequence>) -> Result<Report<SequenceComparison>, Error> {
        let mut right_sequences_map: HashMap<String, &Sequence> = right_sequences.into_iter().map(|t| (t.sequence_name.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_sequence in left_sequences {
            let key = &left_sequence.sequence_name;
            let right_sequence = right_sequences_map.get(key);
    
            match right_sequence {
                None => {
                    entries.push(SequenceRemoved { sequence_name: left_sequence.sequence_name.clone() });
                },
                Some(rs) => {
                    let properties = self.compare_sequence_properties(&left_sequence, rs);
    
                    entries.push(SequenceMaintained { sequence_name: left_sequence.sequence_name.clone(), properties });
    
                    right_sequences_map.remove(key);
                }
            }
        }
    
        if right_sequences_map.len() > 0 {
            let mut added_sequences: Vec<&&Sequence> = right_sequences_map.values().collect();
            added_sequences.sort_unstable_by_key(|s| &s.sequence_name);
    
            for right_sequence in added_sequences {
                entries.push(SequenceAdded { sequence_name: right_sequence.sequence_name.clone() });
            }
        }
    
        Ok(Report { entries })
    }

    fn compare_sequence_properties(&self, left: &Sequence, right: &Sequence) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("sequence_catalog", left, right, |p| &p.sequence_catalog),
                self.compare_property("sequence_schema", left, right, |p| &p.sequence_schema),
                self.compare_property("sequence_name", left, right, |p| &p.sequence_name),
                self.compare_property("data_type", left, right, |c| &c.data_type),
                self.compare_property("numeric_precision", left, right, |c| &c.numeric_precision),
                self.compare_property("numeric_precision_radix", left, right, |c| &c.numeric_precision_radix),
                self.compare_property("numeric_scale", left, right, |c| &c.numeric_scale),
                self.compare_property("start_value", left, right, |c| &c.start_value),
                self.compare_property("minimum_value", left, right, |c| &c.minimum_value),
                self.compare_property("maximum_value", left, right, |c| &c.maximum_value),
                self.compare_property("increment", left, right, |c| &c.increment),
                self.compare_property("cycle_option", left, right, |c| &c.cycle_option),
            ]
        }
    }

    fn compare_tables(&mut self,
                      left_tables: Vec<&Table>,
                      right_tables: Vec<&Table>,
                      left_columns: Vec<&Column>,
                      right_columns: Vec<&Column>,
                      left_column_privileges: Vec<&ColumnPrivilege>,
                      right_column_privileges: Vec<&ColumnPrivilege>,
                      left_indices: Vec<&Index>,
                      right_indices: Vec<&Index>,
                      left_table_privileges : Vec<&TablePrivilege>,
                      right_table_privileges : Vec<&TablePrivilege>,
                      left_table_constraints : Vec<&TableConstraint>,
                      right_table_constraints : Vec<&TableConstraint>,
                      left_table_triggers : Vec<&TableTrigger>,
                      right_table_triggers : Vec<&TableTrigger>,
    ) -> Result<Report<TableComparison>, Error> {
        let mut entries = Vec::new();
        let mut right_tables_map: HashMap<String, &Table> = right_tables.into_iter().map(|t| (t.table_name.clone(), t)).collect();

        for left_table in left_tables {
            let key = &left_table.table_name;
            let right_table = right_tables_map.get(key);
    
            match right_table {
                None => {
                    entries.push(TableRemoved { table_name: left_table.table_name.clone() });
                },
                Some(rt) => {
                    let properties = self.compare_table_properties(&left_table, rt);

                    let left_table_columns : Vec<&Column> = left_columns.iter().filter(|t| t.table_name == left_table.table_name).cloned().collect();
                    let right_table_columns : Vec<&Column> = right_columns.iter().filter(|t| t.table_name == rt.table_name).cloned().collect();
                    let left_table_column_privileges : Vec<&ColumnPrivilege> = left_column_privileges.iter().filter(|p| p.table_name == left_table.table_name).cloned().collect();
                    let right_table_column_privileges : Vec<&ColumnPrivilege> = right_column_privileges.iter().filter(|p| p.table_name == rt.table_name).cloned().collect();
                    let left_table_indices: Vec<&Index> = left_indices.iter().filter(|i| i.table_name == left_table.table_name).cloned().collect();
                    let right_table_indices: Vec<&Index> = right_indices.iter().filter(|i| i.table_name == rt.table_name).cloned().collect();
                    let left_table_table_privileges: Vec<&TablePrivilege> = left_table_privileges.iter().filter(|p| p.table_name == left_table.table_name).cloned().collect();
                    let right_table_table_privileges: Vec<&TablePrivilege> = right_table_privileges.iter().filter(|p| p.table_name == left_table.table_name).cloned().collect();
                    let left_table_table_constraints: Vec<&TableConstraint> = left_table_constraints.iter().filter(|c| c.table_name == left_table.table_name).cloned().collect();
                    let right_table_table_constraints: Vec<&TableConstraint> = right_table_constraints.iter().filter(|c| c.table_name == left_table.table_name).cloned().collect();
                    let left_table_table_triggers : Vec<&TableTrigger> = left_table_triggers.iter().filter(|t| t.event_object_table == left_table.table_name).cloned().collect();
                    let right_table_table_triggers : Vec<&TableTrigger> = right_table_triggers.iter().filter(|t| t.event_object_table == rt.table_name).cloned().collect();
                    
                    let columns = self.compare_table_columns(left_table_columns, right_table_columns, left_table_column_privileges, right_table_column_privileges)?;
                    let indices = self.compare_table_indices(left_table_indices, right_table_indices)?;
                    let privileges = self.compare_table_privileges(left_table_table_privileges, right_table_table_privileges)?;
                    let constraints = self.compare_table_constraints(left_table_table_constraints, right_table_table_constraints)?;
                    let triggers = self.compare_table_triggers(left_table_table_triggers, right_table_table_triggers)?;
    
                    entries.push(TableMaintained { table_name: left_table.table_name.clone(), columns, constraints, indices, privileges, properties, triggers });
    
                    right_tables_map.remove(key);
                },
            }
        }
    
        if right_tables_map.len() > 0 {
            let mut added_tables: Vec<&&Table> = right_tables_map.values().collect();
            added_tables.sort_unstable_by_key(|t| &t.table_name);
    
            for right_table in added_tables {
                entries.push(TableAdded { table_name: right_table.table_name.clone() });
            }
        }
    
        Ok(Report { entries })
    }

    fn compare_table_properties(&self, left: &Table, right: &Table) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("table_catalog", left, right, |c| &c.table_catalog),
                self.compare_property("table_schema", left, right, |c| &c.table_schema),
                self.compare_property("table_name", left, right, |c| &c.table_name),
                self.compare_property("table_type", left, right, |c| &c.table_type),
                self.compare_option_property("self_referencing_column_name", left, right, |t| &t.self_referencing_column_name),
                self.compare_option_property("reference_generation", left, right, |c| &c.reference_generation),
                self.compare_option_property("user_defined_type_catalog", left, right, |c| &c.user_defined_type_catalog),
                self.compare_option_property("user_defined_type_schema", left, right, |c| &c.user_defined_type_schema),
                self.compare_option_property("user_defined_type_name", left, right, |c| &c.user_defined_type_name),
                self.compare_property("is_insertable_into", left, right, |c| &c.is_insertable_into),
                self.compare_property("is_typed", left, right, |c| &c.is_typed),
                self.compare_option_property("commit_action", left, right, |c| &c.commit_action),
            ]
        }
    }

    fn compare_table_privileges(&mut self, left_table_privileges: Vec<&TablePrivilege>, right_table_privileges: Vec<&TablePrivilege>) -> Result<Report<PrivilegeComparison>, Error> {
        if self.ignore_privileges {
            return Ok(Report { entries: vec![] })
        }
    
        Ok(self.compare_privileges(&left_table_privileges, &right_table_privileges))
    }

    fn compare_table_columns(&mut self,
                             left_columns: Vec<&Column>,
                             right_columns: Vec<&Column>,
                             left_column_privileges: Vec<&ColumnPrivilege>,
                             right_column_privileges: Vec<&ColumnPrivilege>,
    ) -> Result<Report<ColumnComparison>, Error> {
        let mut entries = Vec::new();
        let mut right_columns_map : HashMap<String, &Column> = right_columns.into_iter().map(|c| (c.column_name.clone(), c)).collect();
    
        for left_column in left_columns {
            let key = &left_column.column_name;
            let right_column = right_columns_map.get_mut(key);
    
            match right_column {
                None => {
                    entries.push(ColumnRemoved { column_name: left_column.column_name.clone() });
                },
                Some(rc) => {
                    let properties = self.compare_table_column_properties(&left_column, rc);
                    
                    let left_column_privileges = left_column_privileges.iter().filter(|p| p.column_name == left_column.column_name).cloned().collect();
                    let right_column_privileges = right_column_privileges.iter().filter(|p| p.column_name == rc.column_name).cloned().collect();
                    let privileges = self.compare_table_column_privileges(left_column_privileges, right_column_privileges)?;
    
                    entries.push(ColumnMaintained { column_name: rc.column_name.clone(), properties, privileges });
    
                    right_columns_map.remove(key);
                },
            }
        }
    
        if right_columns_map.len() > 0 {
            let mut added_columns : Vec<&&Column> = right_columns_map.values().collect();
            added_columns.sort_unstable_by_key(|c| &c.column_name);
    
            for right_column in added_columns {
                entries.push(ColumnAdded { column_name: right_column.column_name.clone() });
            }
        }
    
        Ok(Report { entries })
    }
    
    fn compare_table_column_properties(&self, left: &Column, right: &Column) -> Report<PropertyComparison> {
        let mut properties = vec![
            self.compare_option_property("column_default", left, right, |c| &c.column_default),
            self.compare_property("is_nullable", left, right, |c| &c.is_nullable),
            self.compare_property("data_type", left, right, |c| &c.data_type),
            self.compare_option_property("character_maximum_length", left, right, |c| &c.character_maximum_length),
            self.compare_option_property("character_octet_length", left, right, |c| &c.character_octet_length),
            self.compare_option_property("numeric_precision", left, right, |c| &c.numeric_precision),
            self.compare_option_property("numeric_precision_radix", left, right, |c| &c.numeric_precision_radix),
            self.compare_option_property("numeric_scale", left, right, |c| &c.numeric_scale),
            self.compare_option_property("datetime_precision", left, right, |c| &c.datetime_precision),
            self.compare_option_property("interval_type", left, right, |c| &c.interval_type),
            self.compare_option_property("interval_precision", left, right, |c| &c.interval_precision),
            self.compare_option_property("character_set_catalog", left, right, |c| &c.character_set_catalog),
            self.compare_option_property("character_set_schema", left, right, |c| &c.character_set_schema),
            self.compare_option_property("character_set_name", left, right, |c| &c.character_set_name),
            self.compare_option_property("collation_catalog", left, right, |c| &c.collation_catalog),
            self.compare_option_property("collation_schema", left, right, |c| &c.collation_schema),
            self.compare_option_property("collation_name", left, right, |c| &c.collation_name),
            self.compare_option_property("domain_catalog", left, right, |c| &c.domain_catalog),
            self.compare_option_property("domain_schema", left, right, |c| &c.domain_schema),
            self.compare_option_property("domain_name", left, right, |c| &c.domain_name),
            self.compare_option_property("udt_catalog", left, right, |c| &c.udt_catalog),
            self.compare_option_property("udt_schema", left, right, |c| &c.udt_schema),
            self.compare_option_property("udt_name", left, right, |c| &c.udt_name),
            self.compare_option_property("scope_catalog", left, right, |c| &c.scope_catalog),
            self.compare_option_property("scope_schema", left, right, |c| &c.scope_schema),
            self.compare_option_property("scope_name", left, right, |c| &c.scope_name),
            self.compare_option_property("maximum_cardinality", left, right, |c| &c.maximum_cardinality),
            self.compare_option_property("dtd_identifier", left, right, |c| &c.character_set_name),
            self.compare_property("is_self_referencing", left, right, |c| &c.is_self_referencing),
            self.compare_property("is_identity", left, right, |c| &c.is_identity),
            self.compare_option_property("identity_generation", left, right, |c| &c.identity_generation),
            self.compare_option_property("identity_start", left, right, |c| &c.identity_start),
            self.compare_option_property("identity_increment", left, right, |c| &c.identity_increment),
            self.compare_option_property("identity_maximum", left, right, |c| &c.identity_maximum),
            self.compare_option_property("identity_minimum", left, right, |c| &c.identity_minimum),
            self.compare_option_property("identity_cycle", left, right, |c| &c.identity_cycle),
            self.compare_property("is_generated", left, right, |c| &c.is_generated),
            self.compare_option_property("generation_expression", left, right, |c| &c.generation_expression),
            self.compare_property("is_updatable", left, right, |c| &c.is_updatable),
        ];
    
        if !self.ignore_column_ordinal {
            properties.push(self.compare_property("ordinal_position", left, right, |c| &c.ordinal_position));
        }
    
        Report { entries: properties }
    }

    fn compare_table_column_privileges(&mut self, left_column_privileges: Vec<&ColumnPrivilege>, right_column_privileges: Vec<&ColumnPrivilege>) -> Result<Report<PrivilegeComparison>, Error> {
        if self.ignore_privileges {
            return Ok(Report { entries: vec![] })
        }
    
        Ok(self.compare_privileges(&left_column_privileges, &right_column_privileges))
    }

    fn compare_table_constraints(&mut self, left_table_constraints: Vec<&TableConstraint>, right_table_constraints: Vec<&TableConstraint>) -> Result<Report<TableConstraintComparison>, Error> {
        let mut right_table_constraints_map : HashMap<String, &TableConstraint> = right_table_constraints.into_iter().map(|t| (t.constraint_name.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_table_constraint in left_table_constraints {
            let key = &left_table_constraint.constraint_name;
            let right_table_constraint = right_table_constraints_map.get(key);
    
            match right_table_constraint {
                None => {
                    entries.push(ConstraintRemoved { constraint_name: left_table_constraint.clone().constraint_name });
                },
                Some(rtc) => {
                    let properties = self.compare_table_constraint_properties(&left_table_constraint, rtc);
    
                    entries.push(ConstraintMaintained { constraint_name: rtc.constraint_name.clone(), properties });
    
                    right_table_constraints_map.remove(key);
                },
            }
        }
    
        if right_table_constraints_map.len() > 0 {
            let mut added_table_constraints : Vec<&&TableConstraint> = right_table_constraints_map.values().collect();
            added_table_constraints.sort_unstable_by_key(|tc| &tc.constraint_name);
    
            for right_table_constraint in added_table_constraints {
                entries.push(ConstraintAdded { constraint_name: right_table_constraint.constraint_name.clone() });
            }
        }
    
        Ok(Report { entries })
    }

    fn compare_table_constraint_properties(&mut self, left: &TableConstraint, right: &TableConstraint) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("constraint_catalog", left, right, |c| &c.constraint_catalog),
                self.compare_property("constraint_schema", left, right, |c| &c.constraint_schema),
                self.compare_property("table_name", left, right, |c| &c.table_name),
                self.compare_property("constraint_type", left, right, |c| &c.constraint_type),
                self.compare_property("is_deferrable", left, right, |c| &c.is_deferrable),
                self.compare_property("initially_deferred", left, right, |c| &c.initially_deferred),
                self.compare_property("enforced", left, right, |c| &c.enforced),
                self.compare_option_property("nulls_distinct", left, right, |c| &c.nulls_distinct),
            ]
        }
    }

    fn compare_table_indices(&mut self,
                             left_indices: Vec<&Index>,
                             right_indices: Vec<&Index>,
    ) -> Result<Report<IndexComparison>, Error> {
        let mut entries = Vec::new();
        let mut right_indices_map : HashMap<String, &Index> = right_indices.into_iter().map(|c| (c.index_name.clone(), c)).collect();

        for left_index in left_indices {
            let key = &left_index.index_name;
            let right_index = right_indices_map.get_mut(key);

            match right_index {
                None => {
                    entries.push(IndexRemoved { index_name: left_index.index_name.clone() });
                },
                Some(ri) => {
                    let properties = self.compare_table_index_properties(&left_index, ri);

                    entries.push(IndexMaintained { index_name: ri.index_name.clone(), properties });

                    right_indices_map.remove(key);
                },
            }
        }

        if right_indices_map.len() > 0 {
            let mut added_indices: Vec<&&Index> = right_indices_map.values().collect();
            added_indices.sort_unstable_by_key(|i| &i.index_name);

            for right_index in added_indices {
                entries.push(IndexAdded { index_name: right_index.index_name.clone() });
            }
        }

        Ok(Report { entries })
    }
    
    fn compare_table_index_properties(&mut self, left: &Index, right: &Index) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("definition", left, right, |i| &i.definition),
            ]
        }
    }

    fn compare_table_triggers(&mut self, left_table_triggers: Vec<&TableTrigger>, right_table_triggers: Vec<&TableTrigger>) -> Result<Report<TableTriggerComparison>, Error> {
        let mut right_triggers_map : HashMap<(String, String), &TableTrigger> = right_table_triggers.into_iter().map(|t| ((t.trigger_name.clone(), t.event_manipulation.clone()), t)).collect();
        let mut entries = Vec::new();
    
        for left_table_trigger in left_table_triggers {
            let key = (left_table_trigger.trigger_name.clone(), left_table_trigger.event_manipulation.clone());
            let right_table_trigger = right_triggers_map.get(&key);
    
            match right_table_trigger {
                None => {
                    entries.push(TriggerRemoved { trigger_name: left_table_trigger.trigger_name.clone(), event_manipulation: left_table_trigger.event_manipulation.clone() });
                },
                Some(rt) => {
                    let properties = self.compare_trigger_properties(&left_table_trigger, rt);
    
                    entries.push(TriggerMaintained { trigger_name: rt.trigger_name.clone(), event_manipulation: rt.event_manipulation.clone(), properties });
    
                    right_triggers_map.remove(&key);
                },
            }
        }
    
        if right_triggers_map.len() > 0 {
            let mut added_triggers : Vec<&&TableTrigger> = right_triggers_map.values().collect();
            added_triggers.sort_unstable_by_key(|t| (&t.trigger_name, &t.event_manipulation));
    
            for right_trigger in added_triggers {
                entries.push(TriggerAdded { trigger_name: right_trigger.trigger_name.clone(), event_manipulation: right_trigger.event_manipulation.clone() });
            }
        }
    
        Ok(Report { entries })
    }

    fn compare_trigger_properties(&mut self, left: &TableTrigger, right: &TableTrigger) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_property("trigger_catalog", left, right, |t| &t.trigger_catalog),
                self.compare_property("trigger_schema", left, right, |t| &t.trigger_schema),
                self.compare_property("action_order", left, right, |t| &t.action_order),
                self.compare_option_property("action_condition", left, right, |t| &t.action_condition),
                self.compare_property("action_statement", left, right, |t| &t.action_statement),
                self.compare_property("action_orientation", left, right, |t| &t.action_orientation),
                self.compare_property("action_timing", left, right, |t| &t.action_timing),
                self.compare_option_property("action_reference_old_table", left, right, |t| &t.action_reference_old_table),
                self.compare_option_property("action_reference_new_table", left, right, |t| &t.action_reference_new_table),
            ]
        }
    }

    fn compare_views(&mut self, left_views: Vec<&View>, right_views: Vec<&View>) -> Result<Report<ViewComparison>, Error> {
        let right_views_map: HashMap<String, &View> = right_views.into_iter().map(|t| (t.table_name.clone(), t)).collect();
        let mut entries = Vec::new();
    
        for left_view in left_views {
            let right_view = right_views_map.get(&left_view.table_name);
    
            match right_view {
                None => {
                    // already detected by table comparison
                    continue
                },
                Some(rv) => {
                    let properties = self.compare_view_properties(&left_view, rv);
    
                    entries.push(ViewMaintained { view_name: rv.table_name.clone(), properties });
                }
            }
        }
    
        // added already detected by table comparison
    
        Ok(Report { entries })
    }

    fn compare_view_properties(&mut self, left: &View, right: &View) -> Report<PropertyComparison> {
        Report {
            entries: vec![
                self.compare_option_property("view_definition", left, right, |c| &c.view_definition),
                self.compare_property("check_option", left, right, |c| &c.check_option),
                self.compare_property("is_updatable", left, right, |c| &c.is_updatable),
                self.compare_property("is_insertable_into", left, right, |c| &c.is_insertable_into),
                self.compare_property("is_trigger_updatable", left, right, |c| &c.is_trigger_updatable),
                self.compare_property("is_trigger_deletable", left, right, |c| &c.is_trigger_deletable),
                self.compare_property("is_trigger_insertable_into", left, right, |c| &c.is_trigger_insertable_into),
            ]
        }
    }

    fn compare_property<'a, T, P>(&self, property_name: &str, left: T, right: T, accessor: fn(T) -> &'a P) -> PropertyComparison
        where P: PartialEq, P: Display
    {
        let left_value = accessor(left);
        let right_value = accessor(right);
        
        if left_value == right_value {
            PropertyUnchanged { property_name: String::from(property_name), value: left_value.to_string() }
        } else {
            PropertyChanged { property_name: String::from(property_name), left_value: left_value.to_string(), right_value: right_value.to_string() }
        }
    }

    fn compare_option_property<'a, T, P>(&self, property_name: &str, left: T, right: T, accessor: fn(T) -> &'a Option<P>) -> PropertyComparison
        where P: PartialEq, P: Display
    {
        let left_value = accessor(left);
        let right_value = accessor(right);
        
        self.compare_option_property_impl(property_name, &left_value, &right_value, &|l: &P, r: &P| l == r)
    }
    
    fn compare_option_property_ignore_whitespace<'a, T, P>(&self, property_name: &str, left: T, right: T, accessor: fn(T) -> &'a Option<P>) -> PropertyComparison
        where P: PartialEq, P: Display
    {
        let left_value = accessor(left);
        let right_value = accessor(right);

        self.compare_option_property_impl(property_name, left_value, right_value, &|l: &P, r: &P| l.to_string().as_str().eq_ignore_whitespace(r.to_string().as_str()))
    }

    fn compare_option_property_impl<T>(&self, property_name: &str, left_value: &Option<T>, right_value: &Option<T>, compare: &dyn Fn(&T, &T) -> bool) -> PropertyComparison
        where T: PartialEq, T: Display
    {
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
