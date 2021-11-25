use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let name = &item.sig.ident;
    let vis = &item.vis;
    let argc = item.sig.inputs.len();
    let pos = 0..argc;

    let gen = if item.sig.asyncness.is_some() {
        quote! {
            use futures::future::FutureExt;
            Box::pin(f.map(jsonrpc::convert))
        }
    } else {
        quote! {
            Box::pin(ready(jsonrpc::convert(f)))
        }
    };

    let gen = quote! {
        #vis fn #name(mut args: Vec<serde_json::Value>) -> jsonrpc::MethodReturnType {
            use std::future::ready;
            use jsonrpc::response::Error;
            use serde_json::from_value;
            use std::mem::take;

            if #argc != args.len() {
                let msg = format!("expected {} parameters, {} given", #argc, args.len());
                return Box::pin(ready(Err(Error::invalid_params(Some(msg)))));
            }

            #item

            let f = #name(#(
                match from_value(take(&mut args[#pos])) {
                    Ok(arg) => arg,
                    Err(err) => return Box::pin(ready(Err(Error::invalid_params(Some(format!("{}", err)))))),
                }
            ),*);

            #gen
        }
    };
    gen.into()
}
