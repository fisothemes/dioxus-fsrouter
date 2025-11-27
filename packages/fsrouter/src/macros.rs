pub use dioxus_fsrouter_macro::*;

/// Assert that the route registry is valid (no duplicates or ambiguities).
///
/// # Example
/// ```ignore
/// #[test]
/// fn test_routes() {
///     dioxus_fsrouter::assert_routes_valid!();
/// }
/// ```
#[macro_export]
macro_rules! assert_routes_valid {
    () => {
        if let Err(e) = $crate::validate_routes() {
            panic!("Route validation failed:\n{}", e);
        }
    };
}

/// Assert that a path matches a specific component.
///
/// # Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_fsrouter::prelude::*;
///
/// #[route("/user/123")]
/// #[component]
/// fn UserProfile() -> Element {
///     rsx!{ "User profile page for user ID 123" }
/// }
///
/// assert_route_matches!("/user/123", UserProfile);
/// ```
#[macro_export]
macro_rules! assert_route_matches {
    ($path:expr, $component:ident) => {
        $crate::assert_route_matches!($path, $component, {})
    };
    ($path:expr, $component:ident, { $($param:ident : $val:expr),* }) => {
        {
            let (route, params) = $crate::find_route($path)
                .expect(&format!("No route matched path '{}'", $path));

            // Check component name (simple string check)
            // Note: This relies on the component name being part of the RouteInfo
            assert!(
                route.component_name().contains(stringify!($component)),
                "Expected path '{}' to match component '{}', but it matched '{}'",
                $path,
                stringify!($component),
                route.component_name()
            );

            // Check params
            $(
                assert_eq!(
                    params.get(stringify!($param)).map(|s| s.as_str()),
                    Some($val),
                    "Parameter '{}' mismatch for path '{}'",
                    stringify!($param),
                    $path
                );
            )*
        }
    };
}
