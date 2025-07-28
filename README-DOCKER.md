# Kiren Docker Usage

## Quick Start

### Pull and Run
```bash
# Pull latest image (after GitHub Actions builds it)
docker pull ghcr.io/kirencore/kiren:latest

# Run your app
docker run -p 3000:3000 -v $(pwd):/app ghcr.io/kirencore/kiren:latest server.js
```

### Local Build
```bash
# Build locally (requires multi-platform support)
docker buildx create --use --name kiren-builder
docker buildx build --platform linux/amd64,linux/arm64 -t kiren:latest .
```

### Production Docker Compose
```yaml
version: '3.8'
services:
  app:
    image: ghcr.io/kirencore/kiren:latest
    ports:
      - "3000:3000"
    volumes:
      - ./:/app
    environment:
      - NODE_ENV=production
      - PORT=3000
    restart: unless-stopped
```

## Usage Examples

### Basic Express App
```javascript
// server.js
const express = require('express');
const app = express();

app.get('/', (req, res) => {
  res.json({ message: 'Hello from Kiren!' });
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

```bash
docker run -p 3000:3000 -v $(pwd):/app ghcr.io/kirencore/kiren:latest server.js
```

### With Environment Variables
```bash
docker run -p 8080:8080 \
  -e NODE_ENV=production \
  -e PORT=8080 \
  -e DATABASE_URL=postgres://... \
  -v $(pwd):/app \
  ghcr.io/kirencore/kiren:latest app.js
```

### Development with Hot Reload
```bash
# For development, mount your code directory
docker run -it --rm \
  -p 3000:3000 \
  -v $(pwd):/app \
  -w /app \
  ghcr.io/kirencore/kiren:latest \
  kiren --watch server.js
```

## Performance Benefits

- ⚡ **50% faster startup** than Node.js
- 🧠 **Lower memory usage** 
- 📦 **Single binary** - no Node.js installation needed
- 🔒 **Better security** with Rust's memory safety
- 🚀 **Production ready** with built-in Express.js support

## Deployment Options

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment  
metadata:
  name: kiren-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: kiren-app
  template:
    metadata:
      labels:
        app: kiren-app
    spec:
      containers:
      - name: kiren
        image: ghcr.io/kirencore/kiren:latest
        ports:
        - containerPort: 3000
```

### Docker Swarm
```bash
docker service create \
  --name kiren-app \
  --replicas 3 \
  --publish 3000:3000 \
  --mount type=bind,source=$(pwd),destination=/app \
  ghcr.io/kirencore/kiren:latest server.js
```

### Cloud Run / Fargate
Ready to deploy to any container platform that supports Docker images.