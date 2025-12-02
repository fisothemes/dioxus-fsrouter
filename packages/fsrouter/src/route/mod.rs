//! This module provides a system to manage and validate routes for a Dioxus application.
//! It allows for automatic route registration and validation, as well as dynamic route lookup.

use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

pub mod pattern;
pub mod validate;

pub use pattern::{RoutePattern, RoutePriority, Segment};
pub use validate::{
    are_patterns_ambiguous, validate_route_registry, validate_routes, validate_routes_or_panic,
};

use crate::errors::ParseError;

pub type StaticRouteRenderFn = fn() -> Element;
pub type DynamicRouteRenderFn = fn(HashMap<String, String>) -> Result<Element, ParseError>;

/// Function pointer types for rendering routes
#[derive(Debug, Clone, Copy, Hash)]
pub enum RenderFn {
    /// Static route with no parameters
    Static(StaticRouteRenderFn),
    /// Dynamic route that requires parameters
    /// Returns Err if parameter parsing fails
    WithParams(DynamicRouteRenderFn),
}

// Global route registry using inventory
inventory::collect!(RouteInfo<'static>);

/// Metadata for a single route
#[derive(Debug, Clone)]
pub struct RouteInfo<'a> {
    /// Primary route path
    path: &'a str,
    /// Parsed pattern for matching (lazy-initialised)
    pattern: &'a OnceLock<Result<RoutePattern, ParseError>>,
    /// Component name
    component_name: &'a str,
    /// Render function
    render_fn: RenderFn,
}

impl<'a> RouteInfo<'a> {
    /// Create new route info
    pub const fn new(
        path: &'a str,
        pattern: &'a OnceLock<Result<RoutePattern, ParseError>>,
        component_name: &'a str,
        render_fn: RenderFn,
    ) -> Self {
        Self {
            path,
            pattern,
            component_name,
            render_fn,
        }
    }

    /// Get the route path
    pub fn path(&self) -> &'a str {
        self.path
    }

    /// Get the parsed pattern (lazy initialisation)
    pub fn pattern(&self) -> Result<&RoutePattern, &ParseError> {
        self.pattern
            .get_or_init(|| RoutePattern::parse(self.path))
            .as_ref()
    }

    /// Get the component name
    pub fn component_name(&self) -> &'a str {
        self.component_name
    }

    /// Get the priority for route matching
    ///
    /// If the pattern cannot be parsed, returns -1 as a fallback priority.
    pub fn priority(&self) -> RoutePriority {
        self.pattern().map(|p| p.priority()).unwrap_or(-1)
    }

    /// Check if this route matches the given URL
    pub fn matches(&self, url: &str) -> Option<HashMap<String, String>> {
        self.pattern().ok()?.matches(url)
    }

    /// Renders the component associated with this route
    pub fn render(&self, params: Option<HashMap<String, String>>) -> Result<Element, ParseError> {
        match (&self.render_fn, params) {
            (RenderFn::Static(f), _) => Ok(f()),
            (RenderFn::WithParams(f), Some(p)) => f(p),
            (RenderFn::WithParams(_), None) => {
                // This should never happen if routing logic is correct
                #[cfg(debug_assertions)]
                panic!(
                    "Route '{}' requires parameters but none were provided. \
                    This is a bug in the router.",
                    self.path
                );

                #[cfg(not(debug_assertions))]
                Err(ParseError::MissingParams {
                    route: self.path.to_string(),
                })
            }
        }
    }
}

/// Get all registered routes (sorted by priority)
///
/// Routes are always returned in priority order (highest first).
/// Initialisation happens automatically on the first call.
pub fn get_routes() -> &'static [&'static RouteInfo<'static>] {
    static SORTED_ROUTES: OnceLock<Vec<&'static RouteInfo<'static>>> = OnceLock::new();

    SORTED_ROUTES.get_or_init(|| {
        let mut routes =
            inventory::iter::<RouteInfo<'static>>().collect::<Vec<&'static RouteInfo>>();

        routes.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        routes
    })
}

/// Find a route matching the given path
///
/// Returns the matched route and extracted parameters.
/// Routes are always checked in priority order.
///
/// The pattern matcher handles URL normalisation and decoding.
///
/// # Performance
/// - First call: O(N log N) to initialise and sort + O(N) to match
/// - Subsequent calls: O(N) to match only
pub fn find_route(path: &str) -> Option<(&'static RouteInfo<'static>, HashMap<String, String>)> {
    // Routes are always sorted by get_routes()
    // Pattern::matches() handles normalization and decoding
    for route in get_routes() {
        if let Some(params) = route.matches(path) {
            return Some((route, params));
        }
    }

    None
}
