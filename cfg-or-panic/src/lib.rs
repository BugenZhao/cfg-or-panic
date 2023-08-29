use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn cfg_or_panic(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let mut function = parse_macro_input!(input as ItemFn);

    let block = function.block;
    function.block = {
        let block = quote::quote! {{
            #[cfg(not(#args))]
            unimplemented!("function unimplemented under `#[cfg(not({}))]`", stringify!(#args));
            #[cfg(#args)]
            #block
        }}
        .into();
        parse_macro_input!(block as syn::Block).into()
    };

    function.into_token_stream().into()
}
