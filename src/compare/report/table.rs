use crate::compare::report::{HasChanges, Report};
use crate::compare::report::privilege::PrivilegeComparison;
use crate::compare::report::property::PropertyComparison;
use crate::compare::report::column::ColumnComparison;
use crate::compare::report::index::IndexComparison;
use crate::compare::report::table_constraint::TableConstraintComparison;
use crate::compare::report::table_trigger::TableTriggerComparison;

pub enum TableComparison {
    TableAdded { table_name: String },
    TableRemoved { table_name: String },
    TableMaintained {
        table_name: String,
        columns: Report<ColumnComparison>,
        constraints: Report<TableConstraintComparison>,
        indices: Report<IndexComparison>,
        privileges: Report<PrivilegeComparison>,
        properties: Report<PropertyComparison>,
        triggers: Report<TableTriggerComparison>
    },
}

impl HasChanges for TableComparison {
    fn has_changes(&self) -> bool {
        match self {
            TableComparison::TableAdded { .. } | TableComparison::TableRemoved { .. } => true,
            TableComparison::TableMaintained { table_name: _table_name, columns, constraints, indices, privileges, properties, triggers } =>
                columns.has_changes() ||
                indices.has_changes() ||
                privileges.has_changes() ||
                properties.has_changes() ||
                constraints.has_changes() ||
                triggers.has_changes(),
        }
    }
}

