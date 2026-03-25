use crate::cli::{Commands, IsoCommands, SnapshotCommands, TemplateCommands};
use crate::config::Config;
use crate::domain::vm::{VirtualMachine, VmConfig, VmState};
use crate::domain::{RequirementsChecker, ResourceMonitor, VmRepository, VmRuntime};
use crate::error::{Result, VelnError};
use crate::infrastructure::zfs::{IsoRepository, ZfsRepository};
use crate::infrastructure::bhyve::BhyveRuntime;
use crate::infrastructure::cloudinit::{CloudInitConfig, CloudInitSeed, CloudInitUser};
use crate::infrastructure::api::ApiServer;
use std::path::PathBuf;

/// # Errors
/// Returns `VelnError::RootRequired` if a privileged command is run without root.
/// Returns `VelnError::Config` if the configuration file is missing or invalid.
/// Returns `VelnError::HostRequirements` if bhyve requirements are not met.
/// Returns `VelnError::InsufficientResources` if host resources are insufficient.
pub fn run(command: Commands) -> Result<()> {
    match command {
        Commands::Check => check_system()?,
        Commands::List => cmd_list()?,
        Commands::Create { name, memory, cpus, disk, iso } => {
            cmd_create(&name, memory, cpus, disk, iso)?;
        }
        Commands::Start { name } => cmd_start(name)?,
        Commands::Stop { name, force } => cmd_stop(name, force)?,
        Commands::Destroy { name, yes } => cmd_destroy(name, yes)?,
        Commands::Status { name, verbose } => cmd_status(name, verbose)?,
        Commands::Snapshot { command } => match command {
            SnapshotCommands::Create { vm, name, comment } => {
                cmd_snapshot_create(vm, name, comment)?;
            }
            SnapshotCommands::List { vm } => cmd_snapshot_list(vm)?,
            SnapshotCommands::Rollback { vm, name, force } => {
                cmd_snapshot_rollback(vm, name, force)?;
            }
            SnapshotCommands::Delete { vm, name } => cmd_snapshot_delete(vm, name)?,
        },
        Commands::Iso { command } => match command {
            IsoCommands::List => cmd_iso_list()?,
            IsoCommands::Fetch {
                name,
                url,
                description,
            } => cmd_iso_fetch(name, url, description)?,
            IsoCommands::Delete { name, yes } => cmd_iso_delete(name, yes)?,
        },
        Commands::CloudInit { command } => match command {
            crate::cli::CloudInitCommands::Generate {
                vm,
                hostname,
                ssh_key,
                admin_user,
                packages,
                dhcp,
                ip,
                gateway,
            } => cmd_cloudinit_generate(vm, hostname, ssh_key, admin_user, packages, dhcp, ip, gateway)?,
            crate::cli::CloudInitCommands::Remove { vm } => cmd_cloudinit_remove(vm)?,
        },
        Commands::Api { bind, port } => cmd_api(bind, port)?,
        Commands::Clone { source, target, linked } => {
            cmd_clone(source, target, linked)?;
        }
        Commands::Console { name, device, vnc, vnc_port } => {
            if vnc {
                cmd_console_vnc(name, vnc_port)?;
            } else {
                cmd_console(name, device)?;
            }
        }
        Commands::Template { command } => match command {
            TemplateCommands::Create { vm, name, description } => {
                cmd_template_create(vm, name, description)?;
            }
            TemplateCommands::List => cmd_template_list()?,
            TemplateCommands::Deploy { template, vm, linked } => {
                cmd_template_deploy(template, vm, linked)?;
            }
            TemplateCommands::Delete { name, yes } => {
                cmd_template_delete(name, yes)?;
            }
        },
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let vms = repo.list()?;

    if vms.is_empty() {
        println!("No VMs configured.");
    } else {
        println!("UUID\t\tNAME\t\tSTATE\tCPUS\tMEMORY\tDISK");
        for vm in vms {
            let uuid_short = &vm.uuid.0[..8];
            println!(
                "{}...\t{}\t{}\t{}\t{}MB\t{}GB",
                uuid_short, vm.name, vm.state, vm.config.cpus, vm.config.memory_mb, vm.config.disk_gb
            );
        }
    }
    
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_create(
    name: &str,
    memory: u64,
    cpus: u8,
    disk: u64,
    iso: Option<String>,
) -> Result<()> {
    require_root()?;
    check_system()?;
    check_resources(memory, disk, cpus)?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if repo.exists(name) {
        return Err(VelnError::VmNotFound(format!(
            "VM '{name}' already exists"
        )));
    }

    // Check if ISO exists if specified
    if let Some(ref iso_name) = iso {
        let iso_repo = IsoRepository::new(&config)?;
        if !iso_repo.exists(iso_name) {
            return Err(VelnError::VmNotFound(format!(
                "ISO image '{iso_name}' not found. Use 'veln iso list' to see available images."
            )));
        }
    }

    let vm_config = VmConfig {
        cpus,
        memory_mb: memory,
        disk_gb: disk,
        ..VmConfig::default()
    };

    let mut vm = VirtualMachine::with_config(name.to_string(), vm_config);
    vm.transition_to(VmState::Stopped)?;

    repo.save(&vm)?;

    if let Some(iso_name) = iso {
        println!("Created VM: {name} (with ISO: {iso_name})");
        println!("Use 'veln start {name}' to boot from ISO");
    } else {
        println!("Created VM: {name}");
    }

    Ok(())
}

fn cmd_start(name: String) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    let vm = repo.load(&name)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());
    let disk_path = PathBuf::from(format!("/dev/zvol/{}/{}/{}/disk", config.zfs_pool, config.vm_root, name));
    
    if !disk_path.exists() {
        println!("Creating ZVOL disk...");
        runtime.create(&name, &vm.config)?;
    }

    println!("Starting VM: {name}");
    runtime.start(&vm)?;
    
    let mut vm = vm;
    vm.transition_to(VmState::Running)?;
    repo.save(&vm)?;
    
    println!("VM '{name}' started successfully");
    
    Ok(())
}

