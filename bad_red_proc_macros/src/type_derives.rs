use quote::{format_ident, quote};
use proc_macro2::{TokenStream, Ident, Span};
use syn::{DataEnum, DeriveInput, Lifetime};

pub fn type_derives(typedef: &DeriveInput) -> TokenStream {
    match typedef.data {
        syn::Data::Struct(_) => quote! {},
        syn::Data::Enum(_) => {
            let name_ident = format_ident!("{}Name", typedef.ident);
            quote! {
                #[strum(serialize_all = "snake_case")]
                #[strum_discriminants(derive(strum_macros::IntoStaticStr, strum_macros::EnumString, strum_macros::EnumIter, Hash))]
                #[derive(strum_macros::EnumDiscriminants)]
                #[strum_discriminants(strum(serialize_all = "snake_case"))]
                #[strum_discriminants(name(#name_ident))]
            }
        }
        syn::Data::Union(_) => unimplemented!(),
    }
}

pub fn name_type_impls(typedef: &DeriveInput) -> TokenStream {
    match &typedef.data {
        syn::Data::Struct(_) => quote! {},
        syn::Data::Enum(enm) => {
            let from_lua = name_type_from_lua(&typedef.ident, enm);
            let into_lua = name_type_into_lua(&typedef.ident);

            quote! {
                #from_lua
                #into_lua
            }
        }
        syn::Data::Union(_) => unimplemented!(),
    }
}

pub fn name_type_from_lua(enum_ident: &Ident, enm: &DataEnum) -> TokenStream {
    let lua_lifetime = Lifetime::new("'lua", Span::call_site());
    let name_ident = format_ident!("{}Name", enum_ident);
    let name_ident_string = name_ident.to_string();
    quote! {
        impl<#lua_lifetime> mlua::FromLua<#lua_lifetime> for #name_ident {
            fn from_lua(value: mlua::Value<#lua_lifetime>, _lua: &#lua_lifetime mlua::Lua) -> mlua::prelude::LuaResult<Self> {
                use std::str::FromStr;

                let name = value
                    .as_str()
                    .ok_or_else(|| mlua::Error::FromLuaConversionError {
                        from: "Value",
                        to: #name_ident_string,
                        message: Some(format!("Expected Lua string for {}. Found: {:?}", #name_ident_string, value)),
                    })?;

                #name_ident::from_str(name).map_err(|e| mlua::Error::FromLuaConversionError {
                    from: "String",
                    to: #name_ident_string,
                    message: Some(format!("Failed to convert from string to {}: {}", #name_ident_string, e)),
                })
            }
        }
    }
}

pub fn name_type_into_lua(enum_ident: &Ident) -> TokenStream {
    let lua_lifetime = Lifetime::new("'lua", Span::call_site());
    let name_ident = format_ident!("{}Name", enum_ident);
    quote! {
        impl<#lua_lifetime> mlua::IntoLua<#lua_lifetime> for #name_ident {
            fn into_lua(
                self,
                lua: &'lua mlua::prelude::Lua,
            ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
                let self_string: &'static str = self.into();
                lua.create_string(self_string)?.into_lua(lua)
            }
        }
    }
}

