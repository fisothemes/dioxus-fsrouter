use crate::{
    ParseError, RenderFn, RouteInfo, RouterError, ValidationErrors, find_route, get_routes,
    validate_route_registry,
};
use dioxus::prelude::*;

use std::sync::OnceLock;

#[allow(dead_code)]
mod fixtures {
    use super::*;

    #[crate::macros::route("/__test_home")]
    #[component]
    fn TestHome() -> Element {
        rsx!(
            div { "Home" }
        )
    }

    #[crate::macros::route("/__test_about")]
    #[component]
    fn TestAbout() -> Element {
        rsx!(
            div { "About" }
        )
    }

    #[crate::macros::route("/__test_user/:username")]
    #[component]
    fn TestUser(username: String) -> Element {
        rsx!(
            div { "User: {username}" }
        )
    }

    #[crate::macros::route("/__test_post/:id")]
    #[component]
    fn TestPost(id: u32) -> Element {
        rsx!(
            div { "Post ID: {id}" }
        )
    }

    #[crate::macros::route("/__test_priority/static")]
    #[component]
    fn TestPriorityStatic() -> Element {
        rsx!(
            div { "Static Winner" }
        )
    }

    #[crate::macros::route("/__test_priority/:slug")]
    #[component]
    fn TestPriorityDynamic(slug: String) -> Element {
        rsx!(
            div { "Dynamic Loser: {slug}" }
        )
    }

    #[crate::macros::route("/__test_catch_all/:..segments")]
    #[component]
    fn TestCatchAll(segments: String) -> Element {
        rsx! {
            div { "Segments: {segments}" }
        }
    }

    #[crate::macros::route("/__test_catch_all_with_vec_params/:..segments")]
    #[component]
    fn TestCatchAllWithVecParams(segments: Vec<String>) -> Element {
        rsx! {
            div { "Segments: {segments:?}" }
        }
    }

    #[crate::macros::route("/__test_query_param?q")]
    #[component]
    fn TestQueryParam(q: String) -> Element {
        rsx! {
            div { "Query Param: {q}" }
        }
    }

    #[crate::macros::route("/__test_optional_query_param?q")]
    #[component]
    fn TestOptionalQueryParam(q: Option<String>) -> Element {
        rsx! {
            div { "Optional Query Param: {q:?}" }
        }
    }

    #[crate::macros::route("/__test_multiple_mixed_query_params?q&page")]
    #[component]
    fn TestMultipleMixedQueryParams(q: String, page: Option<u32>) -> Element {
        rsx! {
            div { "Query: {q}, Page {page:?}" }
        }
    }

    #[crate::macros::route("/__test_original", redirect = ["/__test_redirect"])]
    #[component]
    fn TestRedirect() -> Element {
        rsx!(
            div { "Redirect Test" }
        )
    }

