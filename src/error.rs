use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum VelnError {
    #[error("Root privileges required")]
    #[diagnostic(code(veln::auth::root_required), help("Try running with sudo."))]
    RootRequired,

    #[error("VM not found: {0}")]
    #[diagnostic(code(veln::vm::not_found))]
    VmNotFound(String),

    #[error("ZFS operation failed: {0}")]
    #[diagnostic(code(veln::infra::zfs))]
    ZfsError(String),

    #[error("Configuration error: {0}")]
    #[diagnostic(code(veln::config::invalid))]
    Config(String),

    #[error("Host requirements not met: {0}")]
    #[diagnostic(code(veln::host::requirements), help("Check bhyve documentation for system requirements"))]
    HostRequirements(String),

    #[error("Insufficient resources: {0}")]
    #[diagnostic(code(veln::host::resources), help("Free up resources or reduce VM allocation"))]
    InsufficientResources(String),

    #[error("Invalid VM state transition: {0} to {1}")]
    #[diagnostic(code(veln::vm::invalid_state))]
    InvalidStateTransition(String, String),

    #[error(transparent)]
    #[diagnostic(code(veln::io))]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, VelnError>;
