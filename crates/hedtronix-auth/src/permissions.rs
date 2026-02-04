//! Permission checking for RBAC

use hedtronix_core::UserRole;
use serde::{Deserialize, Serialize};

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }
}

/// Permission checker for RBAC
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if a role has permission for an action on a resource
    pub fn has_permission(role: UserRole, resource: &str, action: &str) -> bool {
        match role {
            UserRole::Admin => true, // Admin has all permissions
            UserRole::Physician => Self::physician_permissions(resource, action),
            UserRole::Nurse => Self::nurse_permissions(resource, action),
            UserRole::Receptionist => Self::receptionist_permissions(resource, action),
            UserRole::Billing => Self::billing_permissions(resource, action),
            UserRole::Patient => Self::patient_permissions(resource, action),
        }
    }

    fn physician_permissions(resource: &str, action: &str) -> bool {
        matches!(
            (resource, action),
            ("patients", "read" | "write" | "create" | "list")
                | ("appointments", "read" | "write" | "create" | "list" | "cancel")
                | ("clinical_notes", "read" | "write" | "create" | "sign" | "list")
                | ("encounters", "read" | "write" | "create" | "list")
                | ("prescriptions", "read" | "write" | "create" | "sign")
                | ("billing", "read" | "list")
                | ("reports", "read")
                | ("users", "read")
                | ("sync", "push" | "pull")
        )
    }

    fn nurse_permissions(resource: &str, action: &str) -> bool {
        matches!(
            (resource, action),
            ("patients", "read" | "write" | "list")
                | ("appointments", "read" | "write" | "list")
                | ("clinical_notes", "read" | "write" | "list")
                | ("encounters", "read" | "write" | "list")
                | ("vitals", "read" | "write" | "create")
                | ("medication_administration", "read" | "write" | "create")
                | ("billing", "read" | "list")
                | ("users", "read")
                | ("sync", "push" | "pull")
        )
    }

    fn receptionist_permissions(resource: &str, action: &str) -> bool {
        matches!(
            (resource, action),
            ("patients", "read" | "write" | "create" | "list")
                | ("appointments", "read" | "write" | "create" | "cancel" | "check_in" | "list")
                | ("billing", "read" | "create_charges" | "list")
                | ("clinical_notes", "read")
                | ("users", "read")
                | ("rooms", "read" | "list")
                | ("sync", "push" | "pull")
        )
    }

    fn billing_permissions(resource: &str, action: &str) -> bool {
        matches!(
            (resource, action),
            ("patients", "read" | "list")
                | ("appointments", "read" | "list")
                | ("clinical_notes", "read" | "list")
                | ("encounters", "read" | "list")
                | ("billing", "read" | "write" | "create" | "submit" | "adjust" | "list")
                | ("reports", "read_financial")
                | ("users", "read")
                | ("sync", "push" | "pull")
        )
    }

    fn patient_permissions(resource: &str, action: &str) -> bool {
        matches!(
            (resource, action),
            ("own_data", "read")
                | ("appointments", "read_own" | "create_own" | "cancel_own")
                | ("clinical_notes", "read_own")
                | ("billing", "read_own" | "pay")
                | ("messages", "read" | "create")
        )
    }

    /// Get all permissions for a role
    pub fn get_permissions(role: UserRole) -> Vec<Permission> {
        match role {
            UserRole::Admin => vec![Permission::new("*", "*")],
            UserRole::Physician => vec![
                Permission::new("patients", "read"),
                Permission::new("patients", "write"),
                Permission::new("patients", "create"),
                Permission::new("patients", "list"),
                Permission::new("appointments", "read"),
                Permission::new("appointments", "write"),
                Permission::new("appointments", "create"),
                Permission::new("appointments", "list"),
                Permission::new("appointments", "cancel"),
                Permission::new("clinical_notes", "read"),
                Permission::new("clinical_notes", "write"),
                Permission::new("clinical_notes", "create"),
                Permission::new("clinical_notes", "sign"),
                Permission::new("clinical_notes", "list"),
                Permission::new("encounters", "read"),
                Permission::new("encounters", "write"),
                Permission::new("encounters", "create"),
                Permission::new("encounters", "list"),
                Permission::new("billing", "read"),
                Permission::new("billing", "list"),
            ],
            UserRole::Nurse => vec![
                Permission::new("patients", "read"),
                Permission::new("patients", "write"),
                Permission::new("patients", "list"),
                Permission::new("appointments", "read"),
                Permission::new("appointments", "write"),
                Permission::new("appointments", "list"),
                Permission::new("clinical_notes", "read"),
                Permission::new("clinical_notes", "write"),
                Permission::new("clinical_notes", "list"),
                Permission::new("vitals", "read"),
                Permission::new("vitals", "write"),
                Permission::new("vitals", "create"),
            ],
            UserRole::Receptionist => vec![
                Permission::new("patients", "read"),
                Permission::new("patients", "write"),
                Permission::new("patients", "create"),
                Permission::new("patients", "list"),
                Permission::new("appointments", "read"),
                Permission::new("appointments", "write"),
                Permission::new("appointments", "create"),
                Permission::new("appointments", "cancel"),
                Permission::new("appointments", "check_in"),
                Permission::new("appointments", "list"),
                Permission::new("billing", "read"),
                Permission::new("billing", "create_charges"),
                Permission::new("billing", "list"),
            ],
            UserRole::Billing => vec![
                Permission::new("patients", "read"),
                Permission::new("patients", "list"),
                Permission::new("appointments", "read"),
                Permission::new("appointments", "list"),
                Permission::new("billing", "read"),
                Permission::new("billing", "write"),
                Permission::new("billing", "create"),
                Permission::new("billing", "submit"),
                Permission::new("billing", "adjust"),
                Permission::new("billing", "list"),
            ],
            UserRole::Patient => vec![
                Permission::new("own_data", "read"),
                Permission::new("appointments", "read_own"),
                Permission::new("appointments", "create_own"),
                Permission::new("appointments", "cancel_own"),
                Permission::new("billing", "read_own"),
                Permission::new("billing", "pay"),
            ],
        }
    }
}

/// Department-scoped permission check
pub fn check_department_access(
    user_department_id: Option<uuid::Uuid>,
    resource_department_id: Option<uuid::Uuid>,
    role: UserRole,
) -> bool {
    // Admin can access all departments
    if role == UserRole::Admin {
        return true;
    }

    // If no department scoping, allow
    match (user_department_id, resource_department_id) {
        (None, _) => true, // User has no department restriction
        (_, None) => true, // Resource has no department
        (Some(user_dept), Some(resource_dept)) => user_dept == resource_dept,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_permissions() {
        assert!(PermissionChecker::has_permission(UserRole::Admin, "anything", "anything"));
    }

    #[test]
    fn test_physician_permissions() {
        assert!(PermissionChecker::has_permission(UserRole::Physician, "patients", "write"));
        assert!(PermissionChecker::has_permission(UserRole::Physician, "clinical_notes", "sign"));
        assert!(!PermissionChecker::has_permission(UserRole::Physician, "billing", "adjust"));
    }

    #[test]
    fn test_nurse_permissions() {
        assert!(PermissionChecker::has_permission(UserRole::Nurse, "vitals", "create"));
        assert!(!PermissionChecker::has_permission(UserRole::Nurse, "clinical_notes", "sign"));
    }

    #[test]
    fn test_receptionist_permissions() {
        assert!(PermissionChecker::has_permission(UserRole::Receptionist, "appointments", "check_in"));
        assert!(!PermissionChecker::has_permission(UserRole::Receptionist, "clinical_notes", "write"));
    }
}
