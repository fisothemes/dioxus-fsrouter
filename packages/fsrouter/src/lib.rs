//! Component-based router for Dioxus with compile-time safety
pub mod route;
pub mod router;

#[doc(hidden)]
pub use inventory;

pub use dioxus_fsrouter_macro as macros;

pub use route::{RouteInfo, find_route, get_routes, validate_routes};
pub use router::{Link, Navigation, Outlet, Router, use_navigation};

pub mod prelude {
    pub use crate::macros::route;
    pub use crate::{
        Link, Navigation, Outlet, Router, get_routes, route, router, use_navigation,
        validate_routes,
    };
    #[doc(hidden)]
    pub use inventory;
}
