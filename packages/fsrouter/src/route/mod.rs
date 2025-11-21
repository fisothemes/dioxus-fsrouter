//! This module provides a system to manage and validate routes for a Dioxus application.
//! It allows for automatic route registration and validation, as well as dynamic route lookup.

use dioxus::prelude::*;
pub mod validate;
pub mod pattern;

pub use validate::{validate_routes, validate_routes_or_panic};

/// Render function type for routes
pub type RenderFn = fn() -> Element;

// Global route registry using inventory
inventory::collect!(RouteInfo);

/// Metadata for a single route
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

    /// Get the route path
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Get the component name
    pub fn component_name(&self) -> &'static str {
        self.component_name
    }

    /// Renders the component associated with this route
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
