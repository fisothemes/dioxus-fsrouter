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
pub type StaticRouteRenderFn = fn() -> Element;
pub type DynamicRouteRenderFn = fn(HashMap<String, String>) -> ParseResult<Element>;

pub enum RenderFn {
    Static(StaticRouteRenderFn),      // Phase 1
    WithParams(DynamicRouteRenderFn), // Phase 2
}
```

Benefits:
* Backward compatible
* Type-safe
* Zero runtime overhead
* Clear intent

### 2.1.2 RouteInfo Fields

```rust
pub struct RouteInfo<'a> {
    path: &'a str,
    pattern: &'a OnceLock<RoutePattern>,  // Lazy-initialised
    component_name: &'a str,
    render_fn: RenderFn,              // Now an enum
}
```

### 2.1.3 Pattern Model

```rust
pub struct RoutePattern {
    raw: String,
    segments: Vec<Segment>,
    priority: usize,
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

To ensure deterministic routing when multiple patterns could match a URL (e.g., `/user/new` vs `/user/:id`), the router calculates a priority score. Higher scores take precedence, prioritising specificity, position, and depth.

#### Scoring Rules

The algorithm uses position-weighted scoring where segments earlier in the path have greater influence:

- **Static Segments**: Base score of 10,000 points
- **Dynamic Segments**: Base score of 1,000 points
- **Position Multiplier**: Segments are weighted by their position (the first segment has the highest weight)
- **Length Bonus**: +1 point per segment (as a tiebreaker)

#### Formula
```rust
for each segment at position i (0-indexed from left):
    position_weight = (total_segments - i)
    
    if Static:
        score += 10,000 × position_weight
    if Dynamic:
        score += 1,000 × position_weight

score += total_segments  // length bonus
```

#### Examples

* **Basic Case**: Static vs Dynamic (Same Position)
  ```text
  URL: /user/new
  
  Pattern 1: /user/new
    - Static(user):  10,000 × 2 = 20,000
    - Static(new):   10,000 × 1 = 10,000
    - Length bonus:  2
    - Total: 30,002  (Winner)
  
  Pattern 2: /user/:id
    - Static(user):  10,000 × 2 = 20,000
    - Dynamic(id):   1,000 × 1  = 1,000
    - Length bonus:  2
    - Total: 21,002
  ```
  **Result**: `/user/new` wins because the static segment at position 2 outweighs the dynamic segment.


* **Advanced Case**: Same Static/Dynamic Ratio, Different Positions
  ```text
  URL: /api/users/123
  
  Pattern 1: /api/:resource/:id
    - Static(api):      10,000 × 3 = 30,000
    - Dynamic(resource): 1,000 × 2 = 2,000
    - Dynamic(id):       1,000 × 1 = 1,000
    - Length bonus:      3
    - Total: 33,003  (Winner)
  
  Pattern 2: /:version/users/:id
    - Dynamic(version): 1,000 × 3  = 3,000
    - Static(users):   10,000 × 2  = 20,000
    - Dynamic(id):      1,000 × 1  = 1,000
    - Length bonus:     3
    - Total: 24,003
  ```
  **Result**: `/api/:resource/:id` wins because the static segment appears first (position 3 weight), which is more valuable than a static segment at position 2.


* **Complex Case**: Multiple Routes with the Same Segment Count
  ```text
  URL: /user/new/posts
  
  Pattern 1: /user/new/posts  (all static)
    - Static(user):  10,000 × 3 = 30,000
    - Static(new):   10,000 × 2 = 20,000
    - Static(posts): 10,000 × 1 = 10,000
    - Length bonus:  3
    - Total: 60,003  (Winner)
  
  Pattern 2: /user/:id/posts  (mixed)
    - Static(user):  10,000 × 3 = 30,000
    - Dynamic(id):    1,000 × 2 = 2,000
    - Static(posts): 10,000 × 1 = 10,000
    - Length bonus:  3
    - Total: 42,003
  
  Pattern 3: /:type/new/posts  (mixed)
    - Dynamic(type):  1,000 × 3 = 3,000
    - Static(new):   10,000 × 2 = 20,000
    - Static(posts): 10,000 × 1 = 10,000
    - Length bonus:  3
    - Total: 33,003
  ```
  **Result**: Specificity order is preserved: all-static `>` early-static `>` late-static.

#### Why Position Matters

Position-based weighting ensures that static segments **early** in the path are valued more than those later. This reflects real-world routing semantics:

- `/api/v1/:resource` - API version is more fundamental than a resource type
- `/docs/guide/:section` - Documentation structure is more specific than a section
- `/admin/users/:id` - Admin area is more privileged than user selection

#### Edge Cases

* **Different Segment Counts**: Routes with different segment counts **cannot conflict** because matching requires exact segment count equality.

