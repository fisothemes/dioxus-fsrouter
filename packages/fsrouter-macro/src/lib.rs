use indexmap::IndexSet as Set;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, LitStr, Pat, Token, Type, parse};

/// Mark a component as a route
///
/// # Syntax
/// ```ignore
/// #[route("/path", redirect = ["/alias1", "/alias2"])]
/// ```
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
///
/// Query parameters:
/// ```ignore
/// #[route("/search?query&page")]
/// #[component]
/// fn Search(query: String, page: Option<u32>) -> Element { ... }
/// ```
///
/// Routes with Redirects (Aliases):
/// ```ignore
/// #[route("/user/:id", redirect = ["/u/:id", "/profile/:id"])]
/// #[component]
/// fn User(id: String) -> Element { ... }
/// ```
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_impl(attr, item).unwrap_or_else(|e| e.into_compile_error().into())
}

/// Struct to hold arguments for `#[route(...)]`
struct RouteArgs {
    path: LitStr,
    redirects: Vec<LitStr>,
}

impl syn::parse::Parse for RouteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        let mut redirects = Vec::new();

        // Check for optional redirect parameter
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            // Parse: redirect = [...]
            let ident: syn::Ident = input.parse()?;
            if ident != "redirect" {
                return Err(syn::Error::new_spanned(
                    ident,
                    "Expected 'redirect' parameter",
                ));
            }

            input.parse::<Token![=]>()?;

            // Parse array of redirect paths
            let content;
            syn::bracketed!(content in input);

            loop {
                if content.is_empty() {
                    break;
                }
                redirects.push(content.parse()?);

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        Ok(RouteArgs { path, redirects })
    }
}

fn route_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = parse::<RouteArgs>(attr)?;
    let func = parse::<ItemFn>(item)?;

    let path = args.path;
    let redirects = args.redirects;

    let full_path_str = path.value();
    let (route_params, contains_catch_all) = parse_and_validate_route(&path, &full_path_str)?;

    let func_args = extract_fn_args(&func);

    validate_consistency(&func, &full_path_str, &route_params, &func_args)?;

    for redirect_path in &redirects {
        let (r_params, _) = parse_and_validate_route(redirect_path, &redirect_path.value())?;
        validate_consistency(&func, &redirect_path.value(), &r_params, &func_args)?;
    }

    generate_code(
        &func,
        &full_path_str,
        &redirects,
        &route_params,
        &func_args,
        contains_catch_all,
    )
}

fn validate_param_name(ctx: &LitStr, name: &str, label: &str) -> syn::Result<()> {
    if name.is_empty() {
        return Err(syn::Error::new_spanned(
            ctx,
            format!("{label} routes must have a non-empty name."),
        ));
    }
    if let Some(c) = name.chars().next()
        && c.is_numeric()
    {
        return Err(syn::Error::new_spanned(
            ctx,
            format!("{label} names must not start with a numeric character."),
        ));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(syn::Error::new_spanned(
            ctx,
            format!("{label} names must only contain alphanumeric characters or underscores."),
        ));
    }
    Ok(())
}

