use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "veln", version, about = "FreeBSD VM Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check system readiness
    Check,

    /// List VMs
    List,

    /// Create a new VM
    Create {
        name: String,
        /// Memory in MB
        #[arg(short, long, default_value = "1024")]
        memory: u64,
        /// CPUs
        #[arg(short, long, default_value = "2")]
        cpus: u8,
        /// Disk size in GB
        #[arg(short, long, default_value = "20")]
        disk: u64,
        /// ISO image to attach for installation
        #[arg(short, long)]
        iso: Option<String>,
    },

    /// Start a VM
    Start { name: String },

    /// Stop a VM gracefully
    Stop {
        name: String,
        /// Force stop (power off) instead of graceful shutdown
        #[arg(short, long)]
        force: bool,
    },

    /// Destroy a VM (stop and delete)
    Destroy {
        name: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Attach to VM console
    Console {
        name: String,
        /// Console device number for serial (default: 0)
        #[arg(short, long, default_value = "0")]
        device: u8,
        /// Connect via VNC instead of serial
        #[arg(short, long)]
        vnc: bool,
        /// VNC port (default: 5900)
        #[arg(long, default_value = "5900")]
        vnc_port: u16,
    },

    /// Show VM status
    Status {
        name: String,
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Manage VM snapshots
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommands,
    },

    /// Manage ISO images
    Iso {
        #[command(subcommand)]
        command: IsoCommands,
    },

    /// Clone a VM
    Clone {
        /// Source VM name
        source: String,
        /// New VM name
        target: String,
        /// Create a linked clone (ZFS clone) instead of full copy
        #[arg(short, long)]
        linked: bool,
    },

    /// Manage VM templates
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },

    /// Configure cloud-init for a VM
    CloudInit {
        #[command(subcommand)]
        command: CloudInitCommands,
    },

    /// Start REST API server
    Api {
        /// Bind address (default: 127.0.0.1)
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
        /// Port (default: 8080)
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Tools and maintenance commands
    Tools {
        /// Uninstall veln from the system
        #[arg(long)]
        uninstall: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
        /// Show what would be removed without removing
        #[arg(long)]
        dry_run: bool,
        /// Remove all data including VMs and ZFS datasets
        #[arg(long)]
        purge: bool,
        /// Installation prefix (default: /usr/local)
        #[arg(short, long, default_value = "/usr/local")]
        prefix: String,
    },
}

#[derive(Subcommand)]
pub enum SnapshotCommands {
    /// Create a new snapshot
    Create {
        /// VM name
        vm: String,
        /// Snapshot name
        #[arg(short, long)]
        name: String,
        /// Description/comment
        #[arg(short, long)]
        comment: Option<String>,
    },

    /// List snapshots for a VM
    List {
        /// VM name
        vm: String,
    },

    /// Rollback VM to a snapshot
    Rollback {
        /// VM name
        vm: String,
        /// Snapshot name
        #[arg(short, long)]
        name: String,
        /// Force rollback (destroy newer snapshots)
        #[arg(short, long)]
        force: bool,
    },

    /// Delete a snapshot
    Delete {
        /// VM name
        vm: String,
        /// Snapshot name
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum IsoCommands {
    /// List available ISO images
    List,

    /// Download an ISO image
    Fetch {
        /// Name for the ISO
        name: String,
        /// URL to download from
        url: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Delete an ISO image
    Delete {
        /// ISO name
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Create a template from an existing VM
    Create {
        /// VM name to convert to template
        vm: String,
        /// Template name (defaults to VM name)
        #[arg(short, long)]
        name: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List available templates
    List,

    /// Create a VM from a template
    Deploy {
        /// Template name
        template: String,
        /// New VM name
        vm: String,
        /// Create a linked clone
        #[arg(short, long)]
        linked: bool,
    },

    /// Delete a template
    Delete {
        /// Template name
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum CloudInitCommands {
    /// Generate cloud-init config for a VM
    Generate {
        /// VM name
        vm: String,
        /// Hostname
        #[arg(short, long)]
        hostname: Option<String>,
        /// SSH public key to add
        #[arg(short, long)]
        ssh_key: Option<String>,
        /// Create admin user with this name
        #[arg(long)]
        admin_user: Option<String>,
        /// Packages to install (comma-separated)
        #[arg(short, long)]
        packages: Option<String>,
        /// Use DHCP for networking
        #[arg(long)]
        dhcp: bool,
        /// Static IP (format: 192.168.1.100/24)
        #[arg(long)]
        ip: Option<String>,
        /// Gateway for static IP
        #[arg(long)]
        gateway: Option<String>,
    },

    /// Remove cloud-init config from a VM
    Remove {
        /// VM name
        vm: String,
    },
}

#[derive(Subcommand)]
pub enum ApiCommands {
    /// Start REST API server
    Serve {
        /// Bind address (default: 127.0.0.1)
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
        /// Port (default: 8080)
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}
