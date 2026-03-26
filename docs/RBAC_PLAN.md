# VELN RBAC (Role-Based Access Control) Planning Document

## Current Implementation: API Key Authentication

### Overview
VELN currently uses a simple API key-based authentication system with three predefined roles.

### Configuration

Add to your `/usr/local/etc/veln/config.toml`:

```toml
zfs_pool = "zroot"
vm_root = "/usr/local/vms"

[api]
auth_enabled = true

[api.keys]
# Admin - Full access
"veln-admin-your-secure-key-here" = { name = "Admin User", role = "admin" }

# Operator - Can start/stop VMs, view everything
"veln-operator-key-here" = { name = "Operator", role = "operator" }

# Viewer - Read-only access
"veln-viewer-key-here" = { name = "Viewer", role = "viewer" }
```

### Available Roles

#### 1. Admin
**Permissions:**
- `vms:read` - View all VMs
- `vms:write` - Create VMs
- `vms:delete` - Destroy VMs
- `vms:start` - Start VMs
- `vms:stop` - Stop VMs
- `system:read` - View system info

**Use Case:** System administrators with full control.

#### 2. Operator
**Permissions:**
- `vms:read` - View all VMs
- `vms:start` - Start VMs
- `vms:stop` - Stop VMs
- `system:read` - View system info

**Use Case:** DevOps engineers who need to manage VMs but shouldn't create/destroy them.

#### 3. Viewer
**Permissions:**
- `vms:read` - View all VMs
- `system:read` - View system info

**Use Case:** Monitoring dashboards, read-only access for support staff.

### Generating API Keys

```bash
# Generate a secure random key
openssl rand -hex 32

# Or use uuidgen
uuidgen | tr '[:upper:]' '[:lower:]'
```

### API Authentication

#### Method 1: Authorization Header (Recommended)
```bash
curl -H "Authorization: Bearer your-api-key-here" \
  http://localhost:8080/api/vms
```

#### Method 2: Query Parameter (for WebSocket)
```bash
curl "http://localhost:8080/api/vms?api_key=your-api-key-here"
```

#### Method 3: Login Endpoint
```bash
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-api-key-here"}'
```

Response:
```json
{
  "success": true,
  "name": "Admin User",
  "role": "admin",
  "permissions": [
    "vms:read",
    "vms:write",
    "vms:delete",
    "vms:start",
    "vms:stop",
    "system:read"
  ]
}
```

---

## Future RBAC Enhancements

### Phase 1: User Management (Planned for v0.2.0)

#### Features
- [ ] User database (SQLite/JSON file)
- [ ] Password-based authentication option
- [ ] Password hashing (bcrypt/argon2)
- [ ] Session tokens with expiration
- [ ] User profile management

#### Configuration
```toml
[api]
auth_type = "users"  # "api_key" or "users" or "pam"

[[api.users]]
username = "admin"
password_hash = "$2b$12$..."
role = "admin"
name = "Administrator"
```

### Phase 2: Granular Permissions (Planned for v0.3.0)

#### Features
- [ ] Custom permission sets
- [ ] Per-VM access control
- [ ] Time-based restrictions
- [ ] IP-based restrictions

#### Permission Granularity
```toml
[[api.users]]
username = "developer1"
role = "custom"
permissions = [
  "vms:read",
  "vms:start:web-server-*",    # Can start VMs matching pattern
  "vms:stop:web-server-*",
]
denied_permissions = [
  "vms:delete",
]
```

### Phase 3: Audit Logging (Planned for v0.4.0)

#### Features
- [ ] Action audit trail
- [ ] User activity logging
- [ ] Failed authentication attempts
- [ ] Log rotation

#### Audit Log Format
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "user": "admin",
  "action": "vms:start",
  "target": "web-server-01",
  "ip": "192.168.1.100",
  "success": true
}
```

### Phase 4: Multi-Tenancy (Planned for v0.5.0)

#### Features
- [ ] Project/tenant isolation
- [ ] Resource quotas per tenant
- [ ] Tenant-specific admins
- [ ] Cross-tenant VM sharing

#### Configuration
```toml
[[tenants]]
id = "engineering"
name = "Engineering Team"
quota_cpus = 32
quota_memory_gb = 128
quota_disk_gb = 1000

[[tenants.users]]
username = "eng-admin"
role = "tenant-admin"
tenant = "engineering"
```

---

## Security Best Practices

### 1. API Key Management
- Rotate keys regularly (every 90 days)
- Use different keys for different applications
- Store keys securely (environment variables, not in code)
- Revoke keys immediately when no longer needed

### 2. Network Security
- Use HTTPS in production
- Restrict API access to specific IP ranges
- Use VPN or private networks
- Enable firewall rules

### 3. Monitoring
- Monitor failed authentication attempts
- Alert on unusual activity
- Review audit logs regularly
- Set up intrusion detection

### 4. Backup
- Backup user database regularly
- Store backups securely
- Test restore procedures

---

## Migration Path

### From API Keys to User Management

1. **Backup existing config**
   ```bash
   cp /usr/local/etc/veln/config.toml /usr/local/etc/veln/config.toml.bak
   ```

2. **Enable user authentication**
   ```toml
   [api]
   auth_type = "users"
   
   # Keep API keys for backward compatibility
   [[api.keys]]
   key = "existing-key"
   name = "Legacy Integration"
   role = "admin"
   ```

3. **Create initial admin user**
   ```bash
   veln user add admin --role=admin
   ```

4. **Test new authentication**
   ```bash
   veln api --test-auth
   ```

5. **Migrate existing keys**
   - Create users for each key
   - Update integrations
   - Remove keys when migrated

---

## Implementation Checklist

### Current (v0.1.0)
- [x] API key authentication
- [x] Three predefined roles (admin, operator, viewer)
- [x] Permission checking
- [x] Login endpoint
- [x] Web UI authentication

### Phase 1 (v0.2.0)
- [ ] User database
- [ ] Password authentication
- [ ] Session management
- [ ] User CRUD operations

### Phase 2 (v0.3.0)
- [ ] Custom permissions
- [ ] Resource-level access control
- [ ] Time-based restrictions
- [ ] IP restrictions

### Phase 3 (v0.4.0)
- [ ] Audit logging
- [ ] Activity monitoring
- [ ] Security alerts
- [ ] Compliance reporting

### Phase 4 (v0.5.0)
- [ ] Multi-tenancy
- [ ] Resource quotas
- [ ] Tenant isolation
- [ ] Cross-tenant sharing

---

## Questions for Future Discussion

1. **Identity Provider Integration**
   - Should we support LDAP/Active Directory?
   - OAuth2/OpenID Connect support?
   - SAML for enterprise?

2. **Two-Factor Authentication**
   - TOTP (Google Authenticator)?
   - U2F/WebAuthn (YubiKey)?
   - SMS/Email verification?

3. **API Rate Limiting**
   - Per-user rate limits?
   - Different limits per role?
   - Burst handling?

4. **Single Sign-On (SSO)**
   - Integration with corporate SSO?
   - Kerberos support?
   - Certificate-based auth?

---

## References

- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [NIST Digital Identity Guidelines](https://pages.nist.gov/800-63-3/)
- [FreeBSD Security Best Practices](https://docs.freebsd.org/en/books/handbook/security/)
