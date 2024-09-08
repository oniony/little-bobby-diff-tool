use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};

pub enum SequenceComparison {
    SequenceAdded { sequence_name: String },
    SequenceRemoved { sequence_name: String },
    SequenceMaintained { sequence_name: String, properties: Report<PropertyComparison> },
}

impl HasChanges for SequenceComparison {
    fn has_changes(&self) -> bool {
        match self {
            SequenceComparison::SequenceAdded { .. } | SequenceComparison::SequenceRemoved { .. } => true,
            SequenceComparison::SequenceMaintained { sequence_name: _sequence_name, properties } =>
                properties.has_changes(),
        }
    }
}

