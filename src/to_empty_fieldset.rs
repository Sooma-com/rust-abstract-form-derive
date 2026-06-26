use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::DeriveInput;

pub fn impl_to_empty_fieldset(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = ast;
    match data {
        syn::Data::Union(_) => {
            quote_spanned! { ident.span() => compile_error!("Unions are not supported for ToEmptyFieldSet") }.into()
        }
        syn::Data::Enum(enum_data) => {
            let has_non_unit = enum_data.variants.iter().any(|v| !v.fields.is_empty());
            if has_non_unit {
                return quote_spanned! { ident.span() =>
                    compile_error!("Only enums with unit variants are supported for ToEmptyFieldSet")
                }.into();
            }
            quote_spanned! { ident.span() =>
                impl abstract_form::fieldset::to_empty_fieldset::ToEmptyFieldSet for #ident {
                    fn to_empty_fieldset() -> abstract_form::FieldSet {
                        let options: Vec<(String, String)> = <#ident as strum::IntoEnumIterator>::iter()
                            .map(|v| (v.to_string(), v.to_string()))
                            .collect();
                        let validation = abstract_form::validation::ClosedSingleChoice::new(options);
                        let field = abstract_form::field::SingleValue::<String> {
                            tag: "".to_string(),
                            label: "".to_string(),
                            value: "".to_string(),
                            validations: vec![std::sync::Arc::new(Box::new(validation))],
                        };
                        abstract_form::FieldSet {
                            tag: "".to_string(),
                            label: "".to_string(),
                            controls: vec![ std::sync::Arc::new(Box::new(field)) ],
                        }
                    }
                }
            }.into()
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
                            for control in inner.field_iter_mut() {
                                let control = std::sync::Arc::get_mut(control).unwrap();
                                control.prepend_tag(#field_name);
                                control.set_label(#field_name);
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
