//! Cloud-init integration for VM initialization
//!
//! Cloud-init is the standard for cloud instance initialization.
//! It supports user-data, meta-data, and network-config for automated VM setup.
//!
//! References:
//! - Cloud-init docs: <https://cloudinit.readthedocs.io/>
//! - FreeBSD cloud-init: <https://github.com/canonical/cloud-init>

use crate::config::Config;
use crate::error::{Result, VelnError};
use std::path::PathBuf;

/// Cloud-init configuration for a VM
#[derive(Debug, Clone)]
pub struct CloudInitConfig {
    /// Hostname for the VM
    pub hostname: String,
    /// SSH public keys to add
    pub ssh_authorized_keys: Vec<String>,
    /// User creation configuration
    pub users: Vec<CloudInitUser>,
    /// Network configuration
    pub network_config: Option<String>,
    /// Custom user-data scripts
    pub runcmd: Vec<String>,
    /// Package installation
    pub packages: Vec<String>,
    /// Write files to the VM
    pub write_files: Vec<WriteFile>,
}

impl Default for CloudInitConfig {
    fn default() -> Self {
        Self {
            hostname: String::from("veln-vm"),
            ssh_authorized_keys: Vec::new(),
            users: Vec::new(),
            network_config: None,
            runcmd: Vec::new(),
            packages: Vec::new(),
            write_files: Vec::new(),
        }
    }
}

/// Cloud-init user configuration
#[derive(Debug, Clone)]
pub struct CloudInitUser {
    pub name: String,
    pub sudo: Option<String>,
    pub ssh_authorized_keys: Vec<String>,
    pub shell: String,
    pub groups: Vec<String>,
}

/// File to write via cloud-init
#[derive(Debug, Clone)]
pub struct WriteFile {
    pub path: String,
    pub content: String,
    pub permissions: Option<String>,
    pub owner: Option<String>,
}

/// Cloud-init seed generator
/// Creates ISO images with cloud-init data for bhyve VMs
pub struct CloudInitSeed {
    seed_dir: PathBuf,
}

impl CloudInitSeed {
    /// Create a new cloud-init seed generator
    /// # Errors
    /// Returns error if directory creation fails
    pub fn new(config: &Config) -> Result<Self> {
        let seed_dir = PathBuf::from(format!("/{}/{}/cloud-init", config.zfs_pool, config.vm_root));
        
        // Ensure directory exists
        std::fs::create_dir_all(&seed_dir)
            .map_err(VelnError::Io)?;
        
        Ok(Self { seed_dir })
    }

