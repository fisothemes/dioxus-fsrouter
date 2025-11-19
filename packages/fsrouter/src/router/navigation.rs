use dioxus::prelude::*;

/// Navigation handle for programmatic navigation
#[derive(Clone, Copy)]
pub struct Navigation {
    current_route: Signal<String>,
}

impl Navigation {
    /// Navigate to a new path
    pub fn push(&mut self, path: impl Into<String>) {
        let path = path.into();

        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ =
                        history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
                }
            }
        }

        // Update the signal directly
        self.current_route.set(path);
    }

    /// Replace the current path without adding to history
    pub fn replace(&mut self, path: impl Into<String>) {
        let path = path.into();

        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.replace_state_with_url(
                        &wasm_bindgen::JsValue::NULL,
                        "",
                        Some(&path),
                    );
                }
            }
        }

        self.current_route.set(path);
    }

    /// Go back in history
    pub fn go_back(&self) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.back();
                }
            }
        }
    }

    /// Go forward in history
    pub fn go_forward(&self) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(history) = window.history().ok() {
                    let _ = history.forward();
                }
            }
        }
    }

    /// Get the current path
    pub fn current_path(&self) -> String {
        (*self.current_route.read()).clone()
    }
}

/// Navigation hook for programmatic navigation
///
/// # Example
/// ```ignore
/// let nav = use_navigation();
/// nav.push("/about");
/// nav.go_back();
/// ```
pub fn use_navigation() -> Navigation {
    let nav_ctx = use_context::<NavigationContext>();
    Navigation {
        current_route: nav_ctx.current_route,
    }
}

/// Navigation context shared across the app
#[derive(Clone, Copy)]
pub(crate) struct NavigationContext {
    pub current_route: Signal<String>,
}
