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
            /// Creates a new substruct with the specified field values.
            ///
            /// # Arguments
            ///
            /// * `#(#field_names: #field_types)` - The values for each updatable field
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let update = #update_struct_name::new(
            ///     Some("John".to_string()),  // name field
            ///     Some(true),               // active field
            /// );
            /// ```
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }

            /// Creates a substruct from an existing instance where all fields indicate "no change".
            ///
            /// This is useful when you want to create a substruct that represents the current state
            /// but doesn't actually change anything when applied.
            ///
            /// # Arguments
            ///
            /// * `source` - The source struct to create the substruct from
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let user = User::new("Alice".to_string(), false, 25);
            /// let no_change_update = #update_struct_name::from_source(&user);
            /// assert!(no_change_update.is_empty());
            /// ```
            pub fn from_source(source: &#struct_name) -> Self {
                Self::from(source)
            }

            /// Returns `true` if no fields would be changed by this update.
            ///
            /// This method checks if all fields are in their "no change" state:
            /// - Wrapped fields are `None`
            /// - Unwrapped fields are `Default::default()`
            /// - JSON fields are `None`
            /// - Nested fields are `None`
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let empty_update = #update_struct_name::default();
            /// assert!(empty_update.is_empty());
            ///
            /// let update = #update_struct_name::new(Some("John".to_string()), None);
            /// assert!(!update.is_empty());
            /// ```
            pub fn is_empty(&self) -> bool {
                #(if let Some(_) = &self.#wrapped_field_names { return false; })*
                #(if self.#unwrapped_field_names != Default::default() { return false; })*
                #(if let Some(_) = &self.#json_field_names { return false; })*
                #(if let Some(_) = &self.#nested_field_names { return false; })*
                true
            }

            /// Returns the number of fields that have values set (non-default fields).
            ///
            /// This method counts fields that would actually change something when applied:
            /// - Wrapped fields that are `Some(value)`
            /// - Unwrapped fields that are not `Default::default()`
            /// - JSON fields that are `Some(value)`
            /// - Nested fields that are `Some(value)`
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let update = #update_struct_name::new(
            ///     Some("John".to_string()),  // name field is set
            ///     None,                     // active field is not set
            /// );
            /// assert_eq!(update.field_count(), 1);
            ///
            /// let full_update = #update_struct_name::new(
            ///     Some("Alice".to_string()),
            ///     Some(true),
            /// );
            /// assert_eq!(full_update.field_count(), 2);
            /// ```
            pub fn field_count(&self) -> usize {
                let mut count = 0;
                #(if let Some(_) = &self.#wrapped_field_names { count += 1; })*
                #(if self.#unwrapped_field_names != Default::default() { count += 1; })*
                #(if let Some(_) = &self.#json_field_names { count += 1; })*
                #(if let Some(_) = &self.#nested_field_names { count += 1; })*
                count
            }

            /// Resets all fields to their default values (no change state).
            ///
            /// This method sets all fields to their "no change" state:
            /// - Wrapped fields become `None`
            /// - Unwrapped fields become `Default::default()`
            /// - JSON fields become `None`
            /// - Nested fields become `None`
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut update = #update_struct_name::new(
            ///     Some("John".to_string()),
            ///     Some(true),
            /// );
            /// assert_eq!(update.field_count(), 2);
            ///
            /// update.clear();
            /// assert_eq!(update.field_count(), 0);
            /// assert!(update.is_empty());
            /// ```
            pub fn clear(&mut self) {
                #(self.#wrapped_field_names = None;)*
                #(self.#unwrapped_field_names = Default::default();)*
                #(self.#json_field_names = None;)*
                #(self.#nested_field_names = None;)*
            }

            /// Applies the updates to a target struct instance.
            ///
            /// This method modifies the target struct by applying all non-default field values
            /// from this substruct. Fields that are in their "no change" state are ignored.
            ///
            /// # Arguments
            ///
            /// * `target` - The mutable reference to the target struct to update
            ///
            /// # Behavior
            ///
            /// - **Wrapped fields**: Only applied if `Some(value)`
            /// - **Unwrapped fields**: Only applied if not `Default::default()`
            /// - **JSON fields**: Deserialized and applied if `Some(value)`
            /// - **Nested fields**: Recursively applied using their own `apply_to` method
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut user = User::new("Alice".to_string(), false, 25);
            /// let update = #update_struct_name::new(
            ///     Some("Bob".to_string()),  // Will change name
            ///     None,                    // Will not change active
            /// );
            ///
            /// update.apply_to(&mut user);
            /// // user.name is now "Bob", user.active remains false
            /// ```
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

            /// Checks if applying this update would modify the target struct.
            ///
            /// This method compares the values in this substruct with the corresponding fields
            /// in the target struct to determine if any changes would occur.
            ///
            /// # Arguments
            ///
            /// * `target` - The target struct to compare against
            ///
            /// # Returns
            ///
            /// * `true` if applying this update would change the target struct
            /// * `false` if no changes would occur
            ///
            /// # Behavior
            ///
            /// - **Wrapped fields**: Compared if `Some(value)` and different from target
            /// - **Unwrapped fields**: Compared if not `Default::default()` and different from target
            /// - **JSON fields**: Serialized and compared if `Some(value)` and different from target
            /// - **Nested fields**: Recursively checked using their own `would_change` method
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let user = User::new("Alice".to_string(), false, 25);
            /// let update = #update_struct_name::new(
            ///     Some("Bob".to_string()),  // Would change name
            ///     Some(false),             // Would not change active (same value)
            /// );
            ///
            /// assert!(update.would_change(&user));  // Would change name
            ///
            /// let no_change = #update_struct_name::new(
            ///     Some("Alice".to_string()),  // Same as current name
            ///     Some(false),               // Same as current active
            /// );
            /// assert!(!no_change.would_change(&user));  // No changes
            /// ```
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
                    let original: serde_json::Value = serde_json::to_value(&target.#json_field_names).expect("Failed to serialize to JSON");
                    if !serde_json::Value::eq(value, &original) {
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

            /// Combines two substructs, with the `other` substruct taking precedence for conflicting fields.
            ///
            /// This method merges the field values from two substructs, with the `other` substruct
            /// taking precedence when both substructs have values for the same field.
            ///
            /// # Arguments
            ///
            /// * `other` - The other substruct to merge with (takes precedence for conflicts)
            ///
            /// # Returns
            ///
            /// A new substruct containing the merged field values.
            ///
            /// # Behavior
            ///
            /// - **Wrapped fields**: `other` value takes precedence if `Some`, otherwise uses `self`
            /// - **Unwrapped fields**: `other` value takes precedence if not `Default::default()`, otherwise uses `self`
            /// - **JSON fields**: `other` value takes precedence if `Some`, otherwise uses `self`
            /// - **Nested fields**: `other` value takes precedence if `Some`, otherwise uses `self`
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let update1 = #update_struct_name::new(
            ///     Some("Alice".to_string()),  // name field
            ///     None,                      // active field not set
            /// );
            ///
            /// let update2 = #update_struct_name::new(
            ///     None,                      // name field not set
            ///     Some(true),               // active field
            /// );
            ///
            /// let merged = update1.merge(update2);
            /// // merged has name: Some("Alice") and active: Some(true)
            /// ```
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

            /// Checks if a specific field has a value set (non-default value).
            ///
            /// This method determines whether a field would actually change something when applied.
            ///
            /// # Arguments
            ///
            /// * `field_name` - The name of the field to check (as a string)
            ///
            /// # Returns
            ///
            /// * `true` if the field has a value set and would change something
            /// * `false` if the field is in its "no change" state or doesn't exist
            ///
            /// # Behavior
            ///
            /// - **Wrapped fields**: Returns `true` if `Some(value)`
            /// - **Unwrapped fields**: Returns `true` if not `Default::default()`
            /// - **JSON fields**: Returns `true` if `Some(value)`
            /// - **Nested fields**: Returns `true` if `Some(value)`
            /// - **Non-existent fields**: Returns `false`
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let update = #update_struct_name::new(
            ///     Some("Alice".to_string()),  // name field is set
            ///     None,                      // active field is not set
            /// );
            ///
            /// assert!(update.has_field("name"));    // name field has a value
            /// assert!(!update.has_field("active")); // active field is not set
            /// assert!(!update.has_field("age"));    // age field doesn't exist in substruct
            /// ```
            pub fn has_field(&self, field_name: &str) -> bool {
                match field_name {
                    #(stringify!(#wrapped_field_names) => self.#wrapped_field_names.is_some(),)*
                    #(stringify!(#unwrapped_field_names) => self.#unwrapped_field_names != Default::default(),)*
                    #(stringify!(#json_field_names) => self.#json_field_names.is_some(),)*
                    #(stringify!(#nested_field_names) => self.#nested_field_names.is_some(),)*
                    _ => false,
                }
            }

            /// Converts the substruct into a flexible HashMap representation with string values.
            ///
            /// This method creates a HashMap where keys are field names and values are string
            /// representations of the field values. This is useful for dynamic field access,
            /// serialization, or when you need to work with field values in a generic way.
            ///
            /// # Returns
            ///
            /// A `HashMap<String, String>` containing only fields that have values set.
            ///
            /// # Behavior
            ///
            /// - **Wrapped fields**: Included if `Some(value)`, formatted using `{:?}`
            /// - **Unwrapped fields**: Included if not `Default::default()`, formatted using `{:?}`
            /// - **JSON fields**: Included if `Some(value)`, converted using `.to_string()`
            /// - **Nested fields**: Recursively converted using their own `into_partial()` method
            /// - **Fields without values**: Not included in the result
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let update = #update_struct_name::new(
            ///     Some("Alice".to_string()),  // name field
            ///     Some(true),                // active field
            /// );
            ///
            /// let partial = update.into_partial();
            ///
            /// // Check that fields are present
            /// assert!(partial.contains_key("name"));
            /// assert!(partial.contains_key("active"));
            ///
            /// // Compare actual values (as string representations)
            /// assert_eq!(partial.get("name"), Some(&"\"Alice\"".to_string()));
            /// assert_eq!(partial.get("active"), Some(&"true".to_string()));
            ///
            /// // Fields that aren't set are not included
            /// assert!(!partial.contains_key("age")); // age field doesn't exist in substruct
            /// ```
            pub fn into_partial(self) -> std::collections::HashMap<String, String> {
                let mut partial = std::collections::HashMap::new();

                // Add wrapped fields
                #(if let Some(value) = self.#wrapped_field_names {
                    partial.insert(stringify!(#wrapped_field_names).to_string(), format!("{:?}", value));
                })*

                // Add unwrapped fields (only if not default)
                #(if self.#unwrapped_field_names != Default::default() {
                    partial.insert(stringify!(#unwrapped_field_names).to_string(), format!("{:?}", &self.#unwrapped_field_names));
                })*

                // Add JSON fields
                #(if let Some(value) = self.#json_field_names {
                    partial.insert(stringify!(#json_field_names).to_string(), value.to_string());
                })*

                // Add nested fields (convert to partial recursively)
                #(if let Some(nested) = self.#nested_field_names {
                    partial.insert(stringify!(#nested_field_names).to_string(), format!("{:?}", nested.into_partial()));
                })*

                partial
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
