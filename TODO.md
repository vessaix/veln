# Veln Foundation — Progress

## Foundation Phase
- [x] Step 1: Initialize project
- [x] Step 2: Create rustfmt.toml
- [x] Step 3: Create justfile
- [x] Step 4: Replace Cargo.toml
- [x] Step 5: Create src/error.rs
- [x] Step 6: Create src/config.rs
- [x] Step 7: Create src/cli.rs
- [x] Step 8: Create src/application.rs
- [x] Step 9: Create src/lib.rs
- [x] Step 10: Replace src/main.rs
- [x] Step 11: Verify (`just qa` passes)

## Domain Module Phase
- [x] Create src/domain/mod.rs with exports
- [x] Create src/domain/vm.rs - VM entity, state machine, config
- [x] Create src/domain/repository.rs - VmRepository + VmRuntime traits
- [x] Create src/domain/requirements.rs - BhyveRequirements checker
- [x] Create src/domain/resources.rs - ResourceMonitor with safety buffers
- [x] Update src/lib.rs to include domain module
- [x] Add new error variants for host requirements and resources
- [x] Create doc/DOMAIN.md documentation with FreeBSD Handbook references

## CLI Integration Phase
- [x] Add Create command to CLI with args (--memory, --cpus, --disk)
- [x] Update application.rs to use domain checks
- [x] Check command: runs bhyve requirements + config + resources
- [x] Create command: checks resources before allowing creation
- [x] Start command: checks requirements before allowing start

## Infrastructure Phase
- [x] Create src/infrastructure/mod.rs
- [x] Create src/infrastructure/zfs.rs - ZfsRepository implementation
- [x] Create src/infrastructure/bhyve.rs - BhyveRuntime implementation
- [x] Update src/lib.rs to include infrastructure module
- [x] Create doc/INFRASTRUCTURE.md with ZFS and bhyve references
- [x] Wire ZfsRepository into application commands (list, create)
- [x] Wire BhyveRuntime into start command
- [x] Use ZVOLs instead of sparse files (`zfs create -V`)

## VM Lifecycle Commands Phase
- [x] Add Stop command with graceful and force options
- [x] Add Destroy command with confirmation
- [x] Refactor application.rs into smaller functions

## UUID Support Phase
- [x] Add Uuid type to domain/vm.rs
- [x] Implement UUID v4 generation
- [x] Add uuid field to VirtualMachine
- [x] Update VmConfigData serialization to include UUID
- [x] Update ZfsRepository to handle UUIDs
- [x] Update list command to display UUIDs
- [x] Export Uuid from domain module

## Console Command Phase
- [x] Add Console command to CLI
- [x] Implement cmd_console function
- [x] Use cu command to attach to nmdm device
- [x] Check if VM is running before attaching

## Status Command Phase
- [x] Add Status command to CLI with verbose option
- [x] Implement cmd_status function
- [x] Show VM name, UUID, state, and running status
- [x] Show detailed configuration with --verbose flag

## Testing Phase
- [x] Add unit tests for VM state machine transitions
- [x] Add unit tests for UUID generation
- [x] Add unit tests for VM configuration defaults
- [x] Add unit tests for state transition validation
- [x] Verify all tests pass with `cargo test`

## Networking Architecture Phase
- [x] Add NetworkBackend enum (TapBridge vs Vale)
- [x] Update NetworkConfig to use backend abstraction
- [x] Update bhyve.rs to generate different network args per backend
- [x] Update ZfsRepository serialization for network backend
- [x] Update status command to display network backend type
- [x] Support both TAP+bridge (simple) and VALE (high-performance)

## Documentation Phase
- [x] Create comprehensive README.md with:
  - [x] Project overview and features
  - [x] Installation instructions
  - [x] Quick start guide
  - [x] Configuration examples
  - [x] Command reference
  - [x] Architecture overview
  - [x] Networking documentation
  - [x] Development guide

