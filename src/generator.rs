use crate::processor::fields::FieldContext;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Ident;

/// Configuration for trait derivation
#[derive(Debug, Clone)]
pub struct TraitConfig {
    pub required_traits: Vec<&'static str>,
    pub contains_f64: bool,
    pub contains_string: bool,
}

impl Default for TraitConfig {
    fn default() -> Self {
        Self {
            required_traits: vec!["Clone", "Debug", "Serialize", "Deserialize"],
            contains_f64: false,
            contains_string: false,
        }
    }
}

/// Generate the derive clause for the struct
pub fn generate_derive_clause(
    trait_idents: &mut Vec<Ident>,
    config: &TraitConfig,
) -> proc_macro2::TokenStream {
    // Ensure required traits are present
    for required in &config.required_traits {
        if !trait_idents.iter().any(|ident| ident == required) {
            trait_idents.push(format_ident!("{}", required));
        }
    }

    // Remove incompatible traits based on field types
    if config.contains_f64 {
        trait_idents.retain(|ident| ident != "Eq");
    }

    if config.contains_string {
        trait_idents.retain(|ident| ident != "PartialEq");
        trait_idents.retain(|ident| ident != "Eq");
    }

    quote! {
        #[derive(#(#trait_idents),*)]
    }
}

/// Generate the From<&T> implementation
pub fn generate_from_ref_impl(
    struct_name: &Ident,
    update_struct_name: &Ident,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    let has_json_fields = !context.json_field_names.is_empty();
    let handled_fields: HashSet<_> = context.json_field_names.iter().collect();
    let defaulted_fields: Vec<_> = context
        .field_names
        .iter()
        .filter(|ident| !handled_fields.contains(ident))
        .collect();

    if has_json_fields {
        let json_field_names = &context.json_field_names;
        quote! {
            impl From<&#struct_name> for #update_struct_name {
                fn from(source: &#struct_name) -> Self {
                    Self {
                        #(#defaulted_fields: Default::default(),)*
                        #(#json_field_names: Some(serde_json::to_value(&source.#json_field_names)
                            .expect("Failed to serialize field to JSON")),)*
                    }
                }
            }
        }
    } else {
        quote! {
            impl From<&#struct_name> for #update_struct_name {
                fn from(source: &#struct_name) -> Self {
                    Self {
                        #(#defaulted_fields: Default::default(),)*
                    }
                }
            }
        }
    }
}

/// Generate the main struct definition
pub fn generate_struct_definition(
    update_struct_name: &Ident,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    let updatable_fields = &context.updatable_fields;
    quote! {
        pub struct #update_struct_name {
            #(#updatable_fields,)*
        }
    }
}

