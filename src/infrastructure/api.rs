//! REST API for veln
//!
//! Provides HTTP endpoints for VM management.
//!
//! ## Authentication
//!
//! API uses Bearer token authentication via the `Authorization` header:
//! ```
//! Authorization: Bearer <api-key>
//! ```
//!
//! Or via query parameter (for WebSocket connections):
//! ```
//! GET /api/vms?api_key=<api-key>
//! ```
//!
//! ## Endpoints
//!
//! ### Authentication
//! - `POST /api/login` - Validate API key and return user info
//!
//! ### VMs
//! - `GET /api/vms` - List all VMs
//! - `GET /api/vms/{name}` - Get VM details
//! - `POST /api/vms` - Create VM
//! - `POST /api/vms/{name}/start` - Start VM
//! - `POST /api/vms/{name}/stop` - Stop VM
//! - `DELETE /api/vms/{name}` - Delete VM
//!
//! ### System
//! - `GET /api/health` - Health check (no auth required)
//! - `GET /api/info` - System information

use crate::config::{ApiKey, Config};
use crate::domain::vm::{VirtualMachine, VmConfig, VmState};
use crate::domain::{RequirementsChecker, ResourceMonitor, VmRepository, VmRuntime};
use crate::error::VelnError;
use crate::infrastructure::bhyve::BhyveRuntime;
use crate::infrastructure::zfs::ZfsRepository;
use rouille::{Request, Response};
use serde::{Deserialize, Serialize};

/// API server state
pub struct ApiServer {
    config: Config,
    zfs_repo: ZfsRepository,
    bhyve_runtime: BhyveRuntime,
}

/// Authentication context for requests
struct AuthContext {
    api_key: String,
    key_info: ApiKey,
}

impl ApiServer {
    /// Create a new API server
    /// # Errors
    /// Returns error if configuration cannot be loaded
    pub fn new() -> crate::error::Result<Self> {
        let config = Config::load()?;
        let zfs_repo = ZfsRepository::new(&config)?;
        let bhyve_runtime = BhyveRuntime::new(config.zfs_pool.clone(), config.vm_root.clone());

        Ok(Self {
            config,
            zfs_repo,
            bhyve_runtime,
        })
    }

    /// Start the HTTP server
    pub fn serve(self, bind: &str, port: u16) {
        let addr = format!("{bind}:{port}");
        println!("Starting veln API server on http://{addr}");
        
        if self.config.api.auth_enabled {
            println!("API authentication enabled with {} key(s)", self.config.api.keys.len());
        } else {
            println!("WARNING: API authentication disabled");
        }

        rouille::start_server(&addr, move |request| {
            self.handle_request(request)
        });
    }

    fn handle_request(&self, request: &Request) -> Response {
        // Handle OPTIONS for CORS preflight
        if request.method() == "OPTIONS" {
            return self.cors_response(Response::empty_204());
        }

        // Public endpoints (no auth required)
        if request.method() == "GET" && request.url() == "/api/health" {
            return self.cors_response(self.health_check());
        }

        // Login endpoint
        if request.method() == "POST" && request.url() == "/api/login" {
            return self.cors_response(self.login(request));
        }

        // All other endpoints require authentication
        let auth = match self.authenticate(request) {
            Ok(auth) => auth,
            Err(response) => return self.cors_response(response),
        };

        // Route the request
        let response = match (request.method(), request.url().as_str()) {
            ("GET", "/api/info") => self.system_info(),
            ("GET", "/api/vms") => self.list_vms(),
            ("GET", path) if path.starts_with("/api/vms/") => {
                let name = &path[9..]; // Remove "/api/vms/"
                if name.contains('/') {
                    self.vm_action(request, name, &auth)
                } else {
                    self.get_vm(name)
                }
            }
            ("POST", "/api/vms") => self.create_vm(request, &auth),
            _ => Response::empty_404(),
        };

        self.cors_response(response)
    }

    fn cors_response(&self, response: Response) -> Response {
        response
            .with_additional_header("Access-Control-Allow-Origin", "*")
            .with_additional_header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
            .with_additional_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
    }

