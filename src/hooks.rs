use crate::{RouterContext, RouteParams};
use dioxus_core::{try_consume_context, use_hook};

use dioxus::prelude::*;
use std::collections::HashMap;

/// Hook to get the navigation function
///
/// # Example
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use dioxus_fsrouter::*;
/// fn MyComponent() -> Element {
///     let mut navigate = use_navigator();
///
///     rsx! {
///         button {
///             onclick: move |_| navigate("/home"),
///             "Go Home"
///         }
///     }
/// }
/// ```
pub fn use_navigator() -> impl FnMut(&str) + Copy {
    let mut current_path = use_context::<RouterContext>().current_path;

    move |path: &str| {
        current_path.set(path.to_string());
    }
}

/// Hook to get route parameters
///
/// # Example
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use dioxus_fsrouter::*;
/// // For route "/blog/:id"
/// fn BlogPost() -> Element {
///     let params = use_params();
///     let blog_id = params.get("id");
///
///     rsx! {
///         div { "Blog ID: {blog_id:?}" }
///     }
/// }
/// ```
pub fn use_params() -> HashMap<String, String> {
    use_context::<RouteParams>().0.clone()
}