# Veln - FreeBSD Virtualization Manager

A modern, lightweight CLI for managing FreeBSD bhyve virtual machines with ZFS integration.

[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

## Overview

Veln is a FreeBSD-native virtualization management tool built in Rust. It provides a simple, robust CLI for creating, managing, and running bhyve virtual machines with automatic ZFS storage provisioning.

### Key Features

- **Native FreeBSD Integration**: Built specifically for FreeBSD using bhyve hypervisor
- **ZFS Storage**: Automatic ZVOL creation and management for VM disks
- **Dual Network Backends**: Support for both TAP+Bridge (simple) and VALE (high-performance) networking
- **Resource Safety**: Automatic host resource monitoring prevents VM exhaustion
- **UUID Tracking**: Each VM gets a unique identifier for reliable management
- **State Machine**: Enforces valid VM lifecycle transitions
- **Modern CLI**: Clean command interface with helpful error messages

## Requirements

### Hardware
- x86-64 or arm64 processor with virtualization support (VT-x/AMD-V)
- Sufficient RAM and disk space for VMs

### Software
- FreeBSD 13.0+ or 14.0+
- Root privileges (for VM operations)
- Required kernel modules:
  ```bash
  kldload vmm          # VMM (Virtual Machine Monitor)
  kldload if_tuntap    # TAP/TUN networking
  kldload nmdm         # Null modem (console)
  ```

### Build Dependencies
- Rust 1.70+ with Cargo
- Just (optional, for running convenience commands)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/vessaix/veln.git
cd veln

# Build release binary
cargo build --release

# Install (optional)
sudo cp target/release/veln /usr/local/bin/
```

### Using Just

```bash
# Build and test
just qa

# Install locally
sudo just install
```

## Quick Start

### 1. Verify System Readiness

```bash
sudo veln check
```

This checks:
- CPU virtualization support
- Required kernel modules
- Host resources (RAM, disk, CPUs)
- Configuration file

### 2. Create Your First VM

```bash
# Create a VM with default settings (2 CPUs, 1024MB RAM, 20GB disk)
sudo veln create myvm

# Or specify custom resources
sudo veln create myvm --cpus 4 --memory 4096 --disk 50

# Create VM with ISO for OS installation
sudo veln iso fetch freebsd14.1 https://download.freebsd.org/ftp/releases/ISO-IMAGES/14.1/FreeBSD-14.1-RELEASE-amd64-disc1.iso --description "FreeBSD 14.1"
sudo veln create myvm --iso freebsd14.1
```

### 3. Start the VM

```bash
sudo veln start myvm
```

### 4. Connect to Console

```bash
# Attach to serial console
sudo veln console myvm

# Use ~. to disconnect
```

### 5. Manage VMs

```bash
# List all VMs
veln list

# Check VM status
veln status myvm --verbose

# Stop VM gracefully
sudo veln stop myvm

# Force stop if stuck
sudo veln stop myvm --force

# Delete VM permanently
sudo veln destroy myvm --yes
```

### 6. Clone VMs

```bash
# Create a full clone
sudo veln clone myvm myvm-clone

# Create a linked clone (saves disk space)
sudo veln clone myvm myvm-linked --linked
```

### 7. Use Templates

```bash
# Create a template from a VM
sudo veln template create myvm --name freebsd-template --description "FreeBSD 14.1 base"

# List templates
veln template list

# Deploy VM from template
sudo veln template deploy freebsd-template new-vm

# Deploy linked clone from template
sudo veln template deploy freebsd-template new-vm --linked
```

## Configuration

### Config File Location

Veln looks for configuration in:
1. `$VELN_CONFIG` environment variable
2. `/usr/local/etc/veln/config.toml` (default)

### Example Configuration

```toml
# /usr/local/etc/veln/config.toml
zfs_pool = "zroot"
vm_root = "veln"
```

This creates VMs under `zroot/veln/<vm-name>/`.

### VM Configuration

Each VM stores its configuration in `veln.toml` within its ZFS dataset:

```toml
uuid = "550e8400-e29b-41d4-a716-446655440000"
name = "myvm"
cpus = 2
memory_mb = 1024
disk_gb = 20
network_type = "tap"
network_device = "bridge0"
mac = "58:9c:fc:01:02:03"
```

## Command Reference

### System Commands

| Command | Description |
|---------|-------------|
| `veln check` | Verify system readiness and requirements |
| `veln list` | List all VMs with their status |

### VM Lifecycle

| Command | Description |
|---------|-------------|
| `veln create <name> [options]` | Create a new VM |
| `veln start <name>` | Start a VM |
| `veln stop <name> [--force]` | Stop a VM (graceful or force) |
| `veln destroy <name> --yes` | Delete a VM permanently |
| `veln status <name> [--verbose]` | Show VM details |
| `veln console <name> [--device N]` | Attach to serial console |
| `veln console <name> --vnc [--vnc-port 5900]` | Show VNC connection info |

### VM Cloning

| Command | Description |
|---------|-------------|
| `veln clone <source> <target> [--linked]` | Clone a VM (full or linked) |

### VM Templates

| Command | Description |
|---------|-------------|
| `veln template create <vm> --name <template>` | Create template from VM |
| `veln template list` | List available templates |
| `veln template deploy <template> <vm> [--linked]` | Deploy VM from template |
| `veln template delete <name> --yes` | Delete a template |

### VM Snapshots

| Command | Description |
|---------|-------------|
| `veln snapshot create <vm> --name <snap>` | Create a snapshot |
| `veln snapshot list <vm>` | List snapshots for a VM |
| `veln snapshot rollback <vm> --name <snap>` | Rollback to snapshot |
| `veln snapshot delete <vm> --name <snap>` | Delete a snapshot |

### ISO Images

| Command | Description |
|---------|-------------|
| `veln iso list` | List available ISO images |
| `veln iso fetch <name> <url>` | Download an ISO image |
| `veln iso delete <name> --yes` | Delete an ISO image |

### Create Options

```bash
veln create <name> \
  --cpus 2 \
  --memory 1024 \
  --disk 20 \
  --iso freebsd14.1
```

- `--cpus`: Number of virtual CPUs (default: 2)
- `--memory`: Memory in MB (default: 1024)
- `--disk`: Disk size in GB (default: 20)
- `--iso`: ISO image to attach for installation (optional)

### Cloud-init Configuration

| Command | Description |
|---------|-------------|
| `veln cloud-init generate <vm> [--hostname name]` | Generate cloud-init config |
| `veln cloud-init generate <vm> --ssh-key <key>` | Add SSH key |
| `veln cloud-init generate <vm> --admin-user <name>` | Create admin user |
| `veln cloud-init generate <vm> --dhcp` | Configure DHCP |
| `veln cloud-init generate <vm> --ip 192.168.1.100/24 --gateway 192.168.1.1` | Static IP |
| `veln cloud-init remove <vm>` | Remove cloud-init config |

**Example:**
```bash
# Generate cloud-init with admin user and SSH key
sudo veln cloud-init generate myvm \
  --hostname myserver \
  --admin-user admin \
  --ssh-key "$(cat ~/.ssh/id_rsa.pub)" \
  --packages "git,python3,sudo" \
  --dhcp
```

## REST API

Veln provides a REST API for programmatic access to VM management.

### Starting the API Server

```bash
# Start API server on default address (127.0.0.1:8080)
sudo veln api

# Custom bind address and port
sudo veln api --bind 0.0.0.0 --port 8080
```

### API Endpoints

#### System

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/info` | System information |

#### VMs

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/vms` | List all VMs |
| `GET` | `/api/vms/{name}` | Get VM details |
| `POST` | `/api/vms` | Create new VM |
| `POST` | `/api/vms/{name}/start` | Start VM |
| `POST` | `/api/vms/{name}/stop` | Stop VM |
| `DELETE` | `/api/vms/{name}` | Delete VM |

### API Examples

```bash
# List all VMs
curl http://localhost:8080/api/vms

# Get VM details
curl http://localhost:8080/api/vms/myvm

# Create a VM
curl -X POST http://localhost:8080/api/vms \
  -H "Content-Type: application/json" \
  -d '{"name": "testvm", "cpus": 2, "memory": 1024, "disk": 20}'

# Start a VM
curl -X POST http://localhost:8080/api/vms/testvm/start

# Stop a VM
curl -X POST http://localhost:8080/api/vms/testvm/stop

# Get system information
curl http://localhost:8080/api/info
```

### Response Format

All API responses are in JSON format.

**Success Response (VM list):**
```json
[
  {
    "name": "myvm",
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "state": "stopped",
    "cpus": 2,
    "memory_mb": 1024,
    "disk_gb": 20,
    "network": "tap:bridge0"
  }
]
```

**Error Response:**
```json
{
  "error": "VM 'testvm' not found"
}
```

### HTTP Status Codes

- `200` - Success
- `201` - Created (VM created successfully)
- `400` - Bad Request (invalid JSON)
- `404` - Not Found (VM doesn't exist)
- `409` - Conflict (VM already exists)
- `422` - Unprocessable Entity (insufficient resources)
- `500` - Internal Server Error
- `503` - Service Unavailable (bhyve requirements not met)

## Architecture

### Domain Layer

Core business logic:
- **VM State Machine**: Enforces valid lifecycle transitions
- **UUID Generation**: Unique VM identifiers
- **Resource Monitoring**: Prevents host resource exhaustion
- **Requirements Checking**: Validates bhyve prerequisites

### Infrastructure Layer

System integration:
- **ZfsRepository**: ZFS dataset and ZVOL management
- **BhyveRuntime**: VM process lifecycle management
- **Network Backends**: TAP/Bridge and VALE switch support

### Storage Layout

```
zroot/
└── veln/
    └── <vm-name>/
        ├── veln.toml          # VM configuration
        └── disk               # ZVOL block device
```

Disk accessible at: `/dev/zvol/zroot/veln/<vm-name>/disk`

## Networking

### TAP + Bridge (Default)

Simple, well-understood networking:

```bash
# Setup bridge
ifconfig bridge0 create
ifconfig bridge0 addm em0
ifconfig bridge0 up

# VM connects automatically
# Network config in veln.toml:
# network_type = "tap"
# network_device = "bridge0"
```

### VALE (High Performance)

For high-throughput production workloads:

```bash
# Load VALE module
kldload vale

# Network config in veln.toml:
# network_type = "vale"
# network_device = "vale0"
```

See [doc/NETWORKING.md](doc/NETWORKING.md) for detailed networking documentation.

## Development

### Project Structure

```
veln/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── cli.rs            # Command definitions
│   ├── application.rs    # Command implementations
│   ├── config.rs         # Configuration loading
│   ├── error.rs          # Error types
│   ├── domain/           # Business logic
│   │   ├── vm.rs         # VM entity & state machine
│   │   ├── repository.rs # Storage traits
│   │   ├── requirements.rs # Host checks
│   │   └── resources.rs  # Resource monitoring
│   └── infrastructure/   # System integration
│       ├── zfs.rs        # ZFS storage
│       └── bhyve.rs      # bhyve runtime
├── doc/                  # Documentation
├── Cargo.toml
└── justfile
```

### Running Tests

```bash
# Unit tests
cargo test

# Full QA (format, clippy, test)
just qa
```

## Roadmap

- [ ] VM snapshots (ZFS-based)
- [ ] ISO/image management
- [ ] VM cloning and templates
- [ ] VNC console support
- [ ] Cloud-init integration
- [ ] REST API
- [ ] Web UI

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `just qa` to verify
5. Submit a pull request

## License

BSD-3-Clause License - See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built for FreeBSD and the bhyve hypervisor
- Inspired by Proxmox, SmartOS, and other virtualization platforms
- Thanks to the FreeBSD community for excellent documentation

## Resources

- [FreeBSD Handbook - Virtualization](https://docs.freebsd.org/en/books/handbook/virtualization/)
- [bhyve(8) man page](https://man.freebsd.org/cgi/man.cgi?query=bhyve)
- [ZFS on FreeBSD](https://docs.freebsd.org/en/books/handbook/zfs/)

## Support

- Report issues: [GitHub Issues](https://github.com/vessaix/veln/issues)
- FreeBSD Forums: [forums.freebsd.org](https://forums.freebsd.org/)
- Documentation: See `doc/` directory

---

**Note**: Veln requires root privileges for VM operations as it manages system resources, kernel modules, and ZFS datasets.
