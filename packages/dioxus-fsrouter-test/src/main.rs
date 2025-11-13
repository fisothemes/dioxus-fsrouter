use dioxus::prelude::*;
use dioxus_fsrouter::prelude::*;

fn main() {
    dioxus::launch(App);
}


#[component]
fn App() -> Element{
    rsx!{
        div { "Hello world!" }
        Test { }
    }
}

#[route("/test")]
fn Test() -> Element {
    rsx!{
        "Test!"
    }
}
