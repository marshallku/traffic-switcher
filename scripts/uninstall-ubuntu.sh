#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/traffic-switcher"
SERVICE_NAME="traffic-switcher"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

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

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then 
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Stop and remove service
remove_service() {
    if [ -f "${SERVICE_FILE}" ]; then
        print_info "Stopping and removing systemd service..."
        
        # Stop service if running
        if systemctl is-active --quiet "${SERVICE_NAME}"; then
            systemctl stop "${SERVICE_NAME}"
        fi
        
        # Disable service
        if systemctl is-enabled --quiet "${SERVICE_NAME}"; then
            systemctl disable "${SERVICE_NAME}"
        fi
        
        # Remove service file
        rm -f "${SERVICE_FILE}"
        
        # Reload systemd
        systemctl daemon-reload
        print_info "Service removed"
    else
        print_info "No service file found"
    fi
}

# Remove binaries
remove_binaries() {
    print_info "Removing binaries..."
    
    if [ -f "${INSTALL_DIR}/traffic-switcher" ]; then
        rm -f "${INSTALL_DIR}/traffic-switcher"
        print_info "Removed traffic-switcher binary"
    fi
    
    if [ -f "${INSTALL_DIR}/tsctl" ]; then
        rm -f "${INSTALL_DIR}/tsctl"
        print_info "Removed tsctl binary"
    fi
}

# Remove service user
remove_user() {
    if id -u traffic-switcher >/dev/null 2>&1; then
        print_info "Removing service user..."
        userdel -r traffic-switcher 2>/dev/null || true
        print_info "Service user removed"
    fi
}

# Main uninstallation
main() {
    print_info "Starting Traffic Switcher uninstallation for Ubuntu..."
    
    # Check root
    check_root
    
    # Check if installed
    if [ ! -f "${INSTALL_DIR}/traffic-switcher" ] && [ ! -f "${SERVICE_FILE}" ]; then
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
        read -p "Do you want to remove configuration? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "${CONFIG_DIR}"
            print_info "Configuration removed"
        else
            print_info "Configuration preserved at: ${CONFIG_DIR}"
        fi
    fi
    
    # Remove service user
    remove_user
    
    print_info ""
    print_info "Uninstallation complete!"
    print_info "Traffic Switcher has been removed from your system"
}

# Run main function
main "$@"