fn cmd_stop(name: String, force: bool) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    let mut vm = repo.load(&name)?;

    if force {
        println!("Force stopping VM: {name}");
        runtime.destroy(&name)?;
    } else {
        println!("Stopping VM gracefully: {name}");
        runtime.stop(&name)?;
    }

    vm.transition_to(VmState::Stopped)?;
    repo.save(&vm)?;

    println!("VM '{name}' stopped");
    
    Ok(())
}

fn cmd_destroy(name: String, yes: bool) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    let _vm = repo.load(&name)?;

    if !yes {
        eprintln!("WARNING: This will permanently delete VM '{name}' and all its data.");
        eprintln!("Run with --yes to confirm.");
        return Err(VelnError::Config(
            "Destruction not confirmed".to_string()
        ));
    }

    if runtime.status(&name)? {
        println!("Stopping VM first...");
        runtime.destroy(&name)?;
    }

    repo.delete(&name)?;
    println!("VM '{name}' destroyed");
    
    Ok(())
}

fn check_system() -> Result<()> {
    print!("Checking bhyve requirements... ");
    RequirementsChecker::verify_or_fail()?;
    println!("OK");

    print!("Checking configuration... ");
    let config = Config::load()?;
    println!("OK (pool={})", config.zfs_pool);

    print!("Checking host resources... ");
    let resources = ResourceMonitor::get_resources()?;
    println!(
        "OK ({}MB RAM, {} cores, {}GB disk free)",
        resources.available_memory_mb,
        resources.available_cores,
        resources.available_disk_gb
    );

    Ok(())
}

fn cmd_console(name: String, device: u8) -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    // Check if VM is running
    if !runtime.status(&name)? {
        return Err(VelnError::VmNotFound(format!(
            "VM '{name}' is not running"
        )));
    }

    // Connect to nmdm console device
    // VMs use nmdm devices: /dev/nmdm<VM_NAME><device_num>A
    let console_path = format!("/dev/nmdm{name}{device}A");
    
    println!("Attaching to console of VM '{name}'...");
    println!("Use ~. to disconnect");
    
    // Execute cu command to attach to console
    let status = std::process::Command::new("cu")
        .args(["-l", &console_path, "-s", "115200"])
        .status()
        .map_err(VelnError::Io)?;

    if !status.success() {
        return Err(VelnError::Io(std::io::Error::other(
            format!("cu command failed with status: {status}")
        )));
    }

    Ok(())
}

