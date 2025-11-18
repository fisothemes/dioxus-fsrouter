//! Router component and utilities

use crate::route::{find_route, validate_routes};
use dioxus::prelude::*;

/// Router component that sets up routing context and handles navigation
///
/// # Example
/// ```ignore
/// fn App() -> Element {
///     rsx! {
///         Router {
///             nav { /* navigation */ }
///             Outlet {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Router(children: Element) -> Element {
    // Get the initial path
    let initial_path = get_current_path();
    let current_route = use_signal(|| initial_path);

    // Validate routes on the first render
    use_hook(|| {
        if let Err(e) = validate_routes() {
            eprintln!("Route validation errors:\n{}", e);
            panic!("Route conflicts detected");
        }
    });

    // Set up a popstate listener for browser back/forward buttons
    #[cfg(target_family = "wasm")]
    {
        use_effect(move || {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;

            let window = web_sys::window().expect("no global window");

            // Clone the signal for the closure
            let mut route_signal = current_route.clone();

            let closure = Closure::wrap(Box::new(move |_event: web_sys::PopStateEvent| {
                let new_path = get_current_path();
                route_signal.set(new_path);
            }) as Box<dyn FnMut(_)>);

            window
                .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
                .expect("failed to add popstate listener");

            // Keep closure alive
            closure.forget();
        });
    }

    // Provide navigation context
    use_context_provider(|| NavigationContext {
        current_route: current_route.clone(),
    });

    rsx! {
        {children}
    }
}

/// Get the current path from the browser
fn get_current_path() -> String {
    #[cfg(target_family = "wasm")]
    {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_else(|| "/".to_string())
    }

    #[cfg(not(target_family = "wasm"))]
    {
        "/".to_string()
    }
}

/// Navigation context shared across the app
#[derive(Clone, Copy)]
struct NavigationContext {
    current_route: Signal<String>,
}

/// Outlet component that renders the matched route
///
/// Must be used inside a Router component.
#[component]
pub fn Outlet() -> Element {
    let nav_ctx = use_context::<NavigationContext>();
    let path = nav_ctx.current_route.read();

    match find_route(&path) {
        Some(route) => route.render(),
        None => rsx! {
            div {
                style: "padding: 2rem; color: #dc2626;",
                h1 { "404 - Not Found" }
                p { "No route found for: {path}" }
            }
        },
    }
}

/// Navigate to a new path
///
/// This updates both the URL and the router state.
pub fn navigate(path: String) {
    #[cfg(target_family = "wasm")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(history) = window.history().ok() {
                let _ = history.push_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&path)
                );

                // Dispatch a custom event to notify the router
                // This is necessary because pushState doesn't trigger popstate
                if let Ok(event) = web_sys::PopStateEvent::new("popstate") {
                    let _ = window.dispatch_event(&event);
                }
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    {
        println!("Navigate to: {}", path);
    }
}

/// Navigation hook for programmatic navigation
///
/// # Example
/// ```ignore
/// let nav = use_navigation();
/// nav.push("/about");
/// nav.go_back();
/// ```
pub fn use_navigation() -> Navigation {
    let nav_ctx = use_context::<NavigationContext>();
    Navigation {
        current_route: nav_ctx.current_route,
    }
}

/// Navigation handle for programmatic navigation
#[derive(Clone, Copy)]
pub struct Navigation {
    current_route: Signal<String>,
}

impl Navigation {
    /// Navigate to a new path
    pub fn push(&mut self, path: impl Into<String>) {
        let path = path.into();

        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.push_state_with_url(
                        &wasm_bindgen::JsValue::NULL,
                        "",
                        Some(&path)
                    );
                }
            }
        }

        // Update the signal directly
        self.current_route.set(path);
    }

    /// Replace the current path without adding to history
    pub fn replace(&mut self, path: impl Into<String>) {
        let path = path.into();

        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.replace_state_with_url(
                        &wasm_bindgen::JsValue::NULL,
                        "",
                        Some(&path)
                    );
                }
            }
        }

        self.current_route.set(path);
    }

    /// Go back in history
    pub fn go_back(&self) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.back();
                }
            }
        }
    }

    /// Go forward in history
    pub fn go_forward(&self) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.forward();
                }
            }
        }
    }

    /// Get the current path
    pub fn current_path(&self) -> String {
        (*self.current_route.read()).clone()
    }
}

/// Simple link component for navigation
///
/// # Example
/// ```ignore
/// Link { to: "/about", "About" }
/// ```
#[component]
pub fn Link(to: String, children: Element) -> Element {
    let mut nav = use_navigation();

    rsx! {
        a {
            href: "{to}",
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            {children}
        }
    }
}