#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/traffic-switcher"
SERVICE_NAME="com.traffic-switcher.server"
PLIST_PATH="$HOME/Library/LaunchAgents/${SERVICE_NAME}.plist"

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Stop and remove service
remove_service() {
    if [ -f "${PLIST_PATH}" ]; then
        print_info "Stopping and removing LaunchAgent service..."
        
        # Unload service if running
        if launchctl list | grep -q "${SERVICE_NAME}"; then
            launchctl unload "${PLIST_PATH}" 2>/dev/null || true
        fi
        
        # Remove plist file
        rm -f "${PLIST_PATH}"
        print_info "Service removed"
    else
        print_info "No service file found"
    fi
}

# Remove binaries
remove_binaries() {
    print_info "Removing binaries..."
    
    if [ -f "${INSTALL_DIR}/traffic-switcher" ]; then
        sudo rm -f "${INSTALL_DIR}/traffic-switcher"
        print_info "Removed traffic-switcher binary"
    fi
    
    if [ -f "${INSTALL_DIR}/tsctl" ]; then
        sudo rm -f "${INSTALL_DIR}/tsctl"
        print_info "Removed tsctl binary"
    fi
}

# Main uninstallation
main() {
    print_info "Starting Traffic Switcher uninstallation for macOS..."
    
    # Check if installed
    if [ ! -f "${INSTALL_DIR}/traffic-switcher" ] && [ ! -f "${PLIST_PATH}" ]; then
        print_warning "Traffic Switcher does not appear to be installed"
        exit 0
    fi
    
    # Confirm uninstallation
    print_warning "This will remove Traffic Switcher from your system"
    read -p "Are you sure you want to uninstall? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi
    
    # Stop and remove service
    remove_service
    
    # Remove binaries
    remove_binaries
    
    # Ask about configuration
    if [ -d "${CONFIG_DIR}" ]; then
        print_warning "Configuration directory found at: ${CONFIG_DIR}"
        read -p "Do you want to remove configuration and logs? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "${CONFIG_DIR}"
            print_info "Configuration and logs removed"
        else
            print_info "Configuration and logs preserved at: ${CONFIG_DIR}"
        fi
    fi
    
    print_info ""
    print_info "Uninstallation complete!"
    print_info "Traffic Switcher has been removed from your system"
}

# Run main function
main "$@"