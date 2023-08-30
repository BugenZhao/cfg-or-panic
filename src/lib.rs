use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Attribute, Block, Error, ImplItem, ImplItemFn, Item, ItemFn,
    ItemImpl, ItemMod, Result, Signature,
};

/// Keep the function body under `#[cfg(..)]`, or replace it with `unimplemented!()` under `#[cfg(not(..))]`.
///
/// # Examples
///
/// `#[cfg_or_panic]` can be used on functions, `mod`, and `impl` blocks.
///
/// ## Function
/// ```should_panic
/// # use cfg_or_panic::cfg_or_panic;
/// #[cfg_or_panic(foo)]
/// fn foo() {
///   println!("foo");
/// }
/// # fn main() { foo(); }
/// ```
///
/// ## `mod`
/// ```should_panic
/// # use cfg_or_panic::cfg_or_panic;
/// #[cfg_or_panic(foo)]
/// mod foo {
///   pub fn foo() {
///     println!("foo");
///   }
/// }
/// # fn main() { foo::foo(); }
/// ```
///
/// ## `impl`
/// ```should_panic
/// # use cfg_or_panic::cfg_or_panic;
/// struct Foo(String);
///
/// #[cfg_or_panic(foo)]
/// impl Foo {
///   fn foo(&self) {
///     println!("foo: {}", self.0);
///   }
/// }
/// # fn main() { Foo("bar".to_owned()).foo(); }
/// ```
#[proc_macro_attribute]
pub fn cfg_or_panic(args: TokenStream, input: TokenStream) -> TokenStream {
    let expander = Expander::new(args);
    let mut item = parse_macro_input!(input as Item);

    if let Err(e) = expander.expand_item(&mut item) {
        return e.to_compile_error().into();
    }
    item.into_token_stream().into()
}

struct Expander {
    args: TokenStream2,
}

impl Expander {
    fn new(args: impl Into<TokenStream2>) -> Self {
        Self { args: args.into() }
    }

    fn expand_item(&self, item: &mut Item) -> Result<()> {
        match item {
            Item::Fn(item_fn) => self.expand_fn(item_fn),
            Item::Impl(item_impl) => self.expand_impl(item_impl),
            Item::Mod(item_mod) => self.expand_mod(item_mod),
            _ => {
                return Err(Error::new_spanned(
                    item,
                    "`#[cfg_or_panic]` can only be used on functions, `mod`, and `impl` blocks",
                ));
            }
        }

        Ok(())
    }

    fn expand_mod(&self, item_mod: &mut ItemMod) {
        let Some((_, content)) = &mut item_mod.content else {
            return;
        };

        for item in content {
            self.expand_item(item).ok();
        }
    }

    fn expand_impl(&self, item_impl: &mut ItemImpl) {
        for item in &mut item_impl.items {
            #[allow(clippy::single_match)]
            match item {
                ImplItem::Fn(impl_item_fn) => self.expand_impl_fn(impl_item_fn),
                _ => {}
            }
        }
    }

    fn expand_fn(&self, f: &mut ItemFn) {
        self.expand_fn_inner(&f.sig, &mut f.block, &mut f.attrs);
    }

    fn expand_impl_fn(&self, f: &mut ImplItemFn) {
        self.expand_fn_inner(&f.sig, &mut f.block, &mut f.attrs);
    }

    fn expand_fn_inner(
        &self,
        sig: &Signature,
        fn_block: &mut Block,
        fn_attrs: &mut Vec<Attribute>,
    ) {
        let name = &sig.ident;
        let args = &self.args;

        let unimplemented = if sig.constness.is_some() {
            // const functions do not support formatting
            quote!(
                unimplemented!();
            )
        } else {
            quote!(
                unimplemented!(
                    "function `{}` unimplemented under `#[cfg(not({}))]`",
                    stringify!(#name), stringify!(#args)
                );
            )
        };

        let block = std::mem::replace(fn_block, parse_quote!({}));
        *fn_block = parse_quote!({
            #[cfg(not(#args))]
            #unimplemented
            #[cfg(#args)]
            #block
        });

        let attr = parse_quote!(
            #[cfg_attr(not(#args), allow(unused_variables))]
        );
        fn_attrs.push(attr);
    }
}
