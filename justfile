default: check test

check:
    cargo check
    cargo clippy -- -D warnings

test:
    cargo test

qa: fmt check test
    @echo "All checks passed."

run *ARGS:
    sudo cargo run -- {{ARGS}}

fmt:
    cargo fmt --all

audit:
    cargo audit

# ============================================================================
# Installation Targets
# ============================================================================

# Install veln from source with full tracking (recommended)
install prefix="/usr/local":
    @echo "Installing veln from source..."
    ./install.sh --prefix={{prefix}}

# Install and create package
install-pkg prefix="/usr/local":
    @echo "Installing veln and creating package..."
    ./install.sh --prefix={{prefix}} --package

# Create package only (don't install)
package:
    @echo "Creating FreeBSD package..."
    ./install.sh --package --package-dir=./packages
    @echo ""
    @echo "Package created in: ./packages/"
    @ls -lh ./packages/*.pkg 2>/dev/null || true

# Create and install package
pkg-install:
    @echo "Creating and installing package..."
    ./install.sh --package --package-dir=./packages
    @pkg add ./packages/veln-*.pkg

# Create local package repository
pkg-repo:
    @echo "Setting up local package repository..."
    ./install.sh --local-repo --repo-dir=/usr/local/poudriere/veln-repo
    @echo ""
    @echo "Add to /usr/local/etc/pkg/repos/veln.conf:"
    @echo 'veln: {'
    @echo '    url: "file:///usr/local/poudriere/veln-repo",'
    @echo '    enabled: yes'
    @echo '}'
    @echo ""
    @echo "Then run: pkg update && pkg install veln"

# ============================================================================
# Uninstallation Targets
# ============================================================================

# Uninstall veln cleanly (removes everything that was installed)
uninstall prefix="/usr/local":
    @echo "Uninstalling veln..."
    ./scripts/uninstall.sh --prefix={{prefix}}

# Show what would be uninstalled (dry run)
uninstall-dry prefix="/usr/local":
    @echo "Showing what would be uninstalled..."
    ./scripts/uninstall.sh --prefix={{prefix}} --dry-run

# Uninstall and remove all data (VMs, configs, etc.)
uninstall-purge prefix="/usr/local":
    @echo "WARNING: This will remove veln and ALL DATA including VMs!"
    @read -p "Continue? [y/N]: " confirm && [ "$$confirm" = "y" ] || exit 0
    ./scripts/uninstall.sh --prefix={{prefix}} --purge --yes

# Uninstall if installed via pkg
pkg-remove:
    @echo "Removing veln package..."
    pkg remove veln

# ============================================================================
# Port Testing Targets
# ============================================================================

port-test:
    @echo "Testing port from local source..."
    make -C port/sysutils/veln LOCAL_SOURCE=on WRKSRC=$(PWD) stage

port-install:
    @echo "Installing from port..."
    cd port/sysutils/veln && make LOCAL_SOURCE=on WRKSRC=$(PWD) install

port-clean:
    @echo "Cleaning port work directory..."
    cd port/sysutils/veln && make clean

# ============================================================================
# Development Helpers
# ============================================================================

clean-all:
    @echo "Cleaning build artifacts..."
    cargo clean
    rm -rf packages/
    rm -rf port/sysutils/veln/work/
    @echo "Done!"

# Show installation status
status:
    @echo "=== Veln Installation Status ==="
    @echo ""
    @if [ -f /usr/local/bin/veln ]; then \
        echo "✓ Binary installed: /usr/local/bin/veln"; \
        /usr/local/bin/veln --version 2>/dev/null || true; \
    else \
        echo "✗ Binary not installed"; \
    fi
    @echo ""
    @if [ -f /usr/local/var/db/veln/manifest ]; then \
        echo "✓ Manifest exists: /usr/local/var/db/veln/manifest"; \
    else \
        echo "✗ No manifest (not installed from source)"; \
    fi
    @echo ""
    @if pkg info veln >/dev/null 2>&1; then \
        echo "✓ Package installed via pkg"; \
        pkg info veln 2>/dev/null | head -3; \
    else \
        echo "✗ Not installed via pkg"; \
    fi
    @echo ""
    @if [ -f ./packages/veln-*.pkg ]; then \
        echo "✓ Package built locally:"; \
        ls -lh ./packages/veln-*.pkg 2>/dev/null | awk '{print "  " $$9 " (" $$5 ")"}'; \
    else \
        echo "✗ No local package found"; \
    fi

# Help
help:
    @echo "Veln Makefile Targets"
    @echo "===================="
    @echo ""
    @echo "Development:"
    @echo "  just check          - Run cargo check and clippy"
    @echo "  just test           - Run tests"
    @echo "  just qa             - Run full QA (fmt, check, test)"
    @echo "  just run <args>     - Run veln with args"
    @echo ""
    @echo "Installation (from source):"
    @echo "  just install [prefix]        - Install with tracking (recommended)"
    @echo "  just install-pkg [prefix]    - Install and create package"
    @echo "  just package                 - Create .pkg package"
    @echo "  just pkg-install             - Create and install package"
    @echo "  just pkg-repo                - Create local package repository"
    @echo ""
    @echo "Uninstallation:"
    @echo "  just uninstall [prefix]      - Clean uninstall (removes everything)"
    @echo "  just uninstall-dry [prefix]  - Show what would be uninstalled"
    @echo "  just uninstall-purge [prefix]- Uninstall and remove all data"
    @echo "  just pkg-remove              - Remove if installed via pkg"
    @echo ""
    @echo "Ports:"
    @echo "  just port-test      - Test FreeBSD port"
    @echo "  just port-install   - Install from port"
    @echo "  just port-clean     - Clean port work directory"
    @echo ""
    @echo "Utilities:"
    @echo "  just status         - Show installation status"
    @echo "  just clean-all      - Clean all build artifacts"
    @echo "  just help           - Show this help"
