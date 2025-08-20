# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Traffic Switcher is a Rust-based HTTP reverse proxy server with dynamic port switching capabilities for blue-green deployments. It consists of two main components:

-   **API Server**: Management API for configuring services and routes (default port 1143)
-   **Proxy Server**: Routes incoming HTTP traffic to backend services based on domain mappings (default port 1144)

## Common Development Commands

### Build & Run

```bash
# Run the main server (reads config.yaml)
cargo run

# Build the project
cargo build

# Build release version
cargo build --release

# Run with Docker
docker compose up
```

### Testing & Quality

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Format code
cargo fmt

# Run clippy for linting
cargo clippy
```

### CLI Tool (tsctl)

```bash
# Build the CLI
cargo build -p tsctl

# Update service port
cargo run -p tsctl -- port <service> <port> [--skip-health]

# Example: Switch blog service to port 18090
cargo run -p tsctl -- port blog 18090
```

## Core Architecture

### Configuration System

The application uses a YAML-based configuration (`config.yaml`) that defines:

-   **Services**: Backend services with host, port, and optional health check configuration
-   **Routes**: Domain routing with multiple types:
    -   **Service Routes**: Proxy requests to backend services
    -   **Static Routes**: Serve static files from disk with configurable index files and fallback rules
    -   **Redirect Routes**: HTTP redirects with customizable status codes (301, 302, 307, 308, etc.)
    -   **Wildcard Support**: Use "\*" for catch-all routing
-   **Ports**: API and proxy server ports

Configuration can be dynamically reloaded and modified at runtime through the API.

### State Management

-   `AppState` (src/env/state.rs:67-182): Central state container using `Arc<RwLock>` for thread-safe access
-   Maintains in-memory maps for services and routes alongside the full config
-   Supports hot-reloading and atomic port updates with health checks

### Request Flow

1. **Proxy Server** (src/routes/proxy.rs): Receives incoming HTTP requests on proxy_port
2. Extracts domain from Host header and determines route type
3. Processes request based on route type:
   - **Service**: Establishes TCP connection to backend service and proxies the request
   - **Static**: Serves files from configured root directory with MIME type detection
   - **Redirect**: Returns HTTP redirect response with configured status code
4. Uses hyper for low-level HTTP handling to preserve headers

### Health Check System

-   Configurable per-service with path, retry count, and delay
-   Automatically triggered during port updates unless skipped
-   Prevents switching to unhealthy services

### API Endpoints

-   `GET /` - Health check endpoint
-   `GET /config` - Get current configuration
-   `GET /config/reload` - Reload configuration from disk
-   `POST /config/port` - Update service port with optional health check

## Key Implementation Details

### Port Switching Logic

The `update_service_port` method (src/env/state.rs:122-181) implements atomic port switching:

1. Updates port in configuration
2. Performs health check on new port (unless skipped)
3. Updates in-memory service map
4. Stores previous port for rollback capability

### Proxy Implementation

The proxy uses raw hyper connections (src/routes/proxy.rs:32-76) to maintain full control over HTTP headers and preserve original request characteristics, critical for proper reverse proxy behavior.

### Static File Serving

The static file handler (src/routes/static_files.rs) provides:

-   **Security**: Path traversal protection with ".." and null byte filtering
-   **Index Files**: Configurable index file lookup for directories (e.g., index.html)
-   **Try Files**: Fallback mechanism for SPA routing (e.g., fallback to /index.html)
-   **MIME Types**: Automatic content-type detection with UTF-8 charset for text files
-   **URL Decoding**: Proper handling of percent-encoded URLs

### Redirect Handling

Supports various HTTP redirect status codes:

-   **301**: Permanent redirect (Moved Permanently)
-   **302**: Temporary redirect (Found)
-   **303**: See Other (POST to GET redirect)
-   **307**: Temporary redirect (preserves method)
-   **308**: Permanent redirect (preserves method)

### Workspace Structure

The project uses Cargo workspaces with two packages:

-   Root package: Main server application (`traffic_switcher`)
-   CLI package: Command-line tool (`tsctl`) that depends on the main package

## CI/CD Pipeline

GitHub Actions workflow (.github/workflows/ci.yml):

1. **Spell Check**: Validates spelling in source code
2. **Build**: Compiles the Rust project
3. **Test**: Runs all tests
4. **Docker**: Builds and pushes Docker image to GitHub Container Registry (ghcr.io) on master branch pushes
