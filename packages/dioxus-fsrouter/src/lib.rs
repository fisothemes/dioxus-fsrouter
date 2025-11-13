//! Component-based router for Dioxus with compile-time safety

pub mod context;
pub mod matching;
pub mod outlet;
pub mod route;
pub mod router;

#[doc(hidden)]
pub use linkme;

pub use dioxus_fsrouter_macro::{route, router};

pub use context::{RouteContext, use_route_context};
pub use outlet::Outlet;
pub use route::{ROUTES, RenderContext, Routable, RouteInfo};
pub use router::{Router, RouterContext, use_route};

pub mod prelude {
    #[doc(hidden)]
    pub use linkme;
    pub use crate::context::{RouteContext, use_route_context};
    pub use crate::outlet::Outlet;
    pub use crate::route::Routable;
    pub use crate::router::{Router, use_route};
    pub use crate::{route, router};
}
