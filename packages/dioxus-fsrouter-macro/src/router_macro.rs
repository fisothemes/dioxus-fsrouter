use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

pub fn router_impl(input: TokenStream) -> Result<TokenStream> {
    // Phase 1: Just wrap input in Router component
    // The input should be RSX content

    Ok(quote! {
        {
            ::dioxus::prelude::rsx! {
                ::dioxus_fsrouter::Router {
                    #input
                }
            }
        }
    })
}
