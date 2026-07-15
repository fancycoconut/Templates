use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, ItemFn, LitStr, Token};

struct RouteArgs {
    method: Ident,
    path: LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;
        Ok(RouteArgs { method, path })
    }
}

/// `#[route(GET, "/health")]` — declares the HTTP method and path a handler serves,
/// right next to the handler itself, and registers it (via `inventory`) so `router.rs`
/// collects every route at startup instead of listing them by hand. See this crate's
/// `src/routing.rs` for the collection side.
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let RouteArgs { method, path } = parse_macro_input!(attr as RouteArgs);
    let handler_fn = parse_macro_input!(item as ItemFn);
    let handler_name = &handler_fn.sig.ident;
    let path_str = path.value();
    let method_str = method.to_string();

    let verb_fn = match method_str.as_str() {
        "GET" => format_ident!("get"),
        "POST" => format_ident!("post"),
        "PATCH" => format_ident!("patch"),
        "PUT" => format_ident!("put"),
        "DELETE" => format_ident!("delete"),
        other => {
            return syn::Error::new_spanned(
                &method,
                format!(
                    "unsupported HTTP method `{other}` in #[route(...)] — expected one of GET, POST, PATCH, PUT, DELETE"
                ),
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        #handler_fn

        ::inventory::submit! {
            crate::routing::RouteEntry {
                method: #method_str,
                path: #path_str,
                register: |router: ::axum::Router| {
                    router.route(#path_str, ::axum::routing::#verb_fn(#handler_name))
                },
            }
        }
    };

    expanded.into()
}
