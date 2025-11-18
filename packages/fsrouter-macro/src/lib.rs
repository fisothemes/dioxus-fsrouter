use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, parse};

/// Mark a component as a route
///
/// # Example
/// ```ignore
/// #[route("/")]
/// #[component]
/// fn Home() -> Element {
///     rsx! { div { "Home" } }
/// }
/// ```
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_impl(attr, item).unwrap_or_else(|e| e.into_compile_error().into())
}

fn route_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let path = parse::<LitStr>(attr)?;
    let func = parse::<ItemFn>(item.clone())?;

    let path_str = path.value();
    let func_name = func.sig.ident.to_string();

    // Validation: a path must start with '/'
    if !path_str.starts_with('/') {
        return Err(syn::Error::new_spanned(
            path,
            "route path must start with '/'",
        ));
    }

    // Validation: no route parameters yet (Phase 1)
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            func.sig.inputs,
            "route parameters not yet supported in Phase 1",
        ));
    }

    let func_ident = &func.sig.ident;
    let render_fn_name = format_ident!("__render_{}", func_name);

    let item: proc_macro2::TokenStream = item.into();

    Ok(quote! {
        #item

        // Generate a wrapper render function
        #[allow(non_snake_case)]
        fn #render_fn_name() -> ::dioxus::prelude::Element {
            #func_ident()
        }

        // Submit this route to the global inventory
        ::dioxus_fsrouter::inventory::submit! {
            ::dioxus_fsrouter::RouteInfo::new(
                #path_str,
                concat!(module_path!(), "::", stringify!(#func_ident)),
                #render_fn_name
            )
        }
    }
    .into())
}
