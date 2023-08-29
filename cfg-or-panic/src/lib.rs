use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, Attribute, Block, Item, ItemFn, ItemImpl};

#[proc_macro_attribute]
pub fn cfg_or_panic(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();

    let mut item = parse_macro_input!(input as Item);

    match &mut item {
        Item::Fn(item_fn) => expand_fn(&args, item_fn),
        Item::Impl(item_impl) => expand_impl(&args, item_impl),
        Item::Mod(_item_mod) => todo!(),
        _ => todo!(),
    }

    item.into_token_stream().into()
}

fn expand_impl(args: &TokenStream2, item_impl: &mut ItemImpl) {
    for item in &mut item_impl.items {
        match item {
            syn::ImplItem::Fn(impl_item_fn) => expand_impl_fn(&args, impl_item_fn),
            _ => {}
        }
    }
}

fn expand_fn(args: &TokenStream2, item_fn: &mut ItemFn) {
    expand_fn_inner(args, &mut item_fn.block, &mut item_fn.attrs);
}

fn expand_impl_fn(args: &TokenStream2, item_fn: &mut syn::ImplItemFn) {
    expand_fn_inner(args, &mut item_fn.block, &mut item_fn.attrs);
}

fn expand_fn_inner(args: &TokenStream2, fn_block: &mut Block, fn_attrs: &mut Vec<Attribute>) {
    let block = fn_block.clone();
    *fn_block = parse_quote! {
        {
            #[cfg(not(#args))]
            unimplemented!("function unimplemented under `#[cfg(not({}))]`", stringify!(#args));
            #[cfg(#args)]
            #block
        }
    };

    let attr = parse_quote! {
        #[cfg_attr(not(#args), allow(unused_variables))]
    };
    fn_attrs.push(attr);
}
