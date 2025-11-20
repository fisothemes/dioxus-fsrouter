# Phase 1: Core Foundation

## Goal

Establish **exact-path routing** (no params yet) backed by a dedicated runtime crate and a single `#[route]` proc macro.

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
* [x] Public API exports finalised
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

---

# Phase 2: Route Parameters

## Goal

Support dynamic route segments like `/user/:id` with automatic parameter extraction and type conversion.

- Parameters passed as component props
- `FromStr` trait for type conversion
- 404 or `#[fallback]` on parse errors
- Ordering for multiple parameters
- Exact matching by default (options later)

---

## 2.1 Architecture Updates

Route parameter support stays backward compatible while layering in new runtime shapes.

### 2.1.1 Render Function Variants

```rust
pub enum RenderFn {
    Static(fn() -> Element),                             // Phase 1
    WithParams(fn(HashMap<String, String>) -> Element),  // Phase 2
}
```

Benefits:
* Backward compatible
* Type-safe
* Zero runtime overhead
* Clear intent

### 2.1.2 RouteInfo Fields

```rust
pub struct RouteInfo {
    path: &'static str,
    pattern: OnceCell<RoutePattern>,  // Lazy-initialised
    component_name: &'static str,
    render_fn: RenderFn,              // Now an enum
}
```

### 2.1.3 Pattern Model

```rust
pub struct RoutePattern {
    raw: String,
    segments: Vec<Segment>,
    priority: i32,
}

pub enum Segment {
    Static(String),  // "user"
    Param(String),   // ":id"
}
```

---

## 2.2 Pattern Matching Core (`packages/fsrouter/src/route/`)

Files:
* `packages/fsrouter/src/route/pattern.rs` (new)
* `packages/fsrouter/src/route/mod.rs` (update)

Checklist:
* [x] Create `RoutePattern` struct
* [x] Create `Segment` enum
* [x] Implement `parse()`
* [x] Implement `matches()`
* [x] Implement `calculate_priority()`
* [x] Write comprehensive tests

### 2.2.1 Priority Algorithm

To ensure deterministic routing when multiple patterns match a URL (e.g., `/user/new` vs `/user/:id`), the router calculates a priority score. Higher scores take precedence, prioritising specificity and depth.

Scoring Rules:
*   Static Segments: +100 points (specific matches are preferred)
*   Dynamic Segments: +50 points (generic matches are secondary)
*   Length Bonus: +1 point per segment (deeper routes are preferred)

Examples:

```text
/user/new            = 100 + 100 + 2 = 202  (Winner over /user/:id)
/user/:id            = 100 +  50 + 2 = 152
/user/:id/posts      = 100 +  50 + 100 + 3 = 253
/:type/:id           =  50 +  50 + 2 = 102
```

---

## 2.3 Route Module (`packages/fsrouter/src/route/mod.rs`)

Checklist:
* [ ] Add `RenderFn` enum
* [ ] Update `RouteInfo` constructors:
  * `new_static()` - Phase 1 routes
  * `new_with_params()` - Phase 2 routes
* [ ] Add lazy pattern initialisation with `OnceCell`
* [ ] Update `find_route()` to use pattern matching
* [ ] Add `find_route_exact()` for Phase 1 compatibility
* [ ] Sort routes by priority

Key function:

```rust
pub fn find_route(path: &str) -> Option<(&'static RouteInfo, HashMap<String, String>)> {
    let mut routes: Vec<_> = get_routes().collect();
    routes.sort_by(|a, b| b.priority().cmp(&a.priority()));
    
    for route in routes {
        if let Some(params) = route.matches(path) {
            return Some((route, params));
        }
    }
    None
}
```

---

## 2.4 Proc Macro (`packages/fsrouter-macro/src/lib.rs`)

Checklist:
* [ ] Detect `:param` syntax in path
* [ ] Extract component parameter names and types
* [ ] Validate route params match component props
* [ ] Generate appropriate wrapper:
  * Static wrapper for no params
  * Dynamic wrapper with `FromStr` parsing
* [ ] Handle parse errors:
  * Debug: panic with helpful message
  * Release: return default value (triggers 404)
* [ ] Add new validation errors

Validation checks:
```rust
// Route param not in component
#[route("/user/:id")]
fn User(name: String) -> Element { ... }

// Duplicate param names
#[route("/user/:id/post/:id")]
fn UserPost(id: String) -> Element { ... }

// Empty param name
#[route("/user/:")]
fn User() -> Element { ... }
```

Generated code example:
```rust
// Input:
#[route("/user/:id")]
#[component]
fn UserProfile(id: String) -> Element { ... }

// Output:
fn __render_UserProfile(params: HashMap<String, String>) -> Element {
    let id = params.get("id")
        .and_then(|s| s.parse::<String>().ok())
        .unwrap_or_else(|| {
            #[cfg(debug_assertions)]
            panic!("Failed to parse 'id' as String");
            
            #[cfg(not(debug_assertions))]
            String::new()
        });

    UserProfile { id }
}

inventory::submit! {
    RouteInfo::new_with_params("/user/:id", "module::UserProfile", __render_UserProfile)
}
```

