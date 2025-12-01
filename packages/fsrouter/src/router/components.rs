use crate::errors::{ParseError, ValidationErrors};
use crate::route::{find_route, validate_routes};
use crate::router::navigation::{NavigationContext, use_navigation};
use dioxus::logger::tracing;
use dioxus::prelude::*;

/// The `Router` component is responsible for managing the application's routing logic. It
/// sets up the initial route, listens to browser's navigation events, and provides a context
/// for components to access and manipulate the current route.
///
/// # Parameters
///
/// - `children`: The child elements to render within the `Router` component. This acts as a
///   placeholder for the application content that will respond to route changes.
///
/// # Behaviour
///
/// 1. **Initial Route Setup**:
///    - The `Router` retrieves the initial path from the browser's `window.location`.
///    - It creates a reactive signal (`current_route`) to store and manage the current route's state.
///
/// 2. **Route Validation**:
///    - During the first render, the `Router` executes a validation function (`validate_routes`) to ensure all defined routes are valid.
///    - If validation fails, it renders the `InternalServerErrors` component to prevent the application from proceeding with invalid routes.
///
/// 3. **Browser Back/Forward Navigation Handling (WASM targets only)**:
///    - A `popstate` event listener is registered to detect when users navigate via browser back/forward buttons.
///    - When a navigation event occurs, the `Router` updates the `current_route` signal with the new path reflected by the browser's address bar.
///
/// 4. **Context Provision**:
///    - The `Router` establishes a `NavigationContext` to share the `current_route` signal with child components, enabling nested components to react to or modify the navigation state.
///
/// 5. **Rendering**:
///    - The child elements (`children`) provided to the `Router` are rendered within the component, allowing them to access and use the routing context.
///
/// # Notes
///
/// - The use of the `use_effect` and event listener setup ensures that navigation changes are reactive.
/// - The `current_route` signal provides a unified way to track and propagate changes to the application's route.
///
/// # Example
///
/// ```ignore
/// #[component]
/// pub fn App() -> Element {
///     rsx! {
///         Router {
///             nav { /* navigation */ }
///             Outlet { } /* display navigation components here */
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
    #[allow(clippy::redundant_closure)]
    let router_error = use_hook(|| validate_routes());

    if let Err(errors) = router_error {
        return rsx! {
            InternalServerErrors{ errors }
        };
    }

    // Set up a popstate listener for browser back/forward buttons
    #[cfg(target_family = "wasm")]
    {
        use_effect(move || {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;

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
    use_context_provider(|| NavigationContext { current_route });

    rsx! {
        {children}
    }
}

/// A placeholder component that renders the content of the currently active route.
///
/// # Functionality
/// - Matches the current path against registered routes.
/// - If a match is found, renders the route component.
/// - If **NO** match is found (or params fail to parse), renders the **children** of the Outlet.
/// - If no children are provided, renders a default `NotFound` component with debug information
///   if in debug mode.
///
/// # Example
/// ```ignore
/// #[component]
/// pub fn App() -> Element {
///     rsx! {
///         Router {
///             NavBar { }
///             Outlet { }
///             Footer { }
///         }
///     }
/// }
/// ```
/// In the component tree, the `Outlet` will render the content of the currently active route
/// or display a fallback "404 - Not Found" page if no matching route is found.
///
/// Pass a component as a child to `Outlet` to display it on 404s:
/// ```ignore
/// #[component]
/// fn App() -> Element {
///     rsx! {
///         Router {
///             NavBar { }
///             Outlet {
///                 div { "Oops! Page Not Found" }
///             }
///             Footer { }
///         }
///     }
/// }
/// ```
#[component]
pub fn Outlet(children: Element) -> Element {
    let nav_ctx = use_context::<NavigationContext>();
    let path = nav_ctx.current_route.read();

    let render_fallback = |parse_error: Option<ParseError>| {
        if children != VNode::empty() {
            return children;
        }

        rsx! {
            NotFound{ path: path.clone() }

            if let Some(parse_error) = parse_error {
                p { strong { "Debug info: " } "{parse_error}" }
            }
        }
    };

    match find_route(&path) {
        Some((route, params)) => {
            match route.render(Some(params)) {
                Ok(element) => element,
                Err(parse_error) => {
                    // Parse error - show 404 or fallback
                    #[cfg(debug_assertions)]
                    {
                        tracing::error!("Parameter parse error: {}", parse_error);
                    }

                    #[cfg(not(debug_assertions))]
                    {
                        tracing::error!("No route found for: {path}");
                    }

                    render_fallback(if cfg!(debug_assertions) {
                        Some(parse_error)
                    } else {
                        None
                    })
                }
            }
        }
        None => render_fallback(None),
    }
}

/// A component that creates a hyperlink (`<a>` element) which allows programmatic navigation
/// while preventing the default browser navigation behaviour. This component is typically used for client-side
/// routing in web applications.
///
/// # Parameters
/// - `attributes`: Extended attributes to apply to the internal anchor element.
/// - `to`: A `String` specifying the target URL or route to navigate to when the hyperlink is clicked.
/// - `children`: The `Element` representing the content of the link (e.g. text or nested elements).
///
/// # Behaviour
/// The component renders an anchor (`<a>`) element with the `href` attribute pointing to the given `to` route.
/// When the link is clicked, it:
/// 1. Prevents the default browser behaviour for `<a>` tags (i.e. no page reload occurs).
/// 2. Uses a navigation hook `use_navigation()` to programmatically navigate to the specified `to` route.
///
/// This component allows smooth client-side navigation in applications with frameworks that support hooks and
/// declarative UI patterns.
///
/// # Example
/// ```ignore
/// #[component]
/// fn App() -> Element {
///     rsx! {
///         Link { to: "/about", "About Us" }
///     }
/// }
/// ```
///
/// In the above example, clicking the "About Us" link will programmatically navigate to the `/about` route
/// without triggering a full page reload.
///
/// # Security Note
/// This component will not work correctly if the `to` route is an external URL (e.g. `https://example.com`).
/// In this case, you should use a standard `a { ... }` element instead.
///
/// This is to prevent Open Redirect vulnerabilities.
#[component]
pub fn Link(
    #[props(extends = GlobalAttributes, extends = a)] attributes: Vec<Attribute>,
    to: String,
    children: Element,
) -> Element {
    if !is_internal_path(&to) {
        return rsx! {
            a {
                ..attributes,
                {children}
            }
        };
    }

    let mut nav = use_navigation();

    rsx! {
        a {
            href: "{to}",
            onclick: move |e| {
                e.prevent_default();
                nav.push(to.clone());
            },
            ..attributes,
            {children}
        }
    }
}

/// Default 404 page to display when no matching route is found.
#[component]
pub fn NotFound(
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
    path: Option<String>,
) -> Element {
    rsx! {
        div {
            ..attributes,
            h1 { "404 - Not Found" }
            if let Some(path) = path {
                p { "No route found for: {path}" }
            }
        }
    }
}

/// Renders an "Internal Server Error" page with optional validation error details.
#[component]
pub fn InternalServerErrors(errors: Option<ValidationErrors>) -> Element {
    rsx! {
        div {
            style: "padding: 2rem; background-color: #fff;  font-family: sans-serif;",
            height: "100vh",
            background_color: "white",
            color: "black",
            h1 { "500 - Internal Server Error" }
            if cfg!(debug_assertions) && let Some(errors) = errors {
                p { "The router encountered validation errors:" }
                ol {
                    for error in errors.errors() {
                        li { "{error}" }
                    }
                }
            }
        }
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

/// Validates that a path is safe for internal navigation.
///
/// Returns `true` only if the path starts with `/` and is NOT protocol-relative (`//`).
pub fn is_internal_path(path: &str) -> bool {
    let trimmed = path.trim_start();
    trimmed.starts_with('/') && !trimmed.starts_with("//")
}
