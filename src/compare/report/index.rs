use crate::compare::report::property::PropertyComparison;
use crate::compare::report::{HasChanges, Report};

pub enum IndexComparison {
    IndexAdded { index_name: String },
    IndexRemoved { index_name: String },
    IndexMaintained { index_name: String, properties: Report<PropertyComparison> }
}

impl HasChanges for IndexComparison {
    fn has_changes(&self) -> bool {
        match self {
            IndexComparison::IndexAdded { .. } | IndexComparison::IndexRemoved { .. } => true,
            IndexComparison::IndexMaintained { index_name: _index_name, properties } => properties.has_changes(),
        }
    }
}
