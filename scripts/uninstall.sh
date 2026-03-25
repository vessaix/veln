#!/bin/sh
#
# Veln Uninstall Script
# Removes all veln components cleanly
# Usage: ./uninstall.sh [OPTIONS]
#
# Options:
#   --prefix=PATH       Installation prefix (default: /usr/local)
#   --manifest=FILE     Manifest file path (default: PREFIX/var/db/veln/manifest)
#   --purge             Remove all data including VMs and configs
#   --dry-run           Show what would be removed without removing
#   --yes               Don't ask for confirmation
#   --help              Show this help

set -e

# Configuration
PREFIX=${PREFIX:-/usr/local}
MANIFEST_FILE="${PREFIX}/var/db/veln/manifest"
PURGE=0
DRY_RUN=0
YES=0

# Colors
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Logging
log_info() {
    echo "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo "${RED}[ERROR]${NC} $1" >&2
}

log_dry() {
    echo "${BLUE}[DRY-RUN]${NC} $1"
}

# Help
show_help() {
    grep "^#" "$0" | sed 's/^# //' | head -20
    exit 0
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix=*)
            PREFIX="${1#*=}"
            MANIFEST_FILE="${PREFIX}/var/db/veln/manifest"
            shift
            ;;
        --manifest=*)
            MANIFEST_FILE="${1#*=}"
            shift
            ;;
        --purge)
            PURGE=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --yes|-y)
            YES=1
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

# Remove a file or directory
remove_item() {
    local item="$1"
    local type="$2"
    
    if [ "$DRY_RUN" -eq 1 ]; then
        log_dry "Would remove: $item"
        return
    fi
    
    if [ ! -e "$item" ]; then
        log_warn "Not found: $item"
        return
    fi
    
    case "$type" in
        dir)
            if [ -d "$item" ]; then
                # Only remove if empty
                if [ -z "$(ls -A "$item" 2>/dev/null)" ]; then
                    rmdir "$item" 2>/dev/null && log_info "Removed directory: $item"
                else
                    log_warn "Directory not empty (skipping): $item"
                fi
            fi
            ;;
        *)
            rm -f "$item" && log_info "Removed: $item"
            ;;
    esac
}

# Uninstall from manifest
uninstall_from_manifest() {
    if [ ! -f "$MANIFEST_FILE" ]; then
        log_error "Manifest file not found: $MANIFEST_FILE"
        log_error "Cannot perform clean uninstall. Manual removal required."
        exit 1
    fi
    
    log_info "Reading manifest: $MANIFEST_FILE"
    
    # Count items
    local count
    count=$(grep -v "^#" "$MANIFEST_FILE" | grep -v "^$" | wc -l)
    log_info "Found $count items to remove"
    echo ""
    
    # Process manifest in reverse order (directories last)
    local items
    items=$(grep -v "^#" "$MANIFEST_FILE" | grep -v "^$")
    
    # First pass: remove files
    echo "$items" | while IFS='|' read -r type file checksum; do
        if [ "$type" != "dir" ]; then
            remove_item "$file" "$type"
        fi
    done
    
    # Second pass: remove directories
    echo "$items" | while IFS='|' read -r type file checksum; do
        if [ "$type" = "dir" ]; then
            remove_item "$file" "dir"
        fi
    done
    
    # Remove manifest itself
    if [ "$DRY_RUN" -eq 0 ]; then
        rm -f "$MANIFEST_FILE"
        # Try to remove parent directories if empty
        rmdir "$(dirname "$MANIFEST_FILE")" 2>/dev/null || true
        rmdir "$(dirname "$(dirname "$MANIFEST_FILE")")" 2>/dev/null || true
    fi
}

# Uninstall pkg package
uninstall_pkg() {
    if command -v pkg >/dev/null 2>&1; then
        if pkg info veln >/dev/null 2>&1; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_dry "Would run: pkg remove veln"
            else
                log_info "Removing veln package..."
                if [ "$YES" -eq 1 ]; then
                    pkg remove -y veln
                else
                    pkg remove veln
                fi
            fi
            return 0
        fi
    fi
    return 1
}

