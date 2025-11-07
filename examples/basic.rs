use dioxus::prelude::*;
use dioxus_fsrouter::*;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    rsx! {
        Router {
            resolver: route_resolver,
            Navbar {}
        }
    }
}

fn route_resolver(path: String) -> Element {
    match_route!(&path => {
        "/" => rsx! { Home {} },
        "/about" => rsx! { About {} },
        "/contact" => rsx! { Contact {} },
    })
}

#[component]
fn Navbar() -> Element {
    rsx! {
        nav {
            style: "padding: 1rem; background: #333; color: white;",
            Link { to: "/".to_string(),
                span { style: "margin-right: 1rem;", "Home" }
            }
            Link { to: "/about".to_string(),
                span { style: "margin-right: 1rem;", "About" }
            }
            Link { to: "/contact".to_string(),
                span { "Contact" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "Home Page" }
            p { "Welcome to the basic routing example!" }
        }
    }
}

#[component]
fn About() -> Element {
    rsx! {
        div {
            h1 { "About Page" }
            p { "This is a simple router for Dioxus." }
        }
    }
}

#[component]
fn Contact() -> Element {
    rsx! {
        div {
            h1 { "Contact Page" }
            p { "Get in touch with us!" }
        }
    }
}