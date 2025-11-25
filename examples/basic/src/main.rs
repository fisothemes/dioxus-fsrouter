mod blog;
mod helpers;
mod users;

use helpers::NavButton;

use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

static CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet{ href: CSS }
        Router {
            NavBar {}
            main { Outlet {} }
        }
    }
}

#[component]
fn NavBar() -> Element {
    rsx! {
        nav {
            Link { to: "/".to_string(), "Home" }
            Link { to: "/blog".to_string(), "Blogs" }
            Link { to: "/users".to_string(), "Users" }
            Link { to: "/about".to_string(), "About" }
        }
    }
}

#[route("/")]
#[component]
fn Home() -> Element {
    rsx! {
        h1{ "Home" }
        p { "Welcome to the Dioxus FsRouter Phase 2 (Route Parameters) demo!" }
        p { "This example demonstrates:" }
        ul {
            li { "Static Routes (e.g., /about)" }
            li { "Dynamic Parameters (e.g., /user/:name)" }
            li { "Multiple Types (String, u32)" }
            li { "Priority Scoring (Static beats Dynamic)" }
        }
        button {
            onclick: move |_| helpers::print_all_routes(), "Print Route Registry"
        }
    }
}

#[route("/about")]
#[component]
fn About() -> Element {
    rsx! {
        h1{ "About" }
        p {
            "This is a simple, attribute-based router for Dioxus applications that provides "
            br { "component-based routing with compile-time safety, automatic route registration, " }
            br { "and minimal boilerplate" }
        }
        p { "The core principles of this router are:" }
        ul {
            li { b { "Component-First: " } "Routes are attached directly to components via attributes" }
            li { b { "Type-Safe: " } "All navigation is type-checked at compile time" }
            li { b { "Minimal Boilerplate: " } "No enums, no manual registration, no string-based routing" }
            li { b { "Compile-Time Validation: " } "Invalid URLs caught at compile time" }
            li { b { "Auto-Discovery: " } "Routes automatically register themselves via global inventory" }
        }
        div{
            id: "page-navigation",
            NavButton {path: "/contact".to_string(), text: "Contact"}
        }
    }
}

#[route("/contact")]
#[component]
fn Contact() -> Element {
    rsx! {
        h1 { "Contact" }
        p { "Check out the repo and get in touch on GitHub!" }
        a {
            href: "https://github.com/fisothemes/dioxus-fsrouter",
            target: "_blank",
            "fisothemes/dioxus-fsrouter"
        }
        div{
            id: "page-navigation",
            NavButton {path: "/about".to_string(), text: "Back to About"}
        }
    }
}
