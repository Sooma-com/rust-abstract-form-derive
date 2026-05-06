mod to_empty_fieldset;

use proc_macro::TokenStream;

#[proc_macro_derive(ToEmptyFieldSet, attributes(abstract_form))]
pub fn to_empty_fieldset_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    to_empty_fieldset::impl_to_empty_fieldset(&ast)
}
