use dioxus::prelude::*;
use std::sync::Once;

pub type RenderFn = fn() -> Element;

// Global route registry using inventory
inventory::collect!(RouteInfo);

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