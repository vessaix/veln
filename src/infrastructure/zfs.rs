use crate::config::Config;
use crate::domain::vm::{VirtualMachine, VmConfig, VmState};
use crate::domain::VmRepository;
use crate::error::{Result, VelnError};
use std::path::{Path, PathBuf};
use std::process::Command;

/// ZFS-based VM repository
/// Stores VMs as ZFS datasets with configuration files
pub struct ZfsRepository {
    pool: String,
    root_dataset: String,
}

impl ZfsRepository {
    /// Create a new ZFS repository from config
    /// # Errors
    /// Returns `VelnError::ZfsError` if the pool is not available
    pub fn new(config: &Config) -> Result<Self> {
        // Convert vm_root path to a valid ZFS dataset name
        // e.g., "/usr/local/vms" -> "usr/local/vms" (remove leading slash)
        let vm_root_clean = config.vm_root.trim_start_matches('/');
        let root_dataset = if vm_root_clean.is_empty() {
            config.zfs_pool.clone()
        } else {
            format!("{}/{}", config.zfs_pool, vm_root_clean)
        };
        
        let repo = Self {
            pool: config.zfs_pool.clone(),
            root_dataset,
        };

        // Ensure root dataset exists
        repo.ensure_dataset(&repo.root_dataset)?;

        Ok(repo)
    }

    fn dataset_path(&self, name: &str) -> String {
        format!("{}/{}", self.root_dataset, name)
    }

    fn mount_point(&self, name: &str) -> Result<PathBuf> {
        // Get the mount point from ZFS
        let output = Command::new("zfs")
            .args(["get", "-H", "-o", "value", "mountpoint", &self.dataset_path(name)])
            .output()
            .map_err(|e| VelnError::ZfsError(format!("Failed to get mountpoint: {e}")))?;

        if !output.status.success() {
            return Ok(PathBuf::from(format!(
                "/{}/{}/{}",
                self.pool, self.root_dataset, name
            )));
        }

        let mountpoint = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(mountpoint))
    }

    fn ensure_dataset(&self, dataset: &str) -> Result<()> {
        if self.dataset_exists(dataset) {
            return Ok(());
        }

        // Use -p flag to create parent datasets as needed
        let output = Command::new("zfs")
            .args(["create", "-p", dataset])
            .output()
            .map_err(|e| VelnError::ZfsError(format!("Failed to create dataset {dataset}: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to create dataset {dataset}: {stderr}"
            )));
        }

        Ok(())
    }

    fn dataset_exists(&self, dataset: &str) -> bool {
        let _ = self;
        Command::new("zfs")
            .args(["list", dataset])
            .output()
            .map_or_else(|_| false, |output| output.status.success())
    }

    fn save_config(&self, vm: &VirtualMachine, mount_point: &Path) -> Result<()> {
        let _ = self;
        let config_path = mount_point.join("veln.toml");
        let config = VmConfigData::from(vm);

        let toml = toml::to_string_pretty(&config)
            .map_err(|e| VelnError::Config(format!("Failed to serialize config: {e}")))?;

        std::fs::write(&config_path, toml)
            .map_err(VelnError::Io)?;

        Ok(())
    }

    fn load_config(&self, mount_point: &Path) -> Result<(crate::domain::vm::Uuid, VmConfig, VmState)> {
        let _ = self;
        let config_path = mount_point.join("veln.toml");

        if !config_path.exists() {
            return Err(VelnError::VmNotFound(
                format!("Config not found at {}", config_path.display())
            ));
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| VelnError::Config(format!("Failed to read config: {e}")))?;

        let data: VmConfigData = toml::from_str(&content)
            .map_err(|e| VelnError::Config(format!("Invalid config: {e}")))?;

        let (uuid, config) = data.into_vm_config();
        Ok((uuid, config, VmState::Stopped))
    }
}

