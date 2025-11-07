# Dioxus FsRouter

[![Crates.io](https://img.shields.io/crates/v/dioxus-fsrouter.svg)](https://crates.io/crates/dioxus-fsrouter)
[![Documentation](https://docs.rs/dioxus-fsrouter/badge.svg)](https://docs.rs/dioxus-fsrouter)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A simple, filesystem-style router for [Dioxus](https://dioxuslabs.com/) applications. Inspired by file-based routing systems, `dioxus-fsrouter` provides an intuitive and declarative way to handle client-side navigation.

## Features

- **Simple & Declarative** - Define routes with a clean macro syntax
- **Dynamic Parameters** - Extract route parameters like `/blog/:id`
- **Client-Side Navigation** - `Link` component for seamless navigation
- **Hooks API** - `use_navigator()` and `use_params()` for programmatic control
- **Type-Safe** - Leverage Rust's type system for reliable routing
- **Lightweight** - Minimal dependencies, just Dioxus core

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus = "0.7"
dioxus-fsrouter = "0.1"
```

## Quick Start

```rust
use dioxus::prelude::*;
use dioxus_fsrouter::*;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    rsx! {
        Router {
            resolver: route_resolver,
            Navbar {}
        }
    }
}

fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/about" => rsx! { About {} },
        "/contact" => rsx! { Contact {} },
    })
}

#[component]
fn Navbar() -> Element {
    rsx! {
        nav {
            Link { to: "/".to_string(), "Home" }
            Link { to: "/about".to_string(), "About" }
            Link { to: "/contact".to_string(), "Contact" }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "Welcome Home!" }
            p { "This is the home page" }
        }
    }
}
```

## Core Concepts

### Router Component

The `Router` component is the foundation of your routing setup. It takes a `resolver` function that maps paths to components:

```rust
rsx! {
    Router {
        resolver: route_resolver,
        // Your navigation/layout components
        Navbar {}
    }
}
```

### Route Matching

Use the `match_route!` macro to define your routes:

```rust
fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/blog" => rsx! { BlogList {} },
        "/blog/:id" => rsx! { BlogPost {} },
        "/user/:username" => rsx! { UserProfile {} },
    })
}
```

Routes are matched in order. The first matching route wins.

### Dynamic Parameters

Extract dynamic segments from URLs using the `:param` syntax:

```rust
// Route: "/blog/:id"
// URL: "/blog/hello-world"

#[component]
fn BlogPost() -> Element {
    let params = use_params();
    let post_id = params.get("id").cloned().unwrap_or_default();
    
    rsx! {
        div {
            h1 { "Blog Post" }
            p { "Post ID: {post_id}" }
        }
    }
}
```

### Navigation

#### Using the Link Component

The `Link` component provides declarative navigation with automatic route handling:

```rust
#[component]
fn Navbar() -> Element {
    rsx! {
        nav {
            Link { to: "/".to_string(), "Home" }
            Link { to: "/about".to_string(), "About" }
            Link { 
                to: "/blog/my-post".to_string(), 
                "Read My Post" 
            }
        }
    }
}
```

#### Programmatic Navigation

Use the `use_navigator()` hook for programmatic navigation:

```rust
#[component]
fn LoginForm() -> Element {
    let navigate = use_navigator();
    
    let handle_submit = move |_| {
        // Perform login logic...
        navigate("/dashboard");
    };
    
    rsx! {
        form {
            // Form fields...
            button {
                onclick: handle_submit,
                "Login"
            }
        }
    }
}
```

## Examples

### Basic Routing

```rust
fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/about" => rsx! { About {} },
        "/contact" => rsx! { Contact {} },
    })
}
```

### Route with Parameters

```rust
fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/blog" => rsx! { BlogList {} },
        "/blog/:id" => rsx! { BlogPost {} },
    })
}

#[component]
fn BlogPost() -> Element {
    let params = use_params();
    let post_id = params.get("id");
    
    rsx! {
        article {
            h1 { "Post: {post_id:?}" }
            // ... post content
        }
    }
}
```

### Multiple Parameters

```rust
fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/user/:username/post/:post_id" => rsx! { UserPost {} },
    })
}

#[component]
fn UserPost() -> Element {
    let params = use_params();
    let username = params.get("username");
    let post_id = params.get("post_id");
    
    rsx! {
        div {
            h1 { "@{username:?}'s Post" }
            p { "Post ID: {post_id:?}" }
        }
    }
}
```

### Nested Layouts

```rust
fn App() -> Element {
    rsx! {
        Router {
            resolver: route_resolver,
            // Shared layout components
            Header {}
            main {
                style: "padding: 2rem;",
                // Routes render here
            }
            Footer {}
        }
    }
}
```

### 404 Handling

The `match_route!` macro automatically provides a 404 page for unmatched routes:

```rust
// If no routes match, renders:
// <div>
//   <h1>404 - Not Found</h1>
//   <p>Route: /unknown/path</p>
// </div>
```

You can customize the 404 page by catching unmatched routes:

```rust
fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/about" => rsx! { About {} },
        // Add a catch-all at the end
        "/:rest" => rsx! { Custom404 {} },
    })
}
```

## API Reference

### Components

#### `Router`

The main router component.

**Props:**
- `resolver: fn(String) -> Element` - Function that maps paths to components
- `children: Element` - Child components (typically layout/navigation)

#### `Link`

Component for client-side navigation.

**Props:**
- `to: String` - Target path
- `children: Element` - Link content

### Hooks

#### `use_navigator() -> impl Fn(&str)`

Returns a function for programmatic navigation.

```rust
let navigate = use_navigator();
navigate("/home");
```

#### `use_params() -> HashMap<String, String>`

Returns route parameters as a HashMap.

```rust
let params = use_params();
let id = params.get("id");
```

### Macros

#### `match_route!`

Matches the current path against route patterns.

```rust
match_route!(&path => {
    "/" => rsx! { Home {} },
    "/user/:id" => rsx! { User {} },
})
```

## Running Examples

Clone the repository and run the examples:

```bash
# Basic routing example
dx serve --example basic 

# Route parameters example
dx serve --example params
```

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under:
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Acknowledgments

- Built for [Dioxus](https://dioxuslabs.com/) - A cross-platform GUI library for Rust
- Inspired by filesystem-based routing in modern web frameworks

## Contact

- Issues: [GitHub Issues](https://github.com/fisothemes/dioxus-fsrouter/issues)
- Discussions: [GitHub Discussions](https://github.com/fisothemes/dioxus-fsrouter/discussions)
