use quote::quote;

extern crate proc_macro;
use proc_macro::TokenStream;

mod from_lua_macro;
mod into_lua_macro;

#[proc_macro_derive(FromLua)]
pub fn derive_from_lua(item: TokenStream) -> TokenStream {
    from_lua_macro::derive_from_lua_impl(item)
}

