use dioxus::prelude::*;
use std::collections::HashMap;

/// Context passed to route render functions
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// The matched URL path
    pub path: String,

    /// Extracted route parameters (e.g., :id)
    pub params: HashMap<String, String>,

    // Future: Query parameters
    // pub query: QueryParams,

    // Future: Preloaded data
    // pub data: Option<Box<dyn Any>>,
}

impl RenderContext {
    /// Create a new render context (Phase 1: no params)
    pub fn new(path: String) -> Self {
        Self {
            path,
            params: HashMap::new(),
        }
    }
}

/// Trait implemented by routable components
pub trait Routable: 'static {
    /// Primary route path
    const PATH: &'static str;

    /// Component name (for debugging)
    const COMPONENT_NAME: &'static str;

    /// Render this component with the given context
    fn render(ctx: RenderContext) -> Element;

    /// Get route metadata
    fn route_info() -> &'static RouteInfo;
}

/// Metadata about a registered route
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// Primary route path
    pub path: &'static str,

    /// Component name
    pub component_name: &'static str,

    /// Render function
    pub render_fn: fn(RenderContext) -> Element,

    /// Priority for matching (higher = more specific)
    pub priority: i32,

    /// Whether this is a nested route (has parent)
    /// Future: Used for nested outlet matching
    pub is_nested: bool,
}

impl RouteInfo {
    pub const fn new(
        path: &'static str,
        component_name: &'static str,
        render_fn: fn(RenderContext) -> Element,
    ) -> Self {
        Self {
            path,
            component_name,
            render_fn,
            priority: 1000, // Phase 1: All routes have same priority
            is_nested: false, // Future: Detect from path hierarchy
        }
    }
}

/// Global route registry using linkme
#[linkme::distributed_slice]
pub static ROUTES: [&'static RouteInfo];