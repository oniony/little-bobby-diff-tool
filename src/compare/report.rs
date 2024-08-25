pub trait HasChanges {
    fn has_changes(&self) -> bool;
}

pub struct SchemaReport {
    pub entries: Vec<SchemaComparison>
}

pub enum SchemaComparison {
    SchemaAdded { schema_name: String },
    SchemaRemoved { schema_name: String },
    SchemaMissing { schema_name: String },
    SchemaMaintained { schema_name: String, properties: PropertyReport, routines: RoutineReport, sequences: SequenceReport, tables: TableReport, views: ViewReport },
}

impl HasChanges for SchemaReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|s| s.has_changes())
    }
}

impl HasChanges for SchemaComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            SchemaComparison::SchemaAdded { .. } | SchemaComparison::SchemaRemoved { .. } | SchemaComparison::SchemaMissing { .. } => true,
            SchemaComparison::SchemaMaintained { schema_name: _schema_name, properties, routines, sequences, tables, views } =>
                properties.has_changes() ||
                routines.has_changes() ||
                sequences.has_changes() ||
                tables.has_changes() ||
                views.has_changes(),
        }
    }
}

pub struct PropertyReport {
    pub entries: Vec<PropertyComparison>
}

pub enum PropertyComparison {
    PropertyChanged { property_name: String, left_value: String, right_value: String },
    PropertyUnchanged { property_name: String, value: String },
}

impl HasChanges for PropertyReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|p| p.has_changes())
    }
}

impl HasChanges for PropertyComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            PropertyComparison::PropertyChanged { .. } => true,
            PropertyComparison::PropertyUnchanged { .. } => false,
        }
    }
 }

pub struct PrivilegeReport {
    pub entries: Vec<PrivilegeComparison>
}

pub enum PrivilegeComparison {
    PrivilegeAdded { privilege_name: String, grantor: String, grantee: String },
    PrivilegeRemoved { privilege_name: String, grantor: String, grantee: String },
    PrivilegeMaintained { privilege_name: String, grantor: String, grantee: String }
}

impl HasChanges for PrivilegeReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|p| p.has_changes())
    }
}

impl HasChanges for PrivilegeComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            PrivilegeComparison::PrivilegeAdded { .. } | PrivilegeComparison::PrivilegeRemoved { .. } => true,
            PrivilegeComparison::PrivilegeMaintained { .. } => false,
        }
    }
}

pub struct RoutineReport {
    pub entries: Vec<RoutineComparison>
}

pub enum RoutineComparison {
    RoutineAdded { routine_signature: String },
    RoutineRemoved { routine_signature: String },
    RoutineMaintained { routine_signature: String, properties: PropertyReport, privileges: PrivilegeReport },
}

impl HasChanges for RoutineReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|r| r.has_changes())
    }
}

impl HasChanges for RoutineComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            RoutineComparison::RoutineAdded { .. } | RoutineComparison::RoutineRemoved { .. } => true,
            RoutineComparison::RoutineMaintained { routine_signature: _routine_signature, properties, privileges } =>
                properties.has_changes() ||
                privileges.has_changes(),
        }
    }
}

pub struct SequenceReport {
    pub entries: Vec<SequenceComparison>
}

pub enum SequenceComparison {
    SequenceAdded { sequence_name: String },
    SequenceRemoved { sequence_name: String },
    SequenceMaintained { sequence_name: String, properties: PropertyReport },
}

impl HasChanges for SequenceReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|s| s.has_changes())
    }
}

impl HasChanges for SequenceComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            SequenceComparison::SequenceAdded { .. } | SequenceComparison::SequenceRemoved { .. } => true,
            SequenceComparison::SequenceMaintained { sequence_name: _sequence_name, properties } => 
                properties.has_changes(),
        }
    }
}

pub struct TableReport {
    pub entries: Vec<TableComparison>
}

pub enum TableComparison {
    TableAdded { table_name: String },
    TableRemoved { table_name: String },
    TableMaintained { table_name: String, properties: PropertyReport, columns: TableColumnReport, privileges: PrivilegeReport, constraints: TableConstraintReport, triggers: TableTriggerReport },
}

impl HasChanges for TableReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|t| t.has_changes())
    }
}

impl HasChanges for TableComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            TableComparison::TableAdded { .. } | TableComparison::TableRemoved { .. } => true,
            TableComparison::TableMaintained { table_name: _table_name, properties, columns, privileges, constraints, triggers } =>
                properties.has_changes() ||
                columns.has_changes() ||
                privileges.has_changes() ||
                constraints.has_changes() ||
                triggers.has_changes(),
        }
    }
}

pub struct TableColumnReport {
    pub entries: Vec<TableColumnComparison>
}

pub enum TableColumnComparison {
    ColumnAdded { column_name: String },
    ColumnRemoved { column_name: String },
    ColumnMaintained { column_name: String, properties: PropertyReport, privileges: PrivilegeReport }
}

impl HasChanges for TableColumnReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|c| c.has_changes())
    }
}

impl HasChanges for TableColumnComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            TableColumnComparison::ColumnAdded { .. } | TableColumnComparison::ColumnRemoved { .. } => true,
            TableColumnComparison::ColumnMaintained { column_name: _column_name, properties, privileges } =>
                properties.has_changes() |
                privileges.has_changes(),
        }
    }
}

pub struct TableConstraintReport {
    pub entries: Vec<TableConstraintComparison>
}

pub enum TableConstraintComparison {
    ConstraintAdded { constraint_name: String },
    ConstraintRemoved { constraint_name: String },
    ConstraintMaintained { constraint_name: String, properties: PropertyReport },
}

impl HasChanges for TableConstraintReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|c| c.has_changes())
    }
}

impl HasChanges for TableConstraintComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            TableConstraintComparison::ConstraintAdded { .. } | TableConstraintComparison::ConstraintRemoved { .. } => true,
            TableConstraintComparison::ConstraintMaintained { constraint_name: _constraint_name, properties } =>
                properties.has_changes(),
        }
    }
}

pub struct TableTriggerReport {
    pub entries: Vec<TableTriggerComparison>
}

pub enum TableTriggerComparison {
    TriggerAdded { trigger_name: String, event_manipulation: String },
    TriggerRemoved { trigger_name: String, event_manipulation: String },
    TriggerMaintained { trigger_name: String, event_manipulation: String, properties: PropertyReport }
}

impl HasChanges for TableTriggerReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|t| t.has_changes())
    }
}

impl HasChanges for TableTriggerComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            TableTriggerComparison::TriggerAdded { .. } | TableTriggerComparison::TriggerRemoved { .. } => true,
            TableTriggerComparison::TriggerMaintained { trigger_name: _trigger_name, event_manipulation: _event_manipulation, properties } => 
                properties.has_changes(),
        }
    }
}

pub struct ViewReport {
    pub entries: Vec<ViewComparison>
}

pub enum ViewComparison {
    ViewMaintained { view_name: String, properties: PropertyReport },
}

impl HasChanges for ViewReport {
    fn has_changes(self: &Self) -> bool {
        self.entries.iter().any(|v| v.has_changes())
    }
}

impl HasChanges for ViewComparison {
    fn has_changes(self: &Self) -> bool {
        match self {
            ViewComparison::ViewMaintained { view_name: _view_name, properties } =>
                properties.has_changes(),
        }
    }
}

