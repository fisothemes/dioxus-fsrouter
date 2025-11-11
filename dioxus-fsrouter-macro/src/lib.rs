use proc_macro::TokenStream;

mod route;
mod router_macro;

/// Define a route on a component
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::route_impl(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Set up router context
#[proc_macro]
pub fn router(input: TokenStream) -> TokenStream {
    router_macro::router_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
