use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    Data, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Generics, Index, Lifetime,
};

pub fn into_lua_impl(derive_input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = derive_input;

    let into_lua_function_impl = into_lua_func(&ident, &data);

    gen_into_lua_impl(&ident, &generics, &into_lua_function_impl)
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

pub fn into_lua_func(ident: &Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(strct) => into_lua_func_struct(ident, strct),
        Data::Enum(_) => todo!(),
        Data::Union(_) => unimplemented!(),
    }
}

pub fn into_lua_func_struct(ident: &Ident, strct: &DataStruct) -> TokenStream {
    let fields_inserts = table_fields_inserts(&strct.fields);
    let make_table = make_type_table(ident, &fields_inserts);

    quote! {
        #make_table
    }
}

pub fn make_type_table(ident: &Ident, fields_inserts: &impl ToTokens) -> TokenStream {
    let ident_string = ident.to_string();
    quote! {
        let table = lua.create_table()?;

        table.set("type", #ident_string)?;

        let values_table = #fields_inserts;
        if let Some(values_table) = values_table {
            table.set("values", values_table)?;
        }

        table.into_lua(lua)
    }
}

pub fn table_fields_inserts(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => table_named_fields_inserts(named),
        Fields::Unnamed(unnamed) => table_unnamed_fields_inserts(unnamed),
        Fields::Unit => quote! { None },
    }
}

pub fn table_unnamed_fields_inserts(unnamed: &FieldsUnnamed) -> TokenStream {
    let field_pushes = unnamed.unnamed.iter().enumerate().map(|(i, _)| {
        let index = Index {
            index: i as u32,
            span: Span::call_site(),
        };
        quote! {
            values.push(self.#index)?;
        }
    });

    fields_value_table(field_pushes)
}

pub fn table_named_fields_inserts(named: &FieldsNamed) -> TokenStream {
    let field_pushes = named.named.iter().map(|f| {
        let ident = f.ident.as_ref().expect("Named field found without ident");
        let ident_string = ident.to_string();
        quote! {
            values.set(#ident_string, self.#ident)?;
        }
    });

    fields_value_table(field_pushes)
}

pub fn fields_value_table<I, T>(field_pushes: I) -> TokenStream
where
    T: ToTokens,
    I: Iterator<Item = T>,
{
    quote! {
        {
            let values = lua.create_table()?;

            #(#field_pushes);*

            Some(values)
        }
    }
}
