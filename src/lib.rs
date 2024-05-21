use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Attribute, Block, Error, Expr, ExprLit, ImplItem, ImplItemFn,
    Item, ItemFn, ItemImpl, ItemMod, Lit, Meta, Result, Signature, Type,
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
///
/// ## Dummy return type
/// For the functions returning an `impl Trait`, you may have to specify a dummy return type for the panic branch.
/// This can be done by adding `#[panic_return = "dummy::return::Type"]` to the function.
/// ```should_panic
/// # use cfg_or_panic::cfg_or_panic;
/// #[cfg_or_panic(foo)]
/// #[panic_return = "std::iter::Empty<_>"]
/// fn my_iter() -> impl Iterator<Item = i32> {
///   (0..10).into_iter()
/// }
/// # fn main() { my_iter().count(); }
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
            _ => Err(Error::new_spanned(
                item,
                "`#[cfg_or_panic]` can only be used on functions, `mod`, and `impl` blocks",
            )),
        }
    }

    fn expand_mod(&self, item_mod: &mut ItemMod) -> Result<()> {
        let Some((_, content)) = &mut item_mod.content else {
            return Ok(());
        };

        for item in content {
            self.expand_item(item).ok();
        }

        Ok(())
    }

    fn expand_impl(&self, item_impl: &mut ItemImpl) -> Result<()> {
        for item in &mut item_impl.items {
            #[allow(clippy::single_match)]
            match item {
                ImplItem::Fn(impl_item_fn) => self.expand_impl_fn(impl_item_fn)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn expand_fn(&self, f: &mut ItemFn) -> Result<()> {
        self.expand_fn_inner(&f.sig, &mut f.block, &mut f.attrs)
    }

    fn expand_impl_fn(&self, f: &mut ImplItemFn) -> Result<()> {
        self.expand_fn_inner(&f.sig, &mut f.block, &mut f.attrs)
    }

    fn expand_fn_inner(
        &self,
        sig: &Signature,
        fn_block: &mut Block,
        fn_attrs: &mut Vec<Attribute>,
    ) -> Result<()> {
        let name = &sig.ident;
        let args = &self.args;

        let return_ty = {
            let mut return_ty = None;
            let mut new_fn_attrs = Vec::new();

            // TODO: use `extract_if` when stable
            for fn_attr in fn_attrs.drain(..) {
                if let Some(ty) = extract_panic_return_attr(&fn_attr) {
                    return_ty = Some(ty?);
                } else {
                    new_fn_attrs.push(fn_attr);
                }
            }

            *fn_attrs = new_fn_attrs;
            return_ty
        };

        let msg = format!(
            "function `{}` unimplemented unless `#[cfg({})]` is activated",
            name, args
        );
        let unimplemented = quote!(
            panic!(#msg);
        );

        let may_with_ret_ty = if let Some(ty) = return_ty {
            quote!(
                #[allow(unreachable_code, clippy::diverging_sub_expression)]
                {
                    let __ret: #ty = #unimplemented;
                    return __ret;
                }
            )
        } else {
            unimplemented
        };

        let block = std::mem::replace(fn_block, parse_quote!({}));
        *fn_block = parse_quote!({
            #[cfg(not(#args))]
            #may_with_ret_ty
            #[cfg(#args)]
            #block
        });

        let attr = parse_quote!(
            #[cfg_attr(not(#args), allow(unused_variables))]
        );
        fn_attrs.push(attr);

        Ok(())
    }
}

fn extract_panic_return_attr(attr: &Attribute) -> Option<Result<Type>> {
    let Meta::NameValue(name_value) = &attr.meta else {
        return None;
    };
    if name_value.path.get_ident()? != "panic_return" {
        return None;
    }

    Some(parse_panic_return_attr(name_value.value.clone()))
}

fn parse_panic_return_attr(value_expr: Expr) -> Result<Type> {
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value_expr
    else {
        return Err(Error::new_spanned(value_expr, "expected a string literal"));
    };

    syn::parse_str(&lit_str.value())
}
