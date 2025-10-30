# Docker Deployment Guide

This guide explains how to deploy the Random Number Validator application in a Docker container, including the frontend, backend, and NIST test suite.

## Quick Start

### Option 1: Using Docker Compose (Recommended)

```bash
# Build and start the container
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the container
docker-compose down
```

The application will be available at `http://localhost:3000`

### Option 2: Using Docker CLI

```bash
# Build the image
docker build -t randomnumbervalidator:latest .

# Run the container
docker run -d \
  --name randomnumbervalidator \
  -p 3000:3000 \
  -e RUST_LOG=info \
  randomnumbervalidator:latest

# View logs
docker logs -f randomnumbervalidator

# Stop and remove the container
docker stop randomnumbervalidator
docker rm randomnumbervalidator
```

## What's Included

The Docker container includes:

- **Backend**: Rust-based web server (Axum framework)
- **Frontend**: Static HTML/JS interface served by the backend
- **NIST Test Suite**: Compiled NIST Statistical Test Suite (STS 2.1.2)

All components are fully integrated and ready to use.

## Container Details

### Multi-Stage Build

The Dockerfile uses a multi-stage build for optimal image size:

1. **Builder Stage**:
   - Based on `rust:1.75`
   - Compiles the Rust application
   - Builds the NIST test suite from source

2. **Runtime Stage**:
   - Based on `debian:bookworm-slim`
   - Contains only the necessary runtime dependencies
   - Much smaller final image size

### Exposed Ports

- **3000**: HTTP server port (configurable via PORT environment variable)

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3000` | Server port |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

### Health Check

The container includes a health check that runs every 30 seconds to ensure the server is responding.

## Advanced Usage

### Custom Port Mapping

```bash
# Run on port 8080 instead of 3000
docker run -d \
  --name randomnumbervalidator \
  -p 8080:3000 \
  randomnumbervalidator:latest

# Or modify the internal port
docker run -d \
  --name randomnumbervalidator \
  -p 8080:8080 \
  -e PORT=8080 \
  randomnumbervalidator:latest
```

### Enable Debug Logging

```bash
docker run -d \
  --name randomnumbervalidator \
  -p 3000:3000 \
  -e RUST_LOG=debug \
  randomnumbervalidator:latest
```

### Persistent Data (Optional)

If you need to persist NIST test data or results:

```yaml
# docker-compose.yml
services:
  randomnumbervalidator:
    # ... other config ...
    volumes:
      - ./nist-data:/app/nist/sts-2.1.2/sts-2.1.2/data
      - ./nist-experiments:/app/nist/sts-2.1.2/sts-2.1.2/experiments
```

## Troubleshooting

### Container won't start

```bash
# Check container logs
docker logs randomnumbervalidator

# Check if port 3000 is already in use
lsof -i :3000  # On macOS/Linux
netstat -ano | findstr :3000  # On Windows
```

### NIST tests failing

The NIST test suite should be automatically built during the Docker build. If tests fail:

```bash
# Verify the NIST binary exists
docker exec randomnumbervalidator ls -la /app/nist/sts-2.1.2/sts-2.1.2/assess

# Check NIST directories
docker exec randomnumbervalidator ls -la /app/nist/sts-2.1.2/sts-2.1.2/

# Rebuild the image from scratch
docker build --no-cache -t randomnumbervalidator:latest .
```

### Can't access the application

1. Check if the container is running:
```bash
docker ps
```

2. Check health status:
```bash
docker inspect randomnumbervalidator | grep -A 5 Health
```

3. Verify port mapping:
```bash
docker port randomnumbervalidator
```

4. Test from inside the container:
```bash
docker exec randomnumbervalidator curl -f http://localhost:3000
```

## Development Workflow

### Rebuilding After Code Changes

```bash
# Using docker-compose
docker-compose build
docker-compose up -d

# Using docker CLI
docker build -t randomnumbervalidator:latest .
docker stop randomnumbervalidator
docker rm randomnumbervalidator
docker run -d --name randomnumbervalidator -p 3000:3000 randomnumbervalidator:latest
```

### Viewing Real-Time Logs

```bash
# Using docker-compose
docker-compose logs -f

# Using docker CLI
docker logs -f randomnumbervalidator
```

### Interactive Shell Access

```bash
# Access the container
docker exec -it randomnumbervalidator bash

# Once inside, you can:
cd /app
ls -la
./nist/sts-2.1.2/sts-2.1.2/assess --help
```

## Container Management

### Start/Stop/Restart

```bash
# Using docker-compose
docker-compose start
docker-compose stop
docker-compose restart

# Using docker CLI
docker start randomnumbervalidator
docker stop randomnumbervalidator
docker restart randomnumbervalidator
```

### View Resource Usage

```bash
docker stats randomnumbervalidator
```

### Remove Everything

```bash
# Using docker-compose
docker-compose down -v --rmi all

# Using docker CLI
docker stop randomnumbervalidator
docker rm randomnumbervalidator
docker rmi randomnumbervalidator:latest
```

## Production Deployment

### Using Docker Swarm

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml randomvalidator

# Check services
docker service ls

# View logs
docker service logs -f randomvalidator_randomnumbervalidator
```

### Using Kubernetes

Create a deployment manifest (example):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: randomnumbervalidator
spec:
  replicas: 2
  selector:
    matchLabels:
      app: randomnumbervalidator
  template:
    metadata:
      labels:
        app: randomnumbervalidator
    spec:
      containers:
      - name: randomnumbervalidator
        image: randomnumbervalidator:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
        - name: HOST
          value: "0.0.0.0"
        - name: PORT
          value: "3000"
---
apiVersion: v1
kind: Service
metadata:
  name: randomnumbervalidator
spec:
  selector:
    app: randomnumbervalidator
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

Deploy:
```bash
kubectl apply -f deployment.yaml
```

## Performance Tuning

### Optimize Image Size

The current setup already uses multi-stage builds, but you can further optimize:

```dockerfile
# Add to .dockerignore
*.md
tests/
examples/
.github/
terraform/
```

### Resource Limits

```yaml
# docker-compose.yml
services:
  randomnumbervalidator:
    # ... other config ...
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Security Considerations

1. **Run as non-root user** (optional enhancement):
```dockerfile
RUN useradd -m -u 1000 appuser
USER appuser
```

2. **Use specific image tags**:
```bash
docker pull rust:1.75-bookworm
docker pull debian:bookworm-slim-20231009
```

3. **Scan for vulnerabilities**:
```bash
docker scan randomnumbervalidator:latest
```

## Support

For issues related to:
- **Docker configuration**: See this guide or open an issue
- **Application functionality**: See README.md
- **GCP deployment**: See DEPLOYMENT.md

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Dockerfile Best Practices](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