fn cmd_status(name: String, verbose: bool) -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    let vm = repo.load(&name)?;
    let is_running = runtime.status(&name)?;

    println!("VM: {}", vm.name);
    println!("UUID: {}", vm.uuid);
    println!("State: {}", vm.state);
    println!("Running: {}", if is_running { "yes" } else { "no" });

    if verbose {
        println!("\nConfiguration:");
        println!("  CPUs: {}", vm.config.cpus);
        println!("  Memory: {} MB", vm.config.memory_mb);
        println!("  Disk: {} GB", vm.config.disk_gb);
        
        match &vm.config.network.backend {
            crate::domain::vm::NetworkBackend::TapBridge { bridge } => {
                println!("  Network: TAP + Bridge ({bridge})");
            }
            crate::domain::vm::NetworkBackend::Vale { switch } => {
                println!("  Network: VALE Switch ({switch})");
            }
        }
        
        if let Some(ref mac) = vm.config.network.mac {
            println!("  MAC: {mac}");
        }
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_snapshot_create(vm: String, name: String, comment: Option<String>) -> Result<()> {
    require_root()?;
    
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    
    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(vm));
    }
    
    println!("Creating snapshot '{name}' for VM '{vm}'...");
    repo.create_snapshot(&vm, &name, comment.as_deref())?;
    println!("Snapshot '{name}' created successfully");
    
    Ok(())
}

fn cmd_snapshot_list(vm: String) -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    
    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(vm));
    }
    
    let snapshots = repo.list_snapshots(&vm)?;
    
    if snapshots.is_empty() {
        println!("No snapshots for VM '{vm}'");
    } else {
        println!("Snapshots for VM '{vm}':");
        println!("{:<20} {:<20} {:<12} COMMENT", "NAME", "CREATED", "SIZE");
        for snap in snapshots {
            let size_str = format_size(snap.size);
            let comment_str = snap.comment.unwrap_or_default();
            println!("{:<20} {:<20} {:<12} {}", 
                snap.name, 
                snap.created,
                size_str,
                if comment_str.len() > 30 { format!("{}...", &comment_str[..27]) } else { comment_str }
            );
        }
    }
    
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_snapshot_rollback(vm: String, name: String, force: bool) -> Result<()> {
    require_root()?;
    
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());
    
    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(vm));
    }
    
    // Check if VM is running
    if runtime.status(&vm)? {
        return Err(VelnError::Config(
            format!("VM '{vm}' is running. Stop the VM before rolling back.")
        ));
    }
    
    if force {
        println!("Rolling back VM '{vm}' to snapshot '{name}' (will destroy newer snapshots)...");
    } else {
        println!("Rolling back VM '{vm}' to snapshot '{name}'...");
    }
    
    repo.rollback_snapshot(&vm, &name, force)?;
    println!("VM '{vm}' rolled back to snapshot '{name}'");
    
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_snapshot_delete(vm: String, name: String) -> Result<()> {
    require_root()?;
    
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    
    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(vm));
    }
    
    println!("Deleting snapshot '{name}' for VM '{vm}'...");
    repo.delete_snapshot(&vm, &name)?;
    println!("Snapshot '{name}' deleted successfully");
    
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{:.0}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

