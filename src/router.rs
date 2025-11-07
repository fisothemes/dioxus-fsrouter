use dioxus::prelude::*;

/// Router context shared across components
#[derive(Clone, Copy)]
pub struct RouterContext {
    pub current_path: Signal<String>,
}

/// Main Router component
///
/// # Example
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use dioxus_fsrouter::*;
/// fn App() -> Element {
///     rsx! {
///         Router {
///             resolver: my_resolver,
///             // Your app components
///         }
///     }
/// }
/// # fn my_resolver(path: String) -> Element { rsx! { div {} } }
/// ```
#[component]
pub fn Router(
    children: Element,
    resolver: fn(String) -> Element
) -> Element {
    let mut current_path = use_signal(|| "/".to_string());
    let mut rendered_route = use_signal(|| None::<Element>);

    // Provide router context
    use_context_provider(|| RouterContext {
        current_path,
    });

    // Re-render when route changes
    use_effect(move || {
        let path = current_path();
        let new_element = resolver(path);
        rendered_route.set(Some(new_element));
    });

    rsx! {
        {children}
        div {
            {rendered_route().unwrap_or_else(|| rsx! { div { "Loading..." } })}
        }
    }
}