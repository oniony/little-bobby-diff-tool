use std::process;
use clap::{arg, Parser};
use postgres::Error;

use crate::{compare, db};
use crate::compare::report::PrivilegeComparison::{PrivilegeAdded, PrivilegeMaintained, PrivilegeRemoved};
use crate::compare::report::{HasChanges, TableColumnReport, TableConstraintReport, SchemaReport, RoutineReport, SequenceReport, ViewReport, PropertyReport, PrivilegeReport, TableReport, TableTriggerReport};
use crate::compare::report::PropertyComparison::{PropertyChanged, PropertyUnchanged};
use crate::compare::report::RoutineComparison::{RoutineAdded, RoutineMaintained, RoutineRemoved};
use crate::compare::report::SchemaComparison::{SchemaAdded, SchemaMaintained, SchemaMissing, SchemaRemoved};
use crate::compare::report::SequenceComparison::{SequenceAdded, SequenceMaintained, SequenceRemoved};
use crate::compare::report::TableColumnComparison::{ColumnAdded, ColumnMaintained, ColumnRemoved};
use crate::compare::report::TableComparison::{TableAdded, TableMaintained, TableRemoved};
use crate::compare::report::TableConstraintComparison::{ConstraintAdded, ConstraintMaintained, ConstraintRemoved};
use crate::compare::report::TableTriggerComparison::{TriggerAdded, TriggerMaintained, TriggerRemoved};
use crate::compare::report::ViewComparison::{ViewMaintained};

pub struct CLI {
    args: Args
}

