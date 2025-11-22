//! Component-based router for Dioxus with compile-time safety

extern crate self as dioxus_fsrouter;
pub mod errors;
pub mod route;
pub mod router;

#[cfg(test)]
mod tests;

pub use dioxus_fsrouter_macro as macros;
#[doc(hidden)]
pub use inventory;

pub use errors::{ParseError, RouterError, ValidationErrors};
pub use route::{RenderFn, RouteInfo, find_route, get_routes, validate_routes};
pub use router::{Link, Navigation, Outlet, Router, use_navigation};

pub mod prelude {
    pub use crate::macros::route;
    pub use crate::{
        Link, Navigation, Outlet, ParseError, RenderFn, Router, RouterError, ValidationErrors,
        get_routes, route, route::validate_routes_or_panic, use_navigation, validate_routes,
    };
    #[doc(hidden)]
    pub use inventory;
}