fn cmd_iso_list() -> Result<()> {
    let config = Config::load()?;
    let iso_repo = IsoRepository::new(&config)?;

    let isos = iso_repo.list()?;

    if isos.is_empty() {
        println!("No ISO images available.");
        println!("Use 'veln iso fetch <name> <url>' to download an ISO.");
    } else {
        println!("Available ISO images:");
        println!("{:<20} {:<12} {:<20} DESCRIPTION", "NAME", "SIZE", "DOWNLOADED");
        for iso in isos {
            let size_str = format_size(iso.size);
            let desc = iso.description.unwrap_or_default();
            let short_desc = if desc.len() > 30 {
                format!("{}...", &desc[..27])
            } else {
                desc
            };
            println!(
                "{:<20} {:<12} {:<20} {}",
                iso.name, size_str, iso.downloaded, short_desc
            );
        }
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_iso_fetch(name: String, url: String, description: Option<String>) -> Result<()> {
    require_root()?;

    let config = Config::load()?;
    let iso_repo = IsoRepository::new(&config)?;

    if iso_repo.exists(&name) {
        return Err(VelnError::Config(format!(
            "ISO '{name}' already exists"
        )));
    }

    iso_repo.fetch(&name, &url, description.as_deref())?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_iso_delete(name: String, yes: bool) -> Result<()> {
    require_root()?;

    let config = Config::load()?;
    let iso_repo = IsoRepository::new(&config)?;

    if !iso_repo.exists(&name) {
        return Err(VelnError::VmNotFound(format!(
            "ISO '{name}' not found"
        )));
    }

    if !yes {
        eprintln!("WARNING: This will permanently delete ISO '{name}'.");
        eprintln!("Run with --yes to confirm.");
        return Err(VelnError::Config(
            "Deletion not confirmed".to_string()
        ));
    }

    println!("Deleting ISO '{name}'...");
    iso_repo.delete(&name)?;
    println!("ISO '{name}' deleted successfully");

    Ok(())
}

fn cmd_console_vnc(name: String, vnc_port: u16) -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

    if !repo.exists(&name) {
        return Err(VelnError::VmNotFound(name));
    }

    // Check if VM is running
    if !runtime.status(&name)? {
        return Err(VelnError::VmNotFound(format!(
            "VM '{name}' is not running"
        )));
    }

    println!("VNC console for VM '{name}' available at:");
    println!("  Host: localhost");
    println!("  Port: {vnc_port}");
    println!();
    println!("Connect with:");
    println!("  vncviewer localhost:{vnc_port}");
    println!("  Or any VNC client");

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_clone(source: String, target: String, linked: bool) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.exists(&source) {
        return Err(VelnError::VmNotFound(format!(
            "Source VM '{source}' not found"
        )));
    }

    if repo.exists(&target) {
        return Err(VelnError::Config(format!(
            "Target VM '{target}' already exists"
        )));
    }

    // Check if source VM is running
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());
    if runtime.status(&source)? {
        return Err(VelnError::Config(format!(
            "Cannot clone running VM '{source}'. Stop the VM first."
        )));
    }

    if linked {
        println!("Creating linked clone '{target}' from '{source}'...");
        repo.clone_vm(&source, &target, true)?;
        println!("Linked clone '{target}' created successfully");
    } else {
        println!("Creating full clone '{target}' from '{source}'...");
        repo.clone_vm(&source, &target, false)?;
        println!("Full clone '{target}' created successfully");
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_template_create(vm: String, name: Option<String>, description: Option<String>) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(format!(
            "VM '{vm}' not found"
        )));
    }

    // Check if VM is running
    let runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());
    if runtime.status(&vm)? {
        return Err(VelnError::Config(format!(
            "Cannot create template from running VM '{vm}'. Stop the VM first."
        )));
    }

    let template_name = name.unwrap_or_else(|| vm.clone());
    
    println!("Creating template '{template_name}' from VM '{vm}'...");
    repo.create_template(&vm, &template_name, description.as_deref())?;
    println!("Template '{template_name}' created successfully");

    Ok(())
}

