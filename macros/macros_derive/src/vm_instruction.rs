type OperandArray = [proc_macro2::TokenStream; 3];
fn extract_operands(n: usize) -> OperandArray {
    let mut result: OperandArray = [quote::quote!(), quote::quote!(), quote::quote!()];
    match n {
        1 => {
            result[0] = quote::quote!(((value >> 8) & 0xFFFFFF) as u8);
        }
        2 => {
            result[0] = quote::quote!(((value >> 8) & 0xFF) as u8);
            result[1] = quote::quote!(((value >> 16) & 0xFFFF) as u16);
        }
        3 => {
            result[0] = quote::quote!(((value >> 8) & 0xFF) as u8);
            result[1] = quote::quote!(((value >> 8) & 0xFF) as u8);
            result[2] = quote::quote!(((value >> 8) & 0xFF) as u8);
        }
        _ => panic!("Max 3 operands"),
    };

    result
}

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

fn extract_variant_fields<const IS_ENCODE: bool>(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(f) => {
            if IS_ENCODE {
                quote::quote!({ .. })
            } else {
                let field_count = f.named.len();
                let fields = extract_operands(field_count);
                let field_names: Vec<proc_macro2::TokenStream> = f
                    .named
                    .iter()
                    .zip(fields)
                    .map(|(field, operand)| {
                        let field_name = &field.ident;
                        quote::quote! {
                            #field_name: (#operand).into(),
                        }
                    })
                    .collect();

                quote::quote! ({ #(#field_names)* })
            }
        }
        syn::Fields::Unit => quote::quote!(),
        syn::Fields::Unnamed(_) => quote::quote!((..)),
    }
}

fn extract_variant_data(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> impl Iterator<
    Item = (
        &syn::Ident,
        proc_macro2::Span,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ),
> {
    variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_span = v.ident.span();

        let fields_encode = extract_variant_fields::<true>(&v.fields);
        let fields_decode = extract_variant_fields::<false>(&v.fields);

        (variant_name, variant_span, fields_encode, fields_decode)
    })
}

pub(crate) fn opcode_derive_macro2(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;
    let enum_name = &ast.ident;
    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();
    let (encoded_variant, decoder_variants): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = match &mut ast.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let opcode_iter = extract_opcodes::<u8>(variants);
            let variant_data_iter = extract_variant_data(variants);

            Ok(variant_data_iter
                .zip(opcode_iter)
                .map(
                    |((variant_name, variant_span, fields_encode, fields_decode), opcode)| {
                        //generate
                        let encoder = quote::quote_spanned! {
                            variant_span=>
                            #enum_name::#variant_name #fields_encode => #opcode,
                        };

                        let opcode: u32 = opcode.unwrap().into();
                        let decoder = quote::quote_spanned! {
                            variant_span =>
                            #opcode => Ok(Self::#variant_name #fields_decode),
                        };

                        (encoder, decoder)
                    },
                )
                .unzip())
        }
        _ => Err(syn::Error::new(
            syn::spanned::Spanned::span(&ast),
            "Can only be applied on an enum type",
        )),
    }?;

    //generate
    Ok(quote::quote! {
        pub enum DecodeError {
            Message(String)
        }

        impl #impl_g TryFrom<u32> for #enum_name #type_g #where_c {
            type Error = DecodeError;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    #(#decoder_variants)*
                    _ => Err(DecodeError::Message(String::from("Unknown opcode"))),
                }
            }
        }

        impl #impl_g VMInstruction for #enum_name #type_g #where_c {
            fn opcode(&self) -> u8 {
                match self {
                    #(#encoded_variant)*
                }
            }
        }


    })
}
