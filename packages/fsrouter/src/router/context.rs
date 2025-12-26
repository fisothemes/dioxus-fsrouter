use dioxus::prelude::*;
use std::collections::HashMap;

/// Context providing metadata about the currently active route.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteContext {
    /// The actual URL path (e.g. "/article/hello-world")
    pub url: String,
    /// The pattern that matched (e.g. "/article/:slug")
    pub pattern: &'static str,
    /// Whether this route was matched via a redirect
    pub is_redirect: bool,
    /// Name of the matched component
    pub component_name: &'static str,
    /// Extracted route parameters
    pub params: HashMap<String, String>,
}

/// Hook to access the current route's metadata.
///
/// # Panics
/// Panics if used outside a component rendered by the Router.
pub fn use_route_context() -> RouteContext {
    use_context::<RouteContext>()
}
