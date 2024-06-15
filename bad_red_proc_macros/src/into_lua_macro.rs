use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Generics, Index,
    Lifetime, Variant,
};

pub fn into_lua_impl(derive_input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = derive_input;

    let into_lua_function_impl = into_lua_func(&ident, &data);

    gen_into_lua_impl(&ident, &generics, &into_lua_function_impl)
}

fn gen_into_lua_impl(ident: &Ident, generics: &Generics, impl_body: &impl ToTokens) -> TokenStream {
    let lua_lifetime = generics
        .lifetimes()
        .next()
        .map(|l| l.lifetime.clone())
        .unwrap_or(Lifetime::new("'lua", Span::call_site()));

    quote! {
        impl<#lua_lifetime> mlua::IntoLua<#lua_lifetime> for #ident #generics {
            fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::prelude::LuaResult<mlua::Value<'lua>> {
                #impl_body
            }
        }

    }
}

fn into_lua_func(ident: &Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(strct) => into_lua_func_struct(ident, strct),
        Data::Enum(enm) => into_lua_func_enum(ident, enm),
        Data::Union(_) => unimplemented!(),
    }
}

fn into_lua_func_struct(ident: &Ident, strct: &DataStruct) -> TokenStream {
    let fields_inserts = table_fields_inserts(&strct.fields);
    let make_table = make_type_table(ident, &fields_inserts);

    quote! {
        #make_table
    }
}

fn into_lua_func_enum(ident: &Ident, enm: &DataEnum) -> TokenStream {
    let enum_variant = table_enum_variant(ident, enm);
    let make_table = make_type_table(ident, &enum_variant);

    quote! {
        #make_table
    }
}

fn make_type_table(ident: &Ident, fields_inserts: &impl ToTokens) -> TokenStream {
    let ident_string = ident.to_string();
    quote! {
        let table = lua.create_table()?;

        table.set("type", #ident_string)?;

        let values_table: Option<mlua::Table> = #fields_inserts;
        if let Some(values_table) = values_table {
            table.set("values", values_table)?;
        }

        table.into_lua(lua)
    }
}

fn table_fields_inserts(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => table_named_fields_inserts(named),
        Fields::Unnamed(unnamed) => table_unnamed_fields_inserts(unnamed),
        Fields::Unit => quote! { None },
    }
}

fn table_unnamed_fields_inserts(unnamed: &FieldsUnnamed) -> TokenStream {
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

fn table_named_fields_inserts(named: &FieldsNamed) -> TokenStream {
    let field_pushes = named.named.iter().map(|f| {
        let ident = f.ident.as_ref().expect("Named field found without ident");
        let ident_string = ident.to_string();
        quote! {
            values.set(#ident_string, self.#ident)?;
        }
    });

    fields_value_table(field_pushes)
}

fn fields_value_table<I, T>(field_pushes: I) -> TokenStream
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

fn table_enum_variant(enum_ident: &Ident, enm: &DataEnum) -> TokenStream {
    let enum_name_ident = format_ident!("{}Name", enum_ident);
    let match_arms = enm
        .variants
        .iter()
        .map(|v| table_enum_variant_arm(enum_ident, &v));
    quote! {
        {
            let variant_name: &'static str = #enum_name_ident::from(&self).into();
            table.set("variant", variant_name)?;
            match self {
                #(#match_arms)*
            }
        }
    }
}

fn table_enum_variant_arm(enum_ident: &Ident, variant: &Variant) -> TokenStream {
    let variant_ident = &variant.ident;
    let values_init = match &variant.fields {
        Fields::Named(named) => table_enum_named_variant_arm(enum_ident, variant_ident, named),
        Fields::Unnamed(unnamed) => {
            table_enum_unnamed_variant_arm(enum_ident, variant_ident, unnamed)
        }
        Fields::Unit => table_enum_unit_variant_arm(enum_ident, variant_ident),
    };

    quote! {
        #values_init
    }
}

fn table_enum_unit_variant_arm(enum_ident: &Ident, variant_ident: &Ident) -> TokenStream {
    quote! {
        #enum_ident::#variant_ident => { None }
    }
}

fn table_enum_unnamed_variant_arm(
    enum_ident: &Ident,
    variant_ident: &Ident,
    unnamed: &FieldsUnnamed,
) -> TokenStream {
    let fields_names = unnamed
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("field{}", i));

    let fields_inserts = fields_names.clone().map(|ident| {
        quote! {
            values.push(#ident)?;
        }
    });

    quote! {
        #enum_ident::#variant_ident(#(#fields_names),*) => {
            let values = lua.create_table()?;
            #(#fields_inserts)*
            Some(values)
        }
    }
}

fn table_enum_named_variant_arm(
    enum_ident: &Ident,
    variant_ident: &Ident,
    named: &FieldsNamed,
) -> TokenStream {
    let fields_idents = named.named.iter().map(|f| {
        f.ident
            .as_ref()
            .expect("Named fields expected to have ident. Found: none")
    });
    let fields_inserts = fields_idents.clone().map(|i| {
        let ident_string = i.to_string();
        quote! {
            values.set(#ident_string, #i)?;
        }
    });

    quote! {
        #enum_ident::#variant_ident { #(#fields_idents),* } => {
            let values = lua.create_table()?;
            #(#fields_inserts)*
            Some(values)
        }
    }
}
