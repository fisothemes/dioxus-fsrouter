//! Component-based router for Dioxus with compile-time safety

extern crate self as dioxus_fsrouter;
pub mod errors;
pub mod macros;
pub mod route;
pub mod router;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use inventory;

pub use errors::{ParseError, ParseResult, RouterError, ValidationErrors};
pub use route::{
    RenderFn, RouteInfo, RoutePattern, RoutePriority, Segment, are_patterns_ambiguous, find_route,
    get_routes, validate_route_registry, validate_routes, validate_routes_or_panic,
};
pub use router::{Link, Navigation, Outlet, Router, use_navigation};

pub mod prelude {
    pub use crate::macros::{redirect, route};
    pub use crate::{
        Link, Navigation, Outlet, ParseError, ParseResult, RenderFn, RouteInfo, RoutePattern,
        RoutePriority, Router, RouterError, Segment, ValidationErrors, assert_route_matches,
        assert_routes_valid, get_routes, route, use_navigation, validate_route_registry,
        validate_routes, validate_routes_or_panic,
    };
    #[doc(hidden)]
    pub use inventory;
}