impl VmRepository for ZfsRepository {
    /// # Errors
    /// Returns `VelnError::ZfsError` if dataset creation fails
    fn save(&self, vm: &VirtualMachine) -> Result<()> {
        let dataset = self.dataset_path(&vm.name);
        self.ensure_dataset(&dataset)?;

        let mount_point = self.mount_point(&vm.name)?;
        self.save_config(vm, &mount_point)?;

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::VmNotFound` if VM dataset does not exist
    fn load(&self, name: &str) -> Result<VirtualMachine> {
        let dataset = self.dataset_path(name);

        if !self.dataset_exists(&dataset) {
            return Err(VelnError::VmNotFound(name.to_string()));
        }

        let mount_point = self.mount_point(name)?;
        let (uuid, config, state) = self.load_config(&mount_point)?;

        let mut vm = VirtualMachine::with_uuid(uuid, name.to_string(), config);
        vm.state = state;

        Ok(vm)
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if dataset destruction fails
    fn delete(&self, name: &str) -> Result<()> {
        let dataset = self.dataset_path(name);

        if !self.dataset_exists(&dataset) {
            return Err(VelnError::VmNotFound(name.to_string()));
        }

        let output = Command::new("zfs")
            .args(["destroy", "-r", &dataset])
            .output()
            .map_err(|e| VelnError::ZfsError(format!("Failed to destroy dataset: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to destroy dataset: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if listing datasets fails
    fn list(&self) -> Result<Vec<VirtualMachine>> {
        let output = Command::new("zfs")
            .args(["list", "-H", "-r", "-o", "name", &self.root_dataset])
            .output()
            .map_err(|e| VelnError::ZfsError(format!("Failed to list datasets: {e}")))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut vms = Vec::new();

        for line in stdout.lines() {
            // Skip the root dataset itself
            if line == self.root_dataset {
                continue;
            }

            // Extract VM name from dataset path
            let prefix = format!("{}/", self.root_dataset);
            if let Some(name) = line.strip_prefix(&prefix) {
                // Only direct children (not nested datasets)
                if !name.contains('/') {
                    if let Ok(vm) = self.load(name) {
                        vms.push(vm);
                    }
                }
            }
        }

        Ok(vms)
    }

    fn exists(&self, name: &str) -> bool {
        self.dataset_exists(&self.dataset_path(name))
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if snapshot creation fails
    fn create_snapshot(&self, name: &str, snapshot_name: &str, comment: Option<&str>) -> Result<()> {
        let disk_dataset = format!("{}/disk", self.dataset_path(name));
        let full_snapshot_name = format!("{disk_dataset}@{snapshot_name}");

        // Check if snapshot already exists
        let check = Command::new("zfs")
            .args(["list", "-t", "snapshot", &full_snapshot_name])
            .output()
            .map_err(VelnError::Io)?;

        if check.status.success() {
            return Err(VelnError::ZfsError(format!(
                "Snapshot '{snapshot_name}' already exists for VM '{name}'"
            )));
        }

        // Create the snapshot
        let output = Command::new("zfs")
            .args(["snapshot", &full_snapshot_name])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to create snapshot: {stderr}"
            )));
        }

        // Store comment as a user property if provided
        if let Some(comment_text) = comment {
            let property_name = format!("veln:comment:{snapshot_name}");
            let _ = Command::new("zfs")
                .args(["set", &format!("{property_name}={comment_text}"), &disk_dataset])
                .output();
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if snapshots cannot be listed
    fn list_snapshots(&self, name: &str) -> Result<Vec<crate::domain::repository::Snapshot>> {
        let disk_dataset = format!("{}/disk", self.dataset_path(name));
        
        // Check if VM exists
        if !self.dataset_exists(&self.dataset_path(name)) {
            return Err(VelnError::VmNotFound(name.to_string()));
        }

        // List snapshots with creation time and size
        let output = Command::new("zfs")
            .args([
                "list",
                "-t", "snapshot",
                "-H", "-o", "name,creation,used",
                "-r", &disk_dataset,
            ])
            .output()
            .map_err(VelnError::Io)?;

        let mut snapshots = Vec::new();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    // Extract snapshot name from full path (dataset@snapshot)
                    if let Some(snapshot_part) = parts[0].split('@').nth(1) {
                        // Get comment if exists
                        let comment_prop = format!("veln:comment:{snapshot_part}");
                        let comment_output = Command::new("zfs")
                            .args(["get", "-H", "-o", "value", &comment_prop, &disk_dataset])
                            .output();
                        
                        let comment = match comment_output {
                            Ok(output) if output.status.success() => {
                                let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                if val == "-" { None } else { Some(val) }
                            }
                            _ => None,
                        };

                        // Parse size (convert from human-readable to bytes)
                        let size_str = parts[2].trim();
                        let size = Self::parse_size(size_str);

                        snapshots.push(crate::domain::repository::Snapshot {
                            name: snapshot_part.to_string(),
                            created: parts[1].to_string(),
                            comment,
                            size,
                        });
                    }
                }
            }
        }

        Ok(snapshots)
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if rollback fails
    fn rollback_snapshot(&self, name: &str, snapshot_name: &str, force: bool) -> Result<()> {
        let disk_dataset = format!("{}/disk", self.dataset_path(name));
        let full_snapshot_name = format!("{disk_dataset}@{snapshot_name}");

        // Check if snapshot exists
        let check = Command::new("zfs")
            .args(["list", "-t", "snapshot", &full_snapshot_name])
            .output()
            .map_err(VelnError::Io)?;

        if !check.status.success() {
            return Err(VelnError::VmNotFound(format!(
                "Snapshot '{snapshot_name}' not found for VM '{name}'"
            )));
        }

        // Rollback
        let mut args = vec!["rollback"];
        if force {
            args.push("-r"); // Destroy newer snapshots
        }
        args.push(&full_snapshot_name);

        let output = Command::new("zfs")
            .args(&args)
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to rollback snapshot: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::ZfsError` if snapshot deletion fails
    fn delete_snapshot(&self, name: &str, snapshot_name: &str) -> Result<()> {
        let disk_dataset = format!("{}/disk", self.dataset_path(name));
        let full_snapshot_name = format!("{disk_dataset}@{snapshot_name}");

        // Check if snapshot exists
        let check = Command::new("zfs")
            .args(["list", "-t", "snapshot", &full_snapshot_name])
            .output()
            .map_err(VelnError::Io)?;

        if !check.status.success() {
            return Err(VelnError::VmNotFound(format!(
                "Snapshot '{snapshot_name}' not found for VM '{name}'"
            )));
        }

        // Delete the snapshot
        let output = Command::new("zfs")
            .args(["destroy", &full_snapshot_name])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to delete snapshot: {stderr}"
            )));
        }

        Ok(())
    }

    fn clone_vm(&self, source: &str, target: &str, linked: bool) -> Result<()> {
        let source_dataset = self.dataset_path(source);
        let target_dataset = self.dataset_path(target);

        if !self.dataset_exists(&source_dataset) {
            return Err(VelnError::VmNotFound(source.to_string()));
        }

        if self.dataset_exists(&target_dataset) {
            return Err(VelnError::Config(format!(
                "Target VM '{target}' already exists"
            )));
        }

        if linked {
            // Create a snapshot for the clone
            let snapshot = format!("{source_dataset}@clone-{}", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs());
            
            let snap_output = Command::new("zfs")
                .args(["snapshot", &snapshot])
                .output()
                .map_err(VelnError::Io)?;

            if !snap_output.status.success() {
                return Err(VelnError::ZfsError("Failed to create snapshot for clone".to_string()));
            }

            // Clone the snapshot
            let output = Command::new("zfs")
                .args(["clone", &snapshot, &target_dataset])
                .output()
                .map_err(VelnError::Io)?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VelnError::ZfsError(format!(
                    "Failed to create linked clone: {stderr}"
                )));
            }
        } else {
            // Full copy using zfs send | recv
            let temp_snap = format!("{source_dataset}@temp-clone-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs());
            
            // Create temporary snapshot
            let snap_output = Command::new("zfs")
                .args(["snapshot", &temp_snap])
                .output()
                .map_err(VelnError::Io)?;

            if !snap_output.status.success() {
                return Err(VelnError::ZfsError("Failed to create temporary snapshot".to_string()));
            }

            // Send and receive
            let send_result = Command::new("sh")
                .args(["-c", &format!("zfs send '{temp_snap}' | zfs receive '{target_dataset}'")])
                .output()
                .map_err(VelnError::Io)?;

            // Clean up temp snapshot regardless of result
            let _ = Command::new("zfs")
                .args(["destroy", &temp_snap])
                .output();

            if !send_result.status.success() {
                let stderr = String::from_utf8_lossy(&send_result.stderr);
                // Clean up partial target if it exists
                let _ = Command::new("zfs")
                    .args(["destroy", "-r", &target_dataset])
                    .output();
                return Err(VelnError::ZfsError(format!(
                    "Failed to create full clone: {stderr}"
                )));
            }
        }

        // Update the config with new name and fresh UUID
        let mount_point = self.mount_point(target)?;
        let (_, config, state) = self.load_config(&mount_point)?;
        
        let mut vm = VirtualMachine::with_config(target.to_string(), config);
        vm.state = state;
        
        // Save updated config
        self.save_config(&vm, &mount_point)?;

        Ok(())
    }

    fn create_template(&self, vm: &str, name: &str, description: Option<&str>) -> Result<()> {
        let vm_dataset = self.dataset_path(vm);
        let template_dataset = format!("{}/{}/templates", self.pool, self.root_dataset);
        let target_dataset = format!("{template_dataset}/{name}");

        if !self.dataset_exists(&vm_dataset) {
            return Err(VelnError::VmNotFound(vm.to_string()));
        }

        // Ensure templates dataset exists
        if !self.dataset_exists(&template_dataset) {
            let output = Command::new("zfs")
                .args(["create", "-p", &template_dataset])
                .output()
                .map_err(VelnError::Io)?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VelnError::ZfsError(format!(
                    "Failed to create templates dataset: {stderr}"
                )));
            }
        }

        if self.dataset_exists(&target_dataset) {
            return Err(VelnError::Config(format!(
                "Template '{name}' already exists"
            )));
        }

        // Create a snapshot for the template
        let snapshot = format!("{vm_dataset}@template-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());

        let snap_output = Command::new("zfs")
            .args(["snapshot", &snapshot])
            .output()
            .map_err(VelnError::Io)?;

        if !snap_output.status.success() {
            return Err(VelnError::ZfsError("Failed to create template snapshot".to_string()));
        }

        // Clone to templates
        let clone_output = Command::new("zfs")
            .args(["clone", &snapshot, &target_dataset])
            .output()
            .map_err(VelnError::Io)?;

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to create template: {stderr}"
            )));
        }

        // Save template metadata
        let _mount_point = self.mount_point(vm)?;
        let template_meta = TemplateMetadata {
            name: name.to_string(),
            description: description.map(String::from),
            created: chrono::Local::now().to_rfc3339(),
            source_vm: vm.to_string(),
        };

        let meta_path = std::path::PathBuf::from(format!("/{}/{}/templates/{}.json", 
            self.pool, self.root_dataset, name));
        let json = serde_json::to_string_pretty(&template_meta)
            .map_err(|e| VelnError::Config(format!("Failed to serialize metadata: {e}")))?;
        
        std::fs::write(&meta_path, json)
            .map_err(VelnError::Io)?;

        Ok(())
    }

    fn list_templates(&self) -> Result<Vec<crate::domain::repository::Template>> {
        let template_dataset = format!("{}/{}/templates", self.pool, self.root_dataset);
        let mut templates = Vec::new();

        if !self.dataset_exists(&template_dataset) {
            return Ok(templates);
        }

        let meta_dir = std::path::PathBuf::from(format!("/{}/{}/templates", self.pool, self.root_dataset));
        
        if let Ok(entries) = std::fs::read_dir(&meta_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(meta) = serde_json::from_str::<TemplateMetadata>(&content) {
                            templates.push(crate::domain::repository::Template {
                                name: meta.name,
                                description: meta.description,
                                created: meta.created,
                            });
                        }
                    }
                }
            }
        }

        templates.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(templates)
    }

    fn deploy_template(&self, template: &str, vm: &str, linked: bool) -> Result<()> {
        let template_dataset = format!("{}/{}/templates/{}", self.pool, self.root_dataset, template);
        let target_dataset = self.dataset_path(vm);

        if !self.dataset_exists(&template_dataset) {
            return Err(VelnError::VmNotFound(format!(
                "Template '{template}' not found"
            )));
        }

        if self.dataset_exists(&target_dataset) {
            return Err(VelnError::Config(format!(
                "VM '{vm}' already exists"
            )));
        }

        if linked {
            // Create linked clone from template
            let snapshot = format!("{template_dataset}@template");
            
            // Check if template snapshot exists
            let snap_check = Command::new("zfs")
                .args(["list", "-t", "snapshot", &snapshot])
                .output()
                .map_err(VelnError::Io)?;

            if snap_check.status.success() {
                // Use existing template snapshot
                let output = Command::new("zfs")
                    .args(["clone", &snapshot, &target_dataset])
                    .output()
                    .map_err(VelnError::Io)?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(VelnError::ZfsError(format!(
                        "Failed to deploy linked clone: {stderr}"
                    )));
                }
            } else {
                // Create new snapshot
                let temp_snap = format!("{template_dataset}@deploy-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs());
                
                Command::new("zfs")
                    .args(["snapshot", &temp_snap])
                    .output()
                    .map_err(VelnError::Io)?;

                let output = Command::new("zfs")
                    .args(["clone", &temp_snap, &target_dataset])
                    .output()
                    .map_err(VelnError::Io)?;

                if !output.status.success() {
                    let _ = Command::new("zfs")
                        .args(["destroy", &temp_snap])
                        .output();
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(VelnError::ZfsError(format!(
                        "Failed to deploy linked clone: {stderr}"
                    )));
                }
            }
        } else {
            // Full copy from template
            let temp_snap = format!("{template_dataset}@deploy-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs());
            
            Command::new("zfs")
                .args(["snapshot", &temp_snap])
                .output()
                .map_err(VelnError::Io)?;

            let send_result = Command::new("sh")
                .args(["-c", &format!("zfs send '{temp_snap}' | zfs receive '{target_dataset}'")])
                .output()
                .map_err(VelnError::Io)?;

            let _ = Command::new("zfs")
                .args(["destroy", &temp_snap])
                .output();

            if !send_result.status.success() {
                let stderr = String::from_utf8_lossy(&send_result.stderr);
                let _ = Command::new("zfs")
                    .args(["destroy", "-r", &target_dataset])
                    .output();
                return Err(VelnError::ZfsError(format!(
                    "Failed to deploy full clone: {stderr}"
                )));
            }
        }

        // Update config with new name and UUID
        let mount_point = self.mount_point(vm)?;
        let (_, config, state) = self.load_config(&mount_point)?;
        
        let mut new_vm = VirtualMachine::with_config(vm.to_string(), config);
        new_vm.state = state;
        self.save_config(&new_vm, &mount_point)?;

        Ok(())
    }

    fn template_exists(&self, name: &str) -> bool {
        let template_dataset = format!("{}/{}/templates/{}", self.pool, self.root_dataset, name);
        self.dataset_exists(&template_dataset)
    }

    fn delete_template(&self, name: &str) -> Result<()> {
        let template_dataset = format!("{}/{}/templates/{}", self.pool, self.root_dataset, name);
        let meta_path = std::path::PathBuf::from(format!("/{}/{}/templates/{}.json", 
            self.pool, self.root_dataset, name));

        if !self.dataset_exists(&template_dataset) {
            return Err(VelnError::VmNotFound(format!(
                "Template '{name}' not found"
            )));
        }

        // Destroy the template dataset
        let output = Command::new("zfs")
            .args(["destroy", "-r", &template_dataset])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to delete template: {stderr}"
            )));
        }

        // Delete metadata file
        if meta_path.exists() {
            let _ = std::fs::remove_file(&meta_path);
        }

        Ok(())
    }
}

