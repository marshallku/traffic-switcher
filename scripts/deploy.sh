#!/bin/bash
set -e

# Configuration
SERVICE_NAME="${1:-webapp}"
IMAGE_TAG="${2:-latest}"
TSCTL="/usr/local/bin/tsctl"
BLUE_PORT=8080
GREEN_PORT=8081
HEALTH_CHECK_RETRIES=30
HEALTH_CHECK_DELAY=2

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if tsctl is available
if ! command -v $TSCTL &> /dev/null; then
    error "tsctl command not found. Please install Traffic Switcher CLI."
fi

# Check server status
log "Checking Traffic Switcher status..."
if ! $TSCTL status &> /dev/null; then
    error "Traffic Switcher is not running. Please start it first with: tsctl start --daemon"
fi

# Get current port
log "Getting current port for service: $SERVICE_NAME"
CURRENT_PORT=$($TSCTL current $SERVICE_NAME 2>/dev/null | grep -oP '\d+' || echo "")

if [ -z "$CURRENT_PORT" ]; then
    log "Service not found or no port configured. Setting up initial deployment on blue port ($BLUE_PORT)"
    TARGET_PORT=$BLUE_PORT
    TARGET_ENV="blue"
elif [ "$CURRENT_PORT" == "$BLUE_PORT" ]; then
    TARGET_PORT=$GREEN_PORT
    TARGET_ENV="green"
else
    TARGET_PORT=$BLUE_PORT
    TARGET_ENV="blue"
fi

log "Current environment: ${CURRENT_PORT:-none} -> Target environment: $TARGET_ENV (port $TARGET_PORT)"

# Stop existing container if any
log "Stopping existing $TARGET_ENV container if running..."
docker stop app-$TARGET_ENV 2>/dev/null || true
docker rm app-$TARGET_ENV 2>/dev/null || true

# Start new container
log "Starting new container app-$TARGET_ENV with image tag: $IMAGE_TAG"
docker run -d \
    --name app-$TARGET_ENV \
    --restart always \
    -p $TARGET_PORT:80 \
    -e ENV=$TARGET_ENV \
    -e PORT=$TARGET_PORT \
    --network proxy-network \
    your-app:$IMAGE_TAG

# Wait for container to be ready
log "Waiting for container to be healthy..."
for i in $(seq 1 $HEALTH_CHECK_RETRIES); do
    if docker exec app-$TARGET_ENV curl -f http://localhost/health &> /dev/null; then
        success "Container is healthy"
        break
    fi
    
    if [ $i -eq $HEALTH_CHECK_RETRIES ]; then
        error "Container failed to become healthy after $HEALTH_CHECK_RETRIES attempts"
    fi
    
    echo -n "."
    sleep $HEALTH_CHECK_DELAY
done

# Switch traffic to new container
log "Switching traffic to $TARGET_ENV environment..."
if $TSCTL port $SERVICE_NAME $TARGET_PORT; then
    success "Traffic successfully switched to port $TARGET_PORT"
else
    error "Failed to switch traffic. Rolling back..."
    docker stop app-$TARGET_ENV
    docker rm app-$TARGET_ENV
    exit 1
fi

# Clean up old container after successful switch
if [ ! -z "$CURRENT_PORT" ]; then
    OLD_ENV=$([ "$CURRENT_PORT" == "$BLUE_PORT" ] && echo "blue" || echo "green")
    log "Waiting 10 seconds before cleaning up old container..."
    sleep 10
    
    log "Stopping old container app-$OLD_ENV..."
    docker stop app-$OLD_ENV 2>/dev/null || true
    docker rm app-$OLD_ENV 2>/dev/null || true
    success "Old container cleaned up"
fi

# Final status
success "Deployment complete!"
log "Service: $SERVICE_NAME"
log "New port: $TARGET_PORT"
log "Environment: $TARGET_ENV"
log "Image: your-app:$IMAGE_TAG"

# Show current status
echo ""
$TSCTL status