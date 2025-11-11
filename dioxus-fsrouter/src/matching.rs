use crate::route::{ROUTES, RouteInfo};

/// Find the best matching route for a given path
pub fn match_route(path: &str) -> Option<&'static RouteInfo> {
    // Phase 1: Simple exact matching
    // Future: Will handle :params, priorities, nested routes

    ROUTES.iter().find(|route| route.path == path).copied()
}

// Future: Extract parameters from path
// pub fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> { ... }

// Future: Match nested routes considering parent paths
// pub fn match_nested_route(path: &str, parent_path: Option<&str>) -> Option<&'static RouteInfo> { ... }
