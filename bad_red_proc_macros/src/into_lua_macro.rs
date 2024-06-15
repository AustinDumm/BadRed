use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Fields, Generics, Lifetime};

pub fn into_lua_impl(derive_input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = derive_input;

    let into_lua_function_impl = into_lua_func(
        &ident,
        &data,
    );

    gen_into_lua_impl(
        &ident,
        &generics,
        &into_lua_function_impl,
    )
}

pub fn gen_into_lua_impl(
    ident: &Ident,
    generics: &Generics,
    impl_body: &impl ToTokens,
) -> TokenStream {
    let lua_lifetime = generics
        .lifetimes()
        .next()
        .map(|l| l.lifetime.clone())
        .unwrap_or(Lifetime::new("'lua", Span::call_site()));

    quote! {
        impl<#lua_lifetime> mlua::IntoLua<#lua_lifetime> for #ident #generics {
            fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
                #impl_body
            }
        }

    }
}

pub fn into_lua_func(
    ident: &Ident,
    data: &Data,
) -> TokenStream {
    match data {
        Data::Struct(strct) => into_lua_func_struct(
            ident,
            strct,
        ),
        Data::Enum(_) => todo!(),
        Data::Union(_) => unimplemented!(),
    }
}

pub fn into_lua_func_struct(
    ident: &Ident,
    strct: &DataStruct
) -> TokenStream {
    let make_table = make_type_table(ident);
    let fields_inserts = table_fields_inserts(&strct.fields);

    quote! {
        #make_table
        #fields_inserts

        table.into_lua(lua)
    }
}

pub fn make_type_table(ident: &Ident) -> TokenStream {
    let ident_string = ident.to_string();
    quote! {
        let table = lua.create_table()?;
        table.set("type", #ident_string)?;
    }
}

pub fn table_fields_inserts(
    fields: &Fields,
) -> TokenStream {
    match fields {
        Fields::Named(_) => todo!(),
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => quote! {},
    }
}

