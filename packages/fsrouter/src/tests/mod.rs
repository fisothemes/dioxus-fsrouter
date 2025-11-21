use crate::{RouterError, ValidationErrors, find_route, get_routes, validate_routes};
use dioxus::prelude::*;

#[allow(dead_code)]
mod fixtures {
    use super::*;

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    #[crate::macros::route("/__test_home")]
    fn TestHome() -> Element {
        rsx!(div { "Home" })
    }

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    #[crate::macros::route("/__test_about")]
    fn TestAbout() -> Element {
        rsx!(div { "About" })
    }
}

#[test]
fn validation_errors_basic_behaviour() {
    let mut errors = ValidationErrors::new();
    assert!(errors.is_empty());
    assert_eq!(errors.len(), 0);

    errors.add(RouterError::NoRoutesRegistered);
    assert!(!errors.is_empty());
    assert_eq!(errors.len(), 1);

    let message = errors.to_string();
    assert!(message.contains("validation failed"));
    assert!(message.contains("No routes"));
}

#[test]
fn inventory_registers_routes_from_macro() {
    let paths: Vec<_> = get_routes().map(|route| route.path()).collect();

    assert!(
        paths.iter().any(|p| *p == "/__test_home"),
        "expected /__test_home in inventory: {:?}",
        &paths
    );
    assert!(
        paths.iter().any(|p| *p == "/__test_about"),
        "expected /__test_about in inventory: {:?}",
        &paths
    );
    assert!(paths.len() >= 2);
}

#[test]
fn find_route_returns_renderable_components() {
    let home = find_route("/__test_home").expect("route /__test_home should exist");
    assert!(home.component_name().contains("TestHome"));
    assert!(home.render().is_ok(), "render should return an Element");

    let about = find_route("/__test_about").expect("route /__test_about should exist");
    assert!(about.component_name().contains("TestAbout"));
    assert!(about.render().is_ok(), "render should return an Element");
}

#[test]
fn validate_routes_succeeds_with_unique_paths() {
    let result = validate_routes();
    assert!(
        result.is_ok(),
        "expected validation to pass but got: {:?}",
        result
    );
}
