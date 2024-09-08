use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};
use crate::compare::report::privilege::PrivilegeComparison;

pub enum RoutineComparison {
    RoutineAdded { routine_signature: String },
    RoutineRemoved { routine_signature: String },
    RoutineMaintained { routine_signature: String, properties: Report<PropertyComparison>, privileges: Report<PrivilegeComparison> },
}

impl HasChanges for RoutineComparison {
    fn has_changes(&self) -> bool {
        match self {
            RoutineComparison::RoutineAdded { .. } | RoutineComparison::RoutineRemoved { .. } => true,
            RoutineComparison::RoutineMaintained { routine_signature: _routine_signature, properties, privileges } =>
                properties.has_changes() ||
                privileges.has_changes(),
        }
    }
}

