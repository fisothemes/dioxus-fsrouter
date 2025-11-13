use crate::context::RouteContext;
use crate::matching::match_route;
use crate::route::RenderContext;
use crate::router::use_route;
use dioxus::prelude::*;

/// Outlet component - renders the matched route
#[component]
pub fn Outlet() -> Element {
    let current_route = use_route();
    let path = current_route();

    // Find a matching route
    let matched = match_route(&path);

    if let Some(route) = matched {
        // Create render context
        let render_ctx = RenderContext::new(path.clone());

        // Create route context for hooks (with empty params for now)
        let route_ctx = RouteContext::new(
            path.clone(),
            route.path,
            route.component_name,
            render_ctx.params.clone(),
        );

        // Provide route context to component
        use_context_provider(|| route_ctx);

        // Render the matched route
        (route.render_fn)(render_ctx)
    } else {
        // Phase 1: Simple 404
        // Future: Will use fallback route if available
        rsx! {
            div { class: "not-found",
                h1 { "404 - Not Found" }
                p { "Route '{path}' does not exist" }
            }
        }
    }
}

// Future: Nested outlet that only matches child routes
// #[component]
// pub fn NestedOutlet() -> Element { ... }
