use proc_macro2::Span;

fn extract_isa(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> impl Iterator<Item = Vec<u32>> + '_ {
    variants
        .into_iter()
        .map(|v| v.attrs.iter().filter(|attr| attr.path().is_ident("isa")))
        .map(|attr| {
            let mut data = Vec::<u32>::new();

            attr.for_each(|attr| {
                let args = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::LitInt, syn::Token![,]>::parse_terminated
            ).unwrap();

                for lit in args {
                    let value: u32 = lit.base10_parse().unwrap();
                    data.push(value);
                }
            });

            data
        })
}

type EField = (syn::Ident, syn::TypePath);
fn extract_variant_fields(fields: &syn::Fields) -> Option<impl Iterator<Item = EField> + '_> {
    match fields {
        syn::Fields::Named(fields) => {
            let f_iter = fields.named.iter().map(|f| {
                let name = f.ident.to_owned().unwrap();
                // let mut ty = String::new();
                let ty = if let syn::Type::Path(type_path) = &f.ty {
                    // ty = type_path.path.segments.last().unwrap().ident.to_string();
                    type_path.to_owned()
                } else {
                    panic!("Only support owned Named types ")
                };

                (name, ty)
            });
            Some(f_iter)
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
        Option<impl Iterator<Item = EField> + '_>,
    ),
> {
    variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_span = v.ident.span();

        let fields = extract_variant_fields(&v.fields);

        (variant_name, variant_span, fields)
    })
}

// TODO: Pad instruction that are not full 32 bit. ex: instructions that only uses registers (8+5+5 = 18 bit used);
fn create_bit_mask(bit_count: u32) -> u32 {
    (1u32 << bit_count) - 1
}

fn generate_fields(
    fields_iter: impl Iterator<Item = EField>,
    isa: Vec<u32>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut encoded_field_value = quote::quote!();
    let mut decoded_field_value = quote::quote!();
    let mut field_names = quote::quote!();

    let isa_iter = isa.iter().scan(0u32, |state, x| {
        *state += x;
        Some(*state)
    });

    for (field_bits, (field_name, ty)) in isa[1..].iter().zip(isa_iter).zip(fields_iter) {
        field_names.extend(quote::quote! {
            #field_name,
        });

        let bit_length = create_bit_mask(*field_bits.0);
        let acc_bits = field_bits.1;

        encoded_field_value.extend(quote::quote! {
            | #field_name.encode(#bit_length, #acc_bits)
        });

        decoded_field_value.extend(quote::quote! {
            #field_name: #ty::decode(value, #acc_bits, #bit_length),
        });
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

pub(crate) fn isa2(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;
    let enum_name = &ast.ident;
    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();

    let result = match &mut ast.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let variant_isa_iter = extract_isa(variants);
            let variant_data_iter = extract_variant_data(variants);

            let result: ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)) = variant_data_iter
                .zip(variant_isa_iter)
                .map(|((variant_name, variant_span, fields), mut isa)| {
                    let opcode = isa[0] as u8;
                    isa[0] = 8; //change opcode value to bits it occupy

                    let (fields_names, raw_encoded_field_value, raw_decoded_field_value) =
                        if let Some(field_iter) = fields {
                            generate_fields(field_iter, isa)
                        } else {
                            (quote::quote!(), quote::quote!(), quote::quote!())
                        };

                    // encode
                    let encoded_field_value = quote::quote! {
                        {
                            (#opcode as u32) #raw_encoded_field_value
                        }
                    };

                    let _encode_stream = quote::quote_spanned! {
                        variant_span=>
                        #enum_name::#variant_name #fields_names => #encoded_field_value,
                    };

                    //decode
                    let _decode_stream = quote::quote_spanned! {
                        variant_span =>
                        #opcode => Ok(Self::#variant_name #raw_decoded_field_value ),
                    };

                    //opcodes
                    let _opcodes_stream = quote::quote_spanned! {
                        variant_span=>
                        #enum_name::#variant_name { .. } => #opcode,
                    };

                    (
                        (_encode_stream, _decode_stream),
                        (variant_name, _opcodes_stream),
                    )
                })
                .unzip();

            let (encode_decode, (variants_name, opcodes)) = result;

            let mnemonic_variants = {
                let span = Span::call_site();
                let _mnemonic_variant: Vec<_> = variants_name
                    .iter()
                    .map(|variant_name| {
                        // let mnemonic_string = variant_name.to_string().to_lowercase();
                        // let lit_str = LitStr::new(mnemonic_string.as_ref(), span);

                        quote::quote_spanned! {
                            span=>
                            #variant_name,
                        }
                    })
                    .collect();

                quote::quote! {

                        #(#_mnemonic_variant)*

                }
            };

            Ok((encode_decode, (mnemonic_variants, opcodes)))
        }
        _ => Err(syn::Error::new(
            syn::spanned::Spanned::span(&ast),
            "Can only be applied on an enum type",
        )),
    }?;

    let ((encoded_variants, decoded_variants), (mnemonic_variants, opcodes)) = result;
    //generate
    Ok(quote::quote! {
        impl #impl_g TryFrom<u32> for #enum_name #type_g #where_c {
            type Error = shared::DecodeError;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                let opcode = value as u8;
                match opcode {
                    #(#decoded_variants)*
                    _ => Err(DecodeError::UnknownOpcode(opcode)),
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

        impl #impl_g From<&#enum_name> for u8 #type_g #where_c {
            fn from(value: &#enum_name) -> Self {
                match value {
                    #(#opcodes)*
                }
            }
        }

        impl #impl_g #enum_name #type_g #where_c {
            pub fn mnemonics<'a>() -> [&'a str; Self::VARIANT_COUNT] {
                Mnemonic::variants()
            }
        }

        #[derive(Debug, EnumVariants, PartialEq, Eq)]
        pub enum Mnemonic {
            #mnemonic_variants
        }

    })
}

// let pattern: Vec<String> = extract_variant_data(variants)
//     .flat_map(|(_, __, fields)| fields)
//     .map(|f| f.map(|(_, ty)| ty.path.segments.last().unwrap().ident.to_string()))
//     .map(|iter| {
//         let it: Vec<String> = iter
//             .map(|mut value| {
//                 let mut res = unsafe {
//                     value
//                         .get_unchecked_mut(0usize..1)
//                         .to_owned()
//                         .to_ascii_lowercase()
//                 };

//                 if value.starts_with("I") {
//                     res.push_str(unsafe { value.get_unchecked((value.len() - 2)..) });
//                 }

//                 value.clear();
//                 value = res;
//                 value.shrink_to_fit();

//                 value
//             })
//             .collect();

//         let f_string = it.join(",");
//         eprintln!("field type: {f_string}");
//         f_string
//     })
//     .collect();