## VM Snapshots Phase
- [x] Add Snapshot command with subcommands (create, list, rollback, delete)
- [x] Implement Snapshot struct with name, created, comment, size fields
- [x] Add snapshot methods to VmRepository trait
- [x] Implement ZFS snapshot operations (zfs snapshot, zfs rollback, zfs destroy)
- [x] Store snapshot comments as ZFS user properties
- [x] Add snapshot listing with size formatting
- [x] Prevent rollback when VM is running
- [x] Update domain/mod.rs exports

## ISO Management Phase
- [x] Add Iso command with subcommands (list, fetch, delete)
- [x] Implement IsoRepository struct for managing ISO images
- [x] Create Iso struct with name, filename, description, size, downloaded fields
- [x] Store ISOs in dedicated ZFS dataset (<pool>/images/)
- [x] Download ISOs using curl with progress bar
- [x] Save ISO metadata as JSON files
- [x] Update Create command to support --iso option
- [x] Validate ISO exists before creating VM
- [x] Add chrono and serde_json dependencies
- [x] Export IsoRepository from infrastructure module

## VM Cloning and Templates Phase
- [x] Add Clone command with --linked option for linked clones
- [x] Implement Template command with subcommands (create, list, deploy, delete)
- [x] Create Template struct with name, description, created fields
- [x] Add clone_vm, create_template, list_templates, deploy_template, template_exists, delete_template to VmRepository trait
- [x] Add stub implementations in ZfsRepository
- [x] Update domain/mod.rs exports

## VNC Console Support Phase
- [x] Update Console command with --vnc and --vnc-port options
- [x] Implement cmd_console_vnc function
- [x] Update VM config to support VNC console type
- [x] Add VNC port configuration to bhyve args generation

## ZFS Clone/Template Implementation Phase
- [x] Implement clone_vm with linked and full clone support
- [x] Implement create_template with ZFS clone and metadata
- [x] Implement list_templates with metadata parsing
- [x] Implement deploy_template with linked and full deployment
- [x] Implement template_exists check
- [x] Implement delete_template with cleanup
- [x] Create TemplateMetadata struct for JSON storage

## Cloud-init Integration Phase
- [x] Create cloudinit.rs module with CloudInitSeed
- [x] Implement CloudInitConfig struct with all options
- [x] Implement user-data and meta-data generation
- [x] Implement seed ISO creation using mkisofs
- [x] Add cloud-init commands to CLI (generate, remove)
- [x] Add CloudInitUser struct with admin user support
- [x] Add network config generation (DHCP and static)
- [x] Export CloudInitSeed from infrastructure module

## REST API Phase
- [x] Add rouille dependency for HTTP server
- [x] Create api.rs module with ApiServer
- [x] Implement health check endpoint (/api/health)
- [x] Implement system info endpoint (/api/info)
- [x] Implement VM list endpoint (GET /api/vms)
- [x] Implement VM details endpoint (GET /api/vms/{name})
- [x] Implement VM create endpoint (POST /api/vms)
- [x] Implement VM start endpoint (POST /api/vms/{name}/start)
- [x] Implement VM stop endpoint (POST /api/vms/{name}/stop)
- [x] Add CORS support for browser clients
- [x] Add JSON request/response handling
- [x] Add proper HTTP status codes
- [x] Create Api command in CLI (veln api --bind 127.0.0.1 --port 8080)

## Installation & Packaging Phase
- [x] Create install.sh with source install and manifest tracking
- [x] Create uninstall.sh for clean removal of all installed files
- [x] Add package creation support (FreeBSD .pkg format)
- [x] Add local package repository setup
- [x] Create quick package builder (make-pkg.sh)
- [x] Update justfile with install/package/uninstall targets
- [x] Create comprehensive INSTALL.md documentation
- [x] Add installation status command
- [x] Add dry-run mode for uninstall
- [x] Add purge mode to remove all VM data
- [x] Support for pkg install/remove workflow
- [x] Create FreeBSD port files (Makefile, pkg-descr, distinfo)

## Future Phases
- [ ] Web UI
- [ ] Integration tests (requires FreeBSD environment)
