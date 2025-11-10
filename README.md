# Dioxus FsRouter

[![Crates.io](https://img.shields.io/crates/v/dioxus-fsrouter.svg)](https://crates.io/crates/dioxus-fsrouter)
[![Documentation](https://docs.rs/dioxus-fsrouter/badge.svg)](https://docs.rs/dioxus-fsrouter)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A simple, filesystem-style router for [Dioxus](https://dioxuslabs.com/) applications that provides component-based routing with compile-time safety, automatic route registration, and zero boilerplate.

## Core Principles

1. **Component-First**: Routes are attached directly to components via attributes
2. **Type-Safe**: All navigation is type-checked at compile time
3. **Zero Boilerplate**: No enums, no manual registration, no string-based routing
4. **Compile-Time Validation**: Route conflicts and invalid URLs caught at compile time
5. **Auto-Discovery**: Routes automatically register themselves via global inventory

## Features

### 1. Route Definition

```rust
#[route("/")]
#[component]
fn Home() -> Element {
    rsx! { div { "Home" } }
}

#[route("/user/:id")]
#[component]
fn UserProfile(id: String) -> Element {
    rsx! { div { "User: {id}" } }
}

#[route("/post/:slug")]
#[alias("/article/:slug")]
#[alias("/blog/:slug")]
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
- Primary route via `#[route("/path")]`
- Multiple aliases via `#[alias("/path")]`
- Grouped routes via `#[route_group("/prefix")]` on modules
- Route parameters (`:param`) automatically parsed and passed as component props
- Component props must match route parameters (compile-time checked)
- Access route metadata via `use_route_context()`

### 2. Router Setup

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

**Capabilities:**
- `router!` macro sets up routing context
- `Outlet` component renders matched route
- Non-route components (Navbar, Footer) render normally
- Routes auto-discovered from global inventory

### 3. Navigation

```rust
fn Navbar() -> Element {
    rsx! {
        nav {
            // Type-safe links to components
            LinkTo::<Home> { "Home" }
            LinkTo::<UserProfile> { id: "alice", "Alice's Profile" }
            LinkTo::<Post> { slug: "hello-world", "Read Post" }
            LinkTo::<admin::Users> { "Admin users" }
        }
    }
}

fn SomeComponent() -> Element {
    let nav = use_navigation();
    
    rsx! {
        button {
            onclick: move |_| nav.push::<UserProfile>(UserProfileProps { 
                id: "bob".to_string() 
            }),
            "Go to Bob's Profile"
        }
        button {
            onclick: move |_| nav.go_back(),
            "Back"
        }
    }
}
```

**Capabilities:**
- `LinkTo::<Component>` for declarative navigation
- `use_navigation()` hook for programmatic navigation
- Type-safe: Can't navigate to non-existent routes
- Props validated at compile time

### 4. Route Parameters

```rust
#[route("/user/:id/posts/:post_id")]
#[component]
fn UserPost(id: String, post_id: u32) -> Element {
    let ctx = use_route_context();
    
    rsx! { 
        div { 
            "User {id}, Post {post_id}"
            // Access params from context too
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
- Parameters defined with `:name` syntax
- Automatically parsed from URL to component props
- Type conversion (String, u32, i32, etc.)
- Parse failures result in 404 or fallback route
- Access raw params via `use_route_context().params`

### 5. Route Context

```rust
#[route("/dashboard")]
#[component]
fn Dashboard() -> Element {
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
- `url: String` - The actual URL path (e.g., "/article/hello-world")
- `pattern: &'static str` - The pattern that matched (e.g., "/article/:slug")
- `is_alias: bool` - Whether matched via an alias
- `component_name: &'static str` - Name of the matched component
- `params: HashMap<String, String>` - Extracted route parameters

**Use Cases:**
- Canonical URL redirects
- Analytics and logging
- Breadcrumb generation
- Conditional rendering based on route type

### 6. Fallback Routes (404 Handling)

```rust
#[fallback]
#[component]
fn NotFound() -> Element {
    let ctx = use_route_context();
    let nav = use_navigation();
    
    // Log 404s
    use_effect(move || {
        log_404(&ctx.url);
    });
    
    rsx! { 
        div { class: "not-found",
            h1 { "404 - Page Not Found" }
            p { "The page '{ctx.url}' does not exist" }
            
            // Smart suggestions based on URL
            if ctx.url.starts_with("/user/") {
                p { "Looking for a user profile?" }
                LinkTo::<UserList> { "Browse Users" }
            }
            
            button {
                onclick: move |_| nav.go_back(),
                "Go Back"
            }
            LinkTo::<Home> { "Go Home" }
        }
    }
}
```

**Capabilities:**
- `#[fallback]` marks a component as the 404 handler
- Only one fallback allowed per application (compile-time enforced)
- Has lowest matching priority
- Access attempted URL via `use_route_context()`
- Cannot be combined with `#[route]` or `#[alias]`

### 7. Compile-Time Validation

#### URL Validation
```rust
#[route("/valid/path")]        // ✅ Valid
#[route("/user/:id")]          // ✅ Valid
#[route("/")]                  // ✅ Valid

#[route("no-slash")]           // ❌ Error: Must start with '/'
#[route("/double//slash")]     // ❌ Error: No double slashes
#[route("/user/:")]            // ❌ Error: Empty parameter name
#[route("/user/:id/:id")]      // ❌ Error: Duplicate parameter ':id'
#[route("/trailing/")]         // ❌ Warning: Trailing slash (optional)
```

#### Duplicate Route Detection
```rust
#[route("/about")]
fn About() -> Element { ... }

#[route("/about")]             // ❌ Error: Route "/about" already registered by `About`
fn AboutUs() -> Element { ... }

#[route("/contact")]
#[alias("/about")]             // ❌ Error: Alias "/about" conflicts with route `About`
fn Contact() -> Element { ... }

#[route("/home")]
#[alias("/")]
fn HomePage() -> Element { ... }

#[route("/")]                  // ❌ Error: Route "/" conflicts with alias on `HomePage`
fn Index() -> Element { ... }
```

#### Fallback Validation
```rust
#[fallback]
fn NotFound() -> Element { ... }

#[fallback]                    // ❌ Error: Fallback already defined by `NotFound`
fn Error404() -> Element { ... }

#[fallback]
#[route("/404")]               // ❌ Error: Fallback cannot have explicit route path
fn NotFound() -> Element { ... }

#[fallback]
#[alias("/404")]               // ❌ Error: Fallback cannot have aliases
fn NotFound() -> Element { ... }
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

Routes are matched in order of specificity:

1. **Exact matches**: `/about` (priority: 1000)
2. **Parameterized routes**: `/user/:id` (priority: 500)
3. **Fallback**: `#[fallback]` (priority: -1000)

Within each tier, longer/more specific paths have higher priority:
- `/user/:id/posts/:post_id` (priority: 502) > `/user/:id` (priority: 501)
- `/user/new` (priority: 1000) > `/user/:id` (priority: 500)

```rust
#[route("/user/new")]          // Priority: 1000 (exact)
fn NewUser() -> Element { ... }

#[route("/user/:id")]          // Priority: 500 (parameterized)
fn UserProfile(id: String) -> Element { ... }

#[fallback]                    // Priority: -1000 (fallback)
fn NotFound() -> Element { ... }

// /user/new → matches NewUser (exact match)
// /user/123 → matches UserProfile (parameterized)
// /user/123/invalid → matches NotFound (fallback)
```