use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FromProto)]
pub fn derive_from_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                // grab the name of the field
                let name = &field.ident;
                quote!(#name: item.#name.into())
            });

            let name = input.ident;
            let proto_name = format_ident!("{}Proto", name);

            return quote!(
                impl From<#proto_name> for #name {
                    fn from(item: #proto_name) -> #name {
                        Self {
                            #(#field_vals),*
                        }
                    }
                }
            ).into();
        }
    }

    // Catchall if we don't match on the structure we don't want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FromProto`",
        )
        .to_compile_error(),
    )
}

#[proc_macro_derive(IntoProto)]
pub fn derive_into_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                // grab the name of the field
                let name = &field.ident;
                quote!(#name: item.#name.into())
            });

            let name = input.ident;
            let proto_name = format_ident!("{}Proto", name);

            return quote!(
                impl From<#name> for #proto_name {
                    fn from(item: #name) -> #proto_name {
                        Self {
                            #(#field_vals),*
                        }
                    }
                }
            ).into();
        }
    }

    // Catchall if we don't match on the structure we don't want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FromProto`",
        )
        .to_compile_error(),
    )
}
