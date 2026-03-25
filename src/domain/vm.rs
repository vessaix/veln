/// VM power states
/// Reference: FreeBSD Handbook - Chapter 22.7 "Managing Virtual Machines"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    /// VM does not exist on disk
    Undefined,
    /// VM exists but is not running
    Stopped,
    /// VM is starting up
    Starting,
    /// VM is running
    Running,
    /// VM is shutting down
    Stopping,
    /// VM encountered an error
    Failed,
}

impl std::fmt::Display for VmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmState::Undefined => write!(f, "undefined"),
            VmState::Stopped => write!(f, "stopped"),
            VmState::Starting => write!(f, "starting"),
            VmState::Running => write!(f, "running"),
            VmState::Stopping => write!(f, "stopping"),
            VmState::Failed => write!(f, "failed"),
        }
    }
}

impl VmState {
    /// Check if state transition is valid
    /// Returns true if the transition from self to target is allowed
    #[must_use]
    pub fn can_transition_to(&self, target: VmState) -> bool {
        matches!(
            (*self, target),
            (VmState::Undefined | VmState::Failed, VmState::Stopped)
                | (VmState::Stopped, VmState::Starting | VmState::Undefined)
                | (VmState::Starting, VmState::Running | VmState::Failed)
                | (VmState::Running, VmState::Stopping | VmState::Failed)
                | (VmState::Stopping, VmState::Stopped | VmState::Failed)
        )
    }
}

/// UUID type for VM identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uuid(pub String);

impl Uuid {
    /// Generate a new v4 UUID
    #[must_use]
    pub fn new_v4() -> Self {
        // Simple UUID v4 generation without external crate
        // Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx where y is 8,9,a, or b
        let mut bytes = [0u8; 16];
        
        // Fill with random-ish data using simple counter
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        #[allow(clippy::cast_possible_truncation)]
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = ((now >> (i * 8)) ^ (i as u128).wrapping_mul(0x9e37_79b9_7f4a_7c15)) as u8;
        }
        
        // Set version (4) and variant (10xxxxxx)
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10
        
        let uuid_str = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        );
        
        Self(uuid_str)
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Virtual Machine entity
/// Reference: FreeBSD Handbook - Chapter 22 "Virtualization"
#[derive(Debug, Clone)]
pub struct VirtualMachine {
    /// Unique identifier (never changes, even if VM is renamed)
    pub uuid: Uuid,
    /// Human-readable name (can be changed)
    pub name: String,
    /// Current power state
    pub state: VmState,
    /// Hardware configuration
    pub config: VmConfig,
}

/// VM hardware configuration
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Number of virtual CPUs
    pub cpus: u8,
    /// Memory in megabytes
    pub memory_mb: u64,
    /// Disk size in gigabytes
    pub disk_gb: u64,
    /// Network configuration
    pub network: NetworkConfig,
    /// Console type
    pub console: ConsoleType,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            cpus: 2,
            memory_mb: 1024,
            disk_gb: 20,
            network: NetworkConfig::default(),
            console: ConsoleType::Tty,
        }
    }
}

/// Network backend types for VM networking
#[derive(Debug, Clone)]
pub enum NetworkBackend {
    /// TAP interface + Bridge (simple, default)
    /// Format: bridge0, bridge1, etc.
    TapBridge { bridge: String },
    /// Netgraph + VALE switch (high performance)
    /// Format: vale0, vale1, etc.
    Vale { switch: String },
}

impl Default for NetworkBackend {
    fn default() -> Self {
        Self::TapBridge {
            bridge: String::from("bridge0"),
        }
    }
}

/// Network configuration for VM
#[derive(Debug, Clone, Default)]
pub struct NetworkConfig {
    /// Network backend (TAP+bridge or VALE)
    pub backend: NetworkBackend,
    /// MAC address (if None, auto-generated)
    pub mac: Option<String>,
}

/// Console types supported by bhyve
/// Reference: FreeBSD Handbook - Section 22.7.4 "Console"
#[derive(Debug, Clone)]
pub enum ConsoleType {
    /// Local TTY device
    Tty,
    /// Null console (background)
    Null,
    /// Framebuffer with VNC
    Framebuffer { vnc_port: u16 },
}