  ```text
  URL: /files/settings  (2 segments)
  
  ✅ Can match: /files/settings   (2 segments)
  ✅ Can match: /files/:name       (2 segments)  
  ❌ Cannot match: /files/:drive/:folder  (3 segments)
  ```

  The priority algorithm only compares routes **after** segment count filtering.


* **Ties (Extremely Rare)**: If two patterns have identical scores (same static/dynamic segments at same positions), the order is determined by registration order (first registered wins). In practice, this only occurs with functionally identical patterns:

  ```text
  /user/:id  vs.  /user/:user_id    (functionally identical)
  ```

  This is considered a validation error and should be caught at startup.

#### Implementation

```rust
pub fn calculate_priority(segments: &[Segment]) -> usize {
  let mut priority = 0usize;
  let len = segments.len();

  for (index, segment) in segments.iter().enumerate() {
    let position_weight = len - index;

    let base_score = match segment {
      Segment::Static(_) => 10_000,
      Segment::Param(_) => 1_000,
    };

    priority += base_score * position_weight;
  }

  // Length bonus for tiebreaking
  priority += len;

  priority
}
```

#### Summary

The position-weighted priority algorithm ensures:

1. **Static segments always beat dynamic** (at the same position)
2. **Earlier segments have more weight** (position matters)
3. **Deeper routes preferred** (length bonus as tiebreaker)
4. **Deterministic matching** (no ambiguity)
5. **Intuitive behaviour** (matches developer expectations)

This creates a natural hierarchy: specificity → position → depth.

---

## 2.3 Route Module (`packages/fsrouter/src/route/mod.rs`)

Checklist:
* [x] Add `RenderFn` enum
* [x] Update `RouteInfo` constructor:
  * [x] generic `new()` accepting `RenderFn` and `&OnceLock`
* [x] Add lazy pattern initialisation with `OnceLock` (std)
* [x] Update `find_route()` to use pattern matching
* [x] Sort routes by priority

Key function:

```rust
pub fn find_route(path: &str) -> Option<(&'static RouteInfo<'static>, HashMap<String, String>)> {
  // Routes are already sorted by priority in get_routes()
  for route in get_routes() {
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
* [x] Detect `:param` syntax in a path
* [x] Extract component parameter names and types
* [x] Validate route params match component props
* [x] Generate appropriate wrapper:
  * [x] Static wrapper for no params
  * [x] Dynamic wrapper with `FromStr` parsing
* [x] Handle parse errors:
  * [x] Debug: panic with a helpful message
  * [x] Release: return default value (triggers 404)
* [x] Add new validation errors

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

---

## 2.5 Router Components (`packages/fsrouter/src/router/components.rs`)

Checklist:
* [x] Update `Outlet` to use new `find_route()` signature
* [x] Pass parameters to the render function
* [x] Handle parse failures (404 or fallback)

Updated Outlet:
```rust
#[component]
pub fn Outlet() -> Element {
    let nav_ctx = use_context::<NavigationContext>();
    let path = nav_ctx.current_route.read();
    
    match find_route(&path) { 
        Some((route, params)) => {
            match route.render(Some(params)) {
                Ok(element) => element,
                Err(parse_error) => {
                    // Log error (debug) and show 404
                    rsx! { /* 404 */ }
                }
            }
          }
        }
        None => rsx! { /* 404 */ }
    }
}
```

---

## 2.6 Validation & Errors (`packages/fsrouter/src/route/validate.rs`, `packages/fsrouter/src/errors.rs`)

Checklist:
* [x] Implement `AmbiguityError` in `RouterError` enum
* [x] Update `validate_routes()` to detect ambiguous patterns:
  * Logic: Two routes are ambiguous if they share the same priority AND match overlapping paths.
  * Example: `/post/:id` vs `/post/:slug` (Conflict!)
  * Non-Example: `/post/new` vs `/post/:id` (No conflict, static wins priority)
* [x] Add Testing Macros (Developer Experience):
  * `assert_routes_valid!()` - Panics if registry has conflicts
  * `assert_route_matches!(path, component)` - Verifies routing logic

```rust
#[test]
fn test_my_routes() {
    // Fails if no route matches or if it matches the wrong component
    assert_route_matches!("/user/123", UserProfile);
    
    // Fails if parameters don't parse correctly
    assert_route_matches!("/user/123", UserProfile, { id: "123" });
}

