# Phase 1: Core Foundation Implementation

## Goal

Stand up **exact-path routing** (no params yet) backed by a dedicated runtime crate and a single `#[route]` proc macro.

- Routes attach directly to components: `#[route("/path")]`
- Registration happens automatically via `inventory` at compile time
- Runtime provides `Router`, `Outlet`, `Link`, and a `use_navigation` hook
- Simple example demonstrates the API end to end

---

## 1.1 Project Structure Setup

Workspace layout:

```text
dioxus-fsrouter/
  Cargo.toml

  packages/
    fsrouter/
      Cargo.toml
      src/
        lib.rs
        errors.rs
        route/
          mod.rs
          validate.rs
        router/
          mod.rs
          components.rs
          navigation.rs
        tests/
          mod.rs

    fsrouter-macro/
      Cargo.toml
      src/
        lib.rs

  examples/
    basic/
      Cargo.toml
      assets/
        main.css
      src/
        main.rs
```

### Checklist
* [x] Configure workspace `Cargo.toml`
* [x] Create runtime crate (`packages/fsrouter`)
* [x] Create proc-macro crate (`packages/fsrouter-macro`)
* [x] Set workspace dependencies + metadata
* [x] Add top-level README
* [x] Scaffold `examples/basic`

---

## 1.2 Core Types and Functions (`packages/fsrouter/src/`)

Runtime logic lives inside the `fsrouter` crate with clear module boundaries.

### 1.2.1 Route Module (`route/`)

The route subsystem owns metadata, registration, matching, and validation.

Files:

* `route/mod.rs`
* `route/validate.rs`

#### `route/mod.rs`

Exports `RouteInfo`, the render fn type alias, and helpers for iterating registered routes.

Checklist:
* [x] Define `RouteInfo` with `path`, `component_name`, and `render_fn`
* [x] Collect routes globally with `inventory::collect!(RouteInfo)`
* [x] Provide `get_routes()` iterator
* [x] Provide `find_route(path: &str)` exact matcher (Phase 1 scope)

#### `route/validate.rs`

Validates the registered routes before rendering starts.

Checklist:
* [x] Detect duplicate paths
* [x] Detect “no routes registered”
* [x] Return aggregated `ValidationErrors`
* [x] Expose `validate_routes_or_panic()` for convenience

### 1.2.2 Error Types (`errors.rs`)

Shared error definitions used by both the runtime and proc macro diagnostics.

Checklist:
* [x] Define `RouterError` enum (duplicate route, no routes, invalid path)
* [x] Add `ValidationErrors` accumulator
* [x] Ensure errors implement `std::error::Error` + Display

---

## 1.3 Router Components (`router/`)

The router module exposes the user-facing components and navigation state.

Files:

* `router/mod.rs`
* `router/components.rs`
* `router/navigation.rs`

### 1.3.1 Components

`Router`, `Outlet`, and `Link` live in `components.rs`.

Checklist:
* [x] `Router` sets up the navigation signal, validates routes on mount, and wires WASM popstate listeners
* [x] Provide `NavigationContext` via `use_context_provider`
* [x] `Outlet` consumes context, runs `find_route`, and renders the active component (or a 404 fallback)
* [x] `Link` renders `<a>` tags and delegates navigation through the hook

### 1.3.2 Navigation

`router/navigation.rs` implements programmatic navigation primitives.

Checklist:
* [x] Define `Navigation` struct backed by `Signal<String>`
* [x] Implement `push`, `replace`, `go_back`, `go_forward`, and `current_path`
* [x] Integrate browser history APIs when targeting WASM
* [x] Provide `use_navigation()` hook for consuming components

---

## 1.4 Public API (`lib.rs`)

Ties everything together:

* Re-exports runtime types (`Router`, `Outlet`, `Link`, `Navigation`, `get_routes`, `validate_routes`, etc.)
* Re-exports the macro crate as `dioxus_fsrouter::macros` and exposes `inventory` for the proc macro
* Provides a `prelude` module so downstream apps can `use dioxus_fsrouter::prelude::*;`

Checklist:
* [x] Public API exports finalized
* [x] Prelude assembled for ergonomic consumer imports

---

## 1.5 Proc Macro (`packages/fsrouter-macro`)

The `#[route("/path")]` attribute marks Dioxus components as routes.

Checklist:
* [x] Validate the literal path (must start with `/`, no trailing slash except `/`, no `//`, spaces, `?`, or `:`)
* [x] Reject components with parameters (Phase 2 feature)
* [x] Generate wrapper render fn calling the user component
* [x] Submit `RouteInfo` via `inventory::submit!` referencing the runtime crate

Notes:
* Any unsupported pattern fails compilation with targeted guidance
* The macro already imports `::dioxus_fsrouter` so downstream users only need `dioxus-fsrouter = { ... }`

---

## 1.6 Example App (`examples/basic`)

Minimal showcase that assembles the public API.

Checklist:
* [x] Three routes (`/`, `/about`, `/contact`) using `#[route]`
* [x] Global layout with `Router`, `NavBar`, and `Outlet`
* [x] Link-based navigation showcasing the hook
* [x] Static CSS injected via `document::Stylesheet`
* [x] Helper function to print `get_routes()` (manual sanity check)

---

## 1.7 Testing

### 1.7.1 Runtime Unit Tests

Location: `packages/fsrouter/src/tests.rs`

Checklist:
* [x] Validate `RouterError` Display output
* [x] Validate `ValidationErrors` aggregation helpers

### 1.7.2 Integration / Smoke Tests

Checklist:
* [x] Register routes via the macro and assert `get_routes().len() >= 2`
* [x] Assert `find_route("/")` and `find_route("/about")` return `Some`
* [x] Ensure `validate_routes()` succeeds with unique routes
* [ ] Cover WASM/history hooks (headless test harness or wasm-bindgen test target)

### 1.7.3 Example Coverage

Checklist:
* [ ] Add CI job to build `examples/basic` (desktop + wasm)
* [ ] Verify `Router` renders `Outlet` correctly via screenshot/snapshot test (optional)