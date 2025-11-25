use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;
use dioxus_logger::tracing;
pub fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[component]
pub fn NavButton(path: String, text: String) -> Element {
    let mut nav = use_navigation();

    rsx! {
        button{
            onclick: move |_| nav.push(path.clone()),
            "{text}"
        }
    }
}

pub fn print_all_routes() {
    let routes = get_routes();

    tracing::info!("Route Registry ({} routes registered)", routes.len());
    tracing::info!(
        "================================================================================"
    );
    tracing::info!(
        "{:<30} | {:<10} | {}",
        "Path Pattern",
        "Priority",
        "Component"
    );
    tracing::info!(
        "-------------------------------+------------+-----------------------------------"
    );

    for route in routes {
        let path = route.path();
        let priority = route.priority();
        let component = route.component_name();

        tracing::info!("{path:<30} | {priority:<10} | {component}");
    }

    tracing::info!(
        "================================================================================\n"
    );
}
