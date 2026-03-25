# Veln REST API Documentation

The Veln REST API provides HTTP endpoints for managing FreeBSD virtual machines programmatically.

## Overview

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **CORS**: Enabled for browser clients

## Starting the API Server

### Command Line

```bash
# Start with default settings (binds to 127.0.0.1:8080)
veln api

# Custom bind address and port
veln api --bind 0.0.0.0 --port 8080

# Background with logging
veln api > /var/log/veln-api.log 2>&1 &
```

### Service (FreeBSD rc.d)

If you installed veln with the RC script option:

```bash
# Enable at boot
sysrc veln_enable="YES"

# Start service
service veln start

# Check status
service veln status
```

The service reads configuration from `/etc/rc.conf`:
```bash
veln_enable="YES"
veln_bind="127.0.0.1"
veln_port="8080"
```

## Authentication

Currently, the API does not implement authentication. It is designed to run on localhost or within trusted networks. For production use, place behind a reverse proxy with authentication (e.g., nginx with basic auth).

## Endpoints

### System Endpoints

#### Health Check
```http
GET /api/health
```

Returns the health status of the API server.

**Response:**
```json
{
  "status": "healthy",
  "service": "veln-api"
}
```

#### System Information
```http
GET /api/info
```

Returns system information including ZFS pool configuration and available resources.

**Response:**
```json
{
  "pool": "zroot",
  "vm_root": "veln",
  "resources": {
    "memory_mb": 16384,
    "cpu_cores": 8,
    "disk_gb": 500
  }
}
```

**Error Response (503):**
```json
{
  "error": "Root privileges required"
}
```

### VM Endpoints

#### List VMs
```http
GET /api/vms
```

Returns a list of all VMs.

**Response:**
```json
[
  {
    "name": "webserver",
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "state": "running",
    "cpus": 2,
    "memory_mb": 2048,
    "disk_gb": 40,
    "network": "tap:bridge0"
  },
  {
    "name": "database",
    "uuid": "660f9511-f3ac-52e5-b827-557766551111",
    "state": "stopped",
    "cpus": 4,
    "memory_mb": 4096,
    "disk_gb": 100,
    "network": "tap:bridge0"
  }
]
```

#### Get VM Details
```http
GET /api/vms/{name}
```

Returns detailed information about a specific VM.

**Parameters:**
- `name` (path): VM name

**Response:**
```json
{
  "name": "webserver",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "state": "running",
  "cpus": 2,
  "memory_mb": 2048,
  "disk_gb": 40,
  "network": "tap:bridge0"
}
```

**Error Response (404):**
```json
{
  "error": "VM 'webserver' not found"
}
```

#### Create VM
```http
POST /api/vms
Content-Type: application/json

{
  "name": "newvm",
  "cpus": 2,
  "memory": 1024,
  "disk": 20
}
```

Creates a new VM with the specified configuration.

**Request Body:**
- `name` (string, required): VM name
- `cpus` (integer, optional): Number of CPUs (default: 2)
- `memory` (integer, optional): Memory in MB (default: 1024)
- `disk` (integer, optional): Disk size in GB (default: 20)

**Response (201):**
```json
{
  "name": "newvm",
  "uuid": "770g0622-g4bd-63f6-c938-668877662222",
  "state": "stopped",
  "cpus": 2,
  "memory_mb": 1024,
  "disk_gb": 20,
  "network": "tap:bridge0"
}
```

**Error Response (409):**
```json
{
  "error": "VM 'newvm' already exists"
}
```

**Error Response (422):**
```json
{
  "error": "Insufficient memory: required 1024MB, available 512MB (keeping 512MB for host)"
}
```

#### Start VM
```http
POST /api/vms/{name}/start
```

Starts a VM.

**Parameters:**
- `name` (path): VM name

**Response:**
```json
{
  "status": "started",
  "vm": "webserver"
}
```

**Error Response (404):**
```json
{
  "error": "VM 'webserver' not found"
}
```

#### Stop VM
```http
POST /api/vms/{name}/stop
```

Stops a VM gracefully.

**Parameters:**
- `name` (path): VM name

**Response:**
```json
{
  "status": "stopped",
  "vm": "webserver"
}
```

**Error Response (500):**
```json
{
  "error": "Failed to stop VM: process not found"
}
```

#### Delete VM
```http
DELETE /api/vms/{name}
```

Deletes a VM permanently.

**Parameters:**
- `name` (path): VM name

**Response:**
```json
{
  "status": "deleted",
  "vm": "webserver"
}
```

**Error Response (404):**
```json
{
  "error": "VM 'webserver' not found"
}
```

## Error Handling

The API uses standard HTTP status codes:

| Status | Meaning |
|--------|---------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 400 | Bad Request - Invalid JSON or parameters |
| 404 | Not Found - Resource doesn't exist |
| 409 | Conflict - Resource already exists |
| 422 | Unprocessable Entity - Validation failed |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - System not ready |

## CORS

The API supports Cross-Origin Resource Sharing (CORS) for browser-based clients. The following headers are included in all responses:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

## Examples

### Create and Start a VM

```bash
# Create VM
curl -X POST http://localhost:8080/api/vms \
  -H "Content-Type: application/json" \
  -d '{
    "name": "webserver",
    "cpus": 2,
    "memory": 2048,
    "disk": 40
  }'

# Start VM
curl -X POST http://localhost:8080/api/vms/webserver/start

# Check status
curl http://localhost:8080/api/vms/webserver
```

### Python Client Example

```python
import requests

BASE_URL = "http://localhost:8080/api"

# Create VM
response = requests.post(f"{BASE_URL}/vms", json={
    "name": "testvm",
    "cpus": 2,
    "memory": 1024,
    "disk": 20
})
print(response.json())

# Start VM
response = requests.post(f"{BASE_URL}/vms/testvm/start")
print(response.json())

# List VMs
response = requests.get(f"{BASE_URL}/vms")
for vm in response.json():
    print(f"{vm['name']}: {vm['state']}")
```

### JavaScript Client Example

```javascript
const BASE_URL = 'http://localhost:8080/api';

// Create VM
async function createVM(name, cpus, memory, disk) {
  const response = await fetch(`${BASE_URL}/vms`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, cpus, memory, disk })
  });
  return response.json();
}

// List VMs
async function listVMs() {
  const response = await fetch(`${BASE_URL}/vms`);
  return response.json();
}

// Usage
createVM('webserver', 2, 2048, 40)
  .then(vm => console.log('Created:', vm))
  .catch(err => console.error('Error:', err));
```

## Limitations

- No built-in authentication (use reverse proxy)
- No HTTPS (use reverse proxy with SSL/TLS)
- VM console access not available via API (use CLI)
- ISO management not available via API (use CLI)
- Snapshot operations not available via API (use CLI)

## Future Enhancements

- Authentication tokens
- WebSocket support for real-time updates
- VM console via WebSocket
- ISO management endpoints
- Snapshot management endpoints
- Metrics and monitoring endpoints
