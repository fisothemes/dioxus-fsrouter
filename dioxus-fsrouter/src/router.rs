use dioxus::prelude::*;

/// Router context provided to all child components
#[derive(Clone, Copy)]
pub struct RouterContext {
    /// Current route signal
    pub current_route: Signal<String>,
}

/// Router component - manages routing state
#[component]
pub fn Router(children: Element) -> Element {
    // Get initial URL from the browser
    let current_route = use_signal(|| get_current_path());

    // Set up router context
    use_context_provider(|| RouterContext { current_route });

    // Listen for browser navigation (back/forward)
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        use_effect(move || {
            let window = web_sys::window().unwrap();
            let mut current_route = current_route.clone();

            let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
                current_route.set(get_current_path());
            }) as Box<dyn FnMut(_)>);

            window
                .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget();
        });
    }

    rsx! { {children} }
}

/// Get current path from browser
fn get_current_path() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_else(|| "/".to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "/".to_string()
    }
}

/// Hook to get current route path
pub fn use_route() -> Signal<String> {
    let ctx = use_context::<RouterContext>();
    ctx.current_route
}

// Future: Hook to navigate programmatically
// pub fn use_navigation() -> Navigator { ... }
