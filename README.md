# Traffic Switcher

A high-performance HTTP reverse proxy server with dynamic port switching capabilities for zero-downtime blue-green deployments. Built with Rust for reliability and performance.

## Features

-   **Dynamic Port Switching**: Seamlessly switch backend service ports with zero downtime
-   **Health Checks**: Built-in health check system with configurable retries and delays
-   **Hot Configuration Reload**: Update routes and services without restarting the proxy
-   **Multi-Domain Support**: Route multiple domains to different backend services
-   **Static File Serving**: Serve static files with index files and SPA fallback support
-   **HTTP Redirects**: Configure redirects with customizable status codes
-   **Wildcard Routing**: Support for catch-all routes with wildcard domains
-   **CLI Tool**: Command-line interface (`tsctl`) for easy management
-   **Blue-Green Deployments**: Native support for blue-green deployment strategies
-   **Docker Ready**: Complete Docker and Docker Compose configurations

## Architecture

Traffic Switcher consists of two main components:

1. **API Server** (default port 1143): Management API for configuring services and routes
2. **Proxy Server** (default port 1144): Routes incoming HTTP traffic to backend services

## Getting Started

### Prerequisites

-   **Rust**: Install from [rust-lang.org](https://www.rust-lang.org/)
-   **Docker** (optional): For containerized deployments from [docker.com](https://www.docker.com/)

### Installation

#### Option 1: Pre-built Binaries

Download the latest binaries for your platform from the [releases page](https://github.com/marshallku/traffic_switcher/releases).

##### Linux/macOS (Quick Install)

```bash
# Install latest version
curl -sSL https://github.com/marshallku/traffic_switcher/releases/latest/download/install.sh | bash

# Or specify a version
curl -sSL https://github.com/marshallku/traffic_switcher/releases/download/v1.0.0/install.sh | bash -s -- v1.0.0
```

##### Manual Installation

1. Download the appropriate binary for your platform:
   - Linux: `traffic-switcher-linux-amd64`, `traffic-switcher-linux-arm64`
   - macOS: `traffic-switcher-darwin-amd64`, `traffic-switcher-darwin-arm64`
   - Windows: `traffic-switcher-windows-amd64.exe`

2. Extract and move to your PATH:
```bash
tar -xzf traffic-switcher-linux-amd64.tar.gz
sudo mv traffic-switcher-linux-amd64 /usr/local/bin/traffic-switcher
sudo chmod +x /usr/local/bin/traffic-switcher

tar -xzf tsctl-linux-amd64.tar.gz
sudo mv tsctl-linux-amd64 /usr/local/bin/tsctl
sudo chmod +x /usr/local/bin/tsctl
```

#### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/marshallku/traffic_switcher.git
cd traffic_switcher

# Build the project
cargo build --release

# Install binaries
sudo cp target/release/traffic_switcher /usr/local/bin/traffic-switcher
sudo cp target/release/tsctl /usr/local/bin/tsctl
sudo chmod +x /usr/local/bin/traffic-switcher /usr/local/bin/tsctl
```

## Configuration

Create a `config.yaml` file to define your services and routes:

```yaml
api_port: 1143
proxy_port: 1144

services:
    - name: api
      host: localhost
      port: 3000
      health_check:
          path: /health
          retry_count: 10
          retry_delay_seconds: 1

    - name: webapp
      host: localhost
      port: 8080
      health_check:
          path: /
          retry_count: 5
          retry_delay_seconds: 2

routes:
    # Proxy to backend service
    - domain: api.example.com
      type: service
      service: api

    # Static file serving
    - domain: static.example.com
      type: static
      root: /var/www/static
      index:
          - index.html
          - index.htm
      try_files:
          - $uri
          - $uri/
          - /404.html

    # SPA application with fallback
    - domain: app.example.com
      type: static
      root: /var/www/app
      try_files:
          - $uri
          - /index.html # SPA fallback

    # Redirect example
    - domain: old.example.com
      type: redirect
      to: https://new.example.com
      code: 307

    # Default catch-all route
    - domain: "*"
      type: service
      service: webapp
```

## Usage

### API Endpoints

#### Get Current Configuration

```bash
curl http://localhost:1143/config
```

#### Reload Configuration from Disk

```bash
curl http://localhost:1143/config/reload
```

#### Update Service Port

```bash
# With health check
curl -X POST http://localhost:1143/config/port \
  -H "Content-Type: application/json" \
  -d '{"service": "blog", "port": 4201}'

# Skip health check
curl -X POST http://localhost:1143/config/port \
  -H "Content-Type: application/json" \
  -d '{"service": "blog", "port": 4201, "skip_health": true}'
```

### CLI Tool (tsctl)

The `tsctl` command-line tool provides an easy way to manage Traffic Switcher:

#### Server Management

```bash
# Start the server
tsctl start                                      # Start in foreground
tsctl start --daemon                             # Start in background
tsctl start --daemon --log-file /var/log/ts.log # With logging
tsctl start --daemon --pid-file /var/run/ts.pid # With PID file
tsctl start --config /path/to/config.yaml       # Custom config path
tsctl start --verbose                            # Enable debug logging

# Stop the server
tsctl stop --pid-file /var/run/ts.pid           # Stop using PID file
tsctl stop --pid 12345                          # Stop specific process

# Check server status
tsctl status                                     # Check server status
tsctl status --pid-file /var/run/ts.pid        # With PID file
```

#### Service Management

```bash
# Update service port
tsctl port <service> <port> [--skip-health]

# Switch between ports (blue-green deployment)
tsctl switch <service> <from-port> <to-port> [--skip-health]

# Rollback to previous port
tsctl rollback <service>

# Show current port for a service
tsctl current <service>

# Automated deployment
tsctl deploy <service> <previous-port> <next-port> [--skip-health]
```

#### Configuration Management

```bash
# Get current configuration
tsctl config

# List all services
tsctl services

# List all routes
tsctl routes

# Check service health
tsctl health <service>
```

#### Examples

```bash
# Start server in background
tsctl start --daemon --pid-file /var/run/traffic-switcher.pid --log-file /var/log/traffic-switcher.log

# Switch blog service to port 4201 with health check
tsctl port blog 4201

# Switch API to port 3001, skip health check
tsctl port api 3001 --skip-health

# Perform blue-green deployment
tsctl switch webapp 8080 8081

# Check server status
tsctl status --pid-file /var/run/traffic-switcher.pid

# Stop the server
tsctl stop --pid-file /var/run/traffic-switcher.pid
```

#### Using tsctl for Configuration Updates

You can update the configuration file and reload it without restarting:

```bash
# 1. Edit config.yaml to add a new service
vim config.yaml

# 2. Reload the configuration
cargo run -p tsctl -- reload

# 3. Verify the changes
cargo run -p tsctl -- config
```

### Docker Deployment

```bash
# Build the Docker image
docker build -t traffic-switcher .

# Run with Docker Compose
docker compose up

# Or run directly
docker run -p 1143:1143 -p 1144:1144 \
  -v $(pwd)/config.yaml:/app/config.yaml \
  traffic-switcher
```

## Blue-Green Deployment Example

Traffic Switcher enables zero-downtime deployments by switching between two environments (blue and green). Here's an example deployment script:

```bash
#!/bin/bash

# Configuration
SERVICE="blog"
BLUE_PORT=4200
GREEN_PORT=4201
TRAFFIC_SWITCHER_URL="http://localhost:1143"

# Get current port
CURRENT_CONFIG=$(curl -s $TRAFFIC_SWITCHER_URL/config)
CURRENT_PORT=$(echo $CURRENT_CONFIG | jq -r ".services.$SERVICE.port")

# Determine target deployment
if [ "$CURRENT_PORT" == "$BLUE_PORT" ]; then
    TARGET_PORT=$GREEN_PORT
    TARGET_ENV="green"
else
    TARGET_PORT=$BLUE_PORT
    TARGET_ENV="blue"
fi

echo "Deploying to $TARGET_ENV environment (port $TARGET_PORT)"

# 1. Start new container on target port
docker run -d --name app-$TARGET_ENV -p $TARGET_PORT:80 myapp:latest

# 2. Wait for container to be healthy
sleep 5

# 3. Switch traffic to new container
curl -X POST $TRAFFIC_SWITCHER_URL/config/port \
  -H "Content-Type: application/json" \
  -d "{\"service\": \"$SERVICE\", \"port\": $TARGET_PORT}"

# 4. Stop old container
if [ "$CURRENT_PORT" == "$BLUE_PORT" ]; then
    docker stop app-blue && docker rm app-blue
else
    docker stop app-green && docker rm app-green
fi

echo "Deployment complete!"
```

For a complete example, see the [deployment script](https://github.com/marshallku/marshallku-blog-frontend/blob/master/scripts/deploy.sh) used in production.

## Project Structure

```
traffic-switcher/
├── src/
│   ├── main.rs             # Application entry point
│   ├── env/
│   │   ├── config.rs       # Configuration structures
│   │   └── state.rs        # Application state management
│   ├── routes/
│   │   ├── api.rs          # Management API endpoints
│   │   ├── proxy.rs        # HTTP proxy implementation
│   │   └── static_files.rs # Static file serving
│   └── utils/              # Utility functions
├── tsctl/                  # CLI tool package
│   └── src/
│       └── main.rs         # CLI implementation
├── config.yaml             # Configuration file
├── Cargo.toml              # Rust dependencies
└── Dockerfile              # Container definition
```

## How It Works

1. **Request Flow**:

    - Client sends request to proxy server (port 1144)
    - Proxy extracts domain from Host header
    - Domain is matched against routes configuration
    - Based on route type:
        - **Service**: Request is forwarded to the backend service
        - **Static**: File is served from disk with proper MIME type
        - **Redirect**: HTTP redirect response is returned

2. **Port Switching**:

    - API receives port update request
    - Optional health check is performed on new port
    - Configuration is atomically updated
    - All new requests are routed to the new port
    - Previous port information is retained for rollback

3. **Health Checks**:

    - Configurable per service with custom path
    - Retry mechanism with configurable count and delay
    - Prevents switching to unhealthy services

4. **Static File Serving**:

    - Secure path resolution with traversal protection
    - Configurable index files for directories
    - Try-files mechanism for SPA routing
    - Automatic MIME type detection

5. **HTTP Redirects**:
    - Support for all standard redirect status codes
    - Configurable destination URLs
    - Method preservation options (307, 308)

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss any changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
