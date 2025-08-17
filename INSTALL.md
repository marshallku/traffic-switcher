# Traffic Switcher Installation Guide

Traffic Switcher is a high-performance HTTP reverse proxy with dynamic port switching capabilities, perfect for blue-green deployments. This guide will help you install and run Traffic Switcher on your system.

## Quick Install

### macOS

```bash
curl -sSL https://raw.githubusercontent.com/your-username/traffic-switcher/master/scripts/install-macos.sh | bash
```

### Ubuntu/Debian

```bash
curl -sSL https://raw.githubusercontent.com/your-username/traffic-switcher/master/scripts/install-ubuntu.sh | sudo bash
```

## Manual Installation

### Prerequisites

-   For building from source: Rust 1.70+ and Cargo
-   For running: No additional dependencies required

### Download Pre-built Binaries

1. Visit the [releases page](https://github.com/your-username/traffic-switcher/releases)
2. Download the appropriate archive for your platform:

    - `traffic-switcher-linux-amd64.tar.gz` - Linux x86_64
    - `traffic-switcher-linux-arm64.tar.gz` - Linux ARM64
    - `traffic-switcher-macos-amd64.tar.gz` - macOS Intel
    - `traffic-switcher-macos-arm64.tar.gz` - macOS Apple Silicon

3. Extract the archive:

```bash
tar -xzf traffic-switcher-*.tar.gz
```

4. Move binaries to your PATH:

```bash
sudo mv traffic_switcher /usr/local/bin/traffic-switcher
sudo mv tsctl /usr/local/bin/tsctl
sudo chmod +x /usr/local/bin/traffic-switcher /usr/local/bin/tsctl
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-username/traffic-switcher.git
cd traffic-switcher

# Build release binaries
cargo build --release
cargo build --release -p tsctl

# Install binaries
sudo cp target/release/traffic_switcher /usr/local/bin/traffic-switcher
sudo cp target/release/tsctl /usr/local/bin/tsctl
sudo chmod +x /usr/local/bin/traffic-switcher /usr/local/bin/tsctl
```

## Configuration

Traffic Switcher uses a YAML configuration file. Create a configuration file at one of these locations:

-   Current directory: `config.yaml`
-   macOS: `~/.config/traffic-switcher/config.yaml`
-   Linux: `/etc/traffic-switcher/config.yaml`

### Example Configuration

```yaml
services:
    - name: "web-app"
      host: "127.0.0.1"
      port: 3000
      health_check:
          path: "/health"
          retry_count: 3
          retry_delay: 1

    - name: "api"
      host: "127.0.0.1"
      port: 8080

routes:
    - domain: "app.example.com"
      service: "web-app"
    - domain: "api.example.com"
      service: "api"
    - domain: "*" # Catch-all route
      service: "web-app"

api_port: 1143 # Management API port
proxy_port: 1144 # HTTP proxy port
```

## Running as a Service

### macOS (launchd)

The macOS installation script automatically creates a LaunchAgent. To manage the service:

```bash
# Start service
launchctl load ~/Library/LaunchAgents/com.traffic-switcher.server.plist

# Stop service
launchctl unload ~/Library/LaunchAgents/com.traffic-switcher.server.plist

# Check status
launchctl list | grep traffic-switcher

# View logs
tail -f ~/.config/traffic-switcher/server.log
```

### Ubuntu/Linux (systemd)

The Ubuntu installation script automatically creates a systemd service. To manage the service:

```bash
# Start service
sudo systemctl start traffic-switcher

# Stop service
sudo systemctl stop traffic-switcher

# Enable auto-start on boot
sudo systemctl enable traffic-switcher

# Check status
sudo systemctl status traffic-switcher

# View logs
sudo journalctl -u traffic-switcher -f
```

### Manual Run (Foreground)

```bash
# Run with default config.yaml in current directory
traffic-switcher

# Run with specific config file
traffic-switcher --config /path/to/config.yaml

# Run in detached mode (background)
nohup traffic-switcher > server.log 2>&1 &
```

## Using the CLI Tool (tsctl)

The `tsctl` command-line tool allows you to manage Traffic Switcher remotely:

```bash
# Update service port
tsctl port <service-name> <new-port>

# Update port without health check
tsctl port <service-name> <new-port> --skip-health

# Examples
tsctl port web-app 3001
tsctl port api 8081 --skip-health
```

## API Endpoints

Traffic Switcher provides a REST API for management (default port 1143):

-   `GET /` - Health check
-   `GET /config` - Get current configuration
-   `GET /config/reload` - Reload configuration from disk
-   `POST /config/port` - Update service port

### Example API Usage

```bash
# Check health
curl http://localhost:1143/

# Get configuration
curl http://localhost:1143/config

# Reload configuration
curl http://localhost:1143/config/reload

# Update service port
curl -X POST http://localhost:1143/config/port \
  -H "Content-Type: application/json" \
  -d '{"service": "web-app", "port": 3001}'
```

## Docker Installation

### Using Docker Compose

```yaml
version: "3.8"

services:
    traffic-switcher:
        image: ghcr.io/your-username/traffic-switcher:latest
        ports:
            - "1143:1143" # API port
            - "1144:1144" # Proxy port
        volumes:
            - ./config.yaml:/app/config.yaml
        restart: unless-stopped
```

### Using Docker CLI

```bash
docker run -d \
  --name traffic-switcher \
  -p 1143:1143 \
  -p 1144:1144 \
  -v $(pwd)/config.yaml:/app/config.yaml \
  ghcr.io/your-username/traffic-switcher:latest
```

## Uninstallation

### macOS

```bash
curl -sSL https://raw.githubusercontent.com/your-username/traffic-switcher/master/scripts/uninstall-macos.sh | bash
```

### Ubuntu/Linux

```bash
curl -sSL https://raw.githubusercontent.com/your-username/traffic-switcher/master/scripts/uninstall-ubuntu.sh | sudo bash
```

## Troubleshooting

### Service Won't Start

1. Check configuration syntax:

```bash
traffic-switcher --config /path/to/config.yaml --check
```

2. Check logs:
    - macOS: `~/.config/traffic-switcher/server.error.log`
    - Linux: `sudo journalctl -u traffic-switcher -n 50`

### Port Already in Use

Change the API or proxy port in your configuration:

```yaml
api_port: 2143 # Change from default 1143
proxy_port: 2144 # Change from default 1144
```

### Permission Denied

-   Ensure binaries have execute permissions: `chmod +x /usr/local/bin/traffic-switcher`
-   For ports below 1024 on Linux, either:
    -   Run as root (not recommended)
    -   Use capabilities: `sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/traffic-switcher`

## Security Considerations

1. **Configuration Security**: Protect your configuration file as it may contain internal network information
2. **Network Security**: Consider using a firewall to restrict access to the management API (port 1143)
3. **TLS/SSL**: For production use, place Traffic Switcher behind a TLS-terminating proxy like nginx or Caddy
4. **Service User**: The systemd service runs as a dedicated user for enhanced security

## Support

-   GitHub Issues: [Report bugs or request features](https://github.com/your-username/traffic-switcher/issues)
-   Documentation: [GitHub Wiki](https://github.com/your-username/traffic-switcher/wiki)

## License

Traffic Switcher is released under the MIT License. See [LICENSE](LICENSE) for details.
