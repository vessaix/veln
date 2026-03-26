#!/bin/sh
#
# Quick package creation script for veln
# Usage: ./make-pkg.sh [OPTIONS]
#
# Options:
#   --install    Create and install the package
#   --repo       Create and add to local repo
#   --clean      Clean package staging
#   --help       Show help

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

show_help() {
    echo "Veln Package Builder"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --install    Create package and install it"
    echo "  --repo       Create package and add to local repository"
    echo "  --clean      Remove package staging directory"
    echo "  --help       Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                    # Create package in ./packages/"
    echo "  $0 --install          # Create and install"
    echo "  $0 --repo             # Create and add to repo"
}

# Parse arguments
INSTALL=0
REPO=0
CLEAN=0

while [ $# -gt 0 ]; do
    case "$1" in
        --install)
            INSTALL=1
            shift
            ;;
        --repo)
            REPO=1
            shift
            ;;
        --clean)
            CLEAN=1
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Clean staging if requested
if [ "$CLEAN" -eq 1 ]; then
    echo "Cleaning package staging..."
    rm -rf ./packages/staging
    echo "Done!"
    exit 0
fi

# Run install.sh with appropriate flags
if [ "$INSTALL" -eq 1 ]; then
    echo "Creating and installing package..."
    ./scripts/install.sh --package --package-dir=./packages
elif [ "$REPO" -eq 1 ]; then
    echo "Creating package and adding to local repository..."
    ./scripts/install.sh --package --package-dir=./packages
    ./scripts/install.sh --local-repo --repo-dir=/usr/local/poudriere/veln-repo
else
    echo "Creating package..."
    ./scripts/install.sh --package --package-dir=./packages
    
    # Show the created package
    echo ""
    echo "Package created in: ./packages/"
    ls -lh ./packages/*.pkg 2>/dev/null || echo "No .pkg files found"
    echo ""
    echo "To install: pkg add ./packages/veln-*.pkg"
fi
