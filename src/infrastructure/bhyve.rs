use crate::domain::vm::{ConsoleType, VirtualMachine};
use crate::domain::VmRuntime;
use crate::error::{Result, VelnError};
use std::path::PathBuf;
use std::process::Command;

/// bhyve runtime implementation
/// Manages bhyve processes and VM lifecycle
/// Uses ZVOLs (ZFS volumes) for VM disks
/// Reference: <https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve>
pub struct BhyveRuntime {
    pool: String,
    vm_root: String,
}

impl BhyveRuntime {
    /// Create a new bhyve runtime
    /// # Arguments
    /// * `pool` - ZFS pool name (e.g., "zroot")
    /// * `vm_root` - Dataset path under pool (e.g., "veln")
    #[must_use]
    pub fn new(pool: String, vm_root: String) -> Self {
        Self { pool, vm_root }
    }

    fn dataset_path(&self, name: &str) -> String {
        format!("{}/{}/{}", self.pool, self.vm_root, name)
    }

    fn disk_dataset(&self, name: &str) -> String {
        format!("{}/disk", self.dataset_path(name))
    }

    fn disk_path(&self, name: &str) -> PathBuf {
        // ZVOLs appear as devices under /dev/zvol/<pool>/<path>
        PathBuf::from(format!("/dev/zvol/{}", self.disk_dataset(name)))
    }

    fn generate_mac() -> String {
        // Generate a random MAC address in the bhyve OUI range
        // bhyve uses locally administered addresses
        format!(
            "58:9c:fc:{:02x}:{:02x}:{:02x}",
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>()
        )
    }

    fn build_bhyve_args(&self, vm: &VirtualMachine) -> Vec<String> {
        let mut args = vec![
            String::from("-c"),
            vm.config.cpus.to_string(),
            String::from("-m"),
            format!("{}", vm.config.memory_mb),
            String::from("-H"),
            String::from("-P"),
            String::from("-A"),
            String::from("-u"),
        ];

        // Network
        let mac = vm
            .config
            .network
            .mac
            .clone()
            .unwrap_or_else(Self::generate_mac);
        args.push(String::from("-s"));

        match &vm.config.network.backend {
            crate::domain::vm::NetworkBackend::TapBridge { bridge } => {
                // Standard TAP + Bridge networking
                args.push(format!("2:0,virtio-net,{bridge},{mac}"));
            }
            crate::domain::vm::NetworkBackend::Vale { switch } => {
                // Netgraph + VALE high-performance networking
                args.push(format!("2:0,virtio-net,{switch},{mac}"));
            }
        }

        // Disk
        let disk = self.disk_path(&vm.name);
        args.push(String::from("-s"));
        args.push(format!(
            "3:0,virtio-blk,{}",
            disk.display()
        ));

        // Console
        match &vm.config.console {
            ConsoleType::Tty => {
                args.push(String::from("-s"));
                args.push(String::from("1:0,lpc"));
                args.push(String::from("-l"));
                args.push(String::from("com1,/dev/nmdm0A"));
            }
            ConsoleType::Null => {
                // No console for background VMs
            }
            ConsoleType::Framebuffer { vnc_port } => {
                args.push(String::from("-s"));
                args.push(format!("1:0,fbuf,tcp=0.0.0.0:{vnc_port}"));
                args.push(String::from("-s"));
                args.push(String::from("31,lpc"));
            }
        }

        // VM name (must be last)
        args.push(vm.name.clone());

        args
    }

    fn vm_is_running(name: &str) -> bool {
        // Check if bhyve process exists for this VM
        Command::new("pgrep")
            .args(["-f", &format!("bhyve.*{name}")])
            .output()
            .is_ok_and(|output| output.status.success())
    }
}

impl VmRuntime for BhyveRuntime {
    /// # Errors
    /// Returns `VelnError::ZfsError` if ZVOL cannot be created
    fn create(&self, name: &str, config: &crate::domain::vm::VmConfig) -> Result<()> {
        let disk_dataset = self.disk_dataset(name);

        // Check if disk zvol already exists
        let check = Command::new("zfs")
            .args(["list", &disk_dataset])
            .output()
            .map_err(VelnError::Io)?;

        if check.status.success() {
            return Err(VelnError::ZfsError(format!(
                "Disk ZVOL already exists for VM '{name}'"
            )));
        }

        // Create ZVOL using zfs create -V
        // volmode=dev exposes it as a block device in /dev/zvol/
        let size_gb = config.disk_gb;
        let output = Command::new("zfs")
            .args([
                "create",
                "-V",
                &format!("{size_gb}G"),
                "-o",
                "volmode=dev",
                &disk_dataset,
            ])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to create ZVOL: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::Io` if bhyve process cannot be started
    fn start(&self, vm: &VirtualMachine) -> Result<()> {
        if Self::vm_is_running(&vm.name) {
            return Err(VelnError::ZfsError(format!(
                "VM '{}' is already running",
                vm.name
            )));
        }

        // Load bhyve kernel module for this VM
        let output = Command::new("bhyveload")
            .args([
                "-c",
                "/dev/nmdm0A",
                "-m",
                &vm.config.memory_mb.to_string(),
                "-d",
                &self.disk_path(&vm.name).display().to_string(),
                &vm.name,
            ])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to load VM: {stderr}"
            )));
        }

        // Start bhyve
        let args = self.build_bhyve_args(vm);
        let output = Command::new("bhyve")
            .args(&args)
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to start VM: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::Io` if ACPI shutdown fails
    fn stop(&self, name: &str) -> Result<()> {
        if !Self::vm_is_running(name) {
            return Err(VelnError::VmNotFound(format!(
                "VM '{name}' is not running"
            )));
        }

        // Try graceful ACPI shutdown
        let output = Command::new("bhyvectl")
            .args(["--acpi-poweroff", "--vm", name])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to stop VM: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::Io` if force destroy fails
    fn destroy(&self, name: &str) -> Result<()> {
        if !Self::vm_is_running(name) {
            return Err(VelnError::VmNotFound(format!(
                "VM '{name}' is not running"
            )));
        }

        // Force destroy
        let output = Command::new("bhyvectl")
            .args(["--force-poweroff", "--vm", name])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to destroy VM: {stderr}"
            )));
        }

        // Also destroy the VM instance
        let output = Command::new("bhyvectl")
            .args(["--destroy", "--vm", name])
            .output()
            .map_err(VelnError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VelnError::ZfsError(format!(
                "Failed to destroy VM instance: {stderr}"
            )));
        }

        Ok(())
    }

    /// # Errors
    /// Returns `VelnError::Io` if status check fails
    fn status(&self, name: &str) -> Result<bool> {
        Ok(Self::vm_is_running(name))
    }
}

// Simple random number generator for MAC addresses
// In production, use the `rand` crate properly
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    pub fn random<T>() -> T
    where
        T: From<u8>,
    {
        let val = COUNTER.fetch_add(1, Ordering::SeqCst);
        T::from((val & 0xFF) as u8)
    }
}
