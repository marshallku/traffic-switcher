# Traffic Switcher

A high-performance HTTP reverse proxy server with dynamic port switching capabilities for zero-downtime blue-green deployments. Built with Rust for reliability and performance.

## Features

-   **Dynamic Port Switching**: Seamlessly switch backend service ports with zero downtime
-   **Health Checks**: Built-in health check system with configurable retries and delays
-   **Hot Configuration Reload**: Update routes and services without restarting the proxy
-   **Multi-Domain Support**: Route multiple domains to different backend services
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

```bash
# Clone the repository
git clone https://github.com/marshallku/traffic_switcher.git
cd traffic_switcher

# Build the project
cargo build --release

# Run the server
cargo run
```

## Configuration

Create a `config.yaml` file to define your services and routes:

```yaml
api_port: 1143
proxy_port: 1144

services:
    blog:
        host: "localhost"
        port: 4200
        health_check:
            path: "/health"
            retry_count: 3
            delay: 1000 # milliseconds

    api:
        host: "localhost"
        port: 3000
        health_check:
            path: "/api/health"
            retry_count: 5
            delay: 2000

routes:
    "blog.example.com": "blog"
    "api.example.com": "api"
    "*": "blog" # Wildcard catch-all route
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

```bash
# Build the CLI tool
cargo build -p tsctl

# Update service port
cargo run -p tsctl -- port <service> <port> [--skip-health]

# Reload configuration from disk
cargo run -p tsctl -- reload

# Get current configuration
cargo run -p tsctl -- config

# Examples
cargo run -p tsctl -- port blog 4201                  # Switch blog to port 4201 with health check
cargo run -p tsctl -- port api 3001 --skip-health     # Switch API to port 3001, skip health check
cargo run -p tsctl -- reload                          # Reload config.yaml
cargo run -p tsctl -- config                          # Display current configuration
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
│   ├── main.rs           # Application entry point
│   ├── env/
│   │   ├── config.rs     # Configuration structures
│   │   └── state.rs      # Application state management
│   ├── routes/
│   │   ├── api.rs        # Management API endpoints
│   │   └── proxy.rs      # HTTP proxy implementation
│   └── utils/            # Utility functions
├── tsctl/                # CLI tool package
│   └── src/
│       └── main.rs       # CLI implementation
├── config.yaml           # Configuration file
├── Cargo.toml           # Rust dependencies
└── Dockerfile           # Container definition
```

## How It Works

1. **Request Flow**:

    - Client sends request to proxy server (port 1144)
    - Proxy extracts domain from Host header
    - Domain is matched against routes configuration
    - Request is forwarded to the corresponding backend service

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

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss any changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
