# Veln Installation Guide

This guide covers all installation methods for veln on FreeBSD, from quick testing to production deployment.

## Quick Reference

```bash
# Quickest: Try veln without installing
./scripts/install.sh --package && pkg add ./packages/veln-*.pkg

# Recommended: Install from source with tracking
./scripts/install.sh

# Production: Create local package repository
./scripts/install.sh --local-repo

# Then use pkg:
pkg install veln
pkg remove veln
```

---

## Installation Methods

### Method 1: Package Installation (Recommended for Testing)

Create a FreeBSD package and install it. Easy to remove later.

```bash
# Create package
./scripts/install.sh --package

# Install it
pkg add ./packages/veln-*.pkg

# Later, uninstall cleanly:
pkg remove veln
```

**Pros:** Clean uninstall via `pkg remove`, can redistribute the .pkg file  
**Cons:** Must be rebuilt for updates

### Method 2: Source Installation with Tracking (Recommended for Development)

Install from source with full tracking for clean uninstall.

```bash
# Install with tracking
./scripts/install.sh

# Or with just:
just install
```

This creates a manifest at `/usr/local/var/db/veln/manifest` that tracks every file installed.

**Uninstall cleanly:**
```bash
# Using the installed uninstaller
/usr/local/bin/veln-uninstall

# Or with just:
just uninstall

# Dry run (see what would be removed without removing)
just uninstall-dry

# Purge everything including VMs and data
just uninstall-purge
```

**Pros:** Clean uninstall, no package needed, tracks all files  
**Cons:** Must use uninstall script (not pkg)

### Method 3: Local Package Repository (Recommended for Multiple Systems)

Create a local package repository that can be used with `pkg install`.

```bash
# Set up local repository
./scripts/install.sh --local-repo

# Or with just:
just pkg-repo
```

Then configure pkg to use it:

```bash
# Create repo config
sudo tee /usr/local/etc/pkg/repos/veln.conf << 'EOF'
veln: {
    url: "file:///usr/local/poudriere/veln-repo",
    enabled: yes
}
EOF

# Update and install
sudo pkg update
sudo pkg install veln

# Later, uninstall normally:
sudo pkg remove veln
```

**Pros:** Use standard pkg commands, easy updates, works across multiple systems  
**Cons:** Initial setup required

### Method 4: FreeBSD Port (Recommended for Production)

Install as a proper FreeBSD port (requires ports tree).

```bash
# Copy port to ports tree
cp -r port/sysutils/veln /usr/ports/sysutils/

# Build and install
cd /usr/ports/sysutils/veln
make install clean

# Or test from local source:
make LOCAL_SOURCE=on WRKSRC=/path/to/veln install
```

**Pros:** Official FreeBSD integration, automatic updates via ports  
**Cons:** Requires ports tree, slower to set up

### Method 5: Manual Build

Build and install manually (experts only).

```bash
# Build
cargo build --release

# Install manually
sudo cp target/release/veln /usr/local/bin/
sudo chmod +x /usr/local/bin/veln
```

**Note:** No automatic uninstall support with this method.

---

## Uninstallation

### Clean Uninstall (Source Install)

If you installed from source with the install script:

```bash
# Interactive uninstall
sudo /usr/local/bin/veln-uninstall

# Non-interactive (auto-yes)
sudo /usr/local/bin/veln-uninstall --yes

# Dry run (show what would be removed)
sudo /usr/local/bin/veln-uninstall --dry-run

# Or use just:
just uninstall
just uninstall-dry
```

### Package Uninstall

If you installed via pkg:

```bash
pkg remove veln
```

### Purge All Data

To remove veln AND all VMs, configs, and ZFS datasets:

```bash
# WARNING: This deletes everything!
sudo /usr/local/bin/veln-uninstall --purge

# Or with just:
just uninstall-purge
```

