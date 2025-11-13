use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{parse2, Error, ItemFn, LitStr, Result};

pub fn route_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse route path from attribute
    let path: LitStr = parse2(attr)?;
    let path_str = path.value();

    // Basic validation: must start with '/'
    if !path_str.starts_with('/') {
        return Err(Error::new_spanned(
            path,
            "Route path must start with '/'",
        ));
    }

    // Parse component function
    let func: ItemFn = parse2(item)?;
    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();

    // Check if function has parameters (for future param support)
    let has_params = !func.sig.inputs.is_empty();

    if has_params {
        // Phase 1: Error if params are used
        return Err(Error::new_spanned(
            &func.sig.inputs,
            "Route parameters not yet supported (coming in Phase 3)"
        ));
    }

    // Create unique names to avoid conflicts
    let route_type_name = format_ident!("__{}__Route", func_name);
    let render_fn_name = format_ident!("__{}__render", func_name);
    let route_info_static = format_ident!("__{}__INFO", func_name);
    let route_registry_name = format_ident!("__{}__REGISTRY", func_name);

    // Generate code
    Ok(quote! {
        // Original function (will be transformed by #[component])
        #func

        // Create a marker type for this route
        struct #route_type_name;

        // Helper function to create the render function pointer
        fn #render_fn_name(ctx: ::dioxus_fsrouter::route::RenderContext) -> ::dioxus::prelude::Element {
            // Phase 1: Ignore context, just render component
            // Future: Extract params from ctx and pass to component
            ::dioxus::prelude::rsx! { #func_name {} }
        }

        // Static route info (const-friendly)
        static #route_info_static: ::dioxus_fsrouter::route::RouteInfo =
            ::dioxus_fsrouter::route::RouteInfo::new(
                #path_str,
                #func_name_str,
                #render_fn_name,
            );

        // Implement Routable trait on the marker type
        impl ::dioxus_fsrouter::route::Routable for #route_type_name {
            const PATH: &'static str = #path_str;
            const COMPONENT_NAME: &'static str = #func_name_str;

            fn render(ctx: ::dioxus_fsrouter::route::RenderContext) -> ::dioxus::prelude::Element {
                #render_fn_name(ctx)
            }

            fn route_info() -> &'static ::dioxus_fsrouter::route::RouteInfo {
                &#route_info_static
            }
        }

        // Register in global inventory with unique name
        #[::dioxus_fsrouter::linkme::distributed_slice(::dioxus_fsrouter::route::ROUTES)]
        #[linkme(crate = ::dioxus_fsrouter::linkme)]
        static #route_registry_name: &'static ::dioxus_fsrouter::route::RouteInfo = &#route_info_static;
    })
}

