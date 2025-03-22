use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, parse_macro_input, Type};

#[proc_macro_derive(FromProto, attributes(proto_type_u32))]
pub fn derive_from_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                let name = &field.ident;

                let mut as_type = None;
                for attr in &field.attrs {
                    // if tagged with proto_type_u32, then set "as <type>" for the model type.
                    if attr.path().is_ident("proto_type_u32") {
                        let ty = &field.ty;
                        if let Type::Path(ty) = ty {
                            as_type = Some(ty.path.get_ident());
                        }
                    }
                }

                if let Some(as_type) = as_type {
                    quote!(#name: item.#name as #as_type)
                } else {
                    quote!(#name: item.#name.into())
                }
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
            )
            .into();
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

#[proc_macro_derive(IntoProto, attributes(proto_type_u32))]
pub fn derive_into_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                // grab the name of the field
                let name = &field.ident;

                let mut as_type = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("proto_type_u32") {
                        as_type = Some(format_ident!("{}", "u32"));
                    }
                }

                if let Some(as_type) = as_type {
                    quote!(#name: item.#name as #as_type)
                } else {
                    quote!(#name: item.#name.into())
                }
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
            )
            .into();
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
