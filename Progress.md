# Phase 1: Core Foundation Implementation

## Goal
Basic routing with exact path matching - no parameters, minimal validation, just get routes working.

## Checklist

### 1. Project Structure Setup
- [x] Create workspace
- [x] Create `dioxus-fsrouter` crate
- [x] Create `dioxus-fsrouter-macro` crate
- [x] Configure Cargo.toml files
- [x] Set up basic README

### 2. Core Types (`dioxus-fsrouter/src/`)
- [ ] `route.rs` - Routable trait, RouteInfo struct
- [ ] `router.rs` - Router component, RouterContext
- [ ] `outlet.rs` - Outlet component
- [ ] `lib.rs` - Public API exports

### 3. Basic Macro (`dioxus-fsrouter-macro/src/`)
- [ ] `lib.rs` - Proc macro exports
- [ ] `route/mod.rs` - Basic #[route] attribute macro
- [ ] `route/parse.rs` - Parse route attribute
- [ ] `route/codegen.rs` - Generate Routable impl

### 4. Router Macro
- [ ] `router/mod.rs` - router! macro
- [ ] `router/codegen.rs` - Generate Router setup

### 5. Testing
- [ ] Basic integration test
- [ ] Example: basic.rs