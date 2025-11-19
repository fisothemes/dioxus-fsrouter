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
            main {
                Outlet {}
            }
        }
    }
}

#[component]
fn NavBar() -> Element {
    rsx! {
        nav {
            Link {
                to: "/".to_string(),
                span {
                    id: "nav-link",
                    "Home"
                }
            }
            Link {
                to: "/about".to_string(),
                span {
                    id: "nav-link",
                    "About"
                }
            }
            Link {
                to: "/contact".to_string(),
                span {
                    id: "nav-link",
                    "Contact"
                }
            }
        }
    }
}

fn _print_all_routes() {
    let msg = "=== Printing registered routes... ===";
    let width = msg.len();

    println!("{msg}");
    for route in get_routes() {
        println!("{} \t-> {}", route.path(), route.component_name());
    }

    println!("{:=<width$}", "");
}

#[route("/")]
#[component]
fn Home() -> Element {
    rsx! {
        h1{ "Home" }
        p { "Welcome to the Dioxus FsRouter Phase 1 demo!" }
    }
}

#[route("/about")]
#[component]
fn About() -> Element {
    rsx! {
        h1{ "About" }
        p {
            style: "color: #6b7280; line-height: 1.6;",
            "This is a minimal router implementation using:"
        }
        ul {
            style: "color: #6b7280; line-height: 1.8; margin-top: 1rem;",
            li { "inventory crate for compile-time registration" }
            li { "Procedural macros for the #[route] attribute" }
            li { "Dioxus components and context" }
        }
        p {
            style: "color: #6b7280; line-height: 1.6; margin-top: 1rem;",
            "Phase 1 focuses on validating the core approach before building advanced features."
        }
    }
}

#[route("/contact")]
#[component]
fn Contact() -> Element {
    rsx! {
        h1 { "Contact" }
        p { "Feel free to get in touch!" }
        a {
            style:
                "display: flex; \
                align-items: center; \
                gap: 0.5rem; \
                text-decoration: none;",
            href: "https://github.com/fisothemes/dioxus-fsrouter",
            img {
                style: "width: 48px; height: 48px;",
                src: "https://img.icons8.com/?size=100&id=12599&format=png&color=000000"
            }
            "GitHub"
        }
    }
}
