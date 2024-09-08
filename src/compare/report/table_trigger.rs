use crate::compare::report::{HasChanges, Report};
use crate::compare::report::property::PropertyComparison;

pub enum TableTriggerComparison {
    TriggerAdded { trigger_name: String, event_manipulation: String },
    TriggerRemoved { trigger_name: String, event_manipulation: String },
    TriggerMaintained { trigger_name: String, event_manipulation: String, properties: Report<PropertyComparison> }
}

impl HasChanges for TableTriggerComparison {
    fn has_changes(&self) -> bool {
        match self {
            TableTriggerComparison::TriggerAdded { .. } | TableTriggerComparison::TriggerRemoved { .. } => true,
            TableTriggerComparison::TriggerMaintained { trigger_name: _trigger_name, event_manipulation: _event_manipulation, properties } =>
                properties.has_changes(),
        }
    }
}
