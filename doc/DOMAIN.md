# Veln Domain Module

## References

### FreeBSD Handbook - Virtualization
- **URL**: https://docs.freebsd.org/en/books/handbook/virtualization/
- **Purpose**: Primary reference for bhyve architecture and management
- **Sections Used**:
  - Chapter 22.2: Hardware Requirements
  - Chapter 22.4: Loading Required Modules
  - Chapter 22.7: Managing Virtual Machines
  - Chapter 22.7.4: Console Configuration

### bhyve Host Requirements
- **URL**: https://docs.freebsd.org/en/books/handbook/virtualization/#virtualization-host-bhyve
- **Purpose**: Automatic host capability checking before VM operations
- **Checks Implemented**:
  - CPU virtualization support (VT-x/AMD-V)
  - Kernel modules (vmm, if_tuntap, nmdm)
  - VMM subsystem initialization
  - TUN/TAP interface availability

## Module Structure

### vm.rs
- VM state machine (Undefined → Stopped → Starting → Running → Stopping → Stopped)
- VM entity with UUID and configuration
- UUID v4 generation for unique VM identification
- State transition validation
- Network backend abstraction (TAP+bridge or VALE)

### requirements.rs
- `BhyveRequirements`: Static checks for host capability
- `RequirementsChecker`: Automatic verification before operations

### resources.rs
- `ResourceMonitor`: Dynamic resource tracking
- `HostResources`: System capacity information
- Prevents resource exhaustion during VM creation

### repository.rs
- `VmRepository`: Trait for VM persistence (ZFS implementation in infrastructure layer)
- `VmRuntime`: Trait for VM lifecycle (bhyve implementation in infrastructure layer)

## Design Decisions

1. **Domain-first architecture**: Business logic separated from infrastructure
2. **Automatic checks**: Requirements verified before any VM operation
3. **Resource safety**: Buffer reserved for host OS (512MB RAM, 1 core, 10% disk)
4. **State machine**: Enforces valid VM lifecycle transitions
5. **Test coverage**: Unit tests for state transitions, UUID generation, and configuration

## Testing

### Unit Tests (`src/domain/vm.rs`)
- State machine transition validation (valid and invalid transitions)
- UUID v4 generation and uniqueness
- VM configuration defaults
- State transition enforcement

Run tests with: `cargo test`
