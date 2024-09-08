mod args;

use std::process;
use clap::{Parser};
use colored::Colorize;
use postgres::Error;

use crate::{compare, db};
use crate::cli::args::{Args, Colouring::Always, Colouring::Never};
use crate::compare::report::{HasChanges, Report};
use crate::compare::report::privilege::PrivilegeComparison;
use crate::compare::report::privilege::PrivilegeComparison::{PrivilegeAdded, PrivilegeMaintained, PrivilegeRemoved};
use crate::compare::report::property::PropertyComparison;
use crate::compare::report::property::PropertyComparison::{PropertyChanged, PropertyUnchanged};
use crate::compare::report::routine::RoutineComparison;
use crate::compare::report::routine::RoutineComparison::{RoutineAdded, RoutineMaintained, RoutineRemoved};
use crate::compare::report::schema::SchemaComparison;
use crate::compare::report::schema::SchemaComparison::{SchemaAdded, SchemaMaintained, SchemaMissing, SchemaRemoved};
use crate::compare::report::sequence::SequenceComparison;
use crate::compare::report::sequence::SequenceComparison::{SequenceAdded, SequenceMaintained, SequenceRemoved};
use crate::compare::report::table::TableComparison;
use crate::compare::report::table::TableComparison::{TableAdded, TableMaintained, TableRemoved};
use crate::compare::report::table_column::TableColumnComparison;
use crate::compare::report::table_column::TableColumnComparison::{ColumnAdded, ColumnMaintained, ColumnRemoved};
use crate::compare::report::table_constraint::TableConstraintComparison;
use crate::compare::report::table_constraint::TableConstraintComparison::{ConstraintAdded, ConstraintMaintained, ConstraintRemoved};
use crate::compare::report::table_trigger::TableTriggerComparison::{TriggerAdded, TriggerMaintained, TriggerRemoved};
use crate::compare::report::table_trigger::{TableTriggerComparison};
use crate::compare::report::view::ViewComparison;
use crate::compare::report::view::ViewComparison::ViewMaintained;

const COLOUR_ADDED: colored::Color = colored::Color::Green;
const COLOUR_CHANGED: colored::Color= colored::Color::Yellow;
const COLOUR_MISSING: colored::Color = colored::Color::Magenta;
const COLOUR_REMOVED: colored::Color = colored::Color::Red;

pub struct CLI {
    args: Args,
}

impl CLI {
    pub fn new() -> CLI {
        let args = Args::parse();
        
        match args.color {
            Always => colored::control::set_override(true),
            Never => colored::control::set_override(false),
            _ => (),
        }
        
        CLI { args }
    }
    
    pub fn run(&self) -> Result<i32, Error> {
        let left_db = db::Database::connect(self.args.left.as_str())?;
        let right_db = db::Database::connect(self.args.right.as_str())?;

        let mut comparer = compare::Comparer::new(
            left_db,
            right_db,
            self.args.ignore_whitespace,
            self.args.ignore_column_ordinal,
            self.args.ignore_privileges);

        let mut differences = 0;

        let report = comparer.compare(self.args.schema.clone())?;
        differences += self.render_schema_report(report);

        process::exit(differences);
    }