impl CLI {
    pub fn new() -> CLI {
        let args = Args::parse();
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

    fn render_schema_report(&self, report: SchemaReport) -> i32{
        let mut differences = 0;

        for schema in &report.entries {
            match schema {
                SchemaMissing { schema_name } => {
                    println!("Schema '{}': missing", schema_name);
                    differences += 1;
                },
                SchemaMaintained { schema_name, properties, routines, sequences, tables, views } => {
                    if schema.has_changes() {
                        println!("Schema '{}':", schema_name);
                    }

                    differences += self.render_property_report(properties, 1);
                    differences += self.render_routine_report(routines);
                    differences += self.render_sequence_report(sequences);
                    differences += self.render_table_report(tables);
                    differences += self.render_view_report(views);
                }
                SchemaAdded { schema_name } => {
                    println!("Schema '{}': added", schema_name);
                    differences += 1;
                },
                SchemaRemoved { schema_name } => {
                    println!("Schema '{}': removed", schema_name);
                    differences += 1;
                }
            }
        }
        
        differences
    }

    fn render_property_report(&self, report: &PropertyReport, depth: usize) -> i32 {
        let mut differences = 0;
        let margin = str::repeat("  ", depth);

        for property in &report.entries {
            match property {
                PropertyChanged { property_name, left_value, right_value } => {
                    println!("{}Property '{}': changed from '{}' to '{}'", margin, property_name, left_value, right_value);
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

    fn render_privilege_report(&self, report: &PrivilegeReport, depth: usize) -> i32 {
        let mut differences = 0;
        let margin = str::repeat("  ", depth);

        for privilege in &report.entries {
            match privilege {
                PrivilegeAdded { privilege_name, grantor, grantee } => {
                    println!("{}Privilege '{}' ({}->{}): added", margin, privilege_name, grantor, grantee);
                    differences += 1;
                }
                PrivilegeRemoved { privilege_name, grantor, grantee } => {
                    println!("{}Privilege '{}' ({}->{}): removed", margin, privilege_name, grantor, grantee);
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

    fn render_routine_report(&self, report: &RoutineReport) -> i32 {
        let mut differences = 0;

        for routine in &report.entries {
            match routine {
                RoutineMaintained { routine_signature, properties, privileges } => {
                    if routine.has_changes() {
                        println!("  Routine '{}':", routine_signature);
                    }
                    
                    differences += self.render_property_report(&properties, 2);
                    differences += self.render_privilege_report(&privileges, 2);
                },
                RoutineAdded { routine_signature } => {
                    println!("  Routine '{}': added", routine_signature);
                    differences += 1;
                }
                RoutineRemoved { routine_signature } => {
                    println!("  Routine '{}': removed", routine_signature);
                    differences += 1;
                }
            }
        }
        
        differences
    }
    
    fn render_sequence_report(&self, report: &SequenceReport) -> i32 {
        let mut differences = 0;

        for sequence in &report.entries {
            match sequence {
                SequenceMaintained { sequence_name, properties } => {
                    if sequence.has_changes() {
                        println!("  Sequence '{}':", sequence_name);
                    }
                    
                    differences += self.render_property_report(&properties, 2);
                },
                SequenceAdded { sequence_name } => {
                    println!("  Sequence '{}': added", sequence_name);
                    differences += 1;
                }
                SequenceRemoved { sequence_name } => {
                    println!("  Sequence '{}': removed", sequence_name);
                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_report(&self, report: &TableReport) -> i32 {
        let mut differences = 0;

        for table in &report.entries {
            match table {
                TableMaintained { table_name, properties, columns, privileges, constraints, triggers } => {
                    if table.has_changes() {
                        println!("  Table '{}':", table_name);
                    }
                    
                    differences += self.render_property_report(&properties, 2);
                    differences += self.render_table_column_report(&columns);
                    differences += self.render_privilege_report(&privileges, 2);
                    differences += self.render_table_constraint_report(&constraints);
                    differences += self.render_table_trigger_report(&triggers);
                },
                TableAdded { table_name } => {
                    println!("  Table '{}': added", table_name);
                    differences += 1;
                }
                TableRemoved { table_name } => {
                    println!("  Table '{}': removed", table_name);
                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_column_report(&self, report: &TableColumnReport) -> i32 {
        let mut differences = 0;

        for column in &report.entries {
            match column {
                ColumnMaintained { column_name, properties, privileges  } => {
                    if column.has_changes() {
                        println!("    Column '{}':", column_name);
                    }

                    differences += self.render_property_report(&properties, 3);
                    differences += self.render_privilege_report(&privileges, 3);
                },
                ColumnAdded { column_name } => {
                    println!("    Column '{}': added", column_name);
                    differences += 1;
                }
                ColumnRemoved { column_name } => {
                    println!("    Column '{}': removed", column_name);
                    differences += 1;
                }
            }
        }

        differences
    }
    
    fn render_table_constraint_report(&self, report: &TableConstraintReport) -> i32 {
        let mut differences = 0;

        for constraint in &report.entries {
            match constraint {
                ConstraintMaintained { constraint_name, properties  } => {
                    if constraint.has_changes() {
                        println!("    Constraint '{}':", constraint_name);
                    }

                    differences += self.render_property_report(&properties, 3);
                },
                ConstraintAdded { constraint_name } => {
                    println!("    Constraint '{}': added", constraint_name);
                    differences += 1;
                }
                ConstraintRemoved { constraint_name } => {
                    println!("    Constraint '{}': removed", constraint_name);
                    differences += 1;
                }
            }
        }

        differences
    }

    fn render_table_trigger_report(&self, report: &TableTriggerReport) -> i32 {
        let mut differences = 0;

        for trigger in &report.entries {
            match trigger {
                TriggerMaintained { trigger_name, event_manipulation, properties } => {
                    if trigger.has_changes() {
                        println!("    Trigger '{}' ({}):", trigger_name, event_manipulation);
                    }

                    differences += self.render_property_report(&properties, 3);
                },
                TriggerAdded { trigger_name, event_manipulation } => {
                    println!("    Trigger '{}' ({}): added", trigger_name, event_manipulation);
                    differences += 1;
                }
                TriggerRemoved { trigger_name, event_manipulation } => {
                    println!("    Trigger '{}' ({}): removed", trigger_name, event_manipulation);
                    differences += 1;
                }
            }
        }

        differences
    }

    fn render_view_report(&self, report: &ViewReport) -> i32 {
        let mut differences = 0;

        for view in &report.entries {
            match view {
                ViewMaintained { view_name, properties } => {
                    if view.has_changes() {
                        println!("  View '{}':", view_name);
                    }

                    differences += self.render_property_report(&properties, 2);
                },
            }
        }

        differences
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, short = 'l', help = "The left database URL")]
    left: String,
    
    #[arg(short, long, short = 'r', help = "The right database URL")]
    right: String,
    
    #[arg(short, long, required = true, short = 's', help = "Schema to compare")]
    schema: Vec<String>,
    
    #[arg(short, long, short = 'w', help = "Ignore routine whitespace differences")]
    ignore_whitespace: bool,
    
    #[arg(short, long, short = 'o', help = "Ignore column ordering differences")]
    ignore_column_ordinal: bool,
    
    #[arg(short, long, short = 'p', help = "Ignore privilege changes")]
    ignore_privileges: bool,
    
    #[arg(short, long, short = 'v', help = "Show matches")]
    verbose: bool,
}
