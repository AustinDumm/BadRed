// This file is part of BadRed.
//
// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DeriveInput, Generics, Ident, Lifetime, Variant};

pub fn script_table_impl(token_stream: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = syn::parse2(token_stream.clone()).unwrap();

    let body = script_table_impl_body(&ident, &generics, &data);

    quote! {
        #token_stream

        #body
    }
}

fn script_table_impl_body(ident: &Ident, _generics: &Generics, data: &Data) -> TokenStream {
    let body = match data {
        Data::Struct(_) => unimplemented!(),
        Data::Enum(enm) => script_table_function_body(ident, enm),
        Data::Union(_) => unimplemented!(),
    };

    let lifetime = Lifetime::new("'lua", Span::call_site());
    let anon_lifetime = Lifetime::new("'_", Span::call_site());

    quote! {
        impl ScriptObject for #ident <#anon_lifetime> {
            fn lua_object<#lifetime>(lua: &#lifetime Lua) -> mlua::Result<Table<#lifetime>> {
                use strum::IntoEnumIterator;
                #body
            }
        }
    }
}

fn script_table_function_body(enum_ident: &Ident, enm: &DataEnum) -> TokenStream {
    let name_ident = format_ident!("{}Name", enum_ident);
    let enum_arms = script_table_enum_arms(enum_ident, &name_ident, enm);

    quote! {
        let table = lua.create_table()?;

        for case in #name_ident::iter() {
            match case {
                #enum_arms
            }
        }

        Ok(table)
    }
}

fn script_table_enum_arms(enum_ident: &Ident, name_ident: &Ident, enm: &DataEnum) -> TokenStream {
    let arms = enm
        .variants
        .iter()
        .map(|variant| script_table_enum_arm(enum_ident, name_ident, variant));

    quote! {
        #(#arms)*
    }
}

fn script_table_enum_arm(enum_ident: &Ident, name_ident: &Ident, variant: &Variant) -> TokenStream {
    let variant_ident = &variant.ident;
    let variant_args = script_table_variant_args(variant);
    let variant_params = script_table_variant_params(variant);
    quote! {
        #name_ident::#variant_ident => {
            table.set(
                Into::<&'static str>::into(case),
                lua.create_function(|_, #variant_args| Ok(#enum_ident::#variant_ident #variant_params))?,
            )?;
        }
    }
}

fn script_table_variant_args(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Named(named) => {
            let field_names = named.named.iter().map(|f| f.ident.as_ref().unwrap());
            let field_types = named.named.iter().map(|f| &f.ty);
            quote! { (#(#field_names),*): (#(#field_types),*) }
        }
        syn::Fields::Unnamed(unnamed) => {
            let field_names = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("field{}", i));
            let field_types = unnamed.unnamed.iter().enumerate().map(|(_, f)| &f.ty);
            quote! { (#(#field_names),*): (#(#field_types),*) }
        }
        syn::Fields::Unit => quote! { _: () },
    }
}

fn script_table_variant_params(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Named(named) => {
            let field_names = named.named.iter().map(|f| f.ident.as_ref().unwrap());
            quote! { { #(#field_names),* } }
        }
        syn::Fields::Unnamed(unnamed) => {
            let field_names = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("field{}", i));
            quote! { (#(#field_names),*) }
        }
        syn::Fields::Unit => quote! {},
    }
}
