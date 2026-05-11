use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::DeriveInput;

pub fn impl_to_field_set(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = ast;
    match data {
        syn::Data::Union(_) => {
            quote_spanned! { ident.span() => compile_error!("Unions are not supported for ToFieldSet") }.into()
        }
        syn::Data::Enum(enum_data) => {
            let has_non_unit = enum_data.variants.iter().any(|v| !v.fields.is_empty());
            if has_non_unit {
                return quote_spanned! { ident.span() =>
                    compile_error!("Only enums with unit variants are supported for ToFieldSet")
                }.into();
            }
            let enum_name = ident.to_string();
            quote_spanned! { ident.span() =>
                impl abstract_form::fieldset::to_field_set::ToFieldSet for #ident {
                    fn to_field_set(&self) -> abstract_form::FieldSet {
                        let options: Vec<String> = <#ident as strum::IntoEnumIterator>::iter()
                            .map(|v| v.to_string())
                            .collect();
                        let validation = abstract_form::validation::ClosedSingleChoice::new(options);
                        let mut field = abstract_form::Field::Text(abstract_form::field::Text::new(
                            "".to_string(),
                            "".to_string(),
                            self.to_string(),
                        ));
                        field.add_validation(std::sync::Arc::new(Box::new(validation)));
                        let mut fieldset = abstract_form::FieldSet::new(
                            #enum_name.to_string(),
                            "".to_string(),
                        );
                        fieldset.controls.push(field);
                        fieldset
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
                        Some(quote_spanned! { field_ident.span() => {
                            let mut inner = self.#field_ident.to_field_set();
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
                    impl abstract_form::fieldset::to_field_set::ToFieldSet for #ident {
                        fn to_field_set(&self) -> abstract_form::FieldSet {
                            use abstract_form::fieldset::to_field_set::ToFieldSet as _;
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
                quote_spanned! { ident.span() => compile_error!("Only structs with named fields are supported for ToFieldSet") }.into()
            }
        },
    }
}
