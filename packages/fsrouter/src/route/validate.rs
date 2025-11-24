use crate::errors::{RouterError, ValidationErrors};
use crate::route::{RouteInfo, RoutePriority, Segment, get_routes};

/// Validate all registered routes
///
/// This checks for:
/// - Duplicate route paths
/// - No routes registered
/// - Ambiguous routes (overlapping patterns with the same priority, e.g. '/user/:id' and '/user/:name')
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
    let routes = get_routes();

    if routes.is_empty() {
        validation_errors.add(RouterError::NoRoutesRegistered);
        return Err(validation_errors);
    }

    let mut seen: HashMap<&str, &str> = HashMap::new();
    let mut priority_buckets: HashMap<RoutePriority, Vec<&RouteInfo>> = HashMap::new();

    for route in routes {
        // Check for exact duplicate paths
        if let Some(existing_component) = seen.get(route.path()) {
            validation_errors.add(RouterError::DuplicateRoute {
                path: route.path().to_string(),
                first_component: existing_component.to_string(),
                second_component: route.component_name().to_string(),
            });
            // Don't add to duplicated routes to the priority buckets to avoid duplicates in the error message
            continue;
        }

        seen.insert(route.path(), route.component_name());

        priority_buckets
            .entry(route.priority())
            .or_default()
            .push(route);
    }

    for (_, bucket) in priority_buckets {
        // Ambiguity can only happen if 2+ routes share the same priority
        if bucket.len() < 2 {
            continue;
        }

        for (i, route_a) in bucket.iter().enumerate() {
            for route_b in bucket.iter().skip(i + 1) {
                if are_patterns_ambiguous(
                    route_a.pattern().segments(),
                    route_b.pattern().segments(),
                ) {
                    validation_errors.add(RouterError::AmbiguousRoutes {
                        path_a: route_a.path().to_string(),
                        component_a: route_a.component_name().to_string(),
                        path_b: route_b.path().to_string(),
                        component_b: route_b.component_name().to_string(),
                    });
                }
            }
        }
    }

    if validation_errors.is_empty() {
        Ok(())
    } else {
        Err(validation_errors)
    }
}

/// Checks if two sets of path segments are ambiguous, meaning that they
/// can overlap (or conflict) when matched against the same request path.
///
/// # Parameters
/// - `segments_a`: A slice of `Segment` representing the first set of segments.
/// - `segments_b`: A slice of `Segment` representing the second set of segments.
///
/// # Returns
/// - `true` if the paths represented by `segments_a` and `segments_b` can potentially overlap
///   (i.e., are ambiguous).
/// - `false` if the paths are inherently distinct and cannot overlap.
///
/// # Examples
/// ```
/// use dioxus_fsrouter:: {Segment, are_patterns_ambiguous};
///
/// let segments_a = vec![Segment::Static("user".into()), Segment::Param("id".into())];
/// let segments_b = vec![Segment::Static("user".into()), Segment::Param("name".into())];
///
/// assert!(are_patterns_ambiguous(&segments_a, &segments_b)); // Paths can conflict
///
/// let segments_c = vec![Segment::Static("user".into()), Segment::Static("profile".into())];
/// let segments_d = vec![Segment::Static("post".into()), Segment::Static("edit".into())];
///
/// assert!(!are_patterns_ambiguous(&segments_c, &segments_d)); // Paths are distinct
/// ```
///
/// # Notes
/// - The ambiguity arises when both paths can match the same input path.
/// - The function assumes that all `Static` segments are case-sensitive.
pub fn are_patterns_ambiguous(segments_a: &[Segment], segments_b: &[Segment]) -> bool {
    if segments_a.len() != segments_b.len() {
        return false;
    }

    // Check segment by segment
    for (a, b) in segments_a.iter().zip(segments_b.iter()) {
        match (a, b) {
            (Segment::Static(sa), Segment::Static(sb)) => {
                if sa != sb {
                    return false;
                }
            }
            _ => continue,
        }
    }

    true
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
