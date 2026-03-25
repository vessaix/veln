use crate::error::{Result, VelnError};
use std::process::Command;

/// bhyve host requirements
/// Reference: <https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve>
pub struct BhyveRequirements;

impl BhyveRequirements {
    /// Check all bhyve requirements
    /// # Errors
    /// Returns `VelnError::HostRequirements` if any requirement is not met
    pub fn check_all() -> Result<()> {
        Self::check_cpu_support()?;
        Self::check_kernel_modules()?;
        Self::check_vmm_support()?;
        Self::check_tun_interface()?;
        Ok(())
    }

    /// Check CPU virtualization support (VT-x/AMD-V)
    /// Reference: FreeBSD Handbook - Section 22.2 "Hardware Requirements"
    fn check_cpu_support() -> Result<()> {
        // Check for VT-x (Intel) or SVM (AMD) in CPU features
        let output = Command::new("sysctl")
            .args(["-n", "hw.vmm.vmx.initialized"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to check CPU support: {e}")))?;

        let vmx_initialized = String::from_utf8_lossy(&output.stdout).trim() == "1";

        if !vmx_initialized {
            // Try AMD SVM
            let output = Command::new("sysctl")
                .args(["-n", "hw.vmm.svm.initialized"])
                .output()
                .map_err(|e| VelnError::HostRequirements(format!("Failed to check CPU support: {e}")))?;

            let svm_initialized = String::from_utf8_lossy(&output.stdout).trim() == "1";

            if !svm_initialized {
                return Err(VelnError::HostRequirements(
                    "CPU does not support virtualization (VT-x or AMD-V)".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if required kernel modules are loaded
    /// Reference: FreeBSD Handbook - Section 22.4 "Loading Required Modules"
    fn check_kernel_modules() -> Result<()> {
        let required_modules = ["vmm", "if_tuntap", "nmdm"];

        for module in &required_modules {
            let output = Command::new("kldstat")
                .args(["-n", module])
                .output()
                .map_err(|e| VelnError::HostRequirements(format!("Failed to check module {module}: {e}")))?;

            if !output.status.success() {
                return Err(VelnError::HostRequirements(format!(
                    "Required kernel module '{module}' is not loaded. Run: kldload {module}"
                )));
            }
        }

        Ok(())
    }

    /// Check vmm subsystem availability
    fn check_vmm_support() -> Result<()> {
        let output = Command::new("sysctl")
            .args(["-n", "hw.vmm.vmm_initialized"])
            .output()
            .map_err(|e| VelnError::HostRequirements(format!("Failed to check vmm: {e}")))?;

        let vmm_initialized = String::from_utf8_lossy(&output.stdout).trim() == "1";

        if !vmm_initialized {
            return Err(VelnError::HostRequirements(
                "VMM subsystem is not initialized. Ensure 'vmm' module is loaded.".to_string(),
            ));
        }

        Ok(())
    }

    /// Check TUN/TAP interface support
    fn check_tun_interface() -> Result<()> {
        // Check if /dev/net/tun exists and is accessible
        if !std::path::Path::new("/dev/net/tun").exists() {
            return Err(VelnError::HostRequirements(
                "TUN/TAP device not found. Ensure 'if_tuntap' module is loaded.".to_string(),
            ));
        }

        Ok(())
    }
}

/// Requirements checker that runs automatically before VM operations
pub struct RequirementsChecker;

impl RequirementsChecker {
    /// Check requirements and continue only if passed
    /// # Errors
    /// Returns `VelnError::HostRequirements` if requirements are not met
    pub fn verify_or_fail() -> Result<()> {
        BhyveRequirements::check_all()
    }
}
