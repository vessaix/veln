VelN - VM orchestration with bhyve on FreeBSD

What this document covers
- Rationale from FreeBSD Porter's Handbook and bhyve manual, guiding secure, auditable development.
- MVP strategy: small, well-audited code paths; Rust backend + Vue frontend later.
- High-level architecture: VM spec -> bhyve launcher wrapper -> REST API -> optional web UI.

Phases (high level)
- Phase 0: Scaffolding and core types (VMConfig, DiskSpec, NetworkSpec), launcher wrapper, in-memory VM registry, API skeleton.
- Phase 1: Full VM lifecycle (start/stop/status), security hardening, input validation.
- Phase 2: Web UI scaffold (Vue) and API polish.
- Phase 3: Packaging, CI, QA, and documentation.

Initial API surface (MVP)
- POST /vm: create VM config
- GET /vm: list VMs
- GET /vm/{id}: VM details
- POST /vm/{id}/start: start VM
- POST /vm/{id}/stop: stop VM

Security best practices (MVP)
- Run launcher under a dedicated user, minimal host exposure.
- Validate inputs strictly and provide safe defaults.
- Plan for API authentication in a minimal form.

Next steps
- Await your review to confirm Phase 0 start; I will then implement the Rust scaffolding and API stubs.
