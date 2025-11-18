//! Route registry and utilities

use dioxus::prelude::*;
use std::sync::Once;

// Global route registry using inventory
inventory::collect!(RouteInfo);

/// Function that renders a component
pub type RenderFn = fn() -> Element;

/// Metadata about a registered route
#[derive(Debug, Clone, Hash)]
pub struct RouteInfo {
    /// Primary route path
    path: &'static str,

    /// Component name
    component_name: &'static str,

    /// Render function
    render_fn: RenderFn,
}

impl RouteInfo {
    pub const fn new(
        path: &'static str,
        component_name: &'static str,
        render_fn: RenderFn,
    ) -> Self {
        Self {
            path,
            component_name,
            render_fn,
        }
    }

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn component_name(&self) -> &'static str {
        self.component_name
    }

    pub fn render(&self) -> Element {
        (self.render_fn)()
    }
}

/// Get all registered routes
pub fn get_routes() -> impl Iterator<Item = &'static RouteInfo> {
    inventory::iter::<RouteInfo>()
}

/// Find a route matching the given path
pub fn find_route(path: &str) -> Option<&'static RouteInfo> {
    get_routes().find(|route| route.path() == path)
}

/// Validate routes at startup (detect duplicates)
///
/// Returns Ok(()) if all routes are valid, or Err with details of conflicts
pub fn validate_routes() -> Result<(), String> {
    use std::collections::HashMap;

    let mut seen: HashMap<&str, &str> = HashMap::new();
    let mut errors = Vec::new();

    for route in get_routes() {
        if let Some(existing) = seen.get(route.path()) {
            errors.push(format!(
                "Duplicate route '{}': defined in both '{}' and '{}'",
                route.path(),
                existing,
                route.component_name()
            ));
        } else {
            seen.insert(route.path(), route.component_name());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
