//! Domain module - Core business logic
//!
//! References:
//! - FreeBSD Handbook Chapter 22: Virtualization
//!   <https://docs.freebsd.org/en/books/handbook/virtualization/>
//! - bhyve host requirements
//!   <https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve>

pub mod repository;
pub mod requirements;
pub mod resources;
pub mod vm;

pub use repository::{Iso, Snapshot, Template, VmRepository, VmRuntime};
pub use requirements::{BhyveRequirements, RequirementsChecker};
pub use resources::{HostResources, ResourceMonitor};
pub use vm::{ConsoleType, NetworkConfig, Uuid, VirtualMachine, VmConfig, VmState};