# Purge data
purge_data() {
    if [ "$PURGE" -eq 0 ]; then
        return
    fi
    
    log_warn "PURGE MODE: Removing all veln data!"
    echo ""
    
    # Check for running VMs
    if command -v veln >/dev/null 2>&1; then
        local running_vms
        running_vms=$(veln list 2>/dev/null | grep -i running | wc -l || echo "0")
        if [ "$running_vms" -gt 0 ]; then
            log_warn "WARNING: $running_vms VM(s) are currently running!"
            if [ "$YES" -eq 0 ] && [ "$DRY_RUN" -eq 0 ]; then
                read -p "Stop all VMs and continue? [y/N]: " confirm
                if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
                    log_info "Aborted."
                    exit 0
                fi
            fi
        fi
    fi
    
    # Remove ZFS datasets
    log_info "Checking for veln ZFS datasets..."
    local zfs_pools
    zfs_pools=$(zpool list -H -o name 2>/dev/null || true)
    
    for pool in $zfs_pools; do
        local datasets
        datasets=$(zfs list -H -o name -r "${pool}/veln" 2>/dev/null || true)
        for ds in $datasets; do
            if [ "$DRY_RUN" -eq 1 ]; then
                log_dry "Would destroy ZFS dataset: $ds"
            else
                log_warn "Destroying ZFS dataset: $ds"
                zfs destroy -r "$ds" 2>/dev/null || true
            fi
        done
    done
    
    # Remove config files
    local configs="/usr/local/etc/veln.toml /usr/local/etc/veln.toml.example"
    for cfg in $configs; do
        if [ -f "$cfg" ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_dry "Would remove config: $cfg"
            else
                rm -f "$cfg"
                log_info "Removed config: $cfg"
            fi
        fi
    done
    
    # Remove data directories
    local data_dirs="/usr/local/vms /var/db/veln"
    for dir in $data_dirs; do
        if [ -d "$dir" ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_dry "Would remove data directory: $dir"
            else
                rm -rf "$dir"
                log_info "Removed data directory: $dir"
            fi
        fi
    done
}

# Main uninstall
main() {
    echo "${RED}"
    echo "========================================"
    echo "  Veln Uninstaller"
    echo "========================================"
    echo "${NC}"
    echo ""
    
    if [ "$DRY_RUN" -eq 1 ]; then
        log_warn "DRY RUN MODE - No files will be removed"
        echo ""
    fi
    
    # Confirmation
    if [ "$YES" -eq 0 ] && [ "$DRY_RUN" -eq 0 ]; then
        echo "This will remove veln from your system."
        if [ "$PURGE" -eq 1 ]; then
            echo "${RED}PURGE MODE: ALL DATA INCLUDING VMs WILL BE REMOVED!${NC}"
        fi
        echo ""
        read -p "Are you sure? [y/N]: " confirm
        if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
            log_info "Aborted."
            exit 0
        fi
    fi
    
    echo ""
    
    # Try pkg first
    if uninstall_pkg; then
        log_info "Package removed via pkg"
    else
        # Fall back to manifest-based uninstall
        if [ -f "$MANIFEST_FILE" ]; then
            uninstall_from_manifest
        else
            log_warn "No manifest found, trying manual detection..."
            
            # Manual fallback
            local files="${PREFIX}/bin/veln ${PREFIX}/bin/veln-uninstall"
            for f in $files; do
                remove_item "$f" "bin"
            done
            
            remove_item "${PREFIX}/etc/rc.d/veln" "rc"
            remove_item "${PREFIX}/etc/veln.toml.example" "config"
            remove_item "${PREFIX}/share/man/man1/veln.1" "man"
        fi
    fi
    
    # Purge data if requested
    purge_data
    
    echo ""
    log_info "Uninstall complete!"
    
    if [ "$DRY_RUN" -eq 1 ]; then
        echo ""
        log_info "This was a dry run. No files were actually removed."
        echo "Run without --dry-run to actually uninstall."
    fi
}

main
