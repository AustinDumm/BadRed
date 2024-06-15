use quote::{format_ident, quote};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn type_derives(typedef: DeriveInput) -> TokenStream {
    match typedef.data {
        syn::Data::Struct(_) => quote! {},
        syn::Data::Enum(_) => {
            let name_ident = format_ident!("{}Name", typedef.ident);
            quote! {
                #[derive(EnumDiscriminants)]
                #[strum(serialize_all = "snake_case")]
                #[strum_discriminants(derive(IntoStaticStr, EnumString, EnumIter))]
                #[strum_discriminants(strum(serialize_all = "snake_case"))]
                #[strum_discriminants(name(#name_ident))]
            }
        },
        syn::Data::Union(_) => unimplemented!(),
    }
}
