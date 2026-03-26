# Veln Infrastructure Module

## Overview

Infrastructure layer implements the domain traits using FreeBSD system tools.

## ZFS Repository (`zfs.rs`)

### Design
- Each VM is a ZFS dataset under `<pool>/<vm_root>/<name>`
- VM configuration stored as `veln.toml` in the dataset
- Uses `zfs(8)` command-line tool for all operations

### Dataset Structure
```
zroot/
└── veln/
    └── myvm/
        ├── veln.toml          # VM configuration (includes UUID)
        └── disk               # ZVOL (block device) created by bhyve runtime
```

The disk is a ZVOL (ZFS volume) accessible at `/dev/zvol/zroot/veln/myvm/disk`

### veln.toml Format
```toml
uuid = "550e8400-e29b-41d4-a716-446655440000"
name = "myvm"
cpus = 2
memory_mb = 1024
disk_gb = 20
network_type = "tap"              # or "vale" for high-performance
network_device = "bridge0"        # or "vale0" for VALE switch
mac = "58:9c:fc:01:02:03"
```

**UUID**: Unique identifier that never changes, even if VM is renamed

**Network Backend**: Supports both TAP+bridge (simple) and VALE (high-performance). See [NETWORKING.md](NETWORKING.md) for details.

### Operations
- **create**: `zfs create <pool>/veln/<name>`
- **destroy**: `zfs destroy -r <pool>/veln/<name>`
- **list**: `zfs list -r <pool>/veln`
- **config**: Read/write `veln.toml` from mountpoint

### References
- FreeBSD Handbook - ZFS: <https://docs.freebsd.org/en/books/handbook/zfs/>
- zfs(8) man page

## bhyve Runtime (`bhyve.rs`)

### Design
- Manages bhyve processes for VM lifecycle
- Creates ZVOLs (ZFS volumes) using `zfs create -V`
- Generates bhyve command-line arguments dynamically
- Uses `bhyve(8)`, `bhyveload(8)`, and `bhyvectl(8)` for control

### VM Configuration Generated
- CPUs: `-c <count>`
- Memory: `-m <MB>`
- Disk: `-s 3:0,virtio-blk,<path>`
- Network: `-s 2:0,virtio-net,<bridge>,<mac>`
- Console: `-s 1:0,lpc` + `-l com1,/dev/nmdm0A`

### Operations
- **create**: `zfs create -V<size>G -o volmode=dev <dataset>/disk`
- **start**: `bhyveload` then `bhyve`
- **stop**: `bhyvectl --acpi-poweroff`
- **destroy**: `bhyvectl --force-poweroff --destroy`
- **status**: `pgrep -f "bhyve.*<name>"`

### ZVOL Benefits
- Block device interface (better performance than files)
- Native ZFS integration (snapshots, compression, etc.)
- Exposed as `/dev/zvol/<pool>/<dataset>/disk`

### References
- FreeBSD Handbook - bhyve: <https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve>
- bhyve(8) man page
- bhyvectl(8) man page
- bhyveload(8) man page

## VM Lifecycle Commands

### create
Creates VM configuration in ZFS and reserves resources.

### start
Creates ZVOL disk if needed, then launches bhyve process.

### stop
Graceful shutdown via ACPI, or force destroy with `--force`.

### destroy
Stops running VM, then deletes ZFS dataset and all data.
Requires `--yes` confirmation flag.

### console
Attaches to VM serial console using `cu` command.
Uses nmdm (null modem) devices: `/dev/nmdm<name><device>A`
Press `~.` to disconnect from console.

### status
Shows current state of a VM including:
- VM name and UUID
- Current state (Stopped, Running, etc.)
- Whether bhyve process is running
- Configuration details with `--verbose` flag

## Integration

The application layer combines both:
- `ZfsRepository` persists VM configuration
- `BhyveRuntime` manages running processes and ZVOLs
- Commands use both for complete VM lifecycle

## REST API (`api.rs`)

### Design
- HTTP server using `rouille` framework
- JSON request/response format
- CORS support for browser clients
- Synchronous request handling (no async/await)

### Endpoints

#### System
- `GET /api/health` - Health check
- `GET /api/info` - System information

#### VMs
- `GET /api/vms` - List all VMs
- `GET /api/vms/{name}` - Get VM details
- `POST /api/vms` - Create VM (JSON body)
- `POST /api/vms/{name}/start` - Start VM
- `POST /api/vms/{name}/stop` - Stop VM
- `DELETE /api/vms/{name}` - Delete VM

### HTTP Status Codes
- `200` - Success
- `201` - Created (VM created)
- `400` - Bad Request (invalid JSON)
- `404` - Not Found (VM doesn't exist)
- `409` - Conflict (VM already exists)
- `422` - Unprocessable Entity (validation failed)
- `500` - Internal Server Error
- `503` - Service Unavailable (bhyve not ready)

### Security Considerations
- No built-in authentication (use reverse proxy)
- No HTTPS support (use reverse proxy with SSL/TLS)
- Designed for localhost/trusted networks

### References
- See [API.md](API.md) for full API documentation

## Related Documentation

- [NETWORKING.md](NETWORKING.md) - Detailed networking architecture and backend options
- [API.md](API.md) - Complete REST API documentation
