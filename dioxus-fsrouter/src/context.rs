use dioxus::prelude::*;
use std::collections::HashMap;

/// Context available to route components via hooks
#[derive(Clone, Debug)]
pub struct RouteContext {
    /// The actual URL path
    pub url: String,

    /// The pattern that matched (e.g., "/user/:id")
    pub pattern: &'static str,

    /// The component name
    pub component_name: &'static str,

    /// Extracted route parameters
    pub params: HashMap<String, String>,
    // Future: Query parameters
    // pub query: QueryParams,

    // Future: Whether matched via alias
    // pub is_alias: bool,
}

impl RouteContext {
    pub fn new(
        url: String,
        pattern: &'static str,
        component_name: &'static str,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            url,
            pattern,
            component_name,
            params,
        }
    }

    /// Get a route parameter by name
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }
}

/// Hook to access route context from within a route component
pub fn use_route_context() -> RouteContext {
    use_context::<RouteContext>()
}

// Future hooks:

// Hook to get a specific route parameter
// Example: let user_id = use_param::<u32>("id")?;
// pub fn use_param<T: FromStr>(name: &str) -> Option<T> { ... }

// Hook to get query parameters
// Example: let (q, page) = (use_query("q")?, use_query("page")?);
// pub fn use_query(name: &str) -> Option<String> { ... }

// Hook to get all query parameters
// pub fn use_query_params() -> QueryParams { ... }
