# Docker Quick Start

Get the Random Number Validator running in Docker in under 2 minutes!

## Prerequisites

- Docker installed ([Get Docker](https://docs.docker.com/get-docker/))
- Docker Compose installed (included with Docker Desktop)

## Quick Start Commands

### Start the application

```bash
docker-compose up -d
```

This will:
1. Build the Docker image (first time only, ~5 minutes)
2. Start the container in the background
3. Make the app available at http://localhost:3000

### View logs

```bash
docker-compose logs -f
```

Press `Ctrl+C` to stop viewing logs (container keeps running).

### Stop the application

```bash
docker-compose down
```

## That's It!

Open your browser to http://localhost:3000 and start validating random numbers!

## Next Steps

- See [DOCKER.md](DOCKER.md) for advanced configuration
- See [README.md](README.md) for application usage
- See [DEPLOYMENT.md](DEPLOYMENT.md) for cloud deployment

## Troubleshooting

### Port 3000 already in use?

Edit `docker-compose.yml` and change:
```yaml
ports:
  - "8080:3000"  # Change 8080 to any available port
```

### Need to rebuild after code changes?

```bash
docker-compose build
docker-compose up -d
```

### Application not responding?

```bash
# Check container status
docker-compose ps

# Check logs
docker-compose logs

# Restart
docker-compose restart
```

## Alternative: Docker CLI

If you prefer not to use docker-compose:

```bash
# Build
docker build -t randomnumbervalidator:latest .

# Run
docker run -d --name randomnumbervalidator -p 3000:3000 randomnumbervalidator:latest

# Stop
docker stop randomnumbervalidator && docker rm randomnumbervalidator
```