    #[crate::macros::route("/__test_user_profile/:id", redirect = ["/__test_u/:id"])]
    #[component]
    fn TestRedirectParams(id: String) -> Element {
        rsx!(
            div { "User {id}" }
        )
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
    let paths: Vec<_> = get_routes().iter().map(|route| route.path()).collect();

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
    assert!(
        paths.iter().any(|p| *p == "/__test_user/:username"),
        "expected /__test_user/:username in inventory: {:?}",
        &paths
    );
    assert!(paths.len() >= 3);
}

#[test]
fn find_route_returns_renderable_components() {
    let (home, params) = find_route("/__test_home").expect("route /__test_home should exist");
    assert!(home.component_name().contains("TestHome"));
    assert!(
        home.render(Some(params)).is_ok(),
        "render should return an Element"
    );

    let (about, params) = find_route("/__test_about").expect("route /__test_about should exist");
    assert!(about.component_name().contains("TestAbout"));
    assert!(
        about.render(Some(params)).is_ok(),
        "render should return an Element"
    );

    let (user, params) =
        find_route("/__test_user/alice").expect("Should match /__test_user/:username");
    assert!(user.component_name().contains("TestUser"));
    assert!(
        user.render(Some(params)).is_ok(),
        "render should return an Element"
    );
}

#[test]
fn should_find_route_with_username_param_and_parse_string() {
    let (route, params) =
        find_route("/__test_user/alice").expect("Should match /__test_user/:username");
    assert!(route.component_name().contains("TestUser"));
    assert_eq!(params.get("username").map(|s| s.as_str()), Some("alice"));
}

#[test]
fn typed_params_in_route_should_match_expected_type() {
    // 1. Success Case
    let (route, params) = find_route("/__test_post/42").expect("Should match /__test_post/:id");
    let res = route.render(Some(params));
    assert!(res.is_ok(), "Should parse '42' as u32");

    // 2. Failure Case (Invalid Type)
    let (route, params) = find_route("/__test_post/not-a-number").unwrap();
    let res = route.render(Some(params));
    match res {
        Err(ParseError::InvalidType {
            param,
            expected_type,
            value,
            ..
        }) => {
            assert_eq!(param, "id");
            assert_eq!(expected_type, "u32");
            assert_eq!(value, "not-a-number");
        }
        _ => panic!("Expected InvalidType error, got {:?}", res),
    }
}

#[test]
fn catch_all_route_exists_in_registry() {
    let routes: Vec<_> = get_routes()
        .iter()
        .filter(|r| r.path().contains(":..segments"))
        .collect();

    assert!(!routes.is_empty(), "Expected catch-all routes in registry");
}

#[test]
fn catch_all_matches_multiple_segments() {
    let (route, params) =
        find_route("/__test_catch_all/a/b/c/d").expect("Should match catch-all route");

    assert!(route.path().contains(":..segments"));
    assert!(route.component_name().contains("TestCatchAll"));

    let segments = params.get("segments").unwrap();
    assert_eq!(segments, "a/b/c/d");
}

#[test]
fn catch_all_matches_single_segment() {
    let (route, params) =
        find_route("/__test_catch_all/readme").expect("Should match catch-all route");

    assert!(route.path().contains(":..segments"));
    let segments = params.get("segments").unwrap();
    assert_eq!(segments, "readme");
}

#[test]
fn catch_all_matches_empty_path() {
    let (route, params) = find_route("/__test_catch_all/").expect("Should match catch-all route");

    assert!(route.path().contains(":..segments"));
    let segments = params.get("segments").unwrap();
    assert_eq!(segments, "");
}

#[test]
fn catch_all_renders_with_string_type() {
    let (route, params) =
        find_route("/__test_catch_all/docs/guide").expect("Should match catch-all route");

    let result = route.render(Some(params));
    assert!(
        result.is_ok(),
        "Should successfully render with String param"
    );
}

#[test]
fn catch_all_renders_with_vec_string_type() {
    let (route, params) = find_route("/__test_catch_all_with_vec_params/docs/guide/intro")
        .expect("Should match catch-all route");

    let result = route.render(Some(params));
    assert!(
        result.is_ok(),
        "Should successfully render with Vec<String> param"
    );
}

#[test]
fn query_param_route_exists_in_registry() {
    let routes: Vec<_> = get_routes()
        .iter()
        .filter(|r| r.path() == "/__test_query_param?q")
        .collect();

    assert_eq!(routes.len(), 1, "Expected query param route in registry");
}

#[test]
fn required_query_param_present() {
    let (route, params) =
        find_route("/__test_query_param?q=rust").expect("Should match query param route");

    assert!(route.component_name().contains("TestQueryParam"));
    assert_eq!(params.get("q").unwrap(), "rust");

    let result = route.render(Some(params));
    assert!(result.is_ok(), "Should successfully parse required param");
}

#[test]
fn required_query_param_missing_causes_error() {
    let (route, params) =
        find_route("/__test_query_param").expect("Route should match even without query");

    let result = route.render(Some(params));
    assert!(
        result.is_err(),
        "Should fail when required param is missing"
    );

    match result {
        Err(ParseError::MissingParam { param, .. }) => {
            assert_eq!(param, "q");
        }
        _ => panic!("Expected MissingParam error, got {:?}", result),
    }
}

#[test]
fn optional_query_param_present() {
    let (route, params) = find_route("/__test_optional_query_param?q=hello")
        .expect("Should match optional query param route");

    assert_eq!(params.get("q").unwrap(), "hello");

    let result = route.render(Some(params));
    assert!(
        result.is_ok(),
        "Should successfully parse optional param when present"
    );
}

#[test]
fn optional_query_param_missing() {
    let (route, params) = find_route("/__test_optional_query_param")
        .expect("Should match optional query param route");

    assert!(!params.contains_key("q"));

    let result = route.render(Some(params));
    assert!(
        result.is_ok(),
        "Should handle missing optional param gracefully"
    );
}

#[test]
fn redirect_main_route_exists() {
    let (route, _) = find_route("/__test_original").expect("Main route should exist");

    assert!(route.component_name().contains("TestRedirect"));
    assert!(
        !route.is_redirect(),
        "Main route should not be marked as redirect"
    );
    assert!(
        route.canonical_path().is_none(),
        "Main route should have no canonical path"
    );
}

#[test]
fn redirect_alias_exists() {
    let (route, _) = find_route("/__test_redirect").expect("Redirect alias should exist");

    assert!(route.component_name().contains("TestRedirect"));
    assert!(route.is_redirect(), "Redirect should be marked as redirect");
    assert_eq!(route.canonical_path(), Some("/__test_original"));
}

#[test]
fn redirect_with_params_main_route() {
    let (route, params) =
        find_route("/__test_user_profile/alice").expect("Main route should exist");

    assert!(route.component_name().contains("TestRedirectParams"));
    assert!(!route.is_redirect());
    assert!(route.canonical_path().is_none());
    assert_eq!(params.get("id").unwrap(), "alice");
}

#[test]
fn static_routes_should_have_higher_priority() {
    // Static match should win
    let (route, _) = find_route("/__test_priority/static").unwrap();
    assert!(route.component_name().contains("TestPriorityStatic"));

    // Dynamic match should be taken only if static doesn't match
    let (route, params) = find_route("/__test_priority/other").unwrap();
    assert!(route.component_name().contains("TestPriorityDynamic"));
    assert_eq!(params.get("slug").map(|s| s.as_str()), Some("other"));
}

#[test]
fn registry_should_reject_duplicate_static_routes() {
    static P1: OnceLock<Result<crate::RoutePattern, ParseError>> = OnceLock::new();
    static P2: OnceLock<Result<crate::RoutePattern, ParseError>> = OnceLock::new();

    fn dummy_render() -> Element {
        rsx! {}
    }

    let r1 = RouteInfo::new(
        "/duplicate",
        &P1,
        "ComponentA",
        RenderFn::Static(dummy_render),
        None,
    );

    let r2 = RouteInfo::new(
        "/duplicate",
        &P2,
        "ComponentB",
        RenderFn::Static(dummy_render),
        None,
    );

    let registry = vec![&r1, &r2];
    let result = validate_route_registry(&registry);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Duplicate route"));
}

#[test]
fn registry_should_reject_ambiguous_routes() {
    // Test conflicting dynamic routes manually (Ambiguity Check)
    // /post/:id vs /post/:slug -> These have the same structure and priority

    use crate::errors::ParseResult;
    use std::collections::HashMap;

    static P3: OnceLock<Result<crate::RoutePattern, ParseError>> = OnceLock::new();
    static P4: OnceLock<Result<crate::RoutePattern, ParseError>> = OnceLock::new();

    fn dummy_dynamic_render(_: HashMap<String, String>) -> ParseResult<Element> {
        Ok(rsx! {})
    }

    let r1 = RouteInfo::new(
        "/post/:id",
        &P3,
        "PostById",
        RenderFn::WithParams(dummy_dynamic_render),
        None,
    );

    let r2 = RouteInfo::new(
        "/post/:slug",
        &P4,
        "PostBySlug",
        RenderFn::WithParams(dummy_dynamic_render),
        None,
    );

    let registry = vec![&r1, &r2];
    let result = validate_route_registry(&registry);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Ambiguous routes"));
    assert!(err.to_string().contains("PostById"));
    assert!(err.to_string().contains("PostBySlug"));
}
