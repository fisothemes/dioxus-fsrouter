# Phase 1: Core Foundation Implementation

## Goal

Basic routing with **exact path matching** (no parameters, minimal validation) using a **modular runtime crate** and a **single `#[route]` proc macro**.

- Routes are attached directly to components with `#[route("/path")]`.
- All routes register themselves at compile time via `inventory`.
- The router tracks the current URL and renders the matching component into an `Outlet`.
- Navigation is done via a simple `Link` component and a `use_navigation` hook.

---

## 1. Project Structure Setup

Workspace layout:

```text
dioxus-fsrouter/
  Cargo.toml                  # Workspace definition

  packages/
    fsrouter/                 # Runtime crate
      Cargo.toml
      src/
        lib.rs                # Public API / prelude
        route/
          mod.rs              # Route module hub
          registry.rs         # RouteInfo, inventory, get_routes()
          matching.rs         # find_route(path)
          validate.rs         # validate_routes()
        router/
          mod.rs              # Router module hub
          components.rs       # Router, Outlet, NavigationContext, get_current_path()
          navigation.rs       # Navigation type, use_navigation(), navigate()
          link.rs             # Link component

    fsrouter-macro/           # Proc macro crate
      Cargo.toml
      src/
        lib.rs                # #[route] attribute macro

  examples/
    basic/
      Cargo.toml
      src/
        main.rs               # Minimal example app using the router
````

### Checklist

* [x] Create workspace `Cargo.toml` at repo root
* [x] Create `packages/fsrouter` runtime crate
* [x] Create `packages/fsrouter-macro` proc macro crate
* [x] Wire crates together in workspace `Cargo.toml` (workspace dependencies, resolver, version, etc.)
* [x] Add top-level README
* [x] Add `examples/basic` example crate

---

## 2. Core Types and Functions (`packages/fsrouter/src/`)

All runtime routing logic lives in the `fsrouter` crate and is split into clear submodules.

### 2.1 Route Module (`route/`)

**Goal:** Owns *route metadata* and *registry operations* — no UI, no Dioxus components.

Files:

* `route/mod.rs`
* `route/registry.rs`
* `route/matching.rs`
* `route/validate.rs`

#### `route/mod.rs`

Acts as a façade for the route subsystem:

* Re-exports:

  * `RouteInfo`
  * `get_routes()`
  * `find_route(path: &str)`
  * `validate_routes()`

Checklist:

* [ ] Create `route/mod.rs` and re-export public route APIs

#### `route/registry.rs`

Defines and registers routes.

Responsibilities:

* Define `RouteInfo`:

  ```rust
  pub struct RouteInfo {
      pub path: &'static str,
      pub component_name: &'static str,
      pub render: fn() -> dioxus::prelude::Element,
  }
  ```

* Declare an `inventory` collection of `RouteInfo` (or wrappers around it).

* Provide:

  ```rust
  pub fn get_routes() -> &'static [RouteInfo];
  ```

Checklist:

* [ ] Define `RouteInfo`
* [ ] Integrate `inventory` (or similar) for global registration
* [ ] Implement `get_routes()` returning a static slice of all routes

#### `route/matching.rs`

Contains matching logic for **exact path** routes.

Responsibilities:

* Implement:

  ```rust
  pub fn find_route(path: &str) -> Option<&'static RouteInfo>;
  ```

* Use `get_routes()` and simple equality (`route.path == path`).

Checklist:

* [ ] Implement `find_route(path)` using exact string comparison

#### `route/validate.rs`

Performs basic route-level validation.

Phase-1 scope:

* Detect duplicate route paths.
* Produce a human-friendly error message listing conflicts.

API:

```rust
pub fn validate_routes() -> Result<(), String>;
```

Checklist:

* [ ] Implement duplicate-path detection
* [ ] Construct clear error messages including paths and component names
* [ ] Return `Ok(())` if everything is fine

---

### 2.2 Router Module (`router/`)

**Goal:** Owns the runtime **routing behaviour** and **Dioxus components**.

Files:

* `router/mod.rs`
* `router/components.rs`
* `router/navigation.rs`
* `router/link.rs`

#### `router/mod.rs`

Facade for the router subsystem.

* Re-exports:

  * Components:

    * `Router`
    * `Outlet`
  * Navigation:

    * `Navigation`
    * `use_navigation`
    * `navigate`
  * UI helper:

    * `Link`

Checklist:

* [ ] Create `router/mod.rs` and re-export all router-related items

#### `router/components.rs`

Contains the actual Dioxus router components and context.

Responsibilities:

* `Router` component:

  * Holds `Signal<String>` for the current path.
  * Reads an initial path from `get_current_path()`.
  * Runs `validate_routes()` once on first render and panics on error.
  * Sets up a `popstate` listener (on WASM) to respond to browser back/forward.
  * Provides a `NavigationContext` via `use_context_provider`.

* `Outlet` component:

  * Reads the current path from `NavigationContext`.
  * Calls `find_route(&path)` to locate a `RouteInfo`.
  * Renders `route.render()` when found.
  * Renders a simple “404 – Not Found” fallback when not found.

* `NavigationContext` (internal):

  ```rust
  #[derive(Clone, Copy)]
  pub(crate) struct NavigationContext {
      pub(crate) current_route: Signal<String>,
  }
  ```

* `get_current_path()`:

  * On WASM: returns `window.location.pathname` (or `"/"` on failure).
  * On non-WASM: returns `"/"`.

Checklist:

* [ ] Implement `get_current_path()` with WASM / non-WASM branches
* [ ] Implement `NavigationContext` with `Signal<String>`
* [ ] Implement `Router` component:

  * [ ] Set up route validation on the first render
  * [ ] Initialize route signal from `get_current_path()`
  * [ ] Attach `popstate` listener on WASM to update route
  * [ ] Expose `NavigationContext` using `use_context_provider`
* [ ] Implement `Outlet` component with basic 404 fallback

#### `router/navigation.rs`

Provides a programmatic navigation API and hook.

Responsibilities:

* Define `Navigation` struct that wraps access to the `NavigationContext` and `current_route` signal.

* Implement:

  ```rust
  pub fn use_navigation() -> Navigation;
  ```

* Implement a convenience function:

  ```rust
  pub fn navigate(path: String);
  ```

* Implement methods on `Navigation`:

  ```rust
  impl Navigation {
      pub fn push(&mut self, path: impl Into<String>);
      pub fn replace(&mut self, path: impl Into<String>);
      pub fn go_back(&self);
      pub fn go_forward(&self);
      pub fn current_path(&self) -> String;
  }
  ```

  * On WASM: use `window.history().push_state_with_url` / `replace_state_with_url` and browser `back`/`forward`.
  * On non-WASM: log to stdout and/or no-op.

Checklist:

* [ ] Implement `Navigation` type
* [ ] Implement `use_navigation()` hook using `use_context::<NavigationContext>()`
* [ ] Implement `navigate(path)` convenience wrapper (calls `push`)
* [ ] Implement `push`, `replace`, `go_back`, `go_forward`, `current_path` with WASM / non-WASM branches

#### `router/link.rs`

Provides a minimal `<a>`-like component for navigation.

Responsibilities:

* Implement:

  ```rust
  #[component]
  pub fn Link(to: String, children: Element) -> Element;
  ```

Behaviour:

* Renders `<a href="{to}">...</a>`.
* On `onclick`:

  * Calls `e.prevent_default()`.
  * Uses `use_navigation().push(to.clone())` to navigate without a page reload.

Checklist:

* [ ] Implement `Link` component using `use_navigation()`
* [ ] Prevent default click behaviour
* [ ] Ensure `href` still reflects the target path for accessibility / middle-click

---

### 2.3 `lib.rs` (Public API)

**Goal:** Provide a clean public surface and a `prelude` for users.

Responsibilities:

* Declare:

  ```rust
  pub mod route;
  pub mod router;
  ```

* Re-export core types / functions:

  ```rust
  pub use route::{RouteInfo, get_routes, find_route, validate_routes};
  pub use router::{Router, Outlet, Link, Navigation, use_navigation, navigate};
  ```

* Re-export `inventory` (needed by macro crate via `dioxus_fsrouter::inventory`).

* Re-export the macro crate as `macros`, and possibly `route` from there.

* Provide a `prelude` module that includes:

  * `route` macro
  * `Router`, `Outlet`, `Link`
  * `Navigation`, `use_navigation`
  * `get_routes`, `validate_routes`

Checklist:

* [ ] Wire up module declarations for `route` and `router`
* [ ] Re-export `RouteInfo`, `get_routes`, `find_route`, `validate_routes`
* [ ] Re-export `Router`, `Outlet`, `Link`, `Navigation`, `use_navigation`, `navigate`
* [ ] Re-export `inventory` for proc macro use
* [ ] Define `prelude` module for ergonomic imports

---

## 3. Basic Macro (`packages/fsrouter-macro/src/`)

**Goal:** Provide a single attribute macro `#[route("/path")]` that:

* Validates the provided path string a little.
* Registers the route in the global inventory at compile time.
* Generates a wrapper render function with the correct signature.

### 3.1 `lib.rs`

Responsibilities:

* Export the proc macro:

  ```rust
  #[proc_macro_attribute]
  pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream;
  ```

* Implementation strategy:

  * Parse `attr` as `LitStr` (e.g. `"/"`, `"/about"`).
  * Parse `item` as `syn::ItemFn`.
  * Validate:

    * Path must start with `/`.
    * Function must have **no input parameters** (Phase 1: no route params).
  * Generate:

    1. The original function is unchanged.

    2. A wrapper:

       ```rust
       #[allow(non_snake_case)]
       fn __render_<FuncName>() -> ::dioxus::prelude::Element {
           <FuncName>()
       }
       ```

    3. An `inventory::submit!` block:

       ```rust
       ::dioxus_fsrouter::inventory::submit! {
           ::dioxus_fsrouter::RouteInfo::new(
               "<path>",
               concat!(module_path!(), "::", stringify!(<FuncName>)),
               __render_<FuncName>,
           )
       }
       ```

Checklist:

* [ ] Implement `#[route]` attribute macro

  * [ ] Parse path literal
  * [ ] Parse target function
  * [ ] Validate path (`starts_with('/')`)
  * [ ] Validate zero parameters (no route params in Phase 1)
  * [ ] Generate wrapper render function
  * [ ] Generate `inventory::submit!` call