impl VirtualMachine {
    /// Create a new VM in the Undefined state with generated UUID
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            state: VmState::Undefined,
            config: VmConfig::default(),
        }
    }

    /// Create a VM with specific configuration and generated UUID
    #[must_use]
    pub fn with_config(name: String, config: VmConfig) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            state: VmState::Undefined,
            config,
        }
    }

    /// Create a VM with specific UUID, name, and configuration
    /// Used when loading from storage
    #[must_use]
    pub fn with_uuid(uuid: Uuid, name: String, config: VmConfig) -> Self {
        Self {
            uuid,
            name,
            state: VmState::Undefined,
            config,
        }
    }

    /// Transition to a new state
    /// # Errors
    /// Returns `VelnError::InvalidStateTransition` if the transition is not allowed
    pub fn transition_to(&mut self, new_state: VmState) -> crate::error::Result<()> {
        if !self.state.can_transition_to(new_state) {
            return Err(crate::error::VelnError::InvalidStateTransition(
                self.state.to_string(),
                new_state.to_string(),
            ));
        }
        self.state = new_state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_state_transitions_valid() {
        // Undefined -> Stopped
        assert!(VmState::Undefined.can_transition_to(VmState::Stopped));
        
        // Stopped -> Starting
        assert!(VmState::Stopped.can_transition_to(VmState::Starting));
        // Stopped -> Undefined (destroy)
        assert!(VmState::Stopped.can_transition_to(VmState::Undefined));
        
        // Starting -> Running
        assert!(VmState::Starting.can_transition_to(VmState::Running));
        // Starting -> Failed
        assert!(VmState::Starting.can_transition_to(VmState::Failed));
        
        // Running -> Stopping
        assert!(VmState::Running.can_transition_to(VmState::Stopping));
        // Running -> Failed
        assert!(VmState::Running.can_transition_to(VmState::Failed));
        
        // Stopping -> Stopped
        assert!(VmState::Stopping.can_transition_to(VmState::Stopped));
        // Stopping -> Failed
        assert!(VmState::Stopping.can_transition_to(VmState::Failed));
        
        // Failed -> Stopped
        assert!(VmState::Failed.can_transition_to(VmState::Stopped));
    }

    #[test]
    fn test_vm_state_transitions_invalid() {
        // Cannot go from Undefined to Running directly
        assert!(!VmState::Undefined.can_transition_to(VmState::Running));
        // Cannot go from Undefined to Starting
        assert!(!VmState::Undefined.can_transition_to(VmState::Starting));
        
        // Cannot go from Stopped to Stopped
        assert!(!VmState::Stopped.can_transition_to(VmState::Stopped));
        // Cannot go from Stopped to Running directly
        assert!(!VmState::Stopped.can_transition_to(VmState::Running));
        
        // Cannot go from Running to Starting
        assert!(!VmState::Running.can_transition_to(VmState::Starting));
        // Cannot go from Running to Undefined directly
        assert!(!VmState::Running.can_transition_to(VmState::Undefined));
    }

    #[test]
    fn test_vm_transition_to_changes_state() {
        let mut vm = VirtualMachine::new("test".to_string());
        assert_eq!(vm.state, VmState::Undefined);
        
        vm.transition_to(VmState::Stopped).unwrap();
        assert_eq!(vm.state, VmState::Stopped);
        
        vm.transition_to(VmState::Starting).unwrap();
        assert_eq!(vm.state, VmState::Starting);
        
        vm.transition_to(VmState::Running).unwrap();
        assert_eq!(vm.state, VmState::Running);
    }

    #[test]
    fn test_vm_transition_to_invalid_fails() {
        let mut vm = VirtualMachine::new("test".to_string());
        
        // Should fail: Undefined -> Running
        let result = vm.transition_to(VmState::Running);
        assert!(result.is_err());
        assert_eq!(vm.state, VmState::Undefined); // State should not change
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        
        // UUIDs should be different
        assert_ne!(uuid1.0, uuid2.0);
        
        // Should be valid UUID format (36 characters with dashes)
        assert_eq!(uuid1.0.len(), 36);
        assert!(uuid1.0.contains('-'));
        
        // Should have version 4 indicator (4 at position 14)
        let parts: Vec<&str> = uuid1.0.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert!(parts[2].starts_with('4'));
    }

    #[test]
    fn test_vm_has_uuid() {
        let vm = VirtualMachine::new("test".to_string());
        
        // Should have a UUID
        assert_eq!(vm.uuid.0.len(), 36);
        
        // UUID should be unique for each VM
        let vm2 = VirtualMachine::new("test2".to_string());
        assert_ne!(vm.uuid.0, vm2.uuid.0);
    }

    #[test]
    fn test_vm_config_default() {
        let config = VmConfig::default();
        
        assert_eq!(config.cpus, 2);
        assert_eq!(config.memory_mb, 1024);
        assert_eq!(config.disk_gb, 20);
        
        // Check default network backend
        match &config.network.backend {
            NetworkBackend::TapBridge { bridge } => {
                assert_eq!(bridge, "bridge0");
            }
            _ => panic!("Expected TapBridge backend by default"),
        }
        assert!(config.network.mac.is_none());
    }

    #[test]
    fn test_vm_display_format() {
        assert_eq!(format!("{}", VmState::Undefined), "undefined");
        assert_eq!(format!("{}", VmState::Stopped), "stopped");
        assert_eq!(format!("{}", VmState::Starting), "starting");
        assert_eq!(format!("{}", VmState::Running), "running");
        assert_eq!(format!("{}", VmState::Stopping), "stopping");
        assert_eq!(format!("{}", VmState::Failed), "failed");
    }
}
