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
            Box::pin(f.map(ws_jsonrpc::convert))
        }
    } else {
        quote! {
            Box::pin(std::future::ready(ws_jsonrpc::convert(f)))
        }
    };

    let gen = quote! {
        #vis fn #name(mut args: Vec<serde_json::Value>) -> ws_jsonrpc::MethodReturnType {
            if #argc != args.len() {
                let msg = format!("expected {} parameters, {} given", #argc, args.len());
                let error = ws_jsonrpc::response::Error::invalid_params(Some(msg));
                return Box::pin(std::future::ready(Err(error)));
            }

            #item

            let f = #name(#(
                match serde_json::from_value(std::mem::take(&mut args[#pos])) {
                    Ok(arg) => arg,
                    Err(err) => {
                        let error = ws_jsonrpc::response::Error::invalid_params(Some(format!("{}", err)));
                        return Box::pin(std::future::ready(Err(error)));
                    }
                }
            ),*);

            #gen
        }
    };
    gen.into()
}
