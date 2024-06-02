use std::fmt::{Display, Formatter};

pub struct Report {
    pub entries: Vec<ReportEntry>,
}

impl Report {
    pub fn new() -> Report {
        Report {
            entries: Vec::new(),
        }
    }
    
    pub fn differences(&self) -> Vec<&ReportEntry> {
        let predicate = |re: &&ReportEntry| -> bool {
            match re {
                ReportEntry::Addition { path: _, thing: _, } => true,
                ReportEntry::Removal { path: _, thing: _, } => true,
                ReportEntry::Change { path: _, left_value: _, right_value: _ } => true,
                ReportEntry::Match { path: _, left_value: _, right_value: _ } => false,
            }
        };
        
        self.entries.iter().filter(predicate).collect()
    }
}

#[derive(Debug)]
pub enum ReportEntry {
    Addition { path: Vec<Thing>, thing: Thing },
    Removal { path: Vec<Thing>, thing: Thing },
    Change { path: Vec<Thing>, left_value: String, right_value: String },
    Match { path: Vec<Thing>, left_value: String, right_value: String },
}

#[derive(Debug)]
pub enum Thing {
    Column(String),
    Constraint(String),
    Property(String),
    Routine(String),
    Schema(String),
    Sequence(String),
    Table(String),
    View(String),
}

impl Display for Thing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Thing::Column(name) => write!(f, "column '{}'", name),
            Thing::Constraint(name) => write!(f, "constraint '{}'", name),
            Thing::Property(name) => write!(f, "property '{}'", name),
            Thing::Routine(name) => write!(f, "routine '{}'", name),
            Thing::Schema(name) => write!(f, "schema '{}'", name),
            Thing::Sequence(name) => write!(f, "sequence '{}'", name),
            Thing::Table(name) => write!(f, "table '{}'", name),
            Thing::View(name) => write!(f, "view '{}'", name),
        }
    }
}
