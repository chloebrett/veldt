use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Type, Field, parse_macro_input};

enum Tag {
    AsType {
        proto_type: Ident,
        model_type: Ident,
    },
    Optional,
    Enum,
    Repeated,
    NoTag,
}

/// Extracts up to one tag from a struct field. Used to determine what to codegen for that field.
fn extract_tag(field: &Field) -> Tag {
    let mut tag = Tag::NoTag;
    for attr in &field.attrs {
        // if tagged with proto_type_u32, then set "as <type>" for the model type.
        if attr.path().is_ident("proto_type_u32") {
            if let Type::Path(ty) = &field.ty {
                tag = Tag::AsType {
                    proto_type: format_ident!("{}", "u32"),
                    model_type: ty.path.get_ident().unwrap().clone(),
                };
            }
        }

        // if tagged with proto_optional, then call ".unwrap()" when creating the model type.
        if attr.path().is_ident("proto_optional") {
            tag = Tag::Optional;
        }

        // if tagged with proto_enum, then call "<field>()" on the field, to return the
        // enum type instead of an i32.
        if attr.path().is_ident("proto_enum") {
            tag = Tag::Enum;
        }

        if attr.path().is_ident("proto_repeated") {
            tag = Tag::Repeated;
        }
    }
    tag
}

#[proc_macro_derive(
    FromProto,
    attributes(proto_type_u32, proto_optional, proto_enum, proto_repeated)
)]
pub fn derive_from_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();
    let proto_name = format_ident!("{}Proto", name.clone());

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let field_vals = fields.named.iter().map(|field| {
                let name = &field.ident;
                let tag = extract_tag(&field);

                match tag {
                    Tag::AsType { model_type, .. } => {
                        quote!(#name: (item.#name as #model_type).into())
                    }
                    Tag::Optional => quote!(#name: item.#name.unwrap().into()),
                    Tag::Repeated => {
                        quote!(#name: item.#name.into_iter().map(|it| it.into()).collect())
                    }
                    Tag::Enum => quote!(#name: item.#name().into()),
                    Tag::NoTag => quote!(#name: item.#name.into()),
                }
            });

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

    if let syn::Data::Enum(ref data) = input.data {
        let variants = &data.variants;
        let mut variant_vals: Vec<_> = variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                // e.g. SimpleResonator -> SimpleResonatorEqType
                let proto_variant_name = format_ident!("{}{}", variant_name.clone(), name.clone());

                quote!(#proto_name::#proto_variant_name => #name::#variant_name)
            })
            .collect();

        variant_vals.push(quote!(_ => panic!("")));

        return quote!(
            impl From<#proto_name> for #name {
                fn from(item: #proto_name) -> #name {
                    match item {
                        #(#variant_vals),*
                    }
                }
            }
        )
        .into();
    }

    // Catchall if we don't match on the structure we want.
    catchall_error(&input)
}

#[proc_macro_derive(
    IntoProto,
    attributes(proto_type_u32, proto_optional, proto_enum, proto_repeated)
)]
pub fn derive_into_proto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();
    let proto_name = format_ident!("{}Proto", name).clone();

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let field_vals = fields.named.iter().map(|field| {
                let name = &field.ident;
                let tag = extract_tag(&field);

                match tag {
                    Tag::AsType { proto_type, .. } => {
                        quote!(#name: (item.#name as #proto_type).into())
                    }
                    Tag::Optional => quote!(#name: Some(item.#name.into())),
                    Tag::Repeated => {
                        quote!(#name: item.#name.into_iter().map(|it| it.into()).collect())
                    }
                    // enums are saved as i32 in protos.
                    // increment by 1 to account for Unkown = 0.
                    Tag::Enum => quote!(#name: item.#name as i32 + 1),
                    Tag::NoTag => quote!(#name: item.#name.into()),
                }
            });

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

    if let syn::Data::Enum(ref data) = input.data {
        let variants = &data.variants;
        let variant_vals = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            // e.g. SimpleResonator -> SimpleResonatorEqType
            let proto_variant_name = format_ident!("{}{}", variant_name.clone(), name.clone());

            quote!(#name::#variant_name => #proto_name::#proto_variant_name)
        });

        return quote!(
            impl From<#name> for #proto_name {
                fn from(item: #name) -> #proto_name {
                    match item {
                        #(#variant_vals),*
                    }
                }
            }
        )
        .into();
    }

    // Catchall if we don't match on the structure we want.
    catchall_error(&input)
}

fn catchall_error(input: &DeriveInput) -> TokenStream {
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FromProto`",
        )
        .to_compile_error(),
    )
}