impl ZfsRepository {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn parse_size(size_str: &str) -> u64 {
        // Parse human-readable sizes like "1.2M", "3G", etc.
        let size_str = size_str.trim();
        if size_str == "-" || size_str.is_empty() {
            return 0;
        }

        // Extract numeric part and unit
        let numeric: String = size_str.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        let unit: String = size_str.chars().skip_while(|c| c.is_ascii_digit() || *c == '.').collect();

        let value: f64 = numeric.parse().unwrap_or(0.0);

        match unit.as_str() {
            "K" | "KiB" => (value * 1024.0) as u64,
            "M" | "MiB" => (value * 1024.0 * 1024.0) as u64,
            "G" | "GiB" => (value * 1024.0 * 1024.0 * 1024.0) as u64,
            "T" | "TiB" => (value * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => value as u64,
        }
    }
}

/// Serializable VM configuration stored in ZFS dataset
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct VmConfigData {
    uuid: String,
    name: String,
    cpus: u8,
    memory_mb: u64,
    disk_gb: u64,
    /// Network backend type: "tap" or "vale"
    #[serde(default = "default_network_type")]
    network_type: String,
    /// For tap: bridge name; for vale: switch name
    #[serde(default = "default_bridge")]
    network_device: String,
    mac: Option<String>,
}

fn default_network_type() -> String {
    String::from("tap")
}

fn default_bridge() -> String {
    String::from("bridge0")
}

impl From<&VirtualMachine> for VmConfigData {
    fn from(vm: &VirtualMachine) -> Self {
        let (network_type, network_device) = match &vm.config.network.backend {
            crate::domain::vm::NetworkBackend::TapBridge { bridge } => {
                (String::from("tap"), bridge.clone())
            }
            crate::domain::vm::NetworkBackend::Vale { switch } => {
                (String::from("vale"), switch.clone())
            }
        };

        Self {
            uuid: vm.uuid.to_string(),
            name: vm.name.clone(),
            cpus: vm.config.cpus,
            memory_mb: vm.config.memory_mb,
            disk_gb: vm.config.disk_gb,
            network_type,
            network_device,
            mac: vm.config.network.mac.clone(),
        }
    }
}

impl VmConfigData {
    fn into_vm_config(self) -> (crate::domain::vm::Uuid, VmConfig) {
        use crate::domain::vm::{NetworkBackend, NetworkConfig};

        let uuid = crate::domain::vm::Uuid(self.uuid);
        let backend = match self.network_type.as_str() {
            "vale" => NetworkBackend::Vale {
                switch: self.network_device,
            },
            _ => NetworkBackend::TapBridge {
                bridge: self.network_device,
            },
        };

        let config = VmConfig {
            cpus: self.cpus,
            memory_mb: self.memory_mb,
            disk_gb: self.disk_gb,
            network: NetworkConfig {
                backend,
                mac: self.mac,
            },
            console: crate::domain::vm::ConsoleType::Tty,
        };
        (uuid, config)
    }
}

/// ISO repository implementation
/// Stores ISO images in <pool>/images/
pub struct IsoRepository {
    #[allow(dead_code)]
    pool: String,
    images_dataset: String,
}

impl IsoRepository {
    /// Create a new image repository
    /// # Errors
    /// Returns `VelnError::ZfsError` if the dataset cannot be created
    pub fn new(config: &Config) -> Result<Self> {
        let repo = Self {
            pool: config.zfs_pool.clone(),
            images_dataset: format!("{}/images", config.zfs_pool),
        };

        // Ensure images dataset exists
        repo.ensure_dataset(&repo.images_dataset)?;

        Ok(repo)
    }

