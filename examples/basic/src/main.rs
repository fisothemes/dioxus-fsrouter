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
                to: "/blog".to_string(),
                span {
                    id: "nav-link",
                    "Blogs"
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

#[route("/blog")]
#[component]
fn BlogHome() -> Element {
    let bloggers = ["Alice", "Bob", "Charlie"];

    rsx! {
        h1{ "Blog" }
        p { "Click on the links below to read some blog posts!" }
        ul {
            for blogger in bloggers.iter() {
                li {
                    Link {
                        to: format!("/blog/{blogger}/0"),
                        b { "{blogger}" }
                    }
                }
            }
        }
    }
}

#[route("/blog/:name/:id")]
#[component]
fn Blog(name: String, id: u32) -> Element {
    let name_for_prev = name.clone();
    let name_for_next = name.clone();

    rsx! {
        h1{ "{name}'s Blog Post" }
        p {
            "This is a blog post about the number "
            b {"{id}"}
            "."
        }
        p { "It's a bit longer than usual, but it's still a blog post." }
        span {
            id: "blog-nav",
            button {
                onclick: move |_| {
                    let id = id.saturating_sub(1);
                    use_navigation().push(format!("/blog/{name_for_prev}/{id}"));
                },
                "Previous"
            }
            button {
                onclick: move |_| {
                    let id = id.saturating_add(1);
                    use_navigation().push(format!("/blog/{name_for_next}/{id}"));
                },
                "Next"
            }
        }

    }
}

#[route("/about")]
#[component]
fn About() -> Element {
    rsx! {
        h1{ "About" }
        p {
            "This is a minimal router implementation using:"
        }
        ul {
            li { "inventory crate for compile-time registration" }
            li { "Procedural macros for the #[route] attribute" }
            li { "Dioxus components and context" }
        }
        p {
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
            id: "github-link",
            href: "https://github.com/fisothemes/dioxus-fsrouter",
            img {
                style: "width: 48px; height: 48px;",
                src: "https://img.icons8.com/?size=100&id=12599&format=png&color=000000"
            }
            "GitHub"
        }
    }
}