fn parse_and_validate_route(
    path_lit: &LitStr,
    full_path_str: &str,
) -> syn::Result<(Set<String>, bool)> {
    let (path_str, query_str) = full_path_str
        .split_once('?')
        .map(|(p, q)| (p, Some(q)))
        .unwrap_or((full_path_str, None));

    // Basic path validations
    if !path_str.starts_with('/') {
        return Err(syn::Error::new_spanned(
            path_lit,
            format!("Route path must start with '/'\nGot: '{path_str}'\nExpected: '/{path_str}'"),
        ));
    }

    if path_str != "/" && path_str.ends_with('/') {
        return Err(syn::Error::new_spanned(
            path_lit,
            format!("Route path should not have a trailing slash\nGot: '{path_str}'"),
        ));
    }

    if path_str.contains("//") {
        return Err(syn::Error::new_spanned(
            path_lit,
            format!("Route path contains double slashes '//'\nGot: '{path_str}'"),
        ));
    }

    let mut route_params = Set::new();
    let mut contains_catch_all = false;

    // Segment validation
    for segment in path_str.split('/').filter(|s| !s.is_empty()) {
        if contains_catch_all {
            return Err(syn::Error::new_spanned(
                path_lit,
                "Catch-all parameter (e.g. ':..segments') must be the last parameter",
            ));
        }

        if segment.contains(' ') {
            return Err(syn::Error::new_spanned(
                path_lit,
                "Route path should not contain whitespace",
            ));
        }

        if segment.contains('*') {
            return Err(syn::Error::new_spanned(
                path_lit,
                "Wildcard routes ('*') are not supported, use catch-all routes ('/:..segments') instead.",
            ));
        }

        if segment.contains('?') {
            return Err(syn::Error::new_spanned(
                path_lit,
                "Query parameters in path (e.g. '/blog?:name') are not supported here.",
            ));
        }

        // Handle parameters
        if let Some(rest) = segment.strip_prefix(":..") {
            validate_param_name(path_lit, rest, "Catch-all")?;
            if !route_params.insert(rest.to_string()) {
                return Err(syn::Error::new_spanned(path_lit, "Duplicate parameter"));
            }
            contains_catch_all = true;
        } else if let Some(param) = segment.strip_prefix(':') {
            validate_param_name(path_lit, param, "Parameter")?;
            if !route_params.insert(param.to_string()) {
                return Err(syn::Error::new_spanned(path_lit, "Duplicate parameter"));
            }
        }
    }

    // Query param validation
    if let Some(query) = query_str {
        for segment in query.split('&') {
            if segment.is_empty() {
                continue;
            }
            let name = segment.trim();

            if name.contains(':') || name.contains('?') {
                return Err(syn::Error::new_spanned(
                    path_lit,
                    "Query parameters must not contain ':' or '?'",
                ));
            }

            if !route_params.insert(name.to_string()) {
                return Err(syn::Error::new_spanned(
                    path_lit,
                    format!("Duplicate query parameter '{}'", name),
                ));
            }
        }
    }

    Ok((route_params, contains_catch_all))
}

fn extract_fn_args(func: &ItemFn) -> Vec<(syn::Ident, &Type)> {
    let mut args = Vec::new();
    for arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            args.push((pat_ident.ident.clone(), &(*pat_type.ty)));
        }
    }
    args
}

fn validate_consistency(
    func: &ItemFn,
    path_str: &str,
    route_params: &Set<String>,
    func_args: &[(syn::Ident, &Type)],
) -> syn::Result<()> {
    let func_name = &func.sig.ident;

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

    for param in route_params {
        if !func_args.iter().any(|(ident, _)| ident == param) {
            return Err(syn::Error::new_spanned(
                &func.sig.inputs,
                format!("Route parameter '{param}' not found in component arguments."),
            ));
        }
    }
    Ok(())
}

fn generate_param_parsing(
    func_args: &[(syn::Ident, &Type)],
    route_params: &Set<String>,
    full_path_str: &str,
    contains_catch_all: bool,
) -> Vec<proc_macro2::TokenStream> {
    func_args.iter().map(|(ident, ty)| {
        let param_name = ident.to_string();

        if contains_catch_all
            && let Some(catch_all) = route_params.last()
            && catch_all == &param_name
        {
            quote! {
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
                                    #full_path_str
                                ),
                            _ => e
                        })?
                };
            }
        } else if let Some(inner_ty) = get_option_inner(ty) {
            quote! {
                let #ident = match params.get(#param_name) {
                    Some(val) => Some(val.parse::<#inner_ty>()
                        .map_err(|_| ::dioxus_fsrouter::errors::ParseError::invalid_type(
                            #param_name,
                            stringify!(#inner_ty),
                            val.clone(),
                            #full_path_str
                        ))?),
                    None => None,
                };
            }
        } else {
            quote! {
                let #ident = {
                    let param_value = params
                        .get(#param_name)
                        .ok_or_else(|| ::dioxus_fsrouter::errors::ParseError::missing(#param_name, #full_path_str))?;

                    param_value.parse::<#ty>()
                        .map_err(|_| {
                            ::dioxus_fsrouter::errors::ParseError::invalid_type(
                                #param_name,
                                stringify!(#ty),
                                param_value.clone(),
                                #full_path_str
                            )
                        })?
                };
            }
        }
    }).collect()
}

