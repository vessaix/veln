//! Infrastructure module - ZFS and bhyve implementations
//!
//! References:
//! - FreeBSD ZFS documentation: <https://docs.freebsd.org/en/books/handbook/zfs/>
//! - zfs(8) man page
//! - FreeBSD bhyve documentation: <https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve>
//! - bhyve(8), bhyvectl(8), bhyveload(8) man pages

pub mod api;
pub mod bhyve;
pub mod cloudinit;
pub mod zfs;

pub use api::ApiServer;
pub use bhyve::BhyveRuntime;
pub use cloudinit::CloudInitSeed;
pub use zfs::{IsoRepository, ZfsRepository};
