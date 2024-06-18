// This file is part of BadRed.
//
// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
use quote::quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod from_lua_macro;
mod into_lua_macro;
mod type_derives;
mod script_table;

#[proc_macro_attribute]
pub fn auto_lua(args: TokenStream, item: TokenStream) -> TokenStream {
    auto_lua_impl(args, item, false)
}

#[proc_macro_attribute]
pub fn auto_lua_defaulting(args: TokenStream, item: TokenStream) -> TokenStream {
    auto_lua_impl(args, item, true)
}

#[proc_macro_attribute]
pub fn auto_script_table(_args: TokenStream, item: TokenStream) -> TokenStream {
    script_table::script_table_impl(item.into()).into()
}

fn auto_lua_impl(_args: TokenStream, item: TokenStream, has_default: bool) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    let typedef: DeriveInput = syn::parse2(item.clone()).expect("Failed to parse");
    let derives = type_derives::type_derives(&typedef);
    let from_lua_impl = from_lua_macro::from_lua_impl(&typedef, has_default);
    let into_lua_impl = into_lua_macro::into_lua_impl(&typedef);
    let name_lua_impls = type_derives::name_type_impls(&typedef);

    quote! {
        #derives
        #item

        #from_lua_impl
        #into_lua_impl

        #name_lua_impls
    }
    .into()
}