#[test]
fn test_registry_sanity() {
  // Fails if any duplicates or ambiguities exist in the whole app
  assert_routes_valid!();
}
```

---

## 2.7 Example App (`examples/basic`)

Checklist:
* [x] Add routes with parameters:
  ```rust
  #[route("/user/:id")]
  fn UserProfile(id: String) -> Element { ... }

  #[route("/post/:id")]
  fn Post(id: u32) -> Element { ... }

  #[route("/user/:user_id/posts/:post_id")]
  fn UserPost(user_id: String, post_id: u32) -> Element { ... }
  ```
* [x] Add navigation to parameterised routes
* [x] Show parameter values in UI
* [x] Test parse failures

---

## 2.8 Testing

### 2.8.1 Unit Tests

Checklist:
* [x] Pattern parsing
* [x] Pattern matching
* [x] Priority calculation
* [x] Parameter extraction
* [x] Type conversion (`FromStr`)
* [x] Parse error handling

### 2.8.2 Integration Tests

Checklist:
* [x] Register mixed routes (static + dynamic)
* [x] Match URLs against patterns
* [x] Extract and parse parameters
* [x] Verify priority ordering
* [x] Test parse failures -> 404

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

## 2.9 Error Handling Strategy

### Parse Failures

The wrapper function returns `Result<Element, ParseError>`.

Debug mode:
```rust
// User may still choose to panic in debug to catch broken links immediately
#[cfg(debug_assertions)]
panic!("Failed to parse parameter 'id' as u32");

// Or simply return the error to test the 404 page (Fallback/404 component)
#[cfg(not(debug_assertions))]
Err(ParseError::InvalidType { param: "id", ... })
```

Release mode:
```rust
// Return the error, causing the Outlet to render the Fallback/404 component
Err(ParseError::InvalidType { param: "id", ... })
```

### Missing Parameters

Should never happen (routing logic prevents this):
```rust
panic!("Route '{}' requires params but none provided. This is a router bug.");
```

---

## 2.10 Performance Considerations

1. Pattern caching - Use `OnceCell` for lazy initialisation
2. Priority sorting - Sort once per navigation (cheap)
3. Parameter parsing - Only parse the matched route
4. Zero overhead - Enum dispatch is optimised away

---

## 2.11 Timeline

Week 1:
* [x] Pattern matching core
* [x] Route module updates
* [x] Basic tests
* [x] Macro updates
* [x] Component updates
* [x] Validation updates
* [x] Error types

Week 2:
* [x] Example updates
* [x] Integration tests
* [x] Documentation
* [x] Polish & bug fixes

Total: ~2 weeks for complete Phase 2 implementation

---

# Phase 3: Extended Matching & Safety

## Goal

Complete the URL matching logic by adding support complex patterns (Catch-alls, Query Params) and ensure the router is secure via default (Link validation) and robust (Custom Fallbacks).

- `Link` component should strictly enforce internal paths to prevent open redirects.
- `Outlet` supports per-instance 404 fallbacks via props.
- Support for `/:..segments` and `?:query` syntax.
- Support for `#[alias]` to reduce duplication.

---

### 3.1 Link Security

Mitigate the **Open Redirect** vulnerability by validating link targets.

Checklist:
* [x] Update `Link` props to block external links.
* [x] Path must start with `/`.
  * [x] Path must NOT start with `//`.
* [x] Update `Link` render logic:
  * [x] If path is valid: Render standard `<a>` with click handler.
  * [x] If path is external/invalid: Render "dead" anchor (no `href`) to prevent navigation.

---

### 3.2 Fallback

Allow developers to customise the "Not Found" UI per-outlet.

Checklist:
* [ ] Update `OutletProps` to accept `fallback: Option<fn() -> Element>`.
* [ ] Update `Outlet` render logic:
  * [ ] If `find_route` returns `None` → Render fallback.
  * [ ] If parameter parsing fails (Result::Err) → Render fallback.
* [ ] Implement default `NotFound` component for when no fallback is provided.


```rust
// Usage
Outlet { fallback: MyCustom404 }
```

---

### 3.3 Catch-All

Support "rest of path" matching.

**Syntax:** `#[route("/files/:..path")]`

Checklist:

---

### 3.4 Query Parameters

Support for type-safe query parameters in routes.

**Syntax:** `#[route("/search?:query&:page")]` becomes `/search?q=hello&page=2`.

Checklist:

---

### 3.5 Alias

Allow multiple paths to map to a single component.

**Syntax:** `#[alias("/user/:id")]`

Checklist:

---

### 3.6 Testing

Checklists:

---

### 3.7 Example

Incorporate all features into a simple example app.

Checklists:

---

# Phase 4: Organisation (Route Groups)

## Goal

A clean way to structure routes for large applications.

### 4.1 Module-Level Macro

---

### 4.2 Example

---

### 4.3 Testing

---

# Phase 5: Type-Safe Navigation

## Goal

Introduce `LinkTo` component to safely navigate between routes.

---

### 5.1 Routable Trait?

---

### 5.2 LinkTo Component

---

### 5.3 Example

---

### 5.4 Testing

# Phase 6: Nested Layouts 

## Goal

Change architecture to support hierarchical rendering.

---

# Phase 7: Documentation, Optimisation & Clean-up

## Goal

Improve documentation, performance & code quality (continuous).