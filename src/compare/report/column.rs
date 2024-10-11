use crate::compare::report::privilege::{PrivilegeComparison};
use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};

pub enum ColumnComparison {
    ColumnAdded { column_name: String },
    ColumnRemoved { column_name: String },
    ColumnMaintained { column_name: String, properties: Report<PropertyComparison>, privileges: Report<PrivilegeComparison> }
}

impl HasChanges for ColumnComparison {
    fn has_changes(&self) -> bool {
        match self {
            ColumnComparison::ColumnAdded { .. } | ColumnComparison::ColumnRemoved { .. } => true,
            ColumnComparison::ColumnMaintained { column_name: _column_name, properties, privileges } =>
                properties.has_changes() |
                privileges.has_changes(),
        }
    }
}
