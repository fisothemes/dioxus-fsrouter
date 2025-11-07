//! # dioxus-fsrouter
//!
//! A simple, filesystem-style router for Dioxus applications.
//!
//! ## Features
//! - Simple declarative routing with `match_route!` macro
//! - Dynamic route parameters (e.g., `/blog/:id`)
//! - Client-side navigation with `Link` component
//! - Navigation hooks for programmatic routing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use dioxus_fsrouter::*;
//!
//! fn App() -> Element {
//!     rsx! {
//!         Router {
//!             resolver: route_resolver,
//!             Navbar {}
//!         }
//!     }
//! }
//!
//! fn route_resolver(path: String) -> Element {
//!     match_route!(&path => {
//!         "/" => rsx! { Home {} },
//!         "/about" => rsx! { About {} },
//!     })
//! }
//! # fn Home() -> Element { rsx! { div { "Home" } } }
//! # fn About() -> Element { rsx! { div { "About" } } }
//! # fn Navbar() -> Element { rsx! { nav {} } }
//! ```

pub mod router;
pub mod link;
pub mod hooks;
pub mod params;

pub use router::{Router, RouterContext};
pub use link::Link;
pub use hooks::{use_navigator, use_params};
pub use params::RouteParams;

pub mod prelude{
    pub use crate::router::{Router, RouterContext};
    pub use crate::link::Link;
    pub use crate::hooks::{use_navigator, use_params};
    pub use crate::params::RouteParams;
}

// Re-export macros
#[macro_use]
pub mod macros;
