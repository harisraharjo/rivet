mod enum_macros;
mod vm_instruction;

#[proc_macro_derive(VMInstruction, attributes(isa))]
pub fn isa(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    vm_instruction::isa2(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(EnumCount)]
pub fn enum_count_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_macros::enum_count_derive_macro2(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(EnumVariants)]
pub fn enum_variants_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_macros::enum_variants(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
