#!/bin/sh
#
# Install veln from source
# Usage: ./install.sh [--prefix=/usr/local]

set -e

PREFIX=${PREFIX:-/usr/local}
REPO_URL="https://github.com/vessaix/veln.git"
SRC_DIR="/tmp/veln-src-$$"

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--prefix=/usr/local]"
            exit 1
            ;;
    esac
done

echo "Installing veln from source..."
echo "Prefix: $PREFIX"

# Check dependencies
check_dep() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed."
        exit 1
    fi
}

echo "Checking dependencies..."
check_dep cargo
check_dep git

# Clone or use local source
if [ -f Cargo.toml ] && [ -d src ]; then
    echo "Using local source directory..."
    SRC_DIR="$(pwd)"
    LOCAL_SRC=1
else
    echo "Cloning repository..."
    git clone "$REPO_URL" "$SRC_DIR"
    cd "$SRC_DIR"
    LOCAL_SRC=0
fi

# Build
echo "Building release binary..."
cargo build --release

# Install binary
echo "Installing binary to $PREFIX/bin/veln..."
install -m 755 target/release/veln "$PREFIX/bin/veln"

# Install config example (optional)
if [ -f veln.toml.example ]; then
    echo "Installing config example to $PREFIX/etc/veln.toml.example..."
    install -m 644 veln.toml.example "$PREFIX/etc/veln.toml.example"
fi

# Install man page if it exists
if [ -f doc/veln.1 ]; then
    echo "Installing man page..."
    install -m 644 doc/veln.1 "$PREFIX/share/man/man1/veln.1"
fi

# Install RC script (FreeBSD only)
if [ "$(uname -s)" = "FreeBSD" ] && [ -f rc.d/veln ]; then
    echo "Installing RC script..."
    install -m 555 rc.d/veln "$PREFIX/etc/rc.d/veln"
    echo ""
    echo "To enable veln API server at boot:"
    echo "  echo 'veln_enable=\"YES\"' >> /etc/rc.conf"
fi

# Cleanup
if [ "$LOCAL_SRC" -eq 0 ]; then
    echo "Cleaning up..."
    rm -rf "$SRC_DIR"
fi

echo ""
echo "Installation complete!"
echo "Run 'veln --help' to get started."
echo ""
echo "To uninstall:"
echo "  rm $PREFIX/bin/veln"
