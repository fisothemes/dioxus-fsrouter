use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

use crate::helpers::{NavButton, capitalize_first};

#[route("/blog")]
#[component]
fn BlogList() -> Element {
    rsx! {
        h1 { "Blog Posts" }
        p { "Choose a post to test parameter parsing:" }

        ul {
            li { Link {  to: "/blog/featured".to_string(), strong { "Featured Post (Priority Test)" } } }
            li { Link { to: "/blog/rust/1".to_string(), "Rust Basics (id: 1)" } }
            li { Link { to: "/blog/dioxus/42".to_string(), "Dioxus Guide (id: 42)" } }

            // Invalid Link (Intentionally Broken)
            li {
                Link {
                    to: "/blog/hacking/not-a-number".to_string(),
                    strong {
                        style: "color: red;",
                        "Broken Link (ID is not u32)"
                    }
                }
            }
        }
    }
}

#[route("/blog/featured")]
#[component]
fn FeaturedPost() -> Element {
    rsx! {
        h1 { "Featured Post" }
        p { "This is a featured blog post." }
        div {
            id: "page-navigation",
            NavButton {path: "/blog".to_string(), text: "Back to Blog List"}
        }
    }
}

#[route("/blog/:category/:id")]
#[component]
fn BlogPost(category: String, id: u32) -> Element {
    let prev_id = id.saturating_sub(1);
    let next_id = id.saturating_add(1);

    rsx! {
        h1 { "{capitalize_first(&category)} Post {id}" }
        p { "This component received strongly typed parameters:" }
        ul {
            li { b { "category: " } "String = \"{category}\"" }
            li { b { "id: " } "u32 = {id}" }
        }
        div {
            id: "page-navigation",
            NavButton {path: format!("/blog/{}/{}", category, prev_id), text: "Previous"}
            NavButton {path: format!("/blog/{}/{}", category, next_id), text: "Next"}
            NavButton {path: "/blog".to_string(), text: "Back to Blog List"}
        }
    }
}
