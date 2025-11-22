use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse;

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
    let path = parse::<syn::LitStr>(attr)?;
    let func = parse::<syn::ItemFn>(item.clone())?;

    let path_str = path.value();
    let func_name = func.sig.ident.to_string();

    // Validation: a path must start with '/'
    if !path_str.starts_with('/') {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route path must start with '/'\n\
                    Got: '{path_str}'\n\
                    Expected: '/{path_str}'\n\
                \n\
                Example: #[route(\"/about\")]"
            ),
        ));
    }

    // Validation: no trailing slashes (except for root)
    if path_str != "/" && path_str.ends_with('/') {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route path should not have a trailing slash\n\
                    Got: '{path_str}'\n\
                    Use: '{0}'\n\
                \n\
                Trailing slashes can cause routing issues.",
                path_str.trim_end_matches('/')
            ),
        ));
    }

    // Validation: no double slashes
    if path_str.contains("//") {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route path contains double slashes '//'\n\
                    Got: '{path_str}'\n\
                \n\
                Double slashes are not allowed in route paths."
            ),
        ));
    }

    // Validation: no whitespace in a path
    if path_str.contains(' ') {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route path should not contain whitespace\n\
                    Got: '{path_str}'\n\
                \n\
                Whitespace is not allowed in route paths."
            ),
        ));
    }

    // Validation: no route parameters yet (Phase 1)
    if path_str.contains(':') {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route parameters are not yet supported in Phase 1\n\
                    Got: '{path_str}'\n\
                \n\
                Route parameters like '/user/:id' will be supported in Phase 2."
            ),
        ));
    }

    // Validation: no query parameters in a path
    if path_str.contains('?') {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "Route path should not contain query parameters\n\
                    Got: '{path_str}'\n\
                \n\
                Query parameters are handled separately and should not be in the route path.",
            ),
        ));
    }

    // Validation: no route parameters yet (Phase 1)
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            func.sig.inputs,
            format!(
                "Route '{path_str}' cannot have parameters in Phase 1\n\
                \n\
                Route parameters will be supported in Phase 2.\n\
                For now, routes must be parameter-free components.",
            ),
        ));
    }

    let func_ident = &func.sig.ident;
    let render_fn_name = format_ident!("__render_{}", func_name);
    let pattern_static_name = format_ident!("__PATTERN_{}", func_name);

    let item: proc_macro2::TokenStream = item.into();

    Ok(quote! {
        #item

        // Generate a wrapper render function
        #[allow(non_snake_case)]
        fn #render_fn_name() -> ::dioxus::prelude::Element {
            #func_ident()
        }

        // Generate named static for the pattern lock
        // This avoids the "borrow of interior mutable temporary" error
        #[allow(non_upper_case_globals)]
        static #pattern_static_name: ::std::sync::OnceLock<::dioxus_fsrouter::route::RoutePattern>
            = ::std::sync::OnceLock::new();

        // Submit this route to the global inventory
        ::dioxus_fsrouter::inventory::submit! {
            ::dioxus_fsrouter::RouteInfo::new(
                #path_str,
                &#pattern_static_name,
                concat!(module_path!(), "::", stringify!(#func_ident)),
                ::dioxus_fsrouter::RenderFn::Static(#render_fn_name)
            )
        }
    }
    .into())
}
