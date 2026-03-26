#!/bin/sh
#
# Veln Installation Script
# Supports: source install, package creation, and local package repo
# Usage: ./scripts/install.sh [OPTIONS]
#
# Options:
#   --prefix=PATH       Install prefix (default: /usr/local)
#   --package           Create a .pkg package instead of installing
#   --package-dir=DIR   Directory for created package (default: ./packages)
#   --local-repo        Create/update local package repository
#   --repo-dir=DIR      Local repo directory (default: /usr/local/poudriere/veln-repo)
#   --manifest=FILE     Save installation manifest for clean uninstall
#   --help              Show this help

set -e

# Configuration
PREFIX=${PREFIX:-/usr/local}
PACKAGE_MODE=0
PACKAGE_DIR="./packages"
LOCAL_REPO=0
REPO_DIR="/usr/local/poudriere/veln-repo"
MANIFEST_FILE="${PREFIX}/var/db/veln/manifest"
SRC_DIR=""
LOCAL_SRC=0
REPO_URL="https://github.com/vessaix/veln.git"

# Colors for output (if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

# Logging functions
log_info() {
    echo "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo "${RED}[ERROR]${NC} $1" >&2
}

# Help message
show_help() {
    grep "^#" "$0" | sed 's/^# //' | head -25
    exit 0
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --package)
            PACKAGE_MODE=1
            shift
            ;;
        --package-dir=*)
            PACKAGE_DIR="${1#*=}"
            shift
            ;;
        --local-repo)
            LOCAL_REPO=1
            shift
            ;;
        --repo-dir=*)
            REPO_DIR="${1#*=}"
            shift
            ;;
        --manifest=*)
            MANIFEST_FILE="${1#*=}"
            shift
            ;;
        --help|-h)
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check dependencies
check_dep() {
    if ! command -v "$1" >/dev/null 2>&1; then
        log_error "$1 is required but not installed."
        exit 1
    fi
}

# Get veln version from Cargo.toml
get_version() {
    if [ -f Cargo.toml ]; then
        grep "^version" Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/'
    else
        echo "0.1.0"
    fi
}

# Create manifest entry
add_to_manifest() {
    local file="$1"
    local type="$2"
    local checksum=""
    
    if [ -f "$file" ]; then
        checksum=$(sha256 "$file" 2>/dev/null || echo "unknown")
    fi
    
    echo "${type}|${file}|${checksum}" >> "$MANIFEST_FILE"
}

# Create directories
setup_dirs() {
    if [ "$PACKAGE_MODE" -eq 0 ]; then
        # Regular install - create system directories
        mkdir -p "$PREFIX/bin"
        mkdir -p "$PREFIX/etc"
        mkdir -p "$PREFIX/share/man/man1"
        mkdir -p "$PREFIX/etc/rc.d"
        mkdir -p "$(dirname "$MANIFEST_FILE")"
    else
        # Package mode - create staging directory
        PKG_STAGING="${PACKAGE_DIR}/staging/veln-$(get_version)"
        rm -rf "$PKG_STAGING"
        mkdir -p "$PKG_STAGING"
        mkdir -p "$PKG_STAGING/bin"
        mkdir -p "$PKG_STAGING/etc"
        mkdir -p "$PKG_STAGING/share/man/man1"
        mkdir -p "$PKG_STAGING/etc/rc.d"
        log_info "Package staging directory: $PKG_STAGING"
    fi
}

# Install files
install_files() {
    local dest_prefix="$1"
    local is_package="$2"
    
    log_info "Installing veln binary..."
    install -m 755 target/release/veln "${dest_prefix}/bin/veln"
    
    if [ "$is_package" -eq 0 ]; then
        add_to_manifest "${dest_prefix}/bin/veln" "bin"
    fi
    
    # Install config example
    if [ -f veln.toml.example ]; then
        log_info "Installing config example..."
        install -m 644 veln.toml.example "${dest_prefix}/etc/veln.toml.example"
        if [ "$is_package" -eq 0 ]; then
            add_to_manifest "${dest_prefix}/etc/veln.toml.example" "config"
        fi
    fi
    
    # Install man page if exists
    if [ -f docs/veln.1 ]; then
        log_info "Installing man page..."
        install -m 644 docs/veln.1 "${dest_prefix}/share/man/man1/veln.1"
        if [ "$is_package" -eq 0 ]; then
            add_to_manifest "${dest_prefix}/share/man/man1/veln.1" "man"
        fi
    fi
    
    # Install RC script on FreeBSD
    if [ -f port/sysutils/veln/files/veln.in ]; then
        log_info "Installing RC script..."
        sed -e "s|@PREFIX@|${PREFIX}|g" port/sysutils/veln/files/veln.in > "${dest_prefix}/etc/rc.d/veln"
        chmod 555 "${dest_prefix}/etc/rc.d/veln"
        if [ "$is_package" -eq 0 ]; then
            add_to_manifest "${dest_prefix}/etc/rc.d/veln" "rc"
        fi
    fi
    
    # Install uninstall script for use by 'veln tools --uninstall'
    if [ "$is_package" -eq 0 ]; then
        log_info "Installing uninstall script..."
        mkdir -p "${dest_prefix}/share/veln"
        install -m 755 scripts/uninstall.sh "${dest_prefix}/share/veln/uninstall.sh"
        add_to_manifest "${dest_prefix}/share/veln/uninstall.sh" "bin"
    fi
}

