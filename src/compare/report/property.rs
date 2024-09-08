use crate::compare::report::HasChanges;

pub enum PropertyComparison {
    PropertyChanged { property_name: String, left_value: String, right_value: String },
    PropertyUnchanged { property_name: String, value: String },
}

impl HasChanges for PropertyComparison {
    fn has_changes(&self) -> bool {
        match self {
            PropertyComparison::PropertyChanged { .. } => true,
            PropertyComparison::PropertyUnchanged { .. } => false,
        }
    }
}