fn generate_code(
    func: &ItemFn,
    full_path_str: &str,
    redirects: &[LitStr],
    route_params: &Set<String>,
    func_args: &[(syn::Ident, &Type)],
    contains_catch_all: bool,
) -> syn::Result<TokenStream> {
    let func_ident = &func.sig.ident;
    let func_name_str = func_ident.to_string();

    let render_func_name = format_ident!("__render_{}", func_name_str);
    let pattern_static_name = format_ident!("__PATTERN_{}", func_name_str);
    let props_struct_name = format_ident!("{}Props", func_name_str);

    // Generate wrapper
    let (wrapper_func, render_fn_variant) = if route_params.is_empty() {
        (
            quote! {
                #[allow(non_snake_case)]
                fn #render_func_name() -> ::dioxus::prelude::Element {
                    #func_ident()
                }
            },
            quote! { ::dioxus_fsrouter::RenderFn::Static(#render_func_name) },
        )
    } else {
        let param_parsing_logic =
            generate_param_parsing(func_args, route_params, full_path_str, contains_catch_all);

        let props_fields = func_args.iter().map(|(ident, _)| quote! { #ident });

        (
            quote! {
                #[allow(non_snake_case)]
                fn #render_func_name(
                    params: std::collections::HashMap<String, String>
                ) -> ::dioxus_fsrouter::errors::ParseResult<::dioxus::prelude::Element> {
                    #(#param_parsing_logic)*
                    Ok(#func_ident( #props_struct_name { #(#props_fields),* }))
                }
            },
            quote! { ::dioxus_fsrouter::RenderFn::WithParams(#render_func_name) },
        )
    };

    // Generate Main Route Submission
    let main_route_submit = quote! {
        #[allow(non_upper_case_globals)]
        static #pattern_static_name: ::std::sync::OnceLock<
            Result<::dioxus_fsrouter::route::RoutePattern, ::dioxus_fsrouter::errors::ParseError>
        > = ::std::sync::OnceLock::new();

        ::dioxus_fsrouter::inventory::submit! {
            ::dioxus_fsrouter::RouteInfo::new(
                #full_path_str,
                &#pattern_static_name,
                concat!(module_path!(), "::", stringify!(#func_ident)),
                #render_fn_variant,
                None,
            )
        }
    };

    // Generate Redirect Submissions
    let redirect_submits = redirects.iter().enumerate().map(|(i, r_path)| {
        let r_path_str = r_path.value();
        let redirect_pattern_name = format_ident!("__PATTERN_{}_REDIRECT_{}", func_name_str, i);

        quote! {
            #[allow(non_upper_case_globals)]
            static #redirect_pattern_name: ::std::sync::OnceLock<
                Result<::dioxus_fsrouter::route::RoutePattern, ::dioxus_fsrouter::errors::ParseError>
            > = ::std::sync::OnceLock::new();

            ::dioxus_fsrouter::inventory::submit! {
                ::dioxus_fsrouter::RouteInfo::new(
                    #r_path_str,
                    &#redirect_pattern_name,
                    concat!(module_path!(), "::", stringify!(#func_ident)),
                    #render_fn_variant,
                    Some(#full_path_str),
                )
            }
        }
    });

    let item_ts: proc_macro2::TokenStream = quote! { #func };

    Ok(quote! {
        #item_ts
        #wrapper_func
        #main_route_submit
        #(#redirect_submits)*
    }
    .into())
}

/// Helper to detect if a type is Option<T> and return T
fn get_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}
