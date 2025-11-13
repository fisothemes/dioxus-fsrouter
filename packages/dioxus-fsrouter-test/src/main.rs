use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

fn main() {
    println!("Starting Dioxus!");
    dioxus::launch(App);
}


#[component]
fn App() -> Element{
    rsx!{
        "Test!"
    }
}

#[route("/test")]
fn Test() -> Element {
    rsx!{
        "Test!"
    }
}