# Create FreeBSD package
create_package() {
    local version
    version=$(get_version)
    local pkg_name="veln-${version}.pkg"
    local pkg_path="${PACKAGE_DIR}/${pkg_name}"
    
    log_info "Creating FreeBSD package: $pkg_name"
    
    # Create package staging
    setup_dirs
    install_files "$PKG_STAGING" 1
    
    # Create +MANIFEST
    cat > "${PKG_STAGING}/+MANIFEST" << EOF
name: veln
version: "${version}"
origin: sysutils/veln
comment: "FreeBSD Virtualization Management CLI"
desc: <<EOD
Veln is a FreeBSD-native virtualization management tool built in Rust.
It provides a simple CLI for creating, managing, and running bhyve
virtual machines with automatic ZFS storage provisioning.

Features:
- VM lifecycle management (create, start, stop, destroy)
- ZFS integration with snapshots, clones, and ZVOLs
- ISO image management
- VM templates for rapid deployment
- Cloud-init integration
- REST API server
- VNC console support
EOD
maintainer: amr@vessaix.com
www: https://github.com/vessaix/veln
prefix: ${PREFIX}
deps: {
}
EOF
    
    # Create package
    mkdir -p "$PACKAGE_DIR"
    if command -v pkg >/dev/null 2>&1; then
        pkg create -M "${PKG_STAGING}/+MANIFEST" -r "$PKG_STAGING" -o "$PACKAGE_DIR"
        log_info "Package created: ${pkg_path}"
        
        # Show package info
        echo ""
        log_info "Package details:"
        pkg info -F "${pkg_path}"
        
        # Offer to install
        echo ""
        read -p "Install package now? [y/N]: " install_now
        if [ "$install_now" = "y" ] || [ "$install_now" = "Y" ]; then
            log_info "Installing package..."
            pkg add "${pkg_path}"
            log_info "Package installed successfully!"
            log_info "You can now use: pkg remove veln"
        else
            echo ""
            log_info "To install manually:"
            echo "  pkg add ${pkg_path}"
            echo ""
            log_info "To uninstall:"
            echo "  pkg remove veln"
        fi
    else
        log_warn "pkg(8) not found. Creating tarball instead..."
        tar -czf "${pkg_path%.pkg}.txz" -C "$PKG_STAGING" .
        log_info "Archive created: ${pkg_path%.pkg}.txz"
    fi
    
    # Cleanup staging
    rm -rf "${PACKAGE_DIR}/staging"
}

# Create local package repository
setup_local_repo() {
    log_info "Setting up local package repository..."
    
    # Create repo directory
    mkdir -p "$REPO_DIR"
    
    # Create package if it doesn't exist
    local version
    version=$(get_version)
    local pkg_name="veln-${version}.pkg"
    
    if [ ! -f "${PACKAGE_DIR}/${pkg_name}" ]; then
        log_info "Package not found, creating..."
        create_package
    fi
    
    # Copy package to repo
    cp "${PACKAGE_DIR}/${pkg_name}" "${REPO_DIR}/"
    
    # Create repo metadata
    if command -v pkg >/dev/null 2>&1; then
        log_info "Creating package repository..."
        pkg repo "$REPO_DIR"
        
        log_info "Repository created at: $REPO_DIR"
        echo ""
        log_info "To use this repository, add to /usr/local/etc/pkg/repos/veln.conf:"
        cat << 'EOF'
veln: {
    url: "file:///usr/local/poudriere/veln-repo",
    enabled: yes
}
EOF
        echo ""
        log_info "Then run:"
        echo "  pkg update"
        echo "  pkg install veln"
    else
        log_warn "pkg(8) not found, skipping repo creation"
    fi
}

# Main installation
main() {
    log_info "Veln Installer"
    log_info "=============="
    echo ""
    
    # Detect source directory
    if [ -f Cargo.toml ] && [ -d src ]; then
        SRC_DIR="$(pwd)"
        LOCAL_SRC=1
        log_info "Using local source: $SRC_DIR"
    else
        SRC_DIR="/tmp/veln-src-$$"
        log_info "Cloning from $REPO_URL..."
        check_dep git
        git clone "$REPO_URL" "$SRC_DIR"
        cd "$SRC_DIR"
    fi
    
    # Check dependencies
    check_dep cargo
    
    # Build
    log_info "Building release binary..."
    cargo build --release
    
    # Handle different modes
    if [ "$PACKAGE_MODE" -eq 1 ]; then
        create_package
    elif [ "$LOCAL_REPO" -eq 1 ]; then
        setup_local_repo
    else
        # Regular install
        log_info "Installing to: $PREFIX"
        setup_dirs
        
        # Create new manifest
        mkdir -p "$(dirname "$MANIFEST_FILE")"
        echo "# Veln Installation Manifest" > "$MANIFEST_FILE"
        echo "# Created: $(date)" >> "$MANIFEST_FILE"
        echo "# Version: $(get_version)" >> "$MANIFEST_FILE"
        echo "# Prefix: $PREFIX" >> "$MANIFEST_FILE"
        echo "" >> "$MANIFEST_FILE"
        
        install_files "$PREFIX" 0
        
        # Save metadata
        echo "" >> "$MANIFEST_FILE"
        echo "# Directories created" >> "$MANIFEST_FILE"
        add_to_manifest "$(dirname "$MANIFEST_FILE")" "dir"
        
        log_info "Installation complete!"
        echo ""
        log_info "To uninstall cleanly:"
        echo "  veln tools --uninstall"
        echo ""
        log_info "Or manually:"
        echo "  cat $MANIFEST_FILE"
        echo ""
        log_info "Quick start:"
        echo "  veln check"
        echo "  veln --help"
    fi
    
    # Cleanup if we cloned
    if [ "$LOCAL_SRC" -eq 0 ]; then
        log_info "Cleaning up temporary files..."
        rm -rf "$SRC_DIR"
    fi
    
    log_info "Done!"
}

# Run main
main
