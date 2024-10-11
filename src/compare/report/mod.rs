pub mod schema;
pub mod property;
pub mod privilege;
pub mod routine;
pub mod sequence;
pub mod table;
pub mod column;
pub mod table_constraint;
pub mod table_trigger;
pub mod view;

pub struct Report<T: HasChanges> {
    pub entries: Vec<T>,
}

impl<T: HasChanges> Report<T> {
    fn has_changes(&self) -> bool {
        self.entries.iter().any(|e| e.has_changes())
    }
}

pub trait HasChanges {
    fn has_changes(&self) -> bool;
}
