use crate::db::Database;

pub struct Comparer {
    left_db: Database,
    right_db: Database
}

impl Comparer {
    pub fn new(left_db : Database, right_db : Database) -> Comparer {
        Comparer{
            left_db,
            right_db
        }
    }

    pub fn compare(&mut self) -> bool {
        let mut same = true;

        same &= self.compare_catalog_name();
        
        same
    }

    fn compare_catalog_name(&mut self) -> bool {
        let left_name = self.left_db.catalog_name();
        let right_name = self.right_db.catalog_name();

        print!("comparing catalog names: left={}, right={}", left_name, right_name);

        left_name == right_name
    }
}