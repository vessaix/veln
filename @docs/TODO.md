VelN - VM orchestration with bhyve on FreeBSD

What I did reading this week
- Porter's Handbook: guidance on building, packaging, testing, and auditing ports; emphasis on reproducibility, security, and clear maintainers' workflows.
- bhyve manual: how to configure a VM (memory, CPUs, devices, bootloaders, networking, storage) and how to start/stop/manage VMs; important for translating VM specs into bhyve invocations.

Why this matters for VelN
- We need a clean, secure, testable codebase to manage bhyve VMs from a Rust backend with a Vue frontend later.
- Start with small, well-audited components; add audits, tests, and reviews incrementally.
- The plan is to wrap bhyve with a Rust launcher that translates a VM spec into a bhyve command or a bhyve configuration file, then expose a REST API to control VMs.

Proposed architecture (high level)
- Backend (Rust): core VM model, a launcher/wrapper for bhyve, a small REST API (e.g., Actix/warp) to manage VMs, and a simple state store.
- Frontend (Vue): UI to create/list/start/stop VMs later, backed by the REST API.
- Data store: simple local JSON/SQLite store for VM state; avoid external dependencies for MVP.
- Security: run a dedicated VelN user, drop privileges, minimize root access, validate inputs, and limit file system exposure.

Phases and TODOs
Phase 0 – Scaffolding and core types (this week)
- [ ] Create repository workspace layout for Rust backend and (placeholder) Vue frontend.
- [ ] Define core VMConfig and Disk/network models in Rust (serializable to JSON).
- [ ] Implement a minimal bhyve launcher wrapper (translate VMConfig to a bhyve command string).
- [ ] Add a small in-process state store to track VMs (in-memory first, then disk-backed).
- [ ] Set up a basic REST API skeleton with endpoints:
  - POST /vm to create VM config
  - GET /vm to list VMs
  - GET /vm/{id} to fetch VM details
  - POST /vm/{id}/start to launch
  - POST /vm/{id}/stop to terminate
- [ ] Add logging and basic error handling; propagate errors clearly.
- [ ] Write initial unit tests for config translation and command construction.

Phase 1 – VM lifecycle and security hardening
- [ ] Complete launcher integration with bhyve (handle memory, CPUs, disks, networks).
- [ ] Implement VM status polling and PID tracking; handle restarts cleanly.
- [ ] Harden security: validate allowed VM parameters; sandbox launcher; non-root execution.
- [ ] Add basic auth or token-based protection for API in MVP (or API key placeholder).

Phase 2 – Web UI scaffold (Vue) and API polish
- [ ] Create basic Vue app scaffold and connect to API endpoints.
- [ ] Build simple VM creation form and list view; wire to API.
- [ ] Add frontend routing and responsive layout; ensure accessibility basics.

Phase 3 – Packaging, CI, and QA
- [ ] Add CI workflow (build, tests, lint).
- [ ] Create a minimal FreeBSD-friendly packaging layout (port-like or binary tarball) as a future step.
- [ ] Audit code for security, add static checks, and document audit findings.

Notes
- We will not start coding without your review. Please approve the high-level plan or adjust priorities.
- If you want to start with a specific phase, tell me Phase 0, Phase 1, or a subset, and I’ll proceed accordingly.
