use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

use crate::helpers::{NavButton, capitalize_first};

static USERS: &[&str] = &["Alice", "Bob", "Charlie"];

#[route("/users")]
#[component]
fn UserList() -> Element {
    rsx! {
        h1 { "Users" }
        ul {
            for user in USERS.iter() {
                li { Link { to: format!("/users/{}", user.to_lowercase()), "{user}" } }
            }
        }
    }
}

#[route("/users/:username")]
#[component]
fn UserProfile(username: String) -> Element {
    rsx! {
        h1 { "User Profile: {capitalize_first(&username)}" }
        p { "This is a dynamic route matching /user/:username" }
        div {
            style: "margin-top: 1rem; padding: 1rem; background: #fff; border-radius: 4px;",
            img {
                src: "https://api.dicebear.com/7.x/avataaars/svg?seed={username}",
                height: "100",
                width: "100"
            }
        }
        div{
            id: "page-navigation",
            NavButton {path: "/users", text: "Back to Users"}
        }
    }
}
