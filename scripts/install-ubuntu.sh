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
GITHUB_REPO="your-username/traffic-switcher"  # Update this with your GitHub username

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

# Detect architecture
get_arch() {
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ]; then
        echo "amd64"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        echo "arm64"
    else
        print_error "Unsupported architecture: $ARCH"
        exit 1
    fi
}

# Download and install binaries
install_binaries() {
    ARCH=$(get_arch)
    print_info "Detected architecture: $ARCH"
    
    # Install required tools if not present
    if ! command -v curl &> /dev/null; then
        print_info "Installing curl..."
        apt-get update && apt-get install -y curl
    fi
    
    # Get latest release
    print_info "Fetching latest release information..."
    LATEST_RELEASE=$(curl -s https://api.github.com/repos/${GITHUB_REPO}/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_RELEASE" ]; then
        print_error "Failed to fetch latest release"
        exit 1
    fi
    
    print_info "Latest version: $LATEST_RELEASE"
    
    # Download binary
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_RELEASE}/traffic-switcher-linux-${ARCH}.tar.gz"
    TEMP_DIR=$(mktemp -d)
    
    print_info "Downloading from: $DOWNLOAD_URL"
    curl -L -o "${TEMP_DIR}/traffic-switcher.tar.gz" "$DOWNLOAD_URL"
    
    # Extract and install
    print_info "Extracting binaries..."
    tar -xzf "${TEMP_DIR}/traffic-switcher.tar.gz" -C "${TEMP_DIR}"
    
    print_info "Installing binaries to ${INSTALL_DIR}..."
    cp "${TEMP_DIR}/traffic_switcher" "${INSTALL_DIR}/traffic-switcher"
    cp "${TEMP_DIR}/tsctl" "${INSTALL_DIR}/tsctl"
    chmod +x "${INSTALL_DIR}/traffic-switcher"
    chmod +x "${INSTALL_DIR}/tsctl"
    
    # Setup config directory
    print_info "Setting up configuration directory..."
    mkdir -p "${CONFIG_DIR}"
    
    if [ ! -f "${CONFIG_DIR}/config.yaml" ]; then
        if [ -f "${TEMP_DIR}/config.yaml.example" ]; then
            cp "${TEMP_DIR}/config.yaml.example" "${CONFIG_DIR}/config.yaml"
            print_info "Example configuration copied to ${CONFIG_DIR}/config.yaml"
            print_warning "Please edit ${CONFIG_DIR}/config.yaml to configure your services"
        else
            cat > "${CONFIG_DIR}/config.yaml" << EOF
services:
  - name: "example"
    host: "127.0.0.1"
    port: 8080
    health_check:
      path: "/health"
      retry_count: 3
      retry_delay: 1

routes:
  - domain: "example.local"
    service: "example"

api_port: 1143
proxy_port: 1144
EOF
            print_info "Default configuration created at ${CONFIG_DIR}/config.yaml"
            print_warning "Please edit ${CONFIG_DIR}/config.yaml to configure your services"
        fi
    fi
    
    # Cleanup
    rm -rf "${TEMP_DIR}"
    
    print_info "Binaries installed successfully!"
}

# Create systemd service
create_systemd_service() {
    print_info "Creating systemd service..."
    
    # Create service user if it doesn't exist
    if ! id -u traffic-switcher >/dev/null 2>&1; then
        print_info "Creating service user 'traffic-switcher'..."
        useradd -r -s /bin/false -d /var/lib/traffic-switcher -m traffic-switcher
    fi
    
    # Set proper permissions
    chown -R traffic-switcher:traffic-switcher "${CONFIG_DIR}"
    
    # Create systemd service file
    cat > "${SERVICE_FILE}" << EOF
[Unit]
Description=Traffic Switcher - HTTP Reverse Proxy with Dynamic Port Switching
Documentation=https://github.com/${GITHUB_REPO}
After=network.target

[Service]
Type=simple
User=traffic-switcher
Group=traffic-switcher
ExecStart=${INSTALL_DIR}/traffic-switcher --config ${CONFIG_DIR}/config.yaml
ExecReload=/bin/kill -USR1 \$MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=traffic-switcher

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${CONFIG_DIR}

# Allow binding to privileged ports if needed
AmbientCapabilities=CAP_NET_BIND_SERVICE
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
EOF
    
    print_info "Systemd service created at ${SERVICE_FILE}"
    
    # Reload systemd
    systemctl daemon-reload
}

# Main installation
main() {
    print_info "Starting Traffic Switcher installation for Ubuntu..."
    
    # Check root
    check_root
    
    # Check if already installed
    if [ -f "${INSTALL_DIR}/traffic-switcher" ]; then
        print_warning "Traffic Switcher is already installed"
        read -p "Do you want to reinstall? (y/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
        
        # Stop service if running
        if systemctl is-active --quiet "${SERVICE_NAME}"; then
            print_info "Stopping existing service..."
            systemctl stop "${SERVICE_NAME}"
        fi
    fi
    
    # Install binaries
    install_binaries
    
    # Create systemd service
    create_systemd_service
    
    # Enable service
    print_info "Enabling service to start on boot..."
    systemctl enable "${SERVICE_NAME}"
    
    # Ask if user wants to start the service
    read -p "Do you want to start Traffic Switcher service now? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Starting Traffic Switcher service..."
        systemctl start "${SERVICE_NAME}"
        sleep 2
        
        if systemctl is-active --quiet "${SERVICE_NAME}"; then
            print_info "Service started successfully!"
            systemctl status "${SERVICE_NAME}" --no-pager
        else
            print_error "Failed to start service. Check logs with: journalctl -u ${SERVICE_NAME} -n 50"
        fi
    else
        print_info "You can start the service later with: sudo systemctl start ${SERVICE_NAME}"
    fi
    
    print_info ""
    print_info "Installation complete!"
    print_info ""
    print_info "Server binary: ${INSTALL_DIR}/traffic-switcher"
    print_info "CLI tool: ${INSTALL_DIR}/tsctl"
    print_info "Configuration: ${CONFIG_DIR}/config.yaml"
    print_info ""
    print_info "Service commands:"
    print_info "  Start:    sudo systemctl start ${SERVICE_NAME}"
    print_info "  Stop:     sudo systemctl stop ${SERVICE_NAME}"
    print_info "  Restart:  sudo systemctl restart ${SERVICE_NAME}"
    print_info "  Status:   sudo systemctl status ${SERVICE_NAME}"
    print_info "  Logs:     sudo journalctl -u ${SERVICE_NAME} -f"
    print_info "  Enable:   sudo systemctl enable ${SERVICE_NAME}"
    print_info "  Disable:  sudo systemctl disable ${SERVICE_NAME}"
    print_info ""
    print_info "CLI usage: tsctl --help"
}

# Run main function
main "$@"