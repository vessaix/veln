use crate::error::{Result, VelnError};
use std::process::Command;

/// Host resource information
#[derive(Debug, Clone)]
pub struct HostResources {
    /// Total system memory in MB
    pub total_memory_mb: u64,
    /// Available memory in MB
    pub available_memory_mb: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Available CPU cores (not in use by other VMs)
    pub available_cores: usize,
    /// Total disk space in GB (ZFS pool)
    pub total_disk_gb: u64,
    /// Available disk space in GB
    pub available_disk_gb: u64,
}

/// Resource monitor for host system
/// Prevents VM creation from exhausting resources
pub struct ResourceMonitor;

impl ResourceMonitor {
    /// Get current host resource status
    /// # Errors
    /// Returns `VelnError::HostRequirements` if system information cannot be read
    pub fn get_resources() -> Result<HostResources> {
        let total_memory_mb = Self::get_total_memory()?;
        let available_memory_mb = Self::get_available_memory()?;
        let cpu_cores = Self::get_cpu_cores()?;
        let (total_disk_gb, available_disk_gb) = Self::get_disk_space()?;

        // Calculate available cores (simplified - in production would check running VMs)
        let available_cores = cpu_cores;

        Ok(HostResources {
            total_memory_mb,
            available_memory_mb,
            cpu_cores,
            available_cores,
            total_disk_gb,
            available_disk_gb,
        })
    }

    /// Check if VM can be created with given requirements
    /// # Errors
    /// Returns `VelnError::InsufficientResources` if resources are insufficient
    pub fn can_create_vm(required_memory_mb: u64, required_disk_gb: u64, required_cores: u8) -> Result<()> {
        let resources = Self::get_resources()?;

        // Reserve 512MB for host OS
        let min_host_memory_mb = 512;
        let usable_memory = resources.available_memory_mb.saturating_sub(min_host_memory_mb);

        if required_memory_mb > usable_memory {
            return Err(VelnError::InsufficientResources(format!(
                "Insufficient memory: required {required_memory_mb}MB, available {usable_memory}MB (keeping {min_host_memory_mb}MB for host)"
            )));
        }

        // Reserve 10% of disk for ZFS overhead
        let min_disk_buffer_gb = (resources.total_disk_gb * 10) / 100;
        let usable_disk = resources.available_disk_gb.saturating_sub(min_disk_buffer_gb);

        if required_disk_gb > usable_disk {
            return Err(VelnError::InsufficientResources(format!(
                "Insufficient disk space: required {required_disk_gb}GB, available {usable_disk}GB (keeping {min_disk_buffer_gb}GB buffer)"
            )));
        }

        // Reserve 1 core for host
        let min_host_cores = 1;
        let usable_cores = resources.available_cores.saturating_sub(min_host_cores);

        if usize::from(required_cores) > usable_cores {
            return Err(VelnError::InsufficientResources(format!(
                "Insufficient CPU cores: required {required_cores}, available {usable_cores} (keeping {min_host_cores} for host)"
            )));
        }

        Ok(())
    }

    fn get_total_memory() -> Result<u64> {
        let output = Command::new("sysctl")
            .args(["-n", "hw.physmem"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to get memory: {e}")))?;

        let bytes: u64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid memory value: {e}")))?;

        Ok(bytes / 1024 / 1024) // Convert to MB
    }

    fn get_available_memory() -> Result<u64> {
        // Use vm.stats.vm.v_free_count * page size
        let output = Command::new("sysctl")
            .args(["-n", "vm.stats.vm.v_free_count"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to get free memory: {e}")))?;

        let free_pages: u64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid free memory value: {e}")))?;

        // Get page size
        let output = Command::new("sysctl")
            .args(["-n", "hw.pagesize"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to get page size: {e}")))?;

        let page_size: u64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid page size: {e}")))?;

        Ok((free_pages * page_size) / 1024 / 1024) // Convert to MB
    }

    fn get_cpu_cores() -> Result<usize> {
        let output = Command::new("sysctl")
            .args(["-n", "hw.ncpu"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to get CPU count: {e}")))?;

        let cores: usize = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid CPU count: {e}")))?;

        Ok(cores)
    }

    fn get_disk_space() -> Result<(u64, u64)> {
        // Get ZFS pool info - default to 'zroot' or from config
        // For now, use df on root filesystem as approximation
        let output = Command::new("df")
            .args(["-k", "/"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to get disk info: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return Err(VelnError::HostRequirements(
                "Could not parse disk information".to_string(),
            ));
        }

        // Parse the second line (first is header)
        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 4 {
            return Err(VelnError::HostRequirements(
                "Invalid disk information format".to_string(),
            ));
        }

        let total_kb: u64 = parts[1]
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid total disk value: {e}")))?;
        let available_kb: u64 = parts[3]
            .parse()
            .map_err(|e| VelnError::HostRequirements(format!("Invalid available disk value: {e}")))?;

        Ok((total_kb / 1024 / 1024, available_kb / 1024 / 1024)) // Convert to GB
    }
}
