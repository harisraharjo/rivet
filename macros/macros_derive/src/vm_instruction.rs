fn extract_opcodes<T>(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> impl Iterator<Item = Option<T>> + '_
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    variants
        .into_iter()
        .map(|v| v.attrs.iter().filter(|attr| attr.path().is_ident("opcode")))
        .map(|attr| {
            attr.map(|attr| {
                let value: syn::LitInt = attr.parse_args().unwrap();
                value.base10_parse().unwrap()
            })
            .next()
        })
}

type EncodedValue = proc_macro2::TokenStream;
fn generate_operands<const IS_ENCODE: bool>(
    n: usize,
    i: usize,
    field_name: &proc_macro2::TokenStream,
) -> EncodedValue {
    match n {
        // 24 bit
        1 => {
            if IS_ENCODE {
                quote::quote!((#field_name & 0xFFFFFF) << 8 )
            } else {
                quote::quote!((#field_name >> 8) & 0xFFFFFF)
            }
        }
        2 => {
            if i < 1 {
                return if IS_ENCODE {
                    quote::quote!((#field_name & 0xFF) << 8)
                } else {
                    quote::quote!((#field_name >> 8) & 0xFF)
                };
            }

            if IS_ENCODE {
                quote::quote!((#field_name & 0xFFFF) << 16 )
            } else {
                quote::quote!((#field_name >> 16) & 0xFFFF)
            }
        }

        3 => match i {
            0 => {
                if IS_ENCODE {
                    quote::quote!(#field_name << 8)
                } else {
                    quote::quote!((#field_name >> 8) & 0xFF)
                }
            }
            1 => {
                if IS_ENCODE {
                    quote::quote!(#field_name << 16)
                } else {
                    quote::quote!((#field_name >> 16) & 0xFF)
                }
            }
            _ => {
                if IS_ENCODE {
                    quote::quote!(#field_name << 24)
                } else {
                    quote::quote!((#field_name >> 24) & 0xFF )
                }
            }
        },
        _ => panic!("Max 3 operands"),
    }
}

type EField = (syn::Ident, String);
fn extract_variant_fields(
    fields: &syn::Fields,
) -> Option<(impl Iterator<Item = EField> + '_, usize)> {
    match fields {
        syn::Fields::Named(f) => {
            let f_ = f.named.iter().map(|f| {
                let name = f.ident.to_owned().unwrap();
                let mut ty = String::new();
                if let syn::Type::Path(type_path) = &f.ty {
                    ty = type_path.path.segments.last().unwrap().ident.to_string();
                }

                (name, ty)
            });
            Some((f_, f.named.len()))
        }
        syn::Fields::Unit => None,
        _ => panic!("Only accept Named & Unit"),
    }
}

fn extract_variant_data(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> impl Iterator<
    Item = (
        &syn::Ident,
        proc_macro2::Span,
        Option<(impl Iterator<Item = EField> + '_, usize)>,
    ),
> {
    variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_span = v.ident.span();

        let fields = extract_variant_fields(&v.fields);

        (variant_name, variant_span, fields)
    })
}

const PRIMITIVES_INT: [&str; 3] = ["u8", "u16", "u32"];

fn generate_field_and_values(
    field_name_iter: impl Iterator<Item = EField>,
    n: usize,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut encoded_field_value = quote::quote!();
    let mut decoded_field_value = quote::quote!();
    let mut field_names = quote::quote!();

    for (i, (field_name, ty)) in field_name_iter.enumerate() {
        field_names.extend(quote::quote! {
            #field_name,
        });

        let mut prefix = quote::quote!(#field_name);
        let suffix = if PRIMITIVES_INT.contains(&ty.as_str()) {
            prefix = quote::quote!(*#field_name as u32);
            let ty: proc_macro2::TokenStream = ty.parse().unwrap();
            quote::quote!( as #ty,)
        } else {
            quote::quote!(.into(),)
        };

        let bit_mask = generate_operands::<true>(n, i, &prefix);
        encoded_field_value.extend(quote::quote! {
            result |= #bit_mask;
        });

        let bit_mask = generate_operands::<false>(n, i, &quote::quote!(value));
        decoded_field_value.extend(quote::quote!(#field_name: (#bit_mask)#suffix));
    }

    (
        quote::quote! {
            { #field_names }
        },
        encoded_field_value,
        quote::quote! {
            { #decoded_field_value }
        },
    )
}

pub(crate) fn opcode_derive_macro2(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;
    let enum_name = &ast.ident;
    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();
    let (encoded_variants, decoded_variants) = match &mut ast.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let variant_opcode_iter = extract_opcodes::<u8>(variants);
            let variant_data_iter = extract_variant_data(variants);

            let (encoded, decoded): (Vec<_>, Vec<_>) = variant_data_iter
                .zip(variant_opcode_iter)
                .map(|((variant_name, variant_span, fields), opcode)| {
                    let (fields, raw_encoded_field_value, raw_decoded_field_value) =
                        if let Some((field_name_iter, length)) = fields {
                            generate_field_and_values(field_name_iter, length)
                        } else {
                            (quote::quote!(), quote::quote!(), quote::quote!())
                        };

                    // encode
                    let encoded_field_value = quote::quote! {
                        {
                            let mut  result: u32 = #opcode as u32;
                            #raw_encoded_field_value
                            result
                        }
                    };

                    let encode_result = quote::quote_spanned! {
                        variant_span=>
                        #enum_name::#variant_name #fields => #encoded_field_value,
                    };

                    //decode
                    let decoded_result = quote::quote_spanned! {
                        variant_span =>
                        #opcode => Ok(Self::#variant_name #raw_decoded_field_value ),
                    };

                    (encode_result, decoded_result)
                })
                .unzip();

            Ok((encoded, decoded))
        }
        _ => Err(syn::Error::new(
            syn::spanned::Spanned::span(&ast),
            "Can only be applied on an enum type",
        )),
    }?;

    //generate
    Ok(quote::quote! {
        #[derive(Debug)]
        pub enum DecodeError {
            // Message(&'static str),
            UnknownOpcode
        }

        impl #impl_g TryFrom<u32> for #enum_name #type_g #where_c {
            type Error = DecodeError;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                let opcode = value as u8;
                match opcode {
                    #(#decoded_variants)*
                    _ => Err(DecodeError::UnknownOpcode),
                }
            }
        }

        impl #impl_g From<&#enum_name> for u32 #type_g #where_c {
            fn from(value: &#enum_name) -> Self {
                match value {
                    #(#encoded_variants)*
                }
            }
        }

        // impl #impl_g VMInstruction for #enum_name #type_g #where_c {
        //     fn opcode(&self) -> u8 {
        //         match self {
        //             #(#encoded_variant)*
        //         }
        //     }
        // }


    })
}
