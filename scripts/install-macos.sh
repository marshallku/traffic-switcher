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

# Detect architecture
get_arch() {
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ]; then
        echo "amd64"
    elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
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
    
    # Get latest release
    print_info "Fetching latest release information..."
    LATEST_RELEASE=$(curl -s https://api.github.com/repos/${GITHUB_REPO}/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_RELEASE" ]; then
        print_error "Failed to fetch latest release"
        exit 1
    fi
    
    print_info "Latest version: $LATEST_RELEASE"
    
    # Download binary
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_RELEASE}/traffic-switcher-macos-${ARCH}.tar.gz"
    TEMP_DIR=$(mktemp -d)
    
    print_info "Downloading from: $DOWNLOAD_URL"
    curl -L -o "${TEMP_DIR}/traffic-switcher.tar.gz" "$DOWNLOAD_URL"
    
    # Extract and install
    print_info "Extracting binaries..."
    tar -xzf "${TEMP_DIR}/traffic-switcher.tar.gz" -C "${TEMP_DIR}"
    
    print_info "Installing binaries to ${INSTALL_DIR}..."
    sudo cp "${TEMP_DIR}/traffic_switcher" "${INSTALL_DIR}/traffic-switcher"
    sudo cp "${TEMP_DIR}/tsctl" "${INSTALL_DIR}/tsctl"
    sudo chmod +x "${INSTALL_DIR}/traffic-switcher"
    sudo chmod +x "${INSTALL_DIR}/tsctl"
    
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

# Create LaunchAgent for running as service
create_launch_agent() {
    print_info "Creating LaunchAgent service..."
    
    cat > "${PLIST_PATH}" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${SERVICE_NAME}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/traffic-switcher</string>
        <string>--config</string>
        <string>${CONFIG_DIR}/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${CONFIG_DIR}/server.log</string>
    <key>StandardErrorPath</key>
    <string>${CONFIG_DIR}/server.error.log</string>
    <key>WorkingDirectory</key>
    <string>${CONFIG_DIR}</string>
</dict>
</plist>
EOF
    
    print_info "LaunchAgent created at ${PLIST_PATH}"
}

# Main installation
main() {
    print_info "Starting Traffic Switcher installation for macOS..."
    
    # Check if already installed
    if [ -f "${INSTALL_DIR}/traffic-switcher" ]; then
        print_warning "Traffic Switcher is already installed"
        read -p "Do you want to reinstall? (y/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
        
        # Stop service if running
        if launchctl list | grep -q "${SERVICE_NAME}"; then
            print_info "Stopping existing service..."
            launchctl unload "${PLIST_PATH}" 2>/dev/null || true
        fi
    fi
    
    # Install binaries
    install_binaries
    
    # Create LaunchAgent
    create_launch_agent
    
    # Ask if user wants to start the service
    read -p "Do you want to start Traffic Switcher service now? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Starting Traffic Switcher service..."
        launchctl load "${PLIST_PATH}"
        sleep 2
        
        if launchctl list | grep -q "${SERVICE_NAME}"; then
            print_info "Service started successfully!"
            print_info "You can check the status with: launchctl list | grep ${SERVICE_NAME}"
            print_info "Logs are available at: ${CONFIG_DIR}/server.log"
        else
            print_error "Failed to start service. Check logs at ${CONFIG_DIR}/server.error.log"
        fi
    else
        print_info "You can start the service later with: launchctl load ${PLIST_PATH}"
    fi
    
    print_info ""
    print_info "Installation complete!"
    print_info ""
    print_info "Server binary: ${INSTALL_DIR}/traffic-switcher"
    print_info "CLI tool: ${INSTALL_DIR}/tsctl"
    print_info "Configuration: ${CONFIG_DIR}/config.yaml"
    print_info ""
    print_info "Commands:"
    print_info "  Start service:   launchctl load ${PLIST_PATH}"
    print_info "  Stop service:    launchctl unload ${PLIST_PATH}"
    print_info "  Check status:    launchctl list | grep ${SERVICE_NAME}"
    print_info "  View logs:       tail -f ${CONFIG_DIR}/server.log"
    print_info "  Use CLI:         tsctl --help"
}

# Run main function
main "$@"