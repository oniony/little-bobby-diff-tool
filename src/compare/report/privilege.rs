use crate::compare::report::HasChanges;

pub enum PrivilegeComparison {
    PrivilegeAdded { privilege_name: String, grantor: String, grantee: String },
    PrivilegeRemoved { privilege_name: String, grantor: String, grantee: String },
    PrivilegeMaintained { privilege_name: String, grantor: String, grantee: String }
}

impl HasChanges for PrivilegeComparison {
    fn has_changes(&self) -> bool {
        match self {
            PrivilegeComparison::PrivilegeAdded { .. } | PrivilegeComparison::PrivilegeRemoved { .. } => true,
            PrivilegeComparison::PrivilegeMaintained { .. } => false,
        }
    }
}
