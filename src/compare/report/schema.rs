use crate::compare::report::{HasChanges, Report};
use crate::compare::report::property::PropertyComparison;
use crate::compare::report::routine::RoutineComparison;
use crate::compare::report::sequence::SequenceComparison;
use crate::compare::report::table::TableComparison;
use crate::compare::report::view::ViewComparison;

pub enum SchemaComparison {
    SchemaAdded { schema_name: String },
    SchemaRemoved { schema_name: String },
    SchemaMissing { schema_name: String },
    SchemaMaintained { schema_name: String, properties: Report<PropertyComparison>, routines: Report<RoutineComparison>, sequences: Report<SequenceComparison>, tables: Report<TableComparison>, views: Report<ViewComparison> },
}

impl HasChanges for SchemaComparison {
    fn has_changes(&self) -> bool {
        match self {
            SchemaComparison::SchemaAdded { .. } | SchemaComparison::SchemaRemoved { .. } | SchemaComparison::SchemaMissing { .. } => true,
            SchemaComparison::SchemaMaintained { schema_name: _schema_name, properties, routines, sequences, tables, views } =>
                properties.has_changes() ||
                routines.has_changes() ||
                sequences.has_changes() ||
                tables.has_changes() ||
                views.has_changes(),
        }
    }
}
