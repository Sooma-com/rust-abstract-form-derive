use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::DeriveInput;

pub fn impl_to_empty_fieldset(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = ast;
    match data {
        syn::Data::Union(_) => {
            quote_spanned! { ident.span() => compile_error!("Unions are not supported for ToEmptyFieldSet") }.into()
        }
        syn::Data::Enum(_) => {
            quote_spanned! { ident.span() => compile_error!("Enums are not supported for ToEmptyFieldSet") }.into()
        }
        syn::Data::Struct(struct_data) => match &struct_data.fields {
            syn::Fields::Named(fields) => {
                let struct_name = ident.to_string();
                let field_stmts = fields
                    .named
                    .iter()
                    .filter(|field| {
                        !field.attrs.iter().any(|attr|
                            attr.path().is_ident("abstract_form") &&
                            matches!(&attr.meta, syn::Meta::List(list) if
                                list.tokens.to_string().contains("skip")))
                    })
                    .filter_map(|field| {
                        let field_ident = field.ident.as_ref()?;
                        let field_name = field_ident.to_string();
                        let field_ty = &field.ty;
                        Some(quote_spanned! { field_ident.span() => {
                            let mut inner = <#field_ty as abstract_form::fieldset::to_empty_fieldset::ToEmptyFieldSet>::to_empty_fieldset();
                            let inner_tag = inner.tag.clone();
                            for control in &mut inner.controls {
                                control.prepend_tag(&inner_tag);
                                control.prepend_tag(#field_name);
                            }
                            fieldset.merge(&inner);
                        }})
                    })
                    .collect::<Vec<_>>();

                quote_spanned! { ident.span() =>
                    impl abstract_form::fieldset::to_empty_fieldset::ToEmptyFieldSet for #ident {
                        fn to_empty_fieldset() -> abstract_form::FieldSet {
                            let mut fieldset = abstract_form::FieldSet::new(
                                #struct_name.to_string(),
                                "".to_string(),
                            );
                            #(#field_stmts)*
                            fieldset
                        }
                    }
                }
                .into()
            }
            _ => {
                quote_spanned! { ident.span() => compile_error!("Only structs with named fields are supported for ToEmptyFieldSet") }.into()
            }
        },
    }
}
