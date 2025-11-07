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
        "/blog" => rsx! { BlogList {} },
        "/blog/:id" => rsx! { BlogPost {} },
        "/user/:username" => rsx! { UserProfile {} },
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
            Link { to: "/blog".to_string(),
                span { style: "margin-right: 1rem;", "Blog" }
            }
            Link { to: "/blog/hello-world".to_string(),
                span { style: "margin-right: 1rem;", "Sample Post" }
            }
            Link { to: "/user/john".to_string(),
                span { "User Profile" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "Home" }
            p { "Click the links above to see route parameters in action!" }
        }
    }
}

#[component]
fn BlogList() -> Element {
    rsx! {
        div {
            h1 { "All Blog Posts" }
            ul {
                li { Link { to: "/blog/first-post".to_string(), "First Post" } }
                li { Link { to: "/blog/second-post".to_string(), "Second Post" } }
                li { Link { to: "/blog/third-post".to_string(), "Third Post" } }
            }
        }
    }
}

#[component]
fn BlogPost() -> Element {
    let params = use_params();
    let post_id = params.get("id").cloned().unwrap_or_default();

    rsx! {
        div {
            h1 { "Blog Post" }
            p { "Viewing post: {post_id}" }
            Link { to: "/blog".to_string(), "← Back to all posts" }
        }
    }
}

#[component]
fn UserProfile() -> Element {
    let params = use_params();
    let username = params.get("username").cloned().unwrap_or_default();

    rsx! {
        div {
            h1 { "User Profile" }
            p { "Username: @{username}" }
            Link { to: "/".to_string(), "← Back home" }
        }
    }
}