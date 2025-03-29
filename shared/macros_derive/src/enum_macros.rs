use proc_macro2::TokenStream;
use syn::{LitStr, spanned::Spanned};

fn extract_enum_variants_count(ast: &mut syn::DeriveInput) -> usize {
    if let syn::Data::Enum(enum_data) = &mut ast.data {
        return enum_data.variants.len();
    }

    0
}

pub(crate) fn enum_count_derive_macro2(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;

    let variant_count = extract_enum_variants_count(&mut ast);

    // impl var
    let ident = &ast.ident;

    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();

    //generate
    Ok(quote::quote! {
        impl #impl_g EnumCount for #ident #type_g #where_c {
            const VARIANT_COUNT: usize = #variant_count;
        }
    })
}

pub(crate) fn enum_variants(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: syn::DeriveInput = syn::parse2(input)?;
    // impl var
    let enum_name = &ast.ident;

    let (impl_g, type_g, where_c) = ast.generics.split_for_impl();
    let variants_str: Vec<TokenStream> = match &mut ast.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => Ok(variants
            .iter()
            .map(|v| {
                let ident = &v.ident;
                let mut ident_string = ident.to_string();
                ident_string.make_ascii_lowercase();
                let span = v.span();
                let ident_str = LitStr::new(ident_string.as_ref(), span);
                quote::quote_spanned! {
                    span=>
                    #ident_str,
                }
            })
            .collect()),
        _ => Err(syn::Error::new(
            syn::spanned::Spanned::span(&ast),
            "Can only be applied on an enum type",
        )),
    }?;

    let len = variants_str.len();

    //generate
    Ok(quote::quote! {
        impl #impl_g EnumVariants<#len> for #enum_name #type_g #where_c {
            fn variants<'a>() -> [&'a str; #len] {
                [#(#variants_str)*]
            }
        }
    })
}
