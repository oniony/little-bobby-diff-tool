use crate::compare::report::{HasChanges, Report};
use crate::compare::report::privilege::PrivilegeComparison;
use crate::compare::report::property::PropertyComparison;
use crate::compare::report::table_column::TableColumnComparison;
use crate::compare::report::table_constraint::TableConstraintComparison;
use crate::compare::report::table_trigger::TableTriggerComparison;

pub enum TableComparison {
    TableAdded { table_name: String },
    TableRemoved { table_name: String },
    TableMaintained { table_name: String, properties: Report<PropertyComparison>, columns: Report<TableColumnComparison>, privileges: Report<PrivilegeComparison>, constraints: Report<TableConstraintComparison>, triggers: Report<TableTriggerComparison> },
}

impl HasChanges for TableComparison {
    fn has_changes(&self) -> bool {
        match self {
            TableComparison::TableAdded { .. } | TableComparison::TableRemoved { .. } => true,
            TableComparison::TableMaintained { table_name: _table_name, properties, columns, privileges, constraints, triggers } =>
                properties.has_changes() ||
                columns.has_changes() ||
                privileges.has_changes() ||
                constraints.has_changes() ||
                triggers.has_changes(),
        }
    }
}

