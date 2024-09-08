use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};

pub enum TableConstraintComparison {
    ConstraintAdded { constraint_name: String },
    ConstraintRemoved { constraint_name: String },
    ConstraintMaintained { constraint_name: String, properties: Report<PropertyComparison> },
}

impl HasChanges for TableConstraintComparison {
    fn has_changes(&self) -> bool {
        match self {
            TableConstraintComparison::ConstraintAdded { .. } | TableConstraintComparison::ConstraintRemoved { .. } => true,
            TableConstraintComparison::ConstraintMaintained { constraint_name: _constraint_name, properties } =>
                properties.has_changes(),
        }
    }
}
