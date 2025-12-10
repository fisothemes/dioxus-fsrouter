# Dioxus FsRouter

[![Crates.io](https://img.shields.io/crates/v/dioxus-fsrouter.svg)](https://crates.io/crates/dioxus-fsrouter)
[![Documentation](https://docs.rs/dioxus-fsrouter/badge.svg)](https://docs.rs/dioxus-fsrouter)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A simple, attribute-based router for [Dioxus](https://dioxuslabs.com/) applications that provides component-based routing with compile-time safety, automatic route registration, and minimal boilerplate.

## Core Principles

1. **Component-First**: Routes are attached directly to components via attributes
2. **Type-Safe**: All navigation is type-checked at compile time
3. **Minimal Boilerplate**: No enums, no manual registration, no string-based routing
4. **Compile-Time Validation**: Invalid URLs caught at compile time
5. **Auto-Discovery**: Routes automatically register themselves via global inventory

## Features

### Route Definition

```rust
// ✅ Implemented: Exact Path
#[route("/")]
#[component]
fn Home() -> Element {
    rsx! { div { "Home" } }
}

// ✅ Implemented: Route Parameters
#[route("/user/:id")]
#[component]
fn UserProfile(id: String) -> Element {
    rsx! { div { "User: {id}" } }
}

// 🚧 Planned: Redirects
#[route("/post/:slug")]
#[redirect("/article/:slug")]
#[redirect("/blog/:slug")]
#[component]
fn Post(slug: String) -> Element {
    let ctx = use_route_context();
    
    // Redirect aliases to canonical URL
    if ctx.is_alias {
        let nav = use_navigation();
        use_effect(move || {
            nav.replace_url(&format!("/post/{}", slug));
        });
    }
    
    rsx! { 
        div { 
            "Post: {slug}"
            small { "via {ctx.pattern}" }
        }
    }
}

// 🚧 Planned: Groups
#[route_group("/admin")]
mod admin {
    #[route("/users")] // Becomes "/admin/users"
    #[component]
    fn Users() -> Element {
        rsx! { div { "Admin Users" } }
    }
    
    #[route("/settings")] // Becomes "/admin/settings"
    #[component]
    fn Settings() -> Elements {
        rsx! { div { "Admin Settings" } }
    }

    #[route("/logs/:date")] // Becomes "/admin/logs/:date"
    #[component]
    fn Logs(date: String) {
        rsx!{ div { "Logs for {date}" } }
    }
}
```

**Capabilities:**
* [x] Primary route via `#[route("/path")]`
* [x] Route parameters (`:param`) automatically parsed and passed as component props
* [x] Component props must match route parameters (compile-time checked)
* [ ] Multiple redirects via `#[redirect("/path")]`
* [ ] Access route metadata via `use_route_context()`
* [ ] Grouped routes via `#[route_group("/prefix")]` on modules

### Router Setup

```rust
fn App() -> Element {
    Router {
        Navbar {}
        main {
            Outlet {}
        }
        Footer {}
    }
}
```

**Capabilities:**
* [x] `Router` component that manages application routing logic
* [x] `Outlet` component renders matched route
* [x] Non-route components (Navbar, Footer) render normally
* [x] Routes auto-discovered from global inventory

### Navigation

```rust
fn Navbar() -> Element {
    rsx! {
        nav {
            // ✅ Implemented: String-based Link
            Link { to: "/".to_string(), "Home" }
            Link { to: "/about".to_string(), "About" }
            
            // 🚧 Planned: Type-safe LinkTo
            LinkTo::<UserProfile> { id: "alice", "Alice's Profile" }
            LinkTo::<Post> { slug: "hello-world", "Read Post" }
            LinkTo::<admin::Users> { "Admin users" }
        }
    }
}

fn SomeComponent() -> Element {
    // ✅ Implemented: Navigation Hook
    let mut nav = use_navigation();

    rsx! {
        button {
            onclick: move |_| nav.push("/about"),
            "Go to About"
        }
    }
}
```

**Capabilities:**
* [x] `Link` for string-based navigation
* [ ] `LinkTo::<Component>` for declarative navigation
* [x] `use_navigation()` hook for programmatic navigation
* [ ] Type-safe: Can't navigate to non-existent routes
* [ ] Props validated at compile time

### Route Parameters

```rust
#[route("/user/:id/posts/:post_id")]
#[component]
fn UserPost(id: String, post_id: u32) -> Element {
    let ctx = use_route_context();
    
    rsx! { 
        div { 
            "User {id}, Post {post_id}"
            // 🚧 Planned: Access params from context too
            p { "ID from context: {ctx.params.get(\"id\").unwrap()}" }
        }
    }
}

// Usage:
LinkTo::<UserPost> { id: "alice", post_id: 42, "View Post" }

// URL: /user/alice/posts/42
// Automatically parsed: id = "alice", post_id = 42
```

**Capabilities:**
* [x] Parameters defined with `:name` syntax
* [x] Automatically parsed from URL to component props
* [x] Type conversion (String, u32, i32, etc.)
* [x] Parse failures result in 404 or fallback route
* [ ] Access raw params via `use_route_context().params`

### Route Context

```rust
#[route("/dashboard")]
#[component]
fn Dashboard() -> Element {
    // 🚧 Planned: Access metadata like ctx.url, ctx.pattern, etc.
    let ctx = use_route_context();
    
    // Log analytics
    use_effect(move || {
        log_page_view(&ctx.url, ctx.component_name);
    });
    
    rsx! { 
        div { 
            "Dashboard"
            small { "Current URL: {ctx.url}" }
            small { "Matched pattern: {ctx.pattern}" }
        }
    }
}
```

**RouteContext Fields:**
* [ ] `url: String` - The actual URL path (e.g., "/article/hello-world")
* [ ] `pattern: &'static str` - The pattern that matched (e.g., "/article/:slug")
* [ ] `is_alias: bool` - Whether matched via an alias
* [ ] `component_name: &'static str` - Name of the matched component
* [ ] `params: HashMap<String, String>` - Extracted route parameters

**Use Cases:**
- Canonical URL redirects
- Analytics and logging
- Breadcrumb generation
- Conditional rendering based on route type

### Catch-All Routes

```rust
// 🚧 Planned: Props-based catch-all routes
// Receive as String (raw path "a/b/c")
#[route("/files/:..path")]
#[component]
fn FileViewer(path: String) -> Element {
  rsx! { "Viewing file at: {path}" }
}

// Receive as Vec<String> (segments ["a", "b", "c"])
#[route("/folders/:..segments")]
#[component]
fn FolderViewer(segments: Vec<String>) -> Element {
  rsx! { "Depth: {segments.len()}" }
}
```

**Capabilities:**
* [x] Catch-all routes (`/:..name`)
* [ ] Type conversion (String, Vec\<T>, etc.)
* [ ] Parse failures result in 404 or fallback route

### Query Parameters

```rust
// 🚧 Planned: Props-based query parameters
#[route("/search?q&page")]
#[component]
fn Search(q: String, page: Option<u32>) -> Element {
    rsx! {
        div {
            "Search: {q}"
            if let Some(page) = page {
                br { }
                "Page: {page}"
            }
        }
    }
}
// URL: /search?q=rust&page=2 → q="rust", page=Some(2)
// URL: /search?q=rust → q="rust", page=None
```

**Capabilities:**
* [ ] Query parameters are parsed from the URL
* [ ] Type conversion (String, u32, i32, etc.)
* [ ] Parse failures result in 404 or fallback route

### Fallback Routes (404 Handling)

```rust
// 1. Default Fallback
// Renders the built-in "404 - Not Found" page
Outlet {}

// 2. Custom Fallback (Inline)
// Renders your custom content when no route matches
Outlet {
    div {
        h1 { "Oops! Page not found" }
        p { "We couldn't find the page you were looking for." }
        Link { to: "/", "Go Home" }
    }
}

// 3. Custom Fallback (Component)
Outlet {
    NotFoundPage {}
}
```

**Capabilities:**
* [x] Per-Outlet fallback content (different 404s for different parts of the app).
* [x] Renders automatically on route mismatch or parameter parsing error.
* [x] Defaults to a built-in debug-friendly 404 page if no content is provided.

### Compile-Time Validation

#### URL Validation
```rust
#[route("/valid/path")]        // ✅ Valid | ✅ Implemented
#[route("/user/:id")]          // ✅ Valid | ✅ Implemented
#[route("/")]                  // ✅ Valid | ✅ Implemented

#[route("no-slash")]           // ❌ Error: Must start with '/'       | ✅ Implemented
#[route("/double//slash")]     // ❌ Error: No double slashes         | ✅ Implemented
#[route("/user/:")]            // ❌ Error: Empty parameter name      | ✅ Implemented
#[route("/user/:id/:id")]      // ❌ Error: Duplicate parameter ':id' | ✅ Implemented
```

#### Type Safety
```rust
#[route("/user/:id")]
fn UserProfile(id: String) -> Element { ... }

LinkTo::<UserProfile> { id: "alice" }           // ✅ Valid
LinkTo::<UserProfile> { id: 123 }               // ❌ Error: Expected String, got i32
LinkTo::<UserProfile> { user_id: "alice" }      // ❌ Error: Unknown prop 'user_id'
LinkTo::<UserProfile> { }                       // ❌ Error: Missing required prop 'id'

#[route("/post/:id")]
fn Post(id: u32) -> Element { ... }

// URL: /post/abc
// Result: 404 (parse failure) or fallback route
```

### 8. Route Priority

The router uses a **Position-Weighted Scoring** algorithm to determine which route to match. This ensures deterministic matching where:

1.  **Static segments beat parameters** (at the same position).
2.  **Earlier segments matter more** (prefix precedence).
3.  **Deeper routes are preferred** (length tie-breaker).

#### Scoring Rules
* **Static Segments**: 10,000 points
* **Parameter Segments**: 1,000 points
* **Multiplier**: Score × (Total Segments - Index)
* **Bonus**: +1 point per segment

#### Examples

**Case 1: Specificity (`/user/new` vs `/user/:id`)**

* **`/user/new`**
  * `user` (Static × 2): 20,000
  * `new` (Static × 1): 10,000
  * Length: +2
  * **Total: 30,002** (Winner) ✅

* **`/user/:id`**
  * `user` (Static × 2): 20,000
  * `:id` (Param × 1): 1,000
  * Length: +2
  * **Total: 21,002**

**Case 2: Root vs Deep**

* **`/about`**
  * `about` (Static × 1): 10,000
  * Length: +1
  * **Total: 10,001**

## Project Progress

This project is being implemented in multiple phases to ensure quality and maintainability.

### Tracking Progress

You can view the current implementation status, completed features, and upcoming work in
the [Progress.md](./Progress.md) file located in the root directory of this repository. This document is regularly
updated to reflect the project's development state.

### Licence

This project is licensed under the [MIT](./LICENSE) licence.