/// Generate the impl block for the struct
pub fn generate_struct_impl(
    update_struct_name: &Ident,
    struct_name: &Ident,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    let field_names = &context.field_names;
    let field_types = &context.field_types;
    let wrapped_field_names = &context.wrapped_field_names;
    let unwrapped_field_names = &context.unwrapped_field_names;
    let json_field_names = &context.json_field_names;
    let nested_field_names = &context.nested_field_names;

    quote! {
        impl #update_struct_name {
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names,)*
                }
            }

            pub fn from_source(source: &#struct_name) -> Self {
                Self::from(source)
            }

            pub fn is_empty(&self) -> bool {
                #(if let Some(_) = &self.#wrapped_field_names { return false; })*
                #(if self.#unwrapped_field_names != Default::default() { return false; })*
                true
            }

            pub fn field_count(&self) -> usize {
                let mut count = 0;
                #(if let Some(_) = &self.#wrapped_field_names { count += 1; })*
                #(if self.#unwrapped_field_names != Default::default() { count += 1; })*
                #(if let Some(_) = &self.#json_field_names { count += 1; })*
                #(if let Some(_) = &self.#nested_field_names { count += 1; })*
                count
            }

            pub fn clear(&mut self) {
                #(self.#wrapped_field_names = None;)*
                #(self.#unwrapped_field_names = Default::default();)*
                #(self.#json_field_names = None;)*
                #(self.#nested_field_names = None;)*
            }

            pub fn apply_to(&self, target: &mut #struct_name) {
                // Apply primitive and JSON fields
                #(if let Some(value) = &self.#wrapped_field_names {
                    target.#wrapped_field_names = value.clone();
                })*
                #(if self.#unwrapped_field_names != Default::default() {
                    target.#unwrapped_field_names = self.#unwrapped_field_names.clone();
                })*
                #(if let Some(value) = &self.#json_field_names {
                    target.#json_field_names = serde_json::from_value(value.clone()).expect("Failed to deserialize JSON");
                })*

                // Apply nested fields recursively
                #(if let Some(nested_update) = &self.#nested_field_names {
                    nested_update.apply_to(&mut target.#nested_field_names);
                })*
            }

            pub fn would_change(&self, target: &#struct_name) -> bool {
                // Check primitive and JSON fields
                #(if let Some(value) = &self.#wrapped_field_names {
                    if value != &target.#wrapped_field_names {
                        return true;
                    }
                })*
                #(if self.#unwrapped_field_names != Default::default() && self.#unwrapped_field_names != target.#unwrapped_field_names {
                    return true;
                })*
                #(if let Some(value) = &self.#json_field_names {
                    let original = serde_json::to_value(&target.#json_field_names).expect("Failed to serialize to JSON");
                    if *value != original {
                        return true;
                    }
                })*

                // Check nested fields recursively
                #(if let Some(nested_update) = &self.#nested_field_names {
                    if nested_update.would_change(&target.#nested_field_names) {
                        return true;
                    }
                })*

                false
            }

            pub fn merge(self, other: Self) -> Self {
                Self {
                    #(#wrapped_field_names: other.#wrapped_field_names.or(self.#wrapped_field_names),)*
                    #(#unwrapped_field_names: if other.#unwrapped_field_names != Default::default() {
                        other.#unwrapped_field_names
                    } else {
                        self.#unwrapped_field_names
                    },)*
                    #(#json_field_names: other.#json_field_names.or(self.#json_field_names),)*
                    #(#nested_field_names: other.#nested_field_names.or(self.#nested_field_names),)*
                }
            }

            pub fn has_field(&self, field_name: &str) -> bool {
                match field_name {
                    #(stringify!(#wrapped_field_names) => self.#wrapped_field_names.is_some(),)*
                    #(stringify!(#unwrapped_field_names) => self.#unwrapped_field_names != Default::default(),)*
                    #(stringify!(#json_field_names) => self.#json_field_names.is_some(),)*
                    #(stringify!(#nested_field_names) => self.#nested_field_names.is_some(),)*
                    _ => false,
                }
            }
        }
    }
}

/// Generate the Default implementation
pub fn generate_default_impl(
    update_struct_name: &Ident,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    let field_names = &context.field_names;
    quote! {
        impl Default for #update_struct_name {
            fn default() -> Self {
                Self {
                    #(#field_names: Default::default(),)*
                }
            }
        }
    }
}

/// Generate the From<T> implementation
pub fn generate_from_impl(
    update_struct_name: &Ident,
    struct_name: &Ident,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    let field_names = &context.field_names;
    quote! {
        impl From<#struct_name> for #update_struct_name {
            fn from(source: #struct_name) -> Self {
                Self {
                    #(#field_names: Default::default(),)*
                }
            }
        }
    }
}

/// Validate the context and ensure it's ready for code generation
pub fn validate_context(context: &FieldContext, struct_name: &Ident) {
    if context.updatable_fields.is_empty() {
        abort!(
            struct_name,
            "No fields are tagged with #[substruct_field]. At least one field must be tagged to generate a substruct."
        );
    }
}

/// Generate the complete output for the macro
pub fn generate_complete_output(
    struct_name: &Ident,
    update_struct_name: &Ident,
    trait_idents: &mut Vec<Ident>,
    context: &FieldContext,
) -> proc_macro2::TokenStream {
    // Validate context before generation
    validate_context(context, struct_name);

    // Create trait configuration
    let trait_config = TraitConfig {
        contains_f64: context.contains_f64,
        contains_string: context.contains_string,
        ..Default::default()
    };

    // Generate all components
    let derive_clause = generate_derive_clause(trait_idents, &trait_config);
    let struct_def = generate_struct_definition(update_struct_name, context);
    let struct_impl = generate_struct_impl(update_struct_name, struct_name, context);
    let default_impl = generate_default_impl(update_struct_name, context);
    let from_impl = generate_from_impl(update_struct_name, struct_name, context);
    let from_ref_impl = generate_from_ref_impl(struct_name, update_struct_name, context);

    quote! {
        #derive_clause
        #struct_def

        #struct_impl

        #default_impl

        #from_impl

        #from_ref_impl
    }
}
