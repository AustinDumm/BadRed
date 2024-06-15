extern crate proc_macro;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::FieldsNamed;
use syn::FieldsUnnamed;
use syn::Generics;
use syn::Lifetime;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Ident};

pub fn derive_from_lua_impl(derive_input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = derive_input;

    match data {
        syn::Data::Struct(strct) => {
            from_lua_impl(ident.clone(), generics, body_from_lua_struct(ident, strct))
        }
        syn::Data::Enum(enm) => derive_from_lua_enum(ident, generics, enm),
        syn::Data::Union(_) => unimplemented!("Union not supported as a FromLua type"),
    }
}

fn body_from_lua_struct(ident: Ident, strct: DataStruct) -> TokenStream {
    match strct.fields {
        syn::Fields::Named(named) => {
            let struct_init = from_lua_impl_struct_named_fields(ident.clone(), named);
            let body = from_lua_impl_struct_type(ident, struct_init);

            quote! {
                #body
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            let struct_init = from_lua_impl_struct_unnamed_fields(ident.clone(), unnamed);
            let body = from_lua_impl_struct_type(ident, struct_init);
            quote! {
                #body
            }
        }
        syn::Fields::Unit => from_lua_impl_struct_type(ident.clone(), from_lua_unit_struct_init(ident)),
    }
}

fn from_lua_impl_struct_type(ident: Ident, struct_init: TokenStream) -> TokenStream {
    let ident_str = ident.to_string();
    quote! {
        let table = match value {
            mlua::Value::Table(table) => Ok(table),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: #ident_str,
                message: Some(format!("Expected Table type FromLua for Rust type {}", #ident_str)),
            }),
        }?;

        let type_name = table.get::<&str, String>("type")?;
        if type_name == #ident_str {
            #struct_init
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: "Table",
                to: #ident_str,
                message: Some(format!("Found unexpected type name while converting {} FromLua: {}", #ident_str, type_name)),
            })
        }
    }
}

fn from_lua_unit_struct_init(ident: Ident) -> TokenStream {
    quote! {
        Ok(#ident)
    }
}

fn from_lua_impl_struct_unnamed_fields(ident: Ident, fields: FieldsUnnamed) -> TokenStream {
    let field_idents = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, f)| (format_ident!("field{}", i), f));

    let field_extractions = field_idents.clone().rev().map(|(field_name, f)| {
        let ty = &f.ty;
        quote! {
            let #field_name: #ty = table.pop()?;
        }
    });

    let field_list = field_idents.map(|(name, _)| name);

    quote! {
        let table = table.get::<&str, mlua::Table>("values")?;
        #(#field_extractions);*;
        Ok(#ident(#(#field_list),*))
    }
}

fn from_lua_impl_struct_named_fields(ident: Ident, fields: FieldsNamed) -> TokenStream {
    let idents_fields_zip = fields.named.iter().map(|f| (f.clone().ident.unwrap(), f));

    let field_extractions = idents_fields_zip.clone().map(|(ident, field)| {
        let ty = &field.ty;
        let ident_str = ident.to_string();
        quote! {
            let #ident = table.get::<&str, #ty>(#ident_str)?;
        }
    });

    let field_idents = idents_fields_zip.map(|(ident, _)| ident);

    quote! {
        let table = table.get::<&str, Table>("values")?;
        #(#field_extractions);*;
        Ok(#ident { #(#field_idents),* })
    }
}

fn derive_from_lua_enum(ident: Ident, generics: Generics, enm: DataEnum) -> TokenStream {
    let init_body = derive_from_lua_enum_init(&ident, &enm);
    let table_init = from_lua_impl_struct_type(ident.clone(), init_body);
    let body = from_lua_impl(ident, generics, table_init);

    quote! {
        #body
    }
}

fn derive_from_lua_enum_init(ident: &Ident, enm: &DataEnum) -> TokenStream {
    let enum_ident_str = ident.to_string();
    let enum_name_ident = format_ident!("{}Name", ident);
    let enum_variant_impl = derive_from_lua_enum_variants(ident, &enum_name_ident, enm);

    quote! {
        let variant_name = table.get::<&str, String>("variant")?;
        let variant = #enum_name_ident::from_str(variant_name.as_str())
            .map_err(|e| mlua::Error::FromLuaConversionError {
                from: "Table",
                to: enum_ident_str,
                message: Some(format!(
                    "Failed to convert 'type' field to valid {} name: {:?}",
                    #enum_ident_str,
                    e,
                )),
            })?;

        match variant {
            #enum_variant_impl
        }
    }
}

fn derive_from_lua_enum_variants(
    enum_ident: &Ident,
    enum_name_ident: &Ident,
    enm: &DataEnum
) -> TokenStream {
    quote! {}
}

fn from_lua_impl(ident: Ident, generics: Generics, impl_body: TokenStream) -> TokenStream {
    let lua_lifetime = generics
        .lifetimes()
        .next()
        .map(|l| l.lifetime.clone())
        .unwrap_or(Lifetime::new("'lua", Span::call_site()));

    quote! {
        impl<#lua_lifetime> mlua::FromLua<#lua_lifetime> for #ident #generics {
            fn from_lua(value: Value<#lua_lifetime>, _lua: &#lua_lifetime Lua) -> mlua::prelude::LuaResult<Self> {
                #impl_body
            }
        }
    }
}