**This will remove:**
- veln binary and all installed files
- Configuration files (`/usr/local/etc/veln.toml`)
- ZFS datasets (pools/veln/*)
- VM storage directories
- RC scripts

---

## Installation Options

### Install Script Options

```bash
./scripts/install.sh [OPTIONS]

Options:
  --prefix=PATH       Install prefix (default: /usr/local)
  --package           Create a .pkg package instead of installing
  --package-dir=DIR   Directory for created package (default: ./packages)
  --local-repo        Create/update local package repository
  --repo-dir=DIR      Local repo directory (default: /usr/local/poudriere/veln-repo)
  --manifest=FILE     Save installation manifest for clean uninstall
  --help              Show help
```

### Examples

```bash
# Install to custom location
./scripts/install.sh --prefix=/opt/veln

# Create package in specific directory
./scripts/install.sh --package --package-dir=/tmp/packages

# Set up repo in custom location
./scripts/install.sh --local-repo --repo-dir=/var/packages/veln
```

---

## Using Just (Recommended)

If you have [just](https://github.com/casey/just) installed:

```bash
# Development
just check          # Run tests and checks
just test           # Run tests only
just qa             # Full QA suite

# Installation
just install                    # Install from source
just install-pkg                # Install and create package
just package                    # Create package only
just pkg-install                # Create and install package
just pkg-repo                   # Set up local repository

# Uninstallation
just uninstall                  # Clean uninstall
just uninstall-dry              # Dry run
just uninstall-purge            # Remove everything
just pkg-remove                 # Remove via pkg

# Status
just status                     # Show installation status
just help                       # Show all targets
```

---

## Requirements

### Build Requirements
- FreeBSD 13.0+ or 14.0+
- Rust 1.70+ with Cargo
- Git (for cloning)

### Runtime Requirements
- Root privileges (for VM operations)
- ZFS filesystem
- Kernel modules: `vmm`, `if_tuntap`, `nmdm`

### Optional
- `pkg` (for package management)
- `just` (for convenience commands)

---

## Troubleshooting

### Installation Fails

```bash
# Check dependencies
which cargo
which git

# Check permissions
ls -la /usr/local/bin/

# Try with explicit prefix
./scripts/install.sh --prefix=$HOME/.local
```

### Uninstall Fails

```bash
# Check if installed via pkg
pkg info veln

# If manifest is missing, manually remove:
sudo rm /usr/local/bin/veln
sudo rm /usr/local/etc/rc.d/veln
sudo rm -rf /usr/local/var/db/veln
```

### Permission Denied

```bash
# Ensure you're root for install/uninstall
sudo ./scripts/install.sh
sudo /usr/local/bin/veln-uninstall
```

### Package Won't Install

```bash
# Check if already installed
pkg info veln

# Force reinstall
pkg add -f ./packages/veln-*.pkg
```

---

## Comparison Table

| Method | Best For | Install | Uninstall | Updates |
|--------|----------|---------|-----------|---------|
| Package | Quick test | `pkg add` | `pkg remove` | Manual |
| Source + Tracking | Development | `./scripts/install.sh` | `veln-uninstall` | Re-run install |
| Local Repo | Multiple systems | `pkg install` | `pkg remove` | Update repo |
| FreeBSD Port | Production | `make install` | `make deinstall` | Port upgrade |

---

## Post-Installation

### Configure veln

```bash
# Create config file
sudo tee /usr/local/etc/veln.toml << 'EOF'
zfs_pool = "zroot"
vm_root = "/usr/local/vms"
EOF
```

### Enable API Server

```bash
# Enable at boot
sudo sysrc veln_enable="YES"

# Start now
sudo service veln start

# Check status
sudo service veln status
```

### First Run

```bash
# Check system readiness
sudo veln check

# List VMs (empty initially)
sudo veln list

# Get help
veln --help
```

---

## Support

- **GitHub:** https://github.com/vessaix/veln
- **Issues:** https://github.com/vessaix/veln/issues
- **Documentation:** See `README.md` and `doc/` directory
