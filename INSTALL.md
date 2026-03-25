# Veln Installation

This directory contains multiple ways to install veln on FreeBSD.

## Quick Install Options

### Option 1: Install Script (Quickest)

Install from GitHub source directly:

```bash
# Download and run install script
curl -sSL https://raw.githubusercontent.com/vessaix/veln/main/install.sh | sh

# Or with custom prefix
curl -sSL https://raw.githubusercontent.com/vessaix/veln/main/install.sh | PREFIX=/opt/veln sh

# Or clone and install
git clone https://github.com/vessaix/veln.git
cd veln
./install.sh
```

**Requirements:** Git, Cargo (Rust)

### Option 2: FreeBSD Port (Official)

Install as a proper FreeBSD port:

```bash
# Method A: From GitHub release (standard)
cd /usr/ports/sysutils/veln
make install clean

# Method B: From local source (development)
cd /usr/ports/sysutils/veln
make LOCAL_SOURCE=on WRKSRC=/path/to/veln install clean
```

**Requirements:** FreeBSD ports tree

### Option 3: Binary Package (Future)

```bash
# When available in FreeBSD packages
pkg install veln
```

## Installation Methods Comparison

| Method | Best For | Pros | Cons |
|--------|----------|------|------|
| `install.sh` | Quick testing | Fast, no ports tree needed | No automatic updates |
| Port (GitHub) | Production | Standard FreeBSD package, updates | Needs ports tree |
| Port (local) | Development | Build from local changes | Manual build process |
| pkg (future) | Users | Easiest, automatic updates | Not yet available |

## Build from Source Manually

```bash
# Clone repository
git clone https://github.com/vessaix/veln.git
cd veln

# Build release
cargo build --release

# Install manually
cp target/release/veln /usr/local/bin/
```

## Post-Installation

### Configure veln

Create a configuration file:

```bash
cat > /usr/local/etc/veln.toml << 'EOF'
zfs_pool = "zroot"
vm_root = "/usr/local/vms"
EOF
```

### Enable API Server (Optional)

```bash
# For install.sh method
cp rc.d/veln /usr/local/etc/rc.d/
echo 'veln_enable="YES"' >> /etc/rc.conf
service veln start

# For port method
make install WITH=API
```

### Verify Installation

```bash
veln --version
veln check
veln list
```

## Uninstall

```bash
# install.sh method
rm /usr/local/bin/veln
rm /usr/local/etc/veln.toml  # if created

# port method
cd /usr/ports/sysutils/veln && make deinstall

# With API
rm /usr/local/etc/rc.d/veln
```

## Troubleshooting

### Missing Dependencies

```bash
# Install Rust (required for all source builds)
pkg install rust

# For port builds
pkg install portmaster portlint
```

### Permission Issues

```bash
# Ensure binary is executable
chmod +x /usr/local/bin/veln

# Check ZFS permissions
zpool list
zfs list
```

## Contributing

To contribute to veln:

1. Fork the repository
2. Make changes in your local copy
3. Test with local source method:
   ```bash
   cd /usr/ports/sysutils/veln
   make LOCAL_SOURCE=on WRKSRC=/path/to/your/fork install
   ```
4. Submit a pull request

## See Also

- [FreeBSD Porter's Handbook](https://docs.freebsd.org/en/books/porters-handbook/book/)
- [Veln README](../README.md)
- [Veln API Documentation](../doc/API.md)
