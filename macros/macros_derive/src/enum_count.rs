pub(crate) fn extract_enum_variants_count(ast: &mut syn::DeriveInput) -> usize {
    if let syn::Data::Enum(enum_data) = &mut ast.data {
        return enum_data.variants.len();
    }

    0
}

pub(crate) fn enum_count_derive_macro2(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
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
