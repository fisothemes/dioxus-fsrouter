use crate::errors::{RouterError, ValidationErrors};
use crate::route::get_routes;

/// Validate all registered routes
///
/// This checks for:
/// - Duplicate route paths
/// - No routes registered
///
/// # Returns
/// - `Ok(())` if all routes are valid
/// - `Err(ValidationErrors)` containing all validation errors found
///
/// # Example
/// ```ignore
/// match validate_routes() {
///     Ok(()) => println!("All routes are valid"),
///     Err(e) => eprintln!("Validation failed:\n{}", e),
/// }
/// ```
pub fn validate_routes() -> std::result::Result<(), ValidationErrors> {
    use std::collections::HashMap;

    let mut validation_errors = ValidationErrors::new();
    let mut seen: HashMap<&str, &str> = HashMap::new();
    let mut route_count = 0;

    for route in get_routes() {
        route_count += 1;

        // Check for duplicate paths
        if let Some(existing_component) = seen.get(route.path()) {
            validation_errors.add(RouterError::DuplicateRoute {
                path: route.path().to_string(),
                first_component: existing_component.to_string(),
                second_component: route.component_name().to_string(),
            });
        } else {
            seen.insert(route.path(), route.component_name());
        }
    }

    // Check if any routes were registered
    if route_count == 0 {
        validation_errors.add(RouterError::NoRoutesRegistered);
    }

    if validation_errors.is_empty() {
        Ok(())
    } else {
        Err(validation_errors)
    }
}

/// Validate routes and panic with a detailed error message if validation fails
///
/// This is useful for ensuring routes are valid at application startup.
///
/// # Panics
/// Panics if any validation errors are found, with a detailed error message.
///
/// # Example
/// ```ignore
/// fn main() {
///     validate_routes_or_panic();
///     dioxus::launch(App);
/// }
/// ```
pub fn validate_routes_or_panic() {
    match validate_routes() {
        Ok(()) => {}
        Err(errors) => {
            panic!("\n\n{}\n", errors);
        }
    }
}
