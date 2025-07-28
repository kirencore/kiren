#!/bin/bash

# Kiren Docker Hub Publish Script
set -e

echo "📦 Building and publishing Kiren Docker image..."

# Configuration
DOCKER_HUB_USERNAME=${DOCKER_HUB_USERNAME:-"mertcanaltin"}
IMAGE_NAME="kiren"
VERSION=${VERSION:-$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}

echo "🏗️  Building image: $DOCKER_HUB_USERNAME/$IMAGE_NAME:$VERSION"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Build multi-platform image
echo "🔧 Setting up buildx..."
docker buildx create --use --name kiren-builder 2>/dev/null || docker buildx use kiren-builder

# Login to Docker Hub
echo "🔐 Please make sure you're logged in to Docker Hub..."
docker login

# Build and push for multiple architectures
echo "🚀 Building and pushing multi-platform image..."
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag $DOCKER_HUB_USERNAME/$IMAGE_NAME:$VERSION \
  --tag $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest \
  --push \
  .

echo "✅ Published to Docker Hub!"
echo "📋 Pull command: docker pull $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest"
echo "🏃 Run command: docker run -p 3000:3000 -v \$(pwd):/app $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest server.js"

# Update README with usage
cat > DOCKER.md << EOF
# Kiren Docker Image

## Quick Start
\`\`\`bash
# Pull latest image
docker pull $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest

# Run your Node.js app
docker run -p 3000:3000 -v \$(pwd):/app $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest
\`\`\`

## Production Use
\`\`\`bash
# Run with environment variables
docker run -d \\
  -p 3000:3000 \\
  -e NODE_ENV=production \\
  -e PORT=3000 \\
  -v \$(pwd):/app \\
  --name my-kiren-app \\
  $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest
\`\`\`

## Docker Compose
\`\`\`yaml
version: '3.8'
services:
  app:
    image: $DOCKER_HUB_USERNAME/$IMAGE_NAME:latest
    ports:
      - "3000:3000"
    volumes:
      - ./:/app
    environment:
      - NODE_ENV=production
\`\`\`
EOF

echo "📝 Created DOCKER.md with usage instructions"