    /// Generate cloud-init seed ISO for a VM
    /// # Errors
    /// Returns error if ISO creation fails
    pub fn generate_seed(&self, vm_name: &str, config: &CloudInitConfig) -> Result<PathBuf> {
        let vm_seed_dir = self.seed_dir.join(vm_name);
        
        // Clean up and recreate
        if vm_seed_dir.exists() {
            let _ = std::fs::remove_dir_all(&vm_seed_dir);
        }
        std::fs::create_dir_all(&vm_seed_dir)
            .map_err(VelnError::Io)?;

        // Generate user-data
        let user_data = Self::generate_user_data(config);
        std::fs::write(vm_seed_dir.join("user-data"), user_data)
            .map_err(VelnError::Io)?;

        // Generate meta-data
        let meta_data = Self::generate_meta_data(vm_name, config);
        std::fs::write(vm_seed_dir.join("meta-data"), meta_data)
            .map_err(VelnError::Io)?;

        // Generate network-config if provided
        if let Some(net_config) = &config.network_config {
            std::fs::write(vm_seed_dir.join("network-config"), net_config)
                .map_err(VelnError::Io)?;
        }

        // Create ISO
        let iso_path = self.seed_dir.join(format!("{vm_name}-seed.iso"));
        let output = std::process::Command::new("mkisofs")
            .args([
                "-o", iso_path.to_str().unwrap_or(&format!("{vm_name}-seed.iso")),
                "-V", "cidata",
                "-J", "-r",
                "-input-charset", "utf-8",
                vm_seed_dir.to_str().unwrap_or(&vm_seed_dir.display().to_string()),
            ])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    VelnError::Config(
                        "mkisofs not found. Install cdrtools package.".to_string()
                    )
                } else {
                    VelnError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::Config(format!(
                "Failed to create cloud-init seed ISO: {stderr}"
            )));
        }

        // Clean up temp files
        let _ = std::fs::remove_dir_all(&vm_seed_dir);

        Ok(iso_path)
    }

    /// Remove seed ISO for a VM
    /// # Errors
    /// Returns error if removal fails
    pub fn remove_seed(&self, vm_name: &str) -> Result<()> {
        let iso_path = self.seed_dir.join(format!("{vm_name}-seed.iso"));
        if iso_path.exists() {
            std::fs::remove_file(&iso_path)
                .map_err(VelnError::Io)?;
        }
        Ok(())
    }

    fn generate_user_data(config: &CloudInitConfig) -> String {
        let mut lines = vec![
            "#cloud-config".to_string(),
            format!("hostname: {}", config.hostname),
            "manage_etc_hosts: true".to_string(),
        ];

        // Users
        if !config.users.is_empty() {
            lines.push("users:".to_string());
            for user in &config.users {
                lines.push(format!("  - name: {}", user.name));
                lines.push(format!("    shell: {}", user.shell));
                if !user.groups.is_empty() {
                    lines.push(format!("    groups: {}", user.groups.join(", ")));
                }
                if let Some(sudo) = &user.sudo {
                    lines.push(format!("    sudo: {sudo}"));
                }
                if !user.ssh_authorized_keys.is_empty() {
                    lines.push("    ssh_authorized_keys:".to_string());
                    for key in &user.ssh_authorized_keys {
                        lines.push(format!("      - {key}"));
                    }
                }
            }
        }

        // SSH authorized keys for default user
        if !config.ssh_authorized_keys.is_empty() {
            lines.push("ssh_authorized_keys:".to_string());
            for key in &config.ssh_authorized_keys {
                lines.push(format!("  - {key}"));
            }
        }

        // Packages
        if !config.packages.is_empty() {
            lines.push("packages:".to_string());
            for pkg in &config.packages {
                lines.push(format!("  - {pkg}"));
            }
            lines.push("package_update: true".to_string());
        }

        // Write files
        if !config.write_files.is_empty() {
            lines.push("write_files:".to_string());
            for file in &config.write_files {
                lines.push(format!("  - path: {}", file.path));
                if let Some(perm) = &file.permissions {
                    lines.push(format!("    permissions: '{perm}'"));
                }
                if let Some(owner) = &file.owner {
                    lines.push(format!("    owner: {owner}"));
                }
                lines.push("    content: |".to_string());
                for line in file.content.lines() {
                    lines.push(format!("      {line}"));
                }
            }
        }

        // Run commands
        if !config.runcmd.is_empty() {
            lines.push("runcmd:".to_string());
            for cmd in &config.runcmd {
                lines.push(format!("  - {cmd}"));
            }
        }

        lines.join("\n")
    }

    fn generate_meta_data(vm_name: &str, config: &CloudInitConfig) -> String {
        format!(r"instance-id: {vm_name}
local-hostname: {}",
            config.hostname
        )
    }

    /// Generate a basic network config for DHCP
    #[must_use]
    pub fn generate_dhcp_network_config() -> String {
        r"version: 2
ethernets:
  nics:
    match:
      driver: virtio_net
    dhcp4: true
".to_string()
    }

    /// Generate a static IP network config
    #[must_use]
    pub fn generate_static_network_config(
        interface: &str,
        ip: &str,
        gateway: &str,
        dns: &[String],
    ) -> String {
        let dns_str = dns.join(", ");
        format!(r"version: 2
ethernets:
  {interface}:
    addresses:
      - {ip}
    gateway4: {gateway}
    nameservers:
      addresses:
        - {dns_str}
",
        )
    }
}

impl CloudInitUser {
    /// Create a standard admin user with sudo access
    #[must_use]
    pub fn admin_user(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sudo: Some("ALL=(ALL) NOPASSWD:ALL".to_string()),
            ssh_authorized_keys: Vec::new(),
            shell: "/bin/sh".to_string(),
            groups: vec!["wheel".to_string()],
        }
    }
}