*(Deeper macro splitting into `route/mod.rs`, `parse.rs`, `codegen.rs` can be deferred to a later phase.)*

---

## 4. Router Macro (Future Phase)

**Not part of Phase 1.**

A future DX improvement is a `router!` macro so users can write:

```rust
fn App() -> Element {
    router! {
        Navbar {}
        main {
            Outlet {}
        }
        Footer {}
    }
}
```

Which would expand to:

```rust
fn App() -> Element {
    rsx! {
        Router {
            Navbar {}
            main { Outlet {} }
            Footer {}
        }
    }
}
```

Phase 1 is intentionally built **without** this macro; the core router and route system should stand on their own.

Checklist (reserved for later):

* [ ] `router/mod.rs` in macro crate for `router!` macro
* [ ] Codegen that wraps RSX children in `Router { ... }`

---

## 5. Testing

**Goal:** Prove Phase 1 works end-to-end with exact routes only.

### 5.1 Basic Integration Test (in `fsrouter`)

* [ ] Build a small module with two routes:

  ```rust
  #[route("/")]
  #[component]
  fn Home() -> Element { /* ... */ }

  #[route("/about")]
  #[component]
  fn About() -> Element { /* ... */ }
  ```

* [ ] Assert:

  * `get_routes().len() >= 2`
  * `find_route("/")` is `Some`
  * `find_route("/about")` is `Some`
  * `validate_routes()` returns `Ok(())` when there are no conflicts

### 5.2 Example: `examples/basic`

* [ ] Add `examples/basic/Cargo.toml` and `src/main.rs`
* [ ] Demonstrate:

  * Two simple routes with `#[route]`
  * Using `Router` + `Outlet` as the main layout
  * Using `Link` for navigation

Example skeleton:

```rust
use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

#[route("/")]
#[component]
fn Home() -> Element {
    rsx! { div { "Home" } }
}

#[route("/about")]
#[component]
fn About() -> Element {
    rsx! { div { "About" } }
}

fn App() -> Element {
    rsx! {
        Router {
            nav {
                Link { to: "/".into(), "Home" }
                Link { to: "/about".into(), "About" }
            }
            main {
                Outlet {}
            }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
```

---

Once all checkboxes above are satisfied, **Phase 1 is complete**:

* Routes are declared on components.
* Routes auto-register at compile time.
* Router can match and render them.
* Navigation works for exact paths.
* The module layout is clean and ready for Phase 2 (parameters, route context, etc.).
