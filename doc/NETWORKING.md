# Veln Networking Architecture

## Overview

Veln supports two networking backends for VM connectivity:
1. **TAP + Bridge** (default) - Simple, well-understood
2. **Netgraph + VALE** (advanced) - High performance, flexible

## TAP + Bridge (Default)

### Architecture
```
VM → virtio-net → TAP device (/dev/tapX) → bridge0 → physical NIC
```

### How it works
- bhyve creates a TAP interface for each VM
- TAP interface joins a bridge (e.g., `bridge0`)
- Bridge connects to physical network
- Standard FreeBSD networking applies

### Configuration
```toml
# veln.toml - Default TAP + Bridge
network_type = "tap"
network_device = "bridge0"
```

### Pros
- **Simple**: Standard FreeBSD networking
- **Well-documented**: Abundant examples online
- **Firewall-friendly**: Works with pf, ipfw
- **Stable**: Battle-tested for years
- **Easy troubleshooting**: ifconfig, tcpdump work normally

### Cons
- **Context switches**: TAP involves kernel/userland transitions
- **Limited flexibility**: One bridge per interface
- **Performance**: Good for most workloads, but not bare-metal speed

### When to use
- Standard VM workloads
- Development environments
- When simplicity matters
- Most use cases

### Setup Commands
```bash
# Load required module
kldload if_tuntap

# Create bridge
ifconfig bridge0 create
ifconfig bridge0 addm em0      # Add physical interface
ifconfig bridge0 up

# VMs will auto-create TAP interfaces
```

## Netgraph + VALE (Advanced)

### Architecture
```
VM → virtio-net → ng_bhyve → VALE switch → physical NIC (via netmap)
```

### How it works
- Uses FreeBSD's netgraph framework
- VALE provides in-kernel switching
- Near bare-metal performance
- Supports complex topologies

### Configuration
```toml
# veln.toml - VALE Switch
network_type = "vale"
network_device = "vale0"
```

### Pros
- **High performance**: Kernel bypass, minimal overhead
- **Flexible**: Complex network topologies
- **VLAN support**: Native VLAN handling
- **Micro-segmentation**: Multiple isolated switches
- **Production-grade**: Used by SmartOS/Illumos

### Cons
- **Complex**: Steeper learning curve
- **Debugging harder**: Less familiar tools
- **Documentation**: Fewer FreeBSD-specific examples
- **Setup**: More initial configuration

### When to use
- High-throughput workloads (10Gbps+)
- Production environments
- Complex network topologies
- When performance is critical

### Setup Commands
```bash
# Load required modules
kldload netgraph
kldload ng_bhyve
kldload vale

# VALE switch is created automatically when first used
# VMs connect via netgraph hooks
```

## Comparison Table

| Feature | TAP + Bridge | VALE |
|---------|--------------|------|
| **Complexity** | Low | High |
| **Performance** | Good | Excellent |
| **Learning Curve** | Easy | Steep |
| **Documentation** | Extensive | Moderate |
| **Debugging** | Easy | Harder |
| **Firewall** | Native | Via ng_ipfw |
| **VLANs** | Via vlang | Native |
| **Production Use** | Common | Specialized |

## Choosing the Right Backend

### Use TAP + Bridge if:
- You're getting started with bhyve
- Running typical workloads (web servers, databases)
- Need simple firewall rules
- Want familiar networking tools
- Performance requirements are moderate (< 10Gbps)

### Use VALE if:
- Running high-throughput applications
- Need micro-segmentation
- Building complex network topologies
- Performance is critical
- You're comfortable with netgraph
- Familiar with SmartOS/Illumos networking

## Migration Between Backends

To change a VM's network backend:

1. Stop the VM:
   ```bash
   veln stop myvm
   ```

2. Edit the VM configuration:
   ```bash
   # Edit /dev/zvol/<pool>/veln/myvm/veln.toml
   # Change network_type from "tap" to "vale" (or vice versa)
   # Update network_device as needed
   ```

3. Start the VM:
   ```bash
   veln start myvm
   ```

## Implementation in Veln

The networking backend is specified in `veln.toml`:

```toml
# TAP + Bridge (default)
uuid = "550e8400-e29b-41d4-a716-446655440000"
name = "myvm"
network_type = "tap"
network_device = "bridge0"
mac = "58:9c:fc:01:02:03"

# Or VALE switch
network_type = "vale"
network_device = "vale0"
```

Veln automatically generates the correct bhyve arguments based on the backend type.

## Future Enhancements

Potential future networking features:
- **SR-IOV**: Hardware NIC passthrough for best performance
- **OVSwitch**: Open vSwitch integration
- **VPN**: WireGuard/OpenVPN integration for secure remote access
- **IPv6**: First-class IPv6 support
- **Network Policies**: Firewall rules per VM

## References

- FreeBSD Handbook - Network Configuration: https://docs.freebsd.org/en/books/handbook/network/
- bhyve(8) man page: https://man.freebsd.org/cgi/man.cgi?query=bhyve
- netgraph(4) man page: https://man.freebsd.org/cgi/man.cgi?query=netgraph
- VALE paper: http://info.iet.unipi.it/~luigi/vale/
- SmartOS Networking: https://docs.smartos.org/networking/
