use crate::compare::report::property::{PropertyComparison};
use crate::compare::report::{HasChanges, Report};

pub enum ViewComparison {
    ViewMaintained { view_name: String, properties: Report<PropertyComparison> },
}

impl HasChanges for ViewComparison {
    fn has_changes(&self) -> bool {
        match self {
            ViewComparison::ViewMaintained { view_name: _view_name, properties } =>
                properties.has_changes(),
        }
    }
}
