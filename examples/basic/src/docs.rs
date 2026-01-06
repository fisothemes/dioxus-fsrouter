use crate::helpers::NavButton;
use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

#[route("/docs/:..segments")]
#[component]
fn Docs(segments: Vec<String>) -> Element {
    let path_str = segments.join(" / ");
    let title = segments.last().map(|s| s.as_str()).unwrap_or("Index");

    rsx! {
        div { style: "padding: 2rem;",
            div { style: "font-family: monospace; color: #666; margin-bottom: 1rem;",
                Link { to: "/docs/intro", "Docs" }
                " / "
                "{path_str}"
            }
            h1 { "Docs: {title}" }
            p {
                "You are currently viewing the documentation for "
                code { "{path_str}" }
            }
            div { style: "margin-top: 2rem; border-top: 1px solid #eee; padding-top: 1rem;",
                h3 { "Browse Topics:" }
                ul {
                    li {
                        Link { to: "/docs/getting-started/installation", "Installation" }
                    }
                    li {
                        Link { to: "/docs/components/props", "Component Props" }
                    }
                    li {
                        Link { to: "/docs/api/v2/reference", "API Reference (Deep Link)" }
                    }
                }
            }
            div { style: "margin-top: 2rem;",
                NavButton { path: "/".to_string(), text: "Back to Home" }
            }
        }
    }
}