fn cmd_template_list() -> Result<()> {
    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    let templates = repo.list_templates()?;

    if templates.is_empty() {
        println!("No templates available.");
        println!("Use 'veln template create <vm>' to create a template from a VM.");
    } else {
        println!("Available templates:");
        println!("{:<20} {:<30} CREATED", "NAME", "DESCRIPTION");
        for template in templates {
            let desc = template.description.unwrap_or_default();
            let short_desc = if desc.len() > 30 {
                format!("{}...", &desc[..27])
            } else {
                desc
            };
            println!(
                "{:<20} {:<30} {}",
                template.name, short_desc, template.created
            );
        }
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_template_deploy(template: String, vm: String, linked: bool) -> Result<()> {
    require_root()?;
    check_system()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.template_exists(&template) {
        return Err(VelnError::VmNotFound(format!(
            "Template '{template}' not found"
        )));
    }

    if repo.exists(&vm) {
        return Err(VelnError::Config(format!(
            "VM '{vm}' already exists"
        )));
    }

    if linked {
        println!("Deploying linked clone '{vm}' from template '{template}'...");
        repo.deploy_template(&template, &vm, true)?;
        println!("Linked clone '{vm}' deployed successfully");
    } else {
        println!("Deploying full clone '{vm}' from template '{template}'...");
        repo.deploy_template(&template, &vm, false)?;
        println!("Full clone '{vm}' deployed successfully");
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_template_delete(name: String, yes: bool) -> Result<()> {
    require_root()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.template_exists(&name) {
        return Err(VelnError::VmNotFound(format!(
            "Template '{name}' not found"
        )));
    }

    if !yes {
        eprintln!("WARNING: This will permanently delete template '{name}'.");
        eprintln!("Any VMs deployed from this template will be affected.");
        eprintln!("Run with --yes to confirm.");
        return Err(VelnError::Config(
            "Deletion not confirmed".to_string()
        ));
    }

    println!("Deleting template '{name}'...");
    repo.delete_template(&name)?;
    println!("Template '{name}' deleted successfully");

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn cmd_cloudinit_generate(
    vm: String,
    hostname: Option<String>,
    ssh_key: Option<String>,
    admin_user: Option<String>,
    packages: Option<String>,
    dhcp: bool,
    ip: Option<String>,
    gateway: Option<String>,
) -> Result<()> {
    require_root()?;

    let config = Config::load()?;
    let repo = ZfsRepository::new(&config)?;

    if !repo.exists(&vm) {
        return Err(VelnError::VmNotFound(vm));
    }

    let mut cloud_config = CloudInitConfig {
        hostname: hostname.unwrap_or_else(|| vm.clone()),
        ..CloudInitConfig::default()
    };

    // Add admin user if specified
    if let Some(username) = admin_user {
        let mut user = CloudInitUser::admin_user(&username);
        if let Some(ref key) = ssh_key {
            user.ssh_authorized_keys.push(key.clone());
        }
        cloud_config.users.push(user);
    } else if let Some(key) = ssh_key {
        cloud_config.ssh_authorized_keys.push(key);
    }

    // Parse packages
    if let Some(pkgs) = packages {
        cloud_config.packages = pkgs.split(',').map(|s| s.trim().to_string()).collect();
    }

    // Network configuration
    if dhcp {
        cloud_config.network_config = Some(CloudInitSeed::generate_dhcp_network_config());
    } else if let (Some(ip_addr), Some(gw)) = (ip, gateway) {
        cloud_config.network_config = Some(CloudInitSeed::generate_static_network_config(
            "vtnet0",
            &ip_addr,
            &gw,
            &["8.8.8.8".to_string()],
        ));
    }

    let seed_gen = CloudInitSeed::new(&config)?;
    let iso_path = seed_gen.generate_seed(&vm, &cloud_config)?;

    println!("Cloud-init seed ISO created: {}", iso_path.display());
    println!("The ISO will be automatically attached when starting the VM");

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_cloudinit_remove(vm: String) -> Result<()> {
    require_root()?;

    let config = Config::load()?;
    let seed_gen = CloudInitSeed::new(&config)?;
    seed_gen.remove_seed(&vm)?;

    println!("Cloud-init configuration removed for VM '{vm}'");

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn cmd_api(bind: String, port: u16) -> Result<()> {
    require_root()?;
    
    let server = ApiServer::new()?;
    println!("Starting veln API server on http://{bind}:{port}");
    println!("Available endpoints:");
    println!("  GET  /api/health       - Health check");
    println!("  GET  /api/info         - System information");
    println!("  GET  /api/vms          - List all VMs");
    println!("  GET  /api/vms/{{name}} - Get VM details");
    println!("  POST /api/vms          - Create VM");
    println!("  POST /api/vms/{{name}}/start - Start VM");
    println!("  POST /api/vms/{{name}}/stop  - Stop VM");
    println!();
    println!("Press Ctrl+C to stop the server");
    
    server.serve(&bind, port);
    
    Ok(())
}

fn check_resources(memory_mb: u64, disk_gb: u64, cpus: u8) -> Result<()> {
    ResourceMonitor::can_create_vm(memory_mb, disk_gb, cpus)
}

fn require_root() -> Result<()> {
    if !is_root::is_root() {
        return Err(VelnError::RootRequired);
    }
    Ok(())
}
