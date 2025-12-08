use indexmap::IndexSet as Set;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, Pat, parse};

/// Mark a component as a route
///
/// # Example
///
/// Static routes:
/// ```ignore
/// #[route("/")]
/// #[component]
/// fn Home() -> Element { ... }
/// ```
///
/// Dynamic routes:
/// ```ignore
/// #[route("/user/:id")]
/// #[component]
/// fn User(id: u32) -> Element { ... }
/// ```
///
/// Catch-all routes:
/// ```ignore
/// #[route("/files/:..path")]
/// #[component]
/// fn Files(path: Vec<String>) -> Element { ... }
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
    let func_ident = &func.sig.ident;

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

    let mut route_params = Set::new();

    let mut contains_catch_all = false;

    for segment in path_str.split('/').filter(|s| !s.is_empty()) {
        // Validation: catch-all must be last parameter
        if contains_catch_all {
            return Err(syn::Error::new_spanned(
                path,
                "Catch-all parameter (e.g. ':..segments') must be the last parameter in the route path",
            ));
        }

        // Validation: no whitespace in a path
        if segment.contains(' ') {
            return Err(syn::Error::new_spanned(
                path,
                format!(
                    "Route path should not contain whitespace\n\
                    Got: '{path_str}'\n\
                    Use: '{}'\n\
                    \n\
                    Whitespace is not allowed in route paths.",
                    path_str.replace(" ", "%20")
                ),
            ));
        }

        // Validation: no wildcards
        if segment.contains('*') {
            return Err(syn::Error::new_spanned(
                path,
                "Wildcard routes ('*') are not supported, use catch-all routes ('/:..segments') instead.",
            ));
        }

        // Validation: no query parameters
        if segment.contains('?') {
            return Err(syn::Error::new_spanned(
                path,
                "Query parameters (e.g. '/blog?:name&:surname') are not yet supported.",
            ));
        }

        if let Some(rest) = segment.strip_prefix(":..") {
            // Validation: catch-all must have a name
            if rest.is_empty() {
                return Err(syn::Error::new_spanned(
                    path,
                    "Catch-all routes (e.g. '/blog/:..segments') must have a non-empty name.",
                ));
            }

            // Validation: catch-all names must not start with a numeric character
            if let Some(c) = rest.chars().next()
                && c.is_numeric()
            {
                return Err(syn::Error::new_spanned(
                    path,
                    "Catch-all parameter names must not start with a numeric character.",
                ));
            }

            // Validation: catch-all names must only contain alphanumeric characters or underscores
            if !rest.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(syn::Error::new_spanned(
                    path,
                    "Catch-all parameter names must only contain alphanumeric characters or underscores.",
                ));
            }

            // Validation: parameters must be unique (e.g. "/user/:id" and "/user/:name" are not allowed)
            if !route_params.insert(rest) {
                return Err(syn::Error::new_spanned(
                    path,
                    format!(
                        "Duplicate parameter '{rest}' in route '{path_str}'.\n\
                            Parameters must be unique.",
                    ),
                ));
            }
            contains_catch_all = true;
        } else if let Some(param) = segment.strip_prefix(':') {
            // Validation: no empty parameters (e.g. "/user/:" or "/file/:/edit")
            if param.is_empty() {
                return Err(syn::Error::new_spanned(
                    path,
                    "Empty parameters (e.g. '/user/:' or '/file/:/edit') are not allowed in route paths.",
                ));
            }

            // Validation: parameter names must not start with a numeric character
            if let Some(c) = param.chars().next()
                && c.is_numeric()
            {
                return Err(syn::Error::new_spanned(
                    path,
                    "Parameter names must not start with a numeric character.",
                ));
            }

            // Validation: parameter names must only contain alphanumeric characters or underscores
            if !param.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(syn::Error::new_spanned(
                    path,
                    "Parameter names must only contain alphanumeric characters or underscores.",
                ));
            }

            // Validation: parameters must be unique (e.g. "/user/:id" and "/user/:name" are not allowed)
            if !route_params.insert(param) {
                return Err(syn::Error::new_spanned(
                    path,
                    format!(
                        "Duplicate parameter '{param}' in route '{path_str}'.\n\
                            Parameters must be unique.",
                    ),
                ));
            }
        }
    }

    // Map function arguments to their types
    let mut func_args = Vec::new();
    for arg in func.sig.inputs.iter() {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            func_args.push((pat_ident.ident.clone(), &pat_type.ty));
        }
    }

    // Validate that route parameters match function arguments
    if route_params.len() != func_args.len() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            format!(
                "Route parameters mismatch.\n\
                Route '{path_str}' has {} parameters: {route_params:?}\n\
                Component '{func_name}' has {} arguments: {:?}\n\
                They must match exactly.",
                route_params.len(),
                func_args.len(),
                func_args
                    .iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<_>>()
            ),
        ));
    }

    // Ensure every route parameter exists in the function arguments
    for param in &route_params {
        if !func_args.iter().any(|(ident, _)| ident == param) {
            return Err(syn::Error::new_spanned(
                &func.sig.inputs,
                format!(
                    "Route parameter '{param}' not found in component arguments.\n\
                    Ensure component has an argument named '{param}'.",
                ),
            ));
        }
    }

    let render_func_name = format_ident!("__render_{}", func_name);
    let pattern_static_name = format_ident!("__PATTERN_{}", func_name);

    // Dioxus components typically generate a Props struct named `{ComponentName}Props`
    let props_struct_name = format_ident!("{}Props", func_name);

    // Generate the wrapper function based on whether the route has parameters
    let wrapper_func = if route_params.is_empty() {
        quote! {
            // Static wrapper function
            #[allow(non_snake_case)]
            fn #render_func_name() -> ::dioxus::prelude::Element {
                #func_ident()
            }
        }
    } else {
        // Generate the parsing logic for each argument
        let param_parsing_logic = func_args.iter().map(|(ident, ty)| {
            let param_name = ident.to_string();

            if contains_catch_all {
                return quote!{
                    let #ident = {
                        let raw_segment = params
                            .get(#param_name)
                            .map(|s| s.as_str()).unwrap_or_default();

                        ::dioxus_fsrouter::route::TryFromRouteSegments::try_from_route_segments(raw_segment)
                            .map_err(|e| match e {
                                ::dioxus_fsrouter::errors::ParseError::InvalidType { value, expected_type, .. } =>
                                    ::dioxus_fsrouter::errors::ParseError::invalid_type(
                                        #param_name,
                                        expected_type,
                                        value,
                                        #path_str
                                    ),
                                _ => e
                            })?
                    };
                }
            } else {
                quote! {
                    let #ident = {
                        let param_value = params
                            .get(#param_name)
                            .ok_or_else(|| ::dioxus_fsrouter::errors::ParseError::missing(#param_name, #path_str))?;

                        param_value.parse::<#ty>()
                            .map_err(|_| {
                                ::dioxus_fsrouter::errors::ParseError::invalid_type(
                                    #param_name,
                                    stringify!(#ty),
                                    param_value.clone(),
                                    #path_str
                                )
                            })?
                    };
                }
            }
        });

        // Generate the field assignments for the Props struct
        let props_fields = func_args.iter().map(|(ident, _)| {
            quote! { #ident }
        });

        quote! {
            // Dynamic wrapper function
            #[allow(non_snake_case)]
            fn #render_func_name(
                params: std::collections::HashMap<String, String>
            ) -> ::dioxus_fsrouter::errors::ParseResult<::dioxus::prelude::Element> {

                // Parse all parameters
                #(#param_parsing_logic)*

                // Call the component with the parsed props
                Ok(#func_ident( #props_struct_name {
                    #(#props_fields),*
                }))
            }
        }
    };

    // Determine the correct enum variant for the inventory submission
    let render_fn_variant = if route_params.is_empty() {
        quote! { ::dioxus_fsrouter::RenderFn::Static(#render_func_name) }
    } else {
        quote! { ::dioxus_fsrouter::RenderFn::WithParams(#render_func_name) }
    };

    let item: proc_macro2::TokenStream = item.into();

    Ok(quote! {
        #item

        // Generate the wrapper function (Static or WithParams)
        #wrapper_func

        // Generate a static lock for the route pattern to ensure lifetime safety
        #[allow(non_upper_case_globals)]
        static #pattern_static_name: ::std::sync::OnceLock<
            Result<::dioxus_fsrouter::route::RoutePattern, ::dioxus_fsrouter::errors::ParseError>
        > = ::std::sync::OnceLock::new();

        // Submit the route to the global inventory
        ::dioxus_fsrouter::inventory::submit! {
            ::dioxus_fsrouter::RouteInfo::new(
                #path_str,
                &#pattern_static_name,
                concat!(module_path!(), "::", stringify!(#func_ident)),
                #render_fn_variant
            )
        }
    }
    .into())
}