---

## 2.5 Router Components (`packages/fsrouter/src/router/components.rs`)

Checklist:
* [ ] Update `Outlet` to use new `find_route()` signature
* [ ] Pass parameters to the render function
* [ ] Handle parse failures (404 or fallback)

Updated Outlet:
```rust
#[component]
pub fn Outlet() -> Element {
    let nav_ctx = use_context::<NavigationContext>();
    let path = nav_ctx.current_route();
    
    match find_route(&path) {
        Some((route, params)) => {
            route.render(if params.is_empty() {
                None
            } else {
                Some(params)
            })
        }
        None => {
            rsx! { div { "404 - Not Found: {path}" } }
        }
    }
}
```

---

## 2.6 Validation & Errors (`packages/fsrouter/src/route/validate.rs`, `packages/fsrouter/src/errors.rs`)

Checklist:
* [ ] Add new error variants:
  * `DuplicateParam` - Same param name twice
  * `EmptyParam` - `:` with no name
  * `MismatchedParams` - Route param not in component
* [ ] Validate parameter names
* [ ] Check for conflicts

New errors:
```rust
#[derive(Error, Debug, Clone)]
pub enum RouterError {
    // ... existing variants ...
    
    #[error("Duplicate parameter '{param}' in route '{path}'")]
    DuplicateParam { path: String, param: String },
    #[error("Empty parameter name in route '{path}'")]
    EmptyParam { path: String },
    #[error("Route parameter '{param}' in '{path}' not found in component props")]
    MismatchedParam {
        path: String,
        param: String,
        component: String,
    },
}
```

---

## 2.7 Example App (`examples/basic`)

Checklist:
* [ ] Add routes with parameters:
  ```rust
  #[route("/user/:id")]
  fn UserProfile(id: String) -> Element { ... }

  #[route("/post/:id")]
  fn Post(id: u32) -> Element { ... }

  #[route("/user/:user_id/posts/:post_id")]
  fn UserPost(user_id: String, post_id: u32) -> Element { ... }
  ```
* [ ] Add navigation to parameterised routes
* [ ] Show parameter values in UI
* [ ] Test parse failures

---

## 2.8 Testing

### 2.8.1 Unit Tests

Checklist:
* [ ] Pattern parsing
* [ ] Pattern matching
* [ ] Priority calculation
* [ ] Parameter extraction
* [ ] Type conversion (`FromStr`)
* [ ] Parse error handling

### 2.8.2 Integration Tests

Checklist:
* [ ] Register mixed routes (static + dynamic)
* [ ] Match URLs against patterns
* [ ] Extract and parse parameters
* [ ] Verify priority ordering
* [ ] Test parse failures -> 404

Example test:
```rust
#[test]
fn test_dynamic_route_matching() {
    let routes = vec![
        RouteInfo::new_static("/about", "About", render_about),
        RouteInfo::new_with_params("/user/:id", "User", render_user),
    ];

    // Static route matches
    let (route, params) = find_route("/about").unwrap();
    assert_eq!(route.path(), "/about");
    assert!(params.is_empty());
    
    // Dynamic route matches
    let (route, params) = find_route("/user/123").unwrap();
    assert_eq!(route.path(), "/user/:id");
    assert_eq!(params.get("id"), Some(&"123".to_string()));
}
```

---

## 2.9 Migration Path

Phase 1 code:
```rust
#[route("/about")]
#[component]
fn About() -> Element {
    rsx! { div { "About" } }
}
```

Phase 2 code (new feature):
```rust
#[route("/user/:id")]
#[component]
fn UserProfile(id: String) -> Element {
    rsx! { div { "User: {id}" } }
}
```

Backward compatibility:
* All Phase 1 routes continue working
* No breaking changes
* Can mix static and dynamic routes
* `find_route_exact()` available for legacy code

---

## 2.10 Error Handling Strategy

### Parse Failures

Debug mode:
```rust
panic!("Failed to parse parameter 'id' as u32 from value 'abc'");
```

Release mode:
```rust
// Return default value, router shows 404
eprintln!("Failed to parse 'id', showing 404");
Default::default()
```

### Missing Parameters

Should never happen (routing logic prevents this):
```rust
panic!("Route '{}' requires params but none provided. This is a router bug.");
```

---

## 2.11 Performance Considerations

1. Pattern caching - Use `OnceCell` for lazy initialisation
2. Priority sorting - Sort once per navigation (cheap)
3. Parameter parsing - Only parse matched route
4. Zero overhead - Enum dispatch is optimised away

---

## 2.12 Timeline

Week 1:
* [ ] Pattern matching core
* [ ] Route module updates
* [ ] Basic tests
* [ ] Macro updates
* [ ] Component updates
* [ ] Validation updates
* [ ] Error types

Week 2:
* [ ] Example updates
* [ ] Integration tests
* [ ] Documentation
* [ ] Polish & bug fixes

Total: ~2 weeks for complete Phase 2 implementation