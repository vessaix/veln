use crate::domain::vm::{VirtualMachine, VmConfig};
use crate::error::Result;

/// Snapshot metadata
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub name: String,
    pub created: String,
    pub comment: Option<String>,
    pub size: u64, // Size in bytes
}

/// ISO Image metadata
#[derive(Debug, Clone)]
pub struct Iso {
    pub name: String,
    pub filename: String,
    pub description: Option<String>,
    pub size: u64,         // Size in bytes
    pub downloaded: String,
}

/// Template metadata
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: Option<String>,
    pub created: String,
}

/// Repository trait for VM persistence
/// Implemented by `ZfsRepository` (infrastructure layer)
pub trait VmRepository: Send + Sync {
    /// Save VM configuration
    /// # Errors
    /// Returns error if VM cannot be persisted
    fn save(&self, vm: &VirtualMachine) -> Result<()>;

    /// Load VM by name
    /// # Errors
    /// Returns `VelnError::VmNotFound` if VM does not exist
    fn load(&self, name: &str) -> Result<VirtualMachine>;

    /// Delete VM
    /// # Errors
    /// Returns error if VM cannot be deleted
    fn delete(&self, name: &str) -> Result<()>;

    /// List all VMs
    /// # Errors
    /// Returns error if VM list cannot be retrieved
    fn list(&self) -> Result<Vec<VirtualMachine>>;

    /// Check if VM exists
    fn exists(&self, name: &str) -> bool;

    /// Create a snapshot of VM disk
    /// # Errors
    /// Returns error if snapshot cannot be created
    fn create_snapshot(&self, name: &str, snapshot_name: &str, comment: Option<&str>) -> Result<()>;

    /// List snapshots for a VM
    /// # Errors
    /// Returns error if snapshots cannot be listed
    fn list_snapshots(&self, name: &str) -> Result<Vec<Snapshot>>;

    /// Rollback VM to a snapshot
    /// # Errors
    /// Returns error if rollback fails
    fn rollback_snapshot(&self, name: &str, snapshot_name: &str, force: bool) -> Result<()>;

    /// Delete a snapshot
    /// # Errors
    /// Returns error if snapshot cannot be deleted
    fn delete_snapshot(&self, name: &str, snapshot_name: &str) -> Result<()>;

    /// Clone a VM (full or linked clone)
    /// # Errors
    /// Returns error if clone fails
    fn clone_vm(&self, source: &str, target: &str, linked: bool) -> Result<()>;

    /// Create a template from a VM
    /// # Errors
    /// Returns error if template creation fails
    fn create_template(&self, vm: &str, name: &str, description: Option<&str>) -> Result<()>;

    /// List all templates
    /// # Errors
    /// Returns error if templates cannot be listed
    fn list_templates(&self) -> Result<Vec<Template>>;

    /// Deploy a VM from a template
    /// # Errors
    /// Returns error if deployment fails
    fn deploy_template(&self, template: &str, vm: &str, linked: bool) -> Result<()>;

    /// Check if template exists
    fn template_exists(&self, name: &str) -> bool;

    /// Delete a template
    /// # Errors
    /// Returns error if template cannot be deleted
    fn delete_template(&self, name: &str) -> Result<()>;
}

/// Runtime trait for VM lifecycle operations
/// Implemented by `BhyveRuntime` (infrastructure layer)
pub trait VmRuntime: Send + Sync {
    /// Create VM storage and configuration
    /// # Errors
    /// Returns error if VM cannot be created
    fn create(&self, name: &str, config: &VmConfig) -> Result<()>;

    /// Start VM
    /// # Errors
    /// Returns error if VM cannot be started
    fn start(&self, vm: &VirtualMachine) -> Result<()>;

    /// Stop VM gracefully
    /// # Errors
    /// Returns error if VM cannot be stopped
    fn stop(&self, name: &str) -> Result<()>;

    /// Force stop VM
    /// # Errors
    /// Returns error if VM cannot be destroyed
    fn destroy(&self, name: &str) -> Result<()>;

    /// Get VM status
    /// # Errors
    /// Returns error if status cannot be determined
    fn status(&self, name: &str) -> Result<bool>;
}
