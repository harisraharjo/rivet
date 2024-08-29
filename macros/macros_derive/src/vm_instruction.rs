#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(opcode))]
struct Opcode(u8);

impl Opcode {
    fn to_tokenstream(&self) -> proc_macro2::TokenStream {
        let data = self.0;
        quote::quote! { #data }
    }
}

pub(crate) fn opcode_derive_macro2(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;
    let enum_name = &ast.ident;
    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();
    let raw_arms = match &mut ast.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => Ok(variants.into_iter().map(|v| {
            let opcode = match deluxe::extract_attributes::<_, Opcode>(v) {
                Ok(o) => o.to_tokenstream(),
                Err(e) => panic!("{e}"),
            };

            let variant_name = &v.ident;
            let variant_span = v.ident.span();
            let fields = match &v.fields {
                syn::Fields::Named(_) => quote::quote!({ .. }),
                syn::Fields::Unnamed(_) => quote::quote!((..)),
                syn::Fields::Unit => quote::quote!(),
            };

            quote::quote_spanned! {
                variant_span=>
                #enum_name::#variant_name #fields => #opcode,
            }
        })),
        _ => Err(syn::Error::new(
            syn::spanned::Spanned::span(&ast),
            "Can only be applied on an enum type",
        )),
    };

    let arms = raw_arms?;
    //generate
    Ok(quote::quote! {
        impl #impl_g VMInstruction for #enum_name #type_g #where_c {
            fn opcode(&self) -> u8 {
                match self {
                    #(#arms)*
                }
            }
        }
    })
}