    fn render_schema_report(&self, report: Report<SchemaComparison>) -> i32{
        let mut differences = 0;

        for schema in &report.entries {
            match schema {
                SchemaMissing { schema_name } => {
                    let message = format!("Schema '{}': missing in both", schema_name);
                    println!("{}", message.color(COLOUR_MISSING));
                    
                    differences += 1;
                },
                SchemaMaintained { schema_name, properties, routines, sequences, tables, views } => {
                    let has_changes = schema.has_changes();
                    
                    if has_changes {
                        let message = format!("Schema '{}':", schema_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("Schema '{}': unchanged", schema_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(properties, 1);
                        differences += self.render_routine_report(routines);
                        differences += self.render_sequence_report(sequences);
                        differences += self.render_table_report(tables);
                        differences += self.render_view_report(views);
                    }
                }
                SchemaAdded { schema_name } => {
                    let message = format!("Schema '{}': added", schema_name);
                    println!("{}", message.color(COLOUR_ADDED));

                    differences += 1;
                },
                SchemaRemoved { schema_name } => {
                    let message = format!("Schema '{}': removed", schema_name);
                    println!("{}", message.red());

                    differences += 1;
                }
            }
        }
        
        differences
    }

    fn render_property_report(&self, report: &Report<PropertyComparison>, depth: usize) -> i32 {
        let mut differences = 0;
        let margin = str::repeat("  ", depth);

        for property in &report.entries {
            match property {
                PropertyChanged { property_name, left_value, right_value } => {
                    let message = format!("{}Property '{}': changed from '{}' to '{}'", margin, property_name, left_value.color(COLOUR_REMOVED), right_value.color(COLOUR_ADDED));
                    println!("{}", message.color(COLOUR_CHANGED));

                    differences += 1;
                }
                PropertyUnchanged { property_name, value } => {
                    if self.args.verbose {
                        println!("{}Property '{}': unchanged at '{}'", margin, property_name, value);
                    }
                },
            }
        }
        
        differences
    }

    fn render_privilege_report(&self, report: &Report<PrivilegeComparison>, depth: usize) -> i32 {
        let mut differences = 0;
        let margin = str::repeat("  ", depth);

        for privilege in &report.entries {
            match privilege {
                PrivilegeAdded { privilege_name, grantor, grantee } => {
                    let message = format!("{}Privilege '{}' ({}->{}): added", margin, privilege_name, grantor, grantee);
                    println!("{}", message.color(COLOUR_ADDED));
                    
                    differences += 1;
                }
                PrivilegeRemoved { privilege_name, grantor, grantee } => {
                    let message = format!("{}Privilege '{}' ({}->{}): removed", margin, privilege_name, grantor, grantee);
                    println!("{}", message.color(COLOUR_REMOVED));

                    differences += 1;
                },
                PrivilegeMaintained { privilege_name, grantor, grantee } => {
                    if self.args.verbose {
                        println!("{}Privilege '{}' ({}->{}): unchanged", margin, privilege_name, grantor, grantee);
                    }
                },
            }
        }

        differences
    }

    fn render_routine_report(&self, report: &Report<RoutineComparison>) -> i32 {
        let mut differences = 0;

        for routine in &report.entries {
            match routine {
                RoutineMaintained { routine_signature, properties, privileges } => {
                    let has_changes = routine.has_changes();
                    
                    if has_changes {
                        let message = format!("  Routine '{}':", routine_signature);
                        println!("{}", message.color(COLOUR_CHANGED));

                    } else if self.args.verbose {
                        println!("  Routine '{}': unchanged", routine_signature);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 2);
                        differences += self.render_privilege_report(&privileges, 2);
                    }
                },
                RoutineAdded { routine_signature } => {
                    let message = format!("  Routine '{}': added", routine_signature);
                    println!("{}", message.color(COLOUR_ADDED));

                    differences += 1;
                }
                RoutineRemoved { routine_signature } => {
                    let message = format!("  Routine '{}': removed", routine_signature);
                    println!("{}", message.color(COLOUR_REMOVED));

                    differences += 1;
                }
            }
        }
        
        differences
    }
    
    fn render_sequence_report(&self, report: &Report<SequenceComparison>) -> i32 {
        let mut differences = 0;

        for sequence in &report.entries {
            match sequence {
                SequenceMaintained { sequence_name, properties } => {
                    let has_changes = sequence.has_changes();
                    
                    if has_changes {
                        let message = format!("  Sequence '{}':", sequence_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("  Sequence '{}': unchanged", sequence_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 2);
                    }
                },
                SequenceAdded { sequence_name } => {
                    let message = format!("  Sequence '{}': added", sequence_name);
                    println!("{}", message.color(COLOUR_ADDED));

                    differences += 1;
                }
                SequenceRemoved { sequence_name } => {
                    let message = format!("  Sequence '{}': removed", sequence_name);
                    println!("{}", message.color(COLOUR_REMOVED));
                    
                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_report(&self, report: &Report<TableComparison>) -> i32 {
        let mut differences = 0;

        for table in &report.entries {
            match table {
                TableMaintained { table_name, properties, columns, privileges, constraints, triggers } => {
                    let has_changes = table.has_changes();
                    
                    if has_changes {
                        let message = format!("  Table '{}':", table_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("  Table '{}': unchanged", table_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 2);
                        differences += self.render_table_column_report(&columns);
                        differences += self.render_privilege_report(&privileges, 2);
                        differences += self.render_table_constraint_report(&constraints);
                        differences += self.render_table_trigger_report(&triggers);
                    }
                },
                TableAdded { table_name } => {
                    let message = format!("  Table '{}': added", table_name);
                    println!("{}", message.color(COLOUR_ADDED));

                    differences += 1;
                }
                TableRemoved { table_name } => {
                    let message = format!("  Table '{}': removed", table_name);
                    println!("{}", message.color(COLOUR_REMOVED));
                    
                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_column_report(&self, report: &Report<TableColumnComparison>) -> i32 {
        let mut differences = 0;

        for column in &report.entries {
            match column {
                ColumnMaintained { column_name, properties, privileges  } => {
                    let has_changes = column.has_changes();
                    
                    if has_changes {
                        let message = format!("    Column '{}':", column_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("    Column '{}': unchanged", column_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 3);
                        differences += self.render_privilege_report(&privileges, 3);
                    }
                },
                ColumnAdded { column_name } => {
                    let message = format!("    Column '{}': added", column_name);
                    println!("{}", message.color(COLOUR_ADDED));

                    differences += 1;
                }
                ColumnRemoved { column_name } => {
                    let message = format!("    Column '{}': removed", column_name);
                    println!("{}", message.color(COLOUR_REMOVED));

                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_constraint_report(&self, report: &Report<TableConstraintComparison>) -> i32 {
        let mut differences = 0;

        for constraint in &report.entries {
            match constraint {
                ConstraintMaintained { constraint_name, properties  } => {
                    let has_changes = constraint.has_changes();
                    
                    if has_changes {
                        let message = format!("    Constraint '{}':", constraint_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("    Constraint '{}': unchanged", constraint_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 3);
                    }
                },
                ConstraintAdded { constraint_name } => {
                    let message = format!("    Constraint '{}': added", constraint_name);
                    println!("{}", message.color(COLOUR_ADDED));
                    
                    differences += 1;
                }
                ConstraintRemoved { constraint_name } => {
                    let message = format!("    Constraint '{}': removed", constraint_name);
                    println!("{}", message.color(COLOUR_REMOVED));

                    differences += 1;
                }
            }
        }

        differences
    }

    fn render_table_trigger_report(&self, report: &Report<TableTriggerComparison>) -> i32 {
        let mut differences = 0;

        for trigger in &report.entries {
            match trigger {
                TriggerMaintained { trigger_name, event_manipulation, properties } => {
                    let has_changes = trigger.has_changes();
                    
                    if has_changes {
                        let message = format!("    Trigger '{}' ({}):", trigger_name, event_manipulation);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("    Trigger '{}' ({}): unchanged", trigger_name, event_manipulation);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 3);
                    }
                },
                TriggerAdded { trigger_name, event_manipulation } => {
                    let message = format!("    Trigger '{}' ({}): added", trigger_name, event_manipulation);
                    println!("{}", message.color(COLOUR_ADDED));
                    
                    differences += 1;
                }
                TriggerRemoved { trigger_name, event_manipulation } => {
                    let message = format!("    Trigger '{}' ({}): removed", trigger_name, event_manipulation);
                    println!("{}", message.color(COLOUR_REMOVED));
                    
                    differences += 1;
                }
            }
        }

        differences
    }

    fn render_view_report(&self, report: &Report<ViewComparison>) -> i32 {
        let mut differences = 0;

        for view in &report.entries {
            match view {
                ViewMaintained { view_name, properties } => {
                    let has_changes = view.has_changes();
                    
                    if has_changes {
                        let message = format!("  View '{}':", view_name);
                        println!("{}", message.color(COLOUR_CHANGED));
                    } else if self.args.verbose {
                        println!("  View '{}': unchanged", view_name);
                    }
                    
                    if has_changes || self.args.verbose {
                        differences += self.render_property_report(&properties, 2);
                    }
                },
            }
        }

        differences
    }
}
