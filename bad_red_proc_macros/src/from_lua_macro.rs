extern crate proc_macro;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::FieldsNamed;
use syn::FieldsUnnamed;
use syn::Generics;
use syn::Lifetime;
use syn::{DataEnum, DataStruct, DeriveInput, Ident};

pub fn from_lua_impl(derive_input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = derive_input;

    match data {
        syn::Data::Struct(strct) => {
            gen_from_lua_impl(&ident, &generics, &body_from_lua_struct(&ident, &strct))
        }
        syn::Data::Enum(enm) => from_lua_enum(&ident, &generics, &enm),
        syn::Data::Union(_) => unimplemented!("Union not supported as a FromLua type"),
    }
}

fn body_from_lua_struct(ident: &Ident, strct: &DataStruct) -> TokenStream {
    match &strct.fields {
        syn::Fields::Named(named) => {
            let struct_init = from_lua_impl_struct_named_fields(&ident, &named);
            let body = from_lua_impl_struct_type(&ident, &struct_init);

            quote! {
                #body
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            let struct_init = from_lua_impl_struct_unnamed_fields(&ident, &unnamed);
            let body = from_lua_impl_struct_type(&ident, &struct_init);
            quote! {
                #body
            }
        }
        syn::Fields::Unit => from_lua_impl_struct_type(&ident, &from_lua_unit_struct_init(&ident)),
    }
}

fn from_lua_impl_struct_type(ident: &Ident, struct_init: &TokenStream) -> TokenStream {
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

fn from_lua_unit_struct_init(ident: &Ident) -> TokenStream {
    quote! {
        Ok(#ident)
    }
}

fn from_lua_impl_struct_unnamed_fields(
    init_name: &impl ToTokens,
    fields: &FieldsUnnamed,
) -> TokenStream {
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
        Ok(#init_name(#(#field_list),*))
    }
}

fn from_lua_impl_struct_named_fields(
    init_expr: &impl ToTokens,
    fields: &FieldsNamed,
) -> TokenStream {
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
        Ok(#init_expr { #(#field_idents),* })
    }
}

fn from_lua_enum(ident: &Ident, generics: &Generics, enm: &DataEnum) -> TokenStream {
    let init_body = from_lua_enum_init(&ident, &enm);
    let table_init = from_lua_impl_struct_type(&ident, &init_body);
    let body = gen_from_lua_impl(ident, generics, &table_init);

    quote! {
        #body
    }
}

fn from_lua_enum_init(ident: &Ident, enm: &DataEnum) -> TokenStream {
    let enum_ident_str = ident.to_string();
    let enum_name_ident = format_ident!("{}Name", ident);
    let enum_variant_impl = from_lua_enum_variants(ident, &enum_name_ident, enm);

    quote! {
        let variant_name = table.get::<&str, String>("variant")?;
        let variant = #enum_name_ident::from_str(variant_name.as_str())
            .map_err(|e| mlua::Error::FromLuaConversionError {
                from: "Table",
                to: #enum_ident_str,
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

fn from_lua_enum_variants(
    enum_ident: &Ident,
    enum_name_ident: &Ident,
    enm: &DataEnum,
) -> TokenStream {
    let arm_iter = enm.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_init_name = quote! { #enum_ident::#variant_ident };
        let values_init = match &variant.fields {
            syn::Fields::Named(named) => from_lua_impl_struct_named_fields(
                &variant_init_name,
                &named,
            ),
            syn::Fields::Unnamed(unnamed) => from_lua_impl_struct_unnamed_fields(
                &variant_init_name,
                &unnamed,
            ),
            syn::Fields::Unit => {
                quote! { #variant_init_name }
            }
        };
        quote! {
            #enum_name_ident::#variant_ident => {
                #values_init
            }
        }
    });

    quote! { #(#arm_iter)* }
}

fn gen_from_lua_impl(ident: &Ident, generics: &Generics, impl_body: &TokenStream) -> TokenStream {
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
