use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, parse_macro_input, Type};

#[proc_macro_derive(FromProto, attributes(proto_type_u32, proto_optional, proto_enum))]
pub fn derive_from_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                let name = &field.ident;

                let mut as_type = None;
                let mut is_optional = false;
                let mut is_enum = false;
                for attr in &field.attrs {
                    // if tagged with proto_type_u32, then set "as <type>" for the model type.
                    if attr.path().is_ident("proto_type_u32") {
                        let ty = &field.ty;
                        if let Type::Path(ty) = ty {
                            as_type = Some(ty.path.get_ident());
                        }
                    }

                    // if tagged with proto_optional, then call ".unwrap()" when creating the model type.
                    if attr.path().is_ident("proto_optional") {
                        is_optional = true;
                    }

                    // if tagged with proto_enum, then call "<field>()" on the field, to return the
                    // enum type instead of an i32.
                    if attr.path().is_ident("proto_enum") {
                        is_enum = true;
                    }
                }

                if let Some(as_type) = as_type {
                    // Note: unwrap doesn't apply if as_type is present.
                    quote!(#name: (item.#name as #as_type).into())
                } else {
                    if is_optional {
                        quote!(#name: item.#name.unwrap().into())
                    } else {
                        if is_enum {
                            quote!(#name: item.#name().into())
                        } else {
                            quote!(#name: item.#name.into())
                        }
                    }
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

    // Catchall if we don't match on the structure we want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FromProto`",
        )
        .to_compile_error(),
    )
}

#[proc_macro_derive(IntoProto, attributes(proto_type_u32, proto_optional, proto_enum))]
pub fn derive_into_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            // Deal with a named-field struct
            let field_vals = fields.named.iter().enumerate().map(|(_i, field)| {
                // grab the name of the field
                let name = &field.ident;

                let mut as_type = None;
                let mut is_optional = false;
                let mut is_enum = false;
                for attr in &field.attrs {
                    if attr.path().is_ident("proto_type_u32") {
                        as_type = Some(format_ident!("{}", "u32"));
                    }

                    if attr.path().is_ident("proto_optional") {
                        is_optional = true;
                    }

                    // if tagged with proto_enum, then call "<field>()" on the field, to return the
                    // enum type instead of an i32.
                    if attr.path().is_ident("proto_enum") {
                        is_enum = true;
                    }
                }

                if let Some(as_type) = as_type {
                    // Note: optional wrapping doesn't apply if as_type is present, since it's only
                    // used for primitives.
                    quote!(#name: (item.#name as #as_type).into())
                } else {
                    if is_optional {
                        quote!(#name: Some(item.#name.into()))
                    } else {
                        if is_enum {
                            // enums are saved as i32 in protos.
                            quote!(#name: item.#name as i32)
                        } else {
                            quote!(#name: item.#name.into())
                        }
                    }
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

    // Catchall if we don't match on the structure we want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FromProto`",
        )
        .to_compile_error(),
    )
}
