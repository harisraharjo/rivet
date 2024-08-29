mod enum_count;
mod vm_instruction;

#[proc_macro_derive(VMInstruction, attributes(opcode))]
pub fn opcode_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    vm_instruction::opcode_derive_macro2(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(EnumCount)]
pub fn enum_count_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_count::enum_count_derive_macro2(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
