use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

#[component]
pub fn SearchBox() -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut nav = use_navigation();
    let mut perform_search = move || {
        let q = search_query();
        if !q.is_empty() {
            nav.push(format!("/search?q={}", q));
        }
    };

    rsx! {
        div { style: "display: flex; gap: 0.5rem;",
            input {
                placeholder: "Search...",
                value: "{search_query}",
                oninput: move |e| search_query.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        perform_search();
                    }
                },
            }

            button { onclick: move |_| perform_search(), "Search" }
        }
    }
}

#[route("/search?q")]
#[component]
fn Search(q: String) -> Element {
    rsx! {
        div {
            h1 { "Search Results" }
            p {
                "You searched for: "
                strong { "\"{q}\"" }
            }
            hr {}
            ul {
                li {
                    Link { to: "/docs/results/{q}", "Documentation: {q}" }
                }
                li {
                    Link { to: "/blog/article/{q}", "Blog post for {q}" }
                }
            }
        }
    }
}
