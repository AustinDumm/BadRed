use quote::quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod from_lua_macro;
mod into_lua_macro;
mod type_derives;

#[proc_macro_attribute]
pub fn auto_lua(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    let typedef: DeriveInput = syn::parse2(item.clone()).expect("Failed to parse");
    let derives = type_derives::type_derives(typedef.clone());
    let from_lua_impl = from_lua_macro::derive_from_lua_impl(typedef.clone());

    quote! {
        #derives
        #item

        #from_lua_impl
    }
    .into()
}