    #[allow(clippy::unused_self)]
    fn ensure_dataset(&self, dataset: &str) -> Result<()> {
        let check = Command::new("zfs")
            .args(["list", dataset])
            .output()
            .map_err(VelnError::Io)?;

        if !check.status.success() {
            // Create dataset
            let output = Command::new("zfs")
                .args(["create", dataset])
                .output()
                .map_err(VelnError::Io)?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VelnError::ZfsError(format!(
                    "Failed to create images dataset: {stderr}"
                )));
            }
        }

        Ok(())
    }

    fn mount_point(&self) -> Result<std::path::PathBuf> {
        let output = Command::new("zfs")
            .args(["get", "-H", "-o", "value", "mountpoint", &self.images_dataset])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            return Ok(std::path::PathBuf::from(format!("/{}", self.images_dataset)));
        }

        let mountpoint = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(std::path::PathBuf::from(mountpoint))
    }

    /// Download an ISO image from URL
    /// # Errors
    /// Returns error if download fails
    /// # Panics
    /// Panics if the filepath cannot be converted to a string (should not happen with valid UTF-8 paths)
    pub fn fetch(&self, name: &str, url: &str, description: Option<&str>) -> Result<()> {
        let mount_point = self.mount_point()?;
        let filename = format!("{name}.iso");
        let filepath = mount_point.join(&filename);
        let meta_path = mount_point.join(format!("{name}.json"));

        if filepath.exists() {
            return Err(VelnError::Config(format!(
                "Image '{name}' already exists"
            )));
        }

        println!("Downloading {url}...");
        
        // Download using curl
        let output = Command::new("curl")
            .args([
                "-L", "-o", 
                filepath.to_str().expect("filepath should be valid UTF-8"),
                url,
                "--progress-bar"
            ])
            .status()
            .map_err(VelnError::Io)?;

        if !output.success() {
            // Clean up partial download
            let _ = std::fs::remove_file(&filepath);
            return Err(VelnError::Io(std::io::Error::other(
                "Download failed"
            )));
        }

        // Get file size
        let metadata = std::fs::metadata(&filepath)
            .map_err(VelnError::Io)?;
        let size = metadata.len();

        // Save metadata
        let iso_meta = IsoMetadata {
            name: name.to_string(),
            filename,
            description: description.map(String::from),
            size,
            downloaded: chrono::Local::now().to_rfc3339(),
            url: url.to_string(),
        };

        let json = serde_json::to_string_pretty(&iso_meta)
            .map_err(|e| VelnError::Config(format!("Failed to serialize metadata: {e}")))?;
        
        std::fs::write(&meta_path, json)
            .map_err(VelnError::Io)?;

        #[allow(clippy::cast_precision_loss)]
        let size_mb = size as f64 / 1024.0 / 1024.0;
        println!("ISO '{name}' downloaded successfully ({size_mb:.1} MB)");

        Ok(())
    }

    /// List all ISOs
    /// # Errors
    /// Returns error if ISOs cannot be listed
    pub fn list(&self) -> Result<Vec<crate::domain::repository::Iso>> {
        let mount_point = self.mount_point()?;
        let mut isos = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&mount_point) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(meta) = serde_json::from_str::<IsoMetadata>(&content) {
                            isos.push(crate::domain::repository::Iso {
                                name: meta.name,
                                filename: meta.filename,
                                description: meta.description,
                                size: meta.size,
                                downloaded: meta.downloaded,
                            });
                        }
                    }
                }
            }
        }

        // Sort by name
        isos.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(isos)
    }

    /// Delete an image
    /// # Errors
    /// Returns error if image cannot be deleted
    pub fn delete(&self, name: &str) -> Result<()> {
        let mount_point = self.mount_point()?;
        let iso_path = mount_point.join(format!("{name}.iso"));
        let meta_path = mount_point.join(format!("{name}.json"));

        if !iso_path.exists() {
            return Err(VelnError::VmNotFound(format!(
                "Image '{name}' not found"
            )));
        }

        std::fs::remove_file(&iso_path)
            .map_err(VelnError::Io)?;
        
        if meta_path.exists() {
            let _ = std::fs::remove_file(&meta_path);
        }

        Ok(())
    }

    /// Get path to ISO file
    /// # Errors
    /// Returns error if image doesn't exist
    pub fn get_iso_path(&self, name: &str) -> Result<std::path::PathBuf> {
        let mount_point = self.mount_point()?;
        let iso_path = mount_point.join(format!("{name}.iso"));

        if !iso_path.exists() {
            return Err(VelnError::VmNotFound(format!(
                "Image '{name}' not found"
            )));
        }

        Ok(iso_path)
    }

    /// Check if image exists
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        if let Ok(mount_point) = self.mount_point() {
            mount_point.join(format!("{name}.iso")).exists()
        } else {
            false
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct IsoMetadata {
    name: String,
    filename: String,
    description: Option<String>,
    size: u64,
    downloaded: String,
    url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TemplateMetadata {
    name: String,
    description: Option<String>,
    created: String,
    source_vm: String,
}
