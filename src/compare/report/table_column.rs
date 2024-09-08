use crate::compare::report::privilege::{PrivilegeComparison};
use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};

pub enum TableColumnComparison {
    ColumnAdded { column_name: String },
    ColumnRemoved { column_name: String },
    ColumnMaintained { column_name: String, properties: Report<PropertyComparison>, privileges: Report<PrivilegeComparison> }
}

impl HasChanges for TableColumnComparison {
    fn has_changes(&self) -> bool {
        match self {
            TableColumnComparison::ColumnAdded { .. } | TableColumnComparison::ColumnRemoved { .. } => true,
            TableColumnComparison::ColumnMaintained { column_name: _column_name, properties, privileges } =>
                properties.has_changes() |
                privileges.has_changes(),
        }
    }
}
