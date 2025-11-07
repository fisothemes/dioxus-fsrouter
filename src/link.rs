use dioxus::prelude::*;
use crate::hooks::use_navigator;

/// Link component for client-side navigation
///
/// # Example
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use dioxus_fsrouter::*;
/// fn Navbar() -> Element {
///     rsx! {
///         nav {
///             Link { to: "/".to_string(), "Home" }
///             Link { to: "/about".to_string(), "About" }
///         }
///     }
/// }
/// ```
#[component]
pub fn Link(to: String, children: Element) -> Element {
    let mut navigate = use_navigator();
    let target_path = to.clone();

    rsx! {
        a {
            href: "{to}",
            onclick: move |evt| {
                evt.prevent_default();
                navigate(&target_path);
            },
            {children}
        }
    }
}