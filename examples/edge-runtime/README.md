# Kiren Edge Runtime Example

Self-hosted edge runtime - Cloudflare Workers style API.

## Usage

```bash
cd examples/edge-runtime
kiren edge serve
```

## Files

```
edge-runtime/
├── kiren.edge.json    # Config file
├── workers/
│   └── api.js         # API worker
└── README.md
```

## Test

```bash
curl http://localhost:3000/api/hello
# {"message":"Hello from Kiren Edge!"}

curl http://localhost:3000/api/time
# {"time":"2024-01-28T12:00:00.000Z"}
```

## Add More Workers

1. Create a new file in `workers/`:

```javascript
// workers/auth.js
export default {
  fetch(request) {
    return Response.json({ authenticated: true });
  }
};
```

2. Add to config:

```json
{
  "port": 3000,
  "workers": {
    "api": { "path": "./workers/api.js", "routes": ["/api/*"] },
    "auth": { "path": "./workers/auth.js", "routes": ["/auth/*"] }
  }
}
```

3. Restart server.
