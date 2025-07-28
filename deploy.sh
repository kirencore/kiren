#!/bin/bash

# Kiren Production Deployment Script
set -e

echo "🚀 Starting Kiren deployment..."

# Configuration
REGISTRY=${DOCKER_REGISTRY:-"your-registry.com"}
APP_NAME=${APP_NAME:-"kiren-app"}
TAG=${TAG:-"latest"}
ENVIRONMENT=${ENVIRONMENT:-"production"}

# Build Docker image
echo "📦 Building Docker image..."
docker build -t $REGISTRY/$APP_NAME:$TAG .

# Run tests (optional)
echo "🧪 Running tests..."
docker run --rm $REGISTRY/$APP_NAME:$TAG kiren test

# Push to registry
echo "📤 Pushing to registry..."
docker push $REGISTRY/$APP_NAME:$TAG

# Deploy based on environment
case $ENVIRONMENT in
  "local")
    echo "🏠 Deploying locally with Docker Compose..."
    docker-compose up -d
    ;;
  "production")
    echo "🌐 Deploying to production..."
    # Update Kubernetes deployment
    kubectl set image deployment/kiren-app kiren=$REGISTRY/$APP_NAME:$TAG
    kubectl rollout status deployment/kiren-app
    ;;
  "staging")
    echo "🎭 Deploying to staging..."
    # Deploy to staging environment
    docker-compose -f docker-compose.staging.yml up -d
    ;;
esac

echo "✅ Deployment completed!"

# Health check
echo "🔍 Running health check..."
sleep 10
curl -f http://localhost:3000/healthcheck || {
    echo "❌ Health check failed!"
    exit 1
}

echo "🎉 Deployment successful and healthy!"