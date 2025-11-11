use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;
use linkme;

fn main() {
    launch(App);
}

fn App() -> Element {
    router! {
        nav {
            a { href: "/", "Home" }
            " | "
            a { href: "/about", "About" }
            " | "
            a { href: "/contact", "Contact" }
        }
        main {
            Outlet {}
        }
    }
}

#[route("/")]
#[component]
fn Home() -> Element {
    // Future: Can access route context
    // let ctx = use_route_context();

    rsx! {
        div {
            h1 { "Home" }
            p { "Welcome to the home page" }
        }
    }
}


#[route("/about")]
#[component]
fn About() -> Element {
    rsx! {
        div {
            h1 { "About" }
            p { "This is the about page" }
        }
    }
}

#[route("/contact")]
#[component]
fn Contact() -> Element {
    let route = use_route();

    rsx! {
        div {
            h1 { "Contact" }
            p { "Get in touch with us" }
            small { "Current route: {route}" }
        }
    }
}



