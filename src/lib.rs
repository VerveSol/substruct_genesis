use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::format_ident;
use syn::{parse_macro_input, Data, DeriveInput};

mod generator;
mod processor;

use generator::generate_complete_output;
use processor::attributes::{extract_struct_name, extract_trait_idents};
use processor::fields::{get_redis_updatable_kind, process_field, FieldContext};

/// Generates a substruct builder for partial updates with comprehensive utility methods.
///
/// This macro creates an independent substruct that contains only the fields you explicitly
/// mark for updates, providing a clean separation of concerns for building update operations.
///
/// # Features
///
/// - **Independent Substructs**: Generated substructs are completely independent of the original struct
/// - **Selective Field Inclusion**: Only fields with `#[substruct_field]` attributes are included
/// - **Multiple Field Types**: Supports primitive, JSON, and nested field types
/// - **Utility Methods**: Built-in methods for field counting, clearing, applying updates, and more
/// - **Custom Naming**: Configure the generated substruct name at the struct level
///
/// # Field Types
///
/// ## Primitive Fields
/// ```rust,ignore
/// #[derive(SubstructBuilder)]
/// struct User {
///     #[substruct_field(primitive)]                    // Wrapped in Option<T> (default)
///     name: String,                                    // -> Option<String>
///     
///     #[substruct_field(primitive, option = false)]    // Not wrapped in Option
///     id: u32,                                         // -> u32
/// }
/// ```
///
/// ## JSON Fields
/// ```rust,ignore
/// #[derive(SubstructBuilder)]
/// struct Settings {
///     #[substruct_field(json)]
///     preferences: UserPreferences,                    // -> Option<serde_json::Value>
/// }
/// ```
///
/// ## Nested Fields
/// ```rust,ignore
/// #[derive(SubstructBuilder)]
/// struct Profile {
///     #[substruct_field(nested)]
///     address: Address,                                // -> Option<AddressSubstruct>
/// }
/// ```
///
/// # Generated Methods
///
/// The macro generates the following methods for your substruct:
///
/// - `new(...)` - Constructor that takes all updatable fields as parameters
/// - `from_source(source: &T) -> Self` - Creates a substruct from an existing instance
/// - `is_empty(&self) -> bool` - Returns true if no fields would be changed
/// - `field_count(&self) -> usize` - Returns the number of fields with values set
/// - `clear(&mut self)` - Resets all fields to their default values
/// - `apply_to(&self, target: &mut T)` - Applies updates to a target struct
/// - `would_change(&self, target: &T) -> bool` - Checks if updates would modify target
/// - `merge(self, other: Self) -> Self` - Combines two substructs
/// - `has_field(&self, field_name: &str) -> bool` - Checks if a specific field is set
/// - `into_partial(self) -> HashMap<String, String>` - Converts to flexible HashMap representation
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,ignore
/// use substruct_genesis::SubstructBuilder;
///
/// #[derive(SubstructBuilder)]
/// struct User {
///     #[substruct_field(primitive)]
///     name: String,
///     #[substruct_field(primitive)]
///     active: bool,
///     age: u32,  // This field is excluded from the substruct
/// }
///
/// // Creates UserSubstruct with only name and active fields
/// let update = UserSubstruct::new(
///     Some("John".to_string()),
///     Some(true)
/// );
/// ```
///
/// ## Custom Naming
/// ```rust,ignore
/// #[derive(SubstructBuilder)]
/// #[substruct_builder(name = "UserBuilder")]
/// struct User {
///     #[substruct_field(primitive)]
///     name: String,
/// }
///
/// // Generates UserBuilder instead of UserSubstruct
/// ```
///
/// ## Nested Types
/// ```rust,ignore
/// #[derive(SubstructBuilder)]
/// struct Address {
///     #[substruct_field(primitive)]
///     street: String,
///     #[substruct_field(primitive)]
///     city: String,
/// }
///
/// #[derive(SubstructBuilder)]
/// struct Person {
///     #[substruct_field(primitive)]
///     name: String,
///     #[substruct_field(nested)]
///     address: Address,
/// }
///
/// let address_update = AddressSubstruct::new(
///     Some("123 Main St".to_string()),
///     Some("New York".to_string())
/// );
/// let person_update = PersonSubstruct::new(
///     Some("Alice".to_string()),
///     Some(address_update)
/// );
/// ```
///
/// # Attributes
///
/// ## Field Attributes
/// - `#[substruct_field(primitive)]` - Include as primitive field (wrapped in Option by default)
/// - `#[substruct_field(primitive, option = false)]` - Include as primitive field without Option wrapping
/// - `#[substruct_field(json)]` - Include as JSON field (Option<serde_json::Value>)
/// - `#[substruct_field(nested)]` - Include as nested substruct (Option<TypeSubstruct>)
/// - `#[substruct_field(nested, nested_type = "CustomName")]` - Use custom name for nested type
///
/// ## Struct Attributes
/// - `#[substruct_builder(name = "CustomName")]` - Set custom name for the generated substruct
///
/// # Requirements
///
/// - **Rust 1.56.0+** (Rust Edition 2021)
/// - `serde` for serialization support
/// - Fields must implement `Clone` and `PartialEq`
/// - At least one field must be tagged with `#[substruct_field]`
#[proc_macro_error]
#[proc_macro_derive(SubstructBuilder, attributes(substruct_field, substruct_builder))]
pub fn derive_updatable_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract the custom name from the struct-level attribute, or use default
    let update_struct_name = extract_struct_name(&input.attrs)
        .unwrap_or_else(|| format_ident!("{}Substruct", struct_name));

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => abort!(
            input.ident,
            "SubstructBuilder can only be derived for structs"
        ),
    };

    let mut trait_idents = extract_trait_idents(&input.attrs);

    let mut context = FieldContext::new();

    for field in fields.iter() {
        let field_kind = get_redis_updatable_kind(&field.attrs);
        process_field(field, &field_kind, &mut context);
    }

    // Generate the complete output using the generator module
    let output = generate_complete_output(
        struct_name,
        &update_struct_name,
        &mut trait_idents,
        &context,
    );

    TokenStream::from(output)
}
