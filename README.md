# Veln - FreeBSD Virtualization Manager

A modern, lightweight CLI for managing FreeBSD bhyve virtual machines with ZFS integration.

**📖 [Full Documentation →](docs/README.md)**

## Quick Start

```bash
# Clone and install
git clone https://github.com/vessaix/veln.git
cd veln
./scripts/install.sh

# Create your first VM
sudo veln create myvm
sudo veln start myvm
```

## Documentation

- **[README.md](docs/README.md)** - Project overview, features, installation
- **[INSTALL.md](docs/INSTALL.md)** - Detailed installation guide
- **[API.md](docs/API.md)** - REST API documentation
- **[DOMAIN.md](docs/DOMAIN.md)** - Domain architecture
- **[INFRASTRUCTURE.md](docs/INFRASTRUCTURE.md)** - Infrastructure details
- **[NETWORKING.md](docs/NETWORKING.md)** - Networking architecture
- **[RBAC_PLAN.md](docs/RBAC_PLAN.md)** - Authentication & authorization roadmap
- **[TODO.md](docs/TODO.md)** - Development progress

## Features

### Core
- VM lifecycle management (create, start, stop, destroy)
- ZFS integration with snapshots and cloning
- ISO management and cloud-init support
- REST API with role-based authentication

### Web UI
- Modern Vue 3 + TypeScript frontend
- Real-time VM monitoring
- Role-based access control (Admin, Operator, Viewer)
- Dark theme with Kinetic Engine design

## Authentication

VELN uses API key authentication for the REST API and Web UI:

```toml
# /usr/local/etc/veln/config.toml
[api]
auth_enabled = true

[api.keys]
"your-api-key" = { name = "Admin", role = "admin" }
```

**Roles:**
- **Admin** - Full access
- **Operator** - Start/stop VMs, view all
- **Viewer** - Read-only access

See [RBAC_PLAN.md](docs/RBAC_PLAN.md) for full details.

## License

BSD-3-Clause - See [LICENSE](LICENSE)