    /// Extract and validate API key from request
    fn authenticate(&self, request: &Request) -> Result<AuthContext, Response> {
        // Try Authorization header first
        let api_key = if let Some(auth_header) = request.header("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                token.to_string()
            } else {
                return Err(Response::json(&serde_json::json!({
                    "error": "Invalid Authorization header format. Use: Bearer <api-key>"
                })).with_status_code(401));
            }
        } else {
            // Try query parameter
            let query = request.get_param("api_key");
            match query {
                Some(key) => key,
                None => {
                    return Err(Response::json(&serde_json::json!({
                        "error": "Missing API key. Provide via Authorization header or api_key query parameter"
                    })).with_status_code(401));
                }
            }
        };

        // Validate the key
        match self.config.validate_api_key(&api_key) {
            Some(key_info) => Ok(AuthContext {
                api_key,
                key_info: key_info.clone(),
            }),
            None => Err(Response::json(&serde_json::json!({
                "error": "Invalid API key"
            })).with_status_code(401)),
        }
    }

    fn login(&self, request: &Request) -> Response {
        #[derive(Deserialize)]
        struct LoginRequest {
            api_key: String,
        }

        let body: LoginRequest = match rouille::input::json_input(request) {
            Ok(b) => b,
            Err(e) => {
                return Response::json(&serde_json::json!({
                    "error": format!("Invalid JSON: {e}")
                })).with_status_code(400);
            }
        };

        match self.config.validate_api_key(&body.api_key) {
            Some(key_info) => {
                Response::json(&serde_json::json!({
                    "success": true,
                    "name": key_info.name,
                    "role": key_info.role,
                    "permissions": self.get_permissions(&key_info.role)
                }))
            }
            None => Response::json(&serde_json::json!({
                "success": false,
                "error": "Invalid API key"
            })).with_status_code(401)
        }
    }

    fn get_permissions(&self, role: &str) -> Vec<String> {
        match role {
            "admin" => vec![
                "vms:read".to_string(),
                "vms:write".to_string(),
                "vms:delete".to_string(),
                "vms:start".to_string(),
                "vms:stop".to_string(),
                "system:read".to_string(),
            ],
            "operator" => vec![
                "vms:read".to_string(),
                "vms:start".to_string(),
                "vms:stop".to_string(),
                "system:read".to_string(),
            ],
            "viewer" => vec![
                "vms:read".to_string(),
                "system:read".to_string(),
            ],
            _ => vec!["vms:read".to_string()],
        }
    }

    #[allow(clippy::unused_self)]
    fn health_check(&self) -> Response {
        Response::json(&serde_json::json!({
            "status": "healthy",
            "service": "veln-api",
            "version": env!("CARGO_PKG_VERSION"),
            "auth_enabled": self.config.api.auth_enabled
        }))
    }

    fn system_info(&self) -> Response {
        match RequirementsChecker::verify_or_fail() {
            Ok(()) => {
                let resources_info = match ResourceMonitor::get_resources() {
                    Ok(resources) => serde_json::json!({
                        "memory_mb": resources.total_memory_mb,
                        "cpu_cores": resources.cpu_cores,
                        "disk_gb": resources.total_disk_gb
                    }),
                    Err(_) => serde_json::json!(null)
                };
                Response::json(&serde_json::json!({
                    "pool": self.config.zfs_pool,
                    "vm_root": self.config.vm_root,
                    "resources": resources_info,
                    "auth_enabled": self.config.api.auth_enabled
                }))
            }
            Err(e) => Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(503)
        }
    }

    fn list_vms(&self) -> Response {
        match self.zfs_repo.list() {
            Ok(vms) => {
                let vm_list: Vec<VmInfo> = vms.iter().map(VmInfo::from).collect();
                Response::json(&vm_list)
            }
            Err(e) => Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(500)
        }
    }

    fn get_vm(&self, name: &str) -> Response {
        match self.zfs_repo.load(name) {
            Ok(vm) => Response::json(&VmInfo::from(&vm)),
            Err(VelnError::VmNotFound(_)) => Response::empty_404(),
            Err(e) => Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(500)
        }
    }

    fn create_vm(&self, request: &Request, auth: &AuthContext) -> Response {
        // Check permissions
        if auth.key_info.role != "admin" {
            return Response::json(&serde_json::json!({
                "error": "Insufficient permissions. Admin role required."
            })).with_status_code(403);
        }

        #[derive(Deserialize)]
        struct CreateVmRequest {
            name: String,
            #[serde(default = "default_cpus")]
            cpus: u8,
            #[serde(default = "default_memory")]
            memory: u64,
            #[serde(default = "default_disk")]
            disk: u64,
        }

        fn default_cpus() -> u8 { 2 }
        fn default_memory() -> u64 { 1024 }
        fn default_disk() -> u64 { 20 }

        let body: CreateVmRequest = match rouille::input::json_input(request) {
            Ok(b) => b,
            Err(e) => {
                return Response::json(&serde_json::json!({
                    "error": format!("Invalid JSON: {e}")
                })).with_status_code(400);
            }
        };

        // Check if VM already exists
        if self.zfs_repo.exists(&body.name) {
            return Response::json(&serde_json::json!({
                "error": format!("VM '{}' already exists", body.name)
            })).with_status_code(409);
        }

        // Check resources
        if let Err(e) = ResourceMonitor::can_create_vm(body.memory, body.disk, body.cpus) {
            return Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(422);
        }

        let vm_config = VmConfig {
            cpus: body.cpus,
            memory_mb: body.memory,
            disk_gb: body.disk,
            ..VmConfig::default()
        };

        let mut vm = VirtualMachine::with_config(body.name.clone(), vm_config);
        if let Err(e) = vm.transition_to(VmState::Stopped) {
            return Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(500);
        }

        match self.zfs_repo.save(&vm) {
            Ok(()) => Response::json(&VmInfo::from(&vm)).with_status_code(201),
            Err(e) => Response::json(&serde_json::json!({
                "error": format!("{e}")
            })).with_status_code(500)
        }
    }

    fn vm_action(&self, request: &Request, path: &str, auth: &AuthContext) -> Response {
        // Parse path like "myvm/start" or "myvm/stop"
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Response::empty_404();
        }

        let vm_name = parts[0];
        let action = parts[1];

        if !self.zfs_repo.exists(vm_name) {
            return Response::json(&serde_json::json!({
                "error": format!("VM '{vm_name}' not found")
            })).with_status_code(404);
        }

        // Check permissions for write operations
        let requires_admin = matches!(action, "start" | "stop" | "destroy");
        if requires_admin && !matches!(auth.key_info.role.as_str(), "admin" | "operator") {
            return Response::json(&serde_json::json!({
                "error": "Insufficient permissions. Operator or Admin role required."
            })).with_status_code(403);
        }

        match action {
            "start" => {
                if request.method() != "POST" {
                    return Response::empty_404();
                }
                match self.zfs_repo.load(vm_name) {
                    Ok(vm) => {
                        match self.bhyve_runtime.start(&vm) {
                            Ok(()) => Response::json(&serde_json::json!({
                                "status": "started",
                                "vm": vm_name
                            })),
                            Err(e) => Response::json(&serde_json::json!({
                                "error": format!("{e}")
                            })).with_status_code(500)
                        }
                    }
                    Err(e) => Response::json(&serde_json::json!({
                        "error": format!("{e}")
                    })).with_status_code(500)
                }
            }
            "stop" => {
                if request.method() != "POST" {
                    return Response::empty_404();
                }
                match self.bhyve_runtime.stop(vm_name) {
                    Ok(()) => Response::json(&serde_json::json!({
                        "status": "stopped",
                        "vm": vm_name
                    })),
                    Err(e) => Response::json(&serde_json::json!({
                        "error": format!("{e}")
                    })).with_status_code(500)
                }
            }
            _ => Response::empty_404()
        }
    }
}

/// VM information for API responses
#[derive(Serialize)]
struct VmInfo {
    name: String,
    uuid: String,
    state: String,
    cpus: u8,
    memory_mb: u64,
    disk_gb: u64,
    network: String,
}

impl From<&VirtualMachine> for VmInfo {
    fn from(vm: &VirtualMachine) -> Self {
        let network = match &vm.config.network.backend {
            crate::domain::vm::NetworkBackend::TapBridge { bridge } => {
                format!("tap:{bridge}")
            }
            crate::domain::vm::NetworkBackend::Vale { switch } => {
                format!("vale:{switch}")
            }
        };

        Self {
            name: vm.name.clone(),
            uuid: vm.uuid.0.clone(),
            state: vm.state.to_string(),
            cpus: vm.config.cpus,
            memory_mb: vm.config.memory_mb,
            disk_gb: vm.config.disk_gb,
            network,
        }
    }
}
